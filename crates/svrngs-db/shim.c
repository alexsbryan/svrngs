// SPDX-License-Identifier: AGPL-3.0-or-later
// cr-sqlite as one of SQLite's own compiled-in extensions, so no Rust in
// svrngs calls it and nothing is `unsafe` (svrn-docs rings/current/
// MAGNETS_BUILD.md §3, slice A). SQLite, as rusqlite bundles it, calls this
// as it opens each connection, with its built-in extensions, named by
// SQLITE_EXTRA_AUTOEXT (/.cargo/config.toml): a part of every open, with no
// step at SQLite's start for another thread's first open to come before
// (tests/first_open.rs). It is the one global symbol of the object build.rs
// links, `nm`-checked there. Upstream registers cr-sqlite at start instead,
// in core/src/core_init.c, which compiles only appended to the amalgamation.
#include "sqlite3.h"

int sqlite3_crsqlite_init(sqlite3 *db, char **err, const sqlite3_api_routines *api);

int svrngs_crsqlite(sqlite3 *db) {
  char *err = 0;
  int rc = sqlite3_crsqlite_init(db, &err, 0);
  sqlite3_free(err);
  return rc;
}
