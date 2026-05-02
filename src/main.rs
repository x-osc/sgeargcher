mod engine;
mod utils;
mod web;

#[tokio::main]
async fn main() {
    web::run().await;
}
