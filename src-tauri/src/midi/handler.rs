use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use tauri::{AppHandle, Manager, Wry};
use tauri::api::path::app_local_data_dir;
use crate::midi::install_instruments;
use crate::midi::instruments::to_instrument;
use crate::syscall::{MidiHandler, MidiRequest};

#[derive(Clone, Serialize)]
struct MidiNote {
    instrument: String,
    note: u64,
    duration: u64,
    volume: u64
}

pub struct ForwardMidi {
    app: AppHandle<Wry>,
    installed: HashSet<u32>
}

impl ForwardMidi {
    async fn install_async(root: Arc<Mutex<ForwardMidi>>, instrument: u32) -> bool {
        let Some(instrument) = to_instrument(instrument as usize) else { return false };
        let instruments = Some(vec![instrument.into()]);

        let handle = root.lock().unwrap().app.clone();

        install_instruments(None, instruments, &handle).await.is_some()
    }
}

impl MidiHandler for Arc<Mutex<ForwardMidi>> {
    fn play(&mut self, request: &MidiRequest) {
        let Some(instrument) = to_instrument(request.instrument as usize) else { return };

        self.lock().unwrap().app.emit_all("play_midi", MidiNote {
            instrument: instrument.into(),
            note: request.pitch as u64,
            duration: request.duration as u64,
            volume: request.volume as u64
        }).ok();
    }

    fn install(&mut self, instrument: u32) -> Pin<Box<dyn Future<Output=bool> + Send>> {
        let clone = self.clone();

        Box::into_pin(Box::new(async move {
            ForwardMidi::install_async(clone, instrument).await
        }))
    }

    fn installed(&mut self, instrument: u32) -> bool {
        let mut lock = self.lock().unwrap();

        if lock.installed.contains(&instrument) {
            return true
        }

        let Some(name) = to_instrument(instrument as usize) else { return false };

        let Some(mut directory) = app_local_data_dir(&lock.app.config()) else {
            return false
        };

        directory.push(format!("{}-mp3.js", name));

        let result = directory.exists();

        if result {
            lock.installed.insert(instrument);
        }

        result
    }
}

pub fn forward_midi(app: AppHandle<Wry>) -> Box<dyn MidiHandler + Send> {
    Box::new(Arc::new(Mutex::new(ForwardMidi { app, installed: HashSet::new() })))
}
