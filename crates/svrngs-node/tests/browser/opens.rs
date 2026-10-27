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
        let body = json!({ "port": port, "kind": kind, "limit": 3, "refused": [], "reads": reads });
        std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
    }
    let port = ada.api_addr().port();
    let at = |name: &str| format!("http://{}.localhost:{port}/", label(2, name));
    let words = "cy’s notes It writes what you say there into your group’s record, as you. It also opens your friends’ films as you.";

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
        json!({ "status": 403, "refused": "unasked", "error": "Say yes on your home page: open this app from there." }),
        "a page opened another way waits for the person's yes:\n{seen:#}"
    );
    let cells = seen["cells"].as_array().cloned().unwrap_or_default();
    for cell in [
        words,
        "cy’s plain (k) It writes what you say there into your group’s record, as you.",
        "cy’s films It writes what you say there into your group’s record, as you.",
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
/// neither writes what you say there as you. Both pages draw the line
/// beside the open from question.js's `agreed`, and the open by `named`.
/// Red at d736ff18: `agreed` said "Open " and the name before what the app
/// does, all of it on the button. Red at 5c6e3b0d: "It reads and writes"
/// first, which Ana read as done on her behalf without her yes (the fifth
/// xenia set, demo/xenia/findings/said-too-much-the-group-s-page.md).
#[test]
fn the_open_says_what_an_app_does_in_one_wording() {
    for page in ["crates/svrngs-node/src/home/home.js", "web/app.js"] {
        let text = std::fs::read_to_string(repo().join(page)).unwrap();
        assert!(
            text.contains("agreed(l, "),
            "{page} draws no open by `agreed`"
        );
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
            ["cy’s notes", "It writes what you say there into House’s record, as you. It also opens your friends’ films and music as you."],
            ["cy’s shelf (k)", "It writes what you say there into House’s record, as you."],
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
        seen["opened"], home,
        "the open lands on the home page: {seen:#}"
    );
    for (key, url, title) in [("lend", &at, "ada's pad"), ("lend_again", &at, "ada's pad")] {
        assert_eq!(
            (&seen[key]["url"], &seen[key]["title"]),
            (&json!(url), &json!(title)),
            "{key}: {seen:#}"
        );
    }
    assert_eq!(seen["home_again"]["url"], home, "{seen:#}");
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
