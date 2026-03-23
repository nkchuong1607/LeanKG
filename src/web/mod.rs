pub mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/graph", get(handlers::graph))
        .route("/browse", get(handlers::browse))
        .route("/docs", get(handlers::docs))
        .route("/quality", get(handlers::quality))
        .route("/api/query", post(handlers::api_query));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("Web UI listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
