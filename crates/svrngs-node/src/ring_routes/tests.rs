// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_routes/tests.rs@e1a43bcd2
//! The checkpoint and live routes (pb-mesh-exit-transport; director
//! phase-b-81 (5)): the successors of the daemon's `ring_checkpoint` unit
//! tests and `ring_live_non_durable`, whose routes are cw-rails' since
//! pb-rails-parity. (svrngs: those routes did not come over, so neither did
//! their tests; the sync route's refusal test did.)

use std::sync::Arc;

use crate::{Digest, Person, RingRail, Roster, SigningKey};

use super::*;

const NS: &str = "house-expenses";

fn key() -> SigningKey {
    SigningKey::from_bytes(&[11u8; 32])
}

/// A rail signing as [`key`], with [`NS`] rostered to it as Ada.
fn rail_with_roster(root: &std::path::Path) -> Arc<RingRail> {
    let rail = Arc::new(RingRail::new(root, Arc::new(key())));
    let mut members = std::collections::BTreeMap::new();
    members.insert(Person::from("Ada"), vec![crate::actor_of(&key())]);
    rail.journal(NS)
        .unwrap()
        .set_roster(&Roster::new(members))
        .unwrap();
    rail
}

/// The ring route over `rail`, served; the address.
async fn serve(rail: Arc<RingRail>) -> std::net::SocketAddr {
    let app = router(RingInbound { rail });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await;
    });
    addr
}

/// A caller with no stamped key reached this loopback port without the
/// acceptor in front — a process on this machine — and is served; a stamp
/// that is not a key is refused by name. The acceptor's half (a stranger
/// never reaches `/internal/ring`) is commonwealth-media
/// `origins::cwth_http_forwards_by_registered_prefix`. Successor of the
/// daemon's `rail_e2e/roster_refusal` pair, whose per-process mark retired
/// with the daemon's acceptor.
#[tokio::test]
async fn an_unstamped_local_caller_is_served_and_a_bad_stamp_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let rail = rail_with_roster(dir.path());
    let addr = serve(rail).await;
    let ask = |stamp: Option<&str>| {
        let mut req = reqwest::Client::new()
            .post(format!("http://{addr}/internal/ring/sync"))
            .json(&serde_json::json!({ "namespace": NS, "digest": Digest::default(), "ops": [] }));
        if let Some(s) = stamp {
            req = req.header(PUBKEY_HEADER, s);
        }
        req.send()
    };
    assert_eq!(ask(None).await.unwrap().status(), 200);
    let refused = ask(Some("not-a-key")).await.unwrap();
    assert_eq!(refused.status(), 403);
    assert!(refused.text().await.unwrap().contains(NS));
}
