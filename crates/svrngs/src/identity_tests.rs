// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/identity.rs@e1a43bcd2
use super::*;

/// covers: FE-3
#[test]
fn load_or_generate_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let first = load_or_generate_node_key(dir.path()).unwrap();
    let second = load_or_generate_node_key(dir.path()).unwrap();
    assert_eq!(first.to_bytes(), second.to_bytes(), "same key across boots");
    assert!(dir.path().join(NODE_KEY_FILE).exists());
}

/// **The failing input** (ARCH 6): upstream kept a fresh key it could
/// not write and ran as it, so the next run was someone else. A data
/// dir this process may not write to cannot hold the key, so it refuses.
#[cfg(unix)]
#[test]
fn a_key_that_cannot_be_saved_refuses() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    let refused = load_or_generate_node_key(dir.path());
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let err = refused.expect_err("an unkept key refuses");
    assert!(matches!(err, KeyError::Unsaved { .. }), "{err}");
    assert!(
        err.to_string()
            .contains(&dir.path().join(NODE_KEY_FILE).display().to_string()),
        "{err}"
    );
}

#[cfg(unix)]
#[test]
fn key_file_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let _ = load_or_generate_node_key(dir.path()).unwrap();
    let mode = std::fs::metadata(dir.path().join(NODE_KEY_FILE))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

/// **The failing input** (ARCH 6): upstream's fallback answered an
/// unreadable key with a fresh one, so the computer silently became
/// someone new. A key file too short to be a key, and one this process
/// may not read, each refuse, naming the file, and leave it as it was.
#[test]
fn an_unreadable_key_refuses_naming_its_file_and_writes_no_new_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(NODE_KEY_FILE);
    std::fs::write(&path, b"way too short").unwrap();
    let err =
        load_or_generate_node_key(dir.path()).expect_err("a key too short to be a key refuses");
    assert!(
        err.to_string().contains(&path.display().to_string()),
        "{err}"
    );
    assert_eq!(std::fs::read(&path).unwrap(), b"way too short");
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        1,
        "no other file written"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let locked = tempfile::tempdir().unwrap();
        let path = locked.path().join(NODE_KEY_FILE);
        std::fs::write(&path, [7u8; 32]).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
        let refused = load_or_generate_node_key(locked.path());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let err = refused.expect_err("a key this process may not read refuses");
        assert!(
            err.to_string().contains(&path.display().to_string()),
            "{err}"
        );
        assert_eq!(std::fs::read(&path).unwrap(), [7u8; 32]);
    }
}
