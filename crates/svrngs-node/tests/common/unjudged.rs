//! A claim a test could not judge, said the one way scripts/check.sh
//! `step` reads it: on stderr, and appended to
//! `target/could-not-judge/<step>.step`, `<step>` the `SVRNGS_STEP` check.sh
//! runs it under ("tests" otherwise). A step that exits 0 with that file
//! non-empty is said could-not-judge, never a pass and never a failure
//! (AGENTS.md 6); a panic would be a failure there. So a test that cannot
//! judge its claim says why here and returns.
//!
//! One copy for every test binary: each includes this file by `#[path]`
//! (crates/svrngs/tests/internet.rs among them), so not every binary uses
//! every function.

#![allow(dead_code, reason = "each test binary uses only some of these")]

use std::io::Write;
use std::net::Ipv4Addr;
use std::path::Path;

/// The claim not judged, `why`: said on stderr, and appended where
/// check.sh `step` says it as the step's verdict.
pub fn unjudged(why: &str) {
    eprintln!("could-not-judge: {why}");
    let step = std::env::var("SVRNGS_STEP").unwrap_or_else(|_| "tests".into());
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/could-not-judge");
    std::fs::create_dir_all(&dir).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(format!("{step}.step")))
        .unwrap();
    // One write per line: a binary's tests run on threads at once, and
    // an append's one write is not split by another's.
    file.write_all(format!("{why}\n").as_bytes()).unwrap();
}

/// This host's network, as svrngs reads it (`network::here`, the one
/// decider), or `None`, unjudged with why and the caller's place, on a host
/// whose network cannot be read: the gate's Linux container, its router at
/// times missing from /proc/net/arp.
#[track_caller]
pub fn network() -> Option<svrngs_node::network::Network> {
    let at = std::panic::Location::caller();
    match svrngs_node::network::here().network {
        Ok(network) => Some(network),
        Err(why) => {
            unjudged(&format!(
                "{at}: this host's network cannot be read ({why:?})"
            ));
            None
        }
    }
}

/// This host's private address on the network it is on, as svrngs reads it
/// (`network::here`), or `None`, unjudged with why and the caller's place,
/// on a host with none.
#[track_caller]
pub fn address() -> Option<Ipv4Addr> {
    let at = std::panic::Location::caller();
    let read = svrngs_node::network::here();
    if read.address.is_none() {
        unjudged(&format!(
            "{at}: this host has no private address on a network ({read:?})"
        ));
    }
    read.address
}
