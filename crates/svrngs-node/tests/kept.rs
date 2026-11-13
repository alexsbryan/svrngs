//! A store keeps what it held across a stop (syp-4 d; the sy README §8's
//! fourth part: three times a member's store gave nothing of a file its
//! directory held, after `kill -9` and after SIGTERM). Ana and Ben each a
//! member process of their own (`common/member.rs`, perf-1's), Cy a node in
//! this one, all on loopback in one group: a file Ana takes and a file she
//! fetches, then her process stopped by SIGTERM, and again by `kill -9`,
//! started each time, and Cy asking her provider for each. A process killed
//! and a path iroh drops are these tests' claims (QUIC, a socket); a restart
//! that claims neither runs on one wire in this process.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
#[path = "common/computers.rs"]
mod computers;
#[path = "common/member.rs"]
mod member;
#[path = "common/on_wire.rs"]
mod on_wire;

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;

use common::{loopback_sockets, noise};
use member::{signing, Addr};
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, PublicKey, RelayConfig};
use svrngs_node::node::Node;

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

/// Cy: a node in this test's process over `dir`, on loopback, and its
/// address as the member processes are told it.
async fn cy_here(dir: &Path) -> (Node, Addr) {
    let relay = RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    };
    let cy = Node::start(
        dir.to_path_buf(),
        signing(2),
        &relay,
        member::seed_file,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let addr = Addr {
        me: 2,
        id: cy.endpoint().id().to_string(),
        sockets: loopback_sockets(cy.endpoint()),
        api: cy.api_addr(),
        token: cy.token(),
    };
    (cy, addr)
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
            .bearer_auth(&who.token)
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
    let (cy, cy_addr) = cy_here(&cy_dir).await;
    let group = member::seed().id();
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

/// **A member restarted is answered at once** (syp-6 b): Ana's process
/// fetches Cy's file, holding its connection into Cy open, as Ben's held
/// his into Ana in S3, and Cy fetches Ana's; Ana's process is
/// stopped by SIGTERM, as a service manager and syp-5's driver stop it,
/// and started, on another port; Cy asks her for the file at once, and is
/// answered within [`svrngs_node::node::ANSWER_WITHIN`]. Red with the
/// process gone on SIGTERM, its endpoint left open (d4fb5bf): Cy's iroh
/// sends only to the path in use, the dead process's, until it idles out
/// at 15 s (iroh-1.0.2 `remote_state.rs:799`, `socket.rs:117`), so the ask
/// is not answered — syp-5's S3, Ana dialling Ben's last process
/// (`demo/runs/syp/2026-10-10/rerun/README.md` §6, item 3). The failure
/// says the addresses Cy's iroh holds for Ana.
#[tokio::test]
async fn a_member_restarted_is_answered_at_once() {
    let root = tempfile::tempdir().unwrap();
    let dir = |name: &str| {
        let d = root.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        d
    };
    let (ana_dir, ben_dir, cy_dir) = (dir("ana"), dir("ben"), dir("cy"));
    let (ana_disk, cy_disk) = (dir("disk/ana/Shared"), dir("disk/cy/Shared"));
    let (cy, cy_addr) = cy_here(&cy_dir).await;
    // Cy holds the group's seed, as each member process does.
    let (group, _) = member::hold_seed(&cy);
    let (ana, ana_addr) = spawn(&ana_dir, 0).await;
    let (ben, ben_addr) = spawn(&ben_dir, 1).await;
    let dirs = [ana_dir.as_path(), ben_dir.as_path()];
    tell(&[&ana_addr, &ben_addr, &cy_addr], &dirs, &cy).await;
    sync(&ana_addr, &ana_disk, &group).await;
    // Cy's round takes Ana's admission of her, so she stands.
    tokio::time::timeout(Duration::from_secs(30), cy.round())
        .await
        .expect("Cy's round finishes");
    sync(&cy_addr, &cy_disk, &group).await;
    let cys = noise(20_000, b"Cy's");
    std::fs::write(cy_disk.join("c.bin"), &cys).unwrap();
    assert!(
        holds(&ana_disk.join("c.bin"), &cys).await,
        "Ana's process fetches Cy's file"
    );
    let bytes = noise(40_000, b"restarted");
    std::fs::write(ana_disk.join("r.bin"), &bytes).unwrap();
    assert!(kept(&ana_dir, "r.bin").await, "Ana's step takes it");
    let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(&bytes).as_bytes());
    let mut before = Err("not asked".to_string());
    for k in 0..30 {
        before = get(
            &cy,
            &ana_addr,
            hash,
            root.path().join(format!("before-{k}")),
        )
        .await;
        if before.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert_eq!(before, Ok(40_000), "Cy holds a path to Ana's first process");

    let pid = ana.0.id().to_string();
    let kill = Command::new("kill").args(["-TERM", &pid]).status();
    assert!(kill.unwrap().success());
    // Its own exit waited on, as syp-5's driver waits (sy run.py:39):
    // dropped now, it is killed outright, mid-close.
    let mut ana = ana;
    ana.0.wait().unwrap();
    drop(ana);
    let (ana, ana_addr) = spawn(&ana_dir, 0).await;
    tell(&[&ana_addr, &ben_addr, &cy_addr], &dirs, &cy).await;
    let t0 = std::time::Instant::now();
    let within = svrngs_node::node::ANSWER_WITHIN;
    let asked = get(&cy, &ana_addr, hash, root.path().join("after"));
    let after = tokio::time::timeout(within, asked).await;
    let took = t0.elapsed();
    let key: PublicKey = ana_addr.id.parse().unwrap();
    let known: Vec<String> = cy
        .endpoint()
        .remote_info(key)
        .await
        .iter()
        .flat_map(|i| i.addrs())
        .map(|a| format!("{:?} {:?}", a.addr(), a.usage()))
        .collect();
    drop((ana, ben));
    cy.endpoint().close().await;
    assert_eq!(
        after,
        Ok(Ok(40_000)),
        "Ana's process, started again on {:?}, answers Cy within {within:?} (took {took:?}); \
         Cy's iroh holding for her {known:?}",
        ana_addr.sockets
    );
}

/// `who` asked, on the wire, to sync `dir` in the group, up to 30 s, until
/// it stands there.
async fn synced(who: &Node, dir: &Path, group: &str) {
    std::fs::create_dir_all(dir).unwrap();
    let body = serde_json::json!({ "dir": dir, "group": group }).to_string();
    for _ in 0..60 {
        let asked = on_wire::http()
            .post(format!("http://{}/v1/folders", who.api_addr()))
            .bearer_auth(who.token())
            .header("content-type", "application/json")
            .body(body.clone())
            .send()
            .await;
        if asked.is_ok_and(|a| a.status().is_success()) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("{} never synced {}", who.net().id(), dir.display());
}

/// **A restart resumes a synced directory unasked** (syp-4 e): Ana syncs
/// her directory and her file reaches Ben; her computer is stopped and
/// started, nobody asking it again; Ben writes a file, and once his act is
/// written Ana writes one of her own, which her resumed step takes and
/// sends, its act nudging her round to pull Ben's; Ben's file then reaches
/// her directory. Red with the directories it syncs not kept (sy-1: the
/// command asked again at each start): her edit is never served, and Ben's
/// file never reaches her.
///
/// And **a change made while stopped is found at the start** (perf-2 f):
/// Ana's file edited while her computer is stopped, no watcher running, is
/// found by her loop's first step, and her provider gives Cy the edit,
/// which only a record placing it allows.
///
/// On one wire (common/on_wire.rs), no endpoint and no socket: stopped is
/// her node closed and dropped, started a node over the same data dir,
/// which a store held past its node kept from opening (node.rs `Stop`).
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
    let ana = on_wire::start(1, &ana_dir).await;
    let ben = on_wire::start(2, &ben_dir).await;
    let cy = on_wire::start(3, &cy_dir).await;
    let group = computers::found("films", 9, &ana, &[(&ben, 2), (&cy, 3)]);
    for node in [&ben, &cy, &ana] {
        computers::round(node).await;
    }
    synced(&ana, &ana_disk, &group).await;
    let first = noise(30_000, b"before the stop");
    std::fs::write(ana_disk.join("first.bin"), &first).unwrap();
    synced(&ben, &ben_disk, &group).await;
    assert!(
        holds(&ben_disk.join("first.bin"), &first).await,
        "Ana's file reaches Ben"
    );

    ana.net().close().await;
    drop(ana);
    let edited = noise(30_500, b"edited while stopped");
    std::fs::write(ana_disk.join("first.bin"), &edited).unwrap();
    let ana = tokio::time::timeout(Duration::from_secs(30), on_wire::start(1, &ana_dir));
    let ana = ana
        .await
        .expect("Ana's computer starts again over her data dir");
    let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(&edited).as_bytes());
    let mut swept = Err("not asked".to_string());
    for k in 0..30 {
        swept = on_wire::get(&cy, &ana, hash, &root.path().join(format!("cy-{k}"))).await;
        if swept.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    assert_eq!(
        swept,
        Ok(30_500),
        "Ana's edit while stopped is found at her start's sweep and placed"
    );
    let after = noise(31_000, b"after the start");
    std::fs::write(ben_disk.join("after.bin"), &after).unwrap();
    assert!(kept(&ben_dir, "after.bin").await, "Ben's step sends it");
    std::fs::write(ana_disk.join("mine.bin"), noise(32_000, b"mine")).unwrap();
    let reached = holds(&ana_disk.join("after.bin"), &after).await;
    for node in [&ana, &ben, &cy] {
        node.net().close().await;
    }
    assert!(
        reached,
        "Ben's file reaches Ana's directory after her start, unasked"
    );
}

/// What the `folder` target traces on this test's thread, kept to be read:
/// a dropped node's tasks run on it, as every task of a `#[tokio::test]`.
#[derive(Clone, Default)]
struct Traced(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for Traced {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// **A node dropped leaves nothing of its folders running** (node.rs
/// `Stop`): Ana syncs a directory, her file is taken and its act written;
/// her node is closed and dropped; she edits the file and waits past three
/// of the store's sweeps (folder.rs `SWEEP`, 1 s). No act is written into
/// her record, no step of her directory's loop runs and the sweep asks her
/// node nothing: the `folder` target says nothing at all after the drop.
/// Red with `Host::stopping` not cancelled at the drop: the old loop
/// steps at the edit, finds it, and stops short only because the store is
/// shut, so no act is written; the act count alone would not see it. On
/// one wire, Ana alone in her group.
#[tokio::test]
async fn a_node_dropped_leaves_no_task_of_its_folders_running() {
    let root = tempfile::tempdir().unwrap();
    let (ana_dir, ana_disk) = (root.path().join("ana"), root.path().join("disk/Shared"));
    std::fs::create_dir_all(&ana_dir).unwrap();
    let ana = on_wire::start(1, &ana_dir).await;
    let group = computers::found("films", 9, &ana, &[]);
    synced(&ana, &ana_disk, &group).await;
    std::fs::write(ana_disk.join("first.bin"), noise(10_000, b"first")).unwrap();
    assert!(kept(&ana_dir, "first.bin").await, "Ana's step takes it");
    let acts = || {
        let rail = svrngs_node::RingRail::new(&ana_dir, std::sync::Arc::new(computers::signing(1)));
        svrngs_core::tests_support::wire_ops(&rail.admission(&group).unwrap()).len()
    };
    let before = acts();

    ana.net().close().await;
    let traced = Traced::default();
    let writer = traced.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("folder=debug"))
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .finish();
    let _on_this_thread = tracing::subscriber::set_default(subscriber);
    drop(ana);
    std::fs::write(ana_disk.join("first.bin"), noise(10_500, b"edited after")).unwrap();
    tokio::time::sleep(Duration::from_millis(3_500)).await;
    let said = String::from_utf8(traced.0.lock().unwrap().clone()).unwrap();
    assert_eq!(
        acts(),
        before,
        "no act written through the dropped node's rail"
    );
    assert!(
        said.is_empty(),
        "nothing of the dropped node's folders runs, no step and no sweep's ask: {said}"
    );
}
