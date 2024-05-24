use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RestoreContent {
    File {
        size: u64,
    },
    Archive {
        size: u64,
        files: u64,
        directories: u64,
    },
}

impl RestoreContent {
    pub const FILE: Self = Self::Archive {
        size: 0,
        files: 1,
        directories: 0,
    };

    pub const DIR: Self = Self::Archive {
        size: 0,
        files: 0,
        directories: 1,
    };

    pub fn data(byte_count: u64) -> Self {
        Self::File { size: byte_count }
    }

    pub fn size(&self) -> u64 {
        match *self {
            RestoreContent::File { size } => size,
            RestoreContent::Archive { size, .. } => size,
        }
    }
}

impl Add for RestoreContent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        use RestoreContent::*;

        match (self, rhs) {
            (File { size: s1 }, File { size: s2 }) => File { size: s1 + s2 },
            (
                Archive {
                    size: s1,
                    files: f1,
                    directories: d1,
                },
                Archive {
                    size: s2,
                    files: f2,
                    directories: d2,
                },
            ) => Archive {
                size: s1 + s2,
                files: f1 + f2,
                directories: d1 + d2,
            },
            (
                Archive {
                    size,
                    files,
                    directories,
                },
                File { size: file_size },
            ) => Archive {
                size: size + file_size,
                files,
                directories,
            },
            _ => panic!("attempted to add non-matching restore contents"),
        }
    }
}
