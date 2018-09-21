CREATE TABLE IF NOT EXISTS storages (
  id            INTEGER PRIMARY KEY NOT NULL,
  hard_state_id INTEGER             NOT NULL,
  FOREIGN KEY (hard_state_id) REFERENCES hard_state (id)
);

CREATE TABLE IF NOT EXISTS entries (
  storage_id INTEGER NOT NULL,
  entry_type INTEGER NOT NULL,
  term       INTEGER NOT NULL,
  "index"    INTEGER NOT NULL,
  data       BLOB    NOT NULL,
  context    BLOB    NOT NULL,
  sync_log   INTEGER NOT NULL,
  FOREIGN KEY (storage_id) REFERENCES storages (id)

);

CREATE TABLE IF NOT EXISTS hard_state (
  id       INTEGER PRIMARY KEY NOT NULL,
  term     INTEGER             NOT NULL,
  vote     INTEGER             NOT NULL,
  "commit" INTEGER             NOT NULL
);

-- TODO:
-- CREATE TABLE IF NOT EXISTS snapshots (
--
-- );