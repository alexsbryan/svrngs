//! A group made and another joined on the home page, while svrngs runs,
//! on the wire: home_group.rs's claims whose decision is the node's, its
//! home routes (home.rs `found`, `link`, `asked`, `join`) asked as the
//! page asks them, and `svrngs new`'s, as cli.rs `serve` runs it. Each
//! computer on one [`Wire`]: no browser, endpoint, relay or socket. What
//! the page says of each answer is web/tests/pages/home.test.mjs's.

use axum::http::StatusCode;
use serde_json::{json, Value};
use svrngs::cli::{self, Said};
use svrngs::group;
use svrngs_node::actor_of;
use svrngs_node::link::{client_link, Link};
use svrngs_node::node::{Node, Wire};
use tower::ServiceExt;

use crate::node_common::heard;
use crate::{computer, name, no_relay, Computer};

/// `method` of `path` asked of `node`'s home page as its own page asks it:
/// at `svrngs.localhost:<port>`, from itself, the token shown. The status
/// and the body read as JSON.
async fn paged(node: &Node, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let own = format!("svrngs.localhost:{}", node.api_addr().port());
    let req = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("host", &own)
        .header("origin", format!("http://{own}"))
        .header("sec-fetch-site", "same-origin")
        .header("sec-fetch-mode", "cors")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", node.token().trim()));
    let mut req = req.body(axum::body::Body::from(body.to_string())).unwrap();
    let caller = std::net::SocketAddr::from(([127, 0, 0, 1], 50_000));
    req.extensions_mut()
        .insert(axum::extract::ConnectInfo(caller));
    let answer = node.api_routes().oneshot(req).await.unwrap();
    let status = answer.status();
    let body = axum::body::to_bytes(answer.into_body(), usize::MAX).await;
    let body = body.unwrap();
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

/// The rule's one trace for a name held `held` times already (group.rs
/// `namesakes`), in what `heard` kept.
fn ruled(traced: &str, name: &str, held: usize) -> bool {
    traced.lines().any(|l| {
        l.contains("group: a name's one rule")
            && l.contains(&format!("name={name} "))
            && l.contains(&format!("held={held}"))
    })
}

/// `svrngs new <name> --as Dee`, as cli.rs `serve`'s New arm runs it:
/// the one rule asked, the group made, what it prints and its folder.
async fn new(dee: &Computer, called: &str) -> (group::Seed, String) {
    let me = actor_of(&dee.key);
    let beside = group::namesakes(dee.node.rail().root(), &name(called), &me);
    let (seed, link) = cli::new_group(&dee.node, name(called), name("Dee"), None, &[], &no_relay())
        .await
        .expect("Dee makes a group");
    let (dir, placed) = dee.node.folders().place(&seed);
    let group = seed.name().clone();
    let made = Said::Made {
        seed: seed.clone(),
        link,
        beside,
    };
    let folder = Said::Folder { group, dir, placed };
    (seed, format!("{made}\n{folder}"))
}

/// **A group made on the home page, joined by another computer with no
/// restart** (home_group.rs, start-group (c)): `svrngs new Lab` then
/// `svrngs new lab` make two groups by the name's one rule, the second
/// said beside the first and its folder apart from Lab's where the disk
/// ignores case; then, on the page, POST /group makes Trip as Dee, its
/// folder placed where a joiner's is, and POST /link answers a link on
/// the page this computer's svrngs came from (its data dir's `origin`,
/// as install.sh keeps it), by which Bo is let in and stands in Trip.
/// GET /group?name=trip answers Trip as Dee's own, and POST /group makes
/// trip beside it, by the one rule.
#[tokio::test]
async fn a_group_made_on_the_home_page_is_joined_by_its_link_with_no_restart() {
    let (heard, _guard) = heard("group=debug");
    let wire = Wire::default();
    let dee = computer(&wire, 5).await;
    let bo = computer(&wire, 4).await;
    let traced = || String::from_utf8_lossy(&heard.0.lock().unwrap()).into_owned();

    let (_, said) = new(&dee, "Lab").await;
    assert!(ruled(&traced(), "Lab", 0), "{said}\n{}", traced());
    let (lab, said) = new(&dee, "lab").await;
    let today = chrono::Local::now().format("%-d %b");
    let second =
        format!("You’re already in Lab (yours, since {today}). This is a second group called lab.");
    assert!(said.contains(&second), "{said}");
    assert!(ruled(&traced(), "lab", 1), "{}", traced());
    let places = dee.dir.path().join("svrngs");
    assert!(places.join("Lab").is_dir(), "Lab's folder");
    let apart = match places.join("LAB").exists() {
        true => places.join(format!("lab-{}", &lab.id()[..8])),
        false => places.join("lab"),
    };
    let folder = format!("lab's folder is {}: ", apart.display());
    assert!(said.contains(&folder), "{said}");

    let came_from = "http://127.0.0.1:9/";
    std::fs::write(dee.dir.path().join("origin"), came_from).unwrap();
    let (status, made) = paged(&dee.node, "POST", "/group", json!(["Trip", "Dee"])).await;
    assert_eq!(
        (status, &made),
        (StatusCode::OK, &json!(["Trip", null, []]))
    );
    assert!(places.join("Trip").is_dir(), "Trip's folder placed");
    let held = group::groups(dee.dir.path());
    let trip = held.iter().find(|s| s.name().as_str() == "Trip").unwrap();

    let body = json!({ "group": trip.id(), "device": false });
    let (status, linked) = paged(&dee.node, "POST", "/link", body).await;
    assert_eq!(status, StatusCode::OK, "{linked}");
    let https = linked["https"].as_str().expect("an https link");
    assert!(https.starts_with(&format!("{came_from}#")), "{linked}");
    let link = Link::parse(https).expect("the link shown is a link");
    let joined = cli::join_group(&bo.node, &bo.key, &link, &name("Bo")).await;
    let Ok(Said::Joined { seed, .. }) = joined else {
        panic!("Bo is let in: {joined:?}");
    };
    assert_eq!(seed.id(), trip.id());
    let standing = bo.node.rail().standing(&trip.id()).unwrap();
    assert!(standing.names(&actor_of(&bo.key)), "Bo stands in Trip");

    let (_, before) = paged(&dee.node, "GET", "/group?name=trip", Value::Null).await;
    let since = before[0]["since"].as_i64().expect("since when it is held");
    let mine = json!([{ "name": "Trip", "maker": null, "since": since }]);
    assert_eq!(before, mine);
    let (status, made) = paged(&dee.node, "POST", "/group", json!(["trip", "Dee"])).await;
    assert_eq!(
        (status, made),
        (StatusCode::OK, json!(["trip", null, mine]))
    );
    assert!(ruled(&traced(), "trip", 1), "{}", traced());
}

/// **Another group joined from its link on the home page, while svrngs
/// runs** (home_group.rs, arch-command (c)): Dee, in Lab, is sent a link
/// to Sam's Trip. GET /join answers what it is for, asked of Sam's
/// computer, for the button to say; POST /join joins as Dee as `svrngs
/// join` joins, places Trip's folder where a joiner's is, and Sam's
/// computer has let Dee in; /v1/groups then lists Trip as Sam's beside
/// Lab, Dee's own, which the page heads by whose each is.
#[tokio::test]
async fn a_group_joined_on_the_home_page_is_joined_as_the_command_joins() {
    let wire = Wire::default();
    let sam = computer(&wire, 6).await;
    let dee = computer(&wire, 5).await;
    let (trip, _) = cli::new_group(&sam.node, name("Trip"), name("Sam"), None, &[], &no_relay())
        .await
        .expect("Sam makes Trip");
    let made = sam.node.host().link(&trip.id(), false, None, None);
    let link = client_link(&made.expect("Sam makes a link to Trip").token);
    new(&dee, "Lab").await;

    let asked = format!("/join?link={}", urlencoding(&link));
    let (status, what) = paged(&dee.node, "GET", &asked, Value::Null).await;
    assert_eq!((status, what), (StatusCode::OK, json!(["Trip", null])));
    let body = json!({ "link": link, "person": "Dee" });
    let (status, joined) = paged(&dee.node, "POST", "/join", body).await;
    assert_eq!(
        (status, joined),
        (StatusCode::OK, json!(["Trip", "Dee", null]))
    );
    assert!(dee.dir.path().join("svrngs/Trip").is_dir(), "Trip's folder");
    let register = sam.node.rail().register(&trip.id()).unwrap();
    assert!(
        (register.members.values()).any(|m| m.person.as_str() == "Dee"),
        "Dee stands in Trip on Sam's computer"
    );
    let (_, groups) = paged(&dee.node, "GET", "/v1/groups", Value::Null).await;
    let whose: Vec<_> = (groups["groups"].as_array().unwrap().iter())
        .map(|g| (g["name"].clone(), g["made"]["maker"].clone()))
        .collect();
    assert!(whose.contains(&(json!("Trip"), json!("Sam"))), "{groups}");
    assert!(whose.contains(&(json!("Lab"), Value::Null)), "{groups}");
}

/// `link` as a query's value, each byte not plain percent-encoded.
fn urlencoding(link: &str) -> String {
    link.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b => format!("%{b:02X}"),
        })
        .collect()
}
