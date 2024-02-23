use std::error::Error;
use tauri::{AppHandle, Manager, Wry};
use tauri::http::{Request, Response, ResponseBuilder};
use tauri::http::method::Method;
use saturn_backend::display::{FlushDisplayBody, FlushDisplayState};
use crate::state::DebuggerBody;

#[tauri::command]
pub fn configure_display(address: u32, width: u32, height: u32, state: tauri::State<FlushDisplayBody>) {
    let mut body = state.lock().unwrap();

    *body = FlushDisplayState {
        address,
        width,
        height,
        data: None,
    }
}

#[tauri::command]
pub fn last_display(state: tauri::State<FlushDisplayBody>) -> FlushDisplayState {
    state.lock().unwrap().clone()
}

pub fn display_protocol(
    app: &AppHandle<Wry>,
    request: &Request,
) -> Result<Response, Box<dyn Error>> {
    // Disable CORS, nothing super private here.
    let builder = ResponseBuilder::new()
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Origin", "*");

    // Check for preflight, very primitive check.
    if request.method() == Method::OPTIONS {
        return builder.body(vec![]);
    }

    let grab_params = || -> Option<(u32, u32, u32)> {
        let headers = request.headers();

        let width = headers.get("width")?.to_str().ok()?;
        let height = headers.get("height")?.to_str().ok()?;
        let address = headers.get("address")?.to_str().ok()?;

        Some((
            width.parse().ok()?,
            height.parse().ok()?,
            address.parse().ok()?,
        ))
    };

    let Some((width, height, address)) = grab_params() else {
        return builder.status(400).body(vec![])
    };

    let state: tauri::State<'_, DebuggerBody> = app.state();

    let Some(pointer) = &*state.lock().unwrap() else {
        return builder.status(400).body(vec![])
    };


    let Some(result) = pointer.read_display(address, width, height) else {
        return builder.status(400).body(vec![])
    };

    builder.body(result)
}
