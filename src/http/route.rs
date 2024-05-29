use axum::Router;

mod browse;
mod restore;

pub fn router() -> Router<()> {
    Router::new()
        .nest("/restore", restore::routes())
        .merge(browse::routes())
}
