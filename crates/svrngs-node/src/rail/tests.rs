// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/rail/tests.rs@e1a43bcd2
//! The rail doors' tests. (svrngs: upstream's fixtures built a mesh whose
//! membership was every ring's roster; here a ring's roster is its seed
//! file, which the register reads, DESIGN §8, U1's inventory. The roster
//! derivation's tests, the two-keys census, the sync doors' and the live
//! lane's did not come over with what they test.)

use std::sync::Arc;

use axum::http::StatusCode;

use crate::{
    actor_of, AttestRefusal, GuestAttestation, Person, RingRail, RingSigner, Roster, SigningKey,
};

use super::{append_act, journal_of};

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
        rail.journal("ledger").unwrap().set_roster(&roster).unwrap();
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
/// choose for itself is this door's, not the body's.
#[tokio::test]
async fn append_signs_as_the_node_and_assigns_the_fields() {
    let (_dir, rail) = rail_with(&[("alex", 7)]);
    let journal = journal_of(&rail, "ledger").unwrap();
    let body = serde_json::json!({
        "op": "record",
        "payload": { "amount": 4 },
        // Every one of these is the server's to assign; a body that could
        // choose them could write as somebody else.
        "seq": 999,
        "id": "forged",
        "ts_unix": 1,
        "on_behalf_of": "beefy",
    });
    let resp = append_act(&rail, &journal, body).await;
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

/// A roster of this node (key 7) and one other member, "halo" (key 3), who
/// signs the guest door's attestations; key 99 is claimed by nobody.
fn attested(seed: u8, namespace: &str, expires_at: i64) -> serde_json::Value {
    let a = GuestAttestation::sign(&key(seed), "guest-ana", namespace, expires_at);
    serde_json::json!({
        "op": "record",
        "payload": { "amount": 4 },
        "attestation": a,
    })
}

const GUESTS: &[(&str, u8)] = &[("alex", 7), ("halo", 3)];

const LATER: i64 = 4_000_000_000;

/// A roster member's attestation lands the act signed by the node and
/// attributed to the guest.
#[tokio::test]
async fn a_valid_attestation_lands_attributed_to_the_guest() {
    let (_dir, rail) = rail_with(GUESTS);
    let journal = journal_of(&rail, "ledger").unwrap();
    let resp = append_act(&rail, &journal, attested(3, "ledger", LATER)).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let out = body_of(resp).await;
    assert_eq!(out["actor"], RingSigner::actor(&key(7)), "the node signs");
    let (ops, _) = journal.read().unwrap();
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].kind.on_behalf_of.as_deref(), Some("guest-ana"));
}

/// Forged, expired and foreign-key attestations are each a 403 naming the
/// refusal, and the journal is untouched.
#[tokio::test]
async fn a_refused_attestation_names_its_refusal_and_writes_nothing() {
    let (_dir, rail) = rail_with(GUESTS);
    let journal = journal_of(&rail, "ledger").unwrap();
    let mut forged = attested(3, "ledger", LATER);
    forged["attestation"]["name"] = "guest-anb".into();
    let cases = [
        (forged, AttestRefusal::Forged),
        (attested(3, "ledger", 1), AttestRefusal::Expired),
        (
            attested(99, "ledger", LATER),
            AttestRefusal::SignerNotInRoster,
        ),
        (attested(3, "lending", LATER), AttestRefusal::WrongNamespace),
    ];
    for (body, want) in cases {
        let resp = append_act(&rail, &journal, body).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN, "{want:?}");
        assert_eq!(body_of(resp).await["kind"], want.name());
    }
    let (ops, _) = journal.read().unwrap();
    assert!(ops.is_empty(), "a refused attestation writes nothing");
}

/// An unstamped append is unchanged: signed as the node, no name on it.
#[tokio::test]
async fn an_unstamped_append_carries_no_name() {
    let (_dir, rail) = rail_with(GUESTS);
    let journal = journal_of(&rail, "ledger").unwrap();
    let body = serde_json::json!({ "op": "record", "payload": { "amount": 4 } });
    let resp = append_act(&rail, &journal, body).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let (ops, _) = journal.read().unwrap();
    assert_eq!(ops[0].kind.on_behalf_of, None);
}

/// Nobody in the roster → every op this node writes would be unreadable to
/// every peer, and the refusal is typed. (svrngs: upstream's sentence named
/// its join verb, chosen for a derived roster; with no seed the rail's own
/// sentence is the one, and the test asserts the type.)
#[tokio::test]
async fn append_outside_the_mesh_refuses_naming_the_join_verb() {
    let (_dir, rail) = rail_with(&[]);
    let journal = journal_of(&rail, "ledger").unwrap();
    let body = serde_json::json!({ "op": "record", "payload": { "amount": 4 } });
    let resp = append_act(&rail, &journal, body).await;
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let out = body_of(resp).await;
    assert_eq!(out["kind"], "not_in_roster", "{out}");
}

/// The namespace is never guessed: absent is a 400 that names the fix, and
/// a name the journal would refuse as a directory is the rail's own 400.
#[tokio::test]
async fn the_namespace_must_be_a_name_the_journal_can_open() {
    let (_dir, rail) = rail_with(&[("alex", 7)]);
    let q = super::RailQuery { namespace: None };
    let err = super::namespace_of(&q).unwrap_err();
    assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    let refused = journal_of(&rail, "Not A Namespace");
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
    let journal = journal_of(&rail, "ledger").unwrap();
    let body = serde_json::json!({ "op": "record", "payload": { "amount": 4 } });
    let resp = append_act(&rail, &journal, body).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let resp = super::log_answer(&rail, &journal).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let out = body_of(resp).await;
    assert_eq!(out["complete"], true);
    // `held` is the journal-line count, refused lines included: one line on
    // disk, one op admitted, so nothing was refused.
    assert_eq!(out["held"], 1);
    assert_eq!(out["ops"].as_array().unwrap().len(), 1);
    assert_eq!(out["ops"][0]["person"], "alex");
    assert_eq!(out["gaps"].as_array().unwrap().len(), 0);
    assert!(
        out["roster"]["members"]["alex"].is_array(),
        "the answer carries the roster the ops were admitted against"
    );
}

/// A seal renders its prune in the append body: the seal is on disk either
/// way, so a refused prune must never read as a failed seal — and a working
/// one reports what it removed.
#[tokio::test]
async fn a_seal_renders_its_prune() {
    let (_dir, rail) = rail_with(&[("alex", 7)]);
    let journal = journal_of(&rail, "ledger").unwrap();
    for _ in 0..2 {
        let body = serde_json::json!({ "op": "record", "payload": { "amount": 1 } });
        assert_eq!(
            append_act(&rail, &journal, body).await.status(),
            StatusCode::OK
        );
    }
    let resp = append_act(&rail, &journal, serde_json::json!({ "op": "seal" })).await;
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
