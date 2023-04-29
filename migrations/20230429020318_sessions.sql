-- Add migration script here
CREATE TABLE IF NOT EXISTS sessions (
  id INTEGER PRIMARY KEY NOT NULL,
  cookie_id varchar(8) NOT NULL,
  user_id INTEGER NOT NULL,
  last_active DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
)
