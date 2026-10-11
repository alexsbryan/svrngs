//! A store keeps what it held across a stop (syp-4 d; the sy README §8's
//! fourth part: three times a member's store gave nothing of a file its
//! directory held, after `kill -9` and after SIGTERM). Ana and Ben each a
//! member process of their own (`common/member.rs`, perf-1's), Cy a node in
//! this one, all on loopback in one group: a file Ana takes and a file she
//! fetches, then her process stopped by SIGTERM, and again by `kill -9`,
//! started each time, and Cy asking her provider for each.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[path = "common/member.rs"]
mod member;

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;

use common::loopback_sockets;
use member::{signing, Addr};
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, PublicKey, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::seed::Seed;
use svrngs_node::Name;

#[test]
#[ignore = "the member process the test below starts"]
fn kept_member() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(member::node());
}

/// A member's process, killed when dropped, so a failing test leaves none
/// running.
struct Member(Child);

impl Drop for Member {
    fn drop(&mut self) {
        self.0.kill().ok();
        self.0.wait().ok();
    }
}

/// Member `me`'s process over `dir`, its log `dir/log`: its address once
/// up.
async fn spawn(dir: &Path, me: usize) -> (Member, Addr) {
    let addr = dir.join("addr.json");
    std::fs::remove_file(&addr).ok();
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("log"))
        .unwrap();
    let child = Member(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "kept_member", "--ignored", "--nocapture"])
            .env("PERF_DIR", dir)
            .env("PERF_ME", me.to_string())
            .env("PERF_MEMBERS", "3")
            .env("PERF_PATH", "loopback")
            .env("RUST_LOG", "folder=debug,warn")
            .stdout(std::process::Stdio::null())
            .stderr(log)
            .spawn()
            .unwrap(),
    );
    for _ in 0..300 {
        if let Ok(text) = std::fs::read(&addr) {
            return (child, serde_json::from_slice(&text).unwrap());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("member {me} did not come up: {}", dir.display());
}

/// How many times member's process over `dir` has been told of its peers.
fn told(dir: &Path) -> usize {
    let log = std::fs::read_to_string(dir.join("log")).unwrap_or_default();
    log.matches("PERF told of").count()
}

/// Every member told where each other listens: each process by its
/// `peers.json`, read before this returns, Cy by her lookup.
async fn tell(addrs: &[&Addr], dirs: &[&Path], cy: &Node) {
    let text = serde_json::to_vec(addrs).unwrap();
    for dir in dirs {
        let (at, before) = (dir.join("peers.json"), told(dir));
        std::fs::write(at.with_extension("tmp"), &text).unwrap();
        std::fs::rename(at.with_extension("tmp"), &at).unwrap();
        for _ in 0..100 {
            if told(dir) > before {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    let book = MemoryLookup::new();
    for a in addrs.iter().filter(|a| a.me != 2) {
        let key: PublicKey = a.id.parse().unwrap();
        let mut ea = EndpointAddr::new(key);
        for s in &a.sockets {
            ea = ea.with_ip_addr(*s);
        }
        book.add_endpoint_info(ea);
        cy.transport().add_known_peer(key, a.sockets.clone());
    }
    cy.endpoint().address_lookup().unwrap().add(book);
}

/// `who` asked to sync `dir` in the group, up to 30 s, until it stands
/// there.
async fn sync(who: &Addr, dir: &Path, group: &str) {
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let body = serde_json::json!({ "dir": dir, "group": group }).to_string();
    for _ in 0..60 {
        let asked = http
            .post(format!("http://{}/v1/folders", who.api))
            .header("content-type", "application/json")
            .body(body.clone())
            .send()
            .await;
        if asked.is_ok_and(|a| a.status().is_success()) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("member {} never synced {}", who.me, dir.display());
}

/// Up to 30 s, until the member's log over `dir` says its directory's
/// file events have been watched `times` times since its first start.
async fn watched(dir: &Path, times: usize) {
    for _ in 0..300 {
        let log = std::fs::read_to_string(dir.join("log")).unwrap_or_default();
        if log.matches("synced: the OS's file events watched").count() >= times {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("{}'s file events never watched", dir.display());
}

/// Up to 30 s, whether `path` came to hold `bytes`.
async fn holds(path: &Path, bytes: &[u8]) -> bool {
    for _ in 0..300 {
        if std::fs::read(path).ok().as_deref() == Some(bytes) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

/// Up to 30 s, whether the kept state over `dir` names `name`: the step
/// that wrote it into the directory is done.
async fn kept(dir: &Path, name: &str) -> bool {
    for _ in 0..300 {
        let files = std::fs::read_dir(dir).unwrap().flatten();
        let mut states = files.filter(|e| e.file_name().to_string_lossy().starts_with("synced-"));
        if states.any(|e| std::fs::read_to_string(e.path()).is_ok_and(|t| t.contains(name))) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    false
}

/// Cy asks `holder`'s provider for `hash` whole, into a store under `dir`:
/// the payload's bytes, or why none came.
async fn get(
    cy: &Node,
    holder: &Addr,
    hash: iroh_blobs::Hash,
    dir: PathBuf,
) -> Result<u64, String> {
    let store = iroh_blobs::store::fs::FsStore::load(dir).await.unwrap();
    let key: PublicKey = holder.id.parse().unwrap();
    let conn = cy.endpoint().connect(key, b"/iroh-bytes/4").await;
    let conn = conn.map_err(|e| format!("the dial: {e}"))?;
    let get = store
        .remote()
        .execute_get(conn, iroh_blobs::protocol::GetRequest::blob(hash));
    let got = tokio::time::timeout(Duration::from_secs(10), get).await;
    let got = got.map_err(|_| "no answer".to_string())?;
    got.map(|s| s.payload_bytes_read)
        .map_err(|e| format!("{e:?}"))
}

fn noise(len: usize, salt: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; len];
    blake3::Hasher::new()
        .update(salt)
        .finalize_xof()
        .fill(&mut out);
    out
}

/// **A store keeps what it held across a stop**: Ana takes a file and
/// fetches one Ben wrote, her kept state naming both; her process is
/// stopped by SIGTERM, started, and her provider gives Cy both, whole;
/// then the same after `kill -9`. Red with the step's writes left to the
/// store's batch (its commit at most 500 ms later): the fetched file's
/// bytes lost at each stop, `sent … done=false payload=0`.
#[tokio::test]
async fn a_store_keeps_what_it_held_across_a_stop() {
    let root = tempfile::tempdir().unwrap();
    let dir = |name: &str| {
        let d = root.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        d
    };
    let (ana_dir, ben_dir, cy_dir) = (dir("ana"), dir("ben"), dir("cy"));
    let (ana_disk, ben_disk) = (dir("disk/ana/Shared"), dir("disk/ben/Shared"));
    let relay = RelayConfig::from_parts(Vec::new(), Some("none"))
        .unwrap()
        .on_loopback_only();
    let cy = Node::start(
        cy_dir.clone(),
        signing(2),
        &relay,
        member::seed_file,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let cy_addr = Addr {
        me: 2,
        id: cy.endpoint().id().to_string(),
        sockets: loopback_sockets(cy.endpoint()),
        api: cy.api_addr(),
    };
    let group = Seed::from_parts(
        Name::parse("perf").unwrap(),
        Name::parse("p0").unwrap(),
        signing(0).verifying_key(),
        [9; 16],
    )
    .id();
    let (mut ana, mut ana_addr) = spawn(&ana_dir, 0).await;
    let (ben, ben_addr) = spawn(&ben_dir, 1).await;
    tell(
        &[&ana_addr, &ben_addr, &cy_addr],
        &[&ana_dir, &ben_dir],
        &cy,
    )
    .await;
    sync(&ana_addr, &ana_disk, &group).await;
    // Ana's act nudges her round, which carries Ben's admission to him.
    let taken = noise(50_000, b"taken");
    std::fs::write(ana_disk.join("taken.bin"), &taken).unwrap();
    sync(&ben_addr, &ben_disk, &group).await;
    assert!(
        holds(&ben_disk.join("taken.bin"), &taken).await,
        "Ana's file reaches Ben"
    );
    let fetched = noise(60_000, b"fetched");
    std::fs::write(ben_disk.join("fetched.bin"), &fetched).unwrap();
    assert!(
        holds(&ana_disk.join("fetched.bin"), &fetched).await,
        "Ben's file reaches Ana"
    );
    assert!(kept(&ana_dir, "fetched.bin").await, "Ana's step holds it");

    let mut said = Vec::new();
    for sig in ["TERM", "KILL"] {
        let pid = ana.0.id().to_string();
        let kill = Command::new("kill")
            .args([&format!("-{sig}"), &pid])
            .status();
        assert!(kill.unwrap().success());
        ana.0.wait().unwrap();
        (ana, ana_addr) = spawn(&ana_dir, 0).await;
        tell(
            &[&ana_addr, &ben_addr, &cy_addr],
            &[&ana_dir, &ben_dir],
            &cy,
        )
        .await;
        for (name, bytes) in [("taken", &taken), ("fetched", &fetched)] {
            let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(bytes).as_bytes());
            let into = root.path().join(format!("asked-{sig}-{name}"));
            said.push((sig, name, get(&cy, &ana_addr, hash, into).await));
        }
    }
    drop((ana, ben));
    cy.endpoint().close().await;
    let whole = |name: &str| if name == "taken" { 50_000 } else { 60_000 };
    assert!(
        said.iter().all(|(_, name, got)| got == &Ok(whole(name))),
        "Ana's provider gives Cy each file whole after each stop: {said:?}\n{}",
        std::fs::read_to_string(ana_dir.join("log")).unwrap_or_default()
    );
}

/// **A restart resumes a synced directory unasked** (syp-4 e): Ana syncs
/// her directory and her file reaches Ben; her process is stopped and
/// started, nobody asking it again; Ben writes a file, and once his act is
/// written Ana writes one of her own, which her resumed step takes and
/// sends, its act nudging her round to pull Ben's; Ben's file then reaches
/// her directory. Ana dials Ben, not Ben her: Ben's process could not dial
/// hers after her start within this test's 30 s (`iroh bridge: dial failed
/// … timed out`), which this test does not judge. Red with the directories
/// it syncs not kept (sy-1: the command asked again at each start): her
/// file is never taken, and Ben's never reaches her.
///
/// And **a change the OS's events missed is found at the sweep** (perf-2
/// f): Ana's file edited while her process is stopped, no watcher running,
/// is found by the walk her loop starts with, and her provider gives Cy the
/// edit, which only a record placing it allows. Red with the start's sweep
/// out (the loop's first step looking at no path): the edit never sent,
/// and Cy refused it. Neither the sweep each ten minutes nor the OS saying
/// it dropped events can be brought about by a test in seconds: each is a
/// walk of the same step.
#[tokio::test]
async fn a_restart_resumes_a_synced_directory_unasked() {
    let root = tempfile::tempdir().unwrap();
    let dir = |name: &str| {
        let d = root.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        d
    };
    let (ana_dir, ben_dir, cy_dir) = (dir("ana"), dir("ben"), dir("cy"));
    let (ana_disk, ben_disk) = (dir("disk/ana/Shared"), dir("disk/ben/Shared"));
    let relay = RelayConfig::from_parts(Vec::new(), Some("none"))
        .unwrap()
        .on_loopback_only();
    let cy = Node::start(
        cy_dir.clone(),
        signing(2),
        &relay,
        member::seed_file,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let cy_addr = Addr {
        me: 2,
        id: cy.endpoint().id().to_string(),
        sockets: loopback_sockets(cy.endpoint()),
        api: cy.api_addr(),
    };
    let group = Seed::from_parts(
        Name::parse("perf").unwrap(),
        Name::parse("p0").unwrap(),
        signing(0).verifying_key(),
        [9; 16],
    )
    .id();
    let (ana, ana_addr) = spawn(&ana_dir, 0).await;
    let (ben, ben_addr) = spawn(&ben_dir, 1).await;
    let dirs = [ana_dir.as_path(), ben_dir.as_path()];
    tell(&[&ana_addr, &ben_addr, &cy_addr], &dirs, &cy).await;
    sync(&ana_addr, &ana_disk, &group).await;
    watched(&ana_dir, 1).await;
    let first = noise(30_000, b"before the stop");
    std::fs::write(ana_disk.join("first.bin"), &first).unwrap();
    sync(&ben_addr, &ben_disk, &group).await;
    assert!(
        holds(&ben_disk.join("first.bin"), &first).await,
        "Ana's file reaches Ben"
    );

    let pid = ana.0.id().to_string();
    let kill = Command::new("kill").args(["-TERM", &pid]).status();
    assert!(kill.unwrap().success());
    drop(ana);
    let edited = noise(30_500, b"edited while stopped");
    std::fs::write(ana_disk.join("first.bin"), &edited).unwrap();
    let (ana, ana_addr) = spawn(&ana_dir, 0).await;
    tell(&[&ana_addr, &ben_addr, &cy_addr], &dirs, &cy).await;
    watched(&ana_dir, 2).await;
    let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(&edited).as_bytes());
    let mut swept = Err("not asked".to_string());
    for k in 0..30 {
        swept = get(&cy, &ana_addr, hash, root.path().join(format!("cy-{k}"))).await;
        if swept.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert_eq!(
        swept,
        Ok(30_500),
        "Ana's edit while stopped is found at her start's sweep and placed: {}",
        std::fs::read_to_string(ana_dir.join("log")).unwrap_or_default()
    );
    let after = noise(31_000, b"after the start");
    std::fs::write(ben_disk.join("after.bin"), &after).unwrap();
    assert!(kept(&ben_dir, "after.bin").await, "Ben's step sends it");
    std::fs::write(ana_disk.join("mine.bin"), noise(32_000, b"mine")).unwrap();
    let reached = holds(&ana_disk.join("after.bin"), &after).await;
    drop((ana, ben));
    cy.endpoint().close().await;
    assert!(
        reached,
        "Ben's file reaches Ana's directory after her start, unasked: {}",
        std::fs::read_to_string(ana_dir.join("log")).unwrap_or_default()
    );
}
