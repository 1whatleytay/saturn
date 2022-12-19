use std::io::Read;
use anyhow::{anyhow, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
pub enum BinaryType {
    Binary32 = 1,
    Binary64 = 2
}

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
pub enum Endian {
    Little = 1,
    Big = 2
}

#[derive(FromPrimitive, ToPrimitive, PartialEq, Debug)]
pub enum InstructionSet {
    Generic = 0x00,
    Sparc = 0x02,
    X86 = 0x03,
    Mips = 0x08,
    PowerPC = 0x14,
    Arm = 0x28,
    SuperH = 0x2A,
    IA64 = 0x32,
    X64 = 0x3E,
    AArch64 = 0xB7,
    RiscV = 0xF3
}

#[derive(Debug)]
pub struct Header {
    pub magic: u32,
    pub binary_type: BinaryType,
    pub endian: Endian,
    pub header_version: u8,
    pub abi: u8,
    pub padding: [u8; 8],
    pub package: u16,
    pub cpu: InstructionSet,
    pub elf_version: u32,
    pub program_entry: u32
}

#[derive(Debug)]
pub struct HeaderDetails {
    pub program_table_position: u32,
    pub section_table_point: u32,
    pub flags: u32,
    pub header_size: u16,
    pub program_entry_size: u16,
    pub program_entry_count: u16,
    pub section_entry_size: u16,
    pub section_entry_count: u16,
    pub names_point: u16,
}

const MAGIC: u32 = 0x464c457f;

impl Header {
    pub fn read<T>(stream: &mut T) -> Result<(Header, HeaderDetails)> where T: Read {
        type Endian = LittleEndian;

        let header = Header {
            magic: stream.read_u32::<Endian>()?,
            binary_type: FromPrimitive::from_u8(stream.read_u8()?)
                .ok_or_else(|| anyhow!("Invalid binary type."))?,
            endian: FromPrimitive::from_u8(stream.read_u8()?)
                .ok_or_else(|| anyhow!("Invalid endian value."))?,
            header_version: stream.read_u8()?,
            abi: stream.read_u8()?,
            padding: {
                let mut buffer = [0; 8];

                stream.read_exact(&mut buffer)?;
                buffer
            },
            package: stream.read_u16::<Endian>()?,
            cpu: FromPrimitive::from_u16(stream.read_u16::<Endian>()?)
                .ok_or_else(|| anyhow!("Invalid version type."))?,
            elf_version: stream.read_u32::<Endian>()?,
            program_entry: stream.read_u32::<Endian>()?,
        };

        if header.magic != MAGIC {
            Err(anyhow!(
                "ELF Magic is invalid, found 0x{:08x} but expected 0x{:08x}.",
                header.magic, MAGIC
            ))
        } else if header.binary_type != BinaryType::Binary32 {
            Err(anyhow!(
                "32 Binary ELF is required, instead found {}",
                header.binary_type.to_u16().unwrap()
            ))
        } else {
            Ok((header, HeaderDetails::read(stream)?))
        }
    }
}

impl HeaderDetails {
    pub fn read<T>(stream: &mut T) -> Result<HeaderDetails> where T: Read {
        type Endian = LittleEndian;

        let details = HeaderDetails {
            program_table_position: stream.read_u32::<Endian>()?,
            section_table_point: stream.read_u32::<Endian>()?,
            flags: stream.read_u32::<Endian>()?,
            header_size: stream.read_u16::<Endian>()?,
            program_entry_size: stream.read_u16::<Endian>()?,
            program_entry_count: stream.read_u16::<Endian>()?,
            section_entry_size: stream.read_u16::<Endian>()?,
            section_entry_count: stream.read_u16::<Endian>()?,
            names_point: stream.read_u16::<Endian>()?,
        };

        Ok(details)
    }
}