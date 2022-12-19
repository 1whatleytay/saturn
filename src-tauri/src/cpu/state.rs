use crate::cpu::Memory;

pub struct State {
    pub pc: u32,
    pub registers: [u32; 32],
    pub lo: u32,
    pub hi: u32,
    pub memory: Memory
}

impl State {
    pub fn new(entry: u32, memory: Memory) -> State {
        State {
            pc: entry,
            registers: [0; 32],
            lo: 0,
            hi: 0,
            memory
        }
    }
}
