//! What an app asks beside the record (DESIGN §3; the u5-1 row): the groups
//! this computer holds, and what a group's people lend, on the record's
//! loopback listener, whose address the node writes to its data dir.
//! Alex founds a group and lets Sam in; both lend. Two nodes in one
//! process, no relay, each listening on 127.0.0.1 alone (#33's fix:
//! ralph/REVIEW_FINDINGS.md), Sam reached by seeded addresses.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::net::SocketAddr;
use std::time::Duration;

use svrngs_node::iroh::RelayConfig;
use svrngs_node::lend;
use svrngs_node::node::Node;
use svrngs_node::seed::Seed;
use svrngs_node::{actor_of, Name, Person, RailAct, SigningKey};

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

async fn start(seed: u8, dir: &std::path::Path) -> Node {
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
fn lends(dir: &std::path::Path, name: &str, port: u16, upload_bps: Option<u64>) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let mut body = serde_json::json!({ "port": port, "kind": "media", "limit": 3 });
    if let Some(bps) = upload_bps {
        body["upload_bps"] = bps.into();
    }
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// Alex's group "house" and Sam in it, each computer holding its seed, and
/// Alex knowing where Sam listens.
struct House {
    alex: Node,
    sam: Option<Node>,
    id: String,
    alex_dir: tempfile::TempDir,
    sam_dir: tempfile::TempDir,
}

impl House {
    async fn new() -> Self {
        let (alex_dir, sam_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let alex = start(1, alex_dir.path()).await;
        let sam = start(2, sam_dir.path()).await;
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
        let admit = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse("sam").unwrap(),
                key: actor_of(&signing(2)),
                invite: None,
            },
            on_behalf_of: None,
        };
        alex.rail().commit(&id, &admit).unwrap().unwrap();
        alex.transport()
            .add_known_peer(sam.endpoint().id(), loopback_sockets(sam.endpoint()));
        House {
            alex,
            sam: Some(sam),
            id,
            alex_dir,
            sam_dir,
        }
    }

    /// An app on Alex's computer asks `path`: its status and body.
    async fn ask(&self, path: &str) -> (u16, serde_json::Value) {
        let http = reqwest::Client::builder().no_proxy().build().unwrap();
        let answer = tokio::time::timeout(
            Duration::from_secs(30),
            http.get(format!("http://{}{path}", self.alex.api_addr()))
                .send(),
        )
        .await
        .expect("the ask finishes")
        .unwrap();
        (answer.status().as_u16(), answer.json().await.unwrap())
    }

    async fn lends(&self) -> serde_json::Value {
        let (status, body) = self.ask(&format!("/v1/lends?namespace={}", self.id)).await;
        assert_eq!(status, 200, "{body:#}");
        body
    }

    async fn close(self) {
        if let Some(sam) = self.sam {
            sam.endpoint().close().await;
        }
        self.alex.endpoint().close().await;
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
    lends(house.alex_dir.path(), "telly", 8096, None);
    lends(house.sam_dir.path(), "films", 8097, Some(30_000_000));
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
    gone.endpoint().close().await;
    drop(gone);
    let body = house.lends().await;
    assert_eq!(
        listed(&body),
        vec![entry("alex", &alex, "telly", None)],
        "Sam's lend is not listed once Sam's computer is gone: {body:#}"
    );
    assert_eq!(body["unanswered"], serde_json::json!(["sam"]), "{body:#}");
    house.close().await;
}

/// **An app asks which groups this computer holds, and finds the API by
/// itself** (the u5-1 row): `/v1/groups` answers the group's id, its name,
/// Alex as this computer's person, that Alex was not removed (U7) and that
/// nobody was removed or asked to be (v1-keeper-office (b)); the data dir's `api` file names the
/// address the app asked at. Planted: the file not written.
#[tokio::test]
async fn an_app_finds_the_api_and_asks_the_groups_this_computer_holds() {
    let house = House::new().await;
    let written = std::fs::read_to_string(house.alex_dir.path().join(svrngs_node::api::API_FILE))
        .expect("the node writes where its API listens");
    assert_eq!(written, format!("http://{}\n", house.alex.api_addr()));

    let (status, body) = house.ask(svrngs_node::surface::GROUPS_ROUTE).await;
    assert_eq!(status, 200, "{body:#}");
    assert_eq!(
        body,
        serde_json::json!({ "groups": [{ "id": house.id, "name": "house", "person": "alex", "removed": false, "removals": [], "folders": [] }] }),
    );
    house.close().await;
}

/// **A register that cannot be read is said as itself** (REVIEW_FINDINGS
/// #123): with Alex's copy of the group's seed unreadable, `/v1/groups`
/// answers `removed: null`, never "not removed", and `removals: null`,
/// never none (v1-keeper-office (b)), so the home page can say
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
    assert_eq!(
        body,
        serde_json::json!({ "groups": [{ "id": house.id, "name": "house", "person": null, "removed": null, "removals": null, "folders": [] }] }),
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
/// and its origin on this computer, never `night` itself; then, Sam's
/// computer gone, `telly` alone and Sam named as not answering. Planted:
/// the unanswered answered none.
#[tokio::test]
async fn a_lent_page_asks_at_its_own_origin_the_lends_it_reads() {
    use tower::ServiceExt;
    let mut house = House::new().await;
    let declare = |dir: &std::path::Path, name: &str, port: u16, kind: &str, reads: &[&str]| {
        let at = lend::lends_dir(dir);
        std::fs::create_dir_all(&at).unwrap();
        let body = serde_json::json!({ "port": port, "kind": kind, "limit": 3, "reads": reads });
        std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
    };
    declare(house.alex_dir.path(), "night", 8596, "night", &["media"]);
    declare(house.alex_dir.path(), "telly", 8096, "media", &[]);
    declare(house.sam_dir.path(), "films", 8097, "media", &[]);
    let port = house.alex.api_addr().port();
    let label = |seed: u8, name: &str| {
        svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name)
    };
    let at = |seed: u8, name: &str| svrngs_node::origin::origin(&label(seed, name), port);
    let ask = |alex: &Node| {
        let mut request = axum::http::Request::get(svrngs_node::surface::READS_PATH)
            .header("host", format!("{}.localhost:{port}", label(1, "night")))
            .body(axum::body::Body::empty())
            .unwrap();
        let caller: SocketAddr = "127.0.0.1:50000".parse().unwrap();
        request
            .extensions_mut()
            .insert(axum::extract::ConnectInfo(caller));
        let routes = alex.api_routes();
        async move {
            let answer = tokio::time::timeout(Duration::from_secs(30), routes.oneshot(request))
                .await
                .expect("the ask finishes")
                .unwrap();
            let status = answer.status().as_u16();
            let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
                .await
                .unwrap();
            (
                status,
                serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            )
        }
    };
    let telly = serde_json::json!({ "kind": "media", "name": "telly", "lender": "alex", "address": at(1, "telly") });

    let (status, body) = ask(&house.alex).await;
    assert_eq!(status, 200, "{body:#}");
    assert_eq!(
        body,
        serde_json::json!({
            "lends": [telly, { "kind": "media", "name": "films", "lender": "sam", "address": at(2, "films") }],
            "unanswered": [],
        }),
    );

    let gone = house.sam.take().unwrap();
    gone.endpoint().close().await;
    drop(gone);
    let (status, body) = ask(&house.alex).await;
    assert_eq!(status, 200, "{body:#}");
    assert_eq!(
        body,
        serde_json::json!({ "lends": [telly], "unanswered": ["sam"] }),
        "Sam's computer gone: his lend not told, he named as not answering"
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
    use tower::ServiceExt;
    let house = House::new().await;
    let declare = |dir: &std::path::Path, name: &str, port: u16, kind: &str, reads: &[&str]| {
        let at = lend::lends_dir(dir);
        std::fs::create_dir_all(&at).unwrap();
        let body = serde_json::json!({ "port": port, "kind": kind, "limit": 3, "reads": reads });
        std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
    };
    declare(house.alex_dir.path(), "night", 8596, "night", &["media"]);
    declare(house.alex_dir.path(), "telly", 8096, "media", &[]);
    declare(house.sam_dir.path(), "films", 8097, "media", &[]);
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("cy").unwrap(),
            key: actor_of(&signing(3)),
            invite: None,
        },
        on_behalf_of: None,
    };
    house
        .alex
        .rail()
        .commit(&house.id, &admit)
        .unwrap()
        .unwrap();
    let (_asleep, at_cy) = common::never_answers();
    let cy =
        svrngs_node::iroh::PublicKey::from_bytes(signing(3).verifying_key().as_bytes()).unwrap();
    house.alex.transport().add_known_peer(cy, at_cy);
    let port = house.alex.api_addr().port();
    let label = |seed: u8, name: &str| {
        svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name)
    };
    let at = |seed: u8, name: &str| svrngs_node::origin::origin(&label(seed, name), port);
    let mut request = axum::http::Request::get(svrngs_node::surface::READS_PATH)
        .header("host", format!("{}.localhost:{port}", label(1, "night")))
        .body(axum::body::Body::empty())
        .unwrap();
    let caller: SocketAddr = "127.0.0.1:50000".parse().unwrap();
    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(caller));

    let asked = std::time::Instant::now();
    let answer = tokio::time::timeout(
        Duration::from_secs(30),
        house.alex.api_routes().oneshot(request),
    )
    .await
    .expect("the ask finishes")
    .unwrap();
    let took = asked.elapsed();
    let status = answer.status().as_u16();
    let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    eprintln!("the reads answered in {took:?}: {body}");
    assert_eq!(status, 200, "{body:#}");
    assert_eq!(
        body,
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
    use tower::ServiceExt;
    let mut house = House::new().await;
    let declare = |dir: &std::path::Path, name: &str, port: u16, kind: &str, reads: &[&str]| {
        let at = lend::lends_dir(dir);
        std::fs::create_dir_all(&at).unwrap();
        let body = serde_json::json!({ "port": port, "kind": kind, "limit": 3, "reads": reads });
        std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
    };
    declare(house.alex_dir.path(), "night", 8596, "night", &["media"]);
    declare(house.alex_dir.path(), "telly", 8096, "media", &[]);
    declare(house.sam_dir.path(), "reel", 8597, "night", &["media"]);
    declare(house.sam_dir.path(), "films", 8097, "media", &[]);
    let port = house.alex.api_addr().port();
    let label = |seed: u8, name: &str| {
        svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name)
    };
    let ask = |alex: &Node, at: svrngs_core::pass::Label, path: &str, from: Option<&str>| {
        let mut request =
            axum::http::Request::get(path).header("host", format!("{at}.localhost:{port}"));
        if let Some(page) = from {
            request = request
                .header("sec-fetch-site", "same-site")
                .header("sec-fetch-mode", "cors")
                .header("origin", format!("http://{page}.localhost:{port}"));
        }
        let mut request = request.body(axum::body::Body::empty()).unwrap();
        let caller: SocketAddr = "127.0.0.1:50000".parse().unwrap();
        request
            .extensions_mut()
            .insert(axum::extract::ConnectInfo(caller));
        let routes = alex.api_routes();
        async move {
            let answer = tokio::time::timeout(Duration::from_secs(30), routes.oneshot(request))
                .await
                .expect("the ask finishes")
                .unwrap();
            let status = answer.status().as_u16();
            let body = axum::body::to_bytes(answer.into_body(), usize::MAX)
                .await
                .unwrap();
            let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
            (status, body["kind"].as_str().map(str::to_string))
        }
    };
    let (night, reel) = (label(1, "night").to_string(), label(2, "reel").to_string());
    let (status, body) = ask(&house.alex, label(2, "reel"), "/svrngs-reads", None).await;
    assert_eq!(
        status, 200,
        "Sam's computer up, reel's reads answered: {body:?}"
    );

    let gone = house.sam.take().unwrap();
    gone.endpoint().close().await;
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
        let got = ask(&house.alex, at, path, from).await;
        eprintln!("{row}: {got:?}");
        assert_eq!(got, away, "{row}: Sam's computer silent, could-not-judge");
    }
    house.close().await;
}
