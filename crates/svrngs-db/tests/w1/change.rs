//! What copies exchange: a change row as cr-sqlite's `crsql_changes`
//! carries it, and a change set, one write transaction's rows. Fly's fork is
//! cited at the svrngs pin, github.com/alexsbryan/cr-sqlite branch `svrngs`
//! at `c3062b8d`, paths under `core/rs/core/src/`.

use std::cmp::Ordering;
use std::collections::BTreeMap;

/// A device's site id: cr-sqlite's `site_id`, 16 bytes (`consts.rs`,
/// `SITE_ID_LEN`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Site(pub [u8; 16]);

impl Site {
    /// A test device's site: its number in every byte, so sites order as
    /// their numbers do.
    pub fn of(n: u8) -> Site {
        Site([n; 16])
    }

    /// A phone's site at its home, in its epoch there: its person's number
    /// first, as a computer's site begins with its own, then its home's,
    /// then the epoch, the rest zero. svrngs-db derives a phone's nonce from
    /// its home and epoch (wire.rs `nonce`), so its site names its home
    /// there as it does here.
    pub fn phone(person: u8, home: Site, epoch: u8) -> Site {
        let mut bytes = [0; 16];
        bytes[..3].copy_from_slice(&[person, home.0[0], epoch]);
        Site(bytes)
    }

    /// The computer whose word vouches for this site's mark: a phone site's
    /// home, every other site's own computer (svrngs-db sync.rs `claimed`).
    pub fn home(self) -> Site {
        match self.0.iter().all(|b| *b == self.0[0]) {
            true => self,
            false => Site::of(self.0[1]),
        }
    }
}

/// A value as SQLite holds it. The test schemas declare each column's type
/// and the tests write that type, so no affinity converts one.
#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl Val {
    pub fn text(s: &str) -> Val {
        Val::Text(s.to_owned())
    }

    /// cr-sqlite's order of values, `crsql_compare_sqlite_values`
    /// (`compare_values.rs:9-45`). Across types it subtracts SQLite's type
    /// codes the wrong way round (`:13-17`; the codes, sqlite_nostd
    /// `nostd.rs:221-227`), so NULL < BLOB < TEXT < REAL < INTEGER, which
    /// is not SQLite's own ORDER BY. Within a type: bytes for TEXT and BLOB,
    /// numbers for INTEGER and REAL, and two REALs neither below the other
    /// are equal.
    pub fn cmp_crsql(&self, other: &Val) -> Ordering {
        let by_type = self.rank().cmp(&other.rank());
        if by_type != Ordering::Equal {
            return by_type;
        }
        match (self, other) {
            (Val::Integer(a), Val::Integer(b)) => a.cmp(b),
            (Val::Real(a), Val::Real(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
            (Val::Text(a), Val::Text(b)) => a.as_bytes().cmp(b.as_bytes()),
            (Val::Blob(a), Val::Blob(b)) => a.cmp(b),
            _ => Ordering::Equal,
        }
    }

    fn rank(&self) -> u8 {
        match self {
            Val::Null => 0,
            Val::Blob(_) => 1,
            Val::Text(_) => 2,
            Val::Real(_) => 3,
            Val::Integer(_) => 4,
        }
    }
}

/// The column id of a row's causal-length sentinel, the row's existence
/// (`c.rs:10-11`: INSERT_SENTINEL and DELETE_SENTINEL are both "-1").
pub const SENTINEL: &str = "-1";

/// One row of `crsql_changes`, less `ts`: svrngs merges by no clock (the
/// operator, 2026-11-17, APP_STATE §4 "How it merges"), so nothing reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub table: String,
    /// The row's key values, packed as `crsql_changes.pk` carries them
    /// (`pack`).
    pub pk: Vec<u8>,
    /// A non-key column's name, or `SENTINEL`.
    pub cid: String,
    /// The column's value; NULL for the sentinel.
    pub val: Val,
    pub col_version: i64,
    pub db_version: i64,
    pub site: Site,
    /// The row's causal length when the set was sealed: odd while it
    /// exists, even once deleted.
    pub cl: i64,
    pub seq: i64,
}

/// A set's name among a copy's kept sets: its site and `db_version`
/// (APP_STATE §4, "Sealed and signed"). A computer's writes never share one.
pub type SetId = (Site, i64);

/// One write transaction's change rows, at one site and one `db_version`:
/// what a device seals and copies exchange.
#[derive(Clone, Debug, PartialEq)]
pub struct ChangeSet {
    changes: Vec<Change>,
    /// Whether it is signed as its device's, as every copy checks before
    /// it reads a set (svrngs-db sync.rs `take`): not only for a phone's set
    /// carried under a pass that doesn't hold (`Replica::prepare`).
    signed: bool,
}

impl ChangeSet {
    /// A set of these rows, or none if there are none.
    pub fn new(changes: Vec<Change>) -> Option<ChangeSet> {
        let first = changes.first()?;
        let id = (first.site, first.db_version);
        assert!(
            changes.iter().all(|c| (c.site, c.db_version) == id),
            "one set is one transaction, at one site and one db_version"
        );
        Some(ChangeSet { changes, signed: true })
    }

    /// The same set, carried under a pass that doesn't hold, so signed as no
    /// device's: every copy drops it (svrngs-db `Took::NotItsDevice`).
    pub fn lapsed(self) -> ChangeSet {
        ChangeSet { signed: false, ..self }
    }

    pub fn signed(&self) -> bool {
        self.signed
    }

    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    pub fn id(&self) -> SetId {
        (self.changes[0].site, self.changes[0].db_version)
    }
}

/// A row's place: its table and packed key.
pub type RowKey = (String, Vec<u8>);

/// A table state: every row that exists, each with its non-key columns that
/// are not NULL. A column set to NULL and one no change ever reached read
/// alike, as they do to a page.
pub type State = BTreeMap<RowKey, BTreeMap<String, Val>>;

/// A key's values packed as cr-sqlite packs them for `crsql_changes.pk`
/// (`pack_columns.rs:31-86`): their count, then for each a type byte, which
/// holds in its upper five bits how many bytes the integer or the length
/// after it takes, then the bytes.
pub fn pack(key: &[Val]) -> Vec<u8> {
    let mut out = vec![u8::try_from(key.len()).expect("at most 255 key columns")];
    for v in key {
        match v {
            Val::Null => out.push(5),
            Val::Real(r) => {
                out.push(2);
                out.extend_from_slice(&r.to_be_bytes());
            }
            Val::Integer(i) => {
                let n = int_bytes(*i);
                out.push(n << 3 | 1);
                out.extend_from_slice(&i.to_be_bytes()[8 - n as usize..]);
            }
            Val::Text(s) => sized(&mut out, 3, s.as_bytes()),
            Val::Blob(b) => sized(&mut out, 4, b),
        }
    }
    out
}

fn sized(out: &mut Vec<u8>, code: u8, bytes: &[u8]) {
    let len = i64::try_from(bytes.len()).expect("a key value fits in memory");
    let n = int_bytes(len);
    out.push(n << 3 | code);
    out.extend_from_slice(&len.to_be_bytes()[8 - n as usize..]);
    out.extend_from_slice(bytes);
}

/// The fewest bytes that hold `i`, as `num_bytes_needed_i64` counts them
/// (`pack_columns.rs:88-114`): 0 for 0, and 8 for any negative number.
fn int_bytes(i: i64) -> u8 {
    let lead = i.to_be_bytes().iter().take_while(|b| **b == 0).count();
    8 - lead as u8
}

/// SplitMix64, S1's generator byte for byte (svrn-docs
/// `rings/current/magnets-spikes/code/src/lib.rs:208-231`), so one seed
/// names one run here as there.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    pub fn chance(&mut self, pct: u64) -> bool {
        self.below(100) < pct
    }

    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            v.swap(i, j);
        }
    }

    /// An index below `n`.
    pub fn pick(&mut self, n: usize) -> usize {
        self.below(n as u64) as usize
    }
}
