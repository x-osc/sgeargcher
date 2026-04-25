use crate::scrapers::brave;

mod scrapers;

#[tokio::main]
async fn main() {
    let query = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "goats".to_string());

    match brave::search(&query).await {
        Ok(results) if results.is_empty() => println!("no results"),
        Ok(results) => {
            for r in results.iter() {
                println!("{}", r.title);
                println!("{}\n", r.url);
                println!("{}\n", r.description);
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
