CREATE TABLE regions (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    transport_company TEXT NOT NULL,
    frequency       BIGINT NOT NULL,
    protocol        TEXT NOT NULL
)
