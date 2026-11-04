// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend's address, and the pass that opens it (DESIGN §3, "Lent apps",
//! calls 1 and 6; u5-2). New: upstream had no lend at an origin of its own
//! and nothing a person signs for another key (PLAN §4, U5's inventory).
//!
//! **A lend's label** is one DNS label naming its lender's key and its
//! name, so a lend can be an origin of its own — `<label>.localhost` on a
//! computer, `<label>.<page host>` on a phone — under a wildcard
//! certificate, which covers one label. A label holds at most 63 bytes, and
//! a lend's name may be 64, so the label is the lender's key in full, which
//! is dialled, then ten letters of the name's hash, which only its lender's
//! door reads back, from what it declares: 62 letters of [`BASE32`].
//!
//! **A pass**: a person's key signs that a fresh key, the holder, may act
//! as them for an app — a lend, whose pages ask — toward it and the lends
//! it reads until a time. The page signs one for the lend it opens, and the
//! lend's origin dials with the holder, so the person's key never enters a
//! lend's origin; a computer signs one for itself, naming the app its page
//! is (sup-1). The door checks it ([`Pass::check`]) and then sets the
//! person as it would for the person's own key, and the app from `app`.
//! Pure: the time is an argument.

use alloc::string::String;
use alloc::vec::Vec;

use base64::Engine as _;
use serde::{Deserialize, Serialize};

use crate::grant::{Capability::Append, Grant};
use crate::{actor_of, field, sign_message, verify_hex, SigningKey};

/// Base32's alphabet for a label: 22 lowercase letters and the ten digits,
/// each a letter a DNS label takes. Not RFC 4648's: `e`, `i`, `o` and `y`
/// are left out, and every word DESIGN §1 keeps from people
/// ([`crate::link::KEPT_WORDS`]) holds one, so no label — printed to a
/// person as a lend's address, and fixed by its lender's key, so never
/// drawn again as a link's token is — spells one.
const BASE32: &[u8; 32] = b"abcdfghjklmnpqrstuvwxz0123456789";

/// The lender's 32 bytes in base32, and the name's part.
const KEY_LETTERS: usize = 52;
const NAME_LETTERS: usize = 10;

/// `bytes` in [`BASE32`], unpadded.
fn base32(bytes: &[u8]) -> String {
    let (mut out, mut acc, mut bits) = (String::new(), 0u32, 0);
    for &b in bytes {
        acc = (acc << 8) | u32::from(b);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(BASE32[((acc >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(BASE32[((acc << (5 - bits)) & 31) as usize] as char);
    }
    out
}

/// The 32 bytes [`base32`] wrote as `letters`, or `None`.
fn key_of(letters: &str) -> Option<[u8; 32]> {
    let (mut out, mut acc, mut bits) = (Vec::with_capacity(33), 0u32, 0);
    for c in letters.bytes() {
        let v = BASE32.iter().position(|&a| a == c)? as u32;
        acc = (acc << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    // 52 letters carry 260 bits: the key's 256 and four zeros.
    (acc & ((1 << bits) - 1) == 0).then_some(())?;
    out.try_into().ok()
}

/// A lend's label (the module docs).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Label(String);

impl Label {
    /// The label of the lend `name` lent by the computer whose key is
    /// `lender`.
    pub fn of(lender: &[u8; 32], name: &str) -> Self {
        let mut hashed = crate::sig::LABEL_DOMAIN.to_vec();
        hashed.extend_from_slice(name.as_bytes());
        let mut label = base32(lender);
        label.push_str(&base32(blake3::hash(&hashed).as_bytes())[..NAME_LETTERS]);
        Self(label)
    }

    /// [`Self::of`] for a lender named by its key in 64 hex, as the record
    /// spells an actor; `None` for text that is not a key — one reading for
    /// the node and the page (REVIEW_FINDINGS #58).
    pub fn of_hex(lender: &str, name: &str) -> Option<Self> {
        let key: [u8; 32] = hex::decode(lender).ok()?.try_into().ok()?;
        Some(Self::of(&key, name))
    }

    /// `raw` as a label: 62 letters of [`BASE32`] whose first 52 spell a
    /// key. `None` for anything else, in any case: a host is lowercased
    /// before it is read.
    pub fn parse(raw: &str) -> Option<Self> {
        (raw.len() == KEY_LETTERS + NAME_LETTERS && raw.bytes().all(|c| BASE32.contains(&c)))
            .then_some(())?;
        key_of(&raw[..KEY_LETTERS])?;
        Some(Self(raw.into()))
    }

    /// The label of the app `kind` carried in `group` and lent by `lender`
    /// ([`carried_name`]): what a holder's pass names as its app, so the
    /// fold reads the kind a holder may write from it (capability-signed).
    pub fn carried(lender: &[u8; 32], kind: &str, group: &str) -> Self {
        Self::of(lender, &carried_name(kind, group))
    }

    /// The lender's key, dialled to reach the lend.
    pub fn lender(&self) -> [u8; 32] {
        key_of(&self.0[..KEY_LETTERS]).expect("a label parsed holds a key")
    }
}
crate::text_newtype!(Label);

impl TryFrom<String> for Label {
    type Error = &'static str;
    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::parse(&raw).ok_or("not a lend's label")
    }
}

impl From<Label> for String {
    fn from(label: Label) -> Self {
        label.0
    }
}

/// The name of the app `kind` carried in `group`: the kind's and the group
/// id's first 16 letters, so a computer in two groups carrying an app of one
/// name gives each a label of its own (COMMUNITY_APPS §7, found 2). One
/// decider for the node's lend and the fold's [`Label::carried`].
pub fn carried_name(kind: &str, group: &str) -> String {
    alloc::format!("{kind}-{}", group.get(..16).unwrap_or(group))
}

/// A pass (the module docs): keys as the record spells an actor, its
/// lapse in seconds since 1970.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pass {
    /// The person it lets act: their key.
    pub person: String,
    /// The key that may act as them.
    pub holder: String,
    /// The app asking: the lend whose pages hold the pass, which it opens.
    pub app: Label,
    /// The other lends it opens: those of the kinds the app reads.
    pub reads: Vec<Label>,
    /// When it stops opening them.
    pub until: i64,
    /// The person's signature over the five, and the approver when named.
    pub sig: String,
    /// The key that approves the holder's entries, one per open, named by
    /// the person: the group's page mints it and hands it to its signer,
    /// never to the app (svrn-docs rings/current/HOLDER_APPROVAL.md §3.D).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approver: Option<String>,
    /// The one entry this grant approves: its id without its grant
    /// ([`crate::grant::bare_id`]). Never in the person's signature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub op: Option<crate::OpId>,
    /// The approver's signature over [`Self::approval_message`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<String>,
}

/// Why a pass opens nothing here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PassFault {
    #[error("an entry approval is not a reach pass")]
    NotReach,
    #[error("not signed by the person it names")]
    Unsigned,
    #[error("held by another key than the one presenting it")]
    NotTheHolder,
    #[error("lapsed")]
    Lapsed,
    #[error("it does not name this lend")]
    NotNamed,
    #[error("its link names more than the pass it carries")]
    Wider,
}

impl Pass {
    /// `person` lets `holder`, actor hex, act as them for `app` toward it
    /// and `reads` until `until`.
    pub fn sign(person: &SigningKey, holder: &str, lends: (Label, Vec<Label>), until: i64) -> Self {
        Self::approvable(person, holder, lends, until, None)
    }

    /// [`Self::sign`], naming `approver`, actor hex, as the key that
    /// approves the holder's entries ([`Self::approve`]).
    pub fn approvable(
        person: &SigningKey,
        holder: &str,
        (app, reads): (Label, Vec<Label>),
        until: i64,
        approver: Option<String>,
    ) -> Self {
        let mut pass = Self {
            person: actor_of(person),
            holder: holder.into(),
            app,
            reads,
            until,
            sig: String::new(),
            approver,
            op: None,
            approval: None,
        };
        pass.sig = sign_message(person, &pass.message());
        pass
    }

    /// `DOMAIN || person || holder || until[8 BE] || app || each read`,
    /// every variable field preceded by its `u32` big-endian length: the
    /// bytes a pass naming `[app, ...reads]` as its lends signed before the
    /// app had a field of its own (sup-1); then, with an approver,
    /// `"approver" || approver`, a tag no label's 62 letters can be.
    pub fn message(&self) -> Vec<u8> {
        let mut msg = crate::sig::PASS_DOMAIN.to_vec();
        field(&mut msg, self.person.as_bytes());
        field(&mut msg, self.holder.as_bytes());
        msg.extend_from_slice(&self.until.to_be_bytes());
        for lend in core::iter::once(&self.app).chain(&self.reads) {
            field(&mut msg, lend.as_str().as_bytes());
        }
        if let Some(approver) = &self.approver {
            field(&mut msg, b"approver");
            field(&mut msg, approver.as_bytes());
        }
        msg
    }

    /// What the approver signs: [`Self::message`], then `"op" || op`.
    pub fn approval_message(&self) -> Vec<u8> {
        let mut msg = self.message();
        if let Some(op) = &self.op {
            field(&mut msg, b"op");
            field(&mut msg, op.as_bytes());
        }
        msg
    }

    /// This grant naming `line`, unsigned, when its approver may approve
    /// it at `now`, the approver's clock: before the pass lapses, by its
    /// holder, appending only its app's part (HOLDER_APPROVAL §2). The
    /// holder's own timestamp is no evidence of when it wrote, so a line
    /// stamped after the pass lapses gets none either.
    pub fn approving(
        &self,
        line: &crate::Op<crate::SignedOp>,
        group: &str,
        now: i64,
    ) -> Option<Self> {
        let payload = match &line.kind.act {
            crate::RailAct::Record { payload } => Ok(Some(payload)),
            crate::RailAct::Correct { replacement, .. } => Ok(replacement.as_ref()),
            _ => Err(()),
        };
        let may = |p: &crate::Payload| Grant(self, group).may(&line.actor, Append(p.app()), now);
        let scoped = payload.is_ok_and(|p| p.is_none_or(|p| may(p).is_ok()));
        let live = now < self.until && line.ts_unix < self.until;
        let approves = self.approver.is_some() && self.op.is_none() && self.signed();
        tracing::debug!(target: "rail:pass", holder = %line.actor, scoped, live, approves, "pass: an entry is approved only by its pass's approver, while the pass is live, in its app's part");
        (scoped && live && approves && self.holder == line.actor).then(|| Self {
            op: Some(crate::grant::bare_id(line, group)),
            ..self.clone()
        })
    }

    /// [`Self::approving`], signed by `approver`, the key this pass names.
    pub fn approve(
        &self,
        approver: &SigningKey,
        line: &crate::Op<crate::SignedOp>,
        group: &str,
        now: i64,
    ) -> Option<Self> {
        let mut pass = self.approving(line, group, now)?;
        (pass.approver.as_deref() == Some(actor_of(approver).as_str())).then_some(())?;
        pass.approval = Some(sign_message(approver, &pass.approval_message()));
        Some(pass)
    }

    /// Whether this pass, presented by `holder` at `now`, opens `lend`: its
    /// app or one of its reads.
    pub fn check(&self, holder: &str, lend: &Label, now: i64) -> Result<(), PassFault> {
        if self.op.is_some() {
            return Err(PassFault::NotReach);
        }
        if !self.signed() {
            return Err(PassFault::Unsigned);
        }
        if self.holder != holder {
            return Err(PassFault::NotTheHolder);
        }
        if now >= self.until {
            return Err(PassFault::Lapsed);
        }
        if &self.app != lend && !self.reads.contains(lend) {
            return Err(PassFault::NotNamed);
        }
        Ok(())
    }

    /// Whether `link`, presented by `to` at `now`, carries this pass to it
    /// for `lend`: each opens `lend` now, `link` held by `to` and
    /// signed by this pass's holder, and naming no app, lend or time this
    /// one does not (capability-signed; DESIGN §3). A phone's holder, a key
    /// no page can copy, names each worker run's endpoint key so.
    pub fn carries(&self, link: &Pass, to: &str, lend: &Label, now: i64) -> Result<(), PassFault> {
        link.check(to, lend, now)?;
        self.check(&link.person, lend, now)?;
        let within = link.reads.iter().all(|r| self.reads.contains(r));
        match within && link.app == self.app && link.until <= self.until {
            true => Ok(()),
            false => Err(PassFault::Wider),
        }
    }

    /// The link this pass's holder signs to `holder`, unsigned, its
    /// [`Self::message`] what the holder signs: the same lends and time, so
    /// [`Self::carries`] passes it.
    pub fn link(&self, holder: &str) -> Self {
        let mut link = self.clone();
        (link.person, link.holder, link.sig) = (self.holder.clone(), holder.into(), String::new());
        link
    }

    /// Whether the person it names signed it and, where it names an
    /// entry, its approver approved that entry. Each signature in one
    /// spelling only, lowercase hex, so no pass has two encodings and no
    /// entry two ids.
    pub fn signed(&self) -> bool {
        let one = |sig: &str| {
            sig.len() == 128 && sig.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        };
        let approved = match (&self.op, &self.approver, &self.approval) {
            (None, _, None) => true,
            (Some(_), Some(by), Some(sig)) => {
                one(sig) && verify_hex(by, &self.approval_message(), sig)
            }
            _ => false,
        };
        one(&self.sig) && approved && verify_hex(&self.person, &self.message(), &self.sig)
    }

    /// The pass as a header carries it: its JSON in base64url.
    pub fn encode(&self) -> String {
        let json = serde_json::to_vec(self).expect("a pass serialises");
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
    }

    /// A pass [`Pass::encode`] wrote, or `None`.
    pub fn decode(raw: &str) -> Option<Self> {
        let json = (base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(raw.trim())).ok()?;
        serde_json::from_slice(&json).ok()
    }
}

#[cfg(test)]
#[path = "pass_tests.rs"]
mod tests;
