mod console;
mod midi;

use std::io::Cursor;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use titan::assembler::string::assemble_from;
use titan::cpu::Memory;
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::elf::Elf;
use titan::execution::Executor;
use titan::execution::trackers::empty::EmptyTracker;
use titan::execution::trackers::history::HistoryTracker;
use titan::execution::trackers::Tracker;
use wasm_bindgen::prelude::*;
use saturn_backend::build::{AssemblerResult, configure_keyboard, create_elf_state, get_binary_finished_pcs, get_elf_finished_pcs, TIME_TRAVEL_HISTORY_SIZE};
use saturn_backend::device::{ExecutionState, setup_state, state_from_binary};
use saturn_backend::display::{FlushDisplayBody, FlushDisplayState};
use saturn_backend::execution::RewindableDevice;
use saturn_backend::keyboard::KeyboardState;
use saturn_backend::syscall::{ConsoleHandler, MidiHandler, SyscallState};
use crate::console::WasmConsole;
use crate::midi::WasmMidi;

// noinspection RsTraitObligations
#[wasm_bindgen]
pub fn assemble_regions(text: &str, options: JsValue) -> JsValue {
    let result = saturn_backend::regions::assemble_regions(
        text, None, serde_wasm_bindgen::from_value(options).unwrap());

    serde_wasm_bindgen::to_value(&result).unwrap()
}

// noinspection RsTraitObligations
#[wasm_bindgen]
pub fn assemble_text(text: &str) -> JsValue {
    let result = saturn_backend::build::assemble(text, None);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

// noinspection RsTraitObligations
#[wasm_bindgen]
pub fn assemble_binary(text: &str) -> JsValue {
    let result = saturn_backend::build::assemble_binary(text, None);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

// noinspection RsTraitObligations
#[wasm_bindgen]
pub fn decode_instruction(pc: u32, instruction: u32) -> JsValue {
    let result = saturn_backend::decode::decode_instruction(pc, instruction);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

// noinspection RsTraitObligations
#[wasm_bindgen]
pub fn disassemble(named: Option<String>, bytes: Vec<u8>) -> JsValue {
    let result = saturn_backend::build::disassemble(named.as_deref(), bytes);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

// noinspection RsTraitObligations
#[wasm_bindgen]
pub fn detailed_disassemble(bytes: Vec<u8>) -> Result<JsValue, String> {
    let result = saturn_backend::decode::detailed_disassemble(bytes)?;

    Ok(serde_wasm_bindgen::to_value(&result).unwrap())
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Runner {
    display: FlushDisplayBody,
    device: Option<Rc<dyn RewindableDevice>>
}

impl Runner {
    pub fn swap<Listen: ListenResponder + Send + 'static, Track: Tracker<SectionMemory<Listen>> + Send + 'static>(
        &mut self,
        debugger: Executor<SectionMemory<Listen>, Track>,
        finished_pcs: Vec<u32>,
        keyboard: Arc<Mutex<KeyboardState>>,
        console: Box<dyn ConsoleHandler + Send>,
        midi: Box<dyn MidiHandler + Send>,
    ) {
        if let Some(device) = &self.device {
            device.pause()
        }

        let wrapped = Arc::new(debugger);
        let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi)));

        self.device = Some(Rc::new(ExecutionState {
            debugger: wrapped,
            keyboard,
            delegate,
            finished_pcs,
        }));
    }

    pub fn swap_watched<Mem: Memory + Send + 'static>(
        &mut self,
        debugger: Executor<WatchedMemory<Mem>, HistoryTracker>,
        finished_pcs: Vec<u32>,
        keyboard: Arc<Mutex<KeyboardState>>,
        console: Box<dyn ConsoleHandler + Send>,
        midi: Box<dyn MidiHandler + Send>,
    ) {
        if let Some(device) = &self.device {
            device.pause()
        }

        let wrapped = Arc::new(debugger);
        let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi)));

        self.device = Some(Rc::new(ExecutionState {
            debugger: wrapped,
            keyboard,
            delegate,
            finished_pcs,
        }));
    }
}

#[wasm_bindgen]
impl Runner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Runner {
        Runner::default()
    }

    // noinspection RsTraitObligations
    pub fn last_display(&self) -> JsValue {
        let display = self.display.lock().unwrap();

        serde_wasm_bindgen::to_value(&*display).unwrap()
    }

    pub fn configure_display(&mut self, address: u32, width: u32, height: u32) {
        self.display = Arc::new(Mutex::new(FlushDisplayState {
            address,
            width,
            height,
            data: None,
        }));
    }

    pub fn configure_elf(
        &mut self,
        bytes: Vec<u8>,
        time_travel: bool
    ) -> bool {
        let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else { return false };

        let finished_pcs = get_elf_finished_pcs(&elf);

        let console = Box::new(WasmConsole { });
        let midi = Box::new(WasmMidi { });
        let history = HistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

        let mut memory = SectionMemory::new();
        let keyboard = configure_keyboard(&mut memory);

        if time_travel {
            let memory = WatchedMemory::new(memory);

            let mut cpu_state = create_elf_state(&elf, 0x100000, memory);
            setup_state(&mut cpu_state);

            self.swap_watched(
                Executor::new(cpu_state, history),
                finished_pcs,
                keyboard,
                console,
                midi,
            );
        } else {
            let mut cpu_state = create_elf_state(&elf, 0x100000, memory);
            setup_state(&mut cpu_state);

            self.swap(
                Executor::new(cpu_state, EmptyTracker { }),
                finished_pcs,
                keyboard,
                console,
                midi,
            );
        }

        true
    }

    // noinspection RsTraitObligations
    pub fn configure_asm(
        &mut self,
        text: &str,
        time_travel: bool,
    ) -> JsValue {
        let binary = assemble_from(text);

        let (binary, result) = AssemblerResult::from_result_with_binary(binary, text);

        let Some(binary) = binary else {
            return serde_wasm_bindgen::to_value(&result).unwrap()
        };

        let finished_pcs = get_binary_finished_pcs(&binary);

        let console = Box::new(WasmConsole { });
        let midi = Box::new(WasmMidi { });
        let history = HistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

        let mut memory = SectionMemory::new();
        let keyboard = configure_keyboard(&mut memory);

        if time_travel {
            let memory = WatchedMemory::new(memory);

            let mut cpu_state = state_from_binary(binary, 0x100000, memory);
            setup_state(&mut cpu_state);

            self.swap_watched(
                Executor::new(cpu_state, history),
                finished_pcs,
                keyboard,
                console,
                midi,
            );
        } else {
            let mut cpu_state = state_from_binary(binary, 0x100000, memory);
            setup_state(&mut cpu_state);

            self.swap(
                Executor::new(cpu_state, EmptyTracker { }),
                finished_pcs,
                keyboard,
                console,
                midi,
            );
        }

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
    
    pub fn last_pc(&mut self) -> Option<u32> {
        self.device.as_ref().and_then(|device| device.last_pc())
    }
    
    pub fn read_bytes(&mut self, address: u32, count: u32) -> JsValue {
        let result = self.device
            .as_ref()
            .map(|device| device.read_bytes(address, count));
        
        serde_wasm_bindgen::to_value(&result).unwrap()
    }
    
    pub fn write_bytes(&mut self, address: u32, bytes: Vec<u8>) {
        if let Some(device) = &mut self.device {
            device.write_bytes(address, bytes)
        }
    }
    
    pub fn set_register(&mut self, register: u32, value: u32) {
        if let Some(device) = &mut self.device {
            device.write_register(register, value)
        }
    }

    pub async fn resume(&mut self, batch_size: usize) -> JsValue {
        /*
         I am so screwed.
         I will probably have to rewrite the backend to optionally use tokio.
         Technically Tokio can be the main event queue (if we build our own runtime)
         This sucks a bit:
          - The reason we use it in Saturn Backend is to
            - Get Time
            - Sync Up Things
            - Cancellation
          - First reason probably doesn't work in WASM. Second is not really a concern on WASM.
          - Last reason is probably the best. 
         */
        todo!()
    }
}