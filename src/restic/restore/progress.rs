mod count;
mod update;
mod writer;

pub(super) use writer::ProgressWriter;

pub use count::ProgressCount;
pub use update::*;

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
