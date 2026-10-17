//! Dee's probe, the mesh-Dropbox spike's instrument (sy-1;
//! `demo/runs/sy/2026-10-09/README.md` §2, S4): a node outside the group,
//! on Dee's data dir and iroh's public relays, given a member's key, the
//! group's id and a file's hash, asks that member for the group's record on
//! the member protocol and for the file's bytes on the blobs protocol, and
//! keeps what it is given in its data dir. Each answer is one JSON line on
//! stdout. Run by the spike's script alone, which names what to ask:
//!
//!   SY_DEE_DIR=<dir> SY_MEMBER=<actor hex> SY_GROUP=<namespace> SY_HASH=<hex> \
//!     cargo test -p svrngs-node --test sy_dee -- --ignored --nocapture

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::PathBuf;
use std::time::Duration;

use iroh_blobs::protocol::GetRequest;
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::Hash;
use svrngs_node::alpn::Alpn;
use svrngs_node::iroh::{actor_key, Contact, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::peer_wire::RingSyncRequest;
use svrngs_node::SigningKey;

fn said(ask: &str, answer: serde_json::Value) {
    println!(
        "SY-DEE {}",
        serde_json::json!({ "ask": ask, "answer": answer })
    );
}

#[tokio::test]
#[ignore = "the spike's instrument: run by demo/runs/sy/2026-10-09/run.py"]
async fn dee_asks_a_member() {
    let var = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k} names what to ask"));
    let dir = PathBuf::from(var("SY_DEE_DIR"));
    let (member, group, hash) = (var("SY_MEMBER"), var("SY_GROUP"), var("SY_HASH"));
    let key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
    let node = Node::start(
        dir.clone(),
        key,
        &RelayConfig::default(),
        |_| Ok(None),
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("Dee's node starts");
    let peer = actor_key(&member).expect("a member's key");

    // The record, on the member protocol: its connection's fate, then a
    // round's first ask over the node's own bridge.
    let dial = |alpn: Alpn| {
        tokio::time::timeout(
            Duration::from_secs(20),
            node.endpoint().connect(peer, alpn.bytes()),
        )
    };
    let fate = match dial(Alpn::Member).await {
        Err(_) => "the dial: no answer in 20s".into(),
        Ok(Err(e)) => format!("the dial: {e}"),
        Ok(Ok(c)) => match tokio::time::timeout(Duration::from_secs(5), c.closed()).await {
            Ok(why) => format!("closed: {why}"),
            Err(_) => "open after 5s".into(),
        },
    };
    said("member-connection", fate.into());
    let contact = Contact {
        key: peer,
        relay_url: None,
        iroh_direct_addrs: Vec::new(),
    };
    let answer = match node.transport().spliced(&contact).await {
        None => serde_json::json!({ "error": "no path to the member" }),
        Some(base) => {
            let ask = RingSyncRequest {
                namespace: group.clone(),
                digest: Default::default(),
                ops: Vec::new(),
            };
            let sent = reqwest::Client::new()
                .post(format!("{base}/internal/ring/sync"))
                .json(&ask)
                .timeout(Duration::from_secs(20))
                .send()
                .await;
            match sent {
                Err(e) => serde_json::json!({ "error": format!("{e:?}") }),
                Ok(r) => {
                    let status = r.status().as_u16();
                    let body = r.text().await.unwrap_or_default();
                    std::fs::write(dir.join("dee-record-answer.json"), &body).unwrap();
                    let ops = serde_json::from_str::<serde_json::Value>(&body)
                        .ok()
                        .and_then(|v| v["ops"].as_array().map(|a| a.len()));
                    serde_json::json!({ "status": status, "ops": ops, "bytes": body.len() })
                }
            }
        }
    };
    said("record", answer);

    // The bytes, on the blobs protocol, into a store of the probe's own in
    // Dee's data dir: the node holds `blobs/`.
    let store = FsStore::load(dir.join("asked")).await.expect("Dee's store");
    let hash = Hash::from_bytes(
        *svrngs_node::ContentHash::from_hex(&hash)
            .expect("a hash")
            .as_bytes(),
    );
    let got = match dial(Alpn::Blobs).await {
        Err(_) => serde_json::json!({ "error": "the dial: no answer in 20s" }),
        Ok(Err(e)) => serde_json::json!({ "error": format!("the dial: {e}") }),
        Ok(Ok(conn)) => {
            let get = store.remote().execute_get(conn, GetRequest::blob(hash));
            match tokio::time::timeout(Duration::from_secs(20), get).await {
                Err(_) => serde_json::json!({ "error": "no answer in 20s" }),
                Ok(Err(e)) => serde_json::json!({ "error": format!("{e:?}") }),
                Ok(Ok(stats)) => serde_json::json!({ "payload": stats.payload_bytes_read }),
            }
        }
    };
    said("bytes", got);
    store.shutdown().await.ok();
    node.endpoint().close().await;
}
