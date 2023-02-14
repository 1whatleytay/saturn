use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use percent_encoding::percent_decode_str;
use tauri::{AppHandle, Wry};
use tauri::api::path::app_local_data_dir;
use tauri::http::{Request, Response, ResponseBuilder, Uri};
use tauri::http::method::Method;

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

        directory.push("midi/");
        let base = directory.clone();

        directory.push(path);

        // Hopefully no workarounds for this.
        if !directory.starts_with(base) {
            return None
        }

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
