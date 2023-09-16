CREATE TYPE IF NOT EXISTS MFA_METHOD AS ENUM ('tp_app', 'email');

CREATE TABLE IF NOT EXISTS Users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(64) NOT NULL UNIQUE,
    actual_email VARCHAR(64) NOT NULL UNIQUE,
    password VARCHAR(128) NOT NULL,
    mfa_method MFA_METHOD,
);