#![cfg(target_os = "linux")]

use std::{
    io::{Read, Write},
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
            let mut request = [0; b"GET /health ".len()];
            stream.read_exact(&mut request).unwrap();
            assert_eq!(&request, b"GET /health ");
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
