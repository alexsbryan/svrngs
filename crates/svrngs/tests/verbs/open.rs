//! `svrngs open` on the wire: what its opener is handed and who can use it,
//! and install.rs's claim that a second browser is told how to come in, and
//! can, asked of a node's routes in this process (cli.rs `Here`;
//! docs/seams.tsv). The pasted line that installs and starts svrngs stays
//! install.rs's.

use std::cell::RefCell;
use std::ffi::{OsStr, OsString};
use std::path::Path;

use axum::body::Body;
use axum::http::Request;
use svrngs::cli::{self, Verb};
use svrngs_node::node::Wire;
use tower::ServiceExt;

use crate::{computer, no_relay, Running};

/// svrngs running here as [`Running`] answers it, with an opener that takes
/// whatever it is handed and keeps it, as `open` or `xdg-open` would get it
/// in its arguments: no browser is started.
struct Opener<'a>(Running<'a>, RefCell<Vec<OsString>>);

impl cli::Here for Opener<'_> {
    fn root(&self) -> &Path {
        self.0.root()
    }
    async fn asked(&self, route: &str, body: Option<String>) -> cli::Answered {
        self.0.asked(route, body).await
    }
    fn answering(
        &self,
        raw: svrngs_node::lend::Raw,
        port: std::num::NonZeroU16,
        pid: Option<u32>,
    ) -> crate::Answered {
        self.0.answering(raw, port, pid)
    }
    async fn bound(&self, lend: &svrngs_node::lend::Lend) -> crate::Checked {
        self.0.bound(lend).await
    }
    fn prompt(&self, n: &svrngs_node::lend::LendName, a: &str, s: bool) -> cli::Prompted {
        self.0.prompt(n, a, s)
    }
    fn browse(&self, what: &OsStr) -> bool {
        self.1.borrow_mut().push(what.to_owned());
        true
    }
}

impl Opener<'_> {
    /// `svrngs open`, its words, and the one argument its opener was handed.
    async fn open(&self) -> (Result<String, cli::Refusal>, Option<String>) {
        let said = cli::answer(Verb::Open(None, false), self, &no_relay()).await;
        let handed = self.1.borrow_mut().drain(..).collect::<Vec<_>>();
        assert!(handed.len() <= 1, "handed more than once: {handed:?}");
        let handed = handed.first().map(|h| h.to_string_lossy().into_owned());
        (said, handed)
    }
}

/// **The opener is handed the open, and only this account's browser can
/// use it** (svrn-docs LEND_OWNER §3): `svrngs open` gives the program that
/// opens addresses the open's own address, its code in it, where any
/// account can read it in the opener's arguments; posted from a socket of
/// another account, the code lets nothing in and stays unspent, and from
/// this account's it lets the browser in, once (svrngs-node session.rs
/// `Sessions::ours`). Watched failing on ffe54450's page: the opener handed
/// a `file://` page, not the open.
#[tokio::test]
async fn the_opener_is_handed_the_open_and_only_this_account_s_browser_can_use_it() {
    use svrngs_node::uid::Owner;
    let wire = Wire::default();
    let alex = computer(&wire, 2).await;
    let opener = Opener(Running(&alex), RefCell::default());
    let (said, handed) = opener.open().await;
    assert_eq!(said.unwrap(), "Your home page is open in your browser.\n");
    let handed = handed.expect("the opener was handed the open");
    let at = format!("svrngs.localhost:{}", alex.node.api_addr().port());
    let path = handed
        .strip_prefix(&format!("http://{at}"))
        .filter(|p| p.starts_with("/svrngs-open?code="))
        .unwrap_or_else(|| panic!("not the open: {handed}"));
    let origin = format!("http://{at}");
    let mine = svrngs_node::uid::mine().unwrap();
    // `browsed` asks from 127.0.0.1:50000, which the wire says is whose.
    wire.owns(alex.node.net().id(), 50_000, Owner::Uid(mine + 1));
    let (status, keyed, _) = browsed(&alex.node, ("POST", &at, path), Some(&origin)).await;
    assert_eq!((status, keyed), (401, false), "another account's program");
    wire.owns(alex.node.net().id(), 50_000, Owner::Uid(mine));
    let (status, keyed, body) = browsed(&alex.node, ("POST", &at, path), Some(&origin)).await;
    assert_eq!(
        (status, keyed),
        (200, true),
        "this account's browser: {body}"
    );
}

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
    assert!(refused.contains("while Play is on for it"), "{refused}");

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
