//! `svrngs new` and `svrngs link` (PLAN §4, U2) on the wire: new.rs's
//! claims whose decision is `cli::new_group`'s, `cli::link_to`'s or
//! `cli::running`'s (docs/seams.tsv). A group made, its seed written beside
//! its record, an `Invite` appended through its door, and a short link
//! printed whose key the `Invite` names (u2-8). What stays in new.rs: a
//! page's host asked whether it serves the page at every name, a listener.

use svrngs::cli::{self, Printed, Refusal, Refused};
use svrngs::group;
use svrngs_node::invite::LINK_LIFE_SECS;
use svrngs_node::join as issuer;
use svrngs_node::link::{Link, Origin, CLIENT_PREFIX};
use svrngs_node::node::{Node, Wire};
use svrngs_node::register::Standing;
use svrngs_node::{actor_of, derive_id, OpId, RailAct};

use crate::{ask, computer, name, no_relay};

/// The lines a group's journal holds on disk, as written.
fn lines(node: &Node, group: &str) -> Vec<svrngs_node::Op<svrngs_node::SignedOp>> {
    let file = node
        .rail()
        .group_dir(group)
        .unwrap()
        .join("ring_oplog.jsonl");
    std::fs::read_to_string(file)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

/// The key a printed link's `Invite` names.
fn key_of(printed: &Printed) -> String {
    let link = Link::parse(&printed.client).expect("the client's link parses");
    actor_of(&link.token.key())
}

/// The `Invite`s a group's record holds: (link key, until, the id its
/// content derives).
fn invites(node: &Node, id: &str) -> Vec<(String, i64, OpId)> {
    let ops = lines(node, id);
    ops.into_iter()
        .filter_map(|op| match &op.kind.act {
            RailAct::Invite { key, until, .. } => Some((key.clone(), *until, derive_id(&op, id))),
            _ => None,
        })
        .collect()
}

/// `new` twice with one name makes two groups with two ids: each its own
/// seed and record, each founded by this computer as Sam, each with one
/// `Invite` naming its link's key for a day; each link is the client's
/// alone, dialling this computer, and at most 90 characters.
#[tokio::test]
async fn new_twice_with_one_name_makes_two_groups_with_two_ids() {
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let node = &sam.node;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut ids = Vec::new();
    for _ in 0..2 {
        let (seed, printed) =
            cli::new_group(node, name("house"), name("Sam"), None, &[], &no_relay())
                .await
                .expect("new makes a group");
        let id = seed.id();
        eprintln!(
            "new: a client's link, {} characters: {}",
            printed.client.len(),
            printed.client
        );
        assert!(printed.client.starts_with(CLIENT_PREFIX), "{printed}");
        assert!(printed.client.len() <= 90, "{printed}");
        assert_eq!(printed.https, None, "no origin, so no https link");
        let link = Link::parse(&printed.client).expect("the printed link parses");
        assert_eq!(
            issuer::key(&link.token),
            node.net().id(),
            "the link dials this computer"
        );
        assert!(link.token.relays.is_empty(), "no relay to carry");
        let invites = invites(node, &id);
        assert_eq!(invites.len(), 1, "{invites:?}");
        assert_eq!(
            invites[0].0,
            key_of(&printed),
            "the Invite names the link's key"
        );
        assert!(
            (invites[0].1 - now - LINK_LIFE_SECS).abs() <= 5,
            "it lapses in a day"
        );
        let Standing::Members(standing) = node.rail().standing(&id).unwrap() else {
            panic!("the group is held and seeded")
        };
        assert_eq!(
            standing
                .values()
                .map(|m| m.person.as_str())
                .collect::<Vec<_>>(),
            ["Sam"]
        );
        ids.push(id);
    }
    assert_ne!(ids[0], ids[1], "two groups called house are two groups");
    let held = group::groups(sam.dir.path());
    assert_eq!(held.len(), 2);
    assert!(held.iter().all(|s| s.name().as_str() == "house"));
}

/// **A link made the moment a new group shows lets its joiner in**
/// (capability-scenes-found (e)): an issuer's newer `Invite` closes their
/// last (`membership`, "their one open link"), so `new`'s first link,
/// minted after the group was kept, closed one the home page made in
/// between, and its joiner was refused "this link lets nobody new in"
/// (capability-pass-key-4's Grapevine scene). Here the home page's link is
/// asked with no pause from the instant the group is held, as `new` runs,
/// for a run of groups: each time, the last link this computer made, the
/// one its record holds open, is the home page's. Kept: `new`'s own first
/// link is in the record, for a group nobody pressed Invite in.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_link_made_as_soon_as_a_new_group_shows_is_the_open_one() {
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let node = &sam.node;
    let runs = 30;
    tokio::task::block_in_place(|| {
        for run in 0..runs {
            let before = node.rail().namespaces().unwrap();
            let (made, home) = std::thread::scope(|s| {
                let home = s.spawn(|| loop {
                    let held = node.rail().namespaces().unwrap();
                    let Some(id) = held.into_iter().find(|g| !before.contains(g)) else {
                        continue;
                    };
                    if let Ok(linked) = node.host().link(&id, false, None, None) {
                        break (id, actor_of(&linked.token.key()));
                    }
                });
                let made = tokio::runtime::Handle::current().block_on(cli::new_group(
                    node,
                    name("house"),
                    name("Sam"),
                    None,
                    &[],
                    &no_relay(),
                ));
                (made.expect("new makes a group"), home.join().unwrap())
            });
            let (seed, printed) = made;
            assert_eq!(home.0, seed.id());
            let keys: Vec<String> = invites(node, &seed.id()).into_iter().map(|i| i.0).collect();
            assert!(
                keys.contains(&key_of(&printed)),
                "run {run}: new's own link is kept: {keys:?}"
            );
            assert_eq!(
                keys.last(),
                Some(&home.1),
                "run {run}: the home page's link, made once the group showed, is closed by new's: {keys:?}"
            );
        }
    });
}

/// `link` appends a new `Invite` to the group its name names, in any case,
/// and prints a link to it; two groups of one name are told apart by the start of
/// their id, and a name no group has is refused.
#[tokio::test]
async fn link_mints_a_new_link_to_the_group_it_names() {
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let node = &sam.node;
    let (house, first) = cli::new_group(node, name("House"), name("Sam"), None, &[], &no_relay())
        .await
        .unwrap();
    let (again, second, _) = cli::link_to(node, "house", None, &no_relay(), false, &[])
        .await
        .unwrap();
    assert_eq!(again, house);
    assert_ne!(first, second, "a new link");
    let keys: Vec<String> = invites(node, &house.id())
        .into_iter()
        .map(|(k, _, _)| k)
        .collect();
    assert_eq!(keys, [key_of(&first), key_of(&second)]);

    assert!(matches!(
        cli::link_to(node, "work", None, &no_relay(), false, &[]).await,
        Err(Refusal::Refused(Refused::NoGroup(_)))
    ));
    let (other, _) = cli::new_group(node, name("house"), name("Sam"), None, &[], &no_relay())
        .await
        .unwrap();
    assert!(matches!(
        cli::link_to(node, "house", None, &no_relay(), false, &[]).await,
        Err(Refusal::Refused(Refused::Ambiguous { .. }))
    ));
    let (picked, _, _) = cli::link_to(node, &other.id()[..8], None, &no_relay(), false, &[])
        .await
        .unwrap();
    assert_eq!(picked, other);
}

/// The page an https link opens on is the group's (u2-7), and nothing
/// depends on one (u2-8): `new --origin` keeps it, so `link` with no flag
/// prints the client's link and the https one there; `link --origin`
/// changes it from then on; a group made with no `--origin` prints the
/// client's link alone, and so does a group whose kept origin is gone.
#[tokio::test]
async fn a_group_keeps_the_origin_its_links_print_on() {
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let node = &sam.node;
    let https = |p: &Printed| {
        assert!(p.client.starts_with(CLIENT_PREFIX), "{p}");
        p.https.clone().unwrap_or_default()
    };
    let x = Origin::parse("https://x.example/").unwrap();
    let (house, first) =
        cli::new_group(node, name("house"), name("Sam"), Some(&x), &[], &no_relay())
            .await
            .unwrap();
    assert!(https(&first).starts_with("https://x.example/#"), "{first}");
    let (_, later, _) = cli::link_to(node, "house", None, &no_relay(), false, &[])
        .await
        .unwrap();
    assert!(https(&later).starts_with("https://x.example/#"), "{later}");

    let y = Origin::parse("https://y.example/ours/").unwrap();
    let (_, moved, _) = cli::link_to(node, "house", Some(&y), &no_relay(), false, &[])
        .await
        .unwrap();
    assert!(
        https(&moved).starts_with("https://y.example/ours/#"),
        "{moved}"
    );
    let (_, after, _) = cli::link_to(node, "house", None, &no_relay(), false, &[])
        .await
        .unwrap();
    assert!(
        https(&after).starts_with("https://y.example/ours/#"),
        "{after}"
    );

    let (work, plain) = cli::new_group(node, name("work"), name("Sam"), None, &[], &no_relay())
        .await
        .unwrap();
    assert_eq!(https(&plain), "", "no origin, no https link: {plain}");
    let (_, again, _) = cli::link_to(node, "work", None, &no_relay(), false, &[])
        .await
        .unwrap();
    assert_eq!(https(&again), "", "{again}");

    // A group whose kept origin is gone prints the client's link alone.
    let kept = node.rail().group_dir(&house.id()).unwrap().join("origin");
    std::fs::remove_file(&kept).unwrap();
    let (_, gone, _) = cli::link_to(node, "house", None, &no_relay(), false, &[])
        .await
        .unwrap();
    assert_eq!(https(&gone), "", "{gone}");
    assert_eq!(invites(node, &house.id()).len(), 5);
    assert_eq!(invites(node, &work.id()).len(), 2);
}

/// **The bare command prints where apps reach the record** (u3-2): a
/// loopback address, `http://127.0.0.1:<port>`, the one the node's API is
/// served at, and its routes answer the record's doors — here a group this
/// computer does not hold, refused by kind. That the API is served on its
/// port is record_api.rs's.
#[tokio::test]
async fn the_bare_command_prints_where_apps_reach_the_record() {
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let node = &sam.node;
    let said = cli::running(sam.dir.path(), node).await.to_string();
    let url = said
        .split_whitespace()
        .find_map(|w| w.strip_prefix("http://"))
        .map(|w| w.trim_end_matches('.'))
        .unwrap_or_else(|| panic!("no address printed: {said:?}"));
    assert!(url.starts_with("127.0.0.1:"), "{url}");
    assert_eq!(url, node.api_addr().to_string(), "where its API is asked");
    let at = "/v1/rail/people?namespace=elsewhere";
    let (status, text) = ask(node, "GET", at, &serde_json::Value::Null).await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND, "{text}");
    let body: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(body["kind"], "not_held");
}

/// **`svrngs new` while svrngs runs asks it to make the group**
/// (new-while-running): the installer runs svrngs at every login, and
/// `new` refused every installed person "svrngs is already running on
/// this computer (… is held); stop it, then run this again" (cli.rs
/// `Refused::Busy`), though `svrngs help` and the guide teach it. Sam's
/// svrngs holds the data dir's lock and serves its record; `svrngs new
/// house --as Sam --relay <url>`, asked through `cli::Here` of its routes,
/// says "Made house, with you as Sam.", a link that dials Sam's computer
/// and carries the relay, and house's folder where Sam's svrngs places
/// folders; it starts nothing (never "Leave this running"). The running
/// svrngs holds house, met at that relay, with the one `Invite` the link
/// names; a second `new house` says the first is there already; a page
/// asked with `--origin` is kept; the branch is traced. Watched failing on the code before it: the lock's
/// refusal, "svrngs is already running on this computer".
#[tokio::test]
async fn new_asks_the_svrngs_running_here_to_make_the_group() {
    let (heard, _heard) = crate::node_common::heard("cli=debug,group=debug");
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let _held = cli::claim(sam.dir.path()).expect("Sam's svrngs holds the data dir");
    let relay = "http://10.0.0.1:3340/";
    let new = |args: &str| {
        let argv = format!("svrngs new {args}");
        cli::parse(argv.split_whitespace().map(str::to_string)).unwrap()
    };
    let asked = new(&format!("house --as Sam --relay {relay}"));
    let said = cli::answer(asked, &crate::Running(&sam), &no_relay()).await;
    let said = said.expect("new, asked of the svrngs running here");
    eprintln!("svrngs new house, svrngs running: {said}");
    assert!(said.contains("Made house, with you as Sam."), "{said}");
    assert!(
        said.contains("Send this link to the people you want in it."),
        "{said}"
    );
    assert!(!said.contains("already running"), "not refused: {said}");
    assert!(!said.contains("Leave this running"), "runs nothing: {said}");
    let line = said
        .lines()
        .find(|l| l.contains(CLIENT_PREFIX))
        .expect(&said);
    let link = Link::parse(line.trim()).expect("the link printed is whole");
    assert_eq!(
        issuer::key(&link.token),
        sam.node.net().id(),
        "it dials Sam"
    );
    assert!(
        link.token.relays.iter().any(|r| r.to_string() == relay),
        "the link carries the relay asked: {:?}",
        link.token.relays
    );
    let places = sam.dir.path().join("svrngs").display().to_string();
    assert!(
        said.lines()
            .any(|l| l.starts_with("house's folder is") && l.contains(&places)),
        "{said}"
    );
    let held = group::groups(sam.dir.path());
    let [house] = held.as_slice() else {
        panic!("one group held: {held:?}")
    };
    let register = sam.node.rail().register(&house.id()).unwrap();
    let meets: Vec<String> = (register.meets.unwrap_or_default().iter())
        .map(ToString::to_string)
        .collect();
    assert_eq!(meets, [relay], "house meets at the relay asked");
    let invites = invites(&sam.node, &house.id());
    assert_eq!(invites.len(), 1, "{invites:?}");
    assert_eq!(invites[0].0, actor_of(&link.token.key()), "{invites:?}");

    let again = cli::answer(new("house --as Sam"), &crate::Running(&sam), &no_relay()).await;
    let again = again.expect("a second house, asked of the svrngs running here");
    assert!(
        again.contains("You’re already in house (yours")
            && again.contains("This is a second group called house."),
        "{again}"
    );
    assert_eq!(group::groups(sam.dir.path()).len(), 2);

    // `--origin`: the command checks the page (`checked_page`, a listener,
    // new.rs's), then asks with it, and the svrngs running keeps it.
    let page = "https://x.example/ours/";
    let asked = serde_json::json!({ "name": "trip", "person": "Sam", "origin": page });
    let (status, made) = ask(&sam.node, "POST", group::NEW_ROUTE, &asked).await;
    assert!(status.is_success(), "{made}");
    let made: group::NewMade = serde_json::from_str(&made).unwrap();
    let https = made.https.unwrap_or_default();
    assert!(https.starts_with(&format!("{page}#")), "{https}");

    let heard = String::from_utf8(heard.0.lock().unwrap().clone()).unwrap();
    for trace in [
        "cli: new — svrngs runs here already, so it is asked to make the group",
        "group: made for the command's `new` by the svrngs running here",
    ] {
        assert!(heard.contains(trace), "{trace}: {heard}");
    }
}

/// **`svrngs new` through the svrngs running here says when it has
/// reached no relay** (new-while-running): a `new` that starts svrngs
/// waits for a home relay and, reaching none, says "after 10 s, this
/// computer has not yet reached the server your groups find it through"
/// (cli.rs `wait_online`); asked of the svrngs running here, it said
/// nothing of it. Sam's svrngs meets at the build's relay, down on the
/// wire, so it is not online (`Host::online`, its wait on a paused clock):
/// the group is made all the same, the route's answer says `online:
/// false`, and the command says so, as `wait_online` does, traced.
/// Watched failing on 005420b9: the answer had no `online`.
#[tokio::test(start_paused = true)]
async fn new_through_a_running_svrngs_that_reached_no_relay_says_so() {
    let (heard, _heard) = crate::node_common::heard("cli=debug");
    let wire = Wire::default();
    let build = "http://10.0.0.3:3340/";
    let ours = svrngs_node::iroh::RelayConfig {
        relay_urls: vec![build.into()],
        n0_services: false,
        loopback_only: false,
    };
    let key = svrngs_node::SigningKey::from_bytes(&[5; 32]);
    let dir = tempfile::tempdir().unwrap();
    let every = std::time::Duration::from_secs(60);
    let at = (dir.path().to_path_buf(), key.clone());
    let node = Node::on_wire_relayed(&wire, at, &ours, &[], every).await;
    let node = node.expect("Sam's svrngs starts, met at the build's relay");
    let sam = crate::Computer { node, key, dir };
    let _held = cli::claim(sam.dir.path()).expect("Sam's svrngs holds the data dir");

    let asked = serde_json::json!({ "name": "work", "person": "Sam" });
    let (status, made) = ask(&sam.node, "POST", group::NEW_ROUTE, &asked).await;
    assert!(status.is_success(), "{made}");
    let made: serde_json::Value = serde_json::from_str(&made).unwrap();
    assert_eq!(made["online"], false, "it says it reached no relay: {made}");

    let verb = cli::parse(["svrngs", "new", "house", "--as", "Sam"].map(String::from));
    let said = cli::answer(verb.unwrap(), &crate::Running(&sam), &ours).await;
    let said = said.expect("house is made all the same");
    assert!(said.contains("Made house, with you as Sam."), "{said}");
    let heard = String::from_utf8(heard.0.lock().unwrap().clone()).unwrap();
    assert!(
        heard.contains("cli: this computer has reached no relay, and is told so"),
        "{heard}"
    );
}
