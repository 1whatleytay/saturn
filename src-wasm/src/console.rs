use saturn_backend::syscall::ConsoleHandler;

pub struct WasmConsole { }

impl ConsoleHandler for WasmConsole {
    fn print(&mut self, _text: &str, _error: bool) {
        todo!()
    }
}
