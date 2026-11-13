//! `svrngs open --print` on the wire: install.rs's claim that a second
//! browser is told how to come in, and can, asked of a node's routes in
//! this process (cli.rs `Here`; docs/seams.tsv). The pasted line that
//! installs and starts svrngs stays install.rs's.

use axum::body::Body;
use axum::http::Request;
use svrngs::cli::{self, Verb};
use svrngs_node::node::Wire;
use tower::ServiceExt;

use crate::{computer, no_relay, Running};

/// `method` of `path` at `host`, asked of `node`'s routes as a browser on
/// this computer asks them, with `origin` as its page's: status, whether
/// it gave the origin its key (session.rs `redeem`), and the body.
async fn browsed(
    node: &svrngs_node::node::Node,
    (method, host, path): (&str, &str, &str),
    origin: Option<&str>,
) -> (u16, bool, String) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("host", host);
    if let Some(origin) = origin {
        req = req.header("origin", origin);
    }
    let mut req = req.body(Body::empty()).unwrap();
    let caller = std::net::SocketAddr::from(([127, 0, 0, 1], 50_000));
    req.extensions_mut()
        .insert(axum::extract::ConnectInfo(caller));
    let answer = node.api_routes().oneshot(req).await.unwrap();
    let status = answer.status().as_u16();
    let body = axum::body::to_bytes(answer.into_body(), usize::MAX).await;
    let body = String::from_utf8_lossy(&body.unwrap()).into_owned();
    (status, body.contains("\"key\":"), body)
}

/// **A second browser is told how to come in, and can** (give-session):
/// it has no session, so the home page refuses it with a page that says
/// what that is and to run `svrngs open --print`; that shows an open,
/// launching nothing, and the open, redeemed as a browser's bootstrap
/// redeems it from its own origin, lets it in with a key. Watched failing
/// on two plants: the open read from no field of the answer (cli.rs
/// `open`), and the refusal page's `svrngs open --print` line gone
/// (session.rs).
#[tokio::test]
async fn another_browser_is_told_how_to_come_in_and_can() {
    let wire = Wire::default();
    let alex = computer(&wire, 2).await;
    let host = format!("svrngs.localhost:{}", alex.node.api_addr().port());
    let (_, _, refused) = browsed(&alex.node, ("GET", &host, "/"), None).await;
    assert!(refused.contains("svrngs open --print"), "{refused}");
    assert!(refused.contains("A program, not a browser"), "{refused}");

    let verb = cli::parse(["svrngs", "open", "--print"].map(String::from)).unwrap();
    assert!(matches!(verb, Verb::Open(None, true)), "{verb:?}");
    let said = cli::answer(verb, &Running(&alex), &no_relay()).await;
    let said = said.expect("an open, shown");
    assert!(
        !said.contains("is open in your browser"),
        "nothing launched: {said}"
    );
    let open = said
        .split_once("within a minute: ")
        .map(|(_, at)| at.trim())
        .unwrap_or_else(|| panic!("no open shown: {said}"));
    let (at, path) = open
        .strip_prefix("http://")
        .unwrap()
        .split_once('/')
        .unwrap();
    let origin = format!("http://{at}");
    let path = format!("/{path}");
    let (status, keyed, body) = browsed(&alex.node, ("POST", at, &path), Some(&origin)).await;
    assert_eq!((status, keyed), (200, true), "the open lets it in: {body}");
}
