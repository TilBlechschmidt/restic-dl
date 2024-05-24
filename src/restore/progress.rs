mod writer;

pub(super) use writer::ProgressWriter;

use crate::repo::RestoreContent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProgressVariable {
    pub current: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Progress {
    pub data: ProgressVariable,
    pub files: Option<ProgressVariable>,
    pub directories: Option<ProgressVariable>,
    pub status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Collecting,
    Restoring,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressUnit {
    Directory,
    File,
    Data,
}

impl ProgressVariable {
    pub fn new(total: u64) -> Self {
        Self { current: 0, total }
    }

    pub fn percentage(&self) -> f64 {
        self.current as f64 / self.total as f64
    }
}

impl From<RestoreContent> for Progress {
    fn from(content: RestoreContent) -> Self {
        match content {
            RestoreContent::File { size } => Self {
                data: ProgressVariable::new(size),
                files: None,
                directories: None,
                status: Status::Restoring,
            },
            RestoreContent::Archive {
                size,
                files,
                directories,
            } => Self {
                data: ProgressVariable::new(size),
                files: Some(ProgressVariable::new(files)),
                directories: Some(ProgressVariable::new(directories)),
                status: Status::Restoring,
            },
        }
    }
}

pub mod count {
    use super::*;
    use std::ops::{AddAssign, Mul};

    /// Certain number of progress units obtained by either
    /// - Calling `.into()` for a count of `1`
    /// - Multiplying a [`ProgressUnit`] with some [`u64`]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ProgressCount {
        pub unit: ProgressUnit,
        pub count: u64,
    }

    impl Mul<u64> for ProgressUnit {
        type Output = ProgressCount;

        fn mul(self, count: u64) -> Self::Output {
            ProgressCount { unit: self, count }
        }
    }

    impl From<ProgressUnit> for ProgressCount {
        fn from(unit: ProgressUnit) -> Self {
            Self { unit, count: 1 }
        }
    }

    impl AddAssign<u64> for ProgressVariable {
        fn add_assign(&mut self, count: u64) {
            self.current += count;
        }
    }

    impl AddAssign<ProgressCount> for Progress {
        fn add_assign(&mut self, c: ProgressCount) {
            match c.unit {
                ProgressUnit::Directory => {
                    *self.directories.as_mut().expect(
                        "attempted to update progress for non-initialized variable `directory`",
                    ) += c.count;
                }
                ProgressUnit::File => {
                    *self.files.as_mut().expect(
                        "attempted to update progress for non-initialized variable `directory`",
                    ) += c.count;
                }
                ProgressUnit::Data => {
                    self.data += c.count;
                }
            }
        }
    }
}

pub mod update {
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
}
