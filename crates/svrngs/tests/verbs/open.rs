//! `svrngs open --print` on the wire: install.rs's claim that a second
//! browser is told how to come in, and can, asked of a node's routes in
//! this process (cli.rs `Here`; docs/seams.tsv). The pasted line that
//! installs and starts svrngs stays install.rs's.

use std::cell::RefCell;
use std::ffi::{OsStr, OsString};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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
    async fn ask(&self, route: &str, body: &impl serde::Serialize) -> cli::Asked {
        self.0.ask(route, body).await
    }
    fn listens(&self, port: std::num::NonZeroU16) -> bool {
        self.0.listens(port)
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

/// The file a `file://` address `handed` names.
fn page_of(handed: &str) -> PathBuf {
    assert!(
        handed.starts_with("file:///"),
        "not a page's address: {handed}"
    );
    let url = reqwest::Url::parse(handed).unwrap();
    url.to_file_path().unwrap()
}

/// The address the page at `page` sends its browser on to, as its script
/// gives it to `location.replace`.
fn sent_on_to(page: &Path) -> String {
    let text = std::fs::read_to_string(page).unwrap();
    let (_, rest) = text.split_once("location.replace(").expect("a script");
    let (json, _) = rest.split_once(')').unwrap();
    serde_json::from_str(json).unwrap()
}

fn mode(path: &Path) -> u32 {
    std::fs::symlink_metadata(path)
        .unwrap()
        .permissions()
        .mode()
        & 0o777
}

/// **The open reaches the browser by a page only this account reads, never
/// as the opener's argument** (svrn-docs MARKETPLACE_REVIEW_SVRNGS.md §3):
/// any account on the computer reads a program's arguments (`ps`, Linux's
/// `/proc/<pid>/cmdline`), so the opener is handed the `file://` address of
/// a page in a folder of the data dir, the folder 0700 and the page 0600,
/// and the page sends the browser on to the open, which still lets it in
/// once. A page older than five minutes is cleared at the next open, a newer
/// one kept. Watched failing on cli.rs before the hand-off: the opener was
/// handed the open's address, its code in it.
#[tokio::test]
async fn the_open_reaches_the_browser_by_a_private_page_never_an_argument() {
    let wire = Wire::default();
    let alex = computer(&wire, 2).await;
    let opener = Opener(Running(&alex), RefCell::default());
    let (said, handed) = opener.open().await;
    assert_eq!(said.unwrap(), "Your home page is open in your browser.\n");
    let handed = handed.expect("the opener was handed something");
    assert!(
        !handed.contains("code="),
        "the code in an argument: {handed}"
    );
    let first = page_of(&handed);
    let folder = first.parent().unwrap();
    assert_eq!((mode(folder), mode(&first)), (0o700, 0o600), "{handed}");
    let open = sent_on_to(&first);
    let (_, code) = open.split_once("/svrngs-open?code=").expect(&open);
    assert!(!handed.contains(code), "the code in an argument: {handed}");
    let (at, path) = open
        .strip_prefix("http://")
        .unwrap()
        .split_once('/')
        .unwrap();
    let origin = format!("http://{at}");
    let path = format!("/{path}");
    let (status, keyed, body) = browsed(&alex.node, ("POST", at, &path), Some(&origin)).await;
    assert_eq!(
        (status, keyed),
        (200, true),
        "the page's open lets it in: {body}"
    );

    let aged = std::fs::File::options().write(true).open(&first).unwrap();
    let long_ago = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1);
    aged.set_modified(long_ago).unwrap();
    let fresh = folder.join("fresh.html");
    std::fs::write(&fresh, "a page an open wrote a moment ago").unwrap();
    let (said, handed) = opener.open().await;
    assert!(said.is_ok(), "{said:?}");
    let second = page_of(&handed.unwrap());
    assert_ne!(second, first);
    assert!(
        second.exists() && fresh.exists(),
        "a page newer than five minutes cleared"
    );
    assert!(!first.exists(), "a page older than five minutes kept");
}

/// **No folder only this account reads, no open handed over**: where the
/// folder cannot be kept private (here a file stands in its place), `open`
/// refuses, says where and why and how to come in instead, and hands its
/// opener nothing; never the address as an argument. Watched failing on
/// cli.rs before the hand-off, which handed the address.
#[tokio::test]
async fn an_open_with_no_private_folder_to_hand_it_through_is_refused() {
    let wire = Wire::default();
    let alex = computer(&wire, 3).await;
    let opener = Opener(Running(&alex), RefCell::default());
    let (_, handed) = opener.open().await;
    let handed = handed.expect("the opener was handed something");
    let folder = page_of(&handed).parent().unwrap().to_path_buf();
    std::fs::remove_dir_all(&folder).unwrap();
    std::fs::write(&folder, "not a folder").unwrap();
    let (said, handed) = opener.open().await;
    let refused = said
        .expect_err("an open with no private folder")
        .to_string();
    assert_eq!(handed, None, "handed over anyway");
    let named = alex.dir.path().join(folder.file_name().unwrap());
    assert!(refused.contains(&named.display().to_string()), "{refused}");
    assert!(refused.contains("svrngs open --print"), "{refused}");
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
