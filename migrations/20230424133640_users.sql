-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  name varchar(30) NOT NULL,
  active BOOLEAN NOT NULL DEFAULT 0
)