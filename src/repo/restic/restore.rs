use super::{Entry, EntryKind, Snapshot};
use crate::repo::Result;
use std::path::Path;

mod content;
mod plan;

pub use content::RestoreContent;
pub use plan::RestorePlan;

impl Snapshot {
    pub fn restore(
        &self,
        source: impl AsRef<Path>,
    ) -> Result<RestorePlan<impl Iterator<Item = Entry> + '_>> {
        let target = self.entry(&source)?;
        let entries = self.enumerate(source, true)?;

        let content = match target.kind {
            EntryKind::File => RestoreContent::File { size: target.size },
            EntryKind::Directory => {
                let (size, files, directories) = entries.clone().fold(
                    (0, 0, 0),
                    |(size, files, directories), entry| match entry.kind {
                        EntryKind::File => (size + entry.size, files + 1, directories),
                        EntryKind::Directory => (size + entry.size, files, directories + 1),
                    },
                );

                RestoreContent::Archive {
                    size,
                    files,
                    directories,
                }
            }
        };

        Ok(RestorePlan {
            snapshot: self,
            entries,

            source: target,
            content,
        })
    }
}
