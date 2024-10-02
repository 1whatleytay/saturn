use std::time::Duration;
use saturn_backend::syscall::TimeHandler;
use futures::channel::oneshot;
use async_trait::async_trait;

pub struct WasmTime { }

#[async_trait]
impl TimeHandler for WasmTime {
    fn time(&self) -> Option<Duration> {
        Some(Duration::from_millis(js_sys::Date::now() as u64))
    }

    async fn sleep(&self, duration: Duration) {
        let (sender, receiver) = oneshot::channel::<()>();

        {
            // Is this going to panic?
            gloo_timers::callback::Timeout::new(duration.as_millis() as u32, move || {
                sender.send(()).ok();
            }).forget();
        }

        receiver.await.ok();
    }
}
