use std::time::{Duration, SystemTime, UNIX_EPOCH};
use saturn_backend::syscall::TimeHandler;
use tokio::time::sleep;
use async_trait::async_trait;

pub struct TokioTimeHandler { }

impl TokioTimeHandler {
    pub fn new() -> Self { Self { } }
}

#[async_trait]
impl TimeHandler for TokioTimeHandler {
    fn time(&self) -> Option<Duration> {
        SystemTime::now().duration_since(UNIX_EPOCH).ok()
    }

    async fn sleep(&self, duration: Duration) {
        sleep(duration).await;
    }
}
