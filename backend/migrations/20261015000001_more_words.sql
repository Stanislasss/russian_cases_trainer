-- Adjectives, pronouns and surnames, and German as a language.

-- Adjective and surname forms vary by gender, and in the accusative also by the
-- animacy of the noun they go with (но́вый стол / но́вого студе́нта). 'any' means
-- the form doesn't depend on it (all noun forms, plural adjective forms, ...).
ALTER TABLE forms
    ADD COLUMN gender  text NOT NULL DEFAULT 'any'
        CHECK (gender IN ('any', 'masculine', 'feminine', 'neuter')),
    ADD COLUMN animacy text NOT NULL DEFAULT 'any'
        CHECK (animacy IN ('any', 'animate', 'inanimate'));
ALTER TABLE forms DROP CONSTRAINT forms_pkey;
ALTER TABLE forms ADD PRIMARY KEY (lexeme_id, grammatical_case, number, gender, animacy, variant);

ALTER TABLE lexemes DROP CONSTRAINT lexemes_pos_check;
ALTER TABLE lexemes ADD CONSTRAINT lexemes_pos_check
    CHECK (pos IN ('noun', 'adjective', 'pronoun', 'personal_pronoun', 'surname'));

-- Attempts on adjective-like words record which form was asked.
ALTER TABLE exercise_attempts
    ADD COLUMN gender  text,
    ADD COLUMN animacy text;

ALTER TABLE users DROP CONSTRAINT users_ui_locale_check;
ALTER TABLE users ADD CONSTRAINT users_ui_locale_check
    CHECK (ui_locale IN ('en', 'fr', 'ru', 'de'));
ALTER TABLE translations DROP CONSTRAINT translations_locale_check;
ALTER TABLE translations ADD CONSTRAINT translations_locale_check
    CHECK (locale IN ('en', 'fr', 'ru', 'de'));
ALTER TABLE rule_translations DROP CONSTRAINT rule_translations_locale_check;
ALTER TABLE rule_translations ADD CONSTRAINT rule_translations_locale_check
    CHECK (locale IN ('en', 'fr', 'ru', 'de'));
