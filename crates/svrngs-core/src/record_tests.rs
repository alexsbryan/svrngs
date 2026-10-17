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

/// **A commit signs no view** (arch-view; svrn-docs A_RECORD_FOR_YEARS §3).
/// bo writes after alex, so a view would name alex; the line bo's commit
/// makes holds none, carries no `view` on the wire, and verifies. The failing
/// input: the commit stamps its signer's view, as at the row's start.
#[test]
fn a_commit_signs_no_view() {
    let store = Memory::holding(&[]);
    let roster = ring();
    let (alex, bo) = (key(1), key(2));
    commit(
        &store,
        &append(record("milk"), None),
        &author(&alex, &roster),
    )
    .unwrap()
    .unwrap();
    let recorded = commit(
        &store,
        &append(record("bread"), None),
        &author(&bo, &roster),
    )
    .unwrap()
    .unwrap();
    let line = &recorded.lines()[0];
    assert_eq!(line.kind.view, None, "a new act holds no view");
    let wire = serde_json::to_string(line).unwrap();
    assert!(!wire.contains("\"view\""), "nor on the wire: {wire}");
    let all: Vec<_> = store.0.borrow().iter().cloned().collect();
    assert!(admitted(&all).gaps().is_empty(), "and it verifies");
}

/// **Every act a commit signed with its view still verifies** (arch-view;
/// fence 3). `fixtures/rail_ops_with_a_view.jsonl` is two lines `commit`
/// wrote at c276c34f, the row's start, which stamped each signer's view:
/// alex's record, then bo's, whose view names alex. They are held, admitted
/// whole and counted, as every journal on disk holding them is.
#[test]
fn acts_signed_with_a_view_still_verify() {
    let ops: Vec<Op<SignedOp>> = include_str!("fixtures/rail_ops_with_a_view.jsonl")
        .lines()
        .map(|l| serde_json::from_str(l).expect("the fixture is a journal line"))
        .collect();
    assert_eq!(ops.len(), 2);
    assert!(
        ops.iter().all(|o| o.kind.view.is_some()),
        "each carries a view"
    );
    let bo = actor_of(&key(2));
    let named = ops
        .iter()
        .find(|o| o.actor == bo)
        .unwrap()
        .kind
        .view
        .as_ref()
        .unwrap();
    assert!(
        named.get(&actor_of(&key(1))).is_some(),
        "bo's view names alex"
    );
    let f = admitted(&ops);
    assert!(f.gaps().is_empty(), "{:?}", f.gaps());
    assert_eq!(f.ops().len(), 2, "both counted");
}
