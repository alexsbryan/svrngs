// SPDX-License-Identifier: AGPL-3.0-or-later
//! The runtime's queries, through its public surface only.

use std::sync::Arc;

use crate::*;
use svrngs_core::tests_support::*;

fn rail(dir: &std::path::Path) -> RingRail {
    let rail = RingRail::new(dir, Arc::new(key(1)));
    rail.set_seed(NS, &ring()).unwrap();
    rail
}

#[test]
fn an_ask_finds_this_computers_newest_invite_naming_the_link() {
    let dir = tempfile::tempdir().unwrap();
    let rail = rail(dir.path());
    let link = actor_of(&key(9));
    let mint = |until| MintInvite {
        key: link.clone(),
        until,
        device_of: None,
    };
    rail.commit(NS, &mint(10_000_000_000)).unwrap().unwrap();
    let newest = rail.commit(NS, &mint(20_000_000_000)).unwrap().unwrap();
    let theirs = signed(
        &key(2),
        1,
        0,
        RailAct::Invite {
            key: link.clone(),
            until: 30_000_000_000,
            device_of: None,
        },
    );
    rail.ingest_all(NS, &[theirs]).unwrap();

    let found = rail.invite_for(&link).unwrap();
    assert_eq!(found, Some((NS.to_string(), newest.lines()[0].id.clone(), None)));
    assert_eq!(rail.invite_for(&actor_of(&key(8))).unwrap(), None);
}
