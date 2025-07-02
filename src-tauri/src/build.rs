use crate::midi::ForwardMidi;
use crate::state::DebuggerBody;
use crate::time::TokioTimeHandler;
use saturn_backend::keyboard::configure_keyboard;
use saturn_backend::platforms::Platform;
use titan::cpu::memory::Memory;

use saturn_backend::mips::device::ExecutionState as MipsExecutionState;
use titan::mips::cpu::registers::registers::RawRegisters as MipsRawRegisters;
use titan::mips::cpu::registers::Registers as MipsRegisters;
use titan::mips::cpu::registers::WatchedRegisters as MipsWatchedRegisters;
use titan::mips::cpu::State as MipsState;
use titan::mips::execution::trackers::history::HistoryTracker as MipsHistoryTracker;

use saturn_backend::riscv::device::ExecutionState as RiscVExecutionState;
use titan::riscv::cpu::registers::registers::RawRegisters as RiscVRawRegisters;
use titan::riscv::cpu::registers::Registers as RiscVRegisters;
use titan::riscv::cpu::registers::WatchedRegisters as RiscVWatchedRegisters;
use titan::riscv::cpu::State as RiscVState;
use titan::riscv::execution::trackers::history::HistoryTracker as RiscVHistoryTracker;

use saturn_backend::mips::configuration::create_elf_state as create_mips_elf_state;
use saturn_backend::mips::device::{
    setup_state as mips_setup_state, state_from_binary as mips_state_from_binary,
};

use saturn_backend::riscv::configuration::create_elf_state as create_riscv_elf_state;
use saturn_backend::riscv::device::{
    setup_state as riscv_setup_state, state_from_binary as riscv_state_from_binary,
};

use saturn_backend::build::{
    assemble_text, get_binary_finished_pcs, get_elf_finished_pcs, AssemblerResult,
    DisassembleResult, PrintPayload, TIME_TRAVEL_HISTORY_SIZE,
};
use saturn_backend::device::RewindableDevice;
use saturn_backend::keyboard::KeyboardState;
use saturn_backend::regions::{AssembleRegionsOptions, AssembledRegions};
use saturn_backend::syscall::{ConsoleHandler, MidiHandler, SyscallState, TimeHandler};
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{Emitter, Wry};
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::elf::Elf;
use titan::execution::trackers::empty::EmptyTracker;
use titan::execution::trackers::Tracker;
use titan::execution::Executor;

struct ForwardPrinter {
    app: tauri::AppHandle<Wry>,
}

impl ConsoleHandler for ForwardPrinter {
    fn print(&mut self, text: &str, error: bool) {
        self.app.emit("print", PrintPayload { text, error }).ok();
    }
}

fn forward_print(app: tauri::AppHandle<Wry>) -> Box<dyn ConsoleHandler + Send + Sync> {
    Box::new(ForwardPrinter { app })
}

fn mips_swap<
    Listen: ListenResponder + Send + 'static,
    Reg: MipsRegisters + Send + 'static,
    Track: Tracker<MipsState<SectionMemory<Listen>, Reg>> + Send + 'static,
>(
    mut pointer: MutexGuard<Option<Arc<dyn RewindableDevice>>>,
    debugger: Executor<MipsState<SectionMemory<Listen>, Reg>, Track>,
    finished_pcs: Vec<u32>,
    keyboard: Arc<Mutex<KeyboardState>>,
    console: Box<dyn ConsoleHandler + Send + Sync>,
    midi: Box<dyn MidiHandler + Send + Sync>,
    time: Arc<dyn TimeHandler + Send + Sync>,
    current_directory: Option<String>,
) {
    if let Some(state) = pointer.as_ref() {
        state.pause();
    }

    let wrapped = Arc::new(debugger);
    let delegate = Arc::new(Mutex::new(SyscallState::new(
        console,
        midi,
        time,
        current_directory,
    )));

    // Drop should cancel the last process and kill the other thread.
    *pointer = Some(Arc::new(MipsExecutionState {
        debugger: wrapped,
        keyboard,
        delegate,
        finished_pcs,
    }));
}

fn mips_swap_watched<Mem: Memory + Send + 'static>(
    mut pointer: MutexGuard<Option<Arc<dyn RewindableDevice>>>,
    debugger: Executor<MipsState<WatchedMemory<Mem>, MipsWatchedRegisters>, MipsHistoryTracker>,
    finished_pcs: Vec<u32>,
    keyboard: Arc<Mutex<KeyboardState>>,
    console: Box<dyn ConsoleHandler + Send + Sync>,
    midi: Box<dyn MidiHandler + Send + Sync>,
    time: Arc<dyn TimeHandler + Send + Sync>,
    current_directory: Option<String>,
) {
    if let Some(state) = pointer.as_ref() {
        state.pause();
    }

    let wrapped = Arc::new(debugger);
    let delegate = Arc::new(Mutex::new(SyscallState::new(
        console,
        midi,
        time,
        current_directory,
    )));

    // Drop should cancel the last process and kill the other thread.
    *pointer = Some(Arc::new(MipsExecutionState {
        debugger: wrapped,
        keyboard,
        delegate,
        finished_pcs,
    }));
}

fn riscv_swap<
    Listen: ListenResponder + Send + 'static,
    Reg: RiscVRegisters + Send + 'static,
    Track: Tracker<RiscVState<SectionMemory<Listen>, Reg>> + Send + 'static,
>(
    mut pointer: MutexGuard<Option<Arc<dyn RewindableDevice>>>,
    debugger: Executor<RiscVState<SectionMemory<Listen>, Reg>, Track>,
    finished_pcs: Vec<u32>,
    keyboard: Arc<Mutex<KeyboardState>>,
    console: Box<dyn ConsoleHandler + Send + Sync>,
    midi: Box<dyn MidiHandler + Send + Sync>,
    time: Arc<dyn TimeHandler + Send + Sync>,
    current_directory: Option<String>,
) {
    if let Some(state) = pointer.as_ref() {
        state.pause();
    }

    let wrapped = Arc::new(debugger);
    let delegate = Arc::new(Mutex::new(SyscallState::new(
        console,
        midi,
        time,
        current_directory,
    )));

    // Drop should cancel the last process and kill the other thread.
    *pointer = Some(Arc::new(RiscVExecutionState {
        debugger: wrapped,
        keyboard,
        delegate,
        finished_pcs,
    }));
}

pub fn riscv_swap_watched<Mem: Memory + Send + 'static>(
    mut pointer: MutexGuard<Option<Arc<dyn RewindableDevice>>>,
    debugger: Executor<RiscVState<WatchedMemory<Mem>, RiscVWatchedRegisters>, RiscVHistoryTracker>,
    finished_pcs: Vec<u32>,
    keyboard: Arc<Mutex<KeyboardState>>,
    console: Box<dyn ConsoleHandler + Send + Sync>,
    midi: Box<dyn MidiHandler + Send + Sync>,
    time: Arc<dyn TimeHandler + Send + Sync>,
    current_directory: Option<String>,
) {
    if let Some(state) = pointer.as_ref() {
        state.pause();
    }

    let wrapped = Arc::new(debugger);
    let delegate = Arc::new(Mutex::new(SyscallState::new(
        console,
        midi,
        time,
        current_directory,
    )));

    // Drop should cancel the last process and kill the other thread.
    *pointer = Some(Arc::new(RiscVExecutionState {
        debugger: wrapped,
        keyboard,
        delegate,
        finished_pcs,
    }));
}

#[tauri::command]
pub fn configure_elf(
    bytes: Vec<u8>,
    time_travel: bool,
    path: Option<String>,
    platform: Platform,
    state: tauri::State<'_, DebuggerBody>,
    app_handle: tauri::AppHandle<Wry>,
) -> bool {
    let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else {
        return false;
    };

    let finished_pcs = get_elf_finished_pcs(&elf);

    let console = forward_print(app_handle.clone());
    let midi = Box::new(ForwardMidi::new(app_handle));
    let time = Arc::new(TokioTimeHandler::new());

    let mut memory = SectionMemory::new();
    let keyboard = configure_keyboard(&mut memory);

    let current_directory = path.and_then(|x| {
        Path::new(&x)
            .parent()
            .map(|x| x.to_string_lossy().to_string())
    });

    if time_travel {
        match platform {
            Platform::Mips => {
                let history = MipsHistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

                let memory = WatchedMemory::new(memory);

                let mut cpu_state =
                    create_mips_elf_state(&elf, 0x100000, memory, MipsWatchedRegisters::default());

                mips_setup_state(&mut cpu_state);

                mips_swap_watched(
                    state.lock().unwrap(),
                    Executor::new(cpu_state, history),
                    finished_pcs,
                    keyboard,
                    console,
                    midi,
                    time,
                    current_directory,
                );
            }
            Platform::RiscV => {
                let history = RiscVHistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

                let memory = WatchedMemory::new(memory);

                let mut cpu_state =
                    create_riscv_elf_state(&elf, 0x100000, memory, RiscVWatchedRegisters::default());

                riscv_setup_state(&mut cpu_state);

                riscv_swap_watched(
                    state.lock().unwrap(),
                    Executor::new(cpu_state, history),
                    finished_pcs,
                    keyboard,
                    console,
                    midi,
                    time,
                    current_directory,
                );
            }
        }
    } else {
        match platform {
            Platform::Mips => {
                let mut cpu_state =
                    create_mips_elf_state(&elf, 0x100000, memory, MipsRawRegisters::default());

                mips_setup_state(&mut cpu_state);

                mips_swap(
                    state.lock().unwrap(),
                    Executor::new(cpu_state, EmptyTracker {}),
                    finished_pcs,
                    keyboard,
                    console,
                    midi,
                    time,
                    current_directory,
                );
            }
            Platform::RiscV => {
                let mut cpu_state =
                    create_riscv_elf_state(&elf, 0x100000, memory, RiscVRawRegisters::default());

                riscv_setup_state(&mut cpu_state);

                riscv_swap(
                    state.lock().unwrap(),
                    Executor::new(cpu_state, EmptyTracker {}),
                    finished_pcs,
                    keyboard,
                    console,
                    midi,
                    time,
                    current_directory,
                );
            }
        }
    }

    true
}

#[tauri::command]
pub fn configure_asm(
    text: &str,
    path: Option<String>,
    time_travel: bool,
    platform: Platform,
    state: tauri::State<'_, DebuggerBody>,
    app_handle: tauri::AppHandle<Wry>,
) -> AssemblerResult {
    let (binary, result) = assemble_text(text, path.as_ref().map(|x| x.as_str()), platform);

    let Some(binary) = binary else { return result };

    let finished_pcs = get_binary_finished_pcs(&binary);

    let console = forward_print(app_handle.clone());
    let midi = Box::new(ForwardMidi::new(app_handle));
    let time = Arc::new(TokioTimeHandler::new());

    let mut memory = SectionMemory::new();
    let keyboard = configure_keyboard(&mut memory);

    let current_directory = path.and_then(|x| {
        Path::new(&x)
            .parent()
            .map(|x| x.to_string_lossy().to_string())
    });

    if time_travel {
        match platform {
            Platform::Mips => {
                let history = MipsHistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

                let memory = WatchedMemory::new(memory);

                let mut cpu_state =
                    mips_state_from_binary(binary, 0x100000, memory, MipsWatchedRegisters::default());

                mips_setup_state(&mut cpu_state);

                mips_swap_watched(
                    state.lock().unwrap(),
                    Executor::new(cpu_state, history),
                    finished_pcs,
                    keyboard,
                    console,
                    midi,
                    time,
                    current_directory,
                );
            }
            Platform::RiscV => {
                let history = RiscVHistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

                let memory = WatchedMemory::new(memory);

                let mut cpu_state =
                    riscv_state_from_binary(binary, 0x100000, memory, RiscVWatchedRegisters::default());

                riscv_setup_state(&mut cpu_state);

                riscv_swap_watched(
                    state.lock().unwrap(),
                    Executor::new(cpu_state, history),
                    finished_pcs,
                    keyboard,
                    console,
                    midi,
                    time,
                    current_directory,
                );
            }
        }
    } else {
        match platform {
            Platform::Mips => {
                let mut cpu_state =
                    mips_state_from_binary(binary, 0x100000, memory, MipsRawRegisters::default());

                mips_setup_state(&mut cpu_state);

                mips_swap(
                    state.lock().unwrap(),
                    Executor::new(cpu_state, EmptyTracker {}),
                    finished_pcs,
                    keyboard,
                    console,
                    midi,
                    time,
                    current_directory,
                );
            }
            Platform::RiscV => {
                let mut cpu_state =
                    riscv_state_from_binary(binary, 0x100000, memory, RiscVRawRegisters::default());

                riscv_setup_state(&mut cpu_state);

                riscv_swap(
                    state.lock().unwrap(),
                    Executor::new(cpu_state, EmptyTracker {}),
                    finished_pcs,
                    keyboard,
                    console,
                    midi,
                    time,
                    current_directory,
                );
            }
        }
    }

    result
}

#[tauri::command]
pub fn assemble(text: &str, path: Option<&str>, platform: Platform) -> AssemblerResult {
    saturn_backend::build::assemble(text, path, platform)
}

#[tauri::command]
pub fn assemble_binary(
    text: &str,
    path: Option<&str>,
    platform: Platform,
) -> (Option<Vec<u8>>, AssemblerResult) {
    saturn_backend::build::assemble_binary(text, path, platform)
}

#[tauri::command]
pub fn assemble_regions(
    text: &str,
    path: Option<&str>,
    platform: Platform,
    options: AssembleRegionsOptions,
) -> (Option<AssembledRegions>, AssemblerResult) {
    saturn_backend::regions::assemble_regions(text, path, platform, options)
}

#[tauri::command]
pub fn disassemble(named: Option<&str>, bytes: Vec<u8>) -> DisassembleResult {
    saturn_backend::build::disassemble(named, bytes)
}
