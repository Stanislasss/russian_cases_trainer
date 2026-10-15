-- The read-only word list shipped with the desktop app, written in one go by
-- `importer --sqlite`. A trimmed copy of the Postgres lexicon: only the words some
-- level can ask, and only the columns the game reads.
--
-- Lexeme ids are derived from the word's source key, so they stay the same
-- across app versions and the attempts in progress.sqlite keep pointing at them.
-- SQLite's lower() only knows ASCII, so the Cyrillic case checks the Postgres
-- queries do with lower() are precomputed (`proper`, `is_lemma`).

CREATE TABLE lexemes (
    id             INTEGER PRIMARY KEY,
    lemma          TEXT NOT NULL,
    stressed_lemma TEXT,
    pos            TEXT NOT NULL,
    gender         TEXT,             -- NULL for pluralia tantum
    indeclinable   INTEGER NOT NULL,
    frequency_rank INTEGER NOT NULL, -- lower = more common
    proper         INTEGER NOT NULL  -- capitalised lemma: place names, surnames
);
CREATE INDEX lexemes_pos_rank_idx ON lexemes (pos, frequency_rank);

CREATE TABLE forms (
    lexeme_id        INTEGER NOT NULL REFERENCES lexemes (id),
    grammatical_case TEXT NOT NULL,
    number           TEXT NOT NULL,
    gender           TEXT NOT NULL,    -- 'any' when the form doesn't vary by it
    animacy          TEXT NOT NULL,
    variant          INTEGER NOT NULL, -- 0 = main form
    form             TEXT NOT NULL,
    stressed_form    TEXT,
    ending           TEXT,
    irregular        INTEGER NOT NULL,
    is_lemma         INTEGER NOT NULL, -- spelled like the dictionary form
    PRIMARY KEY (lexeme_id, grammatical_case, number, gender, animacy, variant)
) WITHOUT ROWID;
CREATE INDEX forms_case_idx ON forms (grammatical_case, lexeme_id);

-- Only the first three meanings per language: all the game shows.
CREATE TABLE translations (
    lexeme_id INTEGER NOT NULL REFERENCES lexemes (id),
    locale    TEXT NOT NULL,
    position  INTEGER NOT NULL,
    text      TEXT NOT NULL,
    PRIMARY KEY (lexeme_id, locale, position)
) WITHOUT ROWID;
