use std::cmp::min;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use titan::cpu::error::Error;
use titan::cpu::error::Error::{CpuSyscall, CpuTrap};
use titan::cpu::Memory;
use titan::cpu::memory::section::SectionMemory;
use titan::debug::Debugger;
use titan::debug::debugger::DebugFrame;
use titan::debug::debugger::DebuggerMode::{Invalid, Recovered};
use tokio::time::Instant;
use crate::keyboard::KeyboardHandler;
use crate::syscall::SyscallResult::{Aborted, Completed, Exception, Failure, Terminated, Unimplemented, Unknown};

type MemoryType = SectionMemory<KeyboardHandler>;
type DebuggerType = Debugger<MemoryType>;

pub enum SyscallResult {
    Completed, // Syscall completed successfully.
    Failure(String), // Failed to complete with message.
    Terminated(u32), // User asked process to be terminated!
    Aborted, // User paused/stopped/asked the program to stop ASAP.
    Unimplemented(u32), // User executed a recognized syscall that is not implemented yet.
    Unknown(u32), // User executed a totally unknown syscall.
    Exception(Error), // Some Memory/CPU error should be reported.
}

pub struct SyscallState {
    pub abort: bool,
    print: Box<dyn FnMut(&str) -> () + Send>,
    generators: HashMap<u32, ChaCha8Rng>
}

impl SyscallState {
    pub fn new(print: Box<dyn FnMut(&str) -> () + Send>) -> SyscallState {
        return SyscallState {
            abort: false,
            print,
            generators: HashMap::from([(0, ChaCha8Rng::from_entropy())])
        }
    }
}

pub struct SyscallDelegate {
    pub state: Arc<Mutex<SyscallState>>
}

fn reg(state: &Mutex<DebuggerType>, index: usize) -> u32 {
    state.lock().unwrap().state().registers.line[index]
}

const V0_REG: usize = 2;
const A0_REG: usize = 4;
const A1_REG: usize = 5;

fn a0(state: &Mutex<DebuggerType>) -> u32 {
    reg(state, A0_REG)
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

    async fn system_time(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => {
                let millis = time.as_millis() as u64;

                let mut debugger = state.lock().unwrap();
                let debugger_state = debugger.state();

                debugger_state.registers.line[A0_REG] = (millis & 0xFFFFFFFF) as u32;
                debugger_state.registers.line[A1_REG] = millis.wrapping_shl(32) as u32;

                Completed
            },
            Err(_) => {
                Failure("System clock failed to get current time.".into())
            }
        }
    }

    async fn midi_out(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(31)
    }

    async fn sleep_for_duration(&self, time: i64) -> bool {
        let interval = 300;

        let start = Instant::now();

        let mut remaining = time;

        while remaining > 0 {
            let wait = min(remaining, interval) as u64;

            if self.state.lock().unwrap().abort {
                return false
            }

            tokio::time::sleep(Duration::from_millis(wait)).await;

            remaining = time - start.elapsed().as_millis() as i64;
        }

        true
    }

    async fn sleep(&self, debugger: &Mutex<DebuggerType>) -> SyscallResult {
        // Not trusting sleep to be exact, so we're using Instant to keep track of the time.
        let time = a0(debugger) as i64;

        if self.sleep_for_duration(time).await {
            Completed
        } else {
            Aborted
        }
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

    async fn set_seed(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let mut syscall = self.state.lock().unwrap();
        let mut debugger = state.lock().unwrap();

        let id = debugger.state().registers.line[A0_REG]; // $a0
        let seed = debugger.state().registers.line[A1_REG]; // $a1

        syscall.generators.insert(id, ChaCha8Rng::seed_from_u64(seed as u64));

        Completed
    }

    fn fail_generator(id: u32) -> SyscallResult {
        Failure(format!(
            "No generator initialized for id {}, try using the default $a0 = 0 generator or create one with syscall 40.",
            id
        ))
    }

    async fn random_int(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let mut syscall = self.state.lock().unwrap();

        let mut debugger = state.lock().unwrap();
        let id = debugger.state().registers.line[A0_REG];
        let Some(generator) = syscall.generators.get_mut(&id) else {
            return Self::fail_generator(id)
        };

        let value: u32 = generator.gen();

        debugger.state().registers.line[A0_REG] = value;

        Completed
    }

    async fn random_int_ranged(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let mut syscall = self.state.lock().unwrap();

        let mut debugger = state.lock().unwrap();
        let id = debugger.state().registers.line[A0_REG];
        let max = debugger.state().registers.line[A1_REG];
        let Some(generator) = syscall.generators.get_mut(&id) else {
            return Self::fail_generator(id)
        };

        let value: u32 = generator.gen_range(0 .. max);

        debugger.state().registers.line[A0_REG] = value;

        Completed
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
                 let code = debugger.lock().unwrap().state().registers.line[V0_REG];
                 let result = self.dispatch(debugger, code).await;

                 (match result {
                     Completed => {
                         debugger.lock().unwrap().invalid_handled();

                         None
                     },
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
