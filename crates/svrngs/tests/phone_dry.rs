//! Sam's computer for U4's dry run (`demo/runs/u4/2026-10-05/dry.sh`),
//! reached only through iroh's relay on 127.0.0.1, plain HTTP: what the
//! `svrngs` command does for `new --origin` and `share`, with the relay the
//! command cannot be pointed at. The dry run's other pass, through iroh's
//! public relay, runs the command itself. Ignored: `scripts/check.sh` never
//! runs it; dry.sh does, by name:
//!
//! ```text
//! DRY_ORIGIN=http://127.0.0.1:<page>/ DRY_LEND=<port> DRY_LIMIT=<n> DRY_DIR=<dir> \
//!   cargo test -p svrngs --test phone_dry -- --ignored --nocapture
//! ```
//!
//! It makes the group `House` as Sam, its links on `DRY_ORIGIN`, lends the
//! service on `DRY_LEND` as `film` to `DRY_LIMIT` people at once, writes the https link to
//! `DRY_DIR/link`, and runs until `DRY_DIR/stop` exists.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(dead_code, unused_macros)]
mod harness;

use std::num::NonZeroU16;
use std::path::PathBuf;
use std::time::Duration;

use svrngs::cli;
use svrngs::group::{self, Name};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend::{self, Kind, Lend, LendName};
use svrngs_node::link::Origin;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

fn var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is set by dry.sh"))
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "U4's dry run, run by demo/runs/u4/2026-10-05/dry.sh"]
async fn sam_through_a_relay_on_this_computer() {
    let dir = PathBuf::from(var("DRY_DIR"));
    let origin = Origin::parse(&var("DRY_ORIGIN")).expect("DRY_ORIGIN is a page address");
    let port: NonZeroU16 = var("DRY_LEND").parse().expect("DRY_LEND is a port");
    let limit: NonZeroU16 = var("DRY_LIMIT")
        .parse()
        .expect("DRY_LIMIT is how many at once");
    let relay = harness::relay().await;
    let ours = RelayConfig::from_parts(vec![harness::relay_url(&relay)], Some("none"))
        .unwrap()
        .on_loopback_only();
    let sam_dir = dir.join("sam");
    std::fs::create_dir_all(&sam_dir).unwrap();
    let sam = Node::start(
        sam_dir.clone(),
        SigningKey::from_bytes(&[1; 32]),
        &ours,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("Sam's computer starts");
    tokio::time::timeout(Duration::from_secs(20), sam.endpoint().online())
        .await
        .expect("Sam's computer reaches the relay on 127.0.0.1");
    let film = Lend {
        name: LendName::parse("film").unwrap(),
        port,
        kind: Kind::parse("film").unwrap(),
        limit,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
    };
    lend::declare(&sam_dir, &film, None).unwrap();
    let (_, printed) = cli::new_group(
        &sam,
        Name::parse("House").unwrap(),
        Name::parse("Sam").unwrap(),
        Some(&svrngs::cli::Pages {
            page: origin.clone(),
            apps: svrngs::cli::Apps::OwnAddress,
        }),
        &ours,
    )
    .await
    .expect("Sam makes House, its links on the page's origin");
    let link = printed.https.expect("an https link on the origin");
    std::fs::write(dir.join("link"), &link).unwrap();
    eprintln!("the link printed: {link}");
    while !dir.join("stop").exists() {
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    drop(sam);
}
