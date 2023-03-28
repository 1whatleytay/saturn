use std::collections::HashMap;

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::api::http::{Client, ClientBuilder, HttpRequestBuilder, StatusCode};
use tauri::api::path::app_local_data_dir;
use tauri::{AppHandle, Manager, PathResolver, Wry};

#[derive(Clone, Serialize)]
struct ConsoleEvent {
    uuid: Option<String>,
    message: String,
}

#[derive(Deserialize)]
pub struct MidiDefaultProvider {
    providers: Vec<String>,
    hashes: Option<HashMap<String, String>>,
}

pub enum MidiProviderContainer {
    None,
    Empty,
    Provider(Arc<MidiDefaultProvider>),
}

async fn load_default_provider(resolver: PathResolver) -> Option<MidiDefaultProvider> {
    let resource = resolver.resolve_resource("resources/midi-default.json")?;

    let buffer = tokio::fs::read_to_string(resource).await.ok()?;
    let data = serde_json::from_str(&buffer).ok()?;

    Some(data)
}

async fn grab_default_provider(app: &AppHandle<Wry>) -> Option<Arc<MidiDefaultProvider>> {
    let state: tauri::State<Mutex<MidiProviderContainer>> = app.state();

    {
        let container = state.lock().ok()?;

        match &*container {
            MidiProviderContainer::None => { /* fallback */ }
            MidiProviderContainer::Empty => return None,
            MidiProviderContainer::Provider(arc) => return Some(arc.clone()),
        }
    }

    let provider = load_default_provider(app.path_resolver())
        .await
        .map(Arc::new);

    let mut container = state.lock().ok()?;

    match provider {
        Some(provider) => {
            *container = MidiProviderContainer::Provider(provider.clone());

            Some(provider)
        }
        None => {
            *container = MidiProviderContainer::Empty;

            None
        }
    }
}

async fn download_file(
    client: &Client,
    base_url: &str,
    file: String,
    sha256: Option<&str>,
) -> Option<(String, Vec<u8>)> {
    let path = format!("{}{}", base_url, file);
    let request = HttpRequestBuilder::new("GET", &path).ok()?;

    let response = client.send(request).await.ok()?;

    if response.status() != StatusCode::OK {
        return None;
    }

    let bytes = response.bytes().await.ok()?.data;

    if let Some(sha256) = sha256 {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();

        let Ok(expected) = hex::decode(sha256) else {
            return None
        };

        if expected != result[..] {
            eprintln!(
                "Failed to verify hashes, expected {} got {}",
                sha256,
                hex::encode(&result[..])
            );

            return None;
        } else {
        }
    }

    Some((file, bytes))
}

async fn download_files<F>(
    url: &str,
    files: &[String],
    handler: &F,
    sha256: &Option<HashMap<String, String>>,
) -> Option<HashMap<String, Vec<u8>>>
where
    F: Fn(usize),
{
    let count = AtomicUsize::new(0);

    let client = ClientBuilder::new().build().ok()?;
    let result = futures::future::try_join_all(files.iter().map(|file| async {
        let hash = if let Some(map) = sha256 {
            Some(map.get(file).map(|x| x.as_str()).ok_or(())?)
        } else {
            None
        };

        let result = download_file(&client, url, file.clone(), hash).await;

        if result.is_some() {
            count.fetch_add(1, Ordering::Acquire);
            handler(count.load(Ordering::Acquire));
        }

        result.ok_or(())
    }))
    .await
    .ok()?;

    Some(result.into_iter().collect())
}

async fn request_providers<F: Fn(usize)>(
    providers: &Vec<String>,
    files: &[String],
    handler: F,
    sha256: &Option<HashMap<String, String>>,
) -> Option<HashMap<String, Vec<u8>>> {
    for url in providers {
        // Storing all file content in memory can be really bad (sound-fonts can be huge).
        let result = download_files(url, files, &handler, sha256).await;

        if let Some(files) = result {
            return Some(files);
        }
    }

    None
}

pub async fn install_instruments(
    provider_url: Option<String>,
    instruments: Option<Vec<String>>,
    app: &AppHandle<Wry>,
) -> Option<()> {
    let mut provider = provider_url.map(|url| {
        Arc::new(MidiDefaultProvider {
            providers: vec![url],
            hashes: None,
        })
    });

    if provider.is_none() {
        provider = grab_default_provider(app).await;
    }

    let provider = provider?;

    let postfix = "-mp3.js";

    let files = instruments
        .map(|instruments| {
            instruments
                .into_iter()
                .map(|item| format!("{}{}", item, postfix))
                .collect::<Vec<String>>()
        })
        .or_else(|| {
            provider
                .hashes
                .as_ref()
                .map(|files| files.keys().cloned().collect::<Vec<String>>())
        })?;

    let download_uuid = uuid::Uuid::new_v4().to_string();

    let handler = |count: usize| {
        app.emit_all(
            "post-console-event",
            ConsoleEvent {
                uuid: Some(download_uuid.clone()),
                message: format!(
                    "Downloading MIDI instruments: {}",
                    if count == files.len() {
                        "DONE".into()
                    } else {
                        format!("{}/{}", count, files.len())
                    }
                ),
            },
        )
        .ok();
    };

    handler(0);

    let data = request_providers(&provider.providers, &files, handler, &provider.hashes).await?;

    let mut directory = app_local_data_dir(&app.config())?;
    directory.push("midi/");
    fs::create_dir_all(&directory).ok()?;

    let write_uuid = uuid::Uuid::new_v4().to_string();

    app.emit_all(
        "post-console-event",
        ConsoleEvent {
            uuid: Some(write_uuid.clone()),
            message: "Writing MIDI files...".into(),
        },
    )
    .ok();

    futures::future::join_all(data.into_iter().map(|(file, binary)| {
        let mut path = directory.clone();
        path.push(&file);

        tokio::fs::write(path, binary)
    }))
    .await;

    app.emit_all(
        "post-console-event",
        ConsoleEvent {
            uuid: Some(write_uuid),
            message: "Finished downloading required MIDI instruments.".into(),
        },
    )
    .ok();

    Some(())
}

#[tauri::command]
pub async fn midi_install(
    provider_url: Option<String>,
    instruments: Option<Vec<String>>,
    app: AppHandle<Wry>,
) -> bool {
    let console = install_instruments(provider_url, instruments, &app).await;

    if console.is_none() {
        app.emit_all(
            "post-console-event",
            ConsoleEvent {
                uuid: None,
                message: "Failed to download MIDI instruments.".into(),
            },
        )
        .ok();
    }

    console.is_some()
}
