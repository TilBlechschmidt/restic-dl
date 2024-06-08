use crate::args::ServerArgs;
use listenfd::ListenFd;

mod extract;
mod middleware;
mod navigation;
mod route;

pub use middleware::{repository::CookieParameters, session::SessionCache};
use tokio::net::TcpListener;

pub async fn serve(args: ServerArgs) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut listenfd = ListenFd::from_env();

    let listener = if let Some(listener) = listenfd.take_tcp_listener(0)? {
        println!("Using provided socket, address and port from args are ignored");
        TcpListener::from_std(listener)?
    } else {
        TcpListener::bind(args.address).await?
    };

    println!("Listening on {}", listener.local_addr()?);

    let app = args.into_layers(route::router());
    axum::serve(listener, app).await?;

    Ok(())
}
