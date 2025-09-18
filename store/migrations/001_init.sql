-- users table
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL,
    public_key TEXT NOT NULL
);

-- assets table
CREATE TABLE assets (
    id TEXT PRIMARY KEY,
    mint_address TEXT NOT NULL UNIQUE,
    decimals INT NOT NULL,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    logo_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL
);

-- balances table
CREATE TABLE balances (
    id TEXT PRIMARY KEY,
    amount NUMERIC NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    asset_id TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE
);

-- indexes for faster lookups
CREATE INDEX idx_balances_user_id ON balances(user_id);
CREATE INDEX idx_balances_asset_id ON balances(asset_id);
