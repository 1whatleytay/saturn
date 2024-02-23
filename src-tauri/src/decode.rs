use saturn_backend::decode::{InspectionItem, InstructionDetails};

#[tauri::command]
pub fn decode_instruction(pc: u32, instruction: u32) -> Option<InstructionDetails> {
    saturn_backend::decode::decode_instruction(pc, instruction)
}

#[tauri::command]
pub fn detailed_disassemble(bytes: Vec<u8>) -> Result<Vec<InspectionItem>, String> {
    saturn_backend::decode::detailed_disassemble(bytes)
}
