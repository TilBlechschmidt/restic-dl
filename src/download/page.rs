use askama::Template;

mod link;
mod progress;

pub use progress::Data;

#[derive(Template)]
#[template(path = "progress/page.html")]
pub struct ProgressPage {
    data: Data,
}

#[derive(Template)]
#[template(path = "link/page.html")]
pub struct LinkPage {
    link: LinkPartial,
}

#[derive(Template)]
#[template(path = "link/fragment.html")]
pub struct LinkFragment {
    link: LinkPartial,
}

struct LinkPartial {
    url: String,
    svg: String,
}
