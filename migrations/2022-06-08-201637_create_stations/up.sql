CREATE TABLE stations (
    id              SERIAL PRIMARY KEY,
    token           VARCHAR(32),
    name            TEXT NOT NULL,
    lat             DOUBLE PRECISION NOT NULL,
    lon             DOUBLE PRECISION NOT NULL,
    region          INT REFERENCES regions(id) NOT NULL,
    owner           UUID REFERENCES users(id) NOT NULL,
    approved        BOOLEAN NOT NULL
)
