#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]
// SPDX-License-Identifier: AGPL-3.0-or-later
#[test]
fn the_client_builds() {
    super::client().expect("a client");
}
