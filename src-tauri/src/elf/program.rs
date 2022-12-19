use std::io::{Read, Seek};
use std::io::SeekFrom::Start;
use anyhow::{anyhow, Result};
use bitflags::bitflags;
use num_derive::{ToPrimitive, FromPrimitive};
use num_traits::{FromPrimitive};
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(ToPrimitive, FromPrimitive, Debug)]
pub enum ProgramHeaderType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interpreter = 3,
    Note = 4,
    ProgramHeader = 6,
}

bitflags! {
    pub struct ProgramHeaderFlags: u32 {
        const EXECUTABLE = 1 << 0;
        const WRITABLE = 1 << 1;
        const READABLE = 1 << 2;
    }
}

impl ProgramHeaderFlags {
    pub fn known_mask() -> u32 { 0x111 }
}

#[derive(Debug)]
pub struct ProgramHeader {
    pub header_type: ProgramHeaderType,
    pub virtual_address: u32,
    pub padding: u32,
    pub memory_size: u32,
    pub flags: ProgramHeaderFlags,
    pub alignment: u32,
    pub data: Vec<u8>
}

impl ProgramHeader {
    pub fn read<T>(stream: &mut T) -> Result<ProgramHeader> where T: Read + Seek {
        type Endian = LittleEndian;

        let raw_header_type = stream.read_u32::<Endian>()?;
        let header_type = FromPrimitive::from_u32(raw_header_type)
            .ok_or_else(|| anyhow!("Invalid header type {}.", raw_header_type))?;

        let file_offset = stream.read_u32::<Endian>()?;
        let virtual_address = stream.read_u32::<Endian>()?;
        let padding = stream.read_u32::<Endian>()?;
        let file_size = stream.read_u32::<Endian>()?;
        let memory_size = stream.read_u32::<Endian>()?;
        let flags = stream.read_u32::<Endian>()?;
        let alignment = stream.read_u32::<Endian>()?;

        let mut data = vec![0; file_size as usize];
        stream.seek(Start(file_offset as u64))?;
        stream.read_exact(&mut data)?;

        let flags = ProgramHeaderFlags::from_bits(
            flags & ProgramHeaderFlags::known_mask())
            .unwrap_or(ProgramHeaderFlags::empty());

        Ok(ProgramHeader {
            header_type,
            virtual_address,
            padding,
            memory_size,
            flags,
            alignment,
            data,
        })
    }
}