-- Add migration script here
DROP TABLE IF EXISTS users;
CREATE OR REPLACE TABLE users (
        id UUID PRIMARY KEY,
        email TEXT NOT NULL UNIQUE,
        username TEXT NOT NULL UNIQUE,
        password TEXT NOT NULL
        verified BOOLEAN NOT NULL DEFAULT FALSE
);
