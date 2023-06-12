use crate::keyboard::{KeyboardHandler, KeyboardState, KEYBOARD_SELECTOR};
use crate::syscall::{ConsoleHandler, MidiHandler, SyscallState};
use std::sync::{Arc, Mutex, MutexGuard};
use titan::assembler::binary::Binary;
use titan::cpu::memory::section::SectionMemory;
use titan::cpu::memory::{Mountable, Region};
use titan::cpu::State;
use titan::debug::Debugger;

pub type MemoryType = SectionMemory<KeyboardHandler>;

pub struct ExecutionState {
    pub debugger: Arc<Debugger<MemoryType>>,
    pub keyboard: Arc<Mutex<KeyboardState>>,
    pub delegate: Arc<Mutex<SyscallState>>,
    pub finished_pcs: Vec<u32>,
}

pub type DebuggerBody = Mutex<Option<ExecutionState>>;

pub fn swap(
    mut pointer: MutexGuard<Option<ExecutionState>>,
    debugger: Debugger<MemoryType>,
    finished_pcs: Vec<u32>,
    console: Box<dyn ConsoleHandler + Send>,
    midi: Box<dyn MidiHandler + Send>,
) {
    if let Some(state) = pointer.as_ref() {
        state.debugger.pause()
    }

    let handler = KeyboardHandler::new();
    let keyboard = handler.state.clone();

    debugger.with_memory(|memory| {
        memory.mount_listen(KEYBOARD_SELECTOR as usize, handler);

        // Mark heap as "Writable"
        for selector in 0x1000..0x8000 {
            memory.mount_writable(selector, 0xCC);
        }
    });

    let wrapped = Arc::new(debugger);
    let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi)));

    // Drop should cancel the last process and kill the other thread.
    *pointer = Some(ExecutionState {
        debugger: wrapped,
        keyboard,
        delegate,
        finished_pcs,
    });
}

pub fn state_from_binary(binary: Binary, heap_size: u32) -> State<MemoryType> {
    let mut memory = SectionMemory::new();

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

pub fn setup_state(state: &mut State<MemoryType>) {
    let max_screen = 0x8000;
    let screen = Region {
        start: 0x10008000,
        data: vec![0; max_screen],
    };

    state.memory.mount(screen);

    state.registers.line[28] = 0x10008000
}
