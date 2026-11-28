// SPDX-License-Identifier: AGPL-3.0-or-later
//! **An existing install reaches a complete new one by its usual path,
//! with no step more** (DESIGN §7, "A computer updates only from a signed
//! release"): a computer running the last release, [`LAST`], its own test
//! binary as its `main` (common/as_main.rs `installed_from`, as the mixed
//! scene runs it), in a folder of its own as `install.sh` leaves svrngs,
//! with `release` the hash of what it installed. Its own updater, `svrngs
//! update`, reads a page on 127.0.0.1 whose signed release is this build:
//! this test binary as `svrngs` and this build's Keeper, signed by the
//! tests' key, which both builds trust in place of the release key, as a
//! fork pins its own (gate.sh). [`LAST`]'s updater puts svrngs alone in
//! place, as it does on people's computers. Then svrngs starts, as the
//! login service starts it, and with no step from the person: Keeper is
//! put beside it from the release it runs, svrngs not restarted for it,
//! svrngs answers, and Keeper answers svrngs's build. Planted, watched
//! failing: V1.14.0 as prepared (689a4985), whose start left Keeper out
//! until a second Update.
//!
//! Run by name in scripts/check.sh's "upgrade" step, through gate.sh,
//! which makes the last release's binary and builds this one trusting the
//! tests' key; anything gate.sh makes that is missing fails here, naming
//! it.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

#[path = "../common/as_main.rs"]
mod as_main;
#[path = "../../../svrngs-node/tests/common/signing.rs"]
mod signing;

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ring::digest::{digest, SHA256};
use svrngs_node::invite::ORIGIN_FILE;
use svrngs_node::update::{signed::RELEASE_KEYS, RELEASE_FILE};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// svrngs itself, as each svrngs this test installs runs it.
#[test]
#[ignore = "run by the svrngs this test installs, in a process of its own, as svrngs"]
fn svrngs_here() {
    as_main::main()
}

/// The last release, whose updater puts svrngs alone in place.
const LAST: &str = "v1.13.3";
/// The seed of the tests' release key, whose point is release-key.
const SEED: u8 = 7;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// This computer's tarball, as install.sh names it from `uname`.
fn tarball() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    format!("svrngs-{os}-{}.tar.gz", std::env::consts::ARCH)
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(digest(&SHA256, bytes).as_ref())
}

/// A page host: `site`'s files at their paths on 127.0.0.1, 404 else, as
/// install.rs's `host` serves one.
async fn host(site: PathBuf) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let site = site.clone();
            tokio::spawn(async move {
                let mut head = vec![0u8; 8192];
                let n = sock.read(&mut head).await.unwrap_or(0);
                let head = String::from_utf8_lossy(&head[..n]);
                let path = head
                    .split(' ')
                    .nth(1)
                    .unwrap_or("/")
                    .trim_start_matches('/');
                let file = (!path.contains("..")).then(|| site.join(path));
                let reply = match file.and_then(|f| std::fs::read(f).ok()) {
                    Some(body) => {
                        let ok = format!(
                            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                            body.len()
                        );
                        [ok.into_bytes(), body].concat()
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

/// The page's release of this build: its tarball of this test binary as
/// `svrngs` and `keeper`, the sums naming it, and their signature by the
/// tests' key, made now, after both builds' commits.
fn site(dir: &Path, keeper: &Path) -> PathBuf {
    let (stage, get) = (dir.join("stage"), dir.join("site/get"));
    std::fs::create_dir_all(&stage).unwrap();
    std::fs::create_dir_all(&get).unwrap();
    std::fs::copy(std::env::current_exe().unwrap(), stage.join("svrngs")).unwrap();
    std::fs::copy(keeper, stage.join("svrngs-keeper")).unwrap();
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(get.join(tarball()))
        .arg("-C")
        .arg(&stage)
        .args(["svrngs", "svrngs-keeper"])
        .status()
        .unwrap();
    assert!(made.success(), "the release's tarball is made");
    let sums = format!(
        "{}  {}\n",
        sha256(&std::fs::read(get.join(tarball())).unwrap()),
        tarball()
    );
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let sig = signing::Packet::at(u32::try_from(now).unwrap())
        .sign(sums.as_bytes(), &signing::test_key(SEED));
    std::fs::write(get.join("SHA256SUMS"), sums).unwrap();
    std::fs::write(get.join("SHA256SUMS.sig"), sig).unwrap();
    dir.join("site")
}

/// svrngs as the login service started it, stopped when the test ends.
struct Started(tokio::process::Child);
impl Drop for Started {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_last_release_updates_to_this_one_whole_with_no_step_more() {
    let last = repo().join(format!("target/last-release/svrngs-{LAST}-test-key"));
    let keeper = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("svrngs-keeper");
    for (made, what) in [
        (&last, format!("{LAST}'s svrngs")),
        (&keeper, "this build's Keeper".into()),
    ] {
        assert!(
            made.is_file(),
            "{what} isn't at {}: `sh crates/svrngs/tests/upgrade/gate.sh` makes it, and runs this",
            made.display()
        );
    }
    let key =
        std::fs::read_to_string(repo().join("crates/svrngs/tests/upgrade/release-key")).unwrap();
    let point = signing::point(&signing::test_key(SEED));
    assert_eq!(
        hex::encode(point),
        key.trim(),
        "release-key is the point of the seed this test signs with"
    );
    assert_eq!(
        RELEASE_KEYS,
        [point],
        "this build trusts the tests' key alone, as gate.sh builds it"
    );

    // The computer as the last release left it: svrngs in a folder of its
    // own, no Keeper, the page it came from and what it installed kept.
    let world = tempfile::tempdir().unwrap();
    let (home, data, bin) = (
        world.path().join("home"),
        world.path().join("data"),
        world.path().join("bin"),
    );
    for dir in [&home, &data, &bin] {
        std::fs::create_dir_all(dir).unwrap();
    }
    std::fs::copy(&last, bin.join("svrngs")).unwrap();
    let svrngs = as_main::installed_from(world.path(), &bin.join("svrngs"), as_main::build_relay());
    let port = host(site(world.path(), &keeper)).await;
    std::fs::write(data.join(ORIGIN_FILE), format!("http://127.0.0.1:{port}/")).unwrap();
    std::fs::write(data.join(RELEASE_FILE), format!("{}\n", "0".repeat(64))).unwrap();
    let log = world.path().join("svrngs.log");
    let run = |args: &[&str]| {
        let mut c = tokio::process::Command::new(&svrngs);
        c.args(args)
            .env_clear()
            .env("HOME", &home)
            .env("SVRNGS_DIR", &data)
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("RUST_LOG", "update=debug,cli=debug")
            .stdin(Stdio::null());
        c
    };

    // The person updates the usual way, with the last release's updater.
    let updated = run(&["update"]).output().await.unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&updated.stdout),
        String::from_utf8_lossy(&updated.stderr)
    );
    assert!(
        updated.status.success() && said.contains("Updated."),
        "{LAST}'s `svrngs update`: {said}"
    );
    let ours = sha256(&std::fs::read(std::env::current_exe().unwrap()).unwrap());
    assert_eq!(
        sha256(&std::fs::read(bin.join("svrngs")).unwrap()),
        ours,
        "this build is in place"
    );
    let keeper_at = bin.join("svrngs-keeper");
    assert!(
        !keeper_at.exists(),
        "{LAST}'s updater puts svrngs alone in place"
    );
    let release = std::fs::read_to_string(data.join(RELEASE_FILE)).unwrap();

    // svrngs starts, as the login service starts it, and nothing more.
    let out = std::fs::File::create(&log).unwrap();
    let started = run(&[])
        .stdout(out.try_clone().unwrap())
        .stderr(out)
        .spawn()
        .unwrap();
    let mut started = Started(started);
    let began = Instant::now();
    while !keeper_at.is_file() && began.elapsed() < Duration::from_secs(60) {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let logged = || std::fs::read_to_string(&log).unwrap_or_default();
    assert!(
        keeper_at.is_file(),
        "Keeper is put beside svrngs at its start, with no step more; its log:\n{}",
        logged()
    );
    assert!(
        started.0.try_wait().unwrap().is_none(),
        "svrngs runs on, not restarted for Keeper; its log:\n{}",
        logged()
    );

    // Both work: the svrngs running answers, and Keeper answers its build.
    let asked = run(&["ask", "/v1/groups"]).output().await.unwrap();
    assert!(
        asked.status.success(),
        "the svrngs running answers: {asked:?}\nits log:\n{}",
        logged()
    );
    let build = option_env!("SVRNGS_COMMIT").unwrap_or("dev");
    let listed = tokio::process::Command::new(&keeper_at)
        .args(["--for-svrngs", build, "list", "--json"])
        .env_clear()
        .env("HOME", &home)
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .output()
        .await
        .unwrap();
    assert!(
        listed.status.success(),
        "Keeper answers svrngs's build: {listed:?}"
    );
    assert_eq!(
        sha256(&std::fs::read(&keeper_at).unwrap()),
        sha256(&std::fs::read(&keeper).unwrap()),
        "Keeper is the release's"
    );
    assert_eq!(
        std::fs::read_to_string(data.join(RELEASE_FILE)).unwrap(),
        release,
        "the release kept is the one it runs"
    );
    assert!(
        started.0.try_wait().unwrap().is_none(),
        "svrngs still runs; its log:\n{}",
        logged()
    );
}
