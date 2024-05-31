use axum::response::{Html, IntoResponse};

pub async fn route() -> impl IntoResponse {
    Html(r#"Hi"#)
}
