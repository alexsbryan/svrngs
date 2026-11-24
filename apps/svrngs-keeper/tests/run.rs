#![cfg(target_os = "linux")]

use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    process::Command,
    time::Duration,
};

#[test]
fn refuses_every_listen_address_before_writing() {
    for addr in ["0.0.0.0:0", "127.0.0.1:0", "[::1]:0"] {
        let listener = TcpListener::bind(addr).unwrap();
        let port = listener.local_addr().unwrap().port().to_string();
        let scratch = tempfile::tempdir().unwrap();
        let got = Command::new(env!("CARGO_BIN_EXE_svrngs-keeper"))
            .env("XDG_DATA_HOME", scratch.path())
            .env("XDG_CONFIG_HOME", scratch.path())
            .args(["run", "test-busy", "--port", &port, "--", "/bin/false"])
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&got.stderr);
        assert!(
            !got.status.success()
                && said.contains(&format!("Port {port} is in use by "))
                && said.contains("the copy you started: stop it, then press Run."),
            "{addr}: {said}"
        );
        assert_eq!(scratch.path().read_dir().unwrap().count(), 0);
        assert!(listener.local_addr().is_ok());
        eprintln!("{addr}: {}", said.lines().next().unwrap());
    }
}

#[test]
fn readiness_needs_an_answer_and_llama_must_finish_loading() {
    use svrngs_keeper::host::wait_for_answer;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    assert!(
        !wait_for_answer(port, false, Duration::from_millis(100)),
        "an open but silent socket is not an HTTP answer"
    );
    drop(listener);

    for (status, ready) in [(503, false), (200, true)] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let serving = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            assert_ne!(reader.read_line(&mut line).unwrap(), 0);
            assert!(line.starts_with("GET /health "));
            // Closing with unread request bytes can reset the connection.
            loop {
                line.clear();
                assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
            }
            write!(stream, "HTTP/1.0 {status} test\r\nContent-Length: 0\r\n\r\n").unwrap();
        });
        assert_eq!(
            wait_for_answer(port, true, Duration::from_millis(500)),
            ready,
            "llama health {status}"
        );
        serving.join().unwrap();
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "bound the test fixture's rendezvous")]
fn a_foreign_answer_after_the_busy_check_does_not_verify_run() {
    use std::{
        fs,
        io::{BufRead, BufReader},
        os::unix::fs::PermissionsExt,
        time::Instant,
    };

    for (state, group) in [("failed", ""), ("active", "/svrngs-keeper-test-foreign.service")] {
        let scratch = tempfile::tempdir().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        for (name, body) in [
            ("loginctl", "#!/bin/sh\necho Linger=yes\n".to_owned()),
            (
                "systemctl",
                format!(
                    "#!/bin/sh\ncase \"$2\" in
                show) case \"$5\" in
                    LoadState) echo LoadState=not-found ;;
                    *) echo ActiveState={state}; echo ControlGroup={group} ;;
                esac ;;
                enable) touch starting; for i in $(seq 100); do
                    if test -f bound; then exit 0; fi; sleep .02
                done; exit 1 ;;
            esac\n"
                ),
            ),
        ] {
            let path = scratch.path().join(name);
            fs::write(&path, body).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let dir = scratch.path().to_owned();
        let serving = std::thread::spawn(move || {
            let end = Instant::now() + Duration::from_secs(5);
            while !dir.join("starting").exists() {
                assert!(Instant::now() < end, "the stand-in manager did not start the unit");
                std::thread::sleep(Duration::from_millis(5));
            }
            let listener = TcpListener::bind(addr).unwrap();
            fs::write(dir.join("bound"), "").unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
            let mut reader = BufReader::new(&mut stream);
            loop {
                let mut line = String::new();
                assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
            }
            write!(stream, "HTTP/1.0 200 OK\r\nContent-Length: 0\r\n\r\n").unwrap();
            listener
        });
        let got = Command::new(env!("CARGO_BIN_EXE_svrngs-keeper"))
            .current_dir(scratch.path())
            .env("HOME", scratch.path())
            .env("XDG_DATA_HOME", scratch.path())
            .env("XDG_CONFIG_HOME", scratch.path())
            .env("PATH", format!("{}:/usr/bin:/bin", scratch.path().display()))
            .args([
                "run",
                "test-foreign",
                "--json",
                "--port",
                &addr.port().to_string(),
                "--",
                "/bin/false",
            ])
            .output()
            .unwrap();
        assert!(got.status.success(), "{}", String::from_utf8_lossy(&got.stderr));
        let _foreign = serving.join().unwrap();
        let said: serde_json::Value = serde_json::from_slice(&got.stdout).unwrap();
        eprintln!("manager={state}, foreign HTTP 200: {said}");
        assert_eq!(said["kind"], "unverified");
        assert_eq!(said["state"], state);
        assert_eq!(said["ready"], true);
    }
}
