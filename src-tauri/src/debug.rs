use std::collections::HashSet;
use crate::state::DebuggerBody;

#[tauri::command]
pub fn swap_breakpoints(breakpoints: Vec<u32>, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

    pointer.set_breakpoints(breakpoints_set)
}

#[tauri::command]
pub fn read_bytes(
    address: u32,
    count: u32,
    state: tauri::State<'_, DebuggerBody>,
) -> Option<Vec<Option<u8>>> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    pointer.read_bytes(address, count)
}

#[tauri::command]
pub fn write_bytes(address: u32, bytes: Vec<u8>, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.write_bytes(address, bytes)
}

#[tauri::command]
pub fn set_register(register: u32, value: u32, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.write_register(register, value)
}
