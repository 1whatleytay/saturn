#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;
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

    let frame = tokio::spawn(async move {
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
    let Some(debugger) = &*state.lock().unwrap() else { return };

    let mut other = debugger.lock().unwrap();
    other.pause()
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(None) as DebuggerState)
        .invoke_handler(tauri::generate_handler![
            disassemble,
            configure,
            resume,
            pause
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
