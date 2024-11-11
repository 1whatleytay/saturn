mod console;
mod midi;
mod time;
mod events;

use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use titan::assembler::string::assemble_from;
use titan::cpu::Memory;
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::elf::Elf;
use titan::execution::Executor;
use titan::execution::executor::ExecutorMode;
use titan::execution::trackers::empty::EmptyTracker;
use titan::execution::trackers::history::HistoryTracker;
use titan::execution::trackers::Tracker;
use wasm_bindgen::prelude::*;
use saturn_backend::build::{AssemblerResult, configure_keyboard, create_elf_state, get_binary_finished_pcs, get_elf_finished_pcs, TIME_TRAVEL_HISTORY_SIZE};
use saturn_backend::device::{ExecutionState, setup_state, state_from_binary};
use saturn_backend::display::{FlushDisplayBody, FlushDisplayState};
use saturn_backend::execution::{BatchOptions, ResumeOptions, RewindableDevice};
use saturn_backend::keyboard::KeyboardState;
use saturn_backend::syscall::{ConsoleHandler, MidiHandler, SyscallState, TimeHandler};
use crate::console::WasmConsole;
use crate::midi::WasmMidi;
use crate::time::WasmTime;

pub use events::EventHandler;

#[wasm_bindgen]
pub fn initialize() {
    wasm_logger::init(wasm_logger::Config::default());

    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn assemble_regions(text: &str, options: JsValue) -> JsValue {
    let result = saturn_backend::regions::assemble_regions(
        text, None, serde_wasm_bindgen::from_value(options).unwrap());

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn assemble_text(text: &str) -> JsValue {
    let result = saturn_backend::build::assemble(text, None);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn assemble_binary(text: &str) -> JsValue {
    let result = saturn_backend::build::assemble_binary(text, None);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn decode_instruction(pc: u32, instruction: u32) -> JsValue {
    let result = saturn_backend::decode::decode_instruction(pc, instruction);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn disassemble(named: Option<String>, bytes: Vec<u8>) -> JsValue {
    let result = saturn_backend::build::disassemble(named.as_deref(), bytes);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn detailed_disassemble(bytes: Vec<u8>) -> Result<JsValue, String> {
    let result = saturn_backend::decode::detailed_disassemble(bytes)?;

    Ok(serde_wasm_bindgen::to_value(&result).unwrap())
}

#[wasm_bindgen]
pub fn get_shortcuts() -> Vec<JsValue> {
    saturn_backend::shortcuts::get_emulated_shortcuts()
        .iter()
        .filter_map(|x| serde_wasm_bindgen::to_value(x).ok())
        .collect()
}

#[wasm_bindgen]
pub struct Runner {
    events: Arc<EventHandler>,
    display: RefCell<FlushDisplayBody>,
    device: RefCell<Option<Rc<dyn RewindableDevice>>>
}

impl Runner {
    fn take_device(&self) -> Option<Rc<dyn RewindableDevice>> {
        self.device.borrow().clone()
    }
    
    pub fn swap<Listen: ListenResponder + Send + 'static, Track: Tracker<SectionMemory<Listen>> + Send + 'static>(
        &self,
        debugger: Executor<SectionMemory<Listen>, Track>,
        finished_pcs: Vec<u32>,
        keyboard: Arc<Mutex<KeyboardState>>,
        console: Box<dyn ConsoleHandler + Send + Sync>,
        midi: Box<dyn MidiHandler + Send + Sync>,
        time: Arc<dyn TimeHandler + Send + Sync>,
    ) {
        if let Some(device) = &self.take_device() {
            device.pause()
        }

        let wrapped = Arc::new(debugger);
        let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi, time)));

        *self.device.borrow_mut() = Some(Rc::new(ExecutionState {
            debugger: wrapped,
            keyboard,
            delegate,
            finished_pcs,
        }));
    }

    pub fn swap_watched<Mem: Memory + Send + 'static>(
        &self,
        debugger: Executor<WatchedMemory<Mem>, HistoryTracker>,
        finished_pcs: Vec<u32>,
        keyboard: Arc<Mutex<KeyboardState>>,
        console: Box<dyn ConsoleHandler + Send + Sync>,
        midi: Box<dyn MidiHandler + Send + Sync>,
        time: Arc<dyn TimeHandler + Send + Sync>,
    ) {
        if let Some(device) = &self.take_device() {
            device.pause()
        }

        let wrapped = Arc::new(debugger);
        let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi, time)));

        *self.device.borrow_mut() = Some(Rc::new(ExecutionState {
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
    pub fn new(events: EventHandler) -> Runner {
        Runner {
            events: Arc::new(events),
            display: RefCell::new(Arc::new(Mutex::new(Default::default()))),
            device: RefCell::new(None),
        }
    }

    pub fn set_breakpoints(&self, breakpoints: &[u32]) {
        if let Some(device) = &self.take_device() {
            device.set_breakpoints(HashSet::<u32>::from_iter(breakpoints.iter().copied()))
        }
    }

    pub fn last_display(&self) -> JsValue {
        let display_borrow = self.display.borrow();
        let display = display_borrow.lock().unwrap();

        serde_wasm_bindgen::to_value(&*display).unwrap()
    }

    pub fn configure_display(&self, address: u32, width: u32, height: u32) {
        *self.display.borrow_mut() = Arc::new(Mutex::new(FlushDisplayState {
            address,
            width,
            height,
            data: None,
        }));
    }

    pub fn configure_elf(
        &self,
        bytes: Vec<u8>,
        time_travel: bool
    ) -> bool {
        let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else { return false };

        let finished_pcs = get_elf_finished_pcs(&elf);

        let console = Box::new(WasmConsole { events: self.events.clone() });
        let midi = Box::new(WasmMidi { });
        let time = Arc::new(WasmTime { });
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
                time,
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
                time,
            );
        }

        true
    }

    pub fn configure_asm(
        &self,
        text: &str,
        time_travel: bool,
    ) -> JsValue {
        let binary = assemble_from(text);

        let (binary, result) = AssemblerResult::from_result_with_binary(binary, text);

        let Some(binary) = binary else {
            return serde_wasm_bindgen::to_value(&result).unwrap()
        };

        let finished_pcs = get_binary_finished_pcs(&binary);

        let console = Box::new(WasmConsole { events: self.events.clone() });
        let midi = Box::new(WasmMidi { });
        let time = Arc::new(WasmTime { });
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
                time,
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
                time,
            );
        }

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
    
    pub fn last_pc(&self) -> Option<u32> {
        self.device.borrow().as_ref().and_then(|device| device.last_pc())
    }
    
    pub fn read_bytes(&self, address: u32, count: u32) -> JsValue {
        let result = self.device
            .borrow()
            .as_ref()
            .and_then(|device| device.read_bytes(address, count));

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
    
    pub fn write_bytes(&self, address: u32, bytes: Vec<u8>) {
        if let Some(device) = &self.take_device() {
            device.write_bytes(address, bytes)
        }
    }
    
    pub fn set_register(&self, register: u32, value: u32) {
        if let Some(device) = &self.take_device() {
            device.write_register(register, value)
        }
    }

    pub fn post_input(&self, text: String) {
        if let Some(device) = &self.take_device() {
            device.post_input(text)
        }
    }

    pub fn post_key(&self, key: char, up: bool) {
        if let Some(device) = &self.take_device() {
            device.post_key(key, up)
        }
    }

    pub fn wake_sync(&self) {
        if let Some(device) = &self.take_device() {
            device.wake_sync()
        }
    }

    pub fn read_display(&self, address: u32, width: u32, height: u32) -> Option<Vec<u8>> {
        if let Some(device) = &self.take_device() {
            device.read_display(address, width, height)
        } else {
            None
        }
    }

    pub async fn resume(&self, batch_size: usize, breakpoints: Option<Vec<u32>>, first_batch: bool, is_step: bool) -> JsValue {
        let Some(device) = &self.take_device() else {
            return JsValue::NULL
        };

        let display = self.display.borrow().clone();

        let result = device.resume(ResumeOptions {
            batch: Some(BatchOptions {
                count: batch_size,
                first_batch,
                allow_interrupt: !is_step,
                break_at_end: is_step
            }),
            breakpoints,
            display: Some(display),
            change_state: if !is_step && first_batch { Some(ExecutorMode::Running) } else { None }
        }).await;
        
        serde_wasm_bindgen::to_value(&result.ok()).unwrap()
    }

    pub fn pause(&self) {
        let Some(device) = &self.take_device() else {
            return
        };

        device.pause()
    }

    pub fn stop(&self) {
        let Some(device) = &self.take_device() else {
            return
        };

        device.pause();

        *self.device.borrow_mut() = None
    }

    pub fn rewind(&self, count: u32) -> JsValue {
        let Some(device) = &self.take_device() else {
            return JsValue::NULL
        };

        let result = device.rewind(count);

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}
