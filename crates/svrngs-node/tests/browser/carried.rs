//! An app the group carries, in a browser (v1ui-shelf (a); V1_UI_BUILD
//! §7, 3.4's bars): its page loads nothing from elsewhere on a computer,
//! and on a phone it opens and writes through another person's computer
//! with the computer that brought it stopped.

use super::*;

/// `files`, each a path under `apps/<app>/` and its bytes, written into
/// `folder`, a synced folder of house's: the app carried, as the shelf
/// brings one.
pub(super) fn carry(folder: &Path, app: &str, files: &[(&str, String)]) {
    let dir = folder.join("apps").join(app);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, bytes) in files {
        std::fs::write(dir.join(path), bytes).unwrap();
    }
}

/// Where `at`'s browser opens the app `app` house carries, once its
/// folder has taken the app and `at` lists it, within 30 s.
pub(super) async fn carried_at(at: &Node, app: &str) -> String {
    carried_in(at, app, NS).await
}

/// [`carried_at`], the app carried by `group`.
pub(super) async fn carried_in(at: &Node, app: &str, group: &str) -> String {
    let url = format!("http://{}/v1/lends?namespace={group}", at.api_addr());
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    for _ in 0..60 {
        let lends: Value = (http.get(&url).bearer_auth(at.token()).send().await)
            .unwrap()
            .json()
            .await
            .unwrap();
        let listed = lends["lent"].as_array().into_iter().flatten();
        if let Some(l) = listed.into_iter().find(|l| l["kind"] == app) {
            return l["origin"].as_str().unwrap().to_string();
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("{app} is never listed on the computer");
}

in_chromium!(
    /// **A carried page reaches no other host** (V1_UI_BUILD §7, 3.4's
    /// third bar; COMMUNITY_APPS §6.2): Probe, carried in house, opened at
    /// its origin on ada's computer, asks a listener on 127.0.0.1 by a
    /// fetch, an image, a script, a style and a socket, and the listener
    /// is reached by none. Planted: the floor's headers dropped, and every
    /// one of the five reaches it — the instrument seeing what it judges
    /// (AGENTS.md 7).
    a_carried_page_reaches_no_other_host
);
async fn a_carried_page_reaches_no_other_host(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let (elsewhere, reached) = kept_page("elsewhere").await;
    let probe = format!(
        r#"const at = 'http://{elsewhere}';
const timed = (p) => Promise.race([p, new Promise((done) => setTimeout(() => done('no answer in 5 s'), 5000))]);
const loaded = (tag, attrs) => new Promise((done) => {{
  const e = Object.assign(document.createElement(tag), attrs);
  e.onload = () => done(`${{tag}} loaded`);
  e.onerror = () => done(`${{tag}} refused`);
  document.head.append(e);
}});
const socket = () => new Promise((done) => {{
  try {{
    const s = new WebSocket('ws://{elsewhere}/socket');
    s.onopen = () => done('socket opened');
    s.onerror = () => done('socket refused');
  }} catch (e) {{
    done(`socket refused: ${{e}}`);
  }}
}});
Promise.all([
  fetch(`${{at}}/fetch`).then(() => 'fetch answered', (e) => `fetch refused: ${{e}}`),
  loaded('img', {{ src: `${{at}}/image` }}),
  loaded('script', {{ src: `${{at}}/script` }}),
  loaded('link', {{ rel: 'stylesheet', href: `${{at}}/style` }}),
  socket(),
].map(timed)).then((said) => {{
  document.body.textContent = said.join('\n');
  document.title = 'tried';
}});
"#
    );
    let page = "<!doctype html><title>trying</title><body><script src=probe.js></script>";
    carry(
        &folder,
        "Probe",
        &[("index.html", page.into()), ("probe.js", probe)],
    );
    let origin = carried_at(&ada, "Probe").await;
    let seen = drive(
        engine,
        "carried.mjs",
        json!({ "url": origin, "title": "tried" }),
        None,
    )
    .await;
    eprintln!("the page's run: {seen:#}");
    assert_eq!(seen["title"], "tried", "{seen:#}");
    let reached = reached.lock().unwrap().clone();
    let firsts: Vec<_> = reached.iter().filter_map(|h| h.lines().next()).collect();
    assert!(
        firsts.is_empty(),
        "the page reached another host {} times: {firsts:?}\n{seen:#}",
        firsts.len()
    );
    drop(ada);
}

in_webkit!(
    /// **An app one computer brought opens and writes on a phone through
    /// another's, with the first stopped** (V1_UI_BUILD §7, 3.4's first
    /// bar, the phone's half): ada's computer made bo's link and brings
    /// Money into house's folder; it reaches cy's. Bo's phone opens the
    /// group while ada's answers; ada's stops; the phone reaches the group
    /// through cy's, lists Money, opens it, and the page writes an expense
    /// and reads it back as bo's, within the 3.3 bar. Planted: a carried
    /// app served only by whoever brought it — the phone lists no Money.
    an_app_one_computer_brought_opens_and_writes_on_a_phone_through_another_s
);
async fn an_app_one_computer_brought_opens_and_writes_on_a_phone_through_another_s(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let ada = node(dirs[0].path(), &relay, &bo).await;
    let cy = joined(dirs[1].path(), &relay, &ada, 2, "cy").await;
    let (at_ada, at_cy) = (dirs[0].path().join("house"), dirs[1].path().join("house"));
    for (computer, folder) in [(&ada, &at_ada), (&cy, &at_cy)] {
        std::fs::create_dir(folder).unwrap();
        keep_folder(computer, folder).await;
    }
    let script = "const acts = 'svrngs/acts';
const wrote = await fetch(acts, { method: 'POST', headers: { 'content-type': 'application/json' },
  body: JSON.stringify({ op: { kind: 'expense', amount_cents: 1200, description: 'the bins' } }) });
const read = wrote.ok ? await (await fetch(acts)).json() : {};
const ops = read.ops ?? [];
document.title = `Money: ${ops.length} by ${ops.map((o) => o.on_behalf_of ?? o.person).join(', ')}`;
";
    let page = "<!doctype html><title>Money</title><script type=module src=money.js></script>";
    carry(
        &at_ada,
        "Money",
        &[("index.html", page.into()), ("money.js", script.into())],
    );
    let held = || at_cy.join("apps/Money/money.js").exists();
    for _ in 0..60 {
        if held() {
            break;
        }
        for computer in [&ada, &cy] {
            let _ = tokio::time::timeout(Duration::from_secs(10), computer.round()).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    assert!(held(), "Money reaches cy's folder");
    carried_at(&cy, "Money").await;
    let ready = dirs[2].path().join("ready");
    let (go, release) = tokio::sync::oneshot::channel();
    let title = "Money: 1 by bo";
    let args = json!({
        "site": site,
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "ready": ready,
        "open": "Money",
        "title": title,
    });
    let start = load::load().await;
    let phone = tokio::spawn(drive(engine, "reach.mjs", args, Some(release)));
    tokio::time::timeout(Duration::from_secs(120), async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("the phone opens the group while ada's computer answers");
    ada.endpoint().close().await;
    drop(ada);
    go.send(()).unwrap();
    let seen = phone.await.unwrap();
    eprintln!("the phone's run: {seen:#}");
    let listed = &seen["listed"];
    assert_eq!(
        listed["status"], "You’re in house, as bo, through cy’s computer",
        "{seen:#}"
    );
    // The index draws Money by its own name and as the group's, never by
    // its lend name or as one person's (v1ui-polish (b)), its open naming
    // it, with what it reads and writes beside it (capability-open; the
    // operator, 2026-10-26).
    assert_eq!(listed["lent"], json!(["Money"]), "{seen:#}");
    let text = listed["text"].as_str().unwrap_or_default();
    // Independent exact copy lives in the fast question contract. This check
    // establishes that the real phone renders the shared formatter's result.
    let consent = in_question(
        "`Money ${q.agreed({group: 'g', kind: 'Money'}, [{name: 'Money', words: 'who owes whom, and for what.'}], {name: 'house'})}`",
        Value::Null,
    );
    assert!(
        text.contains(consent.as_str().expect("the formatter returns text")),
        "{seen:#}"
    );
    assert!(
        !text.contains("Money-") && !text.contains("ada’s Money"),
        "{seen:#}"
    );
    let opened = &seen["opened"];
    assert_eq!(opened["title"], title, "Money opens and writes:\n{seen:#}");
    let ms = opened["ms"].as_u64().unwrap_or(u64::MAX);
    if ms > REACHED_WITHIN {
        let missed = format!("Money opened and wrote in {ms} ms, past the bar's {REACHED_WITHIN}");
        match load::past_busy(&start, load::cores()) {
            Some(why) => could_not_judge(&format!(
                "an_app_one_computer_brought_opens_and_writes_on_a_phone_through_another_s::{}: {missed}; {why}; {start} … {}",
                engine.as_str(),
                load::load().await
            )),
            None => panic!("{missed}, at {start}:\n{seen:#}"),
        }
    }
    drop(cy);
}

in_webkit!(
    /// **The same page writes through the next computer once the first
    /// stops** (KEEP_GOING_AT_HOME 11): Money is one origin in house
    /// whichever computer lends it (svrngs-core `Label::carried`). Bo's
    /// phone opens it through ada's computer and writes; ada's stops; the
    /// same page, never loaded again, writes through cy's, and reads both
    /// entries back as bo's. Planted, watched failing: a write's line
    /// carried to the first computer alone (web/lend.js `writeSigned`), and
    /// the second write is kept on the phone, 502.
    the_same_page_writes_through_the_next_computer_once_the_first_stops
);
async fn the_same_page_writes_through_the_next_computer_once_the_first_stops(engine: Engine) {
    let relay = relay().await;
    let bo = signing(7);
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let ada = node(dirs[0].path(), &relay, &bo).await;
    let cy = joined(dirs[1].path(), &relay, &ada, 2, "cy").await;
    let (at_ada, at_cy) = (dirs[0].path().join("house"), dirs[1].path().join("house"));
    for (computer, folder) in [(&ada, &at_ada), (&cy, &at_cy)] {
        std::fs::create_dir(folder).unwrap();
        keep_folder(computer, folder).await;
    }
    let page = "<!doctype html><title>Money</title>";
    carry(&at_ada, "Money", &[("index.html", page.into())]);
    let held = || at_cy.join("apps/Money/index.html").exists();
    for _ in 0..60 {
        if held() {
            break;
        }
        for computer in [&ada, &cy] {
            let _ = tokio::time::timeout(Duration::from_secs(10), computer.round()).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    assert!(held(), "Money reaches cy's folder");
    let (one, two) = (
        carried_at(&ada, "Money").await,
        carried_at(&cy, "Money").await,
    );
    assert_eq!(
        one.split('.').next(),
        two.split('.').next(),
        "one Money label on both"
    );
    let dir = dirs[2].path();
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "open": "Money",
        "dir": dir,
        "first": "the bins",
        "second": "the milk",
    });
    let phone = tokio::spawn(drive(engine, "carried-on.mjs", args, None));
    tokio::time::timeout(Duration::from_secs(180), async {
        while !dir.join("wrote").exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("the phone opens Money through ada's computer and writes");
    ada.endpoint().close().await;
    drop(ada);
    std::fs::write(dir.join("gone"), "").unwrap();
    let seen = phone.await.unwrap();
    eprintln!("the phone's run: {seen:#}");
    assert_eq!(seen["first"]["status"], 200, "{seen:#}");
    assert_eq!(
        seen["second"]["status"], 200,
        "written through cy's:\n{seen:#}"
    );
    let read = seen["read"].as_str().unwrap_or_default();
    assert!(
        read.contains("the bins") && read.contains("the milk"),
        "{seen:#}"
    );
    assert_eq!(seen["loads"], 1, "the same page: {seen:#}");
    drop(cy);
}

in_chromium!(
    /// **A phone's line past its app's budget is let go, and its chain
    /// rewound** (EXTENSION_POINT §7, G1; web/lend.js `carried`): bo's
    /// computer already holds 200 of bo's Money entries, stamped ahead so
    /// they stay this hour's however slow the run, and both of the phone's
    /// writes are refused `limit`, 429. The second is signed where the
    /// first left off, as line 0 with one to carry, so a refused line
    /// neither waits for a next write nor leaves a hole in the chain.
    /// Planted, watched failing: the rewind taken out of `carried`, and the
    /// second is signed as line 1, two to carry; the entries stamped now and
    /// a 19 s wait before the phone opens, and its first write is taken.
    a_phone_s_line_past_the_budget_is_let_go_and_its_chain_rewound
);
async fn a_phone_s_line_past_the_budget_is_let_go_and_its_chain_rewound(engine: Engine) {
    let relay = relay().await;
    let bo = signing(7);
    let dirs: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    let ada = node(dirs[0].path(), &relay, &bo).await;
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    carry(
        &folder,
        "Money",
        &[("index.html", "<!doctype html><title>Money</title>".into())],
    );
    carried_at(&ada, "Money").await;
    // Stamped an hour ahead, as a fast clock would: `spent` (carried.rs)
    // reads a stamp ahead of the clock as now, so the budget is still spent
    // however long the phone takes to open. Stamped now, it refilled one
    // entry 18 s later, and a slow run's first write was taken.
    let ahead = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 3600;
    let mut prev = None;
    let mut lines = Vec::new();
    for n in 0..200u64 {
        let act = svrngs_core::RailAct::from_json(
            json!({ "op": "record", "payload": { "kind": "Money/expense", "n": n } }),
        )
        .unwrap();
        let line = svrngs_core::sign_act(&bo, NS, ahead, n, prev.take(), act, None, None, None);
        prev = Some(line.id.clone());
        lines.push(line);
    }
    ada.rail().ingest_all(NS, &lines).unwrap();
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "open": "Money",
    });
    let seen = drive(engine, "refused-line.mjs", args, None).await;
    eprintln!("the phone's run: {seen:#}");
    for write in [&seen["first"], &seen["second"]] {
        assert_eq!(
            (write["status"].clone(), write["refused"].clone()),
            (json!(429), json!("limit")),
            "{seen:#}"
        );
    }
    let trace = seen["trace"].as_str().unwrap_or_default();
    let signed = trace
        .lines()
        .filter(|l| l.contains("signed as line 0, 1 to carry"))
        .count();
    assert_eq!(
        signed, 2,
        "both writes signed at the same place, the refused line let go:\n{trace}"
    );
    drop(ada);
}

/// The files of the app in `apps/<dir>/` its page needs: its page, its
/// script and its fold.
fn files_of<'a>(dir: &str, fold: &'a str) -> Vec<(&'a str, String)> {
    let read = |f: &str| std::fs::read_to_string(repo().join("apps").join(dir).join(f));
    let read = |f: &'a str| (f, read(f).unwrap_or_default());
    ["index.html", "index.js", fold].map(read).to_vec()
}

in_chromium!(
    /// **Money and Events, static pages on the route, their folds
    /// unchanged** (v1ui-shelf (b)): each carried in house from its files
    /// in `apps/`, opened at its origin on ada's computer. On Money, ada
    /// adds the gas bill and it is listed as hers; on Events, ada posts a
    /// party, says she's coming, and it says so. Red before the pages:
    /// `apps/expenses/` and `apps/events/` hold no page.
    money_and_events_open_and_write_at_their_origin
);
async fn money_and_events_open_and_write_at_their_origin(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    carry(&folder, "Money", &files_of("expenses", "expenses.js"));
    carry(&folder, "Events", &files_of("events", "events.js"));

    let money = carried_at(&ada, "Money").await;
    let steps = json!([["#what", "the gas bill"], ["#amount", "42"], ["#add"]]);
    let seen = drive(
        engine,
        "carried.mjs",
        json!({ "url": money, "do": steps }),
        None,
    )
    .await;
    eprintln!("Money's run: {seen:#}");
    let after = seen["after"][0].as_str().unwrap_or_default();
    assert!(
        after.contains("You paid $42.00 for the gas bill, split between ada."),
        "{seen:#}"
    );

    let events = carried_at(&ada, "Events").await;
    let steps = json!([
        ["#what", "a party"],
        ["#when", "Saturday"],
        ["#where", "Elm Street"],
        ["#post"],
        ["text=Say you’re coming"]
    ]);
    let seen = drive(
        engine,
        "carried.mjs",
        json!({ "url": events, "do": steps }),
        None,
    )
    .await;
    eprintln!("Events' run: {seen:#}");
    let posted = seen["after"][0].as_str().unwrap_or_default();
    assert!(
        posted.contains("Saturday, at Elm Street. You host it."),
        "{seen:#}"
    );
    let coming = seen["after"][1].as_str().unwrap_or_default();
    assert!(coming.contains("Coming: you."), "{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **Bring things to the table, on the home page** (v1ui-shelf (c);
    /// V1_UI_BUILD §3.4): ada's home page offers Money and Events, each
    /// unticked, so the button alone brings nothing and says to tick
    /// something first (the operator, 2026-10-26; demo/xenia/findings/
    /// after-join-reads-as-permissions.md); she ticks them, one tap brings
    /// them, the page says so, and her computer lists each as house's, the page calling the group by its
    /// name. House is a group held from its seed (`house`), so the apps land
    /// in house's own folder. Red before (c): no shelf on the page. Then the
    /// index, the page opened again, draws each by its kind's own name and
    /// as the group's, no word holding its lend name, `<kind>-<the group's
    /// id>` (v1ui-polish (b); `demo/runs/v1ui/2026-10-15/README.md` finding
    /// 4). Planted: the lend name drawn, and its first lender's, as before.
    /// House rules comes with them (canon-house): its folder `House-rules`,
    /// a kind's name, said as "House rules"; it brings canon's build in
    /// `vendor/`, and opened at its origin it asks how the house decides.
    the_home_page_brings_things_to_the_table
);
async fn the_home_page_brings_things_to_the_table(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, id) = house(dir.path(), &relay, &signing(7)).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder_in(&ada, &folder, &id).await;
    let tick = |app: &str| json!([format!("#home label:has-text('{app}') input"), true]);
    let bring = json!([
        ["text=Bring the ticked ones to house", null, true],
        tick("Money"),
        tick("Events"),
        tick("House rules"),
        ["text=Bring the ticked ones to house", null, true]
    ]);
    let args = json!({ "all": "#home section h3 ~ p", "do": bring });
    let seen = home_page(engine, &ada, args).await;
    eprintln!("the home page's run: {seen:#}");
    let before = seen["all"].to_string();
    assert!(
        before.contains("Money: who owes whom, and for what."),
        "{seen:#}"
    );
    assert!(
        before.contains("Events: the get-togethers people post"),
        "{seen:#}"
    );
    assert!(
        before.contains("House rules: the rules your house agrees"),
        "{seen:#}"
    );
    let none = seen["after"][0].to_string();
    assert!(
        none.contains("Tick something to bring first."),
        "unticked, the button brings nothing: {seen:#}"
    );
    let after = seen["after"][1].to_string();
    assert!(
        after.contains("You brought Money to house: everyone in it can open it in a few seconds.")
            && after.contains("You brought Events to house")
            && after.contains("You brought House rules to house"),
        "{seen:#}"
    );
    for app in ["Money", "Events", "House-rules"] {
        carried_in(&ada, app, &id).await;
        assert!(folder.join("apps").join(app).join("look.css").exists());
    }
    let canon = folder.join("apps/House-rules/vendor/canon.js");
    assert_eq!(
        std::fs::read(canon).unwrap(),
        std::fs::read(repo().join("apps/canon/vendor/canon.js")).unwrap(),
        "House rules brings canon's build as vendored"
    );
    let rules = carried_in(&ada, "House-rules", &id).await;
    let opened = drive(
        engine,
        "carried.mjs",
        json!({ "url": rules, "do": [] }),
        None,
    )
    .await;
    let said = opened["said"].as_str().unwrap_or_default();
    assert!(
        said.contains("How does your house decide its rules?"),
        "House rules runs at its origin under the carried policy: {opened:#}"
    );
    // The group's things lead its section, right under its heading (the
    // operator, 2026-10-26: Money found first).
    let index = home_page(
        engine,
        &ada,
        json!({ "all": "#home section h2 + table.index tr" }),
    )
    .await;
    let rows = index["all"].to_string();
    for (app, does) in [
        ("Money", "Who owes"),
        ("Events", "The get-togethers"),
        ("House rules", "The rules"),
    ] {
        let open = format!("{app} {does}");
        assert!(
            rows.contains(&open),
            "{app} by its name, what it does beside it: {index:#}"
        );
    }
    assert_eq!(
        rows.matches(".houseIn a tab").count(),
        3,
        "each the group's: {index:#}"
    );
    assert!(
        !rows.contains(&format!("-{id}")) && !rows.contains("ada"),
        "{index:#}"
    );
    drop(ada);
}

in_webkit!(
    /// **Bring things to the table, on a phone** (v1ui-shelf (c)): bo's
    /// phone, in house through ada's computer, finds the shelf's boxes
    /// unticked (the operator, 2026-10-26), ticks them, and brings the
    /// shelf's two apps by the one button; the page says so, and ada's computer lists each
    /// as house's, in house's own folder: house is a group held from its
    /// seed (`house`). Red before (c): the page has no shelf.
    the_phone_brings_things_to_the_table
);
async fn the_phone_brings_things_to_the_table(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let bo = signing(7);
    let (ada, id) = house(dir.path(), &relay, &bo).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder_in(&ada, &folder, &id).await;
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "id": id,
    });
    let seen = drive(engine, "shelf.mjs", args, None).await;
    eprintln!("the phone's run: {seen:#}");
    assert_eq!(
        seen["ticked"], 0,
        "the shelf's boxes start unticked: {seen:#}"
    );
    let after = seen["after"].as_str().unwrap_or_default();
    assert!(
        after.contains("You brought Money to house: everyone in it can open it in a few seconds.")
            && after.contains("You brought Events to house"),
        "{seen:#}"
    );
    for app in ["Money", "Events"] {
        carried_in(&ada, app, &id).await;
    }
    drop(ada);
}

in_both_engines!(
    /// **A new version runs as each person only after their own yes**
    /// (canon-pin (c); svrn-docs `rings/current/WHAT_RUNS_AS_YOU.md` §3):
    /// ada's computer brings Money into house's folder and it reaches
    /// kim's; kim changes Money's script, and the change reaches ada's.
    /// Ada's home page says kim changed Money's files, and asks; Money,
    /// opened at its origin on ada's computer, runs as it was brought. Ada
    /// says yes on the page, and Money runs as kim changed it. Then bo's
    /// phone, through ada's computer: its page asks too, Money runs as it
    /// was brought for bo though ada said yes, and as kim changed it once
    /// bo says yes. Planted: the new version run before any yes — `file`
    /// serving the layout's files now, as at canon-pin (a).
    a_new_version_runs_as_each_person_only_after_their_yes
);
async fn a_new_version_runs_as_each_person_only_after_their_yes(engine: Engine) {
    let relay = relay().await;
    let bo = signing(7);
    let dirs: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    let ada = seeded(dirs[0].path(), relay_url(&relay), 1, "ada", 1).await;
    from_a_phone(&ada, "bo", 7);
    tokio::time::timeout(Duration::from_secs(20), ada.endpoint().online())
        .await
        .unwrap();
    let kim = joined(dirs[1].path(), &relay, &ada, 2, "kim").await;
    let (at_ada, at_kim) = (dirs[0].path().join("house"), dirs[1].path().join("house"));
    for (computer, folder) in [(&ada, &at_ada), (&kim, &at_kim)] {
        std::fs::create_dir(folder).unwrap();
        keep_folder(computer, folder).await;
    }
    let page = "<!doctype html><title>Money</title><script type=module src=money.js></script>";
    let (brought, changed) = ("Money as brought", "Money as kim changed it");
    let script = |title: &str| format!("document.title = '{title}';\n");
    carry(
        &at_ada,
        "Money",
        &[("index.html", page.into()), ("money.js", script(brought))],
    );
    let holds = |folder: &Path, title: &str| {
        std::fs::read_to_string(folder.join("apps/Money/money.js"))
            .is_ok_and(|s| s == script(title))
    };
    let rounds = || async {
        for computer in [&ada, &kim] {
            let _ = tokio::time::timeout(Duration::from_secs(10), computer.round()).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    };
    for _ in 0..60 {
        if holds(&at_kim, brought) {
            break;
        }
        rounds().await;
    }
    assert!(holds(&at_kim, brought), "Money reaches kim's folder");
    std::fs::write(at_kim.join("apps/Money/money.js"), script(changed)).unwrap();
    for _ in 0..60 {
        if holds(&at_ada, changed) {
            break;
        }
        rounds().await;
    }
    assert!(holds(&at_ada, changed), "kim's change reaches ada's folder");

    // Ada's own copy, opened at its origin on her computer.
    let url = format!("http://{}/v1/lends?namespace={NS}", ada.api_addr());
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let lends: Value = (http.get(&url).bearer_auth(ada.token()).send().await)
        .unwrap()
        .json()
        .await
        .unwrap();
    let own = |l: &&Value| l["kind"] == "Money" && l["key"] == actor_of(&signing(1));
    let lent = lends["lent"].as_array().unwrap();
    let money = lent.iter().find(own).expect("ada's Money")["origin"]
        .as_str()
        .unwrap()
        .to_string();
    let title = |seen: &Value| seen["title"].as_str().unwrap_or_default().to_string();
    let open = || drive(engine, "carried.mjs", json!({ "url": money }), None);

    let yes = json!([["text=Open the new version", null, true]]);
    let args = json!({ "all": "#acts summary, #acts p", "do": yes, "unfold": true });
    let before = open().await;
    let seen = home_page(engine, &ada, args).await;
    let after = open().await;
    eprintln!("ada's home page: {seen:#}\nMoney before: {before:#}\nMoney after: {after:#}");
    let asked = seen["all"].to_string();
    assert!(
        asked.contains("1 app changed. See what’s new")
            && asked.contains("Money: kim changed its files on"),
        "{seen:#}"
    );
    assert_eq!(
        title(&before),
        brought,
        "Money runs as brought until ada says yes"
    );
    let said = seen["after"][0].to_string();
    assert!(
        said.contains("Money opens at its new version for you now."),
        "{seen:#}"
    );
    assert_eq!(
        title(&after),
        changed,
        "and as kim changed it once she does"
    );

    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "open": "Money",
        "id": NS,
    });
    let phone = drive(engine, "runs.mjs", args, None).await;
    eprintln!("bo's phone: {phone:#}");
    let asked = phone["asked"].as_str().unwrap_or_default();
    assert!(
        asked.contains("Money: kim changed its files on"),
        "{phone:#}"
    );
    assert_eq!(
        phone["before"], brought,
        "bo runs Money as brought, whatever ada said: {phone:#}"
    );
    let said = phone["said"].as_str().unwrap_or_default();
    assert!(
        said.contains("Money opens at its new version for you now."),
        "{phone:#}"
    );
    assert_eq!(phone["after"], changed, "{phone:#}");
    drop((ada, kim));
}

in_webkit!(
    /// **The phone goes on to a computer that answers** (canon-fix (c),
    /// #205; demo/runs/canon/2026-10-17-house, take 2's step 7-dee): bo's
    /// phone holds cy's link and opens house through cy's computer, which
    /// lists ada's pad; ada's computer carries House rules. Cy leaves. Her
    /// computer still runs svrngs and answers the dial, but its
    /// `/people/lends` fails; ada's is on. Opened once more, the page lists
    /// House rules through ada's computer, with the page not opened again.
    /// The run met it as a race, so here cy's computer answers first: ada's
    /// relay is reached through a way that holds each new connection 2 s.
    /// Red at canon-fix (a): "This browser could not open the page." (web/
    /// app.js `failed`). Planted: the first computer's failure ending the
    /// reach, as before (d).
    the_phone_goes_on_to_a_computer_that_answers
);
async fn the_phone_goes_on_to_a_computer_that_answers(engine: Engine) {
    use svrngs_node::removal::{LeaveAsk, Outcome, LEAVES_ROUTE};
    let (near, far) = (relay().await, relay().await);
    let bo = signing(7);
    let dirs: Vec<_> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let slow = slow_way(&far, Duration::from_secs(2)).await;
    let ada = computer_on(dirs[0].path(), slow, &bo, 1, "ada").await;
    let cy = joined(dirs[1].path(), &near, &ada, 2, "cy").await;
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let canon = repo().join("apps/canon");
    let rules = folder.join("apps/House-rules");
    std::fs::create_dir_all(rules.join("vendor")).unwrap();
    for f in [
        "index.html",
        "index.js",
        "house.js",
        "vendor/call.mjs",
        "vendor/canon.js",
    ] {
        std::fs::copy(canon.join(f), rules.join(f)).unwrap();
    }
    carried_at(&ada, "House-rules").await;
    lend(
        dirs[0].path(),
        "pad",
        page_origin("<!doctype html>").await.port(),
    );
    // Cy's computer lists ada's pad, so the phone keeps ada's computer to
    // reach the group through.
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let ada_key = actor_of(&signing(1));
    let mut listed = false;
    for _ in 0..60 {
        let url = format!("http://{}/v1/lends?namespace={NS}", cy.api_addr());
        let asked = http.get(url).bearer_auth(cy.token()).send().await;
        let lends: Value = asked.unwrap().json().await.unwrap();
        let mut lent = lends["lent"].as_array().into_iter().flatten();
        listed = lent.any(|l| l["key"] == ada_key);
        if listed {
            break;
        }
        for computer in [&ada, &cy] {
            let _ = tokio::time::timeout(Duration::from_secs(10), computer.round()).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    assert!(listed, "cy's computer lists ada's lends");
    let ready = dirs[2].path().join("ready");
    let (go, release) = tokio::sync::oneshot::channel();
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(2), &near),
        "ready": ready,
    });
    let phone = tokio::spawn(drive(engine, "reach.mjs", args, Some(release)));
    tokio::time::timeout(Duration::from_secs(120), async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("the phone opens the group through cy's computer");
    let leave = http
        .post(format!("http://{}{LEAVES_ROUTE}", cy.api_addr()))
        .bearer_auth(cy.token());
    let left = leave.json(&LeaveAsk { group: NS.into() }).send().await;
    assert_eq!(
        left.unwrap().json::<Outcome>().await.unwrap(),
        Outcome::Left
    );
    let cy_key = actor_of(&signing(2));
    let mut out = false;
    for _ in 0..60 {
        out = ada.rail().standing_anywhere(&cy_key).is_none();
        if out {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    assert!(out, "cy's leave reaches ada's computer");
    go.send(()).unwrap();
    let seen = phone.await.unwrap();
    eprintln!("the phone's run: {seen:#}");
    assert_eq!(
        seen["first"]["status"], "You’re in house, as bo",
        "{seen:#}"
    );
    let listed = &seen["listed"];
    assert_eq!(
        listed["status"], "You’re in house, as bo, through ada’s computer",
        "{seen:#}"
    );
    assert!(
        listed["lent"].as_array().is_some_and(|l| l
            .iter()
            .any(|b| b.as_str().is_some_and(|b| b == "House rules"))),
        "{seen:#}"
    );
    drop((ada, cy));
}

in_both_engines!(
    /// **An app's entry is approved by the group's signer and carried, the
    /// whole flow** (svrn-docs rings/current/HOLDER_APPROVAL.md §3.D, §6):
    /// on the group's page at its TLS name, bo opens Money, which ada's
    /// computer carries; the page seals the open's approver into the open
    /// and leaves its key in a cookie, and the group's signer, framed under
    /// Money, adopts it and deletes the cookie. Money saves an entry, which
    /// the signer approves and ada's computer takes by `/carry` and counts
    /// as bo's; again after a reload, and after the browser restarts, from
    /// Money's address alone. Opened again, Money saves under the newer
    /// open's approver, and the signer refuses the first open. Notes is
    /// refused Money's open, as it is and as its own. Walled, whose page
    /// keeps frames out in its own `<meta>`, is told why it can't save (the
    /// known gap, DESIGN §3). Money's open, replayed
    /// in a browser that never opened it, says "Open this app again from
    /// your group's page." Each of the four entries counts, each approved by
    /// the approver its own pass names. Planted, watched failing: the
    /// signer's digest check taken out of `approver` (web/sign.js), and the
    /// first open is answered as issued.
    an_app_s_entry_is_approved_by_the_group_s_signer_and_carried
);
async fn an_app_s_entry_is_approved_by_the_group_s_signer_and_carried(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dirs: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    let ada = node(dirs[0].path(), &relay, &bo).await;
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let script = "window.save = async (what) => {
  const r = await fetch('svrngs/acts', { method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ op: { kind: 'expense', amount_cents: 1, description: what } }) });
  return { status: r.status, said: await r.text() };
};
document.title = 'ready';
";
    let page = "<!doctype html><title>…</title><script type=module src=app.js></script>";
    let walled = "<!doctype html><meta http-equiv=Content-Security-Policy content=\"frame-src 'none'\"><title>…</title><script type=module src=app.js></script>";
    for (app, page) in [("Money", page), ("Notes", page), ("Walled", walled)] {
        carry(
            &folder,
            app,
            &[("index.html", page.into()), ("app.js", script.into())],
        );
        carried_at(&ada, app).await;
    }
    let args = json!({
        "site": site,
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "profile": dirs[1].path(),
        "apps": ["Money", "Notes", "Walled"],
    });
    let seen = drive(engine, "approve.mjs", args, None).await;
    eprintln!("the approval run: {seen:#}");
    let saved = |s: &Value| s["status"] == 200;
    let logs = seen["logs"].to_string();
    assert!(
        logs.contains("SIGN: adopted the approver") && seen["opened"]["cookies"] == 0,
        "the signer adopts the open's approver and deletes its cookie:\n{seen:#}"
    );
    for step in [
        &seen["opened"]["saved"],
        &seen["reloaded"],
        &seen["restarted"],
        &seen["again"]["saved"],
    ] {
        assert!(saved(step), "Money saves: {step}\n{seen:#}");
    }
    let again = &seen["again"];
    let refused = json!({ "opened": false, "error": "Open this app again from your group’s page to write in it." });
    assert_eq!(
        (again["newer"]["opened"].clone(), again["newKey"].clone()),
        (json!(true), json!(true)),
        "{seen:#}"
    );
    assert_eq!(
        again["older"], refused,
        "the first open, once a newer is adopted:\n{seen:#}"
    );
    assert_eq!(
        seen["another"]["as"], refused,
        "Money's open, asked from Notes:\n{seen:#}"
    );
    assert_eq!(
        seen["another"]["relabelled"], refused,
        "Money's open as Notes':\n{seen:#}"
    );
    assert_eq!(
        seen["replayed"], "Open this app again from your group’s page.",
        "{seen:#}"
    );
    let kept_out = &seen["walled"];
    let said = kept_out["said"].as_str().unwrap_or_default();
    assert!(
        kept_out["status"] == 403 && said.contains("may keep out your group’s signer"),
        "an app whose page keeps the signer out is told why (DESIGN §3, the known gap):\n{seen:#}"
    );

    let admitted = ada.rail().admission(NS).unwrap();
    let ops = svrngs_core::tests_support::wire_ops(&admitted);
    let money: Vec<_> = ops
        .iter()
        .filter(|o| o.payload.is_some() && o.person.as_str() == "bo")
        .collect();
    let held = ada
        .rail()
        .ops_missing_from_within(NS, &Default::default(), usize::MAX)
        .unwrap()
        .0;
    let approvers: BTreeSet<_> = held
        .iter()
        .filter_map(|o| o.kind.grant.as_ref())
        .inspect(|p| {
            assert!(
                p.op.is_some() && p.approval.is_some(),
                "every entry carried is approved: {p:?}"
            )
        })
        .filter_map(|p| p.approver.clone())
        .collect();
    assert_eq!(
        (money.len(), approvers.len()),
        (4, 2),
        "four entries count as bo's, under two opens' approvers: {money:#?}"
    );
}

in_chromium!(
    /// **A font in the folder loads** (svrn-docs
    /// `rings/current/EXTENSION_POINT.md` §3: a font is a `.woff2` in the
    /// folder, which no test loaded; §7, G3): Lettered, carried in house,
    /// names `own.woff2` beside its page in an `@font-face` of its own
    /// style sheet, and the browser loads it under the policy every answer
    /// carries. `own.woff2` is one box glyph at `O`, 288 bytes, made with
    /// fontTools' FontBuilder (4.63.0) as a woff2. Planted, watched
    /// failing: the policy given `font-src 'none'`, and the page says
    /// "refused".
    a_font_in_the_folder_loads
);
async fn a_font_in_the_folder_loads(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let page = "<!doctype html><title>trying</title><link rel=stylesheet href=look.css>\
        <p style=\"font-family: Own\">O</p><script type=module src=probe.js></script>";
    let look = "@font-face { font-family: Own; src: url(own.woff2) format('woff2'); }";
    let probe = "document.fonts.load('16px Own', 'O').then(\
        (f) => f.map((x) => x.status).join() || 'none', (e) => `refused: ${e}`)\
        .then((said) => { document.body.textContent = said; document.title = 'tried'; });";
    // The font first, so the page is never listed before it is placed.
    std::fs::create_dir_all(folder.join("apps/Lettered")).unwrap();
    std::fs::write(
        folder.join("apps/Lettered/own.woff2"),
        include_bytes!("own.woff2"),
    )
    .unwrap();
    let files = [
        ("look.css", look.into()),
        ("probe.js", probe.into()),
        ("index.html", page.into()),
    ];
    carry(&folder, "Lettered", &files);
    let origin = carried_at(&ada, "Lettered").await;
    let seen = drive(
        engine,
        "carried.mjs",
        json!({ "url": origin, "title": "tried" }),
        None,
    )
    .await;
    assert_eq!(seen["said"], "loaded", "{seen:#}");
    drop(ada);
}

in_chromium!(
    /// **A folder app the policy blanks is told so on its card, before
    /// anyone opens it** (svrn-docs `rings/current/EXTENSION_POINT.md`
    /// §6.3; §7, G3): Blank, carried in house, has a script written in its
    /// page and a font from fonts.googleapis.com. Its row on ada's home
    /// page says both, with what to change, in the words `svrngs check`
    /// prints (folder/check.rs `Refused::words`), and nobody has opened it.
    /// Planted, watched failing: surface.rs `lends` sending no `check`, and
    /// the row says only what opening it lets it do.
    a_folder_app_the_policy_blanks_is_told_so_on_its_card
);
async fn a_folder_app_the_policy_blanks_is_told_so_on_its_card(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let page = "<!doctype html><title>Blank</title>\n\
        <link rel=stylesheet href=\"https://fonts.googleapis.com/css2?family=Inter\">\n\
        <script>document.body.textContent = 'ran';</script>\n";
    carry(&folder, "Blank", &[("index.html", page.into())]);
    carried_at(&ada, "Blank").await;
    let seen = home_page(engine, &ada, json!({ "all": "#home table.index td" })).await;
    let row = seen["all"][0].as_str().unwrap_or_default();
    for said in [
        "This page asks fonts.googleapis.com for a font, and apps load only their own files. That way, opening an app tells nobody outside your group you were here. To fix it, save the .woff2 beside index.html and point @font-face at it.",
        "This page has a script written inside it, and apps run scripts only from their own files. Move it to a .js file beside index.html.",
    ] {
        assert!(row.contains(said), "the card lacks {said:?}: {seen:#}");
    }
    drop(ada);
}

in_both_engines!(
    /// **A carried app that read a friend's lend writes nothing on a phone**
    /// (svrn-docs rings/current/CARRIED_APP_READS.md §5 item 5; the
    /// computer's hold is door.rs `reader`). Ada's computer carries Reader,
    /// whose reads.txt names `k`; cy lends a shelf of `k`. Bo's phone opens
    /// Reader from the group's page, the yes: it is told cy's shelf, reads
    /// it, and saves an entry. Then the yes's Stop, and Reader saves under
    /// the pass it holds, and under the next one, opened at `#open=`. The
    /// rule: none of the three is counted in ada's copy.
    a_carried_app_that_read_friends_lends_writes_on_a_phone
);
async fn a_carried_app_that_read_friends_lends_writes_on_a_phone(engine: Engine) {
    let relay = relay().await;
    let bo = signing(7);
    let dirs: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    let ada = node(dirs[0].path(), &relay, &bo).await;
    let cy = joined(dirs[1].path(), &relay, &ada, 2, "cy").await;
    let (shelf, on_shelf) = kept_page("cy's shelf, as text").await;
    lend(dirs[1].path(), "shelf", shelf.port());
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let script = "window.save = async (what) => {
  const r = await fetch('svrngs/acts', { method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ op: { kind: 'expense', amount_cents: 1, description: what } }) });
  return { status: r.status, said: await r.text() };
};
document.title = 'ready';
";
    let page = "<!doctype html><title>…</title><script type=module src=app.js></script>";
    carry(
        &folder,
        "Reader",
        &[
            ("index.html", page.into()),
            ("app.js", script.into()),
            ("reads.txt", "k\n".into()),
        ],
    );
    let origin = carried_at(&ada, "Reader").await;
    let host = origin.split("//").nth(1).unwrap_or_default();
    let reader = host.split('.').next().unwrap_or_default().to_string();
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "open": "Reader",
        "label": reader,
        "ask": "probe?n=1",
    });
    let seen = drive(engine, "reader-writes.mjs", args, None).await;
    let shown = json!({ "told": seen["told"], "read": seen["read"], "saved": seen["saved"],
        "held": seen["held"], "after": seen["after"], "stopped": seen["stopped"], "error": seen["error"] });
    eprintln!("the reader's run: {shown:#}");
    let words = [
        "SIGN",
        "RUNTIME: an act",
        "carried to",
        "reads were stopped",
        "PAGE: opening",
    ];
    for line in seen["logs"].as_array().into_iter().flatten() {
        let line = line.as_str().unwrap_or_default();
        if words.iter().any(|w| line.contains(w)) {
            eprintln!("  log: {line}");
        }
    }
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    let asked = on_shelf.lock().unwrap().len();
    eprintln!("cy's shelf was asked {asked} time(s)");
    assert_eq!(
        seen["read"]["status"], 200,
        "Reader read cy's shelf: {shown:#}"
    );
    let admitted = ada.rail().admission(NS).unwrap();
    let ops = svrngs_core::tests_support::wire_ops(&admitted);
    let bos: Vec<_> = ops
        .iter()
        .filter(|o| o.payload.is_some() && o.person.as_str() == "bo")
        .map(|o| serde_json::to_string(&o.payload).unwrap_or_default())
        .collect();
    eprintln!("bo's entries counted in ada's copy: {bos:#?}");
    let told = "An app that reads your friends’ apps can’t write in the group’s record.";
    for step in ["saved", "held", "stopped"] {
        let said = seen[step]["said"].as_str().unwrap_or_default();
        assert!(said.contains(told), "{step} was told why: {shown:#}");
    }
    assert!(
        bos.is_empty(),
        "an app that read a friend's lend wrote in the group's record: {bos:#?}\n{shown:#}"
    );
    drop((ada, cy));
}
