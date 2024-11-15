use serde::Serialize;
use std::sync::{Arc, Mutex};
use num::FromPrimitive;
use titan::cpu::{Memory, State};
use titan::unit::register::RegisterName;
use crate::execution::ReadDisplayTarget;

#[derive(Clone, Serialize)]
pub struct FlushDisplayState {
    pub address: u32,
    pub register: Option<u8>,
    pub width: u32,
    pub height: u32,
    pub data: Option<Vec<u8>>, // flush should impact this
}

impl Default for FlushDisplayState {
    fn default() -> FlushDisplayState {
        FlushDisplayState {
            address: 0x10008000,
            register: None,
            width: 64,
            height: 64,
            data: None,
        }
    }
}

impl FlushDisplayState {
    fn get_target(&self) -> ReadDisplayTarget {
        if let Some(register) = self.register
            .and_then(|register| RegisterName::from_u8(register)) {
            ReadDisplayTarget::Register(register)
        } else {
            ReadDisplayTarget::Address(self.address)
        }
    }
    
    pub fn flush<Mem: Memory>(&mut self, state: &mut State<Mem>) {
        let address = self.get_target().to_address(&state.registers);
        
        self.data = read_display(address, self.width, self.height, &mut state.memory);
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
