// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_routes/tests.rs@e1a43bcd2
//! The checkpoint and live routes (pb-mesh-exit-transport; director
//! phase-b-81 (5)): the successors of the daemon's `ring_checkpoint` unit
//! tests and `ring_live_non_durable`, whose routes are cw-rails' since
//! pb-rails-parity. (svrngs: the checkpoint route did not come over, so
//! neither did its tests; the live route's and the sync route's refusal
//! test did.)
//!
//! Each asks the router in-process, as [`IrohAcceptor`](crate::iroh::IrohAcceptor)
//! hands it a request: with the `Dialer` iroh verified as an extension, or
//! with none, as a request off the stream comes. No endpoint, socket or
//! clock (docs/CHECKS.md, unit 4): that the acceptor inserts the verified
//! key, and only that, is tests/accept.rs's, across QUIC.

use std::sync::Arc;

use tower::ServiceExt;

use crate::{Digest, Person, RingRail, Roster, SigningKey};

use super::*;

const NS: &str = "house-expenses";
/// The live namespace a registration holds. (svrngs: a group the rail
/// holds; there are no registrations.)
const LIVE_NS: &str = "ring-doc";
const OTHER_LIVE_NS: &str = "tool-lending";

fn key() -> SigningKey {
    SigningKey::from_bytes(&[11u8; 32])
}

/// A rail signing as [`key`], with [`NS`] rostered to it as Ada.
fn rail_with_roster(root: &std::path::Path) -> Arc<RingRail> {
    let rail = Arc::new(RingRail::new(root, Arc::new(key())));
    let mut members = std::collections::BTreeMap::new();
    members.insert(Person::parse("Ada").unwrap(), vec![crate::actor_of(&key())]);
    rail.set_seed(NS, &Roster::new(members)).unwrap();
    rail
}

/// The ring routes over `rail` and `live`, as `dialer`'s stream reaches
/// them; `None` asks as a request off the stream. (svrngs: upstream served
/// them on a plain loopback port and registered the two live namespaces
/// here; [`fresh`] holds them, before anything is read from disk, and the
/// live route answers only a dialer standing in the group.)
struct Routes {
    router: axum::Router,
    dialer: Option<SigningKey>,
}

impl Routes {
    fn new(inbound: RingInbound, dialer: Option<SigningKey>) -> Self {
        Self {
            router: router(inbound),
            dialer,
        }
    }

    /// POST `body` to `path`, carrying `X-Mesh-Pubkey: <claim>` when given;
    /// the status and the body answered.
    async fn post(&self, path: &str, body: String, claim: Option<&str>) -> (u16, String) {
        let mut req = axum::http::Request::post(path);
        if let Some(c) = claim {
            req = req.header("x-mesh-pubkey", c);
        }
        let mut req = req.body(axum::body::Body::from(body)).unwrap();
        if let Some(key) = &self.dialer {
            req.extensions_mut().insert(crate::iroh::tests::dialer(key));
        }
        let answer = self.router.clone().oneshot(req).await.unwrap();
        let status = answer.status().as_u16();
        let bytes = axum::body::to_bytes(answer.into_body(), usize::MAX)
            .await
            .unwrap();
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }
}

fn inbound(rail: Arc<RingRail>, live: Arc<LiveBuffer>) -> RingInbound {
    RingInbound {
        rail,
        live,
        wake: Default::default(),
    }
}

/// The routes over `rail` and `live`, asked by a dialer holding [`key`],
/// whom the roster names.
fn serve(rail: Arc<RingRail>, live: Arc<LiveBuffer>) -> Routes {
    Routes::new(inbound(rail, live), Some(key()))
}

/// `payload` as it is drained when [`key`], Ada's only key, sent it.
fn from_ada(payload: &str) -> Live {
    let ada = crate::actor_of(&key());
    Live {
        payload: payload.into(),
        key: ada.clone(),
        person_key: ada,
    }
}

fn fresh(dir: &tempfile::TempDir) -> (Arc<RingRail>, Arc<LiveBuffer>) {
    let rail = rail_with_roster(dir.path());
    let mut members = std::collections::BTreeMap::new();
    members.insert(Person::parse("Ada").unwrap(), vec![crate::actor_of(&key())]);
    for ns in [LIVE_NS, OTHER_LIVE_NS] {
        rail.set_seed(ns, &Roster::new(members.clone())).unwrap();
    }
    (rail, Arc::new(LiveBuffer::default()))
}

/// One exchange call, carrying `X-Mesh-Pubkey: <claim>` when given.
async fn ask(routes: &Routes, claim: Option<&str>) -> (u16, String) {
    let body = serde_json::json!({ "namespace": NS, "digest": Digest::default(), "ops": [] });
    routes
        .post("/internal/ring/sync", body.to_string(), claim)
        .await
}

/// The asker is the dialer iroh verified, and nothing else. A request off
/// the stream — no acceptor, so no `Dialer` — is refused by name, even when
/// a header names the rostered key; on the stream the rostered dialer is
/// served, and a stranger whose header names the rostered key is refused.
/// Upstream served the first as a local process and read the asker from the
/// header (its `an_unstamped_local_caller_is_served_and_a_bad_stamp_is_refused`).
/// The failing inputs: a request with no `Dialer` served, and the asker read
/// from the header.
#[tokio::test]
async fn the_asker_is_the_verified_dialer_and_no_header_names_one() {
    let dir = tempfile::tempdir().unwrap();
    let rail = rail_with_roster(dir.path());
    let ada = crate::actor_of(&key());
    let live = || Arc::new(LiveBuffer::default());

    let off_stream = Routes::new(inbound(rail.clone(), live()), None);
    let (status, body) = ask(&off_stream, Some(&ada)).await;
    assert_eq!(status, 403, "a request off the stream is refused");
    assert!(body.contains(NS));

    let rostered = Routes::new(inbound(rail.clone(), live()), Some(key()));
    assert_eq!(
        ask(&rostered, None).await.0,
        200,
        "the rostered dialer is served"
    );

    let stranger = SigningKey::from_bytes(&[12u8; 32]);
    let claiming = Routes::new(inbound(rail, live()), Some(stranger));
    let (status, body) = ask(&claiming, Some(&ada)).await;
    assert_eq!(status, 403, "a header naming Ada makes no stranger Ada");
    assert!(body.contains(NS));
}

fn envelope(namespace: &str, payload: &str) -> String {
    serde_json::json!({ "namespace": namespace, "payload": payload }).to_string()
}

async fn push(routes: &Routes, namespace: &str, payload: &str) -> (u16, String) {
    routes
        .post("/internal/ring/live", envelope(namespace, payload), None)
        .await
}

/// Every file under `dir`, with its length.
fn snapshot(dir: &std::path::Path) -> std::collections::BTreeMap<String, u64> {
    let mut out = std::collections::BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path).unwrap() {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                let rel = entry
                    .path()
                    .strip_prefix(dir)
                    .unwrap()
                    .display()
                    .to_string();
                out.insert(rel, meta.len());
            }
        }
    }
    out
}

/// The daemon's `a_live_payload_touches_nothing_on_disk`: three pushes drain
/// back exactly, in order, and no byte under the rail's directory changed.
#[tokio::test]
async fn a_live_payload_touches_nothing_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, live) = fresh(&dir);
    let before = snapshot(dir.path());
    assert!(!before.is_empty(), "control: the roster file is on disk");
    let routes = serve(rail, live.clone());
    let payloads = ["AQID", "BAUG", "BwgJ"];
    for p in payloads {
        let (status, body) = push(&routes, LIVE_NS, p).await;
        assert_eq!(status, 200, "{body}");
    }
    assert_eq!(live.drain(LIVE_NS), (payloads.map(from_ada).to_vec(), 0));
    assert_eq!(snapshot(dir.path()), before, "the live lane wrote to disk");
}

/// The daemon's `a_restart_empties_the_buffer`: a process over the same
/// directory drains nothing it did not receive.
#[tokio::test]
async fn a_restart_empties_the_buffer() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, live) = fresh(&dir);
    let routes = serve(rail, live.clone());
    assert_eq!(push(&routes, LIVE_NS, "AQID").await.0, 200);
    let restarted = Arc::new(LiveBuffer::default());
    let _restarted = serve(rail_with_roster(dir.path()), restarted.clone());
    assert_eq!(restarted.drain(LIVE_NS), (Vec::new(), 0));
    assert_eq!(
        live.drain(LIVE_NS),
        (vec![from_ada("AQID")], 0),
        "control: the first process held it"
    );
}

/// The buffer half of the daemon's `a_grant_drains_only_its_own_namespace`
/// (the grant half is svrn's, `rail_e2e::live_drain`): what arrived under
/// one namespace is drained under it and no other.
#[tokio::test]
async fn a_drain_takes_only_its_own_namespace() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, live) = fresh(&dir);
    let routes = serve(rail, live.clone());
    for p in ["AQID", "BAUG"] {
        assert_eq!(push(&routes, LIVE_NS, p).await.0, 200);
    }
    assert_eq!(live.drain(OTHER_LIVE_NS), (Vec::new(), 0));
    assert_eq!(
        live.drain(LIVE_NS),
        (vec![from_ada("AQID"), from_ada("BAUG")], 0)
    );
}

/// The daemon's `an_envelope_for_an_unknown_namespace_is_refused`: no live
/// registration holds it, so nobody here could drain it; refused by name
/// and buffered nowhere.
#[tokio::test]
async fn an_envelope_for_an_unknown_namespace_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, live) = fresh(&dir);
    let routes = serve(rail, live.clone());
    let unknown = "nobody-registered-this";
    let (status, body) = push(&routes, unknown, "AQID").await;
    assert_eq!(status, 403);
    assert!(
        body.contains(unknown),
        "the refusal names the namespace: {body}"
    );
    assert_eq!(live.drain(unknown), (Vec::new(), 0));
    assert_eq!(live.drain(LIVE_NS), (Vec::new(), 0));
}

/// The live route asks the sync route's question (svrngs, u3-3): a dialer
/// standing in another group this node holds, but not in the one its
/// envelope names, is refused 403 naming the group, and nothing is
/// buffered; a request off the stream is refused too. Upstream buffered
/// for any member its acceptor admitted, into any registered namespace.
/// The failing input: the standing check skipped.
#[tokio::test]
async fn a_dialer_not_standing_in_the_group_named_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, live) = fresh(&dir);
    let bo = SigningKey::from_bytes(&[12u8; 32]);
    let mut members = std::collections::BTreeMap::new();
    members.insert(Person::parse("Bo").unwrap(), vec![crate::actor_of(&bo)]);
    rail.set_seed("bo-only", &Roster::new(members)).unwrap();
    let as_bo = Routes::new(inbound(rail.clone(), live.clone()), Some(bo));
    let (status, body) = push(&as_bo, LIVE_NS, "AQID").await;
    assert_eq!(status, 403);
    assert!(body.contains(LIVE_NS));
    assert_eq!(live.drain(LIVE_NS), (Vec::new(), 0));
    assert_eq!(
        push(&as_bo, "bo-only", "BAUG").await.0,
        200,
        "control: Bo's own group"
    );

    let off_stream = Routes::new(inbound(rail, live.clone()), None);
    assert_eq!(
        push(&off_stream, LIVE_NS, "AQID").await.0,
        403,
        "a request off the stream is refused"
    );
    assert_eq!(live.drain(LIVE_NS), (Vec::new(), 0));
}

/// A push landing the asker's own `Relay` wakes the round, so the group
/// hears where a newcomer is reached now (relay-in-record): Ada says hers
/// and pushes it to a computer holding the group, whose nudge then holds a
/// permit; pushed again, nothing lands and nothing wakes. The failing
/// input: a route that never wakes, as before. The route wakes before it
/// answers (ring_routes.rs `ring_sync`), so the permit is there once the
/// answer is: each is read with no wait, never on the clock.
#[tokio::test]
async fn a_push_landing_the_askers_own_relay_wakes_the_round() {
    let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = rail_with_roster(a.path());
    let url = svrngs_core::relay::parse("http://127.0.0.1:1/").ok();
    let said = ada.commit(
        NS,
        &svrngs_core::RelayAt {
            key: crate::actor_of(&key()),
            url,
        },
    );
    assert!(
        matches!(said, Ok(Ok(ref s)) if !s.is_empty()),
        "control: Ada's relay is written"
    );
    let issuer = Arc::new(RingRail::new(
        b.path(),
        Arc::new(SigningKey::from_bytes(&[13u8; 32])),
    ));
    let mut members = std::collections::BTreeMap::new();
    members.insert(Person::parse("Ada").unwrap(), vec![crate::actor_of(&key())]);
    issuer.set_seed(NS, &Roster::new(members)).unwrap();
    let wake = Arc::new(tokio::sync::Notify::new());
    let routes = Routes::new(
        RingInbound {
            rail: issuer,
            live: Default::default(),
            wake: wake.clone(),
        },
        Some(key()),
    );
    let body = serde_json::json!({
        "namespace": NS,
        "digest": ada.digest(NS).unwrap(),
        "ops": ada.ops_missing_from_within(NS, &Digest::default(), usize::MAX).unwrap().0,
    })
    .to_string();
    let push = || routes.post("/internal/ring/sync", body.clone(), None);
    let woken = || tokio::time::timeout(std::time::Duration::ZERO, wake.notified());
    assert_eq!(push().await.0, 200);
    assert!(
        woken().await.is_ok(),
        "the asker's own relay landed, and the round was not woken"
    );
    assert_eq!(push().await.0, 200);
    assert!(
        woken().await.is_err(),
        "nothing landed, and the round was woken"
    );
}

/// **Whether a key stands in a group is decided in one place** (#146):
/// the ring's routes serve exactly the keys the register's `names` names
/// (Ada's here, never a stranger's, nobody in a group not held), each with
/// the row `member` answers. Red on the plant the finding names, a change
/// to who stands made in one place: `names` refusing a key with no relay
/// named, which `standing_asker`, deciding inline, did not follow.
#[test]
fn the_routes_serve_the_keys_the_register_names() {
    let dir = tempfile::tempdir().unwrap();
    let rail = rail_with_roster(dir.path());
    let stranger = SigningKey::from_bytes(&[12u8; 32]);
    for ns in [NS, "not-held"] {
        let standing = rail.standing(ns).unwrap();
        for key in [key(), stranger.clone()] {
            let actor = crate::actor_of(&key);
            let dialer = crate::iroh::tests::dialer(&key);
            let served = standing_asker(&rail, Some(dialer), "test", ns);
            assert_eq!(
                served.is_ok(),
                standing.names(&actor),
                "{ns}: served {actor} as the register names it: {served:?}"
            );
        }
    }
}
