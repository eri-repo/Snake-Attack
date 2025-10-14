-- Create sequence for position
CREATE SEQUENCE IF NOT EXISTS increment_position
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


CREATE TABLE users
(
    id            SERIAL PRIMARY KEY NOT NULL,
    address       VARCHAR            NOT NULL,
    username      VARCHAR            NOT NULL,
    position      BIGINT                      DEFAULT nextval('increment_position'),
    is_registered BOOLEAN            NOT NULL DEFAULT true,
    highest_score BIGINT                      DEFAULT 0,
    games_played  BIGINT                      DEFAULT 0,
    updated       BOOLEAN            NOT NULL DEFAULT false,
    registered_at TIMESTAMP          NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (address),
    UNIQUE (username)
);

-- Add index for leaderboard queries
CREATE INDEX IF NOT EXISTS idx_users_highest_score ON users (highest_score DESC);

-- Comments for documentation
COMMENT ON TABLE users IS 'Stores user information for the Starknake game';
COMMENT ON COLUMN users.id IS 'Unique identifier for the user';
COMMENT ON COLUMN users.address IS 'Starknet wallet address (0x + 64 hex chars)';
COMMENT ON COLUMN users.username IS 'User-selected username, max 50 chars';
COMMENT ON COLUMN users.position IS 'Sequential position assigned to the user';
COMMENT ON COLUMN users.is_registered IS 'Indicates if the user is registered';
COMMENT ON COLUMN users.highest_score IS 'User''s highest game score';
COMMENT ON COLUMN users.games_played IS 'Number of games played by the user';
COMMENT ON COLUMN users.updated IS 'Indicates if the username has been updated';
COMMENT ON COLUMN users.registered_at IS 'Timestamp of user registration (UTC)';