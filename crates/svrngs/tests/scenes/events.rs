//! **Events** (demo/events.md; its run, demo/runs/events/2026-10-14/):
//! Sam hosts a dinner for two of Sam's groups, The House (Bo on his
//! computer, Cy on her phone) and dWeb (Di on his computer). Sam runs the
//! app, apps/events/server.mjs, lends it with `svrngs share`, and posts
//! the dinner once, to both. Bo says he's coming and brings Maya; Cy says
//! so from her phone, through the group's page; Di from his computer. At
//! the dinner Bo says he came, and that Maya came; Cy says she came, and
//! her page offers no "Maya came". Sam's page says, for each group, who
//! was invited, who is coming and who came.
//!
//! The invariant it relies on: **each group's answers are its own
//! people's** (demo/events.md, "What this is"; server.mjs, "Which group a
//! request is for"): Bo's page shows The House alone and Di's dWeb alone,
//! and only Sam, who hosts, sees both.

use serde_json::{json, Value};

use crate::harness::{relay_url, repo, Engine};
use crate::people::{Browsers, Scene};

/// The dinner, as the run typed it.
const DINNER: [(&str, &str); 3] = [
    ("what", "Dinner"),
    ("when", "Friday 7"),
    ("where", "the house"),
];

/// The app as `apps/events/` holds it, copied into the scene's place,
/// with the plant `SCENES_PLANT=events:<name>` names applied: the run's
/// own (run.py `PLANTS`), so a plant is the one the run watched fail,
/// its third line where server.mjs now writes it.
fn app(dir: &std::path::Path) -> std::path::PathBuf {
    let to = dir.join("app");
    std::fs::create_dir_all(&to).unwrap();
    for f in std::fs::read_dir(repo().join("apps/events")).unwrap() {
        let f = f.unwrap().path();
        if f.is_file() {
            std::fs::copy(&f, to.join(f.file_name().unwrap())).unwrap();
        }
    }
    let plants: &[(&str, &str, &str)] = match std::env::var("SCENES_PLANT").as_deref() {
        // Every page the host's view of every group this computer holds.
        Ok("events:cross") => &[
            (
                "server.mjs",
                "if (e.host === me) {",
                "if (true || e.host === me) {",
            ),
            (
                "server.mjs",
                "for (const g of groups.filter((x) => x.me || x.unread)) {",
                "for (const g of groups) {",
            ),
            (
                "server.mjs",
                "all.push({ group: g.id, answers: f.answers, me: g.me, complete: f.complete });",
                "all.push({ group: g.id, answers: f.answers, me: e.host, complete: f.complete });",
            ),
        ],
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

/// The groups' sections on a page `view`, by name, each its invited,
/// coming and came.
fn sections(view: &Value) -> Vec<(String, Value)> {
    let all = view["sections"].as_array().into_iter().flatten();
    all.map(|s| {
        (
            s["name"].as_str().unwrap_or_default().to_string(),
            json!([s["invited"], s["coming"], s["came"]]),
        )
    })
    .collect()
}

scene!(
    /// Planted, watched failing: the run's `cross` plant (run.py
    /// `PLANTS`), every page given the host's view of every group the
    /// computer holds, `SCENES_PLANT=events:cross`: Bo's page shows dWeb.
    events
);
async fn events(engine: Engine) {
    let scene = Scene::new("events", engine).await;
    let (page, _served) = scene.page().await;
    let mut browsers = Browsers::open(
        engine,
        &[
            ("sam", Engine::Chromium),
            ("bo", Engine::Chromium),
            ("di", Engine::Chromium),
            ("cy", Engine::Webkit),
        ],
    )
    .await;
    let at = relay_url(&scene.relay);

    // The House: Sam makes it; Bo joins by its link, Cy's phone by the page's.
    let sam = scene
        .start(
            "Sam",
            &[
                "new",
                "The House",
                "--as",
                "Sam",
                "--origin",
                &page,
                "--relay",
                &at,
            ],
        )
        .await;
    let house = sam.group_named("The House").await;
    let links = sam
        .on_home_page("/link", &json!({ "group": house, "device": false }))
        .await;
    let bo = scene
        .joins("Bo", links["client"].as_str().unwrap(), "The House")
        .await;
    let cy = browsers
        .tell(json!({ "do": "join", "who": "cy", "link": links["https"], "name": "Cy" }))
        .await;
    assert!(
        cy["status"]
            .as_str()
            .unwrap_or_default()
            .contains("in The House"),
        "Cy's phone joins: {cy:#}"
    );

    // dWeb: Sam's svrngs stops and makes it; Di joins.
    let sam = scene
        .again(
            sam,
            &[
                "new", "dWeb", "--as", "Sam", "--origin", &page, "--relay", &at,
            ],
        )
        .await;
    let dweb = sam.group_named("dWeb").await;
    let di = scene.joins("Di", &sam.invites(&dweb).await, "dWeb").await;

    // The app on Sam's computer, lent to both.
    let dir = sam.home.parent().unwrap().to_path_buf();
    let config = dir.join("app.json");
    let record = std::fs::read_to_string(sam.home.join(".svrngs/api")).unwrap();
    std::fs::write(
        &config,
        json!({ "record": record.trim().trim_end_matches('/'), "token": sam.home.join(".svrngs/token"), "who": "x-svrngs-key" }).to_string(),
    )
    .unwrap();
    let mut server = tokio::process::Command::new("node")
        .arg(app(&dir).join("server.mjs"))
        .arg(&config)
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs");
    use tokio::io::AsyncBufReadExt;
    let mut printed = tokio::io::BufReader::new(server.stdout.take().unwrap()).lines();
    let port: Value = serde_json::from_str(&printed.next_line().await.unwrap().unwrap()).unwrap();
    let port = port["port"].to_string();
    let shared = scene
        .command(
            &sam,
            &["share", &port, "--kind", "events", "--name", "events"],
            "\n",
        )
        .await;
    assert!(shared.contains("You lend events"), "{shared}");
    let at_sam = sam.lent_by(&house, "Sam", "events").await;
    let at_bo = bo.lent_by(&house, "Sam", "events").await;
    let at_di = di.lent_by(&dweb, "Sam", "events").await;

    // Sam posts the dinner once, to both groups.
    browsers
        .tell(json!({ "do": "goto", "who": "sam", "url": at_sam }))
        .await;
    let fill: serde_json::Map<_, _> = DINNER
        .iter()
        .map(|(k, v)| (format!("input[name={k}]"), json!(v)))
        .collect();
    let check = [
        format!("input[value='{house}']"),
        format!("input[value='{dweb}']"),
    ];
    let posted = browsers
        .tell(
            json!({ "do": "press", "who": "sam", "button": "Post", "fill": fill, "check": check }),
        )
        .await;
    let url = posted["url"].as_str().unwrap_or_default();
    let event = url
        .rsplit("/e/")
        .next()
        .filter(|_| url.contains("/e/"))
        .expect("the post lands on its event");
    let path = |group: &str| format!("g/{group}/e/{event}");

    // Bo is coming, and brings Maya; Cy, from her phone; Di.
    browsers
        .tell(json!({ "do": "goto", "who": "bo", "url": format!("{at_bo}{}", path(&house)) }))
        .await;
    browsers
        .tell(json!({ "do": "press", "who": "bo", "button": "I'm coming" }))
        .await;
    let maya = json!({ "do": "press", "who": "bo", "button": "Save", "fill": { "input[name=guests]": "Maya" } });
    browsers.tell(maya).await;
    let label = at_bo
        .trim_start_matches("http://")
        .split('.')
        .next()
        .unwrap()
        .to_string();
    let on_phone = format!(
        "{page}#open={label}&at={}",
        path(&house).replace('/', "%2F")
    );
    browsers
        .tell(json!({ "do": "goto", "who": "cy", "url": format!("{on_phone}") }))
        .await;
    browsers
        .tell(json!({ "do": "press", "who": "cy", "button": "I'm coming" }))
        .await;
    browsers
        .tell(json!({ "do": "goto", "who": "di", "url": format!("{at_di}{}", path(&dweb)) }))
        .await;
    browsers
        .tell(json!({ "do": "press", "who": "di", "button": "I'm coming" }))
        .await;

    // At the dinner: Bo came, and Maya; Cy came, with no "Maya came" hers.
    browsers
        .tell(json!({ "do": "press", "who": "bo", "button": "I came" }))
        .await;
    let bo_sees = browsers
        .tell(json!({ "do": "press", "who": "bo", "button": "Maya came" }))
        .await;
    let cy_before = browsers
        .tell(json!({ "do": "goto", "who": "cy", "url": on_phone }))
        .await;
    let cy_buttons = cy_before["buttons"].to_string();
    assert!(
        !cy_buttons.contains("Maya came"),
        "Cy's page offers no Maya came: {cy_before:#}"
    );
    let cy_sees = browsers
        .tell(json!({ "do": "press", "who": "cy", "button": "I came" }))
        .await;
    let di_sees = browsers
        .tell(json!({ "do": "goto", "who": "di", "url": format!("{at_di}{}", path(&dweb)) }))
        .await;

    // Each group's answers its own people's: Bo and Cy see The House
    // alone, Di dWeb alone.
    for (who, seen, only) in [
        ("Bo", &bo_sees, "The House"),
        ("Cy", &cy_sees, "The House"),
        ("Di", &di_sees, "dWeb"),
    ] {
        let named: Vec<_> = sections(seen).into_iter().map(|(n, _)| n).collect();
        assert_eq!(named, [only], "{who} sees {only} alone: {seen:#}");
        let invited = &sections(seen)[0].1[0];
        let theirs = if only == "dWeb" {
            json!(["Di"])
        } else {
            json!(["Bo", "Cy"])
        };
        assert_eq!(
            invited, &theirs,
            "{who} sees {only}'s people alone: {seen:#}"
        );
    }

    // The next morning, Sam's page: each group in its own part.
    let sam_sees = browsers
        .tell(json!({ "do": "goto", "who": "sam", "url": format!("{at_sam}{}", path(&house)) }))
        .await;
    let parts = sections(&sam_sees);
    let part = |name: &str| {
        parts
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p.clone())
    };
    // Each answer its own person's, signed by their own computer or phone
    // as the app on Sam's computer named it (capability-signed, F1), so
    // none is said to come through Sam's computer.
    let house_part = json!([
        ["Bo", "Cy"],
        ["Bo + Maya", "Cy"],
        ["Bo", "Cy", "Maya, Bo's guest"]
    ]);
    assert_eq!(
        part("The House"),
        Some(house_part),
        "Sam sees The House: {sam_sees:#}"
    );
    assert_eq!(
        part("dWeb"),
        Some(json!([["Di"], ["Di"], []])),
        "Sam sees dWeb: {sam_sees:#}"
    );
    drop((sam, bo, di, server));
}
