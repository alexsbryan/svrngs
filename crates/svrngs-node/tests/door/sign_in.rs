// SPDX-License-Identifier: AGPL-3.0-or-later
//! A lend's sign-in, through the door (mwp-1; DESIGN §3, "The room", call
//! 6): a stand-in service and a sign-in answerer on loopback, and two
//! people, bo on B and cy on a third computer, through A's door. Each
//! request reaches the service with its person's secret and never
//! another's; the answerer is asked once a person; no byte either receives
//! holds a secret; a sign-in that fails refuses, never carries unsigned.
//! A lend with no sign-in is held by the door's tests before mwp-1,
//! unchanged (`a_declared_credential_displaces_the_client_s_and_arrives_verbatim`,
//! `a_secret_the_service_echoes_never_reaches_the_member`).

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

use super::*;

const STAND_IN: &str = "svrngs-stand-in";

/// The page the service answers at `/page`, its own script in its `<head>`.
const PAGE: &str = "<!doctype html><html><head><script>own()</script></head><body>hi</body></html>";

/// The script the answerer gives the person with `key`.
fn script_of(key: &str) -> String {
    format!("window.who='{key}'")
}

/// The secret the answerer gives the person with `key`.
fn secret_of(key: &str) -> String {
    format!("secret-{}", &key[..24])
}

/// How the answerer answers.
#[derive(Clone, Copy)]
enum Answers {
    Each,
    Fails,
    Nonsense,
    /// As `Each`, but bo's ask (`signing(2)`) is held [`HELD`] first.
    SlowForBo,
}

/// How long the answerer holds bo's ask under [`Answers::SlowForBo`].
const HELD: Duration = Duration::from_secs(3);

/// The sign-in answerer: each person their own secret, as `authorization`
/// too; keeps the head of every ask, each ask answered on its own.
async fn answerer(answers: Answers) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let asks = Arc::new(Mutex::new(Vec::new()));
    let kept = asks.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let Some(head) = origins::read_head(&mut sock).await else {
                    return;
                };
                let key = header_values(&head, "x-svrngs-key")
                    .first()
                    .map(|k| k.to_string())
                    .unwrap_or_default();
                kept.lock().unwrap().push(head);
                if matches!(answers, Answers::SlowForBo) && key == actor_of(&signing(2)) {
                    tokio::time::sleep(HELD).await;
                }
                let secret = secret_of(&format!("{key:0<24}"));
                let (status, body) = match answers {
                    Answers::Each | Answers::SlowForBo => (
                        "200 OK",
                        serde_json::json!({
                            "secret": secret,
                            "headers": { "Authorization": format!("Bearer {secret}") },
                            "script": script_of(&key),
                        })
                        .to_string(),
                    ),
                    Answers::Fails => ("500 Internal Server Error", "down".into()),
                    Answers::Nonsense => ("200 OK", r#"{"secret": "x"}"#.into()),
                };
                let reply = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(reply.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });
    (addr, asks)
}

/// `start`, then `body` in two chunks cut at `cut`, 100 ms apart, so the
/// door reads them as two.
async fn chunked(sock: &mut tokio::net::TcpStream, start: &str, body: &str, cut: usize) {
    let _ = sock.write_all(start.as_bytes()).await;
    for part in [&body[..cut], &body[cut..]] {
        let chunk = format!("{:x}\r\n{part}\r\n", part.len());
        let _ = sock.write_all(chunk.as_bytes()).await;
        let _ = sock.flush().await;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let _ = sock.write_all(b"0\r\n\r\n").await;
}

/// The stand-in service: keeps each head; `/page` answers [`PAGE`] in two
/// chunks cut inside its `<head>`, cacheable, under a script policy naming
/// a nonce when asked `?csp`; `/stale` answers 401; `/echo`
/// answers the query and `authorization` it was sent in JSON, in two
/// chunks cut inside the secret, and the secret again in a header; else
/// 200 `ok`.
async fn service() -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let heads = Arc::new(Mutex::new(Vec::new()));
    let kept = heads.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let kept = kept.clone();
            tokio::spawn(async move {
                let Some(head) = origins::read_head(&mut sock).await else {
                    return;
                };
                kept.lock().unwrap().push(head.clone());
                let target = head.split_whitespace().nth(1).unwrap_or("").to_string();
                if target.starts_with("/page") {
                    let policy = if target.ends_with("?csp") {
                        "Content-Security-Policy: script-src 'nonce-n0nce' 'strict-dynamic'\r\n"
                    } else {
                        ""
                    };
                    let start = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n{policy}ETag: \"v1\"\r\n\
                         Last-Modified: Wed, 07 Oct 2026 00:00:00 GMT\r\nCache-Control: max-age=600\r\n\
                         Transfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
                    );
                    let cut = PAGE.find("<head>").unwrap() + 3;
                    chunked(&mut sock, &start, PAGE, cut).await;
                    return;
                }
                if target.starts_with("/stale") {
                    let no = "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = sock.write_all(no.as_bytes()).await;
                    return;
                }
                if !target.starts_with("/echo") {
                    let ok = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
                    let _ = sock.write_all(ok.as_bytes()).await;
                    return;
                }
                let auth = header_values(&head, "authorization").join(",");
                let query = target.split_once('?').map_or("", |(_, q)| q);
                let body = format!(r#"{{"query":"{query}","auth":"{auth}"}}"#);
                let cut = body.find("secret-").map_or(body.len() / 2, |i| i + 10);
                let start = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Seen: {auth}\r\n\
                     Transfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
                );
                chunked(&mut sock, &start, &body, cut).await;
                let _ = sock.shutdown().await;
            });
        }
    });
    (addr, heads)
}

/// A declares `name` on `port`, its sign-in at `sign_in`.
fn lend_signed(pair: &Pair, name: &str, port: u16, sign_in: u16) {
    let dir = lend::lends_dir(pair.dir.path());
    std::fs::create_dir_all(&dir).unwrap();
    let body = serde_json::json!({ "port": port, "kind": "k", "limit": 3, "sign_in": sign_in });
    std::fs::write(dir.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// cy, admitted to A's group, and a bridge dialling A on the members'
/// protocol with cy's key, as the pass test dials (`main.rs`).
async fn cy_dials_a(pair: &Pair) -> svrngs_node::iroh::HttpBridge {
    use svrngs_core::alpn::Alpn;
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("cy").unwrap(),
            key: actor_of(&signing(3)),
            invite: None,
        },
        on_behalf_of: None,
    };
    pair.a.rail().commit(NS, &admit).unwrap().unwrap();
    bridge_as(&signing(3), &pair.a, Alpn::Member).await
}

/// A chunked body's bytes, its framing taken off, so a secret cut across
/// two chunks reads whole.
fn dechunked(mut framed: &str) -> String {
    let mut body = String::new();
    while let Some((size, rest)) = framed.split_once("\r\n") {
        let n = usize::from_str_radix(size.trim(), 16).expect("a chunk's size");
        if n == 0 {
            break;
        }
        body.push_str(&rest[..n]);
        framed = &rest[n + 2..];
    }
    body
}

/// A request a person's browser makes with the stand-in it was given, in
/// its query and its `authorization`, asking for a compressed answer.
fn asks(path: &str) -> String {
    format!(
        "GET /lend/signed/{path}?api_key={STAND_IN}&x=1 HTTP/1.1\r\nHost: local\r\n\
         Authorization: Bearer {STAND_IN}\r\nAccept-Encoding: gzip\r\nConnection: close\r\n\r\n"
    )
}

/// Two people through the door, each signed in as themselves. Each request
/// reaches the service with its person's secret, in the header the
/// answerer named and in the query where the client put the stand-in, and
/// never another's; the answerer is asked once a person, three first
/// requests at once included; no byte either receives holds a secret — in
/// a header, in a body cut inside it, in a query echoed back — and each
/// sees the stand-in where its secret was. A 401 to a kept answer forgets
/// it, and the next request asks again. Planted: the answerer asked every
/// request; a secret cut across chunks let through; one answer per lend,
/// whoever asks (another's secret on a request).
#[tokio::test]
async fn two_people_are_each_signed_in_as_themselves_and_neither_receives_a_secret() {
    let pair = Pair::new().await;
    let cy = cy_dials_a(&pair).await;
    let cy_bridge = cy.local_addr();
    let (origin, heads) = service().await;
    let (hook, asked) = answerer(Answers::Each).await;
    lend_signed(&pair, "signed", origin.port(), hook.port());
    let (bo_key, cy_key) = (actor_of(&signing(2)), actor_of(&signing(3)));
    let secrets = [secret_of(&bo_key), secret_of(&cy_key)];

    let mut replies = Vec::new();
    let (r1, r2, r3) = tokio::join!(
        raw_to(pair.bridge, asks("echo")),
        raw_to(pair.bridge, asks("echo")),
        raw_to(pair.bridge, asks("echo")),
    );
    replies.extend([r1, r2, r3]);
    for _ in 0..2 {
        replies.push(raw_to(cy_bridge, asks("echo")).await);
    }
    for reply in &replies {
        assert_eq!(reply.status, 200, "{}", reply.text);
        let body = dechunked(reply.text.split_once("\r\n\r\n").unwrap().1);
        for secret in &secrets {
            for read in [&reply.text, &body] {
                assert!(
                    !read.contains(secret.as_str()),
                    "a person received a secret: {read}"
                );
            }
        }
        assert!(
            body.contains(&format!("api_key={STAND_IN}&x=1"))
                && body.contains(&format!("Bearer {STAND_IN}")),
            "the stand-in where the secret was: {body}"
        );
        assert_eq!(
            header_values(&reply.text, "x-seen"),
            vec![format!("Bearer {STAND_IN}")]
        );
    }

    let heads = heads.lock().unwrap().clone();
    assert_eq!(heads.len(), 5);
    for head in &heads {
        let key = header_values(head, "x-svrngs-key")[0];
        let (own, other) = if key == bo_key {
            (&secrets[0], &secrets[1])
        } else {
            assert_eq!(key, cy_key);
            (&secrets[1], &secrets[0])
        };
        assert_eq!(
            header_values(head, "authorization"),
            vec![format!("Bearer {own}")],
            "{head}"
        );
        assert!(
            head.starts_with(&format!("GET /echo?api_key={own}&x=1 ")),
            "{head}"
        );
        assert!(
            !head.contains(other.as_str()),
            "another's secret on a request: {head}"
        );
        assert!(!head.contains(STAND_IN), "{head}");
        assert!(header_values(head, "accept-encoding").is_empty(), "{head}");
    }

    let keys = |asked: &Mutex<Vec<String>>| -> BTreeMap<String, usize> {
        let mut n = BTreeMap::new();
        for head in asked.lock().unwrap().iter() {
            *n.entry(header_values(head, "x-svrngs-key")[0].to_string())
                .or_default() += 1;
        }
        n
    };
    assert_eq!(
        keys(&asked),
        BTreeMap::from([(bo_key.clone(), 1), (cy_key.clone(), 1)]),
        "asked once a person, not once a request"
    );
    for head in asked.lock().unwrap().iter() {
        assert!(head.starts_with("POST / HTTP/1.1\r\n"), "{head}");
        assert!(
            header_values(head, "authorization").is_empty(),
            "no client header asked: {head}"
        );
        assert_eq!(header_values(head, "x-svrngs-person").len(), 1, "{head}");
        assert_eq!(
            header_values(head, "x-svrngs-stand-in"),
            [STAND_IN],
            "{head}"
        );
    }

    assert_eq!(raw_to(pair.bridge, asks("stale")).await.status, 401);
    assert_eq!(raw_to(pair.bridge, asks("echo")).await.status, 200);
    assert_eq!(
        keys(&asked)[&bo_key],
        2,
        "a 401 to a kept answer, asked again"
    );
    assert_eq!(keys(&asked)[&cy_key], 1);
    drop(cy);
    pair.close().await;
}

/// A sign-in that fails — answering 500, answering nonsense, or not
/// listening at all — refuses the person's request, 502, saying the lend
/// could not sign them in, or for one not listening that it is not
/// answering, and the service is never asked; the next request asks
/// again. Planted: a failure carried unsigned.
#[tokio::test]
async fn a_sign_in_that_fails_or_answers_nonsense_refuses_the_request_never_carries_it() {
    let pair = Pair::new().await;
    let (origin, heads) = service().await;
    let closed = common::stopped();
    for answers in [Some(Answers::Fails), Some(Answers::Nonsense), None] {
        let (hook, asked) = match answers {
            Some(a) => answerer(a).await,
            None => (closed, Arc::new(Mutex::new(Vec::new()))),
        };
        lend_signed(&pair, "signed", origin.port(), hook.port());
        let (kind, words) = match answers {
            Some(_) => ("sign-in", "signed could not sign you in"),
            None => ("away", "ada’s signed is not answering"),
        };
        for _ in 0..2 {
            let reply = raw_to(pair.bridge, asks("echo")).await;
            assert_eq!(reply.status, 502, "{}", reply.text);
            let told = format!(r#""kind":"{kind}""#);
            assert!(reply.text.contains(&told), "{}", reply.text);
            assert!(reply.text.contains(words), "{}", reply.text);
        }
        assert_eq!(
            asked.lock().unwrap().len(),
            if answers.is_some() { 2 } else { 0 }
        );
    }
    assert!(
        heads.lock().unwrap().is_empty(),
        "never carried unsigned: {:?}",
        heads.lock().unwrap()
    );
    pair.close().await;
}

/// A sign-in that answers no is said as that; one that does not answer, or
/// answers no while its lend does not answer, is said as the lend not
/// answering, by its lender's name and its own, and never a port
/// (v1ui-polish (a); `demo/runs/v1ui/2026-10-15/README.md` finding 2). A
/// browser's tab is shown either as a page, and a program given its JSON.
/// Planted: the helper stopped, said as "could not sign you in", in braces.
#[tokio::test]
async fn a_stopped_lend_behind_its_sign_in_is_said_as_not_answering_by_its_lender_on_a_page() {
    let pair = Pair::new().await;
    let (up, _) = service().await;
    let stopped = common::stopped();
    let (no, _) = answerer(Answers::Fails).await;
    for (service, helper, kind) in [
        (up, stopped, "away"),
        (stopped, no, "away"),
        (up, no, "sign-in"),
    ] {
        lend_signed(&pair, "signed", service.port(), helper.port());
        let program = raw_to(pair.bridge, asks("echo")).await;
        assert_eq!(program.status, 502, "{}", program.text);
        let told = format!(r#""kind":"{kind}""#);
        assert!(program.text.contains(&told), "{kind}: {}", program.text);
        let tab = raw_to(pair.bridge, asks_page("GET", "signed", "echo")).await;
        assert_eq!(tab.status, 502, "{}", tab.text);
        let page = tab.text.to_ascii_lowercase();
        assert!(
            page.contains("content-type: text/html"),
            "{kind}: {}",
            tab.text
        );
        assert!(
            !page.contains("{\"") && !page.contains(&service.port().to_string()),
            "{}",
            tab.text
        );
        let said = if kind == "away" {
            "ada’s signed is not answering"
        } else {
            "could not sign you in"
        };
        assert!(page.contains(said), "{kind}: {}", tab.text);
    }
    pair.close().await;
}

/// A request for a page, as a browser asks one it holds a copy of.
fn asks_page(method: &str, lend: &str, path: &str) -> String {
    format!(
        "{method} /lend/{lend}/{path} HTTP/1.1\r\nHost: local\r\nAccept: text/html,*/*\r\n\
         If-None-Match: \"v1\"\r\nIf-Modified-Since: Wed, 07 Oct 2026 00:00:00 GMT\r\n\
         Content-Length: 0\r\nConnection: close\r\n\r\n"
    )
}

/// Each person's script goes first in `<head>` of a page answered to their
/// `GET`, before the page's own script, its `<head>` cut across chunks,
/// carrying the nonce of the page's script policy, and kept by no cache:
/// the page's `ETag` and `Last-Modified` dropped, `no-store`, and the
/// service asked without the browser's copy's. Another person's script is
/// in no page of theirs. A `POST`'s page, a non-HTML answer, and the page
/// through a lend with no sign-in, go as before. Planted: the script after
/// the page's own; one person's script in another's page.
#[tokio::test]
async fn each_person_s_script_goes_first_in_their_pages_and_never_in_another_s() {
    let pair = Pair::new().await;
    let cy = cy_dials_a(&pair).await;
    let (origin, heads) = service().await;
    let (hook, _) = answerer(Answers::Each).await;
    lend_signed(&pair, "signed", origin.port(), hook.port());
    lends_on(pair.dir.path(), "plain", origin.port());
    let (bo_key, cy_key) = (actor_of(&signing(2)), actor_of(&signing(3)));
    let body_of = |reply: &Reply| dechunked(reply.text.split_once("\r\n\r\n").unwrap().1);

    for (bridge, own, other) in [
        (pair.bridge, &bo_key, &cy_key),
        (cy.local_addr(), &cy_key, &bo_key),
    ] {
        for (path, open) in [
            ("page", "<script>"),
            ("page?csp", "<script nonce=\"n0nce\">"),
        ] {
            let reply = raw_to(bridge, asks_page("GET", "signed", path)).await;
            assert_eq!(reply.status, 200, "{}", reply.text);
            let want = PAGE.replace(
                "<head>",
                &format!("<head>{open}{}</script>", script_of(own)),
            );
            assert_eq!(body_of(&reply), want, "the person's script first in <head>");
            assert!(
                !reply.text.contains(&script_of(other)),
                "another's script in a person's page: {}",
                reply.text
            );
            assert_eq!(
                header_values(&reply.text, "cache-control"),
                vec!["no-store"]
            );
            for kept in ["etag", "last-modified"] {
                assert!(
                    header_values(&reply.text, kept).is_empty(),
                    "{}",
                    reply.text
                );
            }
        }
    }
    for head in heads.lock().unwrap().iter() {
        for copy in ["if-none-match", "if-modified-since"] {
            assert!(header_values(head, copy).is_empty(), "{head}");
        }
    }

    let posted = raw_to(pair.bridge, asks_page("POST", "signed", "page")).await;
    assert_eq!(body_of(&posted), PAGE, "a POST's page as before");
    let other = raw_to(pair.bridge, asks("ok")).await;
    assert_eq!(body_of(&other), "ok", "a non-HTML answer as before");
    assert!(header_values(&other.text, "cache-control").is_empty());

    let plain = raw_to(pair.bridge, asks_page("GET", "plain", "page")).await;
    assert_eq!(body_of(&plain), PAGE, "a lend with no sign-in as before");
    assert_eq!(header_values(&plain.text, "etag"), vec!["\"v1\""]);
    assert_eq!(
        header_values(&plain.text, "cache-control"),
        vec!["max-age=600"]
    );
    let asked = heads.lock().unwrap().last().cloned().unwrap();
    assert_eq!(header_values(&asked, "if-none-match"), vec!["\"v1\""]);
    drop(cy);
    pair.close().await;
}

/// One person's sign-in held slow holds nobody else's (#88; DESIGN §3,
/// "The room", call 6): while the answerer holds bo's first ask through
/// `signed` for [`HELD`], cy's first requests, through `signed` and through
/// `other`, whose sign-in is another, are each answered within a second;
/// and bo's two first requests at once, both answered, ask once.
#[tokio::test]
async fn one_person_s_slow_sign_in_holds_nobody_else_s() {
    let pair = Pair::new().await;
    let cy = cy_dials_a(&pair).await;
    let (origin, _) = service().await;
    let (slow, asked) = answerer(Answers::SlowForBo).await;
    let (fast, _) = answerer(Answers::Each).await;
    lend_signed(&pair, "signed", origin.port(), slow.port());
    lend_signed(&pair, "other", origin.port(), fast.port());
    let timed = |bridge, request| async move {
        let began = tokio::time::Instant::now();
        let reply = raw_to(bridge, request).await;
        (reply.status, began.elapsed())
    };
    let bo = async {
        let (one, two) = tokio::join!(
            timed(pair.bridge, asks("echo")),
            timed(pair.bridge, asks("echo"))
        );
        [one, two]
    };
    let others = async {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let (same, other) = tokio::join!(
            timed(cy.local_addr(), asks("echo")),
            timed(
                cy.local_addr(),
                asks("echo").replace("/lend/signed/", "/lend/other/")
            ),
        );
        [same, other]
    };
    let (bo, others) = tokio::join!(bo, others);
    let waited: Vec<String> = ["the same lend", "another lend"]
        .iter()
        .zip(others)
        .filter(|(_, (status, took))| *status != 200 || *took >= Duration::from_secs(1))
        .map(|(through, (status, took))| format!("through {through}, {status} after {took:?}"))
        .collect();
    assert!(
        waited.is_empty(),
        "cy's first requests waited on bo's ask: {waited:?}"
    );
    for (status, took) in bo {
        assert_eq!(status, 200);
        assert!(
            took >= HELD,
            "bo's request answered before its ask: {took:?}"
        );
    }
    let bo_key = actor_of(&signing(2));
    let bo_asked = asked
        .lock()
        .unwrap()
        .iter()
        .filter(|h| header_values(h, "x-svrngs-key") == [bo_key.as_str()])
        .count();
    assert_eq!(bo_asked, 1, "bo's two first requests at once, asked once");
    drop(cy);
    pair.close().await;
}

/// **Each device of one person is asked for as itself** (bash-two-at-once
/// (b); demo/runs/two-at-once/2026-10-12/README.md, finding 1): bo asks a
/// lend with a sign-in from one device, then the other, then the first
/// again; the answerer is asked once a device, each ask told bo's one
/// `x-svrngs-key` and the asking device's own key in `x-svrngs-device`, so
/// an app signs each of a person's devices in apart under one account.
/// Red at the row's base: asked once, the answer kept a person
/// (bash-devices (c)), and no device told.
#[tokio::test]
async fn each_device_of_one_person_is_asked_for_as_itself() {
    let bos = Bos::new().await;
    let (origin, _heads) = service().await;
    let (hook, asked) = answerer(Answers::Each).await;
    bos.lend(
        "signed",
        serde_json::json!({ "port": origin.port(), "kind": "k", "sign_in": hook.port() }),
    );
    for i in [0, 1, 0] {
        let reply = raw_to(bos.devices[i].local_addr(), asks("echo")).await;
        assert_eq!(reply.status, 200, "{}", reply.text);
    }
    let asked = asked.lock().unwrap().clone();
    let told = |name| (asked.iter().map(|h| header_values(h, name).join(","))).collect::<Vec<_>>();
    let (bo, bos_other) = (actor_of(&signing(2)), actor_of(&signing(6)));
    assert_eq!(
        told("x-svrngs-key"),
        [bo.clone(), bo.clone()],
        "bo, asked once a device"
    );
    assert_eq!(
        told("x-svrngs-device"),
        [bo, bos_other],
        "each device by its own key"
    );
    bos.a.endpoint().close().await;
}
