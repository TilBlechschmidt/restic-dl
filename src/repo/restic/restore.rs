use super::{Entry, Repository, Snapshot};
use crate::repo::Result;
use rustic_core::{
    repofile::{Node, NodeType},
    LocalDestination, LsOptions, RestoreOptions, RusticResult,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreContent {
    pub size: u64,
    pub files: u64,
    pub directories: u64,
}

pub struct RestorePlan<Source: Iterator<Item = RusticResult<(PathBuf, Node)>>> {
    inner: rustic_core::RestorePlan,
    options: RestoreOptions,
    source: Source,
    destination: LocalDestination,

    entry: Entry,
    repo: Repository,
}

impl<Source: Iterator<Item = RusticResult<(PathBuf, Node)>>> RestorePlan<Source> {
    pub fn execute(self) -> Result<()> {
        self.repo
            .restore(self.inner, &self.options, self.source, &self.destination)?;

        Ok(())
    }

    pub fn content(&self) -> RestoreContent {
        RestoreContent {
            size: self.inner.restore_size,
            files: self.inner.stats.files.restore,
            directories: self.inner.stats.dirs.restore,
        }
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }
}

impl Snapshot {
    pub fn restore(
        &self,
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> Result<RestorePlan<impl Iterator<Item = RusticResult<(PathBuf, Node)>> + '_>> {
        let node = self.node(&source)?;
        let entry = Entry::new(node.clone(), source.as_ref().into())?;

        let source = self
            .repo
            .ls(&node, &LsOptions::default())?
            .filter(|r| match r {
                Ok((_, node)) => {
                    node.node_type == NodeType::File || node.node_type == NodeType::Dir
                }
                Err(_) => false,
            });

        let destination = LocalDestination::new(
            destination
                .as_ref()
                .to_str()
                .expect("attempted to restore into directory with invalid path name"),
            true,
            !node.is_dir(),
        )?;

        let options = RestoreOptions::default().no_ownership(true);

        let plan = self
            .repo
            .prepare_restore(&options, source.clone(), &destination, false)?;

        Ok(RestorePlan {
            inner: plan,
            options,
            source,
            destination,

            entry,
            repo: self.repo.clone(),
        })
    }
}
