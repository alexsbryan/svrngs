// SPDX-License-Identifier: AGPL-3.0-or-later
//! Projections: state folded here, handed out by `project`.

use crate::tests_support::*;
use crate::*;

/// An issuer who invites one key twice: the second is the newest, whatever
/// order the lines arrived in, and another issuer's invite is not theirs.
#[test]
fn the_newest_invite_is_the_last_in_the_records_order() {
    let link = actor_of(&key(9));
    let invite = || RailAct::Invite {
        key: link.clone(),
        until: 1_000,
    };
    let first = signed(&key(1), 100, 0, invite());
    let second = after(&key(1), &first, 101, invite());
    let theirs = signed(&key(2), 102, 0, invite());
    let invites = admitted(&[theirs, second.clone(), first]).project::<Invites>();
    assert_eq!(invites.newest(&actor_of(&key(1)), &link), Some(&second.id));
    assert_eq!(invites.newest(&actor_of(&key(1)), &actor_of(&key(8))), None);
}
