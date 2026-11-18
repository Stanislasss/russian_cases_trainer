-- A noun's first meaning per language as a noun there: its gender and plural,
-- so translated sentences can put it in with the right article ("au chien",
-- "den Hunden"). Unknown parts are NULL.
CREATE TABLE noun_forms (
    lexeme_id   bigint NOT NULL REFERENCES lexemes (id) ON DELETE CASCADE,
    locale      text NOT NULL CHECK (locale IN ('en', 'fr', 'de')),
    singular    text NOT NULL,
    plural      text,     -- the whole phrase in the plural
    gender      text CHECK (gender IN ('masculine', 'feminine', 'neuter')),
    plural_only boolean NOT NULL DEFAULT false,
    oblique     text,     -- German weak nouns: den/dem Jungen
    PRIMARY KEY (lexeme_id, locale)
);
