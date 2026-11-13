// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a line's signer may do as a person, in the log (capability-signed;
//! DESIGN §1, "An entry's author is its signer"; demo/runs/authorship/
//! 2026-10-24/DESIGN.md §4). New: upstream had one signer per person and no
//! key acting for another.
//!
//! Original bytes and native app-act shaping. The shared authority profile
//! interprets holder scope through Eudoxia; membership independently decides
//! whether the original issuer stands at a line's place in the group.

use alloc::format;

/// Why a holder may not: the community profile's (EUDOXIA_JUDGE §3, R1).
pub use crate::authority::Why;

/// The content a per-entry grant binds, before that grant is attached.
pub fn bare_id(line: &crate::Op<crate::SignedOp>, group: &str) -> crate::OpId {
    let mut bare = line.clone();
    bare.kind.grant = None;
    crate::derive_id(&bare, group)
}

impl crate::Op<crate::SignedOp> {
    /// The key whose standing this line counts under: its signer's, or for
    /// a holder, its pass's signer's. Read only of a line admission kept, so
    /// only where the shared authority resource committed its original line.
    pub fn stands_as(&self) -> &str {
        self.kind.grant.as_ref().map_or(&self.actor, |p| &p.person)
    }
}

/// The act an app `kind`'s page asks for, its body `{op}` or `{corrects,
/// op}`, `op` null to void: `op`'s kind put under `<kind>/`. None for a
/// body naming no `op`, or an `op` with no kind. The door and a phone's
/// worker shape it here alike, so both say one act (DESIGN §3).
pub fn app_act(kind: &str, body: &[u8]) -> Option<serde_json::Value> {
    let mut body: serde_json::Map<_, _> = serde_json::from_slice(body).ok()?;
    let mut op = body.remove("op")?;
    if let Some(fields) = op.as_object_mut() {
        let own = fields.get("kind")?.as_str()?;
        fields.insert("kind".into(), format!("{kind}/{own}").into());
    }
    Some(match body.remove("corrects") {
        None => serde_json::json!({ "op": "record", "payload": op }),
        Some(id) => serde_json::json!({ "op": "correct", "corrects": id, "replacement": op }),
    })
}
