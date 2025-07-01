use std::collections::HashSet;
use async_trait::async_trait;
use serde::Serialize;
use titan::execution::ExecutorMode;
use crate::display::FlushDisplayBody;
use crate::mips::execution::MipsRegistersResult;
use crate::riscv::execution::RiscVRegistersResult;

#[derive(Serialize)]
#[serde(tag = "platform")]
pub enum PlatformRegisters {
    Mips(MipsRegistersResult),
    RiscV(RiscVRegistersResult),
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ResumeMode {
    Running,
    Invalid { message: String },
    Paused,
    Breakpoint,
    Finished { pc: u32, code: Option<u32> },
}

#[derive(Serialize)]
pub struct ResumeResult {
    pub mode: ResumeMode,
    pub registers: PlatformRegisters,
}

#[derive(Debug, Clone)]
pub struct BatchOptions {
    pub count: usize,
    // if true, the cancelled flag for syscall delegates will be cleared
    pub first_batch: bool,
    // if false, mode (pausing) will not stop execution
    // handy if we want to start in the breakpoint mode (and we only want to run 1 instruction anyway)
    pub allow_interrupt: bool,
    // If this variable is true, the mode will be forced to breakpoint at the end of the batch.
    // The mode will only be changed if the mode is Running at the end of the batch.
    // Useful for stepping over syscalls, since syscalls will set the mode to Running when finished.
    pub break_at_end: bool,
}

pub struct ResumeOptions {
    pub batch: Option<BatchOptions>,
    pub breakpoints: Option<Vec<u32>>,
    pub display: Option<FlushDisplayBody>,
    // if set_running is true, set state to "Running" and clear cancellation
    // useful for looping batches, like in the WASM backend
    pub change_state: Option<ExecutorMode>,
}

#[derive(Copy, Clone, Debug)]
pub enum ReadDisplayTarget {
    Address(u32),
    DefaultRegister, // $gp on MIPS and RISC-V
}

impl ReadDisplayTarget {
    pub fn from_arguments(use_default_register: bool, address: u32) -> Self {
        if use_default_register {
            ReadDisplayTarget::DefaultRegister
        } else {
            ReadDisplayTarget::Address(address)
        }
    }
}

#[async_trait]
pub trait ExecutionDevice: Send + Sync {
    async fn resume(&self, options: ResumeOptions) -> Result<ResumeResult, ()>;

    fn pause(&self);

    fn set_breakpoints(&self, breakpoints: HashSet<u32>);

    fn read_bytes(&self, address: u32, count: u32) -> Option<Vec<Option<u8>>>;
    fn read_display(&self, target: ReadDisplayTarget, width: u32, height: u32) -> Option<Vec<u8>>;

    fn write_bytes(&self, address: u32, bytes: Vec<u8>);
    fn write_register(&self, register: u32, value: u32);

    fn wake_sync(&self);
    fn post_key(&self, key: char, up: bool);
    fn post_input(&self, text: String);
}

pub trait ExecutionRewindable {
    fn last_pc(&self) -> Option<u32>;
    fn rewind(&self, count: u32) -> ResumeResult;
}

pub trait RewindableDevice: ExecutionDevice + ExecutionRewindable {}

impl<T: ExecutionDevice + ExecutionRewindable> RewindableDevice for T { }
