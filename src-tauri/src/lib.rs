#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod access_manager;
mod build;
mod debug;
mod decode;
mod display;
mod export;
mod menu;
mod midi;
mod state;
mod testing;
mod time;
mod watch;

use crate::access_manager::{
    access_read_file, access_read_text, access_select_open, access_select_save, access_sync,
    access_write_text, AccessManager,
};
use std::sync::{Arc, Mutex};
use tauri::WindowEvent::{Destroyed, DragDrop};
use tauri::{DragDropEvent, Manager, Url};
use tauri_plugin_deep_link::DeepLinkExt;

use crate::menu::{create_menu, handle_event};
use saturn_backend::display::{FlushDisplayBody, FlushDisplayState};

use crate::build::{
    assemble, assemble_binary, assemble_regions, configure_asm, configure_elf, disassemble,
};
use crate::debug::{read_bytes, set_register, swap_breakpoints, write_bytes};
use crate::export::{export_binary_contents, export_hex_contents, export_hex_regions};
use crate::menu::platform_shortcuts;
use crate::midi::{midi_install, midi_protocol, MidiProviderContainer};
use crate::state::DebuggerBody;

use crate::state::{last_pc, pause, post_input, post_key, resume, rewind, stop, wake_sync};
use crate::testing::{all_tests, run_tests};

use crate::decode::{decode_instruction, detailed_disassemble};
use crate::display::{configure_display, display_protocol, last_display};

#[tauri::command]
fn is_debug() -> bool {
    cfg!(debug_assertions)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(None) as DebuggerBody)
        .manage(Arc::new(Mutex::new(FlushDisplayState::default())) as FlushDisplayBody)
        .manage(Mutex::new(MidiProviderContainer::None))
        .menu(create_menu)
        .setup(|app| {
            let map_paths = |paths: Vec<Url>| {
                paths
                    .into_iter()
                    .flat_map(|url| {
                        if url.scheme() == "file" {
                            url.to_file_path().ok()
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };

            let access_manager = AccessManager::load(app.handle().clone());

            let deep_link = app.deep_link();

            if let Ok(Some(paths)) = deep_link.get_current() {
                access_manager.permit(map_paths(paths));
            }
            let handle = app.handle().clone();
            deep_link.on_open_url(move |event| {
                let paths = map_paths(event.urls());

                let manager: tauri::State<AccessManager> = handle.state();
                manager.permit(paths);
            });

            app.manage(access_manager);

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                DragDrop(DragDropEvent::Drop { paths, position: _ }) => {
                    let app = window.app_handle();
                    let manager: tauri::State<AccessManager> = app.state();

                    manager.permit(paths.clone());
                }
                Destroyed => {
                    // Relieve some pressure on tokio.
                    stop(window.state())

                    // Assuming tokio will join threads for me if needed.
                }
                _ => {}
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
        ])
        .register_uri_scheme_protocol("midi", midi_protocol)
        .register_uri_scheme_protocol("display", display_protocol)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
