use serde::Serialize;
use titan::unit::instruction::{InstructionDecoder, InstructionParameter};
use num::ToPrimitive;

#[derive(Serialize)]
#[serde(tag="type", content="value")]
pub enum ParameterItem {
    Register(u32),
    Immediate(u16),
    Address(u32),
    Offset { offset: u16, register: u32 }
}

#[derive(Serialize)]
pub struct InstructionDetails {
    name: &'static str,
    parameters: Vec<ParameterItem>
}

#[tauri::command]
pub fn decode_instruction(pc: u32, instruction: u32) -> Option<InstructionDetails> {
    let Some(instruction) = InstructionDecoder::decode(pc, instruction) else {
        return None
    };

    Some(InstructionDetails {
        name: instruction.name(),
        parameters: instruction.parameters()
            .into_iter()
            .map(|x| {
                match x {
                    InstructionParameter::Register(name) => ParameterItem::Register(name.to_u32().unwrap()),
                    InstructionParameter::Immediate(imm) => ParameterItem::Immediate(imm),
                    InstructionParameter::Address(address) => ParameterItem::Address(address),
                    InstructionParameter::Offset(offset, register) =>
                        ParameterItem::Offset { offset, register: register.to_u32().unwrap() }
                }
            })
            .collect(),
    })
}