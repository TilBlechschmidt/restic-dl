use axum::{http::header, response::IntoResponse};
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};
use syntect_assets::assets::HighlightingAssets;

thread_local!(pub static ASSETS: HighlightingAssets = HighlightingAssets::from_binary());

pub async fn css() -> impl IntoResponse {
    let styles = ASSETS
        .with(|f| css_for_theme_with_class_style(f.get_theme("Nord"), ClassStyle::Spaced))
        .expect("valid highlighting css");

    ([(header::CONTENT_TYPE, "text/css")], styles)
}

/// Takes the content of a paste and the extension passed in by the viewer and will return the content
/// highlighted in the appropriate format in HTML.
///
/// Returns `None` if the extension isn't supported.
pub fn highlight(content: &str, ext: &str) -> Option<String> {
    ASSETS
        .with(|f| {
            let ss = f.get_syntax_set().expect("syntax set should be present");

            let Some(syntax) = ss.find_syntax_by_extension(ext) else {
                return Ok(None);
            };

            let mut html_generator =
                ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);

            for line in LinesWithEndings(content.trim()) {
                html_generator.parse_html_for_line_which_includes_newline(line)?;
            }

            Ok::<_, syntect::Error>(Some(html_generator.finalize()))
        })
        .unwrap_or_else(|_| Some(content.to_string()))
}

pub struct LinesWithEndings<'a>(&'a str);

impl<'a> Iterator for LinesWithEndings<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            None
        } else {
            let split = self.0.find('\n').map_or(self.0.len(), |i| i + 1);
            let (line, rest) = self.0.split_at(split);
            self.0 = rest;
            Some(line)
        }
    }
}
