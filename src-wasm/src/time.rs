use std::time::Duration;
use saturn_backend::syscall::TimeHandler;
use futures::channel::oneshot;

pub struct WasmTime { }

impl TimeHandler for WasmTime {
    fn time(&self) -> Option<Duration> {
        Some(Duration::from_millis(js_sys::Date::now() as u64))
    }

    fn sleep(&self, duration: Duration) -> oneshot::Receiver<()> {
        
    }
}
