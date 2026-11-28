-- Magnets' board: each magnet the group has moved, and where it sits in the
-- sentence. A magnet with no row, or a NULL position, is in the tray, so a
-- fresh board is an empty table and needs no first write.
-- Within what cr-sqlite takes as a CRDT table (superfly/cr-sqlite,
-- core/rs/core/src/tableinfo.rs): a NOT NULL primary key, no other UNIQUE,
-- no foreign key.
CREATE TABLE magnets (
  id TEXT NOT NULL PRIMARY KEY,
  position INTEGER
);
-- Each sentence someone kept as they started a new one: its prompt, its
-- magnets' IDs in order apart by spaces, and when, by that page's clock.
-- Put back on the fridge, its row goes, as the fridge shows it again.
CREATE TABLE kept (
  id TEXT NOT NULL PRIMARY KEY,
  prompt TEXT,
  words TEXT,
  at INTEGER
);
-- The prompt the sentence is under: one row, 'now', and none until a
-- sentence is first kept, when the pack's first prompt is the one.
CREATE TABLE showing (
  id TEXT NOT NULL PRIMARY KEY,
  prompt TEXT
);
