
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
    user_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    username     TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_admin     BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS projects (
    project_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT NOT NULL,
    description  TEXT
);

CREATE TABLE IF NOT EXISTS bugs (
    bug_id       INTEGER PRIMARY KEY AUTOINCREMENT,
    title        TEXT NOT NULL,
    description  TEXT NOT NULL,
    reported_by  INTEGER NOT NULL REFERENCES users(user_id),
    severity     TEXT NOT NULL,
    developer_id INTEGER REFERENCES users(user_id),
    status       TEXT NOT NULL DEFAULT 'open',
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
