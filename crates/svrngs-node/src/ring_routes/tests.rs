// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_routes/tests.rs@e1a43bcd2
//! The checkpoint and live routes (pb-mesh-exit-transport; director
//! phase-b-81 (5)): the successors of the daemon's `ring_checkpoint` unit
//! tests and `ring_live_non_durable`, whose routes are cw-rails' since
//! pb-rails-parity. (svrngs: the checkpoint route did not come over, so
//! neither did its tests; the live route's and the sync route's refusal
//! test did.)

use std::sync::Arc;

use crate::iroh::tests::serve_on_stream;
use crate::iroh::SecretKey;
use crate::ring_sync::tests::serve_at;
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
    rail.journal(NS)
        .unwrap()
        .set_roster(&Roster::new(members))
        .unwrap();
    rail
}

/// The ring routes over `rail` and `live`, served on a plain loopback port
/// with no acceptor in front; the address. (svrngs: upstream registered
/// the two live namespaces here; [`fresh`] holds them, before anything is
/// read from disk.)
async fn serve(rail: Arc<RingRail>, live: Arc<LiveBuffer>) -> std::net::SocketAddr {
    serve_at(router(RingInbound { rail, live })).await
}

fn fresh(dir: &tempfile::TempDir) -> (Arc<RingRail>, Arc<LiveBuffer>) {
    let rail = rail_with_roster(dir.path());
    for ns in [LIVE_NS, OTHER_LIVE_NS] {
        let roster = rail.journal(NS).unwrap().roster_file().unwrap();
        rail.journal(ns).unwrap().set_roster(&roster).unwrap();
    }
    (rail, Arc::new(LiveBuffer::default()))
}

/// One exchange call to `url`, carrying `X-Mesh-Pubkey: <claim>` when given.
async fn ask(url: &str, claim: Option<&str>) -> reqwest::Response {
    let mut req = reqwest::Client::new()
        .post(url)
        .json(&serde_json::json!({ "namespace": NS, "digest": Digest::default(), "ops": [] }));
    if let Some(c) = claim {
        req = req.header("x-mesh-pubkey", c);
    }
    req.send().await.unwrap()
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

    let addr = serve(rail.clone(), Arc::new(LiveBuffer::default())).await;
    let off_stream = ask(&format!("http://{addr}/internal/ring/sync"), Some(&ada)).await;
    assert_eq!(
        off_stream.status(),
        403,
        "a request off the stream is refused"
    );
    assert!(off_stream.text().await.unwrap().contains(NS));

    let (base, _ada) = serve_on_stream(
        router(RingInbound {
            rail: rail.clone(),
            live: Arc::new(LiveBuffer::default()),
        }),
        SecretKey::from_bytes(&key().to_bytes()),
    )
    .await;
    let rostered = ask(&format!("{base}/internal/ring/sync"), None).await;
    assert_eq!(rostered.status(), 200, "the rostered dialer is served");

    let (base, _stranger) = serve_on_stream(
        router(RingInbound {
            rail,
            live: Arc::new(LiveBuffer::default()),
        }),
        SecretKey::from_bytes(&[12u8; 32]),
    )
    .await;
    let claiming = ask(&format!("{base}/internal/ring/sync"), Some(&ada)).await;
    assert_eq!(
        claiming.status(),
        403,
        "a header naming Ada makes no stranger Ada"
    );
    assert!(claiming.text().await.unwrap().contains(NS));
}

fn envelope(namespace: &str, payload: &str) -> String {
    serde_json::json!({ "namespace": namespace, "payload": payload }).to_string()
}

async fn push(addr: std::net::SocketAddr, namespace: &str, payload: &str) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("http://{addr}/internal/ring/live"))
        .body(envelope(namespace, payload))
        .send()
        .await
        .unwrap()
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
    let addr = serve(rail, live.clone()).await;
    let payloads = ["AQID", "BAUG", "BwgJ"];
    for p in payloads {
        let sent = push(addr, LIVE_NS, p).await;
        assert_eq!(sent.status(), 200, "{}", sent.text().await.unwrap());
    }
    assert_eq!(
        live.drain(LIVE_NS),
        (payloads.map(String::from).to_vec(), 0)
    );
    assert_eq!(snapshot(dir.path()), before, "the live lane wrote to disk");
}

/// The daemon's `a_restart_empties_the_buffer`: a process over the same
/// directory drains nothing it did not receive.
#[tokio::test]
async fn a_restart_empties_the_buffer() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, live) = fresh(&dir);
    let addr = serve(rail, live.clone()).await;
    assert_eq!(push(addr, LIVE_NS, "AQID").await.status(), 200);
    let restarted = Arc::new(LiveBuffer::default());
    let _ = serve(rail_with_roster(dir.path()), restarted.clone()).await;
    assert_eq!(restarted.drain(LIVE_NS), (Vec::new(), 0));
    assert_eq!(
        live.drain(LIVE_NS),
        (vec!["AQID".to_string()], 0),
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
    let addr = serve(rail, live.clone()).await;
    for p in ["AQID", "BAUG"] {
        assert_eq!(push(addr, LIVE_NS, p).await.status(), 200);
    }
    assert_eq!(live.drain(OTHER_LIVE_NS), (Vec::new(), 0));
    assert_eq!(
        live.drain(LIVE_NS),
        (vec!["AQID".to_string(), "BAUG".to_string()], 0)
    );
}

/// The daemon's `an_envelope_for_an_unknown_namespace_is_refused`: no live
/// registration holds it, so nobody here could drain it; refused by name
/// and buffered nowhere.
#[tokio::test]
async fn an_envelope_for_an_unknown_namespace_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, live) = fresh(&dir);
    let addr = serve(rail, live.clone()).await;
    let unknown = "nobody-registered-this";
    let resp = push(addr, unknown, "AQID").await;
    assert_eq!(resp.status(), 403);
    let body = resp.text().await.unwrap();
    assert!(
        body.contains(unknown),
        "the refusal names the namespace: {body}"
    );
    assert_eq!(live.drain(unknown), (Vec::new(), 0));
    assert_eq!(live.drain(LIVE_NS), (Vec::new(), 0));
}
