use crate::args::ServerArgs;

mod extract;
mod middleware;
mod navigation;
mod route;

pub use middleware::{repository::CookieParameters, session::SessionCache};

pub async fn serve(args: ServerArgs) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind(args.address).await?;
    let app = args.into_layers(route::router());

    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
