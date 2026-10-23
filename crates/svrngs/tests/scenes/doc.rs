//! **Doc** (apps/doc/README.md; its run, demo/runs/doc/2026-10-20/): Alex
//! lends the doc from her computer, `node apps/doc/server.mjs` and `svrngs
//! share --kind doc`, its first line hers. Bo opens it on his computer, at
//! the address his svrngs lists; Cy on her phone, joining by the group
//! page's link and pressing "doc" there. Each page says who they are, and
//! that both are here. They write at once, a line each, and every page
//! holds both lines: one doc. Each paragraph's note names who wrote it.
//!
//! The invariant it relies on: **each edit is its socket's person's, as
//! svrngs's door named them** (server.mjs, "An edit from a page"; door.rs
//! `x-svrngs-person`): Bo's line is noted as Bo's and Cy's as Cy's, on
//! every page, and what the server kept says so.

use serde_json::{json, Value};

use crate::harness::{relay_url, repo, Engine};
use crate::people::{Browsers, Scene};

const FIRST: &str = "Saturday's dinner";
const LINES: [(&str, &str); 2] = [("bo", "Bo brings chips"), ("cy", "Cy brings drinks")];

/// The app as `apps/doc/` holds it, copied into the scene's place whole,
/// with the plant `SCENES_PLANT=doc:<name>` names applied.
fn app(dir: &std::path::Path) -> std::path::PathBuf {
    let to = dir.join("doc");
    let copied = std::process::Command::new("cp")
        .arg("-R")
        .arg(repo().join("apps/doc"))
        .arg(&to)
        .status()
        .unwrap();
    assert!(copied.success());
    let plants: &[(&str, &str, &str)] = match std::env::var("SCENES_PLANT").as_deref() {
        // Every edit written as the first page's person's, whoever's socket
        // carried it.
        Ok("doc:first") => &[(
            "server.mjs",
            "person: origin.person, update",
            "person: [...pages][0].person, update",
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

/// One of the run's commands (agent.mjs); a wait that ran out, or a
/// command that failed, fails the scene with what the page read.
async fn tell(browsers: &mut Browsers, command: Value) -> Value {
    let got = browsers.tell(command.clone()).await;
    let waited = matches!(command["do"].as_str(), Some("has" | "says"));
    assert!(
        got.get("failed").is_none() && !(waited && got["at"].is_null()),
        "{command}: not as the doc's run says: {got:#}"
    );
    got
}

in_both_engines!(
    /// Planted, watched failing: every edit written as the first page's
    /// person's, `SCENES_PLANT=doc:first`: Cy's line is noted as Bo's.
    doc
);
async fn doc(engine: Engine) {
    let scene = Scene::new("doc", engine).await;
    let (page, _served) = scene.page().await;
    let mut browsers = Browsers::driven("demo/runs/doc/2026-10-20/agent.mjs", engine, &[]).await;
    let at = relay_url(&scene.relay);

    // Alex makes the house and lends the doc; Bo joins on his computer.
    let alex = scene
        .start(
            "Alex",
            &[
                "new", "House", "--as", "Alex", "--origin", &page, "--relay", &at,
            ],
        )
        .await;
    let house = alex.group_named("House").await;
    let links = alex
        .on_home_page("/link", &json!({ "group": house, "device": false }))
        .await;
    let bo = scene
        .joins("Bo", links["client"].as_str().unwrap(), "House")
        .await;
    let dir = alex.home.parent().unwrap().to_path_buf();
    let keep = dir.join("keep");
    let mut server = tokio::process::Command::new("node")
        .arg(app(&dir).join("server.mjs"))
        .args(["--port", "0", "--keep"])
        .arg(&keep)
        .arg("--svrngs")
        .arg(alex.home.join(".svrngs/api"))
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs");
    use tokio::io::AsyncBufReadExt;
    let mut printed = tokio::io::BufReader::new(server.stdout.take().unwrap()).lines();
    let said = printed.next_line().await.unwrap().unwrap_or_default();
    let port = said
        .split("127.0.0.1:")
        .nth(1)
        .and_then(|p| p.split('/').next())
        .expect("the doc says its address")
        .to_string();
    let shared = scene
        .command(
            &alex,
            &["share", &port, "--name", "doc", "--kind", "doc"],
            "",
        )
        .await;
    assert!(shared.contains("You lend doc"), "{shared}");

    // Alex writes its first line, before the room opens it (README).
    let at_alex = alex.lent_by(&house, "Alex", "doc").await;
    tell(
        &mut browsers,
        json!({ "do": "person", "who": "alex", "engine": engine.as_str() }),
    )
    .await;
    tell(
        &mut browsers,
        json!({ "do": "open", "who": "alex", "url": at_alex }),
    )
    .await;
    tell(
        &mut browsers,
        json!({ "do": "type", "who": "alex", "text": FIRST }),
    )
    .await;

    // Bo opens it on his computer; Cy on her phone, through the group page.
    let at_bo = bo.lent_by(&house, "Alex", "doc").await;
    tell(
        &mut browsers,
        json!({ "do": "person", "who": "bo", "engine": engine.as_str() }),
    )
    .await;
    tell(
        &mut browsers,
        json!({ "do": "open", "who": "bo", "url": at_bo }),
    )
    .await;
    let phone = json!({ "do": "person", "who": "cy", "engine": "webkit", "device": "iPhone 13" });
    tell(&mut browsers, phone).await;
    let joined = tell(
        &mut browsers,
        json!({ "do": "join", "who": "cy", "link": links["https"], "name": "Cy" }),
    )
    .await;
    assert!(
        joined["status"]
            .as_str()
            .unwrap_or_default()
            .contains("in House"),
        "Cy's phone joins: {joined:#}"
    );
    tell(
        &mut browsers,
        json!({ "do": "press", "who": "cy", "name": "doc" }),
    )
    .await;
    for (who, name) in [("bo", "Bo"), ("cy", "Cy")] {
        let you = format!("You’re {name}\\.");
        tell(
            &mut browsers,
            json!({ "do": "says", "who": who, "sel": "#you", "re": you, "within": 10_000 }),
        )
        .await;
        let here = "^(?=[\\s\\S]*Bo)(?=[\\s\\S]*Cy)";
        tell(
            &mut browsers,
            json!({ "do": "says", "who": who, "sel": "#here", "re": here, "within": 10_000 }),
        )
        .await;
    }

    // Both write at once; every page holds both lines, one doc.
    for (who, line) in LINES {
        tell(
            &mut browsers,
            json!({ "do": "type", "who": who, "text": line }),
        )
        .await;
    }
    let all = [FIRST, LINES[0].1, LINES[1].1];
    for who in ["alex", "bo", "cy"] {
        tell(
            &mut browsers,
            json!({ "do": "has", "who": who, "texts": all, "within": 15_000 }),
        )
        .await;
    }

    // Each paragraph's note names its writer, on every page.
    for who in ["alex", "bo", "cy"] {
        let noted = "^(?=[\\s\\S]*Alex)(?=[\\s\\S]*Bo)(?=[\\s\\S]*Cy)";
        tell(&mut browsers, json!({ "do": "says", "who": who, "sel": "#attribution", "re": noted, "within": 5_000 })).await;
        let read = tell(&mut browsers, json!({ "do": "read", "who": who })).await;
        let paragraphs = read["paragraphs"].as_array().cloned().unwrap_or_default();
        let notes = read["note_lines"].as_array().cloned().unwrap_or_default();
        for (line, name) in [(LINES[0].1, "Bo"), (LINES[1].1, "Cy")] {
            let at = paragraphs
                .iter()
                .position(|p| p.as_str() == Some(line))
                .unwrap_or_else(|| panic!("{who}'s page holds {line:?}: {read:#}"));
            let note = notes.get(at).and_then(|n| n.as_str()).unwrap_or_default();
            assert!(
                note.contains(&format!("by {name}")),
                "{who}'s page notes {line:?} as {name}'s: {note:?}; {read:#}"
            );
        }
    }

    // What the server kept: each line's edits by its writer alone.
    let kept = std::fs::read_to_string(keep.join("acts.jsonl")).unwrap();
    let people: std::collections::BTreeSet<String> = kept
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter_map(|a| a["person"].as_str().map(str::to_string))
        .collect();
    assert_eq!(
        people,
        ["Alex", "Bo", "Cy"].map(String::from).into(),
        "the doc keeps edits by Alex, Bo and Cy"
    );
    tell(&mut browsers, json!({ "do": "end" })).await;
    drop((alex, bo, server));
}
