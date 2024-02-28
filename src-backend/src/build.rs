use serde::Serialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use titan::assembler::binary::{Binary, RegionFlags};
use titan::assembler::line_details::LineDetails;
use titan::assembler::string::{assemble_from, assemble_from_path, SourceError};
use titan::cpu::memory::{Mountable, Region};
use titan::cpu::memory::section::SectionMemory;
use titan::cpu::{Memory, State};
use titan::execution::elf::inspection::Inspection;
use titan::elf::Elf;
use titan::elf::program::ProgramHeaderFlags;
use crate::keyboard::{KeyboardHandler, KeyboardState, KEYBOARD_SELECTOR};

pub const TIME_TRAVEL_HISTORY_SIZE: usize = 1000;

#[derive(Serialize)]
pub struct LineMarker {
    line: usize,
    offset: usize,
}

#[derive(Serialize)]
pub struct Breakpoint {
    line: usize,
    pcs: Vec<u32>,
}

#[derive(Serialize)]
#[serde(tag = "status")]
pub enum AssemblerResult {
    Error {
        marker: Option<LineMarker>,
        message: String,
        body: Option<String>,
    },
    Success {
        breakpoints: Vec<Breakpoint>,
    },
}

pub fn get_elf_finished_pcs(elf: &Elf) -> Vec<u32> {
    elf
        .program_headers
        .iter()
        .filter(|header| header.flags.contains(ProgramHeaderFlags::EXECUTABLE))
        .map(|header| header.virtual_address + header.data.len() as u32)
        .collect()
}

pub fn get_binary_finished_pcs(binary: &Binary) -> Vec<u32> {
    binary
        .regions
        .iter()
        .filter(|region| region.flags.contains(RegionFlags::EXECUTABLE))
        .map(|region| region.address + region.data.len() as u32)
        .collect()
}

impl AssemblerResult {
    pub fn from_result_with_binary(
        result: Result<Binary, SourceError>,
        source: &str,
    ) -> (Option<Binary>, AssemblerResult) {
        match result {
            Ok(binary) => {
                let breakpoints = binary
                    .source_breakpoints(source, 0)
                    .into_iter()
                    .map(|b| Breakpoint {
                        line: b.line,
                        pcs: b.pcs,
                    })
                    .collect();

                (Some(binary), AssemblerResult::Success { breakpoints })
            }
            Err(error) => {
                let details = error
                    .start()
                    .map(|location| LineDetails::from_offset(source, location.index));

                let marker = details.as_ref().map(|details| LineMarker {
                    line: details.line_number,
                    offset: details.line_offset,
                });

                let body = details
                    .as_ref()
                    .map(|details| format!("{}\n{}", details.line_text, details.marker()));

                (
                    None,
                    AssemblerResult::Error {
                        marker,
                        message: format!("{}", error),
                        body,
                    },
                )
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
    breakpoints: HashMap<u32, usize>,
}

#[derive(Clone, Serialize)]
pub struct PrintPayload<'a> {
    pub text: &'a str,
    pub error: bool,
}

pub fn assemble_text(text: &str, path: Option<&str>) -> Result<Binary, SourceError> {
    if let Some(path) = path {
        assemble_from_path(text.to_string(), PathBuf::from(path))
    } else {
        assemble_from(text)
    }
}


pub fn create_elf_state<Mem: Memory + Mountable>(
    elf: &Elf,
    heap_size: u32,
    mut memory: Mem
) -> State<Mem> {
    for header in &elf.program_headers {
        let region = Region {
            start: header.virtual_address,
            data: header.data.clone(),
        };

        memory.mount(region)
    }

    let heap_end = 0x7FFFFFFCu32;

    let heap = Region {
        start: heap_end - heap_size,
        data: vec![0; heap_size as usize],
    };

    memory.mount(heap);

    let mut state = State::new(elf.header.program_entry, memory);
    state.registers.line[29] = heap_end;

    state
}

pub fn configure_keyboard(memory: &mut SectionMemory<KeyboardHandler>) -> Arc<Mutex<KeyboardState>> {
    let handler = KeyboardHandler::new();
    let keyboard = handler.state.clone();

    memory.mount_listen(KEYBOARD_SELECTOR as usize, handler);

    // Mark heap as "Writable"
    for selector in 0x1000..0x8000 {
        memory.mount_writable(selector, 0xCC);
    }

    keyboard
}

pub fn assemble(text: &str, path: Option<&str>) -> AssemblerResult {
    let result = assemble_text(text, path);

    AssemblerResult::from_result(result, text)
}

pub fn disassemble(named: Option<&str>, bytes: Vec<u8>) -> DisassembleResult {
    let elf = match Elf::read(&mut Cursor::new(bytes)) {
        Ok(elf) => elf,
        Err(error) => {
            return DisassembleResult {
                error: Some(error.to_string()),
                lines: vec![],
                breakpoints: HashMap::new(),
            }
        }
    };

    let inspection = Inspection::new(named, &elf);

    DisassembleResult {
        error: None,
        lines: inspection.lines,
        breakpoints: inspection.breakpoints,
    }
}

pub fn assemble_binary(text: &str, path: Option<&str>) -> (Option<Vec<u8>>, AssemblerResult) {
    let result = assemble_text(text, path);
    let (binary, result) = AssemblerResult::from_result_with_binary(result, text);

    let Some(binary) = binary else {
        return (None, result)
    };

    let elf: Elf = binary.create_elf();

    let mut out: Vec<u8> = vec![];
    let mut cursor = Cursor::new(&mut out);

    if let Err(error) = elf.write(&mut cursor) {
        return (
            None,
            AssemblerResult::Error {
                marker: None,
                message: error.to_string(),
                body: None,
            },
        );
    }

    (Some(out), result)
}
