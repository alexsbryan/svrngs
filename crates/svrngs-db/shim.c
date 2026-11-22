// SPDX-License-Identifier: AGPL-3.0-or-later
// cr-sqlite registered at SQLite's own start, so no Rust in svrngs calls it
// and nothing is `unsafe` (svrn-docs rings/current/MAGNETS_BUILD.md §3,
// slice A). SQLite, as rusqlite bundles it, calls this once from
// sqlite3_initialize, named by SQLITE_EXTRA_INIT (/.cargo/config.toml); it
// has SQLite run cr-sqlite's entry point on each connection opened after.
// It is the one global symbol of the object build.rs links, `nm`-checked
// there. Upstream does the same in core/src/core_init.c, which compiles only
// appended to the amalgamation.
#include "sqlite3.h"

int sqlite3_crsqlite_init(sqlite3 *db, char **err, const sqlite3_api_routines *api);

int svrngs_crsqlite(const char *unused) {
  (void)unused;
  return sqlite3_auto_extension((void (*)(void))sqlite3_crsqlite_init);
}
