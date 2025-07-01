use crate::state::DebuggerBody;
use num::FromPrimitive;
use saturn_backend::display::{FlushDisplayBody, FlushDisplayState};
use saturn_backend::mips::execution::ReadDisplayTarget;
use tauri::http::method::Method;
use tauri::http::{Request, Response};
use tauri::{Manager, UriSchemeContext, Wry};
use titan::mips::assembler::registers::RegisterSlot;

#[tauri::command]
pub fn configure_display(
    address: u32,
    register: Option<u8>,
    width: u32,
    height: u32,
    state: tauri::State<FlushDisplayBody>,
) {
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
    context: UriSchemeContext<'_, Wry>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let app = context.app_handle();

    // Disable CORS, nothing super private here.
    let builder = Response::builder()
        .header("Access-Control-Allow-Headers", "*")
        .header("Access-Control-Allow-Origin", "*");

    // Check for preflight, very primitive check.
    if request.method() == Method::OPTIONS {
        return builder.body(vec![]).expect("Failed to build response");
    }

    let grab_params = || -> Option<(u32, u32, ReadDisplayTarget)> {
        let headers = request.headers();

        let width = headers.get("width")?.to_str().ok()?;
        let height = headers.get("height")?.to_str().ok()?;
        // address is still required as fallback
        let register = headers.get("register").and_then(|x| x.to_str().ok());

        let target = if let Some(register) = register {
            ReadDisplayTarget::Register(RegisterSlot::from_u8(register.parse().ok()?)?)
        } else {
            let address = headers.get("address")?.to_str().ok()?;

            ReadDisplayTarget::Address(address.parse().ok()?)
        };

        Some((width.parse().ok()?, height.parse().ok()?, target))
    };

    let Some((width, height, address)) = grab_params() else {
        return builder
            .status(400)
            .body(vec![])
            .expect("Failed to build response");
    };

    let state: tauri::State<'_, DebuggerBody> = app.state();

    let Some(pointer) = &*state.lock().unwrap() else {
        return builder
            .status(400)
            .body(vec![])
            .expect("Failed to build response");
    };

    let Some(result) = pointer.read_display(address, width, height) else {
        return builder
            .status(400)
            .body(vec![])
            .expect("Failed to build response");
    };

    builder.body(result).expect("Failed to build response")
}
