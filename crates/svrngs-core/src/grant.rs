// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a line's signer may do as a person, in the log (capability-signed;
//! DESIGN §1, "An entry's author is its signer"; demo/runs/authorship/
//! 2026-10-24/DESIGN.md §4). New: upstream had one signer per person and no
//! key acting for another.
//!
//! A key standing as a person is that person's own device, and does
//! anything as them: whether it stands at a line's place is the walk's
//! (`membership`). A holder is what the pass on its line grants: Append and
//! Read of its app's own part of the record, `<kind>/…` of the kind its
//! app's label names, before the pass's `until`, and never Decide. Reach
//! keeps [`Pass::check`] as its one decider; this is the log side alone.
//! Pure: the time is an argument.

use serde::{Deserialize, Serialize};

use crate::pass::{Label, Pass};

/// What a signer asks to do as a person: read or append the app `kind`'s
/// part of the record, or decide (a yes to a version, and every act of the
/// register).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability<'a> {
    Read(&'a str),
    Append(&'a str),
    Decide,
}

/// Why a holder may not.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, thiserror::Error,
)]
#[serde(rename_all = "snake_case")]
pub enum Why {
    #[error("its pass is not signed by the person it names")]
    Unsigned,
    #[error("its pass is another key's")]
    NotTheHolder,
    #[error("its pass had lapsed")]
    Lapsed,
    #[error("an app writes only its own part of the record")]
    OtherKind,
    #[error("an app decides nothing for its person")]
    Decides,
    /// The act it corrects is not held, so whose part it is cannot be told
    /// yet: it may be real data, and counts once its target arrives.
    #[error("what it corrects is not held here yet")]
    Unheld,
}

/// What the pass `.0` on a holder's line grants in the group `.1`, whose
/// id a carried app's label names. A line with no pass is its signer's
/// own device's, and whether it stands is the walk's alone.
#[derive(Debug, Clone, Copy)]
pub struct Grant<'a>(pub &'a Pass, pub &'a str);

impl Grant<'_> {
    /// Whether `holder`, the line's signer, may do `cap` at `at`: passed,
    /// or why not. The pass's signature is the caller's to verify, once per
    /// distinct pass ([`Pass::signed`]).
    pub fn may(&self, holder: &str, cap: Capability<'_>, at: i64) -> Result<(), Why> {
        let Self(pass, group) = *self;
        let verdict = match cap {
            _ if pass.holder != holder => Err(Why::NotTheHolder),
            _ if at >= pass.until => Err(Why::Lapsed),
            Capability::Decide => Err(Why::Decides),
            Capability::Read(kind) | Capability::Append(kind)
                if Label::carried(&pass.app.lender(), kind, group) == pass.app =>
            {
                Ok(())
            }
            Capability::Read(_) | Capability::Append(_) => Err(Why::OtherKind),
        };
        tracing::debug!(target: "rail:grant", holder = %crate::actor_prefix(holder), person = %crate::actor_prefix(&pass.person), app = %pass.app, ?cap, at, ?verdict,
            "grant: a holder reads and appends only its app's own part, before its pass lapses, and decides nothing");
        verdict
    }
}

impl crate::Op<crate::SignedOp> {
    /// The key whose standing this line counts under: its signer's, or for
    /// a holder, its pass's signer's. Read only of a line admission kept, so
    /// only where [`Grant::may`] passed it.
    pub fn stands_as(&self) -> &str {
        self.kind.grant.as_ref().map_or(&self.actor, |p| &p.person)
    }
}
