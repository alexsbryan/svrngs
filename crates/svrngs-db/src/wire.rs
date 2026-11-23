// SPDX-License-Identifier: AGPL-3.0-or-later
//! A change set as it travels and is kept, as postcard writes it: the one
//! home of its form. svrngs-db reads and writes it (`sync.rs`), and a
//! phone's signer compiles this file too (`web/src/set.rs`, by `#[path]`),
//! so the set a phone signs is the one every copy reads. Pure, of serde and
//! svrngs-core alone, so it builds for the page as well; what SQLite binds
//! of a seq and a key is svrngs-db's (`sync.rs`).

use serde::{Deserialize, Serialize};
use svrngs_core::SigningKey;
use svrngs_core::set::{At, Nonce};

/// A set's seq at its database, and a mark, a have or a twin's place: from 1
/// to `i64::MAX`, what SQLite holds. Parsed as the wire is read, so a call
/// naming one past it doesn't decode, and no seq reaches a bind that rusqlite
/// would refuse (a `u64` past `i64::MAX`), or an add past it. Written as
/// its number, as postcard writes a newtype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u64")]
pub struct Seq(pub(crate) u64);

impl Seq {
    /// `n` as a seq, if it is one.
    pub fn new(n: u64) -> Option<Self> {
        (1..=i64::MAX as u64).contains(&n).then_some(Self(n))
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for Seq {
    type Error = String;
    fn try_from(n: u64) -> Result<Self, String> {
        Self::new(n).ok_or_else(|| format!("{n} is no seq: one is from 1 to {}", i64::MAX))
    }
}

/// `N` bytes in lower-case hex, as the record spells a key (`actor_of`) and
/// a signature: parsed as the wire is read, so a device is spelled as the
/// record spells it, and one signature never has two spellings. Written as
/// its text, as postcard writes a newtype.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct Hex<const N: usize>(pub(crate) String);

/// A device's key, as the record spells it.
pub type Device = Hex<32>;

/// A device's signature over a set.
pub type Sig = Hex<64>;

impl<const N: usize> Hex<N> {
    /// `text` as `N` bytes of lower-case hex, if it is.
    pub fn parse(text: &str) -> Option<Self> {
        let hex = text.len() == 2 * N && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        hex.then(|| Self(text.into()))
    }
}

svrngs_core::text_newtype!(Device, Sig);

impl<const N: usize> TryFrom<String> for Hex<N> {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        Self::parse(&text).ok_or_else(|| format!("not {N} bytes of lower-case hex"))
    }
}

/// A set's signed head: its device, the nonce of the database it was
/// written in, its seq there, its schema's hash, its changes' hash
/// (`svrngs_core::set::changes`), and the device's signature over them all
/// (`At::message`). Ordered as its fields are, so by device, nonce and seq
/// first.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Head {
    pub device: Device,
    pub nonce: Nonce,
    pub seq: Seq,
    pub schema: [u8; 32],
    pub changes: [u8; 32],
    pub sig: Sig,
}

/// One change set as it travels and is kept: its signed head, the changes
/// whose hash it signs, and, for a phone's, what it carries beside them
/// ([`Phoned`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Set {
    pub head: Head,
    pub changes: Vec<u8>,
    pub phone: Option<Phoned>,
}

/// What a phone's set carries beside its head (svrn-docs
/// `rings/current/APP_STATE.md` §4, "Phones"; the operator's B′,
/// 2026-11-22): its home, the computer that prepared it and applies it
/// first, which vouches for its site's mark as a computer vouches for its
/// own (`app-state-diligence/sync-simplify-2026-11-22/`, change 4); the id
/// of the phone's signer's store that signed it, drawn with the store, so
/// the nonce is derived from the two ([`nonce`]); the pass of the open it
/// was written under, which names its signer, that open's approver; and
/// the stamp it was signed at, by the phone's clock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Phoned {
    pub home: Device,
    pub store: Nonce,
    pub pass: String,
    pub ts: i64,
}

/// What an open's approver signs for a phone's set: a domain of its own,
/// then the set's own message and its stamp.
const PHONED: &[u8] = b"svrngs/db-phone-set/v1\0";

impl Phoned {
    /// The key `head`, a phone's, is signed by, and what it signs, as a set
    /// of `app` in `group`, where its pass holds: signed by the set's device,
    /// opening that app's carried label, naming an approver, the stamp
    /// within its lapse, and the nonce derived from the home and store. So
    /// a leaked approver is worth one pass's scope, and its stamp can be
    /// backdated only within that pass's lapse (svrn-docs HOLDER_APPROVAL
    /// §2, rule 4).
    pub(crate) fn signer(&self, head: &Head, (group, app): (&str, &str)) -> Option<(String, Vec<u8>)> {
        let pass = svrngs_core::pass::Pass::decode(&self.pass)?;
        let opens = pass.app == svrngs_core::pass::Label::carried(app, group);
        let homed = nonce(self.home.as_str(), &self.store) == head.nonce;
        let holds = pass.signed() && pass.person == head.device.as_str() && opens && self.ts <= pass.until && homed;
        let mut message = PHONED.to_vec();
        message.extend(head.at((group, app)).message(&head.changes));
        message.extend(self.ts.to_be_bytes());
        holds.then_some((pass.approver?, message))
    }
}

impl Head {
    /// What it signs, as a set of `app` in `group` (`At::message`).
    pub(crate) fn at<'h>(&'h self, (group, app): (&'h str, &'h str)) -> At<'h> {
        let (device, nonce, seq, schema) = (self.device.as_str(), &self.nonce, self.seq.0, &self.schema);
        At {
            group,
            app,
            device,
            nonce,
            seq,
            schema,
        }
    }
}

impl Set {
    /// `changes`, written under `schema`, signed by `key` at `seq` of its
    /// database of `nonce`, a set of `app` in `group`: the one place a
    /// computer's set is made and signed.
    pub(crate) fn signed(key: &SigningKey, names: (&str, &str), at: (Nonce, Seq, [u8; 32]), changes: Vec<u8>) -> Self {
        let mut set = Self::unsigned(Hex(svrngs_core::actor_of(key)), at, changes);
        set.head.sig = Hex(set.head.at(names).sign(key, &set.head.changes));
        set
    }

    /// `changes`, written under `schema`, at `seq` of `device`'s database of
    /// `nonce`, its signature still to come: the one place a set is made.
    pub(crate) fn unsigned(device: Device, at: (Nonce, Seq, [u8; 32]), changes: Vec<u8>) -> Self {
        let ((nonce, seq, schema), hash) = (at, svrngs_core::set::changes(&changes));
        let sig = Hex("0".repeat(128));
        let head = Head {
            device,
            nonce,
            seq,
            schema,
            changes: hash,
            sig,
        };
        Self {
            head,
            changes,
            phone: None,
        }
    }
}

/// The nonce of a phone's database at its home, the computer `home` that
/// prepares its sets and applies them first, for its signer's `store`
/// (svrn-docs `rings/current/app-state-diligence/sync-simplify-2026-11-22/`,
/// change 4; `invariants-2026-11-22/`, change 5): derived, so a set's home
/// is read from the nonce its device signed, and a store made anew, its
/// count gone with it, is a site of its own at every home.
pub(crate) fn nonce(home: &str, store: &Nonce) -> Nonce {
    let mut bytes = b"svrngs/db-home/v2\0".to_vec();
    svrngs_core::field(&mut bytes, home.as_bytes());
    bytes.extend_from_slice(store);
    svrngs_core::ContentHash::of(&bytes).as_bytes()[..16]
        .try_into()
        .expect("a hash is longer")
}
