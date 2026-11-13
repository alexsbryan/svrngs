//! The pages' own tests (ralph/next/page/CHARTER.md, "What is settled" 6):
//! what the pages do that the page queue made them do, each read again,
//! answered beside its button, or refused by name, in scripts/check.sh's
//! "pages" step. One module a feature. Each was watched red on the pages
//! before the queue, then green.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
#[path = "../../../svrngs-node/tests/browser/drive.rs"]
#[allow(dead_code)]
mod drive;

// page-open: the phone page opens into its group; the home reads its groups.
mod open;
// page-acts: the record's acts, its changes and uses, and the ask.
mod acts;
// page-home-rooms, page-join, page-shelf, page-index and page-members have
// no test here: each of their claims is a law of either page with no
// browser (web/tests/pages/home.test.mjs, again.test.mjs).

/// `who`, holding the key drawn from `seed`, let into group `id` on `node`
/// by a link `node` mints, consenting as the group's seed asks: a group
/// `svrngs_node::group::draw` makes is sources-v1, and counts no `Admit`
/// but one under an invitation.
fn admit(node: &svrngs_node::node::Node, id: &str, who: &str, seed: u8) {
    use svrngs_node::{actor_of, Device, SigningKey};
    let rail = node.rail();
    let groups = svrngs_node::group::groups(rail.root());
    let _policy = groups
        .iter()
        .find(|s| s.id() == id)
        .and_then(|s| s.policy());
    let joining = SigningKey::from_bytes(&[seed; 32]);
    svrngs_core::tests_support::let_in(
        (id, &actor_of(rail.signer())),
        (&joining, who, Device::Computer),
        |mint| {
            rail.commit(id, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(rail.commit(id, redeem).unwrap().unwrap()),
    );
}
