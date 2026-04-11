-- Add migration script here
-- Add migration script here

DROP TABLE IF EXISTS users;
CREATE TABLE users (
                       id UUID PRIMARY KEY,
                       email TEXT NOT NULL UNIQUE,
                       username TEXT NOT NULL,
                       password TEXT NOT NULL,
                       verified BOOLEAN NOT NULL DEFAULT FALSE,
                       created_at TIMESTAMP NOT NULL DEFAULT NOW(),
                       updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

