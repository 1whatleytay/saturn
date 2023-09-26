/*
use std::path::PathBuf;
use std::sync::Mutex;
use notify::RecommendedWatcher;
use serde::{Deserialize, Serialize};

const AUTO_SAVE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
pub struct AutoSaveDetails {
    version: u32,
    files: Vec<PathBuf>
}

pub struct AutoSaveState {
    details: AutoSaveDetails,
    watcher: Option<RecommendedWatcher>
}

impl AutoSaveState {
    pub fn new(app: tauri::AppHandle) -> AutoSaveState {
        let watcher = notify::recommended_watcher(|event| {
            let Ok(event) = event else { return };


        }).ok();

        AutoSaveState {
            details: AutoSaveDetails {
                version: AUTO_SAVE_VERSION,
                files: vec![],
            },
            watcher
        }
    }

    pub fn read_from_disc(app: tauri::AppHandle) -> AutoSaveState {
        let path = tauri::api::path::app_data_dir(&app.config());

        Self::new(app)
    }
}

#[derive(Serialize)]
pub struct AutoSaveFile {
    path: String,
    name: String
}

#[tauri::command]
pub fn register_files(files: Vec<String>, state: tauri::State<Mutex<AutoSaveState>>, app_handle: tauri::AppHandle) {
    let mut pointer = state.lock().unwrap();

}

#[tauri::command]
pub fn get_files(state: tauri::State<Mutex<AutoSaveState>>) -> Vec<AutoSaveFile> {
    let mut pointer = state.lock().unwrap();

    vec![]
}
*/
