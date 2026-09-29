CREATE TABLE frames (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id TEXT NOT NULL DEFAULT '',
    game_time INTEGER NOT NULL DEFAULT 0,
    clock_time INTEGER NOT NULL DEFAULT 0,
    received_at INTEGER NOT NULL,
    payload TEXT NOT NULL
);
CREATE INDEX idx_frames_match ON frames (match_id, game_time);

CREATE TABLE happenings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id TEXT NOT NULL DEFAULT '',
    tick INTEGER NOT NULL DEFAULT 0,
    kind TEXT NOT NULL,
    actor TEXT,
    detail TEXT NOT NULL DEFAULT '{}',
    recorded_at INTEGER NOT NULL
);
CREATE INDEX idx_happenings_match ON happenings (match_id, tick);
