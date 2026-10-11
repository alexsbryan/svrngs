//! A folder synced both ways (sy-1, the mesh-Dropbox spike; DESIGN §9):
//! Ana, Ben and Cy in one group, each change reaching the others' disks,
//! and the bytes served to members alone. Three nodes in one process, no
//! relay, each reached by the addresses seeded in the others' lookup.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::path::Path;
use std::time::Duration;

use svrngs_node::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::seed::Seed;
use svrngs_node::{actor_of, Name, Person, RailAct, SigningKey};

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn seed_file(dir: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("seed.json")) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

async fn start(seed: u8, dir: &Path) -> Node {
    let relay = RelayConfig::from_parts(Vec::new(), Some("none"))
        .unwrap()
        .on_loopback_only();
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &relay,
        seed_file,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// Ana's group `films`, Ben and Cy admitted, each computer knowing where
/// the others listen, for the round and for a dial by key alike.
struct Group {
    ana: Node,
    ben: Node,
    cy: Node,
    _dirs: Vec<tempfile::TempDir>,
}

/// The seed of a group Ana founds, named `name`, salted by `salt`.
fn seed(name: &str, salt: u8) -> Seed {
    Seed::from_parts(
        Name::parse(name).unwrap(),
        Name::parse("ana").unwrap(),
        signing(1).verifying_key(),
        [salt; 16],
    )
}

/// A group Ana founds, named `name`, salted by `salt`, with each of
/// `others` (Ben 2, Cy 3) admitted: its id, on every node named.
fn found(name: &str, salt: u8, ana: &Node, others: &[(&Node, u8)]) -> String {
    let seed = seed(name, salt);
    let id = seed.id();
    for node in std::iter::once(ana).chain(others.iter().map(|(n, _)| *n)) {
        node.rail().set_seed(&id, &seed.roster()).unwrap();
        let dir = node.rail().group_dir(&id).unwrap();
        std::fs::write(dir.join("seed.json"), seed.to_text()).unwrap();
    }
    for &(_, n) in others {
        let person = ["", "", "ben", "cy"][n as usize];
        let admit = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse(person).unwrap(),
                key: actor_of(&signing(n)),
                invite: None,
            },
            on_behalf_of: None,
        };
        ana.rail().commit(&id, &admit).unwrap().unwrap();
    }
    id
}

impl Group {
    async fn new() -> Self {
        let group = Group::computers().await;
        found("films", 9, &group.ana, &[(&group.ben, 2), (&group.cy, 3)]);
        group.settle().await;
        group
    }

    /// A round on each computer, so each holds what Ana's record says of
    /// who stands where: a folder is synced only where one stands.
    async fn settle(&self) {
        for node in [&self.ben, &self.cy, &self.ana] {
            round(node).await;
        }
    }

    /// Ana's, Ben's and Cy's computers, in no group yet.
    async fn computers() -> Self {
        let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
        let ana = start(1, dirs[0].path()).await;
        let ben = start(2, dirs[1].path()).await;
        let cy = start(3, dirs[2].path()).await;
        for who in [&ana, &ben, &cy] {
            let book = MemoryLookup::new();
            for other in [&ana, &ben, &cy] {
                let mut addr = EndpointAddr::new(other.endpoint().id());
                for a in loopback_sockets(other.endpoint()) {
                    addr = addr.with_ip_addr(a);
                }
                book.add_endpoint_info(addr);
                who.transport()
                    .add_known_peer(other.endpoint().id(), loopback_sockets(other.endpoint()));
            }
            who.endpoint().address_lookup().unwrap().add(book);
        }
        Group {
            ana,
            ben,
            cy,
            _dirs: dirs,
        }
    }

    async fn close(self) {
        for node in [self.ana, self.ben, self.cy] {
            node.endpoint().close().await;
        }
    }
}

async fn round(node: &Node) {
    tokio::time::timeout(Duration::from_secs(30), node.round())
        .await
        .expect("a round finishes");
}

fn http() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// `node`'s record API asked what the command asks it: its status and
/// its answer's text.
async fn ask(node: &Node, body: serde_json::Value) -> (u16, String) {
    let answer = http()
        .post(format!("http://{}/v1/folders", node.api_addr()))
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .unwrap();
    (answer.status().as_u16(), answer.text().await.unwrap())
}

/// `len` bytes no two runs agree on by accident: BLAKE3's output stream.
fn noise(len: usize, salt: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; len];
    blake3::Hasher::new()
        .update(salt)
        .finalize_xof()
        .fill(&mut out);
    out
}

// ── A folder synced both ways (sy-1, the mesh-Dropbox spike) ──────────

/// `who` syncs a directory of its own named `Shared` in `films`, as
/// `share --folder <dir> --group films` asks: the directory.
async fn both_ways(who: &Node, root: &Path) -> std::path::PathBuf {
    synced_in(who, root, &seed("films", 9).id()).await
}

/// `who` syncs a directory of its own named `Shared` under `root`, in the
/// group whose id is `group`: the directory.
async fn synced_in(who: &Node, root: &Path, group: &str) -> std::path::PathBuf {
    let dir = root.join("Shared");
    std::fs::create_dir_all(&dir).unwrap();
    let said = ask(who, serde_json::json!({ "dir": dir, "group": group })).await;
    assert_eq!(said.0, 200, "{}", said.1);
    assert!(said.1.contains("\"Shared\""), "{}", said.1);
    dir
}

/// Up to 30 s, rounds run on every node: whether `done` came true.
async fn until(group: &Group, done: impl Fn() -> bool) -> bool {
    until_on(&[&group.ana, &group.ben, &group.cy], done).await
}

/// Up to 30 s, rounds run on `nodes`: whether `done` came true.
async fn until_on(nodes: &[&Node], done: impl Fn() -> bool) -> bool {
    for _ in 0..30 {
        if done() {
            return true;
        }
        for node in nodes {
            round(node).await;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    done()
}

/// `from` asks `holder`'s provider for `hash` whole, into a store of its
/// own under `dir`: the payload's bytes, or why none came.
async fn get(
    from: &Node,
    holder: &Node,
    hash: iroh_blobs::Hash,
    dir: &Path,
) -> Result<u64, String> {
    let store = iroh_blobs::store::fs::FsStore::load(dir).await.unwrap();
    let conn = from
        .endpoint()
        .connect(holder.endpoint().id(), b"/iroh-bytes/4")
        .await;
    let conn = conn.map_err(|e| format!("the dial: {e}"))?;
    let get = store
        .remote()
        .execute_get(conn, iroh_blobs::protocol::GetRequest::blob(hash));
    let got = tokio::time::timeout(Duration::from_secs(10), get).await;
    let got = got.map_err(|_| "no answer".to_string())?;
    got.map(|s| s.payload_bytes_read)
        .map_err(|e| format!("{e:?}"))
}

/// The contents of every file in `dir`, each once.
fn contents(dir: &Path) -> std::collections::BTreeSet<Vec<u8>> {
    let files = std::fs::read_dir(dir).unwrap().flatten();
    files.filter_map(|e| std::fs::read(e.path()).ok()).collect()
}

/// **A change by any member reaches another's directory**: Ana and Ben
/// each sync a directory; a file Ana writes appears in Ben's, byte for
/// byte; Ben's edit of it appears in Ana's; Ben's delete of it takes it
/// from Ana's. Red with the node's apply taken out (nothing reaches Ben).
#[tokio::test]
async fn a_change_by_any_member_reaches_another_s_directory() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let first = noise(70_000, b"first");
    std::fs::write(ana.join("a.bin"), &first).unwrap();
    let at_ben = || std::fs::read(ben.join("a.bin")).ok();
    assert!(
        until(&group, || at_ben().as_ref() == Some(&first)).await,
        "Ana's file reaches Ben"
    );

    let edit = noise(1 << 20, b"edit");
    std::fs::write(ben.join("a.bin"), &edit).unwrap();
    let at_ana = || std::fs::read(ana.join("a.bin")).ok();
    assert!(
        until(&group, || at_ana().as_ref() == Some(&edit)).await,
        "Ben's edit reaches Ana"
    );

    std::fs::remove_file(ben.join("a.bin")).unwrap();
    assert!(
        until(&group, || at_ana().is_none()).await,
        "Ben's delete reaches Ana"
    );
    group.close().await;
}

/// **A synced file is served by a member who did not write it, and to no
/// one outside the group**: Ana writes, Ben's directory takes it; Cy asks
/// Ben's provider for its hash and is given it, verified; a node standing
/// in no group held at Ben's is closed at the accept. Red with the provider
/// asking only this computer's own shared folders (Ben refuses Cy); and
/// with the gate admitting any dialer (sy-1's `open-gate`), Dee then
/// refused by the provider alone, which asks who stands too (syp-4 a).
#[tokio::test]
async fn a_synced_file_is_served_by_a_member_who_did_not_write_it_and_to_no_one_outside() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let bytes = noise(50_000, b"served");
    std::fs::write(ana.join("b.bin"), &bytes).unwrap();
    let held = || std::fs::read(ben.join("b.bin")).ok() == Some(bytes.clone());
    assert!(until(&group, held).await, "Ana's file reaches Ben");
    let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(&bytes).as_bytes());

    let ask_ben = |from: &Node, dir: std::path::PathBuf| {
        let (endpoint, ben) = (from.endpoint().clone(), group.ben.endpoint().id());
        async move {
            let store = iroh_blobs::store::fs::FsStore::load(dir).await.unwrap();
            let conn = endpoint.connect(ben, b"/iroh-bytes/4").await;
            let conn = conn.map_err(|e| format!("the dial: {e}"))?;
            let get = store
                .remote()
                .execute_get(conn, iroh_blobs::protocol::GetRequest::blob(hash));
            let got = tokio::time::timeout(Duration::from_secs(10), get).await;
            got.map_err(|_| "no answer".to_string())?
                .map_err(|e| format!("{e:?}"))
        }
    };
    let cy = ask_ben(&group.cy, tempfile::tempdir().unwrap().keep()).await;
    assert_eq!(
        cy.map(|s| s.payload_bytes_read),
        Ok(50_000),
        "Ben serves Cy what Ana wrote"
    );

    let dee_dir = tempfile::tempdir().unwrap();
    let dee = start(4, dee_dir.path()).await;
    let book = MemoryLookup::new();
    let mut addr = EndpointAddr::new(group.ben.endpoint().id());
    for a in loopback_sockets(group.ben.endpoint()) {
        addr = addr.with_ip_addr(a);
    }
    book.add_endpoint_info(addr);
    dee.endpoint().address_lookup().unwrap().add(book);
    let refused = ask_ben(&dee, dee_dir.path().join("asked")).await;
    assert!(
        refused.as_ref().is_err_and(|e| e.contains("not admitted")),
        "Dee, in no group, is closed at the accept: {refused:?}"
    );
    dee.endpoint().close().await;
    group.close().await;
}

/// **A link in a synced directory is not followed**: Ana's directory holds
/// a link to a file beside it and a file of its own; the file reaches Ben,
/// the link's target never does. Red with the walk following links (sy-1's
/// `follow-links`, group folders' B7): the target reaches Ben as `link.bin`.
#[tokio::test]
async fn a_link_in_a_synced_directory_is_not_followed() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let outside = noise(30_000, b"outside");
    std::fs::write(ra.path().join("outside.bin"), &outside).unwrap();
    std::os::unix::fs::symlink(ra.path().join("outside.bin"), ana.join("link.bin")).unwrap();
    let own = noise(20_000, b"own");
    std::fs::write(ana.join("z.bin"), &own).unwrap();
    let held = || std::fs::read(ben.join("z.bin")).ok().as_ref() == Some(&own);
    assert!(until(&group, held).await, "Ana's own file reaches Ben");
    // Placed in path order, the link's version is written before `z.bin`.
    assert!(
        !contents(&ben).contains(&outside),
        "the link's target never reaches Ben: {:?}",
        std::fs::read_dir(&ben)
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect::<Vec<_>>()
    );
    group.close().await;
}

/// **A hash no synced folder places is refused**: Ana writes a file and
/// deletes it, the delete reaching Ben, and Cy, a member, asks her
/// provider for it and is refused. Red with the provider serving any hash
/// its store holds (`unchecked`, group folders' B7) only while her store
/// holds the version, which since syp-4 (b) it drops at the delete;
/// `a_synced_folder_is_one_group_s` refuses a hash the store keeps.
#[tokio::test]
async fn a_hash_no_synced_folder_places_is_refused_though_the_store_holds_it() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let bytes = noise(40_000, b"deleted");
    std::fs::write(ana.join("d.bin"), &bytes).unwrap();
    let at_ben = || std::fs::read(ben.join("d.bin")).ok();
    assert!(
        until(&group, || at_ben().as_ref() == Some(&bytes)).await,
        "Ana's file reaches Ben"
    );
    std::fs::remove_file(ana.join("d.bin")).unwrap();
    assert!(
        until(&group, || at_ben().is_none()).await,
        "Ana's delete reaches Ben"
    );
    let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(&bytes).as_bytes());
    let cy = tempfile::tempdir().unwrap();
    let asked = get(&group.cy, &group.ana, hash, cy.path()).await;
    assert!(
        asked.is_err(),
        "Ana refuses Cy a version nothing places: {asked:?}"
    );
    group.close().await;
}

/// **A hash the record stopped placing is refused at the next get**
/// (perf-2 c): Cy gets Ana's file from her provider; then an act of Ana's
/// deleting it is planted in her record, past her directory, and Cy's next
/// get of the same hash, at once, is refused by the record, though her
/// store may still hold it. Red with the layout kept past a moved record
/// (the digest's check out): a layout read before her act answers.
#[tokio::test]
async fn a_hash_the_record_stopped_placing_is_refused_at_the_next_get() {
    use svrngs_core::{Change, SyncPath};
    let said = Said::default();
    let writer = said.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("folder=debug")
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let _said = tracing::subscriber::set_default(subscriber);
    let group = Group::new().await;
    let ra = tempfile::tempdir().unwrap();
    let ana = both_ways(&group.ana, ra.path()).await;
    let bytes = noise(40_000, b"stopped");
    let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(&bytes).as_bytes());
    std::fs::write(ana.join("s.bin"), &bytes).unwrap();
    let ns = seed("films", 9).id();
    let placed = || {
        let layout = group.ana.rail().admission(&ns).unwrap().layout();
        layout.into_iter().find(|p| p.path.as_str() == "s.bin")
    };
    let t0 = std::time::Instant::now();
    while placed().is_none() && t0.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let entry = placed().expect("Ana's act places her file").entry;
    let first = get(
        &group.cy,
        &group.ana,
        hash,
        tempfile::tempdir().unwrap().path(),
    )
    .await;
    let deleted = svrngs_core::tests_support::Fixture {
        act: RailAct::Synced {
            folder: "Shared".into(),
            changes: vec![Change {
                path: SyncPath::parse("s.bin").unwrap(),
                now: None,
                base: Some(entry),
            }],
        },
        on_behalf_of: None,
    };
    group.ana.rail().commit(&ns, &deleted).unwrap().unwrap();
    let second = get(
        &group.cy,
        &group.ana,
        hash,
        tempfile::tempdir().unwrap().path(),
    )
    .await;
    let log = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let answers: Vec<&str> = log
        .lines()
        .filter(|l| {
            l.contains(&hash.to_string())
                && l.contains("folder: ")
                && l.contains("the dialer stands in places the hash")
        })
        .collect();
    group.close().await;
    assert_eq!(first, Ok(40_000), "Ana serves Cy her file: {answers:#?}");
    assert!(second.is_err(), "the second get is refused: {second:?}");
    assert!(
        answers.last().is_some_and(|l| l.contains("refused")),
        "the record refused it, not the store: {answers:#?}"
    );
}

/// **A step's act names its changes in the walk's order** (perf-2 e): 24
/// files in Ana's directory before she shares it, the first the walk meets
/// the largest, 8 MB, and each after smaller, down to 1 KiB; her one act
/// names them in the order the directory lists them (the walk's, for a
/// directory of files alone). Red with the files taken at once answered
/// as each import finished (`buffered_unordered`): the 8 MB file named
/// after files met after it.
#[tokio::test]
async fn a_step_s_act_names_its_changes_in_the_walk_s_order() {
    let (dir, ra) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ana = start(1, dir.path()).await;
    let ns = found("films", 9, &ana, &[]);
    let shared = ra.path().join("Shared");
    std::fs::create_dir_all(&shared).unwrap();
    for k in 0..24 {
        std::fs::write(shared.join(format!("w{k:02}.bin")), b"").unwrap();
    }
    let walked: Vec<String> = std::fs::read_dir(&shared)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    for (i, name) in walked.iter().enumerate() {
        let len = if i == 0 { 8 << 20 } else { (24 - i) << 10 };
        std::fs::write(shared.join(name), noise(len, name.as_bytes())).unwrap();
    }
    synced_in(&ana, ra.path(), &ns).await;
    let journal = ana.rail().group_dir(&ns).unwrap().join("ring_oplog.jsonl");
    let act = || {
        let text = std::fs::read_to_string(&journal).unwrap_or_default();
        let line = text
            .lines()
            .find(|l| walked.iter().all(|n| l.contains(n.as_str())));
        line.map(str::to_owned)
    };
    let t0 = std::time::Instant::now();
    while act().is_none() && t0.elapsed() < Duration::from_secs(20) {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let line = act().expect("one act names all 24");
    ana.endpoint().close().await;
    let mut named = walked.clone();
    named.sort_by_key(|n| line.find(n.as_str()));
    assert_eq!(named, walked, "the act's order is the walk's");
}

/// **A save by rename is synced, and a directory moved whole** (perf-2 f):
/// Ana's file reaches Ben; she saves it again as editors do, writing a new
/// file beside it and renaming it over the old, and Ben holds the new
/// bytes; then a directory of two files moved into her directory whole
/// reaches Ben's, and moved out whole leaves it. Red with the OS's renames
/// not looked at (events of kind `Modify(Name)` dropped): the save never
/// reaches Ben.
#[tokio::test]
async fn a_save_by_rename_is_synced_and_a_directory_moved_whole() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let first = noise(20_000, b"first save");
    std::fs::write(ana.join("r.txt"), &first).unwrap();
    let at_ben = |rel: &str| std::fs::read(ben.join(rel)).ok();
    assert!(
        until(&group, || at_ben("r.txt").as_ref() == Some(&first)).await,
        "Ana's file reaches Ben"
    );
    let second = noise(21_000, b"second save");
    std::fs::write(ana.join(".r.txt.swp"), &second).unwrap();
    std::fs::rename(ana.join(".r.txt.swp"), ana.join("r.txt")).unwrap();
    assert!(
        until(&group, || at_ben("r.txt").as_ref() == Some(&second)).await,
        "Ana's save by rename reaches Ben: {:?}",
        names(&ben)
    );
    let moved = ra.path().join("moved");
    std::fs::create_dir_all(&moved).unwrap();
    let (one, two) = (noise(5_000, b"one"), noise(6_000, b"two"));
    std::fs::write(moved.join("one.bin"), &one).unwrap();
    std::fs::write(moved.join("two.bin"), &two).unwrap();
    std::fs::rename(&moved, ana.join("moved")).unwrap();
    let both = || {
        at_ben("moved/one.bin").as_ref() == Some(&one)
            && at_ben("moved/two.bin").as_ref() == Some(&two)
    };
    assert!(
        until(&group, both).await,
        "a directory moved in whole reaches Ben"
    );
    std::fs::rename(ana.join("moved"), &moved).unwrap();
    let gone = || at_ben("moved/one.bin").is_none() && at_ben("moved/two.bin").is_none();
    assert!(
        until(&group, gone).await,
        "a directory moved out whole leaves Ben's"
    );
    group.close().await;
}

/// **A file written as the step takes it is sent as it was taken, and the
/// next edit still reaches a member**: Ana rewrites a file whole, 100 MiB in
/// one write, so the step its truncate wakes stats it mid-write and takes it
/// after; Ben holds it, then Ana's small edit of it. Red with the file taken
/// as its stat before the take said (perf-2 h, the targets' pass): in
/// release, an act said 0 bytes of a 100 MiB hash, Ben's file never matched
/// what he held, and Ana's edit after it never reached him; in debug, the
/// store's import panics on a file that grew past its own stat, and the
/// rewrite reaches no one.
#[tokio::test]
async fn a_file_written_as_it_is_taken_is_sent_as_taken_and_the_next_edit_arrives() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let at_ben = || std::fs::read(ben.join("big.bin")).ok();
    let first = noise(4_000, b"first");
    std::fs::write(ana.join("big.bin"), &first).unwrap();
    assert!(
        until(&group, || at_ben().as_ref() == Some(&first)).await,
        "Ana's file reaches Ben"
    );
    // Truncated, then written while the step woken by the truncate looks.
    let file = std::fs::File::create(ana.join("big.bin")).unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    let big = noise(100 << 20, b"big");
    let written = big.clone();
    tokio::task::spawn_blocking(move || std::io::Write::write_all(&mut &file, &written))
        .await
        .unwrap()
        .unwrap();
    assert!(
        until(&group, || at_ben().as_ref() == Some(&big)).await,
        "Ana's rewrite reaches Ben"
    );
    let edit = noise(5_000, b"edit");
    std::fs::write(ana.join("big.bin"), &edit).unwrap();
    assert!(
        until(&group, || at_ben().as_ref() == Some(&edit)).await,
        "Ana's edit after it reaches Ben"
    );
    group.close().await;
}

/// **A holder asleep is waited on for a bound, then the next asked**: Ana
/// writes, Ben's directory takes it, Ana's node goes away; Cy, syncing
/// after, asks Ana first, as the writer, and holds the file from Ben within
/// 10 s. Red with the holder waited on without
/// [`svrngs_node::node::ANSWER_WITHIN`] (`unbounded`, group folders' B6):
/// Cy waits on Ana, and holds nothing.
#[tokio::test]
async fn a_holder_asleep_is_waited_on_for_a_bound_then_the_next_asked() {
    let group = Group::new().await;
    let (ra, rb, rc) = (
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
        tempfile::tempdir().unwrap(),
    );
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let bytes = noise(60_000, b"asleep");
    std::fs::write(ana.join("e.bin"), &bytes).unwrap();
    let at = |dir: &Path| std::fs::read(dir.join("e.bin")).ok() == Some(bytes.clone());
    assert!(until(&group, || at(&ben)).await, "Ana's file reaches Ben");
    // Cy's record holds Ana's act before Ana goes.
    round(&group.cy).await;
    group.ana.endpoint().close().await;
    let t0 = std::time::Instant::now();
    let cy = both_ways(&group.cy, rc.path()).await;
    // No round: one would wait on Ana too, and Cy's record needs none.
    assert!(
        until_on(&[], || at(&cy)).await,
        "Cy holds Ana's file from Ben"
    );
    assert!(
        t0.elapsed() < Duration::from_secs(10),
        "Cy holds it within 10 s: {:?}",
        t0.elapsed()
    );
    for node in [&group.ben, &group.cy] {
        node.endpoint().close().await;
    }
}

/// **Both sides of a conflict are kept**: Ana and Ben each edit a file both
/// hold, each from the version they last held; every directory then holds
/// both edits, one under the file's name and the other beside it. Red with
/// a path's first head alone placed (sy-1's `later-wins`): one edit lost.
#[tokio::test]
async fn both_sides_of_a_conflict_are_kept() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let first = noise(10_000, b"before");
    std::fs::write(ana.join("c.bin"), &first).unwrap();
    let held = || std::fs::read(ben.join("c.bin")).ok().as_ref() == Some(&first);
    assert!(until(&group, held).await, "Ana's file reaches Ben");

    let (by_ana, by_ben) = (noise(11_000, b"ana"), noise(12_000, b"ben"));
    std::fs::write(ana.join("c.bin"), &by_ana).unwrap();
    std::fs::write(ben.join("c.bin"), &by_ben).unwrap();
    let both: std::collections::BTreeSet<_> = [by_ana, by_ben].into();
    let kept = || contents(&ana) == both && contents(&ben) == both;
    assert!(
        until(&group, kept).await,
        "both edits in both directories: Ana's {:?}, Ben's {:?}",
        std::fs::read_dir(&ana)
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect::<Vec<_>>(),
        std::fs::read_dir(&ben)
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect::<Vec<_>>()
    );
    group.close().await;
}

// ── A synced folder is one group's (syp-4 a) ──────────────────────────

/// The names of the files in `dir`.
fn names(dir: &Path) -> std::collections::BTreeSet<String> {
    let files = std::fs::read_dir(dir).unwrap().flatten();
    files
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
}

/// **A synced folder is one group's**: Ana stands in `films` with Ben and
/// in `house` with Cy, and syncs a directory `Shared` in each. Her file in
/// `films`'s reaches Ben's and neither Cy's nor her own `house` directory;
/// Cy's in `house`'s reaches her `house` directory and not her `films` one;
/// Cy, a member of `house` alone, asks Ana's provider for her `films` file
/// and is refused, while Ben is given it. Red with the per-group rule out
/// (the act written, the layout read and the provider asked across every
/// group held here, as sy-1 did): Cy is given the bytes, and each file
/// reaches both of Ana's directories.
#[tokio::test]
async fn a_synced_folder_is_one_group_s() {
    let group = Group::computers().await;
    let films = found("films", 9, &group.ana, &[(&group.ben, 2)]);
    let house = found("house", 7, &group.ana, &[(&group.cy, 3)]);
    group.settle().await;
    let roots: Vec<_> = (0..4).map(|_| tempfile::tempdir().unwrap()).collect();
    let ana_films = synced_in(&group.ana, roots[0].path(), &films).await;
    let ana_house = synced_in(&group.ana, roots[1].path(), &house).await;
    let ben = synced_in(&group.ben, roots[2].path(), &films).await;
    let cy = synced_in(&group.cy, roots[3].path(), &house).await;

    let by_ana = noise(30_000, b"films");
    std::fs::write(ana_films.join("f.bin"), &by_ana).unwrap();
    let by_cy = noise(20_000, b"house");
    std::fs::write(cy.join("h.bin"), &by_cy).unwrap();
    let arrived = || {
        std::fs::read(ben.join("f.bin")).ok().as_ref() == Some(&by_ana)
            && std::fs::read(ana_house.join("h.bin")).ok().as_ref() == Some(&by_cy)
    };
    assert!(until(&group, arrived).await, "each file reaches its group");
    // Three steps more, for what would arrive where it must not.
    for _ in 0..3 {
        group.settle().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let one = |dir: &Path, name: &str| names(dir) == [name.to_string()].into();
    assert!(
        one(&ana_films, "f.bin"),
        "Ana's films: {:?}",
        names(&ana_films)
    );
    assert!(
        one(&ana_house, "h.bin"),
        "Ana's house: {:?}",
        names(&ana_house)
    );
    assert!(one(&ben, "f.bin"), "Ben's films: {:?}", names(&ben));
    assert!(one(&cy, "h.bin"), "Cy's house: {:?}", names(&cy));

    let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(&by_ana).as_bytes());
    let asked = tempfile::tempdir().unwrap();
    let to_cy = get(&group.cy, &group.ana, hash, &asked.path().join("cy")).await;
    assert!(to_cy.is_err(), "Ana refuses Cy her films file: {to_cy:?}");
    let to_ben = get(&group.ben, &group.ana, hash, &asked.path().join("ben")).await;
    assert_eq!(to_ben, Ok(30_000), "Ana serves Ben her films file");
    group.close().await;
}

/// **A group whose record cannot be read is said so, never read as
/// none**: Ana's file reaches Ben's directory; then Ben's journal of the
/// group cannot be read, and his steps stop by the group's name, the file
/// left on his disk. Red with the record read as placing nothing when it
/// cannot be read (`admission(ns).ok()`, #117): Ben's apply deletes it,
/// "the record placing nothing there".
#[tokio::test]
async fn a_group_whose_record_cannot_be_read_is_said_so_never_read_as_none() {
    use std::os::unix::fs::PermissionsExt as _;
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let bytes = noise(25_000, b"kept");
    std::fs::write(ana.join("k.bin"), &bytes).unwrap();
    let at_ben = || std::fs::read(ben.join("k.bin")).ok();
    assert!(
        until(&group, || at_ben().as_ref() == Some(&bytes)).await,
        "Ana's file reaches Ben"
    );
    let journal = group
        .ben
        .rail()
        .group_dir(&seed("films", 9).id())
        .unwrap()
        .join("ring_oplog.jsonl");
    std::fs::set_permissions(&journal, std::fs::Permissions::from_mode(0o000)).unwrap();
    assert!(
        group.ben.rail().admission(&seed("films", 9).id()).is_err(),
        "the plant: Ben's record of the group cannot be read"
    );
    tokio::time::sleep(Duration::from_secs(4)).await;
    let kept = at_ben();
    std::fs::set_permissions(&journal, std::fs::Permissions::from_mode(0o644)).unwrap();
    let len = kept.as_ref().map(Vec::len);
    assert!(kept == Some(bytes), "Ben's file stays on his disk: {len:?}");
    group.close().await;
}

// ── A store keeps what the layout places (syp-4 b) ────────────────────

/// The bytes `node`'s store holds of `hash` on disk: its data file's
/// length, or none. A file past 16 KiB is never inlined in the store's
/// database (iroh-blobs 0.103 `InlineOptions::default`).
fn stored(node: &Node, hash: &[u8; 32]) -> Option<u64> {
    let name = format!("{}.data", iroh_blobs::Hash::from_bytes(*hash).to_hex());
    let at = node.rail().root().join("blobs/data").join(name);
    std::fs::metadata(at).ok().map(|m| m.len())
}

/// **A store keeps what the layout places, and no more**: Ana writes a
/// file and edits it, and each version reaches Ben; then neither store
/// holds the first version, and each holds the edit's, in bytes. Ana's
/// earlier edit met by Ben's, both kept as heads, stays in both stores.
/// Red with nothing dropped (sy-1: every version copied and kept): the
/// first version's bytes in both stores.
#[tokio::test]
async fn a_store_keeps_what_the_layout_places_and_no_more() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let first = noise(200_000, b"first");
    std::fs::write(ana.join("m.bin"), &first).unwrap();
    let at_ben = || std::fs::read(ben.join("m.bin")).ok();
    assert!(
        until(&group, || at_ben().as_ref() == Some(&first)).await,
        "the first version reaches Ben"
    );
    let edit = noise(210_000, b"edit");
    std::fs::write(ana.join("m.bin"), &edit).unwrap();
    assert!(
        until(&group, || at_ben().as_ref() == Some(&edit)).await,
        "the edit reaches Ben"
    );
    let (v1, v2) = (
        *blake3::hash(&first).as_bytes(),
        *blake3::hash(&edit).as_bytes(),
    );
    let dropped = || {
        [&group.ana, &group.ben]
            .iter()
            .all(|n| stored(n, &v1).is_none())
    };
    until(&group, dropped).await;
    for (who, node) in [("Ana", &group.ana), ("Ben", &group.ben)] {
        assert_eq!(
            stored(node, &v1),
            None,
            "{who}'s store drops the first version"
        );
        assert_eq!(
            stored(node, &v2),
            Some(210_000),
            "{who}'s store keeps the edit's"
        );
    }

    let (by_ana, by_ben) = (noise(30_000, b"ana's side"), noise(40_000, b"ben's side"));
    std::fs::write(ana.join("m.bin"), &by_ana).unwrap();
    std::fs::write(ben.join("m.bin"), &by_ben).unwrap();
    let both: std::collections::BTreeSet<_> = [by_ana.clone(), by_ben.clone()].into();
    assert!(
        until(&group, || contents(&ana) == both && contents(&ben) == both).await,
        "both sides kept"
    );
    for _ in 0..3 {
        group.settle().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    for (who, node) in [("Ana", &group.ana), ("Ben", &group.ben)] {
        for side in [&by_ana, &by_ben] {
            let hash = *blake3::hash(side).as_bytes();
            assert_eq!(
                stored(node, &hash),
                Some(side.len() as u64),
                "{who} keeps each side"
            );
        }
    }
    group.close().await;
}

// ── A path under the folder, parsed (syp-4 c) ─────────────────────────

/// **A member's act naming a path outside the folder is refused at
/// admission**: Ben's file reaches Ana's directory; then an act of Ben's,
/// signed by his key, naming those bytes at `../outside` reaches Ana's
/// record. Her record reads it as a malformed line naming the path, and
/// nothing is written beside her directory. Red with the parse taking any
/// text: Ana writes Ben's bytes at `../outside`, beside her directory.
#[tokio::test]
async fn a_member_s_act_naming_a_path_outside_the_folder_is_refused_at_admission() {
    use svrngs_core::tests_support::after_in;
    use svrngs_core::{act_id, body_json, sign_ring_op, Change, Op, SignedOp, SyncPath};
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let bytes = noise(20_000, b"outside");
    std::fs::write(ben.join("in.bin"), &bytes).unwrap();
    let at_ana = || std::fs::read(ana.join("in.bin")).ok();
    assert!(
        until(&group, || at_ana().as_ref() == Some(&bytes)).await,
        "Ben's file reaches Ana"
    );

    // Ben's act after his last, its path written `../outside` and signed
    // again, as a build that does not parse it could write it.
    let ns = seed("films", 9).id();
    let journal = |n: &Node| n.rail().group_dir(&ns).unwrap().join("ring_oplog.jsonl");
    let ben_key = actor_of(&signing(2));
    let last: Op<SignedOp> = std::fs::read_to_string(journal(&group.ben))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Op<SignedOp>>(l).unwrap())
        .rfind(|op| op.actor == ben_key)
        .unwrap();
    let hash = svrngs_core::ContentHash::from_bytes(*blake3::hash(&bytes).as_bytes());
    let act = RailAct::Synced {
        folder: "Shared".into(),
        changes: vec![Change {
            path: SyncPath::parse("PLACEHOLDER").unwrap(),
            now: Some((hash, bytes.len() as u64)),
            base: None,
        }],
    };
    let held = after_in(&ns, &signing(2), &last, last.ts_unix + 1, act);
    let body = body_json(&held.kind.act, None, None).replace("PLACEHOLDER", "../outside");
    let prev = held.kind.prev.as_ref();
    let id = act_id(&ns, &ben_key, held.kind.seq, prev, held.ts_unix, &body);
    let line = serde_json::to_string(&held)
        .unwrap()
        .replace("PLACEHOLDER", "../outside")
        .replace(&held.id.to_string(), &id.to_string())
        .replace(&held.kind.sig, &sign_ring_op(&signing(2), &id));
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(journal(&group.ana))
        .unwrap();
    std::io::Write::write_all(&mut file, format!("{line}\n").as_bytes()).unwrap();

    let gaps = format!("{:?}", group.ana.rail().admission(&ns).unwrap().gaps());
    tokio::time::sleep(Duration::from_secs(3)).await;
    let outside = ra.path().join("outside");
    assert!(
        !outside.exists(),
        "nothing is written beside Ana's directory: {gaps}"
    );
    assert!(
        gaps.contains("MalformedLine") && gaps.contains("`../outside` is not under the folder"),
        "Ana's record names the line it refused: {gaps}"
    );
    group.close().await;
}

// ── What a node keeps of its syncing (syp-4 e) ────────────────────────

/// **A kept state that cannot be read is refused, never read as empty**:
/// two files reach both Ana's and Ben's directories; then Ben's kept state
/// for the folder cannot be parsed, and his node stops syncing that
/// directory, by its name, rather than send every file again with no base.
/// No directory holds a conflict. Red with the kept state read as empty
/// when it cannot be read (sy-1): every file of Ben's sent again with no
/// base, and every directory holds each beside itself as a conflict.
#[tokio::test]
async fn a_kept_state_that_cannot_be_read_is_refused_never_read_as_empty() {
    let group = Group::new().await;
    let (ra, rb) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (ana, ben) = (
        both_ways(&group.ana, ra.path()).await,
        both_ways(&group.ben, rb.path()).await,
    );
    let (one, two) = (noise(20_000, b"one"), noise(21_000, b"two"));
    std::fs::write(ana.join("one.bin"), &one).unwrap();
    std::fs::write(ana.join("two.bin"), &two).unwrap();
    let both: std::collections::BTreeSet<_> = [one, two].into();
    assert!(
        until(&group, || contents(&ben) == both).await,
        "Ana's files reach Ben"
    );
    let kept = group
        .ben
        .rail()
        .root()
        .join(format!("synced-{}-Shared.json", seed("films", 9).id()));
    assert!(kept.exists(), "Ben's kept state: {}", kept.display());
    // Twice, 300 ms apart: a step under way at the first write would
    // write its own state over it as it ends.
    for _ in 0..2 {
        std::fs::write(&kept, b"not the kept state").unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    for _ in 0..4 {
        group.settle().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let conflicted = |dir: &Path| {
        names(dir)
            .into_iter()
            .filter(|n| n.contains("(conflict"))
            .count()
    };
    assert_eq!(
        (conflicted(&ana), conflicted(&ben)),
        (0, 0),
        "no directory holds a conflict: Ana's {:?}, Ben's {:?}",
        names(&ana),
        names(&ben)
    );
    let now = std::fs::read(&kept).unwrap();
    assert!(
        now == b"not the kept state",
        "Ben's node leaves the kept state as it found it: {}",
        String::from_utf8_lossy(&now)
    );
    group.close().await;
}

/// **One directory keeps one group's folder, with one loop** (syp-4 e;
/// #119): asked again for the directory it syncs, Ana's node answers as
/// before and starts nothing more; asked for a second directory named
/// `Shared` in the same group, or for the same directory in another, it
/// refuses by the directory's name. Red with every ask taken (sy-1: a loop
/// each, two over one kept state).
#[tokio::test]
async fn one_directory_keeps_one_group_s_folder() {
    let group = Group::computers().await;
    let films = found("films", 9, &group.ana, &[(&group.ben, 2)]);
    let house = found("house", 7, &group.ana, &[(&group.cy, 3)]);
    let (ra, other) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let dir = synced_in(&group.ana, ra.path(), &films).await;
    let again = ask(
        &group.ana,
        serde_json::json!({ "dir": dir, "group": films }),
    )
    .await;
    assert_eq!(again.0, 200, "asked again, answered as before: {}", again.1);
    let second = other.path().join("Shared");
    std::fs::create_dir_all(&second).unwrap();
    for (d, g) in [(&second, &films), (&dir, &house)] {
        let refused = ask(&group.ana, serde_json::json!({ "dir": d, "group": g })).await;
        assert_eq!(refused.0, 409, "{}", refused.1);
        assert!(
            refused.1.contains("one directory keeps one group's folder"),
            "{}",
            refused.1
        );
    }
    group.close().await;
}

// ── A holder that sends wrong bytes (syp-4 f; gf's row 6) ─────────────

/// What the tests' subscriber writes, kept to be read.
#[derive(Clone, Default)]
struct Said(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for Said {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The encoding of `bytes` a holder sends for a get of them whole, and
/// their hash.
async fn wire_of(bytes: &[u8]) -> (iroh_blobs::Hash, Vec<u8>) {
    let hash = iroh_blobs::Hash::from_bytes(*blake3::hash(bytes).as_bytes());
    let mem = iroh_blobs::store::mem::MemStore::new();
    mem.add_slice(bytes).await.unwrap();
    let all = iroh_blobs::protocol::ChunkRanges::all();
    (hash, mem.export_bao(hash, all).bao_to_vec().await.unwrap())
}

/// Ana's computer in a group with Ben's key admitted, Ben's key an
/// endpoint of this test's, past any store, answering each get with `wire`
/// in `pieces` parts `pause` apart; Ana syncing `Shared` under `root`, and
/// her record placing `size` bytes of `hash` at `f.bin` as hers, written
/// from a store that is not hers, so she asks Ben for them as she would any
/// holder.
struct Asking {
    ana: Node,
    ben: iroh::Endpoint,
    shared: std::path::PathBuf,
    asked: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    _dir: tempfile::TempDir,
}

impl Asking {
    async fn new(
        root: &Path,
        (hash, size): (iroh_blobs::Hash, u64),
        wire: Vec<u8>,
        pieces: usize,
        pause: Duration,
    ) -> Self {
        use svrngs_core::{Change, ContentHash, SyncPath};
        let dir = tempfile::tempdir().unwrap();
        let ana = start(1, dir.path()).await;
        let ns = found("films", 9, &ana, &[]);
        let admit = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse("ben").unwrap(),
                key: actor_of(&signing(2)),
                invite: None,
            },
            on_behalf_of: None,
        };
        ana.rail().commit(&ns, &admit).unwrap().unwrap();
        let relay = RelayConfig::from_parts(Vec::new(), Some("none"))
            .unwrap()
            .on_loopback_only();
        let ben = svrngs_node::guest::build_relayed_endpoint(
            svrngs_node::iroh::SecretKey::from_bytes(&[2; 32]),
            vec![b"/iroh-bytes/4".to_vec()],
            &relay,
        )
        .await
        .unwrap();
        let asked = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (serving, counted) = (ben.clone(), asked.clone());
        let part = wire.len().div_ceil(pieces);
        tokio::spawn(async move {
            while let Some(incoming) = serving.accept().await {
                let Ok(conn) = incoming.await else { continue };
                let (wire, counted) = (wire.clone(), counted.clone());
                tokio::spawn(async move {
                    while let Ok((mut send, mut recv)) = conn.accept_bi().await {
                        recv.read_to_end(1 << 16).await.ok();
                        counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        for (i, piece) in wire.chunks(part).enumerate() {
                            if i > 0 {
                                tokio::time::sleep(pause).await;
                            }
                            send.write_all(piece).await.ok();
                        }
                        send.finish().ok();
                    }
                });
            }
        });
        let book = MemoryLookup::new();
        let mut addr = EndpointAddr::new(ben.id());
        for a in loopback_sockets(&ben) {
            addr = addr.with_ip_addr(a);
        }
        book.add_endpoint_info(addr);
        ana.endpoint().address_lookup().unwrap().add(book);

        let shared = synced_in(&ana, root, &ns).await;
        let act = RailAct::Synced {
            folder: "Shared".into(),
            changes: vec![Change {
                path: SyncPath::parse("f.bin").unwrap(),
                now: Some((ContentHash::from_bytes(*hash.as_bytes()), size)),
                base: None,
            }],
        };
        let written = svrngs_core::tests_support::Fixture {
            act,
            on_behalf_of: None,
        };
        ana.rail().commit(&ns, &written).unwrap().unwrap();
        Asking {
            ana,
            ben,
            shared,
            asked,
            _dir: dir,
        }
    }

    fn asked(&self) -> usize {
        self.asked.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn close(self) {
        self.ben.close().await;
        self.ana.endpoint().close().await;
    }
}

/// **A holder that sends a flipped byte is refused by name**: Ana's record
/// places a file her store lacks, and Ben's key, the group's other member,
/// answers her fetch with the file's encoding one byte flipped on the wire
/// — a provider of this test's, past any store. Ana's fetch refuses it naming Ben and the
/// mismatch, and her directory never holds the file. Red against a fetch
/// that does not check (the response read raw and written): the flipped
/// bytes in her directory.
#[tokio::test]
async fn a_holder_that_sends_a_flipped_byte_is_refused_by_name() {
    let said = Said::default();
    let writer = said.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("folder=debug")
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let _said = tracing::subscriber::set_default(subscriber);

    // Ben's key, answering any get with the file's encoding, flipped.
    let ra = tempfile::tempdir().unwrap();
    let bytes = noise(10_000, b"flipped");
    let (hash, mut wire) = wire_of(&bytes).await;
    assert_eq!(
        wire.len(),
        8 + 10_000,
        "one chunk group: its size, then its bytes"
    );
    wire[8 + 5_000] ^= 1;
    let asking = Asking::new(ra.path(), (hash, 10_000), wire, 1, Duration::ZERO).await;
    for _ in 0..50 {
        if asking.asked() >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    tokio::time::sleep(Duration::from_millis(500)).await;
    let log = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let refusal = log
        .lines()
        .find(|l| {
            l.contains("a holder gave nothing of what the store lacks")
                && l.contains(&actor_of(&signing(2)))
        })
        .unwrap_or_default()
        .to_string();
    let (held, asked) = (names(&asking.shared), asking.asked());
    asking.close().await;
    assert!(asked >= 1, "Ben's key was asked: {log}");
    assert!(
        held.is_empty(),
        "Ana's directory never holds the flipped file: {held:?}"
    );
    assert!(
        refusal.contains("Mismatch") || refusal.contains("mismatch"),
        "the fetch names Ben and the mismatch: {refusal:?}"
    );
}

/// **A holder slower than [`svrngs_node::node::ANSWER_WITHIN`] to its last
/// byte still answers** (perf-2 a): Ben's key sends a file of 200,000 bytes
/// in four parts 1.5 s apart, 4.5 s to its last byte, each part within the
/// bound of the last; Ana's directory holds the file, byte for byte. Red
/// with the bound on the whole fetch (e25c939's `hold`): Ben taken as asleep
/// at 3 s, every time, and the file never held.
#[tokio::test]
async fn a_holder_slower_than_the_bound_to_its_last_byte_still_answers() {
    let ra = tempfile::tempdir().unwrap();
    let bytes = noise(200_000, b"slow");
    let (hash, wire) = wire_of(&bytes).await;
    let pause = Duration::from_millis(1_500);
    let asking = Asking::new(ra.path(), (hash, 200_000), wire, 4, pause).await;
    let at = asking.shared.join("f.bin");
    let t0 = std::time::Instant::now();
    while std::fs::read(&at).ok().as_deref() != Some(&bytes[..])
        && t0.elapsed() < Duration::from_secs(20)
    {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let (held, took, asked) = (std::fs::read(&at).ok(), t0.elapsed(), asking.asked());
    asking.close().await;
    assert!(
        held.as_deref() == Some(&bytes[..]),
        "Ana holds Ben's file, 4.5 s to its last byte; asked {asked} times in {took:?}"
    );
}
