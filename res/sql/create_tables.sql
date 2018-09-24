CREATE TABLE IF NOT EXISTS Cores (
  core_id INTEGER NOT NULL PRIMARY KEY
);

CREATE TABLE IF NOT EXISTS Entries (
  "index"    INTEGER NOT NULL PRIMARY KEY,
  term       INTEGER NOT NULL,
  entry_type INTEGER NOT NULL,
  data       BLOB    NOT NULL,
  context    BLOB    NOT NULL,
  sync_log   INTEGER NOT NULL,
  core_id    INTEGER NOT NULL,
  FOREIGN KEY (core_id) REFERENCES Cores (core_id)
);

CREATE TABLE IF NOT EXISTS HardStates (
  term     INTEGER NOT NULL,
  vote     INTEGER NOT NULL,
  "commit" INTEGER NOT NULL,
  core_id  INTEGER NOT NULL UNIQUE,
  FOREIGN KEY (core_id) REFERENCES Cores (core_id)
);

CREATE TABLE IF NOT EXISTS Snapshots (
  data    BLOB    NOT NULL,
  "index" INTEGER NOT NULL,
  term    INTEGER NOT NULL,
  core_id INTEGER NOT NULL UNIQUE,
  FOREIGN KEY (core_id) REFERENCES Cores (core_id)
);

CREATE TABLE IF NOT EXISTS Nodes (
  node_id   INTEGER NOT NULL,
  node_type INTEGER NOT NULL,
  core_id   INTEGER NOT NULL,
  FOREIGN KEY (core_id) REFERENCES Snapshots (core_id)
);
