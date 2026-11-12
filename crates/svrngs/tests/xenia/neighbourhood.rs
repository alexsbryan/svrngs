//! A community lifecycle on the real surface; its run-of-show and limits
//! live in demo/xenia/permaculture/README.md. Nothing backstage completes
//! a stranger's action. The observer records provision separately from what
//! Xenia counts and what the people say.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::process::Child;

use crate::give::{brings, give_at, give_built, keeps, no_podman, prefix, remove};
use crate::harness::{repo, Engine};
use crate::people::{Person, Scene};
use crate::{foldered, opened, origin, stay_up};

const GROUP: &str = "Seed swap";
const RUN: &str = "perm";
const NOTE: &str = "Seed envelopes are in the blue tin.";

struct World {
    _scene: Scene,
    _sam: Person,
    _jo: Person,
    ren: Person,
    group: String,
    _kept: Kept,
}

struct Kept(Vec<Child>);

impl Drop for Kept {
    fn drop(&mut self) {
        for child in &mut self.0 {
            let _ = child.start_kill();
        }
        for who in ["Sam", "Jo"] {
            remove(&format!("{}notebook", prefix(RUN, who)));
        }
    }
}

/// Give removes a stopped pod (runtime.rs `stop`). Observe absence through
/// `pod exists`'s explicit exit 1; an inspect/connection error is not absence.
fn pod(who: &str) -> Value {
    let name = format!("{}notebook", prefix(RUN, who));
    match Command::new("podman")
        .args(["pod", "exists", &name])
        .output()
    {
        Ok(o) if o.status.code() == Some(1) => return json!({ "exists": false, "exists_exit": 1 }),
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            return json!({ "could_not_observe": String::from_utf8_lossy(&o.stderr), "exists_exit": o.status.code() })
        }
        Err(e) => return json!({ "could_not_observe": e.to_string() }),
    }
    match Command::new("podman")
        .args(["pod", "inspect", "--format", "{{.State}}", &name])
        .output()
    {
        Ok(o) => json!({
            "exists": true,
            "exit": o.status.code(),
            "state": String::from_utf8_lossy(&o.stdout).trim(),
            "error": String::from_utf8_lossy(&o.stderr).trim(),
        }),
        Err(e) => json!({ "could_not_observe": e.to_string() }),
    }
}

/// A program's authenticated read through Ren's real door, not the model's
/// report that a note survived. The fixture itself answers from its volume.
async fn notebook(who: &Person, at: &str) -> Value {
    let url = reqwest::Url::parse(at).unwrap();
    let host = url.host_str().unwrap();
    let socket = ([127, 0, 0, 1], url.port().unwrap()).into();
    let client = reqwest::Client::builder()
        .no_proxy()
        .resolve(host, socket)
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let token = std::fs::read_to_string(who.home.join(".svrngs/token")).unwrap();
    let answer = client
        .get(format!("{}/cgi-bin/book", at.trim_end_matches('/')))
        .bearer_auth(token.trim())
        .send()
        .await;
    match answer {
        Ok(r) => {
            let status = r.status().as_u16();
            match r.text().await {
                Ok(text) => {
                    json!({ "status": status, "contains_ren_note": text.contains(NOTE), "body": text })
                }
                Err(e) => json!({ "status": status, "could_not_read": e.to_string() }),
            }
        }
        Err(e) => json!({ "could_not_observe": e.to_string() }),
    }
}

async fn observe(world: World, evidence: PathBuf) {
    let began = Instant::now();
    let mut history = Vec::new();
    let mut previous = Value::Null;
    loop {
        let record = world
            .ren
            .get(&format!("/v1/rail/log?namespace={}", world.group))
            .await;
        let lends = world
            .ren
            .get(&format!("/v1/lends?namespace={}", world.group))
            .await;
        let mut notebooks = Vec::new();
        if let Some(lent) = lends["lent"].as_array() {
            for lend in lent.iter().filter(|l| l["name"] == "notebook") {
                if let Some(at) = lend["origin"].as_str() {
                    notebooks.push(
                        json!({ "lender": lend["person"], "read": notebook(&world.ren, at).await }),
                    );
                }
            }
        }
        let current = json!({
            "record": record,
            "lends": lends,
            "notebooks": notebooks,
            "pods": { "Sam": pod("Sam"), "Jo": pod("Jo") },
        });
        if current != previous {
            history.push(
                json!({ "elapsed_seconds": began.elapsed().as_secs_f64(), "observed": current }),
            );
            let saved = json!({
                "simulated": true,
                "scope": "One host, real svrngs nodes and Give, a notebook fixture; observer reads only. Cleanup is not a participant action.",
                "observations": history,
            });
            // Xenia copies attachments while the world is still up; a read
            // must see a whole sample, never a partly rewritten JSON file.
            let next = evidence.with_extension("next");
            std::fs::write(&next, serde_json::to_vec_pretty(&saved).unwrap()).unwrap();
            std::fs::rename(next, &evidence).unwrap();
            previous = current;
        }
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

struct Observing(tokio::task::JoinHandle<()>);
impl Drop for Observing {
    fn drop(&mut self) {
        self.0.abort();
    }
}

fn part(id: &str, brief: Value, reach: &[&str]) -> Value {
    json!({
        "id": id, "brief": brief, "reach": reach,
        "minutes": 4, "talk": 180,
        "feedback": [
            "What did you actually do, and what could you see afterwards?",
            { "ask": "Where did you get stuck or need somebody's help?", "bad": "stuck" },
            { "ask": "Was there something you wanted to do that the pages did not let you do?", "bad": "missing" },
        ],
    })
}

pub async fn world() {
    if let Some(why) = no_podman() {
        panic!("neighbourhood never-ran: {why}");
    }
    let scene = Scene::new("xenia-neighbourhood", Engine::Chromium).await;
    let (sam, group) = scene.founds("Sam", GROUP).await;
    let jo = scene.joins("Jo", &sam.invites(&group).await, GROUP).await;
    let ren = scene.joins("Ren", &sam.invites(&group).await, GROUP).await;
    for person in [&sam, &jo, &ren] {
        foldered(person, &group).await;
    }
    let program = give_built().await;
    let card = repo().join("demo/xenia/permaculture/cards/notebook");
    let card = card.to_str().unwrap();
    let scratch = scene.scratch("sam-give");
    brings(&program, RUN, &sam, &scratch, GROUP, card).await;
    let kept = Kept(vec![
        keeps(&program, RUN, &sam, &scratch, card, None),
        keeps(&program, RUN, &jo, &scene.scratch("jo-give"), card, None),
    ]);
    let sam_give = give_at(&sam, &group).await;
    let jo_give = give_at(&jo, &group).await;
    // The notebook was already started before this visit. No handover,
    // refusal, or participant's new note is pre-written.
    eprintln!(
        "neighbourhood: Sam at {}, Give at {sam_give}",
        sam.home_page()
    );
    // The version is published by Give's own shelf, not reconstructed here.
    let groups = sam.get("/v1/groups").await;
    let folder = groups["groups"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["id"] == group)
        .unwrap()["folders"][0]
        .as_str()
        .unwrap();
    let cards: Value = serde_json::from_slice(
        &std::fs::read(PathBuf::from(folder).join("apps/give/cards.json")).unwrap(),
    )
    .unwrap();
    let version = cards
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == "notebook")
        .unwrap()["version"]
        .clone();
    sam.on_lend(&sam_give, "/svrngs/acts", &json!({ "op": { "kind": "started", "app": "notebook", "card": version, "agreed": false } })).await;
    ren.lent_by(&group, "Sam", "notebook").await;

    let evidence = scene.scratch("evidence.json");
    std::fs::write(
        &evidence,
        r#"{"simulated":true,"observations":[],"status":"observer has not sampled yet"}"#,
    )
    .unwrap();
    let mut reach: Vec<_> = [&sam, &jo, &ren]
        .iter()
        .map(|p| origin(&p.home_page()))
        .collect();
    for person in [&sam, &jo, &ren] {
        let listed = person.get(&format!("/v1/lends?namespace={group}")).await;
        for l in listed["lent"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|l| l["kind"] == "give" || l["name"] == "notebook")
        {
            if let Some(at) = l["origin"].as_str() {
                reach.push(origin(at));
            }
        }
    }
    // Jo has not lent yet. Use the core's label and the node's origin
    // constructor to declare the reach needed after a real handover.
    let listed = jo.get("/v1/groups").await;
    let key = listed["key"].as_str().unwrap();
    let label = svrngs_core::pass::Label::of_hex(key, "notebook").unwrap();
    for person in [&sam, &jo, &ren] {
        let port = reqwest::Url::parse(&person.home_page())
            .unwrap()
            .port()
            .unwrap();
        reach.push(origin(&svrngs_node::origin::origin(&label, port)));
    }
    reach.sort();
    reach.dedup();
    let attachments: Vec<_> = ["Sam", "Jo", "Ren"]
        .map(|who| json!({ "as": who, "path": evidence }))
        .into();
    let line = json!({
        "begins": "Sam, Jo and Ren on separate computers, already in Seed swap with their home pages open. Installation, invitations and starting Sam's notebook were prepared, not observed. Give is running on Sam's and Jo's computers; Ren has no keeping program. All three computers and containers are on one host. The notebook is a small real stateful rehearsal fixture, not a shipped app.",
        "cast": [
            { "as": "Sam", "person": "people/sam.md", "device": "computer", "url": opened(&sam.home_page()) },
            { "as": "Jo", "person": "people/jo.md", "device": "computer", "url": opened(&jo.home_page()) },
            { "as": "Ren", "person": "people/ren.md", "device": "computer", "url": opened(&ren.home_page()) },
        ],
        "screens": [
            { "name": "the notebook", "text": "What neighbours know" },
            { "name": "Give", "text": "Apps your group keeps" },
            { "name": "Home", "url": "svrngs.localhost" },
        ],
        "channels": [{ "name": "the street chat", "members": ["Sam", "Jo", "Ren"], "opening": [] }],
        "scenes": [
            part("benefit", json!([{ "from": "Mira", "text": "We're doing the street seed swap this weekend. Sam put the details in Neighbourhood notebook in Seed swap. Have a look so you know when and where to come, and say here if anything is unclear." }]), &["Home", "the notebook"]),
            part("retain", json!([
                { "from": "Mira", "to": ["Ren"], "text": format!("You found the spare envelopes. Please leave this in the notebook for the next person: {NOTE} You don't need to look after the program to add a note.") },
                { "from": "Mira", "to": ["Sam", "Jo"], "text": "Ren found where the spare envelopes are. See whether the notebook tells you now; you shouldn't have to chase Ren privately." }
            ]), &["the notebook"]),
            part("not-now", json!([
                { "from": "Mira", "to": ["Sam"], "text": "You're travelling soon and want a break from keeping the notebook on your laptop. Ask Jo to take it on using what Seed swap already offers; it must stay with you unless Jo agrees." },
                { "from": "Mira", "to": ["Jo"], "text": "You can't keep the notebook on your laptop this week. If Sam asks, say not now through the pages as well as the chat. You still want to use the notebook." },
                { "from": "Mira", "to": ["Ren"], "text": "You're still using the notebook for Saturday. If Sam and Jo discuss looking after it, check whether you can still read your note without volunteering your computer." }
            ]), &["Give", "the notebook"]),
            part("handover", json!([
                { "from": "Mira", "to": ["Sam"], "text": "A week later, Jo has time to keep the notebook. Arrange the handover together, using the pages. Ren's note must come with it. You want your laptop to stop running the notebook once Jo has it." },
                { "from": "Mira", "to": ["Jo"], "text": "A week later, you have time and are willing to keep the notebook on your laptop. Read what you're taking on, then arrange it with Sam through the pages. Check that Ren's note is still there." },
                { "from": "Mira", "to": ["Ren"], "text": "Sam and Jo are arranging who keeps the notebook. Keep using it as a neighbour: see whether your note survives, and say if you lose it." }
            ]), &["Give", "Home", "the notebook"]),
            part("end", json!([{ "from": "Mira", "text": "The seed-swap season is over. You all agree to stop running the notebook for now, keeping its notes so another season can use them. Try to end that arrangement with the pages you have. Check what actually stopped and what you can still keep. If the pages don't offer a way, say what is missing; don't claim it stopped just because a link disappeared." }]), &["Give", "Home", "the notebook"]),
        ],
        "faults": [{ "text": "can’t be read just now" }, { "expr": "document.querySelector('input[type=password]') !== null" }],
        "reach": reach,
        "attachments": attachments,
    });
    eprintln!(
        "neighbourhood: Jo's Give at {jo_give}; evidence {}",
        evidence.display()
    );
    let world = World {
        _scene: scene,
        _sam: sam,
        _jo: jo,
        ren,
        group,
        _kept: kept,
    };
    let observing = Observing(tokio::spawn(observe(world, evidence)));
    stay_up(line, observing).await;
}
