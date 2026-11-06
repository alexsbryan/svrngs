//! A keeper changes where a group meets, and only a keeper may: the same
//! group, its people and its files, met at another relay, with the new link
//! carrying it (relay change; relay fallbacks). The act is svrngs-core's
//! `MeetsAt`, which counts only the keeper's newest `Meets`; the command
//! writes it from `link --relay` and mints the link that carries it.
//! Planted: `link_to` not calling `Host::meet`, so the record and the new
//! link keep the old relay.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs::cli::{self, MeetAt, Verb};
use svrngs::group;
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::link::Link;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

fn argv(s: &str) -> Vec<String> {
    std::iter::once("svrngs".to_string())
        .chain(s.split_whitespace().map(str::to_string))
        .collect()
}

async fn computer(dir: &std::path::Path, seed: u8, relay: &RelayConfig) -> Node {
    Node::start(
        dir.to_path_buf(),
        SigningKey::from_bytes(&[seed; 32]),
        relay,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a computer starts")
}

async fn relay_here() -> (Server, String) {
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    let relay = Server::spawn(config).await.expect("a relay on 127.0.0.1");
    let url = format!("http://{}/", relay.http_addr().unwrap());
    (relay, url)
}

fn meets(node: &Node, group: &str) -> Vec<String> {
    node.rail()
        .register(group)
        .unwrap()
        .meets
        .unwrap_or_default()
        .into_iter()
        .map(|u| u.to_string())
        .collect()
}

fn acts(node: &Node, group: &str) -> usize {
    let dir = node.rail().group_dir(group).unwrap();
    std::fs::read_to_string(dir.join("ring_oplog.jsonl"))
        .unwrap()
        .lines()
        .count()
}

/// `link` takes `--relay`, and a keeper changes the group's own relays to
/// exactly what it names; the link it mints carries them, and a member who
/// is not the keeper is refused without writing an act.
#[tokio::test(flavor = "multi_thread")]
async fn a_keeper_changes_where_the_group_meets_and_a_member_may_not() {
    let ((_old, old), (_new, new), (_build, build)) =
        (relay_here().await, relay_here().await, relay_here().await);
    let dirs = [(); 2].map(|()| tempfile::tempdir().unwrap());
    let ours = RelayConfig {
        relay_urls: vec![build.clone()],
        ..cli::reach(MeetAt::Svrngs)
    };

    let made = cli::parse(argv(&format!("new house --as Sam --relay {old}"))).unwrap();
    let sam = computer(dirs[0].path(), 71, &ours).await;
    let Verb::New {
        name,
        person,
        relays,
        ..
    } = made
    else {
        panic!("new parses as new");
    };
    let (seed, printed) = cli::new_group(&sam, name, person.unwrap(), None, &relays, &ours)
        .await
        .expect("Sam makes house");
    let old_link = Link::parse(&printed.client).unwrap();
    assert!(
        old_link.token.relays.iter().any(|r| r.to_string() == old),
        "the first link carries the old relay"
    );

    let new: RelayUrl = new.parse().unwrap();
    let (_, printed_new, _) = cli::link_to(&sam, "house", None, &ours, false, &[new.clone()])
        .await
        .expect("Sam changes the relay and mints a link");
    let new_link = Link::parse(&printed_new.client).unwrap();
    assert!(
        new_link
            .token
            .relays
            .iter()
            .any(|r| r.to_string() == new.to_string()),
        "the new link carries the new relay"
    );
    assert_eq!(meets(&sam, &seed.id()), vec![new.to_string()]);

    // Two phases: keep the old beside the new, then drop it.
    let old: RelayUrl = old.parse().unwrap();
    sam.host()
        .meet(&seed.id(), &[old.clone(), new.clone()])
        .expect("the keeper adds the new relay beside the old");
    assert_eq!(
        meets(&sam, &seed.id()),
        vec![old.to_string(), new.to_string()]
    );
    sam.host()
        .meet(&seed.id(), &[new.clone()])
        .expect("the keeper drops the old relay");
    assert_eq!(meets(&sam, &seed.id()), vec![new.to_string()]);
    let quiet = acts(&sam, &seed.id());
    sam.host()
        .meet(&seed.id(), &[new.clone()])
        .expect("the same set writes nothing");
    assert_eq!(
        acts(&sam, &seed.id()),
        quiet,
        "an unchanged set writes no act"
    );

    let join = cli::parse(argv(&format!("join {} --as Alex", printed_new.client))).unwrap();
    let alex = computer(dirs[1].path(), 72, &ours).await;
    let Verb::Join { link, person, .. } = join else {
        panic!("join parses as join");
    };
    let key = SigningKey::from_bytes(&[72; 32]);
    cli::join_group(&alex, &key, &link, &person.unwrap())
        .await
        .expect("Alex joins by the old link");

    let before = acts(&alex, &seed.id());
    let refused = alex.host().meet(&seed.id(), &[old.clone()]);
    assert!(refused.is_err(), "a member who is not keeper is refused");
    let refused = cli::link_to(&alex, "house", None, &ours, false, &[old.clone()]).await;
    assert!(
        matches!(refused, Err(svrngs::cli::Refusal::Meet(_))),
        "the command refuses before it mints a link"
    );
    assert_eq!(acts(&alex, &seed.id()), before, "the refusal writes no act");
    assert_eq!(
        meets(&alex, &seed.id()),
        vec![new.to_string()],
        "the group still meets where the keeper said"
    );

    sam.endpoint().close().await;
    alex.endpoint().close().await;
}

/// A relay's address that takes each connection and never answers, and
/// how many it has taken.
async fn stalled() -> (svrngs_core::relay::Url, Arc<AtomicUsize>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("https://{}/", listener.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let counting = count.clone();
    tokio::spawn(async move {
        while let Ok((sock, _)) = listener.accept().await {
            counting.fetch_add(1, Ordering::SeqCst);
            std::mem::forget(sock);
        }
    });
    (url.parse().unwrap(), count)
}

/// **Every relay down, the attempt ends bounded and nothing is left
/// running** (KEEP_GOING_AT_HOME "every attempt failing without leaving
/// anything running"; the native half of `join_page.rs`'s
/// `every_route_down_fails_bounded_and_closes_the_endpoint`): the build's
/// relay and both a link names take a connection and never answer. The
/// join refuses in the command's words inside `BOUND`, the same 30 s the
/// browser's is held to, plus the 10 s `ONLINE_WAIT` the command spends
/// first; the endpoint closes as `start` closes it; and with the node
/// dropped, past one `ASKED_EVERY`, no relay is dialled again.
#[tokio::test(flavor = "multi_thread")]
async fn every_relay_down_a_join_ends_bounded_and_leaves_nothing_running() {
    const BOUND: Duration = Duration::from_secs(40);
    let ((build, b), (one, o), (two, t)) = (stalled().await, stalled().await, stalled().await);
    let dir = tempfile::tempdir().unwrap();
    let ours = RelayConfig {
        relay_urls: vec![build.to_string()],
        ..cli::reach(MeetAt::Svrngs)
    };
    let key = SigningKey::from_bytes(&[76; 32]);
    let relays = vec![one, two];
    let node = Node::start_asking(
        dir.path().to_path_buf(),
        key.clone(),
        &ours,
        &relays,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a computer starts");
    let token = svrngs_node::link::Token {
        issuer: SigningKey::from_bytes(&[77; 32]).verifying_key(),
        secret: [7; 16],
        relays,
    };
    let link = Link::parse(&svrngs_node::link::client_link(&token)).unwrap();
    let person = group::Name::parse("Pia").unwrap();

    let began = Instant::now();
    let tried = tokio::time::timeout(BOUND, cli::join_group(&node, &key, &link, &person)).await;
    eprintln!("every relay down: {:?}", began.elapsed());
    let tried = tried.unwrap_or_else(|_| panic!("the join still waits after {BOUND:?}"));
    assert!(matches!(tried, Err(cli::Refusal::IssuerAway)), "{tried:?}");

    let (endpoint, taken) = (node.endpoint().clone(), [b, o, t]);
    tokio::time::timeout(Duration::from_secs(10), endpoint.close())
        .await
        .expect("the endpoint closes bounded");
    assert!(endpoint.is_closed());
    drop(node);
    let count =
        |t: &[Arc<AtomicUsize>; 3]| t.iter().map(|c| c.load(Ordering::SeqCst)).sum::<usize>();
    assert!(count(&taken) > 0, "control: the relays were dialled");
    tokio::time::sleep(Duration::from_secs(1)).await;
    let closed = count(&taken);
    tokio::time::sleep(svrngs_node::tiers::ASKED_EVERY + Duration::from_secs(2)).await;
    assert_eq!(
        count(&taken),
        closed,
        "something still dials after the close"
    );
}

/// **`link --relay` while svrngs runs asks it** (KEEP_GOING_AT_HOME 28): the
/// program, as a person runs it, beside a svrngs holding the data dir's lock
/// and serving the group's record. Sam's asks his: where house meets changes
/// and the link printed carries it. Alex's, not the keeper's, is refused in
/// the keeper rule's words and writes nothing.
#[tokio::test(flavor = "multi_thread")]
async fn link_relay_asks_the_svrngs_running_here() {
    let ((_old, old), (_new, new), (_build, build)) =
        (relay_here().await, relay_here().await, relay_here().await);
    let dirs = [(); 2].map(|()| tempfile::tempdir().unwrap());
    let ours = RelayConfig {
        relay_urls: vec![build.clone()],
        ..cli::reach(MeetAt::Svrngs)
    };
    let sam = computer(dirs[0].path(), 78, &ours).await;
    let (name, person) = (
        group::Name::parse("house").unwrap(),
        group::Name::parse("Sam"),
    );
    let relays = [old.parse().unwrap()];
    let (seed, printed) = cli::new_group(&sam, name, person.unwrap(), None, &relays, &ours)
        .await
        .expect("Sam makes house");
    let Verb::Join { link, person, .. } =
        cli::parse(argv(&format!("join {} --as Alex", printed.client))).unwrap()
    else {
        panic!("join parses as join");
    };
    let alex = computer(dirs[1].path(), 79, &ours).await;
    let key = SigningKey::from_bytes(&[79; 32]);
    cli::join_group(&alex, &key, &link, &person.unwrap())
        .await
        .expect("Alex joins");
    let _held = dirs
        .each_ref()
        .map(|d| cli::claim(d.path()).expect("the lock is free"));
    let link = |dir: &std::path::Path| {
        let new = new.clone();
        let dir = dir.to_path_buf();
        tokio::task::spawn_blocking(move || {
            std::process::Command::new(env!("CARGO_BIN_EXE_svrngs"))
                .args(["link", "house", "--relay", &new])
                .env("SVRNGS_DIR", &dir)
                .env("HOME", &dir)
                .output()
                .unwrap()
        })
    };

    let before = acts(&alex, &seed.id());
    let refused = link(dirs[1].path()).await.unwrap();
    let said = String::from_utf8_lossy(&refused.stderr);
    assert!(!refused.status.success(), "{refused:?}");
    assert!(
        said.contains("only the group's keeper decides where it meets"),
        "{said}"
    );
    assert_eq!(acts(&alex, &seed.id()), before, "the refusal writes no act");

    let done = link(dirs[0].path()).await.unwrap();
    let said = String::from_utf8_lossy(&done.stdout);
    assert!(done.status.success(), "{done:?}");
    assert_eq!(meets(&sam, &seed.id()), vec![new.clone()]);
    let line = said
        .lines()
        .find(|l| l.contains("svrngs://join/"))
        .expect(&said);
    let minted = Link::parse(line.trim()).expect("the link printed is whole");
    assert!(
        minted.token.relays.iter().any(|r| r.to_string() == new),
        "the link printed carries the new relay: {:?}",
        minted.token.relays
    );
    sam.endpoint().close().await;
    alex.endpoint().close().await;
}

/// `link --relay` parses, and a group name is still required.
#[test]
fn link_takes_relay() {
    let parsed = cli::parse(argv("link house --relay http://10.0.0.2:3340/")).unwrap();
    let Verb::Link { relays, .. } = parsed else {
        panic!("link parses as link");
    };
    assert_eq!(relays.len(), 1);
    assert!(cli::parse(argv("link --relay http://10.0.0.2:3340/")).is_err());
}

/// A keeper's change reaches a member on the next exchange, and the member
/// still reaches the keeper.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_folds_the_new_relay_from_the_old_one() {
    let ((_old, old), (_new, new), (_build, build)) =
        (relay_here().await, relay_here().await, relay_here().await);
    let dirs = [(); 2].map(|()| tempfile::tempdir().unwrap());
    let ours = RelayConfig {
        relay_urls: vec![build.clone()],
        ..cli::reach(MeetAt::Svrngs)
    };
    let made = cli::parse(argv(&format!("new house --as Sam --relay {old}"))).unwrap();
    let sam = computer(dirs[0].path(), 73, &ours).await;
    let Verb::New {
        name,
        person,
        relays,
        ..
    } = made
    else {
        panic!("new parses as new");
    };
    let (seed, printed) = cli::new_group(&sam, name, person.unwrap(), None, &relays, &ours)
        .await
        .expect("Sam makes house");
    let join = cli::parse(argv(&format!("join {} --as Alex", printed.client))).unwrap();
    let alex = computer(dirs[1].path(), 74, &ours).await;
    let Verb::Join { link, person, .. } = join else {
        panic!("join parses as join");
    };
    let key = SigningKey::from_bytes(&[74; 32]);
    cli::join_group(&alex, &key, &link, &person.unwrap())
        .await
        .expect("Alex joins");

    let new: RelayUrl = new.parse().unwrap();
    sam.host()
        .meet(&seed.id(), &[new.clone()])
        .expect("Sam changes the relay");
    let at = svrngs_node::iroh::EndpointAddr::new(sam.endpoint().id())
        .with_relay_url(new.clone())
        .with_relay_url(build.parse().unwrap());
    let synced = alex.host().sync_with(&seed.id(), &at).await;
    assert!(synced.stop.is_none(), "Alex reaches Sam: {:?}", synced.stop);
    assert_eq!(meets(&alex, &seed.id()), vec![new.to_string()]);

    sam.endpoint().close().await;
    alex.endpoint().close().await;
}
