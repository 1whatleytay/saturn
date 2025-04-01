use crate::device::ExecutionState;
use crate::display::{read_display, FlushDisplayBody};
use crate::syscall::{SyscallDelegate, SyscallResult};
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashSet;
use titan::cpu::error::Error::{CpuTrap, MemoryAlign, MemoryUnmapped};
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::cpu::registers::WhichRegister::{Hi, Line, Lo, Pc};
use titan::cpu::registers::{RegisterEntry, WatchedRegisters};
use titan::cpu::state::Registers;
use titan::cpu::{Memory, State};
use titan::execution::executor::{DebugFrame, ExecutorMode};
use titan::execution::trackers::history::HistoryTracker;
use titan::execution::trackers::Tracker;
use titan::unit::instruction::InstructionDecoder;
use titan::unit::register::RegisterName;
use titan::unit::suggestions::MemoryErrorReason;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ResumeMode {
    Running,
    Invalid { message: String },
    Paused,
    Breakpoint,
    Finished { pc: u32, code: Option<u32> },
}

fn format_error<Mem: Memory, Reg: Registers>(
    error: titan::cpu::error::Error,
    state: &State<Mem, Reg>,
) -> String {
    let memory = |reason: MemoryErrorReason| {
        let pc = state.registers.get(Pc);

        let description = state
            .memory
            .get_u32(pc)
            .ok()
            .and_then(|value| InstructionDecoder::decode(pc, value))
            .and_then(|instruction| instruction.describe_memory_error(reason, &state.registers));

        if let Some(description) = description {
            description.to_string()
        } else {
            error.to_string()
        }
    };

    match error {
        MemoryAlign(_, _) => memory(MemoryErrorReason::Alignment),
        MemoryUnmapped(_) => memory(MemoryErrorReason::Unmapped),
        CpuTrap => {
            let pc = state.registers.get(Pc).wrapping_sub(4);

            let description = state
                .memory
                .get_u32(pc)
                .ok()
                .and_then(|value| InstructionDecoder::decode(pc, value))
                .and_then(|instruction| instruction.describe_trap_error(&state.registers));

            if let Some(description) = description {
                description.to_string()
            } else {
                error.to_string()
            }
        }
        _ => error.to_string(),
    }
}

impl ResumeMode {
    fn from_executor<Mem: Memory, Reg: Registers>(
        value: ExecutorMode,
        state: &State<Mem, Reg>,
    ) -> Self {
        match value {
            ExecutorMode::Running => ResumeMode::Running,
            ExecutorMode::Invalid(error) => ResumeMode::Invalid {
                message: format_error(error, state),
            },
            ExecutorMode::Paused => ResumeMode::Paused,
            ExecutorMode::Breakpoint => ResumeMode::Breakpoint,
        }
    }
}

#[derive(Serialize)]
pub struct RegistersResult {
    pc: u32,
    line: [u32; 32],
    lo: u32,
    hi: u32,
}

impl<T: Registers> From<T> for RegistersResult {
    fn from(value: T) -> Self {
        let raw = value.raw();
        RegistersResult {
            pc: raw.pc,
            line: raw.line,
            lo: raw.lo,
            hi: raw.hi,
        }
    }
}

#[derive(Serialize)]
pub struct ResumeResult {
    pub mode: ResumeMode,
    pub registers: RegistersResult,
}

impl ResumeResult {
    fn from_frame<Mem: Memory, Reg: Registers>(
        frame: DebugFrame,
        finished_pcs: &[u32],
        result: Option<SyscallResult>,
        state: &State<Mem, Reg>,
    ) -> ResumeResult {
        let mode = match result {
            Some(SyscallResult::Failure(message)) => ResumeMode::Invalid { message },
            Some(SyscallResult::Terminated(code)) => ResumeMode::Finished {
                pc: frame.registers.pc,
                code: Some(code),
            },
            Some(SyscallResult::Aborted) => ResumeMode::Paused,
            Some(SyscallResult::Exception(error)) => ResumeMode::Invalid {
                message: format_error(error, state),
            },
            Some(SyscallResult::Unimplemented(code)) => ResumeMode::Invalid {
                message: format!(
                    "Unimplemented syscall {}, file a bug or make a \
                contribution at https://github.com/1whatleytay/saturn.",
                    code
                ),
            },
            Some(SyscallResult::Unknown(code)) => ResumeMode::Invalid {
                message: format!(
                    "Unrecognized syscall {}, select a syscall by loading \
                a value into $v0.\n > li $v0, new_value\n\
                You can make a feature request or make a contribution at \
                https://github.com/1whatleytay/saturn.",
                    code
                ),
            },
            _ => {
                // This is probably okay...
                if finished_pcs.contains(&frame.registers.pc) {
                    ResumeMode::Finished {
                        pc: frame.registers.pc,
                        code: None,
                    }
                } else {
                    ResumeMode::from_executor(frame.mode, state)
                }
            }
        };

        ResumeResult {
            mode,
            registers: frame.registers.into(),
        }
    }
}

pub trait ExecutionRewindable {
    fn last_pc(&self) -> Option<u32>;
    fn rewind(&self, count: u32) -> ResumeResult;
}

pub trait RewindableDevice: ExecutionDevice + ExecutionRewindable {}

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
    Register(RegisterName),
}

impl ReadDisplayTarget {
    pub fn to_address<Reg: Registers>(self, registers: &Reg) -> u32 {
        match self {
            ReadDisplayTarget::Address(address) => address,
            ReadDisplayTarget::Register(register) => registers.get_l(register),
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

#[async_trait]
impl<Mem: Memory + Send, Reg: Registers + Send, Track: Tracker<Mem, Reg> + Send> ExecutionDevice
    for ExecutionState<Mem, Reg, Track>
{
    async fn resume(&self, options: ResumeOptions) -> Result<ResumeResult, ()> {
        let debugger = self.debugger.clone();
        let state = self.delegate.clone();
        let finished_pcs = self.finished_pcs.clone();

        if let Some(breakpoints) = options.breakpoints {
            let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

            debugger.set_breakpoints(breakpoints_set);
        }

        let is_breakpoint = debugger.is_breakpoint();

        let debugger_clone = debugger.clone();

        if let Some(mode) = options.change_state {
            debugger.override_mode(mode);
        }

        // Ensure the cancel token hasn't been set previously.
        if options
            .batch
            .as_ref()
            .map(|batch| batch.first_batch)
            .unwrap_or(true)
        {
            state.lock().unwrap().clear_cancelled();
        }

        let delegate = SyscallDelegate::new(state);

        let (frame, result) = {
            if let Some(batch) = &options.batch {
                let (frame, result) = delegate
                    .run_batch(
                        &debugger,
                        batch.count,
                        is_breakpoint && batch.first_batch,
                        batch.allow_interrupt,
                    )
                    .await
                    .unwrap_or((debugger.frame(), None));

                let frame = if frame.mode == ExecutorMode::Running && batch.break_at_end {
                    debugger.override_mode(ExecutorMode::Breakpoint);

                    // re-fetch the new frame, post override
                    // too tired to fetch it myself
                    debugger.frame()
                } else {
                    frame
                };

                (frame, result)
            } else {
                delegate.run(&debugger, is_breakpoint).await
            }
        };

        if let Some(display) = &options.display {
            let mut lock = display.lock().unwrap();

            debugger_clone.with_state(|state| lock.flush(state))
        }

        debugger.with_state(|state| {
            Ok(ResumeResult::from_frame(
                frame,
                &finished_pcs,
                result,
                state,
            ))
        })
    }

    fn pause(&self) {
        self.debugger.pause();
        self.delegate.lock().unwrap().cancel();
    }

    fn set_breakpoints(&self, breakpoints: HashSet<u32>) {
        self.debugger.set_breakpoints(breakpoints)
    }

    fn read_bytes(&self, address: u32, count: u32) -> Option<Vec<Option<u8>>> {
        let end = address
            .checked_add(count)
            .and_then(|value| value.checked_sub(1))
            .unwrap_or(u32::MAX);

        let value: Vec<Option<u8>> = self
            .debugger
            .with_memory(|memory| (address..=end).map(|a| memory.get(a).ok()).collect());

        Some(value)
    }

    fn read_display(&self, target: ReadDisplayTarget, width: u32, height: u32) -> Option<Vec<u8>> {
        self.debugger.with_state(|state| {
            let address = target.to_address(&state.registers);

            read_display(address, width, height, &mut state.memory)
        })
    }

    fn write_bytes(&self, address: u32, bytes: Vec<u8>) {
        self.debugger.with_memory(|memory| {
            for (index, byte) in bytes.iter().enumerate() {
                memory.set(address + index as u32, *byte).ok();
            }
        })
    }

    fn write_register(&self, register: u32, value: u32) {
        self.debugger.with_state(|state| match register {
            0..=31 => state.registers.set(Line(register as u8), value),
            32 => state.registers.set(Hi, value),
            33 => state.registers.set(Lo, value),
            34 => state.registers.set(Pc, value),
            _ => {}
        })
    }

    fn wake_sync(&self) {
        if let Some(sender) = self.delegate.lock().unwrap().sync_wake.take() {
            sender.send(()).ok();
        }
    }

    fn post_key(&self, key: char, up: bool) {
        self.keyboard.lock().unwrap().push_key(key, up)
    }

    fn post_input(&self, text: String) {
        self.delegate
            .lock()
            .unwrap()
            .input_buffer
            .send(text.into_bytes())
    }
}

impl<Listen: ListenResponder, Reg: Registers, Track: Tracker<SectionMemory<Listen>, Reg>>
    ExecutionRewindable for ExecutionState<SectionMemory<Listen>, Reg, Track>
{
    fn last_pc(&self) -> Option<u32> {
        None
    }

    fn rewind(&self, _: u32) -> ResumeResult {
        let frame = self.debugger.frame();

        self.debugger
            .with_state(|state| ResumeResult::from_frame(frame, &[], None, state))
    }
}

impl<Mem: Memory> ExecutionRewindable
    for ExecutionState<WatchedMemory<Mem>, WatchedRegisters, HistoryTracker>
{
    fn last_pc(&self) -> Option<u32> {
        self.debugger.with_tracker(|tracker| {
            tracker.last().as_ref().map(|entry| {
                let RegisterEntry(_, value) = entry
                    .registers
                    .iter()
                    .find(|RegisterEntry(name, _)| *name == Pc)
                    .unwrap();
                *value
            })
        })
    }

    fn rewind(&self, count: u32) -> ResumeResult {
        for _ in 0..count {
            let entry = self.debugger.with_tracker(|tracker| tracker.pop());
            let Some(entry) = entry else {
                let frame = self.debugger.frame();

                return self
                    .debugger
                    .with_state(|state| ResumeResult::from_frame(frame, &[], None, state));
            };

            self.debugger.pause();

            self.debugger.with_state(|state| {
                entry.apply(&mut state.registers, &mut state.memory.backing);
            });
        }

        let frame = self.debugger.frame();

        self.debugger
            .with_state(|state| ResumeResult::from_frame(frame, &[], None, state))
    }
}

impl<
        Listen: ListenResponder + Send,
        Reg: Registers + Send,
        Track: Tracker<SectionMemory<Listen>, Reg> + Send,
    > RewindableDevice for ExecutionState<SectionMemory<Listen>, Reg, Track>
{
}
impl<Mem: Memory + Send> RewindableDevice
    for ExecutionState<WatchedMemory<Mem>, WatchedRegisters, HistoryTracker>
{
}
