use std::cmp::min;
use std::sync::Mutex;
use std::time::Duration;
use titan::cpu::error::Error::CpuSyscall;
use titan::cpu::memory::section::SectionMemory;
use titan::debug::Debugger;
use titan::debug::debugger::DebugFrame;
use titan::debug::debugger::DebuggerMode::Invalid;
use tokio::time::Instant;
use crate::keyboard::KeyboardHandler;
use crate::syscall::SyscallResult::{Aborted, Completed, Terminated, Unimplemented, Unknown};

type MemoryType = SectionMemory<KeyboardHandler>;
type DebuggerType = Debugger<MemoryType>;

pub enum SyscallResult {
    Completed,
    Terminated(u32),
    Aborted,
    Unimplemented,
    Unknown,
}

pub struct SyscallDelegate {
    abort: bool
}

// get $a0
fn a0(state: &Mutex<DebuggerType>) -> u32 {
    state.lock().unwrap().state().registers.line[4]
}

impl SyscallDelegate {
    pub fn new() -> SyscallDelegate {
        SyscallDelegate { abort: false }
    }

    async fn print_integer(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn print_float(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn print_double(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn print_string(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn read_integer(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn read_float(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn read_double(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn read_string(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn alloc_heap(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn terminate(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Terminated(0)
    }

    async fn print_character(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn read_character(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn open_file(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn read_file(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn write_file(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn close_file(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn terminate_valued(&mut self, debugger: &Mutex<DebuggerType>) -> SyscallResult {
        Terminated(a0(debugger))
    }

    async fn system_time(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn midi_out(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn sleep(&mut self, debugger: &Mutex<DebuggerType>) -> SyscallResult {
        let interval = 300;

        // Not trusting sleep to be exact, so we're using Instant to keep track of the time.
        let time = a0(debugger) as i64;
        let start = Instant::now();

        let mut remaining = time;

        while remaining > 0 {
            let wait = min(remaining, interval) as u64;

            if self.abort {
                return Aborted
            }

            tokio::time::sleep(Duration::from_millis(wait)).await;

            remaining = time - start.elapsed().as_millis() as i64;
        }

        Completed
    }

    async fn midi_out_sync(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn print_hexadecimal(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn print_binary(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn print_unsigned(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn set_seed(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn random_int(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn random_int_ranged(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn random_float(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    async fn random_double(&mut self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented
    }

    pub async fn dispatch(&mut self, state: &Mutex<DebuggerType>, code: u32) -> SyscallResult {
        match code {
            1 => self.print_integer(state).await,
            2 => self.print_float(state).await,
            3 => self.print_double(state).await,
            4 => self.print_string(state).await,
            5 => self.read_integer(state).await,
            6 => self.read_float(state).await,
            7 => self.read_double(state).await,
            8 => self.read_string(state).await,
            9 => self.alloc_heap(state).await,
            10 => self.terminate(state).await,
            11 => self.print_character(state).await,
            12 => self.read_character(state).await,
            13 => self.open_file(state).await,
            14 => self.read_file(state).await,
            15 => self.write_file(state).await,
            16 => self.close_file(state).await,
            17 => self.terminate_valued(state).await,
            30 => self.system_time(state).await,
            31 => self.midi_out(state).await,
            32 => self.sleep(state).await,
            33 => self.midi_out_sync(state).await,
            34 => self.print_hexadecimal(state).await,
            35 => self.print_binary(state).await,
            36 => self.print_unsigned(state).await,
            40 => self.set_seed(state).await,
            41 => self.random_int(state).await,
            42 => self.random_int_ranged(state).await,
            43 => self.random_float(state).await,
            44 => self.random_double(state).await,
            _ => Unknown
        }
    }

    async fn handle_frame(
        &mut self, debugger: &Mutex<DebuggerType>, frame: DebugFrame
    ) -> Option<DebugFrame> {
        match frame.mode {
             Invalid(CpuSyscall) => {
                // $v0
                let code = debugger.lock().unwrap().state().registers.line[2];
                let _ = self.dispatch(debugger, code).await;

                 None
            }
            _ => Some(frame)
        }
    }

    pub async fn run(&mut self, debugger: &Mutex<DebuggerType>) -> DebugFrame {
        loop {
            let frame = Debugger::run(debugger);
            let frame = self.handle_frame(debugger, frame).await;

            if let Some(frame) = frame {
                return frame
            }
        }
    }

    pub async fn cycle(&mut self, debugger: &Mutex<DebuggerType>) -> DebugFrame {
        let frame = {
            let mut pointer = debugger.lock().unwrap();

            pointer.cycle(true)
        };

        if let Some(frame) = frame {
            self.handle_frame(debugger, frame).await
        } else {
            None
        }
            .unwrap_or_else(|| debugger.lock().unwrap().frame())
    }
}
