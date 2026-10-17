// SPDX-License-Identifier: AGPL-3.0-or-later
//! The name a computer joins as with no `--as` (bash-person (e)), asked of
//! the program as a person runs it. A binary of its own: its children,
//! spawned beside an in-process `cli::claim`, would hold that test's lock
//! between their fork and exec (`new.rs`'s
//! `a_second_svrngs_on_one_data_dir_is_refused` failed 3 of 15 runs with
//! this test beside it, 0 of 15 without).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
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

/// The name `svrngs new house`, given no `--as`, makes its founder, with
/// `USER` as given and a home and data of its own.
fn founded_as(user: &str) -> String {
    use std::io::BufRead;
    let home = tempfile::tempdir().unwrap();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_svrngs"))
        .args(["new", "house"])
        .env("HOME", home.path())
        .env("SVRNGS_DIR", home.path().join(".svrngs"))
        .env("USER", user)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let mut first = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut first)
        .unwrap();
    let _ = child.kill();
    let _ = child.wait();
    first
}

/// **A computer is named as its person reads themselves** (bash-person
/// (e); Demo C's dry run, finding 4; `demo/runs/onboarding/2026-10-11/`,
/// finding 1): with no `--as`, `new` makes its founder the account's full
/// name as the system shows it — "Alex Home" for this host's `alexsbryan`
/// — and an account the system does not know is named by `USER` alone.
/// Red at bash-person's base: "Made house, with you as alexsbryan." On a
/// host whose account's full name is its login, or none, the first half
/// cannot tell the two apart; the second still holds.
#[test]
fn new_with_no_name_takes_the_accounts_full_name() {
    let user = std::env::var("USER").expect("the test runs as an account");
    let full = full_name_shown(&user);
    let want = if full.is_empty() { user.clone() } else { full };
    assert_eq!(
        founded_as(&user),
        format!("Made house, with you as {want}.\n"),
        "{user}'s full name, as the system shows it"
    );
    assert_eq!(
        founded_as("no-such-account-here"),
        "Made house, with you as no-such-account-here.\n"
    );
}
