use std::path::Path;

use super::RestoreContent;
use crate::{
    restic::repository::{Entry, EntryKind, Snapshot},
    restic::restore::destination::RestoreDestination,
    Result,
};

pub struct RestorePlan<'s, Entries: Iterator<Item = Entry>> {
    pub(super) snapshot: &'s Snapshot,
    pub(super) entries: Entries,

    pub(super) source: Entry,
    pub(super) content: RestoreContent,
}

impl<'s, Entries: Iterator<Item = Entry>> RestorePlan<'s, Entries> {
    pub fn execute(self, mut destination: impl RestoreDestination) -> Result<()> {
        for entry in self.entries {
            match entry.kind {
                EntryKind::Directory => {
                    destination.add_dir(entry.path)?;
                }
                EntryKind::File => {
                    let node = self.snapshot.node(&entry.path)?;
                    let mut writer = destination.add_file(entry.path)?;
                    self.snapshot.repo.dump(&node, &mut writer)?;
                }
            }
        }

        Ok(())
    }

    pub fn content(&self) -> &RestoreContent {
        &self.content
    }

    pub fn source(&self) -> &Entry {
        &self.source
    }
}

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
