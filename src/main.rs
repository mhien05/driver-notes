#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    driver_notes_api::run().await
}
