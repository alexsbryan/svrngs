//! Give asks svrngs's record only once svrngs has proved, on that one
//! connection, that it holds the token (svrngs's `POST /v1/proof`): what
//! holds svrngs's address and can't prove it is sent none of the token.

#![allow(clippy::disallowed_methods, reason = "a test dials what it runs")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::Duration;

use give::record::Record;

/// The token each data dir [`data_dir_at`] makes keeps.
const TOKEN: &str = "the-token-svrngs-keeps";

/// A data dir whose `api` names 127.0.0.1:`port` and whose token is
/// [`TOKEN`], for `test`.
fn data_dir_at(test: &str, port: u16) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("give-record-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("api"), format!("http://127.0.0.1:{port}\n")).unwrap();
    std::fs::write(dir.join("token"), format!("{TOKEN}\n")).unwrap();
    dir
}

/// A program on 127.0.0.1 that isn't svrngs: one connection, its first
/// request answered as svrngs's `/v1/groups` would, and all it was sent
/// until the connection closes.
fn not_svrngs() -> (u16, std::thread::JoinHandle<String>) {
    let held = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = held.local_addr().unwrap().port();
    let heard = std::thread::spawn(move || {
        let (mut c, _) = held.accept().unwrap();
        c.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let (mut got, mut buf, mut answered) = (Vec::new(), [0; 65536], false);
        while let Ok(n @ 1..) = c.read(&mut buf) {
            got.extend_from_slice(&buf[..n]);
            if !answered && got.windows(4).any(|w| w == b"\r\n\r\n") {
                let body = r#"{"key": "k", "groups": []}"#;
                let answer = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{body}", body.len());
                answered = c.write_all(answer.as_bytes()).is_ok();
            }
        }
        String::from_utf8_lossy(&got).into_owned()
    });
    (port, heard)
}

/// What a program at svrngs's address was sent: none of the token, not one
/// byte of it, and no `authorization` at all.
fn sent_none_of_the_token(seen: &str) {
    let shown = seen.contains(TOKEN) || seen.to_ascii_lowercase().contains("authorization");
    assert!(!shown, "it was sent the token: {seen}");
}

/// **A program at svrngs's address that can't prove it holds the token is
/// sent none of it** (V1.13.3's fix review, finding 3): while svrngs is
/// down, another program holding its address is asked for svrngs's proof
/// alone, and Give's ask of the record is refused, saying so. Planted,
/// watched failing: V1.13.3's candidate (588b0abc) sent it `GET
/// /v1/groups` with `authorization: Bearer the-token-svrngs-keeps`.
#[test]
fn a_program_at_svrngs_s_address_is_sent_none_of_the_token() {
    let (port, heard) = not_svrngs();
    let asked = Record::at(&data_dir_at("squatter", port)).unwrap().groups();
    let seen = heard.join().unwrap();
    sent_none_of_the_token(&seen);
    assert!(seen.starts_with("POST /v1/proof HTTP/1.1\r\n"), "{seen}");
    let refused = asked.expect_err("refused").to_string();
    assert!(refused.contains("Something other than svrngs answers"), "{refused}");
}

/// **Another account's program at svrngs's address is sent none of the
/// token**, on a real socket of another uid: a uid of this account's own
/// subordinate range (/etc/subuid), as `unshare --map-auto` gives it
/// without root. By hand on Linux: it needs newuidmap and a range in
/// /etc/subuid. Planted, watched failing: V1.13.3's candidate (588b0abc)
/// sent it `GET /v1/groups` with the token.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "by hand on Linux: unshare --map-auto and a range in /etc/subuid"]
fn another_account_s_program_at_svrngs_s_address_is_sent_none_of_the_token() {
    use std::io::{BufRead, BufReader};
    let script = "import socket\ns = socket.socket()\ns.bind(('127.0.0.1', 0))\ns.listen()\nprint(s.getsockname()[1], flush=True)\nc, _ = s.accept()\nc.settimeout(2)\ngot = b''\ntry:\n    while b'\\r\\n\\r\\n' not in got:\n        b = c.recv(65536)\n        if not b: break\n        got += b\n    c.sendall(b'HTTP/1.1 200 OK\\r\\nContent-Length: 2\\r\\n\\r\\n{}')\n    while True:\n        b = c.recv(65536)\n        if not b: break\n        got += b\nexcept OSError: pass\nprint(got.decode('latin-1'), flush=True)";
    let theirs = [
        "--map-auto",
        "--map-root-user",
        "setpriv",
        "--reuid=1",
        "--regid=1",
        "--clear-groups",
    ];
    let mut child = std::process::Command::new("unshare")
        .args(theirs)
        .args(["python3", "-I", "-c", script])
        .current_dir("/")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("unshare runs");
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut port = String::new();
    out.read_line(&mut port).unwrap();
    let port = port.trim().parse().unwrap_or_else(|_| panic!("no port: {port:?}"));
    let asked = Record::at(&data_dir_at("theirs", port)).unwrap().groups();
    let mut seen = String::new();
    out.read_to_string(&mut seen).unwrap();
    let _ = child.wait();
    sent_none_of_the_token(&seen);
    eprintln!("RESULT: sent {seen:?}, then {asked:?}");
    assert!(asked.is_err(), "{asked:?}");
}

/// **Give's copy of svrngs's proof meets svrngs's vectors**
/// (crates/svrngs-node/src/api_tests.rs `VECTORS`).
#[test]
fn the_proof_meets_svrngs_s_vectors() {
    let vectors = [
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
    for (byte, client, server, want) in vectors {
        let ends = (client.parse().unwrap(), server.parse().unwrap());
        assert_eq!(
            give::record::proof("test-token", &byte.repeat(32), ends.0, ends.1),
            want
        );
    }
}

/// One HTTP/1.1 request read off `c`: its head and its body.
fn request(c: &mut std::net::TcpStream) -> Option<(String, String)> {
    let (mut got, mut buf) = (Vec::new(), [0; 1]);
    while !got.ends_with(b"\r\n\r\n") {
        c.read_exact(&mut buf).ok()?;
        got.push(buf[0]);
    }
    let head = String::from_utf8_lossy(&got).into_owned();
    let length = (head.lines())
        .find_map(|l| {
            l.to_ascii_lowercase()
                .strip_prefix("content-length:")
                .map(|n| n.trim().parse().unwrap())
        })
        .unwrap_or(0);
    let mut body = vec![0; length];
    c.read_exact(&mut body).ok()?;
    Some((head, String::from_utf8_lossy(&body).into_owned()))
}

/// svrngs's stand-in on 127.0.0.1, holding [`TOKEN`] and taking its first
/// connection once `after` has passed: it proves it holds the token for the
/// challenge on that connection's two ends, as svrngs's `POST /v1/proof`
/// does (`proves`; else it answers 404, as svrngs 1.13.2 does), then
/// answers `/v1/groups`. What it was asked on that connection.
fn svrngs(proves: bool, after: Duration) -> (u16, std::thread::JoinHandle<Vec<String>>) {
    let held = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = held.local_addr().unwrap().port();
    let heard = std::thread::spawn(move || {
        std::thread::sleep(after);
        let (mut c, client) = held.accept().unwrap();
        let server = c.local_addr().unwrap();
        let mut heard = Vec::new();
        while let Some((head, body)) = request(&mut c) {
            let body: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
            let challenge = body["challenge"].as_str().unwrap_or_default();
            let answer = match head.starts_with("POST /v1/proof ") {
                true if proves => serde_json::json!({ "proof": give::record::proof(TOKEN, challenge, client, server) }),
                true => serde_json::Value::Null,
                false => serde_json::json!({ "key": "k", "groups": [] }),
            };
            let (status, answer) = if answer.is_null() {
                ("404 Not Found", String::new())
            } else {
                ("200 OK", answer.to_string())
            };
            heard.push(head);
            let said = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\n\r\n{answer}", answer.len());
            c.write_all(said.as_bytes()).unwrap();
        }
        heard
    });
    (port, heard)
}

/// A program on 127.0.0.1 carrying one connection to 127.0.0.1:`to` on
/// one of its own, as a relay does: its port, and what it carried there.
fn relay(to: u16) -> (u16, std::thread::JoinHandle<String>) {
    let held = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = held.local_addr().unwrap().port();
    let carried = std::thread::spawn(move || {
        let (mut asker, _) = held.accept().unwrap();
        let mut svrngs = std::net::TcpStream::connect(("127.0.0.1", to)).unwrap();
        let (mut back, mut answered) = (svrngs.try_clone().unwrap(), asker.try_clone().unwrap());
        std::thread::spawn(move || std::io::copy(&mut back, &mut answered));
        let (mut seen, mut buf) = (Vec::new(), [0; 65536]);
        while let Ok(n @ 1..) = asker.read(&mut buf) {
            seen.extend_from_slice(&buf[..n]);
            if svrngs.write_all(&buf[..n]).is_err() {
                break;
            }
        }
        let _ = svrngs.shutdown(std::net::Shutdown::Both);
        String::from_utf8_lossy(&seen).into_owned()
    });
    (port, carried)
}

/// **svrngs proving it holds the token on a connection is sent it, on that
/// connection**: Give's ask follows the proof on the one connection,
/// showing the token; svrngs taking the connection 300 ms late, as a
/// starting svrngs does, is waited for; a relay carrying the challenge to
/// svrngs on a connection of its own carries back a proof for that one,
/// so it is refused and carries none of the token (planted, watched
/// failing: a proof over the challenge alone, which sent the relay the
/// token); a svrngs with no proof to give (404) is older than Give.
#[test]
fn svrngs_proving_it_holds_the_token_is_sent_it_on_that_connection() {
    let (port, heard) = svrngs(true, Duration::from_millis(300));
    let groups = Record::at(&data_dir_at("proved", port)).unwrap().groups();
    let heard = heard.join().unwrap();
    assert_eq!(groups.unwrap().key, "k");
    assert!(heard[0].starts_with("POST /v1/proof HTTP/1.1\r\n"), "{heard:?}");
    sent_none_of_the_token(&heard[0]);
    assert!(heard[1].starts_with("GET /v1/groups HTTP/1.1\r\n"), "{heard:?}");
    assert!(
        heard[1].contains(&format!("Authorization: Bearer {TOKEN}\r\n")),
        "{heard:?}"
    );

    let (to, _) = svrngs(true, Duration::ZERO);
    let (port, carried) = relay(to);
    let relayed = Record::at(&data_dir_at("relayed", port)).unwrap().groups();
    let carried = carried.join().unwrap();
    sent_none_of_the_token(&carried);
    let refused = relayed.expect_err("refused").to_string();
    assert!(refused.contains("Something other than svrngs answers"), "{refused}");

    let (port, _) = svrngs(false, Duration::ZERO);
    let older = Record::at(&data_dir_at("older", port)).unwrap().groups();
    let older = older.expect_err("refused").to_string();
    assert!(older.contains("older than this give: run `svrngs restart`"), "{older}");
}
