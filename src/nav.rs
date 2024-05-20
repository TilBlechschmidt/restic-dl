use crate::{filters, repo};
use askama::Template;

pub trait Breadcrumb: Template {}
pub type Breadcrumbs = Vec<dyn Breadcrumb>;

impl Breadcrumb for Home {}
impl Breadcrumb for Repositories {}
impl Breadcrumb for Snapshots {}
impl Breadcrumb for FileTree {}

#[derive(Template)]
#[template(path = "frag/nav.html", block = "home")]
pub struct Home;

#[derive(Template)]
#[template(path = "frag/nav.html", block = "browse")]
pub struct Repositories;

#[derive(Template)]
#[template(path = "frag/nav.html", block = "repo")]
pub struct Snapshots(pub repo::Repository);

#[derive(Template)]
#[template(path = "frag/nav.html", block = "snapshot")]
pub struct FileTree(pub repo::Snapshot);

impl Snapshots {
    fn repository(&self) -> &str {
        self.0.name()
    }
}

impl FileTree {
    fn id(&self) -> String {
        self.0.id().to_string()
    }

    fn repository(&self) -> &str {
        self.0.repo().name()
    }
}
