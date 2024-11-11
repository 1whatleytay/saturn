use std::str::FromStr;
#[cfg(target_os = "macos")]
use tauri::AboutMetadata;
#[cfg(target_os = "windows")]
use saturn_backend::shortcuts::get_emulated_shortcuts;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, WindowMenuEvent, Wry};
use saturn_backend::shortcuts::{MenuOptions, MenuOptionsData};

fn make_item(option: MenuOptions) -> CustomMenuItem {
    let item = CustomMenuItem::new(option.to_string(), option.label());

    if let Some(accelerator) = option.accelerator() {
        item.accelerator(accelerator.combo())
    } else {
        item
    }
}

pub fn get_platform_emulated_shortcuts() -> Vec<MenuOptionsData> {
    #[cfg(target_os = "windows")]
    return get_emulated_shortcuts();
    
    #[cfg(not(target_os = "windows"))]
    vec![]
}

#[tauri::command]
pub fn platform_shortcuts() -> Vec<MenuOptionsData> {
    get_platform_emulated_shortcuts()
}

pub fn create_menu() -> Menu {
    let mut menu = Menu::new();

    #[cfg(target_os = "macos")]
    {
        let meta = AboutMetadata::default();

        menu = menu.add_submenu(Submenu::new(
            "Saturn",
            Menu::new()
                .add_native_item(MenuItem::About("Saturn".into(), meta))
                .add_item(make_item(MenuOptions::ToggleSettings))
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Hide)
                .add_native_item(MenuItem::HideOthers)
                .add_native_item(MenuItem::ShowAll)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Quit),
        ));
    }

    menu = menu.add_submenu(Submenu::new(
        "File",
        Menu::new()
            .add_item(make_item(MenuOptions::NewTab))
            .add_item(make_item(MenuOptions::OpenFile))
            .add_item(make_item(MenuOptions::CloseTab))
            .add_native_item(MenuItem::Separator)
            .add_item(make_item(MenuOptions::Save))
            .add_item(make_item(MenuOptions::SaveAs))
            .add_native_item(MenuItem::Separator)
            .add_item(make_item(MenuOptions::Assemble))
            .add_item(make_item(MenuOptions::Disassemble))
            .add_item(make_item(MenuOptions::Export))
            .add_item(make_item(MenuOptions::ExportHex))
    ));

    // windows unsupported for some of these, hopefully this wont cause a crash
    menu = menu.add_submenu(Submenu::new(
        "Edit",
        Menu::new()
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Undo)
            .add_native_item(MenuItem::Redo)
            .add_native_item(MenuItem::Separator)
            .add_item(make_item(MenuOptions::Find))
            .add_native_item(MenuItem::SelectAll),
    ));

    menu = menu.add_submenu(Submenu::new(
        "Build",
        Menu::new()
            .add_item(make_item(MenuOptions::Build))
            .add_item(make_item(MenuOptions::Run))
            .add_native_item(MenuItem::Separator)
            .add_item(make_item(MenuOptions::Step))
            .add_item(make_item(MenuOptions::Pause))
            .add_item(make_item(MenuOptions::Stop)),
    ));

    menu = menu.add_submenu(Submenu::new(
        "Window",
        Menu::new()
            .add_native_item(MenuItem::Minimize)
            .add_item(make_item(MenuOptions::ToggleConsole))
    ));

    menu
}

pub fn handle_event(event: WindowMenuEvent<Wry>) {
    let catch_emit = |result: tauri::Result<()>| {
        if result.is_err() {
            eprintln!(
                "Failed to emit event from {} menu option",
                event.menu_item_id()
            );
        }
    };

    let emit_normal = |name: &str| catch_emit(event.window().emit(name, ()));

    let Ok(item) = MenuOptions::from_str(event.menu_item_id()) else {
        return eprintln!("Unknown menu ID: {}", event.menu_item_id())
    };

    emit_normal(&item.to_string())
}
