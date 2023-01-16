use std::sync::{Arc, Mutex};
use titan::cpu::error;
use titan::cpu::error::Error::MemoryUnmapped;
use titan::cpu::memory::section::ListenResponder;

pub const KEYBOARD_MOUNT_POINT: u32 = 0xFFFF0000;

pub struct KeyboardState {
    last: Option<char>,
    keys: Vec<char>
}

pub struct KeyboardHandler {
    pub state: Arc<Mutex<KeyboardState>>
}

impl KeyboardState {
    pub fn push_key(&mut self, key: char) {
        self.keys.push(key)
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
            keys: vec![]
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
        let keyboard_handled = KEYBOARD_MOUNT_POINT .. KEYBOARD_MOUNT_POINT + 8;

        if !keyboard_handled.contains(&address) {
            return Err(MemoryUnmapped(address))
        }

        let offset = address - KEYBOARD_MOUNT_POINT;

        let mut state = self.state.lock().unwrap();

        Ok(match offset {
            0 => if state.keys.is_empty() { 0 } else { 1 },
            4 => state.pop_key().map(|c| c as u8).unwrap_or(0),
            _ => 0
        })
    }

    fn write(&mut self, address: u32, _: u8) -> error::Result<()> {
        Err(MemoryUnmapped(address))
    }
}
