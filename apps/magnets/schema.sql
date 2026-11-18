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
