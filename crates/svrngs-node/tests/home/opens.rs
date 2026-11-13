//! A carried app's reads at its own origin, on the wire: the node's half
//! of browser::opens::a_carried_app_reads_the_lends_it_declares_at_its_own_origin,
//! its computers on one `Wire` (common/on_wire.rs), the app's page asked as
//! carried.rs `sent` asks it, each lent service answered in this process
//! (on_wire.rs `serve`, CHECKS.md harness 2): no socket.

use std::path::Path;
use std::time::Duration;

use serde_json::{json, Value};
use svrngs_node::node::Node;
use svrngs_node::{actor_of, lend};
use tokio::io::AsyncWriteExt;

use super::computers::{found, round, signing};
use super::on_wire::{http, serve, start};

type Heads = std::sync::Arc<std::sync::Mutex<Vec<String>>>;

/// Ben's films as a reading app meets them, at `port` on `node`, answered
/// in this process (on_wire.rs `serve`, harness 2): each request's head
/// kept, and answered with its `authorization` echoed in a header of its
/// own, the lend's cookie set and the app's storage cleared, as JSON, or
/// as a script at a path ending `.js` (browser/opens.rs `echoing`).
fn echoing(node: &Node, port: u16) -> Heads {
    let heads = Heads::default();
    let kept = heads.clone();
    serve(node, port, move |mut sock| {
        let kept = kept.clone();
        async move {
            let mut head = [0u8; 8192];
            let n = tokio::io::AsyncReadExt::read(&mut sock, &mut head)
                .await
                .unwrap_or(0);
            let head = String::from_utf8_lossy(&head[..n]).into_owned();
            let auth = (head.lines())
                .find(|l| l.to_ascii_lowercase().starts_with("authorization: "))
                .map(|l| l["authorization: ".len()..].to_string())
                .unwrap_or_else(|| "none".into());
            let script = head.lines().next().is_some_and(|l| l.contains(".js "));
            kept.lock().unwrap().push(head);
            let (kind, body) = if script {
                ("Text/JavaScript", "document.title = 'ran';")
            } else {
                ("application/json", r#"{"films":"ben's films"}"#)
            };
            let answer = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: {kind}\r\nx-echo: {auth}\r\n\
                 clear-site-data: \"storage\"\r\n\
                 set-cookie: lend-session=kept-by-the-lend; Path=/\r\n\
                 content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = sock.write_all(answer.as_bytes()).await;
        }
    });
    heads
}

fn declare(dir: &Path, name: &str, port: u16, kind: &str, group: &str) {
    let at = lend::lends_dir(dir);
    std::fs::create_dir_all(&at).unwrap();
    let body = json!({ "groups": [group], "port": port, "kind": kind, "limit": 3, "refused": [] });
    std::fs::write(at.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// What `node`'s `/v1/lends` lists of `group`.
async fn lends(node: &Node, group: &str) -> Value {
    let url = format!("http://{}/v1/lends?namespace={group}", node.api_addr());
    let answer = (http().get(url).bearer_auth(node.token().trim()).send())
        .await
        .unwrap();
    answer.json().await.unwrap()
}

/// `method` at `path` under the lend `label` from the page at `origin`,
/// with a header of the page's own: its status, refusal word, headers and
/// body.
async fn read(
    node: &Node,
    origin: &str,
    (method, label, path): (&str, &str, &str),
) -> (u16, String, Vec<String>, String) {
    let host = origin.trim_start_matches("http://").trim_end_matches('/');
    let url = format!("http://{}/svrngs-lend/{label}{path}", node.api_addr());
    let answer = (http().request(method.parse().unwrap(), url))
        .header("host", host)
        .header("x-page", "the page's own")
        .bearer_auth(node.token().trim())
        .send()
        .await
        .unwrap();
    let status = answer.status().as_u16();
    let heads = answer.headers();
    let word = (heads.get("x-svrngs-refused"))
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let mut named: Vec<String> = (heads.iter())
        .map(|(n, v)| format!("{n}: {}", v.to_str().unwrap_or_default()))
        .filter(|h| {
            !["date:", "content-length:"]
                .iter()
                .any(|d| h.starts_with(d))
        })
        .collect();
    named.sort_unstable();
    (status, word, named, answer.text().await.unwrap_or_default())
}

/// **A carried app reads the lends it declares, at its own origin, on its
/// person's yes, and nothing it is answered holds a secret** (the operator,
/// 2026-11-08; COMMUNITY_APPS §6.2): Probe, carried in house on Ana's
/// computer, declares `films`. Ben lends films, kept with a readers'
/// account, and notes; Ana lends films to another group alone. On Ana's
/// yes naming Ben's films, Probe's GET there is answered 200 as the
/// readers' account, the page's own header not carried; its answer holds
/// only cache-control, a sandbox policy, the type and nosniff, never the
/// echoed account, the lend's cookie or its clearing of storage; a script
/// there is refused `closed`; Ben's notes, a kind it does not declare, and
/// Ana's films of the other group are `closed`, never asked. Before the
/// yes and after its Stop, `unasked`, is surface.rs's
/// (a_yes_names_lends_so_a_new_one_waits_for_the_next_open,
/// a_stop_on_the_home_page_holds_from_the_app_s_next_request).
#[tokio::test]
async fn a_carried_app_reads_the_lends_it_declares_and_holds_no_secret() {
    let dirs: Vec<_> = (0..2).map(|_| tempfile::tempdir().unwrap()).collect();
    let ana = start(1, dirs[0].path()).await;
    let ben = start(2, dirs[1].path()).await;
    let group = found("house", 61, &ana, &[(&ben, 2)]);
    for node in [&ben, &ana] {
        round(node).await;
    }
    let (films, mine) = (8096, 8097);
    let heads = echoing(&ben, films);
    let on_mine = echoing(&ana, mine);
    declare(dirs[1].path(), "films", films, "films", &group);
    declare(dirs[1].path(), "notes", films, "notes", &group);
    declare(dirs[0].path(), "films", mine, "films", "elsewhere");
    let name = lend::LendName::parse("films").unwrap();
    lend::keep_readers(
        dirs[1].path(),
        &name,
        ("authorization", "Readers films-for-apps"),
    )
    .unwrap();
    let shared = dirs[0].path().join("Shared");
    std::fs::create_dir_all(shared.join("apps/Probe")).unwrap();
    let kept = http()
        .post(format!("http://{}/v1/folders", ana.api_addr()))
        .bearer_auth(ana.token())
        .json(&json!({ "dir": shared, "group": group }))
        .send()
        .await
        .unwrap();
    assert!(kept.status().is_success(), "{:?}", kept.text().await);
    let app = shared.join("apps/Probe");
    std::fs::write(
        app.join("index.html"),
        "<!doctype html><title>Probe</title>",
    )
    .unwrap();
    std::fs::write(app.join("reads.txt"), "films\n").unwrap();
    let mut origin = None;
    for _ in 0..30 {
        let seen = lends(&ana, &group).await;
        let probe = (seen["lent"].as_array().into_iter().flatten()).find(|l| l["kind"] == "Probe");
        if let Some(probe) = probe {
            origin = probe["origin"].as_str().map(str::to_string);
            break;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let origin = origin.expect("Probe carried");
    let app = origin
        .trim_start_matches("http://")
        .split('.')
        .next()
        .unwrap();
    let label = |seed: u8, name: &str| {
        svrngs_core::pass::Label::of(signing(seed).verifying_key().as_bytes(), name).to_string()
    };
    let (films_at, notes_at, mine_at) = (label(2, "films"), label(2, "notes"), label(1, "films"));
    let yes =
        json!({ actor_of(&signing(1)): { app: { "said": "yes", "at": 1, "lends": [films_at] } } });
    std::fs::write(dirs[0].path().join("reaching.json"), yes.to_string()).unwrap();

    let (status, word, named, body) = read(&ana, &origin, ("GET", &films_at, "/films?n=1")).await;
    assert_eq!((status, word.as_str()), (200, ""), "{body}");
    assert_eq!(body, r#"{"films":"ben's films"}"#);
    assert_eq!(
        named,
        [
            "cache-control: no-store",
            "content-security-policy: sandbox",
            "content-type: application/json",
            "x-content-type-options: nosniff",
        ],
        "only what svrngs says reaches the page"
    );
    let asked = heads.lock().unwrap().clone();
    assert_eq!(asked.len(), 1, "{asked:#?}");
    assert!(asked[0].starts_with("GET /films?n=1 "), "{asked:#?}");
    assert!(
        asked[0].contains("films-for-apps"),
        "as the readers' account: {asked:#?}"
    );
    assert!(
        !asked[0].to_ascii_lowercase().contains("x-page"),
        "the page's own header is not carried: {asked:#?}"
    );
    let (status, word, _, _) = read(&ana, &origin, ("GET", &films_at, "/app.js")).await;
    assert_eq!(
        (status, word.as_str()),
        (403, "closed"),
        "a lend's script never runs here"
    );
    for (whose, at) in [
        ("ben's notes", &notes_at),
        ("ana's films elsewhere", &mine_at),
    ] {
        let (status, word, _, _) = read(&ana, &origin, ("GET", at, "/films")).await;
        assert_eq!((status, word.as_str()), (403, "closed"), "{whose}");
    }
    assert_eq!(
        heads.lock().unwrap().len(),
        2,
        "the script asked, never the notes"
    );
    assert!(
        on_mine.lock().unwrap().is_empty(),
        "ana's films elsewhere never asked"
    );
    for node in [&ana, &ben] {
        node.net().close().await;
    }
}
