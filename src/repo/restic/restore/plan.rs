use super::RestoreContent;
use crate::{
    repo::{Entry, EntryKind, Result, Snapshot},
    restore::RestoreDestination,
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
