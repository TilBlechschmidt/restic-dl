use crate::{
    helper::filters,
    http::navigation::Navigation,
    restic::repository::{Repository, Snapshot},
    Result,
};
use askama::Template;
use axum::{response::IntoResponse, Extension};

pub async fn route(Extension(repository): Extension<Repository>) -> Result<impl IntoResponse> {
    let mut snapshots = repository.list_snapshots()?;

    snapshots.sort_by(|l, r| r.info().time.cmp(&l.info().time));

    // dbg!(snapshots.iter().map(|s| s.info()).collect::<Vec<_>>());

    Ok(SnapshotsPage {
        repository,
        snapshots,
    })
}

#[derive(Template)]
#[template(path = "browse/snapshots.html")]
struct SnapshotsPage {
    repository: Repository,
    snapshots: Vec<Snapshot>,
}

impl SnapshotsPage {
    fn title(&self) -> &'static str {
        "Snapshots"
    }
}
