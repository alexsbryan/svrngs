// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/rail/tests.rs@e1a43bcd2
//! The rail doors' tests. (svrngs: upstream's fixtures built a mesh whose
//! membership was every ring's roster; here a ring's roster is its seed
//! file, which the register reads, DESIGN §8, U1's inventory. The roster
//! derivation's tests, the two-keys census and the sync doors' did not come
//! over with what they test, and the two attestation tests went with the
//! branch they tested.)

use std::sync::Arc;

use axum::http::StatusCode;

use crate::{actor_of, Person, RingRail, RingSigner, Roster, SigningKey};

use super::{append_act, drain_answer, group_of, push_answer, LiveBuffer, RailApi};

fn key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// A rail over a tempdir signing as key 7, with `ledger` seeded with
/// `members`, each a person and the seed of their key. The tempdir is
/// returned BY VALUE and must be kept: the rail writes under it for as long
/// as it lives. (svrngs: upstream installed a mesh as the default roster.)
fn rail_with(members: &[(&str, u8)]) -> (tempfile::TempDir, Arc<RingRail>) {
    let dir = tempfile::tempdir().unwrap();
    let rail = Arc::new(RingRail::new(dir.path(), Arc::new(key(7))));
    if !members.is_empty() {
        let roster = Roster::new(
            members
                .iter()
                .map(|(name, seed)| (Person::parse(name).unwrap(), vec![actor_of(&key(*seed))]))
                .collect(),
        );
        rail.set_seed("ledger", &roster).unwrap();
    }
    (dir, rail)
}

async fn body_of(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// One act in, one admitted op out — and every field a client could try to
/// choose for itself is this door's, not the body's. (svrngs: upstream's
/// body named `on_behalf_of` among them, which it dropped; here an app
/// names the person it serves, tested below.)
#[tokio::test]
async fn append_signs_as_the_node_and_assigns_the_fields() {
    let (_dir, rail) = rail_with(&[("alex", 7)]);
    group_of(&rail, "ledger").unwrap();
    let journal = "ledger";
    let body = serde_json::json!({
        "op": "record",
        "payload": { "amount": 4 },
        // Every one of these is the server's to assign; a body that could
        // choose them could write as somebody else.
        "seq": 999,
        "id": "forged",
        "ts_unix": 1,
    });
    let resp = append_act(&rail, journal, body).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let out = body_of(resp).await;
    assert_eq!(out["seq"], 0, "the door assigns the sequence, not the body");
    assert_eq!(out["actor"], RingSigner::actor(&key(7)));
    assert!(out["ts_unix"].as_i64().unwrap_or(0) > 0);
    assert!(out["id"].as_str().is_some());
    assert_eq!(out["namespace"], "ledger");
    assert!(
        out.get("retired").is_none(),
        "a record has no prune to render"
    );
}

const GUESTS: &[(&str, u8)] = &[("alex", 7), ("halo", 3)];

/// An app serving someone else names them (svrngs, DESIGN §3): the act is
/// signed as this computer with `on_behalf_of` naming that person, when
/// they stand in the group. A name of nobody in it, a name that is not one,
/// and a name that is not text are each refused by kind, and write nothing.
/// The failing input: `on_behalf_of` dropped, as upstream's session-less
/// door dropped it.
#[tokio::test]
async fn an_append_names_the_person_an_app_serves() {
    let (_dir, rail) = rail_with(GUESTS);
    group_of(&rail, "ledger").unwrap();
    let journal = "ledger";
    let body = |named: serde_json::Value| serde_json::json!({ "op": "record", "payload": { "amount": 4 }, "on_behalf_of": named });
    for (named, kind) in [
        (serde_json::json!("beefy"), "not_in_group"),
        (serde_json::json!(" "), "not_a_name"),
        (serde_json::json!("a\u{1b}[2J"), "not_a_name"),
        (serde_json::json!(7), "not_a_name"),
    ] {
        let resp = append_act(&rail, journal, body(named.clone())).await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY, "{named}");
        assert_eq!(body_of(resp).await["kind"], kind, "{named}");
    }
    assert!(
        svrngs_core::tests_support::wire_ops(&rail.admission(journal).unwrap()).is_empty(),
        "a refusal writes nothing"
    );

    let resp = append_act(&rail, journal, body(serde_json::json!("halo"))).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        body_of(resp).await["actor"],
        RingSigner::actor(&key(7)),
        "this computer signs"
    );
    let ops = svrngs_core::tests_support::wire_ops(&rail.admission(journal).unwrap());
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].on_behalf_of.as_deref(), Some("halo"));
}

/// An unstamped append is unchanged: signed as the node, no name on it.
#[tokio::test]
async fn an_unstamped_append_carries_no_name() {
    let (_dir, rail) = rail_with(GUESTS);
    group_of(&rail, "ledger").unwrap();
    let journal = "ledger";
    let body = serde_json::json!({ "op": "record", "payload": { "amount": 4 } });
    let resp = append_act(&rail, journal, body).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let ops = svrngs_core::tests_support::wire_ops(&rail.admission(journal).unwrap());
    assert_eq!(ops[0].on_behalf_of, None);
}

/// Nobody in the roster → every op this node writes would be unreadable to
/// every peer, and the refusal is typed. (svrngs: upstream's sentence named
/// its join verb, chosen for a derived roster; here a seed that names
/// another key is a group this computer is held in but does not stand in,
/// and the test asserts the type.)
#[tokio::test]
async fn append_outside_the_mesh_refuses_naming_the_join_verb() {
    let (_dir, rail) = rail_with(&[("halo", 3)]);
    let journal = "ledger";
    let body = serde_json::json!({ "op": "record", "payload": { "amount": 4 } });
    let resp = append_act(&rail, journal, body).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let out = body_of(resp).await;
    assert_eq!(out["kind"], "not_in_roster", "{out}");
}

/// The register is asked before a journal is opened (svrngs, DESIGN §8,
/// U3's inventory): a group this computer does not hold is refused
/// `not_held` and leaves nothing on disk, and one held with no seed is
/// refused `no_seed`. The failing input: `journal_of` opening any
/// namespace that is a name, as upstream's did.
#[tokio::test]
async fn a_group_not_held_or_without_a_seed_is_refused_before_it_is_opened() {
    let (dir, rail) = rail_with(&[("alex", 7)]);
    let refused = group_of(&rail, "elsewhere").err().unwrap();
    assert_eq!(refused.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_of(refused).await["kind"], "not_held");
    assert!(
        !dir.path().join("rings/elsewhere").exists(),
        "nothing is opened on disk"
    );

    std::fs::create_dir_all(dir.path().join("rings/unseeded")).unwrap();
    let refused = group_of(&rail, "unseeded").err().unwrap();
    assert_eq!(refused.status(), StatusCode::CONFLICT);
    assert_eq!(body_of(refused).await["kind"], "no_seed");

    assert!(
        group_of(&rail, "ledger").is_ok(),
        "control: a held, seeded group opens"
    );
}

/// The namespace is never guessed: absent is a 400 that names the fix, and
/// a name the journal would refuse as a directory is the rail's own 400.
#[tokio::test]
async fn the_namespace_must_be_a_name_the_journal_can_open() {
    let (_dir, rail) = rail_with(&[("alex", 7)]);
    let q = super::RailQuery { namespace: None };
    let err = super::namespace_of(&q).unwrap_err();
    assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    let refused = group_of(&rail, "Not A Namespace");
    assert!(
        refused.is_err(),
        "an unopenable namespace is refused, not sanitized"
    );
}

/// Log ships the admitted order, the roster it was admitted against, and
/// completeness — acts without their gaps would be a confident answer over
/// a subset.
#[tokio::test]
async fn log_ships_admitted_ops_gaps_and_the_roster() {
    let (_dir, rail) = rail_with(&[("alex", 7)]);
    group_of(&rail, "ledger").unwrap();
    let journal = "ledger";
    let body = serde_json::json!({ "op": "record", "payload": { "amount": 4 } });
    let resp = append_act(&rail, journal, body).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = super::log_answer(&rail, journal).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let out = body_of(resp).await;
    assert_eq!(out["complete"], true);
    // `held` is the journal-line count, refused lines included: one line on
    // disk, one op admitted, so nothing was refused.
    assert_eq!(out["held"], 1);
    assert_eq!(out["ops"].as_array().unwrap().len(), 1);
    assert_eq!(out["ops"][0]["person"], "alex");
    assert_eq!(out["gaps"].as_array().unwrap().len(), 0);
    assert_eq!(
        out["people"],
        serde_json::json!([{ "person": "alex", "key": actor_of(&key(7)) }]),
        "the answer carries the people standing after the ops (svrngs: \
         upstream's carried the roster they were admitted against)"
    );
}

/// A seal renders its prune in the append body: the seal is on disk either
/// way, so a refused prune must never read as a failed seal — and a working
/// one reports what it removed.
#[tokio::test]
async fn a_seal_renders_its_prune() {
    let (_dir, rail) = rail_with(&[("alex", 7)]);
    group_of(&rail, "ledger").unwrap();
    let journal = "ledger";
    for _ in 0..2 {
        let body = serde_json::json!({ "op": "record", "payload": { "amount": 1 } });
        assert_eq!(
            append_act(&rail, journal, body).await.status(),
            StatusCode::OK
        );
    }
    let resp = append_act(&rail, journal, serde_json::json!({ "op": "seal" })).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let out = body_of(resp).await;
    let retired = &out["retired"];
    assert!(
        retired.get("refused").is_none(),
        "a prune this roster admits must not read as refused: {retired}"
    );
    assert_eq!(retired["removed"], 2, "both sealed-over acts retire");
    assert_eq!(retired["kept"], 1, "the seal itself stays");
}

/// The people door answers who stands in the group from the register — the
/// seed's founder and everyone an `Admit` let in — each with their key's hex,
/// and refuses a group this computer does not hold (svrngs, DESIGN §3). The
/// failing input: the people read from the seed alone, which knows only the
/// founder.
#[tokio::test]
async fn the_people_door_answers_the_register_with_keys() {
    let (_dir, rail) = rail_with(&[("alex", 7)]);
    group_of(&rail, "ledger").unwrap();
    let journal = "ledger";
    let admit_bo = crate::RailAct::Admit {
        person: Person::parse("bo").unwrap(),
        key: actor_of(&key(3)),
        invite: None,
    };
    let fixture = svrngs_core::tests_support::Fixture {
        act: admit_bo,
        on_behalf_of: None,
    };
    rail.commit(journal, &fixture).unwrap().unwrap();
    let out = body_of(super::people_answer(&rail, "ledger")).await;
    // In the order of their keys, as the register holds them.
    let mut want = vec![
        serde_json::json!({ "person": "alex", "key": actor_of(&key(7)) }),
        serde_json::json!({ "person": "bo", "key": actor_of(&key(3)) }),
    ];
    want.sort_by_key(|p| p["key"].as_str().unwrap().to_string());
    assert_eq!(out["people"], serde_json::Value::from(want));
    let refused = super::people_answer(&rail, "elsewhere");
    assert_eq!(refused.status(), StatusCode::NOT_FOUND);
}

/// The live lane bounds its memory and reports what it drops: 257 pushes
/// leave 256 payloads and one honest `dropped`.
#[test]
fn the_live_buffer_evicts_the_oldest_and_reports_it() {
    let live = LiveBuffer::default();
    for i in 0..257 {
        live.push("room", format!("p{i}"));
    }
    let (payloads, dropped) = live.drain("room");
    assert_eq!(payloads.len(), 256);
    assert_eq!(dropped, 1);
    assert_eq!(payloads[0], "p1", "the OLDEST payload is the one evicted");
    let (again, dropped) = live.drain("room");
    assert!(
        again.is_empty() && dropped == 0,
        "a drain empties the buffer"
    );
    // Another namespace's buffer is untouched.
    live.push("other", "x".into());
    let (mine, _) = live.drain("room");
    assert!(mine.is_empty());
}

/// The live lane's refusals are named, and its success honestly reports
/// what was fanned out. (svrngs: upstream's daemon carried no fabric, and
/// reported `peers: []`; here the push is offered to each key standing in
/// the group but this node's, and the answer names each and whether it took
/// it. Bo stands in `ledger` with no address and no lookup to find one, so
/// Bo is named, not delivered, with the reason; nothing is buffered here,
/// and a group not held is refused as the append door refuses it.)
#[tokio::test]
async fn live_push_reports_each_peer_it_offered_the_payload() {
    let (_dir, rail) = rail_with(&[("ada", 7), ("bo", 3)]);
    let endpoint = crate::iroh::tests::hermetic_endpoint(7, Vec::new()).await;
    let api = RailApi {
        rail,
        nudge: Arc::new(tokio::sync::Notify::new()),
        live: Arc::new(LiveBuffer::default()),
        transport: Arc::new(crate::iroh::IrohTransport::new(endpoint.clone())),
    };
    let oversize = vec![b'x'; 4097];
    let resp = push_answer(&api, "ledger", oversize.into()).await;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let not_text = vec![0xff, 0xfe, 0x00];
    let resp = push_answer(&api, "ledger", not_text.into()).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let resp = push_answer(&api, "elsewhere", b"a".to_vec().into()).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_of(resp).await["kind"], "not_held");
    let fits = vec![b'a'; 4096];
    let resp = push_answer(&api, "ledger", fits.into()).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let out = body_of(resp).await;
    assert_eq!(out["bytes"], 4096);
    let peers = out["peers"].as_array().unwrap();
    assert_eq!(peers.len(), 1, "this node is not its own peer: {out}");
    assert_eq!(peers[0]["node"], actor_of(&key(3)));
    assert_eq!(peers[0]["name"], "bo");
    assert_eq!(peers[0]["delivered"], false);
    assert!(peers[0]["error"].as_str().is_some_and(|e| !e.is_empty()));
    assert_eq!(out["delivered"], 0);
    let out = body_of(drain_answer(&api.live, "ledger")).await;
    assert_eq!(out["payloads"].as_array().unwrap().len(), 0);
    assert_eq!(out["dropped"], 0);
    endpoint.close().await;
}
