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
/// asking only this computer's own shared folders (Ben refuses Cy).
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
        refused.is_err(),
        "Dee, in no group, is given nothing: {refused:?}"
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
