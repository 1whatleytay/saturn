use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use tauri::{Manager, Wry};
use titan::cpu::Memory;
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::elf::Elf;
use titan::execution::Executor;
use titan::execution::trackers::empty::EmptyTracker;
use titan::execution::trackers::history::HistoryTracker;
use titan::execution::trackers::Tracker;
use saturn_backend::build::{assemble_text, AssemblerResult, configure_keyboard, create_elf_state, DisassembleResult, get_binary_finished_pcs, get_elf_finished_pcs, PrintPayload, TIME_TRAVEL_HISTORY_SIZE};
use saturn_backend::device::{ExecutionState, setup_state, state_from_binary};
use saturn_backend::execution::RewindableDevice;
use saturn_backend::keyboard::KeyboardState;
use saturn_backend::regions::{AssembledRegions, AssembleRegionsOptions};
use saturn_backend::syscall::{ConsoleHandler, MidiHandler, SyscallState, TimeHandler};
use crate::midi::ForwardMidi;
use crate::state::DebuggerBody;
use crate::time::TokioTimeHandler;

struct ForwardPrinter {
    app: tauri::AppHandle<Wry>,
}

impl ConsoleHandler for ForwardPrinter {
    fn print(&mut self, text: &str, error: bool) {
        self.app
            .emit_all("print", PrintPayload { text, error })
            .ok();
    }
}

fn forward_print(app: tauri::AppHandle<Wry>) -> Box<dyn ConsoleHandler + Send + Sync> {
    Box::new(ForwardPrinter { app })
}

pub fn swap<Listen: ListenResponder + Send + 'static, Track: Tracker<SectionMemory<Listen>> + Send + 'static>(
    mut pointer: MutexGuard<Option<Arc<dyn RewindableDevice>>>,
    debugger: Executor<SectionMemory<Listen>, Track>,
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
    let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi, time, current_directory)));

    // Drop should cancel the last process and kill the other thread.
    *pointer = Some(Arc::new(ExecutionState {
        debugger: wrapped,
        keyboard,
        delegate,
        finished_pcs,
    }));
}

pub fn swap_watched<Mem: Memory + Send + 'static>(
    mut pointer: MutexGuard<Option<Arc<dyn RewindableDevice>>>,
    debugger: Executor<WatchedMemory<Mem>, HistoryTracker>,
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
    let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi, time, current_directory)));

    // Drop should cancel the last process and kill the other thread.
    *pointer = Some(Arc::new(ExecutionState {
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
    state: tauri::State<'_, DebuggerBody>,
    app_handle: tauri::AppHandle<Wry>,
) -> bool {
    let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else { return false };

    let finished_pcs = get_elf_finished_pcs(&elf);
    
    let console = forward_print(app_handle.clone());
    let midi = Box::new(ForwardMidi::new(app_handle));
    let time = Arc::new(TokioTimeHandler::new());
    let history = HistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

    let mut memory = SectionMemory::new();
    let keyboard = configure_keyboard(&mut memory);

    let current_directory = path
        .and_then(|x| Path::new(&x).parent()
            .map(|x| x.to_string_lossy().to_string()));

    if time_travel {
        let memory = WatchedMemory::new(memory);

        let mut cpu_state = create_elf_state(&elf, 0x100000, memory);
        setup_state(&mut cpu_state);

        swap_watched(
            state.lock().unwrap(),
            Executor::new(cpu_state, history),
            finished_pcs,
            keyboard,
            console,
            midi,
            time,
            current_directory,
        );
    } else {
        let mut cpu_state = create_elf_state(&elf, 0x100000, memory);
        setup_state(&mut cpu_state);

        swap(
            state.lock().unwrap(),
            Executor::new(cpu_state, EmptyTracker { }),
            finished_pcs,
            keyboard,
            console,
            midi,
            time,
            current_directory,
        );
    }

    true
}

#[tauri::command]
pub fn configure_asm(
    text: &str,
    path: Option<String>,
    time_travel: bool,
    state: tauri::State<'_, DebuggerBody>,
    app_handle: tauri::AppHandle<Wry>,
) -> AssemblerResult {
    let binary = assemble_text(text, path.as_ref().map(|x| x.as_str()));

    let (binary, result) = AssemblerResult::from_result_with_binary(binary, text);

    let Some(binary) = binary else { return result };

    let finished_pcs = get_binary_finished_pcs(&binary);

    let console = forward_print(app_handle.clone());
    let midi = Box::new(ForwardMidi::new(app_handle));
    let time = Arc::new(TokioTimeHandler::new());
    let history = HistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

    let mut memory = SectionMemory::new();
    let keyboard = configure_keyboard(&mut memory);

    let current_directory = path
        .and_then(|x| Path::new(&x).parent()
            .map(|x| x.to_string_lossy().to_string()));

    if time_travel {
        let memory = WatchedMemory::new(memory);

        let mut cpu_state = state_from_binary(binary, 0x100000, memory);
        setup_state(&mut cpu_state);

        swap_watched(
            state.lock().unwrap(),
            Executor::new(cpu_state, history),
            finished_pcs,
            keyboard,
            console,
            midi,
            time,
            current_directory,
        );
    } else {
        let mut cpu_state = state_from_binary(binary, 0x100000, memory);
        setup_state(&mut cpu_state);

        swap(
            state.lock().unwrap(),
            Executor::new(cpu_state, EmptyTracker { }),
            finished_pcs,
            keyboard,
            console,
            midi,
            time,
            current_directory,
        );
    }

    result
}

#[tauri::command]
pub fn assemble(text: &str, path: Option<&str>) -> AssemblerResult {
    saturn_backend::build::assemble(text, path)
}

#[tauri::command]
pub fn assemble_binary(text: &str, path: Option<&str>) -> (Option<Vec<u8>>, AssemblerResult) {
    saturn_backend::build::assemble_binary(text, path)
}

#[tauri::command]
pub fn assemble_regions(text: &str, path: Option<&str>, options: AssembleRegionsOptions) -> (Option<AssembledRegions>, AssemblerResult) {
    saturn_backend::regions::assemble_regions(text, path, options)
}

#[tauri::command]
pub fn disassemble(named: Option<&str>, bytes: Vec<u8>) -> DisassembleResult {
    saturn_backend::build::disassemble(named, bytes)
}

