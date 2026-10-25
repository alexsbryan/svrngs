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
use common::heard;
#[path = "common/computers.rs"]
mod computers;
use computers::{found, http, known, round, signing, start, until_on as until};

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, RailAct};

/// Ana's, Ben's and Cy's computers, each knowing where the others listen.
async fn computers() -> (Vec<Node>, Vec<tempfile::TempDir>) {
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let mut nodes = Vec::new();
    for (i, dir) in dirs.iter().enumerate() {
        nodes.push(start(i as u8 + 1, dir.path()).await);
    }
    known(&nodes.iter().collect::<Vec<_>>());
    (nodes, dirs)
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
/// not voided, and stating something. `by` as the apps read it
/// (`apps/expenses/expenses.js`): `on_behalf_of` when the op names one,
/// else its `person`.
fn live(read: &Value) -> Vec<Value> {
    let ops = read["ops"].as_array().expect("the route answers its ops");
    let counted = ops
        .iter()
        .filter(|o| o["voided"] == false && !o["payload"].is_null());
    counted
        .map(|o| {
            let by = Some(&o["on_behalf_of"]).filter(|b| !b.is_null());
            json!([by.unwrap_or(&o["person"]), o["payload"]])
        })
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
/// request's body — Events' act is Ben's. A correction with no `op` is
/// no act, never a void, and a write past an act's most is told so, not
/// that it is no JSON (#165). Planted: HEAD before #165's fix, which voided
/// Money's act on `{corrects}` alone.
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

    let bare = json!({ "corrects": theirs });
    let refused = at(cy, &money, "POST", "/svrngs/acts", Some(bare)).await;
    assert_eq!(refused.status(), 422, "{{corrects}} alone is no act");
    let read: Value = at(cy, &money, "GET", "/svrngs/acts", None)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(live(&read).len(), 1, "and voids nothing:\n{read:#}");
    let big = json!({ "op": { "kind": "note", "text": "x".repeat(70 * 1024) } });
    let refused = at(cy, &money, "POST", "/svrngs/acts", Some(big)).await;
    assert_eq!(refused.status(), 413);
    let said = refused.text().await.unwrap();
    assert!(
        said.contains("too large") && !said.contains("{op}"),
        "{said}"
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

/// **A file the store holds but cannot read is said so, not as not held
/// yet** (#163): Ana carries an app with a 100 KiB file, which her store
/// keeps outside its database, and it opens; then its data is cut to
/// nothing under the store. The file is refused 500, `fault`, as unread
/// from this computer's store, never 503 "does not hold it yet", which is
/// a computer that has not been reached. Planted: HEAD before #163's fix,
/// which said every failed read as not held yet.
#[tokio::test]
async fn a_file_the_store_cannot_read_is_said_so_never_as_not_held_yet() {
    let (nodes, dirs) = computers().await;
    let ana = &nodes[0];
    let group = found("house", 9, ana, &[]);
    round(ana).await;
    let root = tempfile::tempdir().unwrap();
    let folder = synced_in(ana, root.path(), &group).await;
    carry(&folder, "Money");
    let big = "x".repeat(100 * 1024);
    std::fs::write(folder.join("apps/Money/big.js"), &big).unwrap();
    let mut origin = None;
    for _ in 0..30 {
        let seen = lends(ana, &group).await;
        origin = of_kind(&seen, "Money")
            .first()
            .map(|l| l["origin"].as_str().unwrap().to_string());
        if let Some(o) = &origin {
            if at(ana, o, "GET", "/big.js", None).await.status() == 200 {
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let origin = origin.expect("Money is listed");
    assert_eq!(at(ana, &origin, "GET", "/big.js", None).await.status(), 200);
    let data = dirs[0].path().join("blobs/data");
    let held = std::fs::read_dir(&data).unwrap().map(|e| e.unwrap().path());
    let held: Vec<_> = held
        .filter(|p| std::fs::metadata(p).unwrap().len() == big.len() as u64)
        .collect();
    assert_eq!(held.len(), 1, "the file's data in {}", data.display());
    std::fs::OpenOptions::new()
        .write(true)
        .open(&held[0])
        .unwrap()
        .set_len(0)
        .unwrap();
    let answer = at(ana, &origin, "GET", "/big.js", None).await;
    let status = answer.status();
    let said = answer.text().await.unwrap();
    assert_eq!(status, 500, "{said}");
    assert!(
        said.contains(r#""kind":"fault""#) && !said.contains("does not hold it yet"),
        "{said}"
    );
    ana.endpoint().close().await;
}

/// **Bringing from the shelf traces why it could not write** (#164): Ana's
/// folder of the group made unwritable, her home page brings Money; it is
/// refused 500 in the page's words, and the trace names the write's own
/// error, which the page is not told. Planted: HEAD before #164's fix,
/// whose trace said the status alone.
#[cfg(unix)]
#[tokio::test]
async fn bringing_from_the_shelf_traces_why_it_could_not_write() {
    use std::os::unix::fs::PermissionsExt;
    let (said, _heard) = heard("folder=debug");
    let (nodes, _dirs) = computers().await;
    let ana = &nodes[0];
    let group = found("house", 9, ana, &[]);
    round(ana).await;
    let root = tempfile::tempdir().unwrap();
    let folder = synced_in(ana, root.path(), &group).await;
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o555)).unwrap();
    let home = format!("svrngs.localhost:{}", ana.api_addr().port());
    let answer = http()
        .post(format!("http://{}/shelf", ana.api_addr()))
        .header("host", &home)
        .header("sec-fetch-site", "same-origin")
        .header("sec-fetch-mode", "cors")
        .header("origin", format!("http://{home}"))
        .json(&json!([group, ["Money"]]))
        .send()
        .await
        .unwrap();
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).unwrap();
    let status = answer.status();
    let words = answer.text().await.unwrap();
    assert_eq!(status, 500, "{words}");
    assert!(
        words.contains("can’t be written") && !words.contains("denied"),
        "{words}"
    );
    let traced = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let why = |l: &str| l.contains("Permission denied") && l.contains("can’t be written");
    assert!(
        traced.lines().any(why),
        "the write's error traced:\n{traced}"
    );
    ana.endpoint().close().await;
}

/// **A folder under apps/ no lend can be named by is traced as left out**
/// (#167): Ana's folder holds Money and `apps/no kind!/`, whose name is no
/// kind; Money is carried, and the trace names the other as left out.
/// Planted: HEAD before #167's fix, which dropped it untraced.
#[tokio::test]
async fn a_folder_under_apps_no_lend_can_be_named_by_is_traced_as_left_out() {
    let (said, _heard) = heard("folder=debug");
    let (nodes, _dirs) = computers().await;
    let ana = &nodes[0];
    let group = found("house", 9, ana, &[]);
    round(ana).await;
    let root = tempfile::tempdir().unwrap();
    let folder = synced_in(ana, root.path(), &group).await;
    carry(&folder, "Money");
    carry(&folder, "no kind!");
    let mut seen = lends(ana, &group).await;
    for _ in 0..30 {
        if !of_kind(&seen, "Money").is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        seen = lends(ana, &group).await;
    }
    assert_eq!(of_kind(&seen, "Money").len(), 1, "{seen:#}");
    let traced = String::from_utf8_lossy(&said.0.lock().unwrap()).into_owned();
    let left = |l: &str| l.contains("no kind!") && l.contains("left out");
    assert!(
        traced.lines().any(left),
        "the folder left out is named:\n{traced}"
    );
    ana.endpoint().close().await;
}

/// **A carried app runs as it was brought after a member changes its
/// files** (canon-pin; svrn-docs `rings/current/WHAT_RUNS_AS_YOU.md` §2,
/// §3): Ana brings Money into the group's folder and it reaches Ben's and
/// Cy's; Ben replaces its `app.js` in his folder, and his change reaches
/// Ana's and Cy's. Each of their computers still serves Money's `app.js`
/// as Ana brought it. Red at HEAD (canon-pin (a)): a carried app's file is
/// served by the hash the group's layout places now (carried.rs `file`),
/// so every member's browser runs Ben's script at once, writing as them.
#[tokio::test]
async fn a_carried_app_runs_as_it_was_brought_after_a_member_changes_it() {
    let (nodes, _dirs) = computers().await;
    let (ana, ben, cy) = (&nodes[0], &nodes[1], &nodes[2]);
    let group = found("house", 9, ana, &[(ben, 2), (cy, 3)]);
    for node in [cy, ben, ana] {
        round(node).await;
    }
    let roots: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let mut folders = Vec::new();
    for (node, root) in [ana, ben, cy].into_iter().zip(&roots) {
        folders.push(synced_in(node, root.path(), &group).await);
    }
    carry(&folders[0], "Money");
    let script = |i: usize| std::fs::read_to_string(folders[i].join("apps/Money/app.js"));
    let brought = "// Money's script\n";
    assert!(
        until(&[ana, ben, cy], || (0..3)
            .all(|i| script(i).is_ok_and(|s| s == brought)))
        .await,
        "Money reaches every folder"
    );
    let changed = "// Ben's script, run as whoever opens Money\n";
    std::fs::write(folders[1].join("apps/Money/app.js"), changed).unwrap();
    assert!(
        until(&[ben, ana, cy], || [0, 2]
            .iter()
            .all(|&i| script(i).is_ok_and(|s| s == changed)))
        .await,
        "Ben's change reaches Ana's and Cy's folders"
    );
    for (who, node, seed) in [("Ana", ana, 1), ("Cy", cy, 3)] {
        let listed = lends(node, &group).await;
        let own = of_kind(&listed, "Money")
            .into_iter()
            .find(|l| l["key"] == actor_of(&signing(seed)));
        let origin = own.expect("its own copy")["origin"]
            .as_str()
            .unwrap()
            .to_string();
        let js = at(node, &origin, "GET", "/app.js", None).await;
        assert_eq!(js.status(), 200);
        assert_eq!(
            js.text().await.unwrap(),
            brought,
            "Money runs as it was brought on {who}'s computer, not as Ben changed it"
        );
    }
    for node in [ana, ben, cy] {
        node.endpoint().close().await;
    }
}

/// **A computer that never held an app as it was brought fetches it**
/// (canon-pin (b)): Ana brings Money and Ben changes its script before Cy
/// keeps a folder of the group, so Cy's folder and store take Ben's
/// version alone. Cy's computer serves Money's script as Ana brought it,
/// fetched from a member's. Planted: no fetch — 503, "does not hold it
/// yet", for as long as Cy's computer is on.
#[tokio::test]
async fn a_computer_that_never_held_an_app_as_brought_fetches_it() {
    let (nodes, _dirs) = computers().await;
    let (ana, ben, cy) = (&nodes[0], &nodes[1], &nodes[2]);
    let group = found("house", 9, ana, &[(ben, 2), (cy, 3)]);
    for node in [cy, ben, ana] {
        round(node).await;
    }
    let roots: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let at_ana = synced_in(ana, roots[0].path(), &group).await;
    let at_ben = synced_in(ben, roots[1].path(), &group).await;
    carry(&at_ana, "Money");
    let script = |dir: &Path| std::fs::read_to_string(dir.join("apps/Money/app.js"));
    let (brought, changed) = ("// Money's script\n", "// Ben's script\n");
    assert!(until(&[ana, ben], || script(&at_ben).is_ok_and(|s| s == brought)).await);
    std::fs::write(at_ben.join("apps/Money/app.js"), changed).unwrap();
    assert!(until(&[ben, ana], || script(&at_ana).is_ok_and(|s| s == changed)).await);
    // Cy's record takes Ben's change first, so Cy's folder never fetches
    // Money as it was brought.
    round(cy).await;
    let at_cy = synced_in(cy, roots[2].path(), &group).await;
    assert!(
        until(&[cy, ana, ben], || script(&at_cy)
            .is_ok_and(|s| s == changed))
        .await
    );
    let listed = lends(cy, &group).await;
    let own = of_kind(&listed, "Money")
        .into_iter()
        .find(|l| l["key"] == actor_of(&signing(3)));
    let origin = own.expect("Cy's own copy")["origin"]
        .as_str()
        .unwrap()
        .to_string();
    let js = at(cy, &origin, "GET", "/app.js", None).await;
    assert_eq!(js.status(), 200);
    assert_eq!(
        js.text().await.unwrap(),
        brought,
        "fetched as Ana brought it"
    );
    for node in [ana, ben, cy] {
        node.endpoint().close().await;
    }
}

/// **A house's history, asked** (canon-spans (a)): Ana founds a group, Ben
/// joins, Cy joins and then leaves, and Ana's computer is asked
/// `svrngs/acts` for Money. It answers each person's time in the group,
/// Cy's with when Cy left, so a house's rules count a past decision with
/// whoever stood then; `people` stays today's names, so Money is
/// untouched. Each time is read against Ana's register, never this host's
/// clock: a commit is timed after every act it holds (svrngs-core
/// command.rs `commit`), so may run ahead of it. Red at ce3396d8: the
/// answer had today's names alone. Planted (canon-spans (b)): `left` read
/// as none for everyone, and Cy reads as standing still.
#[tokio::test]
async fn an_app_is_told_each_person_s_time_in_the_group() {
    use svrngs_node::removal::{LeaveAsk, Outcome, LEAVES_ROUTE};
    let (nodes, _dirs) = computers().await;
    let (ana, ben, cy) = (&nodes[0], &nodes[1], &nodes[2]);
    let group = found("house", 9, ana, &[(ben, 2), (cy, 3)]);
    for node in [cy, ben, ana] {
        round(node).await;
    }
    let root = tempfile::tempdir().unwrap();
    carry(&synced_in(ana, root.path(), &group).await, "Money");
    let leave = http().post(format!("http://{}{LEAVES_ROUTE}", cy.api_addr()));
    let ask = leave
        .json(&LeaveAsk {
            group: group.clone(),
        })
        .send();
    assert_eq!(
        ask.await.unwrap().json::<Outcome>().await.unwrap(),
        Outcome::Left
    );
    let cy_key = actor_of(&signing(3));
    assert!(
        until(&[ana], || ana.rail().standing_anywhere(&cy_key).is_none()).await,
        "Cy's leave reaches Ana's computer"
    );
    let mut seen = lends(ana, &group).await;
    for _ in 0..30 {
        if !of_kind(&seen, "Money").is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        seen = lends(ana, &group).await;
    }
    let origin = of_kind(&seen, "Money")[0]["origin"]
        .as_str()
        .unwrap()
        .to_string();
    let read: Value = (at(ana, &origin, "GET", "/svrngs/acts", None)
        .await
        .json()
        .await)
        .unwrap();
    assert_eq!(
        read["people"],
        json!(["ana", "ben"]),
        "today's names, as before:\n{read:#}"
    );
    assert_eq!(
        read["group"],
        json!(group),
        "the group, which House rules names its house by:\n{read:#}"
    );
    let spans = read["spans"]
        .as_array()
        .expect("each person's time in the group");
    let span = |who: &str| -> Vec<(Value, Value)> {
        let theirs = spans.iter().filter(|s| s["person"] == who);
        theirs
            .map(|s| (s["joined"].clone(), s["left"].clone()))
            .collect()
    };
    let register = ana.rail().register(&group).unwrap();
    let ben_joined = register.members[&actor_of(&signing(2))].joined;
    let cy_left = (register.removals.iter())
        .find(|r| r.key == cy_key)
        .map(|r| r.at)
        .expect("Ana's register holds Cy's leave");
    assert_eq!(
        span("ana"),
        [(Value::Null, Value::Null)],
        "Ana founded it and stands:\n{read:#}"
    );
    let ben_s = span("ben");
    assert!(
        matches!(&ben_s[..], [(j, Value::Null)] if svrngs_core::Joined::At(j.as_i64().unwrap()) == ben_joined),
        "Ben joined when Ana admitted him, {ben_joined:?}, and stands:\n{read:#}"
    );
    let cy_s = span("cy");
    let (cy_joined, ben_joined) = (cy_s.first().and_then(|s| s.0.as_i64()), ben_s[0].0.as_i64());
    assert!(cy_joined > ben_joined, "Cy joined after Ben:\n{read:#}");
    assert_eq!(
        cy_s.iter().map(|s| s.1.as_i64()).collect::<Vec<_>>(),
        [Some(cy_left)],
        "Cy's one stretch ends when Cy's leave is timed:\n{read:#}"
    );
    assert_eq!(spans.len(), 3, "{read:#}");
    for node in [ana, ben, cy] {
        node.endpoint().close().await;
    }
}

/// **A line a device signed is carried unchanged, and only one its signer
/// stands to sign** (capability-signed; `demo/runs/authorship/2026-10-24/
/// DESIGN.md` §3, "The computer's part"): Ben's phone carries lines to
/// Ana's computer by `/carry`. A line a stranger's key signed, and Ben's
/// line with another act put under his signature, are each refused,
/// `not_signed`, "This entry was not signed by anyone in …", and Ana's copy
/// holds neither. Kept: Ben's own line is stored as he signed it, its
/// bytes, its actor and its person his, and counts, and Ana's computer
/// writes no line of its own for it. Planted: the signature's check left
/// out — the tampered line is stored.
#[tokio::test]
async fn a_line_is_carried_only_signed_by_someone_in_the_group() {
    let (nodes, _dirs) = computers().await;
    let (ana, ben) = (&nodes[0], &nodes[1]);
    let group = found("house", 11, ana, &[(ben, 2)]);
    let bridge = common::bridge_as(&signing(2), ana, svrngs_core::alpn::Alpn::Member).await;
    let carry = |line: &svrngs_core::Op<svrngs_core::SignedOp>| {
        let at = format!("http://{}/carry", bridge.local_addr());
        http().post(at).json(&json!([group, line])).send()
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1;
    let act = |n: u32| {
        RailAct::from_json(
            json!({ "op": "record", "payload": { "kind": "Money/expense", "n": n } }),
        )
        .unwrap()
    };
    let line = |k: u8, n: u32| {
        svrngs_core::sign_act(&signing(k), &group, now, 0, None, act(n), None, None, None)
    };
    let mut tampered = line(2, 1);
    tampered.kind.act = act(2);
    for refused in [line(99, 1), tampered] {
        let answer = carry(&refused).await.unwrap();
        assert_eq!(answer.status(), 422, "{}", refused.actor);
        let said: Value = answer.json().await.unwrap();
        assert_eq!(said["kind"], "not_signed", "{said}");
        assert!(
            said["error"]
                .as_str()
                .unwrap()
                .starts_with("This entry was not signed by anyone in"),
            "{said}"
        );
    }
    let held = || {
        let a = ana.rail().admission(&group).unwrap();
        let ops = svrngs_core::tests_support::wire_ops(&a);
        ops.into_iter()
            .filter(|o| o.payload.is_some())
            .collect::<Vec<_>>()
    };
    // Every line but the keeper's cutover, which ana's computer, keeping
    // the group, writes once on a round of its own (capability-signed).
    let all = || {
        let (mut ops, _) = ana
            .rail()
            .ops_missing_from_within(&group, &Default::default(), usize::MAX)
            .unwrap();
        ops.retain(|o| !matches!(o.kind.act, svrngs_core::RailAct::Signers { .. }));
        ops
    };
    let anas = || {
        all()
            .iter()
            .filter(|o| o.actor == actor_of(&signing(1)))
            .count()
    };
    assert!(held().is_empty(), "neither is held");
    let before = (all().len(), anas());

    let own = line(2, 1);
    let answer = carry(&own).await.unwrap();
    assert_eq!(answer.status(), 200);
    let said: Value = answer.json().await.unwrap();
    assert_eq!(
        (said["id"].as_str(), said["counted"].as_bool()),
        (Some(own.id.to_string().as_str()), Some(true)),
        "{said}"
    );
    let ops = held();
    assert_eq!(
        (all().len(), anas()),
        (before.0 + 1, before.1),
        "ana's computer wrote no line of its own"
    );
    assert_eq!(
        (ops.len(), ops[0].actor.as_str(), ops[0].person.as_str()),
        (1, own.actor.as_str(), "ben")
    );
    let stored = all().into_iter().find(|o| o.id == own.id).unwrap();
    assert_eq!(
        serde_json::to_string(&stored).unwrap(),
        serde_json::to_string(&own).unwrap(),
        "carried unchanged"
    );
}

/// **A phone's act in an app is signed on the phone, and any computer
/// carries it as that phone's person's** (capability-signed; `demo/runs/
/// authorship/2026-10-24/DESIGN.md` §3, §4): the holder Ben's phone keeps
/// for Money, its pass on each line, carries a line to Ana's computer by
/// `/carry` on the holders' protocol, which any key may speak. Kept: the
/// line is stored unchanged and counts, its actor the holder and its person
/// Ben. Refused, `not_signed`, and not held: a holder line under a pass a
/// stranger signed, and one under Ben's pass with its person rewritten to
/// Ana's key.
#[tokio::test]
async fn a_phone_s_line_is_carried_and_counts_as_its_person() {
    use svrngs_core::pass::{Label, Pass};
    let (nodes, _dirs) = computers().await;
    let (ana, ben) = (&nodes[0], &nodes[1]);
    let group = found("house", 11, ana, &[(ben, 2)]);
    let holder = signing(9);
    let bridge = common::bridge_as(&holder, ana, svrngs_core::alpn::Alpn::Lend).await;
    let carry = |line: &svrngs_core::Op<svrngs_core::SignedOp>| {
        let at = format!("http://{}/carry", bridge.local_addr());
        http().post(at).json(&json!([group, line])).send()
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 1;
    let app = Label::carried(&signing(1).verifying_key().to_bytes(), "Money", &group);
    let pass = |by: u8| {
        Pass::sign(
            &signing(by),
            &actor_of(&holder),
            (app.clone(), vec![]),
            now + 600,
        )
    };
    let act = json!({ "op": "record", "payload": { "kind": "Money/expense", "n": 1 } });
    let line = |pass: Pass| {
        let act = RailAct::from_json(act.clone()).unwrap();
        svrngs_core::sign_act(&holder, &group, now, 0, None, act, None, None, Some(pass))
    };
    let mut renamed = pass(2);
    renamed.person = actor_of(&signing(1));
    for refused in [line(pass(99)), line(renamed)] {
        let answer = carry(&refused).await.unwrap();
        assert_eq!(answer.status(), 422);
        let said: Value = answer.json().await.unwrap();
        assert_eq!(said["kind"], "not_signed", "{said}");
    }
    let own = line(pass(2));
    let answer = carry(&own).await.unwrap();
    assert_eq!(answer.status(), 200);
    let said: Value = answer.json().await.unwrap();
    assert_eq!(said["counted"], true, "{said}");
    let a = ana.rail().admission(&group).unwrap();
    let ops: Vec<_> = (svrngs_core::tests_support::wire_ops(&a).into_iter())
        .filter(|o| o.payload.is_some())
        .collect();
    assert_eq!(ops.len(), 1, "only Ben's phone's own line is held: {ops:?}");
    assert_eq!(
        (ops[0].actor.as_str(), ops[0].person.as_str()),
        (own.actor.as_str(), "ben")
    );
}

/// **A member's home page opens this computer's own copy, and another
/// computer never writes as them** (capability-signed; DESIGN.md §7 beside
/// `demo/runs/authorship/2026-10-24/`, the case the scenes found, c7452b2c):
/// Ana carries Money and it reaches Cy's computer, both on. Cy's list names
/// both copies, Cy's own first, so the page's one copy (question.js `once`)
/// is Cy's, whose acts Cy's computer signs. Refused: a write from Cy's
/// computer reaching Ana's copy, which Ana's computer would sign
/// `on_behalf_of` Cy: 403, "Open Money from your home page to write in
/// it.", and nothing written; Ana's copy still answers Cy's reads. Kept:
/// Cy's write on Cy's own copy is Cy's. Planted: the list sorted by person
/// alone, as before, and Ana's copy is first.
#[tokio::test]
async fn a_member_writes_on_their_own_computer_s_copy_alone() {
    let (nodes, _dirs) = computers().await;
    let (ana, ben, cy) = (&nodes[0], &nodes[1], &nodes[2]);
    let group = found("house", 12, ana, &[(ben, 2), (cy, 3)]);
    for node in [cy, ben, ana] {
        round(node).await;
    }
    let roots: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    let at_ana = synced_in(ana, roots[0].path(), &group).await;
    let at_cy = synced_in(cy, roots[1].path(), &group).await;
    carry(&at_ana, "Money");
    let script = at_cy.join("apps/Money/app.js");
    assert!(
        until(&[ana, cy], || script.exists()).await,
        "Money reaches Cy's folder"
    );
    let mut listed = lends(cy, &group).await;
    for _ in 0..30 {
        if of_kind(&listed, "Money").len() == 2 {
            break;
        }
        round(cy).await;
        listed = lends(cy, &group).await;
    }
    let money = of_kind(&listed, "Money");
    let keys: Vec<_> = money.iter().map(|m| m["key"].as_str().unwrap()).collect();
    let (anas, cys) = (actor_of(&signing(1)), actor_of(&signing(3)));
    assert_eq!(
        keys,
        [cys.as_str(), anas.as_str()],
        "Cy's own copy first:\n{listed:#}"
    );

    let expense = json!({ "op": { "kind": "expense", "amount_cents": 1 } });
    let theirs = money[1]["origin"].as_str().unwrap();
    let wrote = at(cy, theirs, "POST", "/svrngs/acts", Some(expense.clone())).await;
    assert_eq!(wrote.status(), 403, "Ana's computer writes nothing as Cy");
    let said: Value = wrote.json().await.unwrap();
    assert_eq!(
        said["error"],
        "Open Money from your home page to write in it."
    );
    let read = at(cy, theirs, "GET", "/svrngs/acts", None).await;
    assert_eq!(read.status(), 200, "Ana's copy still answers Cy's reads");
    let ops = read.json::<Value>().await.unwrap()["ops"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(ops, 0, "nothing written");

    let own = money[0]["origin"].as_str().unwrap();
    let wrote = at(cy, own, "POST", "/svrngs/acts", Some(expense)).await;
    assert!(wrote.status().is_success(), "Cy writes on Cy's own copy");
    let op: Value = wrote.json().await.unwrap();
    assert_eq!(op["actor"], cys.as_str(), "{op:#}");
}

/// **A phone's write in an app is the phone's own: the computer it reaches
/// writes nothing as its person** (capability-signed; `demo/runs/
/// authorship/2026-10-24/DESIGN.md` §3, §5): Bo stands in house from a
/// phone, and Bo's holder, under Bo's pass for Money, posts an expense to
/// Money's route on Ana's computer, as a phone's worker did before it
/// signed its own lines. Refused, 403, and nothing written: Ana's computer
/// would sign it `on_behalf_of` Bo, which no copy can tell from a forgery;
/// Bo's phone signs its own and carries it (`/carry`). Kept: the holder's
/// read of the route.
#[tokio::test]
async fn a_computer_writes_nothing_as_a_phone_s_person() {
    use svrngs_core::pass::{Label, Pass};
    use svrngs_node::{Device, Person, Redemption};
    let (nodes, _dirs) = computers().await;
    let ana = &nodes[0];
    let group = found("house", 13, ana, &[]);
    let append = |act| {
        let fixture = svrngs_core::tests_support::Fixture { act };
        ana.rail()
            .commit(&group, &fixture)
            .unwrap()
            .unwrap()
            .lines()[0]
            .clone()
    };
    let link = signing(8);
    let invite = append(RailAct::Invite {
        key: actor_of(&link),
        until: i64::MAX,
        device_of: None,
    });
    let bo = Person::parse("bo").unwrap();
    let redeemed = Redemption::sign(&signing(7), &group, &invite.id, &bo, Device::Browser);
    append(RailAct::Admit {
        invite: Some(redeemed),
        person: bo,
        key: actor_of(&signing(7)),
    });
    let root = tempfile::tempdir().unwrap();
    let folder = synced_in(ana, root.path(), &group).await;
    carry(&folder, "Money");
    let mut origin = None;
    for _ in 0..30 {
        let seen = lends(ana, &group).await;
        origin = (of_kind(&seen, "Money").first()).map(|l| l["origin"].to_string());
        if origin.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let origin = origin.expect("Money is listed");
    let label = origin.trim_matches('"').trim_start_matches("http://");
    let label = label.split('.').next().unwrap();
    let holder = signing(9);
    let until = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 600;
    let app = (Label::parse(label).unwrap(), vec![]);
    let pass = Pass::sign(&signing(7), &actor_of(&holder), app, until).encode();
    let bridge = common::bridge_as(&holder, ana, svrngs_core::alpn::Alpn::Lend).await;
    let route = format!("http://{}/lend/{label}/svrngs/acts", bridge.local_addr());
    let expense = json!({ "op": { "kind": "expense", "amount_cents": 1 } });
    let wrote = (http().post(&route).header("x-svrngs-pass", &pass))
        .json(&expense)
        .send()
        .await
        .unwrap();
    assert_eq!(wrote.status(), 403, "{:?}", wrote.text().await);
    let read = http()
        .get(&route)
        .header("x-svrngs-pass", &pass)
        .send()
        .await;
    let read = read.unwrap();
    assert_eq!(read.status(), 200, "Bo's phone still reads Money");
    let read: Value = read.json().await.unwrap();
    assert_eq!(live(&read), Vec::<Value>::new(), "nothing written as bo");
}
