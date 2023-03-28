use crate::midi::protocol::MidiProtocolError::{
    FileIO, MissingAppDir, OutOfBounds, PathResolve, URIDecode, URIParse,
};
use percent_encoding::percent_decode_str;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use tauri::api::path::app_local_data_dir;
use tauri::http::method::Method;
use tauri::http::{Request, Response, ResponseBuilder, Uri};
use tauri::{AppHandle, Wry};

#[derive(Debug)]
enum MidiProtocolError {
    URIDecode(String),
    URIParse(String),
    MissingAppDir,
    PathResolve(String),
    OutOfBounds,
    FileIO(Option<String>),
}

impl Display for MidiProtocolError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            URIDecode(url) => write!(f, "Failed to decode given URI: {}", url),
            URIParse(url) => write!(f, "Failed to parse percent decoded URI: {}", url),
            MissingAppDir => write!(f, "Could not resolve app directory."),
            PathResolve(path) => write!(f, "Could not resolve path: {}", path),
            OutOfBounds => write!(f, "Directory moved out of bounds."),
            FileIO(path) => write!(
                f,
                "Could not read from file: {}",
                path.as_ref().unwrap_or(&"None".to_string())
            ),
        }
    }
}

impl Error for MidiProtocolError {}

fn get_path(uri: &str, config: &tauri::Config) -> Result<String, MidiProtocolError> {
    let decode = percent_decode_str(uri)
        .decode_utf8()
        .map_err(|_| URIDecode(uri.into()))?;

    let uri = Uri::try_from(&*decode).map_err(|_| URIParse(decode.to_string()))?;
    let mut directory = app_local_data_dir(config).ok_or(MissingAppDir)?;

    let path = Path::new(uri.path())
        .strip_prefix("/")
        .map_err(|_| PathResolve(uri.path().into()))?;

    directory.push("midi/");
    let base = directory.clone();

    directory.push(path);

    // Hopefully no workarounds for this.
    if !directory.starts_with(base) {
        return Err(OutOfBounds);
    }

    fs::read_to_string(&directory).map_err(|_| FileIO(directory.to_str().map(|x| x.into())))
}

pub fn midi_protocol(app: &AppHandle<Wry>, request: &Request) -> Result<Response, Box<dyn Error>> {
    // Disable CORS, nothing super private here.
    let builder = ResponseBuilder::new()
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Origin", "*");

    // Check for preflight, very primitive check.
    if request.method() == Method::OPTIONS {
        return builder.body(vec![]);
    }

    match get_path(request.uri(), &app.config()) {
        Ok(text) => builder.body(text.into_bytes()),
        Err(error) => builder.status(400).body(error.to_string().into_bytes()),
    }
}
