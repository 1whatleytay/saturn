use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify::event::ModifyKind;
use serde::{Deserialize, Serialize};
use tauri::api::dialog::FileDialogBuilder;
use tauri::{AppHandle, Config, Manager};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use crate::access_manager::AccessFileData::{Binary, Text};
use crate::watch::Watch;

const ACCESS_CONFIG_VERSION: u32 = 1;
const ACCESS_CONFIG_FILENAME: &str = "save-state.json";

#[derive(Serialize)]
#[serde(untagged)]
pub enum FileContent {
    Text(String),
    Data(Vec<u8>)
}

#[derive(Debug, Serialize)]
pub enum AccessError {
    NotFound(PathBuf),
    AccessDenied(PathBuf)
}

impl Display for AccessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessError::NotFound(path) =>
                write!(f, "Could not find file at {}.", path.to_string_lossy()),
            AccessError::AccessDenied(path) =>
                write!(f, "Access to file {} is denied.", path.to_string_lossy())
        }
    }
}

impl Error for AccessError { }

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AccessManagerState {
    version: u32,

    #[serde(skip)]
    locked: bool,
    #[serde(skip)]
    dismiss: HashSet<PathBuf>,

    access: HashSet<PathBuf>
}

pub struct AccessManager {
    watcher: Option<Arc<Mutex<RecommendedWatcher>>>,
    state: Arc<Watch<AccessManagerState>>
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum AccessFileData {
    Binary(Vec<u8>),
    Text(String)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum AccessSelection {
    AllText,
    AllBinary,
    Binary(HashSet<String>),
    Text(HashSet<String>),
}

impl AccessSelection {
    pub fn is_binary(&self, extension: &Option<&str>) -> bool {
        match self {
            AccessSelection::AllText => false,
            AccessSelection::AllBinary => true,
            AccessSelection::Binary(values) =>
                extension.map(|x| values.contains(x)).unwrap_or(false),
            AccessSelection::Text(values) =>
                extension.map(|x| !values.contains(x)).unwrap_or(false)
        }
    }
}

#[derive(Serialize)]
pub struct AccessFile<Data> {
    pub path: PathBuf,
    pub name: Option<String>,
    pub extension: Option<String>,
    pub data: Data
}

#[derive(Deserialize)]
pub struct AccessFilter {
    pub name: String,
    pub extensions: Vec<String>
}

#[derive(Clone, Serialize)]
pub struct AccessModify {
    pub path: String,
    pub data: AccessFileData
}

impl AccessManager {
    fn watcher(
        app: AppHandle, details: Arc<Watch<AccessManagerState>>
    ) -> Option<RecommendedWatcher> {
        notify::recommended_watcher(move |event: Result<notify::Event, notify::Error>| {
            let Ok(event) = event else { return };

            let mut details = details.get_silent();

            let Some(path) = event.paths.first() else {
                return
            };

            match event.kind {
                notify::EventKind::Create(_) => {
                    // app.emit_all("save:create", path).ok();
                }
                notify::EventKind::Remove(_) => {
                    // app.emit_all("save:remove", path).ok();
                }
                notify::EventKind::Modify(ModifyKind::Name(_)) => {
                    if path.exists() {
                        // app.emit_all("save:create", path).ok();
                    } else {
                        // app.emit_all("save:remove", path).ok();
                    }
                }
                notify::EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Any) => {
                    // If the path is in dismiss, we probably caused the save.
                    if details.dismiss.remove(path) {
                        return
                    }

                    if let Ok(data) = fs::read_to_string(path) {
                        // app.emit_all("save:modify", AccessModify {
                        //     path: path.to_string_lossy().to_string(), data: Text(data)
                        // }).ok();
                    }
                }
                _ => {}
            }
        }).ok()
    }

    fn create_config_path(config: &Config) -> Option<PathBuf> {
        let mut path = tauri::api::path::app_data_dir(config)?;

        fs::create_dir_all(&path).ok()?;

        path.push(ACCESS_CONFIG_FILENAME);

        Some(path)
    }

    fn read_config(path: &Path) -> Option<AccessManagerState> {
        let value = fs::read_to_string(path).ok()?;

        serde_json::from_str(&value).ok()
            .filter(|state: &AccessManagerState| state.version == ACCESS_CONFIG_VERSION)
    }

    fn write_config(path: &Path, state: &AccessManagerState) -> Option<()> {
        let value = serde_json::to_string(state).ok()?;

        fs::write(path, value).ok()
    }

    fn new(app: AppHandle, state: AccessManagerState) -> Self {
        let config_path = Self::create_config_path(&app.config());
        let watch = Watch::new(state);

        let config_path_clone = config_path.clone();

        watch.listen(move |state, _| {
            if let Some(path) = &config_path_clone {
                Self::write_config(path, state);
            }
        });

        let state = Arc::new(watch);
        let watcher = Self::watcher(app, state.clone())
            .map(|watcher| Arc::new(Mutex::new(watcher)));

        if let Some(watcher) = &watcher {
            let watcher = watcher.clone();

            state.listen(move |state, old| {
                let new_states = state.access.difference(&old.access);
                let removed_states = old.access.difference(&state.access);

                let mut watcher = watcher.lock().unwrap();

                for path in new_states {
                    watcher.watch(path, RecursiveMode::NonRecursive).ok();
                }

                for path in removed_states {
                    watcher.unwatch(path).ok();
                }
            });
        }

        AccessManager {
            state,
            watcher
        }
    }

    pub fn load(app: AppHandle) -> Self {
        // Create Config Path is also called in new()
        let config_path = Self::create_config_path(&app.config());

        let config = config_path
            .and_then(|path| Self::read_config(&path))
            .unwrap_or(AccessManagerState {
                version: ACCESS_CONFIG_VERSION,
                locked: false,
                dismiss: HashSet::new(),
                access: HashSet::new()
            });

        Self::new(app, config)
    }

    pub fn watch(&self, path: &Path) {
        if let Some(watcher) = &self.watcher {
            watcher.lock().unwrap().watch(path, RecursiveMode::NonRecursive).ok();
        }
    }

    pub fn sync(&self, items: HashSet<PathBuf>) {
        let mut value = self.state.get_mut();

        value.access.retain(|x| items.contains(x));

        if let Some(watcher) = &self.watcher {
            let mut watcher = watcher.lock().unwrap();

            for path in &value.access {
                watcher.watch(path, RecursiveMode::NonRecursive).ok();
            }
        }
    }

    fn builder(title: &str, filters: &[AccessFilter]) -> FileDialogBuilder {
        let mut builder = FileDialogBuilder::new()
            .set_title(title);

        for filter in filters {
            let extensions: Vec<&str> = filter.extensions.iter().map(|x| x.as_str()).collect();

            builder = builder.add_filter(&filter.name, &extensions);
        }

        builder
    }

    pub async fn lock<T, Fut: Future<Output=T>>(&self, f: Fut) -> Option<T> {
        {
            let mut lock = self.state.get_silent();

            if lock.locked {
                return None
            }

            lock.locked = true;
        }

        let t = f.await;

        self.state.get_silent().locked = false;

        Some(t)
    }

    pub fn has_access(&self, path: &PathBuf) -> bool {
        self.state.get().access.contains(path)
    }

    pub fn permit(&self, mut paths: Vec<PathBuf>) {
        let mut value = self.state.get_mut();

        paths.retain(|path| !value.access.contains(path));

        value.access.extend(paths);
    }

    async fn dispatch_select<F: FnOnce(Sender<Option<PathBuf>>)>(&self, f: F, permit: bool) -> Option<PathBuf> {
        self.lock(async {
            let (sender, receiver) = oneshot::channel();

            f(sender);

            let path = receiver.await.ok().flatten()?;

            if permit {
                self.permit(vec![path.clone()]);
            }

            Some(path)
        }).await.flatten()
    }

    pub async fn select_save(&self, title: &str, filters: &[AccessFilter], permit: bool) -> Option<PathBuf> {
        self.dispatch_select(|sender| {
            Self::builder(title, filters)
                .save_file(|file| { sender.send(file).ok(); });
        }, permit).await
    }

    pub async fn select_open(&self, title: &str, filters: &[AccessFilter], permit: bool) -> Option<PathBuf> {
        self.dispatch_select(|sender| {
            Self::builder(title, filters)
                .pick_file(|file| { sender.send(file).ok(); });
        }, permit).await
    }
}

fn make_string(str: Option<&OsStr>) -> Option<String> {
    str
        .and_then(|x| x.to_str())
        .map(|x| x.to_string())
}

#[tauri::command]
pub fn access_sync(paths: HashSet<PathBuf>, state: tauri::State<AccessManager>) {
    state.sync(paths)
}

#[tauri::command]
pub async fn access_select_save(
    title: &str, filters: Vec<AccessFilter>, state: tauri::State<'_, AccessManager>
) -> Result<Option<AccessFile<()>>, ()> {
    let Some(path) = state.select_save(title, &filters, true).await else {
        return Ok(None)
    };

    let name = make_string(path.file_name());
    let extension = make_string(path.extension());

    Ok(Some(AccessFile {
        path,
        name,
        extension,
        data: ()
    }))
}

#[tauri::command]
pub fn access_write_text(
    path: PathBuf, content: &str, state: tauri::State<'_, AccessManager>
) -> Result<(), AccessError> {
    if !state.has_access(&path) {
        return Err(AccessError::AccessDenied(path))
    }

    state.state.get_silent().dismiss.insert(path.clone());

    fs::write(&path, content).map_err(|_| AccessError::NotFound(path.clone()))?;

    // Initial watch from select_open may fail because the file does not exist.
    state.watch(&path);

    Ok(())
}

#[tauri::command]
pub fn access_read_text(
    path: PathBuf, state: tauri::State<'_, AccessManager>
) -> Result<String, AccessError> {
    if !state.has_access(&path) {
        return Err(AccessError::AccessDenied(path))
    }

    fs::read_to_string(&path).map_err(|_| AccessError::NotFound(path))
}

#[tauri::command]
pub fn access_read_file(
    path: PathBuf, state: tauri::State<'_, AccessManager>
) -> Result<AccessFile<FileContent>, AccessError> {
    if !state.has_access(&path) {
        return Err(AccessError::AccessDenied(path))
    }

    let name = make_string(path.file_name());
    let extension = make_string(path.extension());

    if extension == Some("elf".to_string()) {
        let data = fs::read(&path).map_err(|_| AccessError::NotFound(path.clone()))?;

        Ok(AccessFile {
            name,
            extension,
            path,
            data: FileContent::Data(data)
        })
    } else {
        let data = fs::read_to_string(&path).map_err(|_| AccessError::NotFound(path.clone()))?;

        Ok(AccessFile {
            name,
            extension,
            path,
            data: FileContent::Text(data)
        })
    }
}

#[tauri::command]
pub async fn access_select_open(
    title: &str, filters: Vec<AccessFilter>, selection: Option<AccessSelection>, state: tauri::State<'_, AccessManager>
) -> Result<Option<AccessFile<AccessFileData>>, ()> {
    let selection = selection.unwrap_or(AccessSelection::AllText);

    let Some(path) = state.select_open(title, &filters, true).await else {
        return Ok(None)
    };

    let name = make_string(path.file_name());
    let extension = make_string(path.extension());

    let is_binary = selection.is_binary(&extension.as_deref());

    let data = if is_binary {
        fs::read(&path).ok().map(Binary)
    } else {
        fs::read_to_string(&path).ok().map(Text)
    };

    // Cannot Read Data -> We Probably Want to Discard
    let Some(data) = data else {
        return Ok(None)
    };

    Ok(Some(AccessFile {
        path,
        name,
        extension,
        data
    }))
}
