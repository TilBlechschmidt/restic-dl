use super::Directory;
use crate::{helper::filters, navigation::Navigation, repo::EntryKind};
use askama::Template;

#[derive(Template)]
#[template(path = "directory/page.html")]
pub struct Page {
    directory: Directory,
}

impl Page {
    pub fn new(directory: Directory) -> Self {
        Self { directory }
    }

    fn title(&self) -> String {
        "Test".into()
    }
}
