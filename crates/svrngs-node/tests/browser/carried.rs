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
