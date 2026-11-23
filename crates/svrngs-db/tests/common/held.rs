//! What each device holds at a copy, counted from the file as `Db::held`
//! counted it before SQLite kept its totals (svrngs-db `sync.rs` at
//! `445eec74`): the reference the totals are held to, read on a second
//! connection to the copy's file.

use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::Connection;

/// Each device's rows of the ask and bytes of the database, by its key.
pub fn counted(side: &Connection) -> BTreeMap<String, (u64, u64)> {
    let asks = "SELECT h.device, count(*) FROM (SELECT site FROM svrngs_sites UNION ALL SELECT site FROM svrngs_above \
        UNION ALL SELECT site FROM svrngs_sets s WHERE EXISTS (SELECT 1 FROM svrngs_sets t WHERE t.site = s.site \
        AND t.seq = s.seq AND t.sig <> s.sig) OR site = (SELECT retired FROM svrngs_db)) AS r \
        JOIN svrngs_sites h USING (site) GROUP BY h.device";
    let bytes = "SELECT h.device, coalesce(sum(length(s.bytes)), 0) + 100 * count(DISTINCT h.site) \
        FROM svrngs_sites h LEFT JOIN svrngs_sets s USING (site) GROUP BY h.device";
    let read = |sql: &str| {
        let mut stmt = side.prepare(sql).unwrap();
        let each = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u64>(1)?)));
        each.unwrap().map(Result::unwrap).collect::<BTreeMap<_, _>>()
    };
    let bytes = read(bytes);
    read(asks)
        .into_iter()
        .map(|(d, n)| (d.clone(), (n, bytes[&d])))
        .collect()
}

/// `db`'s totals, by each device's key, to be held to [`counted`].
pub fn totals(db: &svrngs_db::Db) -> BTreeMap<String, (u64, u64)> {
    let held = db.holds().expect("what each device holds");
    held.into_iter().map(|(d, n)| (d.as_str().to_owned(), n)).collect()
}

/// `db`'s totals, its file at `file`, equal [`counted`] over its file.
#[allow(
    dead_code,
    reason = "each test binary compiles this file, and only some hold a copy to it whole"
)]
pub fn as_counted(db: &svrngs_db::Db, file: &Path) {
    let side = Connection::open(file).unwrap();
    assert_eq!(totals(db), counted(&side), "its totals are the count over its file");
    svrngs_db::close(side).unwrap();
}
