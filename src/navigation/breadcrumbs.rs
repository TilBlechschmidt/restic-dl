use crate::{
    helper::path_to_url,
    repo::{Repository, Snapshot},
};
use askama::Template;
use std::path::{Path, PathBuf};

#[derive(Template)]
#[template(path = "breadcrumbs.html")]
pub struct Breadcrumbs(Vec<Breadcrumb>);

pub struct Breadcrumb {
    url: String,
    kind: BreadcrumbKind,
}

enum BreadcrumbKind {
    Repository { name: String },
    Snapshot { id: String },
    Directory { name: String },
}

impl<P: AsRef<Path>> From<(&Snapshot, P)> for Breadcrumbs {
    fn from((snapshot, path): (&Snapshot, P)) -> Self {
        let repository: Breadcrumb = snapshot.repo().into();
        let snapshot: Breadcrumb = snapshot.into();

        let mut cumulative_path = PathBuf::new();
        let mut breadcrumbs: Vec<_> = path
            .as_ref()
            .components()
            .skip_while(|x| x.as_os_str() == "/")
            .map(|component| {
                let name = component.as_os_str().to_string_lossy().to_string();
                cumulative_path.push(component);

                Breadcrumb {
                    url: format!("{}/{}", &snapshot.url, path_to_url(&cumulative_path)),
                    kind: BreadcrumbKind::Directory { name },
                }
            })
            .collect();

        breadcrumbs.insert(0, repository);
        breadcrumbs.insert(1, snapshot);

        Self(breadcrumbs)
    }
}

impl From<&Snapshot> for Breadcrumbs {
    fn from(snapshot: &Snapshot) -> Self {
        Self(vec![
            Breadcrumb::from(snapshot.repo()),
            Breadcrumb::from(snapshot),
        ])
    }
}

impl From<&Snapshot> for Breadcrumb {
    fn from(snapshot: &Snapshot) -> Self {
        let repo = snapshot.repo().name();
        let id = snapshot.id()[0..8].to_string();

        Breadcrumb {
            url: format!("/browse/{repo}/{id}"),
            kind: BreadcrumbKind::Snapshot { id },
        }
    }
}

impl From<&Repository> for Breadcrumb {
    fn from(repository: &Repository) -> Self {
        let name = repository.name().to_string();

        Self {
            url: format!("/browse/{name}"),
            kind: BreadcrumbKind::Repository { name },
        }
    }
}
