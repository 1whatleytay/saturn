use serde::Serialize;
use std::sync::{Arc, Mutex};
use titan::cpu::Memory;

#[derive(Clone, Serialize)]
pub struct FlushDisplayState {
    pub address: u32,
    pub width: u32,
    pub height: u32,
    pub data: Option<Vec<u8>>, // flush should impact this
}

impl Default for FlushDisplayState {
    fn default() -> FlushDisplayState {
        FlushDisplayState {
            address: 0x10008000,
            width: 64,
            height: 64,
            data: None,
        }
    }
}

impl FlushDisplayState {
    pub fn flush<Mem: Memory>(&mut self, memory: &mut Mem) {
        self.data = read_display(self.address, self.width, self.height, memory);
    }
}

pub type FlushDisplayBody = Arc<Mutex<FlushDisplayState>>;

// NOT a tauri command.
pub fn read_display<Mem: Memory>(
    address: u32,
    width: u32,
    height: u32,
    memory: &mut Mem,
) -> Option<Vec<u8>> {
    let pixels = width.checked_mul(height)?;

    let mut result = vec![0u8; (pixels * 4) as usize];

    for i in 0..pixels {
        let point = address.wrapping_add(i.wrapping_mul(4));

        let pixel = memory.get_u32(point).ok()?;

        // Assuming little endian: 0xAARRGGBB -> [BB, GG, RR, AA] -> want [RR, GG, BB, AA]
        let start = (i as usize) * 4;
        result[start] = (pixel.wrapping_shr(16) & 0xFF) as u8;
        result[start + 1] = (pixel.wrapping_shr(8) & 0xFF) as u8;
        result[start + 2] = (pixel.wrapping_shr(0) & 0xFF) as u8;
        result[start + 3] = 255;
    }

    Some(result)
}
