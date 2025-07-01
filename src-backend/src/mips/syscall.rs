use titan::cpu::Memory;
use titan::execution::Executor;
use titan::execution::trackers::Tracker;
use titan::mips::cpu::{Registers, State};
use titan::mips::cpu::registers::WhichRegister::Fp;
use titan::mips::unit::register::RegisterName;
use crate::syscall_access::{SyscallAccess, SyscallFloatRegister, SyscallRegister};


fn fp_raw(registers: &impl Registers, index: u8) -> u32 {
    registers.get(Fp(index))
}

fn fp(registers: &impl Registers, index: u8) -> f32 {
    f32::from_bits(fp_raw(registers, index))
}

fn fp_double(registers: &impl Registers, index: u8) -> f64 {
    f64::from_bits(fp_raw(registers, index) as u64 | ((fp_raw(registers, index + 1) as u64) << 32))
}

fn set_fp(registers: &mut impl Registers, index: u8, value: f32) {
    registers.set(Fp(index), value.to_bits())
}

fn set_fp_double(registers: &mut impl Registers, index: u8, value: f64) {
    let value = value.to_bits();
    
    let lower = (value & 0xFFFFFFFF) as u32;
    let upper = (value >> 32) as u32;
    
    registers.set(Fp(index), lower);
    registers.set(Fp(index + 1), upper);
}

impl<Reg: Registers, Mem: Memory, Track: Tracker<State<Mem, Reg>>> SyscallAccess for Executor<State<Mem, Reg>, Track> {
    type Mem = Mem;

    fn get_register(&self, register: SyscallRegister) -> u32 {
        self.with_state(|state| match register {
            SyscallRegister::SyscallNumber => state.registers.get_l(RegisterName::Value0),
            SyscallRegister::SyscallResult => state.registers.get_l(RegisterName::Value0),
            SyscallRegister::Parameter0 => state.registers.get_l(RegisterName::Parameter0),
            SyscallRegister::Parameter1 => state.registers.get_l(RegisterName::Parameter1),
            SyscallRegister::Parameter2 => state.registers.get_l(RegisterName::Parameter2),
            SyscallRegister::Parameter3 => state.registers.get_l(RegisterName::Parameter3),
        })
    }

    fn set_register(&self, register: SyscallRegister, value: u32) {
        self.with_state(|state| match register {
            SyscallRegister::SyscallNumber => state.registers.set_l(RegisterName::Value0, value),
            SyscallRegister::SyscallResult => state.registers.set_l(RegisterName::Value0, value),
            SyscallRegister::Parameter0 => state.registers.set_l(RegisterName::Parameter0, value),
            SyscallRegister::Parameter1 => state.registers.set_l(RegisterName::Parameter1, value),
            SyscallRegister::Parameter2 => state.registers.set_l(RegisterName::Parameter2, value),
            SyscallRegister::Parameter3 => state.registers.set_l(RegisterName::Parameter3, value),
        })
    }

    fn get_float_register(&self, register: SyscallFloatRegister) -> Option<f32> {
        Some(self.with_state(|state| fp(&state.registers, register.index())))
    }

    fn get_double_register(&self, register: SyscallFloatRegister) -> Option<f64> {
        Some(self.with_state(|state| fp_double(&state.registers, register.index())))
    }

    fn set_float_register(&self, register: SyscallFloatRegister, value: f32) -> bool {
        self.with_state(|state| set_fp(&mut state.registers, register.index(), value));
        
        true
    }

    fn set_double_register(&self, register: SyscallFloatRegister, value: f64) -> bool {
        self.with_state(|state| set_fp_double(&mut state.registers, register.index(), value));

        true
    }

    fn with_memory<T, F: FnOnce(&mut Self::Mem) -> T>(&self, f: F) -> T {
        self.with_memory(|memory| f(memory))
    }
}
