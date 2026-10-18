//! Join by link, U2's bar (PLAN §4): Sam makes a group and sends one
//! link; Alex, Bo and Cy each open it with `svrngs join` and are let in
//! once, by Sam's computer, and each copy syncs Sam's record. A closed
//! link, a lapsed one, a forged one and an open one's key on a request its
//! secret did not sign are refused in one sentence that names no group; a
//! stranger on the members' protocol is closed with no answer; nothing the
//! command prints, its links included, uses a word DESIGN §1 keeps from
//! people. The link is one short token (u2-8): the joiner asks Sam's
//! computer for the group's seed and the `Invite`'s id, and believes the
//! seed only when it hashes to the group named. Nodes in one process, no
//! relay: each joiner is told Sam's loopback addresses, as lookup or a
//! relay would find them outside one process.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use svrngs::cli::{self, Printed, Refusal, Said, Verb};
use svrngs::group::{self, Name, Seed};
use svrngs_node::iroh::{Contact, EndpointAddr, MemoryLookup, RelayConfig, RelayUrl};
use svrngs_node::join as issuer;
use svrngs_node::link::{client_link, https_link, Link, Origin, Token, KEPT_WORDS};
use svrngs_node::node::Node;
use svrngs_node::redeem::{
    ask, redeem, AskRequest, Asked, RedeemError, RedeemRequest, Redeemed, REFUSAL,
};
use svrngs_node::{
    actor_of, derive_id, redemption_message, sign_message, Device, OpId, Person, RailAct,
    SigningKey, BROWSER_LINE_VERSION, LINK_LINE_VERSION,
};

#[path = "../../svrngs-node/tests/common/mod.rs"]
mod node_common;
use node_common::heard;

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
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
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
    computer_on(seed, &no_relay()).await
}

async fn computer_on(seed: u8, relay: &RelayConfig) -> Computer {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[seed; 32]);
    let node = Node::start(
        dir.path().to_path_buf(),
        key.clone(),
        relay,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts");
    Computer {
        node,
        key,
        _dir: dir,
    }
}

/// Sam's computer makes the group house, as Sam: no page, no `--relay`,
/// and no relay reached ([`no_relay`]).
async fn sams_house(sam: &Computer) -> (Seed, Printed) {
    cli::new_group(
        &sam.node,
        name("house"),
        name("Sam"),
        None,
        None,
        &no_relay(),
    )
    .await
    .expect("Sam makes house")
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

/// Tell `who` the loopback addresses Sam's key resolves to, as address
/// lookup or a relay would outside one process.
fn knows(who: &Computer, sam: &Node) {
    let mut addr = EndpointAddr::new(sam.endpoint().id());
    for a in loopback(sam) {
        addr = addr.with_ip_addr(a);
    }
    let book = MemoryLookup::new();
    book.add_endpoint_info(addr);
    who.node.endpoint().address_lookup().unwrap().add(book);
}

/// A printed client's link, parsed, after checking it dials Sam.
fn parsed(printed: &Printed, sam: &Node) -> Link {
    let link = Link::parse(&printed.client).expect("the printed link parses");
    assert_eq!(issuer::key(&link.token), sam.endpoint().id());
    link
}

/// The id of the `Invite` in Sam's record naming `link`'s key.
fn invite_of(node: &Node, group: &str, link: &Link) -> OpId {
    let key = actor_of(&link.token.key());
    let ops = lines(node, group);
    ops.iter()
        .find(|op| matches!(&op.kind.act, RailAct::Invite { key: k, .. } if *k == key))
        .map(|op| derive_id(op, group))
        .expect("Sam's record names the link's key")
}

/// The ids Sam's record holds, each as its content derives it.
fn held(node: &Node, group: &str) -> BTreeSet<OpId> {
    let ops = lines(node, group);
    ops.iter().map(|op| derive_id(op, group)).collect()
}

/// The keys Sam's record names in an `Admit`, one entry per `Admit`.
fn admitted(node: &Node, group: &str) -> Vec<String> {
    let ops = lines(node, group);
    ops.into_iter()
        .filter_map(|op| match op.kind.act {
            RailAct::Admit { key, .. } => Some(key),
            _ => None,
        })
        .collect()
}

fn stands(node: &Node, group: &str, key: &SigningKey) -> bool {
    node.rail().standing(group).unwrap().names(&actor_of(key))
}

/// The issuer's answer to `link` presented by `who` on the wire, under
/// the `Invite` `invite` of `group`: its refusal's words, or what it did.
async fn present(
    who: &Computer,
    link: &Link,
    group: &str,
    invite: &OpId,
    person: &str,
) -> Result<Redeemed, String> {
    present_from(who, link, group, invite, person, Device::Computer).await
}

/// [`present`], the request naming `device` as what holds `who`'s key.
async fn present_from(
    who: &Computer,
    link: &Link,
    group: &str,
    invite: &OpId,
    person: &str,
    device: Device,
) -> Result<Redeemed, String> {
    let req = RedeemRequest::sign(
        &link.token.key(),
        &who.key,
        group,
        invite,
        &Person::parse(person).unwrap(),
        device,
    );
    send(who, link, &req).await
}

/// The issuer's answer to `req`, sent by `who` to `link`'s issuer.
async fn send(who: &Computer, link: &Link, req: &RedeemRequest) -> Result<Redeemed, String> {
    words(redeem(who.node.endpoint(), issuer::addr(&link.token), req).await)
}

/// The issuer's answer to `who` asking what `link` is for.
async fn asks(who: &Computer, link: &Link) -> Result<Asked, String> {
    let req = AskRequest::sign(&link.token.key(), &who.key);
    words(ask(who.node.endpoint(), issuer::addr(&link.token), &req).await)
}

/// An answer, or a refusal's words; never no answer.
fn words<T>(answer: Result<T, RedeemError>) -> Result<T, String> {
    match answer {
        Ok(r) => Ok(r),
        Err(RedeemError::Refused(words)) => Err(words),
        Err(other) => panic!("an answer, not {other}"),
    }
}

fn argv(rest: &[&str]) -> Vec<String> {
    std::iter::once("svrngs")
        .chain(rest.iter().copied())
        .map(String::from)
        .collect()
}

/// `svrngs join <raw>` parsed: the link the command would redeem.
fn typed(raw: &str) -> Link {
    match cli::parse(argv(&["join", raw])).expect("join parses") {
        Verb::Join { link, .. } => *link,
        other => panic!("{other:?}"),
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
    let (seed, printed) = sams_house(&sam).await;
    let group = seed.id();
    let mut said = vec![Said::Made {
        seed: seed.clone(),
        link: printed.clone(),
        beside: Vec::new(),
    }
    .to_string()];
    let link = parsed(&printed, &sam.node);
    let invite = invite_of(&sam.node, &group, &link);

    // ── Alex, Bo and Cy open Sam's one link: Alex as the client's link,
    // Bo on a page of the group's, Cy as the bare token.
    let token = link.token.encode();
    let page = Origin::parse("https://x.example/").unwrap();
    let forms = [
        (2u8, "Alex", printed.client.clone()),
        (3, "Bo", https_link(&page, &link.token)),
        (4, "Cy", token.clone()),
    ];
    let mut friends = Vec::new();
    for (n, who, raw) in forms {
        let friend = computer(n).await;
        knows(&friend, &sam.node);
        let opened = typed(&raw);
        assert_eq!(
            opened.token, link.token,
            "{who}'s form carries the one token"
        );
        let started = Instant::now();
        let joined = cli::join_group(&friend.node, &friend.key, &opened, &name(who))
            .await
            .unwrap_or_else(|e| panic!("{who} is let in: {e}"));
        let took = started.elapsed();
        eprintln!("join: {who} let in and synced once in {took:?} on loopback, from {raw}");
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
        let dir = friend.node.rail().group_dir(&group).unwrap();
        let kept = svrngs_node::invite::kept_page(&dir).unwrap();
        assert_eq!(
            kept, opened.origin,
            "{who} keeps the origin of an https link, and none from another"
        );
        friends.push(friend);
    }
    let three: Vec<String> = friends.iter().map(|f| actor_of(&f.key)).collect();
    assert_eq!(admitted(&sam.node, &group), three, "one Admit each");

    // ── Alex opens it again: nothing new is written.
    let alex = &friends[0];
    assert_eq!(
        present(alex, &link, &group, &invite, "Alex").await,
        Ok(Redeemed::AlreadyIn)
    );
    let again = cli::join_group(&alex.node, &alex.key, &link, &name("Alex"))
        .await
        .unwrap();
    assert_eq!(again, Said::AlreadyIn(seed.clone()));
    said.push(again.to_string());
    assert_eq!(admitted(&sam.node, &group), three, "no second Admit");

    // ── Sam sends a new link: Dee, with the old one, is refused.
    let (linked, printed2, _) = cli::link_to(&sam.node, "house", None, &no_relay(), false)
        .await
        .unwrap();
    said.push(
        Said::Linked {
            seed: linked,
            link: printed2.clone(),
            device: None,
        }
        .to_string(),
    );
    let dee = computer(5).await;
    knows(&dee, &sam.node);
    let closed = present(&dee, &link, &group, &invite, "Dee").await;
    let refused = cli::join_group(&dee.node, &dee.key, &link, &name("Dee")).await;
    assert!(
        matches!(refused, Err(Refusal::LinkRefused)),
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

    // ── A link Sam never issued: asked about, and presented with Sam's
    // open Invite, Fay is refused.
    let open = parsed(&printed2, &sam.node);
    let open_invite = invite_of(&sam.node, &group, &open);
    let forged = Link {
        token: Token {
            secret: [0xF0; 16],
            ..open.token.clone()
        },
        origin: None,
    };
    let fay = computer(7).await;
    knows(&fay, &sam.node);
    let unasked = asks(&fay, &forged).await.map(|_| ());
    let never = present(&fay, &forged, &group, &open_invite, "Fay").await;
    assert!(matches!(
        cli::join_group(&fay.node, &fay.key, &forged, &name("Fay")).await,
        Err(Refusal::LinkRefused)
    ));
    assert!(!stands(&sam.node, &group, &fay.key));

    // ── Sam's open link key, which every copy of the record shows in its
    // `Invite`, on a request another key signed: Gus is refused.
    let gus = computer(8).await;
    knows(&gus, &sam.node);
    let mut req = RedeemRequest::sign(
        &forged.token.key(),
        &gus.key,
        &group,
        &open_invite,
        &Person::parse("Gus").unwrap(),
        Device::Computer,
    );
    req.link = actor_of(&open.token.key());
    let unsigned = send(&gus, &open, &req).await;
    let mut asked = AskRequest::sign(&forged.token.key(), &gus.key);
    asked.link = actor_of(&open.token.key());
    let unsigned_ask =
        words(ask(gus.node.endpoint(), issuer::addr(&open.token), &asked).await).map(|_| ());
    assert!(!stands(&sam.node, &group, &gus.key));

    // ── Sam's link, past its lapse: Eve is refused.
    let lapsed_link = Link {
        token: Token {
            secret: [0xE0; 16],
            ..open.token.clone()
        },
        origin: None,
    };
    let mint = svrngs_node::MintInvite {
        key: actor_of(&lapsed_link.token.key()),
        until: now() - 1,
        device_of: None,
    };
    let lapsed = sam
        .node
        .rail()
        .commit(&group, &mint)
        .unwrap()
        .unwrap()
        .lines()[0]
        .clone();
    let eve = computer(6).await;
    knows(&eve, &sam.node);
    let past = present(&eve, &lapsed_link, &group, &lapsed.id, "Eve").await;
    assert!(matches!(
        cli::join_group(&eve.node, &eve.key, &lapsed_link, &name("Eve")).await,
        Err(Refusal::LinkRefused)
    ));
    assert!(!stands(&sam.node, &group, &eve.key));

    // ── One refusal, naming nothing.
    let unit = |a: &Result<Redeemed, String>| a.clone().map(|_| ());
    for (who, answer) in [
        ("Dee", unit(&closed)),
        ("Eve", unit(&past)),
        ("Fay", unit(&never)),
        ("Fay, asking", unasked),
        ("Gus", unit(&unsigned)),
        ("Gus, asking", unsigned_ask),
    ] {
        let words = answer.expect_err(&format!("{who} is refused"));
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
        .spliced(&Contact {
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

    // ── Nothing printed says a word kept from people, the links included.
    let api = sam.node.api_addr();
    said.push(
        Said::Running {
            groups: vec![name("house")],
            api,
            lends: Default::default(),
        }
        .to_string(),
    );
    said.push(
        Said::Running {
            groups: Vec::new(),
            api,
            lends: Default::default(),
        }
        .to_string(),
    );
    for r in [
        Refusal::NoNameToJoin,
        Refusal::LinkRefused,
        Refusal::IssuerAway,
    ] {
        said.push(r.to_string());
    }
    for line in &said {
        let lower = line.to_lowercase();
        for word in KEPT_WORDS {
            assert!(!lower.contains(word), "{line:?} says {word}");
        }
    }
    for c in [&sam, &dee, &eve, &fay, &gus].into_iter().chain(&friends) {
        c.node.endpoint().close().await;
    }
}

/// **A person's name is parsed at Sam's computer too** (u2-9;
/// REVIEW_FINDINGS #23). A redemption naming an empty person, 10 KB or a
/// terminal's escape codes, its request signed by the link's key and its
/// consent by the joiner's, as a hand-built client could sign them, is
/// refused with why the name is not one, naming no group, and Sam's record
/// is byte for byte what it was. Zoë Ann, a name with a space and an
/// accent, is let in, and Sam's record names her so.
#[tokio::test(flavor = "multi_thread")]
async fn a_name_that_is_not_one_is_refused_and_nothing_is_written() {
    let sam = computer(1).await;
    let (seed, printed) = sams_house(&sam).await;
    let group = seed.id();
    let link = parsed(&printed, &sam.node);
    let invite = invite_of(&sam.node, &group, &link);
    let file = sam
        .node
        .rail()
        .group_dir(&group)
        .unwrap()
        .join("ring_oplog.jsonl");
    let hal = computer(9).await;
    knows(&hal, &sam.node);
    for (bad, why) in [
        (String::new(), "cannot be empty"),
        ("x".repeat(10 * 1024), "at most 64 bytes"),
        ("\x1b[2J".to_string(), "control characters"),
    ] {
        let before = std::fs::read_to_string(&file).unwrap();
        let mut req = RedeemRequest::sign(
            &link.token.key(),
            &hal.key,
            &group,
            &invite,
            &Person::parse("Hal").unwrap(),
            Device::Computer,
        );
        req.person = bad.clone();
        req.cosig = sign_message(
            &hal.key,
            &redemption_message(&group, &invite, &bad, &actor_of(&hal.key)),
        );
        req.sig = sign_message(&link.token.key(), &req.message());
        let answer = send(&hal, &link, &req).await;
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            before,
            "{bad:?}: Sam's record is unchanged, answered {answer:?}"
        );
        let words = answer.expect_err(&format!("{bad:?} is refused"));
        assert!(words.contains(why), "{bad:?}: {words}");
        assert!(
            !words.contains(&group[..8]) && !words.contains("house"),
            "{bad:?}'s refusal names the group: {words}"
        );
        assert!(!stands(&sam.node, &group, &hal.key));
    }

    let zoe = computer(10).await;
    knows(&zoe, &sam.node);
    cli::join_group(&zoe.node, &zoe.key, &link, &name("Zoë Ann"))
        .await
        .expect("Zoë Ann is let in");
    assert!(stands(&sam.node, &group, &zoe.key));
    let ops = lines(&sam.node, &group);
    assert!(ops.iter().any(|op| matches!(&op.kind.act,
        RailAct::Admit { person, key, .. }
            if person.as_str() == "Zoë Ann" && *key == actor_of(&zoe.key))));
    for c in [&sam, &hal, &zoe] {
        c.node.endpoint().close().await;
    }
}

/// **A name binds once where a stranger gives one** (bash-person; the
/// charter's "The name"). Sam founds house; a second computer joining Sam's
/// link as `sam` is refused, told who goes by it and how to join under
/// another name, exit 1, and Sam's record is byte for byte what it was; the
/// same computer joining as `Hal` is let in.
#[tokio::test(flavor = "multi_thread")]
async fn a_name_someone_in_the_group_goes_by_is_refused_with_how_to_join() {
    let sam = computer(1).await;
    let (seed, printed) = sams_house(&sam).await;
    let group = seed.id();
    let link = parsed(&printed, &sam.node);
    let file = sam
        .node
        .rail()
        .group_dir(&group)
        .unwrap()
        .join("ring_oplog.jsonl");
    let before = std::fs::read_to_string(&file).unwrap();
    let hal = computer(9).await;
    knows(&hal, &sam.node);
    let refused = cli::join_group(&hal.node, &hal.key, &link, &name("sam"))
        .await
        .expect_err("a second sam is refused");
    let words = refused.to_string();
    assert!(
        matches!(&refused, Refusal::NameTaken { holder, .. } if *holder == name("Sam")),
        "{refused:?}"
    );
    // How to add this computer as Sam, if it is Sam's (bash-devices (d)).
    assert!(
        words.contains("already goes by Sam")
            && words.contains("`svrngs link house --device`")
            && words.contains("--as <your name>"),
        "{words}"
    );
    assert_eq!(refused.exit_code(), 1);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
    assert!(!stands(&sam.node, &group, &hal.key));
    cli::join_group(&hal.node, &hal.key, &link, &name("Hal"))
        .await
        .expect("Hal is let in");
    assert!(stands(&sam.node, &group, &hal.key));
    for c in [&sam, &hal] {
        c.node.endpoint().close().await;
    }
}

/// **Another computer of Sam's joins as Sam, by a link Sam's computer makes
/// for Sam's devices** (bash-devices (d); `svrngs link house --device`):
/// asked no name — the name the command would give, `whoever`, is not the
/// one it joins as — it is let in as Sam, prints so, and stands in Sam's
/// register as Sam's own person.
#[tokio::test]
async fn another_computer_of_sams_joins_as_sam_by_a_link_for_sams_devices() {
    let sam = computer(1).await;
    let (seed, _) = sams_house(&sam).await;
    let (_, printed, device) = cli::link_to(&sam.node, "house", None, &no_relay(), true)
        .await
        .expect("Sam's computer makes a link for Sam's devices");
    assert_eq!(device.map(|p| p.to_string()), Some("Sam".to_string()));
    let link = parsed(&printed, &sam.node);
    let second = computer(4).await;
    knows(&second, &sam.node);
    let joined = cli::join_group(&second.node, &second.key, &link, &name("whoever"))
        .await
        .expect("Sam's second computer is let in");
    assert_eq!(joined.to_string(), "You're in house, as Sam.\n");
    let group = seed.id();
    let standing = sam.node.rail().standing(&group).unwrap();
    let svrngs_node::register::Standing::Members(members) = standing else {
        panic!("house is held at Sam's")
    };
    let m = &members[&actor_of(&second.key)];
    assert_eq!(
        (m.person.as_str(), m.person_key.as_str()),
        ("Sam", actor_of(&sam.key).as_str()),
        "Sam's second computer is Sam, by Sam's first key"
    );
    for c in [&sam, &second] {
        c.node.endpoint().close().await;
    }
}

/// The redeem route writes what the request names (u4-1; DESIGN §3
/// "Joining", call 2): Pia's request names a browser, and Sam's record holds
/// one `Admit` of Pia's key naming a browser, at line version 4, and Sam's
/// register answers that the key stands as a browser's; Quin's names a
/// computer, and the `Admit` of Quin's key names a computer at version 3.
/// Watched failing with the route writing a computer whatever the request
/// named.
#[tokio::test(flavor = "multi_thread")]
async fn the_redeem_route_writes_the_device_the_request_names() {
    let sam = computer(1).await;
    let (seed, printed) = sams_house(&sam).await;
    let group = seed.id();
    let link = parsed(&printed, &sam.node);
    let invite = invite_of(&sam.node, &group, &link);
    for (n, who, device, version) in [
        (11u8, "Pia", Device::Browser, BROWSER_LINE_VERSION),
        (12, "Quin", Device::Computer, LINK_LINE_VERSION),
    ] {
        let joiner = computer(n).await;
        knows(&joiner, &sam.node);
        let answer = present_from(&joiner, &link, &group, &invite, who, device).await;
        assert!(
            matches!(answer, Ok(Redeemed::Admitted(_))),
            "{who}: {answer:?}"
        );
        let admits: Vec<_> = lines(&sam.node, &group)
            .into_iter()
            .filter(|op| matches!(&op.kind.act, RailAct::Admit { key, .. } if *key == actor_of(&joiner.key)))
            .collect();
        assert_eq!(admits.len(), 1, "{who}: one Admit");
        let RailAct::Admit {
            invite: Some(redeemed),
            ..
        } = &admits[0].kind.act
        else {
            panic!("{who}: an Admit redeeming the link, not {:?}", admits[0])
        };
        assert_eq!(
            (redeemed.device, admits[0].v),
            (device, version),
            "{who}: the Admit names what the request named"
        );
        let register = sam
            .node
            .rail()
            .admission(&group)
            .unwrap()
            .membership()
            .clone();
        assert_eq!(
            register.device(&actor_of(&joiner.key)),
            Some(device),
            "{who}"
        );
        joiner.node.endpoint().close().await;
    }
    sam.node.endpoint().close().await;
}

/// How soon a newcomer's computer lists what the group's other computers
/// lend once let in (bash-first-minute (b)). The split
/// (`demo/runs/first-minute/2026-10-12/README.md`): the join took 0.7 s,
/// listing took under 0.1 s once Ana's computer held the `Admit`, and the
/// rest, 56 s, was waiting for Sam's round at its 60 s period. A round's
/// exchange on loopback took 14 ms there; five seconds holds one ask cut at
/// `ANSWER_WITHIN`, 3 s, and a loaded host, and is far inside the 51-56 s
/// Demo C's dry run measured.
const HEARD_WITHIN: Duration = Duration::from_secs(5);

/// A newcomer is heard by the group at once (bash-first-minute (b); Demo
/// C's dry run, finding 8): Ana is in Sam's group and lends one thing;
/// Alex is let in by Sam's computer, and Alex's computer lists Ana's lend,
/// asked of Ana's computer, within [`HEARD_WITHIN`] — every round's period
/// left at its 60 s. Watched failing with the redeem route waking no round.
#[tokio::test(flavor = "multi_thread")]
async fn a_newcomer_lists_what_every_computer_lends_within_seconds() {
    let sam = computer(1).await;
    let (_, printed) = sams_house(&sam).await;
    let link = parsed(&printed, &sam.node);
    let ana = computer(2).await;
    knows(&ana, &sam.node);
    cli::join_group(&ana.node, &ana.key, &link, &name("ana"))
        .await
        .expect("Ana is let in");
    let lends = svrngs_node::lend::lends_dir(ana._dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let music = serde_json::json!({ "port": 4533, "kind": "navidrome", "limit": 3 });
    std::fs::write(lends.join("music.json"), music.to_string()).unwrap();
    knows(&sam, &ana.node);
    let alex = computer(3).await;
    knows(&alex, &sam.node);
    knows(&alex, &ana.node);
    cli::join_group(&alex.node, &alex.key, &link, &name("alex"))
        .await
        .expect("Alex is let in");
    let started = Instant::now();
    loop {
        let lends = alex.node.lends().await;
        if lends.lent.iter().any(|l| l.person.as_str() == "ana") {
            eprintln!(
                "Alex lists Ana's lend {:?} after being let in",
                started.elapsed()
            );
            break;
        }
        assert!(
            started.elapsed() < HEARD_WITHIN,
            "Alex's computer lists no lend of Ana's {:?} after being let in: {lends:?}",
            started.elapsed()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    for c in [&sam, &ana, &alex] {
        c.node.endpoint().close().await;
    }
}

/// A member's later link opens the page the founder's did (u2-7): Sam
/// makes house on x.example; Alex, let in by its https link, keeps that
/// origin, and the link Alex prints with no `--origin` is on x.example
/// too. Cy, let in by the client's link to the same group, keeps none, and
/// prints the client's link alone.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_prints_links_on_the_page_the_founders_opened() {
    let sam = computer(1).await;
    let x = Origin::parse("https://x.example/").unwrap();
    let (_, printed) = cli::new_group(
        &sam.node,
        name("house"),
        name("Sam"),
        Some(&x),
        None,
        &no_relay(),
    )
    .await
    .expect("Sam makes house on x.example");
    let https = printed.https.clone().expect("an https link on x.example");
    let alex = computer(2).await;
    let cy = computer(4).await;
    for (who, n, raw) in [(&alex, "Alex", &https), (&cy, "Cy", &printed.client)] {
        knows(who, &sam.node);
        cli::join_group(&who.node, &who.key, &typed(raw), &name(n))
            .await
            .unwrap_or_else(|e| panic!("{n} is let in: {e}"));
    }
    let (_, theirs, _) = cli::link_to(&alex.node, "house", None, &no_relay(), false)
        .await
        .expect("Alex, in house, prints a link");
    let on = theirs.https.unwrap_or_default();
    assert!(on.starts_with("https://x.example/#"), "{on}");
    let (_, cys, _) = cli::link_to(&cy.node, "house", None, &no_relay(), false)
        .await
        .expect("Cy, in house, prints a link");
    assert_eq!(cys.https, None, "{cys}");
    for c in [&sam, &alex, &cy] {
        c.node.endpoint().close().await;
    }
}

/// A computer run without address lookup puts its relay in its link,
/// without the root's dot the operator wrote (u4-9), and a joiner with
/// lookup off is let in by it. The relay named is not run in
/// one process, so the joiner is told Sam's loopback addresses beside it.
#[tokio::test(flavor = "multi_thread")]
async fn a_link_from_a_computer_without_lookup_carries_its_relay() {
    let relay: RelayUrl = "https://relay.example./".parse().unwrap();
    let ours = RelayConfig {
        relay_urls: vec![relay.to_string()],
        n0_services: false,
        loopback_only: false,
    };
    assert!(!ours.n0_services, "lookup off");
    let sam = computer_on(1, &ours).await;
    let (seed, printed) = cli::new_group(&sam.node, name("house"), name("Sam"), None, None, &ours)
        .await
        .expect("Sam makes house");
    eprintln!(
        "a relayed client's link, {} characters: {}",
        printed.client.len(),
        printed.client
    );
    let link = parsed(&printed, &sam.node);
    assert_eq!(
        link.token.relay,
        Some("https://relay.example/".parse().unwrap()),
        "the link carries the relay, dotless"
    );
    let alex = computer(2).await;
    knows(&alex, &sam.node);
    let joined = cli::join_group(&alex.node, &alex.key, &link, &name("Alex"))
        .await
        .expect("Alex is let in, with lookup off");
    assert_eq!(
        joined,
        Said::Joined {
            seed: seed.clone(),
            person: name("Alex")
        }
    );
    assert!(stands(&sam.node, &seed.id(), &alex.key));
    for c in [&sam, &alex] {
        c.node.endpoint().close().await;
    }
}

/// The issuer's answer is believed only when its seed hashes to the group
/// it names: an issuer answering another group's seed is not believed.
#[test]
fn an_issuer_answering_another_groups_seed_is_not_believed() {
    let dir = tempfile::tempdir().unwrap();
    let founder = SigningKey::from_bytes(&[1; 32]).verifying_key();
    let house = Seed::from_parts(name("house"), name("Sam"), founder, [9; 16]);
    let work = Seed::from_parts(name("work"), name("Sam"), founder, [9; 16]);
    group::write_seed(dir.path(), &house).unwrap();
    let text = group::seed_text(dir.path()).unwrap().expect("a seed kept");
    assert_eq!(group::answered(&house.id(), &text), Some(house.clone()));
    assert_eq!(group::answered(&work.id(), &text), None);
    assert_eq!(group::answered(&house.id(), "{}"), None);
}

/// `join` parses a link in each of its forms and `--as`; a link cut short
/// is refused before anything is dialled.
#[test]
fn join_parses_its_link() {
    let founder = SigningKey::from_bytes(&[1; 32]);
    let token = Token {
        issuer: founder.verifying_key(),
        secret: [7; 16],
        relay: None,
    };
    let page = Origin::parse("https://x.example/").unwrap();
    for (raw, origin) in [
        (client_link(&token), None),
        (https_link(&page, &token), Some(page.clone())),
        (token.encode(), None),
    ] {
        assert_eq!(
            cli::parse(argv(&["join", &raw, "--as", "Alex"])).unwrap(),
            Verb::Join {
                link: Box::new(Link {
                    token: token.clone(),
                    origin,
                }),
                person: Some(name("Alex")),
                run: true,
            },
            "{raw}"
        );
        let cut = &raw[..raw.len() - 4];
        assert!(cli::parse(argv(&["join", cut])).is_err(), "{cut}");
    }
    let url = client_link(&token);
    assert!(matches!(
        cli::parse(argv(&["join", &url, "--no-run"])),
        Ok(Verb::Join { run: false, .. })
    ));
    assert!(cli::parse(argv(&["new", "x", "--no-run"])).is_err());
    assert!(cli::parse(argv(&["join"])).is_err());
    assert!(cli::parse(argv(&["join", &url, "--origin", "https://x.example/"])).is_err());
}

/// **A computer the group cut is told so by the issuer, never that its
/// link was refused** (groups-leave-others (d), #181). Hal joins house and
/// Sam's computer cuts him. Hal's key, on a computer whose copy of house
/// cannot be read (a roster that is no JSON), opens Sam's next link: the
/// join traces that whether it was cut cannot be read here, Sam's computer
/// traces its refusal as the cut, and Hal is told he is no longer in house
/// on this computer, never that the link was refused. Red at f53e06f6: the
/// link refused, and neither trace.
#[tokio::test]
async fn a_computer_the_group_cut_is_told_so_by_the_issuer() {
    let (said, _heard) = heard("join=debug,redeem=debug");
    let sam = computer(1).await;
    let (seed, printed) = sams_house(&sam).await;
    let group = seed.id();
    let hal = computer(9).await;
    knows(&hal, &sam.node);
    let link = parsed(&printed, &sam.node);
    cli::join_group(&hal.node, &hal.key, &link, &name("Hal"))
        .await
        .expect("Hal is let in");
    let cut = svrngs_node::Removal {
        person: Person::parse("Hal").unwrap(),
        joined: None,
    };
    let cut = sam.node.rail().commit(&group, &cut).unwrap();
    assert!(cut.is_ok(), "control: Sam's computer cuts Hal: {cut:?}");
    hal.node.endpoint().close().await;

    let again = computer(9).await;
    knows(&again, &sam.node);
    let dir = again.node.rail().group_dir(&group).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("roster.json"), "no roster").unwrap();
    let (_, printed, _) = cli::link_to(&sam.node, "house", None, &no_relay(), false)
        .await
        .expect("Sam's computer makes its next link");
    let link = parsed(&printed, &sam.node);
    let refused = cli::join_group(&again.node, &again.key, &link, &name("Hal")).await;
    assert!(
        matches!(&refused, Err(Refusal::Left(g)) if *g == name("house")),
        "{refused:?}"
    );
    let said = String::from_utf8(said.0.lock().unwrap().clone()).unwrap();
    for trace in [
        "join: whether this computer was cut in the group cannot be read here",
        "redeem: refused — the group cut this key",
    ] {
        assert!(said.contains(trace), "{trace}: {said}");
    }
    for c in [&sam, &again] {
        c.node.endpoint().close().await;
    }
}
