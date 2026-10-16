// SPDX-License-Identifier: AGPL-3.0-or-later
//! What an app may write, given / when / then, through `commit` in memory.

use crate::tests_support::*;
use crate::*;

fn author<'a>(signer: &'a SigningKey, roster: &'a Roster) -> Author<'a> {
    Author {
        signer,
        roster,
        namespace: NS,
        verifier: &Ed25519Verifier,
        now: 1_000,
    }
}

fn append(act: RailAct, on_behalf_of: Option<&str>) -> Append {
    Append {
        act,
        on_behalf_of: on_behalf_of.map(p),
    }
}

#[test]
fn an_app_record_is_written_and_stamped_with_whom_it_speaks_for() {
    let store = Memory::holding(&[]);
    let (signer, roster) = (key(1), ring());
    let recorded = commit(
        &store,
        &append(record("x"), Some("bo")),
        &author(&signer, &roster),
    )
    .unwrap()
    .unwrap();
    assert_eq!(recorded.lines().len(), 1);
    assert_eq!(recorded.lines()[0].kind.on_behalf_of.as_deref(), Some("bo"));
}

#[test]
fn an_app_cannot_write_the_registers_acts_or_a_seal() {
    let invite = RailAct::Invite {
        key: actor_of(&key(9)),
        until: 2_000,
        device_of: None,
    };
    let admit = RailAct::Admit {
        person: p("hal"),
        key: actor_of(&key(7)),
        invite: None,
    };
    let before = admitted(&[]);
    for act in [invite, admit, RailAct::Seal] {
        assert_eq!(
            append(act.clone(), None).decide(&before),
            Err(AppendRefusal::NotAnAppAct),
            "{act:?}"
        );
    }
}

#[test]
fn on_behalf_of_someone_outside_the_group_writes_nothing() {
    let store = Memory::holding(&[]);
    let (signer, roster) = (key(1), ring());
    let refused = commit(
        &store,
        &append(record("x"), Some("zed")),
        &author(&signer, &roster),
    )
    .unwrap();
    assert_eq!(
        refused,
        Err(Refused::Command(AppendRefusal::NotInGroup(p("zed"))))
    );
    assert!(store.0.borrow().is_empty());
}
