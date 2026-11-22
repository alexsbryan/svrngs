//! Give asks svrngs's record only once svrngs has proved, on that one
//! connection, that it holds the token (svrngs's `POST /v1/proof`), and
//! then with a code for that connection alone, never the token: what holds
//! svrngs's address and can't prove it is sent none of the token, nor a
//! code.

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
    assert!(refused.contains("give sent it none of your token"), "{refused}");
}

/// **What answers at svrngs's address is read no further than a proof**
/// (the 1.13.3 proof review, finding 7): a program answering Give's
/// challenge with 64 MiB, and a head of one endless line, is read past
/// neither's bound, and Give says something other than svrngs answers.
/// Planted, watched failing: dd2efe63's Give read all 64 MiB.
#[test]
fn an_endless_answer_is_read_no_further_than_a_proof() {
    for head in [
        "HTTP/1.1 200 OK\r\nContent-Length: 67108864\r\n\r\n",
        "HTTP/1.1 200 OK\r\nX: ",
    ] {
        let held = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = held.local_addr().unwrap().port();
        let sending = std::thread::spawn(move || {
            let (mut c, _) = held.accept().unwrap();
            let _ = c.read(&mut [0; 65536]);
            let mut sent = c.write(head.as_bytes()).unwrap_or(0);
            while sent < 64 << 20 {
                match c.write(&[b'x'; 65536]) {
                    Ok(n) => sent += n,
                    Err(_) => break,
                }
            }
            sent
        });
        let asked = Record::at(&data_dir_at("endless", port)).unwrap().groups();
        let sent = sending.join().unwrap();
        assert!(sent < 16 << 20, "Give read {sent} bytes of it");
        let refused = asked.expect_err("refused").to_string();
        assert!(
            refused.contains("Something other than svrngs answers"),
            "{head:?}: {refused}"
        );
    }
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

/// **Give's copies of svrngs's proof and code meet svrngs's vectors**
/// (crates/svrngs-node/src/api_tests.rs `VECTORS`, `ASKED`). Planted,
/// watched failing: the proof's first line in the code's.
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
    let asked = [
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
    for (byte, client, server, want) in asked {
        let ends = (client.parse().unwrap(), server.parse().unwrap());
        assert_eq!(give::record::ask("test-token", &byte.repeat(32), ends.0, ends.1), want);
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

/// The nonce [`svrngs`] gives its connection.
const NONCE: &str = "1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f";

/// svrngs's stand-in on 127.0.0.1, holding [`TOKEN`] and taking its first
/// connection once `after` has passed: it proves it holds the token for the
/// challenge on that connection's two ends and gives it the nonce
/// `proves`, as svrngs's `POST /v1/proof` does (else it answers 404, as
/// svrngs 1.13.2 does), then answers `/v1/groups` asked with the code for
/// that nonce and connection, and 401 anything else. What it was asked on
/// that connection.
fn svrngs(proves: Option<&'static str>, after: Duration) -> (u16, std::thread::JoinHandle<Vec<String>>) {
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
            let code = format!(
                "Authorization: Svrngs {}\r\n",
                give::record::ask(TOKEN, NONCE, client, server)
            );
            let answer = match head.starts_with("POST /v1/proof ") {
                true if proves.is_some() => {
                    serde_json::json!({ "proof": give::record::proof(TOKEN, challenge, client, server), "nonce": proves })
                }
                true => serde_json::Value::Null,
                false if head.contains(&code) => serde_json::json!({ "key": "k", "groups": [] }),
                false => serde_json::json!({ "kind": "unasked" }),
            };
            let (status, answer) = match (answer.is_null(), answer["kind"].is_string()) {
                (true, _) => ("404 Not Found", String::new()),
                (_, true) => ("401 Unauthorized", answer.to_string()),
                _ => ("200 OK", answer.to_string()),
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

/// **svrngs proving it holds the token on a connection is asked on that
/// connection, with the code for it**: Give's ask follows the proof on the
/// one connection, with the code for the nonce the proof gave, never the
/// token; svrngs taking the connection 300 ms late, as a starting svrngs
/// does, is waited for; a relay carrying the challenge to svrngs on a
/// connection of its own carries back a proof for that one, so it is
/// refused and carries none of the token (planted, watched failing: a
/// proof over the challenge alone, which sent the relay the token); a
/// svrngs with no proof to give (404), or a proof and no nonce, is older
/// than Give.
#[test]
fn svrngs_proving_it_holds_the_token_is_asked_on_that_connection() {
    let (port, heard) = svrngs(Some(NONCE), Duration::from_millis(300));
    let groups = Record::at(&data_dir_at("proved", port)).unwrap().groups();
    let heard = heard.join().unwrap();
    assert_eq!(groups.unwrap().key, "k");
    assert!(heard[0].starts_with("POST /v1/proof HTTP/1.1\r\n"), "{heard:?}");
    sent_none_of_the_token(&heard[0]);
    assert!(heard[1].starts_with("GET /v1/groups HTTP/1.1\r\n"), "{heard:?}");
    assert!(heard[1].contains("\r\nAuthorization: Svrngs "), "{heard:?}");
    assert!(!heard[1].contains(TOKEN), "{heard:?}");

    let (to, _) = svrngs(Some(NONCE), Duration::ZERO);
    let (port, carried) = relay(to);
    let relayed = Record::at(&data_dir_at("relayed", port)).unwrap().groups();
    let carried = carried.join().unwrap();
    sent_none_of_the_token(&carried);
    let refused = relayed.expect_err("refused").to_string();
    assert!(refused.contains("Something other than svrngs answers"), "{refused}");

    for (row, proves) in [("no proof", None), ("a proof and no nonce", Some(""))] {
        let (port, heard) = svrngs(proves, Duration::ZERO);
        let older = Record::at(&data_dir_at("older", port)).unwrap().groups();
        let older = older.expect_err("refused").to_string();
        assert!(
            older.contains("older than this give: run `svrngs restart`"),
            "{row}: {older}"
        );
        assert_eq!(heard.join().unwrap().len(), 1, "{row}: asked the proof alone");
    }
}

/// **Give's record never prints svrngs's token** (the never-send-the-key
/// review, finding 3): `Record` debug-printed, plainly or pretty, as a
/// trace or a panic would print it, shows where svrngs listens and none of
/// the token. Planted, watched failing: `Loopback`'s `Debug` derived.
#[test]
fn a_record_never_prints_the_token() {
    let record = Record::at(&data_dir_at("debug", 1)).unwrap();
    for shown in [format!("{record:?}"), format!("{record:#?}")] {
        assert!(!shown.contains(TOKEN), "{shown}");
        assert!(shown.contains("127.0.0.1:1"), "{shown}");
    }
}

/// svrngs's stand-in on 127.0.0.1 proving it holds [`TOKEN`] for its first
/// connection, in the right words, a byte at a time every `each`, and
/// then reading whatever else it is sent: its port, and what it heard.
fn dribbling(each: Duration) -> (u16, std::thread::JoinHandle<Vec<String>>) {
    let held = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = held.local_addr().unwrap().port();
    let heard = std::thread::spawn(move || {
        let (mut c, client) = held.accept().unwrap();
        let server = c.local_addr().unwrap();
        let (head, body) = request(&mut c).unwrap();
        let body: serde_json::Value = serde_json::from_str(&body).unwrap();
        let proof = give::record::proof(TOKEN, body["challenge"].as_str().unwrap(), client, server);
        let said = serde_json::json!({ "proof": proof, "nonce": NONCE }).to_string();
        let said = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{said}", said.len());
        let mut heard = vec![head];
        for byte in said.as_bytes() {
            std::thread::sleep(each);
            if c.write_all(&[*byte]).is_err() {
                return heard;
            }
        }
        heard.extend(request(&mut c).map(|(head, _)| head));
        heard
    });
    (port, heard)
}

/// **svrngs has one deadline to prove itself, the whole proof's** (the
/// never-send-the-key review, finding 5): a stand-in giving the right
/// proof a byte every 20 ms, too slowly to finish within Give's 300 ms of
/// a test's clock, though each byte comes well within it, is refused
/// within that deadline, saying svrngs didn't answer, and is asked nothing
/// more. Planted, watched failing: 96ea3da7's Give waited each read alone,
/// so it took the whole dribbled proof and then asked.
#[test]
fn a_proof_dribbled_past_its_deadline_is_refused_in_time() {
    let (port, heard) = dribbling(Duration::from_millis(20));
    let record = Record::within(&data_dir_at("dribbled", port), Duration::from_millis(300)).unwrap();
    let began = std::time::Instant::now();
    let refused = record.groups().expect_err("refused").to_string();
    let took = began.elapsed();
    assert!(took < Duration::from_secs(1), "refused after {took:?}");
    assert!(refused.contains("did not answer"), "{refused}");
    let heard = heard.join().unwrap();
    assert_eq!(heard.len(), 1, "asked the proof alone: {heard:?}");
}
