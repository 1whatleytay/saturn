use std::io::Cursor;
use wasm_bindgen::prelude::*;
use saturn_backend::build::assemble_text;

#[wasm_bindgen]
pub fn build(text: &str) -> Option<Vec<u8>> {
    let mut data = vec![];
    
    assemble_text(text, None).ok()?.create_elf().write(&mut Cursor::new(&mut data)).ok()?;

    Some(data)
}
