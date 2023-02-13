use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use futures::{StreamExt};
use futures::stream::FuturesUnordered;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PathResolver, Wry};
use tauri::api::http::{Client, ClientBuilder, HttpRequestBuilder};
use tauri::api::path::app_local_data_dir;
use tauri::http::{Request, Response, ResponseBuilder};
use tauri::http::method::Method;
use wry::http::Uri;

#[derive(Clone, Serialize)]
struct ConsoleEvent {
    uuid: Option<String>,
    message: String
}

#[derive(Deserialize)]
pub struct MidiDefaultProvider {
    providers: Vec<String>,
    hashes: Option<HashMap<String, String>>
}

pub enum MidiProviderContainer {
    None,
    Empty,
    Provider(Arc<MidiDefaultProvider>)
}

async fn load_default_provider(resolver: PathResolver) -> Option<MidiDefaultProvider> {
    let resource = resolver.resolve_resource("resources/midi-default.json")?;

    let buffer = tokio::fs::read_to_string(resource).await.ok()?;
    let data = serde_json::from_str(&buffer).ok()?;

    Some(data)
}

async fn grab_default_provider(
    app: &AppHandle<Wry>
) -> Option<Arc<MidiDefaultProvider>> {
    let state: tauri::State<Mutex<MidiProviderContainer>> = app.state();

    {
        let container = state.lock().ok()?;

        match &*container {
            MidiProviderContainer::None => { /* fallback */ },
            MidiProviderContainer::Empty => return None,
            MidiProviderContainer::Provider(arc) => return Some(arc.clone())
        }
    }

    let provider = load_default_provider(app.path_resolver()).await
        .map(|x| Arc::new(x));

    let mut container = state.lock().ok()?;

    match provider {
        Some(provider) => {
            *container = MidiProviderContainer::Provider(provider.clone());

            Some(provider)
        },
        None => {
            println!("Loaded default provider is totally empty!");
            *container = MidiProviderContainer::Empty;

            None
        }
    }
}

async fn download_file(
    client: &Client, base_url: &str, file: String
) -> tauri::api::Result<(String, Vec<u8>)> {
    let path = format!("{}{}", base_url, file);
    println!("Downloading file {}...", &path);
    let request = HttpRequestBuilder::new("GET", &path)?;

    let response = client.send(request).await?;
    let bytes = response.bytes().await?.data;

    println!("File {} gave {} bytes!", &path, bytes.len());

    Ok((file, bytes))
}

async fn download_files<F>(
    url: &str,
    files: &Vec<String>,
    handler: F
) -> Option<HashMap<String, Vec<u8>>> where F: Fn(usize) -> () {
    let count = AtomicUsize::new(0);

    let client = ClientBuilder::new().build().ok()?;
    let result = files.iter()
        .map(|file| {
            async {
                let result = download_file(&client, url, file.clone()).await;

                count.fetch_add(1, Ordering::Acquire);
                handler(count.load(Ordering::Acquire));

                result
            }
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|x| x.ok())
        .collect::<HashMap<String, Vec<u8>>>();

    Some(result)
}

async fn request_providers<F: Fn(usize) -> ()>(
    providers: &Vec<String>, files: &Vec<String>, handler: F
) -> Option<HashMap<String, Vec<u8>>> {
    for url in providers {
        // Storing all file content in memory can be really bad (sound-fonts can be huge).
        if let Some(files) = download_files(url, &files, |x| handler(x)).await {
            return Some(files)
        }
    }

    None
}

pub async fn install_instruments(
    provider_url: Option<String>,
    instruments: Option<Vec<String>>,
    app: AppHandle<Wry>
) -> Option<()> {
    let mut provider = provider_url
        .map(|url| Arc::new(MidiDefaultProvider { providers: vec![url], hashes: None }));

    if provider.is_none() {
        provider = grab_default_provider(&app).await;
    }

    let provider = provider?;

    let postfix = "-mp3.js";

    println!("Creating Instruments");

    let files = instruments
        .map(|instruments| instruments.into_iter()
            .map(|item| format!("{}{}", item, postfix))
            .collect::<Vec<String>>())
        .or_else(|| provider.hashes.as_ref()
            .map(|files| files.keys().cloned().collect::<Vec<String>>()))?;

    println!("Requesting Providers");

    let download_uuid = uuid::Uuid::new_v4().to_string();

    let handler = |count: usize| {
        app.emit_all("post-console-event", ConsoleEvent {
            uuid: Some(download_uuid.clone()),
            message: format!("Downloading MIDI instruments: {}", if count == files.len() {
                "DONE".into()
            } else {
                format!("{}/{}", count, files.len())
            })
        }).ok();
    };

    handler(0);

    let data = request_providers(&provider.providers, &files, handler).await?;

    let mut directory = app_local_data_dir(&app.config())?;
    directory.push("midi/");
    fs::create_dir_all(&directory).ok()?;

    let write_uuid = uuid::Uuid::new_v4().to_string();

    app.emit_all("post-console-event", ConsoleEvent {
        uuid: Some(write_uuid.clone()),
        message: "Writing MIDI files...".into()
    }).ok();

    futures::future::join_all(data.into_iter().map(|(file, binary)| {
        let mut path = directory.clone();
        path.push(&file);

        tokio::fs::write(path, binary)
    })).await;

    app.emit_all("post-console-event", ConsoleEvent {
        uuid: Some(write_uuid),
        message: "Finished downloading required MIDI instruments.".into()
    }).ok();

    Some(())
}


#[tauri::command]
pub async fn midi_install(
    provider_url: Option<String>,
    instruments: Option<Vec<String>>,
    app: AppHandle<Wry>
) -> bool {
    let console = install_instruments(provider_url, instruments, app).await;

    console.is_some()
}

pub fn midi_protocol(app: &AppHandle<Wry>, request: &Request) -> Result<Response, Box<dyn Error>> {
    // Disable CORS, nothing super private here.
    let builder = ResponseBuilder::new()
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Origin", "*");

    // Check for preflight, very primitive check.
    if request.method() == Method::OPTIONS {
        return builder.body(vec![])
    }

    let file_path = || -> Option<PathBuf> {
        let decode = percent_decode_str(request.uri()).decode_utf8().ok()?;
        let uri = Uri::try_from(&*decode).ok()?;
        let mut directory = app_local_data_dir(&app.config())?;
        let path = Path::new(uri.path()).strip_prefix("/").ok()?;

        directory.push(path);

        Some(directory)
    };

    let Some(path) = file_path() else {
        return builder.status(400).body(vec![]);
    };

    let Ok(text) = fs::read_to_string(path) else {
        return builder.status(400).body(vec![]);
    };

    builder.body(text.into_bytes())
}
