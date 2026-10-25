// SPDX-License-Identifier: AGPL-3.0-or-later
//! Every kind of act as svrngs writes it on 2026-10-24, frozen before
//! authorship by signature changes what an act is (capability-authorship;
//! `demo/runs/authorship/2026-10-24/DESIGN.md`). One group's journal,
//! `fixtures/acts_2026-10-24.jsonl`, one line per act a computer holds
//! today: each link and redemption, a device link and a phone's
//! redemption, a plain `Admit`, records and corrections with and without
//! `on_behalf_of`, a folder's change and a member's change to a carried
//! app, a yes to its version for another person and for oneself and one
//! naming files no folder placed, a relay and none, a visit and its Stop,
//! an older build's seal, a cut and a hand-on.
//!
//! The lines are bytes every copy already holds, so they must keep their
//! bytes, their ids, their signatures and their line versions; and they
//! must fold as they do today, `fixtures/acts_2026-10-24.fold.json`. A
//! change that moves either is red here: the design names each act whose
//! fold it moves, and the operator says yes to that move before it lands.
//! Only the public API, as a computer on a newer svrngs reads an older
//! journal.

use std::collections::BTreeMap;

use svrngs_core::{
    actor_of, derive_id, sign_act, ActSet, Change, ContentHash, Device, Ed25519Verifier, Entry,
    Journaled, Op, OpId, Person, RailAct, Redemption, RingVerifier, Roster, SignedOp, SigningKey,
    SyncPath, Visit,
};

const NS: &str = "golden";
const JOURNAL: &str = include_str!("fixtures/acts_2026-10-24.jsonl");
const FOLD: &str = include_str!("fixtures/acts_2026-10-24.fold.json");
/// The keeper's cutover written past that journal, and acts past it
/// (capability-signed; DESIGN §1, "An entry's author is its signer"): a
/// second fixture beside the first, never in place of it.
const CUTOVER: &str = include_str!("fixtures/acts_2026-10-24.cutover.jsonl");
const CUTOVER_FOLD: &str = include_str!("fixtures/acts_2026-10-24.cutover.fold.json");
/// The first act's time; each next act is one second on, so the record's
/// order is the order written.
const T: i64 = 1_760_000_000;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn person(name: &str) -> Person {
    Person::parse(name).unwrap()
}

/// alex started the group: the seed names alex's computer alone.
fn seed() -> Roster {
    Roster::new(BTreeMap::from([(person("alex"), vec![actor_of(&key(1))])]))
}

fn journal() -> Vec<(String, Op<SignedOp>)> {
    lines(JOURNAL)
}

fn lines(of: &str) -> Vec<(String, Op<SignedOp>)> {
    (of.lines().filter(|l| !l.trim().is_empty()))
        .map(|l| {
            (
                l.to_string(),
                serde_json::from_str(l).expect("a journal line"),
            )
        })
        .collect()
}

/// What a computer folds of the journal, every part an app or a page
/// reads: the acts in their order as the record API gives them, the gaps,
/// who stands and how, what runs of each app as whom, and where the folder
/// places each file.
fn fold(ops: &[Op<SignedOp>]) -> serde_json::Value {
    let a = svrngs_core::admit(&ActSet::of(ops), &[], &seed(), NS, &Ed25519Verifier);
    let runs = a.runs();
    let said: Vec<_> = (runs.said.iter())
        .map(|((kind, whose), files)| serde_json::json!({ "kind": kind, "whose": whose, "files": files }))
        .collect();
    let changed: BTreeMap<_, _> = (runs.changed.iter())
        .map(|(kind, (who, at))| (kind, serde_json::json!({ "who": who, "at": at })))
        .collect();
    let runs_as = |kind: &str, who: &str| serde_json::json!(runs.of(kind, who));
    serde_json::json!({
        "ops": a.wire(),
        "gaps": a.gaps().iter().map(ToString::to_string).collect::<Vec<_>>(),
        "complete": a.is_complete(),
        "membership": a.membership(),
        "runs": {
            "brought": runs.brought,
            "said": said,
            "changed": changed,
            "money as alex": runs_as("money", "alex"),
            "money as bo": runs_as("money", "bo"),
            "money as cy": runs_as("money", "cy"),
        },
        "layout": a.layout(),
    })
}

/// Given the journal as svrngs wrote it on 2026-10-24, when a computer
/// reads each line, then the line serialises back to its own bytes, derives
/// the id it carries, verifies under its actor's key and names the line
/// version its kind ships at.
#[test]
fn every_kind_of_act_written_today_keeps_its_bytes_id_signature_and_version() {
    let lines = journal();
    assert_eq!(lines.len(), 26, "the journal as frozen");
    for (raw, op) in &lines {
        let at = format!("{} (seq {} of {})", op.id, op.kind.seq, op.actor);
        assert_eq!(&serde_json::to_string(op).unwrap(), raw, "{at}: its bytes");
        assert_eq!(derive_id(op, NS), op.id, "{at}: its id");
        assert!(
            Ed25519Verifier.verify(&op.actor, &op.id, &op.kind.sig),
            "{at}: its signature"
        );
        assert_eq!(op.kind.line_version(), op.v, "{at}: its line version");
    }
    let versions: Vec<u32> = lines.iter().map(|(_, op)| op.v).collect();
    assert_eq!(
        versions,
        [3, 3, 3, 3, 8, 4, 2, 1, 1, 1, 1, 1, 6, 6, 10, 10, 10, 9, 9, 11, 1, 1, 1, 7, 7, 1],
        "each kind at the line version it ships at today"
    );
}

/// Given the same journal, when a computer folds it, then every act counts,
/// voids, is refused and is attributed as it is today: `fold.json` holds
/// the whole answer, so any move in what an old act means is a diff here.
#[test]
fn every_kind_of_act_written_today_folds_as_it_does_today() {
    let ops: Vec<Op<SignedOp>> = journal().into_iter().map(|(_, op)| op).collect();
    let now = fold(&ops);
    let frozen: serde_json::Value = serde_json::from_str(FOLD).expect("the frozen fold");
    assert!(
        now == frozen,
        "the journal folds otherwise than it did on 2026-10-24; now:\n{}",
        serde_json::to_string_pretty(&now).unwrap()
    );
}

/// The same journal, its lines in another order: the fold is
/// a function of the set, so the frozen answer holds however the lines
/// arrived.
#[test]
fn the_frozen_fold_holds_whatever_order_the_lines_arrive_in() {
    let mut ops: Vec<Op<SignedOp>> = journal().into_iter().map(|(_, op)| op).collect();
    ops.reverse();
    let frozen: serde_json::Value = serde_json::from_str(FOLD).unwrap();
    assert!(
        fold(&ops) == frozen,
        "the fold moved with the order the lines arrived in"
    );
}

/// Given the frozen journal and, past it, the keeper's cutover and four
/// acts, when a computer reads each new line, then it keeps its bytes, id,
/// signature and line version, the cutover at version 12.
#[test]
fn the_keepers_cutover_and_the_acts_past_it_keep_their_bytes_id_signature_and_version() {
    let lines = lines(CUTOVER);
    for (raw, op) in &lines {
        let at = format!("{} (seq {} of {})", op.id, op.kind.seq, op.actor);
        assert_eq!(&serde_json::to_string(op).unwrap(), raw, "{at}: its bytes");
        assert_eq!(derive_id(op, NS), op.id, "{at}: its id");
        assert!(
            Ed25519Verifier.verify(&op.actor, &op.id, &op.kind.sig),
            "{at}: its signature"
        );
        assert_eq!(op.kind.line_version(), op.v, "{at}: its line version");
    }
    let versions: Vec<u32> = lines.iter().map(|(_, op)| op.v).collect();
    assert_eq!(versions, [12, 1, 1, 1], "the cutover, then three records");
}

/// Given the same, when a computer folds the whole, then every act the
/// frozen journal holds folds as it did (the cutover moves no old act), and
/// past it alex's computer's record for bo counts nowhere, named
/// `carried_for_another`, while alex's own and bo's own count: the whole
/// answer frozen in `cutover.fold.json`.
#[test]
fn past_the_keepers_cutover_an_act_for_another_counts_nowhere_and_old_acts_as_before() {
    let ops: Vec<Op<SignedOp>> = (journal().into_iter().chain(lines(CUTOVER)))
        .map(|(_, op)| op)
        .collect();
    let now = fold(&ops);
    let before: serde_json::Value = serde_json::from_str(FOLD).unwrap();
    for old in before["ops"].as_array().unwrap() {
        assert!(
            now["ops"].as_array().unwrap().contains(old),
            "the cutover moved an old act: {old}"
        );
    }
    let gaps = now["gaps"].as_array().unwrap();
    for old in before["gaps"].as_array().unwrap() {
        assert!(gaps.contains(old), "the cutover moved an old gap: {old}");
    }
    let past = lines(CUTOVER)[1].1.id.clone();
    let carried = svrngs_core::RailGap::CarriedForAnother {
        id: past,
        actor: actor_of(&key(1)),
        named: "bo".into(),
    };
    assert!(
        gaps.contains(&serde_json::json!(carried.to_string())),
        "{gaps:?}"
    );
    let frozen: serde_json::Value = serde_json::from_str(CUTOVER_FOLD).unwrap();
    assert!(
        now == frozen,
        "the journal past the cutover folds otherwise than frozen; now:\n{}",
        serde_json::to_string_pretty(&now).unwrap()
    );
}

/// One actor's chain: its next seq and the id it names as `prev`.
#[derive(Default)]
struct Chains(BTreeMap<String, (u64, Option<OpId>)>);

impl Chains {
    fn sign(
        &mut self,
        signer: &SigningKey,
        at: i64,
        act: RailAct,
        obo: Option<&str>,
    ) -> Op<SignedOp> {
        let (seq, prev) = self.0.entry(actor_of(signer)).or_default();
        let op = sign_act(signer, NS, T + at, *seq, prev.clone(), act, obo, None, None);
        (*seq, *prev) = (*seq + 1, Some(op.id.clone()));
        op
    }
}

fn record(payload: serde_json::Value) -> RailAct {
    RailAct::from_json(serde_json::json!({ "op": "record", "payload": payload })).unwrap()
}

fn correct(id: &OpId, replacement: Option<serde_json::Value>) -> RailAct {
    let body = serde_json::json!({ "op": "correct", "corrects": id, "replacement": replacement });
    RailAct::from_json(body).unwrap()
}

/// The generator for `fixtures/acts_2026-10-24.jsonl` and its fold: the acts
/// each command and route signs today (cli.rs, `register.rs`, `rail.rs`
/// `append_act`, `carried.rs`'s `ACTS` and `says`, `folder`'s sync, a
/// relay, a visit), in one group: alex started it on computer 1; bo joins
/// on computer 2 and adds a phone, 4; cy joins on computer 3 and is cut.
/// Kept so the freeze is reproducible; it prints the journal, then a line
/// `--`, then the fold.
#[test]
#[ignore = "fixture generator — run with --ignored --nocapture"]
fn emit_golden_journal() {
    let (alex, bo, cy, phone) = (key(1), key(2), key(3), key(4));
    let (link_bo, link_cy, link_phone) = (key(11), key(12), key(13));
    let mut c = Chains::default();
    let mut ops = Vec::new();
    let until = T + 86_400;
    let invite = |k: &SigningKey, device_of: Option<String>| RailAct::Invite {
        key: actor_of(k),
        until,
        device_of,
    };
    let admit = |who: &str, k: &SigningKey, of: &OpId, device| RailAct::Admit {
        person: person(who),
        key: actor_of(k),
        invite: Some(Redemption::sign(k, NS, of, &person(who), device)),
    };

    let to_bo = c.sign(&alex, 0, invite(&link_bo, None), None);
    let bo_in = c.sign(
        &alex,
        1,
        admit("bo", &bo, &to_bo.id, Device::Computer),
        None,
    );
    ops.extend([to_bo, bo_in]);
    let to_cy = c.sign(&alex, 2, invite(&link_cy, None), None);
    let cy_in = c.sign(
        &alex,
        3,
        admit("cy", &cy, &to_cy.id, Device::Computer),
        None,
    );
    ops.extend([to_cy, cy_in]);
    let bos_phone = c.sign(&bo, 4, invite(&link_phone, Some(actor_of(&bo))), None);
    let phone_in = c.sign(
        &bo,
        5,
        admit("bo", &phone, &bos_phone.id, Device::Browser),
        None,
    );
    ops.extend([bos_phone, phone_in]);
    let plain = RailAct::Admit {
        person: person("dee"),
        key: actor_of(&key(5)),
        invite: None,
    };
    ops.push(c.sign(&alex, 6, plain, None));

    // What apps write: through the record API as the computer's own, and
    // through a carried app's route, as the person the door resolved.
    let dinner =
        serde_json::json!({ "kind": "money/expense", "what": "dinner", "amount_cents": 4000 });
    let own = c.sign(&alex, 7, record(dinner.clone()), None);
    let carried = c.sign(
        &alex,
        8,
        record(serde_json::json!({ "kind": "money/expense", "what": "milk", "amount_cents": 250 })),
        Some("alex"),
    );
    let bos_by_phone = c.sign(
        &alex,
        9,
        record(
            serde_json::json!({ "kind": "money/expense", "what": "rent", "amount_cents": 90000 }),
        ),
        Some("bo"),
    );
    let bo_fixes = c.sign(&bo, 10, correct(&bos_by_phone.id, Some(serde_json::json!({ "kind": "money/expense", "what": "rent", "amount_cents": 95000 }))), Some("bo"));
    let cy_voids = c.sign(&cy, 11, correct(&own.id, None), Some("cy"));
    ops.extend([own, carried, bos_by_phone, bo_fixes, cy_voids]);

    // A carried app brought, then changed by bo, and a yes to each version.
    let page = SyncPath::parse("apps/money/index.html").unwrap();
    let (brought, changed, stray) = (
        ContentHash::of(b"<p>money 1"),
        ContentHash::of(b"<p>money 2"),
        ContentHash::of(b"<p>not placed"),
    );
    let place = |now: &ContentHash, base| RailAct::Synced {
        folder: "shared".into(),
        changes: vec![Change {
            path: page.clone(),
            now: Some((*now, 10)),
            base,
        }],
    };
    let first = c.sign(&alex, 12, place(&brought, None), None);
    let base = Entry {
        key: first.actor.clone(),
        seq: first.kind.seq,
        path: page.clone(),
    };
    ops.extend([first, c.sign(&bo, 13, place(&changed, Some(base)), None)]);
    let files = |h: &ContentHash| BTreeMap::from([(page.clone(), *h)]);
    let runs = |h| RailAct::Runs {
        app: "money".into(),
        files: files(h),
    };
    ops.push(c.sign(&alex, 14, runs(&changed), Some("bo")));
    ops.push(c.sign(&bo, 15, runs(&changed), Some("bo")));
    ops.push(c.sign(&cy, 16, runs(&stray), Some("cy")));

    ops.push(c.sign(
        &alex,
        17,
        RailAct::Relay {
            url: Some("https://relay.example.net/".into()),
        },
        None,
    ));
    ops.push(c.sign(&bo, 18, RailAct::Relay { url: None }, None));
    let opens = BTreeMap::from([(actor_of(&alex), ["films".to_string()].into())]);
    let visit = Visit {
        guest: person("gil"),
        key: actor_of(&key(6)),
        opens,
        until: T + 19 + 3_600,
    };
    let visit = c.sign(&alex, 19, RailAct::Visit(visit), None);
    let stop = c.sign(&alex, 20, correct(&visit.id, None), None);
    ops.extend([visit, stop, c.sign(&alex, 21, RailAct::Seal, None)]);
    let cys_last = c.sign(
        &cy,
        22,
        record(serde_json::json!({ "kind": "money/expense", "what": "tea", "amount_cents": 300 })),
        None,
    );
    let cut = RailAct::Remove {
        key: actor_of(&cy),
        through: vec![cys_last.id.clone()],
    };
    ops.extend([
        cys_last,
        c.sign(&alex, 23, cut, None),
        c.sign(&alex, 24, RailAct::HandOn { key: actor_of(&bo) }, None),
    ]);
    ops.push(c.sign(
        &bo,
        25,
        record(serde_json::json!({ "kind": "events/event", "what": "dinner" })),
        Some("bo"),
    ));

    for op in &ops {
        println!("{}", serde_json::to_string(op).unwrap());
    }
    println!("--");
    println!("{}", serde_json::to_string_pretty(&fold(&ops)).unwrap());
}

/// The generator for `fixtures/acts_2026-10-24.cutover.jsonl` and its
/// fold: each chain picked up where the frozen journal leaves it; bo, whom
/// alex handed the keeping to, writes the cutover naming every key's newest
/// act, as `register.rs`'s `Cutover` does; then alex's computer writes a
/// record for bo and one of its own, and bo one naming bo. It prints the new
/// lines, then `--`, then the fold of the whole.
#[test]
#[ignore = "fixture generator — run with --ignored --nocapture"]
fn emit_cutover_journal() {
    let old: Vec<Op<SignedOp>> = journal().into_iter().map(|(_, op)| op).collect();
    let mut c = Chains::default();
    for op in &old {
        c.0.insert(op.actor.clone(), (op.kind.seq + 1, Some(op.id.clone())));
    }
    let through = c.0.values().filter_map(|(_, at)| at.clone()).collect();
    let (alex, bo) = (key(1), key(2));
    let tea = |what| {
        record(serde_json::json!({ "kind": "money/expense", "what": what, "amount_cents": 300 }))
    };
    let new = [
        c.sign(&bo, 26, RailAct::Signers { through }, None),
        c.sign(&alex, 27, tea("for bo"), Some("bo")),
        c.sign(&alex, 28, tea("alex's"), None),
        c.sign(&bo, 29, tea("bo's"), Some("bo")),
    ];
    for op in &new {
        println!("{}", serde_json::to_string(op).unwrap());
    }
    println!("--");
    let all: Vec<_> = old.into_iter().chain(new).collect();
    println!("{}", serde_json::to_string_pretty(&fold(&all)).unwrap());
}
