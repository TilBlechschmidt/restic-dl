use crate::args::SiteUrl;
use crate::http::extract::HxRequest;
use crate::restic::restore::RestoreId;
use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::extract::Path;
use axum::Extension;
use fast_qr::convert::{svg::SvgBuilder, Builder, Shape};
use fast_qr::qr::QRBuilder;

pub async fn route(
    fragment: HxRequest,
    Path(id): Path<RestoreId>,
    Extension(SiteUrl(site_url)): Extension<SiteUrl>,
) -> Response {
    let url = format!("{site_url}/restore/{id}");

    if *fragment {
        ShareFragment::new(url).into_response()
    } else {
        SharePage::new(url).into_response()
    }
}

#[derive(Template)]
#[template(path = "share/page.html")]
pub struct SharePage {
    share: SharePartial,
}

#[derive(Template)]
#[template(path = "share/fragment.html")]
pub struct ShareFragment {
    share: SharePartial,
}

struct SharePartial {
    url: String,
    svg: String,
}

impl SharePage {
    pub fn title(&self) -> &str {
        "Share link"
    }

    pub fn new(url: String) -> Self {
        Self {
            share: SharePartial::new(url),
        }
    }
}

impl ShareFragment {
    pub fn new(url: String) -> Self {
        Self {
            share: SharePartial::new(url),
        }
    }
}

impl SharePartial {
    fn new(url: String) -> Self {
        // Since we are using a somewhat fixed length of data, we use expect here
        let qrcode = QRBuilder::new(url.as_bytes())
            .build()
            .expect("share URL does not fit into QR code");

        let svg = SvgBuilder::default()
            .shape(Shape::RoundedSquare)
            .to_str(&qrcode);

        Self { url, svg }
    }
}
