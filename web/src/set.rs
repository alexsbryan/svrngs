// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a phone signs for a write to a carried app's database (svrn-docs
//! `rings/current/APP_STATE.md` §4, "Phones"; MAGNETS_BUILD S2), run by the
//! group's signer alone (web/sign.js), never by the app's page or its
//! worker: a computer's prepare of the page's write list, taken only as a
//! computer of the open signed it (svrngs-db `Db::prepare`), checked against
//! that list, and signed as the phone's own set at the place the signer
//! picks, the seq after the last it signed at that computer, its home
//! there, for its store (svrn-docs `app-state-diligence/invariants-2026-11-22/`,
//! change 5). Pure: the phone's key in, the set's bytes or the words for
//! why not out. The set's form and its changes' are
//! svrngs-db's own files, compiled here (`wire.rs`, `change.rs`).
//!
//! Every change row it signs is one the ops make (svrn-docs
//! `app-state-diligence/security-review-2026-11-23/`, H1), each one op's,
//! its table and its key packed as cr-sqlite packs one: the row's own
//! change (`-1`) at an odd causal length only where an insert makes the
//! row, at an even one only where a delete removes it last; any other, a
//! column of a row that stands, at an odd causal length, that the ops name
//! at the value they leave it, or, on a row an insert makes, one they don't
//! name at NULL; and no version past half the bound every copy holds
//! ([`MOST`]). An op with no row is a no-op only where its row stood, before
//! the write, as the op would leave it. What the phone can't check is the
//! residual (§4, "The residual"): the causal length and column version the
//! computer set, the row a no-op stands on, and the order of a composite
//! key's columns, each as that computer reads its copy, as it serves the
//! app's page too.

use std::collections::BTreeMap;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Deserialize;
use serde_json::Value;
use svrngs_core::pass::Label;

#[path = "../../crates/svrngs-db/src/change.rs"]
#[rustfmt::skip]
#[allow(dead_code)]
pub(crate) mod change;
#[path = "../../crates/svrngs-db/src/wire.rs"]
#[rustfmt::skip]
#[allow(dead_code)]
mod wire;
type Sql = change::Value;
use change::{cell, pack};

/// What the signer is asked to sign: the page's write list, in the group and
/// app the open names, and the computer's prepare of it, its text as `by`
/// signed it with `sig`; and the place the signer picked, its store's id
/// and the seq it signs at `by`, its home.
#[derive(Deserialize)]
pub struct Asked {
    pub group: String,
    pub app: String,
    pub ops: Value,
    pub prepared: String,
    pub by: String,
    pub sig: String,
    pub pass: String,
    pub ts: i64,
    pub store: [u8; 16],
    pub seq: u64,
}

/// A prepare's text (svrngs-db `Db::prepare`), as the phone reads it.
#[derive(Deserialize)]
struct Prepared {
    ops: Value,
    changes: String,
    schema: String,
    keys: BTreeMap<String, Vec<String>>,
    stands: Vec<Value>,
}

/// One op of the write list, read for its rows: insert, update or delete
/// (0, 1, 2), its table, its key packed, and the columns but its key's it
/// names.
struct Op<'v> {
    kind: usize,
    table: &'v str,
    pk: Vec<u8>,
    named: Vec<(&'v String, Sql)>,
}

/// For `asked`, under the label `label` of the app whose page asked: what
/// its open's approver signs, the set that signature goes in ([`sealed`]),
/// postcard's bytes in base64, and its place, `home/seq`; none where
/// the computer's prepare holds no change row and the ops, so checked,
/// change nothing; or the words for why not.
pub fn signed(label: &str, asked: &Asked) -> Result<Option<(Vec<u8>, String, String)>, String> {
    let why = |what| {
        format!(
            "This phone didn’t sign {}’s write, since {what}. Try again, or open it through another computer.",
            asked.app
        )
    };
    let ours = Label::carried(&asked.app, &asked.group).to_string() == label;
    let message = svrngs_core::set::prepared(&asked.group, &asked.app, &asked.prepared);
    if !ours || !svrngs_core::verify_hex(&asked.by, &message, &asked.sig) {
        return Err(why("what came back isn’t a computer’s prepare of it"));
    }
    let (p, changes, rows, schema) = read(&asked.prepared).ok_or_else(|| why("its prepare can’t be read"))?;
    let listed = (p.ops == asked.ops).then(|| asked.ops.as_array()).flatten();
    let ops = listed.and_then(|l| l.iter().map(|o| op(o, &p.keys)).collect::<Option<Vec<_>>>());
    let ops = ops.ok_or_else(|| why("the computer prepared other ops than it asked"))?;
    // Each change row is one the ops leave, in their order ([`Left`]): a
    // row's own change at an odd causal length where an insert makes it, an
    // even one where a delete removes it last; a column of a row that
    // stands, at an odd one, at the value they name it, or NULL on a row an
    // insert makes where they don't; each version at most [`MOST`].
    let net = (p.stands.len() == ops.len()).then(|| net(&ops, &p.stands));
    let net = net.ok_or_else(|| why("its prepare can’t be read"))?;
    for c in &rows {
        let (life, versions) = (c.cl % 2 == 1, 1..=MOST);
        let left = net.get(&(c.table.as_str(), &c.pk[..]));
        let fits = left.is_some_and(|l| match c.cid.as_str() {
            "-1" if life => l.made,
            "-1" => l.gone,
            cid => {
                let unnamed = l.made && !l.named.contains_key(cid) && c.val == Sql::Null;
                life && l.stands && (l.named.get(cid) == Some(&c.val) || unnamed)
            }
        });
        (fits && versions.contains(&c.col_version) && versions.contains(&c.cl))
            .then_some(())
            .ok_or_else(|| why("the computer’s rows hold a change it didn’t ask for"))?;
    }
    // And each effect they have is there: a row they make has a change, and
    // each column they name, unless NULL, its value's; one they leave as it
    // stood, each column they name changed from how it stood; one they
    // remove, its own.
    let made = |key: &(&str, &[u8]), cid: Option<&str>| {
        let at = |c: &&change::Change| (c.table.as_str(), &c.pk[..]) == *key;
        rows.iter().filter(at).any(|c| cid.is_none_or(|cid| c.cid == cid))
    };
    let whole = net.iter().all(|(key, l)| {
        let mut named = l.named.iter();
        match (&l.before, l.stands) {
            _ if l.made => made(key, None) && named.all(|(cid, v)| *v == Sql::Null || made(key, Some(cid))),
            (Some(b), true) => named.all(|(cid, v)| b.get(cid) == Some(v) || made(key, Some(cid))),
            (Some(_), false) => made(key, Some("-1")),
            (None, _) => true,
        }
    });
    if !whole {
        return Err(why("the computer left out a change it asked for"));
    }
    // A write that changes nothing, every op checked so, is no set.
    if rows.is_empty() {
        return Ok(None);
    }
    // The phone's device is its pass's person, and the open's approver
    // signs, where the pass holds as every copy checks it (wire.rs
    // `Phoned::signer`).
    let pass = svrngs_core::pass::Pass::decode(&asked.pass);
    let device = pass
        .and_then(|p| wire::Device::parse(&p.person))
        .zip(wire::Device::parse(&asked.by));
    let (device, home) = device.ok_or_else(|| why("its open’s pass can’t be read"))?;
    let seq = wire::Seq::new(asked.seq).ok_or_else(|| why("this phone’s place for it can’t be read"))?;
    let at = (wire::nonce(&asked.by, &asked.store), seq, schema);
    let mut set = wire::Set::unsigned(device, at, changes);
    let phoned = wire::Phoned {
        home,
        store: asked.store,
        pass: asked.pass.clone(),
        ts: asked.ts,
    };
    let signer = phoned.signer(&set.head, (&asked.group, &asked.app));
    let (_, message) = signer.ok_or_else(|| why("its open’s pass doesn’t hold"))?;
    set.phone = Some(phoned);
    let set = postcard::to_allocvec(&set).expect("a set is written in memory");
    let place = format!("{}/{}", asked.by, asked.seq);
    Ok(Some((message, STANDARD.encode(set), place)))
}

/// `set`, a set [`signed`] made, with its approver's signature `sig`: the
/// set the phone carries, postcard's bytes in base64; none where either
/// isn't one.
pub fn sealed(set: &str, sig: &str) -> Option<String> {
    let mut set: wire::Set = postcard::from_bytes(&STANDARD.decode(set).ok()?).ok()?;
    set.head.sig = wire::Sig::parse(sig)?;
    Some(STANDARD.encode(postcard::to_allocvec(&set).ok()?))
}

/// A prepare read: it, its changes as bytes and as rows, postcard's whole,
/// and its schema's hash.
type Read = (Prepared, Vec<u8>, Vec<change::Change>, [u8; 32]);

/// A prepare's `text`, read.
fn read(text: &str) -> Option<Read> {
    let p: Prepared = serde_json::from_str(text).ok()?;
    let changes = STANDARD.decode(&p.changes).ok()?;
    let (rows, rest) = postcard::take_from_bytes::<Vec<change::Change>>(&changes).ok()?;
    let schema = STANDARD.decode(&p.schema).ok()?.try_into().ok()?;
    rest.is_empty().then_some((p, changes.clone(), rows, schema))
}

/// `op`, of the write form, read against each table's key columns in
/// order; none where it isn't one, or names no column of its key.
fn op<'v>(op: &'v Value, keys: &BTreeMap<String, Vec<String>>) -> Option<Op<'v>> {
    let kinds = ["insert", "update", "delete"];
    let kind = kinds.iter().position(|k| op.get(k).is_some())?;
    let table = op[kinds[kind]].as_str()?;
    let field = |f: &str| -> Option<Vec<(&String, Sql)>> {
        let named = op.get(f)?.as_object()?;
        named.iter().map(|(c, v)| Some((c, cell(v)?))).collect()
    };
    let (key, columns) = (field(if kind == 0 { "row" } else { "key" })?, keys.get(table)?);
    let of = |k: &String| key.iter().find(|(c, _)| c == &k).map(|(_, v)| v.clone());
    let pk = pack(&columns.iter().map(of).collect::<Option<Vec<_>>>()?)?;
    let named = match kind {
        0 => key.into_iter().filter(|(c, _)| !columns.contains(c)).collect(),
        1 => field("set")?,
        _ => Vec::new(),
    };
    Some(Op { kind, table, pk, named })
}

/// The most a version a phone signs may be: half the bound every copy
/// holds (svrngs-db `change.rs`, `BOUND`), whatever the computer that
/// prepared it says the cell held. An honest cell rises one a write, so no
/// use comes near either half, and a cell a phone writes keeps at least as
/// much room above it as below: no computer can have a phone sign a set
/// that leaves a cell no room, as a version at the bound would freeze it.
const MOST: i64 = change::BOUND / 2;

/// A row as it stands: its columns to their values, none where there is
/// no row.
type Row = Option<BTreeMap<String, Sql>>;

/// A row the ops touch, as they leave it: as it stood before them, by the
/// computer's word (`stands`); whether it stands after, whether an insert
/// made it, and whether a delete removed it last; and each column they name
/// since it was made or removed, to the value they leave it.
#[derive(Default)]
struct Left {
    before: Row,
    stands: bool,
    made: bool,
    gone: bool,
    named: BTreeMap<String, Sql>,
}

/// Each row `ops` touch, by table and packed key, as they leave it, in
/// their order, as the write form runs them (svrngs-db form.rs), against
/// each op's row as it stood before any ran (svrngs-db `Db::prepare`): an
/// insert makes a row of what it names, or sets that on a row there; an
/// update sets what it names on a row there, and changes nothing where
/// there is none; a delete removes it.
fn net<'o>(ops: &'o [Op], stands: &[Value]) -> BTreeMap<(&'o str, &'o [u8]), Left> {
    let mut rows = BTreeMap::new();
    for (op, stood) in ops.iter().zip(stands) {
        let row =
            |o: &serde_json::Map<String, Value>| o.iter().filter_map(|(c, v)| Some((c.clone(), cell(v)?))).collect();
        let before: Row = stood.as_object().map(row);
        let left = rows.entry((op.table, &op.pk[..])).or_insert_with(|| Left {
            stands: before.is_some(),
            before,
            ..Left::default()
        });
        let named = op.named.iter().map(|(c, v)| ((*c).clone(), v.clone()));
        match (op.kind, left.stands) {
            (2, _) => (left.named, left.stands, left.made, left.gone) = (BTreeMap::new(), false, false, true),
            (0, false) => (left.named, left.stands, left.made, left.gone) = (named.collect(), true, true, false),
            (_, true) => left.named.extend(named),
            (_, false) => {}
        }
    }
    rows
}
