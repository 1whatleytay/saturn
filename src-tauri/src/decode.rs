use saturn_backend::decode::{InspectionItem, InstructionDetails};
use saturn_backend::platforms::Platform::{self, Mips};

#[tauri::command]
pub fn decode_instruction(pc: u32, instruction: u32, platform: Platform) -> Option<InstructionDetails> {
    saturn_backend::decode::decode_instruction(pc, instruction, platform)
}

#[tauri::command]
pub fn detailed_disassemble(bytes: Vec<u8>, platform: Platform) -> Result<Vec<InspectionItem>, String> {
    saturn_backend::decode::detailed_disassemble(bytes, platform)
}
