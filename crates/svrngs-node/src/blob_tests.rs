// SPDX-License-Identifier: AGPL-3.0-or-later
//! A get's silences, asked where no route reaches them (#148): an ask
//! whose task panicked, and a lapse this computer was given for bytes its
//! store lost; and a put's bytes through each sweep of the store.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::HashSet;
use std::path::Path as FsPath;

use iroh_blobs::store::fs::{options::Options, FsStore};
use iroh_blobs::store::GcConfig;
use tokio::sync::{mpsc, oneshot, watch};

use super::*;
use crate::folder::{protect, KeepAsk};
use crate::iroh::tests::{loopback_sockets, Said};
use crate::iroh::{EndpointAddr, MemoryLookup, Net as _, RelayConfig};
use crate::node::Node;
use crate::seed::Seed;
use crate::{actor_of, Name, SigningKey};

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn seed_file(dir: &FsPath) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("seed.json")) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// Ana's and Ben's computers on one wire (docs/CHECKS.md, harness 3), no
/// endpoint and no socket, in a group `films` Ana founds and Ben is
/// admitted to: its id. A Blobs get is served by the holder's own provider
/// over an in-memory pipe ([`crate::node::Wire`]).
async fn ana_and_ben(dirs: [&FsPath; 2]) -> (Node, Node, String) {
    let wire = crate::node::Wire::default();
    let mut nodes = Vec::new();
    for (n, dir) in [(1, dirs[0]), (2, dirs[1])] {
        let node = Node::on_wire(&wire, dir.to_path_buf(), signing(n), seed_file);
        nodes.push(node.await.expect("a node starts"));
    }
    in_films(nodes).await
}

/// [`ana_and_ben`] over QUIC on loopback, each knowing where the other
/// listens: for the claim on the connection a fetch rides, which only a
/// QUIC endpoint keeps.
async fn ana_and_ben_over_quic(dirs: [&FsPath; 2]) -> (Node, Node, String) {
    let relay = RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    };
    let mut nodes = Vec::new();
    for (n, dir) in [(1, dirs[0]), (2, dirs[1])] {
        let api = "127.0.0.1:0".parse().unwrap();
        let node = Node::start(dir.to_path_buf(), signing(n), &relay, seed_file, api);
        nodes.push(node.await.expect("a node starts"));
    }
    for who in &nodes {
        let book = MemoryLookup::new();
        for other in &nodes {
            let mut addr = EndpointAddr::new(other.endpoint().id());
            for a in loopback_sockets(other.endpoint()) {
                addr = addr.with_ip_addr(a);
            }
            book.add_endpoint_info(addr);
            let at = loopback_sockets(other.endpoint());
            who.transport().add_known_peer(other.endpoint().id(), at);
        }
        who.endpoint().address_lookup().unwrap().add(book);
    }
    in_films(nodes).await
}

/// `nodes`, Ana's and Ben's, in the group `films` Ana founds and Ben is
/// let in to, each having run a round: its id.
async fn in_films(mut nodes: Vec<Node>) -> (Node, Node, String) {
    let ana = Name::parse("ana").unwrap();
    let seed = Seed::from_parts(
        Name::parse("films").unwrap(),
        ana,
        signing(1).verifying_key(),
        [9; 16],
    );
    let id = seed.id();
    for node in &nodes {
        node.rail().set_seed(&id, &seed.roster()).unwrap();
        let dir = node.rail().group_dir(&id).unwrap();
        std::fs::write(dir.join("seed.json"), seed.to_text()).unwrap();
    }
    // Ben let in by a link Ana's computer mints: no bare `Admit` counts.
    let ana_rail = nodes[0].rail();
    svrngs_core::tests_support::let_in(
        (&id, &actor_of(&signing(1))),
        (&signing(2), "ben", svrngs_core::Device::Computer),
        |mint| {
            ana_rail.commit(&id, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(ana_rail.commit(&id, redeem).unwrap().unwrap()),
    );
    for node in [&nodes[1], &nodes[0]] {
        let round = tokio::time::timeout(Duration::from_secs(30), node.round());
        round.await.expect("a round finishes");
    }
    let ben = nodes.pop().unwrap();
    (nodes.pop().unwrap(), ben, id)
}

/// **An ask whose task panicked is said, never dropped** (#148 a): Ben's
/// computer says never held and Cy's ask panics; both are in the replies,
/// Cy's with why, so the get cannot count every reply as never held with
/// Cy unheard. Red with the plant the finding names, a failed task dropped
/// (the `.flatten()` the ask's join had): one reply, Ben's.
#[tokio::test]
async fn an_ask_whose_task_panicked_is_said_never_dropped() {
    let never: Spawned = tokio::spawn(async { Ok(Holds::NeverHeld) });
    let panicked: Spawned = tokio::spawn(async { panic!("a planted panic in Cy's ask") });
    let actors = ["ben".to_string(), "cy".to_string()];
    let joined = actors
        .into_iter()
        .zip(n0_future::join_all([never, panicked]).await);
    let answers = heard(Hash::new(b"asked"), "films", joined);
    assert_eq!(
        answers.len(),
        2,
        "each computer asked is in the replies: {answers:?}"
    );
    assert_eq!(answers[0], ("ben".into(), Ok(Holds::NeverHeld)));
    let (cy, said) = &answers[1];
    assert_eq!(cy, "cy");
    assert!(
        said.as_ref().is_err_and(|why| why.contains("failed")),
        "Cy's ask said failed: {said:?}"
    );
}

type Spawned = tokio::task::JoinHandle<Result<Holds, String>>;

/// **A lapse this computer was given outlasts a fetch that failed** (#148
/// b): Ana and Ben each carry a hash for a minute, and neither store holds
/// its bytes (the store lost what the sweep keeps). Ana's get finds Ben
/// holding it, the fetch from him fails, and the get is refused
/// unanswered; Ana still carries the hash, so asked again she says it is
/// held, never "never held". Red with the plant the finding names, every
/// failed fetch forgetting the entry: Ana says never held.
#[tokio::test]
async fn a_lapse_this_computer_was_given_outlasts_a_fetch_that_failed() {
    let dirs = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
    let (ana, ben, films) = ana_and_ben([dirs[0].path(), dirs[1].path()]).await;
    let hash = Hash::new(b"bytes neither store holds");
    let until = Instant::now() + Duration::from_secs(60);
    for node in [&ana, &ben] {
        node.folders().carry(hash, &films, Some(until));
    }
    let asked = BlobQuery {
        group: films.clone(),
        keep: None,
    };
    let answer = get(State(ana.folders().clone()), Path(hash), Query(asked))
        .await
        .into_response();
    assert_eq!(
        answer.status(),
        StatusCode::BAD_GATEWAY,
        "the fetch from Ben fails"
    );
    let held = ana.folders().held(hash, &films, Instant::now());
    assert!(
        matches!(held, Holds::For { .. }),
        "Ana still carries it: {held:?}"
    );
    for node in [ana, ben] {
        node.net().close().await;
    }
}

/// **A refused put, and a get that asks the group, are traced** (#149):
/// Ana puts bytes naming no `keep`, and is refused; then she gets a hash
/// she carries but her store lacks, and her computer asks the group. A
/// person reading her trace finds each: the refusal and its words, and the
/// get asking with her lapse given. Red with the plant the finding names,
/// the refusal and the branch untraced (`err` and the get as they were):
/// neither in the trace.
#[tokio::test]
async fn a_refused_put_and_a_get_that_asks_the_group_are_traced() {
    let said = Said::default();
    let writer = said.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("rails=debug,blob=debug")
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let _said = tracing::subscriber::set_default(subscriber);
    // A second dispatcher kept alive: with one, tracing-core 0.1.36
    // (callsite.rs:545, `has_just_one`) asks only the registering thread's,
    // so a callsite another test's thread reaches first is cached unheard.
    let _two = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
    let dirs = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
    let (ana, ben, films) = ana_and_ben([dirs[0].path(), dirs[1].path()]).await;

    let unkept = BlobQuery {
        group: films.clone(),
        keep: None,
    };
    let bytes = axum::body::Bytes::from_static(b"kept for how long?");
    let answer = put(State(ana.folders().clone()), Query(unkept), bytes)
        .await
        .into_response();
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);

    let hash = Hash::new(b"carried, not in the store");
    let until = Instant::now() + Duration::from_secs(60);
    ana.folders().carry(hash, &films, Some(until));
    let asked = BlobQuery {
        group: films.clone(),
        keep: None,
    };
    let _ = get(State(ana.folders().clone()), Path(hash), Query(asked)).await;

    let log = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let line = |words: &str| {
        log.lines()
            .find(|l| l.contains(words))
            .unwrap_or_default()
            .to_string()
    };
    let refused = line("&keep=<seconds>");
    assert!(
        refused.contains("refused") && refused.contains("400"),
        "the put's refusal traced: {log}"
    );
    let asking = line("blob: not held here whole");
    assert!(
        asking.contains("given=true") && asking.contains(&hash.to_string()),
        "the get asking the group traced, Ana's lapse given: {log}"
    );
    for node in [ana, ben] {
        node.net().close().await;
    }
}

/// **A fetch rides the connection the transport keeps** (#177): Ben puts
/// two files' bytes, and Ana gets each, so her computer fetches each from
/// Ben on the blobs protocol. Her transport, which keeps a member's
/// connection, keeps that one too, by Ben's key and the protocol, and the
/// second fetch rides it: one rule for both. Red with the plant the finding
/// names, the fetch deciding its own connection (folder.rs `fetch` at
/// c43d2cc7): the transport keeps none.
#[tokio::test]
async fn a_fetch_rides_the_connection_the_transport_keeps() {
    let dirs = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
    // Over QUIC: a kept connection is the transport's own (Q's claim).
    let dirs = [dirs[0].path(), dirs[1].path()];
    let (ana, ben, films) = ana_and_ben_over_quic(dirs).await;
    let at = (
        *ben.endpoint().id().as_bytes(),
        crate::iroh::Alpn::Blobs.bytes(),
    );
    let mut ids = Vec::new();
    for bytes in [&b"the first file"[..], &b"the second file"[..]] {
        let kept = BlobQuery {
            group: films.clone(),
            keep: Some(60),
        };
        let bytes = axum::body::Bytes::from_static(bytes);
        let put = put(State(ben.folders().clone()), Query(kept), bytes.clone())
            .await
            .into_response();
        assert_eq!(put.status(), StatusCode::OK);
        let asked = BlobQuery {
            group: films.clone(),
            keep: None,
        };
        let hash = Hash::new(&bytes);
        let answer = get(State(ana.folders().clone()), Path(hash), Query(asked))
            .await
            .into_response();
        assert_eq!(answer.status(), StatusCode::OK, "Ana gets it from Ben");
        let conns = ana.transport().conns.borrow();
        ids.push(conns.get(&at).map(|c| c.stable_id()));
    }
    assert!(
        ids[0].is_some() && ids[0] == ids[1],
        "Ana's transport keeps one connection to Ben on the blobs protocol, both fetches on it: {ids:?}"
    );
    for node in [ana, ben] {
        node.endpoint().close().await;
    }
}

/// **The bytes a put answered for are kept by every sweep of the store**,
/// the one that asked what to keep before the put included: Ana's folders
/// over a store of their own, whose sweeps' asks this test hands their
/// keeper, and the keeper's answers back. The first sweep's answer, read
/// before Ana puts, reaches it once the put has answered; that sweep keeps
/// the bytes, and so does the next, and her get answers them. Red with the
/// plant the finding names, the put's tag dropped once it is carried (as at
/// 15f458cf): the first sweep drops the bytes.
#[tokio::test]
async fn a_put_is_kept_by_a_sweep_that_asked_before_it() {
    let dirs = [tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap()];
    let (ana, ben, films) = ana_and_ben([dirs[0].path(), dirs[1].path()]).await;
    let blobs = tempfile::tempdir().unwrap();
    let (ask, mut asked) = mpsc::unbounded_channel();
    let mut options = Options::new(blobs.path());
    // The interval only paces the asks: each sweep waits on its answer.
    options.gc = Some(GcConfig {
        interval: Duration::from_millis(1),
        add_protected: Some(protect(ask)),
    });
    let fs = FsStore::load_with_opts(blobs.path().join("blobs.db"), options);
    let fs = fs.await.expect("the store opens");
    // Ana's host, over this store, with what it carries and holds its own.
    let f = Folders {
        store: (*fs).clone(),
        carried: watch::Sender::new(BTreeMap::new()),
        fresh: watch::Sender::new(Vec::new()),
        ..ana.folders().clone()
    };
    let (to_keeper, keep_asked) = mpsc::unbounded_channel();
    tokio::spawn(f.clone().keeper(keep_asked));
    let kept = || {
        let (answer, answered) = oneshot::channel();
        to_keeper.send(answer).unwrap();
        async { answered.await.expect("the keeper answers") }
    };

    let first: KeepAsk = asked.recv().await.expect("the store's first sweep asks");
    let keep = kept().await;
    assert_eq!(
        keep,
        Some(HashSet::new()),
        "nothing kept yet, so the sweep goes on"
    );
    let bytes = axum::body::Bytes::from_static(b"put while a sweep was under way");
    let query = BlobQuery {
        group: films.clone(),
        keep: Some(60),
    };
    let answer = put(State(f.clone()), Query(query), bytes.clone()).await;
    assert_eq!(answer.into_response().status(), StatusCode::OK);
    let hash = Hash::new(&bytes);
    first.send(keep).unwrap();

    // Each sweep asks only once the last is done (iroh-blobs `run_gc`).
    let second: KeepAsk = asked.recv().await.expect("the store sweeps again");
    assert!(
        f.store.has(hash).await.unwrap(),
        "the sweep that asked before the put kept its bytes"
    );
    second.send(kept().await).unwrap();
    let _third = asked.recv().await.expect("the store sweeps again");
    let query = BlobQuery {
        group: films.clone(),
        keep: None,
    };
    let got = get(State(f.clone()), Path(hash), Query(query))
        .await
        .into_response();
    assert_eq!(got.status(), StatusCode::OK, "the next sweep kept them too");
    let body = axum::body::to_bytes(got.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body, bytes, "Ana's get answers what she put");
    fs.shutdown().await.ok();
    for node in [ana, ben] {
        node.net().close().await;
    }
}
