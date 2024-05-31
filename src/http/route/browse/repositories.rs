use axum::response::{Html, IntoResponse};

// TODO Add global authentication (also put it into the Repository extractor)
pub async fn route() -> impl IntoResponse {
    Html(
        r#"<form method="POST" action="/tmp"><input name="password" /><button>Login</button></form>"#,
    )
}
