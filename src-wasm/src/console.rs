use std::sync::Arc;
use saturn_backend::syscall::ConsoleHandler;
use crate::EventHandler;

pub struct WasmConsole {
    pub events: Arc<EventHandler>
}

impl ConsoleHandler for WasmConsole {
    fn print(&mut self, text: &str, error: bool) {
        self.events.send_console_write(text, error)
    }
}
