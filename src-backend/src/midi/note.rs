use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct MidiNote {
    pub sync: bool,
    pub instrument: u64,
    pub name: String,
    pub note: u64,
    pub duration: f64,
    pub volume: u64,
}
