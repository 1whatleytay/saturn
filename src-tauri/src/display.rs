use crate::state::{DebuggerBody, MemoryType};
use serde::Serialize;
use std::error::Error;
use std::sync::Mutex;
use tauri::http::method::Method;
use tauri::http::Response;
use tauri::http::{Request, ResponseBuilder};
use tauri::{AppHandle, Manager, Wry};
use titan::cpu::Memory;

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

pub type FlushDisplayBody = Mutex<FlushDisplayState>;

// NOT a tauri command.
pub fn read_display(
    address: u32,
    width: u32,
    height: u32,
    memory: &mut MemoryType,
) -> Option<Vec<u8>> {
    let pixels = width.checked_mul(height)?;

    let mut result = vec![0u8; (pixels * 4) as usize];

    for i in 0..pixels {
        let point = address.wrapping_add(i.wrapping_mul(4));

        let pixel = memory.get_u32(point).ok()?;

        // Assuming little endian: 0xAARRGGBB -> [BB, GG, RR, AA] -> want [RR, GG, BB, AA]
        let start = (i as usize) * 4;
        result[start] = (pixel.wrapping_shr(16) & 0xFF) as u8;
        result[start + 1] = (pixel.wrapping_shr(8) & 0xFF) as u8;
        result[start + 2] = (pixel.wrapping_shr(0) & 0xFF) as u8;
        result[start + 3] = 255;
    }

    Some(result)
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

    let mut debugger = pointer.debugger.lock().unwrap();
    let memory = debugger.memory();

    let Some(result) = read_display(address, width, height, memory) else {
        return builder.status(400).body(vec![])
    };

    builder.body(result)
}
