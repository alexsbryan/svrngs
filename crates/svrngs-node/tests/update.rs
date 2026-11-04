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
use svrngs_node::update::{apply_at, check, Check, Outcome, UpdateError, RELEASE_FILE};
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
        apply_at(dir.path(), &exe(dir.path())).await.unwrap(),
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
        apply_at(dir.path(), &exe(dir.path())).await,
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
        apply_at(dir.path(), &exe(dir.path())).await,
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
        apply_at(dir.path(), &exe(dir.path())).await,
        Err(UpdateError::Busy)
    ));
}
