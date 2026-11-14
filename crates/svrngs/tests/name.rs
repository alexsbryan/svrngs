// SPDX-License-Identifier: AGPL-3.0-or-later
//! The name a computer joins as with no `--as` (bash-person (e)): the
//! command's choice and its words, on the wire, no process started. A
//! binary of its own, since it sets `USER` as a person's shell gives it,
//! and no other test here reads it.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use svrngs::cli::{self, Said, Verb};
use svrngs::group;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::{Node, Wire};
use svrngs_node::SigningKey;

/// The account's full name as this system shows it, asked as a person
/// would: `id -F` on macOS, the GECOS field of `getent passwd` up to its
/// first comma elsewhere; empty when it shows none.
fn full_name_shown(user: &str) -> String {
    let (cmd, args) = if cfg!(target_os = "macos") {
        ("id", vec!["-F", user])
    } else {
        ("getent", vec!["passwd", user])
    };
    let out = std::process::Command::new(cmd)
        .args(&args)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    match cfg!(target_os = "macos") {
        _ if !out.status.success() => String::new(),
        true => text,
        false => text
            .split(':')
            .nth(4)
            .unwrap_or("")
            .split(',')
            .next()
            .unwrap()
            .trim()
            .into(),
    }
}

/// What `svrngs new house`, given no `--as`, says of its founder, with
/// `USER` as given: the name `new` takes when none is typed
/// (`cli::user_name`, as cli.rs `serve` takes it), founding house on a
/// computer on the wire, and the first line `new` prints.
async fn founded_as(user: &str) -> String {
    let argv = ["svrngs", "new", "house"].map(String::from);
    let Ok(Verb::New {
        person: None, name, ..
    }) = cli::parse(argv)
    else {
        panic!("`new house` parses with no person");
    };
    std::env::set_var("USER", user);
    let person = cli::user_name().expect("a name, from the account or USER");
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[4; 32]);
    let wire = Wire::default();
    let node = Node::on_wire(&wire, dir.path().into(), key, group::seed_text);
    let node = node.await.unwrap();
    let relay = RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    };
    let (seed, link) = cli::new_group(&node, name, person, None, &[], &relay)
        .await
        .unwrap();
    let made = Said::Made {
        seed,
        link,
        beside: Vec::new(),
    };
    format!("{}\n", made.to_string().lines().next().unwrap())
}

/// **A computer is named as its person reads themselves** (bash-person
/// (e); Demo C's dry run, finding 4; `demo/runs/onboarding/2026-10-11/`,
/// finding 1): with no `--as`, `new` makes its founder the account's full
/// name as the system shows it — "Alex Home" for this host's `alexsbryan`
/// — and an account the system does not know is named by `USER` alone.
/// Red at bash-person's base: "Made house, with you as alexsbryan." On a
/// host whose account's full name is its login, or none, the first half
/// cannot tell the two apart; the second still holds. Watched failing on
/// two plants: `user_name` taking `USER` alone, and `Said::Made`'s words
/// changed (both cli.rs).
#[tokio::test]
async fn new_with_no_name_takes_the_accounts_full_name() {
    let user = std::env::var("USER").expect("the test runs as an account");
    let full = full_name_shown(&user);
    let want = if full.is_empty() { user.clone() } else { full };
    assert_eq!(
        founded_as(&user).await,
        format!("Made house, with you as {want}.\n"),
        "{user}'s full name, as the system shows it"
    );
    assert_eq!(
        founded_as("no-such-account-here").await,
        "Made house, with you as no-such-account-here.\n"
    );
}
