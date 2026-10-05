//! Joining from a phone (DESIGN §3, "Joining"; §8, U4; u4-5): Sam makes a
//! group whose links open a page on an origin, `svrngs new --origin`, and
//! lends a service. A phone opens the https link printed, in Playwright's
//! headless Chromium: the page asks the person's name once, makes their
//! key, redeems the link at Sam's computer naming itself a browser, keeps
//! the key sealed, and lists what the group's people lend as the bare
//! command prints it. Opening the link again, or the page alone, is in
//! without asking; a lend opens at its address under the page's service
//! worker, which dials its lender's key. Everything
//! runs on this computer: a relay on 127.0.0.1, plain HTTP, Sam's computer
//! reached only through it, and the page served on 127.0.0.1. The browser
//! half is `web/tests/join.mjs`, which fails naming the install command
//! when Playwright or its Chromium is absent — it never skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
mod harness;

use std::num::NonZeroU16;
use std::process::Stdio;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs::cli;
use svrngs::group::{self, Name};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend::{self, Kind, Lend, LendName};
use svrngs_node::link::Origin;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Device, RailAct, SigningKey};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

/// A service on 127.0.0.1 answering every request `200 hi`.
async fn service() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut head = [0u8; 4096];
                let _ = sock.read(&mut head).await;
                let _ = sock
                    .write_all(
                        b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nhi",
                    )
                    .await;
            });
        }
    });
    port
}

/// Each `Admit` in Sam's record: its person, key and device, if it
/// redeems a link.
fn admits(sam: &Node, group: &str) -> Vec<(String, String, Option<Device>)> {
    let file = sam
        .rail()
        .group_dir(group)
        .unwrap()
        .join("ring_oplog.jsonl");
    std::fs::read_to_string(file)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<svrngs_node::Op<svrngs_node::SignedOp>>(l).unwrap())
        .filter_map(|op| match op.kind.act {
            RailAct::Admit {
                person,
                key,
                invite,
            } => Some((person.as_str().to_string(), key, invite.map(|i| i.device))),
            _ => None,
        })
        .collect()
}

/// Planted: the key stored unsealed beside its seal (store.js `keep`
/// writing `raw: key`); a second `Admit` on reopening (app.js joining
/// anew, with a fresh key, on every opening of a link).
#[tokio::test(flavor = "multi_thread")]
async fn a_phone_joins_by_the_link_sam_printed_and_is_in_again_without_asking() {
    let site = harness::site();
    let relay = harness::relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ours = RelayConfig::from_parts(vec![harness::relay_url(&relay)], Some("none"))
        .unwrap()
        .on_loopback_only();
    let sam = Node::start(
        dir.path().to_path_buf(),
        SigningKey::from_bytes(&[1; 32]),
        &ours,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("Sam's computer starts");
    tokio::time::timeout(Duration::from_secs(20), sam.endpoint().online())
        .await
        .expect("Sam's computer reaches the relay on 127.0.0.1");
    let shelf = Lend {
        name: LendName::parse("shelf").unwrap(),
        port: NonZeroU16::new(service().await).unwrap(),
        kind: Kind::parse("shelf").unwrap(),
        limit: NonZeroU16::new(3).unwrap(),
        refused: Vec::new(),
        upload: None,
    };
    lend::declare(dir.path(), &shelf, None).unwrap();

    let mut phone = tokio::process::Command::new("node")
        .arg(harness::repo().join("web/tests/join.mjs"))
        .arg(json!({ "site": site }).to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs: the browser tests need Node 18 or later");
    let mut out = BufReader::new(phone.stdout.take().unwrap()).lines();
    let first = out.next_line().await.unwrap().unwrap_or_else(|| {
        panic!(
            "the browser half printed no origin; it exited {:?}",
            phone.try_wait()
        )
    });
    let origin: Value = serde_json::from_str(&first).unwrap();
    let origin = Origin::parse(origin["origin"].as_str().unwrap()).unwrap();

    let (seed, printed) = cli::new_group(
        &sam,
        Name::parse("house").unwrap(),
        Name::parse("Sam").unwrap(),
        Some(&origin),
        &ours,
    )
    .await
    .expect("Sam makes house, its links on the page's origin");
    let link = printed.https.clone().expect("an https link on the origin");
    eprintln!("the link printed: {link}");
    let mut stdin = phone.stdin.take().unwrap();
    stdin
        .write_all(format!("{}\n", json!({ "link": link, "name": "Pia" })).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let mut stderr = String::new();
    let mut err = phone.stderr.take().unwrap();
    let mut last = None;
    while let Some(line) = out.next_line().await.unwrap() {
        last = Some(line);
    }
    err.read_to_string(&mut stderr).await.unwrap();
    let status = phone.wait().await.unwrap();
    assert!(
        status.success(),
        "the browser half failed ({status}):\n{stderr}"
    );
    let seen: Value = serde_json::from_str(&last.expect("the browser half printed")).unwrap();

    // In: asked the name once, and listed Sam's shelf as the command prints it.
    let first = &seen["first"];
    assert_eq!(
        first["asked"], true,
        "the page asks a newcomer's name:\n{seen:#}"
    );
    assert_eq!(first["status"], "You’re in house, as Pia", "{seen:#}");
    assert_eq!(first["lent"], json!(["Sam’s shelf"]), "{seen:#}");

    // The issuer's journal: one Admit naming a browser's key and that name,
    // and the key the one the page keeps sealed.
    let stored = &seen["stored"];
    let key: [u8; 32] = hex::decode(stored["key"].as_str().unwrap())
        .unwrap()
        .try_into()
        .expect("the page's key is 32 bytes");
    let pia = actor_of(&SigningKey::from_bytes(&key));
    let id = seed.id();
    let founder = actor_of(&SigningKey::from_bytes(&[1; 32]));
    let joined: Vec<_> = admits(&sam, &id)
        .into_iter()
        .filter(|(_, key, _)| *key != founder)
        .collect();
    assert_eq!(
        joined,
        vec![("Pia".to_string(), pia.clone(), Some(Device::Browser))],
        "one Admit, naming a browser's key and the name typed:\n{seen:#}"
    );

    // The key's 32 bytes, unsealed, in no storage; its seal not extractable.
    assert_eq!(
        stored["found"],
        json!([]),
        "the key appears unsealed in storage:\n{stored:#}"
    );
    assert_eq!(stored["extractable"], json!([false]), "{stored:#}");

    // Again, by the link and by the page alone: in, asking nothing.
    for again in ["again", "alone"] {
        let o = &seen[again];
        assert_eq!(o["asked"], false, "{again}: asked again:\n{seen:#}");
        assert_eq!(o["status"], "You’re in house, as Pia", "{again}:\n{seen:#}");
        assert_eq!(o["lent"], json!(["Sam’s shelf"]), "{again}:\n{seen:#}");
    }

    // A lend opens at its address under the service worker's scope, which
    // serves it from Sam's computer, dialled by Sam's key (u4-6).
    assert_eq!(
        seen["opened"],
        json!({ "path": format!("/l/{founder}/shelf/"), "text": "hi" }),
        "{seen:#}"
    );
    sam.endpoint().close().await;
}
