mod engine;
mod url;
mod web;

#[tokio::main]
async fn main() {
    web::run().await;
}
