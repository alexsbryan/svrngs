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
    let seen = drive(engine, "carried.mjs", json!({ "url": origin }), None).await;
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
const read = wrote.ok ? await (await fetch(acts)).json() : { acts: [] };
document.title = `Money: ${read.acts.length} by ${read.acts.map((a) => a.by).join(', ')}`;
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
