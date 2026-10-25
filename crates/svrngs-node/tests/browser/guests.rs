//! A visit made with a link in any chat (guests-link; svrn-docs
//! `rings/current/GUESTS.md` §3.9, §5 "How a visit is made"; the operator,
//! 2026-10-18, "A link in any chat"). The browser half is
//! `web/tests/visit.mjs`.

use super::*;

/// Whose computer Kim's is, Bo's and Cy's, and the key of a group of Sam's
/// own, kept on Sam's page.
const KIM: u8 = 1;
const BO: u8 = 7;
const CY: u8 = 8;
const OWN: u8 = 9;

in_both_engines!(
    /// **A visit by a link** (guests-link (a)). Kim's computer holds house,
    /// Kim and Bo in it, and lends Kim's Jellyfin, a page and its film; the
    /// group keeps the page's origin, so its links open there. On Kim's home
    /// page Kim taps **Have a guest**, keeps the 48 hours it starts at, and
    /// gets a link to send in any chat. Sam opens it in a browser holding
    /// nothing of any group, is asked their name once, and opens Kim's
    /// Jellyfin from what the page lists: the film plays, and Jellyfin is
    /// told "Sam (Kim's guest)" on every request, never Kim. Kim's record
    /// holds one visit, Sam's, of 48 hours from when Sam opened the link.
    ///
    /// And the row's plants (guests-link (b)): no length past 48 hours is
    /// offered, and one asked of `/link` anyway lets Ana in nowhere; Sam's
    /// page, holding a group of Sam's own, makes the visit's key fresh and
    /// keeps the visit apart from that group, shows it again on the link
    /// opened again, and says when it has ended; the link lets Ana in
    /// after Sam nowhere; and its secret never leaves the fragment. Cy is a
    /// third member, so the house's bound leaves room for both guests.
    sam_opens_the_link_kim_sent_and_watches_kims_film_as_kims_guest
);
async fn sam_opens_the_link_kim_sent_and_watches_kims_film_as_kims_guest(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let kim = computer(dir.path(), &relay, &signing(BO), KIM, "Kim").await;
    let cy = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("Cy").unwrap(),
            key: actor_of(&signing(CY)),
            invite: None,
        },
    };
    kim.rail().commit(NS, &cy).unwrap().unwrap();
    let mp4 = dir.path().join("title.mp4");
    let size = film(&mp4);
    let (film_origin, film_heads) = origins::kept_range_origin(mp4, size).await;
    let html = "<!doctype html><title>Jellyfin</title>\
                <video src=\"title.mp4\" playsinline preload=\"auto\"></video>";
    let jellyfin = one_origin(page_origin(html).await, film_origin).await;
    lend(dir.path(), "jellyfin", jellyfin.port());

    let mut half = tokio::process::Command::new("node")
        .arg(repo().join("web/tests/visit.mjs"))
        .arg(json!({ "engine": engine.as_str(), "site": site() }).to_string())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs: the browser tests need Node 18 or later");
    use tokio::io::{AsyncBufReadExt, AsyncReadExt};
    let mut out = tokio::io::BufReader::new(half.stdout.take().unwrap()).lines();
    let first = out
        .next_line()
        .await
        .unwrap()
        .expect("the browser half prints its origin");
    let origin: Value = serde_json::from_str(&first).unwrap();
    let origin = origin["origin"].as_str().unwrap().to_string();
    // The group's links open on the page the browser half serves, as
    // `svrngs new --origin` keeps it.
    let group = kim.rail().group_dir(NS).unwrap();
    std::fs::write(group.join(svrngs_node::invite::ORIGIN_FILE), &origin).unwrap();
    let home = format!("http://svrngs.localhost:{}/", kim.api_addr().port());
    let mut stdin = half.stdin.take().unwrap();
    let own = hex::encode(signing(OWN).to_bytes());
    let told =
        json!({ "home": home, "name": "Sam", "open": "jellyfin", "group": NS, "ownKey": own });
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
    half.stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .await
        .unwrap();
    let status = half.wait().await.unwrap();
    assert!(
        status.success(),
        "the browser half failed ({status}):\n{stderr}"
    );
    let seen: Value = serde_json::from_str(&last.expect("the browser half printed")).unwrap();
    let mut shown = seen.clone();
    shown["logs"] = json!(format!(
        "{} lines",
        seen["logs"].as_array().map_or(0, Vec::len)
    ));
    eprintln!("the visit's run: {seen:#}");

    // Kim kept the length the page chose to start, of none past 48 hours,
    // and got a link on the group's page, its secret in the fragment, which
    // no request to the page carried.
    assert_eq!(seen["length"], "48 hours", "{shown:#}");
    let hours = |l: &Value| (l.as_str()?.strip_suffix(" hours")?.parse::<i64>()).ok();
    let lengths = seen["lengths"].as_array().cloned().unwrap_or_default();
    assert!(
        !lengths.is_empty() && lengths.iter().all(|l| hours(l).is_some_and(|h| h <= 48)),
        "{shown:#}"
    );
    assert_eq!(seen["secretAsked"], json!([]), "{shown:#}");
    let link = seen["link"].as_str().unwrap_or_default();
    // A guest's link, `<page>#visit=<token>`: its token a join link's.
    assert!(link.starts_with(&format!("{origin}#visit=")), "{shown:#}");
    let token = svrngs_core::link::Link::parse(&link.replacen("#visit=", "#", 1))
        .expect("the link shown is a link")
        .token;
    assert_eq!(token.issuer, signing(KIM).verifying_key(), "{shown:#}");

    // Sam asked their name once, and in as Kim's guest.
    assert_eq!(seen["askedAgain"], false, "{shown:#}");
    assert!(
        seen["status"]
            .as_str()
            .is_some_and(|s| s.contains("Kim’s guest")),
        "{shown:#}"
    );
    assert!(
        at_origin(&seen["opened"], &label(KIM, "jellyfin")),
        "{shown:#}"
    );
    assert!(
        seen["first"]["ms"].is_u64(),
        "Kim's film must play for Sam:\n{shown:#}"
    );

    // Jellyfin told Sam, as Kim's guest, never Kim.
    let heads = film_heads.lock().unwrap().clone();
    assert!(!heads.is_empty(), "{shown:#}");
    for head in &heads {
        assert_eq!(
            told_in(head, "x-svrngs-person").as_deref(),
            Some("Sam (Kim's guest)"),
            "{head}"
        );
        assert_ne!(
            told_in(head, "x-svrngs-key"),
            Some(actor_of(&signing(KIM))),
            "{head}"
        );
    }

    // Ana, at a link of 72 hours, is let in nowhere; at Sam's link after
    // Sam, the same. Sam, at the link again, is shown the visit again, then
    // told it has ended, never asked again; Sam's page keeps the group of
    // Sam's own alone.
    let refused = "This link lets nobody in. Ask for a new one.";
    assert_eq!(seen["anaLong"], refused, "{shown:#}");
    assert_eq!(seen["anaAgain"], refused, "{shown:#}");
    assert_eq!(seen["again"]["asked"], false, "{shown:#}");
    assert_eq!(seen["again"]["status"], seen["status"], "{shown:#}");
    assert_eq!(seen["ended"]["asked"], false, "{shown:#}");
    assert!(
        seen["ended"]["status"]
            .as_str()
            .is_some_and(|s| s.starts_with("Your visit as Kim’s guest ended")),
        "{shown:#}"
    );
    assert_eq!(seen["groups"], json!(["own"]), "{shown:#}");

    // One visit in Kim's record, Sam's, 48 hours from its act, its key
    // made for it, never the key of Sam's own group.
    let register = kim.rail().register(NS).unwrap();
    let visits: Vec<_> = register.visits.values().collect();
    assert_eq!(visits.len(), 1, "{visits:#?}");
    let sam = visits[0];
    assert_eq!(sam.visit.guest.as_str(), "Sam", "{sam:#?}");
    assert_ne!(sam.visit.key, actor_of(&signing(OWN)), "{sam:#?}");
    assert_eq!(sam.host, actor_of(&signing(KIM)), "{sam:#?}");
    let length = sam.visit.until - sam.at;
    assert!(
        (svrngs_core::VISIT_LONGEST - 5..=svrngs_core::VISIT_LONGEST).contains(&length),
        "48 hours from when Sam opened the link, not {length} s: {sam:#?}"
    );
    assert!(
        heads
            .iter()
            .all(|h| told_in(h, "x-svrngs-key") == Some(sam.visit.key.clone())),
        "Jellyfin told Sam's visit key: {heads:#?}"
    );
    drop(kim);
}

/// Sam's visit key, in the record alone: no page of Sam's runs here.
const SAM: u8 = 10;

/// Kim's computer, holding house, Kim and Bo in it, and Sam visiting as
/// Kim's guest until 40 hours from now, written as Kim's `/visit` writes
/// it; and Bo's computer, Kim's record synced to it. The visit's id, and
/// the day and time it ends, as a page in this computer's own time says it.
async fn sam_visiting(kim_dir: &Path, bo_dir: &Path, relay: &Server) -> (Node, Node, String) {
    let kim = computer(kim_dir, relay, &signing(BO), KIM, "Kim").await;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    let until = now.unwrap().as_secs() as i64 + 40 * 60 * 60;
    let visit = svrngs_core::Visit {
        guest: Person::parse("Sam").unwrap(),
        key: actor_of(&signing(SAM)),
        opens: Default::default(),
        until,
    };
    kim.rail().commit(NS, &visit).unwrap().unwrap();
    let bo = standing_under(bo_dir, relay, &kim, BO, "bo", "Kim").await;
    use chrono::TimeZone;
    let day = chrono::Local.timestamp_opt(until, 0).unwrap();
    (kim, bo, day.format("%A").to_string())
}

/// Who each home page lists as visiting, and as its own guests.
const LISTED: &str = "#home .yours li, #home .visiting li";

in_both_engines!(
    /// **Who's visiting, on every home page** (guests-pages (a); GUESTS
    /// §3.2, §5 "The pages", §7 bar 7). Kim hosts Sam until a day and an
    /// hour 40 hours from now. Bo's home page lists Sam visiting, Kim's
    /// guest, until then; Kim's lists Sam under Who you have over too, with
    /// **Stop**, which says what it does before it does it. Kim presses
    /// it: Kim's page, opened again, lists nobody; Bo's computer syncs
    /// Kim's record, and Bo's lists nobody either. Planted: the visit kept
    /// off the record, so only its host's page lists it; Stop taking the
    /// row off Kim's page alone.
    every_home_page_shows_sam_visiting_and_kims_stop_ends_it
);
async fn every_home_page_shows_sam_visiting_and_kims_stop_ends_it(engine: Engine) {
    let relay = relay().await;
    let (kim_dir, bo_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (kim, bo, day) = sam_visiting(kim_dir.path(), bo_dir.path(), &relay).await;
    let visiting = format!("Sam, Kim’s guest, until {day}");
    let listed = |seen: &Value| -> Vec<String> {
        let all = seen.as_array().cloned().unwrap_or_default();
        all.iter()
            .map(|l| l.as_str().unwrap().to_string())
            .collect()
    };

    let bos = home_page(engine, &bo, json!({ "all": LISTED })).await;
    eprintln!("Bo's home page: {bos:#}");
    let on_bos = listed(&bos["all"]);
    assert!(
        on_bos.len() == 1 && on_bos[0].starts_with(&visiting),
        "Bo's home page lists Sam visiting, Kim's guest, until {day}: {bos:#}"
    );

    let stop = json!([["#home .yours button"]]);
    let kims = home_page(engine, &kim, json!({ "all": LISTED, "do": stop })).await;
    eprintln!("Kim's home page: {kims:#}");
    let on_kims = listed(&kims["all"]);
    assert!(
        on_kims.len() == 2
            && on_kims[0].starts_with(&format!("Sam, until {day}"))
            && on_kims[0].contains("Stop ends the visit now")
            && on_kims[1].starts_with(&visiting),
        "Kim's home page lists Sam as Kim's guest, with Stop, and visiting: {kims:#}"
    );
    assert_eq!(listed(&kims["after"][0]), Vec::<String>::new(), "{kims:#}");
    let register = kim.rail().register(NS).unwrap();
    assert!(
        register.visits.values().all(|g| g.stopped.is_some()),
        "Kim's Stop is in Kim's record: {:#?}",
        register.visits
    );

    let synced = bo.host().sync_with(NS, &kim.endpoint().addr()).await;
    assert!(synced.stop.is_none(), "Bo syncs Kim's record: {synced:?}");
    let bos = home_page(engine, &bo, json!({ "all": LISTED })).await;
    eprintln!("Bo's home page after Kim's Stop: {bos:#}");
    assert_eq!(listed(&bos["all"]), Vec::<String>::new(), "{bos:#}");
    drop((kim, bo));
}

/// Ana's phone, in house beside Kim and Bo.
const ANA: u8 = 11;

/// Who the phone's page lists as visiting, opened by a group's link that
/// Bo's computer made, as Ana.
async fn on_anas_phone(engine: Engine, relay: &Server) -> Value {
    let args = json!({
        "site": site(),
        "key": hex::encode(signing(ANA).to_bytes()),
        "token": token(&signing(BO), relay),
        "person": "Ana",
    });
    let seen = drive(engine, "drive.mjs", args, None).await;
    eprintln!("Ana's phone: {seen:#}");
    assert_eq!(seen["status"], "You’re in house, as Ana", "{seen:#}");
    seen
}

in_both_engines!(
    /// **Who's visiting, on the phone's page too** (guests-pages (b);
    /// GUESTS §5 "The pages", §7 bar 7). Kim hosts Sam, as in (a), and Ana
    /// is in house by her phone, whose page reaches Bo's computer: it lists
    /// Sam visiting, Kim's guest, until then. Kim presses Stop on Kim's
    /// home page; Bo's computer syncs Kim's record, and Ana's page, opened
    /// again, lists nobody. Planted: the visit kept off the record, so only
    /// its host's computer lists it; Stop taking the row off Kim's page
    /// alone.
    the_phones_page_shows_sam_visiting_and_kims_stop_ends_it
);
async fn the_phones_page_shows_sam_visiting_and_kims_stop_ends_it(engine: Engine) {
    let relay = relay().await;
    let (kim_dir, bo_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ana = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("Ana").unwrap(),
            key: actor_of(&signing(ANA)),
            invite: None,
        },
    };
    let (kim, bo, day) = sam_visiting(kim_dir.path(), bo_dir.path(), &relay).await;
    kim.rail().commit(NS, &ana).unwrap().unwrap();
    let synced = bo.host().sync_with(NS, &kim.endpoint().addr()).await;
    assert!(
        synced.stop.is_none(),
        "Bo syncs Ana's admission: {synced:?}"
    );

    let seen = on_anas_phone(engine, &relay).await;
    let visiting = format!("Sam, Kim’s guest, until {day}");
    let on = seen["visiting"].as_array().cloned().unwrap_or_default();
    assert!(
        on.len() == 1 && on[0].as_str().is_some_and(|l| l.starts_with(&visiting)),
        "Ana's phone lists Sam visiting, Kim's guest, until {day}: {seen:#}"
    );

    let stop = json!({ "all": LISTED, "do": [["#home .yours button"]] });
    let kims = home_page(engine, &kim, stop).await;
    assert_eq!(kims["after"][0], json!([]), "{kims:#}");
    let synced = bo.host().sync_with(NS, &kim.endpoint().addr()).await;
    assert!(synced.stop.is_none(), "Bo syncs Kim's Stop: {synced:?}");
    let seen = on_anas_phone(engine, &relay).await;
    assert_eq!(seen["visiting"], json!([]), "{seen:#}");
    drop((kim, bo));
}
