use crate::restic::restore::{progress::Progress, RestoreId};

use super::fragment::ProgressFragment;
use askama::Template;

#[derive(Template)]
#[template(path = "progress/page.html")]
pub struct ProgressPage {
    refresh_url: String,
    sse_url: String,
    data: ProgressFragment,
}

impl ProgressPage {
    pub fn title(&self) -> String {
        // TODO Make it show the live percentage! :D
        format!("Restore progress")
    }

    pub fn new(id: RestoreId, progress: Progress) -> Self {
        Self {
            refresh_url: format!("/restore/{id}"),
            sse_url: format!("/restore/{id}/progress"),
            data: progress.into(),
        }
    }
}
