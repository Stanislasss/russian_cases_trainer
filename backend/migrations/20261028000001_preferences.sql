-- What each player chose in the settings; no row: the defaults.
CREATE TABLE user_preferences (
    user_id           uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    -- words the player keeps getting right come up less often
    fewer_known_words boolean NOT NULL DEFAULT true
);
