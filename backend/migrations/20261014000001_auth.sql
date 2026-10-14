-- Accounts are separate from how users prove who they are, so OAuth providers
-- can be added later as new `user_identities.provider` values.

CREATE TABLE users (
    id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    email        text NOT NULL UNIQUE,  -- normalised to lowercase by the domain
    display_name text NOT NULL,
    ui_locale    text NOT NULL DEFAULT 'en' CHECK (ui_locale IN ('en', 'fr', 'ru')),
    created_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE user_identities (
    provider         text NOT NULL,  -- 'password', later 'google', 'github', ...
    provider_subject text NOT NULL,  -- user id for 'password', the provider's subject otherwise
    user_id          uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at       timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (provider, provider_subject)
);
CREATE INDEX user_identities_user_id_idx ON user_identities (user_id);

CREATE TABLE password_credentials (
    user_id       uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    password_hash text NOT NULL,  -- argon2id PHC string
    updated_at    timestamptz NOT NULL DEFAULT now()
);

-- Only the SHA-256 of the session token is stored; the token itself lives in the cookie.
CREATE TABLE sessions (
    token_hash bytea PRIMARY KEY,
    user_id    uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL
);
CREATE INDEX sessions_user_id_idx ON sessions (user_id);
