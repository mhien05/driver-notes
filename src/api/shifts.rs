use super::auth::authenticated_user;
use crate::{
    db::{AppState, with_db},
    error::{ApiError, ApiResult},
    time::{now_ms, valid_time},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Shift {
    pub(crate) id: String,
    pub(crate) started_at: i64,
    pub(crate) ended_at: Option<i64>,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

fn shift_row(row: &Row<'_>) -> rusqlite::Result<Shift> {
    Ok(Shift {
        id: row.get(0)?,
        started_at: row.get(1)?,
        ended_at: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

pub(crate) fn shifts_for(db: &Connection, user_id: &str) -> ApiResult<Vec<Shift>> {
    let mut stmt = db.prepare("SELECT id, started_at, ended_at, created_at, updated_at FROM shifts WHERE user_id = ?1 ORDER BY started_at DESC")
        .map_err(|_| ApiError::internal())?;
    stmt.query_map([user_id], shift_row)
        .map_err(|_| ApiError::internal())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| ApiError::internal())
}

#[utoipa::path(get, path = "/api/v1/shifts", tag = "Ca làm",
    responses((status = 200, body = [Shift]), (status = 401)))]
pub(crate) async fn list_shifts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Shift>>> {
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| shifts_for(db, &user_id).map(Json)).await
}

#[utoipa::path(post, path = "/api/v1/shifts", tag = "Ca làm",
    responses((status = 201, body = Shift), (status = 401), (status = 409)))]
pub(crate) async fn start_shift(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<(StatusCode, Json<Shift>)> {
    let user_id = authenticated_user(&state, &headers).await?;
    let time = now_ms();
    let shift = Shift {
        id: Uuid::new_v4().to_string(),
        started_at: time,
        ended_at: None,
        created_at: time,
        updated_at: time,
    };
    let id = shift.id.clone();
    with_db(&state, move |db| {
        db.execute("INSERT INTO shifts (id, user_id, started_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?3, ?3)",
            params![id, user_id, time]).map_err(|_| ApiError::conflict("An active shift already exists"))?;
        Ok(())
    }).await?;
    Ok((StatusCode::CREATED, Json(shift)))
}

#[utoipa::path(post, path = "/api/v1/shifts/{id}/end", tag = "Ca làm",
    params(("id" = String, Path, description = "ID ca làm")),
    responses((status = 200, body = Shift), (status = 401), (status = 404)))]
pub(crate) async fn end_shift(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<Shift>> {
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| {
        let started_at: i64 = db.query_row("SELECT started_at FROM shifts WHERE id = ?1 AND user_id = ?2 AND ended_at IS NULL",
            params![id, user_id], |row| row.get(0)).optional().map_err(|_| ApiError::internal())?
            .ok_or_else(ApiError::not_found)?;
        let time = now_ms().max(started_at + 1);
        let count = db.execute("UPDATE shifts SET ended_at = ?1, updated_at = ?1 WHERE id = ?2 AND user_id = ?3 AND ended_at IS NULL",
            params![time, id, user_id]).map_err(|_| ApiError::internal())?;
        if count == 0 { return Err(ApiError::not_found()); }
        db.query_row("SELECT id, started_at, ended_at, created_at, updated_at FROM shifts WHERE id = ?1 AND user_id = ?2",
            params![id, user_id], shift_row).map(Json).map_err(|_| ApiError::internal())
    }).await
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ShiftInput {
    /// Thời điểm bắt đầu, Unix milliseconds.
    pub(crate) started_at: i64,
    /// Thời điểm kết thúc, Unix milliseconds; phải sau startedAt.
    pub(crate) ended_at: i64,
}

#[utoipa::path(put, path = "/api/v1/shifts/{id}", tag = "Ca làm",
    params(("id" = String, Path, description = "ID ca làm")),
    request_body = ShiftInput,
    responses((status = 200, body = Shift), (status = 400), (status = 401), (status = 404)))]
pub(crate) async fn replace_shift(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<ShiftInput>,
) -> ApiResult<Json<Shift>> {
    if !valid_time(input.started_at)
        || !valid_time(input.ended_at)
        || input.ended_at <= input.started_at
    {
        return Err(ApiError::bad("Invalid shift times"));
    }
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| {
        let count = db.execute("UPDATE shifts SET started_at = ?1, ended_at = ?2, updated_at = ?3 WHERE id = ?4 AND user_id = ?5 AND ended_at IS NOT NULL",
            params![input.started_at, input.ended_at, now_ms(), id, user_id]).map_err(|_| ApiError::internal())?;
        if count == 0 { return Err(ApiError::not_found()); }
        db.query_row("SELECT id, started_at, ended_at, created_at, updated_at FROM shifts WHERE id = ?1 AND user_id = ?2",
            params![id, user_id], shift_row).map(Json).map_err(|_| ApiError::internal())
    }).await
}

#[utoipa::path(delete, path = "/api/v1/shifts/{id}", tag = "Ca làm",
    params(("id" = String, Path, description = "ID ca làm")),
    responses((status = 204), (status = 401), (status = 404)))]
pub(crate) async fn delete_shift(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| {
        let count = db
            .execute(
                "DELETE FROM shifts WHERE id = ?1 AND user_id = ?2",
                params![id, user_id],
            )
            .map_err(|_| ApiError::internal())?;
        if count == 0 {
            return Err(ApiError::not_found());
        }
        Ok(StatusCode::NO_CONTENT)
    })
    .await
}
