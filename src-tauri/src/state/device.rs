use crate::keyboard::KeyboardState;
use crate::syscall::SyscallState;
use std::sync::{Arc, Mutex};
use titan::cpu::Memory;
use titan::execution::Executor;
use titan::execution::trackers::Tracker;

pub struct ExecutionState<Mem: Memory, Track: Tracker<Mem>> {
    pub debugger: Arc<Executor<Mem, Track>>,
    pub keyboard: Arc<Mutex<KeyboardState>>,
    pub delegate: Arc<Mutex<SyscallState>>,
    pub finished_pcs: Vec<u32>,
}
