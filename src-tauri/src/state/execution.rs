use crate::display::FlushDisplayBody;
use crate::syscall::{SyscallDelegate, SyscallResult};
use serde::Serialize;
use std::collections::HashSet;
use async_trait::async_trait;
use titan::cpu::error::Error::{CpuTrap, MemoryAlign, MemoryUnmapped};
use titan::cpu::{Memory, State};
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::cpu::state::Registers;
use titan::execution::executor::{DebugFrame, ExecutorMode};
use titan::execution::trackers::history::HistoryTracker;
use titan::execution::trackers::Tracker;
use titan::unit::instruction::InstructionDecoder;
use titan::unit::suggestions::MemoryErrorReason;
use crate::state::device::ExecutionState;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ResumeMode {
    Running,
    Invalid { message: String },
    Paused,
    Breakpoint,
    Finished { pc: u32, code: Option<u32> },
}

fn format_error<Mem: Memory>(error: titan::cpu::error::Error, state: &State<Mem>) -> String {
    let memory = |reason: MemoryErrorReason| {
        let pc = state.registers.pc.wrapping_sub(4);

        let description = state.memory.get_u32(pc).ok()
            .and_then(|value| InstructionDecoder::decode(pc, value))
            .and_then(|instruction| instruction.describe_memory_error(reason, &state.registers));

        if let Some(description) = description {
            description.to_string()
        } else {
            error.to_string()
        }
    };

    match error {
        MemoryAlign(_) => memory(MemoryErrorReason::Alignment),
        MemoryUnmapped(_) => memory(MemoryErrorReason::Unmapped),
        CpuTrap => {
            let pc = state.registers.pc.wrapping_sub(4);

            let description = state.memory.get_u32(pc).ok()
                .and_then(|value| InstructionDecoder::decode(pc, value))
                .and_then(|instruction| instruction.describe_trap_error(&state.registers));

            if let Some(description) = description {
                description.to_string()
            } else {
                error.to_string()
            }
        }
        _ => error.to_string()
    }
}

impl ResumeMode {
    fn from_executor<Mem: Memory>(value: ExecutorMode, state: &State<Mem>) -> Self {
        match value {
            ExecutorMode::Running => ResumeMode::Running,
            ExecutorMode::Recovered => ResumeMode::Breakpoint,
            ExecutorMode::Invalid(error) => ResumeMode::Invalid {
                message: format_error(error, state)
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

impl From<Registers> for RegistersResult {
    fn from(value: Registers) -> Self {
        RegistersResult {
            pc: value.pc,
            line: value.line,
            lo: value.lo,
            hi: value.hi,
        }
    }
}

#[derive(Serialize)]
pub struct ResumeResult {
    mode: ResumeMode,
    registers: RegistersResult,
}

impl ResumeResult {
    fn from_frame<Mem: Memory>(
        frame: DebugFrame,
        finished_pcs: &[u32],
        result: Option<SyscallResult>,
        state: &State<Mem>
    ) -> ResumeResult {
        let mode = match result {
            Some(SyscallResult::Failure(message)) => ResumeMode::Invalid { message },
            Some(SyscallResult::Terminated(code)) => ResumeMode::Finished {
                pc: frame.registers.pc, code: Some(code)
            },
            Some(SyscallResult::Aborted) => ResumeMode::Paused,
            Some(SyscallResult::Exception(error)) => ResumeMode::Invalid {
                message: format_error(error, state)
            },
            Some(SyscallResult::Unimplemented(code)) => ResumeMode::Invalid {
                message: format!("Unimplemented syscall {}, file a bug or make a \
                contribution at https://github.com/1whatleytay/saturn.", code)
            },
            Some(SyscallResult::Unknown(code)) => ResumeMode::Invalid {
                message: format!("Unrecognized syscall {}, select a syscall by loading \
                a value into $v0.\n > li $v0, new_value\n\
                You can make a feature request or make a contribution at \
                https://github.com/1whatleytay/saturn.", code)
            },
            _ => {
                // This is probably okay...
                if finished_pcs.contains(&frame.registers.pc) {
                    ResumeMode::Finished { pc: frame.registers.pc, code: None }
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

// NOT a tauri command.
pub fn read_display<Mem: Memory>(
    address: u32,
    width: u32,
    height: u32,
    memory: &mut Mem,
) -> Option<Vec<u8>> {
    let pixels = width.checked_mul(height)?;

    let mut result = vec![0u8; (pixels * 4) as usize];

    for i in 0..pixels {
        let point = address.wrapping_add(i.wrapping_mul(4));

        let pixel = memory.get_u32(point).ok()?;

        // Assuming little endian: 0xAARRGGBB -> [BB, GG, RR, AA] -> want [RR, GG, BB, AA]
        let start = (i as usize) * 4;
        result[start] = (pixel.wrapping_shr(16) & 0xFF) as u8;
        result[start + 1] = (pixel.wrapping_shr(8) & 0xFF) as u8;
        result[start + 2] = (pixel.wrapping_shr(0) & 0xFF) as u8;
        result[start + 3] = 255;
    }

    Some(result)
}

pub trait ExecutionRewindable {
    fn last_pc(&self) -> Option<u32>;
    fn rewind(&self, count: u32) -> ResumeResult;
}

pub trait RewindableDevice: ExecutionDevice + ExecutionRewindable { }

#[async_trait]
pub trait ExecutionDevice: Send + Sync {
    async fn resume(
        &self,
        count: Option<u32>,
        breakpoints: Option<Vec<u32>>,
        display: Option<FlushDisplayBody>
    ) -> Result<ResumeResult, ()>;

    fn pause(&self);

    fn set_breakpoints(&self, breakpoints: HashSet<u32>);

    fn read_bytes(&self, address: u32, count: u32) -> Option<Vec<Option<u8>>>;
    fn read_display(&self, address: u32, width: u32, height: u32) -> Option<Vec<u8>>;

    fn write_bytes(&self, address: u32, bytes: Vec<u8>);
    fn write_register(&self, register: u32, value: u32);

    fn wake_sync(&self);
    fn post_key(&self, key: char, up: bool);
    fn post_input(&self, text: String);
}

#[async_trait]
impl<Mem: Memory + Send, Track: Tracker<Mem> + Send> ExecutionDevice for ExecutionState<Mem, Track> {
    async fn resume(
        &self,
        count: Option<u32>,
        breakpoints: Option<Vec<u32>>,
        display: Option<FlushDisplayBody>
    ) -> Result<ResumeResult, ()> {
        let debugger = self.debugger.clone();
        let state = self.delegate.clone();
        let finished_pcs = self.finished_pcs.clone();

        if let Some(breakpoints) = breakpoints {
            let breakpoints_set = HashSet::from_iter(breakpoints.iter().copied());

            debugger.set_breakpoints(breakpoints_set);
        }

        let debugger_clone = debugger.clone();

        // Ensure the cancel token hasn't been set previously.
        state.lock().unwrap().renew();

        let delegate = SyscallDelegate::new(state);

        let (frame, result) = {
            if let Some(count) = count {
                for _ in 0..count - 1 {
                    delegate.cycle(&debugger).await;
                }

                if count > 0 {
                    delegate.cycle(&debugger).await
                } else {
                    return Err(());
                }
            } else {
                delegate.run(&debugger).await
            }
        };

        if let Some(display) = display {
            let mut lock = display.lock().unwrap();

            debugger_clone.with_memory(|memory| {
                lock.flush(memory)
            })
        }

        debugger.with_state(|state| {
            Ok(ResumeResult::from_frame(frame, &finished_pcs, result, state))
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

        let value: Vec<Option<u8>> = self.debugger.with_memory(|memory| {
            (address..=end).map(|a| memory.get(a).ok()).collect()
        });

        Some(value)
    }

    fn read_display(&self, address: u32, width: u32, height: u32) -> Option<Vec<u8>> {
        self.debugger.with_memory(|memory| {
            read_display(address, width, height, memory)
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
        self.debugger.with_state(|state| {
            match register {
                0..=31 => state.registers.line[register as usize] = value,
                32 => state.registers.hi = value,
                33 => state.registers.lo = value,
                34 => state.registers.pc = value,
                _ => {}
            }
        })
    }

    fn wake_sync(&self) {
        self.delegate.lock().unwrap().sync_wake.notify_one();
    }

    fn post_key(&self, key: char, up: bool) {
        self.keyboard.lock().unwrap().push_key(key, up)
    }

    fn post_input(&self, text: String) {
        self.delegate.lock().unwrap().input_buffer.send(text.into_bytes())
    }
}

impl<Listen: ListenResponder, Track: Tracker<SectionMemory<Listen>>> ExecutionRewindable for ExecutionState<SectionMemory<Listen>, Track> {
    fn last_pc(&self) -> Option<u32> {
        None
    }

    fn rewind(&self, _: u32) -> ResumeResult {
        let frame = self.debugger.frame();

        self.debugger.with_state(|state| {
            ResumeResult::from_frame(frame, &[], None, state)
        })
    }
}

impl<Mem: Memory> ExecutionRewindable for ExecutionState<WatchedMemory<Mem>, HistoryTracker> {
    fn last_pc(&self) -> Option<u32> {
        self.debugger.with_tracker(|tracker| {
            tracker.last()
                .as_ref()
                .map(|entry| {
                    entry.registers.pc
                })
        })
    }

    fn rewind(&self, count: u32) -> ResumeResult {
        for _ in 0 .. count {
            let entry = self.debugger.with_tracker(|tracker| tracker.pop());
            let Some(entry) = entry else {
                let frame = self.debugger.frame();

                return self.debugger.with_state(|state| {
                    ResumeResult::from_frame(frame, &[], None, state)
                })
            };

            self.debugger.pause();

            self.debugger.with_state(|state| {
                entry.apply(&mut state.registers, &mut state.memory.backing);
            });
        }

        let frame = self.debugger.frame();

        self.debugger.with_state(|state| {
            ResumeResult::from_frame(frame, &[], None, state)
        })
    }
}

impl<Listen: ListenResponder + Send, Track: Tracker<SectionMemory<Listen>> + Send> RewindableDevice for ExecutionState<SectionMemory<Listen>, Track> { }
impl<Mem: Memory + Send> RewindableDevice for ExecutionState<WatchedMemory<Mem>, HistoryTracker> { }
