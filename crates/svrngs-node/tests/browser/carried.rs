//! An app the group carries, in a browser (v1ui-shelf (a); V1_UI_BUILD
//! §7, 3.4's bars): its page loads nothing from elsewhere on a computer,
//! and on a phone it opens and writes through another person's computer
//! with the computer that brought it stopped.

use super::*;

/// `files`, each a path under `apps/<app>/` and its bytes, written into
/// `folder`, a synced folder of house's: the app carried, as the shelf
/// brings one.
fn carry(folder: &Path, app: &str, files: &[(&str, String)]) {
    let dir = folder.join("apps").join(app);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, bytes) in files {
        std::fs::write(dir.join(path), bytes).unwrap();
    }
}

/// Where `at`'s browser opens the app `app` house carries, once its
/// folder has taken the app and `at` lists it, within 30 s.
async fn carried_at(at: &Node, app: &str) -> String {
    let url = format!("http://{}/v1/lends?namespace={NS}", at.api_addr());
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    for _ in 0..60 {
        let lends: Value = http.get(&url).send().await.unwrap().json().await.unwrap();
        let listed = lends["lent"].as_array().into_iter().flatten();
        if let Some(l) = listed.into_iter().find(|l| l["kind"] == app) {
            return l["origin"].as_str().unwrap().to_string();
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("{app} is never listed on the computer");
}

in_both_engines!(
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

in_both_engines!(
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
document.title = `Money: ${ops.length} by ${ops.map((o) => o.on_behalf_of).join(', ')}`;
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
    // its lend name or as one person's (v1ui-polish (b)).
    assert_eq!(listed["lent"], json!(["Money"]), "{seen:#}");
    let text = listed["text"].as_str().unwrap_or_default();
    assert!(text.contains("Money\tMoney\thouse\t"), "{seen:#}");
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

/// The files of the app in `apps/<dir>/` its page needs: its page, its
/// script and its fold.
fn files_of<'a>(dir: &str, fold: &'a str) -> Vec<(&'a str, String)> {
    let read = |f: &str| std::fs::read_to_string(repo().join("apps").join(dir).join(f));
    let read = |f: &'a str| (f, read(f).unwrap_or_default());
    ["index.html", "index.js", fold].map(read).to_vec()
}

in_both_engines!(
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

in_both_engines!(
    /// **An entry another person's computer wrote says whose** (gv-via
    /// (b); `ralph/FINDING_ON_BEHALF_OF.md`'s scene): a script on cy's
    /// computer writes to its loopback API, in bo's name, "bo paid $400 for
    /// the rent" in Money, and a dinner bo hosts with bo and Maya coming in
    /// Events. Ada's computer takes cy's record, and on each page opened at
    /// its origin there each reads as bo's, through cy's computer, in the
    /// phone's words (web/app.js). Planted: the label left out, as before
    /// gv-via, and each page says bo's alone.
    an_entry_another_s_computer_wrote_says_whose
);
async fn an_entry_another_s_computer_wrote_says_whose(engine: Engine) {
    let relay = relay().await;
    let dirs: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    let ada = node(dirs[0].path(), &relay, &signing(7)).await;
    let cy = joined(dirs[1].path(), &relay, &ada, 2, "cy").await;
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    carry(&folder, "Money", &files_of("expenses", "expenses.js"));
    carry(&folder, "Events", &files_of("events", "events.js"));
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let event = "0123456789abcdef";
    for payload in [
        json!({ "kind": "Money/expense", "amount_cents": 40000, "description": "the rent" }),
        json!({ "kind": "Events/event", "event": event, "what": "dinner", "when": "Friday 7", "where": "the house" }),
        json!({ "kind": "Events/coming", "event": event, "guests": ["Maya"] }),
    ] {
        let act = json!({ "op": "record", "payload": payload, "on_behalf_of": "bo" });
        let at = format!("http://{}/v1/rail/append?namespace={NS}", cy.api_addr());
        let wrote = http.post(at).json(&act).send().await.unwrap();
        assert!(wrote.status().is_success(), "{:?}", wrote.text().await);
    }
    let synced = ada.host().sync_with(NS, &cy.endpoint().addr()).await;
    assert!(synced.stop.is_none(), "ada takes cy's record: {synced:?}");
    for (app, says) in [
        (
            "Money",
            vec!["bo, through cy’s computer, paid $400.00 for the rent, split between bo."],
        ),
        (
            "Events",
            vec![
                "Friday 7, at the house. bo, through cy’s computer, hosts it.",
                "Coming: bo + Maya, through cy’s computer.",
            ],
        ),
    ] {
        let url = carried_at(&ada, app).await;
        let seen = drive(engine, "carried.mjs", json!({ "url": url, "do": [] }), None).await;
        eprintln!("{app}'s run: {seen:#}");
        let said = seen["said"].as_str().unwrap_or_default();
        for words in says {
            assert!(said.contains(words), "{app} says {words:?}:\n{seen:#}");
        }
    }
    drop((ada, cy));
}

in_both_engines!(
    /// **Bring things to the table, on the home page** (v1ui-shelf (c);
    /// V1_UI_BUILD §3.4): ada's home page offers Money and Events, both
    /// ticked; one tap brings them, the page says so, and her computer
    /// lists each as house's. House keeps no seed here, so the page calls
    /// it "your group". Red before (c): no shelf on the page. Then the
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
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let bring = json!([["text=Bring the ticked ones to your group", null, true]]);
    let args = json!({ "all": "#home section h3 ~ p", "do": bring });
    let seen = home_page(engine, &ada, args).await;
    eprintln!("the home page's run: {seen:#}");
    let before = seen["all"].to_string();
    assert!(
        before.contains("Money: who paid for what, and who owes whom."),
        "{seen:#}"
    );
    assert!(before.contains("Events: post a get-together"), "{seen:#}");
    assert!(
        before.contains("House rules: the rules your house agrees"),
        "{seen:#}"
    );
    let after = seen["after"][0].to_string();
    assert!(
        after.contains(
            "You brought Money to your group: everyone in it can open it in a few seconds."
        ) && after.contains("You brought Events to your group")
            && after.contains("You brought House rules to your group"),
        "{seen:#}"
    );
    for app in ["Money", "Events", "House-rules"] {
        carried_at(&ada, app).await;
        assert!(folder.join("apps").join(app).join("look.css").exists());
    }
    let canon = folder.join("apps/House-rules/vendor/canon.js");
    assert_eq!(
        std::fs::read(canon).unwrap(),
        std::fs::read(repo().join("apps/canon/vendor/canon.js")).unwrap(),
        "House rules brings canon's build as vendored"
    );
    let rules = carried_at(&ada, "House-rules").await;
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
    let index = home_page(engine, &ada, json!({ "all": "#home table.index tr" })).await;
    let rows = index["all"].to_string();
    for app in ["Money", "Events", "House rules"] {
        let drawn = format!("Open {app}{app}your groupIn a tab");
        assert!(
            rows.contains(&drawn),
            "{app} the group's, by its name: {index:#}"
        );
    }
    assert!(
        !rows.contains(&format!("-{NS}")) && !rows.contains("ada"),
        "{index:#}"
    );
    drop(ada);
}

in_both_engines!(
    /// **Bring things to the table, on a phone** (v1ui-shelf (c)): bo's
    /// phone, in house through ada's computer, brings the shelf's two apps
    /// by the one button; the page says so, and ada's computer lists each
    /// as house's. Red before (c): the page has no shelf.
    the_phone_brings_things_to_the_table
);
async fn the_phone_brings_things_to_the_table(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let bo = signing(7);
    let ada = node(dir.path(), &relay, &bo).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &relay),
        "id": NS,
    });
    let seen = drive(engine, "shelf.mjs", args, None).await;
    eprintln!("the phone's run: {seen:#}");
    let after = seen["after"].as_str().unwrap_or_default();
    assert!(
        after.contains("You brought Money to house: everyone in it can open it in a few seconds.")
            && after.contains("You brought Events to house"),
        "{seen:#}"
    );
    for app in ["Money", "Events"] {
        carried_at(&ada, app).await;
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
    let ada = node(dirs[0].path(), &relay, &bo).await;
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
    let lends: Value = http.get(&url).send().await.unwrap().json().await.unwrap();
    let own = |l: &&Value| l["kind"] == "Money" && l["key"] == actor_of(&signing(1));
    let lent = lends["lent"].as_array().unwrap();
    let money = lent.iter().find(own).expect("ada's Money")["origin"]
        .as_str()
        .unwrap()
        .to_string();
    let title = |seen: &Value| seen["title"].as_str().unwrap_or_default().to_string();
    let open = || drive(engine, "carried.mjs", json!({ "url": money }), None);

    let yes = json!([["text=Yes, use the new version of Money", null, true]]);
    let args = json!({ "all": "#home section h3 ~ p", "do": yes });
    let before = open().await;
    let seen = home_page(engine, &ada, args).await;
    let after = open().await;
    eprintln!("ada's home page: {seen:#}\nMoney before: {before:#}\nMoney after: {after:#}");
    let asked = seen["all"].to_string();
    assert!(
        asked.contains("kim changed Money’s files on") && asked.contains("Open the new version?"),
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
    assert!(asked.contains("kim changed Money’s files on"), "{phone:#}");
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
