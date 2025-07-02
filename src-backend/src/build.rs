use serde::Serialize;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use titan::assembler::binary::{Binary, RegionFlags};
use titan::assembler::lexer::Location;
use titan::assembler::line_details::LineDetails;
use titan::elf::program::ProgramHeaderFlags;
use titan::elf::Elf;
use titan::execution::elf::inspection::Inspection;
use titan::mips::cpu::disassemble::MipsInspectionDisassembler;
use titan::riscv::cpu::disassemble::RiscVInspectionDisassembler;
use crate::platforms::Platform;

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
    elf.program_headers
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
    pub fn from_result(
        result: Result<Binary, (Option<Location>, String)>,
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
            Err((details, message)) => {
                let details = details
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
                        message,
                        body,
                    },
                )
            }
        }
    }
    
    pub fn from_mips_result(
        result: Result<Binary, titan::mips::assembler::string::SourceError>,
        source: &str,
    ) -> (Option<Binary>, AssemblerResult) {
        let result = result.map_err(|err| (err.start(), err.to_string()));
        
        AssemblerResult::from_result(result, source)
    }

    pub fn from_risc_v_result(
        result: Result<Binary, titan::riscv::assembler::string::SourceError>,
        source: &str,
    ) -> (Option<Binary>, AssemblerResult) {
        let result = result.map_err(|err| (err.start(), err.to_string()));

        AssemblerResult::from_result(result, source)   
    }
}

#[derive(Serialize)]
pub struct DisassembleResult {
    platform: Option<Platform>,
    
    error: Option<String>,

    lines: Vec<String>,
    breakpoints: HashMap<u32, usize>,
}

#[derive(Clone, Serialize)]
pub struct PrintPayload<'a> {
    pub text: &'a str,
    pub error: bool,
}

pub fn assemble_text(text: &str, path: Option<&str>, platform: Platform) -> (Option<Binary>, AssemblerResult) {
    match platform {
        Platform::Mips => {
            let result = if let Some(path) = path {
                titan::mips::assembler::string::assemble_from_path(text.to_string(), PathBuf::from(path))
            } else {
                titan::mips::assembler::string::assemble_from(text)
            };
            
            AssemblerResult::from_mips_result(result, text)
        }
        Platform::RiscV => {
            let result = if let Some(path) = path {
                titan::riscv::assembler::string::assemble_from_path(text.to_string(), PathBuf::from(path))
            } else {
                titan::riscv::assembler::string::assemble_from(text)
            };

            AssemblerResult::from_risc_v_result(result, text)
        }
    }
}


pub fn assemble(text: &str, path: Option<&str>, platform: Platform) -> AssemblerResult {
    assemble_text(text, path, platform).1
}

pub fn disassemble(named: Option<&str>, bytes: Vec<u8>) -> DisassembleResult {
    let elf = match Elf::read(&mut Cursor::new(bytes)) {
        Ok(elf) => elf,
        Err(error) => {
            return DisassembleResult {
                platform: None,
                error: Some(error.to_string()),
                lines: vec![],
                breakpoints: HashMap::new(),
            }
        }
    };
    
    if let Ok(platform) = elf.header.cpu.try_into() {
        match platform {
            Platform::Mips => {
                let inspection = Inspection::new(named, &elf, &mut MipsInspectionDisassembler);

                DisassembleResult {
                    error: None,
                    platform: Some(Platform::Mips),
                    lines: inspection.lines,
                    breakpoints: inspection.breakpoints,
                }
            }
            Platform::RiscV => {
                let inspection = Inspection::new(named, &elf, &mut RiscVInspectionDisassembler);

                DisassembleResult {
                    error: None,
                    platform: Some(Platform::RiscV),
                    lines: inspection.lines,
                    breakpoints: inspection.breakpoints,
                }
            }
        }
    } else {
        DisassembleResult {
            platform: None,
            error: Some(format!("Unsupported ELF disassembly of {:?} ISA", elf.header.cpu)),
            lines: vec![],
            breakpoints: HashMap::new(),
        }
    }
}

pub fn assemble_binary(text: &str, path: Option<&str>, platform: Platform) -> (Option<Vec<u8>>, AssemblerResult) {
    let (binary, result) = assemble_text(text, path, platform);

    let Some(binary) = binary else {
        return (None, result);
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
