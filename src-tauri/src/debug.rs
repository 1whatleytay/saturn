use std::collections::HashSet;
use titan::cpu::Memory;
use crate::state::DebuggerBody;

#[tauri::command]
pub fn swap_breakpoints(breakpoints: Vec<u32>, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

    pointer.debugger.lock().unwrap().set_breakpoints(breakpoints_set);
}

#[tauri::command]
pub fn read_bytes(
    address: u32, count: u32, state: tauri::State<'_, DebuggerBody>
) -> Option<Vec<Option<u8>>> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    let mut debugger = pointer.debugger.lock().unwrap();
    let memory = debugger.memory();

    let end = address
        .checked_add(count)
        .map_or(None, |value| value.checked_sub(1))
        .unwrap_or(u32::MAX);

    let value: Vec<Option<u8>> = (address ..= end)
        .map(|a| memory.get(a).ok())
        .collect();

    Some(value)
}

#[tauri::command]
pub fn write_bytes(address: u32, bytes: Vec<u8>, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut debugger = pointer.debugger.lock().unwrap();
    let memory = debugger.memory();

    for (index, byte) in bytes.iter().enumerate() {
        memory.set(address + index as u32, *byte).ok();
    }
}

#[tauri::command]
pub fn set_register(register: u32, value: u32, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut debugger = pointer.debugger.lock().unwrap();
    let state = debugger.state();

    match register {
        0 ..= 31 => state.registers.line[register as usize] = value,
        32 => state.registers.hi = value,
        33 => state.registers.lo = value,
        34 => state.registers.pc = value,
        _ => { }
    }
}
