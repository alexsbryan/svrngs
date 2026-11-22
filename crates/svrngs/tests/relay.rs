//! `svrngs relay` (relay-verb (b)): a relay the verb starts on 127.0.0.1,
//! over the data of a computer that runs svrngs beside it, lets through a
//! key standing in a group that computer holds and a key a pass of the
//! group's names, itself or by its holder's link, and refuses one that
//! stands in none and holds no pass;
//! `--open` lets both through; a computer in no group is refused the verb.
//! Planted: every key let through, as an open relay would.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
mod common;

#[path = "common/made.rs"]
mod made;
use iroh::dns::DnsResolver;
use iroh_relay::client::{ClientBuilder, ConnectError};
use svrngs::cli::{self, Refusal, Refused, Unjudged};
use svrngs::group::Name;
use svrngs::relay::{self, At, Relay, SHARE_BURST, SHARE_RATE};
use svrngs_core::authority::resolve;
use svrngs_core::pass::{presenting, Label, Pass};
use svrngs_node::iroh::{RelayUrl, SecretKey};
use svrngs_node::SigningKey;

const SAM: u8 = 31;

/// The relay asked for, over `root`, on 127.0.0.1.
async fn relay_over(
    root: &std::path::Path,
    open: bool,
) -> Result<iroh_relay::server::Server, Refusal> {
    let at = At::Here("127.0.0.1:0".parse().unwrap());
    relay::spawn(&Relay { at, open }, root).await
}

/// A client at `server` with `key`, presenting `token` when given, as the
/// computers and phones that dial it build one: plain HTTP, but the TLS
/// config built all the same.
async fn client_with(
    server: &iroh_relay::server::Server,
    key: &SecretKey,
    token: Option<&str>,
) -> Result<iroh_relay::client::Client, ConnectError> {
    let url: RelayUrl = format!("http://{}/", server.http_addr().unwrap())
        .parse()
        .unwrap();
    let tls = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_root_certificates(rustls::RootCertStore::empty())
    .with_no_client_auth();
    let builder = ClientBuilder::new(url, key.clone(), DnsResolver::new()).tls_client_config(tls);
    match token {
        Some(token) => builder.auth_token(token),
        None => builder,
    }
    .connect()
    .await
}

/// A client at `server` with the key of `seed`, presenting no token.
async fn client_at(
    server: &iroh_relay::server::Server,
    seed: u8,
) -> Result<iroh_relay::client::Client, ConnectError> {
    client_with(server, &SecretKey::from_bytes(&[seed; 32]), None).await
}

/// Whether the relay at `server` lets `key` through, presenting `token`
/// when given.
async fn admits_with(
    server: &iroh_relay::server::Server,
    key: &SecretKey,
    token: Option<&str>,
) -> bool {
    match client_with(server, key, token).await {
        Ok(_) => true,
        Err(e @ ConnectError::Handshake { .. }) => {
            assert!(e.to_string().contains("relay denied"), "{e}");
            false
        }
        Err(e) => panic!("the relay neither let it through nor refused it: {e:?}"),
    }
}

/// Whether the relay at `server` lets the key of `seed` through.
async fn admits(server: &iroh_relay::server::Server, seed: u8) -> bool {
    admits_with(server, &SecretKey::from_bytes(&[seed; 32]), None).await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_relay_lets_through_its_groups_alone_and_open_everyone() {
    let (_relay, _, dir, sam) = common::sams_computer(SAM).await;
    let house = Name::parse("house").unwrap();
    made::new_group(&sam, house, Name::parse("Sam").unwrap(), None, &[])
        .await
        .expect("Sam makes house");

    let groups = relay_over(dir.path(), false)
        .await
        .expect("Sam's relay starts");
    assert!(
        admits(&groups, SAM).await,
        "Sam's key stands in house, so it is let through"
    );
    assert!(
        !admits(&groups, 99).await,
        "a key in no group of Sam's is refused"
    );

    let open = relay_over(dir.path(), true)
        .await
        .expect("an open relay starts");
    assert!(admits(&open, SAM).await, "an open relay lets Sam through");
    assert!(
        admits(&open, 99).await,
        "an open relay lets a stranger through"
    );
    sam.endpoint().close().await;
}

/// **A group-only relay admits a pass's holder for the group's lends**
/// (self-host; RELAY_COMMONS design 6): Sam's relay, group-only, refuses a
/// stranger's key; the same fresh key presenting the pass Sam signed for
/// it, naming a lend of Sam's, is let through — a lend's page dials from a
/// key that stands in no group, and the pass its person signed is what a
/// group's own relay admits it by, as the door admits it for the pass's
/// lends. A key the pass does not name, presenting it, is refused. Planted,
/// watched red: the pass branch removed, and the holder was refused as a
/// stranger.
#[tokio::test(flavor = "multi_thread")]
async fn a_group_only_relay_admits_a_pass_s_holder_for_the_group_s_lends() {
    let (_relay, _, dir, sam) = common::sams_computer(SAM).await;
    let house = Name::parse("house").unwrap();
    made::new_group(&sam, house, Name::parse("Sam").unwrap(), None, &[])
        .await
        .expect("Sam makes house");
    let groups = relay_over(dir.path(), false)
        .await
        .expect("Sam's relay starts");

    // The phone's fresh key, and the pass Sam signs for it.
    let sam_key = SigningKey::from_bytes(&[SAM; 32]);
    let holder = SecretKey::from_bytes(&[7; 32]);
    let holder_hex = hex::encode(holder.public().as_bytes());
    let app = Label::of(&sam_key.verifying_key().to_bytes(), "film");
    let until = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("this host's clock reads after 1970")
        .as_secs() as i64
        + 3600;
    let pass = Pass::sign(&sam_key, &holder_hex, (app, vec![]), until).encode();

    assert!(
        !admits_with(&groups, &holder, None).await,
        "a key of no group is still refused there"
    );
    assert!(
        admits_with(&groups, &holder, Some(&pass)).await,
        "the pass's holder is let through for the group's lends"
    );
    let other = SecretKey::from_bytes(&[8; 32]);
    assert!(
        !admits_with(&groups, &other, Some(&pass)).await,
        "the pass names its holder, and another key presenting it is refused"
    );
    sam.endpoint().close().await;
}

/// Seconds since 1970 on this host's clock.
fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("this host's clock reads after 1970")
        .as_secs() as i64
}

/// `pass`, and the link `holder` signs in its own name carrying it to
/// `run`'s key, as a lend's worker has its holder sign one for the key of
/// its run (web/lend.js `linkTo`), where `holder` is the pass's holder.
fn linked(pass: &Pass, holder: &SigningKey, run: &SecretKey) -> (Pass, Option<Pass>) {
    let mut link = pass.link(&hex::encode(run.public().as_bytes()));
    link.person = svrngs_core::actor_of(holder);
    link.sig = svrngs_core::sign_message(holder, &link.message());
    (pass.clone(), Some(link))
}

/// What a lend's worker presents at its relays for a pass and its link:
/// web/src/lib.rs `holder_endpoint`'s own call.
fn token((pass, link): &(Pass, Option<Pass>)) -> String {
    presenting(&pass.encode(), link.as_ref().map(Pass::encode).as_deref())
}

/// **A worker dialing from its run's key reaches a lender at its group's
/// own relay** (relay-link): a lend's worker binds its endpoint as
/// web/src/lib.rs `holder_endpoint` does — relay-only, handed the group's
/// own relay and the build's fallback (here the harness's open relay), its
/// key this run's own (web/lend.js `opened`), presenting its pass and the
/// holder's link naming that key — and dials a lender homed at the group's
/// own relay alone (tiers.rs: a group's own HTTPS relay alone while it
/// answers) at the relay the lender's record names, as lend.js `dialled`
/// does. Planted, watched red: the relay's old rule, the pass's holder the
/// dialing key (`pass.holder == key`), where the dial had no answer.
#[tokio::test(flavor = "multi_thread")]
async fn a_worker_dialing_from_its_run_s_key_reaches_a_lender_at_its_group_s_own_relay() {
    use std::time::Duration;
    const ALPN: &[u8] = b"relay-link/1";
    let (fallback, _, dir, sam) = common::sams_computer(SAM).await;
    let house = Name::parse("house").unwrap();
    made::new_group(&sam, house, Name::parse("Sam").unwrap(), None, &[])
        .await
        .expect("Sam makes house");
    let own = relay_over(dir.path(), false)
        .await
        .expect("Sam's relay starts");
    let own_url: RelayUrl = format!("http://{}/", own.http_addr().unwrap())
        .parse()
        .unwrap();
    let fallback_url: RelayUrl = harness::relay_url(&fallback).parse().unwrap();

    // The lender: Sam's key, homed at house's own relay alone, answering.
    let ours = svrngs_node::iroh::RelayConfig {
        relay_urls: vec![own_url.to_string()],
        n0_services: false,
        loopback_only: true,
    };
    let lender = svrngs_node::iroh::build_relayed_endpoint(
        SecretKey::from_bytes(&[SAM; 32]),
        vec![ALPN.to_vec()],
        &ours,
    )
    .await
    .expect("the lender binds");
    tokio::time::timeout(Duration::from_secs(20), lender.online())
        .await
        .expect("the lender is let through at house's own relay");
    let accepting = lender.clone();
    tokio::spawn(async move {
        let mut answered = Vec::new();
        while let Some(incoming) = accepting.accept().await {
            answered.extend(incoming.await.ok());
        }
    });

    // The pass Sam's page signs to the holder, and this run's key, which
    // the holder names in a link.
    let sam_key = SigningKey::from_bytes(&[SAM; 32]);
    let holder = SigningKey::from_bytes(&[7; 32]);
    let app = Label::of(&sam_key.verifying_key().to_bytes(), "film");
    let pass = Pass::sign(
        &sam_key,
        &svrngs_core::actor_of(&holder),
        (app, vec![]),
        unix_now() + 3600,
    );
    let run = SecretKey::from_bytes(&[9; 32]);
    let presented = token(&linked(&pass, &holder, &run));

    let map: iroh::RelayMap = [own_url.clone(), fallback_url].into_iter().collect();
    let worker = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .crypto_provider(std::sync::Arc::new(rustls::crypto::ring::default_provider()))
        .secret_key(run)
        .relay_mode(iroh::RelayMode::Custom(map.with_auth_token(presented)))
        .clear_ip_transports()
        .bind()
        .await
        .expect("the worker binds");
    let at = iroh::EndpointAddr::new(lender.id()).with_relay_url(own_url);
    let dialed = tokio::time::timeout(Duration::from_secs(20), worker.connect(at, ALPN)).await;
    assert!(
        matches!(dialed, Ok(Ok(_))),
        "the run's key reaches the lender through house's own relay: {dialed:?}"
    );
    worker.close().await;
    lender.close().await;
    sam.endpoint().close().await;
}

/// **A group-only relay admits the key a holder's link names, as the door
/// does, and no other** (relay-link): the run's key presenting the pass and
/// the holder's link naming it is let through; the run's key with the pass
/// alone, with a link from another holder, or with a link in the holder's
/// name another key signed, and another key presenting the run's pass and
/// link, are refused; and on each the door's own call,
/// [`resolve`] for the pass's lend (door.rs `holder_of`), answers the same.
/// Planted, each watched red: the old rule, the pass's holder the dialing
/// key, where the run's key was refused; any key presenting a live pass
/// let through, where the pass alone was let through; and the link judged
/// by itself, not as the pass's holder's word, where another holder's link
/// was.
#[tokio::test(flavor = "multi_thread")]
async fn a_group_only_relay_admits_the_key_a_holder_s_link_names_and_no_other() {
    let (_relay, _, dir, sam) = common::sams_computer(SAM).await;
    let house = Name::parse("house").unwrap();
    made::new_group(&sam, house, Name::parse("Sam").unwrap(), None, &[])
        .await
        .expect("Sam makes house");
    let groups = relay_over(dir.path(), false)
        .await
        .expect("Sam's relay starts");

    let sam_key = SigningKey::from_bytes(&[SAM; 32]);
    let holder = SigningKey::from_bytes(&[7; 32]);
    let stranger = SigningKey::from_bytes(&[6; 32]);
    let app = Label::of(&sam_key.verifying_key().to_bytes(), "film");
    let now = unix_now();
    let pass = Pass::sign(
        &sam_key,
        &svrngs_core::actor_of(&holder),
        (app.clone(), vec![]),
        now + 3600,
    );
    let (run, other) = (
        SecretKey::from_bytes(&[9; 32]),
        SecretKey::from_bytes(&[8; 32]),
    );
    // A link in the holder's name that another key signed.
    let (_, mut forged) = linked(&pass, &holder, &run);
    if let Some(link) = forged.as_mut() {
        link.sig = svrngs_core::sign_message(&stranger, &link.message());
    }
    let cases = [
        (
            "the run's key, the pass and the holder's link naming it",
            &run,
            linked(&pass, &holder, &run),
            true,
        ),
        (
            "the run's key and the pass alone",
            &run,
            (pass.clone(), None),
            false,
        ),
        (
            "the run's key and a link from another holder",
            &run,
            linked(&pass, &stranger, &run),
            false,
        ),
        (
            "the run's key and a link in the holder's name another key signed",
            &run,
            (pass.clone(), forged),
            false,
        ),
        (
            "another key and the run's pass and link",
            &other,
            linked(&pass, &holder, &run),
            false,
        ),
    ];
    for (case, key, pair, admitted) in cases {
        let dialer = hex::encode(key.public().as_bytes());
        let door = resolve(&pair.0, pair.1.as_ref(), &dialer, &app, now, || ()).is_ok();
        assert_eq!(door, admitted, "the door: {case}");
        assert_eq!(
            admits_with(&groups, key, Some(&token(&pair))).await,
            admitted,
            "the relay: {case}"
        );
    }
    sam.endpoint().close().await;
}

/// **A client sending past its burst is held to the share, and told**
/// (relay-share): an open relay holds each computer to [`SHARE_RATE`]
/// after a [`SHARE_BURST`]; a client sending a burst and more to another
/// sees the rest arrive at the rate, and `Status::RateLimited` reaches
/// it. Planted: no limit, where the rest arrives at once and nothing is
/// told.
#[tokio::test(flavor = "multi_thread")]
async fn a_client_sending_past_its_burst_is_held_to_the_share_and_told() {
    use iroh_relay::protos::relay::{ClientToRelayMsg, Datagrams, RelayToClientMsg, Status};
    use n0_future::{SinkExt, StreamExt};

    const PAYLOAD: usize = 65_000;
    // The last frames whose arrival rate is measured: the window sits past
    // the burst, so the limiter's one read through an empty bucket is not
    // in it.
    const MEASURED: usize = 6;
    let empty = tempfile::tempdir().unwrap();
    let server = relay_over(empty.path(), true)
        .await
        .expect("an open relay starts");
    let (mut stream, mut sink) = client_at(&server, 1)
        .await
        .expect("the sender is let through")
        .split();
    let (mut back, _) = client_at(&server, 2)
        .await
        .expect("the receiver is let through")
        .split();

    // A burst, and past it half a megabyte: four seconds at the rate.
    let frames = (SHARE_BURST.get() as usize + 500_000).div_ceil(PAYLOAD);
    let to = SecretKey::from_bytes(&[2; 32]).public();
    let receiving = tokio::spawn(async move {
        let (mut seen, mut first, mut last) = (0usize, None, None);
        while seen < frames {
            if let Some(Ok(RelayToClientMsg::Datagrams { .. })) = back.next().await {
                seen += 1;
                if seen == frames - MEASURED {
                    first = Some(std::time::Instant::now());
                }
                last = Some(std::time::Instant::now());
            }
        }
        (first.unwrap(), last.unwrap())
    });
    for _ in 0..frames {
        sink.send(ClientToRelayMsg::Datagrams {
            dst_endpoint_id: to,
            datagrams: Datagrams::from(vec![0u8; PAYLOAD]),
        })
        .await
        .expect("the relay takes the datagram");
    }
    let told = tokio::time::timeout(std::time::Duration::from_secs(20), async {
        while let Some(next) = stream.next().await {
            if matches!(next, Ok(RelayToClientMsg::Status(Status::RateLimited))) {
                return;
            }
        }
        panic!("the sender's stream ended first");
    })
    .await;
    let (first, last) = tokio::time::timeout(std::time::Duration::from_secs(30), receiving)
        .await
        .expect("the datagrams arrive")
        .unwrap();
    let rate = (MEASURED * PAYLOAD) as f64 / last.duration_since(first).as_secs_f64();
    let share = SHARE_RATE.get() as f64;
    assert!(
        rate <= 2.0 * share && rate >= share / 2.0,
        "the tail moved at {rate} B/s, not the share's {share}"
    );
    told.expect("Status::RateLimited reaches the sender");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_computer_in_no_group_is_refused_a_relay_for_its_groups() {
    let empty = tempfile::tempdir().unwrap();
    let refused = relay_over(empty.path(), false).await.map(|_| ());
    assert!(
        matches!(refused, Err(Refusal::Refused(Refused::RelayForNobody))),
        "{refused:?}"
    );
    assert!(
        relay_over(empty.path(), true).await.is_ok(),
        "an open relay needs no group"
    );
}

/// Every line the test below traces on its own thread.
static TRACES: std::sync::Mutex<Vec<u8>> = std::sync::Mutex::new(Vec::new());

struct Kept;

impl std::io::Write for Kept {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        TRACES.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// **Groups that cannot be read are said so, never taken as none**
/// (ralph/REVIEW_FINDINGS.md #128): Sam makes house, and then the
/// directory holding Sam's groups cannot be read. Saying where Sam's
/// computer is reached traces that it could not list them, and a relay for
/// Sam's groups is refused as their not being readable, not as Sam being in
/// none. Planted: the directory's error read as no groups, as before.
#[tokio::test(flavor = "multi_thread")]
async fn groups_that_cannot_be_read_are_said_so_never_taken_as_none() {
    use std::os::unix::fs::PermissionsExt;
    // This thread's own, beside the harness's file, which keeps neither
    // target: say_relay and spawn's refusal run on it.
    let kept = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new("node=debug,relay=debug"))
        .with_writer(|| Kept)
        .with_ansi(false)
        .finish();
    let _kept = tracing::subscriber::set_default(kept);
    let (_relay, _, dir, sam) = common::sams_computer(SAM).await;
    let house = Name::parse("house").unwrap();
    made::new_group(&sam, house, Name::parse("Sam").unwrap(), None, &[])
        .await
        .expect("Sam makes house");
    let rings = dir.path().join("rings");
    std::fs::set_permissions(&rings, std::fs::Permissions::from_mode(0o000)).unwrap();

    assert!(
        !sam.host().say_relay(sam.host().home()),
        "nothing is written in groups not read"
    );
    let refused = relay_over(dir.path(), false).await.map(|_| ());
    std::fs::set_permissions(&rings, std::fs::Permissions::from_mode(0o755)).unwrap();
    let traced = String::from_utf8_lossy(&TRACES.lock().unwrap()).into_owned();
    for said in [
        "node: cannot list the groups held, so this computer's relay is said in none",
        "relay: refused — the groups this computer holds cannot be read",
    ] {
        let of_groups: Vec<&str> = traced.lines().filter(|l| l.contains("group")).collect();
        assert!(
            traced.contains(said),
            "not traced: {said}; of groups: {of_groups:#?}"
        );
    }
    let refused = refused.expect_err("a relay over groups it cannot read is refused");
    assert!(
        matches!(refused, Refusal::Unjudged(Unjudged::GroupsUnread(_))),
        "{refused:?}: {refused}"
    );
    assert!(
        refused.to_string().contains("cannot be read") && !refused.to_string().contains("no group"),
        "{refused}"
    );
    sam.endpoint().close().await;
}

#[test]
fn the_verb_takes_a_name_never_an_address() {
    let parse = |args: &[&str]| {
        cli::parse(
            ["svrngs", "relay"]
                .iter()
                .chain(args)
                .map(|a| a.to_string()),
        )
    };
    assert_eq!(
        parse(&["Relay.Example.org", "--open"]),
        Ok(cli::Verb::Relay(Relay {
            at: At::Name {
                name: "relay.example.org".into(),
                staging: false
            },
            open: true
        }))
    );
    for address in ["203.0.113.7", "[2001:db8::1]"] {
        let said = parse(&[address]).unwrap_err();
        assert!(
            said.contains("is an address, and a relay's certificate is for a name"),
            "{said}"
        );
    }
    assert!(parse(&["localhost"]).is_err());
    assert!(cli::parse(["svrngs", "new", "x", "--open"].map(String::from)).is_err());
}

/// **A relay `--at` an address serves plain HTTP there, and help lists no
/// such flag**: `svrngs relay --at <address>` parses to [`At::Here`], and
/// the relay it asks starts and lets a key through, as one at a name
/// would; `svrngs help` names no `--at`; with `--staging`, with a name
/// beside it, with no port, or on another verb, it is refused. Planted:
/// `--at` listed in USAGE, and `--staging` taken beside it.
#[tokio::test(flavor = "multi_thread")]
async fn a_relay_at_an_address_is_plain_http_and_unlisted() {
    let parse = |args: &[&str]| {
        cli::parse(
            ["svrngs", "relay"]
                .iter()
                .chain(args)
                .map(|a| a.to_string()),
        )
    };
    let Ok(cli::Verb::Relay(asked)) = parse(&["--at", "127.0.0.1:0", "--open"]) else {
        panic!("--at parses to a relay");
    };
    assert_eq!(
        asked,
        Relay {
            at: At::Here("127.0.0.1:0".parse().unwrap()),
            open: true
        }
    );
    let empty = tempfile::tempdir().unwrap();
    let server = relay::spawn(&asked, empty.path())
        .await
        .expect("a relay --at an address starts");
    assert!(
        server.https_addr().is_none(),
        "no certificate at an address"
    );
    assert!(
        admits(&server, SAM).await,
        "an open relay --at lets a key through"
    );

    assert!(!cli::USAGE.contains("--at"), "help lists --at");
    let refused = |args: &[&str], says: &str| {
        let said = parse(args).unwrap_err();
        assert!(said.contains(says), "{args:?}: {said}");
    };
    refused(&["--at", "10.42.3.20:3340", "--staging"], "--staging");
    refused(
        &["relay.example.org", "--at", "10.42.3.20:3340"],
        "not both",
    );
    refused(&["--at", "10.42.3.20"], "an address and port");
    refused(&["--at"], "--at wants a value");
    let elsewhere = cli::parse(["svrngs", "new", "x", "--at", "10.42.3.20:3340"].map(String::from));
    assert_eq!(elsewhere.err(), Some("unknown argument `--at`".to_string()));
}

/// What `scripts/relay-answers.sh` says of `url`: whether it exited 0.
async fn answers(url: String) -> bool {
    let script = harness::repo().join("scripts/relay-answers.sh");
    let ran = tokio::process::Command::new("sh")
        .arg(script)
        .arg(url)
        .output()
        .await
        .expect("sh runs");
    ran.status.success()
}

/// `scripts/relay-answers.sh`, which deploy-relay.sh waits on and
/// release.sh publishes no build meeting its groups at svrngs's without
/// (relay-default (c), relay-park (d); DESIGN §7):
/// a relay the verb starts answers; a page answering 200 at its `/ping`, as
/// a host's wildcard may at the name, does not, nor an address nothing
/// listens at. Planted: the script taking any 200 as a relay.
#[tokio::test(flavor = "multi_thread")]
async fn a_relay_answers_and_a_page_at_its_name_does_not() {
    let empty = tempfile::tempdir().unwrap();
    let relay = relay_over(empty.path(), true)
        .await
        .expect("a relay starts");
    let at = format!("http://{}", relay.http_addr().unwrap());
    assert!(answers(at.clone()).await, "the relay at {at} answers");

    let page = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let page_at = format!("http://{}/", page.local_addr().unwrap());
    tokio::spawn(async move {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        while let Ok((mut stream, _)) = page.accept().await {
            let _ = stream.read(&mut [0; 1024]).await;
            let page = "HTTP/1.1 200 OK\r\ncontent-length: 9\r\nconnection: close\r\n\r\nnot here.";
            let _ = stream.write_all(page.as_bytes()).await;
        }
    });
    assert!(
        !answers(page_at.clone()).await,
        "a page at {page_at} is no relay"
    );

    let nothing = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let nothing_at = format!("http://{}/", nothing.local_addr().unwrap());
    drop(nothing);
    assert!(
        !answers(nothing_at).await,
        "nothing listening answers nothing"
    );
}
