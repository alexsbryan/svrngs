// SPDX-License-Identifier: AGPL-3.0-or-later
//! Projections: state folded here, handed out by `Admission`'s methods.

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
        device_of: None,
    };
    let first = signed(&key(1), 100, 0, invite());
    let second = after(&key(1), &first, 101, invite());
    let theirs = signed(&key(2), 102, 0, invite());
    let invites = admitted(&[theirs, second.clone(), first]).invites();
    assert_eq!(invites.newest(&actor_of(&key(1)), &link), Some(&second.id));
    assert_eq!(invites.newest(&actor_of(&key(1)), &actor_of(&key(8))), None);
}

#[test]
fn current_invite_is_per_issuer_not_per_old_link_or_arrival_order() {
    let first = signed(
        &key(1),
        100,
        0,
        RailAct::Invite {
            key: actor_of(&key(9)),
            until: 1_000,
            device_of: None,
        },
    );
    let newer = after(
        &key(1),
        &first,
        101,
        RailAct::Invite {
            key: actor_of(&key(8)),
            until: 102,
            device_of: Some(actor_of(&key(1))),
        },
    );
    let theirs = signed(
        &key(2),
        103,
        0,
        RailAct::Invite {
            key: actor_of(&key(7)),
            until: 2_000,
            device_of: None,
        },
    );
    let a = admitted(&[theirs.clone(), newer.clone(), first.clone()]);
    assert_eq!(a.current_invite(&actor_of(&key(1))).unwrap().0, newer.id);
    assert_eq!(a.current_invite(&actor_of(&key(2))).unwrap().0, theirs.id);
    assert_eq!(a.current_invite(&actor_of(&key(3))), None);
    assert_eq!(
        a.invites().newest(&actor_of(&key(1)), &actor_of(&key(9))),
        Some(&first.id)
    );
}
