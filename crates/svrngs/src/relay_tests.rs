// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`super::verdict`]: whom the relay lets through, by the read of where
//! a key stands.

use super::*;

/// Every line the test traces on its own thread: a test's sink, which a
/// subscriber's writer needs `Sync`.
#[expect(clippy::disallowed_types, reason = "a test's trace sink")]
static TRACES: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());

struct Kept;

impl std::io::Write for Kept {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        TRACES.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// **A read that failed is refused as that** (ralph/REVIEW_FINDINGS.md
/// #129): the read of where a key stands panics, and the relay refuses the
/// key, traced as the record not read, never as the key standing in no
/// group; a read that finds it in none is refused as that. Planted: the
/// failed read taken as `None`, as before.
#[tokio::test]
async fn a_read_that_panicked_is_refused_as_that_never_as_no_group() {
    let kept = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("relay=debug"))
        .with_writer(|| Kept)
        .with_ansi(false)
        .finish();
    let _kept = tracing::subscriber::set_default(kept);
    let panicked = tokio::task::spawn_blocking(|| -> Option<(String, Member)> {
        panic!("planted: the record's read panicked")
    });
    let refused = Access::Deny { reason: None };
    assert_eq!(verdict("a1", panicked.await), refused);
    assert_eq!(verdict("b2", Ok(None)), refused);
    let traced = String::from_utf8_lossy(&TRACES.lock().unwrap()).into_owned();
    let of = |key: &str| -> Vec<String> {
        (traced.lines())
            .filter(|l| l.contains(&format!("key=\"{key}\"")))
            .map(|l| {
                l.split_once("relay: ")
                    .map_or(l, |(_, said)| said)
                    .to_string()
            })
            .collect()
    };
    let (a1, b2) = (of("a1"), of("b2"));
    assert!(
        a1.len() == 1 && a1[0].contains("relay: refused — the record could not be read"),
        "the panicked read: {a1:#?}"
    );
    assert!(
        b2.len() == 1 && b2[0].contains("relay: refused — the key stands in no group held here"),
        "the read finding none: {b2:#?}"
    );
}
