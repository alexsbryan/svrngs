//! The relay a new group keeps, by one rule wherever it is made
//! (arch-command-relay (a); AUDIT §2.3, "Two makers each, which already
//! differ"): `svrngs new` with no `--relay` keeps what a group made on the
//! home page keeps, where this computer is met now. At n0's both keep none
//! (`group::kept_relay`), so n0's default cannot tell them apart; here the
//! computer is met at a relay on 127.0.0.1, as every relay is met once
//! `cli::SHIPPED` flips: none of n0's. A binary of its own, its `HOME` a
//! folder of its own, where the page places the group's folder. Planted:
//! `svrngs new` keeping a relay only for `--relay`, as at 75fd900d.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::time::Duration;

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};
use svrngs::cli;
use svrngs::group::{self, Name};
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::node::Node;
use svrngs_node::SigningKey;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The relay the group called `name` keeps on `sam`.
fn kept(sam: &Node, name: &str) -> Option<RelayUrl> {
    let held = group::groups(sam.rail().root());
    let seed = held.iter().find(|s| s.name().as_str() == name);
    let dir = sam.rail().group_dir(&seed.expect(name).id()).unwrap();
    group::read_relay(&dir).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn svrngs_new_keeps_the_relay_a_group_made_on_the_home_page_keeps() {
    let home = tempfile::tempdir().unwrap();
    std::env::set_var("HOME", home.path());
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    let relay = Server::spawn(config).await.expect("a relay on 127.0.0.1");
    let url = format!("http://{}/", relay.http_addr().unwrap());
    let ours = RelayConfig {
        relay_urls: vec![url.clone()],
        n0_services: false,
        loopback_only: true,
    };
    let dir = tempfile::tempdir().unwrap();
    let sam = Node::start(
        dir.path().to_path_buf(),
        SigningKey::from_bytes(&[61; 32]),
        &ours,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("Sam's computer starts");
    tokio::time::timeout(Duration::from_secs(20), sam.endpoint().online())
        .await
        .expect("Sam's computer reaches the relay on 127.0.0.1");

    // "Make a group" on the home page, as its own page posts it, showing
    // the token: the relay kept is what this test holds, and a browser's
    // session is session.rs's (give-session).
    let at = format!("svrngs.localhost:{}", sam.api_addr().port());
    let body = r#"["Lab","Sam"]"#;
    let token = sam.token();
    let request = format!(
        "POST /group HTTP/1.1\r\nHost: {at}\r\nOrigin: http://{at}\r\nSec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        token.trim(),
        body.len()
    );
    let mut sock = tokio::net::TcpStream::connect(sam.api_addr())
        .await
        .unwrap();
    sock.write_all(request.as_bytes()).await.unwrap();
    let mut answer = String::new();
    sock.read_to_string(&mut answer).await.unwrap();
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    let paged = kept(&sam, "Lab");
    assert_eq!(
        paged.as_ref().map(ToString::to_string).as_deref(),
        Some(url.as_str()),
        "the page's group keeps the relay this computer is met at"
    );

    let (name, person) = (Name::parse("Trip").unwrap(), Name::parse("Sam").unwrap());
    cli::new_group(&sam, name, person, None, None, &ours)
        .await
        .expect("Sam makes Trip");
    assert_eq!(kept(&sam, "Trip"), paged, "`svrngs new` keeps the page's");
    sam.endpoint().close().await;
}
