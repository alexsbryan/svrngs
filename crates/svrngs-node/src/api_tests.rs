// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]
use super::*;
use axum::routing::{any, get};

/// **svrngs starts without ::1 only where this computer has no IPv6 at
/// all** (V1.13.3's second practice review, finding 5): the family refused
/// (`EAFNOSUPPORT`), and no program here can listen at ::1 either. Every
/// other refusal of ::1 at svrngs's port stops it starting, ::1 off among
/// them (`EADDRNOTAVAIL`), since another account could take ::1 there once
/// it is on and answer every `<label>.localhost` first. Planted, watched
/// failing: the candidate's rule, any refusal but "in use" carried on
/// alone (`EADDRNOTAVAIL` was let start).
#[test]
fn svrngs_starts_without_ipv6_s_loopback_only_where_there_is_no_ipv6() {
    // errno.h: EAFNOSUPPORT, EADDRNOTAVAIL, EACCES, EPERM, EINVAL.
    let (none, off) = if cfg!(target_os = "macos") {
        (47, 49)
    } else {
        (97, 99)
    };
    assert!(alone(&std::io::Error::from_raw_os_error(none)), "no IPv6");
    for refused in [off, 13, 1, 22] {
        let e = std::io::Error::from_raw_os_error(refused);
        assert!(!alone(&e), "{e} carried on without ::1");
    }
}

/// **Linux holds ::1 at svrngs's port while it is off, so nothing else
/// takes it once on**: in a network namespace whose ::1 is off, svrngs
/// binds 127.0.0.1 and ::1 (`IPV6_FREEBIND`); once ::1 is on, another
/// program's bind there is refused as in use, and a connection to ::1
/// reaches svrngs's listener. By hand, as root of a namespace of its own:
/// `unshare --map-root-user --net sh -c 'ip link set lo up && echo 1 >
/// /proc/sys/net/ipv6/conf/lo/disable_ipv6 && exec <this test binary>
/// --ignored --exact api::tests::linux_holds_ipv6_s_loopback_while_it_is_off'`.
#[cfg(target_os = "linux")]
#[tokio::test]
#[ignore = "by hand on Linux, in a network namespace whose ::1 is off"]
async fn linux_holds_ipv6_s_loopback_while_it_is_off() {
    let off = std::net::TcpListener::bind((Ipv6Addr::LOCALHOST, 0)).unwrap_err();
    assert_eq!(
        off.kind(),
        std::io::ErrorKind::AddrNotAvailable,
        "::1 is off here"
    );
    let bound = bind("127.0.0.1:0".parse().unwrap())
        .await
        .expect("svrngs starts");
    assert_eq!(bound.listeners.len(), 2, "on 127.0.0.1 and ::1");
    let six = SocketAddr::from((Ipv6Addr::LOCALHOST, bound.addr.port()));
    std::fs::write("/proc/sys/net/ipv6/conf/lo/disable_ipv6", "0").unwrap();
    let taken = std::net::TcpListener::bind(six).unwrap_err();
    assert_eq!(
        taken.kind(),
        std::io::ErrorKind::AddrInUse,
        "::1 is svrngs's once on"
    );
    let dialled = tokio::net::TcpStream::connect(six)
        .await
        .expect("::1 answers once on");
    let (_, from) = bound.listeners[1].accept().await.unwrap();
    assert_eq!(from, dialled.local_addr().unwrap());
    eprintln!("RESULT: ::1 held while off, svrngs's once on, at {six}");
}

/// The proof's vectors, which every maker and checker of it asserts:
/// this one decider, svrngs's command through it ([`asked`]), Give's copy
/// (apps/give/src/record.rs) and the Omarchy plugin's.
const VECTORS: [(&str, &str, &str, &str); 2] = [
    (
        "00",
        "127.0.0.1:50000",
        "127.0.0.1:40000",
        "e5a8107be851bcf50a977799f3dd752fb8a7db2607128372ab6936aa42fa046f",
    ),
    (
        "ab",
        "[::1]:50001",
        "[::1]:40000",
        "bdaab3ee1e00b484f65ee220d690a8b5d462ecff25a6d9f5ac898148dec38472",
    ),
];

/// The ask's vectors, which every maker and checker of its code asserts:
/// this one decider, svrngs's command through it, Give's copy, the
/// Omarchy plugin's and apps/client/svrngs.mjs. The nonce, then the ends.
const ASKED: [(&str, &str, &str, &str); 2] = [
    (
        "11",
        "127.0.0.1:50000",
        "127.0.0.1:40000",
        "12948174292b9641bc74ea49e12918bd39931f81cddef77ae9c04d46e6585245",
    ),
    (
        "cd",
        "[::1]:50001",
        "[::1]:40000",
        "d806666da2d6c78fa07dc51fe301f57596fed866e930680a54e5bbc9de104f1c",
    ),
];

fn challenged(byte: &str) -> Challenge {
    Challenge::try_from(byte.repeat(32)).unwrap()
}

/// **The ask's code is the one decider's, to its vectors** (`ask`): the
/// token `test-token`, a nonce of 32 repeated bytes and a connection's two
/// ends; never the proof's for the same bytes. Planted, watched failing:
/// the proof's message in the ask's place.
#[test]
fn the_ask_meets_its_vectors() {
    for (byte, client, server, want) in ASKED {
        let (client, server): (SocketAddr, SocketAddr) =
            (client.parse().unwrap(), server.parse().unwrap());
        assert_eq!(ask("test-token", &challenged(byte), client, server), want);
        assert_eq!(ask("test-token\n", &challenged(byte), client, server), want);
        let proved = proof("test-token", &challenged(byte), client, server);
        assert_ne!(proved, want, "the proof's");
    }
}

/// **The proof is the one decider's, to its vectors** (`proof`): the
/// token `test-token`, a challenge of 32 repeated bytes, and a
/// connection's two ends as Rust prints them, each a `SocketAddr` that
/// prints as the vector's.
#[test]
fn the_proof_meets_its_vectors() {
    for (byte, client, server, want) in VECTORS {
        let (client, server): (SocketAddr, SocketAddr) =
            (client.parse().unwrap(), server.parse().unwrap());
        assert_eq!(proof("test-token", &challenged(byte), client, server), want);
        assert_eq!(
            proof(" test-token\n", &challenged(byte), client, server),
            want,
            "the token, trimmed"
        );
    }
}

/// **The command asks only on a connection whose own ends the proof was
/// made for, with the nonce it gave** (`proven`, as `asked` checks
/// svrngs's answer): a vector's proof proves its connection, and no other,
/// such as the one a relay opened to svrngs (planted: a proof over the
/// challenge alone, which a relay's connection proved too), and gives its
/// nonce, if it names the command's own build; one naming another is
/// another build, the one it names; a proof with no nonce, or one that
/// isn't 64 lowercase hex, or no build, and 404 are a svrngs older than the
/// command, naming none; no answer is svrngs away, and none in time is
/// unanswered; anything else is not svrngs proving itself. Asked to stop
/// (surface.rs `STOP_ROUTE`), a proof of another build is taken. Planted,
/// watched failing: the build left unread, and a proof of 1.12.2 taken.
#[test]
fn the_command_asks_only_where_svrngs_proved_this_connection() {
    let said =
        |status, body: serde_json::Value| Some(Some((status, body.to_string().into_bytes())));
    let nonce = "1f".repeat(32);
    for (byte, client, server, shown) in VECTORS {
        let (client, server): (SocketAddr, SocketAddr) =
            (client.parse().unwrap(), server.parse().unwrap());
        let wanted = proof("test-token", &challenged(byte), client, server);
        let built = |b: &str| {
            said(
                200,
                serde_json::json!({ "proof": shown, "nonce": nonce, "build": b }),
            )
        };
        let proved = built(&build());
        assert_eq!(proven(proved.clone(), &wanted, false), Ok(challenged("1f")));
        let another = Withheld::Another(Some("1.12.2 (c95c6509)".into()));
        assert_eq!(
            proven(built("1.12.2 (c95c6509)"), &wanted, false),
            Err(another)
        );
        let stopped = proven(built("1.12.2 (c95c6509)"), &wanted, true);
        assert_eq!(stopped, Ok(challenged("1f")), "asked to stop, any build is");
        let unnamed = said(200, serde_json::json!({ "proof": shown, "nonce": nonce }));
        assert_eq!(
            proven(unnamed, &wanted, false),
            Err(Withheld::Another(None))
        );
        let relayed = SocketAddr::new(client.ip(), client.port() + 2);
        let elsewhere = proof("test-token", &challenged(byte), relayed, server);
        assert_eq!(
            proven(proved.clone(), &elsewhere, false),
            Err(Withheld::Unproven),
            "relayed"
        );
        let other = proof("another-token", &challenged(byte), client, server);
        assert_eq!(
            proven(proved, &other, false),
            Err(Withheld::Unproven),
            "another svrngs"
        );
        let shouted = said(
            200,
            serde_json::json!({ "proof": shown.to_uppercase(), "nonce": nonce }),
        );
        assert_eq!(proven(shouted, &wanted, false), Err(Withheld::Unproven));
        for older in [
            serde_json::json!({ "proof": shown }),
            serde_json::json!({ "proof": shown, "nonce": nonce.to_uppercase() }),
        ] {
            assert_eq!(
                proven(said(200, older), &wanted, false),
                Err(Withheld::Another(None))
            );
        }
        assert_eq!(
            proven(said(404, serde_json::json!({})), &wanted, false),
            Err(Withheld::Another(None))
        );
        assert_eq!(
            proven(said(401, serde_json::json!({})), &wanted, false),
            Err(Withheld::Unproven)
        );
        assert_eq!(
            proven(
                said(200, serde_json::json!({ "open": shown })),
                &wanted,
                false
            ),
            Err(Withheld::Unproven)
        );
        assert_eq!(proven(Some(None), &wanted, false), Err(Withheld::Away));
        assert_eq!(proven(None, &wanted, false), Err(Withheld::Unanswered));
    }
}

/// **A challenge is 64 lowercase hex characters, and nothing else**.
#[test]
fn a_challenge_is_64_lowercase_hex_characters() {
    assert!(Challenge::try_from("0a".repeat(32)).is_ok());
    for refused in [
        "0A".repeat(32),
        "0".repeat(63),
        "0".repeat(65),
        "g".repeat(64),
        String::new(),
    ] {
        assert!(Challenge::try_from(refused.clone()).is_err(), "{refused}");
    }
}

/// The proof's route and one route that asks for a code `token` makes,
/// as [`router`] layers them, served by [`serve`] on 127.0.0.1 and ::1 once `after` has
/// passed since they were bound, as svrngs serves after it binds. Without
/// the proof's route, a route it doesn't serve is 404 without the token,
/// as svrngs 1.13.2 answers `POST /v1/proof` (9be0fcc7, asked 2026-11-21).
async fn proving(
    token: &Bearer,
    proves: bool,
    after: std::time::Duration,
) -> (SocketAddr, tokio::task::JoinHandle<Serving>) {
    let asked = Router::new().route("/v1/asked", any(|| async { "asked, as you" }));
    let asked = asked.route("/v1/long", get(|| async { "x".repeat(3 << 20) }));
    let asked = asked.route_layer(axum::middleware::from_fn(holds_the_token));
    let routes = match proves {
        true => asked.route("/v1/proof", post(proved).with_state(token.clone())),
        false => asked,
    };
    let routes = routes.layer(axum::middleware::from_fn(not_from_a_page));
    let routes = routes.layer(axum::middleware::from_fn_with_state(token.clone(), as_whom));
    let bound = bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
    let addr = bound.addr;
    let served = tokio::spawn(async move {
        tokio::time::sleep(after).await;
        serve(routes, bound)
    });
    (addr, served)
}

/// A program on 127.0.0.1 carrying each connection to `to` on one of its
/// own, as a relay does: its address, and what it carried towards `to`.
async fn relay(to: SocketAddr) -> (SocketAddr, tokio::task::JoinHandle<Vec<u8>>) {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let held = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let at = held.local_addr().unwrap();
    let carried = tokio::spawn(async move {
        let (asker, _) = held.accept().await.unwrap();
        let svrngs = tokio::net::TcpStream::connect(to).await.unwrap();
        let ((mut from_asker, mut to_asker), (mut from_svrngs, mut to_svrngs)) =
            (asker.into_split(), svrngs.into_split());
        let back =
            tokio::spawn(async move { tokio::io::copy(&mut from_svrngs, &mut to_asker).await });
        let (mut seen, mut buf) = (Vec::new(), [0; 65536]);
        while let Ok(n @ 1..) = from_asker.read(&mut buf).await {
            seen.extend_from_slice(&buf[..n]);
            if to_svrngs.write_all(&buf[..n]).await.is_err() {
                break;
            }
        }
        back.abort();
        seen
    });
    (at, carried)
}

/// **svrngs proves it holds the token on a connection, and only then is
/// asked on it, with the code for it** (`asked` against `POST /v1/proof` served
/// as svrngs serves it): on 127.0.0.1 and on ::1, svrngs is asked and
/// answers; serving 300 ms after it binds, as a starting svrngs does,
/// waited for and asked, never refused (V1.13.3's fix review, findings 1
/// and 2); a relay that carries the challenge to svrngs on a connection of
/// its own carries back a proof for that connection, so it is refused and
/// carries none of the token (planted, watched failing: a proof over the
/// challenge alone, which sent the relay the token); a svrngs with no
/// proof route is older; a page in a browser and a challenge that isn't
/// one are refused.
#[tokio::test(flavor = "multi_thread")]
async fn svrngs_proves_it_holds_the_token_before_it_is_asked() {
    let dir = tempfile::tempdir().unwrap();
    let token = kept_token(dir.path()).unwrap();
    let secret = token.secret().to_string();
    let late = std::time::Duration::from_millis(300);
    let (addr, served) = proving(&token, true, late).await;
    let asked = asked(
        &format!("http://{addr}\n"),
        "/v1/asked",
        &secret,
        Some("{}".into()),
    )
    .await;
    let _served = served.await.unwrap();
    assert_eq!(asked, Ok((200, b"asked, as you".to_vec())), "served late");
    let six = SocketAddr::from((Ipv6Addr::LOCALHOST, addr.port()));
    let asked = asked_at(six, &secret).await;
    assert_eq!(asked, Ok((200, b"asked, as you".to_vec())), "on ::1");

    let (through, carried) = relay(addr).await;
    assert_eq!(
        asked_at(through, &secret).await,
        Err(Withheld::Unproven),
        "relayed"
    );
    let carried = String::from_utf8_lossy(&carried.await.unwrap()).into_owned();
    assert!(
        carried.starts_with("POST /v1/proof HTTP/1.1\r\n"),
        "{carried}"
    );
    assert!(
        !carried.contains(&secret) && !carried.to_lowercase().contains("authorization"),
        "{carried}"
    );

    let raw = |head: &str, body: &str| {
        let mut c = std::net::TcpStream::connect(addr).unwrap();
        let asked = format!("POST /v1/proof HTTP/1.1\r\nHost: {addr}\r\n{head}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        std::io::Write::write_all(&mut c, asked.as_bytes()).unwrap();
        let mut answer = String::new();
        std::io::Read::read_to_string(&mut c, &mut answer).unwrap();
        answer
    };
    let fresh = format!(r#"{{"challenge": "{}"}}"#, "0a".repeat(32));
    assert!(
        raw("", &fresh).starts_with("HTTP/1.1 200 OK"),
        "a program, no token"
    );
    for page in [
        "Origin: http://svrngs.localhost:1\r\n",
        "Sec-Fetch-Site: same-site\r\n",
    ] {
        assert!(raw(page, &fresh).starts_with("HTTP/1.1 403"), "{page}");
    }
    for refused in [
        r#"{"challenge": "0A"}"#.to_string(),
        String::new(),
        fresh.replace('{', r#"{"more": 1, "#),
    ] {
        assert!(raw("", &refused).starts_with("HTTP/1.1 400"), "{refused}");
    }

    let (older, served) = proving(&token, false, std::time::Duration::ZERO).await;
    let _served = served.await.unwrap();
    assert_eq!(asked_at(older, &secret).await, Err(Withheld::Another(None)));
}

/// [`asked`] of svrngs at `at`, as its `api` file names it.
async fn asked_at(at: SocketAddr, token: &str) -> Asked {
    asked(
        &format!("http://{at}"),
        "/v1/asked",
        token,
        Some("{}".into()),
    )
    .await
}

/// `text`, a raw HTTP/1 request's head, and `body` sent on `on`: the
/// answer's status and body, read by its length, the connection kept.
async fn said(on: &mut tokio::net::TcpStream, text: &str, body: &str) -> (u16, String) {
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    let asked = format!("{text}Content-Length: {}\r\n\r\n{body}", body.len());
    on.write_all(asked.as_bytes()).await.unwrap();
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        head.push(on.read_u8().await.unwrap());
    }
    let head = String::from_utf8(head).unwrap().to_ascii_lowercase();
    let length = (head.lines().find_map(|l| l.strip_prefix("content-length:")))
        .map_or(0, |n| n.trim().parse().unwrap());
    let mut said = vec![0; length];
    on.read_exact(&mut said).await.unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, String::from_utf8(said).unwrap())
}

/// The nonce svrngs's proof gives `on`, asked on it.
async fn nonce_of(on: &mut tokio::net::TcpStream) -> Challenge {
    let head = "POST /v1/proof HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\n";
    let (status, answer) = said(
        on,
        head,
        &format!(r#"{{"challenge":"{}"}}"#, "0a".repeat(32)),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    let answer: serde_json::Value = serde_json::from_str(&answer).unwrap();
    Challenge::try_from(answer["nonce"].as_str().unwrap().to_string()).unwrap()
}

/// **A code asks only on the connection, and after the proof, it was made
/// for; and svrngs's token itself is refused** (`Bearer::asked`, `as_whom`,
/// `holds_the_token`, served as svrngs serves them): each answered 401 in
/// words that say what to do, a Bearer that isn't the token as no code.
/// Planted, each watched failing: another Bearer said to be the token; a
/// code with
/// none of its connection's ends in it (the relay's, made for the asker's
/// own ends, was taken); one nonce for every connection, the last proof's
/// (a code for an unproved connection was taken); every nonce svrngs gave
/// kept, not this connection's alone (another connection's was taken); a
/// second proof keeping the first's nonce (the code before it was taken);
/// the first `Authorization` alone read (its code, then the token, taken).
#[tokio::test(flavor = "multi_thread")]
async fn a_code_asks_only_on_the_connection_and_after_the_proof_it_was_made_for() {
    let dir = tempfile::tempdir().unwrap();
    let token = kept_token(dir.path()).unwrap();
    let secret = token.secret().to_string();
    let (addr, served) = proving(&token, true, std::time::Duration::ZERO).await;
    let _served = served.await.unwrap();
    let open = || async { tokio::net::TcpStream::connect(addr).await.unwrap() };
    let code = |nonce: &Challenge, on: &tokio::net::TcpStream| {
        let (client, server) = (on.local_addr().unwrap(), on.peer_addr().unwrap());
        format!("Svrngs {}", ask(&secret, nonce, client, server))
    };
    let asking =
        |code: &str| format!("GET /v1/asked HTTP/1.1\r\nHost: {addr}\r\nAuthorization: {code}\r\n");
    let refused = |answer: (u16, String), why: Unasked, row: &str| {
        let words = serde_json::json!({ "error": why.to_string(), "kind": if why == Unasked::Bearer { "no_bearer" } else { "unasked" } });
        let said = serde_json::from_str(&answer.1).unwrap_or(serde_json::Value::String(answer.1));
        assert_eq!((answer.0, said), (401, words), "{row}");
    };

    let mut a = open().await;
    let first = nonce_of(&mut a).await;
    let mine = code(&first, &a);
    assert_eq!(
        said(&mut a, &asking(&mine), "").await,
        (200, "asked, as you".into())
    );
    assert_eq!(
        said(&mut a, &asking(&mine), "").await,
        (200, "asked, as you".into()),
        "the same code, again on its connection"
    );

    let mut b = open().await;
    let pooled = code(&first, &b);
    refused(
        said(&mut b, &asking(&pooled), "").await,
        Unasked::Unproved,
        "a connection nothing was proved on, its code made for another's nonce",
    );
    refused(
        said(&mut b, &asking(&mine), "").await,
        Unasked::Unproved,
        "another connection's code",
    );
    let second = nonce_of(&mut b).await;
    assert_ne!(first, second, "a nonce for each connection");
    refused(
        said(&mut b, &asking(&pooled), "").await,
        Unasked::Elsewhere,
        "another connection's nonce",
    );
    let theirs = code(&second, &b);
    assert_eq!(said(&mut b, &asking(&theirs), "").await.0, 200);

    let again = nonce_of(&mut a).await;
    refused(
        said(&mut a, &asking(&mine), "").await,
        Unasked::Elsewhere,
        "the code from before a second proof",
    );
    let now = code(&again, &a);
    assert_eq!(said(&mut a, &asking(&now), "").await.0, 200);

    let (through, _carried) = relay(addr).await;
    let mut relayed = tokio::net::TcpStream::connect(through).await.unwrap();
    let theirs = nonce_of(&mut relayed).await;
    let own_ends = code(&theirs, &relayed);
    refused(
        said(&mut relayed, &asking(&own_ends), "").await,
        Unasked::Elsewhere,
        "a relay's connection's nonce, the code made for the asker's own ends",
    );

    let bearer = format!("Bearer {secret}");
    refused(
        said(&mut a, &asking(&bearer), "").await,
        Unasked::Bearer,
        "the token itself",
    );
    refused(
        said(&mut a, &asking("Bearer another"), "").await,
        Unasked::Nothing,
        "another Bearer, not the token",
    );
    let none = format!("GET /v1/asked HTTP/1.1\r\nHost: {addr}\r\n");
    refused(said(&mut a, &none, "").await, Unasked::Nothing, "nothing");
    refused(
        said(&mut a, &asking("Svrngs as-you"), "").await,
        Unasked::Elsewhere,
        "not a code",
    );

    let twice = |one: &str, two: &str| {
        format!("GET /v1/asked HTTP/1.1\r\nHost: {addr}\r\nAuthorization: {one}\r\nAuthorization: {two}\r\n")
    };
    for (one, two, why, row) in [
        (
            now.as_str(),
            bearer.as_str(),
            Unasked::Bearer,
            "its code, then the token",
        ),
        (&bearer, &now, Unasked::Bearer, "the token, then its code"),
        (&now, &now, Unasked::Twice, "its code twice"),
        (
            &now,
            "Bearer another",
            Unasked::Twice,
            "its code, then another Bearer",
        ),
    ] {
        refused(said(&mut a, &twice(one, two), "").await, why, row);
    }
    assert_eq!(
        said(&mut a, &asking(&now), "").await.0,
        200,
        "its code alone, after"
    );
}

/// **svrngs's answer to its command is never cut short in silence** (the
/// never-send-the-key review, finding 6): an answer of 3 MiB, past what is
/// read of anything before it proves itself, is read whole once svrngs has
/// proved itself, as `svrngs ask` prints it. Planted, watched failing: the
/// proved answer read as an unproved one is, its status alone, 200 and
/// nothing, which `svrngs ask` printed as an empty success.
#[tokio::test(flavor = "multi_thread")]
async fn a_long_answer_is_read_whole_once_svrngs_proved_itself() {
    let dir = tempfile::tempdir().unwrap();
    let token = kept_token(dir.path()).unwrap();
    let (addr, served) = proving(&token, true, std::time::Duration::ZERO).await;
    let _served = served.await.unwrap();
    let asked = asked(&format!("http://{addr}"), "/v1/long", token.secret(), None).await;
    let (status, said) = asked.expect("svrngs proves itself and answers");
    assert_eq!((status, said.len()), (200, 3 << 20));
}

/// **A svrngs proving itself with no nonce is older than the command, told
/// apart from what never proves itself, and neither is asked anything
/// more** (the never-send-the-key review's gate): on real sockets, a right
/// proof and no nonce, as dd2efe63's svrngs answers, is `Another`, which the
/// command restarts for or says to update; a proof under another token is
/// `Unproven`; and nothing past the proof is sent to either. Planted,
/// watched failing: a proof with no nonce taken (the ask sent, with a code
/// for no nonce).
#[tokio::test(flavor = "multi_thread")]
async fn an_older_proof_and_none_are_told_apart_and_neither_is_asked_more() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let dir = tempfile::tempdir().unwrap();
    let secret = kept_token(dir.path()).unwrap().secret().to_string();
    for (with, want) in [
        (secret.clone(), Withheld::Another(None)),
        ("another".into(), Withheld::Unproven),
    ] {
        let more = std::sync::Arc::new(AtomicUsize::new(0));
        let counted = more.clone();
        let proves = move |request: Request| async move {
            let client = crate::session::peer(&request).unwrap();
            let server = request.extensions().get::<Line>().unwrap().server.unwrap();
            let body = axum::body::to_bytes(request.into_body(), 1024)
                .await
                .unwrap();
            let Challenged { challenge } = serde_json::from_slice(&body).unwrap();
            Json(serde_json::json!({ "proof": proof(&with, &challenge, client, server) }))
        };
        let routes = Router::new().route("/v1/proof", post(proves));
        let routes = routes.fallback(move || {
            counted.fetch_add(1, Ordering::SeqCst);
            async {}
        });
        let bound = bind("127.0.0.1:0".parse().unwrap()).await.unwrap();
        let addr = bound.addr;
        let _served = serve(routes, bound);
        assert_eq!(asked_at(addr, &secret).await, Err(want.clone()));
        assert_eq!(
            more.load(Ordering::SeqCst),
            0,
            "{want:?}: asked more than the proof"
        );
    }
}
