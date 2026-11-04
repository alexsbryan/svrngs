// SPDX-License-Identifier: AGPL-3.0-or-later
//! #211, reproduced in svrngs-core's fold alone: no node, no clock, no
//! network. Each test builds the acts the run wrote, folds them on the
//! copies the run had (one holding every act, one lacking the leaver's last
//! act), prints what each copy counts, and asserts what HEAD does: the
//! divergence. A test here going red means HEAD's fold changed.
//!
//! Not in the crate: `demo/runs/leave/2026-10-21/repro.sh` copies it into
//! `crates/svrngs-core/tests/`, runs it with `--features test-support`, and
//! removes it.

use std::collections::BTreeMap;

use svrngs_core::tests_support::{after, applied, key, p, record, signed, NS};
use svrngs_core::{
    actor_of, admit, Admission, Ed25519Verifier, Op, OpId, RailAct, RailGap, Roster, SignedOp,
};

const FRI: i64 = 1_000_000;
const HOUR: i64 = 3_600;

/// Oak as a seed of three keys: Ana (1), who keeps it, her name sorting
/// first; Kim (2); Bo (3).
fn oak() -> Roster {
    let mut m = BTreeMap::new();
    m.insert(p("ana"), vec![actor_of(&key(1))]);
    m.insert(p("kim"), vec![actor_of(&key(2))]);
    m.insert(p("bo"), vec![actor_of(&key(3))]);
    Roster::new(m)
}

fn fold(ops: &[Op<SignedOp>]) -> Admission {
    admit(&svrngs_core::ActSet::of(ops), &[], &oak(), NS, &Ed25519Verifier)
}

/// `host` (a key seed) has `guest` on a fresh key, `hours` from `at`.
fn visit(host: u8, guest: &str, fresh: u8, at: i64, hours: i64) -> RailAct {
    let mut opens = BTreeMap::new();
    opens.insert(actor_of(&key(host)), ["jellyfin".into()].into());
    RailAct::Visit(svrngs_core::Visit {
        guest: p(guest),
        key: actor_of(&key(fresh)),
        opens,
        until: at + hours * HOUR,
    })
}

fn correct(target: &Op<SignedOp>) -> RailAct {
    RailAct::Correct {
        corrects: target.id.clone(),
        replacement: None,
    }
}

/// A `Remove` of `k` naming its newest act held, as svrngs-core's `Leave`
/// and `Removal` write one for each key (register.rs `removes`,
/// `before.tips(k)`): signed by `k`, a leave; by the keeper, the cut.
fn remove(k: u8, tip: &Op<SignedOp>) -> RailAct {
    RailAct::Remove {
        key: actor_of(&key(k)),
        through: vec![tip.id.clone()],
    }
}

/// Who each copy says is visiting at `at`: guest and host's key seed.
fn visiting(a: &Admission, at: i64) -> Vec<String> {
    let seed_of = |k: &str| (1..=3).find(|s| actor_of(&key(*s)) == k).unwrap_or(0);
    (a.membership().visiting(at))
        .map(|(_, g)| format!("{} ({}'s guest)", g.visit.guest, ["?", "ana", "kim", "bo"][seed_of(&g.host) as usize]))
        .collect()
}

fn gap_kinds(a: &Admission) -> Vec<String> {
    a.gaps()
        .iter()
        .map(|g| serde_json::to_value(g).unwrap()["gap"].as_str().unwrap().to_string())
        .collect()
}

fn show(name: &str, a: &Admission, at: i64) {
    eprintln!(
        "  {name:<22} visiting {:?}; records applied {:?}; gaps {:?}; complete {}",
        visiting(a, at),
        applied(a),
        gap_kinds(a),
        a.is_complete()
    );
}

fn removed(a: &Admission, op: &Op<SignedOp>) -> bool {
    a.gaps()
        .iter()
        .any(|g| matches!(g, RailGap::Removed { id, .. } if *id == op.id))
}

fn unheld(a: &Admission) -> Vec<OpId> {
    a.gaps()
        .iter()
        .filter_map(|g| match g {
            RailGap::UnheldCut { missing, .. } => Some(missing.clone()),
            _ => None,
        })
        .collect()
}

/// **The run's story** (demo/runs/guests/2026-10-20-run, "What it found"
/// 1), with a money entry and a correction of Bo's entry beside the
/// visits. Kim hosts Sam and Jo; Ana's Max is past the house's bound (two
/// guests on, three people); Kim stops Sam, writes a money entry, corrects
/// Bo's typo, hosts Lee, and two seconds later leaves. Ana's copy holds
/// every act but Lee's visit, as Ana's computer did in all six takes.
#[test]
fn a_leave_ahead_of_its_last_act_costs_the_leavers_earlier_acts() {
    let k0 = signed(&key(2), FRI, 0, visit(2, "Sam", 9, FRI, 48));
    let k1 = after(&key(2), &k0, FRI + 60, visit(2, "Jo", 10, FRI + 60, 6));
    let max = signed(&key(1), FRI + 120, 0, visit(1, "Max", 11, FRI + 120, 48));
    let k2 = after(&key(2), &k1, FRI + 130, correct(&k0)); // Kim's Stop of Sam
    let k3 = after(&key(2), &k2, FRI + 140, record("kim-paid-40"));
    let typo = signed(&key(3), FRI + 150, 0, record("bo-typo"));
    let k4 = after(&key(2), &k3, FRI + 160, correct(&typo)); // Kim voids Bo's typo
    let lee = after(&key(2), &k4, FRI + 200, visit(2, "Lee", 12, FRI + 200, 48));
    let left = after(&key(2), &lee, FRI + 202, remove(2, &lee));
    let every = vec![
        k0.clone(), k1.clone(), max.clone(), k2.clone(), k3.clone(),
        typo.clone(), k4.clone(), lee.clone(), left.clone(),
    ];
    let lacking: Vec<_> = every.iter().filter(|op| op.id != lee.id).cloned().collect();
    let (kims, anas) = (fold(&every), fold(&lacking));
    let healed = fold(&[lacking.clone(), vec![lee.clone()]].concat());
    let end = FRI + 300;
    eprintln!("the run's story, at the end (after Kim's leave):");
    show("Kim's copy (every act)", &kims, end);
    show("Ana's copy (no Lee)", &anas, end);
    show("Ana's, Lee's arrived", &healed, end);

    // Kim's copy: the leave covers all five of Kim's acts; Kim's cut ends
    // Jo's and Lee's visits; Max was never a visit; Bo's typo is voided.
    assert!([&k0, &k1, &k2, &k3, &k4, &lee, &left].iter().all(|op| kims.membership().counts(&op.id)));
    assert!(visiting(&kims, end).is_empty());
    assert!(!kims.membership().counts(&max.id), "Max past the bound");
    assert_eq!(applied(&kims), ["kim-paid-40"]);

    // Ana's copy: the leave covers only itself (membership.rs `covered`,
    // the prev chain stops at Lee's visit, which is not held), so every
    // earlier act of Kim's is `Removed`; Max, judged with no visit before
    // it, counts, and outlives Kim's cut since Ana hosts him; Kim's void
    // of Bo's typo is gone, so the typo counts again; Kim's money entry
    // counts nowhere.
    assert!([&k0, &k1, &k2, &k3, &k4].iter().all(|op| removed(&anas, op)));
    assert!(anas.membership().counts(&left.id), "the leave itself counts");
    assert_eq!(unheld(&anas), vec![lee.id.clone()]);
    assert!(anas.membership().counts(&max.id), "Max counts on Ana's copy");
    assert_eq!(visiting(&anas, end), ["Max (ana's guest)"]);
    assert_eq!(applied(&anas), ["bo-typo"]);

    // The fold is a function of the set: the act arriving heals it.
    assert_eq!(healed.membership(), kims.membership());
    assert_eq!(applied(&healed), applied(&kims));
}

/// **The leaver's Admit too.** Kim admitted Dee; Dee writes; Kim writes
/// once more, unsent, and leaves. On a copy lacking that last act Kim's
/// Admit of Dee counts nowhere, so Dee is no member there: her act is an
/// unknown signer's, and that copy's register refuses her sync and lends
/// (`RingRail::standing`, the one answer the round, route and door ask).
#[test]
fn a_leave_ahead_of_its_last_act_unseats_whoever_the_leaver_admitted() {
    let dee_in = signed(
        &key(2),
        FRI,
        0,
        RailAct::Admit { person: p("dee"), key: actor_of(&key(4)), invite: None },
    );
    let dee_pays = signed(&key(4), FRI + 10, 0, record("dee-paid"));
    let last = after(&key(2), &dee_in, FRI + 100, record("kim-last"));
    let left = after(&key(2), &last, FRI + 102, remove(2, &last));
    let every = vec![dee_in.clone(), dee_pays.clone(), last.clone(), left.clone()];
    let lacking: Vec<_> = every.iter().filter(|op| op.id != last.id).cloned().collect();
    let (kims, anas) = (fold(&every), fold(&lacking));
    eprintln!("Kim admitted Dee, then left:");
    show("Kim's copy (every act)", &kims, FRI + 200);
    show("Ana's copy (no last)", &anas, FRI + 200);
    let dee = actor_of(&key(4));
    assert!(kims.membership().standing.contains(&dee), "Dee stands on Kim's copy");
    assert_eq!(applied(&kims), ["dee-paid", "kim-last"]);
    assert!(!anas.membership().standing.contains(&dee), "Dee is no member on Ana's");
    assert!(anas.gaps().iter().any(|g| matches!(g, RailGap::UnknownSigner { id, .. } if *id == dee_pays.id)));
    assert!(applied(&anas).is_empty());
}

/// **A keeper's removal, beside it.** Ana keeps Oak and removes Kim, having
/// held Kim's newest act; Bo's copy holds the cut and not that act. There,
/// as on a leave, Kim's earlier act counts nowhere — but the act the cut
/// names is the keeper's to hold, and the keeper stands, so Bo's next
/// exchange with Ana carries it (sync.rs, author-blind), and the fold heals.
#[test]
fn a_keepers_cut_ahead_of_the_act_it_names_costs_the_same_until_it_arrives() {
    let early = signed(&key(2), FRI, 0, record("kim-early"));
    let newest = after(&key(2), &early, FRI + 10, record("kim-newest"));
    let cut = signed(&key(1), FRI + 20, 0, remove(2, &newest));
    let every = vec![early.clone(), newest.clone(), cut.clone()];
    let bos = fold(&[early.clone(), cut.clone()]);
    let anas = fold(&every);
    eprintln!("Ana, keeping, removed Kim:");
    show("Ana's copy (every act)", &anas, FRI + 30);
    show("Bo's copy (no newest)", &bos, FRI + 30);
    assert_eq!(applied(&anas), ["kim-early", "kim-newest"]);
    assert!(removed(&bos, &early) && applied(&bos).is_empty());
    assert_eq!(unheld(&bos), vec![newest.id.clone()]);
}

/// **What a missing act decides, no fold can count** (option D's limit).
/// Kim hosts Jo, then Lee, her last act; Bo hosts Pat a second later; Kim
/// leaves. With Jo and Lee on, Pat is past the house's bound (two guests,
/// three people: membership.rs, `on.len() + 1 < members`), so Kim's copy
/// holds no visit of Pat's. Ana's copy lacks Lee's visit, and counts Pat:
/// at HEAD, since Jo's visit counts nowhere there; and under a rule that
/// counted Jo's (held, and below the leave), still, since one guest on is
/// under the bound. Only Lee's visit arriving settles it.
#[test]
fn a_missing_act_still_decides_what_turns_on_it() {
    let jo = signed(&key(2), FRI, 0, visit(2, "Jo", 10, FRI, 6));
    let lee = after(&key(2), &jo, FRI + 200, visit(2, "Lee", 12, FRI + 200, 48));
    let pat = signed(&key(3), FRI + 201, 0, visit(3, "Pat", 13, FRI + 201, 48));
    let left = after(&key(2), &lee, FRI + 202, remove(2, &lee));
    let kims = fold(&[jo.clone(), lee.clone(), pat.clone(), left.clone()]);
    let anas = fold(&[jo.clone(), pat.clone(), left.clone()]);
    eprintln!("Bo hosts Pat between Kim's last act and her leave:");
    show("Kim's copy (every act)", &kims, FRI + 300);
    show("Ana's copy (no Lee)", &anas, FRI + 300);
    assert!(!kims.membership().counts(&pat.id), "past the bound on Kim's copy");
    assert!(visiting(&kims, FRI + 300).is_empty());
    assert!(anas.membership().counts(&pat.id), "a visit on Ana's");
    assert_eq!(visiting(&anas, FRI + 300), ["Pat (bo's guest)"]);
}
