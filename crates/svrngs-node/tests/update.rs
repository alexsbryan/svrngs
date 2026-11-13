//! The home page’s update (PLAN §1, the update row): the page’s
//! `SHA256SUMS` is the one decider of its release, `release` in the data
//! dir the one record of what this computer installed, and only a tarball
//! that matches the page’s sum and unpacks to a regular file replaces the
//! binary. Each refusal keeps the running bytes.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::HashMap;
use std::path::Path;

use sha2::{Digest, Sha256};
use svrngs_node::invite::ORIGIN_FILE;
use svrngs_node::update::{
    apply_at, check, check_by, fetch, Check, Get, Outcome, UpdateError, RELEASE_FILE,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The tarball’s name, as `install.sh` and `scripts/build-release.sh`
/// spell it for this computer.
fn tarball_name() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    format!("svrngs-{os}-{}.tar.gz", std::env::consts::ARCH)
}

/// A page on 127.0.0.1 serving `files` at their paths, 404 else.
async fn host(files: HashMap<String, Vec<u8>>) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let files = files.clone();
            tokio::spawn(async move {
                let mut head = vec![0u8; 8192];
                let n = sock.read(&mut head).await.unwrap_or(0);
                let head = String::from_utf8_lossy(&head[..n]);
                let path = head.split(' ').nth(1).unwrap_or("/");
                let path = path.split(['?', '#']).next().unwrap_or(path);
                let path = path.trim_start_matches('/');
                let reply = match files.get(path) {
                    Some(body) => {
                        let mut reply = format!(
                            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                            body.len()
                        )
                        .into_bytes();
                        reply.extend(body);
                        reply
                    }
                    None => {
                        b"HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                            .to_vec()
                    }
                };
                let _ = sock.write_all(&reply).await;
            });
        }
    });
    port
}

/// The page this computer was installed from.
fn keep_origin(dir: &Path, port: u16) {
    std::fs::write(dir.join(ORIGIN_FILE), format!("http://127.0.0.1:{port}/")).unwrap();
}

/// `bytes` in a tarball named `name`, as the release’s is.
fn tarball(dir: &Path, name: &str, bytes: &[u8]) -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    std::fs::write(src.path().join("svrngs"), bytes).unwrap();
    let at = dir.join(name);
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&at)
        .arg("-C")
        .arg(src.path())
        .arg("svrngs")
        .status()
        .unwrap();
    assert!(made.success());
    std::fs::read(at).unwrap()
}

/// A tarball whose `svrngs` is a symlink, never a file this computer runs.
fn symlink_tarball(dir: &Path, name: &str) -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink("/etc/passwd", src.path().join("svrngs")).unwrap();
    let at = dir.join(name);
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&at)
        .arg("-C")
        .arg(src.path())
        .arg("svrngs")
        .status()
        .unwrap();
    assert!(made.success());
    std::fs::read(at).unwrap()
}

/// `SHA256SUMS`’s line for `name`, as the release writes it.
fn sums(name: &str, tarball: &[u8]) -> Vec<u8> {
    format!("{}  {name}\n", hex::encode(Sha256::digest(tarball))).into_bytes()
}

fn exe(dir: &Path) -> std::path::PathBuf {
    dir.join("svrngs")
}

#[tokio::test]
async fn a_page_s_release_is_checked_and_its_verified_bytes_replace_the_binary() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let port = host(HashMap::from([
        ("get/SHA256SUMS".to_string(), sums(&name, &tarball)),
        (format!("get/{name}"), tarball.clone()),
    ]))
    .await;
    keep_origin(dir.path(), port);
    std::fs::write(exe(dir.path()), b"the old svrngs").unwrap();

    assert_eq!(check(dir.path()).await.unwrap(), Check::Available);
    assert_eq!(
        apply_at(dir.path(), &exe(dir.path()), fetch).await.unwrap(),
        Outcome::Updated
    );
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the new svrngs");
    assert_eq!(
        std::fs::read_to_string(dir.path().join(RELEASE_FILE)).unwrap(),
        hex::encode(Sha256::digest(&tarball))
    );
    assert_eq!(check(dir.path()).await.unwrap(), Check::Current);
}

#[tokio::test]
async fn a_tarball_that_is_not_the_page_s_is_refused_and_the_binary_stays() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let mut sums = sums(&name, &tarball);
    sums[0] = if sums[0] == b'0' { b'1' } else { b'0' };
    let port = host(HashMap::from([
        ("get/SHA256SUMS".to_string(), sums),
        (format!("get/{name}"), tarball),
    ]))
    .await;
    keep_origin(dir.path(), port);
    std::fs::write(exe(dir.path()), b"the old svrngs").unwrap();

    assert!(matches!(
        apply_at(dir.path(), &exe(dir.path()), fetch).await,
        Err(UpdateError::Mismatch)
    ));
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");
    assert!(!dir.path().join(RELEASE_FILE).exists());
}

#[tokio::test]
async fn a_tarball_whose_svrngs_is_a_symlink_is_refused_and_the_binary_stays() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = symlink_tarball(dir.path(), &name);
    let port = host(HashMap::from([
        ("get/SHA256SUMS".to_string(), sums(&name, &tarball)),
        (format!("get/{name}"), tarball),
    ]))
    .await;
    keep_origin(dir.path(), port);
    std::fs::write(exe(dir.path()), b"the old svrngs").unwrap();

    assert!(matches!(
        apply_at(dir.path(), &exe(dir.path()), fetch).await,
        Err(UpdateError::Refused(_))
    ));
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");
}

#[tokio::test]
async fn a_page_with_no_line_for_this_computer_is_not_judged_as_current() {
    let dir = tempfile::tempdir().unwrap();
    let port = host(HashMap::from([(
        "get/SHA256SUMS".to_string(),
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  svrngs-other.tar.gz\n"
            .to_vec(),
    )]))
    .await;
    keep_origin(dir.path(), port);
    assert!(matches!(
        check(dir.path()).await,
        Err(UpdateError::NotListed)
    ));
}

#[tokio::test]
async fn a_page_that_does_not_answer_is_not_judged_as_current() {
    let dir = tempfile::tempdir().unwrap();
    let free = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = free.local_addr().unwrap().port();
    drop(free);
    keep_origin(dir.path(), port);
    assert!(matches!(
        check(dir.path()).await,
        Err(UpdateError::CouldNotJudge(_))
    ));
}

#[tokio::test]
async fn an_update_already_running_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let held = std::fs::File::create(dir.path().join("update.lock")).unwrap();
    held.try_lock().unwrap();
    std::fs::write(exe(dir.path()), b"the old svrngs").unwrap();

    assert!(matches!(
        apply_at(dir.path(), &exe(dir.path()), fetch).await,
        Err(UpdateError::Busy)
    ));
}

// ── The same claims with no socket: the page's files read by a `Get` ──

/// The page this computer was installed from, kept as `install.sh` keeps
/// it, at an address no test asks.
const PAGE: &str = "http://page.invalid/";

/// A page with no socket under it: `files` at their paths under [`PAGE`],
/// any other answered as [`fetch`] answers a page with no such file.
fn page(files: &HashMap<String, Vec<u8>>) -> impl Get + '_ {
    async move |url: &str, _max: usize| {
        let path = url.strip_prefix(PAGE).unwrap_or(url);
        let file = files.get(path).cloned();
        file.ok_or_else(|| UpdateError::CouldNotJudge(format!("the page answered 404 for {url}")))
    }
}

/// `dir`'s computer, installed from [`PAGE`], running `the old svrngs`.
fn installed(dir: &Path) {
    std::fs::write(dir.join(ORIGIN_FILE), PAGE).unwrap();
    std::fs::write(exe(dir), b"the old svrngs").unwrap();
}

/// **A page's release is checked and its verified bytes replace the
/// binary, with no socket**: as
/// `a_page_s_release_is_checked_and_its_verified_bytes_replace_the_binary`,
/// the page's `get/SHA256SUMS` and tarball read by a [`Get`] (update.rs
/// `check_by`, `apply_at`). Planted: the release file not written after
/// the rename (`check` then says Available); the tarball's hash compared
/// to the installed record, not the page's sum (Mismatch).
#[tokio::test]
async fn a_page_s_release_replaces_the_binary_read_with_no_socket() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let files = HashMap::from([
        ("get/SHA256SUMS".to_string(), sums(&name, &tarball)),
        (format!("get/{name}"), tarball.clone()),
    ]);
    installed(dir.path());

    assert_eq!(
        check_by(dir.path(), page(&files)).await.unwrap(),
        Check::Available
    );
    let applied = apply_at(dir.path(), &exe(dir.path()), page(&files)).await;
    assert_eq!(applied.unwrap(), Outcome::Updated);
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the new svrngs");
    assert_eq!(
        std::fs::read_to_string(dir.path().join(RELEASE_FILE)).unwrap(),
        hex::encode(Sha256::digest(&tarball))
    );
    assert_eq!(
        check_by(dir.path(), page(&files)).await.unwrap(),
        Check::Current
    );
}

/// **A tarball that is not the page's is refused, with no socket**: the
/// page's sum one digit off; Mismatch, the binary stays, no release file.
/// Planted: the hash check skipped (update.rs `apply_at`), the bytes put
/// in place; the sum's first digit not compared.
#[tokio::test]
async fn a_tarball_not_the_page_s_is_refused_read_with_no_socket() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let mut sums = sums(&name, &tarball);
    sums[0] = if sums[0] == b'0' { b'1' } else { b'0' };
    let files = HashMap::from([
        ("get/SHA256SUMS".to_string(), sums),
        (format!("get/{name}"), tarball),
    ]);
    installed(dir.path());

    let applied = apply_at(dir.path(), &exe(dir.path()), page(&files)).await;
    assert!(matches!(applied, Err(UpdateError::Mismatch)), "{applied:?}");
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");
    assert!(!dir.path().join(RELEASE_FILE).exists());
}

/// **A tarball whose svrngs is a symlink is refused, with no socket**:
/// Refused, the binary stays. Planted: the regular-file check skipped
/// (update.rs `apply_at`); the link followed (`metadata` for
/// `symlink_metadata`), its target put in place.
#[tokio::test]
async fn a_tarball_whose_svrngs_is_a_symlink_is_refused_read_with_no_socket() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = symlink_tarball(dir.path(), &name);
    let files = HashMap::from([
        ("get/SHA256SUMS".to_string(), sums(&name, &tarball)),
        (format!("get/{name}"), tarball),
    ]);
    installed(dir.path());

    let applied = apply_at(dir.path(), &exe(dir.path()), page(&files)).await;
    assert!(
        matches!(applied, Err(UpdateError::Refused(_))),
        "{applied:?}"
    );
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");
}

/// **A page with no line for this computer is not judged current, with no
/// socket**: NotListed. Planted: any tarball's line taken (update.rs
/// `wanted`); no line read as a sum of zeros.
#[tokio::test]
async fn a_page_with_no_line_for_this_computer_is_not_current_read_with_no_socket() {
    let dir = tempfile::tempdir().unwrap();
    let other =
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  svrngs-other.tar.gz\n";
    let files = HashMap::from([("get/SHA256SUMS".to_string(), other.to_vec())]);
    installed(dir.path());
    let checked = check_by(dir.path(), page(&files)).await;
    assert!(
        matches!(checked, Err(UpdateError::NotListed)),
        "{checked:?}"
    );
}

/// **A page that does not answer is not judged current, with no socket**:
/// [`fetch`] asked where no request can go says it cannot judge, by the
/// same branch a refused connection takes (update.rs `fetch`, `send`'s
/// error), and `check` over a page that cannot be asked says that, never
/// Current. Planted: `send`'s error read as an empty answer; `check`
/// reading a page it cannot ask as Current.
#[tokio::test]
async fn a_page_that_does_not_answer_is_not_current_with_no_socket() {
    let said = fetch("http://[not-an-address/get/SHA256SUMS", 1024).await;
    assert!(
        matches!(&said, Err(UpdateError::CouldNotJudge(why)) if why.starts_with("cannot reach the page svrngs came from")),
        "{said:?}"
    );
    let dir = tempfile::tempdir().unwrap();
    installed(dir.path());
    let unanswered = async |url: &str, max: usize| fetch(&format!("http://[{url}"), max).await;
    let checked = check_by(dir.path(), unanswered).await;
    assert!(
        matches!(checked, Err(UpdateError::CouldNotJudge(_))),
        "{checked:?}"
    );
}
