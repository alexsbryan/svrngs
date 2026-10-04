// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/ring_routes/tests.rs@e1a43bcd2
//! The checkpoint and live routes (pb-mesh-exit-transport; director
//! phase-b-81 (5)): the successors of the daemon's `ring_checkpoint` unit
//! tests and `ring_live_non_durable`, whose routes are cw-rails' since
//! pb-rails-parity. (svrngs: those routes did not come over, so neither did
//! their tests; the sync route's refusal test did.)

use std::sync::Arc;

use crate::iroh::tests::serve_on_stream;
use crate::iroh::SecretKey;
use crate::ring_sync::tests::serve_at;
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
    members.insert(Person::parse("Ada").unwrap(), vec![crate::actor_of(&key())]);
    rail.journal(NS)
        .unwrap()
        .set_roster(&Roster::new(members))
        .unwrap();
    rail
}

/// The ring route over `rail`, served on a plain loopback port with no
/// acceptor in front; the address.
async fn serve(rail: Arc<RingRail>) -> std::net::SocketAddr {
    serve_at(router(RingInbound { rail })).await
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

    let addr = serve(rail.clone()).await;
    let off_stream = ask(&format!("http://{addr}/internal/ring/sync"), Some(&ada)).await;
    assert_eq!(
        off_stream.status(),
        403,
        "a request off the stream is refused"
    );
    assert!(off_stream.text().await.unwrap().contains(NS));

    let (base, _ada) = serve_on_stream(
        router(RingInbound { rail: rail.clone() }),
        SecretKey::from_bytes(&key().to_bytes()),
    )
    .await;
    let rostered = ask(&format!("{base}/internal/ring/sync"), None).await;
    assert_eq!(rostered.status(), 200, "the rostered dialer is served");

    let (base, _stranger) = serve_on_stream(
        router(RingInbound { rail }),
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
