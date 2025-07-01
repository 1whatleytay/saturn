use titan::cpu::Memory;

pub enum SyscallRegister {
    SyscallNumber, // V0
    SyscallResult, // V0
    Parameter0, // A0
    Parameter1, // A1
    Parameter2, // A2
    Parameter3, // A3
}

// RISC-V does not support FP Registers at the moment.
// But we might still want to access them.
pub enum SyscallFloatRegister {
    Float0, // f0
    Float12, // f12
}

impl SyscallFloatRegister {
    pub fn index(&self) -> u8 {
        match self {
            SyscallFloatRegister::Float0 => 0,
            SyscallFloatRegister::Float12 => 12,
        }
    }
}

// Syscall Access trait to allow different cores to expose different registers to syscalls.
// But still remain compatible with the existing syscall system.
pub trait SyscallAccess {
    type Mem: Memory;
    
    fn get_register(&self, register: SyscallRegister) -> u32;
    fn set_register(&self, register: SyscallRegister, value: u32);
    
    fn get_float_register(&self, register: SyscallFloatRegister) -> Option<f32>;
    fn get_double_register(&self, register: SyscallFloatRegister) -> Option<f64>;

    // Return true on supported, false on unsupported
    fn set_float_register(&self, register: SyscallFloatRegister, value: f32) -> bool;
    fn set_double_register(&self, register: SyscallFloatRegister, value: f64) -> bool;
    
    fn with_memory<T, F: FnOnce(&mut Self::Mem) -> T>(&self, f: F) -> T;
}
