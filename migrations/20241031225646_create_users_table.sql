CREATE TABLE users (
	public_key TEXT PRIMARY KEY,
	token_amount INTEGER NOT NULL DEFAULT 0 check (token_amount >= 0),
	subscribed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
)
