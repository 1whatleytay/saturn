use crate::keyboard::KeyboardState;
use crate::syscall::SyscallState;
use std::sync::{Arc, Mutex};
use titan::assembler::binary::Binary;
use titan::cpu::memory::{Mountable, Region};
use titan::execution::Executor;
use titan::execution::trackers::Tracker;
use titan::mips::assembler::registers::RegisterSlot::{GeneralPointer, StackPointer};
use titan::mips::cpu::registers::WhichRegister::Pc;
use titan::mips::cpu::{Memory, Registers, State};

pub struct ExecutionState<Mem: Memory, Reg: Registers, Track: Tracker<State<Mem, Reg>>> {
    pub debugger: Arc<Executor<State<Mem, Reg>, Track>>,
    pub keyboard: Arc<Mutex<KeyboardState>>,
    pub delegate: Arc<Mutex<SyscallState>>,
    pub finished_pcs: Vec<u32>,
}

pub fn state_from_binary<Mem: Memory + Mountable, Reg: Registers>(
    binary: Binary,
    heap_size: u32,
    mut memory: Mem,
    mut registers: Reg,
) -> State<Mem, Reg> {
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

    registers.set_l(StackPointer, heap_end - 4); // give some space
    registers.set(Pc, binary.entry);

    State::new(registers, memory)
}

pub fn setup_state<Mem: Memory + Mountable, Reg: Registers>(state: &mut State<Mem, Reg>) {
    let max_screen = 0x8000;
    let screen = Region {
        start: 0x10008000,
        data: vec![0; max_screen],
    };

    state.memory.mount(screen);

    state.registers.set_l(GeneralPointer, 0x10008000);

    state.registers.clear();
}
