use crate::riscv::device::ExecutionState;
use crate::display::read_display;
use crate::syscall::{SyscallDelegate, SyscallResult};
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashSet;
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::execution::{DebugFrame, ExecutorMode};
use titan::execution::trackers::Tracker;
use titan::riscv::assembler::registers::RegisterSlot;
use titan::riscv::cpu::registers::registers::RawRegisters;
use titan::riscv::cpu::registers::WhichRegister::{Line, Pc};
use titan::riscv::cpu::registers::{RegisterEntry, WatchedRegisters};
use titan::riscv::cpu::state::Registers;
use titan::riscv::cpu::{Memory, State};
use titan::riscv::execution::trackers::history::HistoryTracker;
use crate::device::{ExecutionDevice, ExecutionRewindable, PlatformRegisters, ReadDisplayTarget, ResumeMode, ResumeOptions, ResumeResult};

impl ResumeMode {
    fn from_risc_v_executor(
        value: ExecutorMode
    ) -> Self {
        match value {
            ExecutorMode::Running => ResumeMode::Running,
            ExecutorMode::Invalid(error) => ResumeMode::Invalid {
                message: error.to_string()
            },
            ExecutorMode::Paused => ResumeMode::Paused,
            ExecutorMode::Breakpoint => ResumeMode::Breakpoint,
        }
    }
}

#[derive(Serialize)]
pub struct RiscVRegistersResult {
    pc: u32,
    line: [u32; 32],
}

impl<T: Registers> From<T> for RiscVRegistersResult {
    fn from(value: T) -> Self {
        let raw = value.raw();
        RiscVRegistersResult {
            pc: raw.pc,
            line: raw.line,
        }
    }
}


impl ResumeResult {
    fn from_risc_v_frame(
        frame: DebugFrame<RawRegisters>,
        finished_pcs: &[u32],
        result: Option<SyscallResult>,
    ) -> ResumeResult {
        let mode = match result {
            Some(SyscallResult::Failure(message)) => ResumeMode::Invalid { message },
            Some(SyscallResult::Terminated(code)) => ResumeMode::Finished {
                pc: frame.registers.pc,
                code: Some(code),
            },
            Some(SyscallResult::Aborted) => ResumeMode::Paused,
            Some(SyscallResult::Exception(error)) => ResumeMode::Invalid {
                message: error.to_string(),
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
                    ResumeMode::from_risc_v_executor(frame.mode)
                }
            }
        };

        ResumeResult {
            mode,
            registers: PlatformRegisters::RiscV(frame.registers.into()),
        }
    }
}

impl ReadDisplayTarget {
    pub fn to_address<Reg: Registers>(self, registers: &Reg) -> u32 {
        match self {
            ReadDisplayTarget::Address(address) => address,
            ReadDisplayTarget::DefaultRegister => registers.get_l(RegisterSlot::GlobalPointer),
        }
    }
}

#[async_trait]
impl<Mem: Memory + Send, Reg: Registers + Send, Track: Tracker<State<Mem, Reg>> + Send>
ExecutionDevice for ExecutionState<Mem, Reg, Track>
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

            let data = self.read_display(lock.target, lock.width, lock.height);
            
            lock.flush(data);
        }
        
        Ok(ResumeResult::from_risc_v_frame(
            frame,
            &finished_pcs,
            result,
        ))
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
            // 32 => state.registers.set(Hi, value),
            // 33 => state.registers.set(Lo, value),
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

impl<
    Listen: ListenResponder,
    Reg: Registers,
    Track: Tracker<State<SectionMemory<Listen>, Reg>>,
> ExecutionRewindable for ExecutionState<SectionMemory<Listen>, Reg, Track>
{
    fn last_pc(&self) -> Option<u32> {
        None
    }

    fn rewind(&self, _: u32) -> ResumeResult {
        let frame = self.debugger.frame();
        
        ResumeResult::from_risc_v_frame(frame, &[], None)
    }
}

impl<Mem: Memory> ExecutionRewindable
for ExecutionState<WatchedMemory<Mem>, WatchedRegisters, HistoryTracker>
{
    fn last_pc(&self) -> Option<u32> {
        self.debugger
            .with_tracker(|tracker| {
                tracker.last().map(|entry| {
                    entry
                        .registers
                        .iter()
                        .find(|RegisterEntry(name, _)| *name == Pc)
                        .map(|RegisterEntry(_, value)| *value)
                })
            })
            .map(|x| x.unwrap_or_else(|| self.debugger.with_state(|state| state.registers.get(Pc))))
            .map(|x| x.wrapping_sub(4))
    }

    fn rewind(&self, count: u32) -> ResumeResult {
        for _ in 0..count {
            let entry = self.debugger.with_tracker(|tracker| tracker.pop());
            let Some(entry) = entry else {
                let frame = self.debugger.frame();

                return ResumeResult::from_risc_v_frame(frame, &[], None)
            };

            self.debugger.pause();

            self.debugger.with_state(|state| {
                entry.apply(&mut state.registers.backing, &mut state.memory.backing);
            });
        }

        let frame = self.debugger.frame();

        ResumeResult::from_risc_v_frame(frame, &[], None)
    }
}
