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
}

/// The sign-in answerer: each person their own secret, as `authorization`
/// too; keeps the head of every ask.
async fn answerer(answers: Answers) -> (SocketAddr, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let asks = Arc::new(Mutex::new(Vec::new()));
    let kept = asks.clone();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let Some(head) = origins::read_head(&mut sock).await else {
                continue;
            };
            let key = header_values(&head, "x-svrngs-key")
                .first()
                .map(|k| k.to_string())
                .unwrap_or_default();
            kept.lock().unwrap().push(head);
            let secret = secret_of(&format!("{key:0<24}"));
            let (status, body) = match answers {
                Answers::Each => (
                    "200 OK",
                    serde_json::json!({
                        "secret": secret,
                        "headers": { "Authorization": format!("Bearer {secret}") },
                        "script": null,
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
        }
    });
    (addr, asks)
}

/// The stand-in service: keeps each head; `/stale` answers 401; `/echo`
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
                let _ = sock.write_all(start.as_bytes()).await;
                for part in [&body[..cut], &body[cut..]] {
                    let chunk = format!("{:x}\r\n{part}\r\n", part.len());
                    let _ = sock.write_all(chunk.as_bytes()).await;
                    let _ = sock.flush().await;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                let _ = sock.write_all(b"0\r\n\r\n").await;
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
    use svrngs_node::iroh::{
        ring_crypto_provider, EndpointAddr, EndpointBuilder, HttpBridge, SecretKey,
    };
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("cy").unwrap(),
            key: actor_of(&signing(3)),
            invite: None,
        },
        on_behalf_of: None,
    };
    pair.a.rail().commit(NS, &admit).unwrap().unwrap();
    let dialling = EndpointBuilder::empty()
        .crypto_provider(ring_crypto_provider())
        .secret_key(SecretKey::from_bytes(&signing(3).to_bytes()))
        .bind()
        .await
        .unwrap();
    let mut target = EndpointAddr::new(pair.a.endpoint().id());
    for sock in loopback_sockets(pair.a.endpoint()) {
        target = target.with_ip_addr(sock);
    }
    HttpBridge::spawn_preferring(dialling, target, Alpn::Member.bytes(), None)
        .await
        .unwrap()
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
/// could not sign them in, and the service is never asked; the next
/// request asks again. Planted: a failure carried unsigned.
#[tokio::test]
async fn a_sign_in_that_fails_or_answers_nonsense_refuses_the_request_never_carries_it() {
    let pair = Pair::new().await;
    let (origin, heads) = service().await;
    let closed = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap()
        .local_addr()
        .unwrap();
    for answers in [Some(Answers::Fails), Some(Answers::Nonsense), None] {
        let (hook, asked) = match answers {
            Some(a) => answerer(a).await,
            None => (closed, Arc::new(Mutex::new(Vec::new()))),
        };
        lend_signed(&pair, "signed", origin.port(), hook.port());
        for _ in 0..2 {
            let reply = raw_to(pair.bridge, asks("echo")).await;
            assert_eq!(reply.status, 502, "{}", reply.text);
            assert!(reply.text.contains(r#""kind":"sign-in""#), "{}", reply.text);
            assert!(
                reply.text.contains("signed could not sign you in"),
                "{}",
                reply.text
            );
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
