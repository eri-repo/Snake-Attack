CREATE SEQUENCE increment_position
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE TABLE users
(
    id            SERIAL PRIMARY KEY,
    address       VARCHAR   NOT NULL,
    username      VARCHAR   NOT NULL UNIQUE,
    position      BIGINT             DEFAULT nextval('increment_position'),
    is_registered BOOLEAN   NOT NULL DEFAULT true,
    highest_score BIGINT             DEFAULT 0,
    games_played  BIGINT             DEFAULT 0,
    updated       BOOLEAN   NOT NULL DEFAULT false,
    registered_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (address)
);


CREATE TABLE game_scores
(
    id             SERIAL PRIMARY KEY,
    user_id        INTEGER REFERENCES users (id),
    wallet_address VARCHAR   NOT NULL,
    score          INTEGER   NOT NULL,
    game_duration  INTEGER   NOT NULL,
    played_at      TIMESTAMP NOT NULL
);