use serde::Serialize;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tauri::http::method::Method;
use tauri::http::Response;
use tauri::http::{Request, ResponseBuilder};
use tauri::{AppHandle, Manager, Wry};
use titan::cpu::Memory;
use crate::state::commands::DebuggerBody;
use crate::state::execution::read_display;

#[derive(Clone, Serialize)]
pub struct FlushDisplayState {
    pub address: u32,
    pub width: u32,
    pub height: u32,
    pub data: Option<Vec<u8>>, // flush should impact this
}

impl Default for FlushDisplayState {
    fn default() -> FlushDisplayState {
        FlushDisplayState {
            address: 0x10008000,
            width: 64,
            height: 64,
            data: None,
        }
    }
}

impl FlushDisplayState {
    pub fn flush<Mem: Memory>(&mut self, memory: &mut Mem) {
        self.data = read_display(self.address, self.width, self.height, memory);
    }
}

pub type FlushDisplayBody = Arc<Mutex<FlushDisplayState>>;

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
