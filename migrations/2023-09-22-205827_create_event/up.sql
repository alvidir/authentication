CREATE TABLE IF NOT EXISTS Events (
    checksum INTEGER PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    payload TEXT NOT NULL,
);