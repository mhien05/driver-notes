PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    token_hash TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS sessions_by_user ON sessions(user_id);

CREATE TABLE IF NOT EXISTS shifts (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK (ended_at IS NULL OR ended_at > started_at)
);
CREATE UNIQUE INDEX IF NOT EXISTS one_active_shift_per_user
    ON shifts(user_id) WHERE ended_at IS NULL;
CREATE INDEX IF NOT EXISTS shifts_by_user_start ON shifts(user_id, started_at);

CREATE TABLE IF NOT EXISTS entries (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    shift_id TEXT REFERENCES shifts(id) ON DELETE SET NULL,
    kind TEXT NOT NULL CHECK (kind IN ('delivery', 'bonus', 'expense')),
    delivery_count INTEGER,
    expense_category TEXT,
    amount_vnd INTEGER NOT NULL CHECK (amount_vnd > 0),
    tip_vnd INTEGER NOT NULL DEFAULT 0 CHECK (tip_vnd >= 0),
    occurred_at INTEGER NOT NULL,
    note TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    CHECK (
        (kind = 'delivery' AND delivery_count IN (1, 2, 3) AND expense_category IS NULL)
        OR (kind = 'bonus' AND delivery_count IS NULL AND expense_category IS NULL AND tip_vnd = 0)
        OR (kind = 'expense' AND delivery_count IS NULL AND expense_category IN ('fuel', 'food', 'other') AND tip_vnd = 0)
    )
);
CREATE INDEX IF NOT EXISTS entries_by_user_time ON entries(user_id, occurred_at);
