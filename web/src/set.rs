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
use svrngs_core::SigningKey;

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
/// (0, 1, 2), its table, its key packed, and the columns it names.
struct Op<'v> {
    kind: usize,
    table: &'v str,
    pk: Vec<u8>,
    named: Vec<(&'v String, Sql)>,
}

/// The set `key` signs for `asked`, its place `(home, epoch, seq)` beside
/// it, under the label `label` of the app whose page asked, or the words
/// for why not.
pub fn signed(key: &SigningKey, label: &str, asked: &Asked) -> Result<(String, (String, u64, u64)), String> {
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
    let mut covered = vec![false; ops.len()];
    for c in &rows {
        let i = ops.iter().position(|op| covers(op, c));
        covered[i.ok_or_else(|| why("the computer’s rows hold a change it didn’t ask for"))?] = true;
    }
    let unchanged = |(i, op): (usize, &Op)| covered[i] || stands(op, p.stands.get(i).unwrap_or(&Value::Null));
    if !ops.iter().enumerate().all(unchanged) {
        return Err(why("the computer left out a change it asked for"));
    }
    let at = (wire::nonce(&asked.by, p.epoch), seq, schema);
    let set = wire::Set::signed(key, (&asked.group, &asked.app), at, changes);
    let set = postcard::to_allocvec(&set).expect("a set is written in memory");
    Ok((STANDARD.encode(set), (asked.by.clone(), p.epoch, p.seq)))
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
    let key = field(if kind == 0 { "row" } else { "key" })?;
    let of = |k: &String| key.iter().find(|(c, _)| c == &k).map(|(_, v)| v.clone());
    let pk = packed(&keys.get(table)?.iter().map(of).collect::<Option<Vec<_>>>()?)?;
    let named = match kind {
        0 => key,
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

/// Whether the change row `c` is one of `op`'s effects.
fn covers(op: &Op, c: &change::Change) -> bool {
    let named = op.named.iter().find(|(n, _)| **n == c.cid).map(|(_, v)| v);
    let effect = match (c.cid.as_str(), op.kind) {
        ("-1", 2) => c.cl % 2 == 0,
        ("-1", _) => c.cl % 2 == 1,
        (_, 2) => false,
        (_, kind) => named.map_or(kind == 0 && c.val == Sql::Null, |v| *v == c.val),
    };
    op.table == c.table && op.pk == c.pk && effect
}

/// Whether `op`, with no row, is a no-op: its row, as `row` stood before
/// the write, already as the op would leave it, a delete's gone.
fn stands(op: &Op, row: &Value) -> bool {
    let same = |(c, v): &(&String, Sql)| value(&row[c.as_str()]).as_ref() == Some(v);
    match op.kind {
        2 => row.is_null(),
        _ => row.is_object() && op.named.iter().all(same),
    }
}
