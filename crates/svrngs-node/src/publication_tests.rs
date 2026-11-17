// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// A computer whose data dir has no `join-resolver` file publishes to
/// svrngs's own, as a page needs no host of ours set (DESIGN §3, the
/// short-invitation publication); one that names another uses it as
/// named; and a file that is there but cannot be read is said, never
/// passed over for ours.
#[test]
fn a_computer_naming_no_resolver_publishes_to_svrngs_s_own() {
    let absent = std::io::Error::from(std::io::ErrorKind::NotFound);
    assert_eq!(resolver(Err(absent)).unwrap(), "https://join.svrnme.sh/");
    let named = "http://127.0.0.1:9/\n";
    assert_eq!(resolver(Ok(named.into())).unwrap(), named);
    let unreadable = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert!(resolver(Err(unreadable)).is_err());
}
