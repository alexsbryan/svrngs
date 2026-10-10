//! Shared folders (gf-1, group folders' second spike; DESIGN §9): Ana
//! shares a folder, Ben's computer is the meeting point and Cy is in the
//! group. Ben's endpoint answers a range of Ana's file with her bytes,
//! fetched by verified range; refuses a range whose bytes do not match the
//! hash, naming Ana; and Ana's provider refuses a hash she stopped sharing
//! though Ben's record still names it. Three nodes in one process, no relay,
//! each reached by the addresses seeded in the others' lookup.

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

/// Ana's group, Ben and Cy admitted, each computer knowing where the
/// others listen, for the round and for a dial by key alike.
struct Group {
    ana: Node,
    ben: Node,
    cy: Node,
    id: String,
    _dirs: Vec<tempfile::TempDir>,
}

impl Group {
    async fn new() -> Self {
        let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
        let ana = start(1, dirs[0].path()).await;
        let ben = start(2, dirs[1].path()).await;
        let cy = start(3, dirs[2].path()).await;
        let seed = Seed::from_parts(
            Name::parse("films").unwrap(),
            Name::parse("ana").unwrap(),
            signing(1).verifying_key(),
            [9; 16],
        );
        let id = seed.id();
        for node in [&ana, &ben, &cy] {
            node.rail().set_seed(&id, &seed.roster()).unwrap();
            let dir = node.rail().group_dir(&id).unwrap();
            std::fs::write(dir.join("seed.json"), seed.to_text()).unwrap();
        }
        for (name, n) in [("ben", 2), ("cy", 3)] {
            let admit = svrngs_core::tests_support::Fixture {
                act: RailAct::Admit {
                    person: Person::parse(name).unwrap(),
                    key: actor_of(&signing(n)),
                    invite: None,
                },
                on_behalf_of: None,
            };
            ana.rail().commit(&id, &admit).unwrap().unwrap();
        }
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
            id,
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

/// `node`'s record API asked to share `dir` or stop `name`, as the command
/// asks it: the folder its act names.
async fn ask(node: &Node, body: serde_json::Value) -> serde_json::Value {
    let answer = http()
        .post(format!("http://{}/v1/folders", node.api_addr()))
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .unwrap();
    assert!(
        answer.status().is_success(),
        "{}",
        answer.text().await.unwrap()
    );
    answer.json().await.unwrap()
}

/// What `node`'s record API lists: its endpoint, folders, unanswered.
async fn listed(node: &Node) -> serde_json::Value {
    let answer = http()
        .get(format!("http://{}/v1/folders", node.api_addr()))
        .send()
        .await
        .unwrap();
    answer.json().await.unwrap()
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

/// Ben's endpoint asked for `[start, end]` of `hash`: status, body.
async fn get_range(ben: &Node, hash: &str, start: u64, end: u64) -> (u16, Vec<u8>) {
    let at = listed(ben).await["endpoint"].as_str().unwrap().to_string();
    let answer = tokio::time::timeout(
        Duration::from_secs(30),
        http()
            .get(format!("{at}{hash}"))
            .header("range", format!("bytes={start}-{end}"))
            .send(),
    )
    .await
    .expect("the endpoint answers")
    .unwrap();
    let status = answer.status().as_u16();
    (
        status,
        answer.bytes().await.map(|b| b.to_vec()).unwrap_or_default(),
    )
}

/// Ana's folder of one 3 MiB file, a link inside it to a file beside it.
fn folder_of(root: &Path) -> (std::path::PathBuf, Vec<u8>) {
    let dir = root.join("shared");
    std::fs::create_dir_all(&dir).unwrap();
    let film = noise(3 << 20, b"film");
    std::fs::write(dir.join("film.bin"), &film).unwrap();
    std::fs::write(root.join("beside.bin"), noise(1 << 16, b"beside")).unwrap();
    std::os::unix::fs::symlink(root.join("beside.bin"), dir.join("link.bin")).unwrap();
    (dir, film)
}

/// **A range at the meeting point is the holder's bytes, verified**: Ana
/// shares a folder; its act names the file and not the link beside it;
/// after Ben's round, Ben's endpoint answers a range spanning two pieces
/// with exactly Ana's bytes, and a hash no folder names with 404. Planted:
/// the walk following links (`SVRNGS_GF_PLANT=follow-links`, the act names
/// the file beside the folder); the endpoint reading its store without
/// fetching (the range is not held).
#[tokio::test]
async fn a_range_at_the_meeting_point_is_the_holder_s_bytes_verified() {
    let group = Group::new().await;
    let root = tempfile::tempdir().unwrap();
    let (dir, film) = folder_of(root.path());
    let shared = ask(&group.ana, serde_json::json!({ "share": dir })).await;
    let paths: Vec<&str> = shared["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        vec!["film.bin"],
        "the link is not shared: {shared:#}"
    );
    let hash = shared["files"][0]["hash"].as_str().unwrap().to_string();
    assert_eq!(hash, blake3::hash(&film).to_hex().as_str());
    round(&group.ben).await;

    let (start, end) = ((1 << 20) - 5, (2 << 20) + 100);
    let (status, body) = get_range(&group.ben, &hash, start, end).await;
    assert_eq!(status, 206);
    assert!(
        body == film[start as usize..=end as usize],
        "the range is Ana's bytes"
    );
    let (status, _) = get_range(&group.ben, &"0".repeat(64), 0, 9).await;
    assert_eq!(status, 404, "a hash no folder names is refused");
    group.close().await;
}

/// **A holder whose bytes do not match the hash is refused by name**: Ana
/// shares her file, then a byte of it flips on her disk (her store took it
/// by reference); Cy holds nothing; Ben's endpoint refuses the range 502,
/// naming Ana's key and why, and sends none of it. Planted: the first
/// piece's refusal dropped (the endpoint answers 206 and fails mid-body).
#[tokio::test]
async fn a_holder_whose_bytes_do_not_match_the_hash_is_refused_by_name() {
    let group = Group::new().await;
    let root = tempfile::tempdir().unwrap();
    let (dir, mut film) = folder_of(root.path());
    let shared = ask(&group.ana, serde_json::json!({ "share": dir })).await;
    let hash = shared["files"][0]["hash"].as_str().unwrap().to_string();
    film[100_000] ^= 0xff;
    std::fs::write(dir.join("film.bin"), &film).unwrap();
    round(&group.ben).await;

    let (status, body) = get_range(&group.ben, &hash, 0, 200_000).await;
    let said: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
    assert_eq!(status, 502, "{said:#}");
    assert_eq!(
        said["holders"][0][0],
        actor_of(&signing(1)),
        "Ana is named: {said:#}"
    );
    assert_eq!(said["hash"], hash.as_str());
    group.close().await;
}

/// **A holder's provider refuses a hash she stopped sharing**: Ana shares,
/// Ben fetches the first piece, Ana stops; Ben, his record not yet synced,
/// still names her folder and asks her for the next piece, and her provider
/// refuses it: 502 naming Ana. After Ben's round the hash is 404 at his
/// endpoint. Planted: the provider serving any hash its store holds
/// (`SVRNGS_GF_PLANT=unchecked`, the second ask answers 206).
#[tokio::test]
async fn a_holder_s_provider_refuses_a_hash_she_stopped_sharing() {
    let group = Group::new().await;
    let root = tempfile::tempdir().unwrap();
    let (dir, film) = folder_of(root.path());
    let shared = ask(&group.ana, serde_json::json!({ "share": dir })).await;
    let hash = shared["files"][0]["hash"].as_str().unwrap().to_string();
    round(&group.ben).await;
    let (status, body) = get_range(&group.ben, &hash, 0, 1000).await;
    assert_eq!((status, body.as_slice()), (206, &film[..=1000]));

    // Written to her record without the nudge `stop` gives, so Ben's round
    // has not carried it yet, as when Ben's last round came before it.
    let stop = svrngs_core::ShareFolder {
        name: "shared".into(),
        files: Vec::new(),
    };
    group.ana.rail().commit(&group.id, &stop).unwrap().unwrap();
    let (status, body) = get_range(&group.ben, &hash, 2 << 20, (2 << 20) + 9).await;
    let said: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
    assert_eq!(
        status, 502,
        "Ana's provider refuses what she stopped sharing"
    );
    assert_eq!(said["holders"][0][0], actor_of(&signing(1)), "{said:#}");
    round(&group.ben).await;
    let (status, _) = get_range(&group.ben, &hash, 0, 9).await;
    assert_eq!(status, 404, "a stopped folder names no hash at Ben's");
    assert_eq!(listed(&group.ben).await["folders"], serde_json::json!([]));
    group.close().await;
}

// ── A folder synced both ways (sy-1, the mesh-Dropbox spike) ──────────

/// `who` syncs a directory of its own named `Shared`, as `share --folder
/// <dir> --both-ways` asks: the directory.
async fn both_ways(who: &Node, root: &Path) -> std::path::PathBuf {
    let dir = root.join("Shared");
    std::fs::create_dir_all(&dir).unwrap();
    let said = ask(who, serde_json::json!({ "bothways": dir })).await;
    assert_eq!(said["name"], "Shared");
    dir
}

/// Up to 30 s, rounds run on every node: whether `done` came true.
async fn until(group: &Group, done: impl Fn() -> bool) -> bool {
    for _ in 0..30 {
        if done() {
            return true;
        }
        for node in [&group.ana, &group.ben, &group.cy] {
            round(node).await;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    done()
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
