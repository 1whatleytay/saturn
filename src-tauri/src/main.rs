#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::HashMap;
use std::io::Cursor;
use serde::{Serialize};

use titan::elf::Elf;
use titan::debug::elf::inspection::Inspection;

#[derive(Serialize)]
struct DisassembleResult {
    error: Option<String>,

    lines: Vec<String>,
    breakpoints: HashMap<usize, u32>
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![disassemble])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
