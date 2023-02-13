use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use futures::{StreamExt, TryFutureExt};
use futures::stream::FuturesUnordered;
use percent_encoding::percent_decode_str;
use serde::{Deserialize};
use tauri::{AppHandle, Manager, PathResolver, Wry};
use tauri::api::http::{Client, ClientBuilder, HttpRequestBuilder};
use tauri::api::path::app_local_data_dir;
use tauri::http::{Request, Response, ResponseBuilder};
use tauri::http::method::Method;
use wry::http::Uri;

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

async fn download_files(
    url: &str,
    files: &Vec<String>
) -> Option<HashMap<String, Vec<u8>>> {
    let client = ClientBuilder::new().build().ok()?;
    let result = files.iter()
        .map(|file| download_file(&client, url, file.clone()))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|x| x.ok())
        .collect::<HashMap<String, Vec<u8>>>();

    Some(result)
}

async fn request_providers(providers: &Vec<String>, files: &Vec<String>) -> Option<HashMap<String, Vec<u8>>> {
    for url in providers {
        // Storing all file content in memory can be really bad (sound-fonts can be huge).
        if let Some(files) = download_files(url, &files).await {
            return Some(files)
        }
    }

    None
}

#[tauri::command]
pub async fn midi_install(
    provider_url: Option<String>,
    instruments: Option<Vec<String>>,
    app: AppHandle<Wry>
) -> Option<String> {
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

    let data = request_providers(&provider.providers, &files).await?;

    println!("Making Directories...");

    let mut directory = app_local_data_dir(&app.config())?;
    directory.push("midi/");
    fs::create_dir_all(&directory).ok()?;

    println!("Writing Files...");

    let result = futures::future::join_all(data.into_iter().map(|(file, binary)| {
        let mut path = directory.clone();
        println!("Directory: {}, file: {}", directory.to_str().unwrap_or("None"), &file);
        path.push(&file);
        println!("Writing to {}...", path.to_str().unwrap_or("None"));

        tokio::fs::write(path, binary)
    })).await;

    for p in result {
        if let Err(error) = p {
            println!("Error: {}", error);
        }
    }

    Some("Success".into())
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
