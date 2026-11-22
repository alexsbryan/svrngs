// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]
use super::*;

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

fn challenged(byte: &str) -> Challenge {
    Challenge::try_from(byte.repeat(32)).unwrap()
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

/// **The command sends the token only on a connection whose own ends the
/// proof was made for** (`proven`, as `asked` checks svrngs's answer): a
/// vector's proof proves its connection, and no other, such as the one a
/// relay opened to svrngs (planted: a proof over the challenge alone,
/// which a relay's connection proved too); 404 is a svrngs older than the
/// command; no answer is svrngs away, and none in time is unanswered;
/// anything else is not svrngs proving itself.
#[test]
fn the_command_sends_the_token_only_where_svrngs_proved_this_connection() {
    let said =
        |status, body: serde_json::Value| Some(Some((status, body.to_string().into_bytes())));
    for (byte, client, server, shown) in VECTORS {
        let (client, server): (SocketAddr, SocketAddr) =
            (client.parse().unwrap(), server.parse().unwrap());
        let wanted = proof("test-token", &challenged(byte), client, server);
        let proved = said(200, serde_json::json!({ "proof": shown }));
        assert_eq!(proven(proved.clone(), &wanted), Ok(()));
        let relayed = SocketAddr::new(client.ip(), client.port() + 2);
        let elsewhere = proof("test-token", &challenged(byte), relayed, server);
        assert_eq!(
            proven(proved.clone(), &elsewhere),
            Err(Withheld::Unproven),
            "relayed"
        );
        let other = proof("another-token", &challenged(byte), client, server);
        assert_eq!(
            proven(proved, &other),
            Err(Withheld::Unproven),
            "another svrngs"
        );
        let shouted = said(200, serde_json::json!({ "proof": shown.to_uppercase() }));
        assert_eq!(proven(shouted, &wanted), Err(Withheld::Unproven));
        assert_eq!(
            proven(said(404, serde_json::json!({})), &wanted),
            Err(Withheld::Older)
        );
        assert_eq!(
            proven(said(401, serde_json::json!({})), &wanted),
            Err(Withheld::Unproven)
        );
        assert_eq!(
            proven(said(200, serde_json::json!({ "open": shown })), &wanted),
            Err(Withheld::Unproven)
        );
        assert_eq!(proven(Some(None), &wanted), Err(Withheld::Away));
        assert_eq!(proven(None, &wanted), Err(Withheld::Unanswered));
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

/// The proof's route and one route that asks for `token`, as [`router`]
/// layers them, served by [`serve`] on 127.0.0.1 and ::1 once `after` has
/// passed since they were bound, as svrngs serves after it binds. Without
/// the proof's route, a route it doesn't serve is 404 without the token,
/// as svrngs 1.13.2 answers `POST /v1/proof` (9be0fcc7, asked 2026-11-21).
async fn proving(
    token: &Bearer,
    proves: bool,
    after: std::time::Duration,
) -> (SocketAddr, tokio::task::JoinHandle<Serving>) {
    let asked = Router::new().route("/v1/asked", post(|| async { "asked, with the token" }));
    let asked = asked.route_layer(axum::middleware::from_fn_with_state(
        token.clone(),
        holds_the_token,
    ));
    let routes = match proves {
        true => asked.route("/v1/proof", post(proved).with_state(token.clone())),
        false => asked,
    };
    let routes = routes.layer(axum::middleware::from_fn(not_from_a_page));
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
/// sent it, on that connection** (`asked` against `POST /v1/proof` served
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
async fn svrngs_proves_it_holds_the_token_before_it_is_sent_it() {
    let dir = tempfile::tempdir().unwrap();
    let token = kept_token(dir.path()).unwrap();
    let secret = token.secret().to_string();
    let late = std::time::Duration::from_millis(300);
    let (addr, served) = proving(&token, true, late).await;
    let asked = asked(
        &format!("http://{addr}\n"),
        "/v1/asked",
        &secret,
        "{}".into(),
    )
    .await;
    let _served = served.await.unwrap();
    assert_eq!(
        asked,
        Ok((200, b"asked, with the token".to_vec())),
        "served late"
    );
    let six = SocketAddr::from((Ipv6Addr::LOCALHOST, addr.port()));
    let asked = asked_at(six, &secret).await;
    assert_eq!(
        asked,
        Ok((200, b"asked, with the token".to_vec())),
        "on ::1"
    );

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
    assert_eq!(asked_at(older, &secret).await, Err(Withheld::Older));
}

/// [`asked`] of svrngs at `at`, as its `api` file names it.
async fn asked_at(at: SocketAddr, token: &str) -> Asked {
    asked(&format!("http://{at}"), "/v1/asked", token, "{}".into()).await
}
