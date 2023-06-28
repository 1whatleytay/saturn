use crate::keyboard::{KeyboardHandler, KeyboardState};
use crate::syscall::SyscallState;
use std::sync::{Arc, Mutex};
use titan::cpu::memory::section::SectionMemory;
use titan::cpu::memory::watched::WatchedMemory;
use titan::debug::Debugger;
use titan::debug::trackers::history::HistoryTracker;

pub type MemoryType = WatchedMemory<SectionMemory<KeyboardHandler>>;

pub struct ExecutionState {
    pub debugger: Arc<Debugger<MemoryType, HistoryTracker>>,
    pub keyboard: Arc<Mutex<KeyboardState>>,
    pub delegate: Arc<Mutex<SyscallState>>,
    pub finished_pcs: Vec<u32>,
}
