// SPDX-License-Identifier: AGPL-3.0-or-later
//! A get's silences, asked where no route reaches them (#148): an ask
//! whose task panicked, and a lapse this computer was given for bytes its
//! store lost.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::Path as FsPath;

use super::*;
use crate::iroh::tests::{loopback_sockets, Said};
use crate::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use crate::node::Node;
use crate::seed::Seed;
use crate::{actor_of, Name, Person, RailAct, SigningKey};

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

/// Ana's and Ben's computers, each knowing where the other listens, in a
/// group `films` Ana founds and Ben is admitted to: its id.
async fn ana_and_ben(dirs: [&FsPath; 2]) -> (Node, Node, String) {
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
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("ben").unwrap(),
            key: actor_of(&signing(2)),
            invite: None,
        },
        on_behalf_of: None,
    };
    nodes[0].rail().commit(&id, &admit).unwrap().unwrap();
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
    let answer = get(State(ana.folders().clone()), Path(hash), Query(asked)).await;
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
        node.endpoint().close().await;
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
    let answer = put(State(ana.folders().clone()), Query(unkept), bytes).await;
    assert_eq!(answer.status(), StatusCode::BAD_REQUEST);

    let hash = Hash::new(b"carried, not in the store");
    let until = Instant::now() + Duration::from_secs(60);
    ana.folders().carry(hash, &films, Some(until));
    let asked = BlobQuery {
        group: films.clone(),
        keep: None,
    };
    get(State(ana.folders().clone()), Path(hash), Query(asked)).await;

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
        node.endpoint().close().await;
    }
}
