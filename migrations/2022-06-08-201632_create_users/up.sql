CREATE TABLE users (
    id              UUID PRIMARY KEY,
    name            TEXT NOT NULL,
    email           TEXT NOT NULL,
    password        VARCHAR(100) NOT NULL,
    role            INT NOT NULL
)
