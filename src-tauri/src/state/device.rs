use crate::keyboard::KeyboardState;
use crate::syscall::SyscallState;
use std::sync::{Arc, Mutex};
use titan::cpu::Memory;
use titan::debug::Debugger;
use titan::debug::trackers::Tracker;

pub struct ExecutionState<Mem: Memory, Track: Tracker<Mem>> {
    pub debugger: Arc<Debugger<Mem, Track>>,
    pub keyboard: Arc<Mutex<KeyboardState>>,
    pub delegate: Arc<Mutex<SyscallState>>,
    pub finished_pcs: Vec<u32>,
}
