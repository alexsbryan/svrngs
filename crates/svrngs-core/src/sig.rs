// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rail-core/src/sig.rs@e1a43bcd2
//! What a ring op is signed over, and the two functions that agree on it.
//!
//! # Why an op is signed at all
//!
//! `Op::actor` is a string the writer supplies, and ring ops arrive over
//! gossip from every node in the ring. A fold that trusted `actor` would be
//! asserting on a field the subject authored — the same defect as a wrong-slot
//! guard reading the `model` an SSE client echoed back (ARCH §18.1). The
//! signature is the part the subject *cannot* author for someone else, so it
//! is what admission actually checks; `actor` is then not a claim but the
//! public key the check ran under.
//!
//! # The layout, and why it is length-prefixed
//!
//! Copied in shape — not in bytes — from [`commonwealth_core::dial_sig`],
//! which is the tree's existing answer to "sign a struct that rides gossip."
//! Two properties carry over:
//!
//! - **A domain separator**, so a ring-op signature can never be replayed as a
//!   dial-info signature or a join proof, and vice versa.
//! - **Length prefixes on every variable field**, so no field boundary can
//!   bleed into the next. Without them `("ab", "c")` and `("a", "bc")` sign
//!   identical bytes, and a payer could be renamed inside a description.
//!
//! One field is ours rather than inherited: **the namespace**. A signed op
//! lifted out of the tool-lending board and replayed into the expense ledger
//! would otherwise verify perfectly. Binding the namespace into the message
//! makes that cross-namespace replay fail the signature rather than fail a
//! check somewhere downstream that someone might forget to write.
//!
//! # The id, and why the signature is over it
//!
//! svrngs (U2, DESIGN §6 "Ids cover everything signed"): an act's id is the
//! whole BLAKE3 hash of its namespace, actor, seq, previous act, timestamp
//! and body ([`act_id`]), and the signature is over that id
//! ([`ring_op_message`]). Upstream signed the fields and derived the id from
//! the line, signature included, so one act carrying two valid signatures
//! had two ids, and the id was not what anyone signed. Now the id is the one
//! thing signed, and nothing on the line is outside both: a copy that holds
//! an id holds exactly the act its author signed. Admission still re-derives
//! the id from content and ignores the one on the line ([`derive_id`]).

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::hash::ContentHash;
use crate::op::{Journaled, Op, OpId};
use crate::SignedOp;

// ── Every domain svrngs signs or hashes under, in this one home ──────────
//
// svrngs (onair-sign): each was beside the one thing it separates; gathered
// here so a new one is written beside every other, and `sig_tests.rs`
// (`domains_live_here_and_none_begins_another`) holds that none begins
// another and that no core file outside this one spells one.

/// Domain separator for the signature. Distinct from `cwth-dial-info-binding:`
/// and from the join proof-of-possession, so no signature is valid in two
/// contexts; and from upstream's `cwth-ring-op-binding:`, whose message was
/// the fields rather than the id.
pub(crate) const RING_OP_DOMAIN: &[u8] = b"svrngs-act-id-binding:";

/// Domain separator for the id's hash, so an act id is never the hash of
/// anything else the record hashes (a view head, a payload).
const ACT_ID_DOMAIN: &[u8] = b"svrngs/act/v1\0";

/// Domain separator for a redemption's co-signature (svrngs, U2), so the
/// admitted key's consent is never valid as an act's signature, nor one as
/// the other.
pub(crate) const REDEMPTION_DOMAIN: &[u8] = b"svrngs-redemption-binding:";

/// Domain separator for the link key's signature over a request
/// ([`crate::joining`]), so it is never valid as an act's signature or a
/// consent, nor either as it.
pub(crate) const REQUEST_DOMAIN: &[u8] = b"svrngs-redeem-request:";

/// Domain separator for the link key's signature over an ask.
pub(crate) const ASK_DOMAIN: &[u8] = b"svrngs-redeem-ask:";

/// Domain separator for a lend name's hash in its label ([`crate::pass`]).
pub(crate) const LABEL_DOMAIN: &[u8] = b"svrngs-lend-label:";

/// Domain separator for a person's signature over a pass, so it is never
/// valid as an act's, a link's or a consent.
pub(crate) const PASS_DOMAIN: &[u8] = b"svrngs-pass:";

/// Domain separator for what a computer's key signs for an app
/// ([`crate::app`]): bytes the app chose, never an act's, a link's, a
/// pass's or a consent.
pub(crate) const APP_DOMAIN: &[u8] = b"svrngs-app:";

/// Domain separator for a group id's hash ([`crate::seed`]), so an id is
/// never the hash of anything else svrngs hashes (an act id is
/// `svrngs/act/v1`).
pub(crate) const GROUP_ID_DOMAIN: &[u8] = b"svrngs/group/v1\0";

/// Domain separation and definition version for a view's fold
/// ([`crate::view`]). A second definition gets a second label; old heads
/// stay meaningful.
pub(crate) const VIEW_DOMAIN: &[u8] = b"cwth/view/1";

/// The hex form of a node's public key, which is how a ring op names its
/// signer. 64 lowercase hex characters.
///
/// A hex string rather than a `NodePubkey` because it is what lands on the
/// journal line — `Op::actor` is a `String`, and re-parsing it once per
/// verification is cheaper than a second wire representation to keep in sync.
pub fn actor_of(key: &SigningKey) -> String {
    hex::encode(key.verifying_key().to_bytes())
}

/// Append `bytes` to `msg` behind its `u32` big-endian length: the ONE
/// framing every signed message here uses, so no field's end is ambiguous.
pub fn field(msg: &mut Vec<u8>, bytes: &[u8]) {
    msg.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    msg.extend_from_slice(bytes);
}

/// An act's id: the whole BLAKE3 hash of what its author signs.
///
/// `DOMAIN || ns || actor || seq[8 BE] || prev || ts[8 BE] || body`, every
/// variable field preceded by its `u32` big-endian length, and `prev` a tag
/// byte — 0 for none, 1 followed by the 32 bytes of the previous act's id —
/// so no boundary can bleed into the next. Without the length prefixes
/// `("ab", "c")` and `("a", "bc")` hash the same, and a payer could be
/// renamed inside a description.
///
/// The namespace is in it so an act lifted out of one group and replayed
/// into another derives another id, whose signature fails. `prev` is the id
/// of the author's act at `seq − 1`, none at seq 0: each act names its
/// parent, like a git commit (DESIGN §6).
///
/// `body` is [`body_json`](crate::body_json): the act, in declaration order,
/// its payload canonical by construction, so the same act produces the same
/// bytes on every node and build. That is the property the whole scheme
/// rests on, and the payload half of it is why `Payload` is a type rather
/// than a `serde_json::Value`.
pub fn act_id(
    namespace: &str,
    actor: &str,
    seq: u64,
    prev: Option<&OpId>,
    ts_unix: i64,
    body_json: &str,
) -> OpId {
    let mut input = Vec::with_capacity(ACT_ID_DOMAIN.len() + body_json.len() + 160);
    input.extend_from_slice(ACT_ID_DOMAIN);
    field(&mut input, namespace.as_bytes());
    field(&mut input, actor.as_bytes());
    input.extend_from_slice(&seq.to_be_bytes());
    match prev {
        None => input.push(0),
        Some(p) => {
            input.push(1);
            input.extend_from_slice(p.as_bytes());
        }
    }
    input.extend_from_slice(&ts_unix.to_be_bytes());
    field(&mut input, body_json.as_bytes());
    OpId::from_hash(ContentHash::of(&input))
}

/// The id `op`'s content derives in `namespace`, ignoring the `id` its line
/// carries — identity from essence (ARCH §7.5). The ONE reading of an act's
/// identity: admission, membership, the view, sync and the journal's held set
/// all ask this, so a rewritten `id` field cannot make one act pass for
/// another anywhere.
pub fn derive_id(op: &Op<SignedOp>, namespace: &str) -> OpId {
    act_id(
        namespace,
        &op.actor,
        op.kind.seq,
        op.kind.prev.as_ref(),
        op.ts_unix,
        &crate::body_json(
            &op.kind.act,
            op.kind.on_behalf_of.as_deref(),
            op.kind.view.as_ref(),
            op.kind.grant.as_ref(),
        ),
    )
}

/// The bytes signed and verified for one act: `DOMAIN || id`.
pub fn ring_op_message(id: &OpId) -> Vec<u8> {
    [RING_OP_DOMAIN, id.as_bytes().as_slice()].concat()
}

/// The bytes a redeeming `Admit`'s new key signs:
/// `DOMAIN || ns || invite[32] || person || key`, every variable field
/// length-prefixed. The namespace keeps a consent from being replayed into
/// another group, the `Invite`'s id from being moved to another link, and
/// the person from being renamed. `person` is the name's text, so a test
/// can sign a consent to one no `Person` holds, as a hand could.
pub fn redemption_message(namespace: &str, invite: &OpId, person: &str, key: &str) -> Vec<u8> {
    let mut msg = Vec::with_capacity(REDEMPTION_DOMAIN.len() + 160);
    msg.extend_from_slice(REDEMPTION_DOMAIN);
    field(&mut msg, namespace.as_bytes());
    msg.extend_from_slice(invite.as_bytes());
    field(&mut msg, person.as_bytes());
    field(&mut msg, key.as_bytes());
    msg
}

/// Hex Ed25519 signature over `msg`: the ONE place a signature is made and
/// spelled, for an act's id, a redemption, a joiner's ask and a pass alike,
/// as [`verify_hex`] is the one place one is read.
pub fn sign_message(key: &SigningKey, msg: &[u8]) -> String {
    hex::encode(key.sign(msg).to_bytes())
}

/// Sign one act's id. Hex-encoded 64-byte Ed25519 signature, matching how the
/// join proof and the dial-info signature are already carried (serde has no
/// `[u8; 64]` impl).
pub fn sign_ring_op(key: &SigningKey, id: &OpId) -> String {
    sign_message(key, &ring_op_message(id))
}

/// Sign one act with `signer`, as its actor: derive its id, sign the id, and build the
/// line. The ONE place an act is made, for the journal's door, a holder and
/// the fixtures alike, so the bytes signed have one definition (ARCH §10.6).
#[allow(clippy::too_many_arguments)]
pub fn sign_act(
    signer: &SigningKey,
    namespace: &str,
    ts_unix: i64,
    seq: u64,
    prev: Option<OpId>,
    act: crate::RailAct,
    on_behalf_of: Option<&str>,
    view: Option<crate::Digest>,
    grant: Option<crate::pass::Pass>,
) -> Op<SignedOp> {
    let actor = actor_of(signer);
    let body = crate::body_json(&act, on_behalf_of, view.as_ref(), grant.as_ref());
    let id = act_id(namespace, &actor, seq, prev.as_ref(), ts_unix, &body);
    let kind = SignedOp {
        seq,
        prev,
        sig: sign_ring_op(signer, &id),
        act,
        on_behalf_of: on_behalf_of.map(str::to_string),
        view,
        grant,
    };
    Op {
        v: kind.line_version(),
        id,
        ts_unix,
        actor,
        kind,
    }
}

/// Verify one act's signature over `id` under the public key its `actor`
/// names.
///
/// Crate-private since the [`RingVerifier`](crate::RingVerifier) seam landed:
/// [`Ed25519Verifier`](crate::Ed25519Verifier) is its one caller, and a
/// hardcode left reachable beside a seam is the asymmetry coming back.
///
/// `false` on ANY malformed input — bad hex, wrong length, a public key that
/// is not on the curve. The caller turns that into a gap, never into a silent
/// admit (ARCH §4.3: unknown-id handling is explicit and loud).
///
/// Verifying under `actor`'s own key is self-certifying and therefore proves
/// nothing on its own — anyone can mint a keypair. It is the
/// [`Roster`](crate::Roster) that makes it mean something, by saying which
/// keys belong to people in this ring. Both checks are required and admission
/// runs both.
pub(crate) fn verify_ring_op(actor: &str, id: &OpId, sig_hex: &str) -> bool {
    verify_hex(actor, &ring_op_message(id), sig_hex)
}

/// Verify `sig_hex` over `msg` under the hex public key `signer`. The ONE
/// place a hex key and a hex signature are parsed, shared by ring ops,
/// redemptions, asks and passes; `false` on any malformed input, never a
/// panic.
pub fn verify_hex(signer: &str, msg: &[u8], sig_hex: &str) -> bool {
    let Ok(key_bytes) = hex::decode(signer) else {
        return false;
    };
    let Ok(key_arr) = <[u8; 32]>::try_from(key_bytes.as_slice()) else {
        return false;
    };
    let Ok(verifying) = VerifyingKey::from_bytes(&key_arr) else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(sig_hex) else {
        return false;
    };
    let Ok(sig_arr) = <[u8; 64]>::try_from(sig_bytes.as_slice()) else {
        return false;
    };
    verifying
        .verify(msg, &Signature::from_bytes(&sig_arr))
        .is_ok()
}

#[cfg(test)]
#[path = "sig_tests.rs"]
mod tests;
