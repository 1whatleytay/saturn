use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{Read, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use titan::cpu::error::Error;
use titan::cpu::error::Error::{CpuSyscall, CpuTrap};
use titan::cpu::Memory;
use titan::cpu::memory::section::SectionMemory;
use titan::cpu::state::Registers;
use titan::debug::Debugger;
use titan::debug::debugger::DebugFrame;
use titan::debug::debugger::DebuggerMode::{Invalid, Recovered};
use tokio::select;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
use crate::channels::ByteChannel;
use crate::channels::ByteChannelConsumption::{ConsumeAndContinue, ConsumeAndStop, IgnoreAndStop};
use crate::keyboard::KeyboardHandler;
use crate::syscall::SyscallResult::{Aborted, Completed, Exception, Failure, Terminated, Unimplemented, Unknown};

type MemoryType = SectionMemory<KeyboardHandler>;
type DebuggerType = Debugger<MemoryType>;

pub struct MidiRequest {
    pub pitch: u32, // 0 - 127
    pub duration: u32, // in ms
    pub instrument: u32, // 0 - 127
    pub volume: u32, // 0 - 127
}

pub enum SyscallResult {
    Completed, // Syscall completed successfully.
    Failure(String), // Failed to complete with message.
    Terminated(u32), // User asked process to be terminated!
    Aborted, // User paused/stopped/asked the program to stop ASAP.
    Unimplemented(u32), // User executed a recognized syscall that is not implemented yet.
    Unknown(u32), // User executed a totally unknown syscall.
    Exception(Error), // Some Memory/CPU error should be reported.
}

pub trait ConsoleHandler {
    fn print(&mut self, text: &str, error: bool);
}

pub trait MidiHandler {
    fn play(&mut self, request: &MidiRequest, sync: bool);
    fn install(&mut self, instrument: u32) -> Pin<Box<dyn Future<Output = bool> + Send>>;
    fn installed(&mut self, instrument: u32) -> bool;
}

pub struct SyscallState {
    pub cancel_token: CancellationToken,
    pub input_buffer: Arc<ByteChannel>,
    pub sync_wake: Arc<Notify>,
    heap_start: u32,
    console: Box<dyn ConsoleHandler + Send>,
    midi: Box<dyn MidiHandler + Send>,
    generators: HashMap<u32, ChaCha8Rng>,
    next_file: u32,
    file_map: HashMap<u32, File>
}

impl SyscallState {
    pub fn new(
        console: Box<dyn ConsoleHandler + Send>,
        midi: Box<dyn MidiHandler + Send>
    ) -> SyscallState {
        return SyscallState {
            cancel_token: CancellationToken::new(),
            input_buffer: Arc::new(ByteChannel::new()),
            sync_wake: Arc::new(Notify::new()),
            heap_start: 0x20000000,
            console,
            midi,
            generators: HashMap::from([(0, ChaCha8Rng::from_entropy())]),
            next_file: 3,
            file_map: HashMap::new()
        }
    }

    pub fn renew(&mut self) {
        self.cancel_token = CancellationToken::new()
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel()
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
const A2_REG: usize = 6;
const A3_REG: usize = 7;

fn a0(state: &Mutex<DebuggerType>) -> u32 {
    reg(state, A0_REG)
}
fn midi_request(registers: &Registers) -> MidiRequest {
    MidiRequest {
        pitch: registers.line[A0_REG],
        duration: registers.line[A1_REG],
        instrument: registers.line[A2_REG],
        volume: registers.line[A3_REG],
    }
}

const PRINT_BUFFER_TIME: Duration = Duration::from_millis(5);

impl SyscallDelegate {
    pub fn new(state: Arc<Mutex<SyscallState>>) -> SyscallDelegate {
        SyscallDelegate { state }
    }

    async fn send_print(&self, text: &str) {
        self.state.lock().unwrap().console.print(text, false);

        tokio::time::sleep(PRINT_BUFFER_TIME).await;
    }

    fn play_installed(&self, request: &MidiRequest, sync: bool) -> bool {
        let mut syscall = self.state.lock().unwrap();

        if syscall.midi.installed(request.instrument) {
            syscall.midi.play(&request, sync);

            true
        } else {
            false
        }
    }

    async fn print_integer(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let value = a0(state);
        self.send_print(&format!("{}", value as i32)).await;

        Completed
    }

    async fn print_float(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(2)
    }

    async fn print_double(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(3)
    }

    fn grab_string<Mem: Memory>(mut address: u32, memory: &Mem, max: Option<usize>) -> Result<String, Error> {
        let mut buffer = String::new();

        loop {
            if max.map(|max| buffer.len() >= max).unwrap_or(false) {
                break
            }

            let byte = memory.get(address)?;

            if byte == 0 {
                break
            }

            buffer.push(byte as char);

            let Some(next_address) = address.checked_add(1) else {
                return Err(CpuTrap)
            };

            address = next_address
        }

        Ok(buffer)
    }

    async fn print_string(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let buffer = {
            let mut lock = state.lock().unwrap();
            let address = lock.state().registers.line[A0_REG];

            match Self::grab_string(address, lock.memory(), Some(1000)) {
                Ok(buffer) => buffer,
                Err(error) => return Exception(error),
            }
        };

        self.send_print(&buffer).await;

        Completed
    }

    fn lock_input(&self) -> Arc<ByteChannel> {
        self.state.lock().unwrap().input_buffer.clone()
    }

    async fn read_integer(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let buffer = self.lock_input();

        let mut positive = Option::<bool>::None;
        let mut value: i64 = 0;

        fn sign(c: char) -> Option<bool> {
            match c {
                '+' => Some(true),
                '-' => Some(false),
                _ => None
            }
        }

        let result = buffer.read_until(|b| {
            // No regards to utf8.
            let c = b as char;

            if positive.is_none() {
                if c.is_whitespace() {
                    return ConsumeAndContinue // just consume leading whitespace
                }

                let position = sign(c);

                positive = position.or(Some(true));

                if position.is_some() {
                    return ConsumeAndContinue
                }
            }

            if let Some(digit) = c.to_digit(10) {
                value *= 10;
                value += digit as i64;

                ConsumeAndContinue
            } else {
                IgnoreAndStop
            }
        }).await;

        if result.is_none() {
            return Aborted
        }

        let sign_value = if positive.unwrap_or(true) { 1i64 } else { -1i64 };

        let final_value = sign_value * value;

        state.lock().unwrap().state().registers.line[V0_REG] = final_value as u32;

        Completed
    }

    async fn read_float(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(6)
    }

    async fn read_double(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Unimplemented(7)
    }

    async fn read_string(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let (address, count) = {
            let mut lock = state.lock().unwrap();
            let registers = &mut lock.state().registers;

            (registers.line[A0_REG], registers.line[A1_REG])
        };

        let count = count as usize;

        if count < 1 {
            return Completed
        }

        let data = {
            let input_buffer = self.lock_input();

            let mut data: Vec<u8> = vec![];

            input_buffer.read_until(|b| {
                let c = b as char;

                if data.len() >= count - 1 { // buffer is full or done
                    return IgnoreAndStop
                }

                if c == '\n' {
                    return ConsumeAndStop
                }

                data.push(b);

                ConsumeAndContinue
            }).await;

            data.push('\0' as u8);

            assert!(data.len() <= count);

            data
        };

        let mut debugger = state.lock().unwrap();
        let memory = debugger.memory();

        for (i, b) in data.into_iter().enumerate() {
            let result = memory.set(address.wrapping_add(i as u32), b);

            if let Err(error) = result {
                return Exception(error)
            }
        }

        Completed
    }

    async fn alloc_heap(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let mut debugger = state.lock().unwrap();
        let registers = &mut debugger.state().registers;
        let count = registers.line[A0_REG];

        // Primitive Heap Alloc, assuming 0x20000000 is safe heap.
        let mut syscall = self.state.lock().unwrap();
        let pointer = syscall.heap_start;
        syscall.heap_start += count;

        registers.line[V0_REG] = pointer;

        Completed
    }

    async fn terminate(&self, _: &Mutex<DebuggerType>) -> SyscallResult {
        Terminated(0)
    }

    async fn print_character(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let character = state.lock().unwrap().state().registers.line[A0_REG] as u8 as char;

        self.send_print(&character.to_string()).await;

        Completed
    }

    async fn read_character(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let buffer = self.lock_input();

        let result = buffer.read(1).await;

        let Some(result) = result else {
            return Aborted
        };

        if result.len() != 1 {
            return Aborted
        }

        state.lock().unwrap().state().registers.line[V0_REG] = result[0] as u32;

        Completed
    }

    async fn open_file(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let mut debugger = state.lock().unwrap();
        let address = debugger.state().registers.line[A0_REG];
        let flags = debugger.state().registers.line[A1_REG];
        // Mode/$a2 is ignored.

        let filename = match Self::grab_string(address, debugger.memory(), Some(400)) {
            Ok(buffer) => buffer,
            Err(error) => return Exception(error),
        };

        let file = match flags {
            0 => File::open(filename),
            1 => File::create(filename),
            9 => OpenOptions::new().append(true).open(filename),
            _ => return Failure(format!("Invalid flags {} for opening file {}", flags, filename))
        };

        let Ok(file) = file else {
            debugger.state().registers.line[V0_REG] = (-1i32) as u32;

            return Completed
        };

        let mut syscall = self.state.lock().unwrap();

        let descriptor = syscall.next_file;

        syscall.next_file += 1;
        syscall.file_map.insert(descriptor, file);

        debugger.state().registers.line[V0_REG] = descriptor;

        Completed
    }

    async fn read_file(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let mut debugger = state.lock().unwrap();
        let cpu = debugger.state();

        let descriptor = cpu.registers.line[A0_REG];
        let address = cpu.registers.line[A1_REG];
        let size = cpu.registers.line[A2_REG];

        let mut syscall = self.state.lock().unwrap();

        let Some(file) = syscall.file_map.get_mut(&descriptor) else {
            cpu.registers.line[V0_REG] = -1i32 as u32; // descriptor does not exist

            return Completed
        };

        let mut buffer = vec![0u8; size as usize];
        let Ok(bytes) = file.read(buffer.as_mut_slice()) else {
            cpu.registers.line[V0_REG] = -2i32 as u32; // file is not opened for read

            return Completed
        };

        for i in 0 .. bytes {
            let Some(next) = address.checked_add(i as u32) else {
                return Exception(CpuTrap)
            };

            let result = cpu.memory.set(next, buffer[i]);

            if let Err(error) = result {
                return Exception(error)
            }
        }

        cpu.registers.line[V0_REG] = bytes as u32;

        Completed
    }

    async fn write_file(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let mut debugger = state.lock().unwrap();
        let cpu = debugger.state();

        let descriptor = cpu.registers.line[A0_REG];
        let address = cpu.registers.line[A1_REG];
        let size = cpu.registers.line[A2_REG];

        let mut syscall = self.state.lock().unwrap();

        let Some(file) = syscall.file_map.get_mut(&descriptor) else {
            cpu.registers.line[V0_REG] = -1i32 as u32; // descriptor does not exist

            return Completed
        };

        let mut buffer = vec![0u8; size as usize];

        for i in 0 .. size {
            let Some(next) = address.checked_add(i as u32) else {
                return Exception(CpuTrap)
            };

            match cpu.memory.get(next) {
                Ok(byte) => buffer[i as usize] = byte,
                Err(error) => return Exception(error),
            }
        }

        let Ok(bytes) = file.write(buffer.as_slice()) else {
            cpu.registers.line[V0_REG] = -2i32 as u32; // file was not opened for writing

            return Completed
        };

        cpu.registers.line[V0_REG] = bytes as u32;

        Completed
    }

    async fn close_file(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let descriptor = a0(state);

        let mut syscall = self.state.lock().unwrap();
        syscall.file_map.remove(&descriptor);

        Completed
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
                debugger_state.registers.line[A1_REG] = millis.wrapping_shr(32) as u32;

                Completed
            },
            Err(_) => {
                Failure("System clock failed to get current time.".into())
            }
        }
    }

    async fn midi_out(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let request = midi_request(&mut state.lock().unwrap().state().registers);

        if self.play_installed(&request, false) {
            return Completed
        }

        let state_clone = self.state.clone();

        tokio::spawn(async move {
            let install = {
                let mut lock = state_clone.lock().unwrap();

                lock.midi.install(request.instrument)
            };

            if install.await {
                state_clone.lock().unwrap().midi.play(&request, false)
            }
        });

        Completed
    }

    async fn sleep_for_duration(&self, time: u64) {
        let duration = Duration::from_millis(time);

        tokio::time::sleep(duration).await
    }

    async fn sleep(&self, debugger: &Mutex<DebuggerType>) -> SyscallResult {
        // Not trusting sleep to be exact, so we're using Instant to keep track of the time.
        let time = a0(debugger) as u64;

        self.sleep_for_duration(time).await;

        Completed
    }

    async fn midi_out_sync(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let request = midi_request(&mut state.lock().unwrap().state().registers);

        if self.play_installed(&request, true) {
            let notifier = self.state.lock().unwrap().sync_wake.clone();
            notifier.notified().await;

            return Completed
        }

        let install = self.state.lock().unwrap().midi.install(request.instrument);

        if install.await {
            self.state.lock().unwrap().midi.play(&request, true);

            let notifier = self.state.lock().unwrap().sync_wake.clone();
            notifier.notified().await;
        }

        Completed
    }

    async fn print_hexadecimal(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let value = a0(state);
        self.send_print(&format!("{:x}", value as i32)).await;

        Completed
    }

    async fn print_binary(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let value = a0(state);
        self.send_print(&format!("{:b}", value as i32)).await;

        Completed
    }

    async fn print_unsigned(&self, state: &Mutex<DebuggerType>) -> SyscallResult {
        let value = a0(state);
        self.send_print(&format!("{}", value)).await;

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

    async fn wrap_cancel<F: Future<Output = SyscallResult>>(&self, f: F) -> SyscallResult {
        let token = self.state.lock().unwrap().cancel_token.clone();

        let result = select! {
            result = f => result,
            _ = token.cancelled() => Aborted
        };

        result
    }

    pub async fn dispatch(&self, state: &Mutex<DebuggerType>, code: u32) -> SyscallResult {
        match code {
            1 => self.wrap_cancel(self.print_integer(state)).await,
            2 => self.wrap_cancel(self.print_float(state)).await,
            3 => self.wrap_cancel(self.print_double(state)).await,
            4 => self.wrap_cancel(self.print_string(state)).await,
            5 => self.wrap_cancel(self.read_integer(state)).await,
            6 => self.wrap_cancel(self.read_float(state)).await,
            7 => self.wrap_cancel(self.read_double(state)).await,
            8 => self.wrap_cancel(self.read_string(state)).await,
            9 => self.wrap_cancel(self.alloc_heap(state)).await,
            10 => self.wrap_cancel(self.terminate(state)).await,
            11 => self.wrap_cancel(self.print_character(state)).await,
            12 => self.wrap_cancel(self.read_character(state)).await,
            13 => self.wrap_cancel(self.open_file(state)).await,
            14 => self.wrap_cancel(self.read_file(state)).await,
            15 => self.wrap_cancel(self.write_file(state)).await,
            16 => self.wrap_cancel(self.close_file(state)).await,
            17 => self.wrap_cancel(self.terminate_valued(state)).await,
            30 => self.wrap_cancel(self.system_time(state)).await,
            31 => self.wrap_cancel(self.midi_out(state)).await,
            32 => self.wrap_cancel(self.sleep(state)).await,
            33 => self.wrap_cancel(self.midi_out_sync(state)).await,
            34 => self.wrap_cancel(self.print_hexadecimal(state)).await,
            35 => self.wrap_cancel(self.print_binary(state)).await,
            36 => self.wrap_cancel(self.print_unsigned(state)).await,
            40 => self.wrap_cancel(self.set_seed(state)).await,
            41 => self.wrap_cancel(self.random_int(state)).await,
            42 => self.wrap_cancel(self.random_int_ranged(state)).await,
            43 => self.wrap_cancel(self.random_float(state)).await,
            44 => self.wrap_cancel(self.random_double(state)).await,
            _ => return Unknown(code)
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
