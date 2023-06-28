use std::sync::{Arc, Mutex};
use titan::assembler::binary::Binary;
use titan::cpu::{Memory, State};
use titan::cpu::memory::{Mountable, Region};
use titan::cpu::memory::section::SectionMemory;
use titan::cpu::memory::watched::WatchedMemory;
use crate::display::FlushDisplayBody;
use crate::state::device::MemoryType;
use crate::state::execution::{ExecutionDevice, ResumeResult};

pub type DebuggerBody = Mutex<Option<Arc<dyn ExecutionDevice>>>;

pub fn state_from_binary(binary: Binary, heap_size: u32) -> State<MemoryType> {
    let mut memory = WatchedMemory::new(SectionMemory::new());

    for region in binary.regions {
        let region = Region {
            start: region.address,
            data: region.data,
        };

        memory.mount(region);
    }

    // Keeping this around temporarily.
    let heap_end = 0x80000000u32;

    let heap = Region {
        start: heap_end - heap_size,
        data: vec![0; heap_size as usize],
    };

    memory.mount(heap);

    let mut state = State::new(binary.entry, memory);

    state.registers.line[29] = heap_end - 4; // give some space

    state
}

pub fn setup_state<Mem: Memory + Mountable>(state: &mut State<Mem>) {
    let max_screen = 0x8000;
    let screen = Region {
        start: 0x10008000,
        data: vec![0; max_screen],
    };

    state.memory.mount(screen);

    state.registers.line[28] = 0x10008000
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
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    pointer.rewind(count)
}

#[tauri::command]
pub fn pause(state: tauri::State<'_, DebuggerBody>, display: tauri::State<'_, FlushDisplayBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.pause(Some(&mut display.lock().unwrap()))
}

#[tauri::command]
pub fn stop(state: tauri::State<'_, DebuggerBody>, display: tauri::State<'_, FlushDisplayBody>) {
    let debugger = &mut *state.lock().unwrap();

    if let Some(pointer) = debugger {
        pointer.pause(Some(&mut display.lock().unwrap()));
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
