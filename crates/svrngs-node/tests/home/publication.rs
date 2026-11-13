// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot};

const PHRASE: &str = "acorn-basil-cabin-daisy-eagle";

struct Call {
    body: Value,
    reply: oneshot::Sender<Option<String>>,
}

struct Resolver {
    calls: mpsc::Receiver<Call>,
    task: tokio::task::JoinHandle<()>,
}

impl Drop for Resolver {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Resolver {
    async fn start(dir: &std::path::Path) -> Self {
        let at = std::fs::read_to_string(dir.join("join-resolver"))
            .ok()
            .map(|url| {
                url.trim_start_matches("http://")
                    .trim_end_matches('/')
                    .to_string()
            })
            .unwrap_or_else(|| "127.0.0.1:0".into());
        let listener = tokio::net::TcpListener::bind(at).await.unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        std::fs::write(dir.join("join-resolver"), &url).unwrap();
        let (send, calls) = mpsc::channel(10);
        let task = tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let send = send.clone();
                tokio::spawn(async move {
                    let mut bytes = Vec::new();
                    let (head, body) = loop {
                        let mut buf = [0; 4096];
                        let n = socket.read(&mut buf).await.unwrap_or(0);
                        // Closed before a whole request: no call. Something
                        // else on this computer looked at the port, as
                        // share::listening does from another test in this
                        // binary, which on Linux reaches the ephemeral
                        // ports (32768-60999) this stub listens on
                        // (demo/runs/flakes/2026-11-12).
                        if n == 0 {
                            return;
                        }
                        bytes.extend_from_slice(&buf[..n]);
                        if let Some(at) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                            let head = String::from_utf8(bytes[..at].to_vec()).unwrap();
                            let length: usize = head
                                .lines()
                                .find_map(|line| {
                                    let (key, value) = line.split_once(':')?;
                                    key.eq_ignore_ascii_case("content-length")
                                        .then(|| value.trim().parse().unwrap())
                                })
                                .unwrap();
                            if bytes.len() >= at + 4 + length {
                                break (head, bytes[at + 4..at + 4 + length].to_vec());
                            }
                        }
                    };
                    let header = |key: &str| {
                        head.lines()
                            .find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case(key).then(|| value.trim())
                            })
                            .unwrap()
                    };
                    let key = header("x-svrngs-key");
                    assert_eq!(key, actor_of(&signing(1)));
                    let app = svrngs_core::app::App::parse("onboarding").unwrap();
                    assert!(app.check(key, &body, header("x-svrngs-signature")));
                    assert!(!head.to_ascii_lowercase().contains("authorization:"));
                    let (reply, receive) = oneshot::channel();
                    send.send(Call {
                        body: serde_json::from_slice(&body).unwrap(),
                        reply,
                    })
                    .await
                    .unwrap();
                    if let Ok(Some(response)) = receive.await {
                        socket.write_all(response.as_bytes()).await.unwrap();
                    }
                });
            }
        });
        Self { calls, task }
    }

    async fn call(&mut self) -> Call {
        tokio::time::timeout(Duration::from_secs(15), self.calls.recv())
            .await
            .unwrap()
            .unwrap()
    }

    async fn ok(&mut self) -> Value {
        let call = self.call().await;
        let body = if call.body["action"] == "close" {
            json!({"id": call.body["id"], "closed": true})
        } else {
            json!({"id": call.body["id"], "phrase": PHRASE})
        };
        call.reply
            .send(Some(response(200, &body.to_string())))
            .unwrap();
        call.body
    }
}

fn response(status: u16, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn receipt(node: &Node) -> (std::path::PathBuf, Value) {
    let path = node.rail().group_dir(NS).unwrap().join("publication.json");
    let body = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    (path, body)
}

fn valid(node: &Node, token: &str) -> bool {
    let token = svrngs_core::link::Token::decode(token).unwrap();
    node.rail()
        .current_invite(NS)
        .unwrap()
        .is_some_and(|(_, i)| i.key == actor_of(&token.key()))
}

async fn seeded(dir: &std::path::Path) -> Node {
    let node = on_wire::start(1, dir).await;
    node.rail()
        .set_seed(
            NS,
            &Roster::new(
                [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
                    .into_iter()
                    .collect(),
            ),
        )
        .unwrap();
    node
}

async fn status(node: &Node) -> Value {
    let got = publish(&Browser::of(node), "GET", "/publication?group=house", "").await;
    assert_eq!(got.status, 200, "{}", got.body);
    got.json()
}

async fn publish(browser: &Browser<'_>, method: &str, path: &str, body: &str) -> Got {
    let home = format!("svrngs.localhost:{}", browser.api().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    ask(browser, method, &home, path, &own, body).await
}

#[tokio::test]
async fn publication_unavailable_is_retryable_not_an_ended_invitation() {
    let dir = tempfile::tempdir().unwrap();
    let node = on_wire::start(1, dir.path()).await;
    node.rail()
        .set_seed(
            NS,
            &Roster::new(
                [(Person::parse("alex").unwrap(), vec![actor_of(&signing(1))])]
                    .into_iter()
                    .collect(),
            ),
        )
        .unwrap();
    let got = publish(
        &Browser::of(&node),
        "POST",
        "/publication",
        r#"{"group":"house"}"#,
    )
    .await;
    assert_eq!(got.status, 503, "{}", got.body);
    assert!(
        got.body.contains("joining is temporarily unavailable"),
        "{}",
        got.body
    );
    assert!(got.body.contains("retry"), "{}", got.body);
    node.net().close().await;
}

#[tokio::test]
async fn publication_survives_reload_and_closes_only_its_invitation() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let mut resolver = Resolver::start(dir.path()).await;
    let node = seeded(dir.path()).await;
    let browser = Browser::of(&node);
    let home = format!("svrngs.localhost:{}", node.api_addr().port());
    let foreign = "Sec-Fetch-Site: cross-site\r\nSec-Fetch-Mode: cors\r\nOrigin: https://elsewhere.invalid\r\n";
    for path in ["/publication", "/publication/close"] {
        let got = ask(
            &browser,
            "POST",
            &home,
            path,
            foreign,
            r#"{"group":"house"}"#,
        )
        .await;
        assert_eq!(got.status, 403, "{}", got.body);
    }
    assert_eq!(status(&node).await, Value::Null);
    let (got, sent) = tokio::join!(
        publish(&browser, "POST", "/publication", r#"{"group":"house"}"#),
        resolver.ok()
    );
    assert_eq!(got.status, 200, "{}", got.body);
    assert_eq!(got.json()["state"], "open");
    assert_eq!(got.json()["phrase"], PHRASE);
    let (path, kept) = receipt(&node);
    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(valid(&node, sent["token"].as_str().unwrap()));
    let token = svrngs_core::link::Token::decode(sent["token"].as_str().unwrap()).unwrap();
    let invite = node.rail().current_invite(NS).unwrap().unwrap().0;
    let redeem = |seed, name| {
        let person = Person::parse(name).unwrap();
        svrngs_core::Redeem {
            invite: invite.clone(),
            cosig: svrngs_core::Redemption::sign(
                &signing(seed),
                NS,
                &invite,
                &person,
                svrngs_core::Device::Computer,
            )
            .sig,
            person,
            key: actor_of(&signing(seed)),
            device: svrngs_core::Device::Computer,
            link: actor_of(&token.key()),
            issuer: actor_of(&signing(1)),
        }
    };
    node.rail().commit(NS, &redeem(2, "sam")).unwrap().unwrap();
    assert!(!got.body.contains(sent["token"].as_str().unwrap()));
    assert_eq!(status(&node).await, got.json());
    let close = json!({"group": NS, "id": kept["id"]}).to_string();
    let (closed, sent_close) = tokio::join!(
        publish(&browser, "POST", "/publication/close", &close),
        resolver.ok()
    );
    assert_eq!(closed.status, 200, "{}", closed.body);
    assert_eq!(closed.json()["state"], "closed");
    assert_eq!(sent_close["id"], kept["id"]);
    assert_eq!(sent_close["token"], Value::Null);
    assert!(
        !valid(&node, sent["token"].as_str().unwrap()),
        "copied long link must end"
    );
    assert!(matches!(
        node.rail().commit(NS, &redeem(3, "cy")).unwrap(),
        Err(svrngs_core::Refused::Command(
            svrngs_core::RedeemRefusal::Uncounted
        ))
    ));
    assert!(
        node.rail()
            .standing(NS)
            .unwrap()
            .member(&actor_of(&signing(2)))
            .is_some(),
        "Sam already joined and stays"
    );
    assert!(node
        .rail()
        .standing(NS)
        .unwrap()
        .member(&actor_of(&signing(1)))
        .is_some());
    // A duplicate close is local and cannot mint again.
    let current = node.rail().current_invite(NS).unwrap();
    let again = publish(&browser, "POST", "/publication/close", &close).await;
    assert_eq!(again.json()["state"], "closed");
    assert_eq!(node.rail().current_invite(NS).unwrap(), current);
    // Replacing the closed receipt requires its id; stale close cannot hit it.
    let (new, _) = tokio::join!(
        publish(&browser, "POST", "/publication", &close),
        resolver.ok()
    );
    assert_ne!(new.json()["id"], kept["id"]);
    let current = node.rail().current_invite(NS).unwrap();
    let stale = publish(&browser, "POST", "/publication/close", &close).await;
    assert_eq!(stale.status, 503);
    assert_eq!(node.rail().current_invite(NS).unwrap(), current);
    node.net().close().await;
}

#[tokio::test]
async fn publication_failures_keep_one_recoverable_invitation_and_receipt() {
    let dir = tempfile::tempdir().unwrap();
    let mut resolver = Resolver::start(dir.path()).await;
    let node = seeded(dir.path()).await;
    let browser = Browser::of(&node);
    let mut original = None;
    for failure in [
        Some((429, "rate limit")),
        Some((500, "bandwidth")),
        Some((503, "quota/storage")),
        Some((200, "not JSON")),
        Some((
            200,
            r#"{"id":"wrong","phrase":"acorn-basil-cabin-daisy-eagle"}"#,
        )),
        None,
    ] {
        let fail = async {
            let call = resolver.call().await;
            call.reply
                .send(failure.map(|(code, body)| response(code, body)))
                .unwrap();
            call.body
        };
        let (got, body) = tokio::join!(
            publish(&browser, "POST", "/publication", r#"{"group":"house"}"#),
            fail
        );
        assert_eq!(got.status, 503, "{}", got.body);
        assert!(
            got.body.contains("joining is temporarily unavailable") && got.body.contains("retry")
        );
        assert_eq!(receipt(&node).1["phase"], "pending");
        assert_eq!(status(&node).await["state"], "pending");
        assert!(valid(&node, body["token"].as_str().unwrap()));
        if let Some(ref first) = original {
            assert_eq!(&body, first);
        } else {
            original = Some(body);
        }
    }
    assert_eq!(status(&node).await["state"], "pending");
    let (got, retried) = tokio::join!(
        publish(&browser, "POST", "/publication", r#"{"group":"house"}"#),
        resolver.ok()
    );
    assert_eq!(got.json()["state"], "open");
    assert_eq!(Some(retried.clone()), original);
    let close = json!({"group": NS, "id": retried["id"]}).to_string();
    let fail = async {
        let call = resolver.call().await;
        call.reply.send(Some(response(429, "quota"))).unwrap();
    };
    let (failed, ()) = tokio::join!(
        publish(&browser, "POST", "/publication/close", &close),
        fail
    );
    assert_eq!(failed.status, 503);
    assert_eq!(status(&node).await["state"], "closing");
    assert!(valid(&node, retried["token"].as_str().unwrap()));
    assert_eq!(status(&node).await["state"], "closing");
    let (got, _) = tokio::join!(
        publish(&browser, "POST", "/publication/close", &close),
        resolver.ok()
    );
    assert_eq!(got.json()["state"], "closed");
    assert!(!valid(&node, retried["token"].as_str().unwrap()));
    node.net().close().await;
}

#[tokio::test]
async fn publication_late_answers_cannot_override_new_links_or_closure() {
    let dir = tempfile::tempdir().unwrap();
    let mut resolver = Resolver::start(dir.path()).await;
    let node = seeded(dir.path()).await;
    let browser = Browser::of(&node);
    let race = async {
        let call = resolver.call().await;
        // This must finish while the network request is outstanding.
        let newer = tokio::time::timeout(
            Duration::from_secs(2),
            publish(
                &browser,
                "POST",
                "/link",
                r#"{"group":"house","device":false}"#,
            ),
        )
        .await
        .unwrap();
        assert_eq!(newer.status, 200, "{}", newer.body);
        let current = node.rail().current_invite(NS).unwrap();
        call.reply
            .send(Some(response(
                200,
                &json!({"id": call.body["id"], "phrase": PHRASE}).to_string(),
            )))
            .unwrap();
        current
    };
    let (late, current) = tokio::join!(
        publish(&browser, "POST", "/publication", r#"{"group":"house"}"#),
        race
    );
    assert_eq!(late.json()["state"], "ended");
    assert_eq!(
        receipt(&node).1["phase"],
        "pending",
        "a late answer must not mark an old invitation open"
    );
    let old = json!({"group": NS, "id": late.json()["id"]}).to_string();
    let (closed, _) = tokio::join!(
        publish(&browser, "POST", "/publication/close", &old),
        resolver.ok()
    );
    assert_eq!(closed.json()["state"], "closed");
    assert_eq!(
        node.rail().current_invite(NS).unwrap(),
        current,
        "stale close must keep newer ordinary invite"
    );
    let race = async {
        let call = resolver.call().await;
        let close = json!({"group": NS, "id": call.body["id"]}).to_string();
        let (closed, _) = tokio::join!(
            publish(&browser, "POST", "/publication/close", &close),
            resolver.ok()
        );
        assert_eq!(closed.json()["state"], "closed");
        call.reply
            .send(Some(response(
                200,
                &json!({"id": call.body["id"], "phrase": PHRASE}).to_string(),
            )))
            .unwrap();
    };
    let (late, ()) = tokio::join!(publish(&browser, "POST", "/publication", &old), race);
    assert_eq!(late.json()["state"], "closed");
    assert_eq!(status(&node).await["state"], "closed");
    node.net().close().await;
}

// A restart is a process boundary. An embedded FsStore owns database threads
// beyond a Node/runtime drop; only a stopped process models this requirement.
#[test]
fn publication_recovers_pending_open_closing_and_closed_across_process_restarts() {
    let dir = tempfile::tempdir().unwrap();
    for phase in ["pending", "open", "closing", "closed", "inspect"] {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "publication::publication_restart_child",
                "--nocapture",
            ])
            .env("SVRNGS_PUBLICATION_TEST_DIR", dir.path())
            .env("SVRNGS_PUBLICATION_TEST_PHASE", phase)
            .spawn()
            .unwrap();
        let began = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "restart phase {phase}: {status}");
                break;
            }
            if began.elapsed() > Duration::from_secs(30) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("restart phase {phase} timed out");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

#[tokio::test]
async fn publication_restart_child() {
    let Ok(dir) = std::env::var("SVRNGS_PUBLICATION_TEST_DIR") else {
        return;
    };
    let phase = std::env::var("SVRNGS_PUBLICATION_TEST_PHASE").unwrap();
    let dir = std::path::Path::new(&dir);
    let mut resolver = Resolver::start(dir).await;
    let node = if phase == "pending" {
        seeded(dir).await
    } else {
        on_wire::start(1, dir).await
    };
    let browser = Browser::of(&node);
    let previous = status(&node).await;
    match phase.as_str() {
        "pending" => assert_eq!(previous, Value::Null),
        "open" => assert_eq!(previous["state"], "pending"),
        "closing" => assert_eq!(previous["state"], "open"),
        "closed" => assert_eq!(previous["state"], "closing"),
        "inspect" => {
            assert_eq!(previous["state"], "closed");
            assert!(!valid(&node, receipt(&node).1["token"].as_str().unwrap()));
            node.net().close().await;
            return;
        }
        _ => panic!("unknown restart phase"),
    }
    let closing = matches!(phase.as_str(), "closing" | "closed");
    let body = json!({"group": NS, "id": previous["id"]}).to_string();
    let path = if closing {
        "/publication/close"
    } else {
        "/publication"
    };
    let failed = matches!(phase.as_str(), "pending" | "closing");
    let reply = async {
        if failed {
            let call = resolver.call().await;
            call.reply.send(None).unwrap(); // Accepted remotely, response lost.
        } else {
            resolver.ok().await;
        }
    };
    let (got, ()) = tokio::join!(publish(&browser, "POST", path, &body), reply);
    assert_eq!(got.status, if failed { 503 } else { 200 }, "{}", got.body);
    if phase == "open" {
        assert_eq!(got.json()["id"], previous["id"]);
    }
    if phase != "closed" {
        assert!(valid(&node, receipt(&node).1["token"].as_str().unwrap()));
    }
    node.net().close().await;
}

/// The request a redirect followed would have sent `elsewhere`, if one came
/// within 100 ms. A connection that sends nothing and closes is not one: a
/// look at the port by something else on this computer (share::listening,
/// from another test in this binary, reaches it on Linux), which as
/// `accept` alone failed this test 5 runs of 5 beside such a prober and
/// none of 5 without (demo/runs/flakes/2026-11-12).
async fn followed(elsewhere: &tokio::net::TcpListener) -> Option<String> {
    let request = async {
        loop {
            let (mut socket, _) = elsewhere.accept().await.unwrap();
            let mut buf = [0; 1024];
            let n = socket.read(&mut buf).await.unwrap_or(0);
            if n > 0 {
                return String::from_utf8_lossy(&buf[..n]).into_owned();
            }
        }
    };
    tokio::time::timeout(Duration::from_millis(100), request)
        .await
        .ok()
}

#[tokio::test]
async fn publication_refuses_redirects_oversized_bodies_and_timeouts() {
    let dir = tempfile::tempdir().unwrap();
    let mut resolver = Resolver::start(dir.path()).await;
    let node = seeded(dir.path()).await;
    let browser = Browser::of(&node);
    let elsewhere = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    for mode in ["redirect", "oversized", "timeout", "phrase"] {
        let fail = async {
            let call = resolver.call().await;
            let reply = match mode {
                "redirect" => format!("HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{}/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n", elsewhere.local_addr().unwrap()),
                "oversized" => response(200, &" ".repeat(8193)),
                "phrase" => response(200, &json!({"id": call.body["id"], "phrase": "not-five-words"}).to_string()),
                "timeout" => {
                    tokio::time::sleep(Duration::from_secs(11)).await;
                    return;
                }
                _ => unreachable!(),
            };
            call.reply.send(Some(reply)).unwrap();
        };
        let (got, ()) = tokio::join!(
            publish(&browser, "POST", "/publication", r#"{"group":"house"}"#),
            fail
        );
        assert_eq!(got.status, 503, "{mode}: {}", got.body);
        assert!(
            got.body.contains("joining is temporarily unavailable") && got.body.contains("retry")
        );
        assert_eq!(status(&node).await["state"], "pending");
        assert!(valid(&node, receipt(&node).1["token"].as_str().unwrap()));
        assert_eq!(
            followed(&elsewhere).await,
            None,
            "invitation must never follow a resolver redirect"
        );
    }
    node.net().close().await;
}
