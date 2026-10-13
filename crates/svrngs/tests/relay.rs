//! `svrngs relay` (relay-verb (b)): a relay the verb starts on 127.0.0.1,
//! over the data of a computer that runs svrngs beside it, lets through a
//! key standing in a group that computer holds and refuses one that stands
//! in none; `--open` lets both through; a computer in no group is refused
//! the verb. Planted: every key let through, as an open relay would.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
mod common;

use iroh::dns::DnsResolver;
use iroh_relay::client::{ClientBuilder, ConnectError};
use svrngs::cli::{self, Refusal};
use svrngs::group::Name;
use svrngs::relay::{self, At, Relay};
use svrngs_node::iroh::{RelayUrl, SecretKey};

const SAM: u8 = 31;

/// The relay asked for, over `root`, on 127.0.0.1.
async fn relay_over(
    root: &std::path::Path,
    open: bool,
) -> Result<iroh_relay::server::Server, Refusal> {
    let at = At::Here("127.0.0.1:0".parse().unwrap());
    relay::spawn(&Relay { at, open }, root).await
}

/// Whether the relay at `server` lets the key of `seed` through.
async fn admits(server: &iroh_relay::server::Server, seed: u8) -> bool {
    let url: RelayUrl = format!("http://{}/", server.http_addr().unwrap())
        .parse()
        .unwrap();
    let key = SecretKey::from_bytes(&[seed; 32]);
    // Plain HTTP, but the client builds its TLS config all the same.
    let tls = rustls::ClientConfig::builder_with_provider(std::sync::Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .unwrap()
    .with_root_certificates(rustls::RootCertStore::empty())
    .with_no_client_auth();
    let client = ClientBuilder::new(url, key, DnsResolver::new()).tls_client_config(tls);
    match client.connect().await {
        Ok(_) => true,
        Err(e @ ConnectError::Handshake { .. }) => {
            assert!(e.to_string().contains("relay denied"), "{e}");
            false
        }
        Err(e) => panic!("the relay neither let it through nor refused it: {e:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_relay_lets_through_its_groups_alone_and_open_everyone() {
    let (_relay, ours, dir, sam) = common::sams_computer(SAM).await;
    let house = Name::parse("house").unwrap();
    cli::new_group(&sam, house, Name::parse("Sam").unwrap(), None, None, &ours)
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

#[tokio::test(flavor = "multi_thread")]
async fn a_computer_in_no_group_is_refused_a_relay_for_its_groups() {
    let empty = tempfile::tempdir().unwrap();
    let refused = relay_over(empty.path(), false).await.map(|_| ());
    assert!(
        matches!(refused, Err(Refusal::RelayForNobody)),
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
    let (_relay, ours, dir, sam) = common::sams_computer(SAM).await;
    let house = Name::parse("house").unwrap();
    cli::new_group(&sam, house, Name::parse("Sam").unwrap(), None, None, &ours)
        .await
        .expect("Sam makes house");
    let rings = dir.path().join("rings");
    std::fs::set_permissions(&rings, std::fs::Permissions::from_mode(0o000)).unwrap();

    assert!(
        !sam.host().say_relay(),
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
        matches!(refused, Refusal::GroupsUnread(_)),
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
/// deploy-page.sh publishes no build meeting its groups at svrngs's without
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
