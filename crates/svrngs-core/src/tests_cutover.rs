// SPDX-License-Identifier: AGPL-3.0-or-later
//! The keeper's cutover (capability-signed; demo/runs/authorship/2026-10-24/
//! DESIGN.md §5): an act whose signer names another person than its own
//! (`on_behalf_of`, which no new act writes) counts as that person's only
//! where the keeper's `Signers` covers it. Its bytes cannot tell an honest
//! old act from a forgery, and a timestamp can be backdated, so a hash
//! settles it, as a cut's `through` does.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::tests_support::*;
use crate::*;

/// `k`'s act at `seq` after `prev`, naming `named` as whose it is.
fn by(
    k: &SigningKey,
    prev: Option<&Op<SignedOp>>,
    ts: i64,
    act: RailAct,
    named: Option<&str>,
) -> Op<SignedOp> {
    let (seq, prev) = prev.map_or((0, None), |p| (p.kind.seq + 1, Some(p.id.clone())));
    sign_act(k, NS, ts, seq, prev, act, named, None, None)
}

/// What each counted record says, in the fold's order.
fn says(a: &Admission) -> Vec<String> {
    let what = |o: &AdmittedOp| {
        Some(
            serde_json::to_value(o.payload.as_ref()?).ok()?["what"]
                .as_str()?
                .into(),
        )
    };
    a.ops().iter().filter_map(what).collect()
}

/// **Only carry what you signed, past the cutover.** Alex's computer wrote
/// a record for bo (an old act, as `carried.rs` wrote a phone's), then the
/// keeper, alex, cuts over naming it, then alex's computer writes another
/// for bo and one for alex. Refused: the second for bo, written past the
/// cutover, which counts nowhere and is named `carried_for_another`, a
/// contradiction, so the answer stays complete. Kept: the old one, which
/// the cutover covers, still bo's as before; alex's own, which names its
/// signer's person. Kept too: with no cutover, or one by a key that does
/// not keep the group, each counts as today. Red where no fold reads the
/// cutover; each half its plant.
#[test]
fn past_the_keepers_cutover_an_act_for_another_person_counts_nowhere() {
    let alex = key(1);
    let old = by(&alex, None, 10, record("old"), Some("bo"));
    let cut = by(
        &alex,
        Some(&old),
        20,
        RailAct::Signers {
            through: vec![old.id.clone()],
        },
        None,
    );
    let new = by(&alex, Some(&cut), 30, record("new"), Some("bo"));
    let own = by(&alex, Some(&new), 40, record("own"), Some("alex"));
    let after = admitted(&[old.clone(), cut, new.clone(), own.clone()]);
    assert_eq!(says(&after), ["old", "own"], "{:?}", after.gaps());
    let gaps = serde_json::to_value(after.gaps()).unwrap();
    let named = serde_json::json!([{ "gap": "carried_for_another", "id": new.id, "actor": new.actor, "named": "bo" }]);
    assert_eq!(gaps, named);
    assert!(after.is_complete(), "a contradiction, never an absence");
    let whose = after
        .ops()
        .iter()
        .find(|o| o.id == old.id)
        .and_then(|o| o.on_behalf_of.clone());
    assert_eq!(whose.as_deref(), Some("bo"), "an old act keeps its meaning");

    let none = by(&alex, Some(&old), 30, record("new"), Some("bo"));
    let before = admitted(&[old.clone(), none]);
    assert_eq!(says(&before), ["old", "new"], "with no cutover, as today");

    let cy = key(3);
    let not_kept = by(&cy, None, 20, RailAct::Signers { through: vec![] }, None);
    let new = by(&alex, Some(&old), 30, record("new"), Some("bo"));
    let theirs = admitted(&[old, not_kept, new]);
    assert_eq!(says(&theirs), ["old", "new"], "{:?}", theirs.gaps());
}
