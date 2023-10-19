/*
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tauri::Manager;

const AUTO_SAVE_VERSION: u32 = 1;
const AUTO_SAVE_FILENAME: &str = "save-state.json";

#[derive(Serialize, Deserialize)]
pub struct AutoSaveDetails {
    version: u32,
    files: HashSet<PathBuf>,
    dismiss: HashSet<PathBuf>
}

pub struct AutoSaveState {
    details: Arc<Mutex<AutoSaveDetails>>,
    watcher: Option<RecommendedWatcher>
}

// Print critical to debug errors to the console.
fn log_error<T, E: Error>(result: &Result<T, E>) {
    if let Err(error) = result {
        eprintln!("{error}");
    }
}

impl Default for AutoSaveDetails {
    fn default() -> Self {
        AutoSaveDetails {
            version: AUTO_SAVE_VERSION,
            files: HashSet::new(),
            dismiss: HashSet::new()
        }
    }
}

impl AutoSaveState {
    fn watcher(app: tauri::AppHandle, details: Arc<Mutex<AutoSaveDetails>>) -> Option<RecommendedWatcher> {
        notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            let Ok(event) = event else { return };

            let Some(path) = event.paths.first() else {
                return
            };

            let mut details = details.lock().unwrap();

            // If the path is in dismiss, we probably caused the save.
            if details.dismiss.remove(path) {
                return
            }

            let Some(path) = event.paths.first().and_then(|x| x.to_str()) else {
                return
            };

            match event.kind {
                notify::EventKind::Create(_) => {
                    app.emit_all("save:create", path).ok();
                },
                notify::EventKind::Remove(_) => {
                    app.emit_all("save:remove", path).ok();
                },
                notify::EventKind::Modify(_) => {
                    app.emit_all("save:modify", path).ok();
                },
                _ => {}
            }
        }).ok()
    }

    pub fn write_to(&mut self, text: String, path: PathBuf) {
        let mut details = self.details.lock().unwrap();

        if !details.files.contains(&path) {
            let path = path.to_string_lossy();
            eprintln!("Write to {path} rejected as it was not present in file set.");

            return
        }

        log_error(&fs::write(&path, text));

        details.dismiss.insert(path);
    }

    pub fn merge(&mut self, files: HashSet<PathBuf>) {
        let mut details = self.details.lock().unwrap();

        let removed = &details.files - &files;
        let added = &files - &details.files;

        if let Some(watcher) = &mut self.watcher {
            for file in removed {
                log_error(&watcher.unwatch(&file))
            }

            for file in added {
                log_error(&watcher.watch(&file, RecursiveMode::NonRecursive))
            }
        }

        details.files = files;
    }

    pub fn new(app: tauri::AppHandle) -> AutoSaveState {
        let details = Arc::new(Mutex::new(AutoSaveDetails::default()));
        let watcher = Self::watcher(app, details.clone());

        AutoSaveState {
            details,
            watcher
        }
    }

    pub fn read_from_disc(app: tauri::AppHandle) -> Option<AutoSaveState> {
        let mut path = tauri::api::path::app_data_dir(&app.config())?;

        path.push(AUTO_SAVE_FILENAME);

        let details: AutoSaveDetails = serde_json::from_reader(File::open(path).ok()?).ok()?;
        let details = Arc::new(Mutex::new(details));
        let watcher = Self::watcher(app, details.clone());

        Some(AutoSaveState {
            details,
            watcher
        })
    }

    pub fn from_disc(app: tauri::AppHandle) -> AutoSaveState {
        Self::read_from_disc(app.clone()).unwrap_or_else(|| Self::new(app))
    }
}

#[derive(Serialize)]
pub struct AutoSaveFile {
    path: String,
    name: String
}

#[tauri::command]
pub fn save_sync(files: Vec<PathBuf>, state: tauri::State<AutoSaveState>) {

}

#[tauri::command]
pub fn save_store(uuid: String, content: String) {

}

#[tauri::command]
pub fn save_store_sync(uuids: Vec<String>) {

}

#[tauri::command]
pub fn save_sync_get(files: Vec<PathBuf>, state: tauri::State<AutoSaveState>) {

}

#[tauri::command]
pub fn save_write(file: PathBuf, content: String, state: tauri::State<AutoSaveState>) {

}
*/
