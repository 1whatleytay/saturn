#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod syscall;
mod keyboard;
mod menu;
mod state;
mod display;
mod execution;
mod build;
mod debug;
mod channels;
mod midi;

use std::sync::Mutex;
use tauri::Manager;
use tauri::WindowEvent::Destroyed;

use crate::display::{display_protocol, FlushDisplayBody, FlushDisplayState};
use crate::menu::{create_menu, handle_event};
use crate::state::DebuggerBody;

use crate::menu::platform_shortcuts;
use crate::build::{assemble, disassemble, assemble_binary, configure_elf, configure_asm};
use crate::execution::{resume, pause, stop};
use crate::debug::{read_bytes, write_bytes, set_register, swap_breakpoints};
use crate::midi::{midi_protocol, midi_install, MidiProviderContainer};

#[tauri::command]
fn post_key(key: char, up: bool, state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    let mut keyboard = pointer.keyboard.lock().unwrap();

    keyboard.push_key(key, up)
}

#[tauri::command]
fn configure_display(address: u32, width: u32, height: u32, state: tauri::State<FlushDisplayBody>) {
    let mut body = state.lock().unwrap();

    *body = FlushDisplayState { address, width, height, data: None }
}

#[tauri::command]
fn last_display(state: tauri::State<FlushDisplayBody>) -> FlushDisplayState {
    state.lock().unwrap().clone()
}

#[tauri::command]
fn post_input(text: String, state: tauri::State<'_, DebuggerBody>) {
    let channel = {
        let Some(pointer) = &*state.lock().unwrap() else { return };
        let syscall = pointer.delegate.lock().unwrap();

        syscall.input_buffer.clone()
    };

    channel.send(text.into_bytes())
}

#[tauri::command]
fn wake_sync(state: tauri::State<'_, DebuggerBody>) {
    let Some(pointer) = &*state.lock().unwrap() else { return };

    pointer.delegate.lock().unwrap().sync_wake.notify_one();
}

#[tauri::command]
fn is_debug() -> bool {
    #[cfg(debug_assertions)]
    { true }

    #[cfg(not(debug_assertions))]
    { false }
}

fn main() {
    let menu = create_menu();

    tauri::Builder::default()
        .manage(Mutex::new(None) as DebuggerBody)
        .manage(Mutex::new(FlushDisplayState::default()) as FlushDisplayBody)
        .manage(Mutex::new(MidiProviderContainer::None))
        .menu(menu)
        .on_window_event(|event| {
            if let Destroyed = event.event() {
                // Relieve some pressure on tokio.
                stop(
                    event.window().state(),
                    event.window().state()
                )

                // Assuming tokio will join threads for me if needed.
            }
        })
        .on_menu_event(handle_event)
        .invoke_handler(tauri::generate_handler![
            platform_shortcuts, // util
            assemble, // build
            disassemble, // build
            assemble_binary, // build
            configure_elf, // build
            configure_asm, // build
            resume, // execution
            pause, // execution
            stop, // execution
            read_bytes, // debug
            write_bytes, // debug
            set_register, // debug
            swap_breakpoints, // debug
            post_key, // bitmap
            post_input, // bitmap
            configure_display, // bitmap
            last_display, // bitmap
            midi_install,
            wake_sync,
            is_debug
        ])
        .register_uri_scheme_protocol("midi", midi_protocol)
        .register_uri_scheme_protocol("display", display_protocol)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
