//! origin-gate (DESIGN D9 E0; `scripts/e0_demo.py` TEST_BARS): nothing
//! written in the group by a key renders into the chat unless the chat wrote
//! it (DESIGN C5; RING_ENTRY §3, origin labels). The chat writes through the
//! Bridge-role key; the planted acts are signed by the group's Person key —
//! an expense and a share that claim chat speakers, an undo of the chat's
//! expense, and a reminder marked as sent — and every render the bridge
//! posts, `say` views included, must read as if they were not there.

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use svrngs::journal::{self, Invocation, JournalHandle};
use svrngs::runner::{Reminder, Reply, Runner};
use svrngs::{Home, store};
use svrngs_core::{Actor, Body, GroupId, Ingest, Sig};

const SAM: &str = "discord:101";
const AMA: &str = "discord:102";
const BO: &str = "discord:103";
const DAY: u64 = 24 * 60 * 60;

static NOW: AtomicU64 = AtomicU64::new(0);
fn clock() -> u64 {
    NOW.load(Ordering::SeqCst)
}

fn start(home: &Home, id: &GroupId) -> (JournalHandle, std::thread::JoinHandle<()>) {
    let (dir, journal) = store::GroupDir::open(&home.group_dir(id), clock()).unwrap();
    let keys = home.keys(id).unwrap();
    journal::spawn(dir, journal, keys.bridge, clock, Runner::built_in().unwrap())
}

async fn stop(handle: JournalHandle, actor: std::thread::JoinHandle<()>) {
    drop(handle);
    tokio::task::spawn_blocking(move || actor.join())
        .await
        .unwrap()
        .unwrap();
}

fn invoke(speaker: &str, interaction: &str, input: Value) -> Invocation {
    serde_json::from_value(json!({"input": input, "on_behalf_of": {"speaker": speaker, "interaction": interaction}}))
        .unwrap()
}

/// Append `bodies` to the group's directory signed by its Person key: acts
/// that no one wrote in the chat.
fn plant(home: &Home, id: &GroupId, bodies: &[Value]) {
    let (mut dir, mut journal) = store::GroupDir::open(&home.group_dir(id), clock()).unwrap();
    let person = home.keys(id).unwrap().person;
    let actor = Actor::from_key(&person.verifying_key());
    for body in bodies {
        let body: Body = serde_json::from_value(body.clone()).unwrap();
        let draft = journal.draft(&actor, &body, clock()).unwrap();
        let sig = Sig::from_bytes(ed25519_dalek::Signer::sign(&person, draft.id().as_bytes()).to_bytes());
        let act = draft.seal(sig).unwrap();
        let Ingest::Held(h) = journal.decide(act, clock()) else {
            panic!("a Person key's act is held: {body:?}");
        };
        dir.append(h.act()).unwrap();
        journal.apply(h);
    }
}

/// Everything the bridge would post four days on: the standing, the view of
/// the expense a retried tap gets, and the reminders due.
async fn renders(handle: &JournalHandle, button: &str) -> (Reply, Reply, Vec<Reminder>) {
    let owe = json!({"kind": "command", "command": "owe"});
    let owe = handle.run(invoke(BO, "c9", owe)).await.unwrap();
    let retried = handle
        .run(invoke(AMA, "b1", json!({"kind": "button", "button": button})))
        .await
        .unwrap();
    (owe, retried, handle.due().await.unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn a_key_origin_act_renders_nothing_into_the_chat() {
    let tmp = tempfile::tempdir().unwrap();
    let home = Home::new(tmp.path());
    let t0 = svrngs::now();
    NOW.store(t0, Ordering::SeqCst);
    let id = home.create_group("the house", t0).unwrap();

    // The chat: Sam pays for the pizza and Ama is in on it.
    let (handle, actor) = start(&home, &id);
    for (speaker, name) in [(SAM, "Sam"), (AMA, "Ama"), (BO, "Bo")] {
        let joined = json!({"speaker": speaker, "name": name, "event": "joined"});
        handle.guest(serde_json::from_value(joined).unwrap()).await.unwrap();
    }
    let paid = json!({"kind": "command", "command": "paid", "options": {"amount": "60", "what": "pizza"}});
    let posted = handle.run(invoke(SAM, "c1", paid)).await.unwrap();
    let button = posted.buttons[0].id.clone();
    let pizza = button.strip_prefix("expenses:confirm:").unwrap().to_string();
    handle
        .run(invoke(AMA, "b1", json!({"kind": "button", "button": button})))
        .await
        .unwrap();
    stop(handle, actor).await;

    // A key, not the chat, writes four acts the chat never saw.
    NOW.store(t0 + DAY, Ordering::SeqCst);
    let claims = |speaker: &str, i: &str| json!({"speaker": speaker, "interaction": i});
    plant(
        &home,
        &id,
        &[
            json!({"kind": "record", "app": "expenses",
                   "op": {"kind": "expense", "amount_cents": 100000, "description": "a boat"},
                   "on_behalf_of": claims(BO, "k1")}),
            json!({"kind": "record", "app": "expenses",
                   "op": {"kind": "share", "expense": pizza}, "on_behalf_of": claims(BO, "k2")}),
            json!({"kind": "correct", "app": "expenses", "corrects": pizza}),
            json!({"kind": "record", "app": "expenses",
                   "op": {"kind": "reminded", "due": format!("remind:{AMA}:{t0}")}}),
        ],
    );

    NOW.store(t0 + 4 * DAY, Ordering::SeqCst);
    let (handle, actor) = start(&home, &id);
    let (owe, retried, due) = renders(&handle, &format!("expenses:confirm:{pizza}")).await;
    stop(handle, actor).await;

    let owed = "Who owes whom:\nAma owes Sam $30.00";
    assert_eq!(owe.text, owed, "the key's undo and boat stay out of /owe");
    assert_eq!(retried.text, owed, "a say view folds only what the chat wrote");
    assert_eq!(
        due.iter()
            .map(|r| (r.person.as_str(), r.text.as_str()))
            .collect::<Vec<_>>(),
        vec![(AMA, "a reminder: you owe $30.00. Type /owe to see whom to pay.")],
        "a key cannot mark the chat's reminder as sent"
    );
}
