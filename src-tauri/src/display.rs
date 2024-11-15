use std::error::Error;
use num::FromPrimitive;
use tauri::{AppHandle, Manager, Wry};
use tauri::http::{Request, Response, ResponseBuilder};
use tauri::http::method::Method;
use titan::unit::register::RegisterName;
use saturn_backend::display::{FlushDisplayBody, FlushDisplayState};
use saturn_backend::execution::ReadDisplayTarget;
use crate::state::DebuggerBody;

#[tauri::command]
pub fn configure_display(address: u32, register: Option<u8>, width: u32, height: u32, state: tauri::State<FlushDisplayBody>) {
    let mut body = state.lock().unwrap();

    *body = FlushDisplayState {
        address,
        register,
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

    let grab_params = || -> Option<(u32, u32, ReadDisplayTarget)> {
        let headers = request.headers();

        let width = headers.get("width")?.to_str().ok()?;
        let height = headers.get("height")?.to_str().ok()?;
        // address is still required as fallback
        let register = headers.get("register")
            .and_then(|x| x.to_str().ok());
        
        let target = if let Some(register) = register {
            ReadDisplayTarget::Register(RegisterName::from_u8(register.parse().ok()?)?)
        } else {
            let address = headers.get("address")?.to_str().ok()?;
            
            ReadDisplayTarget::Address(address.parse().ok()?)
        };
        
        Some((
            width.parse().ok()?,
            height.parse().ok()?,
            target,
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
