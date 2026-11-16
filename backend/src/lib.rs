mod api;
mod db;
mod error;
mod time;

use rusqlite::Connection;
use std::{
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let database_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "data/driver-notes.db".into());
    if let Some(parent) = std::path::Path::new(&database_path)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let connection = Connection::open(database_path)?;
    connection.execute_batch("PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;")?;
    connection.execute_batch(include_str!("../migrations/001_init.sql"))?;
    let address: SocketAddr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()?;
    let origin = env::var("FRONTEND_ORIGIN").ok();
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Driver Notes API listening on {address}");
    axum::serve(
        listener,
        api::router(
            db::AppState {
                db: Arc::new(Mutex::new(connection)),
            },
            origin.as_deref(),
        ),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests;
