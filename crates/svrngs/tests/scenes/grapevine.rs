//! **Grapevine** (demo/grapevine.md; its run, demo/runs/grapevine/
//! 2026-10-16/): Sam and Jo are 1400 Page; Jo and Kim are Waller block.
//! Each runs their own grapevine, apps/grapevine/server.mjs, on their own
//! computer. Kim lists her tile saw; Sam asks for one, one ring away; Jo
//! carries the ask to Waller block with one tap; Kim offers hers. Sam reads
//! the offer, sealed; Jo's page says only that a sealed message went by.
//! Both say yes, and only then see each other's names. Sam takes the ask
//! back, and it leaves Kim's page too.
//!
//! The invariant it relies on: **Jo passes their messages on and can't
//! read them** (demo/grapevine.md, "What everyone in a group sees"): no
//! word of the offer or of either name is in Jo's copy of either group's
//! record, though every reply went through it.

use serde_json::{json, Value};

use crate::harness::{repo, Engine};
use crate::people::{late, Browsers, Person, Scene};

/// What only Sam and Kim may read: the offer, and their names.
const SEALED: [&str; 4] = [
    "Saturday morning",
    "Waller and Pierce",
    "Sam Reyes",
    "Kim Okafor",
];
const NAME: &str = "Your name, which they’ll see once you’ve both said yes";

/// The app as `apps/grapevine/` holds it, copied into the scene's place
/// whole, with the plant `SCENES_PLANT=grapevine:<name>` names applied:
/// the run's own (run.py `PLANTS`).
fn app(dir: &std::path::Path) -> std::path::PathBuf {
    let to = dir.join("grapevine");
    let copied = std::process::Command::new("cp")
        .arg("-R")
        .arg(repo().join("apps/grapevine"))
        .arg(&to)
        .status()
        .unwrap();
    assert!(copied.success());
    let plants: &[(&str, &str, &str)] = match std::env::var("SCENES_PLANT").as_deref() {
        // Every reply written as its words, not sealed.
        Ok("grapevine:unsealed") => &[(
            "seal.mjs",
            "return { nonce: b64(nonce), box: b64(nacl.box(text, nonce, them, mine)) };",
            "return { nonce: b64(nonce), box: JSON.stringify(body) };",
        )],
        _ => &[],
    };
    for (file, old, new) in plants {
        let text = std::fs::read_to_string(to.join(file)).unwrap();
        assert_eq!(
            text.matches(old).count(),
            1,
            "the plant's text is in {file} once: {old}"
        );
        std::fs::write(to.join(file), text.replace(old, new)).unwrap();
    }
    to
}

/// `who`'s grapevine started on their computer, as step 4's two lines
/// start it: its address, and the program, killed on drop.
async fn started(app: &std::path::Path, who: &Person) -> (String, tokio::process::Child) {
    let record = std::fs::read_to_string(who.home.join(".svrngs/api")).unwrap();
    let config = who.home.join("grapevine.json");
    let said = json!({ "record": record.trim(), "data": who.home });
    std::fs::write(&config, said.to_string()).unwrap();
    let mut server = tokio::process::Command::new("node")
        .arg(app.join("server.mjs"))
        .arg(&config)
        .env("HOME", &who.home)
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs");
    use tokio::io::AsyncBufReadExt;
    let mut printed = tokio::io::BufReader::new(server.stdout.take().unwrap()).lines();
    let first: Value = serde_json::from_str(&printed.next_line().await.unwrap().unwrap()).unwrap();
    tokio::spawn(async move { while let Ok(Some(_)) = printed.next_line().await {} });
    (first["at"].as_str().unwrap().to_string(), server)
}

/// One of the run's commands (doc.py `tell`): an `await` met, a `press`
/// found its button and was not refused; else the scene fails, with the
/// page's words.
async fn tell(browsers: &mut Browsers, command: Value) -> Value {
    let got = browsers.tell(command.clone()).await;
    let text = got["text"].as_str().unwrap_or_default();
    if command["do"] == "await" && got["met"] != true {
        late(format!(
            "{command}: not within its bound, as demo/grapevine.md says: {text}"
        ))
        .await;
    }
    let ok = command["do"] != "press" || (got["absent"] != true && got["said"].is_null());
    assert!(
        ok,
        "{command}: not as demo/grapevine.md says: {}",
        got["text"].as_str().unwrap_or_default()
    );
    got
}

/// Every act in `who`'s copy of `group`'s record, as its words.
async fn record(who: &Person, group: &str) -> String {
    let log = who.get(&format!("/v1/rail/log?namespace={group}")).await;
    assert!(
        log["ops"].is_array(),
        "{}'s record of {group}: {log}",
        who.name
    );
    log.to_string()
}

/// Every reply went through Jo: no word of `sealed` in Jo's copy of any
/// of `groups`, and none on Jo's page, its closed parts too.
async fn unread(browsers: &mut Browsers, jo: &Person, groups: &[&str], sealed: &[&str]) {
    let shown = tell(browsers, json!({ "do": "await", "who": "jo", "words": [] })).await;
    for group in groups {
        let held = record(jo, group).await;
        for word in sealed {
            assert!(!held.contains(word), "Jo's copy of {group} holds {word:?}");
            assert!(
                !shown["all"].to_string().contains(word),
                "Jo's page says {word:?}"
            );
        }
    }
}

scene!(
    /// Planted, watched failing: the run's `unsealed` plant (run.py
    /// `PLANTS`), every reply written as its words, `SCENES_PLANT=
    /// grapevine:unsealed`: Jo's copy of Waller block holds Kim's offer.
    grapevine
);
async fn grapevine(engine: Engine) {
    let scene = Scene::new("grapevine", engine).await;
    let mut browsers = Browsers::driven(
        "demo/runs/grapevine/2026-10-16/people.mjs",
        engine,
        &[
            ("sam", Engine::Chromium),
            ("jo", Engine::Chromium),
            ("kim", Engine::Chromium),
        ],
    )
    .await;
    let at = crate::harness::relay_url(&scene.relay);

    // 1-3. Sam makes 1400 Page; Jo joins it, `--no-run`, and makes
    // Waller block; Kim joins Waller block.
    let (sam, page) = scene.founds("Sam", "1400 Page").await;
    let link = sam.invites(&page).await;
    let joined = scene
        .once("Jo", &["join", &link, "--as", "Jo", "--no-run"])
        .await;
    assert!(joined.contains("You're in 1400 Page, as Jo."), "{joined}");
    let jo = scene
        .start("Jo", &["new", "Waller block", "--as", "Jo", "--relay", &at])
        .await;
    let waller = jo.group_named("Waller block").await;
    jo.group_named("1400 Page").await;
    let kim = scene
        .joins("Kim", &jo.invites(&waller).await, "Waller block")
        .await;

    // 4. Each person's grapevine, each page that person's groups alone.
    let app = app(sam.home.parent().unwrap());
    let mut running = vec![];
    for (who, theirs) in [("sam", &sam), ("jo", &jo), ("kim", &kim)] {
        let (url, server) = started(&app, theirs).await;
        running.push(server);
        let got = tell(
            &mut browsers,
            json!({ "do": "goto", "who": who, "url": url }),
        )
        .await;
        let groups: Vec<&str> = match who {
            "sam" => vec!["1400 Page"],
            "jo" => vec!["1400 Page", "Waller block"],
            _ => vec!["Waller block"],
        };
        let mut shown: Vec<_> = got["groups"].as_array().cloned().unwrap_or_default();
        shown.sort_by_key(|g| g.to_string());
        assert_eq!(shown, groups, "{who}'s page shows their groups alone");
    }

    // 5. Kim lists her tile saw; 6. Sam asks, one ring away.
    tell(&mut browsers, json!({ "do": "press", "who": "kim", "within": [["section", "What you have"], ["div.kind", "tool"]],
        "fill": { "What": "tile saw" }, "select": { "In": "Waller block" }, "button": "List it" })).await;
    let asked = tell(&mut browsers, json!({ "do": "press", "who": "sam", "within": [["section", "Ask your groups"], ["div.kind", "tool"]],
        "fill": { "What": "tile saw", "When": "Saturday" }, "select": { "In": "1400 Page", "How far": "One ring away" }, "button": "Ask" })).await;
    let text = asked["text"].as_str().unwrap_or_default();
    assert!(text.contains("You asked: tile saw, Saturday"), "{text}");

    // 7. Jo carries it, told what Waller block will see.
    tell(&mut browsers, json!({ "do": "await", "who": "jo", "words": ["Sam asks: tile saw, Saturday", "Carry to Waller block",
        "Everyone in Waller block will see what was asked, and that you carried it; never who asked."] })).await;
    tell(&mut browsers, json!({ "do": "press", "who": "jo", "within": [["li.ask", "tile saw"]], "button": "Carry to Waller block" })).await;

    // 8. Kim offers, carried by Jo and never said whose.
    let kim_sees = tell(&mut browsers, json!({ "do": "await", "who": "kim",
        "words": ["A neighbour of Jo’s needs: tile saw, Saturday — carried by Jo", "You might have this."] })).await;
    assert!(
        !kim_sees["text"].to_string().contains("Sam"),
        "Kim's page never says Sam before her yes"
    );
    tell(
        &mut browsers,
        json!({ "do": "press", "who": "kim", "within": [["li.ask", "tile saw"]],
        "fill": { "When": "Saturday morning", "Where": "Waller and Pierce" }, "button": "Offer" }),
    )
    .await;

    // 9. Jo's page says a sealed message went by, and its size, and Jo's
    // copy of each group holds none of what it says; Sam reads the offer.
    let jo_sees = json!({ "do": "await", "who": "jo", "words": ["A sealed message went by toward the person who asked"] });
    tell(&mut browsers, jo_sees.clone()).await;
    unread(&mut browsers, &jo, &[&page, &waller], &SEALED[..2]).await;
    let sam_sees = json!({ "do": "await", "who": "sam",
        "words": ["Someone who saw it offered: when Saturday morning; where Waller and Pierce", "Sealed: only you two can read this."] });
    tell(&mut browsers, sam_sees).await;

    // 10. Both say yes; only then each sees the other's name.
    tell(&mut browsers, json!({ "do": "press", "who": "sam", "within": [["li.thread", "About your ask"]], "fill": { NAME: "Sam Reyes" }, "button": "Yes" })).await;
    let got = tell(
        &mut browsers,
        json!({ "do": "await", "who": "kim", "words": ["The person who asked said yes"] }),
    )
    .await;
    assert!(
        !got["text"].to_string().contains("Sam Reyes"),
        "Kim's page names Sam before her own yes"
    );
    tell(&mut browsers, json!({ "do": "press", "who": "kim", "within": [["li.thread", "which you offered to help with"]], "fill": { NAME: "Kim Okafor" }, "button": "Yes" })).await;
    tell(
        &mut browsers,
        json!({ "do": "await", "who": "kim", "words": ["You both said yes: they’re Sam Reyes"] }),
    )
    .await;
    tell(
        &mut browsers,
        json!({ "do": "await", "who": "sam", "words": ["they’re Kim Okafor"] }),
    )
    .await;

    // Their yeses went through Jo too, and Jo's copy holds neither name.
    unread(&mut browsers, &jo, &[&page, &waller], &SEALED).await;

    // 11. Sam takes it back, and it leaves Kim's page within a turn of Jo's.
    tell(&mut browsers, json!({ "do": "press", "who": "sam", "within": [["li.ask", "tile saw"]], "button": "Take it back" })).await;
    tell(
        &mut browsers,
        json!({ "do": "await", "who": "kim", "gone": ["needs: tile saw, Saturday"] }),
    )
    .await;

    // 12. Jo's svrngs restarts, as an upgrade or a crash restarts it, and
    // Sam asks again at once: the ask reaches Jo's copy within a restart's
    // bound, not at Sam's next round (`DEFAULT_RING_SYNC_INTERVAL`).
    let jo = scene.again(jo, &[]).await;
    tell(&mut browsers, json!({ "do": "press", "who": "sam", "within": [["section", "Ask your groups"], ["div.kind", "tool"]],
        "fill": { "What": "ladder", "When": "Sunday" }, "select": { "In": "1400 Page", "How far": "One ring away" }, "button": "Ask" })).await;
    let t0 = std::time::Instant::now();
    while !record(&jo, &page).await.contains("ladder") {
        if t0.elapsed() >= RESTARTED_WITHIN {
            let log = sam.log();
            late(format!("Sam's ask is not in Jo's copy {RESTARTED_WITHIN:?} after Jo restarted; Sam's log is {}", log.display())).await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    drop((running, sam, jo, kim));
}

/// How soon an append reaches a member whose svrngs restarted: a request
/// on the connection kept to its old self fails within `DIAL_WITHIN` of
/// silence (dial.rs `held`), the one dialled again answers within
/// `DIAL_WITHIN`, and the exchange on it within `ANSWERED_WITHIN`, well
/// inside the round's 60 s.
const RESTARTED_WITHIN: std::time::Duration = std::time::Duration::from_secs(
    2 * svrngs_node::dial::DIAL_WITHIN.as_secs() + svrngs_node::iroh::ANSWERED_WITHIN.as_secs(),
);
