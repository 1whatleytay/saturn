use std::cmp::min;
use std::sync::Mutex;
use tokio::sync::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError::Full;

struct ByteChannelReceiver {
    receiver: mpsc::Receiver<Vec<u8>>,
    peek: Option<(usize, Vec<u8>)>
}

impl ByteChannelReceiver {
    fn new(receiver: mpsc::Receiver<Vec<u8>>) -> ByteChannelReceiver {
        ByteChannelReceiver {
            receiver,
            peek: None
        }
    }
}

struct ByteChannelSender {
    sender: mpsc::Sender<Vec<u8>>,
    cache: Vec<Vec<u8>>
}

impl ByteChannelSender {
    fn new(sender: mpsc::Sender<Vec<u8>>) -> ByteChannelSender {
        ByteChannelSender {
            sender,
            cache: vec![]
        }
    }
}

pub struct ByteChannel {
    sender: Mutex<ByteChannelSender>,
    receiver: AsyncMutex<ByteChannelReceiver>
}

pub enum ByteChannelConsumption {
    ConsumeAndContinue,
    ConsumeAndStop,
    IgnoreAndStop
}

impl ByteChannelConsumption {
    fn do_consume(&self) -> bool {
        match self {
            ByteChannelConsumption::ConsumeAndContinue => true,
            ByteChannelConsumption::ConsumeAndStop => true,
            _ => false
        }
    }

    fn do_stop(&self) -> bool {
        match self {
            ByteChannelConsumption::ConsumeAndStop => true,
            ByteChannelConsumption::IgnoreAndStop => true,
            _ => false
        }
    }
}

impl ByteChannel {
    pub fn new() -> ByteChannel {
        // Not using unbounded channels out of lack of trust
        let (sender, receiver) = mpsc::channel(12);

        ByteChannel {
            sender: Mutex::new(ByteChannelSender::new(sender)),
            receiver: AsyncMutex::new(ByteChannelReceiver::new(receiver))
        }
    }

    pub fn send(&self, data: Vec<u8>) {
        let mut state = self.sender.lock().unwrap();

        if let Err(Full(data)) = state.sender.try_send(data) {
            state.cache.push(data)
        }
    }

    pub fn pop_cache(&self) -> Option<Vec<u8>> {
        let mut sender_state = self.sender.lock().unwrap();

        if !sender_state.cache.is_empty() {
            Some(sender_state.cache.remove(0))
        } else {
            None
        }
    }

    async fn take(
        &self, state: &mut AsyncMutexGuard<'_, ByteChannelReceiver>
    ) -> Option<(usize, Vec<u8>)> {
        if let Some(value) = state.peek.take() {
            Some(value)
        } else {
            let result = state.receiver.recv().await;

            result
                .or_else(|| self.pop_cache())
                .map(|x| (0, x))
        }
    }

    pub async fn read(&self, count: usize) -> Option<Vec<u8>> {
        let mut output = vec![];

        while count > output.len() {
            let needs = count - output.len();

            let mut state = self.receiver.lock().await;

            let (index, value) = self.take(&mut state).await?;
            let bytes = min(needs, value.len() - index);

            let end = index + bytes;
            output.extend_from_slice(&value[index .. end]);

            if end < value.len() {
                state.peek = Some((end, value))
            }
        }

        Some(output)
    }

    pub async fn read_until<F: FnMut(u8) -> ByteChannelConsumption>(
        &self, mut f: F
    ) -> Option<Vec<u8>> {
        let mut output = vec![];
        let mut full = false;

        while !full {
            let mut state = self.receiver.lock().await;

            let (index, value) = self.take(&mut state).await?;

            let mut end = index;

            while end < value.len() {
                let consume = f(value[end]);

                if consume.do_consume() {
                    end += 1;
                }

                if consume.do_stop() {
                    full = true;
                    break
                }
            }

            output.extend_from_slice(&value[index .. end]);

            if end < value.len() {
                state.peek = Some((end, value))
            }
        }

        Some(output)
    }
}

