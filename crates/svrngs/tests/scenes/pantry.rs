//! **The pantry** (apps/pantry; the models story's run, demo/runs/models/
//! 2026-10-22-run/, bars C1 to C3): Ana lends a model, Sam a tool, Cy a
//! model of another shape, each a stand-in server on a port of their own
//! computer, lent as `svrngs share <port> --kind <anything> --name <name>`
//! (apps/pantry/README.md, "To add something"). Bo, on his own computer,
//! runs the pantry as it comes. `pantry list` names each lend by what its
//! protocol answered, nothing registered (C1); `pantry serve` gives one
//! address, and a streamed chat for `Ana/coder` reaches Ana's model under
//! its own id (C2); Cy's model, whose shape no pantry test uses, answers
//! through it too, the pantry's files unchanged (C3).
//!
//! The stand-ins are the models story's servers in miniature, a few lines
//! of node's http each: real weights are the story's run, not the gate's.
//! No browser takes part, so the scene is the same in both engines.
//!
//! The invariant it relies on: **a friend's model hears the person who
//! asked, named by its lender's door**: Ana's model is told Bo, never the
//! pantry, nor Ana, nor nobody (README.md, "Write an adapter": "every lend
//! it reaches is reached as you").

use std::process::Stdio;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::AsyncBufReadExt;

use crate::harness::{repo, Engine};
use crate::people::{Person, Scene};

/// A stand-in server: `model <id> <whose>` answers OpenAI's `/v1/models`
/// and `/v1/chat/completions`, streamed when asked, saying which id it
/// heard and whom its door named; `tool` answers MCP's `initialize` and
/// `tools/list` over Streamable HTTP. It prints its port.
const STANDIN: &str = r#"
const [kind, id, whose] = process.argv.slice(1);
const send = (r, v) => r.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify(v));
const s = require('node:http').createServer((q, r) => {
  let raw = ''; q.on('data', (c) => (raw += c)); q.on('end', () => {
    const b = raw ? JSON.parse(raw) : {};
    if (kind === 'tool') {
      if (q.url !== '/mcp' || q.method !== 'POST') return r.writeHead(404).end();
      if (b.method === 'initialize') return send(r, { jsonrpc: '2.0', id: b.id, result: { protocolVersion: '2025-06-18', capabilities: { tools: {} }, serverInfo: { name: 'sam-tools', version: '1' } } });
      if (b.method === 'tools/list') return send(r, { jsonrpc: '2.0', id: b.id, result: { tools: [
        { name: 'whoami', inputSchema: { type: 'object' }, annotations: { readOnlyHint: true } },
        { name: 'erase', inputSchema: { type: 'object' }, annotations: { destructiveHint: true } }] } });
      return r.writeHead(202).end();
    }
    if (q.url === '/v1/models') return send(r, whose === 'cy'
      ? { object: 'list', data: [{ id, object: 'model', owned_by: 'llama-swap', meta: { swapped: true } }] }
      : { object: 'list', data: [{ id, object: 'model' }] });
    if (q.url !== '/v1/chat/completions') return r.writeHead(404).end();
    const words = `${whose}'s ${b.model} heard ${q.headers['x-svrngs-person'] ?? 'nobody'}`;
    if (!b.stream) return send(r, { object: 'chat.completion', model: b.model, choices: [{ index: 0, message: { role: 'assistant', content: words }, finish_reason: 'stop' }] });
    r.writeHead(200, { 'content-type': 'text/event-stream' });
    for (const delta of [{ role: 'assistant' }, { content: words }])
      r.write(`data: ${JSON.stringify({ object: 'chat.completion.chunk', model: b.model, choices: [{ index: 0, delta }] })}\n\n`);
    r.end('data: [DONE]\n\n');
  });
});
s.listen(0, '127.0.0.1', () => console.log(s.address().port));
"#;

/// `who`'s stand-in, `kind` `id`, lent as `name`: the server, killed on drop.
async fn lends(
    scene: &Scene,
    who: &Person,
    kind: &str,
    id: &str,
    name: &str,
) -> tokio::process::Child {
    let whose = who.name.to_lowercase();
    let mut served = tokio::process::Command::new("node")
        .args(["-e", STANDIN, kind, id, &whose])
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .expect("node runs");
    let mut printed = tokio::io::BufReader::new(served.stdout.take().unwrap()).lines();
    let port = printed
        .next_line()
        .await
        .unwrap()
        .expect("it prints its port");
    let shared = scene
        .command(who, &["share", &port, "--kind", kind, "--name", name], "\n")
        .await;
    assert!(shared.contains(&format!("You lend {name}")), "{shared}");
    served
}

/// `node apps/pantry/pantry.mjs <args>` on `who`'s computer, its record
/// found where the pantry looks, `~/.svrngs/api`.
fn pantry_on(who: &Person, args: &[&str]) -> tokio::process::Command {
    let mut c = tokio::process::Command::new("node");
    c.arg(repo().join("apps/pantry/pantry.mjs"))
        .args(args)
        .env_clear()
        .env("HOME", &who.home)
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .stdin(Stdio::null())
        .kill_on_drop(true);
    c
}

/// The pantry's files, as they are now.
fn files() -> Vec<Vec<u8>> {
    let at = |f: &str| std::fs::read(repo().join("apps/pantry").join(f)).unwrap();
    vec![at("pantry.mjs"), at("mcp.mjs")]
}

/// What a chat for `model` through the pantry at `at` answered, streamed
/// when `stream`: the words, joined from its chunks.
async fn chat(at: &str, model: &str, stream: bool) -> String {
    let asked = json!({ "model": model, "stream": stream, "messages": [{ "role": "user", "content": "who asks?" }] });
    let got = reqwest::Client::builder().no_proxy().build().unwrap();
    let got = got
        .post(format!("{at}/v1/chat/completions"))
        .header("content-type", "application/json")
        .body(asked.to_string())
        .send();
    let got = got.await.unwrap();
    let status = got.status();
    let text = got.text().await.unwrap();
    assert!(
        status.is_success(),
        "the pantry answered {model} {status}: {text}"
    );
    if !stream {
        let v: Value = serde_json::from_str(&text).unwrap();
        return v["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .into();
    }
    let chunks = text.lines().filter_map(|l| l.strip_prefix("data: "));
    let chunks = chunks.filter_map(|c| serde_json::from_str::<Value>(c).ok());
    chunks
        .filter_map(|c| {
            c["choices"][0]["delta"]["content"]
                .as_str()
                .map(String::from)
        })
        .collect()
}

in_both_engines!(
    /// Planted, watched failing: a hand edit, the door telling every lend
    /// "someone" for the person (door.rs, `identity`'s PERSON_HEADER),
    /// reverted after.
    pantry
);
async fn pantry(engine: Engine) {
    let scene = Scene::new("pantry", engine).await;
    let before = files();
    let (ana, group) = scene.founds("Ana", "Kitchen").await;
    let link = ana.invites(&group).await;
    let sam = scene.joins("Sam", &link, "Kitchen").await;
    let cy = scene.joins("Cy", &link, "Kitchen").await;
    let bo = scene.joins("Bo", &link, "Kitchen").await;
    let _coder = lends(&scene, &ana, "llama-server", "coder", "ana-coder").await;
    let _tools = lends(&scene, &sam, "tool", "tools", "sam-tools").await;
    let _cy = lends(&scene, &cy, "llama-swap", "cy-model", "cy-model").await;
    for (person, name) in [
        ("Ana", "ana-coder"),
        ("Sam", "sam-tools"),
        ("Cy", "cy-model"),
    ] {
        bo.lent_by(&group, person, name).await;
    }

    // C1: `pantry list` on Bo's computer, each lend by its protocol.
    let listed = pantry_on(&bo, &["list"]).output();
    let listed = tokio::time::timeout(Duration::from_secs(60), listed).await;
    let listed = listed.expect("pantry list ends within 60 s").unwrap();
    let said = String::from_utf8_lossy(&listed.stdout);
    eprintln!(
        "Bo's pantry list: {said}{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    for words in [
        "model Ana/coder",
        "tool whoami (read-only), erase (destructive)",
        "model Cy/cy-model",
    ] {
        assert!(
            said.contains(words),
            "Bo's pantry list lacks {words:?}: {said}"
        );
    }

    // C2: one address on Bo's computer, routed by `model`, streamed.
    let mut served = pantry_on(&bo, &["serve", "--port", "0"]);
    let mut served = served.stdout(Stdio::piped()).spawn().expect("node runs");
    let mut printed = tokio::io::BufReader::new(served.stdout.take().unwrap()).lines();
    let line = printed.next_line().await.unwrap().unwrap_or_default();
    let at = line
        .strip_prefix("Point your interface at ")
        .and_then(|a| a.strip_suffix("/v1"))
        .unwrap_or_else(|| panic!("pantry serve printed no address: {line}"))
        .to_string();
    assert_eq!(
        chat(&at, "Ana/coder", true).await,
        "ana's coder heard Bo",
        "a streamed chat for Ana/coder reaches Ana's model, under its own id, as Bo"
    );

    // C3: Cy's model, of a shape no pantry test uses, answers too, and
    // the pantry's files are as they were.
    assert_eq!(
        chat(&at, "Cy/cy-model", false).await,
        "cy's cy-model heard Bo"
    );
    assert!(files() == before, "the pantry's files changed in the scene");
    drop((ana, sam, cy, bo, served));
}
