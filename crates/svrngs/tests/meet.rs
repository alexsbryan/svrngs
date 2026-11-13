//! Where a group meets, across real relays: a join with every relay down
//! ends bounded and leaves nothing running, a computer whose public relay
//! is gone joins at home. A keeper's change, a member's fold and `link
//! --relay` asked of the svrngs running here are verbs/meet.rs's, on the
//! wire (docs/seams.tsv).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs::cli::{self, MeetAt, Verb};
use svrngs::group;
use svrngs_node::iroh::RelayConfig;
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
    let node = Node::start_every(
        (dir.path().to_path_buf(), Some(dir.path().join("svrngs"))),
        key.clone(),
        &ours,
        &relays,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
        svrngs_node::ring_sync::DEFAULT_RING_SYNC_INTERVAL,
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

/// **A computer with the program joins at home with the internet out**
/// (KEEP_GOING_AT_HOME 3, 1): Sam's group meets at the home relay and his
/// build's public relay is gone, so he has moved. Pia's program knows only
/// the same gone public relay, and her link names both. She is in, as
/// `svrngs join` joins, within the bound the refusal is held to. A planted
/// bad input, kept as the control: the same link naming the public relay
/// alone, as an old invitation does, leaves the issuer away.
#[tokio::test(flavor = "multi_thread")]
async fn the_program_joins_at_home_with_the_public_relay_gone() {
    const BOUND: Duration = Duration::from_secs(40);
    let ((_home, home), (public, at_public)) = (relay_here().await, relay_here().await);
    let dirs = [(); 3].map(|()| tempfile::tempdir().unwrap());
    let ours = RelayConfig {
        relay_urls: vec![at_public.clone()],
        ..cli::reach(MeetAt::Svrngs)
    };
    let sam = computer(dirs[0].path(), 81, &ours).await;
    let made = cli::parse(argv(&format!("new house --as Sam --relay {home}"))).unwrap();
    let Verb::New {
        name,
        person,
        relays,
        ..
    } = made
    else {
        panic!("new parses as new");
    };
    let (_seed, printed) = cli::new_group(&sam, name, person.unwrap(), None, &relays, &ours)
        .await
        .expect("Sam makes house");
    let link = Link::parse(&printed.client).unwrap();
    let names: Vec<String> = link.token.relays.iter().map(ToString::to_string).collect();
    assert!(
        names.contains(&home) && names.contains(&at_public),
        "both ways in: {names:?}"
    );

    public.shutdown().await.unwrap();
    let every = svrngs_node::tiers::ASKED_EVERY * (svrngs_node::tiers::LEFT_AFTER + 2);
    tokio::time::timeout(every, async {
        while !sam
            .endpoint()
            .addr()
            .relay_urls()
            .any(|r| r.to_string() == home)
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("Sam moves to the home relay");

    let mut old = link.clone();
    old.token.relays.retain(|r| r.to_string() == at_public);
    let late = computer(dirs[1].path(), 82, &ours).await;
    let key = SigningKey::from_bytes(&[82; 32]);
    let lee = group::Name::parse("Lee").unwrap();
    let refused = cli::join_group(&late, &key, &old, &lee);
    let refused = tokio::time::timeout(BOUND, refused).await.expect("bounded");
    assert!(
        matches!(refused, Err(cli::Refusal::IssuerAway)),
        "{refused:?}"
    );

    let pia = computer(dirs[2].path(), 83, &ours).await;
    let key = SigningKey::from_bytes(&[83; 32]);
    let person = group::Name::parse("Pia").unwrap();
    let began = Instant::now();
    let joined = tokio::time::timeout(BOUND, cli::join_group(&pia, &key, &link, &person))
        .await
        .expect("the join ends inside the refusal's bound");
    eprintln!("joined at home in {:?}", began.elapsed());
    assert!(matches!(joined, Ok(cli::Said::Joined { .. })), "{joined:?}");
    sam.endpoint().close().await;
    pia.endpoint().close().await;
    late.endpoint().close().await;
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
