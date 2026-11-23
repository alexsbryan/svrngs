// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a phone signs for a write to a carried app's database (svrn-docs
//! `rings/current/APP_STATE.md` §4, "Phones"; MAGNETS_BUILD S2), run by the
//! group's signer alone (web/sign.js), never by the app's page or its
//! worker: a computer's prepare of the page's write list, taken only as a
//! computer of the open signed it (svrngs-db `Db::prepare`), checked against
//! that list, and signed as the phone's own set at the place the computer,
//! its home there, hands it. Pure: the phone's key in, the set's bytes or
//! the words for why not out. The set's form and its changes' are
//! svrngs-db's own files, compiled here (`wire.rs`, `change.rs`).
//!
//! Each change row must be one op's: its table, and its key packed as
//! cr-sqlite packs one; the row's own change (`-1`) at an odd causal length
//! for an insert or update, an even one for a delete; any other, a column
//! the op names at the op's value, or, for an insert, one it doesn't name
//! at NULL. An op with no row is a no-op only where its row stood, before
//! the write, as the op would leave it. What the phone can't check is the residual (§4, "The
//! residual"): the causal length and column version the computer set, the
//! row a no-op stands on, and the order of a composite key's columns, each
//! as that computer reads its copy, as it serves the app's page too.

use std::collections::BTreeMap;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Deserialize;
use serde_json::Value;
use svrngs_core::pass::Label;

#[path = "../../crates/svrngs-db/src/change.rs"]
#[rustfmt::skip]
pub(crate) mod change;
#[path = "../../crates/svrngs-db/src/wire.rs"]
#[rustfmt::skip]
#[allow(dead_code)]
mod wire;
type Sql = change::Value;

/// What the signer is asked to sign: the page's write list, in the group and
/// app the open names, and the computer's prepare of it, its text as `by`
/// signed it with `sig`.
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
}

/// A prepare's text (svrngs-db `Db::prepare`), as the phone reads it.
#[derive(Deserialize)]
struct Prepared {
    ops: Value,
    changes: String,
    schema: String,
    keys: BTreeMap<String, Vec<String>>,
    stands: Vec<Value>,
    epoch: u64,
    seq: u64,
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
/// postcard's bytes in base64, and its place, `home/epoch/seq`; or the
/// words for why not.
pub fn signed(label: &str, asked: &Asked) -> Result<(Vec<u8>, String, String), String> {
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
    let (p, changes, rows, schema, seq) = read(&asked.prepared).ok_or_else(|| why("its prepare can’t be read"))?;
    let listed = (p.ops == asked.ops).then(|| asked.ops.as_array()).flatten();
    let ops = listed.and_then(|l| l.iter().map(|o| op(o, &p.keys)).collect::<Option<Vec<_>>>());
    let ops = ops.ok_or_else(|| why("the computer prepared other ops than it asked"))?;
    // Each change row is one the ops leave, in their order: a row's own
    // change at an odd causal length where they leave a row, an even one
    // where they remove it; a column at the value they leave it at, NULL
    // where a row they make doesn't name it.
    let net = (p.stands.len() == ops.len()).then(|| net(&ops, &p.stands));
    let net = net.ok_or_else(|| why("its prepare can’t be read"))?;
    for c in &rows {
        let left = net.get(&(c.table.as_str(), &c.pk[..])).map(|(_, after)| after);
        let fits = match (left, c.cid.as_str()) {
            (Some(after), "-1") => (c.cl % 2 == 1) == after.is_some(),
            (Some(Some(row)), cid) => row.get(cid).unwrap_or(&Sql::Null) == &c.val,
            _ => false,
        };
        fits.then_some(())
            .ok_or_else(|| why("the computer’s rows hold a change it didn’t ask for"))?;
    }
    // And each effect they have is there: a row they make has a change, one
    // they remove its own, and each column changed from how it stood, or
    // named on a row they make, unless NULL, its value's.
    let made = |key: &(&str, &[u8]), cid: Option<&str>| {
        let at = |c: &&change::Change| (c.table.as_str(), &c.pk[..]) == *key;
        rows.iter().filter(at).any(|c| cid.is_none_or(|cid| c.cid == cid))
    };
    let whole = net.iter().all(|(key, (before, after))| match (before, after) {
        (Some(b), Some(a)) => a.iter().all(|(cid, v)| b.get(cid) == Some(v) || made(key, Some(cid))),
        (None, Some(a)) => made(key, None) && a.iter().all(|(cid, v)| *v == Sql::Null || made(key, Some(cid))),
        (Some(_), None) => made(key, Some("-1")),
        (None, None) => true,
    });
    if !whole {
        return Err(why("the computer left out a change it asked for"));
    }
    // The phone's device is its pass's person, and the open's approver
    // signs, where the pass holds as every copy checks it (wire.rs
    // `Phoned::signer`).
    let pass = svrngs_core::pass::Pass::decode(&asked.pass);
    let device = pass
        .and_then(|p| wire::Device::parse(&p.person))
        .zip(wire::Device::parse(&asked.by));
    let (device, home) = device.ok_or_else(|| why("its open’s pass can’t be read"))?;
    let mut set = wire::Set::unsigned(device, (wire::nonce(&asked.by, p.epoch), seq, schema), changes);
    let phoned = wire::Phoned {
        home,
        epoch: p.epoch,
        pass: asked.pass.clone(),
        ts: asked.ts,
    };
    let signer = phoned.signer(&set.head, (&asked.group, &asked.app));
    let (_, message) = signer.ok_or_else(|| why("its open’s pass doesn’t hold"))?;
    set.phone = Some(phoned);
    let set = postcard::to_allocvec(&set).expect("a set is written in memory");
    let place = format!("{}/{}/{}", asked.by, p.epoch, p.seq);
    Ok((message, STANDARD.encode(set), place))
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
/// its schema's hash, and the seq it hands the phone.
type Read = (Prepared, Vec<u8>, Vec<change::Change>, [u8; 32], wire::Seq);

/// A prepare's `text`, read.
fn read(text: &str) -> Option<Read> {
    let p: Prepared = serde_json::from_str(text).ok()?;
    let changes = STANDARD.decode(&p.changes).ok()?;
    let (rows, rest) = postcard::take_from_bytes::<Vec<change::Change>>(&changes).ok()?;
    let schema = STANDARD.decode(&p.schema).ok()?.try_into().ok()?;
    let seq = wire::Seq::new(p.seq)?;
    rest.is_empty().then_some((p, changes.clone(), rows, schema, seq))
}

/// `op`, of the write form, read against each table's key columns in
/// order; none where it isn't one, or names no column of its key.
fn op<'v>(op: &'v Value, keys: &BTreeMap<String, Vec<String>>) -> Option<Op<'v>> {
    let kinds = ["insert", "update", "delete"];
    let kind = kinds.iter().position(|k| op.get(k).is_some())?;
    let table = op[kinds[kind]].as_str()?;
    let field = |f: &str| -> Option<Vec<(&String, Sql)>> {
        let named = op.get(f)?.as_object()?;
        named.iter().map(|(c, v)| Some((c, value(v)?))).collect()
    };
    let (key, columns) = (field(if kind == 0 { "row" } else { "key" })?, keys.get(table)?);
    let of = |k: &String| key.iter().find(|(c, _)| c == &k).map(|(_, v)| v.clone());
    let pk = packed(&columns.iter().map(of).collect::<Option<Vec<_>>>()?)?;
    let named = match kind {
        0 => key.into_iter().filter(|(c, _)| !columns.contains(c)).collect(),
        1 => field("set")?,
        _ => Vec::new(),
    };
    Some(Op { kind, table, pk, named })
}

/// `v`, a value of the write form, as SQLite holds it (svrngs-db form.rs
/// `value`): null, a number, a string, or `{base64}` for a BLOB.
fn value(v: &Value) -> Option<Sql> {
    Some(match v {
        Value::Null => Sql::Null,
        Value::String(s) => Sql::Text(s.clone()),
        Value::Number(n) => n.as_i64().map(Sql::Integer).or(n.as_f64().map(Sql::Real))?,
        _ => Sql::Blob(STANDARD.decode(v.get("base64")?.as_str()?).ok()?),
    })
}

/// `values`, a key, as cr-sqlite packs one
/// (`vendor/cr-sqlite/core/rs/core/src/pack_columns.rs:31-82`), the form
/// svrngs-db's parse reads (`parse.rs`, `packed`): the count, then each
/// value's type and the fewest bytes of its integer or length, then its
/// bytes; none for NULL, which no key holds.
fn packed(values: &[Sql]) -> Option<Vec<u8>> {
    let mut out = vec![u8::try_from(values.len()).ok()?];
    for v in values {
        let (kind, n, bytes) = match v {
            Sql::Integer(i) => (1, *i as u64, &[][..]),
            Sql::Real(r) => (2, 0, &r.to_be_bytes()[..]),
            Sql::Text(t) => (3, t.len() as u64, t.as_bytes()),
            Sql::Blob(b) => (4, b.len() as u64, &b[..]),
            Sql::Null => return None,
        };
        let width = 8 - n.leading_zeros() as usize / 8;
        out.push(kind | (width as u8) << 3);
        out.extend_from_slice(&n.to_be_bytes()[8 - width..]);
        out.extend_from_slice(bytes);
    }
    Some(out)
}

/// A row as it stands: its columns to their values, none where there is
/// no row.
type Row = Option<BTreeMap<String, Sql>>;

/// Each row `ops` touch, by table and packed key: as it stood before the
/// write, from `stands`, each op's row read before any ran (svrngs-db
/// `Db::prepare`), and as the ops leave it, in their order, as the write
/// form runs them (svrngs-db form.rs): an insert makes a row of what it
/// names, or sets that on a row there; an update sets what it names on a
/// row there, and changes nothing where there is none; a delete removes it.
fn net<'o>(ops: &'o [Op], stands: &[Value]) -> BTreeMap<(&'o str, &'o [u8]), (Row, Row)> {
    let mut rows = BTreeMap::new();
    for (op, stood) in ops.iter().zip(stands) {
        let row =
            |o: &serde_json::Map<String, Value>| o.iter().filter_map(|(c, v)| Some((c.clone(), value(v)?))).collect();
        let before: Row = stood.as_object().map(row);
        let (_, now) = rows
            .entry((op.table, &op.pk[..]))
            .or_insert_with(|| (before.clone(), before));
        let named = op.named.iter().map(|(c, v)| ((*c).clone(), v.clone()));
        match (op.kind, now.as_mut()) {
            (2, _) => *now = None,
            (0, None) => *now = Some(named.collect()),
            (_, row) => row.into_iter().for_each(|r| r.extend(named.clone())),
        }
    }
    rows
}
