#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use titan::assembler::binary::Binary;
use titan::assembler::line_details::LineDetails;
use titan::assembler::source::{assemble_from, SourceError};
use titan::cpu::{Memory, State};
use titan::cpu::memory::{Mountable, Region};
use titan::cpu::memory::section::SectionMemory;
use titan::debug::Debugger;
use titan::debug::debugger::{DebugFrame, DebuggerMode};

use titan::elf::Elf;
use titan::debug::elf::inspection::Inspection;
use titan::debug::elf::setup::{create_simple_state};

#[derive(Serialize)]
struct DisassembleResult {
    error: Option<String>,

    lines: Vec<String>,
    breakpoints: HashMap<u32, usize>
}

#[derive(Serialize)]
struct LineMarker {
    line: usize,
    offset: usize
}

#[derive(Serialize)]
#[serde(tag="status")]
enum AssemblerResult {
    Error { marker: Option<LineMarker>, message: String, body: Option<String> },
    Success { breakpoints: HashMap<u32, usize> }
}

#[derive(Serialize)]
#[serde(tag="type", content="value")]
enum ResumeMode {
    Running,
    Invalid(String),
    Paused,
    Breakpoint,
    Finished(u32),
}

impl From<DebuggerMode> for ResumeMode {
    fn from(value: DebuggerMode) -> Self {
        match value {
            DebuggerMode::Running => ResumeMode::Running,
            DebuggerMode::Invalid(error) => ResumeMode::Invalid(format!("{}", error)),
            DebuggerMode::Paused => ResumeMode::Paused,
            DebuggerMode::Breakpoint => ResumeMode::Breakpoint,
            DebuggerMode::Finished(address) => ResumeMode::Finished(address),
        }
    }
}

#[derive(Serialize)]
struct ResumeResult {
    mode: ResumeMode,

    pc: u32,
    registers: [u32; 32],
    lo: u32,
    hi: u32
}

impl AssemblerResult {
    fn from_result_with_binary(
        result: Result<Binary, SourceError>, source: &str
    ) -> (Option<Binary>, AssemblerResult) {
        match result {
            Ok(binary) => {
                let breakpoints = binary.source_breakpoints(source);

                (Some(binary), AssemblerResult::Success { breakpoints })
            },
            Err(error) => {
                let details = error.start()
                    .map(|offset| LineDetails::from_offset(source, offset));

                let marker = details.as_ref()
                    .map(|details| LineMarker {
                        line: details.line_number,
                        offset: details.line_number
                    });

                let body = details.as_ref()
                    .map(|details| format!("{}\n{}", details.line_text, details.marker()));

                (None, AssemblerResult::Error {
                    marker,
                    message: format!("{}", error),
                    body
                })
            }
        }
    }

    fn from_result(result: Result<Binary, SourceError>, source: &str) -> AssemblerResult {
        Self::from_result_with_binary(result, source).1
    }
}

impl ResumeResult {
    fn from_frame(frame: DebugFrame) -> ResumeResult {
        ResumeResult {
            mode: frame.mode.into(),
            pc: frame.pc,
            registers: frame.registers,
            lo: frame.lo,
            hi: frame.hi,
        }
    }
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

type DebuggerPointer = Arc<Mutex<Debugger<SectionMemory>>>;
type DebuggerState = Mutex<Option<DebuggerPointer>>;

fn state_from_binary(binary: Binary, heap_size: u32) -> State<SectionMemory> {
    let mut memory = SectionMemory::new();

    for region in binary.regions {
        let region = Region { start: region.address, data: region.data };

        memory.mount(region);
    }

    let heap_end = 0x7FFFFFFCu32;

    let heap = Region {
        start: heap_end - heap_size,
        data: vec![0; heap_size as usize]
    };

    memory.mount(heap);

    State::new(binary.entry, memory)
}

fn setup_state<T: Memory + Mountable>(state: &mut State<T>) {
    let screen = Region { start: 0x10008000, data: vec![0; 0x4000] };
    let keyboard = Region { start: 0xFFFF0000, data: vec![0; 0x100] };

    state.memory.mount(screen);
    state.memory.mount(keyboard);

    state.registers[28] = 0x10008000
}

#[tauri::command]
fn configure_elf(bytes: Vec<u8>, state: tauri::State<'_, DebuggerState>) -> bool {
    let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else { return false };

    let mut cpu_state = create_simple_state(&elf, 0x100000);
    setup_state(&mut cpu_state);

    let mut value = state.lock().unwrap();
    let debugger = Arc::new(Mutex::new(Debugger::new(cpu_state)));

    // Drop should cancel the last process and kill the other thread.
    *value = Some(debugger);

    true
}

pub fn source_breakpoints(map: &HashMap<usize, u32>, source: &str) -> HashMap<usize, u32> {
    let mut result = HashMap::new();

    let mut line_number = 0;
    let mut input = source;

    while let Some(c) = input.chars().next() {
        let next = &input[1..];

        let start = input.as_ptr() as usize - source.as_ptr() as usize;
        if let Some(pc) = map.get(&start).copied() {
            result.insert(line_number, pc);
        }

        if c == '\n' {
            line_number += 1;
        }

        input = next;
    }

    result
}

#[tauri::command]
fn configure_asm(text: &str, state: tauri::State<'_, DebuggerState>) -> AssemblerResult {
    let binary = assemble_from(text);
    let (binary, result) = AssemblerResult::from_result_with_binary(binary, text);

    let Some(binary) = binary else { return result };

    let mut cpu_state = state_from_binary(binary, 0x100000);
    setup_state(&mut cpu_state);

    let mut value = state.lock().unwrap();
    let debugger = Arc::new(Mutex::new(Debugger::new(cpu_state)));

    // Drop should cancel the last process and kill the other thread.
    *value = Some(debugger);

    result
}

#[tauri::command]
async fn resume(breakpoints: Vec<u32>, state: tauri::State<'_, DebuggerState>) -> Result<ResumeResult, ()> {
    let debugger = {
        let Some(pointer) = &*state.lock().unwrap() else { return Err(()) };

        pointer.clone()
    };

    let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

    debugger.lock().unwrap().set_breakpoints(breakpoints_set);

    let frame: DebugFrame = tokio::spawn(async move {
        Debugger::run(&debugger)
    }).await.unwrap();

    Ok(ResumeResult::from_frame(frame))
}

#[tauri::command]
fn swap_breakpoints(breakpoints: Vec<u32>, state: tauri::State<'_, DebuggerState>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

    pointer.lock().unwrap().set_breakpoints(breakpoints_set);
}

#[tauri::command]
fn step(state: tauri::State<'_, DebuggerState>) -> Option<ResumeResult> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    let mut debugger = pointer.lock().unwrap();

    let frame = debugger.cycle(true)
        .unwrap_or_else(|| debugger.frame());

    Some(ResumeResult::from_frame(frame))
}

#[tauri::command]
fn pause(state: tauri::State<'_, DebuggerState>) -> Option<ResumeResult> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    let mut debugger = pointer.lock().unwrap();
    debugger.pause();

    Some(ResumeResult::from_frame(debugger.frame()))
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

#[tauri::command]
fn assemble(text: &str) -> AssemblerResult {
    let result = assemble_from(text);

    AssemblerResult::from_result(result, text)
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(None) as DebuggerState)
        .invoke_handler(tauri::generate_handler![
            disassemble,
            configure_elf,
            configure_asm,
            resume,
            pause,
            step,
            stop,
            read_bytes,
            write_bytes,
            set_register,
            assemble,
            swap_breakpoints
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
