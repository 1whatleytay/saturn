#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod build;
mod debug;
mod menu;
mod midi;
mod state;
mod testing;
mod access_manager;
mod watch;
mod export;
mod decode;
mod display;
mod time;

use std::sync::{Arc, Mutex};
use tauri::{FileDropEvent, Manager};
use tauri::WindowEvent::{Destroyed, FileDrop};
use crate::access_manager::{access_read_file, access_read_text, access_select_open, access_select_save, access_sync, access_write_text, AccessManager};

use saturn_backend::display::{FlushDisplayBody, FlushDisplayState};
use crate::menu::{create_menu, handle_event};

use crate::build::{assemble, assemble_binary, assemble_regions, configure_asm, configure_elf, disassemble};
use crate::debug::{read_bytes, set_register, swap_breakpoints, write_bytes};
use crate::menu::platform_shortcuts;
use crate::midi::{midi_install, midi_protocol, MidiProviderContainer};
use crate::export::{export_binary_contents, export_hex_contents, export_hex_regions};
use crate::state::DebuggerBody;

use crate::state::{last_pc, pause, post_input, post_key, resume, rewind, stop, wake_sync};
use crate::testing::{all_tests, run_tests};

use crate::decode::{decode_instruction, detailed_disassemble};
use crate::display::{configure_display, last_display, display_protocol};

#[tauri::command]
fn is_debug() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
fn send_trace(text: String) {
    tracing::info!("(JS) {text}");
}

fn main() {
    let mut desktop = tauri::api::path::home_dir().unwrap();
    
    desktop.push("Desktop/");
    
    let appender = tracing_appender::rolling::daily(desktop.clone(), "saturn-log");
    let (writer, _guard) = tracing_appender::non_blocking(appender);
    
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .init();
    
    tracing::info!("Logging initialized. Hello world!");
    
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
                    tracing::info!("File drop.");
                    
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
            access_sync,
            access_select_save,
            access_select_open,
            access_read_text,
            access_write_text,
            access_read_file,
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
            send_trace,
        ])
        .register_uri_scheme_protocol("midi", midi_protocol)
        .register_uri_scheme_protocol("display", display_protocol)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
