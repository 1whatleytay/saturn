use std::collections::HashMap;
use std::io::Cursor;
use titan::debug::elf::inspection::Inspection;
use serde::Serialize;
use tauri::{Manager, Wry};
use titan::assembler::binary::Binary;
use titan::assembler::line_details::LineDetails;
use titan::assembler::source::{assemble_from, SourceError};
use titan::debug::Debugger;
use titan::debug::elf::setup::create_simple_state;
use titan::elf::Elf;
use titan::elf::program::ProgramHeaderFlags;
use crate::state::{DebuggerBody, setup_state, state_from_binary, swap};

#[derive(Serialize)]
pub struct LineMarker {
    line: usize,
    offset: usize
}

#[derive(Serialize)]
#[serde(tag="status")]
pub enum AssemblerResult {
    Error { marker: Option<LineMarker>, message: String, body: Option<String> },
    Success { breakpoints: HashMap<u32, usize> }
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

#[derive(Serialize)]
pub struct DisassembleResult {
    error: Option<String>,

    lines: Vec<String>,
    breakpoints: HashMap<u32, usize>
}

#[derive(Clone, Serialize)]
struct PrintPayload<'a> {
    text: &'a str,
    error: bool
}

fn forward_print(app_handle: tauri::AppHandle<Wry>) -> Box<dyn FnMut(&str, bool) -> () + Send> {
    Box::new(move |text: &str, error: bool| {
        app_handle.emit_all("print", PrintPayload { text, error }).ok();
    })
}

#[tauri::command]
pub fn assemble(text: &str) -> AssemblerResult {
    let result = assemble_from(text);

    AssemblerResult::from_result(result, text)
}

#[tauri::command]
pub fn disassemble(named: Option<&str>, bytes: Vec<u8>) -> DisassembleResult {
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

#[tauri::command]
pub fn assemble_binary(text: &str) -> (Option<Vec<u8>>, AssemblerResult) {
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

#[tauri::command]
pub fn configure_elf(
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
pub fn configure_asm(
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
