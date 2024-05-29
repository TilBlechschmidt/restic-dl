
use tokio::sync::broadcast;

use super::{count::ProgressCount, Progress, Status};
use std::{
    ops::AddAssign,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct ProgressTracker {
    channel: broadcast::Sender<Progress>,
    state: Arc<Mutex<Progress>>,
}

#[derive(Clone)]
pub struct ProgressReceiver(ProgressTracker);

pub struct ProgressSubscription(broadcast::Receiver<Progress>);

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            channel: broadcast::channel(16).0,
            state: Arc::new(Mutex::new(Progress::default())),
        }
    }

    /// Overwrites the initial state of the tracker so progress can be added.
    /// The primary purpose is to update the totals once they are known but
    /// before any progress has been made.
    pub fn set_state(&mut self, new_state: Progress) {
        let mut state = self.state.lock().expect("progress mutex poisoned");
        *state = new_state;
        _ = self.channel.send(new_state);
    }

    pub fn set_status(&mut self, status: Status) {
        let mut state = self.state.lock().expect("progress mutex poisoned");
        state.status = status;
        _ = self.channel.send(*state);
    }

    pub fn handle(&self) -> ProgressReceiver {
        ProgressReceiver(self.clone())
    }
}

impl<C: Into<ProgressCount>> AddAssign<C> for ProgressTracker {
    fn add_assign(&mut self, count: C) {
        let count: ProgressCount = count.into();

        let mut state = self.state.lock().expect("progress mutex poisoned");
        *state += count;

        _ = self.channel.send(*state);
    }
}

impl ProgressReceiver {
    pub fn current(&self) -> Progress {
        *self.0.state.lock().expect("progress mutex poisoned")
    }

    pub fn subscribe(&self) -> ProgressSubscription {
        ProgressSubscription(self.0.channel.subscribe())
    }
}

impl ProgressSubscription {
    pub async fn recv(&mut self) -> Result<Progress, broadcast::error::RecvError> {
        self.0.recv().await
    }
}
