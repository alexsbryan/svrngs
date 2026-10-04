// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-media/src/declared.rs@e1a43bcd2
use super::*;

#[test]
fn round_trips_a_declaration_and_trims() {
    let dir = tempfile::tempdir().unwrap();
    write_declared_in(dir.path(), "X-Emby-Token", "  abc123\n").unwrap();
    assert_eq!(
        read_declared_in(dir.path()),
        vec![("x-emby-token".to_string(), "abc123".to_string())]
    );
}

/// Trimming is for the trailing newline an editor adds, not for the
/// credential's own shape. Jellyfin 12 wants `MediaBrowser Token="<key>"`
/// in one header value, so an inner space and two quotes have to survive
/// the store or the declaration is unusable on every current Jellyfin.
#[test]
fn a_value_with_inner_spaces_and_quotes_survives_the_store() {
    let dir = tempfile::tempdir().unwrap();
    let credential = r#"MediaBrowser Token="a-key-1234""#;
    write_declared_in(dir.path(), "Authorization", &format!("{credential}\n")).unwrap();
    assert_eq!(
        read_declared_in(dir.path()),
        vec![("authorization".to_string(), credential.to_string())]
    );
}

#[test]
fn an_absent_directory_is_empty_not_an_error() {
    assert!(read_declared_in(Path::new("/nonexistent/media/secrets")).is_empty());
}

#[test]
fn an_empty_value_deletes_the_declaration() {
    let dir = tempfile::tempdir().unwrap();
    write_declared_in(dir.path(), "authorization", "x").unwrap();
    write_declared_in(dir.path(), "authorization", "  ").unwrap();
    assert!(read_declared_in(dir.path()).is_empty());
    // Idempotent: deleting what is already gone is not a failure.
    write_declared_in(dir.path(), "authorization", "").unwrap();
}

/// The filename reaches a request head, so it must not be able to name a
/// path or end a line. Rejected, never repaired.
#[test]
fn a_name_that_could_escape_the_directory_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    for hostile in [
        "../../etc/passwd",
        "a/b",
        "x\r\ny",
        "has space",
        "",
        "..",
        ".",
    ] {
        assert!(
            write_declared_in(dir.path(), hostile, "v").is_err(),
            "{hostile:?} must be refused"
        );
    }
    assert!(read_declared_in(dir.path()).is_empty());
}

/// A file dropped in by hand with an unusable name is skipped rather than
/// poisoning every other declaration on the node.
#[test]
fn a_hand_dropped_file_with_a_bad_name_is_skipped_not_fatal() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("not a header"), "v").unwrap();
    write_declared_in(dir.path(), "x-emby-token", "good").unwrap();
    assert_eq!(
        read_declared_in(dir.path()),
        vec![("x-emby-token".to_string(), "good".to_string())]
    );
}

#[cfg(unix)]
#[test]
fn the_declaration_file_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    write_declared_in(dir.path(), "x-emby-token", "abc").unwrap();
    let mode = std::fs::metadata(dir.path().join("x-emby-token"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

/// svrngs: the directory is owner-only too, and so is a file that was
/// already there with wider modes. The failing input: a secret written
/// over a hand-dropped 0644 file, left 0644.
#[cfg(unix)]
#[test]
fn the_declaration_dir_and_a_rewritten_file_are_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("secrets");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("authorization");
    std::fs::write(&file, "old").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    write_declared_in(&dir, "authorization", "new").unwrap();
    let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&file), 0o600);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "new");
}

#[test]
fn order_is_stable_so_a_head_is_reproducible() {
    let dir = tempfile::tempdir().unwrap();
    write_declared_in(dir.path(), "z-last", "1").unwrap();
    write_declared_in(dir.path(), "a-first", "2").unwrap();
    let names: Vec<String> = read_declared_in(dir.path())
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    assert_eq!(names, vec!["a-first".to_string(), "z-last".to_string()]);
}
