use super::{
    auth::authenticated_user,
    entries::{Entry, EntryInput, entries_for},
    shifts::{Shift, shifts_for},
};
use crate::{
    db::{AppState, with_db},
    error::{ApiError, ApiResult},
    time::{now_ms, valid_time},
};
use axum::{Json, extract::State, http::HeaderMap};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    schema_version: u32,
    fetched_at: i64,
    shifts: Vec<Shift>,
    entries: Vec<Entry>,
}

#[utoipa::path(get, path = "/api/v1/snapshot", tag = "Sao lưu",
    responses((status = 200, body = Snapshot), (status = 401)))]
pub(crate) async fn snapshot(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Snapshot>> {
    let user_id = authenticated_user(&state, &headers).await?;
    with_db(&state, move |db| {
        Ok(Json(Snapshot {
            schema_version: 1,
            fetched_at: now_ms(),
            shifts: shifts_for(db, &user_id)?,
            entries: entries_for(db, &user_id)?,
        }))
    })
    .await
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Backup {
    schema_version: u32,
    /// Toàn bộ ca từ snapshot của tài khoản.
    shifts: Vec<Shift>,
    /// Toàn bộ giao dịch từ snapshot của tài khoản.
    entries: Vec<Entry>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RestoreResult {
    shifts_restored: usize,
    entries_restored: usize,
}

#[utoipa::path(post, path = "/api/v1/restore", tag = "Sao lưu",
    request_body = Backup,
    responses((status = 200, body = RestoreResult), (status = 400), (status = 401)))]
pub(crate) async fn restore(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(backup): Json<Backup>,
) -> ApiResult<Json<RestoreResult>> {
    let user_id = authenticated_user(&state, &headers).await?;
    if backup.schema_version != 1 || backup.shifts.len() > 50_000 || backup.entries.len() > 50_000 {
        return Err(ApiError::bad("Unsupported or oversized backup"));
    }
    let mut seen = HashSet::new();
    let mut shift_ids = HashMap::new();
    let mut active_count = 0;
    for shift in &backup.shifts {
        if !valid_time(shift.started_at)
            || shift
                .ended_at
                .is_some_and(|end| !valid_time(end) || end <= shift.started_at)
            || !seen.insert(&shift.id)
            || Uuid::parse_str(&shift.id).is_err()
        {
            return Err(ApiError::bad("Invalid shift in backup"));
        }
        if shift.ended_at.is_none() {
            active_count += 1;
        }
        shift_ids.insert(shift.id.clone(), Uuid::new_v4().to_string());
    }
    if active_count > 1 {
        return Err(ApiError::bad("Backup has multiple active shifts"));
    }
    seen.clear();
    for entry in &backup.entries {
        let input = EntryInput {
            shift_id: entry.shift_id.clone(),
            kind: entry.kind.clone(),
            delivery_count: entry.delivery_count,
            expense_category: entry.expense_category.clone(),
            amount_vnd: entry.amount_vnd,
            tip_vnd: entry.tip_vnd,
            occurred_at: entry.occurred_at,
            note: entry.note.clone(),
        };
        input.validate()?;
        if !seen.insert(&entry.id)
            || Uuid::parse_str(&entry.id).is_err()
            || entry
                .shift_id
                .as_ref()
                .is_some_and(|id| !shift_ids.contains_key(id))
        {
            return Err(ApiError::bad("Invalid entry in backup"));
        }
    }
    let result = RestoreResult {
        shifts_restored: backup.shifts.len(),
        entries_restored: backup.entries.len(),
    };
    with_db(&state, move |db| {
        let tx = db.transaction().map_err(|_| ApiError::internal())?;
        tx.execute("DELETE FROM entries WHERE user_id = ?1", [&user_id]).map_err(|_| ApiError::internal())?;
        tx.execute("DELETE FROM shifts WHERE user_id = ?1", [&user_id]).map_err(|_| ApiError::internal())?;
        let time = now_ms();
        for shift in backup.shifts {
            tx.execute("INSERT INTO shifts (id, user_id, started_at, ended_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                params![shift_ids[&shift.id], user_id, shift.started_at, shift.ended_at, time])
                .map_err(|_| ApiError::internal())?;
        }
        for entry in backup.entries {
            let shift_id = entry.shift_id.as_ref().map(|id| &shift_ids[id]);
            tx.execute("INSERT INTO entries (id, user_id, shift_id, kind, delivery_count, expense_category, amount_vnd, tip_vnd, occurred_at, note, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
                params![Uuid::new_v4().to_string(), user_id, shift_id, entry.kind,
                    entry.delivery_count, entry.expense_category, entry.amount_vnd, entry.tip_vnd,
                    entry.occurred_at, entry.note, time]).map_err(|_| ApiError::internal())?;
        }
        tx.commit().map_err(|_| ApiError::internal())?;
        Ok(Json(result))
    }).await
}
