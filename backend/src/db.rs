use crate::error::{ApiError, ApiResult};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: Arc<Mutex<Connection>>,
}

pub(crate) async fn with_db<T, F>(state: &AppState, operation: F) -> ApiResult<T>
where
    T: Send + 'static,
    F: FnOnce(&mut Connection) -> ApiResult<T> + Send + 'static,
{
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let mut connection = db.lock().map_err(|_| ApiError::internal())?;
        operation(&mut connection)
    })
    .await
    .map_err(|_| ApiError::internal())?
}
