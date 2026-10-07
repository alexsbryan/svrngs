//! `svrngs new` and `svrngs link` (DESIGN §8, U2): a group made, its seed
//! written beside its record, an `Invite` appended through its door, and a
//! short link printed whose key the `Invite` names (u2-8). One node in one
//! process, no relay.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use svrngs::cli::{self, Printed, Refusal, Verb, LINK_LIFE_SECS};
use svrngs::group::{self, Name};
use svrngs::issuer;
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::link::{Link, Origin, CLIENT_PREFIX};
use svrngs_node::node::Node;
use svrngs_node::register::Standing;
use svrngs_node::{actor_of, derive_id, Ed25519Verifier, OpId, RailAct, SigningKey};

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

fn no_relay() -> RelayConfig {
    RelayConfig::from_parts(Vec::new(), Some("none")).unwrap()
}

async fn start(dir: &tempfile::TempDir) -> Node {
    Node::start(
        dir.path().to_path_buf(),
        SigningKey::from_bytes(&[1; 32]),
        &no_relay(),
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// The key a printed link's `Invite` names.
fn key_of(printed: &Printed) -> String {
    let link = Link::parse(&printed.client).expect("the client's link parses");
    actor_of(&link.token.key())
}

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

/// The `Invite`s a group's record holds: (link key, until, the id its
/// content derives).
fn invites(node: &Node, id: &str) -> Vec<(String, i64, OpId)> {
    let ops = lines(node, id);
    ops.into_iter()
        .filter_map(|op| match &op.kind.act {
            RailAct::Invite { key, until } => Some((key.clone(), *until, derive_id(&op, id))),
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
    let dir = tempfile::tempdir().unwrap();
    let node = start(&dir).await;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut ids = Vec::new();
    for _ in 0..2 {
        let (seed, printed) = cli::new_group(&node, name("house"), name("Sam"), None, &no_relay())
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
            node.endpoint().id(),
            "the link dials this computer"
        );
        assert_eq!(link.token.relay, None, "no relay to carry");
        let invites = invites(&node, &id);
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
        let Standing::Members(standing) = node.rail().standing(&id, &Ed25519Verifier).unwrap()
        else {
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
    let held = group::groups(dir.path());
    assert_eq!(held.len(), 2);
    assert!(held.iter().all(|s| s.name().as_str() == "house"));
}

/// `link` appends a new `Invite` to the group its name names, in any case,
/// and prints a link to it; two groups of one name are told apart by the start of
/// their id, and a name no group has is refused.
#[tokio::test]
async fn link_mints_a_new_link_to_the_group_it_names() {
    let dir = tempfile::tempdir().unwrap();
    let node = start(&dir).await;
    let (house, first) = cli::new_group(&node, name("House"), name("Sam"), None, &no_relay())
        .await
        .unwrap();
    let (again, second) = cli::new_link(&node, "house", None, &no_relay())
        .await
        .unwrap();
    assert_eq!(again, house);
    assert_ne!(first, second, "a new link");
    let keys: Vec<String> = invites(&node, &house.id())
        .into_iter()
        .map(|(k, _, _)| k)
        .collect();
    assert_eq!(keys, [key_of(&first), key_of(&second)]);

    assert!(matches!(
        cli::new_link(&node, "work", None, &no_relay()).await,
        Err(Refusal::NoGroup(_))
    ));
    let (other, _) = cli::new_group(&node, name("house"), name("Sam"), None, &no_relay())
        .await
        .unwrap();
    assert!(matches!(
        cli::new_link(&node, "house", None, &no_relay()).await,
        Err(Refusal::Ambiguous { .. })
    ));
    let (picked, _) = cli::new_link(&node, &other.id()[..8], None, &no_relay())
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
    let dir = tempfile::tempdir().unwrap();
    let node = start(&dir).await;
    let https = |p: &Printed| {
        assert!(p.client.starts_with(CLIENT_PREFIX), "{p}");
        p.https.clone().unwrap_or_default()
    };
    let x = Origin::parse("https://x.example/").unwrap();
    let (house, first) = cli::new_group(&node, name("house"), name("Sam"), Some(&x), &no_relay())
        .await
        .unwrap();
    assert!(https(&first).starts_with("https://x.example/#"), "{first}");
    let (_, later) = cli::new_link(&node, "house", None, &no_relay())
        .await
        .unwrap();
    assert!(https(&later).starts_with("https://x.example/#"), "{later}");

    let y = Origin::parse("https://y.example/ours/").unwrap();
    let (_, moved) = cli::new_link(&node, "house", Some(&y), &no_relay())
        .await
        .unwrap();
    assert!(
        https(&moved).starts_with("https://y.example/ours/#"),
        "{moved}"
    );
    let (_, after) = cli::new_link(&node, "house", None, &no_relay())
        .await
        .unwrap();
    assert!(
        https(&after).starts_with("https://y.example/ours/#"),
        "{after}"
    );

    let (work, plain) = cli::new_group(&node, name("work"), name("Sam"), None, &no_relay())
        .await
        .unwrap();
    assert_eq!(https(&plain), "", "no origin, no https link: {plain}");
    let (_, again) = cli::new_link(&node, "work", None, &no_relay())
        .await
        .unwrap();
    assert_eq!(https(&again), "", "{again}");

    // A group whose kept origin is gone prints the client's link alone.
    let kept = node.rail().group_dir(&house.id()).unwrap().join("origin");
    std::fs::remove_file(&kept).unwrap();
    let (_, gone) = cli::new_link(&node, "house", None, &no_relay())
        .await
        .unwrap();
    assert_eq!(https(&gone), "", "{gone}");
    assert_eq!(invites(&node, &house.id()).len(), 5);
    assert_eq!(invites(&node, &work.id()).len(), 2);
}

/// A link carries a relay only from a computer without address lookup,
/// which finds a bare key otherwise: its home relay, or the first one
/// configured while it reports none, read without the root's dot (u4-9,
/// svrngs_node::relay); with neither, the key alone.
#[test]
fn a_link_carries_the_relay_only_without_lookup() {
    let ours: RelayUrl = "https://relay.example./".parse().unwrap();
    let home: RelayUrl = "https://home.example./".parse().unwrap();
    let lookup = RelayConfig::from_parts(vec![ours.to_string()], None).unwrap();
    assert_eq!(cli::link_relay(&lookup, Some(home.clone())), None);
    assert_eq!(cli::link_relay(&RelayConfig::default(), None), None);
    let without = RelayConfig::from_parts(vec![ours.to_string()], Some("none")).unwrap();
    assert_eq!(cli::link_relay(&without, Some(home.clone())), Some(home));
    let dotless: RelayUrl = "https://relay.example/".parse().unwrap();
    assert_ne!(ours, dotless, "control: the operator wrote the dot");
    assert_eq!(cli::link_relay(&without, None), Some(dotless));
    assert_eq!(cli::link_relay(&no_relay(), None), None);
}

/// One svrngs per data dir: a second claim is refused while the first is
/// held, and taken once it is let go.
#[test]
fn a_second_svrngs_on_one_data_dir_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let held = cli::claim(dir.path()).expect("the first claim");
    assert!(matches!(cli::claim(dir.path()), Err(Refusal::Busy(_))));
    drop(held);
    cli::claim(dir.path()).expect("free again");
}

/// A group's seed or origin that cannot be read is named, never read as
/// none kept (ARCH 6): none kept is only no file there. Each is planted as
/// a directory, which no read opens.
#[test]
fn an_unreadable_seed_or_origin_refuses_naming_its_file() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(group::read_seed(dir.path()), Ok(None)));
    assert!(matches!(group::read_origin(dir.path()), Ok(None)));
    for file in ["seed.json", "origin"] {
        std::fs::create_dir(dir.path().join(file)).unwrap();
    }
    for (file, read) in [
        ("seed.json", group::read_seed(dir.path()).map(|_| ())),
        ("origin", group::read_origin(dir.path()).map(|_| ())),
    ] {
        let refused = read.expect_err(file).to_string();
        assert!(refused.contains(file), "{refused}");
    }
}

fn argv(s: &str) -> Vec<String> {
    std::iter::once("svrngs".to_string())
        .chain(s.split_whitespace().map(str::to_string))
        .collect()
}

#[test]
fn the_verbs_parse() {
    assert_eq!(cli::parse(argv("")).unwrap(), Verb::Run);
    assert_eq!(
        cli::parse(argv("new house --as Sam")).unwrap(),
        Verb::New {
            name: name("house"),
            person: Some(name("Sam")),
            origin: None,
        }
    );
    assert_eq!(
        cli::parse(argv("link house --origin https://links.example.org/")).unwrap(),
        Verb::Link {
            group: "house".into(),
            origin: Some(Origin::parse("https://links.example.org/").unwrap()),
        }
    );
    assert_eq!(
        cli::parse(argv("link house")).unwrap(),
        Verb::Link {
            group: "house".into(),
            origin: None,
        },
        "no --origin is the group's own, not the default"
    );
    assert_eq!(cli::parse(argv("--help")).unwrap(), Verb::Help);
}

/// Every one of these is a guess some hand-rolled parser has made: a
/// missing value, an unknown flag, a flag on the wrong verb, a typo'd verb.
#[test]
fn a_wrong_argument_is_an_error_not_a_guess() {
    for wrong in [
        "new",
        "link",
        "new house --as",
        "new house --forever",
        "link house --as Sam",
        "new house extra",
        "new house --origin svrnme.sh",
        "neww house",
    ] {
        assert!(cli::parse(argv(wrong)).is_err(), "{wrong}");
    }
}

/// Nothing a refusal of `new` or `link` says uses a word DESIGN §1 keeps
/// from people.
#[test]
fn refusals_use_only_a_persons_words() {
    let refusals = [
        Refusal::NoDataDir,
        Refusal::Busy("/home/sam/.svrngs/lock".into()),
        Refusal::Start,
        Refusal::NoName(name("house")),
        Refusal::Keep(name("house")),
        Refusal::NoGroup("work".into()),
        Refusal::Ambiguous {
            name: "house".into(),
            ids: vec!["3f2a91c0".into()],
        },
        Refusal::NotIn(name("house")),
    ];
    // This computer's key, too short to read, refused with its file named;
    // the temporary directory's own name is not the command's to choose.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(svrngs::identity::NODE_KEY_FILE), b"short").unwrap();
    let unreadable = svrngs::identity::load_or_generate_node_key(dir.path()).unwrap_err();
    let refusals = refusals.into_iter().chain([Refusal::Key(unreadable)]);
    for r in refusals {
        let said = r
            .to_string()
            .replace(&dir.path().display().to_string(), "")
            .to_lowercase();
        for word in ["ring", "member", "key", "sync", "journal", "node", "relay"] {
            assert!(!said.contains(word), "{said:?} says {word}");
        }
    }
}

/// **The bare command prints where apps reach the record** (u3-2): a
/// loopback address, `http://127.0.0.1:<port>`, and that address answers
/// the record's doors — here a group this computer does not hold, refused
/// by kind.
#[tokio::test]
async fn the_bare_command_prints_where_apps_reach_the_record() {
    let dir = tempfile::tempdir().unwrap();
    let node = start(&dir).await;
    let said = cli::running(dir.path(), &node).await.to_string();
    let url = said
        .split_whitespace()
        .find_map(|w| w.strip_prefix("http://"))
        .map(|w| w.trim_end_matches('.'))
        .unwrap_or_else(|| panic!("no address printed: {said:?}"));
    assert!(url.starts_with("127.0.0.1:"), "{url}");
    let answer = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(format!("http://{url}/v1/rail/people?namespace=elsewhere"))
        .send()
        .await
        .expect("the printed address answers");
    assert_eq!(answer.status(), reqwest::StatusCode::NOT_FOUND);
    let body: serde_json::Value = serde_json::from_str(&answer.text().await.unwrap()).unwrap();
    assert_eq!(body["kind"], "not_held");
    node.endpoint().close().await;
}

/// A stand-in for a page's host on 127.0.0.1, at `localhost`, whose every
/// name this computer answers: at `localhost` itself it serves the page;
/// at every other name, what `elsewhere` says.
async fn page_host(elsewhere: Elsewhere) -> Origin {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    const PAGE: &str = "<!doctype html><title>the group's page</title>";
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut conn, _)) = listener.accept().await {
            tokio::spawn(async move {
                let (mut head, mut buf) = (Vec::new(), [0u8; 1024]);
                while !head.windows(4).any(|w| w == b"\r\n\r\n") {
                    match conn.read(&mut buf).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => head.extend_from_slice(&buf[..n]),
                    }
                }
                let head = String::from_utf8_lossy(&head).to_ascii_lowercase();
                let at_page = head.contains(&format!("\r\nhost: localhost:{port}\r\n"));
                let (status, body) = match (at_page, elsewhere) {
                    (true, _) | (false, Elsewhere::Page) => ("200 OK", PAGE),
                    (false, Elsewhere::Other) => ("200 OK", "<!doctype html>another site"),
                    (false, Elsewhere::Nothing) => ("404 Not Found", ""),
                };
                let answer = format!(
                    "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = conn.write_all(answer.as_bytes()).await;
            });
        }
    });
    Origin::parse(&format!("http://localhost:{port}/")).unwrap()
}

/// A page refused for giving no app an address of its own, the reason
/// saying what its host must do.
#[track_caller]
fn no_app_address(refused: &Refusal) {
    assert!(matches!(refused, Refusal::NoAppAddress(_)), "{refused}");
    assert_eq!(refused.exit_code(), 1);
    let said = refused.to_string();
    assert!(
        said.contains("serves your page at every name under it"),
        "the refusal says what the host must do: {said}"
    );
}

/// What a stand-in page host serves at a name other than its own.
#[derive(Debug, Clone, Copy)]
enum Elsewhere {
    Page,
    Other,
    Nothing,
}

/// **Each app at an address of its own only where the page opens there**
/// (the u5-10 row; DESIGN §7): a host whose every name serves the page
/// opens each app at an address of its own; one whose every name resolves
/// but serves something else, or nothing, is refused, the reason saying
/// what the host must do — as `www.svrnme.sh/g/` was not, its every name resolving
/// while none but its own served the page. Planted: today's check, which
/// asked only whether a name nobody made resolves.
#[tokio::test]
async fn each_app_opens_at_its_own_address_only_where_the_page_opens_there() {
    let host = page_host(Elsewhere::Page).await;
    assert_eq!(cli::checked_page(host.clone()).await.unwrap(), host);
    for elsewhere in [Elsewhere::Other, Elsewhere::Nothing] {
        let refused = cli::checked_page(page_host(elsewhere).await).await;
        let Err(refused) = refused else {
            panic!("a host serving {elsewhere:?} at other names is refused: {refused:?}")
        };
        no_app_address(&refused);
    }
}

/// **Where a group's apps open on a phone** (the u5-2 row; DESIGN §7): a
/// page origin whose host serves the page at every name under it — a
/// stand-in at `localhost`, whose every name this computer answers, as
/// group.svrnme.sh's are — opens each app at an address of its own, and
/// the link carries nothing more, then or on the next; an origin at an
/// address no name falls under is refused, the reason saying what its host
/// must do; a page address that does not give the page is not judged (exit
/// 3). `new` prints where the apps open. Planted: an IP address taken as a
/// name, so 127.0.0.1's page is asked for and not judged (u6-00).
#[tokio::test]
async fn a_page_origin_says_where_its_apps_open_or_is_refused() {
    let page = |raw: &str| Origin::parse(raw).unwrap();
    let ours = cli::checked_page(page_host(Elsewhere::Page).await)
        .await
        .unwrap();
    let at = ours
        .as_str()
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string();

    let refused = cli::checked_page(page("http://127.0.0.1:8123/"))
        .await
        .unwrap_err();
    no_app_address(&refused);
    let away = cli::checked_page(page("https://nowhere.invalid/"))
        .await
        .unwrap_err();
    assert!(matches!(away, Refusal::CannotCheck(_)), "{away}");
    assert_eq!(away.exit_code(), 3);

    let dir = tempfile::tempdir().unwrap();
    let node = start(&dir).await;
    let (_, own) = cli::new_group(&node, name("house"), name("Sam"), Some(&ours), &no_relay())
        .await
        .unwrap();
    let https = own.https.clone().unwrap();
    assert!(
        https.starts_with(&format!("{}#", ours.as_str())) && !https.contains('~'),
        "the link carries nothing more: {https}"
    );
    assert!(
        own.to_string().contains(&format!(
            "each app opens at an address of its own under {at}"
        )),
        "{own}"
    );
    let (_, again) = cli::new_link(&node, "house", None, &no_relay())
        .await
        .unwrap();
    assert!(
        again
            .https
            .clone()
            .unwrap()
            .starts_with(&format!("{}#", ours.as_str())),
        "kept for the next link: {again}"
    );
    for said in [own.to_string(), refused.to_string(), away.to_string()] {
        for word in svrngs_node::link::KEPT_WORDS {
            assert!(!said.to_lowercase().contains(word), "{said:?} says {word}");
        }
    }
    node.endpoint().close().await;
}

/// **`room` names lends, and says who can open them in a person's words**
/// (u6-1): one name or more, each a lend's; with none, or one that is not
/// a name, an error. What it prints and its refusals use no word DESIGN §1
/// keeps from people.
#[test]
fn room_names_lends_and_says_who_can_open_them() {
    let lend = |n: &str| svrngs_node::lend::LendName::parse(n).unwrap();
    assert_eq!(
        cli::parse(argv("room jellyfin music")).unwrap(),
        Verb::Room(vec![lend("jellyfin"), lend("music")])
    );
    for wrong in [
        "room",
        "room jelly/fin",
        "room jellyfin --as Sam",
        "share 8096 extra",
    ] {
        assert!(cli::parse(argv(wrong)).is_err(), "{wrong}");
    }
    let offered = serde_json::from_value(serde_json::json!({
        "name": "jellyfin", "kind": "jellyfin", "limit": 3, "using": 0, "upload_bps": null
    }))
    .unwrap();
    let lent = svrngs_node::node::Lent {
        person: svrngs_node::Person::parse("sam").unwrap(),
        key: "ab".repeat(32),
        offered,
        address: "http://x.localhost:1/".into(),
    };
    let said = cli::Said::Room(vec![(lent, "192.168.1.4:45053".parse().unwrap())]).to_string();
    assert!(
        said.contains("sam's jellyfin: http://192.168.1.4:45053/"),
        "{said}"
    );
    println!("{said}");
    let refusals = [
        Refusal::NoRoomAddress,
        Refusal::Room(svrngs_node::room::RoomError::NotLent(lend("jellyfin"))),
    ];
    for line in refusals.iter().map(ToString::to_string).chain([said]) {
        let lower = line.to_lowercase();
        for word in ["ring", "member", "key", "sync", "journal", "node", "relay"] {
            assert!(!lower.contains(word), "{line:?} says {word}");
        }
    }
}
