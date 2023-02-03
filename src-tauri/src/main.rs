#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod syscall;
mod keyboard;
mod menu;
mod state;

use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use tauri::{Manager, Wry};
use tauri::http::ResponseBuilder;
use titan::assembler::binary::Binary;
use titan::assembler::line_details::LineDetails;
use titan::assembler::source::{assemble_from, SourceError};
use titan::cpu::Memory;
use titan::cpu::state::Registers;
use titan::debug::debugger::{Debugger, DebugFrame, DebuggerMode};

use titan::elf::Elf;
use titan::debug::elf::inspection::Inspection;
use titan::debug::elf::setup::{create_simple_state};
use titan::elf::program::ProgramHeaderFlags;
use crate::menu::{create_menu, handle_event};
use crate::state::{DebuggerBody, MemoryType, setup_state, state_from_binary, swap};
use crate::syscall::{SyscallDelegate, SyscallResult, SyscallState};

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
#[serde(tag="type")]
enum ResumeMode {
    Running,
    Invalid { message: String },
    Paused,
    Breakpoint,
    Finished { pc: u32, code: Option<u32> },
}

impl From<DebuggerMode> for ResumeMode {
    fn from(value: DebuggerMode) -> Self {
        match value {
            DebuggerMode::Running => ResumeMode::Running,
            DebuggerMode::Recovered => ResumeMode::Breakpoint,
            DebuggerMode::Invalid(error) => ResumeMode::Invalid {
                message: format!("{}", error)
            },
            DebuggerMode::Paused => ResumeMode::Paused,
            DebuggerMode::Breakpoint => ResumeMode::Breakpoint
        }
    }
}

#[derive(Serialize)]
struct RegistersResult {
    pc: u32,
    line: [u32; 32],
    lo: u32,
    hi: u32,
}

impl From<Registers> for RegistersResult {
    fn from(value: Registers) -> Self {
        RegistersResult {
            pc: value.pc,
            line: value.line,
            lo: value.lo,
            hi: value.hi
        }
    }
}

#[derive(Serialize)]
struct ResumeResult {
    mode: ResumeMode,
    registers: RegistersResult
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
                        offset: details.line_offset
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
    fn from_frame(frame: DebugFrame, finished_pcs: &Vec<u32>, result: Option<SyscallResult>) -> ResumeResult {
        let mode = match result {
            Some(SyscallResult::Failure(message)) => ResumeMode::Invalid { message },
            Some(SyscallResult::Terminated(code)) => ResumeMode::Finished {
                pc: frame.registers.pc, code: Some(code)
            },
            Some(SyscallResult::Exception(error)) => ResumeMode::Invalid {
                message: error.to_string()
            },
            Some(SyscallResult::Unimplemented(code)) => ResumeMode::Invalid {
                message: format!("Unimplemented syscall {}, file a bug or make a contribution at https://github.com/1whatleytay/saturn.", code)
            },
            Some(SyscallResult::Unknown(code)) => ResumeMode::Invalid {
                message: format!("Unrecognized syscall {}.", code)
            },
            _ => {
                // This is probably okay...
                if finished_pcs.contains(&frame.registers.pc) {
                    ResumeMode::Finished { pc: frame.registers.pc, code: None }
                } else {
                    frame.mode.into()
                }
            }
        };

        ResumeResult {
            mode,
            registers: frame.registers.into()
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

fn forward_print(app_handle: tauri::AppHandle<Wry>) -> Box<dyn FnMut(&str) -> () + Send> {
    Box::new(move |text: &str| {
        app_handle.emit_all("print", text).ok();
    })
}

#[tauri::command]
fn configure_elf(
    bytes: Vec<u8>, state: tauri::State<'_, DebuggerBody>,
    app_handle: tauri::AppHandle<Wry>
) -> bool {
    let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else { return false };

    let finished_pcs = elf.program_headers.iter()
        .filter(|header| header.flags.contains(ProgramHeaderFlags::EXECUTABLE))
        .map(|header| header.virtual_address + header.data.len() as u32)
        .collect();

    let mut cpu_state = create_simple_state(&elf, 0x100000);
    setup_state(&mut cpu_state);

    let print = forward_print(app_handle);
    swap(state.lock().unwrap(), Debugger::new(cpu_state), finished_pcs, print);

    true
}

#[tauri::command]
fn configure_asm(
    text: &str, state: tauri::State<'_, DebuggerBody>,
    app_handle: tauri::AppHandle<Wry>
) -> AssemblerResult {
    let binary = assemble_from(text);
    let (binary, result) = AssemblerResult::from_result_with_binary(binary, text);

    let Some(binary) = binary else { return result };

    // No EXECUTABLE marked regions from assembler yet.
    let finished_pcs = binary.regions.iter()
        .map(|region| region.address + region.data.len() as u32)
        .collect();

    let mut cpu_state = state_from_binary(binary, 0x100000);
    setup_state(&mut cpu_state);

    let print = forward_print(app_handle);
    swap(state.lock().unwrap(), Debugger::new(cpu_state), finished_pcs, print);

    result
}

type CloneResult = (Arc<Mutex<Debugger<MemoryType>>>, Arc<Mutex<SyscallState>>, Vec<u32>);

fn lock_and_clone(state: tauri::State<'_, DebuggerBody>) -> Option<CloneResult> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    Some((pointer.debugger.clone(), pointer.delegate.clone(), pointer.finished_pcs.clone()))
}

#[tauri::command]
async fn resume(breakpoints: Vec<u32>, state: tauri::State<'_, DebuggerBody>) -> Result<ResumeResult, ()> {
    let (
        debugger, state, finished_pcs
    ) = lock_and_clone(state).ok_or(())?;

    let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

    debugger.lock().unwrap().set_breakpoints(breakpoints_set);

    let (frame, result) = tokio::spawn(async move {
        SyscallDelegate::new(state).run(&debugger).await
    }).await.unwrap();

    Ok(ResumeResult::from_frame(frame, &finished_pcs, result))
}

#[tauri::command]
async fn step(state: tauri::State<'_, DebuggerBody>) -> Result<ResumeResult, ()> {
    let (
        debugger, state, finished_pcs
    ) = lock_and_clone(state).ok_or(())?;

    let (frame, result) = SyscallDelegate::new(state).cycle(&debugger).await;

    Ok(ResumeResult::from_frame(frame, &finished_pcs, result))
}

#[tauri::command]
fn swap_breakpoints(breakpoints: Vec<u32>, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

    pointer.debugger.lock().unwrap().set_breakpoints(breakpoints_set);
}

#[tauri::command]
fn pause(state: tauri::State<'_, DebuggerBody>) -> Option<ResumeResult> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    let mut debugger = pointer.debugger.lock().unwrap();
    debugger.pause();

    Some(ResumeResult::from_frame(debugger.frame(), &vec![], None))
}

#[tauri::command]
fn stop(state: tauri::State<'_, DebuggerBody>) {
    let debugger = &mut *state.lock().unwrap();

    if let Some(pointer) = debugger {
        pointer.debugger.lock().unwrap().pause();
        pointer.delegate.lock().unwrap().abort = true;
    }

    *debugger = None;
}

#[tauri::command]
fn post_key(key: char, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut keyboard = pointer.keyboard.lock().unwrap();

    keyboard.push_key(key)
}

#[tauri::command]
fn read_bytes(address: u32, count: u32, state: tauri::State<'_, DebuggerBody>) -> Option<Vec<Option<u8>>> {
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
fn write_bytes(address: u32, bytes: Vec<u8>, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut debugger = pointer.debugger.lock().unwrap();
    let memory = debugger.memory();

    for (index, byte) in bytes.iter().enumerate() {
        memory.set(address + index as u32, *byte).ok();
    }
}

#[tauri::command]
fn set_register(register: u32, value: u32, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut debugger = pointer.debugger.lock().unwrap();
    let state = debugger.state();

    match register {
        0 ..= 31 => state.registers.line[register as usize] = value,
        32 => state.registers.hi = value,
        33 => state.registers.lo = value,
        _ => { }
    }
}

#[tauri::command]
fn assemble(text: &str) -> AssemblerResult {
    let result = assemble_from(text);

    AssemblerResult::from_result(result, text)
}

#[tauri::command]
fn assemble_binary(text: &str) -> (Option<Vec<u8>>, AssemblerResult) {
    let result = assemble_from(text);
    let (binary, result) = AssemblerResult::from_result_with_binary(result, text);

    let Some(binary) = binary else {
        return (None, result)
    };

    let elf: Elf = binary.into();

    let mut out: Vec<u8> = vec![];
    let mut cursor = Cursor::new(&mut out);

    if let Err(error) = elf.write(&mut cursor) {
        return (None, AssemblerResult::Error {
            marker: None,
            message: error.to_string(),
            body: None,
        })
    }

    (Some(out), result)
}

// NOT a tauri command.
fn read_display(address: u32, width: u32, height: u32, state: tauri::State<'_, DebuggerBody>) -> Option<Vec<u8>> {
    let Some(pointer) = &*state.lock().unwrap() else { return None };

    let mut debugger = pointer.debugger.lock().unwrap();
    let memory = debugger.memory();

    let pixels = width.checked_mul(height)?;

    let mut result = vec![0u8; (pixels * 4) as usize];

    for i in 0 .. pixels {
        let point = address.wrapping_add(i.wrapping_mul(4));

        let pixel = memory.get_u32(point).ok()?;

        // Assuming little endian: 0xAARRGGBB -> [BB, GG, RR, AA] -> want [RR, GG, BB, AA]
        let start = (i as usize) * 4;
        result[start + 0] = (pixel.wrapping_shr(16) & 0xFF) as u8;
        result[start + 1] = (pixel.wrapping_shr(8) & 0xFF) as u8;
        result[start + 2] = (pixel.wrapping_shr(0) & 0xFF) as u8;
        result[start + 3] = 255;
    }

    Some(result)
}

fn main() {
    let menu = create_menu();

    tauri::Builder::default()
        .manage(Mutex::new(None) as DebuggerBody)
        .menu(menu)
        .on_menu_event(handle_event)
        .invoke_handler(tauri::generate_handler![
            disassemble,
            configure_elf,
            configure_asm,
            resume,
            step,
            pause,
            stop,
            read_bytes,
            write_bytes,
            set_register,
            assemble,
            post_key,
            swap_breakpoints,
            assemble_binary
        ])
        .register_uri_scheme_protocol("display", |app, request| {
            let grab_params = || -> Option<(u32, u32, u32)> {
                let headers = request.headers();

                let width = headers.get("width")?.to_str().ok()?;
                let height = headers.get("height")?.to_str().ok()?;
                let address = headers.get("address")?.to_str().ok()?;

                Some((width.parse().ok()?, height.parse().ok()?, address.parse().ok()?))
            };

            // Disable CORS, nothing super private here.
            let builder = ResponseBuilder::new()
                .header("Access-Control-Allow-Origin", "*");

            let Some((width, height, address)) = grab_params() else {
                return builder
                    .status(400)
                    .body(vec![]);
            };

            let state: tauri::State<'_, DebuggerBody> = app.state();

            let Some(result) = read_display(address, width, height, state) else {
                return builder
                    .status(400)
                    .body(vec![]);
            };

            builder.body(result)
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
