use axum::Router;
use axum_embed::{FallbackBehavior, ServeEmbed};
use rust_embed::RustEmbed;

#[derive(RustEmbed, Clone)]
#[folder = "assets/"]
struct Assets;

pub fn routes() -> Router<()> {
    Router::new().nest_service(
        "/assets",
        ServeEmbed::<Assets>::with_parameters(None, FallbackBehavior::NotFound, None),
    )
}
