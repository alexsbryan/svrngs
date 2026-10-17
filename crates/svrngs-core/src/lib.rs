// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/lib.rs@e1a43bcd2
//! The ring rail — a shared append-only log that converges without a server.
//!
//! # What this is for
//!
//! A house of twenty people already keeps its shared state somewhere:
//! Splitwise for the expenses, a spreadsheet for the chores, a running Signal
//! thread for who has the drill. Every one of those is a private database
//! somewhere else. This is the same book, kept as an append-only journal on
//! each housemate's own machine, gossiped between them, and handed back in an
//! order every node agrees on.
//!
//! # The rail carries an opaque payload, and that is the whole design
//!
//! This started as an expense ledger, and the ledger's types were the rail's
//! types: the journal line's body was an `ExpenseOp`, and the reader's
//! failure enum had thirteen variants of which five were about money. It
//! worked, and it was one layer pretending to be one thing.
//!
//! The cut is now explicit. **The rail knows about delivery, authenticity and
//! convergence. It does not know what an act means.**
//!
//! ```text
//!   Payload          the app's act, canonical bytes, opaque here
//!   RailAct          Record{payload} | Correct{corrects, replacement?}
//!   SignedOp         + seq + signature — the journal line's body
//!   Op<SignedOp>     corpus-engine's envelope: id, v, ts, actor
//!   RingJournal      the log on disk, one writer
//!   admit()          ops -> (acts in ONE order, gaps)
//!   <the app>        acts -> whatever the app is about
//! ```
//!
//! What the app gets back from [`admit`] is already deduplicated, signature-
//! checked, roster-admitted, sequence-audited, void-filtered and **totally
//! ordered**. Its reducer runs over that list and cannot reintroduce a
//! divergence, because there is no ordering decision left to make.
//!
//! # Two things live in the rail that look like they belong to an app
//!
//! **Correction.** "This earlier act was wrong, and it never comes back" is
//! not an expense rule — a tool-lending board needs it the first time someone
//! writes *I returned the drill* and then *no I didn't*. It is also the rule
//! most easily got wrong: the void set has to be built from every correction
//! at once, never by walking for liveness, or it stops being commutative.
//! One implementation, in [`admit`], and no app re-derives it.
//!
//! **The roster.** Who is in this ring and which keys they sign with is
//! membership, not meaning. An op signed by a key nobody claims is a gap
//! before any app sees it.
//!
//! # Three decisions worth knowing before reading the code
//!
//! **The node key signs, the roster binds.** An op's `actor` is the hex
//! public key that signed it, because that is the only field in the whole
//! line the writer cannot forge for someone else (ARCH §18.1). The [`Roster`]
//! then says which keys belong to which person. One person with two laptops
//! is two entries in one roster row — accepted, not solved.
//!
//! **A correction is a compensating entry, and it is visible.** Nothing is
//! ever rewritten or deleted. A [`RailAct::Correct`] voids an earlier op and
//! may re-state it; the void is permanent, so a correction never resurrects
//! what an earlier correction voided. To put something back, write it again.
//!
//! **The rail refuses to fake completeness.** [`admit`] returns
//! [`RailGap`]s alongside the acts. A log that cannot say "I may be missing
//! something" lets an app state a wrong total with complete confidence, which
//! is worse than refusing to answer (ARCH §18.3).
//!
//! # Where the journal is
//!
//! This crate is the FOLD and nothing else: vocabulary, signing, admission,
//! the sync digest. It performs no I/O and opens no file. The append-only
//! journal on disk — `RingRail`, `RingJournal`, and the door that refuses
//! what the rail can judge — is `commonwealth-rail`, which depends on this
//! crate. The split is what lets a second application (canon) reuse the fold
//! without inheriting a file layout, and it is why `admit` takes ops rather
//! than a path.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod act_set;
mod admit;
pub mod alpn;
pub mod app;
mod command;
mod hash;
pub mod joining;
pub mod link;
mod membership;
mod op;
pub mod pass;
mod payload;
mod project;
mod record;
mod register;
pub mod relay;
pub mod seed;
mod sig;
mod sync;
mod synced;
mod view;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// The key an act is signed with ([`sign_act`]): the one signer. It was a
/// `RingSigner` trait with this as its one shipped implementation until
/// arch-core; the trait let the rail hold a signer without the key, and the
/// rail holds the key all the same.
pub use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};

pub use act_set::ActSet;
pub use admit::{admit, body_json, Admission, AdmittedOp, GapClass, RailGap};
pub use command::{commit, Author, Authored, Command, Reads, Recorded, Refused, Store, Tip};
pub use membership::{membership, standing_reads, Cut, InviteFault, Invited, Joined, Membership};
pub use payload::{Payload, PayloadError, MAX_PAYLOAD_BYTES};
pub use project::{Invites, Wire};
pub use record::{Append, AppendRefusal};
pub use register::{
    HandOnRefusal, HandOnTo, Leave, MintInvite, Redeem, RedeemRefusal, RelayAt, Removal,
    RemovalRefusal,
};
pub use sig::{
    act_id, actor_of, derive_id, field, redemption_message, ring_op_message, sign_act,
    sign_message, sign_ring_op, verify_hex,
};
pub use sync::{digest, ops_missing_from_within, DigestFold, NO_BUDGET};
pub use synced::{Change, Entry, Files, NotASyncPath, Placed, Runs, SyncFolder, SyncPath};
pub use view::{Digest, View, DIGEST_V};

/// The journal envelope, re-exported so a consumer of the rail names ONE
/// crate. `Op<SignedOp>` is what crosses the ring-sync wire
/// (`commonwealth-api/src/routes_internal/ring_sync.rs`,
/// `sovereign-mesh/src/ring_sync.rs`), and a caller that had to name `oplog`
/// separately would be free to reach a different version of it.
pub use hash::ContentHash;
pub use op::{Journaled, Op, OpId, SkippedLine};

// Until 2026-09-04 neither `SigningKey` (above) nor serde_json's `Value` was
// exported — so the only signer was a type the crate refused to hand you, and
// the only `Payload` constructor took a type it never named. A mismatch makes
// another ed25519-dalek's key not the one `sign_act` takes, a type error a
// long way from its cause. Found by
// physically lifting the package (cw-lift). `Value`'s re-export went in lean-3:
// svrngs's crates take serde_json from the workspace's one pin, and only a test
// read it here.

// ── People ───────────────────────────────────────────────────

/// A name kept as text, read as it: `as_str`, and `Display` writing those
/// bytes — each name's own, where every one wrote the two alike. For a
/// newtype whose field is `String` or another of these; never for a secret,
/// which no `Display` may print.
#[macro_export]
macro_rules! text_newtype {
    ($($name:ty),+) => {$(
        impl $name {
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(self.as_str())
            }
        }
    )+};
}
crate::text_newtype!(Name, Person);

/// A name a person reads: a group's, or a person's. Trimmed, 1..=64 bytes,
/// no control characters: it is printed in a terminal, carried in a group's
/// seed and, as a [`Person`], signed into every copy of the record (svrngs,
/// u2-9; REVIEW_FINDINGS #23). [`Self::parse`] is the one rule, for the
/// command and the node alike.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

/// Why some text is not a [`Name`]. Its words are a person's (DESIGN §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAName {
    #[error("a name cannot be empty")]
    Empty,
    #[error("a name is at most 64 bytes")]
    TooLong,
    #[error("a name cannot carry control characters")]
    Control,
    /// Kept text is read as written, never trimmed: trimming it would
    /// change bytes a signature covers.
    #[error("a name kept on this computer cannot start or end with a space")]
    Untrimmed,
}

impl Name {
    /// A name from what a person typed, trimmed.
    pub fn parse(raw: &str) -> Result<Self, NotAName> {
        let name = raw.trim();
        let why = if name.is_empty() {
            NotAName::Empty
        } else if name.len() > 64 {
            NotAName::TooLong
        } else if name.chars().any(char::is_control) {
            NotAName::Control
        } else {
            return Ok(Self(name.to_string()));
        };
        tracing::debug!(target: "name", raw, why = ?why, "name: not a name");
        Err(why)
    }

    /// A name from text this computer keeps — a record's line, a request
    /// on the wire — which must already be as [`Self::parse`] makes it.
    pub fn kept(raw: String) -> Result<Self, NotAName> {
        let name = Self::parse(&raw)?;
        if name.0 != raw {
            tracing::debug!(target: "name", raw, "name: not a name — spaces at its ends");
            return Err(NotAName::Untrimmed);
        }
        Ok(name)
    }
}

/// A member of the ring, by the name the house calls them.
///
/// A display name and not a key: the whole point of a ring app is that "Alex
/// paid for groceries" renders as *Alex*. The binding from name to signing
/// key lives in the [`Roster`], which is why the name can be changed without
/// rewriting history.
///
/// Made only from a [`Name`] (svrngs, u2-9): deserialising one runs
/// [`Name::kept`], so a line naming an empty person, 10 KB or a terminal's
/// escape codes is a [`RailGap::MalformedLine`], never an act.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Person(Name);

impl Person {
    /// A person from what someone typed: [`Name::parse`].
    pub fn parse(raw: &str) -> Result<Self, NotAName> {
        Name::parse(raw).map(Self)
    }

    pub fn name(&self) -> &Name {
        &self.0
    }
}

impl From<Name> for Person {
    fn from(name: Name) -> Self {
        Self(name)
    }
}

impl Serialize for Person {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Person {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Name::kept(raw).map(Self).map_err(serde::de::Error::custom)
    }
}

/// Who is in this ring, and which keys they sign with.
///
/// A parameter of [`admit`], never part of the admitted state. It decides
/// *membership* — an op signed by a key nobody claims is a gap, not an act —
/// and it is never handed to an app as anything but the `person` on an
/// already-admitted op. That separation is what lets an app's arithmetic stay
/// a function of the op set as people join and leave: an app that reads the
/// roster to decide who shares a cost re-divides every past expense the day a
/// housemate moves in, and the reference app has a test pinning that it does
/// not.
///
/// (svrngs: upstream's row could also carry the introduction a key was
/// admitted on, `vouches`; svrngs never wrote one, and it went with
/// `introduce.rs` in core-0. A roster written with `vouches` still parses,
/// since unknown fields are ignored, and one without serializes to the bytes
/// it always did.)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roster {
    /// Person → the hex node public keys that person signs with. Two laptops
    /// is two keys in one row.
    pub members: BTreeMap<Person, Vec<String>>,
}

impl Roster {
    pub fn new(members: BTreeMap<Person, Vec<String>>) -> Self {
        Self { members }
    }
}

// ── What a line says ─────────────────────────────────────────

/// What one line of a ring journal asserts.
///
/// Two cases, and the rail understands neither payload. `Correct` is here
/// rather than in an app because voiding-without-resurrection is the rule
/// that makes the whole thing commutative, and it is generic — see the module
/// docs.
///
/// A correction's `replacement` is a [`Payload`] and never another
/// `RailAct`, so a correction of a correction is *unrepresentable* rather
/// than checked (ARCH §7.1). There is no nested-correction branch in
/// [`admit`] because there is no way to write one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum RailAct {
    /// The app said something.
    Record { payload: Payload },
    /// An earlier op was wrong. It is voided — permanently, and visibly — and
    /// optionally re-stated by `replacement`.
    ///
    /// A correction of a correction cancels that correction's replacement and
    /// leaves the original op voided. Nothing ever comes back to life; to
    /// restore an act, write it again.
    Correct {
        corrects: OpId,
        replacement: Option<Payload>,
    },
    /// An older build's seal: it retired everything its signer wrote before
    /// it, on every copy it reached. Here it does nothing (arch-seal; svrn-docs
    /// `rings/current/A_RECORD_FOR_YEARS.md` §2, §6 q2): a line on disk is read
    /// and held as an act, [`admit`] says so at debug, and it retires, voids
    /// and carries nothing. No command writes one: an app's is refused as
    /// `NotAnAppAct`. The variant stays so an older journal still parses.
    Seal,
    /// A member says: this person and this key are in the ring. One of the
    /// two membership acts (`RING_APPLICATIONS.md` "Amendment 2026-09-18");
    /// membership is computed from the seed plus these and [`Self::Remove`],
    /// and nothing else on the journal changes it.
    ///
    /// The binding rides IN the record — the reason a `roster.json` edit
    /// cannot retroactively flip a member's past: acts admitted through an
    /// `Admit` resolve through the record, not through the file. Nothing
    /// undoes it but the keeper's [`Self::Remove`] (U7): a `Correct` of an
    /// act that moves membership voids nothing.
    ///
    /// Carries no [`Payload`] and never reaches an app reducer — an app that
    /// read membership would re-divide every past expense the day someone
    /// joins (the no-re-division property).
    Admit {
        person: Person,
        /// The key being admitted, as actor hex — the same spelling every
        /// signature on the wire already uses.
        key: String,
        /// The [`Self::Invite`] this `Admit` redeems, with the new key's
        /// consent (svrngs, U2; DESIGN §6 "A link admits each person once").
        /// `membership` counts a redeeming `Admit` only under an `Invite`
        /// its own signer wrote that still counts. Skipped when absent, so
        /// an `Admit` naming no `Invite` keeps its bytes, its id and its
        /// meaning.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        invite: Option<Redemption>,
    },
    /// A member says: this key is out (svrngs, U7; DESIGN §6 "Removal has one
    /// decider"). Anyone standing may sign one: from the group's keeper it is
    /// the cut, from anyone else it is an ask the keeper may sequence, and
    /// it cuts nobody. A removed key's own `Remove` counts nowhere.
    ///
    /// The cut names the removed key's acts by hash, never by position or
    /// time: `through` is that key's newest acts the remover held, by id,
    /// and the cut covers them and every act they descend from by `prev`.
    /// Every other act of the key counts nowhere, wherever its timestamp puts
    /// it — a backdated act, or one beside an act the remover saw. Empty when
    /// the remover held none. Nothing undoes a cut: a [`Self::Correct`] of
    /// an act that moves membership voids nothing.
    Remove {
        key: String,
        /// The removed key's newest acts the remover held: each act of the
        /// key that no other held act of the key names as its previous act.
        through: Vec<OpId>,
    },
    /// The keeper says: this standing key is the group's keeper now (svrngs,
    /// U7; DESIGN §6). The keeper at the seed is the maker's key; only the
    /// keeper's own `HandOn` moves it, so the keeper is one key at every
    /// place in the order. Carries no [`Payload`].
    HandOn { key: String },
    /// A member says: whoever holds this link's key may join, until `until`
    /// (svrngs, U2; DESIGN §6 "A link admits each person once"). `key` is
    /// the link's public key as actor hex; the secret half travels in the
    /// link and never in the record. Each use is an [`Self::Admit`] naming
    /// this act's id, signed by this act's signer; the issuer's next
    /// `Invite` closes the link. Carries no [`Payload`] and grants nothing
    /// itself.
    Invite {
        key: String,
        /// Unix seconds; a redeeming `Admit` counts only when timed before it.
        until: i64,
        /// A device link's (bash-devices; DESIGN §6): the person whose
        /// device it adds, by their first key. It counts only when its
        /// signer stands as that person, and a key it admits is that
        /// person's. Skipped when absent, so any other link keeps its
        /// bytes, its id and its meaning.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        device_of: Option<String>,
    },
    /// A member says: these files of my folder `folder`, synced both ways,
    /// changed, each naming the entry it replaces (sy-1, [`synced`]).
    /// Carries no [`Payload`] and never reaches an app reducer.
    Synced {
        folder: String,
        changes: Vec<Change>,
    },
    /// A computer says: reach me through this relay, or, with none, through
    /// no relay the record names (relay-in-record; DESIGN §6). Signed by
    /// the key it speaks for, so nobody says where another is reached; its
    /// newest counting one wins, and the register carries it beside the
    /// key ([`Membership::relay`]). As written: the fold reads it, dotless
    /// ([`relay::parse`]). Carries no [`Payload`].
    Relay {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
}

/// What a redeeming [`RailAct::Admit`] carries: the [`RailAct::Invite`] it
/// redeems, and the admitted key's signature over the namespace, that
/// `Invite`'s id, the person and the key ([`sig::redemption_message`]). The
/// issuer's signature on the `Admit` says the issuer let this key in; this
/// one says the key asked to join, as this person, through this link — so
/// no issuer can redeem its own link for a key that never opened it.
///
/// It also says what holds the new key ([`Device`]; u4-1). The co-signature
/// does not cover that: the issuer's node writes the device the request
/// names only when the request's dialler is the new key, and the issuer's
/// signature on the `Admit` covers it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Redemption {
    /// The id the `Invite`'s content derives ([`derive_id`]).
    pub id: OpId,
    /// Hex Ed25519 signature by the admitted key.
    pub sig: String,
    /// What holds the new key. Absent on every line before
    /// [`BROWSER_LINE_VERSION`], which admitted only computers, so absent
    /// reads as [`Device::Computer`]; and skipped for a computer, so a
    /// computer's redemption keeps the bytes, id and signature version 3
    /// gave it (`fixtures/redemption_before_browsers.jsonl`).
    #[serde(
        default = "Device::before_browsers",
        skip_serializing_if = "Device::is_computer"
    )]
    pub device: Device,
}

/// What holds a standing key (u4-1; DESIGN §3 "Joining", call 2): the
/// register answers it for each standing key ([`Membership::device`]), and
/// whether the key is dialled is [`Device::dialled`]'s alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Device {
    /// A computer: it holds a copy of the record and is dialled.
    Computer,
    /// A browser: it dials and is never dialled. Its endpoint lives only
    /// while its page runs, so a round dialling it would wait out a dial
    /// timeout each time.
    Browser,
}

impl Device {
    /// Whether the round, the live fan-out and the list of what members
    /// lend dial a key this device holds.
    pub const fn dialled(self) -> bool {
        match self {
            Device::Computer => true,
            Device::Browser => false,
        }
    }

    /// The device of a key admitted before a redemption could name one:
    /// a computer, the only device that could join.
    const fn before_browsers() -> Self {
        Device::Computer
    }

    const fn is_computer(&self) -> bool {
        matches!(self, Device::Computer)
    }

    /// The word a request signs (`svrngs_node::redeem`): serde's spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Device::Computer => "computer",
            Device::Browser => "browser",
        }
    }
}

impl Redemption {
    /// `joining`, held by `device`, consents to join `namespace` as
    /// `person` under the `Invite` whose id is `invite`.
    pub fn sign(
        joining: &SigningKey,
        namespace: &str,
        invite: &OpId,
        person: &Person,
        device: Device,
    ) -> Self {
        let msg = sig::redemption_message(namespace, invite, person.as_str(), &actor_of(joining));
        Self {
            id: invite.clone(),
            sig: sig::sign_message(joining, &msg),
            device,
        }
    }

    /// Whether `sig` is `key`'s signature over this redemption of `person`
    /// in `namespace`. `false` on anything malformed.
    pub(crate) fn verifies(&self, namespace: &str, person: &Person, key: &str) -> bool {
        let msg = sig::redemption_message(namespace, &self.id, person.as_str(), key);
        sig::verify_hex(key, &msg, &self.sig)
    }
}

impl RailAct {
    /// Read an act from an app's request body, with refusals a person can
    /// act on.
    ///
    /// The rail's own refusals ([`PayloadError`]) are already sentences
    /// naming the fix, but they reach here wrapped by serde, which renders a
    /// `Deserialize` failure with its own prose and a trailing position that
    /// means nothing for a value that never came from a file. This unwraps
    /// that: the route returns what the type said, not what serde said about
    /// what the type said.
    ///
    /// One entry point for the same reason the gap renderer is one function
    /// (ARCH §10.6) — the 422 body an app shows a housemate must be the same
    /// words the terminal shows.
    pub fn from_json(value: serde_json::Value) -> Result<Self, RailError> {
        serde_json::from_value(value).map_err(|e| {
            let raw = e.to_string();
            // serde appends " at line 0 column 0" to a `from_value` failure,
            // which is a position in a document that does not exist.
            let cleaned = raw
                .split(" at line ")
                .next()
                .unwrap_or(&raw)
                .trim()
                .to_string();
            RailError::Rejected(if cleaned.is_empty() { raw } else { cleaned })
        })
    }
}

/// The journal line's body: the act, plus the two fields that make it
/// attributable and countable.
///
/// `seq` is the keystone of the whole design. Per-actor sequence numbers are
/// the only thing here that can tell "nothing happened" apart from "it never
/// reached me", and they give a sync digest of `{actor → max_seq}` — about
/// 600 bytes regardless of how long the log is — instead of shipping the
/// whole namespace to every peer.
///
/// They also fix a collision the envelope documents as by-design:
/// [`Op::new`] hashes `(prefix, ts, actor, body)`, so two byte-identical acts
/// in the same second by the same actor share an id. Harmless for governance;
/// for a money app it silently merges two real coffees into one. `seq` is in
/// [`act_id`]'s input, so the ids differ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedOp {
    /// This actor's own counter within this namespace, starting at 0 and
    /// contiguous. A hole means an op has not reached us.
    pub seq: u64,
    /// The id of this author's act at `seq − 1`; none at seq 0. Inside the
    /// id, so inside the signature: an act names its parent the way a git
    /// commit does, and admission names a gap for one whose parent is not
    /// its author's act at `seq − 1` (svrngs, U2; DESIGN §6). Skipped when
    /// absent, so a first act carries no `prev` on its line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev: Option<OpId>,
    /// Hex Ed25519 signature over [`sig::ring_op_message`] — the act's id.
    /// Not itself signed (that would be circular) and not in the id, so one
    /// act carrying two valid signatures is one act.
    pub sig: String,
    #[serde(flatten)]
    pub act: RailAct,
    /// Whose words this act was, when the key that signed it is not theirs —
    /// a door that authenticated somebody the ring holds no key for. A name
    /// and nothing else: not a key, not a role, not a scope.
    ///
    /// **A member's signature over somebody else's name means "this door says
    /// so", and no more.** The rail never looks the name up in the
    /// [`Roster`], never refuses on it and never lets it stand in for
    /// authorship — `actor` is still the only field a writer cannot forge for
    /// someone else. Carrying is not interpreting; what a name is allowed to
    /// mean is the door's question, decided where the door authenticates.
    ///
    /// Declared LAST and skipped when absent, so every op written before this
    /// field existed serialises to the same bytes, derives the same id, and
    /// verifies under the same signature — which
    /// `rail_ops_written_before_on_behalf_of_still_verify` holds to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<String>,
    /// The author's view of the ring at the moment of writing: every actor's
    /// [`View`] over what this writer held (the sync digest's entries),
    /// inside the signature. Every commit stamped one until arch-view; none
    /// does now ([`commit`] signs no view), since nothing but the signature
    /// over it ever read it: a cut names what its remover saw of the removed
    /// key in its own body ([`RailAct::Remove`]'s `through`), and a fork is
    /// named by the chain (`SequenceFork`).
    ///
    /// Kept so every act a computer already holds with one is read, derives
    /// the same id and verifies (`acts_signed_with_a_view_still_verify`).
    /// LAST and skipped when absent, so an act without one signs the bytes
    /// it would have before the field existed (the `on_behalf_of` comment
    /// above).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<crate::Digest>,
}

/// The line format version the membership act cohort ships at
/// (`RailAct::Admit` / [`RailAct::Remove`]). The original three kinds stay at
/// `Journaled::VERSION` (1) — see [`SignedOp::line_version`].
pub const MEMBERSHIP_LINE_VERSION: u32 = 2;

/// The line format version the link ships at (svrngs, U2): an
/// [`RailAct::Invite`], and an [`RailAct::Admit`] that redeems one. A build
/// at [`MEMBERSHIP_LINE_VERSION`] would read a redeeming `Admit` as a plain
/// one, drop the field it does not know, and derive another id; at this
/// version it names the line `NewerVersionLine` instead.
pub const LINK_LINE_VERSION: u32 = 3;

/// The line format version a browser's redemption ships at (u4-1): an
/// [`RailAct::Admit`] whose [`Redemption`] names [`Device::Browser`]. A
/// build at [`LINK_LINE_VERSION`] would drop the field, read the key as a
/// computer's and derive another id; at this version it names the line
/// `NewerVersionLine` instead.
pub const BROWSER_LINE_VERSION: u32 = 4;

/// The line format version a synced folder's change ships at (sy-1). 5
/// was a one-way shared folder's (gf-1), dropped by syp-1 before any record
/// outside a run's scratch held one: no build reads it.
pub const SYNCED_LINE_VERSION: u32 = 6;

/// The line format version removal ships at (U7): a [`RailAct::Remove`]
/// naming the acts it covers by id, and a [`RailAct::HandOn`]. A build at
/// [`SYNCED_LINE_VERSION`] read `Remove` by a seq it no longer carries; at
/// this version it names the line `NewerVersionLine` instead.
pub const REMOVAL_LINE_VERSION: u32 = 7;

/// The line format version a device link ships at (bash-devices): an
/// [`RailAct::Invite`] naming the person it adds a device of. A build at
/// [`REMOVAL_LINE_VERSION`] would drop the field, read it as any link and
/// derive another id; at this version it names the line `NewerVersionLine`
/// instead, so it holds no device link and counts no `Admit` under one.
pub const DEVICE_LINE_VERSION: u32 = 8;

/// The line format version a computer's relay ships at (relay-in-record):
/// a [`RailAct::Relay`]. A build at [`DEVICE_LINE_VERSION`] names it
/// `NewerVersionLine`, so it dials that computer as it did before.
pub const RELAY_LINE_VERSION: u32 = 9;

impl Journaled for SignedOp {
    const FILE: &'static str = "ring_oplog.jsonl";
    const ID_PREFIX: &'static str = "ring";
    const LABEL: &'static str = "ring_rail";
    /// This build reads the membership cohort's, the link's, a browser's
    /// redemption's, a folder's, removal's, a device link's and a relay's
    /// lines too (see [`Self::line_version`]).
    const UNDERSTANDS: u32 = RELAY_LINE_VERSION;
    /// The membership acts — [`RailAct::Admit`] and [`RailAct::Remove`] —
    /// ship at the bumped line version and the original three keep the bytes
    /// they always had, so an un-upgraded node names exactly the acts it
    /// cannot read `NewerVersionLine` (leg 5 of `ra-membership-is-order-free`)
    /// and a ring with no membership act folds byte-identically to the build
    /// before.
    fn line_version(&self) -> u32 {
        match self.act {
            RailAct::Admit {
                invite:
                    Some(Redemption {
                        device: Device::Browser,
                        ..
                    }),
                ..
            } => BROWSER_LINE_VERSION,
            RailAct::Invite {
                device_of: Some(_), ..
            } => DEVICE_LINE_VERSION,
            RailAct::Invite { .. }
            | RailAct::Admit {
                invite: Some(_), ..
            } => LINK_LINE_VERSION,
            RailAct::Admit { .. } => MEMBERSHIP_LINE_VERSION,
            RailAct::Remove { .. } | RailAct::HandOn { .. } => REMOVAL_LINE_VERSION,
            RailAct::Synced { .. } => SYNCED_LINE_VERSION,
            RailAct::Relay { .. } => RELAY_LINE_VERSION,
            _ => Self::VERSION,
        }
    }
}

// ── Errors ───────────────────────────────────────────────────

/// Named `RailError` and not `LedgerError` because `sovereign-pipeline`
/// already owns that name for a cost ledger — a different concept that
/// shared a word — and because this layer stopped being a ledger when the
/// payload became opaque.
#[derive(Debug, thiserror::Error)]
pub enum RailError {
    /// A namespace is a directory name. Anything that is not plainly one is
    /// refused here rather than sanitised downstream, so no caller can reach
    /// outside its own namespace by naming a path (ARCH §7.1).
    #[error("'{0}' is not a ring namespace — use 1..=64 characters from a-z, 0-9, '-' and '_'")]
    BadNamespace(String),
    /// The wall clock is unreadable. Refusing to sign is the only honest
    /// answer: the previous `unwrap_or(0)` quietly signed lines stamped 0,
    /// on the signature path itself (ROOT_CAUSE_FIXES C4).
    #[error("the wall clock is unreadable — refusing to sign a line stamped 0")]
    Clock,
    #[error("ring rail io: {0}")]
    Io(String),
    /// The keeper's cuts in this group do not settle to one answer (#124),
    /// so who stands in it is not judged: never read as anyone standing.
    #[error("who is in `{0}` cannot be judged: its removals do not come to one answer")]
    Unsettled(String),
    /// The door refused to author this act. The string is a sentence, because
    /// it is the 422 body an app shows a person.
    #[error("refused: {0}")]
    Rejected(String),
    /// Authoring under a key that holds no standing in this namespace
    /// (svrngs: the register's standing — the seed and its `Admit` and
    /// `Remove` acts — where upstream asked whether the roster claimed it).
    ///
    /// Typed rather than one more [`RailError::Rejected`] sentence so a
    /// renderer recognises it by the variant: matching on the prose would be
    /// the string `match` §2.1 forbids — and the prose is a 422 body, so it
    /// is allowed to change. (Upstream's reason was that the fix depended on
    /// where the roster came from, `RingRail::roster_origin` in
    /// `commonwealth-rail`; svrngs derives no roster.)
    #[error(
        "this node signs as {}… and that key holds no standing in `{namespace}` — \
         neither its seed nor an `Admit` of it names the key — so every op it \
         writes would be unreadable to the ring; a member must admit it first",
        actor_prefix(.actor)
    )]
    NotInRoster { actor: String, namespace: String },
}

/// Enough of an actor key to recognise, short enough to read in a sentence.
fn actor_prefix(actor: &str) -> String {
    actor.chars().take(12).collect()
}

// ── Rendering an op id to a person ───────────────────────────
//
// An op id is 40-odd characters, and every surface that shows one to a
// person cuts it the same way: a refusal's sentence must not spell an id
// differently from the log line printed above it (ARCH §10.6). (svrngs:
// `short_stamp`, its date-shaped twin, went with the roster warrant that
// alone used it, core-0.)

/// Enough of an op id to recognise, short enough to read in a sentence.
pub fn short_id(id: &str) -> String {
    if id.chars().count() > 12 {
        format!("{}…", id.chars().take(12).collect::<String>())
    } else {
        id.to_string()
    }
}

/// How admission asks whether a key signed an act's id. (It was the other
/// half of a `RingSigner` trait, which went in arch-core: a signer is a
/// [`SigningKey`], and this seam stays for the tests that pin what admission
/// does with a refusal, `tests_sealing.rs`.)
///
/// **Refusal is the only safe answer.** An implementation that cannot judge a
/// signature returns `false`; [`admit`] turns that into a
/// [`RailGap::BadSignature`] and never into an admitted op. There is no
/// "cannot tell" that means yes (ARCH §18.3).
///
/// **A verifier is not a roster.** It answers "did this key sign these bytes",
/// and nothing else. Membership stays [`Roster`]'s, so installing a permissive
/// verifier still cannot put a stranger's act into the ring.
pub trait RingVerifier: Send + Sync {
    /// Which scheme this is, named in the admission trace. A verifier that
    /// admits everything is a catastrophe no log can show unless the log says
    /// which verifier ran (ARCH §9.1).
    fn name(&self) -> &'static str;
    /// Whether `sig_hex` is `actor`'s signature over [`sig::ring_op_message`]
    /// of `id`, the id the caller derived from the act's content
    /// ([`derive_id`]). `false` on anything it cannot judge.
    fn verify(&self, actor: &str, id: &OpId, sig_hex: &str) -> bool;
}

/// The shipped verifier: Ed25519 over the exact bytes [`sign_act`] signs. The one production answer, and the reason
/// `sig::verify_ring_op` is no longer reachable from outside this crate —
/// leaving it public would leave the hardcode next to the seam, which is how
/// the asymmetry comes back (ARCH §7).
pub struct Ed25519Verifier;

impl RingVerifier for Ed25519Verifier {
    fn name(&self) -> &'static str {
        "ed25519"
    }
    fn verify(&self, actor: &str, id: &OpId, sig_hex: &str) -> bool {
        sig::verify_ring_op(actor, id, sig_hex)
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_sealing;

/// The rail's ONE set of op fixtures.
///
/// Public behind `test-support` because the journal tests live in
/// `commonwealth-rail` and the fold tests live here, and both have to be
/// talking about the same signed op or neither proves anything. Two copies of
/// "what does a signed op look like" would be two answers to the question the
/// signature exists to settle (ARCH §10.6).
#[cfg(any(test, feature = "test-support"))]
// Fixture code, and it compiles as LIB code under the feature rather than
// under `cfg(test)` — so clippy's allow-unwrap-in-tests exemption does not
// reach it and the panic ratchet would read a builder as production risk.
#[allow(clippy::unwrap_used, clippy::expect_used)]
pub mod tests_support;
