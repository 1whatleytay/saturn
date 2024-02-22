#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod build;
mod channels;
mod debug;
mod display;
mod keyboard;
mod menu;
mod midi;
mod syscall;
mod state;
mod testing;
mod decode;
mod hex_format;
mod access_manager;
mod watch;
mod export;

use std::sync::{Arc, Mutex};
use tauri::{FileDropEvent, Manager};
use tauri::WindowEvent::{FileDrop, Destroyed};
use crate::access_manager::AccessManager;

use crate::display::{display_protocol, FlushDisplayBody, FlushDisplayState};
use crate::menu::{create_menu, handle_event};

use crate::build::{assemble, assemble_binary, assemble_regions, configure_asm, configure_elf, disassemble};
use crate::debug::{read_bytes, set_register, swap_breakpoints, write_bytes};
use crate::menu::platform_shortcuts;
use crate::midi::{midi_install, midi_protocol, MidiProviderContainer};
use crate::state::commands::DebuggerBody;
use crate::export::{export_hex_regions, export_hex_contents, export_binary_contents};

use crate::state::commands::{resume, rewind, pause, stop, last_pc, post_key, post_input, wake_sync};
use crate::testing::{all_tests, run_tests};

use crate::decode::{decode_instruction, detailed_disassemble};

#[tauri::command]
fn configure_display(address: u32, width: u32, height: u32, state: tauri::State<FlushDisplayBody>) {
    let mut body = state.lock().unwrap();

    *body = FlushDisplayState {
        address,
        width,
        height,
        data: None,
    }
}

#[tauri::command]
fn last_display(state: tauri::State<FlushDisplayBody>) -> FlushDisplayState {
    state.lock().unwrap().clone()
}

#[tauri::command]
fn is_debug() -> bool {
    cfg!(debug_assertions)
}

fn main() {
    let menu = create_menu();

    tauri::Builder::default()
        .manage(Mutex::new(None) as DebuggerBody)
        .manage(Arc::new(Mutex::new(FlushDisplayState::default())) as FlushDisplayBody)
        .manage(Mutex::new(MidiProviderContainer::None))
        .menu(menu)
        .setup(|app| {
            app.manage(AccessManager::load(app.handle()));

            Ok(())
        })
        .on_window_event(|event| {
            match event.event() {
                FileDrop(FileDropEvent::Dropped(paths)) => {
                    let app = event.window().app_handle();
                    let manager: tauri::State<AccessManager> = app.state();

                    manager.permit(paths.clone());
                },
                Destroyed => {
                    // Relieve some pressure on tokio.
                    stop(event.window().state())

                    // Assuming tokio will join threads for me if needed.
                }
                _ => { }
            }
        })
        .on_menu_event(handle_event)
        .invoke_handler(tauri::generate_handler![
            platform_shortcuts, // util
            assemble,           // build
            disassemble,        // build
            assemble_binary,    // build
            assemble_regions,   // build
            configure_elf,      // build
            configure_asm,      // build
            resume,             // execution
            rewind,             // execution
            pause,              // execution
            stop,               // execution
            last_pc,            // execution
            read_bytes,         // debug
            write_bytes,        // debug
            set_register,       // debug
            swap_breakpoints,   // debug
            post_key,           // bitmap
            post_input,         // bitmap
            configure_display,  // bitmap
            last_display,       // bitmap
            access_manager::access_sync,
            access_manager::access_select_save,
            access_manager::access_select_open,
            access_manager::access_read_text,
            access_manager::access_write_text,
            midi_install,
            is_debug,
            wake_sync,
            all_tests,
            run_tests,
            decode_instruction,
            detailed_disassemble,
            export_hex_regions,
            export_hex_contents,
            export_binary_contents,
        ])
        .register_uri_scheme_protocol("midi", midi_protocol)
        .register_uri_scheme_protocol("display", display_protocol)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
