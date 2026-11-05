//! Joining from a phone (DESIGN §3, "Joining"; PLAN §4, U4; u4-5): Sam makes a
//! group whose links open a page on an origin, `svrngs new --origin`, and
//! lends a service. A phone opens the https link printed, in Playwright's
//! headless Chromium and in its WebKit as an iPhone (u4-7): the page asks the person's name, makes their
//! key, redeems the link at Sam's computer naming itself a browser, keeps
//! the key sealed, and lists what the group's people lend as the bare
//! command prints it. The person types `sam` first, a name Sam goes by:
//! the page says so and asks again (bash-person). Opening the link again, or the page alone, is in
//! without asking; a lend opens at an origin of its own, `<label>.<the
//! page's host>`, whose worker dials its lender's key with a pass (u5-2).
//! The page's origin is `localhost`, whose every name this computer
//! answers, as svrnme.sh's are, so `svrngs new` decides that itself, and
//! nothing more is typed: the bar u5-2 keeps. The phone opens the link
//! printed at the same server's TLS name, the tests' page host
//! (web/tests/page.mjs `PAGE_HOST`), where the group's signer works. Everything runs on this
//! computer: a relay on 127.0.0.1, plain HTTP, Sam's computer reached only
//! through it, and the page served on 127.0.0.1. The browser
//! half is `web/tests/join.mjs`, which fails naming the install command
//! when Playwright or its browsers are absent — it never skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
mod common;

use std::num::NonZeroU16;
use std::process::Stdio;

use serde_json::{json, Value};
use svrngs::cli;
use svrngs::group::Name;
use svrngs_node::lend::{self, Kind, Lend, LendName};
use svrngs_node::link::Origin;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Device, RailAct, SigningKey};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

/// Sam's shelf as the phone's page offers it, its open and the line beside
/// it: a port lend writes as the person opening it, said beside its open
/// (svrngs-node home/question.js `named`, `agreed`).
const SHELF: &str = "Sam’s shelf It writes what you say there into house’s record, as you.";

/// A service on 127.0.0.1 answering every request `200 hi`.
async fn service() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut head = [0u8; 4096];
                let _ = sock.read(&mut head).await;
                let _ = sock
                    .write_all(
                        b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\nhi",
                    )
                    .await;
            });
        }
    });
    port
}

/// Each `Admit` in Sam's record: its person, key and device, if it
/// redeems a link.
fn admits(sam: &Node, group: &str) -> Vec<(String, String, Option<Device>)> {
    let file = sam
        .rail()
        .group_dir(group)
        .unwrap()
        .join("ring_oplog.jsonl");
    std::fs::read_to_string(file)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<svrngs_node::Op<svrngs_node::SignedOp>>(l).unwrap())
        .filter_map(|op| match op.kind.act {
            RailAct::Admit {
                person,
                key,
                invite,
            } => Some((person.as_str().to_string(), key, invite.map(|i| i.device))),
            _ => None,
        })
        .collect()
}

/// Whether the page says the phone is in Sam's house as `person`: its
/// heading that it is in, and above the group's people who made the group
/// and since when this phone has held it, as it kept them when it joined
/// (v1ui-door (d)), the day in the browser's own words. Until the fifth
/// xenia set the heading said both (demo/xenia/findings/
/// wordy-the-group-s-page.md).
fn in_house(seen: &Value, person: &str) -> bool {
    let whose = seen["whose"].as_str().unwrap_or_default();
    seen["status"] == format!("You’re in house, as {person}")
        && whose.starts_with("house (Sam’s, since ")
        && whose.ends_with(')')
}

in_webkit!(
    /// Planted: the key stored unsealed beside its seal (store.js `keep`
    /// writing `raw: key`); a second `Admit` on reopening (app.js joining
    /// anew, with a fresh key, on every opening of a link); a taken name
    /// said as the link refused (app.js's 409 not asked again).
    a_phone_joins_by_the_link_sam_printed_and_is_in_again_without_asking
);
async fn a_phone_joins_by_the_link_sam_printed_and_is_in_again_without_asking(
    engine: harness::Engine,
) {
    let site = harness::site();
    let (_relay, ours, dir, sam) = common::sams_computer(1).await;
    let shelf = Lend {
        name: LendName::parse("shelf").unwrap(),
        source: svrngs_node::lend::Source::Port(NonZeroU16::new(service().await).unwrap()),
        kind: Kind::parse("shelf").unwrap(),
        limit: NonZeroU16::new(3),
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        sign_in_with_svrngs: None,
        asks: None,
    };
    lend::declare(dir.path(), &shelf, None).unwrap();

    let mut phone = tokio::process::Command::new("node")
        .arg(harness::repo().join("web/tests/join.mjs"))
        .arg(json!({ "engine": engine.as_str(), "site": site }).to_string())
        .env("SVRNGS_BROWSERS", harness::browsers())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs: the browser tests need Node 18 or later");
    let mut out = BufReader::new(phone.stdout.take().unwrap()).lines();
    let first = out.next_line().await.unwrap().unwrap_or_else(|| {
        panic!(
            "the browser half printed no origin; it exited {:?}",
            phone.try_wait()
        )
    });
    let origin: Value = serde_json::from_str(&first).unwrap();
    let origin = Origin::parse(origin["origin"].as_str().unwrap()).unwrap();
    let port = origin
        .as_str()
        .trim_end_matches('/')
        .rsplit(':')
        .next()
        .unwrap()
        .to_string();

    let page = cli::checked_page(origin.clone())
        .await
        .expect("the page's server serves it at every name under localhost, so no other step");
    let (seed, printed) = cli::new_group(
        &sam,
        Name::parse("house").unwrap(),
        Name::parse("Sam").unwrap(),
        Some(&page),
        &[],
        &ours,
    )
    .await
    .expect("Sam makes house, its links on the page's origin");
    let link = printed.https.clone().expect("an https link on the origin");
    eprintln!("the link printed: {link}");
    let mut stdin = phone.stdin.take().unwrap();
    stdin
        .write_all(
            format!(
                "{}\n",
                json!({ "link": link, "name": "Pia", "taken": "sam", "hint": true })
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let mut stderr = String::new();
    let mut err = phone.stderr.take().unwrap();
    let mut last = None;
    while let Some(line) = out.next_line().await.unwrap() {
        last = Some(line);
    }
    err.read_to_string(&mut stderr).await.unwrap();
    let status = phone.wait().await.unwrap();
    assert!(
        status.success(),
        "the browser half failed ({status}):\n{stderr}"
    );
    let seen: Value = serde_json::from_str(&last.expect("the browser half printed")).unwrap();

    // In: asked the name once, and listed Sam's shelf as the command prints it.
    let first = &seen["first"];
    assert_eq!(
        first["asked"], true,
        "the page asks a newcomer's name:\n{seen:#}"
    );
    assert_eq!(
        first["why"], "Someone in house already goes by Sam. If that’s you, open your home page on your computer, press “Add your phone or another computer”, and open that link here. Else join under another name.",
        "a name someone goes by is refused, and another asked (bash-person):\n{seen:#}"
    );
    assert!(in_house(first, "Pia"), "{seen:#}");
    assert_eq!(first["lent"], json!([SHELF]), "{seen:#}");
    // No row says to keep a lend on a phone: a Home Screen web app keeps
    // a storage of its own, so one added after joining joins again, as
    // someone new (the operator's iPhone, iOS 18.7, 2026-10-30).
    assert_eq!(first["kept"], json!([]), "{seen:#}");
    // So before the name is asked, in iPhone Safari alone, whose
    // `navigator.standalone` is false, the page says to join from the Home
    // Screen; in a Home Screen web app (true) and any other browser
    // (undefined) nothing. Red on c31c666b, which said nothing.
    assert_eq!(
        seen["hint"],
        json!({
            "false": "To keep house on this phone, add this page to your Home Screen first, and join from there.",
            "true": null,
            "undefined": null,
        }),
        "{seen:#}"
    );
    // Joined, the page asked the browser to keep its storage, or says why
    // it did not: a Home Screen web app on the operator's iPhone forgot its
    // person within some ten hours, never asked (iOS 18.7, 2026-10-31;
    // store.js `persist`). Red on d8821e6a, whose page never asked.
    let storage = |o: &Value| {
        (o["lines"].as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .any(|l| l.starts_with("PAGE: ") && l.contains("the page's storage"))
    };
    assert!(storage(first), "{seen:#}");
    // Just joined, the page leads with the group's things, before its
    // people; the shelf is folded as "Bring more to house", and "What acts
    // as you" under its heading (the operator, 2026-10-26;
    // demo/xenia/findings/after-join-reads-as-permissions.md).
    assert_eq!(first["leads"], true, "{seen:#}");
    assert_eq!(
        first["folded"],
        json!(["Bring more to house", "What acts as you"]),
        "{seen:#}"
    );
    // One look, in Ana's twenty words (demo/xenia/people/ana.md) and in
    // xenia's order, things to tap first, takes in the open and the line
    // beside it saying what it does: the fifth set's Ana read 60-70 words
    // before her first tap there, 6 of 6 accounts (demo/xenia/findings/
    // wordy-the-group-s-page.md). Planted, watched failing: the page as
    // 5c6e3b0d left it, its heading naming the maker (`in_house` above).
    assert_eq!(
        first["look"],
        "Sam’s shelf Bring more to house What acts as you You’re in house, as Pia It writes what you say",
        "{seen:#}"
    );

    // The issuer's journal: one Admit naming a browser's key and that name,
    // and the key the one the page keeps sealed.
    let stored = &seen["stored"];
    let key: [u8; 32] = hex::decode(stored["key"].as_str().unwrap())
        .unwrap()
        .try_into()
        .expect("the page's key is 32 bytes");
    let pia = actor_of(&SigningKey::from_bytes(&key));
    let id = seed.id();
    let founder = actor_of(&SigningKey::from_bytes(&[1; 32]));
    let joined: Vec<_> = admits(&sam, &id)
        .into_iter()
        .filter(|(_, key, _)| *key != founder)
        .collect();
    assert_eq!(
        joined,
        vec![("Pia".to_string(), pia.clone(), Some(Device::Browser))],
        "one Admit, naming a browser's key and the name typed:\n{seen:#}"
    );

    // The key's 32 bytes, unsealed, in no storage; its seal not extractable.
    assert_eq!(
        stored["found"],
        json!([]),
        "the key appears unsealed in storage:\n{stored:#}"
    );
    assert_eq!(stored["extractable"], json!([false]), "{stored:#}");

    // Again, by the link and by the page alone: in, asking nothing.
    for again in ["again", "alone"] {
        let o = &seen[again];
        assert_eq!(o["asked"], false, "{again}: asked again:\n{seen:#}");
        assert!(in_house(o, "Pia"), "{again}:\n{seen:#}");
        assert_eq!(o["lent"], json!([SHELF]), "{again}:\n{seen:#}");
        // A group opened asks again, unless kept already.
        assert!(storage(o), "{again}:\n{seen:#}");
        // In already, the shelf is open as it always was.
        assert_eq!(
            o["folded"],
            json!(["What acts as you"]),
            "{again}:\n{seen:#}"
        );
    }

    // A lend opens at an origin of its own, `<label>.<page host>`, whose
    // worker serves it from Sam's computer, dialled by Sam's key (u5-2).
    let shelf = svrngs_core::pass::Label::of(
        SigningKey::from_bytes(&[1; 32]).verifying_key().as_bytes(),
        "shelf",
    );
    assert_eq!(
        seen["opened"],
        json!({ "origin": format!("https://{shelf}.{}:{port}", harness::PAGE_HOST), "text": "hi" }),
        "{seen:#}"
    );
    sam.endpoint().close().await;
}

in_webkit!(
    /// **A phone opening a link Sam's computer made for Sam's devices is
    /// added as Sam, asked no name** (bash-devices (d); `svrngs link house
    /// --device`). It reads "Adding this phone to house as Sam’s…", then
    /// that it is in as Sam; Sam's record holds one `Admit` of a browser's
    /// key, and Sam's register names that key Sam, by Sam's first key. Red
    /// before the page read the link's person: it asked the name.
    a_phone_opening_sams_device_link_is_added_as_sam_asked_no_name
);
async fn a_phone_opening_sams_device_link_is_added_as_sam_asked_no_name(engine: harness::Engine) {
    let site = harness::site();
    let (_relay, ours, _dir, sam) = common::sams_computer(1).await;
    let mut phone = tokio::process::Command::new("node")
        .arg(harness::repo().join("web/tests/join.mjs"))
        .arg(json!({ "engine": engine.as_str(), "site": site }).to_string())
        .env("SVRNGS_BROWSERS", harness::browsers())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs: the browser tests need Node 18 or later");
    let mut out = BufReader::new(phone.stdout.take().unwrap()).lines();
    let first = out
        .next_line()
        .await
        .unwrap()
        .expect("the browser half prints its origin");
    let origin: Value = serde_json::from_str(&first).unwrap();
    let origin = Origin::parse(origin["origin"].as_str().unwrap()).unwrap();
    let page = cli::checked_page(origin).await.expect("the page is served");
    let (seed, _) = cli::new_group(
        &sam,
        Name::parse("house").unwrap(),
        Name::parse("Sam").unwrap(),
        Some(&page),
        &[],
        &ours,
    )
    .await
    .expect("Sam makes house");
    let (_, printed, _) = cli::link_to(&sam, "house", Some(&page), &ours, true, &[])
        .await
        .expect("Sam's computer makes a link for Sam's devices");
    let link = printed.https.clone().expect("an https link on the origin");
    let mut stdin = phone.stdin.take().unwrap();
    let told = json!({ "link": link, "name": "Pia" });
    stdin
        .write_all(format!("{told}\n").as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();
    let mut last = None;
    while let Some(line) = out.next_line().await.unwrap() {
        last = Some(line);
    }
    let mut stderr = String::new();
    phone
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .await
        .unwrap();
    let status = phone.wait().await.unwrap();
    assert!(
        status.success(),
        "the browser half failed ({status}):\n{stderr}"
    );
    let seen: Value = serde_json::from_str(&last.expect("the browser half printed")).unwrap();

    let first = &seen["first"];
    assert_eq!(
        first["asked"], false,
        "a device link asks no name:\n{seen:#}"
    );
    assert!(
        (first["said"].as_array().unwrap())
            .contains(&json!("Adding this phone to house as Sam’s…")),
        "the phone reads whose it is being added as:\n{seen:#}"
    );
    assert!(in_house(first, "Sam"), "{seen:#}");

    let key: [u8; 32] = hex::decode(seen["stored"]["key"].as_str().unwrap())
        .unwrap()
        .try_into()
        .expect("the page's key is 32 bytes");
    let phones = actor_of(&SigningKey::from_bytes(&key));
    let id = seed.id();
    let joined: Vec<_> = admits(&sam, &id)
        .into_iter()
        .filter_map(|(_, key, device)| (key == phones).then_some(device))
        .collect();
    assert_eq!(
        joined,
        vec![Some(Device::Browser)],
        "one Admit of the phone's key:\n{seen:#}"
    );
    let svrngs_node::register::Standing::Members(members) = sam.rail().standing(&id).unwrap()
    else {
        panic!("house is held at Sam's")
    };
    let m = &members[&phones];
    let sams = actor_of(&SigningKey::from_bytes(&[1; 32]));
    assert_eq!(
        (m.person.as_str(), m.person_key.as_str()),
        ("Sam", sams.as_str())
    );
    sam.endpoint().close().await;
}

/// Drive `web/tests/join.mjs` in `engine` at the built page served under
/// `policy` (`None`: the tests' allowance; `"production"`: the built meta
/// alone), joining `<origin>#<token>` as Pia, and answer what it saw and
/// the page's origin.
async fn phone_joining(
    engine: harness::Engine,
    site: &str,
    token: &str,
    policy: Option<&str>,
) -> (Value, String) {
    let mut phone = tokio::process::Command::new("node")
        .arg(harness::repo().join("web/tests/join.mjs"))
        .arg(json!({ "engine": engine.as_str(), "site": site, "policy": policy }).to_string())
        .env("SVRNGS_BROWSERS", harness::browsers())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs: the browser tests need Node 18 or later");
    let mut out = BufReader::new(phone.stdout.take().unwrap()).lines();
    let first = out.next_line().await.unwrap().unwrap_or_else(|| {
        panic!(
            "the browser half printed no origin; it exited {:?}",
            phone.try_wait()
        )
    });
    let origin: Value = serde_json::from_str(&first).unwrap();
    let origin = origin["origin"]
        .as_str()
        .expect("the page's origin")
        .to_string();
    let mut stdin = phone.stdin.take().unwrap();
    stdin
        .write_all(
            format!(
                "{}\n",
                json!({ "link": format!("{origin}#{token}"), "name": "Pia" })
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    stdin.flush().await.unwrap();
    let mut stderr = String::new();
    let mut err = phone.stderr.take().unwrap();
    let mut last = None;
    while let Some(line) = out.next_line().await.unwrap() {
        last = Some(line);
    }
    err.read_to_string(&mut stderr).await.unwrap();
    let status = phone.wait().await.unwrap();
    assert!(
        status.success(),
        "the browser half failed ({status}):\n{stderr}"
    );
    (
        serde_json::from_str(&last.expect("the browser half printed")).unwrap(),
        origin,
    )
}

/// Sam's computer, its group `house` met at `own` and reached through
/// `build` — which the node holds plainly, while the page reaches the same
/// front over TLS — and a link carrying `relays`. The data directory is
/// kept alive by the caller.
async fn sam_meeting_at(
    seed: u8,
    own: svrngs_core::relay::Url,
    build: &iroh_relay::server::Server,
    relays: Vec<svrngs_core::relay::Url>,
) -> (tempfile::TempDir, Node, String) {
    let dir = tempfile::tempdir().unwrap();
    let key = SigningKey::from_bytes(&[seed; 32]);
    let sam = Node::start(
        dir.path().to_path_buf(),
        key.clone(),
        &svrngs_node::iroh::RelayConfig {
            relay_urls: vec![harness::relay_url(build)],
            n0_services: false,
            loopback_only: true,
        },
        svrngs::group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("Sam's computer starts");
    let house = svrngs_node::group::draw(
        Name::parse("house").unwrap(),
        Name::parse("Sam").unwrap(),
        key.verifying_key(),
    );
    sam.host()
        .keep_group(&house, None, &[own])
        .expect("house kept");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let token = sam
        .host()
        .mint(&house.id(), relays, None, now)
        .expect("a link")
        .encode();
    (dir, sam, token)
}

in_webkit!(
    /// **A fresh browser under the published page's policy joins through
    /// the advertised public fallback, the group's own HTTP relay never
    /// tried** (relay-fallback charter). Sam's group is met at A, a
    /// plain-HTTP relay (the Pi's stand-in); his build's fallback is B,
    /// which the page reaches over TLS at the same front the node reaches
    /// plainly. The link he mints prefers A and carries B. The published
    /// `connect-src 'self' wss: https:` refuses A, so the page takes B
    /// alone, joins once and opens the lend. Planted: the page binding the
    /// link's first relay whatever its policy takes, and the join hangs at
    /// the blocked request (no name form — the incident's shape).
    a_browser_under_the_published_policy_joins_through_the_public_fallback
);
async fn a_browser_under_the_published_policy_joins_through_the_public_fallback(
    engine: harness::Engine,
) {
    use svrngs_core::relay::Url;
    let site = harness::site();
    let (own, public) = (harness::relay().await, harness::relay().await);
    let own_url: Url = harness::relay_url(&own).parse().unwrap();
    let public_https = format!("https://{}/", harness::relay_at(&public));
    let (dir, sam, token) = sam_meeting_at(
        1,
        own_url.clone(),
        &public,
        vec![own_url.clone(), public_https.parse().unwrap()],
    )
    .await;
    let shelf = Lend {
        name: LendName::parse("shelf").unwrap(),
        source: svrngs_node::lend::Source::Port(NonZeroU16::new(service().await).unwrap()),
        kind: Kind::parse("shelf").unwrap(),
        limit: NonZeroU16::new(3),
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        sign_in_with_svrngs: None,
        asks: None,
    };
    lend::declare(dir.path(), &shelf, None).unwrap();

    let (seen, origin) =
        phone_joining(engine, site.to_str().unwrap(), &token, Some("production")).await;
    let port = origin
        .trim_end_matches('/')
        .rsplit(':')
        .next()
        .unwrap()
        .to_string();
    eprintln!("published policy: {seen:#}");

    // In through B alone, and the lend opens at its own origin from A's side
    // of the group's record.
    let first = &seen["first"];
    assert!(in_house(first, "Pia"), "{seen:#}");
    for again in ["again", "alone"] {
        assert!(in_house(&seen[again], "Pia"), "{again}:\n{seen:#}");
    }
    let shelf_label = svrngs_core::pass::Label::of(
        SigningKey::from_bytes(&[1; 32]).verifying_key().as_bytes(),
        "shelf",
    );
    assert_eq!(
        seen["opened"],
        json!({ "origin": format!("https://{shelf_label}.{}:{port}", harness::PAGE_HOST), "text": "hi" }),
        "{seen:#}"
    );

    // A was never tried: no request was blocked, and no line names it.
    let logs = seen["logs"].as_array().unwrap();
    let refused = |l: &Value| {
        let l = l.as_str().unwrap_or_default();
        [
            "Content Security Policy",
            "violates",
            "mixed content",
            "blocked",
        ]
        .iter()
        .any(|word| l.contains(word))
    };
    assert!(
        !logs.iter().any(refused),
        "a request was refused by the page's policy:\n{seen:#}"
    );
    let lines = seen["lines"].as_array().unwrap();
    let said: Vec<&str> = lines.iter().filter_map(Value::as_str).collect();
    assert!(
        !said.iter().any(|l| l.contains(&own_url.to_string())),
        "the HTTP relay was dialled:\n{seen:#}"
    );
    assert!(
        said.iter()
            .any(|l| l.contains("LAYER parse: ok — relay") && l.contains(&public_https)),
        "the fallback was bound:\n{seen:#}"
    );
    sam.endpoint().close().await;
}

in_webkit!(
    /// **A relay that answers but cannot carry to the issuer does not win**
    /// (relay-fallback charter). The link prefers A, a live relay the
    /// keeper is not at — its probe answers, and a dial there finds no
    /// one — and carries B, where the keeper is. The browser tries both at
    /// once and is let in through B. Planted: `reach` dialling the first
    /// route alone, and the join fails.
    a_route_that_answers_but_cannot_reach_the_issuer_does_not_win
);
async fn a_route_that_answers_but_cannot_reach_the_issuer_does_not_win(engine: harness::Engine) {
    use svrngs_core::relay::Url;
    let site = harness::site();
    let (dead, public) = (harness::relay().await, harness::relay().await);
    let dead_https = format!("https://{}/", harness::relay_at(&dead));
    let dead_own: Url = harness::relay_url(&dead).parse().unwrap();
    let public_https = format!("https://{}/", harness::relay_at(&public));
    let (_dir, sam, token) = sam_meeting_at(
        2,
        dead_own,
        &public,
        vec![dead_https.parse().unwrap(), public_https.parse().unwrap()],
    )
    .await;
    let (seen, _) = phone_joining(engine, site.to_str().unwrap(), &token, Some("production")).await;
    eprintln!("dead-end route: {seen:#}");
    assert!(in_house(&seen["first"], "Pia"), "{seen:#}");
    let said: Vec<&str> = seen["lines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        said.iter()
            .any(|l| l.contains("dialing") && l.contains(&dead_https)),
        "the dead-end route was not tried:\n{seen:#}"
    );
    assert!(
        said.iter()
            .any(|l| l.contains("QUIC connection established")),
        "{seen:#}"
    );
    sam.endpoint().close().await;
}
