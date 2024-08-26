use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures::channel::oneshot::Receiver;
use saturn_backend::syscall::TimeHandler;
use futures::channel::oneshot;
use tokio::time::sleep;

pub struct TokioTimeHandler { }

impl TokioTimeHandler {
    pub fn new() -> Self { Self { } }
}

impl TimeHandler for TokioTimeHandler {
    fn time(&self) -> Option<Duration> {
        SystemTime::now().duration_since(UNIX_EPOCH).ok()
    }

    fn sleep(&self, duration: Duration) -> Receiver<()> {
        let (sender, receiver) = oneshot::channel();

        tokio::spawn(async move {
            sleep(duration).await;

            sender.send(())
        });

        receiver
    }
}
