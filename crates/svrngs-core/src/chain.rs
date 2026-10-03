//! The chain (DESIGN C1, "Ids and the chain"): an act on the wire, its id
//! `H(domain ‖ actor ‖ seq ‖ prev ‖ ts ‖ H(body))`, the signature over the
//! id, and [`VerifiedAct::parse`], the one way bytes become an act.
//!
//! H is BLAKE3. Every field but the body is fixed-width, so no boundary can
//! bleed into the next; the body enters as its own hash over canonical bytes
//! ([`Payload`]).

use std::fmt;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::payload::{Payload, PayloadError};

const ACT_DOMAIN: &[u8] = b"svrngs/act/v1\0";
const GROUP_DOMAIN: &[u8] = b"svrngs/group/v1\0";
const ADMIT_DOMAIN: &[u8] = b"svrngs/admit/v1\0";

/// Fixed-size bytes spelled as lowercase hex on the wire.
macro_rules! hex_bytes {
    ($(#[$m:meta])* $name:ident, $n:literal) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name([u8; $n]);

        impl $name {
            pub fn from_bytes(b: [u8; $n]) -> Self {
                Self(b)
            }
            pub fn as_bytes(&self) -> &[u8; $n] {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = String;
            fn try_from(s: String) -> Result<Self, String> {
                let mut b = [0u8; $n];
                hex::decode_to_slice(&s, &mut b)
                    .map_err(|e| format!("{} must be {} hex characters: {e}", stringify!($name), 2 * $n))?;
                Ok(Self(b))
            }
        }
        impl From<$name> for String {
            fn from(v: $name) -> String {
                hex::encode(v.0)
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&hex::encode(self.0))
            }
        }
        impl fmt::Debug for $name {
            // 12 hex characters: enough to correlate in a trace.
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), &hex::encode(self.0)[..12])
            }
        }
    };
}

hex_bytes!(
    /// An act's id, derived from the act; never chosen.
    ActId, 32
);
hex_bytes!(
    /// A group's id: the hash of its genesis.
    GroupId, 32
);
hex_bytes!(
    /// The genesis nonce, so two groups of one name and starter differ.
    Nonce, 32
);
hex_bytes!(
    /// An Ed25519 signature.
    Sig, 64
);

/// A signing key's public half: who wrote an act. Only a point on the curve
/// parses.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Actor([u8; 32]);

impl Actor {
    pub fn from_key(key: &VerifyingKey) -> Self {
        Self(key.to_bytes())
    }
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    fn verify(&self, msg: &[u8], sig: &Sig) -> bool {
        VerifyingKey::from_bytes(&self.0).is_ok_and(|k| k.verify(msg, &Signature::from_bytes(sig.as_bytes())).is_ok())
    }
}
impl TryFrom<String> for Actor {
    type Error = String;
    fn try_from(s: String) -> Result<Self, String> {
        let mut b = [0u8; 32];
        hex::decode_to_slice(&s, &mut b).map_err(|e| format!("an actor is 64 hex characters: {e}"))?;
        VerifyingKey::from_bytes(&b).map_err(|e| format!("an actor is an Ed25519 public key: {e}"))?;
        Ok(Self(b))
    }
}
impl From<Actor> for String {
    fn from(v: Actor) -> String {
        hex::encode(v.0)
    }
}
impl fmt::Debug for Actor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Actor({})", &hex::encode(self.0)[..12])
    }
}

/// A short non-empty text, parsed once at the boundary.
macro_rules! text {
    ($(#[$m:meta])* $name:ident, $max:literal, $ok:expr) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = String;
            fn try_from(s: String) -> Result<Self, String> {
                let ok: fn(char) -> bool = $ok;
                if s.is_empty() || s.len() > $max || !s.chars().all(ok) {
                    return Err(format!("{} must be 1 to {} characters of {}, not {s:?}",
                        stringify!($name), $max, stringify!($ok)));
                }
                Ok(Self(s))
            }
        }
        impl TryFrom<&str> for $name {
            type Error = String;
            fn try_from(s: &str) -> Result<Self, String> {
                Self::try_from(s.to_string())
            }
        }
        impl From<$name> for String {
            fn from(v: $name) -> String {
                v.0
            }
        }
    };
}

text!(
    /// What a person calls the group, or a key.
    Name, 100, |c| !c.is_control()
);
text!(
    /// An app's lineage name: the acts it reads and corrects are its own.
    AppId, 64, |c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'
);
text!(
    /// A chat speaker by the platform's user id, e.g. `discord:80351110224678912`.
    Speaker, 128, |c| c.is_ascii_graphic()
);
text!(
    /// The chat platform's id for one interaction (a button press, a command).
    /// A retry of the same interaction carries the same id.
    Interaction, 128, |c| c.is_ascii_graphic()
);

/// What a group's preimage is (DESIGN C1, "Genesis and continuation"): the
/// starter's key, a nonce, the name. Not signed: a genesis that does not hash
/// to the group id is simply another group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Genesis {
    pub key: Actor,
    pub nonce: Nonce,
    pub name: Name,
}

impl Genesis {
    pub fn group_id(&self) -> GroupId {
        let mut h = blake3::Hasher::new();
        h.update(GROUP_DOMAIN);
        h.update(self.key.as_bytes());
        h.update(self.nonce.as_bytes());
        h.update(self.name.as_str().as_bytes());
        GroupId(*h.finalize().as_bytes())
    }
}

/// The role an `Admit` grants. The starter's key is a Person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Person,
    Bridge,
}

/// The chat person an act speaks for, and the interaction it answers.
/// Honoured only from a Bridge-role key (DESIGN C2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnBehalfOf {
    pub speaker: Speaker,
    pub interaction: Interaction,
}

/// Binds `key` with `role`. `cosig` is the admitted key's signature over
/// [`admit_message`], so no key is granted a role it did not ask for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admit {
    pub key: Actor,
    pub role: Role,
    pub name: Name,
    pub cosig: Sig,
}

/// An app said something.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub app: AppId,
    pub op: Payload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<OnBehalfOf>,
}

/// What the chat platform reported about a speaker (DESIGN C5). Closed: the
/// guest book's rule over these is [`crate::guests`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuestEvent {
    /// They joined the chat: the one event that lifts an earlier removal,
    /// because a ban is lifted only by a person in the chat.
    Joined,
    /// They acted or were mentioned.
    Seen,
    Left,
    Banned,
}

impl GuestEvent {
    pub const fn removes(self) -> bool {
        matches!(self, GuestEvent::Left | GuestEvent::Banned)
    }
}

/// A line of the guest book, keyed by the platform's user id (DESIGN D4).
/// Honoured only from a Bridge-role key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guest {
    pub speaker: Speaker,
    pub name: Name,
    pub event: GuestEvent,
}

/// An earlier act of the same app was wrong: it is voided, never erased, and
/// optionally re-stated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correct {
    pub app: AppId,
    pub corrects: ActId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<Payload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_behalf_of: Option<OnBehalfOf>,
}

/// The kinds of act. Closed: a kind this version does not know is refused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Body {
    Admit(Admit),
    Record(Record),
    Correct(Correct),
    Guest(Guest),
}

impl Body {
    pub fn on_behalf_of(&self) -> Option<&OnBehalfOf> {
        match self {
            Body::Record(r) => r.on_behalf_of.as_ref(),
            Body::Correct(c) => c.on_behalf_of.as_ref(),
            Body::Admit(_) | Body::Guest(_) => None,
        }
    }
}

/// What the admitted key co-signs: who admits it, as what, by what name.
pub fn admit_message(admitter: &Actor, key: &Actor, role: Role, name: &Name) -> [u8; 32] {
    let mut h = blake3::Hasher::new();
    h.update(ADMIT_DOMAIN);
    h.update(admitter.as_bytes());
    h.update(key.as_bytes());
    h.update(&[role as u8]);
    h.update(name.as_str().as_bytes());
    *h.finalize().as_bytes()
}

fn derive_id(actor: &Actor, seq: u64, prev: Option<&ActId>, ts: u64, body: &Payload) -> ActId {
    let mut h = blake3::Hasher::new();
    h.update(ACT_DOMAIN);
    h.update(actor.as_bytes());
    h.update(&seq.to_be_bytes());
    match prev {
        None => h.update(&[0]),
        Some(p) => h.update(&[1]).update(p.as_bytes()),
    };
    h.update(&ts.to_be_bytes());
    h.update(blake3::hash(&body.to_bytes()).as_bytes());
    ActId(*h.finalize().as_bytes())
}

/// Why bytes are not an act. Each is refused, never offered again (C1 rule 1).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("not an act line: {0}")]
    Malformed(String),
    #[error("not an act this version knows: {0}")]
    Body(String),
    #[error("the line names id {claimed} but the act derives {derived}")]
    IdMismatch { claimed: ActId, derived: ActId },
    #[error("the signature does not verify over the id under the actor's key")]
    BadSignature,
    #[error("the admitted key did not co-sign its Admit")]
    BadCosignature,
    #[error("seq {seq} with prev {prev}: a first act is seq 1 with no prev, any other names one")]
    SeqPrev { seq: u64, prev: bool },
}

/// One journal line, as written.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Line {
    id: ActId,
    actor: Actor,
    seq: u64,
    prev: Option<ActId>,
    ts: u64,
    body: Payload,
    sig: Sig,
}

/// An act whose id derives from it and whose signature verifies over that id.
/// Only [`VerifiedAct::parse`] and [`Draft::seal`] make one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedAct {
    id: ActId,
    actor: Actor,
    seq: u64,
    prev: Option<ActId>,
    ts: u64,
    raw_body: Payload,
    body: Body,
    sig: Sig,
}

impl VerifiedAct {
    /// Rule 1 of ingest: the id derives from the act and the signature
    /// verifies over it; an `Admit` also carries its admitted key's co-signature.
    pub fn parse(line: &str) -> Result<Self, ParseError> {
        let l: Line = serde_json::from_str(line).map_err(|e| ParseError::Malformed(e.to_string()))?;
        Self::check(Some(l.id), l.actor, l.seq, l.prev, l.ts, l.body, l.sig)
    }

    fn check(
        claimed: Option<ActId>,
        actor: Actor,
        seq: u64,
        prev: Option<ActId>,
        ts: u64,
        raw_body: Payload,
        sig: Sig,
    ) -> Result<Self, ParseError> {
        if (seq == 1) != prev.is_none() || seq == 0 {
            return Err(ParseError::SeqPrev {
                seq,
                prev: prev.is_some(),
            });
        }
        let body: Body =
            serde_json::from_value(raw_body.as_value().clone()).map_err(|e| ParseError::Body(e.to_string()))?;
        let id = derive_id(&actor, seq, prev.as_ref(), ts, &raw_body);
        if let Some(claimed) = claimed.filter(|c| *c != id) {
            return Err(ParseError::IdMismatch { claimed, derived: id });
        }
        if !actor.verify(id.as_bytes(), &sig) {
            return Err(ParseError::BadSignature);
        }
        if let Body::Admit(a) = &body
            && !a.key.verify(&admit_message(&actor, &a.key, a.role, &a.name), &a.cosig)
        {
            return Err(ParseError::BadCosignature);
        }
        Ok(Self {
            id,
            actor,
            seq,
            prev,
            ts,
            raw_body,
            body,
            sig,
        })
    }

    /// The journal line, without its newline.
    pub fn to_line(&self) -> String {
        let l = Line {
            id: self.id,
            actor: self.actor,
            seq: self.seq,
            prev: self.prev,
            ts: self.ts,
            body: self.raw_body.clone(),
            sig: self.sig,
        };
        serde_json::to_string(&l).unwrap_or_default()
    }

    pub fn id(&self) -> &ActId {
        &self.id
    }
    pub fn actor(&self) -> &Actor {
        &self.actor
    }
    pub fn seq(&self) -> u64 {
        self.seq
    }
    pub fn prev(&self) -> Option<&ActId> {
        self.prev.as_ref()
    }
    pub fn ts(&self) -> u64 {
        self.ts
    }
    pub fn body(&self) -> &Body {
        &self.body
    }
}

/// An act stamped but not yet signed. The caller signs [`Draft::id`] with the
/// actor's key — this crate holds no key — and [`Draft::seal`]s it.
#[derive(Debug, Clone)]
pub struct Draft {
    actor: Actor,
    seq: u64,
    prev: Option<ActId>,
    ts: u64,
    raw_body: Payload,
}

impl Draft {
    pub(crate) fn new(actor: Actor, seq: u64, prev: Option<ActId>, ts: u64, body: &Body) -> Result<Self, PayloadError> {
        let value = serde_json::to_value(body).map_err(|_| PayloadError::NotAnObject)?;
        Ok(Self {
            actor,
            seq,
            prev,
            ts,
            raw_body: Payload::new(value)?,
        })
    }

    /// The bytes to sign.
    pub fn id(&self) -> ActId {
        derive_id(&self.actor, self.seq, self.prev.as_ref(), self.ts, &self.raw_body)
    }

    pub fn seal(self, sig: Sig) -> Result<VerifiedAct, ParseError> {
        VerifiedAct::check(None, self.actor, self.seq, self.prev, self.ts, self.raw_body, sig)
    }
}
