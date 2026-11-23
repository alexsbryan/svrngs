//! cr-sqlite on every connection, loaded as SQLite opens it by svrngs-db's
//! shim (build.rs, shim.c, /.cargo/config.toml), with no Rust calling it: a
//! connection opened plainly has its functions, and a table made a CRR has
//! its changes read.
//!
//! Plant: a build without the shim, `LIBSQLITE3_FLAGS= cargo test -p
//! svrngs-db --test crsqlite`, where SQLite calls no shim. This test then
//! fails: `no such function: crsql_site_id`.

// Linked for its object, which holds cr-sqlite and the shim SQLite calls.
extern crate svrngs_db;

use rusqlite::Connection;

#[test]
fn every_connection_opens_with_cr_sqlite() {
    let conn = Connection::open_in_memory().unwrap();
    let site = conn.query_row("SELECT crsql_site_id()", [], |r| r.get::<_, Vec<u8>>(0));
    assert_eq!(site.expect("cr-sqlite's functions, on a plain connection").len(), 16);
    conn.execute_batch("CREATE TABLE t (id TEXT NOT NULL PRIMARY KEY, v INTEGER)")
        .unwrap();
    conn.query_row("SELECT crsql_as_crr('t')", [], |_| Ok(())).unwrap();
    conn.execute("INSERT INTO t VALUES ('a', 1)", []).unwrap();
    let changes: i64 = conn
        .query_row("SELECT count(*) FROM crsql_changes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(changes, 1, "the insert's change, as cr-sqlite reads it");
}
