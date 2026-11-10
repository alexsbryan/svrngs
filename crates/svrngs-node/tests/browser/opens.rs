//! Opening an app is the yes to what it does as you (capability-open;
//! svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md` §3 shortfalls 3 and 7,
//! §7.1 and §7.3, §9 "Opening is the yes"): an app another member lends,
//! whose lender declared it reads a kind (`--reads`), opens their lends of
//! that kind as you only once you said yes, and opening it from your home
//! page, by a button naming it with what it does said beside it, is that
//! yes. Beside the open of a carried app is what it reads and writes, as the
//! shelf words it.

use super::*;

/// The first line of each request `heads` kept, its method and path.
fn asked(heads: &KeptHeads) -> Vec<String> {
    let heads = heads.lock().unwrap();
    let firsts = heads.iter().filter_map(|h| h.lines().next());
    firsts
        .map(|l| l.rsplit_once(' ').map_or(l, |(at, _)| at).to_string())
        .collect()
}

in_chromium!(
    /// **A page reaching another member's lend as you waits for your yes,
    /// and opening it from your home page is the yes** (shortfall 3). Cy
    /// lends notes, of kind `notes`, declared to read `films`; films; and
    /// a plain page reading nothing. Ada's browser opens cy's notes by its
    /// typed address, as a bookmark opens it, and its fetch of cy's films
    /// is refused, `unasked`, in words ada can act on: before this test it
    /// reached cy's films as ada, on nobody's yes but cy's `--reads`. On
    /// her home page the notes' button names them, and beside it is what
    /// opening it lets it do, "It also opens your friends’ films as you"
    /// (the operator, 2026-10-26: the button names the app); one tap opens it, answering
    /// no question, and from then its page reaches the films, in the new
    /// tab and the bookmarked one alike, at the address its `/svrngs-reads`
    /// gives, and never at the films' plain origin (give-session). Kept:
    /// the plain lend opens by its
    /// link as before, its words unchanged. Planted: `surface::reaches`
    /// carrying a reach with no yes, and the bookmarked fetch reaches the
    /// films.
    opening_an_app_from_the_home_page_is_the_yes_to_what_it_reads
);
async fn opening_an_app_from_the_home_page_is_the_yes_to_what_it_reads(engine: Engine) {
    let relay = relay().await;
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &signing(7)).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 2, "cy").await;
    let notes = page_origin("<!doctype html><title>cy's notes</title><body>").await;
    let (films, on_films) = kept_page("<!doctype html><title>cy's films</title><body>").await;
    let plain = page_origin("<!doctype html><title>cy's plain page</title><body>").await;
    let lends = svrngs_node::lend::lends_dir(cy_dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    for (name, port, kind, reads) in [
        ("notes", notes.port(), "notes", json!(["films"])),
        ("films", films.port(), "films", json!([])),
        ("plain", plain.port(), "k", json!([])),
    ] {
        let body = json!({ "groups": [NS], "port": port, "kind": kind, "limit": 3, "refused": [], "reads": reads });
        std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
    }
    let port = ada.api_addr().port();
    let at = |name: &str| format!("http://{}.localhost:{port}/", label(2, name));
    let words = "Open cy’s notesReads cy’s films. They can see it’s you, and what it asks for. cy’s films Opening it lets it write its own part of your group’s record, as you.";

    let seen = drive(
        engine,
        "opens.mjs",
        json!({
            "home": format!("http://svrngs.localhost:{port}/"),
            "page": at("notes"),
            "read": at("films"),
            "words": "cy’s notes",
            "plain": "cy’s plain (k)",
        }),
        None,
    )
    .await;
    eprintln!("the open's run: {seen:#}");

    assert_eq!(
        seen["bookmark"],
        json!({ "status": "no address", "new": [{ "kind": "films", "name": "films", "lender": "cy" }] }),
        "a page opened another way is given no address until the person's yes, and told what is new:\n{seen:#}"
    );
    assert_eq!(
        seen["unticked"],
        json!({ "line": "Reads none of your friends’ films.", "reads": "", "left": label(2, "films").to_string() }),
        "an unticked lend leaves the line and what the Open posts:\n{seen:#}"
    );
    let cells = seen["cells"].as_array().cloned().unwrap_or_default();
    for cell in [
        words,
        "Open cy’s plain (k) Opening it lets it write its own part of your group’s record, as you.",
        "Open cy’s films Opening it lets it write its own part of your group’s record, as you.",
    ] {
        assert!(
            cells.contains(&json!(cell)),
            "the index says {cell:?}:\n{seen:#}"
        );
    }
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    assert!(
        at_origin(&seen["there"]["url"], &label(2, "notes")),
        "{seen:#}"
    );
    assert!(at_origin(&seen["plain"], &label(2, "plain")), "{seen:#}");
    let on_films = asked(&on_films);
    assert!(
        !on_films.iter().any(|l| l == "GET /bookmark"),
        "the films are never reached before the yes: {on_films:?}"
    );
    assert!(
        !on_films.iter().any(|l| l == "GET /bare"),
        "the films are never reached at their plain origin, which a fetch carrying no capability asks: {on_films:?}\n{seen:#}"
    );
    for after in ["GET /opened", "GET /after"] {
        assert!(
            on_films.iter().any(|l| l == after),
            "once opened from the home page the notes reach the films, `{after}`: {on_films:?}\n{seen:#}"
        );
    }
    drop((ada, cy));
}

/// **The open names the app, and beside it is what it does, in one
/// wording for both pages** (shortfall 7; the note's §9; the operator,
/// 2026-10-26, from demo/xenia/findings/first-set/
/// after-join-reads-as-permissions.md): an app the group carries is first
/// what the shelf says it is for, or else "its own part of the group’s
/// record", then that opening it lets it read and write that as you, so its
/// first version runs as a person only once they open it so named; a lend
/// reading other kinds opens your friends' lends of them as you; one doing
/// neither may write its own part of the record as you. Both pages draw the line
/// beside the open from question.js's `agreed`, and the open by `named`,
/// each in its `page/index.js` since page-switchover. The call is read, not
/// its argument's name: js-lint (a) named the lend `lend` (75e95f3a), and
/// this test's `agreed(l, ` failed on the rename alone (js-lint-6's gate).
/// The page's whole `page/` directory is read: js-read (b) moved the
/// phone's open into web/page/lend-line.js (js-read-2's run).
/// Red at d736ff18: `agreed` said "Open " and the name before what the app
/// does, all of it on the button. Red at 5c6e3b0d: "It reads and writes"
/// first, which Ana read as done on her behalf without her yes (the fifth
/// xenia set, demo/xenia/findings/said-too-much-the-group-s-page.md).
#[test]
fn the_open_says_what_an_app_does_in_one_wording() {
    for page in ["crates/svrngs-node/src/home/page", "web/page"] {
        let text = page_source(page);
        assert!(text.contains("agreed("), "{page} draws no open by `agreed`");
    }
    let shelf = json!([{ "name": "Money", "words": "who owes whom, and for what." }]);
    let carried = |kind: &str| json!({ "group": "g", "kind": kind, "person": "cy", "name": "x", "reads": [] });
    let lends = json!([
        carried("Money"),
        carried("Quiz"),
        { "person": "cy", "name": "notes", "kind": "notes", "reads": ["films", "music"] },
        { "person": "cy", "name": "shelf", "kind": "k", "reads": [] },
    ]);
    assert_eq!(
        in_question(
            "args[0].map((l) => [q.named(l), q.agreed(l, args[1], { name: 'House' })])",
            json!([lends, shelf])
        ),
        json!([
            ["Money", "Who owes whom, and for what. Opening it lets it read and write that, as you."],
            ["Quiz", "Its own part of the group’s record. Opening it lets it read and write that, as you."],
            ["cy’s notes", "Opening it lets it write its own part of House’s record, as you. It also opens your friends’ films and music as you, and they can see what it asks."],
            ["cy’s shelf (k)", "Opening it lets it write its own part of House’s record, as you."],
        ])
    );
}

in_chromium!(
    /// **An open leads to the home page, a lend opened from it, and a
    /// bookmark of each opens it again** (give-session; WHAT_CAN_ACT_AS_YOU
    /// §9, "nothing needs a terminal"). A browser svrngs never opened is
    /// told at the home page and at a lend's origin alike to open svrngs
    /// from the computer, naming the opener in the person's words, and the
    /// lend's service is never asked. The address `svrngs open` hands the
    /// browser opens the home page; its index opens Ada's pad in a tab, and
    /// that tab opens both again by their typed addresses, as bookmarks do.
    /// Before give-session the fresh browser was answered the home page and
    /// the pad as the person. Planted: the Neither branch taken out of
    /// `only_itself`, and the fresh browser is shown the home page.
    an_open_leads_home_then_to_a_lend_and_a_bookmark_opens_each_again
);
async fn an_open_leads_home_then_to_a_lend_and_a_bookmark_opens_each_again(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let (pad, on_pad) = kept_page("<!doctype html><title>ada's pad</title><body>the pad").await;
    lend(dir.path(), "pad", pad.port());
    let port = ada.api_addr().port();
    let home = format!("http://svrngs.localhost:{port}/");
    let at = format!("http://{}.localhost:{port}/", label(1, "pad"));
    let open: Value = reqwest::Client::new()
        .post(format!("http://{}/v1/open", ada.api_addr()))
        .bearer_auth(ada.token().trim())
        .json(&json!({}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let seen = drive(
        engine,
        "open.mjs",
        json!({ "home": home, "lend": at, "open": open["open"], "words": "pad" }),
        None,
    )
    .await;
    eprintln!("the open's run: {seen:#}");

    let opener = if cfg!(target_os = "macos") {
        "Open svrngs from your Applications"
    } else {
        "Open svrngs from your applications menu"
    };
    for fresh in ["fresh_home", "fresh_lend"] {
        assert_eq!(
            seen[fresh]["title"], "Open svrngs from your computer",
            "{seen:#}"
        );
        let text = seen[fresh]["text"].as_str().unwrap_or_default();
        assert!(text.contains(opener), "{fresh} names the opener: {seen:#}");
    }
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    assert_eq!(
        seen["opened"],
        format!("{home}?ui=a"),
        "the open lands on the home page, long as page.mjs `longPage` holds it: {seen:#}"
    );
    for (key, url, title) in [("lend", &at, "ada's pad"), ("lend_again", &at, "ada's pad")] {
        assert_eq!(
            (&seen[key]["url"], &seen[key]["title"]),
            (&json!(url), &json!(title)),
            "{key}: {seen:#}"
        );
    }
    assert_eq!(
        seen["home_again"]["url"],
        format!("{home}?ui=a"),
        "{seen:#}"
    );
    assert_ne!(
        seen["home_again"]["title"], "Open svrngs from your computer",
        "a bookmark opens the home page again: {seen:#}"
    );
    let asked = on_pad.lock().unwrap().len();
    assert_eq!(
        asked, 2,
        "the pad is asked only once opened, twice: {seen:#}"
    );
}

in_chromium!(
    /// **Consent/scope preserved** (docs/APP_ACCESS_AND_STOP.md A4, §5):
    /// Probe, carried in house on ada's computer as Give is, asks svrngs's
    /// opener at its own origin (`/svrngs-to`, surface.rs `to`) by fetch
    /// whether cy's notes open, cy's `hidden`, lent to another group only,
    /// and the notes of dee, whom ada removed from house: the first is
    /// told only that it opens, 204, and each other is not lent there,
    /// 404. Its link to cy's notes, followed in a tab, is sent on to their
    /// plain origin, 303, no code, capability or proof in the address, and
    /// opens there through the home page's `/to`; and there cy's notes,
    /// which read `films`, are still refused cy's films, `unasked`, until
    /// ada's yes on her home page: the open is no yes, and it hands Probe
    /// nothing to read or sign with. Planted, watched failing: the opener
    /// sending the tab on with a code for the lend's origin
    /// (`Redirect::to(&at.sessions.open(..))`), and the lend found by its
    /// name alone (`l.key == to.lender` dropped).
    a_carried_app_opens_one_lend_of_its_group_and_no_more
);
async fn a_carried_app_opens_one_lend_of_its_group_and_no_more(engine: Engine) {
    let relay = relay().await;
    let dirs = [(); 3].map(|()| tempfile::tempdir().unwrap());
    let ada = node(dirs[0].path(), &relay, &signing(9)).await;
    let cy = joined(dirs[1].path(), &relay, &ada, 2, "cy").await;
    let dee = joined(dirs[2].path(), &relay, &ada, 3, "dee").await;
    let notes = page_origin("<!doctype html><title>cy's notes</title><body>cy's notes").await;
    let films = page_origin("<!doctype html><title>cy's films</title><body>").await;
    let theirs = page_origin("<!doctype html><title>dee's notes</title><body>").await;
    for (dir, name, port, kind, groups, reads) in [
        (
            dirs[1].path(),
            "notes",
            notes.port(),
            "notes",
            json!([NS]),
            json!(["films"]),
        ),
        (
            dirs[1].path(),
            "films",
            films.port(),
            "films",
            json!([NS]),
            json!([]),
        ),
        (
            dirs[1].path(),
            "hidden",
            films.port(),
            "k",
            json!(["elsewhere"]),
            json!([]),
        ),
        (
            dirs[2].path(),
            "notes",
            theirs.port(),
            "notes",
            json!([NS]),
            json!([]),
        ),
    ] {
        let lends = svrngs_node::lend::lends_dir(dir);
        std::fs::create_dir_all(&lends).unwrap();
        let body = json!({ "groups": groups, "port": port, "kind": kind, "limit": 3, "refused": [], "reads": reads });
        std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
    }
    assert_eq!(
        removal(&ada, "dee").await,
        svrngs_node::removal::Outcome::Out
    );
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let key = |seed: u8| hex::encode(signing(seed).verifying_key().as_bytes());
    let probe = format!(
        r#"const at = (lender, lend) => `/svrngs-to?${{new URLSearchParams({{ lender, lend }})}}`;
const asked = async (who, lender, lend) => {{
  const r = await fetch(`${{at(lender, lend)}}&whether=true`);
  return `${{who}}'s ${{lend}}: ${{r.status}}`;
}};
Promise.all([asked('cy', '{cy}', 'notes'), asked('cy', '{cy}', 'hidden'), asked('dee', '{dee}', 'notes')]).then((said) => {{
  const p = Object.assign(document.createElement('pre'), {{ id: 'said', textContent: said.join('\n') }});
  const a = Object.assign(document.createElement('a'), {{ href: at('{cy}', 'notes'), target: '_blank', textContent: 'Open notes' }});
  document.body.append(p, a);
  document.title = 'tried';
}});
"#,
        cy = key(2),
        dee = key(3),
    );
    let page = "<!doctype html><title>trying</title><body><script src=probe.js></script>";
    carried::carry(
        &folder,
        "Probe",
        &[("index.html", page.into()), ("probe.js", probe)],
    );
    let origin = carried::carried_at(&ada, "Probe").await;
    let port = ada.api_addr().port();
    let seen = drive(engine, "opens-one.mjs", json!({ "url": origin }), None).await;
    eprintln!("the open's run: {seen:#}");
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    assert_eq!(
        seen["said"], "cy's notes: 204\ncy's hidden: 404\ndee's notes: 404",
        "a page is told only whether one lend of its group opens:\n{seen:#}"
    );
    let plain = format!("http://{}.localhost:{port}/", label(2, "notes"));
    assert_eq!(
        seen["sent"][0],
        json!({ "status": 303, "location": plain }),
        "followed, it goes on to the lend's plain origin, no code, capability or proof in it:\n{seen:#}"
    );
    assert!(at_origin(&seen["url"], &label(2, "notes")), "{seen:#}");
    assert_eq!(
        seen["read"],
        json!({ "addresses": 0, "new": ["films"] }),
        "opened so, the notes still wait for ada's yes: no address, and told a lend is new:\n{seen:#}"
    );
    drop((cy, dee));
}

in_chromium!(
    /// **A carried app reads a program that takes a key** (CONTRACTS §4,
    /// `readers: key x-api-key`, EXTENSION_POINT §10 row 9). Cy lends
    /// `photos`, program immich, on a service that answers only the exact
    /// `x-api-key` the lender keeps (`share::keep_key`, the row in
    /// src/programs/immich.txt) and 401 to anything else. Ada's carried
    /// app, whose reads.txt names immich, reads it through its own
    /// origin's `/svrngs-lend/` after her yes. A fake: this is the gate's
    /// run; `a_carried_app_reads_a_real_immich` is the same body against
    /// one. Planted, watched failing: the door sending the lend's own
    /// secret in its place (`Admit::Readers => Vec::new()`) answers 401.
    a_carried_app_reads_a_program_that_takes_a_key
);
async fn a_carried_app_reads_a_program_that_takes_a_key(engine: Engine) {
    let (service, heads) = keyed_service("apps-key-1").await;
    let seen = app_reads_with_a_key(engine, service.port(), "apps-key-1", "/photos?n=1").await;
    assert_eq!(
        (
            &seen["yes"]["films"]["status"],
            &seen["yes"]["films"]["body"]
        ),
        (&json!(200), &json!(r#"{"photos":"cy's photos"}"#)),
        "{seen:#}"
    );
    assert!(
        heads
            .lock()
            .unwrap()
            .iter()
            .any(|h| h.to_ascii_lowercase().contains("x-api-key: apps-key-1")),
        "it reached the service with the exact key: {heads:?}"
    );
}

/// The same read as [`a_carried_app_reads_a_program_that_takes_a_key`],
/// against a real Immich run by hand: a real one is not in the gate, so
/// this is ignored and needs `SVRNGS_TEST_IMMICH_PORT`, a loopback port its
/// `/api/users/me` answers on (an `ssh -L` tunnel to one), and
/// `SVRNGS_TEST_IMMICH_KEY`, an API key of its. The kept key is the bytes
/// the lender pasted, exactly as a real paste is.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs a real Immich: SVRNGS_TEST_IMMICH_PORT and SVRNGS_TEST_IMMICH_KEY"]
async fn a_carried_app_reads_a_real_immich() {
    let port = std::env::var("SVRNGS_TEST_IMMICH_PORT").expect("SVRNGS_TEST_IMMICH_PORT");
    let key = std::env::var("SVRNGS_TEST_IMMICH_KEY").expect("SVRNGS_TEST_IMMICH_KEY");
    let seen = app_reads_with_a_key(
        Engine::Chromium,
        port.parse().unwrap(),
        &key,
        "/api/users/me",
    )
    .await;
    assert_eq!(seen["yes"]["films"]["status"], json!(200), "{seen:#}");
    let body = seen["yes"]["films"]["body"].as_str().unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(body).expect("a JSON answer");
    eprintln!("the app read this from a real Immich's /api/users/me: {body:#}");
    assert!(body["email"].as_str().is_some(), "{body:#}");
    assert!(
        !seen.to_string().contains(&key),
        "the key never reached the page: {seen:#}"
    );
}

/// A service refusing, 401, any request without exactly `x-api-key: key`,
/// each request's head kept; `{"photos":"cy's photos"}` for one that has it.
async fn keyed_service(key: &'static str) -> (std::net::SocketAddr, KeptHeads) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let heads = KeptHeads::default();
    let kept = heads.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let mut head = [0u8; 8192];
                let n = tokio::io::AsyncReadExt::read(&mut sock, &mut head)
                    .await
                    .unwrap_or(0);
                let head = String::from_utf8_lossy(&head[..n]).into_owned();
                let has = head
                    .lines()
                    .any(|l| l.eq_ignore_ascii_case(&format!("x-api-key: {key}")));
                kept.lock().unwrap().push(head);
                let (status, body) = if has {
                    ("200 OK", r#"{"photos":"cy's photos"}"#)
                } else {
                    ("401 Unauthorized", r#"{"error":"no key"}"#)
                };
                let answer = format!(
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(answer.as_bytes()).await;
            });
        }
    });
    (addr, heads)
}

/// Cy lends `photos` (program `immich`) at `port` with the key kept as
/// `key`; ada carries an app whose reads.txt names immich and opens it from
/// her home page, her yes; what the app's page read at `path` through its
/// own `/svrngs-lend/`, by the driver `carried-reads.mjs`, as "films".
async fn app_reads_with_a_key(engine: Engine, port: u16, key: &str, path: &str) -> Value {
    let relay = relay().await;
    let dirs = [(); 2].map(|()| tempfile::tempdir().unwrap());
    let ada = node(dirs[0].path(), &relay, &signing(9)).await;
    let cy = joined(dirs[1].path(), &relay, &ada, 2, "cy").await;
    let lends = svrngs_node::lend::lends_dir(dirs[1].path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "groups": [NS], "port": port, "kind": "immich", "limit": 3, "refused": [] });
    std::fs::write(lends.join("photos.json"), body.to_string()).unwrap();
    let name = svrngs_node::lend::LendName::parse("photos").unwrap();
    let program = svrngs_node::lend::Program::parse("immich").unwrap();
    assert!(svrngs_node::share::keep_key(
        dirs[1].path(),
        &name,
        svrngs_node::share::key_of(&program, ""),
        key
    )
    .unwrap());
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let page = "<!doctype html><title>Probe</title><body>Probe";
    let files = [
        ("index.html", page.into()),
        ("reads.txt", "immich\n".into()),
    ];
    carried::carry(&folder, "Probe", &files);
    let origin = carried::carried_at(&ada, "Probe").await;
    let api = ada.api_addr().port();
    let seen = drive(
        engine,
        "carried-reads.mjs",
        json!({ "url": origin, "home": format!("http://svrngs.localhost:{api}/"), "words": "Open Probe",
            "tries": { "films": label(2, "photos") }, "path": path }),
        None,
    )
    .await;
    eprintln!("the carried app's read of the keyed program: {seen:#}");
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    assert_eq!(
        seen["before"]["films"]["refused"],
        json!("unasked"),
        "{seen:#}"
    );
    assert_eq!(
        seen["stopped"]["films"]["refused"],
        json!("unasked"),
        "{seen:#}"
    );
    drop(cy);
    drop(ada);
    seen
}

/// cy's films as a reading app meets them: each request's head kept, and
/// answered with its `authorization` echoed in a header of its own and a
/// cookie of the lend's own set, as JSON, or as a script at a path ending
/// `.js`.
async fn echoing() -> (std::net::SocketAddr, KeptHeads) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let heads = KeptHeads::default();
    let kept = heads.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let mut head = [0u8; 8192];
                let n = tokio::io::AsyncReadExt::read(&mut sock, &mut head)
                    .await
                    .unwrap_or(0);
                let head = String::from_utf8_lossy(&head[..n]).into_owned();
                let auth = (head.lines())
                    .find(|l| l.to_ascii_lowercase().starts_with("authorization: "))
                    .map(|l| l["authorization: ".len()..].to_string())
                    .unwrap_or_else(|| "none".into());
                let script = head.lines().next().is_some_and(|l| l.contains(".js "));
                kept.lock().unwrap().push(head);
                let (kind, body) = if script {
                    ("Text/JavaScript", "document.title = 'ran';")
                } else {
                    ("application/json", r#"{"films":"cy's films"}"#)
                };
                let answer = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: {kind}\r\nx-echo: {auth}\r\n\
                     clear-site-data: \"storage\"\r\n\
                     set-cookie: lend-session=kept-by-the-lend; Path=/\r\n\
                     content-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(answer.as_bytes()).await;
            });
        }
    });
    (addr, heads)
}

in_chromium!(
    /// **A carried app reads the lends it declares, at its own origin, on
    /// your yes** (the operator, 2026-11-08; svrn-docs COMMUNITY_APPS §6.2,
    /// "It knows only what it's told"): Probe, carried in house on ada's
    /// computer, declares `films` in its reads.txt; its page's policy lets
    /// it ask its own origin alone, so it reaches lends at
    /// `/svrngs-lend/<label>/` there. Its `/svrngs-reads` lists cy's films
    /// at that address, holding no capability. Opened by its typed address,
    /// every lend is refused before anything reaches them: cy's films
    /// `unasked`; cy's notes, a kind it does not declare, ada's films, lent
    /// to elsewhere alone, another group of ada's, and the films of dee,
    /// whom ada removed, `closed`. Opened from the home page, whose line
    /// beside it says "It also opens your friends’ films as you", the yes:
    /// cy's films answer its GET, let in as cy's readers' account; its POST
    /// carries no readers' secret; a script is refused; the answer is
    /// sandboxed and never sniffed; and nothing the page can read holds
    /// the readers' secret, which cy's films echo back, the lend's cookie,
    /// a capability or ada's token. The others stay `closed`. After the
    /// yes's Stop, cy's films are `unasked` again.
    a_carried_app_reads_the_lends_it_declares_at_its_own_origin
);
async fn a_carried_app_reads_the_lends_it_declares_at_its_own_origin(engine: Engine) {
    let relay = relay().await;
    let dirs = [(); 3].map(|()| tempfile::tempdir().unwrap());
    let ada = node(dirs[0].path(), &relay, &signing(9)).await;
    let cy = joined(dirs[1].path(), &relay, &ada, 2, "cy").await;
    let dee = joined(dirs[2].path(), &relay, &ada, 3, "dee").await;
    // A second group, elsewhere, of ada's alone.
    let ada_alone = [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])];
    let seed = Roster::new(ada_alone.into_iter().collect());
    ada.rail().set_seed("elsewhere", &seed).unwrap();
    let (films, on_films) = echoing().await;
    let notes = page_origin("<!doctype html><title>cy's notes</title><body>").await;
    let (theirs, on_theirs) = kept_page("<!doctype html><title>dee's films</title><body>").await;
    let (mine, on_mine) = kept_page("<!doctype html><title>ada's films</title><body>").await;
    for (dir, name, port, kind, group) in [
        (dirs[1].path(), "films", films.port(), "films", NS),
        (dirs[1].path(), "notes", notes.port(), "notes", NS),
        (dirs[2].path(), "films", theirs.port(), "films", NS),
        (dirs[0].path(), "films", mine.port(), "films", "elsewhere"),
    ] {
        let lends = svrngs_node::lend::lends_dir(dir);
        std::fs::create_dir_all(&lends).unwrap();
        let body =
            json!({ "groups": [group], "port": port, "kind": kind, "limit": 3, "refused": [] });
        std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
    }
    let readers = ("authorization", "Readers films-for-apps");
    let name = svrngs_node::lend::LendName::parse("films").unwrap();
    svrngs_node::lend::keep_readers(dirs[1].path(), &name, readers).unwrap();
    assert_eq!(
        removal(&ada, "dee").await,
        svrngs_node::removal::Outcome::Out
    );
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let page = "<!doctype html><title>Probe</title><body>Probe";
    let files = [("index.html", page.into()), ("reads.txt", "films\n".into())];
    carried::carry(&folder, "Probe", &files);
    let origin = carried::carried_at(&ada, "Probe").await;
    let port = ada.api_addr().port();
    // A yes kept from a version that read only notes: it covers no films.
    let app = origin
        .trim_start_matches("http://")
        .split('.')
        .next()
        .unwrap();
    let narrower =
        json!({ actor_of(&signing(1)): { app: { "said": "yes", "at": 1, "reads": ["notes"] } } });
    std::fs::write(dirs[0].path().join("reaching.json"), narrower.to_string()).unwrap();
    let tries = json!({
        "films": label(2, "films"),
        "notes": label(2, "notes"),
        "elsewhere": label(1, "films"),
        "removed": label(3, "films"),
    });
    let seen = drive(
        engine,
        "carried-reads.mjs",
        json!({ "url": origin, "home": format!("http://svrngs.localhost:{port}/"), "words": "Open Probe", "tries": tries }),
        None,
    )
    .await;
    eprintln!("the carried app's reads: {seen:#}");
    assert_eq!(seen["error"], Value::Null, "{seen:#}");

    let reads = |step: &str| -> Value {
        serde_json::from_str(seen[step]["reads"]["body"].as_str().unwrap()).unwrap()
    };
    let before = reads("before");
    assert_eq!(before["lends"], json!([]), "no yes, no address:\n{seen:#}");
    assert_eq!(
        before["new"],
        json!([{ "kind": "films", "name": "films", "lender": "cy" }]),
        "before the yes it is told what is new, with no address:\n{seen:#}"
    );
    let address = format!("{origin}svrngs-lend/{}/", label(2, "films"));
    assert_eq!(
        reads("yes")["lends"],
        json!([{ "kind": "films", "name": "films", "lender": "cy", "address": address,
            "to": format!("{origin}svrngs-to?lender={}&lend=films", actor_of(&signing(2))) }]),
        "its reads are its declared kind's lends of its group, at its own origin:\n{seen:#}"
    );
    let refused = |step: &str, name: &str| {
        (
            seen[step][name]["status"].clone(),
            seen[step][name]["refused"].clone(),
        )
    };
    let closed = (json!(403), json!("closed"));
    let unasked = (json!(403), json!("unasked"));
    for (step, name, said) in [
        ("before", "films", &unasked),
        ("stopped", "films", &unasked),
    ]
    .into_iter()
    .chain(["before", "yes", "stopped"].into_iter().flat_map(|step| {
        ["notes", "elsewhere", "removed", "post", "act"].map(|name| (step, name, &closed))
    })) {
        assert_eq!(&refused(step, name), said, "{step}: {name}:\n{seen:#}");
    }
    let cell = "Open ProbeReads cy’s films. They can see it’s you, and what it asks for. cy’s films Its own part of the group’s record. Opening it lets it read that, and never write it, as you.";
    assert!(
        seen["cells"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|c| c == cell),
        "the open says what the app reads:\n{seen:#}"
    );
    let yes = &seen["yes"];
    assert_eq!(
        (&yes["films"]["status"], &yes["films"]["body"]),
        (&json!(200), &json!(r#"{"films":"cy's films"}"#)),
        "once said yes to, it reads its declared lend:\n{seen:#}"
    );
    let mut names: Vec<_> = (yes["films"]["headers"].as_array().unwrap().iter())
        .map(|h| h.as_str().unwrap())
        .filter(|h| {
            !["date:", "content-length:"]
                .iter()
                .any(|d| h.starts_with(d))
        })
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "cache-control: no-store",
            "content-security-policy: sandbox",
            "content-type: application/json",
            "x-content-type-options: nosniff",
        ],
        "only what svrngs says reaches the page:\n{seen:#}"
    );
    assert_eq!(
        refused("yes", "script"),
        closed,
        "a lend's script never runs here:\n{seen:#}"
    );
    let heads = on_films.lock().unwrap().clone();
    let asked = |line: &str| {
        heads
            .iter()
            .find(|h| h.starts_with(line))
            .cloned()
            .unwrap_or_default()
    };
    assert!(
        asked("GET /films?n=1").contains("films-for-apps"),
        "its GET is let in as cy's readers' account: {heads:#?}"
    );
    assert!(
        !asked("GET /films?n=1").to_lowercase().contains("x-page"),
        "of the page's headers only Accept is carried: {heads:#?}"
    );
    assert!(
        asked("POST /films").is_empty(),
        "its POST never reaches the lend: {heads:#?}"
    );
    assert_eq!(
        heads.len(),
        2,
        "cy's films are read only after the yes that names them, twice: {heads:#?}"
    );
    let said = seen.to_string();
    for kept in [
        "films-for-apps",
        "lend-session",
        "svrngs-as/",
        ada.token().trim(),
    ] {
        assert!(
            !said.contains(kept),
            "the page reads nothing holding {kept:?}:\n{seen:#}"
        );
    }
    for (whose, heads) in [("dee's", &on_theirs), ("ada's elsewhere", &on_mine)] {
        assert!(
            heads.lock().unwrap().is_empty(),
            "{whose} films are never asked"
        );
    }
    drop((cy, dee));
}

in_chromium!(
    /// **The apps' account, made from the home page, with no terminal**
    /// (capability connections §5 step 5, §6 bar 1): ada lends a stand-in
    /// Jellyfin. Her home page says apps can't read it yet and folds out
    /// how to make the account for her group; she types its name and
    /// password and saves it, and the row then says apps read it with the
    /// account she made and can't change anything. The password crosses
    /// only to her own computer, and what her home page reads back never
    /// holds it or the token svrngs keeps. Planted, watched failing: the
    /// row drawn from the words alone, the button posting nothing.
    the_apps_account_is_made_from_the_home_page
);
async fn the_apps_account_is_made_from_the_home_page(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(9)).await;
    let jellyfin = sign_in_stand_in().await;
    let lends = svrngs_node::lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body =
        json!({ "groups": [NS], "port": jellyfin.port(), "kind": "jellyfin", "refused": [] });
    std::fs::write(lends.join("jellyfin.json"), body.to_string()).unwrap();
    let row = "#lent li";
    let kept = dir
        .path()
        .join("secrets/lends/jellyfin/readers/authorization");
    let seen = home_page(
        engine,
        &ada,
        json!({
            "unfold": true,
            "all": "#lent li",
            "do": [
                [format!("{row} input[name=account]"), "films-group"],
                [format!("{row} input[name=password]"), "pw-1"],
                [format!("{row} button:has-text('Save the account')"), null, true, "account you made"],
            ],
            "text": true,
        }),
    )
    .await;
    eprintln!("ada's home page: {seen:#}");
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    let before = seen["all"].to_string();
    assert!(
        before.contains(
            "Apps can’t read your jellyfin yet. To let them, make one account for your group."
        ) && before.contains("Dashboard, Users"),
        "the row says what is missing, and how:\n{seen:#}"
    );
    let after = seen["after"][0].to_string();
    assert!(
        after.contains("Apps your friends open can read your jellyfin with the account you made. They can’t change anything."),
        "the row says what the account does:\n{seen:#}"
    );
    assert_eq!(
        std::fs::read_to_string(&kept).unwrap(),
        r#"MediaBrowser Token="tok-1""#,
        "kept as the command keeps it:\n{seen:#}"
    );
    let sent = seen["sent"].as_array().unwrap();
    let gives_password = |s: &&Value| s[2].as_str().is_some_and(|b| b.contains("pw-1"));
    let posts: Vec<_> = sent.iter().filter(gives_password).collect();
    assert_eq!(posts.len(), 1, "the password is sent once:\n{seen:#}");
    assert!(
        posts[0][1].as_str().is_some_and(|u| u.ends_with("/lent")),
        "and only to her own computer's /lent:\n{seen:#}"
    );
    let mut stack = vec![dir.path().to_path_buf()];
    while let Some(at) = stack.pop() {
        for entry in std::fs::read_dir(&at).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if std::fs::read(&path)
                .unwrap_or_default()
                .windows(4)
                .any(|w| w == b"pw-1")
            {
                panic!("the password is kept at {path:?}");
            }
        }
    }
    drop(ada);
}

in_chromium!(
    /// **A key is pasted on the home page, with no terminal** (a lender
    /// with no terminal; `readers: key x-api-key`, CONTRACTS §4): ada lends
    /// a stand-in Immich. Her home page says apps can't read it yet and
    /// how to make a key; she pastes it into the one box and saves it, and
    /// the row then says apps read it with the key she made. It crosses
    /// only to her own computer, and what her home page reads back never
    /// holds it. Planted, watched failing: the row drawn from the words
    /// alone, the button posting nothing.
    a_readers_key_is_pasted_on_the_home_page
);
async fn a_readers_key_is_pasted_on_the_home_page(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(9)).await;
    let immich = page_origin("<!doctype html><title>Immich</title><body>").await;
    let lends = svrngs_node::lend::lends_dir(dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "groups": [NS], "port": immich.port(), "kind": "immich", "refused": [] });
    std::fs::write(lends.join("immich.json"), body.to_string()).unwrap();
    let row = "#lent li";
    let seen = home_page(
        engine,
        &ada,
        json!({
            "unfold": true,
            "all": "#lent li",
            "do": [
                [format!("{row} input[name=password]"), "pasted-key-1"],
                [format!("{row} button:has-text('Save the key')"), null, true, "key you made"],
            ],
            "text": true,
        }),
    )
    .await;
    eprintln!("ada's home page: {seen:#}");
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    let before = seen["all"].to_string();
    assert!(
        before.contains(
            "Apps can’t read your immich yet. To let them, make a key in immich and paste it here."
        ) && before.contains("New API Key"),
        "the row says what is missing, and how:\n{seen:#}"
    );
    let after = seen["after"][0].to_string();
    assert!(
        after.contains("Apps your friends open can read your immich with the key you made. They can’t change anything."),
        "the row says what the key does:\n{seen:#}"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("secrets/lends/immich/readers/x-api-key")).unwrap(),
        "pasted-key-1",
        "kept as the command keeps it:\n{seen:#}"
    );
    let sent = seen["sent"].as_array().unwrap();
    let gives_key = |s: &&Value| s[2].as_str().is_some_and(|b| b.contains("pasted-key-1"));
    let posts: Vec<_> = sent.iter().filter(gives_key).collect();
    assert_eq!(posts.len(), 1, "the key is sent once:\n{seen:#}");
    assert!(
        posts[0][1].as_str().is_some_and(|u| u.ends_with("/lent")),
        "and only to her own computer's /lent:\n{seen:#}"
    );
    drop(ada);
}

/// A stand-in for Jellyfin's sign-in: `tok-1` for a body holding `pw-1`,
/// else 401.
async fn sign_in_stand_in() -> std::net::SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut got = Vec::new();
                let mut chunk = [0u8; 4096];
                loop {
                    let n = tokio::io::AsyncReadExt::read(&mut sock, &mut chunk)
                        .await
                        .unwrap_or(0);
                    got.extend_from_slice(&chunk[..n]);
                    let text = String::from_utf8_lossy(&got);
                    if n == 0 || text.contains("Pw") {
                        break;
                    }
                }
                let text = String::from_utf8_lossy(&got);
                let (status, answer) = match text.contains("pw-1") {
                    true => ("200 OK", r#"{"AccessToken":"tok-1"}"#),
                    false => ("401 Unauthorized", ""),
                };
                let reply = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{answer}",
                    answer.len()
                );
                let _ = tokio::io::AsyncWriteExt::write_all(&mut sock, reply.as_bytes()).await;
            });
        }
    });
    addr
}
