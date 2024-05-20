use crate::{
    filters,
    repo::{Repository, Result, SnapshotInfo},
};
use askama::Template;

#[derive(Template)]
#[template(path = "page/browse/snapshots.html")]
pub struct SnapshotListTemplate {
    repository: Repository,
    snapshots: Vec<SnapshotInfo>,
}

pub async fn list(repository: Repository) -> Result<SnapshotListTemplate> {
    let snapshots = repository.list_snapshots()?;

    Ok(SnapshotListTemplate {
        repository,
        snapshots,
    })
}

impl SnapshotListTemplate {
    fn title(&self) -> &'static str {
        "Snapshots"
    }
}
