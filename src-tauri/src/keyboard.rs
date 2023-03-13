use std::sync::{Arc, Mutex};
use titan::cpu::error;
use titan::cpu::error::Error::MemoryUnmapped;
use titan::cpu::memory::section::ListenResponder;

pub const KEYBOARD_ADDRESS: u32 = 0xFFFF0000;
pub const KEYBOARD_HOLDING: u32 = 0xFFFF0080;
pub const KEYBOARD_SELECTOR: u32 = KEYBOARD_ADDRESS >> 16;

pub struct KeyboardState {
    last: Option<char>,
    keys: Vec<char>,
    holding: [bool; 128]
}

pub struct KeyboardHandler {
    pub state: Arc<Mutex<KeyboardState>>
}

impl KeyboardState {
    pub fn push_key(&mut self, key: char, up: bool) {
        if !up {
            self.keys.push(key)
        }

        let offset = key as usize;

        if offset < self.holding.len() {
            self.holding[offset] = !up
        }
    }

    fn pop_key(&mut self) -> Option<char> {
        if let Some(value) = self.keys.pop() {
            self.last = Some(value)
        }

        return self.last
    }

    pub fn new() -> KeyboardState {
        KeyboardState {
            last: None,
            keys: vec![],
            holding: [false; 128]
        }
    }
}

impl KeyboardHandler {
    pub fn new() -> KeyboardHandler {
        KeyboardHandler {
            state: Arc::new(Mutex::new(KeyboardState::new()))
        }
    }
}

impl ListenResponder for KeyboardHandler {
    fn read(&self, address: u32) -> error::Result<u8> {
        let keyboard_handled = KEYBOARD_ADDRESS .. KEYBOARD_ADDRESS + 8;
        let keyboard_holding = KEYBOARD_HOLDING .. KEYBOARD_HOLDING + 128;

        if keyboard_handled.contains(&address) {
            let offset = address - KEYBOARD_ADDRESS;

            let mut state = self.state.lock().unwrap();

            Ok(match offset {
                0 => if state.keys.is_empty() { 0 } else { 1 },
                4 => state.pop_key().map(|c| c as u8).unwrap_or(0),
                _ => 0
            })
        } else if keyboard_holding.contains(&address) {
            let offset = address - KEYBOARD_HOLDING;

            let state = self.state.lock().unwrap();

            if let Some(item) = state.holding.get(offset as usize) {
                Ok(*item as u8)
            } else {
                Err(MemoryUnmapped(address))
            }
        } else {
            Err(MemoryUnmapped(address))
        }
    }

    fn write(&mut self, address: u32, _: u8) -> error::Result<()> {
        Err(MemoryUnmapped(address))
    }
}
