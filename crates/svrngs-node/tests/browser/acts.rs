//! "What acts as you" (capability-what-acts; svrn-docs
//! `rings/current/WHAT_CAN_ACT_AS_YOU.md` §9, "One place"): the home page
//! and the group page each show one section, grouped by what each thing
//! can do as the person, each line saying what, since when, and its Stop.
//! What it protects: a yes the person gave is something they can see and
//! take back, and a Stop holds — on a computer from the app's next
//! request, on a phone from the next pass it signs, within 12 hours.
//! Nobody else's computer is listed as deciding for you (§4).

use super::*;
use svrngs_node::door::limit::READS_A_MINUTE;

/// "3 Oct", as question.js `day` says a day in this computer's time.
fn today() -> String {
    chrono::Local::now().format("%-d %b").to_string()
}

// Pure wording/decider vectors are web/tests/question.test.mjs's; below, real
// browsers and native resource owners establish that consent and Stop take effect.

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
    /// computer, and cy's shelf was asked no more than were taken. Past 600,
    /// it takes no more than ten a second of the run let in: judged by the
    /// run's length, since a page reading slower than 25 a second takes
    /// 1,000 in 40 s, which the rate lets in whole (`READS_A_MINUTE`; on the
    /// Fedora gate image, WebKit's 1,000 reads were all taken while the host
    /// was loaded, demo/runs/flakes/2026-11-12). A page that never read
    /// faster than the rate could not be refused: could-not-judge, never a
    /// pass. Planted, watched failing: the door counting no reads
    /// (`lend_for` never asking `Reads::read`), and 2,000 reads were taken.
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
            "most": 2000,
        }),
        None,
    )
    .await;
    eprintln!(
        "the phone's reads: {}",
        json!({ "taken": seen["taken"], "ms": seen["ms"], "status": seen["status"], "refused": seen["refused"], "retry": seen["retry"], "words": seen["words"], "error": seen["error"] })
    );
    let taken = seen["taken"].as_u64().unwrap_or_default();
    // What the rate lets in over the run: its burst, then `READS_A_MINUTE`
    // a minute over the run's ms, and the read under way.
    let burst = u64::from(READS_A_MINUTE);
    let ms = seen["ms"].as_u64().unwrap_or(u64::MAX);
    let let_in = burst + ms.saturating_mul(burst) / 60_000 + 1;
    assert!(
        (burst..=let_in).contains(&taken),
        "600 at once, then ten a second: {taken} taken in {ms} ms, of {let_in} let in"
    );
    if seen["status"].is_null() && seen["error"].is_null() {
        // Never refused, and no more taken than the rate let in: the page
        // read no faster than the rate, so it could not be past it.
        return could_not_judge(&format!(
            "acts::a_phone_s_page_reading_past_the_rate_is_refused_with_the_words::{} \
             the page read {taken} in {ms} ms, no faster than {burst} a minute past {burst} \
             at once, so no read could be refused",
            engine.as_str()
        ));
    }
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
