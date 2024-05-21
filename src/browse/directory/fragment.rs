use super::Directory;
use crate::repo::EntryKind;
use askama::Template;

#[derive(Template)]
#[template(path = "directory/fragment.html")]
pub struct Fragment {
    directory: Directory,
}

impl Fragment {
    pub fn new(directory: Directory) -> Self {
        Self { directory }
    }
}
