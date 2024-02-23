use std::sync::{Arc, Mutex};
use saturn_backend::display::FlushDisplayBody;
use saturn_backend::execution::{ResumeResult, RewindableDevice};

pub type DebuggerBody = Mutex<Option<Arc<dyn RewindableDevice>>>;

#[tauri::command]
pub async fn last_pc(state: tauri::State<'_, DebuggerBody>) -> Result<Option<u32>, ()> {
    Ok(state.lock().unwrap()
        .as_ref()
        .map(|debugger| debugger.last_pc())
        .and_then(|x| x))
}

#[tauri::command]
pub async fn resume(
    count: Option<u32>,
    breakpoints: Option<Vec<u32>>,
    state: tauri::State<'_, DebuggerBody>,
    display: tauri::State<'_, FlushDisplayBody>,
) -> Result<ResumeResult, ()> {
    let context = {
        let Some(pointer) = &*state.lock().unwrap() else { return Err(()) };

        pointer.clone()
    };

    context.resume(count, breakpoints, Some(display.inner().clone())).await
}

#[tauri::command]
pub fn rewind(state: tauri::State<'_, DebuggerBody>, count: u32) -> Option<ResumeResult> {
    Some(state.lock().unwrap().as_ref()?.rewind(count))
}

#[tauri::command]
pub fn pause(state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.pause();
}

#[tauri::command]
pub fn stop(state: tauri::State<'_, DebuggerBody>) {
    let debugger = &mut *state.lock().unwrap();

    if let Some(pointer) = debugger {
        pointer.pause();
    }

    *debugger = None;
}

#[tauri::command]
pub fn post_key(key: char, up: bool, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.post_key(key, up)
}

#[tauri::command]
pub fn post_input(text: String, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.post_input(text)
}

#[tauri::command]
pub fn wake_sync(state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.wake_sync()
}
