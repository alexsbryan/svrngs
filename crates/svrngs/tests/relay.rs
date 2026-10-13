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
