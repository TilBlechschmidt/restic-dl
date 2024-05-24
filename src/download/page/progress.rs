use super::ProgressPage;
use crate::{
    helper::filters,
    restore::progress::{Progress, Status},
};
use askama::Template;
use axum::response::sse::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StatusData {
    progress: usize,
    status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BytesData {
    total: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirectoriesData {
    total: u64,
    remaining: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FilesData {
    total: u64,
    remaining: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Template, Default)]
#[template(path = "progress/data.html")]
pub struct Data {
    status: Option<StatusData>,
    bytes: Option<BytesData>,
    directories: Option<DirectoriesData>,
    files: Option<FilesData>,
}

impl Data {
    pub fn is_empty(&self) -> bool {
        self.status.is_none()
            && self.bytes.is_none()
            && self.directories.is_none()
            && self.files.is_none()
    }

    pub fn delta(&self, previous: &Data) -> Self {
        let mut data = self.clone();

        macro_rules! clear_if_equal {
            ($f:ident) => {
                if data.$f == previous.$f {
                    data.$f = None
                }
            };
        }

        clear_if_equal!(status);
        clear_if_equal!(bytes);
        clear_if_equal!(directories);
        clear_if_equal!(files);

        data
    }
}

impl From<Data> for Event {
    fn from(data: Data) -> Self {
        let data = data.render().expect("failed to render data template");
        Event::default().event("data").data(data)
    }
}

impl From<Progress> for Data {
    fn from(progress: Progress) -> Data {
        let status = Some(StatusData {
            status: progress.status,
            progress: (progress.data.percentage() * 100.0) as usize,
        });

        let bytes = Some(BytesData {
            total: progress.data.total,
        });

        let directories = progress.directories.map(|dirs| DirectoriesData {
            total: dirs.total,
            remaining: dirs.total - dirs.current,
        });

        let files = progress.files.map(|dirs| FilesData {
            total: dirs.total,
            remaining: dirs.total - dirs.current,
        });

        Data {
            status,
            bytes,
            directories,
            files,
        }
    }
}

impl ProgressPage {
    pub fn title(&self) -> String {
        // TODO Make it show the live percentage! :D
        format!("Restore progress")
    }

    pub fn new(progress: Progress) -> Self {
        Self {
            data: progress.into(),
        }
    }
}
