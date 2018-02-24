CREATE TABLE users (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    confirmed BOOLEAN NOT NULL DEFAULT 'f',
    CONSTRAINT name_unique UNIQUE(name)
)
