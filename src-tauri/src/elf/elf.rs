use std::io::{Read, Seek};
use std::io::SeekFrom::Start;
use anyhow::Result;
use num::range;
use crate::elf::Header;
use crate::elf::program::ProgramHeader;

#[derive(Debug)]
pub struct Elf {
    pub header: Header,
    pub program_headers: Vec<ProgramHeader>
}

impl Elf {
    pub fn read<T>(stream: &mut T) -> Result<Elf> where T: Read + Seek {
        let (header, details) = Header::read(stream)?;

        let mut start_index = details.program_table_position as u64;
        let mut program_headers: Vec<ProgramHeader> = vec![];

        for _ in range(0, details.program_entry_count) {
            stream.seek(Start(start_index))?;

            if let Ok(header) = ProgramHeader::read(stream) {
                program_headers.push(header)
            }

            start_index += details.program_entry_size as u64;
        }

        Ok(Elf { header, program_headers })
    }
}
