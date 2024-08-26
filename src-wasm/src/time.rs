use std::time::Duration;
use saturn_backend::syscall::TimeHandler;
use futures::channel::oneshot;

pub struct WasmTime { }

impl TimeHandler for WasmTime {
    fn time(&self) -> Option<Duration> {
        todo!()
    }

    fn sleep(&self, duration: Duration) -> oneshot::Receiver<()> {
        todo!()
    }
}
