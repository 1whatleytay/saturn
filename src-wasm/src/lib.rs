mod console;
mod events;
mod midi;
mod time;

use crate::console::WasmConsole;
use crate::midi::WasmMidi;
use crate::time::WasmTime;
use saturn_backend::build::{
    get_binary_finished_pcs, get_elf_finished_pcs,
    TIME_TRAVEL_HISTORY_SIZE,
};
use saturn_backend::display::{FlushDisplayBody, FlushDisplayState};
use saturn_backend::keyboard::{configure_keyboard, KeyboardState};
use saturn_backend::syscall::{ConsoleHandler, MidiHandler, SyscallState, TimeHandler};
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use titan::cpu::memory::section::{ListenResponder, SectionMemory};
use titan::cpu::memory::watched::WatchedMemory;
use titan::cpu::Memory;
use titan::elf::Elf;
use titan::execution::ExecutorMode;
use titan::execution::trackers::empty::EmptyTracker;
use titan::execution::trackers::Tracker;
use titan::execution::Executor;

use titan::mips::cpu::State as MipsState;
use titan::mips::cpu::registers::registers::RawRegisters as MipsRawRegisters;
use titan::mips::cpu::registers::WatchedRegisters as MipsWatchedRegisters;
use titan::mips::execution::trackers::history::HistoryTracker as MipsHistoryTracker;
use saturn_backend::mips::device::ExecutionState as MipsExecutionState;

use titan::riscv::cpu::State as RiscVState;
use titan::riscv::cpu::registers::registers::RawRegisters as RiscVRawRegisters;
use titan::riscv::cpu::registers::WatchedRegisters as RiscVWatchedRegisters;
use titan::riscv::execution::trackers::history::HistoryTracker as RiscVHistoryTracker;
use saturn_backend::riscv::device::ExecutionState as RiscVExecutionState;

use wasm_bindgen::prelude::*;

pub use events::EventHandler;
use saturn_backend::device::{BatchOptions, ReadDisplayTarget, ResumeOptions, RewindableDevice};
use saturn_backend::platforms::Platform;

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum WasmPlatform {
    Mips,
    RiscV,
}

impl Into<Platform> for WasmPlatform {
    fn into(self) -> Platform {
        match self {
            WasmPlatform::Mips => Platform::Mips,
            WasmPlatform::RiscV => Platform::RiscV,
        }
    }
}

impl Into<WasmPlatform> for Platform {
    fn into(self) -> WasmPlatform {
        match self {
            Platform::Mips => WasmPlatform::Mips,
            Platform::RiscV => WasmPlatform::RiscV,
        }
    }
}

#[wasm_bindgen]
pub fn initialize() {
    wasm_logger::init(wasm_logger::Config::default());

    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn assemble_regions(text: &str, platform: WasmPlatform, options: JsValue) -> JsValue {
    let result = saturn_backend::regions::assemble_regions(
        text,
        None,
        platform.into(),
        serde_wasm_bindgen::from_value(options).unwrap(),
    );

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn assemble_text(text: &str, platform: WasmPlatform) -> JsValue {
    let result = saturn_backend::build::assemble(text, None, platform.into());

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn assemble_binary(text: &str, platform: WasmPlatform) -> JsValue {
    let result = saturn_backend::build::assemble_binary(text, None, platform.into());

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn decode_instruction(pc: u32, instruction: u32, platform: WasmPlatform) -> JsValue {
    let result = saturn_backend::decode::decode_instruction(pc, instruction, platform.into());

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn disassemble(named: Option<String>, bytes: Vec<u8>) -> JsValue {
    let result = saturn_backend::build::disassemble(named.as_deref(), bytes);

    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[wasm_bindgen]
pub fn detailed_disassemble(bytes: Vec<u8>, platform: WasmPlatform) -> Result<JsValue, String> {
    let result = saturn_backend::decode::detailed_disassemble(bytes, platform.into())?;

    Ok(serde_wasm_bindgen::to_value(&result).unwrap())
}

#[wasm_bindgen]
pub struct Runner {
    events: Arc<EventHandler>,
    display: RefCell<FlushDisplayBody>,
    device: RefCell<Option<Rc<dyn RewindableDevice>>>,
}

impl Runner {
    fn take_device(&self) -> Option<Rc<dyn RewindableDevice>> {
        self.device.borrow().clone()
    }

    pub fn swap_mips<
        Listen: ListenResponder + Send + 'static,
        Reg: titan::mips::cpu::Registers + Send + 'static,
        Track: Tracker<MipsState<SectionMemory<Listen>, Reg>> + Send + 'static,
    >(
        &self,
        debugger: Executor<MipsState<SectionMemory<Listen>, Reg>, Track>,
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
        let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi, time, None)));

        *self.device.borrow_mut() = Some(Rc::new(MipsExecutionState {
            debugger: wrapped,
            keyboard,
            delegate,
            finished_pcs,
        }));
    }

    pub fn swap_mips_watched<Mem: Memory + Send + 'static>(
        &self,
        debugger: Executor<MipsState<WatchedMemory<Mem>, MipsWatchedRegisters>, MipsHistoryTracker>,
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
        let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi, time, None)));

        *self.device.borrow_mut() = Some(Rc::new(MipsExecutionState {
            debugger: wrapped,
            keyboard,
            delegate,
            finished_pcs,
        }));
    }

    pub fn swap_risc_v<
        Listen: ListenResponder + Send + 'static,
        Reg: titan::riscv::cpu::Registers + Send + 'static,
        Track: Tracker<RiscVState<SectionMemory<Listen>, Reg>> + Send + 'static,
    >(
        &self,
        debugger: Executor<RiscVState<SectionMemory<Listen>, Reg>, Track>,
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
        let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi, time, None)));

        *self.device.borrow_mut() = Some(Rc::new(RiscVExecutionState {
            debugger: wrapped,
            keyboard,
            delegate,
            finished_pcs,
        }));
    }

    pub fn swap_risc_v_watched<Mem: Memory + Send + 'static>(
        &self,
        debugger: Executor<RiscVState<WatchedMemory<Mem>, RiscVWatchedRegisters>, RiscVHistoryTracker>,
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
        let delegate = Arc::new(Mutex::new(SyscallState::new(console, midi, time, None)));

        *self.device.borrow_mut() = Some(Rc::new(RiscVExecutionState {
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
        let display = display_borrow.lock().unwrap().clone_data();

        serde_wasm_bindgen::to_value(&display).unwrap()
    }

    pub fn configure_display(&self, use_default_register: bool, address: u32, width: u32, height: u32) {
        *self.display.borrow_mut() = Arc::new(Mutex::new(FlushDisplayState {
            target: ReadDisplayTarget::from_arguments(use_default_register, address),
            width,
            height,
            data: None,
        }));
    }

    pub fn configure_elf(&self, bytes: Vec<u8>, time_travel: bool) -> Option<WasmPlatform> {
        let Ok(elf) = Elf::read(&mut Cursor::new(bytes)) else {
            return None;
        };
        
        let Ok(platform) = elf.header.cpu.try_into() else {
            return None;
        };

        let finished_pcs = get_elf_finished_pcs(&elf);

        let console = Box::new(WasmConsole {
            events: self.events.clone(),
        });
        let midi = Box::new(WasmMidi {
            events: self.events.clone(),
        });
        let time = Arc::new(WasmTime {});

        let mut memory = SectionMemory::new();
        let keyboard = configure_keyboard(&mut memory);

        match platform {
            Platform::Mips => {
                if time_travel {
                    let history = MipsHistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);
                    
                    let memory = WatchedMemory::new(memory);

                    let mut cpu_state =
                        saturn_backend::mips::configuration::create_elf_state(&elf, 0x100000, memory, MipsWatchedRegisters::default());
                    saturn_backend::mips::device::setup_state(&mut cpu_state);

                    self.swap_mips_watched(
                        Executor::new(cpu_state, history),
                        finished_pcs,
                        keyboard,
                        console,
                        midi,
                        time,
                    );
                } else {
                    let mut cpu_state = saturn_backend::mips::configuration::create_elf_state(&elf, 0x100000, memory, MipsRawRegisters::default());
                    saturn_backend::mips::device::setup_state(&mut cpu_state);

                    self.swap_mips(
                        Executor::new(cpu_state, EmptyTracker {}),
                        finished_pcs,
                        keyboard,
                        console,
                        midi,
                        time,
                    );
                }
            }
            Platform::RiscV => {
                if time_travel {
                    let history = RiscVHistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

                    let memory = WatchedMemory::new(memory);

                    let mut cpu_state =
                        saturn_backend::riscv::configuration::create_elf_state(&elf, 0x100000, memory, RiscVWatchedRegisters::default());
                    saturn_backend::riscv::device::setup_state(&mut cpu_state);

                    self.swap_risc_v_watched(
                        Executor::new(cpu_state, history),
                        finished_pcs,
                        keyboard,
                        console,
                        midi,
                        time,
                    );
                } else {
                    let mut cpu_state = saturn_backend::riscv::configuration::create_elf_state(&elf, 0x100000, memory, RiscVRawRegisters::default());
                    saturn_backend::riscv::device::setup_state(&mut cpu_state);

                    self.swap_risc_v(
                        Executor::new(cpu_state, EmptyTracker {}),
                        finished_pcs,
                        keyboard,
                        console,
                        midi,
                        time,
                    );
                }
            }
        }

        Some(platform.into())
    }

    pub fn configure_asm(&self, text: &str, time_travel: bool, platform: WasmPlatform) -> JsValue {
        let (binary, result) = saturn_backend::build::assemble_text(text, None, platform.into());
        
        let Some(binary) = binary else {
            return serde_wasm_bindgen::to_value(&result).unwrap();
        };

        let finished_pcs = get_binary_finished_pcs(&binary);

        let console = Box::new(WasmConsole {
            events: self.events.clone(),
        });
        let midi = Box::new(WasmMidi {
            events: self.events.clone(),
        });
        let time = Arc::new(WasmTime {});

        let mut memory = SectionMemory::new();
        let keyboard = configure_keyboard(&mut memory);

        match platform.into() {
            Platform::Mips => {
                if time_travel {
                    let history = MipsHistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

                    let memory = WatchedMemory::new(memory);

                    let mut cpu_state =
                        saturn_backend::mips::device::state_from_binary(binary, 0x100000, memory, MipsWatchedRegisters::default());
                    saturn_backend::mips::device::setup_state(&mut cpu_state);

                    self.swap_mips_watched(
                        Executor::new(cpu_state, history),
                        finished_pcs,
                        keyboard,
                        console,
                        midi,
                        time,
                    );
                } else {
                    let mut cpu_state =
                        saturn_backend::mips::device::state_from_binary(binary, 0x100000, memory, MipsRawRegisters::default());
                    saturn_backend::mips::device::setup_state(&mut cpu_state);

                    self.swap_mips(
                        Executor::new(cpu_state, EmptyTracker {}),
                        finished_pcs,
                        keyboard,
                        console,
                        midi,
                        time,
                    );
                }
            }
            Platform::RiscV => {
                if time_travel {
                    let history = RiscVHistoryTracker::new(TIME_TRAVEL_HISTORY_SIZE);

                    let memory = WatchedMemory::new(memory);

                    let mut cpu_state =
                        saturn_backend::riscv::device::state_from_binary(binary, 0x100000, memory, RiscVWatchedRegisters::default());
                    saturn_backend::riscv::device::setup_state(&mut cpu_state);

                    self.swap_risc_v_watched(
                        Executor::new(cpu_state, history),
                        finished_pcs,
                        keyboard,
                        console,
                        midi,
                        time,
                    );
                } else {
                    let mut cpu_state =
                        saturn_backend::riscv::device::state_from_binary(binary, 0x100000, memory, RiscVRawRegisters::default());
                    saturn_backend::riscv::device::setup_state(&mut cpu_state);

                    self.swap_risc_v(
                        Executor::new(cpu_state, EmptyTracker {}),
                        finished_pcs,
                        keyboard,
                        console,
                        midi,
                        time,
                    );
                }
            }
        }

        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    pub fn last_pc(&self) -> Option<u32> {
        self.device
            .borrow()
            .as_ref()
            .and_then(|device| device.last_pc())
    }

    pub fn read_bytes(&self, address: u32, count: u32) -> JsValue {
        let result = self
            .device
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

    pub fn read_display(
        &self,
        address: u32,
        use_default_register: bool,
        width: u32,
        height: u32,
    ) -> Option<Vec<u8>> {
        if let Some(device) = &self.take_device() {
            let target = ReadDisplayTarget::from_arguments(use_default_register, address);

            device.read_display(target, width, height)
        } else {
            None
        }
    }

    pub async fn resume(
        &self,
        batch_size: usize,
        breakpoints: Option<Vec<u32>>,
        first_batch: bool,
        is_step: bool,
    ) -> JsValue {
        let Some(device) = &self.take_device() else {
            return JsValue::NULL;
        };

        let display = self.display.borrow().clone();

        let result = device
            .resume(ResumeOptions {
                batch: Some(BatchOptions {
                    count: batch_size,
                    first_batch,
                    allow_interrupt: !is_step,
                    break_at_end: is_step,
                }),
                breakpoints,
                display: Some(display),
                change_state: if !is_step && first_batch {
                    Some(ExecutorMode::Running)
                } else {
                    None
                },
            })
            .await;

        serde_wasm_bindgen::to_value(&result.ok()).unwrap()
    }

    pub fn pause(&self) {
        let Some(device) = &self.take_device() else {
            return;
        };

        device.pause()
    }

    pub fn stop(&self) {
        let Some(device) = &self.take_device() else {
            return;
        };

        device.pause();

        *self.device.borrow_mut() = None
    }

    pub fn rewind(&self, count: u32) -> JsValue {
        let Some(device) = &self.take_device() else {
            return JsValue::NULL;
        };

        let result = device.rewind(count);

        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}
