//! "What acts as you" (capability-what-acts; svrn-docs
//! `rings/current/WHAT_CAN_ACT_AS_YOU.md` §9, "One place"): the home page
//! and the group page each show one section, grouped by what each thing
//! can do as the person, each line saying what, since when, and its Stop.
//! What it protects: a yes the person gave is something they can see and
//! take back, and a Stop holds — on a computer from the app's next
//! request, on a phone from the next pass it signs, within 12 hours.
//! Nobody else's computer is listed as deciding for you (§4).

use super::carried::{carried_at, carry};
use super::*;

/// "3 Oct", as question.js `day` says a day in this computer's time.
fn today() -> String {
    chrono::Local::now().format("%-d %b").to_string()
}

/// **The section's words, one wording for both pages** (§9): an app the
/// group carries reads and writes what the shelf says, when you open it,
/// with no Stop; a yes to an app opening your friends' things says since
/// when, and a stop says when; who can decide for you is your own
/// devices, this one first, never another person's of your name.
#[test]
fn what_acts_as_you_is_said_in_one_wording() {
    let shelf = json!([{ "name": "Money", "words": "who paid for what, and who owes whom." }]);
    let notes = json!({ "person": "cy", "name": "notes", "kind": "notes", "reads": ["films"] });
    let at = chrono::Local::now().timestamp();
    let friends: Vec<_> = ["Sam", "Bo", "Cy", "Dee", "Eli"]
        .iter()
        .map(|p| json!({ "person": p, "kind": "jellyfin", "new": false }))
        .collect();
    let row = |key: &str, person: &str, first: &str, device: &str| json!({ "key": key, "person": person, "person_key": first, "device": device });
    let rows = json!([
        row("a1", "ada", "a1", "computer"),
        row("a2", "ada", "a1", "browser"),
        row("n1", "ada", "n1", "computer"),
        row("b1", "bo", "b1", "browser"),
    ]);
    assert_eq!(
        in_question(
            "[q.carrying({ kind: 'Money' }, args.shelf), q.carrying({ kind: 'Quiz' }, args.shelf), q.carrying({ kind: 'Movie-Night', reads: ['jellyfin'] }, args.shelf), q.reading(args.notes, { said: 'yes', at: args.at }), q.reading(args.notes, { said: 'stop', at: args.at }), q.reading(args.notes, { said: 'yes', at: args.at, lends: ['x'] }), q.reading(args.notes, { said: 'yes', at: args.at, lends: [] }), q.readLine([args.sam, args.bo], 2, ['jellyfin']), q.readLine([args.sam, { ...args.bo, new: true }], 2, ['jellyfin']), q.readLine([], 0, ['jellyfin']), q.readLine([], 2, ['jellyfin']), q.readLine(args.friends, 5, ['jellyfin']), q.deciders(args.rows, 'a1', 'computer'), q.deciders(args.rows, 'a2', 'browser'), q.deciders(args.rows, 'b1', 'browser')]",
            json!({ "shelf": shelf, "notes": notes, "at": at, "rows": rows,
                "sam": { "person": "Sam", "kind": "jellyfin", "new": false },
                "bo": { "person": "Bo", "kind": "jellyfin", "new": false },
                "friends": friends }),
        ),
        json!([
            "Money reads and writes who paid for what, and who owes whom, when you open it.",
            "Quiz reads and writes its own part of the group’s record, when you open it.",
            "Movie Night reads its own part of the group’s record, when you open it.",
            format!("cy’s notes opens your friends’ films as you. Since {}. ", today()),
            format!("cy’s notes stopped opening your friends’ films as you on {}. Opening it again says yes again.", today()),
            format!("cy’s notes opens the films you chose, as you. Since {}. ", today()),
            "cy’s notes opens nothing of your friends’ as you. Open it again to choose what it can. ",
            "Reads Sam’s and Bo’s jellyfin. They can see it’s you, and what it asks for.",
            "Reads Sam’s and Bo’s jellyfin. Bo’s is new since you last opened it.",
            "Reads your friends’ jellyfin. Nobody lends one now.",
            "Reads none of your friends’ jellyfin.",
            "Reads jellyfin from 5 friends. They can see it’s you, and what it asks for.",
            "This computer, and your phone",
            "This phone, and your computer",
            "This phone",
        ])
    );
}

in_chromium!(
    /// **The home page says what acts as you, and its Stop holds** (§9).
    /// Cy lends notes, declared to read `films`, and films; ada's computer
    /// carries Money. Ada opens cy's notes from her home page, which is her
    /// yes (capability-open). Her home page then shows "What acts as you":
    /// Money reads and writes what the shelf says, when she opens it; cy's
    /// notes opens her friends' films as her, since today, with Stop; what
    /// she opens knows her name; and only this computer can decide for her,
    /// not bo's, admitted to the group. Its Stop holds from the notes' next
    /// request: a page of the notes reaching cy's films as ada is refused,
    /// `unasked`, after it. Its fetch is at the address `/svrngs-reads`
    /// gives, never the films' plain origin (give-session). Red at 0573f402: no such parts, and no Stop for
    /// the yes. Planted: the Stop kept as a yes, and the notes still reach
    /// the films.
    the_home_page_says_what_acts_as_you_and_its_stop_holds
);
async fn the_home_page_says_what_acts_as_you_and_its_stop_holds(engine: Engine) {
    let relay = relay().await;
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &signing(7)).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 2, "cy").await;
    let folder = ada_dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let money = "<!doctype html><title>Money</title>".to_string();
    carry(&folder, "Money", &[("index.html", money)]);
    carried_at(&ada, "Money").await;
    let notes = page_origin("<!doctype html><title>cy's notes</title><body>").await;
    let (films, on_films) = kept_page("<!doctype html><title>cy's films</title><body>").await;
    let lends = svrngs_node::lend::lends_dir(cy_dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    for (name, port, reads) in [
        ("notes", notes.port(), json!(["films"])),
        ("films", films.port(), json!([])),
    ] {
        let body = json!({ "groups": [NS], "port": port, "kind": name, "limit": 3, "refused": [], "reads": reads });
        std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
    }
    let port = ada.api_addr().port();
    let at = |name: &str| format!("http://{}.localhost:{port}/", label(2, name));

    let seen = drive(
        engine,
        "acts.mjs",
        json!({
            "home": format!("http://svrngs.localhost:{port}/"),
            "page": at("notes"),
            "read": at("films"),
            "words": "cy’s notes",
        }),
        None,
    )
    .await;
    eprintln!("the home page's run: {seen:#}");
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    assert_eq!(
        seen["folded"], true,
        "\"What acts as you\" is folded under its heading: {seen:#}"
    );
    let before = seen["before"].as_str().unwrap_or_default();
    for part in [
        "What acts as you\n",
        "Reads and writes for you\n",
        "Money reads and writes who owes whom, and for what, when you open it.",
        "Opens friends’ things as you\n",
        "Each Stop holds from the app’s next request.",
        &format!(
            "cy’s notes opens the films you chose, as you. Since {}.",
            today()
        ),
        "Knows who you are\n",
        "What you open knows your name.",
        "Can decide for you\n",
        "Your group: This computer.",
    ] {
        assert!(
            before.contains(part),
            "the section says {part:?}:\n{before}"
        );
    }
    let after = seen["after"].as_str().unwrap_or_default();
    assert!(
        after.contains(&format!(
            "cy’s notes stopped opening your friends’ films as you on {}.",
            today()
        )),
        "the Stop is said:\n{after}"
    );
    assert_eq!(
        seen["stopped"],
        json!({ "status": 403, "refused": "unasked", "error": "Open it from your home page first." }),
        "a Stop holds from the app's next request:\n{seen:#}"
    );
    let asked: Vec<String> = on_films
        .lock()
        .unwrap()
        .iter()
        .filter_map(|h| h.lines().next().map(str::to_string))
        .collect();
    assert!(
        asked.iter().any(|l| l.starts_with("GET /opened "))
            && !asked.iter().any(|l| l.starts_with("GET /stopped ")),
        "the films are reached on the yes, and not after its Stop: {asked:?}"
    );
    assert!(
        !asked.iter().any(|l| l.starts_with("GET /bare ")),
        "the films are never reached at their plain origin, which a fetch carrying no capability asks (give-session): {asked:?}"
    );
    drop((ada, cy));
}

in_webkit!(
    /// **The group page says what acts as you, and its Stop holds at the
    /// next pass** (§9, "On a phone"). Ada lends a page reading kind `k`;
    /// cy lends a shelf of it. Bo's phone opens ada's page from the group
    /// page, which is bo's yes, and the pass names cy's shelf. Back on the
    /// group page, "What acts as you" says ada's page opens bo's friends' k
    /// as bo, since today, with Stop, and that a Stop holds from the next
    /// pass, within 12 hours; only this phone decides for bo, not ada's or
    /// cy's computers. After the Stop, the page asking for another pass, as
    /// a lend's origin does when its pass lapses, is given one naming no
    /// lend it reads. Red at 0573f402: no such parts, no Stop. Planted: the
    /// pass signed as before, whatever was said, and it still names the
    /// shelf.
    the_group_page_says_what_acts_as_you_and_its_stop_holds_at_the_next_pass
);
async fn the_group_page_says_what_acts_as_you_and_its_stop_holds_at_the_next_pass(engine: Engine) {
    let relay = relay().await;
    let bo = signing(7);
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &bo).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 2, "cy").await;
    let page = page_origin("<!doctype html><title>ada's page</title>").await;
    lend_reading(ada_dir.path(), "page", page.port(), &["k"]);
    let shelf = page_origin("<!doctype html><title>cy's shelf</title>").await;
    lend(cy_dir.path(), "shelf", shelf.port());

    let seen = drive(
        engine,
        "acts.mjs",
        json!({
            "site": site(),
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "label": label(1, "page"),
            "id": NS,
        }),
        None,
    )
    .await;
    eprintln!("the group page's run: {seen:#}");
    let reads = |at: &str| {
        serde_json::from_str::<Value>(seen[at]["body"].as_str().unwrap_or("{}")).unwrap_or_default()
            ["lends"]
            .clone()
    };
    assert_eq!(
        reads("yes")[0]["name"],
        "shelf",
        "the open is the yes:\n{seen:#}"
    );
    assert_eq!(
        seen["folded"], true,
        "\"What acts as you\" is folded under its heading: {seen:#}"
    );
    let before = seen["before"].as_str().unwrap_or_default();
    for part in [
        "What acts as you\n",
        "Opens friends’ things as you\n",
        "A Stop here holds from the next pass this phone signs for the app, within 12 hours.",
        &format!(
            "ada’s page (k) opens the k you chose, as you. Since {}.",
            today()
        ),
        "Knows who you are\n",
        "What you open knows your name.",
        "Can decide for you\n",
        "This phone.",
    ] {
        assert!(
            before.contains(part),
            "the section says {part:?}:\n{before}"
        );
    }
    let stopped = seen["stopped"].as_str().unwrap_or_default();
    assert!(
        stopped.contains("ada’s page (k) stops opening your friends’ k as you within 12 hours, when the pass it holds ends."),
        "the Stop says when it holds:\n{stopped}"
    );
    assert_eq!(
        reads("after"),
        json!([]),
        "the next pass names no lend it reads:\n{seen:#}"
    );
    drop((ada, cy));
}

in_webkit!(
    /// **A phone's page reading a lend past the rate is refused with the
    /// words** (EXTENSION_POINT §7, G2: "a refill at the lender's door, so
    /// phones and computers meet one check"; until now held below the
    /// browser alone, door/limit.rs `READS_A_MINUTE`). Ada lends a page
    /// reading kind `k`; cy lends a shelf of it. Bo's phone opens ada's page
    /// from the group's page, the yes, and the page reads cy's shelf at the
    /// address it is told as fast as it can: past 600 at once it is refused
    /// 429, `limit`, with a `Retry-After` and the words cy's door says to a
    /// computer, and cy's shelf was asked no more than were taken. Planted,
    /// watched failing: the door counting no reads (`lend_for` never asking
    /// `Reads::read`), and 1,000 reads were taken.
    a_phone_s_page_reading_past_the_rate_is_refused_with_the_words
);
async fn a_phone_s_page_reading_past_the_rate_is_refused_with_the_words(engine: Engine) {
    let relay = relay().await;
    let bo = signing(7);
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &bo).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 2, "cy").await;
    let page = page_origin("<!doctype html><title>ada's page</title>").await;
    lend_reading(ada_dir.path(), "page", page.port(), &["k"]);
    let (shelf, asked) = kept_page("cy's shelf, as text").await;
    lend(cy_dir.path(), "shelf", shelf.port());

    let seen = drive(
        engine,
        "read-rate.mjs",
        json!({
            "site": site(),
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "read": "shelf",
            "most": 1000,
        }),
        None,
    )
    .await;
    eprintln!(
        "the phone's reads: {}",
        json!({ "taken": seen["taken"], "status": seen["status"], "refused": seen["refused"], "retry": seen["retry"], "words": seen["words"], "error": seen["error"] })
    );
    let taken = seen["taken"].as_u64().unwrap_or_default();
    assert!(
        (600..1000).contains(&taken),
        "600 at once, then ten a second: {taken} taken"
    );
    assert_eq!(
        (seen["status"].clone(), seen["refused"].clone()),
        (json!(429), json!("limit")),
        "{seen:#}"
    );
    assert!(
        seen["retry"]
            .as_str()
            .is_some_and(|r| r.parse::<u64>().is_ok()),
        "{seen:#}"
    );
    assert_eq!(
        seen["words"],
        "This app is reading shelf faster than an app may, about 600 times a minute, because cy’s computer serves it. It can read again in 1 second. Read once and keep the answer, rather than on a timer.",
        "{seen:#}"
    );
    let served = asked.lock().unwrap().len() as u64;
    assert!(
        served <= taken,
        "cy's shelf asked {served} times, {taken} taken"
    );
    drop((ada, cy));
}
