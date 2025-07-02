use serde::Serialize;
use titan::elf::header::InstructionSet;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Platform {
    Mips,
    RiscV,
}

impl TryFrom<InstructionSet> for Platform {
    type Error = ();

    fn try_from(value: InstructionSet) -> Result<Self, Self::Error> {
        match value {
            InstructionSet::Mips => Ok(Platform::Mips),
            InstructionSet::RiscV => Ok(Platform::RiscV),
            _ => Err(()),
        }
    }
}
