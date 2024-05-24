use super::{LinkFragment, LinkPage, LinkPartial};
use fast_qr::convert::{svg::SvgBuilder, Builder, Shape};
use fast_qr::qr::QRBuilder;

impl LinkPage {
    pub fn title(&self) -> &str {
        "Share link"
    }

    pub fn new(url: String) -> Self {
        Self {
            link: LinkPartial::new(url),
        }
    }
}

impl LinkFragment {
    pub fn new(url: String) -> Self {
        Self {
            link: LinkPartial::new(url),
        }
    }
}

impl LinkPartial {
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
