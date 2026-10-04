//! `svrngs new` and `svrngs link` (DESIGN §8, U2): a group made, its seed
//! written beside its record, an `Invite` appended through its door, and a
//! link printed that parses back to the group. One node in one process, no
//! relay: its link dials its loopback address.

use svrngs::cli::{self, Command, Refusal, LINK_LIFE_SECS};
use svrngs::group::{self, Name};
use svrngs::link::{parse_https_link, Origin};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::register::Standing;
use svrngs_node::{Ed25519Verifier, RailAct, SigningKey};

fn no_relay() -> RelayConfig {
    RelayConfig::from_parts(Vec::new(), Some("none")).unwrap()
}

async fn start(dir: &tempfile::TempDir) -> Node {
    Node::start(
        dir.path().to_path_buf(),
        SigningKey::from_bytes(&[1; 32]),
        &no_relay(),
    )
    .await
    .expect("a node starts")
}

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

/// The `Invite`s a group's record holds: (link key, until).
fn invites(node: &Node, id: &str) -> Vec<(String, i64)> {
    let (ops, _) = node.rail().journal(id).unwrap().read().unwrap();
    ops.into_iter()
        .filter_map(|op| match op.kind.act {
            RailAct::Invite { key, until } => Some((key, until)),
            _ => None,
        })
        .collect()
}

/// `new` twice with one name makes two groups with two ids: each its own
/// seed and record, each founded by this computer as Sam, each with one
/// `Invite` naming its link's key for a day, and each link parsing back to
/// its own group.
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
        let (seed, url) = cli::new_group(
            &node,
            name("house"),
            name("Sam"),
            &Origin::default(),
            &no_relay(),
        )
        .await
        .expect("new makes a group");
        let id = seed.id();
        let link = parse_https_link(&url).expect("the printed link parses");
        assert_eq!(
            link.seed.id(),
            id,
            "the link names the group it was made for"
        );
        assert_eq!(
            link.issuer.addr().id,
            node.endpoint().id(),
            "the link dials this computer"
        );
        let invites = invites(&node, &id);
        assert_eq!(invites.len(), 1, "{invites:?}");
        assert_eq!(invites[0].0, link.key(), "the Invite names the link's key");
        assert!(
            (invites[0].1 - now - LINK_LIFE_SECS).abs() <= 5,
            "it lapses in a day"
        );
        let Standing::Members(standing) = node.rail().standing(&id, &Ed25519Verifier).unwrap()
        else {
            panic!("the group is held and seeded")
        };
        assert_eq!(
            standing.values().map(|p| p.as_str()).collect::<Vec<_>>(),
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
    let origin = Origin::default();
    let (house, first) = cli::new_group(&node, name("House"), name("Sam"), &origin, &no_relay())
        .await
        .unwrap();
    let (again, second) = cli::new_link(&node, "house", &origin, &no_relay())
        .await
        .unwrap();
    assert_eq!(again, house);
    assert_ne!(first, second, "a new link");
    let keys: Vec<String> = invites(&node, &house.id())
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    assert_eq!(
        keys,
        [
            parse_https_link(&first).unwrap().key(),
            parse_https_link(&second).unwrap().key()
        ]
    );

    assert!(matches!(
        cli::new_link(&node, "work", &origin, &no_relay()).await,
        Err(Refusal::NoGroup(_))
    ));
    let (other, _) = cli::new_group(&node, name("house"), name("Sam"), &origin, &no_relay())
        .await
        .unwrap();
    assert!(matches!(
        cli::new_link(&node, "house", &origin, &no_relay()).await,
        Err(Refusal::Ambiguous { .. })
    ));
    let (picked, _) = cli::new_link(&node, &other.id()[..8], &origin, &no_relay())
        .await
        .unwrap();
    assert_eq!(picked, other);
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

fn argv(s: &str) -> Vec<String> {
    std::iter::once("svrngs".to_string())
        .chain(s.split_whitespace().map(str::to_string))
        .collect()
}

#[test]
fn the_verbs_parse() {
    assert_eq!(cli::parse(argv("")).unwrap(), Command::Run);
    assert_eq!(
        cli::parse(argv("new house --as Sam")).unwrap(),
        Command::New {
            name: name("house"),
            person: Some(name("Sam")),
            origin: Origin::default(),
        }
    );
    assert_eq!(
        cli::parse(argv("link house --origin https://links.example.org/")).unwrap(),
        Command::Link {
            group: "house".into(),
            origin: Origin::parse("https://links.example.org/").unwrap(),
        }
    );
    assert_eq!(cli::parse(argv("--help")).unwrap(), Command::Help);
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
        Refusal::Unreachable,
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
