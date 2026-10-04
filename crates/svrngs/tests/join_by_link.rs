//! Join by link, U2's bar (DESIGN §8): Sam makes a group and sends one
//! link; Alex, Bo and Cy each open it with `svrngs join` and are let in
//! once, by Sam's computer, and each copy syncs Sam's record. A closed
//! link, a lapsed one and a forged one are refused in one sentence that
//! names no group; a stranger on the members' protocol is closed with no
//! answer; nothing the command prints uses a word DESIGN §1 keeps from
//! people. Nodes in one process, no relay: each link dials Sam's loopback
//! address.

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use svrngs::cli::{self, Refusal, Said};
use svrngs::group::{Name, Seed};
use svrngs::link::{parse_https_link, Dial, Link, Origin};
use svrngs_node::iroh::{Contact, EndpointAddr, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::redeem::{redeem, RedeemError, RedeemRequest, Redeemed, REFUSAL};
use svrngs_node::{actor_of, derive_id, Ed25519Verifier, OpId, Person, RailAct, SigningKey};

fn no_relay() -> RelayConfig {
    RelayConfig::from_parts(Vec::new(), Some("none")).unwrap()
}

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

/// One person's computer: its node, its key and its data dir.
struct Computer {
    node: Node,
    key: SigningKey,
    _dir: tempfile::TempDir,
}

async fn computer(seed: u8) -> Computer {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[seed; 32]);
    let node = Node::start(dir.path().to_path_buf(), key.clone(), &no_relay())
        .await
        .expect("a node starts");
    Computer {
        node,
        key,
        _dir: dir,
    }
}

/// iroh binds the wildcard; a dial in one process needs loopback.
fn loopback(node: &Node) -> Vec<SocketAddr> {
    node.endpoint()
        .bound_sockets()
        .into_iter()
        .map(|mut a| {
            if a.ip().is_unspecified() {
                a.set_ip(if a.is_ipv4() {
                    "127.0.0.1".parse().unwrap()
                } else {
                    "::1".parse().unwrap()
                });
            }
            a
        })
        .collect()
}

/// `url` parsed, its dial pointed at Sam's loopback sockets, as the
/// relay's address would be outside one process.
fn link_at(url: &str, sam: &Node) -> Link {
    let mut link = parse_https_link(url).expect("the printed link parses");
    assert_eq!(link.issuer.addr().id, sam.endpoint().id());
    let mut addr = EndpointAddr::new(sam.endpoint().id());
    for a in loopback(sam) {
        addr = addr.with_ip_addr(a);
    }
    link.issuer = Dial::of(&addr).unwrap();
    link
}

/// The ids Sam's record holds, each as its content derives it.
fn held(node: &Node, group: &str) -> BTreeSet<OpId> {
    let (ops, _) = node.rail().journal(group).unwrap().read().unwrap();
    ops.iter().map(|op| derive_id(op, group)).collect()
}

/// The keys Sam's record names in an `Admit`, one entry per `Admit`.
fn admitted(node: &Node, group: &str) -> Vec<String> {
    let (ops, _) = node.rail().journal(group).unwrap().read().unwrap();
    ops.into_iter()
        .filter_map(|op| match op.kind.act {
            RailAct::Admit { key, .. } => Some(key),
            _ => None,
        })
        .collect()
}

fn stands(node: &Node, group: &str, key: &SigningKey) -> bool {
    node.rail()
        .standing(group, &Ed25519Verifier)
        .unwrap()
        .names(&actor_of(key))
}

/// The issuer's answer to `link` presented by `who` on the wire: its
/// refusal's words, or what it did.
async fn present(who: &Computer, link: &Link, person: &str) -> Result<Redeemed, String> {
    let req = RedeemRequest::sign(
        &link.secret,
        &who.key,
        &link.seed.id(),
        &link.invite,
        &Person::from(person),
    );
    match redeem(who.node.endpoint(), link.issuer.addr().clone(), &req).await {
        Ok(r) => Ok(r),
        Err(RedeemError::Refused(words)) => Err(words),
        Err(other) => panic!("an answer, not {other}"),
    }
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[tokio::test(flavor = "multi_thread")]
async fn three_people_open_one_link_and_each_is_let_in_once() {
    let sam = computer(1).await;
    let (seed, url) = cli::new_group(
        &sam.node,
        name("house"),
        name("Sam"),
        &Origin::default(),
        &no_relay(),
    )
    .await
    .expect("Sam makes house");
    let group = seed.id();
    let mut said = vec![Said::Made {
        seed: seed.clone(),
        url: url.clone(),
    }
    .to_string()
    .replace(&url, "<link>")];
    let link = link_at(&url, &sam.node);

    // ── Alex, Bo and Cy open Sam's one link.
    let mut friends = Vec::new();
    for (n, who) in [(2u8, "Alex"), (3, "Bo"), (4, "Cy")] {
        let friend = computer(n).await;
        let started = Instant::now();
        let joined = cli::join_group(&friend.node, &friend.key, &link, &name(who))
            .await
            .unwrap_or_else(|e| panic!("{who} is let in: {e}"));
        let took = started.elapsed();
        eprintln!("join: {who} let in and synced once in {took:?} on loopback");
        assert_eq!(
            joined,
            Said::Joined {
                seed: seed.clone(),
                person: name(who)
            }
        );
        said.push(joined.to_string());
        assert!(
            stands(&sam.node, &group, &friend.key),
            "{who} stands at Sam's"
        );
        assert!(
            held(&sam.node, &group).is_subset(&held(&friend.node, &group)),
            "{who}'s copy holds everything Sam's held"
        );
        assert!(
            stands(&friend.node, &group, &friend.key),
            "{who}'s own copy of the register lets {who} in"
        );
        friends.push(friend);
    }
    let three: Vec<String> = friends.iter().map(|f| actor_of(&f.key)).collect();
    assert_eq!(admitted(&sam.node, &group), three, "one Admit each");

    // ── Alex opens it again: nothing new is written.
    let alex = &friends[0];
    assert_eq!(present(alex, &link, "Alex").await, Ok(Redeemed::AlreadyIn));
    let again = cli::join_group(&alex.node, &alex.key, &link, &name("Alex"))
        .await
        .unwrap();
    assert_eq!(again, Said::AlreadyIn(seed.clone()));
    said.push(again.to_string());
    assert_eq!(admitted(&sam.node, &group), three, "no second Admit");

    // ── Sam sends a new link: Dee, with the old one, is refused.
    let (linked, url2) = cli::new_link(&sam.node, "house", &Origin::default(), &no_relay())
        .await
        .unwrap();
    said.push(
        Said::Linked {
            seed: linked,
            url: url2.clone(),
        }
        .to_string()
        .replace(&url2, "<link>"),
    );
    let dee = computer(5).await;
    let closed = present(&dee, &link, "Dee").await;
    let refused = cli::join_group(&dee.node, &dee.key, &link, &name("Dee")).await;
    assert!(
        matches!(refused, Err(Refusal::LinkRefused(_))),
        "Dee with the old link: {}",
        refused.map_or_else(|e| e.to_string(), |s| s.to_string())
    );
    assert!(!stands(&sam.node, &group, &dee.key));
    for f in &friends {
        assert!(
            stands(&sam.node, &group, &f.key),
            "the old link's three stay"
        );
    }

    // ── A link key Sam never issued, presented with Sam's open Invite.
    let forged = Link {
        secret: SigningKey::from_bytes(&[0xF0; 32]),
        ..link_at(&url2, &sam.node)
    };
    let fay = computer(7).await;
    let never = present(&fay, &forged, "Fay").await;
    assert!(!stands(&sam.node, &group, &fay.key));

    // ── Sam's open link, past its lapse: Eve is refused.
    let lapsed_secret = SigningKey::from_bytes(&[0xE0; 32]);
    let lapsed = sam
        .node
        .rail()
        .journal(&group)
        .unwrap()
        .append(
            RailAct::Invite {
                key: actor_of(&lapsed_secret),
                until: now() - 1,
            },
            sam.node.rail().signer(),
            &seed.roster(),
            None,
            &Ed25519Verifier,
        )
        .unwrap();
    let lapsed_link = Link {
        seed: seed.clone(),
        secret: lapsed_secret,
        invite: lapsed.id,
        issuer: link.issuer.clone(),
    };
    let eve = computer(6).await;
    let past = present(&eve, &lapsed_link, "Eve").await;
    assert!(!stands(&sam.node, &group, &eve.key));

    // ── One refusal, naming nothing.
    for (who, answer) in [("Dee", &closed), ("Eve", &past), ("Fay", &never)] {
        let words = answer
            .as_ref()
            .expect_err(&format!("{who} is refused"))
            .clone();
        assert_eq!(
            words,
            closed.clone().unwrap_err(),
            "{who}: the same refusal"
        );
        assert!(words.contains(REFUSAL), "{words}");
        assert!(
            !words.contains(&group) && !words.contains(&group[..8]) && !words.contains("house"),
            "{who}'s refusal names the group: {words}"
        );
    }
    assert_eq!(admitted(&sam.node, &group), three, "nobody else let in");

    // ── A stranger on the members' protocol: closed, no answer.
    let transport = dee.node.transport();
    let base = transport
        .base_url(&Contact {
            key: sam.node.endpoint().id(),
            relay_url: None,
            iroh_direct_addrs: loopback(&sam.node),
        })
        .await
        .expect("Sam's loopback address resolves");
    let answer = tokio::time::timeout(
        Duration::from_secs(20),
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .post(format!("{base}/internal/ring/sync"))
            .body("{}")
            .send(),
    )
    .await
    .expect("a closed connection is an error, not a hang");
    assert!(
        answer.is_err(),
        "a stranger must get no answer on the members' protocol, got {:?}",
        answer.map(|r| r.status())
    );

    // ── Nothing printed says a word kept from people. The links are left
    // out: their origin, svrnme.sh/ring/, is the operator's (DESIGN §10).
    said.push(Said::Running(vec![name("house")]).to_string());
    said.push(Said::Running(Vec::new()).to_string());
    for r in [
        Refusal::NoNameToJoin(name("house")),
        Refusal::LinkRefused(name("house")),
        Refusal::IssuerAway(name("house")),
    ] {
        said.push(r.to_string());
    }
    for line in &said {
        let lower = line.to_lowercase();
        for word in ["ring", "member", "key", "sync", "journal", "node", "relay"] {
            assert!(!lower.contains(word), "{line:?} says {word}");
        }
    }
    for c in [&sam, &dee, &eve, &fay].into_iter().chain(&friends) {
        c.node.endpoint().close().await;
    }
}

/// `join` parses a link and `--as`; a link cut short is refused before
/// anything is dialled.
#[test]
fn join_parses_its_link() {
    let founder = SigningKey::from_bytes(&[1; 32]);
    let seed = Seed::from_parts(name("house"), name("Sam"), founder.verifying_key(), [9; 16]);
    let addr = EndpointAddr::new(
        svrngs_node::iroh::PublicKey::from_bytes(&founder.verifying_key().to_bytes()).unwrap(),
    )
    .with_ip_addr("127.0.0.1:4242".parse().unwrap());
    let link = Link {
        seed,
        secret: SigningKey::from_bytes(&[7; 32]),
        invite: OpId::from_hex(&"cd".repeat(32)).unwrap(),
        issuer: Dial::of(&addr).unwrap(),
    };
    let url = svrngs::link::build_https_link(&link, &Origin::default());
    let argv = |rest: &[&str]| {
        std::iter::once("svrngs")
            .chain(rest.iter().copied())
            .map(String::from)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        cli::parse(argv(&["join", &url, "--as", "Alex"])).unwrap(),
        cli::Command::Join {
            link: Box::new(link),
            person: Some(name("Alex")),
        }
    );
    let cut = &url[..url.len() - 20];
    assert!(cli::parse(argv(&["join", cut])).is_err());
    assert!(cli::parse(argv(&["join"])).is_err());
    assert!(cli::parse(argv(&["join", &url, "--origin", "https://x.example/"])).is_err());
}
