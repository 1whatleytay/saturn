#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};

use anyhow::{anyhow, Result};
use tauri::api::shell::Program;
use crate::cpu::decoder::Decoder;
use crate::cpu::disassemble::Disassembler;
use crate::elf::program::{ProgramHeader, ProgramHeaderFlags};

mod cpu;
mod elf;

fn select_executable(entry: u32, headers: &[ProgramHeader]) -> Option<&ProgramHeader> {
    let executables: Vec<&ProgramHeader> = headers.iter()
        .filter(|header| header.flags.contains(ProgramHeaderFlags::EXECUTABLE))
        .collect();

    executables.iter()
        .find(|header| header.virtual_address == entry)
        .or_else(|| executables.first())
        .map(|x| x.clone())
}

fn disassemble_raw(bytes: Vec<u8>) -> Result<Vec<String>> {
    let elf = elf::Elf::read(&mut Cursor::new(bytes))?;
    let entry = elf.header.program_entry;

    let Some(header) = select_executable(entry, &elf.program_headers) else {
        return Err(anyhow!("No executable header (with elf entry: {:08x})", entry))
    };

    let mut instructions = Cursor::new(&header.data);
    let mut disassembler = Disassembler { };

    let mut result = vec![];

    // Can't think of a way to stream this.
    while let Ok(instruction) = instructions.read_u32::<LittleEndian>() {
        result.push(disassembler.dispatch(instruction)
            .unwrap_or_else(|| "INVALID".to_string()))
    }

    Ok(result)
}

#[tauri::command]
fn disassemble(bytes: Vec<u8>) -> Vec<String> {
    disassemble_raw(bytes).unwrap_or_else(|err| vec![err.to_string()])
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, disassemble])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
