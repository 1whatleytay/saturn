use crate::keyboard::KeyboardState;
use crate::syscall::SyscallState;
use std::sync::{Arc, Mutex};
use titan::assembler::binary::Binary;
use titan::cpu::{Memory, State};
use titan::cpu::memory::{Mountable, Region};
use titan::execution::Executor;
use titan::execution::trackers::Tracker;

pub struct ExecutionState<Mem: Memory, Track: Tracker<Mem>> {
    pub debugger: Arc<Executor<Mem, Track>>,
    pub keyboard: Arc<Mutex<KeyboardState>>,
    pub delegate: Arc<Mutex<SyscallState>>,
    pub finished_pcs: Vec<u32>,
}

pub fn state_from_binary<Mem: Memory + Mountable>(binary: Binary, heap_size: u32, mut memory: Mem) -> State<Mem> {
    for region in binary.regions {
        let region = Region {
            start: region.address,
            data: region.data,
        };

        memory.mount(region);
    }

    // Keeping this around temporarily.
    let heap_end = 0x80000000u32;

    let heap = Region {
        start: heap_end - heap_size,
        data: vec![0; heap_size as usize],
    };

    memory.mount(heap);

    let mut state = State::new(binary.entry, memory);

    state.registers.line[29] = heap_end - 4; // give some space

    state
}

pub fn setup_state<Mem: Memory + Mountable>(state: &mut State<Mem>) {
    let max_screen = 0x8000;
    let screen = Region {
        start: 0x10008000,
        data: vec![0; max_screen],
    };

    state.memory.mount(screen);

    state.registers.line[28] = 0x10008000
}
