use super::highlight::highlight;
use crate::repo::{Result, Snapshot};
use askama::{Html, MarkupDisplay, Template};
use std::path::PathBuf;

#[derive(Template)]
#[template(path = "frag/preview.html")]
pub struct FilePreview {
    content: Option<PreviewContent>,
}

struct PreviewContent {
    text: MarkupDisplay<Html, String>,
    truncated_by: u64,
}

impl FilePreview {
    pub fn new(snapshot: &Snapshot, path: &PathBuf, size_limit: u64) -> Result<Self> {
        let file = snapshot.read(path, Some(size_limit))?;

        let content = String::from_utf8(file.data).ok().map(|text| {
            let highlighted = highlight(
                &text,
                &path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            )
            .unwrap_or(text);

            let html = format!(
                "<code>{}</code>",
                highlighted.replace('\n', "</code><code>")
            );

            PreviewContent {
                text: MarkupDisplay::new_safe(html, Html),
                truncated_by: file.truncated_by,
            }
        });

        Ok(Self { content })
    }
}
