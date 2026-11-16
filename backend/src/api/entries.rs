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

const MAX_MONEY: i64 = 1_000_000_000_000;

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Entry {
    pub(crate) id: String,
    pub(crate) shift_id: Option<String>,
    pub(crate) kind: String,
    pub(crate) delivery_count: Option<i64>,
    pub(crate) expense_category: Option<String>,
    pub(crate) amount_vnd: i64,
    pub(crate) tip_vnd: i64,
    pub(crate) occurred_at: i64,
    pub(crate) note: String,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

fn entry_row(row: &Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        shift_id: row.get(1)?,
        kind: row.get(2)?,
        delivery_count: row.get(3)?,
        expense_category: row.get(4)?,
        amount_vnd: row.get(5)?,
        tip_vnd: row.get(6)?,
        occurred_at: row.get(7)?,
        note: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

pub(crate) fn entries_for(db: &Connection, user_id: &str) -> ApiResult<Vec<Entry>> {
    let mut stmt = db.prepare("SELECT id, shift_id, kind, delivery_count, expense_category, amount_vnd, tip_vnd, occurred_at, note, created_at, updated_at FROM entries WHERE user_id = ?1 ORDER BY occurred_at DESC")
        .map_err(|_| ApiError::internal())?;
    stmt.query_map([user_id], entry_row)
        .map_err(|_| ApiError::internal())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_| ApiError::internal())
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct EntryInput {
    /// ID ca làm, có thể bỏ trống.
    pub(crate) shift_id: Option<String>,
    /// delivery, bonus hoặc expense.
    pub(crate) kind: String,
    /// 1, 2 hoặc 3; chỉ dùng cho delivery.
    pub(crate) delivery_count: Option<i64>,
    /// fuel, food hoặc other; chỉ dùng cho expense.
    pub(crate) expense_category: Option<String>,
    /// Số tiền nguyên VND, lớn hơn 0.
    pub(crate) amount_vnd: i64,
    #[serde(default)]
    /// Tiền tip nguyên VND; chỉ dùng cho delivery.
    pub(crate) tip_vnd: i64,
    /// Thời điểm phát sinh, Unix milliseconds.
    pub(crate) occurred_at: i64,
    #[serde(default)]
    pub(crate) note: String,
}

impl EntryInput {
    pub(crate) fn validate(&self) -> ApiResult<()> {
        if self.amount_vnd <= 0
            || self.amount_vnd > MAX_MONEY
            || self.tip_vnd < 0
            || self.tip_vnd > MAX_MONEY
            || !valid_time(self.occurred_at)
            || self.note.chars().count() > 500
        {
            return Err(ApiError::bad("Invalid entry values"));
        }
        let valid = match self.kind.as_str() {
            "delivery" => {
                matches!(self.delivery_count, Some(1..=3)) && self.expense_category.is_none()
            }
            "bonus" => {
                self.delivery_count.is_none()
                    && self.expense_category.is_none()
                    && self.tip_vnd == 0
            }
            "expense" => {
                self.delivery_count.is_none()
                    && matches!(
                        self.expense_category.as_deref(),
                        Some("fuel" | "food" | "other")
                    )
                    && self.tip_vnd == 0
            }
            _ => false,
        };
        if !valid {
            return Err(ApiError::bad("Invalid entry kind or category"));
        }
        Ok(())
    }
}

fn check_shift_owner(db: &Connection, user_id: &str, shift_id: &Option<String>) -> ApiResult<()> {
    if let Some(id) = shift_id {
        let exists: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM shifts WHERE id = ?1 AND user_id = ?2)",
                params![id, user_id],
                |row| row.get(0),
            )
            .map_err(|_| ApiError::internal())?;
        if !exists {
            return Err(ApiError::bad("Shift does not belong to this user"));
        }
    }
    Ok(())
}

fn get_entry(db: &Connection, user_id: &str, id: &str) -> ApiResult<Entry> {
    db.query_row("SELECT id, shift_id, kind, delivery_count, expense_category, amount_vnd, tip_vnd, occurred_at, note, created_at, updated_at FROM entries WHERE id = ?1 AND user_id = ?2",
        params![id, user_id], entry_row).optional().map_err(|_| ApiError::internal())?.ok_or_else(ApiError::not_found)
}

#[utoipa::path(get, path = "/api/v1/entries", tag = "Giao dịch",
    responses((status = 200, body = [Entry]), (status = 401)))]
pub(crate) async fn list_entries(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Entry>>> {
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| entries_for(db, &user_id).map(Json)).await
}

#[utoipa::path(post, path = "/api/v1/entries", tag = "Giao dịch",
    request_body = EntryInput,
    responses((status = 201, body = Entry), (status = 400), (status = 401)))]
pub(crate) async fn create_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<EntryInput>,
) -> ApiResult<(StatusCode, Json<Entry>)> {
    input.validate()?;
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| {
        check_shift_owner(db, &user_id, &input.shift_id)?;
        let id = Uuid::new_v4().to_string();
        db.execute("INSERT INTO entries (id, user_id, shift_id, kind, delivery_count, expense_category, amount_vnd, tip_vnd, occurred_at, note, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
            params![id, user_id, input.shift_id, input.kind, input.delivery_count, input.expense_category,
                input.amount_vnd, input.tip_vnd, input.occurred_at, input.note, now_ms()])
            .map_err(|_| ApiError::internal())?;
        get_entry(db, &user_id, &id).map(|entry| (StatusCode::CREATED, Json(entry)))
    }).await
}

#[utoipa::path(put, path = "/api/v1/entries/{id}", tag = "Giao dịch",
    params(("id" = String, Path, description = "ID giao dịch")),
    request_body = EntryInput,
    responses((status = 200, body = Entry), (status = 400), (status = 401), (status = 404)))]
pub(crate) async fn replace_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<EntryInput>,
) -> ApiResult<Json<Entry>> {
    input.validate()?;
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| {
        check_shift_owner(db, &user_id, &input.shift_id)?;
        let count = db.execute("UPDATE entries SET shift_id = ?1, kind = ?2, delivery_count = ?3, expense_category = ?4, amount_vnd = ?5, tip_vnd = ?6, occurred_at = ?7, note = ?8, updated_at = ?9 WHERE id = ?10 AND user_id = ?11",
            params![input.shift_id, input.kind, input.delivery_count, input.expense_category,
                input.amount_vnd, input.tip_vnd, input.occurred_at, input.note, now_ms(), id, user_id])
            .map_err(|_| ApiError::internal())?;
        if count == 0 { return Err(ApiError::not_found()); }
        get_entry(db, &user_id, &id).map(Json)
    }).await
}

#[utoipa::path(delete, path = "/api/v1/entries/{id}", tag = "Giao dịch",
    params(("id" = String, Path, description = "ID giao dịch")),
    responses((status = 204), (status = 401), (status = 404)))]
pub(crate) async fn delete_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| {
        let count = db
            .execute(
                "DELETE FROM entries WHERE id = ?1 AND user_id = ?2",
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
