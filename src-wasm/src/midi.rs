use std::future::Future;
use std::pin::Pin;
use saturn_backend::syscall::{MidiHandler, MidiRequest};

pub struct WasmMidi { }

impl MidiHandler for WasmMidi {
    fn play(&mut self, _request: &MidiRequest, _sync: bool) {
        todo!()
    }

    fn install(&mut self, _instrument: u32) -> Pin<Box<dyn Future<Output=bool> + Send>> {
        todo!()
    }

    fn installed(&mut self, _instrument: u32) -> bool {
        todo!()
    }
}
