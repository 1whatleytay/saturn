/*
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};

const AUTO_SAVE_VERSION: u32 = 1;
const AUTO_SAVE_FILENAME: &str = "save-state.json";

#[derive(Serialize, Deserialize)]
pub struct AutoSaveDetails {
    version: u32,
    files: HashSet<PathBuf>
}

pub struct AutoSaveState {
    details: AutoSaveDetails,
    watcher: Option<RecommendedWatcher>
}

impl AutoSaveState {
    fn watcher(app: tauri::AppHandle) -> Option<RecommendedWatcher> {
        notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            let Ok(event) = event else { return };

            match event.kind {
                notify::EventKind::Create(kind) => { },
                notify::EventKind::Remove(kind) => { },
                notify::EventKind::Modify(kind) => { },
                _ => {}
            }
        }).ok()
    }

    pub fn write_to(file: PathBuf) {

    }

    pub fn merge(&mut self, files: HashSet<PathBuf>) {
        let removed = &self.details.files - &files;
        let added = &files - &self.details.files;

        if let Some(watcher) = &mut self.watcher {
            for file in removed {
                watcher.unwatch(&file).ok();
            }

            for file in added {
                watcher.watch(&file, RecursiveMode::NonRecursive).ok();
            }
        }

        self.details.files = files;
    }

    pub fn new(app: tauri::AppHandle) -> AutoSaveState {
        AutoSaveState {
            details: AutoSaveDetails {
                version: AUTO_SAVE_VERSION,
                files: HashSet::new()
            },
            watcher: Self::watcher(app)
        }
    }

    pub fn read_from_disc(app: tauri::AppHandle) -> Option<AutoSaveState> {
        let mut path = tauri::api::path::app_data_dir(&app.config())?;

        path.push(AUTO_SAVE_FILENAME);

        let details: AutoSaveDetails = serde_json::from_reader(File::open(path).ok()?).ok()?;

        Some(AutoSaveState {
            details,
            watcher: Self::watcher(app)
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
*/
