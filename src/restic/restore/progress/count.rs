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
