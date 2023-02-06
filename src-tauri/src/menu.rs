use std::str::FromStr;
use serde::Serialize;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, WindowMenuEvent, Wry};
#[cfg(target_os = "macos")]
use tauri::AboutMetadata;

#[derive(Serialize)]
pub struct Accelerator {
    command: bool,
    shift: bool,
    key: String
}

impl Accelerator {
    fn combo(&self) -> String {
        let command_leading = if self.command {
            "CmdOrCtrl+"
        } else {
            ""
        };

        let shift_leading = if self.shift {
            "Shift+"
        } else {
            ""
        };

        format!("{}{}{}", command_leading, shift_leading, self.key)
    }

    fn command(key: &str) -> Accelerator {
        Accelerator {
            command: true,
            shift: false,
            key: key.into()
        }
    }
}

enum MenuOptions {
    NewTab,
    OpenFile,
    CloseTab,
    Save,
    SaveAs,
    Find,
    Disassemble,
    Assemble,
    Export,
    Build,
    Run,
    Step,
    Pause,
    Stop,
    ToggleConsole,
}

impl ToString for MenuOptions {
    fn to_string(&self) -> String {
        match self {
            MenuOptions::NewTab => "new-tab",
            MenuOptions::OpenFile => "open-file",
            MenuOptions::CloseTab => "close-tab",
            MenuOptions::Save => "save",
            MenuOptions::SaveAs => "save-as",
            MenuOptions::Find => "find",
            MenuOptions::Disassemble => "disassemble",
            MenuOptions::Assemble => "assemble",
            MenuOptions::Export => "export",
            MenuOptions::Build => "build",
            MenuOptions::Run => "run",
            MenuOptions::Step => "step",
            MenuOptions::Pause => "pause",
            MenuOptions::Stop => "stop",
            MenuOptions::ToggleConsole => "toggle-console",
        }.into()
    }
}

impl FromStr for MenuOptions {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "new-tab" => MenuOptions::NewTab,
            "open-file" => MenuOptions::OpenFile,
            "close-tab" => MenuOptions::CloseTab,
            "save" => MenuOptions::Save,
            "save-as" => MenuOptions::SaveAs,
            "find" => MenuOptions::Find,
            "disassemble" => MenuOptions::Disassemble,
            "assemble" => MenuOptions::Assemble,
            "export" => MenuOptions::Export,
            "build" => MenuOptions::Build,
            "run" => MenuOptions::Run,
            "step" => MenuOptions::Step,
            "pause" => MenuOptions::Pause,
            "stop" => MenuOptions::Stop,
            "toggle-console" => MenuOptions::ToggleConsole,
            _ => return Err(())
        })
    }
}

impl MenuOptions {
    fn label(&self) -> &'static str {
        match self {
            MenuOptions::NewTab => "New Tab",
            MenuOptions::OpenFile => "Open File",
            MenuOptions::CloseTab => "Close Tab",
            MenuOptions::Save => "Save",
            MenuOptions::SaveAs => "Save As",
            MenuOptions::Find => "Find",
            MenuOptions::Disassemble => "Disassemble Elf",
            MenuOptions::Assemble => "Assemble Elf",
            MenuOptions::Export => "Export Elf",
            MenuOptions::Build => "Build",
            MenuOptions::Run => "Run",
            MenuOptions::Step => "Step",
            MenuOptions::Pause => "Pause",
            MenuOptions::Stop => "Stop",
            MenuOptions::ToggleConsole => "Toggle Console",
        }
    }

    fn accelerator(&self) -> Option<Accelerator> {
        Some(match self {
            MenuOptions::NewTab => Accelerator::command("N"),
            MenuOptions::OpenFile => Accelerator::command("O"),
            MenuOptions::CloseTab => Accelerator::command("W"),
            MenuOptions::Save => Accelerator::command("S"),
            MenuOptions::SaveAs => Accelerator { command: true, shift: true, key: "S".into() },
            MenuOptions::Find => Accelerator::command("F"),
            MenuOptions::Build => Accelerator::command("B"),
            MenuOptions::Run => Accelerator::command("K"),
            MenuOptions::Step => Accelerator::command("L"),
            MenuOptions::Pause => Accelerator::command("J"),
            MenuOptions::Stop => Accelerator::command("P"),
            MenuOptions::ToggleConsole => Accelerator::command("T"),
            _ => return None
        })
    }

    fn make_item(&self) -> CustomMenuItem {
        let item = CustomMenuItem::new(self.to_string(), self.label());

        if let Some(accelerator) = self.accelerator() {
            item.accelerator(accelerator.combo())
        } else {
            item
        }
    }
}

#[derive(Serialize)]
pub struct MenuOptionsData {
    event: String,
    accelerator: Accelerator
}

impl TryFrom<MenuOptions> for MenuOptionsData {
    type Error = ();

    fn try_from(value: MenuOptions) -> Result<Self, Self::Error> {
        Ok(MenuOptionsData {
            event: value.to_string(),
            accelerator: value.accelerator().ok_or(())?
        })
    }
}

pub fn get_platform_emulated_shortcuts() -> Vec<MenuOptionsData> {
    #[cfg(target_os = "windows")]
    return vec![
        MenuOptions::NewTab.try_into(),
        MenuOptions::OpenFile.try_into(),
        MenuOptions::CloseTab.try_into(),
        MenuOptions::Save.try_into(),
        MenuOptions::SaveAs.try_into(),
        MenuOptions::Find.try_into(),
        MenuOptions::Disassemble.try_into(),
        MenuOptions::Assemble.try_into(),
        MenuOptions::Export.try_into(),
        MenuOptions::Build.try_into(),
        MenuOptions::Run.try_into(),
        MenuOptions::Step.try_into(),
        MenuOptions::Pause.try_into(),
        MenuOptions::Stop.try_into(),
        MenuOptions::ToggleConsole.try_into(),
    ].into_iter()
        .filter_map(|x| x.ok())
        .collect();

    #[cfg(not(target_os = "windows"))]
    return vec![];
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

        menu = menu.add_submenu(Submenu::new("Saturn", Menu::new()
            .add_native_item(MenuItem::About("Saturn".into(), meta))
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Hide)
            .add_native_item(MenuItem::HideOthers)
            .add_native_item(MenuItem::ShowAll)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Quit),
        ));
    }

    menu = menu.add_submenu(Submenu::new("File", Menu::new()
        .add_item(MenuOptions::NewTab.make_item())
        .add_item(MenuOptions::OpenFile.make_item())
        .add_item(MenuOptions::CloseTab.make_item())
        .add_native_item(MenuItem::Separator)
        .add_item(MenuOptions::Save.make_item())
        .add_item(MenuOptions::SaveAs.make_item())
        .add_native_item(MenuItem::Separator)
        // .add_item(MenuOptions::Export.make_item())
        .add_item(MenuOptions::Assemble.make_item())
        .add_item(MenuOptions::Disassemble.make_item())
    ));

    // windows unsupported for some of these, hopefully this wont cause a crash
    menu = menu.add_submenu(Submenu::new("Edit", Menu::new()
        .add_native_item(MenuItem::Cut)
        .add_native_item(MenuItem::Copy)
        .add_native_item(MenuItem::Paste)
        .add_native_item(MenuItem::Separator)
        .add_native_item(MenuItem::Undo)
        .add_item(MenuOptions::Find.make_item())
        .add_native_item(MenuItem::SelectAll)
    ));

    menu = menu.add_submenu(Submenu::new("Build", Menu::new()
        .add_item(MenuOptions::Build.make_item())
        .add_item(MenuOptions::Run.make_item())
        .add_native_item(MenuItem::Separator)
        .add_item(MenuOptions::Step.make_item())
        .add_item(MenuOptions::Pause.make_item())
        .add_item(MenuOptions::Stop.make_item())
    ));

    menu = menu.add_submenu(Submenu::new("Window", Menu::new()
        .add_native_item(MenuItem::Minimize)
        .add_item(MenuOptions::ToggleConsole.make_item())
        .add_native_item(MenuItem::CloseWindow)
    ));

    menu
}

pub fn handle_event(event: WindowMenuEvent<Wry>) {
    let catch_emit = |result: tauri::Result<()>| {
        if let Err(_) = result {
            eprintln!("Failed to emit event from {} menu option", event.menu_item_id());
        }
    };

    let emit_normal = |name: &str| {
        catch_emit(event.window().emit(name, ()))
    };

    let Ok(item) = MenuOptions::from_str(event.menu_item_id()) else {
        return eprintln!("Unknown menu ID: {}", event.menu_item_id())
    };

    emit_normal(&item.to_string())
}
