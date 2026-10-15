//! An app the group carries (v1ui-shelf (a); svrn-docs
//! `rings/current/V1_UI_BUILD.md` §3.4, §7; `COMMUNITY_APPS.md` §6-§7): a
//! folder in the group's synced folder, served at its own origin by each
//! computer that holds it, from the store; its route, reading only the
//! app's own acts and writing only as the person; the floor's headers on
//! every answer. Ana, Ben and Cy in one group, three nodes in one process,
//! no relay, each reached by the addresses seeded in the others' lookup,
//! as `folders.rs` stands them up.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
mod common;
use common::loopback_sockets;

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::iroh::{EndpointAddr, MemoryLookup, RelayConfig};
use svrngs_node::node::Node;
use svrngs_node::seed::Seed;
use svrngs_node::{actor_of, Name, Person, RailAct, SigningKey};

fn signing(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

fn seed_file(dir: &Path) -> Result<Option<String>, String> {
    match std::fs::read_to_string(dir.join("seed.json")) {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

async fn start(seed: u8, dir: &Path) -> Node {
    let relay = RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    };
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &relay,
        seed_file,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

/// A group Ana founds, named `name`, salted by `salt`, with each of
/// `others` (Ben 2, Cy 3) admitted: its id, on every node named.
fn found(name: &str, salt: u8, ana: &Node, others: &[(&Node, u8)]) -> String {
    let seed = Seed::from_parts(
        Name::parse(name).unwrap(),
        Name::parse("ana").unwrap(),
        signing(1).verifying_key(),
        [salt; 16],
    );
    let id = seed.id();
    for node in std::iter::once(ana).chain(others.iter().map(|(n, _)| *n)) {
        node.rail().set_seed(&id, &seed.roster()).unwrap();
        let dir = node.rail().group_dir(&id).unwrap();
        std::fs::write(dir.join("seed.json"), seed.to_text()).unwrap();
    }
    for &(_, n) in others {
        let person = ["", "", "ben", "cy"][n as usize];
        let admit = svrngs_core::tests_support::Fixture {
            act: RailAct::Admit {
                person: Person::parse(person).unwrap(),
                key: actor_of(&signing(n)),
                invite: None,
            },
            on_behalf_of: None,
        };
        ana.rail().commit(&id, &admit).unwrap().unwrap();
    }
    id
}

/// Ana's, Ben's and Cy's computers, each knowing where the others listen.
async fn computers() -> (Vec<Node>, Vec<tempfile::TempDir>) {
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let mut nodes = Vec::new();
    for (i, dir) in dirs.iter().enumerate() {
        nodes.push(start(i as u8 + 1, dir.path()).await);
    }
    for who in &nodes {
        let book = MemoryLookup::new();
        for other in &nodes {
            let mut addr = EndpointAddr::new(other.endpoint().id());
            for a in loopback_sockets(other.endpoint()) {
                addr = addr.with_ip_addr(a);
            }
            book.add_endpoint_info(addr);
            who.transport()
                .add_known_peer(other.endpoint().id(), loopback_sockets(other.endpoint()));
        }
        who.endpoint().address_lookup().unwrap().add(book);
    }
    (nodes, dirs)
}

async fn round(node: &Node) {
    tokio::time::timeout(Duration::from_secs(30), node.round())
        .await
        .expect("a round finishes");
}

/// Up to 30 s, rounds run on `nodes`: whether `done` came true.
async fn until(nodes: &[&Node], done: impl Fn() -> bool) -> bool {
    for _ in 0..30 {
        if done() {
            return true;
        }
        for node in nodes {
            round(node).await;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    done()
}

fn http() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

/// `who` syncs a directory of its own under `root` in `group`, as
/// `share --folder` asks it: the directory.
async fn synced_in(who: &Node, root: &Path, group: &str) -> PathBuf {
    let dir = root.join("Shared");
    std::fs::create_dir_all(&dir).unwrap();
    let said = http()
        .post(format!("http://{}/v1/folders", who.api_addr()))
        .json(&json!({ "dir": dir, "group": group }))
        .send()
        .await
        .unwrap();
    assert!(said.status().is_success(), "{:?}", said.text().await);
    dir
}

/// The app `name` written into `dir`, the group's synced folder, as the
/// shelf brings one: its page, which loads its script, and the script.
fn carry(dir: &Path, name: &str) {
    let app = dir.join("apps").join(name);
    std::fs::create_dir_all(&app).unwrap();
    let page =
        format!("<!doctype html><title>{name}</title><script type=module src=app.js></script>");
    std::fs::write(app.join("index.html"), page).unwrap();
    std::fs::write(app.join("app.js"), format!("// {name}'s script\n")).unwrap();
}

/// What `node` lends its browser in `group` now (`GET /v1/lends`).
async fn lends(node: &Node, group: &str) -> Value {
    let url = format!("http://{}/v1/lends?namespace={group}", node.api_addr());
    let answer = http().get(url).send().await.unwrap();
    assert!(answer.status().is_success(), "{:?}", answer.text().await);
    answer.json().await.unwrap()
}

/// The lends of `kind` listed in `lends`.
fn of_kind<'a>(lends: &'a Value, kind: &str) -> Vec<&'a Value> {
    let lent = lends["lent"].as_array().unwrap();
    lent.iter().filter(|l| l["kind"] == kind).collect()
}

/// `path` asked at the origin `origin` on `node`'s listener, as a
/// program on that computer asks it: the answer.
async fn at(
    node: &Node,
    origin: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> reqwest::Response {
    let host = origin.trim_start_matches("http://").trim_end_matches('/');
    let url = format!("http://{}{path}", node.api_addr());
    let ask = http()
        .request(method.parse().unwrap(), url)
        .header("host", host);
    let ask = match body {
        Some(b) => ask.json(&b),
        None => ask,
    };
    ask.send().await.unwrap()
}

/// The ops of an app's route's answer that a fold counts, each `[by, op]`:
/// not voided, and stating something.
fn live(read: &Value) -> Vec<Value> {
    let ops = read["ops"].as_array().expect("the route answers its ops");
    let counted = ops
        .iter()
        .filter(|o| o["voided"] == false && !o["payload"].is_null());
    counted
        .map(|o| json!([o["on_behalf_of"], o["payload"]]))
        .collect()
}

/// The containment floor's headers (COMMUNITY_APPS §6.2) on `answer`, or
/// what of them is missing.
fn floor(answer: &reqwest::Response) -> Result<(), String> {
    let h = answer.headers();
    let policy = h
        .get("content-security-policy")
        .and_then(|v| v.to_str().ok());
    let policy = policy.ok_or("no content-security-policy")?;
    for directive in [
        "default-src 'self'",
        "script-src 'self'",
        "connect-src 'self'",
        "object-src 'none'",
        "base-uri 'none'",
    ] {
        if !policy.contains(directive) {
            return Err(format!("the policy lacks {directive}: {policy}"));
        }
    }
    if h.get("x-dns-prefetch-control")
        .and_then(|v| v.to_str().ok())
        != Some("off")
    {
        return Err("no x-dns-prefetch-control: off".into());
    }
    Ok(())
}

/// **An app Ana's computer carried opens and writes on Cy's with Ana's
/// stopped** (V1_UI_BUILD §7, 3.4's first bar, the computer's half): Ana
/// writes Money into the group's folder; it reaches Cy's; Ana's computer
/// stops. Cy's home page lists Money once, at Cy's own origin (found 4),
/// its page and script come from Cy's store with the floor's headers, and
/// an expense written there is Cy's, read back by the route. Planted: a
/// carried app served only by whoever brought it — the carried apps read
/// from the entries this computer wrote — and Cy lists no Money.
#[tokio::test]
async fn an_app_one_computer_carried_opens_and_writes_on_another_with_it_stopped() {
    let (nodes, _dirs) = computers().await;
    let (ana, ben, cy) = (&nodes[0], &nodes[1], &nodes[2]);
    let group = found("house", 9, ana, &[(ben, 2), (cy, 3)]);
    for node in [cy, ben, ana] {
        round(node).await;
    }
    let roots: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    let at_ana = synced_in(ana, roots[0].path(), &group).await;
    let at_cy = synced_in(cy, roots[1].path(), &group).await;
    carry(&at_ana, "Money");
    let page = at_cy.join("apps/Money/index.html");
    let script = at_cy.join("apps/Money/app.js");
    assert!(
        until(&[ana, cy], || page.exists() && script.exists()).await,
        "Money reaches Cy's folder"
    );
    ana.endpoint().close().await;

    let listed = lends(cy, &group).await;
    let money = of_kind(&listed, "Money");
    assert_eq!(
        money.len(),
        1,
        "Cy's copy, Ana's computer not answering:\n{listed:#}"
    );
    let money = money[0];
    assert_eq!(
        money["key"],
        actor_of(&signing(3)),
        "Cy's own copy:\n{listed:#}"
    );
    assert_eq!(money["group"], group.as_str(), "{listed:#}");
    let origin = money["origin"].as_str().unwrap();

    let opened = at(cy, origin, "GET", "/", None).await;
    assert_eq!(opened.status(), 200);
    assert_eq!(floor(&opened), Ok(()));
    let html = opened.text().await.unwrap();
    assert!(html.contains("<title>Money</title>"), "{html}");
    let js = at(cy, origin, "GET", "/app.js", None).await;
    assert_eq!(
        js.headers()["content-type"],
        "text/javascript; charset=utf-8"
    );
    assert_eq!(js.text().await.unwrap(), "// Money's script\n");

    let op = json!({ "kind": "expense", "amount_cents": 4200, "description": "the gas bill" });
    let wrote = at(
        cy,
        origin,
        "POST",
        "/svrngs/acts",
        Some(json!({ "op": op })),
    )
    .await;
    assert!(wrote.status().is_success(), "{:?}", wrote.text().await);
    assert_eq!(floor(&wrote), Ok(()));
    let read: Value = at(cy, origin, "GET", "/svrngs/acts", None)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(read["you"], "cy", "{read:#}");
    assert_eq!(
        live(&read),
        [json!(["cy", op])],
        "the app reads its act as it wrote it, Cy's:\n{read:#}"
    );
    for node in [ben, cy] {
        node.endpoint().close().await;
    }
}

/// **An app reads only its own acts and writes only as the person, from
/// its route** (V1_UI_BUILD §7, 3.4's second bar; COMMUNITY_APPS §6.2,
/// §7 found 6): Money and Events carried on Cy's computer. Events reads
/// none of Money's acts; an act it writes naming another person, or
/// another app's kind, is still its own and Cy's; it may not correct
/// Money's act, and Money may. Planted: the stamp read from the
/// request's body — Events' act is Ben's.
#[tokio::test]
async fn an_app_reads_only_its_own_acts_and_writes_only_as_the_person() {
    let (nodes, _dirs) = computers().await;
    let (ana, ben, cy) = (&nodes[0], &nodes[1], &nodes[2]);
    let group = found("house", 9, ana, &[(ben, 2), (cy, 3)]);
    for node in [cy, ben, ana] {
        round(node).await;
    }
    let root = tempfile::tempdir().unwrap();
    let at_cy = synced_in(cy, root.path(), &group).await;
    carry(&at_cy, "Money");
    carry(&at_cy, "Events");
    let listed = || async { lends(cy, &group).await };
    let mut seen = listed().await;
    for _ in 0..30 {
        if of_kind(&seen, "Money").len() == 1 && of_kind(&seen, "Events").len() == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        seen = listed().await;
    }
    let origin = |kind: &str| {
        of_kind(&seen, kind)[0]["origin"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let (money, events) = (origin("Money"), origin("Events"));
    assert_ne!(money, events);

    let wrote: Value = at(
        cy,
        &money,
        "POST",
        "/svrngs/acts",
        Some(json!({ "op": { "kind": "expense", "amount_cents": 100 } })),
    )
    .await
    .json()
    .await
    .unwrap();
    let theirs = wrote["id"]
        .as_str()
        .expect("Money's act has an id")
        .to_string();

    let read: Value = at(cy, &events, "GET", "/svrngs/acts", None)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        read["ops"],
        json!([]),
        "Events reads none of Money's acts:\n{read:#}"
    );

    let as_ben = json!({ "op": { "kind": "Money/expense", "amount_cents": 1 }, "on_behalf_of": "ben", "by": "ben" });
    let posted = at(cy, &events, "POST", "/svrngs/acts", Some(as_ben)).await;
    assert!(posted.status().is_success(), "{:?}", posted.text().await);
    let read: Value = at(cy, &events, "GET", "/svrngs/acts", None)
        .await
        .json()
        .await
        .unwrap();
    let acts = live(&read);
    assert_eq!(acts.len(), 1, "{read:#}");
    assert_eq!(
        acts[0][0], "cy",
        "the stamp is the person's, never the body's:\n{read:#}"
    );
    let money_read: Value = at(cy, &money, "GET", "/svrngs/acts", None)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        live(&money_read).len(),
        1,
        "an act Events wrote is never Money's, whatever its kind says:\n{money_read:#}"
    );

    let void = json!({ "corrects": theirs, "op": null });
    let refused = at(cy, &events, "POST", "/svrngs/acts", Some(void.clone())).await;
    assert_eq!(refused.status(), 403, "Events may not void Money's act");
    assert_eq!(floor(&refused), Ok(()));
    let own = at(cy, &money, "POST", "/svrngs/acts", Some(void)).await;
    assert!(own.status().is_success(), "{:?}", own.text().await);
    let money_read: Value = at(cy, &money, "GET", "/svrngs/acts", None)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        live(&money_read),
        Vec::<Value>::new(),
        "Money voided its own act:\n{money_read:#}"
    );
    let ops = money_read["ops"].as_array().unwrap();
    assert!(
        ops.iter().any(|o| o["corrects"] == theirs.as_str()),
        "and reads the correction, as a fold that decides voids needs:\n{money_read:#}"
    );
    for node in [ana, ben, cy] {
        node.endpoint().close().await;
    }
}

/// **A carried app is its own group's alone, and its name carries the
/// group** (COMMUNITY_APPS §7, found 2 and 3): Ana's computer carries a
/// Money in each of two groups, Ben in one and Cy in the other. Ana's
/// lists one Money in each, at two origins; Ben's computer is told only
/// his group's, and its door lets Ben into his alone.
#[tokio::test]
async fn a_carried_app_is_its_own_group_s_alone() {
    let (nodes, _dirs) = computers().await;
    let (ana, ben, cy) = (&nodes[0], &nodes[1], &nodes[2]);
    let films = found("films", 9, ana, &[(ben, 2)]);
    let house = found("house", 7, ana, &[(cy, 3)]);
    for node in [cy, ben, ana] {
        round(node).await;
    }
    let roots: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    for (root, group) in roots.iter().zip([&films, &house]) {
        carry(&synced_in(ana, root.path(), group).await, "Money");
    }
    let mut origins = Vec::new();
    for group in [&films, &house] {
        let mut seen = lends(ana, group).await;
        for _ in 0..30 {
            if !of_kind(&seen, "Money").is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
            seen = lends(ana, group).await;
        }
        let money = of_kind(&seen, "Money");
        assert_eq!(money.len(), 1, "one Money in each group:\n{seen:#}");
        origins.push(money[0]["origin"].as_str().unwrap().to_string());
    }
    assert_ne!(origins[0], origins[1], "two groups' Money, two origins");

    let at_ben = lends(ben, &films).await;
    let money = of_kind(&at_ben, "Money");
    assert_eq!(money.len(), 1, "Ben is told his group's Money:\n{at_ben:#}");
    assert_eq!(money[0]["group"], films.as_str(), "{at_ben:#}");
    let label = |o: &str| {
        o.trim_start_matches("http://")
            .split('.')
            .next()
            .unwrap()
            .to_string()
    };
    let bridge = common::bridge_as(&signing(2), ana, svrngs_core::alpn::Alpn::Member).await;
    let ask = |l: String| {
        http()
            .get(format!("http://{}/lend/{l}/", bridge.local_addr()))
            .send()
    };
    let his = ask(label(&origins[0])).await.unwrap();
    assert_eq!(his.status(), 200, "Ben opens his group's Money");
    let not_his = ask(label(&origins[1])).await.unwrap();
    assert_eq!(
        not_his.status(),
        403,
        "Ben is let into no other group's Money"
    );
    assert_eq!(not_his.headers()["x-svrngs-refused"], "closed");
    for node in [ana, ben, cy] {
        node.endpoint().close().await;
    }
}
