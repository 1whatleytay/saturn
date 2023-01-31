use std::cmp::min;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use titan::cpu::error::Error;
use titan::cpu::error::Error::{CpuSyscall, CpuTrap};
use titan::cpu::Memory;
use titan::cpu::memory::section::SectionMemory;
use titan::debug::Debugger;
use titan::debug::debugger::DebugFrame;
use titan::debug::debugger::DebuggerMode::{Invalid, Recovered};
use tokio::time::Instant;
use crate::keyboard::KeyboardHandler;
use crate::syscall::SyscallResult::{Aborted, Completed, Exception, Terminated, Unimplemented, Unknown};

type MemoryType = SectionMemory<KeyboardHandler>;
type DebuggerType = Debugger<MemoryType>;

pub enum SyscallResult {
    Completed,
    Terminated(u32),
    Aborted,
    Unimplemented(u32),
    Exception(Error),
    Unknown(u32),
}

pub struct SyscallState {
    pub abort: bool,
    pub print: Box<dyn FnMut(&str) -> () + Send>
}

impl SyscallState {
    pub fn new(print: Box<dyn FnMut(&str) -> () + Send>) -> SyscallState {
        return SyscallState {
            abort: false,
            print
        }
    }
}

pub struct SyscallDelegate {
    pub state: Arc<Mutex<SyscallState>>
}

// get $a0
fn a0(state: &Mutex<DebuggerType>) -> u32 {
    state.lock().unwrap().state().registers.line[4]
}

const PRINT_BUFFER_TIME: Duration = Duration::from_millis(50);

impl SyscallDelegate {
    pub fn new(state: Arc<Mutex<SyscallState>>) -> SyscallDelegate {
        SyscallDelegate { state }
    }

    async fn print_integer(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        {
            let value = a0(state);

            (self.state.lock().unwrap().print)(&format!("{}", value as i32));
        }

        // Artificial Sleep to Prevent Spam
        tokio::time::sleep(PRINT_BUFFER_TIME).await;

        Completed
    }

    async fn print_float(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(2)
    }

    async fn print_double(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(3)
    }

    async fn print_string(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        {
            let mut address = a0(state);

            let mut buffer = String::new();

            let mut lock = state.lock().unwrap();
            let memory = lock.memory();

            loop {
                let byte = match memory.get(address) {
                    Ok(value) => value,
                    Err(error) => return Exception(error)
                };

                if byte == 0 {
                    break
                }

                buffer.push(byte as char);

                let Some(next_address) = address.checked_add(1) else {
                    return Exception(CpuTrap)
                };

                address = next_address
            }

            (self.state.lock().unwrap().print)(&buffer);
        }

        // Artificial Sleep to Prevent Spam
        tokio::time::sleep(PRINT_BUFFER_TIME).await;

        Completed
    }

    async fn read_integer(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(5)
    }

    async fn read_float(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(6)
    }

    async fn read_double(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(7)
    }

    async fn read_string(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(8)
    }

    async fn alloc_heap(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(9)
    }

    async fn terminate(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Terminated(0)
    }

    async fn print_character(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(11)
    }

    async fn read_character(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(12)
    }

    async fn open_file(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(13)
    }

    async fn read_file(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(14)
    }

    async fn write_file(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(15)
    }

    async fn close_file(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(16)
    }

    async fn terminate_valued(&self, debugger: &Mutex<DebuggerType>) -> SyscallResult {
        Terminated(a0(debugger))
    }

    async fn system_time(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(30)
    }

    async fn midi_out(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(31)
    }

    async fn sleep(&self, debugger: &Mutex<DebuggerType>) -> SyscallResult {
        let interval = 300;

        // Not trusting sleep to be exact, so we're using Instant to keep track of the time.
        let time = a0(debugger) as i64;
        let start = Instant::now();

        let mut remaining = time;

        while remaining > 0 {
            let wait = min(remaining, interval) as u64;

            if self.state.lock().unwrap().abort {
                return Aborted
            }

            tokio::time::sleep(Duration::from_millis(wait)).await;

            remaining = time - start.elapsed().as_millis() as i64;
        }

        Completed
    }

    async fn midi_out_sync(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(33)
    }

    async fn print_hexadecimal(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        {
            let value = a0(state);

            (self.state.lock().unwrap().print)(&format!("{:x}", value as i32));
        }

        // Artificial Sleep to Prevent Spam
        tokio::time::sleep(PRINT_BUFFER_TIME).await;

        Completed
    }

    async fn print_binary(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        {
            let value = a0(state);

            (self.state.lock().unwrap().print)(&format!("{:b}", value as i32));
        }

        // Artificial Sleep to Prevent Spam
        tokio::time::sleep(PRINT_BUFFER_TIME).await;

        Completed
    }

    async fn print_unsigned(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        {
            let value = a0(state);

            (self.state.lock().unwrap().print)(&format!("{}", value));
        }

        // Artificial Sleep to Prevent Spam
        tokio::time::sleep(PRINT_BUFFER_TIME).await;

        Completed
    }

    async fn set_seed(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(40)
    }

    async fn random_int(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(41)
    }

    async fn random_int_ranged(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(42)
    }

    async fn random_float(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(43)
    }

    async fn random_double(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(44)
    }

    pub async fn dispatch(&self, state: &Mutex<DebuggerType>, code: u32) -> SyscallResult {
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
            _ => Unknown(code)
        }
    }

    async fn handle_frame(
        &self, debugger: &Mutex<DebuggerType>, frame: DebugFrame
    ) -> (Option<DebugFrame>, Option<SyscallResult>) {
        match frame.mode {
             Invalid(CpuSyscall) => {
                 // $v0
                 let code = debugger.lock().unwrap().state().registers.line[2];
                 let result = self.dispatch(debugger, code).await;

                 (match result {
                     Completed => None,
                     _ => Some(frame)
                 }, Some(result))
            }
            _ => (Some(frame), None)
        }
    }

    pub async fn run(&self, debugger: &Mutex<DebuggerType>) -> (DebugFrame, Option<SyscallResult>) {
        loop {
            let frame = Debugger::run(debugger);
            let (frame, result) = self.handle_frame(debugger, frame).await;
            
            if let Some(frame) = frame {
                return (frame, result)
            }

            // Need to re-check.
            let frame = debugger.lock().unwrap().frame();

            if frame.mode != Recovered {
                return (frame, None)
            }
        }
    }

    pub async fn cycle(&self, debugger: &Mutex<DebuggerType>) -> (DebugFrame, Option<SyscallResult>) {
        let frame = {
            let mut pointer = debugger.lock().unwrap();

            pointer.cycle(true)
        };

        let (frame, result) = if let Some(frame) = frame {
            self.handle_frame(debugger, frame).await
        } else {
            (None, None)
        };
        
        (frame.unwrap_or_else(|| debugger.lock().unwrap().frame()), result)
    }
}
