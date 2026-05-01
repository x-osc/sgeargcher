use axum::{Router, routing::get};

mod index;
mod search;

pub async fn run() {
    let app = Router::new()
        .route("/", get(index::get))
        .route("/search", get(search::get));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
