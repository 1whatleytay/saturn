#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use serde::{Serialize};
use titan::debug::Debugger;
use titan::debug::debugger::DebugFrame;

use titan::elf::Elf;
use titan::debug::elf::inspection::Inspection;
use titan::debug::elf::setup::{create_simple_state, SMALL_HEAP_SIZE};

#[derive(Serialize)]
struct DisassembleResult {
    error: Option<String>,

    lines: Vec<String>,
    breakpoints: HashMap<usize, u32>
}

#[derive(Serialize)]
struct ResumeResult {
    mode: String,

    pc: u32,
    registers: [u32; 32],
    lo: u32,
    hi: u32
}

#[tauri::command]
fn disassemble(named: Option<&str>, bytes: Vec<u8>) -> DisassembleResult {
    let elf = match Elf::read(&mut Cursor::new(bytes)) {
        Ok(elf) => elf,
        Err(error) => return DisassembleResult {
            error: Some(error.to_string()),
            lines: vec![], breakpoints: HashMap::new()
        }
    };

    let inspection = Inspection::new(named, &elf);

    DisassembleResult {
        error: None,
        lines: inspection.lines,
        breakpoints: inspection.breakpoints
    }
}

type DebuggerPointer = Arc<Mutex<Debugger>>;
type DebuggerState = Mutex<Option<DebuggerPointer>>;

#[tauri::command]
fn configure(bytes: Vec<u8>, state: tauri::State<'_, DebuggerState>) -> bool {
    let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else { return false };

    let cpu_state = create_simple_state(&elf, SMALL_HEAP_SIZE);

    let mut value = state.lock().unwrap();
    let debugger = Arc::new(Mutex::new(Debugger::new(cpu_state)));

    // Drop should cancel the last process and kill the other thread.
    *value = Some(debugger);

    true
}

#[tauri::command]
async fn resume(breakpoints: Vec<u32>, state: tauri::State<'_, DebuggerState>) -> Result<ResumeResult, ()> {
    let debugger = {
        let Some(pointer) = &*state.lock().unwrap() else { return Err(()) };

        pointer.clone()
    };

    let frame: DebugFrame = tokio::spawn(async move {
        let breakpoints_set = HashSet::from_iter(breakpoints.iter().cloned());

        Debugger::run(&debugger, &breakpoints_set)
    }).await.unwrap();

    Ok(ResumeResult {
        mode: format!("{:?}", frame.mode),
        pc: frame.pc,
        registers: frame.registers,
        lo: frame.lo,
        hi: frame.hi,
    })
}

#[tauri::command]
fn pause(state: tauri::State<'_, DebuggerState>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut debugger = pointer.lock().unwrap();
    debugger.pause()
}

#[tauri::command]
fn stop(state: tauri::State<'_, DebuggerState>) {
    let debugger = &mut *state.lock().unwrap();

    *debugger = None;
}

#[tauri::command]
fn read_bytes(address: u32, count: u32, state: tauri::State<'_, DebuggerState>) -> Option<Vec<Option<u8>>> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    let mut debugger = pointer.lock().unwrap();
    let memory = debugger.memory();

    let value = (address .. count)
        .map(|a| memory.get(a).ok())
        .collect();

    Some(value)
}

#[tauri::command]
fn write_bytes(address: u32, bytes: Vec<u8>, state: tauri::State<'_, DebuggerState>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut debugger = pointer.lock().unwrap();
    let memory = debugger.memory();

    for (index, byte) in bytes.iter().enumerate() {
        memory.set(address + index as u32, *byte).ok();
    }
}

#[tauri::command]
fn set_register(register: u32, value: u32, state: tauri::State<'_, DebuggerState>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut debugger = pointer.lock().unwrap();
    let state = debugger.state();

    match register {
        0 ..= 31 => state.registers[register as usize] = value,
        32 => state.hi = value,
        33 => state.lo = value,
        _ => { }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(None) as DebuggerState)
        .invoke_handler(tauri::generate_handler![
            disassemble,
            configure,
            resume,
            pause,
            stop,
            read_bytes,
            write_bytes,
            set_register
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
