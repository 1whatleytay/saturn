use send_wrapper::SendWrapper;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use saturn_backend::midi::note::MidiNote;

#[wasm_bindgen]
pub struct EventHandler {
    on_console_write: SendWrapper<js_sys::Function>,
    on_midi_play: SendWrapper<js_sys::Function>,
}

#[wasm_bindgen]
impl EventHandler {
    #[wasm_bindgen(constructor)]
    pub fn new(
        on_console_write: js_sys::Function,
        on_midi_play: js_sys::Function,
    ) -> EventHandler {
        EventHandler {
            on_console_write: SendWrapper::new(on_console_write),
            on_midi_play: SendWrapper::new(on_midi_play)
        }
    }
}

impl EventHandler {
    pub fn send_console_write(&self, text: &str, error: bool) {
        self.on_console_write.call2(
            &JsValue::UNDEFINED,
            &JsValue::from_str(text),
            &JsValue::from_bool(error)
        ).ok();
    }

    pub fn send_midi_play(&self, note: MidiNote) {
        self.on_midi_play.call1(
            &JsValue::UNDEFINED,
            &serde_wasm_bindgen::to_value(&note).unwrap()
        ).ok();
    }
}
