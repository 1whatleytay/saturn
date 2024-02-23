use std::io::Cursor;
use serde::Serialize;
use titan::unit::instruction::{InstructionDecoder, InstructionParameter};
use num::ToPrimitive;
use titan::elf::Elf;
use titan::execution::elf::detailed_inspection::{make_inspection_lines, InspectionLine};

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
    pc: u32,
    instruction: u32,
    name: &'static str,
    parameters: Vec<ParameterItem>
}

fn parameter_to_item(parameter: InstructionParameter) -> ParameterItem {
    match parameter {
        InstructionParameter::Register(name) => ParameterItem::Register(name.to_u32().unwrap()),
        InstructionParameter::Immediate(imm) => ParameterItem::Immediate(imm),
        InstructionParameter::Address(address) => ParameterItem::Address(address),
        InstructionParameter::Offset(offset, register) =>
            ParameterItem::Offset { offset, register: register.to_u32().unwrap() }
    }
}

pub fn decode_instruction(pc: u32, instruction: u32) -> Option<InstructionDetails> {
    let inst = InstructionDecoder::decode(pc, instruction)?;

    Some(InstructionDetails {
        pc,
        instruction,
        name: inst.name(),
        parameters: inst.parameters()
            .into_iter()
            .map(parameter_to_item)
            .collect(),
    })
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum InspectionItem {
    Instruction { details: InstructionDetails },
    Blank,
    Comment { message: String },
    Label { name: String },
}

pub fn detailed_disassemble(bytes: Vec<u8>) -> Result<Vec<InspectionItem>, String> {
    let elf = Elf::read(&mut Cursor::new(bytes)).map_err(|e| e.to_string())?;

    Ok(make_inspection_lines(&elf)
        .into_iter()
        .map(|line| match line {
            InspectionLine::Instruction(inst) => InspectionItem::Instruction {
                details: InstructionDetails {
                    pc: inst.pc,
                    instruction: inst.instruction,
                    name: inst.name,
                    parameters: inst.parameters
                        .into_iter()
                        .map(parameter_to_item)
                        .collect()
                }
            },
            InspectionLine::Blank => InspectionItem::Blank,
            InspectionLine::Comment(value) => InspectionItem::Comment { message: value },
            InspectionLine::Label(value) => InspectionItem::Label { name: value }
        })
        .collect())
}
