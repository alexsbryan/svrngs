// SPDX-License-Identifier: AGPL-3.0-or-later
//! A guest, for a visit (guests-visit; svrn-docs `rings/current/GUESTS.md`
//! §3, §5): a `Visit` the host's computer signs, folded beside membership,
//! so every copy agrees who is visiting as it agrees who is a member.
use super::*;
use crate::tests_support::{key, p, run, NS};
use crate::{actor_of, admit, Admission, Ed25519Verifier};

/// Friday evening, and the hour.
const FRI: i64 = 1_000_000;
const HOUR: i64 = 3_600;
/// Sunday evening: 48 hours on.
const SUNDAY: i64 = FRI + 48 * HOUR;

/// A house of three, as a seed of three keys: alex keeps it, kim and bo.
fn house() -> Roster {
    let mut m = BTreeMap::new();
    m.insert(p("alex"), vec![actor_of(&key(1))]);
    m.insert(p("kim"), vec![actor_of(&key(2))]);
    m.insert(p("bo"), vec![actor_of(&key(3))]);
    Roster::new(m)
}

fn folded(ops: &[Op<SignedOp>]) -> Admission {
    admit(&crate::ActSet::of(ops), &[], &house(), NS, &Ed25519Verifier)
}

/// `host` has `guest`, holding the fresh key `fresh`, over until `until`,
/// opening the host's own Jellyfin.
fn visit(host: u8, guest: &str, fresh: u8, until: i64) -> RailAct {
    let mut opens = BTreeMap::new();
    opens.insert(actor_of(&key(host)), ["jellyfin".into()].into());
    RailAct::Visit {
        guest: p(guest),
        key: actor_of(&key(fresh)),
        opens,
        until,
    }
}

/// Who the fold says is visiting at `at`: each guest's name, their host's
/// key and until when.
fn visiting(a: &Admission, at: i64) -> Vec<(String, String, i64)> {
    (a.membership().visiting(at))
        .map(|(_, g)| (g.guest.to_string(), g.host.clone(), g.until))
        .collect()
}

/// **A guest, asked** (part (a)): Kim hosts Sam until Sunday, and the fold
/// says Sam is visiting, Kim's guest, until Sunday. Sam's key never
/// stands. Red at HEAD, since the record has no such act.
#[test]
fn kim_hosts_sam_until_sunday_and_the_fold_says_so() {
    let ops = run(&key(2), FRI, [visit(2, "sam", 9, SUNDAY)]);
    let a = folded(&ops);
    assert_eq!(
        visiting(&a, FRI + HOUR),
        [("sam".into(), actor_of(&key(2)), SUNDAY)],
        "Sam is visiting, Kim's guest, until Sunday: {:?}",
        a.gaps()
    );
    assert!(
        !a.membership().standing.contains(&actor_of(&key(9))),
        "a guest's key never stands"
    );
}
