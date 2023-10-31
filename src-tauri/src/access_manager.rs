use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tauri::api::dialog::FileDialogBuilder;
use tauri::{AppHandle, Config};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use crate::access_manager::AccessFileData::{Binary, Text};
use crate::watch::Watch;

const ACCESS_CONFIG_VERSION: u32 = 1;
const ACCESS_CONFIG_FILENAME: &str = "save-state.json";

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
    access: Vec<PathBuf>
}

pub struct AccessManager {
    app: AppHandle,
    state: Arc<Watch<AccessManagerState>>
}

#[derive(Serialize)]
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

impl AccessManager {
    fn config_path(config: &Config) -> Option<PathBuf> {
        let mut path = tauri::api::path::app_data_dir(config)?;

        path.push(ACCESS_CONFIG_FILENAME);

        Some(path)
    }

    fn read_config(config: &Config) -> Option<AccessManagerState> {
        let value = fs::read_to_string(Self::config_path(config)?).ok()?;

        serde_json::from_str(&value).ok()
            .filter(|state: &AccessManagerState| state.version == ACCESS_CONFIG_VERSION)
    }

    fn write_config(config: &Config, state: &AccessManagerState) -> Option<()> {
        let value = serde_json::to_string(state).ok()?;

        fs::write(Self::config_path(config)?, value).ok()
    }

    fn new(app: AppHandle, state: AccessManagerState) -> Self {
        let app_clone = app.clone();

        let watch = Watch::new(state)
            .listen(move |state| {
                Self::write_config(&app_clone.config(), state);
            })
            .listen(|state| {
                println!("Modified: {:?}", state);
            });

        AccessManager {
            state: Arc::new(watch),
            app
        }
    }

    pub fn load(app: AppHandle) -> Self {
        let config = Self::read_config(&app.config())
            .unwrap_or(AccessManagerState {
                version: ACCESS_CONFIG_VERSION,
                locked: false,
                access: vec![]
            });

        Self::new(app, config)
    }

    pub fn sync(&self, items: HashSet<PathBuf>) {
        self.state.get_mut().access.retain(|x| items.contains(x));
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

    async fn dispatch_select<F: FnOnce(Sender<Option<PathBuf>>)>(&self, f: F) -> Option<PathBuf> {
        self.lock(async {
            let (sender, receiver) = oneshot::channel();

            f(sender);

            let path = receiver.await.ok().flatten()?;

            self.permit(vec![path.clone()]);

            Some(path)
        }).await.flatten()
    }

    pub async fn select_save(&self, title: &str, filters: &[AccessFilter]) -> Option<PathBuf> {
        self.dispatch_select(|sender| {
            Self::builder(title, filters)
                .save_file(|file| { sender.send(file).ok(); });
        }).await
    }

    pub async fn select_open(&self, title: &str, filters: &[AccessFilter]) -> Option<PathBuf> {
        self.dispatch_select(|sender| {
            Self::builder(title, filters)
                .pick_file(|file| { sender.send(file).ok(); });
        }).await
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
    let Some(path) = state.select_save(title, &filters).await else {
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

    fs::write(&path, content).map_err(|_| AccessError::NotFound(path))
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
pub async fn access_select_open(
    title: &str, filters: Vec<AccessFilter>, selection: Option<AccessSelection>, state: tauri::State<'_, AccessManager>
) -> Result<Option<AccessFile<AccessFileData>>, ()> {
    let selection = selection.unwrap_or(AccessSelection::AllText);

    let Some(path) = state.select_open(title, &filters).await else {
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
