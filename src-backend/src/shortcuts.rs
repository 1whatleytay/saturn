use std::fmt::Display;
use std::str::FromStr;
use serde::Serialize;

#[derive(Serialize)]
pub struct Accelerator {
    command: bool,
    shift: bool,
    key: String,
}

impl Accelerator {
    pub fn combo(&self) -> String {
        let command_leading = if self.command { "CmdOrCtrl+" } else { "" };

        let shift_leading = if self.shift { "Shift+" } else { "" };

        format!("{}{}{}", command_leading, shift_leading, self.key)
    }

    pub fn command(key: &str) -> Accelerator {
        Accelerator {
            command: true,
            shift: false,
            key: key.into(),
        }
    }
}

pub enum MenuOptions {
    NewTab,
    OpenFile,
    CloseTab,
    Save,
    SaveAs,
    Find,
    Disassemble,
    Assemble,
    Export,
    ExportHex,
    Build,
    Run,
    Step,
    Pause,
    Stop,
    ToggleConsole,
    ToggleSettings,
}

impl Display for MenuOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            MenuOptions::NewTab => "new-tab",
            MenuOptions::OpenFile => "open-file",
            MenuOptions::CloseTab => "close-tab",
            MenuOptions::Save => "save",
            MenuOptions::SaveAs => "save-as",
            MenuOptions::Find => "find",
            MenuOptions::Disassemble => "disassemble",
            MenuOptions::Assemble => "assemble",
            MenuOptions::Export => "export",
            MenuOptions::ExportHex => "export-hex",
            MenuOptions::Build => "build",
            MenuOptions::Run => "run",
            MenuOptions::Step => "step",
            MenuOptions::Pause => "pause",
            MenuOptions::Stop => "stop",
            MenuOptions::ToggleConsole => "toggle-console",
            MenuOptions::ToggleSettings => "toggle-settings",
        };

        write!(f, "{}", str)
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
            "export-hex" => MenuOptions::ExportHex,
            "build" => MenuOptions::Build,
            "run" => MenuOptions::Run,
            "step" => MenuOptions::Step,
            "pause" => MenuOptions::Pause,
            "stop" => MenuOptions::Stop,
            "toggle-console" => MenuOptions::ToggleConsole,
            "toggle-settings" => MenuOptions::ToggleSettings,
            _ => return Err(()),
        })
    }
}

impl MenuOptions {
    pub fn label(&self) -> &'static str {
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
            MenuOptions::ExportHex => "Export Regions",
            MenuOptions::Build => "Build",
            MenuOptions::Run => "Run",
            MenuOptions::Step => "Step",
            MenuOptions::Pause => "Pause",
            MenuOptions::Stop => "Stop",
            MenuOptions::ToggleConsole => "Toggle Console",
            MenuOptions::ToggleSettings => "Toggle Settings",
        }
    }

    pub fn accelerator(&self) -> Option<Accelerator> {
        Some(match self {
            MenuOptions::NewTab => Accelerator::command("N"),
            MenuOptions::OpenFile => Accelerator::command("O"),
            MenuOptions::CloseTab => Accelerator::command("W"),
            MenuOptions::Save => Accelerator::command("S"),
            MenuOptions::SaveAs => Accelerator {
                command: true,
                shift: true,
                key: "S".into(),
            },
            MenuOptions::Find => Accelerator::command("F"),
            MenuOptions::Build => Accelerator::command("B"),
            MenuOptions::Run => Accelerator::command("K"),
            MenuOptions::Step => Accelerator::command("L"),
            MenuOptions::Pause => Accelerator::command("J"),
            MenuOptions::Stop => Accelerator::command("P"),
            MenuOptions::ToggleConsole => Accelerator::command("T"),
            MenuOptions::ToggleSettings => Accelerator::command(","),
            _ => return None,
        })
    }
}

#[derive(Serialize)]
pub struct MenuOptionsData {
    pub event: String,
    pub accelerator: Accelerator,
}

impl TryFrom<MenuOptions> for MenuOptionsData {
    type Error = ();

    fn try_from(value: MenuOptions) -> Result<Self, Self::Error> {
        Ok(MenuOptionsData {
            event: value.to_string(),
            accelerator: value.accelerator().ok_or(())?,
        })
    }
}

pub fn get_emulated_shortcuts() -> Vec<MenuOptionsData> {
    [
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
        MenuOptions::ToggleSettings.try_into(),
    ]
        .into_iter()
        .filter_map(|x| x.ok())
        .collect()
}
