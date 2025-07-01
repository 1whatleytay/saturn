use titan::cpu::Memory;
use titan::execution::Executor;
use titan::execution::trackers::Tracker;
use titan::riscv::assembler::registers::RegisterSlot;
use titan::riscv::cpu::{Registers, State};
use crate::syscall_access::{SyscallAccess, SyscallFloatRegister, SyscallRegister};

impl<Reg: Registers, Mem: Memory, Track: Tracker<State<Mem, Reg>>> SyscallAccess for Executor<State<Mem, Reg>, Track> {
    type Mem = Mem;

    fn get_register(&self, register: SyscallRegister) -> u32 {
        self.with_state(|state| match register {
            SyscallRegister::SyscallNumber => state.registers.get_l(RegisterSlot::Parameter7),
            SyscallRegister::SyscallResult => state.registers.get_l(RegisterSlot::Parameter0),
            SyscallRegister::Parameter0 => state.registers.get_l(RegisterSlot::Parameter0),
            SyscallRegister::Parameter1 => state.registers.get_l(RegisterSlot::Parameter1),
            SyscallRegister::Parameter2 => state.registers.get_l(RegisterSlot::Parameter2),
            SyscallRegister::Parameter3 => state.registers.get_l(RegisterSlot::Parameter3),
        })
    }

    fn set_register(&self, register: SyscallRegister, value: u32) {
        self.with_state(|state| match register {
            SyscallRegister::SyscallNumber => state.registers.set_l(RegisterSlot::Parameter7, value),
            SyscallRegister::SyscallResult => state.registers.set_l(RegisterSlot::Parameter0, value),
            SyscallRegister::Parameter0 => state.registers.set_l(RegisterSlot::Parameter0, value),
            SyscallRegister::Parameter1 => state.registers.set_l(RegisterSlot::Parameter1, value),
            SyscallRegister::Parameter2 => state.registers.set_l(RegisterSlot::Parameter2, value),
            SyscallRegister::Parameter3 => state.registers.set_l(RegisterSlot::Parameter3, value),
        })
    }

    fn get_float_register(&self, _register: SyscallFloatRegister) -> Option<f32> {
        None
    }

    fn get_double_register(&self, _register: SyscallFloatRegister) -> Option<f64> {
        None
    }

    fn set_float_register(&self, _register: SyscallFloatRegister, _value: f32) -> bool {
        false
    }

    fn set_double_register(&self, _register: SyscallFloatRegister, _value: f64) -> bool {
        false
    }

    fn with_memory<T, F: FnOnce(&mut Self::Mem) -> T>(&self, f: F) -> T {
        self.with_memory(|memory| f(memory))
    }
}
