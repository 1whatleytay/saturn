use crate::state::DebuggerBody;
use saturn_backend::display::{DisplayData, FlushDisplayBody, FlushDisplayState};
use tauri::http::method::Method;
use tauri::http::{Request, Response};
use tauri::{Manager, UriSchemeContext, Wry};
use saturn_backend::device::ReadDisplayTarget;

#[tauri::command]
pub fn configure_display(
    address: u32,
    use_default_register: bool,
    width: u32,
    height: u32,
    state: tauri::State<FlushDisplayBody>,
) {
    let mut body = state.lock().unwrap();

    *body = FlushDisplayState {
        target: ReadDisplayTarget::from_arguments(use_default_register, address),
        width,
        height,
        data: None,
    }
}

#[tauri::command]
pub fn last_display(state: tauri::State<FlushDisplayBody>) -> DisplayData {
    state.lock().unwrap().clone().clone_data()
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
        let use_default_register = headers.get("use-default-register")
            .and_then(|x| x.to_str().ok())
            .map(|x| match x.to_lowercase().as_str() {
                "true" => true,
                "false" => false,
                _ => false,
            })
            .unwrap_or(false);

        let target = if use_default_register {
            ReadDisplayTarget::DefaultRegister
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
