//! What an app asks beside the record (DESIGN §3; the u5-1 row): the groups
//! this computer holds, and what a group's people lend, on the record's
//! loopback listener, whose address the node writes to its data dir.
//! Alex founds a group and lets Sam in; both lend. Two nodes in one
//! process on one wire (common/on_wire.rs; docs/CHECKS.md, harness 3), no
//! endpoint and no socket, each API asked of its routes. The listener
//! itself, and the file naming it, is asked of two nodes over QUIC, each
//! listening on 127.0.0.1 alone (#33's fix: ralph/REVIEW_FINDINGS.md), Sam
//! reached by seeded addresses.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;
use svrngs_node::iroh::Net as _;
#[path = "common/on_wire.rs"]
mod on_wire;

use std::net::SocketAddr;
use std::time::Duration;

use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend;
use svrngs_node::node::Node;
use svrngs_node::seed::Seed;
use svrngs_node::{actor_of, Name, SigningKey};

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// The seed a group's directory keeps, as the command keeps it
/// (`svrngs::group::seed_text`): `seed.json`.
fn seed_file(dir: &std::path::Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("seed.json")) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// `seed`'s computer over QUIC, its API on a loopback port: the
/// listener's own claim.
async fn start_quic(seed: u8, dir: &std::path::Path) -> Node {
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &RelayConfig {
            relay_urls: Vec::new(),
            n0_services: false,
            loopback_only: true,
        },
        seed_file,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// `dir`'s computer lends `name` on `port`, declaring `upload_bps` or none.
fn lends(dir: &std::path::Path, group: &str, name: &str, port: u16, upload_bps: Option<u64>) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let mut body =
        serde_json::json!({ "port": port, "kind": "media", "limit": 3, "groups": [group] });
    if let Some(bps) = upload_bps {
        body["upload_bps"] = bps.into();
    }
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// `dir`'s computer lends `name` on `port`, of `kind`, declaring the kinds
/// it `reads`.
fn declare(dir: &std::path::Path, group: &str, name: &str, port: u16, kind: &str, reads: &[&str]) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = serde_json::json!({ "port": port, "kind": kind, "limit": 3, "reads": reads, "groups": [group] });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// `path` asked of `node`'s API at the origin of the lend `at`, by a
/// program on loopback, with the token; from the page at `from`'s origin,
/// with its browser's fetch headers and independent read capability,
/// when one is named (give-session). Its status and JSON body.
async fn at_origin(
    node: &Node,
    at: &svrngs_core::pass::Label,
    path: &str,
    from: Option<&str>,
) -> (u16, serde_json::Value) {
    use tower::ServiceExt;
    let port = node.api_addr().port();
    // An address a page already holds, capability and all, is asked as it is.
    let path = match from.and_then(svrngs_core::pass::Label::parse) {
        _ if path.starts_with("/svrngs-as/") => path.into(),
        Some(app) if &app != at => format!("/svrngs-as/{}{path}", node.read_capability(&app, at)),
        _ => path.into(),
    };
    let mut request =
        axum::http::Request::get(&path).header("host", format!("{at}.localhost:{port}"));
    if let Some(page) = from {
        if page == at.as_str() {
            let headers = on_wire::proved(node, Some(at.as_str()), "GET", &path, "").await;
            for (name, value) in headers.iter() {
                request = request.header(name, value);
            }
        }
        request = request
            .header("sec-fetch-site", "same-site")
            .header("sec-fetch-mode", "cors")
            .header("origin", format!("http://{page}.localhost:{port}"));
    } else {
        request = request.header("authorization", format!("Bearer {}", node.token().trim()));
    }
    let mut request = request.body(axum::body::Body::empty()).unwrap();
    let caller: SocketAddr = "127.0.0.1:50000".parse().unwrap();
    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(caller));
    let answer = tokio::time::timeout(Duration::from_secs(30), node.api_routes().oneshot(request))
        .await
        .expect("the ask finishes")
        .unwrap();
    let status = answer.status().as_u16();
    let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

/// `body`'s reads with each address's capability taken off, once it is
/// seen there: a read's address is its origin under `/svrngs-as/<cap>/`
/// (give-session; tests/session.rs holds the cap itself).
fn uncapped(mut body: serde_json::Value) -> serde_json::Value {
    for read in body["lends"].as_array_mut().into_iter().flatten() {
        let address = read["address"].as_str().unwrap_or_default();
        let (origin, cap) = address
            .split_once("svrngs-as/")
            .unwrap_or_else(|| panic!("{read}"));
        let cap = cap.strip_suffix('/').unwrap_or_default();
        assert!(
            !cap.is_empty() && !cap.contains('/'),
            "a capability in {address}"
        );
        read["address"] = origin.into();
    }
    body
}

/// Alex's group "house" and Sam in it, each computer holding its seed, and
/// Alex reaching Sam: on this test's wire, or over QUIC.
struct House {
    alex: Node,
    sam: Option<Node>,
    id: String,
    alex_dir: tempfile::TempDir,
    sam_dir: tempfile::TempDir,
    quic: bool,
}

impl House {
    /// Alex's and Sam's computers on this test's wire.
    async fn new() -> Self {
        Self::on(false).await
    }

    /// Alex's and Sam's computers over QUIC, Alex knowing where Sam
    /// listens, each API on a loopback port and asked there.
    async fn over_quic() -> Self {
        Self::on(true).await
    }

    async fn on(quic: bool) -> Self {
        let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let (alex, sam) = if quic {
            let alex = start_quic(1, alex_dir.path()).await;
            let sam = start_quic(2, sam_dir.path()).await;
            alex.transport()
                .add_known_peer(sam.endpoint().id(), loopback_sockets(sam.endpoint()));
            (alex, sam)
        } else {
            let alex = on_wire::start(1, alex_dir.path()).await;
            (alex, on_wire::start(2, sam_dir.path()).await)
        };
        let seed = Seed::from_parts(
            Name::parse("house").unwrap(),
            Name::parse("alex").unwrap(),
            signing(1).verifying_key(),
            [7; 16],
        );
        let id = seed.id();
        for node in [&alex, &sam] {
            node.rail().set_seed(&id, &seed.roster()).unwrap();
            let dir = node.rail().group_dir(&id).unwrap();
            std::fs::write(dir.join("seed.json"), seed.to_text()).unwrap();
        }
        svrngs_core::tests_support::let_in_by(&id, &signing(2), "sam", |act| {
            let act = svrngs_core::tests_support::Fixture { act };
            alex.rail().commit(&id, &act).unwrap().unwrap().lines()[0]
                .id
                .clone()
        });
        // Sam's admission in Sam's own record too, synced from Alex as a
        // computer that joined holds it: a record that does not name Sam
        // lends nothing through the group (door.rs `lends`, `stands_here`).
        let mut at = alex.net().addr();
        if quic {
            for socket in loopback_sockets(alex.endpoint()) {
                at = at.with_ip_addr(socket);
            }
        }
        let synced = sam.host().sync_with(&id, &at).await;
        assert!(synced.stop.is_none(), "{synced:?}");
        House {
            alex,
            sam: Some(sam),
            id,
            alex_dir,
            sam_dir,
            quic,
        }
    }

    /// An app on Alex's computer asks `path`: its status and body. Over
    /// QUIC, at the port its API listens on.
    async fn ask(&self, path: &str) -> (u16, serde_json::Value) {
        let url = format!("http://{}{path}", self.alex.api_addr());
        let within = Duration::from_secs(30);
        if self.quic {
            let http = reqwest::Client::builder().no_proxy().build().unwrap();
            let asked = http.get(url).bearer_auth(self.alex.token()).send();
            let answer = tokio::time::timeout(within, asked).await;
            let answer = answer.expect("the ask finishes").unwrap();
            return (answer.status().as_u16(), answer.json().await.unwrap());
        }
        let asked = on_wire::http()
            .get(url)
            .bearer_auth(self.alex.token())
            .send();
        let answer = tokio::time::timeout(within, asked).await;
        let answer = answer.expect("the ask finishes").unwrap();
        (answer.status().as_u16(), answer.json().await.unwrap())
    }

    async fn lends(&self) -> serde_json::Value {
        let (status, body) = self.ask(&format!("/v1/lends?namespace={}", self.id)).await;
        assert_eq!(status, 200, "{body:#}");
        body
    }

    async fn close(self) {
        if let Some(sam) = self.sam {
            sam.net().close().await;
        }
        self.alex.net().close().await;
    }
}

/// A lend as listed: `(person, key, name, kind, limit, using, upload_bps)`.
type Listed = (String, String, String, String, u64, u64, Option<u64>);

/// Each lend listed, as [`Listed`].
fn listed(body: &serde_json::Value) -> Vec<Listed> {
    body["lent"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["person"].as_str().unwrap().to_string(),
                l["key"].as_str().unwrap().to_string(),
                l["name"].as_str().unwrap().to_string(),
                l["kind"].as_str().unwrap().to_string(),
                l["limit"].as_u64().unwrap(),
                l["using"].as_u64().unwrap(),
                l["upload_bps"].as_u64(),
            )
        })
        .collect()
}

/// **An app asks what the group's people lend** (the u5-1 row): Alex's
/// `/v1/lends` lists Sam's lend with Sam's person and key and the upload
/// Sam declares, and Alex's own, its upload `null` since Alex declares
/// none; then, Sam's computer gone, Sam is named as not answering and
/// Sam's lend is not listed. Planted: this computer's own lend left out
/// (Alex's missing); the last answer kept for a computer that does not
/// answer (Sam's lend listed stale).
#[tokio::test]
async fn an_app_asks_what_the_group_s_people_lend_its_own_included_and_none_stale() {
    let house = House::new().await;
    lends(house.alex_dir.path(), &house.id, "telly", 8096, None);
    lends(
        house.sam_dir.path(),
        &house.id,
        "films",
        8097,
        Some(30_000_000),
    );
    let (alex, sam) = (actor_of(&signing(1)), actor_of(&signing(2)));
    let entry = |person: &str, key: &str, name: &str, upload| {
        (
            person.to_string(),
            key.to_string(),
            name.to_string(),
            "media".to_string(),
            3,
            0,
            upload,
        )
    };

    let body = house.lends().await;
    assert_eq!(
        listed(&body),
        vec![
            entry("alex", &alex, "telly", None),
            entry("sam", &sam, "films", Some(30_000_000)),
        ],
        "{body:#}"
    );
    assert!(
        body["lent"][0]
            .as_object()
            .unwrap()
            .contains_key("upload_bps"),
        "an upload none declared is said, null, not left out: {body:#}"
    );
    assert_eq!(body["unanswered"], serde_json::json!([]), "{body:#}");

    let mut house = house;
    let gone = house.sam.take().unwrap();
    gone.net().close().await;
    drop(gone);
    let body = house.lends().await;
    assert_eq!(
        listed(&body),
        vec![entry("alex", &alex, "telly", None)],
        "Sam's lend is not listed once Sam's computer is gone: {body:#}"
    );
    let sam = serde_json::json!([{ "person": "sam", "person_key": actor_of(&signing(2)) }]);
    assert_eq!(body["unanswered"], sam, "{body:#}");
    house.close().await;
}

/// **An app asks which groups this computer holds, and finds the API by
/// itself** (the u5-1 row): `/v1/groups` answers the group's id, its name,
/// Alex as this computer's person, that Alex was not removed (U7), that
/// nobody was removed or asked to be (v1-keeper-office (b)), that Alex did
/// not leave it and keeps the order of removals (groups-leave), and that
/// nobody is visiting it (guests-pages); who made it,
/// Alex's own computer, and since when it holds it, the time its seed was
/// written (v1ui-door (c)); beside the groups this computer's key, which
/// its home page tells its own devices by (capability-what-acts); house,
/// made before sources-v1 and kept here, offered Move (EUDOXIA_JUDGE §2e
/// §4); the data dir's `api` file names the
/// address the app asked at. Planted: the file not written. Over QUIC:
/// the file is written where the listener binds, and the app asks there.
#[tokio::test]
async fn an_app_finds_the_api_and_asks_the_groups_this_computer_holds() {
    let house = House::over_quic().await;
    let written = std::fs::read_to_string(house.alex_dir.path().join(svrngs_node::api::API_FILE))
        .expect("the node writes where its API listens");
    assert_eq!(written, format!("http://{}\n", house.alex.api_addr()));

    let (status, body) = house.ask(svrngs_node::surface::GROUPS_ROUTE).await;
    assert_eq!(status, 200, "{body:#}");
    let made = made_by_alex(&body);
    assert_eq!(
        body,
        serde_json::json!({ "groups": [{ "id": house.id, "name": "house", "made": made, "person": "alex", "removed": false, "removals": [], "left": false, "keeps": true, "visiting": [], "slots": 2, "folders": [], "waiting": [], "fallback": null, "vacates": true }], "key": actor_of(&signing(1)) }),
    );
    house.close().await;
}

/// The first group's `made`, as `/v1/groups` answers it on Alex's
/// computer: house, made by this computer (no `maker`), held since a time.
fn made_by_alex(body: &serde_json::Value) -> serde_json::Value {
    let made = &body["groups"][0]["made"];
    assert!(made["since"].is_i64(), "{body:#}");
    serde_json::json!({ "name": "house", "maker": null, "since": made["since"] })
}

/// **A register that cannot be read is said as itself** (REVIEW_FINDINGS
/// #123): with Alex's copy of the group's seed unreadable, `/v1/groups`
/// answers `removed: null`, never "not removed", and `removals: null`,
/// never none (v1-keeper-office (b)), and `left` and `keeps` null beside
/// them (groups-leave), and `visiting` null, never nobody (guests-pages),
/// and how many each computer can have visiting null, never the default,
/// so the home page can say
/// this computer could not read who is in the group. Red at 09e1484,
/// which read it `removed: false`.
#[tokio::test]
async fn an_unreadable_register_is_not_read_as_not_removed() {
    use std::os::unix::fs::PermissionsExt;
    let house = House::new().await;
    let roster = house
        .alex
        .rail()
        .group_dir(&house.id)
        .unwrap()
        .join("roster.json");
    std::fs::set_permissions(&roster, std::fs::Permissions::from_mode(0o000)).unwrap();
    let unread = house.alex.rail().register(&house.id).is_err();
    let (status, body) = house.ask(svrngs_node::surface::GROUPS_ROUTE).await;
    std::fs::set_permissions(&roster, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(unread, "the plant: Alex's register cannot be read");
    assert_eq!(status, 200, "{body:#}");
    let made = made_by_alex(&body);
    assert_eq!(
        body,
        serde_json::json!({ "groups": [{ "id": house.id, "name": "house", "made": made, "person": null, "removed": null, "removals": null, "left": null, "keeps": null, "visiting": null, "slots": null, "folders": [], "waiting": null, "fallback": null }], "key": actor_of(&signing(1)) }),
    );
    house.close().await;
}

/// **A caller off loopback is refused on both** (the u5-1 row): the new
/// routes stand behind the record API's one check, asked of the routes the
/// node serves, so a caller at a network address is refused 403
/// `not_loopback` and one on loopback is answered. Planted: the routes
/// merged after the check, so it does not cover them.
#[tokio::test]
async fn a_caller_off_loopback_is_refused_the_groups_and_the_lends() {
    use tower::ServiceExt;
    let house = House::new().await;
    for path in [
        svrngs_node::surface::GROUPS_ROUTE.to_string(),
        format!("/v1/lends?namespace={}", house.id),
    ] {
        for (caller, want) in [("192.168.1.5:50000", 403), ("127.0.0.1:50000", 200)] {
            let mut request = axum::http::Request::get(&path)
                .header("authorization", format!("Bearer {}", house.alex.token()))
                .body(axum::body::Body::empty())
                .unwrap();
            let caller: SocketAddr = caller.parse().unwrap();
            request
                .extensions_mut()
                .insert(axum::extract::ConnectInfo(caller));
            let answer = house.alex.api_routes().oneshot(request).await.unwrap();
            let status = answer.status().as_u16();
            let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
                .await
                .unwrap();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(status, want, "{path} from {caller}: {body}");
            if want == 403 {
                assert_eq!(body["kind"], "not_loopback", "{path}");
            }
        }
    }
    house.close().await;
}

/// **A lent page asks, at its own origin, the lends it reads** (mnp-2):
/// Alex lends a page, `night`, reading kind `media`, and a `telly` of it;
/// Sam a `films`. At `night`'s origin on Alex's computer, `/svrngs-reads`
/// answers `telly` and `films`, each with its kind, name, lender's person
/// and its origin on this computer under a capability ([`uncapped`]),
/// never `night` itself; then, Sam's computer gone, `telly` alone and Sam
/// named as not answering. Planted: the unanswered answered none.
#[tokio::test]
async fn a_lent_page_asks_at_its_own_origin_the_lends_it_reads() {
    let mut house = House::new().await;
    declare(
        house.alex_dir.path(),
        &house.id,
        "night",
        8596,
        "night",
        &["media"],
    );
    declare(
        house.alex_dir.path(),
        &house.id,
        "telly",
        8096,
        "media",
        &[],
    );
    declare(house.sam_dir.path(), &house.id, "films", 8097, "media", &[]);
    let port = house.alex.api_addr().port();
    let label = |seed: u8, name: &str| {
        svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name)
    };
    let at = |seed: u8, name: &str| svrngs_node::origin::origin(&label(seed, name), port);
    let (night, reads) = (label(1, "night"), svrngs_node::surface::READS_PATH);
    let telly = serde_json::json!({ "kind": "media", "name": "telly", "lender": "alex", "address": at(1, "telly") });

    let (status, body) = at_origin(&house.alex, &night, reads, None).await;
    assert_eq!(status, 200, "{body:#}");
    assert_eq!(
        uncapped(body),
        serde_json::json!({
            "lends": [telly, { "kind": "media", "name": "films", "lender": "sam", "address": at(2, "films") }],
            "unanswered": [],
        }),
    );

    let gone = house.sam.take().unwrap();
    gone.net().close().await;
    drop(gone);
    let (status, body) = at_origin(&house.alex, &night, reads, None).await;
    assert_eq!(status, 200, "{body:#}");
    assert_eq!(
        uncapped(body),
        serde_json::json!({ "lends": [telly], "unanswered": ["sam"] }),
        "Sam's computer gone: his lend not told, he named as not answering"
    );
    house.close().await;
}

/// The person's yes to the app `app`, as the home page's Open posts it to
/// `POST /reaching`, at this computer's home origin, with the token.
async fn says_yes(node: &Node, app: &str, reads: &[&str], left: &[&str]) -> u16 {
    let form = format!(
        "app={app}&reads={}&left={}",
        reads.join("+"),
        left.join("+")
    );
    says(node, form).await
}

/// `form` posted to `POST /reaching` as the home page's form posts it: a
/// yes ([`says_yes`]), or `said=stop`, the Stop under "What acts as you"
/// (home/page/reach.js `Reach`).
async fn says(node: &Node, form: String) -> u16 {
    use tower::ServiceExt;
    let own = format!("svrngs.localhost:{}", node.api_addr().port());
    let mut request = axum::http::Request::post("/reaching")
        .header("host", own.clone())
        .header("sec-fetch-site", "same-origin")
        .header("origin", format!("http://{own}"))
        .header("authorization", format!("Bearer {}", node.token().trim()))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(axum::body::Body::from(form))
        .unwrap();
    let caller: SocketAddr = "127.0.0.1:50000".parse().unwrap();
    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(caller));
    let answer = node.api_routes().oneshot(request).await.unwrap();
    answer.status().as_u16()
}

/// **A yes names lends** (CAPABILITY_CONNECTIONS §6 bar 7; §2 "A grant's
/// scope is explicit"). Sam lends a page, `reel`, reading `media`, and a
/// `films`; Alex lends a `telly`, and unticks it on his open. At Alex's
/// computer: `/svrngs-reads` lists only what the yes names, and says of a
/// lend it doesn't name, with no address, that it is new; a new lend of
/// Sam's, `more`, is unread until Alex's next open, while `films` keeps
/// working; the unticked `telly` is refused, unnamed and not said new, and
/// stays so through the next open; and a yes from before this, naming
/// kinds, covers nothing. Planted: `Choice::names` answering true for
/// every yes (surface.rs `reaches`), the `new`-lend reach is let through;
/// `reads` listing every lend read (surface.rs `reads`), the new lend is
/// addressed; the unticked lends' `left` dropped from the next open
/// (choices.rs `own_reaching`), telly comes back ticked and said new.
#[tokio::test]
async fn a_yes_names_lends_so_a_new_one_waits_for_the_next_open() {
    let house = House::new().await;
    declare(
        house.sam_dir.path(),
        &house.id,
        "reel",
        8597,
        "night",
        &["media"],
    );
    declare(house.sam_dir.path(), &house.id, "films", 8097, "media", &[]);
    declare(
        house.alex_dir.path(),
        &house.id,
        "telly",
        8096,
        "media",
        &[],
    );
    let label = |seed: u8, name: &str| {
        svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name)
    };
    let (reel, films, telly, more) = (
        label(2, "reel"),
        label(2, "films"),
        label(1, "telly"),
        label(2, "more"),
    );
    let reads = svrngs_node::surface::READS_PATH;
    let alex = &house.alex;
    let (app, reel_str) = (reel.clone(), reel.to_string());
    let listed = || at_origin(alex, &app, reads, None);
    let reach = |to: svrngs_core::pass::Label| {
        let reel = reel_str.clone();
        async move {
            let (status, body) = at_origin(alex, &to, "/x", Some(&reel)).await;
            (status, body["kind"].as_str().map(str::to_string))
        }
    };
    // Reached past the yes: the lender's door answers then, here refusing
    // or not, as Sam's register hears of Alex no other way in this test.
    let unasked = (403, Some("unasked".to_string()));
    let lender_of =
        |name: &str| serde_json::json!({ "kind": "media", "name": name, "lender": "sam" });

    // Before any yes, nothing is reached, and each is told as new, unaddressed.
    let (status, before) = listed().await;
    assert_eq!(status, 200, "{before:#}");
    assert_eq!(before["lends"], serde_json::json!([]), "{before:#}");
    assert_eq!(
        before["new"].as_array().map(Vec::len),
        Some(2),
        "telly and films, each with no address: {before:#}"
    );
    assert_eq!(reach(films.clone()).await, unasked, "no yes, no reach");

    // Alex's open names `films` and leaves `telly` unticked.
    let yes = says_yes(alex, reel.as_str(), &[films.as_str()], &[telly.as_str()]).await;
    assert_eq!(yes, 303, "the yes kept, then the app's own page");
    let (_, said) = listed().await;
    assert_eq!(said["lends"].as_array().map(Vec::len), Some(1), "{said:#}");
    assert_eq!(said["lends"][0]["name"], "films", "{said:#}");
    assert_eq!(
        said.get("new"),
        None,
        "the unticked lend is not said to be new: {said:#}"
    );
    assert_ne!(reach(films.clone()).await, unasked, "films, named, reached");
    assert_eq!(
        reach(telly.clone()).await,
        unasked,
        "the unticked lend is refused"
    );

    // Sam lends a new library; nothing about Alex's yes names it.
    declare(house.sam_dir.path(), &house.id, "more", 8098, "media", &[]);
    let (_, said) = listed().await;
    assert_eq!(said["lends"].as_array().map(Vec::len), Some(1), "{said:#}");
    assert_eq!(
        said["new"],
        serde_json::json!([lender_of("more")]),
        "{said:#}"
    );
    assert_eq!(
        reach(more.clone()).await,
        unasked,
        "new since the yes: unread"
    );
    assert_ne!(
        reach(films.clone()).await,
        unasked,
        "Sam's films keep working"
    );
    let row = house.lends().await;
    let reel_row = row["lent"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["name"] == "reel");
    let told = &reel_row.expect("reel is listed")["reading"];
    let of = |name: &str| {
        (told.as_array().unwrap().iter())
            .find(|l| l["name"] == name)
            .unwrap_or_else(|| panic!("{name} in {told:#}"))
            .clone()
    };
    assert_eq!(
        (&of("films")["ticked"], &of("films")["new"]),
        (&true.into(), &false.into()),
        "{told:#}"
    );
    assert_eq!(
        (&of("telly")["ticked"], &of("telly")["new"]),
        (&false.into(), &false.into()),
        "{told:#}"
    );
    assert_eq!(
        (&of("more")["ticked"], &of("more")["new"]),
        (&true.into(), &true.into()),
        "{told:#}"
    );

    // Alex's next open posts what its line showed, `more` with it, no extra step.
    let next = [films.as_str(), more.as_str()];
    assert_eq!(
        says_yes(alex, reel.as_str(), &next, &[telly.as_str()]).await,
        303
    );
    assert_ne!(reach(more.clone()).await, unasked, "named at the next open");
    assert_ne!(reach(films.clone()).await, unasked, "still named");
    assert_eq!(reach(telly.clone()).await, unasked, "still unticked");

    // A yes that says nothing about telly (a row that didn't show it) keeps its word.
    assert_eq!(
        says_yes(alex, reel.as_str(), &[more.as_str()], &[]).await,
        303
    );
    assert_ne!(
        reach(films.clone()).await,
        unasked,
        "films, not shown then, keeps its word"
    );
    assert_eq!(
        reach(telly.clone()).await,
        unasked,
        "telly, not shown then, keeps its word"
    );

    // A yes from before this names kinds: it parses and names no lend.
    let person = actor_of(&signing(1));
    let old = serde_json::json!({ person: { reel.as_str(): { "said": "yes", "at": 1, "reads": ["media"] } } });
    std::fs::write(house.alex_dir.path().join("reaching.json"), old.to_string()).unwrap();
    assert_eq!(
        reach(films.clone()).await,
        unasked,
        "an old yes covers nothing"
    );
    let (_, said) = listed().await;
    assert_eq!(said["lends"], serde_json::json!([]), "{said:#}");
    house.close().await;
}

/// **A Stop on the home page holds from the app's next request**
/// (capability-what-acts; WHAT_CAN_ACT_AS_YOU §9; the node's half of
/// browser::acts::the_home_page_says_what_acts_as_you_and_its_stop_holds,
/// whose drawing is web/tests/pages/opens.test.mjs's). Sam lends
/// `notes`, reading `films`, and `films`. Alex's open names films: the
/// notes reach them at the address their page holds, and never at the
/// films' plain origin, a fetch carrying no capability. Alex's Stop, posted
/// as the home page's form posts it, goes back to the home page, and from
/// the notes' next request the address held before it is refused,
/// `unasked`, in words Alex can act on. Planted, each watched failing:
/// door/choices.rs `own_reaching` answering a Stop and keeping nothing;
/// a Stop kept as the yes it stops (`Choice::names` without `said`).
#[tokio::test]
async fn a_stop_on_the_home_page_holds_from_the_app_s_next_request() {
    let house = House::new().await;
    let sam_dir = house.sam_dir.path();
    declare(sam_dir, &house.id, "notes", 8598, "notes", &["films"]);
    declare(sam_dir, &house.id, "films", 8099, "films", &[]);
    let label =
        |name: &str| svrngs_core::pass::Label::of(signing(2).verifying_key().as_bytes(), name);
    let (notes, films) = (label("notes"), label("films"));
    let alex = &house.alex;
    let unasked = |body: &serde_json::Value| {
        (
            body["kind"].as_str().map(str::to_string),
            body["error"].clone(),
        )
    };
    let words = || {
        (
            Some("unasked".to_string()),
            serde_json::json!("Open it from your home page first."),
        )
    };

    let (status, before) = at_origin(alex, &films, "/bookmark", Some(notes.as_str())).await;
    assert_eq!(
        (status, unasked(&before)),
        (403, words()),
        "no yes: {before:#}"
    );
    assert_eq!(
        says_yes(alex, notes.as_str(), &[films.as_str()], &[]).await,
        303
    );
    let held = format!(
        "/svrngs-as/{}/stopped",
        alex.read_capability(&notes, &films)
    );
    let (status, opened) = at_origin(alex, &films, "/opened", Some(notes.as_str())).await;
    assert_ne!(
        (status, unasked(&opened)),
        (403, words()),
        "on the yes the notes reach the films: {opened:#}"
    );
    // The notes' page fetching the films' plain origin: no capability.
    let port = alex.api_addr().port();
    let bare = on_wire::http()
        .get(format!("http://{}/bare", alex.api_addr()))
        .header("host", format!("{films}.localhost:{port}"))
        .header("sec-fetch-site", "same-site")
        .header("sec-fetch-mode", "cors")
        .header("origin", format!("http://{notes}.localhost:{port}"))
        .send()
        .await
        .unwrap();
    let (status, bare) = (bare.status().as_u16(), bare.text().await.unwrap());
    // Refused there, 401, as a browser with no session is: never sent on.
    assert_eq!(status, 401, "the films' plain origin is no address: {bare}");

    let stop = format!("app={}&said=stop", notes.as_str());
    assert_eq!(
        says(alex, stop).await,
        303,
        "the Stop kept, then the home page"
    );
    let (status, stopped) = at_origin(alex, &films, &held, Some(notes.as_str())).await;
    assert_eq!(
        (status, unasked(&stopped)),
        (403, words()),
        "from the next request the address held before the Stop is refused: {stopped:#}"
    );
    house.close().await;
}

/// **A visit is listed on every computer of the group, and its host's Stop
/// ends it on each** (guests-pages (a), (b); GUESTS §3.2, §5 "The pages",
/// §7 bar 7; the node's half of browser::guests's two pages tests, whose
/// drawing is web/tests/pages/opens.test.mjs's). Alex hosts Sam until 40
/// hours from now, written as Alex's `/visit` writes it; Sam's computer
/// syncs Alex's record. Each `/v1/groups` lists Sam visiting, Alex's guest,
/// until then, Alex's as Alex's own. Alex's Stop, posted as the home page
/// posts it (`POST /visits`), is in Alex's record, and Alex's list is
/// empty; Sam's computer syncs, and its list is empty too. Planted: the
/// fold's live visits read without the Stop (rail.rs `visiting`), and
/// both still list Sam.
#[tokio::test]
async fn a_visit_is_listed_on_every_computer_and_its_host_s_stop_ends_it_on_each() {
    let house = House::new().await;
    let (alex, sam, id) = (&house.alex, house.sam.as_ref().unwrap(), &house.id);
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let until = now.unwrap().as_secs() as i64 + 40 * 60 * 60;
    let guest = signing(10);
    let visit = svrngs_core::Visit {
        guest: svrngs_node::Person::parse("Sam").unwrap(),
        key: actor_of(&guest),
        opens: Default::default(),
        until,
        consent: svrngs_core::tests_support::consent(&guest, &actor_of(alex.rail().signer())),
        under: None,
    };
    alex.rail().commit(id, &visit).unwrap().unwrap();
    let synced = || async {
        let synced = sam.host().sync_with(id, &alex.net().addr()).await;
        assert!(synced.stop.is_none(), "{synced:?}");
    };
    let visiting = |node: &Node| {
        let url = format!("http://{}/v1/groups", node.api_addr());
        let asked = on_wire::http().get(url).bearer_auth(node.token()).send();
        async move {
            let body: serde_json::Value = asked.await.unwrap().json().await.unwrap();
            body["groups"][0]["visiting"].clone()
        }
    };
    synced().await;
    let visit_id = alex
        .rail()
        .register(id)
        .unwrap()
        .visits
        .keys()
        .next()
        .unwrap()
        .to_string();
    let listed = |yours: bool| {
        serde_json::json!([{ "id": visit_id, "guest": "Sam", "host": "alex",
            "host_key": actor_of(&signing(1)), "until": until, "yours": yours }])
    };
    assert_eq!(visiting(alex).await, listed(true), "Alex's, as its host");
    assert_eq!(
        visiting(sam).await,
        listed(false),
        "Sam's computer's, Alex's guest"
    );
    // What a phone of the group reads through Sam's computer: its people's
    // lends, asked as a member dials it (node.rs:1029), here as Alex's key.
    let bridge = on_wire::bridge_as(&signing(1), sam, svrngs_node::iroh::Alpn::Member).await;
    let phone = || {
        let url = format!(
            "http://{}{}",
            bridge.local_addr(),
            svrngs_node::node::PEOPLE_LENDS_ROUTE
        );
        let asked = on_wire::http()
            .get(url)
            .header(
                "x-svrngs-version",
                svrngs_node::version::VERSION.to_string(),
            )
            .send();
        async move {
            let body: serde_json::Value = asked.await.unwrap().json().await.unwrap();
            body["visiting"].clone()
        }
    };
    assert_eq!(
        phone().await,
        listed(false),
        "a phone's, through Sam's computer"
    );

    let own = format!("svrngs.localhost:{}", alex.api_addr().port());
    let stop = on_wire::http()
        .post(format!("http://{}/visits", alex.api_addr()))
        .header("host", &own)
        .header("origin", format!("http://{own}"))
        .header("sec-fetch-site", "same-origin")
        .bearer_auth(alex.token().trim())
        .json(&[id.as_str(), visit_id.as_str()])
        .send()
        .await
        .unwrap();
    assert!(stop.status().is_success(), "{:?}", stop.text().await);
    let register = alex.rail().register(id).unwrap();
    assert!(
        register.visits.values().all(|g| g.stopped.is_some()),
        "Alex's Stop is in Alex's record: {:#?}",
        register.visits
    );
    // A visit is live while the time is before its Stop's
    // (membership.rs:180, `at < e`), so it is gone from the second after.
    let stopped = register
        .visits
        .values()
        .filter_map(|g| g.stopped)
        .max()
        .unwrap();
    while std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        <= stopped
    {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        visiting(alex).await,
        serde_json::json!([]),
        "Alex's, stopped"
    );
    synced().await;
    assert_eq!(
        visiting(sam).await,
        serde_json::json!([]),
        "Sam's computer's, synced"
    );
    assert_eq!(
        phone().await,
        serde_json::json!([]),
        "a phone's, through Sam's computer"
    );
    house.close().await;
}

/// **A lent page is not held by a computer that never answers**
/// (asleep-1-none-held (b); `demo/runs/mnp/2026-10-09/README.md`, "What no
/// prediction named" 2): as `a_lent_page_asks_at_its_own_origin_the_lends_it_reads`,
/// with Cy let in too and Cy's computer asleep. `/svrngs-reads` at `night`'s
/// origin answers `telly` and Sam's `films` within
/// [`svrngs_node::node::ANSWER_WITHIN`] and a second, Cy named as not
/// answering, never dropped. Planted: every lender's ask awaited to its own
/// end (node.rs `lends_in` before this row), which held the answer to the
/// asleep dial's timeout.
#[tokio::test]
async fn a_lent_page_is_not_held_by_a_computer_that_never_answers() {
    let house = House::new().await;
    declare(
        house.alex_dir.path(),
        &house.id,
        "night",
        8596,
        "night",
        &["media"],
    );
    declare(
        house.alex_dir.path(),
        &house.id,
        "telly",
        8096,
        "media",
        &[],
    );
    declare(house.sam_dir.path(), &house.id, "films", 8097, "media", &[]);
    svrngs_core::tests_support::let_in_by(&house.id, &signing(3), "cy", |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        house
            .alex
            .rail()
            .commit(&house.id, &act)
            .unwrap()
            .unwrap()
            .lines()[0]
            .id
            .clone()
    });
    let cy =
        svrngs_node::iroh::PublicKey::from_bytes(signing(3).verifying_key().as_bytes()).unwrap();
    on_wire::asleep(cy);
    let port = house.alex.api_addr().port();
    let label = |seed: u8, name: &str| {
        svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name)
    };
    let at = |seed: u8, name: &str| svrngs_node::origin::origin(&label(seed, name), port);
    let reads = svrngs_node::surface::READS_PATH;

    let asked = std::time::Instant::now();
    let (status, body) = at_origin(&house.alex, &label(1, "night"), reads, None).await;
    let took = asked.elapsed();
    eprintln!("the reads answered in {took:?}: {body}");
    assert_eq!(status, 200, "{body:#}");
    assert_eq!(
        uncapped(body),
        serde_json::json!({
            "lends": [
                { "kind": "media", "name": "telly", "lender": "alex", "address": at(1, "telly") },
                { "kind": "media", "name": "films", "lender": "sam", "address": at(2, "films") },
            ],
            "unanswered": ["cy"],
        }),
        "Cy's computer asleep: named as not answering, never dropped"
    );
    let bound = svrngs_node::node::ANSWER_WITHIN + Duration::from_secs(1);
    assert!(
        took < bound,
        "the reads waited {took:?} on a computer that never answers, past {bound:?}"
    );
    house.close().await;
}

/// **A silent lender is could-not-judge, on both paths** (onair-economize
/// (a)): one decider of what a lent page reads, and one reading of a lender
/// whose computer did not say what it lends. Alex lends a page, `night`,
/// reading `media`, and a `telly`; Sam a page, `reel`, reading `media`, and
/// a `films`. Sam's computer gone, at Alex's: `/svrngs-reads` at `reel`'s
/// origin, `night`'s fetch at Sam's `films` and `reel`'s fetch at Alex's
/// `telly` are each refused `away`, 502 — never "nothing is lent here", nor
/// "this page does not read that". Red at a20ed46: the reads answered 404
/// `unlent` for `reel`, as if Sam had answered that it is not lent.
#[tokio::test]
async fn a_silent_lender_is_could_not_judge_on_the_page_s_reads_and_at_the_origin() {
    let mut house = House::new().await;
    declare(
        house.alex_dir.path(),
        &house.id,
        "night",
        8596,
        "night",
        &["media"],
    );
    declare(
        house.alex_dir.path(),
        &house.id,
        "telly",
        8096,
        "media",
        &[],
    );
    declare(
        house.sam_dir.path(),
        &house.id,
        "reel",
        8597,
        "night",
        &["media"],
    );
    declare(house.sam_dir.path(), &house.id, "films", 8097, "media", &[]);
    let label = |seed: u8, name: &str| {
        svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name)
    };
    let alex = &house.alex;
    let ask = |at, path, from| async move {
        let (status, body) = at_origin(alex, &at, path, from).await;
        (status, body["kind"].as_str().map(str::to_string))
    };
    let (night, reel) = (label(1, "night").to_string(), label(2, "reel").to_string());
    let (status, body) = ask(label(2, "reel"), "/svrngs-reads", None).await;
    assert_eq!(
        status, 200,
        "Sam's computer up, reel's reads answered: {body:?}"
    );

    let gone = house.sam.take().unwrap();
    gone.net().close().await;
    drop(gone);
    let away = (502, Some("away".to_string()));
    for (row, at, path, from) in [
        ("reel's reads", label(2, "reel"), "/svrngs-reads", None),
        (
            "night's fetch at Sam's films",
            label(2, "films"),
            "/x",
            Some(night.as_str()),
        ),
        (
            "reel's fetch at Alex's telly",
            label(1, "telly"),
            "/x",
            Some(reel.as_str()),
        ),
    ] {
        let got = ask(at, path, from).await;
        eprintln!("{row}: {got:?}");
        assert_eq!(got, away, "{row}: Sam's computer silent, could-not-judge");
    }
    house.close().await;
}
