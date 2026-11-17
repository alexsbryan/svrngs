//! One svrngs per data dir (`cli::claim`). A binary of its own: the claim
//! is a `flock` on the lock file's open file, and a child that any other
//! thread of the process spawns is given a copy of it, kept until its exec
//! closes it (std opens every file close-on-exec). So a claim let go while
//! a sibling test spawns is still held, by that child: beside new.rs's
//! help, which spawns `svrngs guide`, "free again" was Busy in 9 runs of
//! 40; traced, each refusal came while that child still ran the test's own
//! image, and the claim was free again just before the spawn returned.

use svrngs::cli::{self, Refusal};

/// A second claim is refused while the first is held, and taken once it
/// is let go.
#[test]
fn a_second_svrngs_on_one_data_dir_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let held = cli::claim(dir.path()).expect("the first claim");
    assert!(matches!(cli::claim(dir.path()), Err(Refusal::Busy(_))));
    drop(held);
    cli::claim(dir.path()).expect("free again");
}
