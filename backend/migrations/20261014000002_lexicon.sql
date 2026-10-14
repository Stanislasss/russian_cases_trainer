-- Words, their inflected forms and translations. Enum-like columns are text with
-- CHECK constraints; the values match the snake_case serde names in `domain`.

CREATE TABLE lexemes (
    id             bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    lemma          text NOT NULL,
    stressed_lemma text,
    pos            text NOT NULL CHECK (pos IN ('noun', 'adjective', 'pronoun')),
    gender         text CHECK (gender IN ('masculine', 'feminine', 'neuter')),  -- NULL for pluralia tantum
    animacy        text CHECK (animacy IN ('animate', 'inanimate')),
    indeclinable   boolean NOT NULL DEFAULT false,
    frequency_rank integer,  -- lower = more common
    source         text NOT NULL,
    source_id      text NOT NULL,
    UNIQUE (source, source_id)
);
CREATE INDEX lexemes_pos_rank_idx ON lexemes (pos, frequency_rank);

CREATE TABLE forms (
    lexeme_id        bigint NOT NULL REFERENCES lexemes (id) ON DELETE CASCADE,
    grammatical_case text NOT NULL CHECK (grammatical_case IN
                         ('nominative', 'genitive', 'dative', 'accusative', 'instrumental', 'prepositional')),
    number           text NOT NULL CHECK (number IN ('singular', 'plural')),
    variant          smallint NOT NULL DEFAULT 0,  -- 0 = main form, 1.. = alternatives (e.g. водой / водою)
    form             text NOT NULL,
    stressed_form    text,
    stem             text,  -- NULL when the form can't be split cleanly
    ending           text,
    irregular        boolean NOT NULL DEFAULT false,
    PRIMARY KEY (lexeme_id, grammatical_case, number, variant)
);

CREATE TABLE translations (
    lexeme_id bigint NOT NULL REFERENCES lexemes (id) ON DELETE CASCADE,
    locale    text NOT NULL CHECK (locale IN ('en', 'fr', 'ru')),
    position  smallint NOT NULL DEFAULT 0,
    text      text NOT NULL,
    PRIMARY KEY (lexeme_id, locale, position)
);

CREATE TABLE rules (
    id               bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    grammatical_case text NOT NULL CHECK (grammatical_case IN
                         ('nominative', 'genitive', 'dative', 'accusative', 'instrumental', 'prepositional')),
    slug             text NOT NULL UNIQUE,
    sort_order       integer NOT NULL DEFAULT 0
);

CREATE TABLE rule_translations (
    rule_id bigint NOT NULL REFERENCES rules (id) ON DELETE CASCADE,
    locale  text NOT NULL CHECK (locale IN ('en', 'fr', 'ru')),
    title   text NOT NULL,
    body_md text NOT NULL,
    PRIMARY KEY (rule_id, locale)
);

CREATE TABLE rule_examples (
    rule_id          bigint NOT NULL REFERENCES rules (id) ON DELETE CASCADE,
    lexeme_id        bigint NOT NULL REFERENCES lexemes (id) ON DELETE CASCADE,
    grammatical_case text NOT NULL,
    number           text NOT NULL,
    sort_order       integer NOT NULL DEFAULT 0,
    PRIMARY KEY (rule_id, lexeme_id, grammatical_case, number)
);

CREATE TABLE exercise_attempts (
    id               bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    user_id          uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    exercise_type    text NOT NULL,  -- e.g. 'suffix'
    lexeme_id        bigint NOT NULL REFERENCES lexemes (id) ON DELETE CASCADE,
    grammatical_case text NOT NULL,
    number           text NOT NULL,
    answer           text NOT NULL,
    correct          boolean NOT NULL,
    answered_at      timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX exercise_attempts_user_idx ON exercise_attempts (user_id, answered_at DESC);
