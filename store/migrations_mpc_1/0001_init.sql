CREATE TABLE keyshares (
    id SERIAL PRIMARY KEY,
    user_id TEXT NOT NULL,
    public_key TEXT NOT NULL,
    secret_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
