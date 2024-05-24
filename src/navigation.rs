use askama::{DynTemplate, Template};

mod breadcrumbs;
pub use breadcrumbs::Breadcrumbs;

#[derive(Template)]
#[template(path = "navigation.html")]
pub struct Navigation {
    content: String,
    buttons: Option<String>,
}

impl Navigation {
    pub fn new(content: &dyn DynTemplate) -> Self {
        Self {
            content: content.dyn_render().expect("navigation content render"),
            buttons: None,
        }
    }

    pub fn with_buttons(mut self, buttons: &dyn DynTemplate) -> Self {
        self.buttons = Some(buttons.dyn_render().expect("navigation buttons render"));
        self
    }
}
