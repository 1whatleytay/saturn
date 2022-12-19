use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub struct Region {
    pub start: u32,
    pub data: Vec<u8>
}

pub struct Memory {
    regions: Vec<Region>
}

impl Region {
    pub fn contains(&self, address: u32) -> bool {
        self.start <= address && address < self.start + self.data.len() as u32
    }
}

const fn extend_u16(byte: u8) -> u16 {
    let byte = byte as u16;

    byte << 8 | byte
}

const fn extend_u32(byte: u8) -> u32 {
    let byte = byte as u32;

    byte << 24 | byte << 16 | byte << 8 | byte
}

type Endian = LittleEndian;
const INVALID_READ: u8 = 0xCC;

const INVALID_READ_16: u16 = extend_u16(INVALID_READ);
const INVALID_READ_32: u32 = extend_u32(INVALID_READ);

impl Memory {
    pub fn get(&self, address: u32) -> u8 {
        for region in &self.regions {
            if region.contains(address) {
                return region.data[(address - region.start) as usize]
            }
        }

        INVALID_READ
    }

    pub fn get_u16(&self, address: u32) -> u16 {
        if address % 2 != 0 {
            panic!("Address 0x{:08x} is not aligned for u16 read.", address);
        }

        for region in &self.regions {
            if region.contains(address) {
                let start = (address - region.start) as usize;

                return (&region.data[start .. start + 2])
                    .read_u16::<Endian>()
                    .unwrap_or(INVALID_READ_16)
            }
        }

        INVALID_READ_16
    }

    pub fn get_u32(&self, address: u32) -> u32 {
        if address % 4 != 0 {
            panic!("Address 0x{:08x} is not aligned for u32 read.", address);
        }

        for region in &self.regions {
            if region.contains(address) {
                let start = (address - region.start) as usize;

                return (&region.data[start .. start + 4])
                    .read_u32::<Endian>()
                    .unwrap_or(INVALID_READ_32)
            }
        }

        INVALID_READ_32
    }

    pub fn set(&mut self, address: u32, value: u8) {
        for region in &mut self.regions {
            if region.contains(address) {
                region.data[(address - region.start) as usize] = value;

                return
            }
        }

        panic!()
    }

    pub fn set_u16(&mut self, address: u32, value: u16) {
        if address % 2 != 0 {
            panic!("Address 0x{:08x} is not aligned for u16 read.", address);
        }

        for region in &mut self.regions {
            if region.contains(address) {
                let start = (address - region.start) as usize;

                (&mut region.data[start .. start + 2])
                    .write_u16::<Endian>(value)
                    .unwrap();

                return
            }
        }

        panic!()
    }

    pub fn set_u32(&mut self, address: u32, value: u32) {
        if address % 4 != 0 {
            panic!("Address 0x{:08x} is not aligned for u32 read.", address);
        }

        for region in &mut self.regions {
            if region.contains(address) {
                let start = (address - region.start) as usize;

                (&mut region.data[start .. start + 4])
                    .write_u32::<Endian>(value)
                    .unwrap();

                return
            }
        }

        panic!()
    }

    pub fn mount(&mut self, region: Region) {
        self.regions.push(region)
    }

    pub fn new() -> Memory {
        return Memory {
            regions: vec![]
        }
    }
}
