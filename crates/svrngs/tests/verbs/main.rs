//! The command's verbs on the wire (docs/CHECKS.md, harness 3; the
//! ledger's unit 7): each claim whose decision is a verb's, its function
//! called in this process, as `cli::new_group` is, on computers met on one
//! [`Wire`]: no endpoint, no relay, no socket. What stays on a real one is
//! each source file's seam: a process asked while it runs, a relay, a
//! listener (docs/seams.tsv names where each claim went).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod home;
mod join;
mod leave;
mod meet;
mod new;
mod open;
mod remove;
mod share;

use std::net::SocketAddr;

use axum::http::StatusCode;
use svrngs::cli;
use svrngs::group::{self, Name};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::{Node, Wire};
use svrngs_node::SigningKey;
use tower::ServiceExt;

/// svrngs-node's tests' shared helpers: `heard`, a test's own trace.
#[path = "../../../svrngs-node/tests/common/mod.rs"]
#[allow(dead_code)]
mod node_common;

/// One person's computer on a [`Wire`]: its node, its key and its data dir.
pub struct Computer {
    pub node: Node,
    pub key: SigningKey,
    pub dir: tempfile::TempDir,
}

/// The computer `seed` signs as, on `wire`, its groups' seeds kept as the
/// command keeps them.
pub async fn computer(wire: &Wire, seed: u8) -> Computer {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[seed; 32]);
    let node = Node::on_wire(
        wire,
        dir.path().to_path_buf(),
        key.clone(),
        group::seed_text,
    )
    .await
    .expect("a node starts on the wire");
    Computer { node, key, dir }
}

/// What the command is given when no relay is named: none, here.
pub fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
}

pub fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

/// `method` of `path` asked of `node`'s record API, as the command asks svrngs
/// running here (cli.rs `ask_running`): its token shown, the caller on
/// loopback, the routes asked by `oneshot`. The status and the body.
pub async fn ask(
    node: &Node,
    method: &str,
    path: &str,
    body: &serde_json::Value,
) -> (StatusCode, String) {
    let req = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("host", node.api_addr().to_string())
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", node.token().trim()));
    let mut req = req.body(axum::body::Body::from(body.to_string())).unwrap();
    let caller = SocketAddr::from(([127, 0, 0, 1], 50_000));
    req.extensions_mut()
        .insert(axum::extract::ConnectInfo(caller));
    let answer = node.api_routes().oneshot(req).await.unwrap();
    let status = answer.status();
    let body = axum::body::to_bytes(answer.into_body(), usize::MAX).await;
    (status, String::from_utf8(body.unwrap().to_vec()).unwrap())
}

/// svrngs running over `0`'s data dir as the command reaches it
/// (cli.rs `Here`): each ask answered by its node's routes, as [`ask`]
/// asks them, the API's socket aside.
pub struct Running<'a>(pub &'a Computer);

impl cli::Here for Running<'_> {
    fn root(&self) -> &std::path::Path {
        self.0.dir.path()
    }
    async fn ask(&self, route: &str, body: &impl serde::Serialize) -> cli::Asked {
        let body = serde_json::to_value(body).unwrap();
        let (status, text) = ask(&self.0.node, "POST", route, &body).await;
        Ok((status.is_success(), text))
    }
}
