-- The desktop app's own data, in the user's app-data folder. There is one local
-- user, so nothing here refers to users. lexeme_id points into lexicon.sqlite,
-- which is attached read-only as `lex` (no foreign key across databases).

CREATE TABLE exercise_attempts (
    id               INTEGER PRIMARY KEY,
    exercise_type    TEXT NOT NULL,
    lexeme_id        INTEGER NOT NULL,
    grammatical_case TEXT NOT NULL,
    number           TEXT NOT NULL,
    gender           TEXT,
    animacy          TEXT,
    answer           TEXT NOT NULL,
    correct          INTEGER NOT NULL,
    answered_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);
CREATE INDEX exercise_attempts_answered_idx ON exercise_attempts (answered_at DESC);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
