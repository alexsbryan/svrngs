//! plain-words (DESIGN D0, "No universe to explain"; D9 E0;
//! `scripts/e0_demo.py` TEST_BARS): every word a person reads is on one
//! allowlist, and no word on it is one of D0's internal words. A new word
//! fails here until someone puts it on the list, where a reviewer sees it.
//!
//! What a person reads, today: the bot's words (`bridge/words.json`), the
//! commands Expenses declares (`apps/expenses/manifest.json`, which Discord
//! lists), what Expenses answers in the chat, driven through the runner, and
//! what `svrngs start` prints (`crates/svrngs/src/start.json`, the only
//! words `start.rs` prints). The app's shell and the iOS permission strings
//! do not exist yet.
//!
//! A `{placeholder}` is filled with a person's or group's own words, or with
//! Discord's own label for one of its switches, so it is not read here;
//! neither are the names and amounts this test types, nor web addresses.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use svrngs::runner::{Manifest, Runner};
use svrngs::{Home, journal, store};

/// D0: "ring, member, guest, key, sync, journal, host, relay — appear in
/// nothing a person reads". A word starting with one is never allowed.
const INTERNAL: &[&str] = &["ring", "member", "guest", "key", "sync", "journal", "host", "relay"];

/// Every word a person may read. Add a word only once it reads plainly to
/// someone who has never heard of svrngs.
const ALLOWED: &str = "
a above after again already amount an and answer any application authorize
awake back before book bot by call can can't change changes channel computer
confirm copy couldn't did didn't discord does done down enter even everyone's
expense finished first five for found from get getting goes group had here hi
how i i'll i'm if in install installing is isn't it it's its itself join js
just keep keeps later leave left let let's letters like link logged longer make
minute minutes moment more much name needs never new no node not nothing now
number numbers of off on once one only open or other owe owes paid part
particular parts paste pay people pick please press pressed put reach read
reads reminder reset right run running runs save see serve server set settings
settle setup shares should shows sleep sleeps so some someone something speak
split start starts step steps stopped stops stored stores svrngs system take
takes talk tap tell tells text than that the them then there things this time
to token track try turn turned type up uses using waiting was welcome went what
when whenever where whether which while who whom whose will with write wrong
yet you you're you've your yourself
";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `text` with each web address, each `{placeholder}`, and a possessive `'s`
/// after one, removed.
fn unfilled(text: &str) -> String {
    let text = &text
        .split(' ')
        .filter(|w| !w.starts_with("https://"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut out = String::new();
    let mut rest: &str = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let close = rest[open..].find('}').map_or(rest.len(), |c| open + c + 1);
        rest = &rest[close..];
        rest = rest.strip_prefix("'s").unwrap_or(rest);
        out.push(' ');
    }
    out + rest
}

/// The words in `text`, lowercased: runs of letters and apostrophes.
fn words(text: &str) -> Vec<String> {
    text.replace('\u{2019}', "'")
        .split(|c: char| !(c.is_alphabetic() || c == '\''))
        .map(|w| w.trim_matches('\'').to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Each word in `texts` that is not allowed, with where it was read and the
/// text it came from. `typed` are the words this test typed in.
fn unplain(source: &str, texts: &[String], typed: &[&str]) -> Vec<String> {
    let allowed: Vec<&str> = ALLOWED.split_whitespace().collect();
    let mut found = Vec::new();
    for text in texts {
        for w in words(&unfilled(text)) {
            if !allowed.contains(&w.as_str()) && !typed.contains(&w.as_str()) {
                found.push(format!("{source}: {w:?} in {text:?}"));
            }
        }
    }
    found
}

#[test]
fn the_allowlist_holds_no_internal_word() {
    let bad: Vec<&str> = ALLOWED
        .split_whitespace()
        .filter(|w| INTERNAL.iter().any(|i| w.starts_with(i)))
        .collect();
    assert!(bad.is_empty(), "D0's internal words are on the allowlist: {bad:?}");
}

#[test]
fn the_bot_speaks_plainly() {
    let path = root().join("bridge/words.json");
    let words: serde_json::Map<String, Value> = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let texts: Vec<String> = words
        .values()
        .map(|v| v.as_str().expect("every bot word is a string").to_string())
        .collect();
    assert!(texts.len() >= 4, "{} has the bot's words", path.display());
    let bad = unplain("bridge/words.json", &texts, &[]);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn expenses_commands_read_plainly() {
    let path = root().join("apps/expenses/manifest.json");
    let manifest: Manifest = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut texts = Vec::new();
    for c in &manifest.commands {
        texts.extend([c.name.clone(), c.description.clone()]);
        for o in &c.options {
            texts.extend([o.name.clone(), o.description.clone()]);
        }
    }
    let bad = unplain("apps/expenses/manifest.json", &texts, &[]);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

static NOW: AtomicU64 = AtomicU64::new(0);
fn clock() -> u64 {
    NOW.load(Ordering::SeqCst)
}

#[tokio::test(flavor = "multi_thread")]
async fn expenses_answers_plainly() {
    const SAM: &str = "discord:101";
    const AMA: &str = "discord:102";
    let tmp = tempfile::tempdir().unwrap();
    let home = Home::new(tmp.path());
    let start = svrngs::now();
    let id = home.create_group("the house", start).unwrap();
    let (dir, journal) = store::GroupDir::open(&home.group_dir(&id), start).unwrap();
    let keys = home.keys(&id).unwrap();
    NOW.store(start, Ordering::SeqCst);
    let (handle, actor) = journal::spawn(dir, journal, keys.bridge, clock, Runner::built_in().unwrap());
    for (speaker, name) in [(SAM, "Sam"), (AMA, "Ama")] {
        let guest = json!({"speaker": speaker, "name": name, "event": "joined"});
        handle.guest(serde_json::from_value(guest).unwrap()).await.unwrap();
    }

    let mut said = Vec::new();
    let mut n = 0;
    let mut run = async |speaker: &str, input: Value| {
        n += 1;
        let inv = json!({"input": input, "on_behalf_of": {"speaker": speaker, "interaction": format!("i{n}")}});
        let reply = handle.run(serde_json::from_value(inv).unwrap()).await.unwrap();
        said.push(reply.text.clone());
        said.extend(reply.buttons.iter().map(|b| b.label.clone()));
        reply
    };
    let command = |command: &str, options: Value| json!({"kind": "command", "command": command, "options": options});
    run(SAM, command("owe", json!({}))).await;
    let paid = run(SAM, command("paid", json!({"amount": "60", "what": "pizza"}))).await;
    let confirm = json!({"kind": "button", "button": paid.buttons[0].id});
    run(AMA, confirm.clone()).await;
    run(AMA, confirm).await;
    run(AMA, json!({"kind": "button", "button": "expenses:confirm:gone"})).await;
    run(AMA, command("paid", json!({"amount": "lots", "what": "pizza"}))).await;
    run(AMA, command("paid", json!({"amount": "5", "what": " "}))).await;
    run(AMA, command("settle", json!({"amount": "5", "to": AMA}))).await;
    run(AMA, command("settle", json!({"amount": "5", "to": SAM}))).await;
    NOW.store(start + 30 * 24 * 60 * 60, Ordering::SeqCst);
    said.extend(handle.due().await.unwrap().into_iter().map(|r| r.text));
    drop(handle);
    tokio::task::spawn_blocking(move || actor.join())
        .await
        .unwrap()
        .unwrap();

    assert!(said.iter().any(|t| t.contains("a reminder")), "{said:#?}");
    let bad = unplain("what Expenses answers", &said, &["sam", "ama", "pizza"]);
    assert!(bad.is_empty(), "{}\n\nall it said: {said:#?}", bad.join("\n"));
}

#[test]
fn setup_speaks_plainly() {
    let path = root().join("crates/svrngs/src/start.json");
    let words: serde_json::Map<String, Value> = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let texts: Vec<String> = words
        .values()
        .map(|v| v.as_str().expect("every setup word is a string").to_string())
        .collect();
    assert!(texts.len() >= 20, "{} has setup's words", path.display());
    let bad = unplain("crates/svrngs/src/start.json", &texts, &[]);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// `start.rs` prints only `start.json`'s words: no format string in a
/// macro that prints or fails holds a letter outside its `{}`.
#[test]
fn setup_prints_only_its_words() {
    let path = root().join("crates/svrngs/src/start.rs");
    let code = std::fs::read_to_string(&path).unwrap();
    let mut found = Vec::new();
    for mac in ["println!(\"", "print!(\"", "eprintln!(\"", "bail!(\""] {
        for (at, _) in code.match_indices(mac) {
            let literal = code[at + mac.len()..].split('"').next().unwrap_or_default();
            if unfilled(literal).chars().any(char::is_alphabetic) {
                found.push(format!("{mac}{literal}\""));
            }
        }
    }
    assert!(
        found.is_empty(),
        "start.rs prints words that are not in start.json: {found:?}"
    );
}
