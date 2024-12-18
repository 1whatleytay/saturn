use crate::EventHandler;
use saturn_backend::midi::instruments::to_instrument;
use saturn_backend::midi::note::MidiNote;
use saturn_backend::syscall::{MidiHandler, MidiRequest};
use std::pin::Pin;
use std::{future::Future, sync::Arc};

pub struct WasmMidi {
    pub events: Arc<EventHandler>,
}

impl MidiHandler for WasmMidi {
    fn play(&mut self, request: &MidiRequest, _sync: bool) {
        let Some(name) = to_instrument(request.instrument as usize) else {
            return;
        };
        self.events.send_midi_play(MidiNote {
            sync: _sync,
            name: name.to_owned(),
            instrument: request.instrument as u64,
            note: request.pitch as u64,
            duration: request.duration as f64 / 1000f64,
            volume: request.volume as u64,
        })
    }

    fn install(&mut self, _instrument: u32) -> Pin<Box<dyn Future<Output = bool> + Send>> {
        return Box::pin(async { true });
    }

    fn installed(&mut self, _instrument: u32) -> bool {
        true
    }
}
