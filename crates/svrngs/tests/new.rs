//! `svrngs new` and `svrngs link` (PLAN §4, U2): the verbs parsed, the
//! help and the refusals in a person's words, the build's relays, and a
//! page's host asked whether it serves the page at every name under it (a
//! listener). A group made and its links, decided by `cli::new_group` and
//! `cli::link_to`, are verbs/new.rs's, on the wire (docs/seams.tsv).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use svrngs::cli::{self, Refusal, Verb};
use svrngs::group::{self, Name};
use svrngs_core::relay::Meeting;
use svrngs_node::invite::kept_page;
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::link::Origin;
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
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

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

/// A link carries its issuer's home relay, with address lookup or without
/// it (relay-park (c)), so a phone's page, which asks no lookup, opens it:
/// its home relay, or the build's first while it reports none, read
/// without the root's dot (u4-9, svrngs_node::relay); with neither, the key
/// alone (relay fallbacks: `Meeting::carried`, of a group naming none).
/// Planted: the key alone from a computer with lookup, as before.
#[test]
fn a_link_carries_its_issuer_s_relay_with_lookup_or_without() {
    let url = |u: &str| svrngs_core::relay::Url::parse(u).unwrap();
    let home = url("https://relay.example/");
    for n0_services in [true, false] {
        let relay = RelayConfig {
            relay_urls: vec!["https://relay.example./".into()],
            n0_services,
            loopback_only: false,
        };
        let said = format!("n0_services {n0_services}");
        let meeting = Meeting::of([], &relay.build());
        assert_eq!(
            meeting.carried(Some(&home)),
            [home.clone()],
            "{said}: its home relay"
        );
        assert_eq!(
            meeting.carried(None),
            [home.clone()],
            "{said}: the build's first, dotless"
        );
    }
    assert!(Meeting::of([], &no_relay().build())
        .carried(None)
        .is_empty());
}

/// The build's relays, which no group's record names and a joiner's file
/// keeps none of (`Host::keep_group`), so none outlasts the operator's
/// switch (relay-park (c)): n0's four at n0's, svrngs's own at svrngs's,
/// none without either. Planted: `RelayConfig::build` answering the
/// configured URLs alone, and n0's four were missing at n0's.
#[test]
fn the_build_s_relays_are_n0_s_at_n0_s_and_svrngs_s_at_svrngs_s() {
    let n0: Vec<RelayUrl> = svrngs_node::relay::n0_relays().urls();
    assert_eq!(n0.len(), 4, "control: n0's four, {n0:?}");
    let at_n0 = RelayConfig {
        relay_urls: Vec::new(),
        ..cli::reach(cli::MeetAt::N0)
    };
    for config in [cli::reach(cli::MeetAt::N0), at_n0] {
        assert_eq!(
            config.build(),
            n0.iter().cloned().map(Into::into).collect::<Vec<_>>()
        );
    }
    assert_eq!(
        cli::reach(cli::MeetAt::Svrngs).build(),
        [cli::DEFAULT_RELAY.parse().unwrap()]
    );
    assert!(no_relay().build().is_empty());
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

/// A group's seed or origin that cannot be read is refused, never read as
/// none kept (ARCH 6): none kept is only no file there (#20). Each is
/// planted as a directory, which no read opens; the seed's refusal names
/// its file. The origin is asked of `kept_page`, the reader every caller
/// runs, each saying its own refusal.
#[test]
fn an_unreadable_seed_or_origin_is_refused_never_none() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(group::read_seed(dir.path()), Ok(None)));
    assert!(matches!(kept_page(dir.path()), Ok(None)));
    for file in ["seed.json", "origin"] {
        std::fs::create_dir(dir.path().join(file)).unwrap();
    }
    let refused = group::read_seed(dir.path())
        .expect_err("seed.json")
        .to_string();
    assert!(refused.contains("seed.json"), "{refused}");
    kept_page(dir.path()).expect_err("an origin that cannot be read is refused");
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
            relays: Vec::new(),
        }
    );
    // --relay: the group's own, a name read as its https address
    // (relay-default (a)).
    for (typed, url) in [
        ("relay.example.org", "https://relay.example.org/"),
        ("https://relay.example.org./", "https://relay.example.org/"),
        ("http://127.0.0.1:3340", "http://127.0.0.1:3340/"),
    ] {
        let Verb::New { relays, .. } =
            cli::parse(argv(&format!("new house --relay {typed}"))).unwrap()
        else {
            panic!("new");
        };
        let said: Vec<String> = relays.iter().map(ToString::to_string).collect();
        assert_eq!(said, [url], "{typed}");
    }
    // Once for each relay the group meets at (relay fallbacks).
    let Verb::New { relays, .. } = cli::parse(argv(
        "new house --relay http://10.0.0.2:3340 --relay http://10.0.0.3:3340",
    ))
    .unwrap() else {
        panic!("new");
    };
    assert_eq!(relays.len(), 2);
    assert!(cli::parse(argv("new house --relay ::")).is_err());
    assert!(cli::parse(argv("join x --relay relay.example.org")).is_err());
    assert_eq!(
        cli::parse(argv("link house --origin https://links.example.org/")).unwrap(),
        Verb::Link {
            group: "house".into(),
            origin: Some(Origin::parse("https://links.example.org/").unwrap()),
            device: false,
            relays: Vec::new(),
        }
    );
    assert_eq!(
        cli::parse(argv("link house --device")).unwrap(),
        Verb::Link {
            group: "house".into(),
            origin: None,
            device: true,
            relays: Vec::new(),
        },
        "--device: a link for another device of yours (bash-devices)"
    );
    assert_eq!(
        cli::parse(argv("link house")).unwrap(),
        Verb::Link {
            group: "house".into(),
            origin: None,
            device: false,
            relays: Vec::new(),
        },
        "no --origin is the group's own, not the default"
    );
    assert_eq!(cli::parse(argv("--help")).unwrap(), Verb::Help);
}

/// **`--help` opens with how to build an app, and `guide` prints the
/// guide whole** (svrn-docs `rings/current/EXTENSION_POINT.md` §7, G4,
/// G6): in one agent run (2026-11-09), `--help` was all it read, and it
/// never learned of a folder app or that the group's folder is kept by
/// every member. The help's first paragraph names the folder, the route,
/// the permanence, a group of one's own and `svrngs guide`, which the
/// binary answers with docs/guide/build.md byte for byte. Planted, watched
/// failing: the help as 25444fb6 left it, and `guide` as an unknown command.
#[test]
fn the_help_opens_with_how_to_build_an_app_and_the_guide_prints_whole() {
    let opening = cli::USAGE.split("\n\n").nth(1).unwrap();
    for said in [
        "~/svrngs/<group>/apps/<name>/index.html",
        "fetch('svrngs/acts')",
        "goes to every member",
        "for good",
        "(`svrngs new`)",
        "`svrngs guide`",
    ] {
        assert!(
            opening.contains(said),
            "the help's opening lacks {said}: {opening}"
        );
    }
    assert_eq!(cli::parse(argv("guide")).unwrap(), Verb::Guide);
    let printed = std::process::Command::new(env!("CARGO_BIN_EXE_svrngs"))
        .arg("guide")
        .output()
        .unwrap();
    assert!(printed.status.success(), "{printed:?}");
    let guide = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/guide/build.md");
    let guide = std::fs::read_to_string(guide).unwrap();
    assert_eq!(String::from_utf8(printed.stdout).unwrap(), guide);
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
        Refusal::Keep("house".into()),
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
    assert_eq!(
        cli::checked_page(Some(host.clone())).await.unwrap(),
        Some(host)
    );
    for elsewhere in [Elsewhere::Other, Elsewhere::Nothing] {
        let refused = cli::checked_page(Some(page_host(elsewhere).await)).await;
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
    let page = |raw: &str| Some(Origin::parse(raw).unwrap());
    let ours = cli::checked_page(Some(page_host(Elsewhere::Page).await))
        .await
        .unwrap()
        .expect("a page given is a page checked");
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
    let (_, own) = cli::new_group(
        &node,
        name("house"),
        name("Sam"),
        Some(&ours),
        &[],
        &no_relay(),
    )
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
    let (_, again, _) = cli::link_to(&node, "house", None, &no_relay(), false, &[])
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
/// a name, an error. Each lend on is said with where the room serves it,
/// in the one wording the home page says too (room.rs `Served::said`). What it prints and its refusals use no word DESIGN §1
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
    let served: svrngs_node::room::Served =
        serde_json::from_value(serde_json::json!({ "at": "192.168.1.4:45053" })).unwrap();
    let on = svrngs_node::room::InRoom {
        person: Some(svrngs_node::Person::parse("sam").unwrap()),
        key: "ab".repeat(32),
        name: lend("jellyfin"),
        served,
        said: served.said(),
    };
    let said = cli::Said::Room(vec![on]).to_string();
    assert!(
        said.contains(
            "sam's jellyfin: Open to this WiFi: type http://192.168.1.4:45053/ into an app or a browser on a device here"
        ),
        "{said}"
    );
    println!("{said}");
    let refusals = [Refusal::Room(
        "nobody in your groups lends jellyfin to this computer now".into(),
    )];
    for line in refusals.iter().map(ToString::to_string).chain([said]) {
        let lower = line.to_lowercase();
        for word in ["ring", "member", "key", "sync", "journal", "node", "relay"] {
            assert!(!lower.contains(word), "{line:?} says {word}");
        }
    }
}
