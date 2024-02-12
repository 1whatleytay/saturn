use crate::midi::forward_midi;
use crate::syscall::{ConsoleHandler, MidiHandler, SyscallState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use base64::Engine;
use tauri::{Manager, Wry};
use titan::assembler::binary::{Binary, RawRegion, RegionFlags};
use titan::assembler::line_details::LineDetails;
use titan::assembler::string::{assemble_from, assemble_from_path, SourceError};
use titan::cpu::memory::{Mountable, Region};
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::cpu::{Memory, State};
use titan::execution::elf::inspection::Inspection;
use titan::execution::Executor;
use titan::execution::trackers::empty::EmptyTracker;
use titan::execution::trackers::history::HistoryTracker;
use titan::execution::trackers::Tracker;
use titan::elf::program::ProgramHeaderFlags;
use titan::elf::Elf;
use crate::build::AssembleRegionsKind::HexV3;
use crate::hex_format::{encode_hex_with_encoding, HexEncoding};
use crate::keyboard::{KEYBOARD_SELECTOR, KeyboardHandler, KeyboardState};
use crate::state::commands::{DebuggerBody, setup_state, state_from_binary};
use crate::state::device::ExecutionState;
use crate::state::execution::RewindableDevice;

const TIME_TRAVEL_HISTORY_SIZE: usize = 1000;

#[derive(Serialize)]
pub struct LineMarker {
    line: usize,
    offset: usize,
}

#[derive(Serialize)]
pub struct Breakpoint {
    line: usize,
    pcs: Vec<u32>,
}

#[derive(Serialize)]
#[serde(tag = "status")]
pub enum AssemblerResult {
    Error {
        marker: Option<LineMarker>,
        message: String,
        body: Option<String>,
    },
    Success {
        breakpoints: Vec<Breakpoint>,
    },
}

impl AssemblerResult {
    fn from_result_with_binary(
        result: Result<Binary, SourceError>,
        source: &str,
    ) -> (Option<Binary>, AssemblerResult) {
        match result {
            Ok(binary) => {
                let breakpoints = binary
                    .source_breakpoints(source, 0)
                    .into_iter()
                    .map(|b| Breakpoint {
                        line: b.line,
                        pcs: b.pcs,
                    })
                    .collect();

                (Some(binary), AssemblerResult::Success { breakpoints })
            }
            Err(error) => {
                let details = error
                    .start()
                    .map(|location| LineDetails::from_offset(source, location.index));

                let marker = details.as_ref().map(|details| LineMarker {
                    line: details.line_number,
                    offset: details.line_offset,
                });

                let body = details
                    .as_ref()
                    .map(|details| format!("{}\n{}", details.line_text, details.marker()));

                (
                    None,
                    AssemblerResult::Error {
                        marker,
                        message: format!("{}", error),
                        body,
                    },
                )
            }
        }
    }

    fn from_result(result: Result<Binary, SourceError>, source: &str) -> AssemblerResult {
        Self::from_result_with_binary(result, source).1
    }
}

#[derive(Serialize)]
pub struct DisassembleResult {
    error: Option<String>,

    lines: Vec<String>,
    breakpoints: HashMap<u32, usize>,
}

#[derive(Clone, Serialize)]
struct PrintPayload<'a> {
    text: &'a str,
    error: bool,
}

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

fn assemble_text(text: &str, path: Option<&str>) -> Result<Binary, SourceError> {
    if let Some(path) = path {
        assemble_from_path(text.to_string(), PathBuf::from(path))
    } else {
        assemble_from(text)
    }
}

fn forward_print(app: tauri::AppHandle<Wry>) -> Box<dyn ConsoleHandler + Send> {
    Box::new(ForwardPrinter { app })
}

pub fn create_elf_state<Mem: Memory + Mountable>(
    elf: &Elf,
    heap_size: u32,
    mut memory: Mem
) -> State<Mem> {
    for header in &elf.program_headers {
        let region = Region {
            start: header.virtual_address,
            data: header.data.clone(),
        };

        memory.mount(region)
    }

    let heap_end = 0x7FFFFFFCu32;

    let heap = Region {
        start: heap_end - heap_size,
        data: vec![0; heap_size as usize],
    };

    memory.mount(heap);

    let mut state = State::new(elf.header.program_entry, memory);
    state.registers.line[29] = heap_end;

    state
}

#[tauri::command]
pub fn assemble(text: &str, path: Option<&str>) -> AssemblerResult {
    let result = assemble_text(text, path);

    AssemblerResult::from_result(result, text)
}

#[tauri::command]
pub fn disassemble(named: Option<&str>, bytes: Vec<u8>) -> DisassembleResult {
    let elf = match Elf::read(&mut Cursor::new(bytes)) {
        Ok(elf) => elf,
        Err(error) => {
            return DisassembleResult {
                error: Some(error.to_string()),
                lines: vec![],
                breakpoints: HashMap::new(),
            }
        }
    };

    let inspection = Inspection::new(named, &elf);

    DisassembleResult {
        error: None,
        lines: inspection.lines,
        breakpoints: inspection.breakpoints,
    }
}

#[tauri::command]
pub fn assemble_binary(text: &str, path: Option<&str>) -> (Option<Vec<u8>>, AssemblerResult) {
    let result = assemble_text(text, path);
    let (binary, result) = AssemblerResult::from_result_with_binary(result, text);

    let Some(binary) = binary else {
        return (None, result)
    };

    let elf: Elf = binary.create_elf();

    let mut out: Vec<u8> = vec![];
    let mut cursor = Cursor::new(&mut out);

    if let Err(error) = elf.write(&mut cursor) {
        return (
            None,
            AssemblerResult::Error {
                marker: None,
                message: error.to_string(),
                body: None,
            },
        );
    }

    (Some(out), result)
}

#[derive(Serialize, Deserialize)]
pub struct HexRegion {
    pub name: String,
    pub data: String // base64 encoded
}

#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum AssembleRegionsKind {
    Plain,
    HexV3
}

#[derive(Deserialize)]
pub struct AssembleRegionsOptions {
    pub kind: AssembleRegionsKind,
    pub continuous: bool,
    pub encoding: HexEncoding // Encoding option ignored if kind != HexV3
}

#[derive(Serialize)]
#[serde(rename_all="snake_case", tag="type", content="value")]
pub enum AssembledRegions {
    Binary(String), // base64 encoded
    Split(Vec<HexRegion>)
}

fn region_name(region: &RawRegion, entry: bool) -> String {
    let address = region.address;

    let heading = if entry {
        "entry"
    } else if region.flags.contains(RegionFlags::EXECUTABLE) {
        "code"
    } else {
        "data"
    };

    let flags = [
        (RegionFlags::EXECUTABLE, "x"),
        (RegionFlags::READABLE, "r"),
        (RegionFlags::WRITABLE, "w")
    ]
        .into_iter()
        .map(|(f, s)| {
            if region.flags.contains(f) { s } else { "o" }
        })
        .collect::<Vec<&str>>()
        .join("");

    format!("{heading}_{address:x}_{flags}")
}

fn encode_region_data(data: &[u8], options: &AssembleRegionsOptions) -> String {
    match options.kind {
        HexV3 => {
            let encoding = encode_hex_with_encoding(data, options.encoding);

            base64::engine::general_purpose::STANDARD.encode(encoding)
        },
        AssembleRegionsKind::Plain => {
            base64::engine::general_purpose::STANDARD.encode(data)
        }
    }
}

fn export_continuous(binary: &Binary, options: &AssembleRegionsOptions) -> String {
    let mut output: Vec<u8> = vec![];

    for region in &binary.regions {
        if region.data.is_empty() {
            continue
        }

        // Potential Overflow!
        let end = region.address as usize + region.data.len();

        if end > output.len() {
            output.resize(end, 0);
        }

        output[region.address as usize .. end].copy_from_slice(&region.data);
    };

    encode_region_data(&output, options)
}

fn export_regions(binary: &Binary, options: &AssembleRegionsOptions) -> Vec<HexRegion> {
    binary.regions.iter().filter_map(|region| {
        if region.data.is_empty() {
            return None
        }

        let name = region_name(region, region.address == binary.entry);

        Some(HexRegion {
            name,
            data: encode_region_data(&region.data, options),
        })
    }).collect()
}

#[tauri::command]
pub fn assemble_regions(
    text: &str, path: Option<&str>, options: AssembleRegionsOptions
) -> (Option<AssembledRegions>, AssemblerResult) {
    let result = assemble_text(text, path);
    let (binary, result) = AssemblerResult::from_result_with_binary(result, text);

    let Some(binary) = binary else {
        return (None, result)
    };

    let regions = if options.continuous {
        AssembledRegions::Binary(export_continuous(&binary, &options))
    } else {
        AssembledRegions::Split(export_regions(&binary, &options))
    };

    (Some(regions), result)
}

pub fn configure_keyboard(memory: &mut SectionMemory<KeyboardHandler>) -> Arc<Mutex<KeyboardState>> {
    let handler = KeyboardHandler::new();
    let keyboard = handler.state.clone();

    memory.mount_listen(KEYBOARD_SELECTOR as usize, handler);

    // Mark heap as "Writable"
    for selector in 0x1000..0x8000 {
        memory.mount_writable(selector, 0xCC);
    }

    keyboard
}

pub fn swap_watched<Mem: Memory + Send + 'static>(
    mut pointer: MutexGuard<Option<Arc<dyn RewindableDevice>>>,
    debugger: Executor<WatchedMemory<Mem>, HistoryTracker>,
    finished_pcs: Vec<u32>,
    keyboard: Arc<Mutex<KeyboardState>>,
    console: Box<dyn ConsoleHandler + Send>,
    midi: Box<dyn MidiHandler + Send>,
) {
    if let Some(state) = pointer.as_ref() {
        state.pause();
    }

    let wrapped = Arc::new(debugger);
    let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi)));

    // Drop should cancel the last process and kill the other thread.
    *pointer = Some(Arc::new(ExecutionState {
        debugger: wrapped,
        keyboard,
        delegate,
        finished_pcs,
    }));
}

pub fn swap<Listen: ListenResponder + Send + 'static, Track: Tracker<SectionMemory<Listen>> + Send + 'static>(
    mut pointer: MutexGuard<Option<Arc<dyn RewindableDevice>>>,
    debugger: Executor<SectionMemory<Listen>, Track>,
    finished_pcs: Vec<u32>,
    keyboard: Arc<Mutex<KeyboardState>>,
    console: Box<dyn ConsoleHandler + Send>,
    midi: Box<dyn MidiHandler + Send>,
) {
    if let Some(state) = pointer.as_ref() {
        state.pause();
    }

    let wrapped = Arc::new(debugger);
    let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi)));

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
    state: tauri::State<'_, DebuggerBody>,
    app_handle: tauri::AppHandle<Wry>,
) -> bool {
    let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else { return false };

    let finished_pcs = elf
        .program_headers
        .iter()
        .filter(|header| header.flags.contains(ProgramHeaderFlags::EXECUTABLE))
        .map(|header| header.virtual_address + header.data.len() as u32)
        .collect();

    let console = forward_print(app_handle.clone());
    let midi = forward_midi(app_handle);
    let history = HistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

    let mut memory = SectionMemory::new();
    let keyboard = configure_keyboard(&mut memory);

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
        );
    }

    true
}

#[tauri::command]
pub fn configure_asm(
    text: &str,
    path: Option<&str>,
    time_travel: bool,
    state: tauri::State<'_, DebuggerBody>,
    app_handle: tauri::AppHandle<Wry>,
) -> AssemblerResult {
    let binary = assemble_text(text, path);

    let (binary, result) = AssemblerResult::from_result_with_binary(binary, text);

    let Some(binary) = binary else { return result };

    // No EXECUTABLE marked regions from assembler yet.
    let finished_pcs = binary
        .regions
        .iter()
        .map(|region| region.address + region.data.len() as u32)
        .collect();

    let console = forward_print(app_handle.clone());
    let midi = forward_midi(app_handle);
    let history = HistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

    let mut memory = SectionMemory::new();
    let keyboard = configure_keyboard(&mut memory);

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
        );
    }

    result
}
