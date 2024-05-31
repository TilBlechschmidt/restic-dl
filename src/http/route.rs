use axum::Router;

mod assets;
mod browse;
mod restore;

pub fn router() -> Router<()> {
    Router::new()
        .nest("/restore", restore::routes())
        .merge(browse::routes())
        .merge(assets::routes())
}
