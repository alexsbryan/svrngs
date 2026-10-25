//! What the browser tests share — this binary's and the command's joining
//! test (crates/svrngs/tests/join_page.rs), which includes this file by
//! path: the page built once per run, and iroh's relay on 127.0.0.1.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use iroh_relay::server::{RelayConfig as RelayServer, Server, ServerConfig};

/// The browsers every test drives (u4-7): Chromium as a desktop opens the
/// page, and WebKit as an iPhone does, the engine of iOS Safari
/// (web/tests/page.mjs).
#[derive(Clone, Copy, Debug)]
pub enum Engine {
    Chromium,
    Webkit,
}

/// `in_both_engines!(test)`: the async fn `test(engine)` run as two tests,
/// `test::chromium` and `test::webkit`, so a failure names its engine; the
/// doc comments before it are the pair's.
macro_rules! in_both_engines {
    ($(#[$doc:meta])* $test:ident) => {
        $(#[$doc])*
        mod $test {
            #[tokio::test(flavor = "multi_thread")]
            async fn chromium() {
                super::$test($crate::harness::Engine::Chromium).await
            }
            #[tokio::test(flavor = "multi_thread")]
            async fn webkit() {
                super::$test($crate::harness::Engine::Webkit).await
            }
        }
    };
}

/// `in_chromium!(test)`: a test of a computer's page — the home page, a
/// lend's or carried app's page opened on a computer — run once, as
/// `test::chromium`, in the engine a desktop opens it in. Both engines are
/// for a test whose subject differs between them, `in_both_engines!`; a
/// phone's page is `in_webkit!`'s (the operator, 2026-10-25: each page in
/// the engine it stands for, "we can't be so maximalist here").
macro_rules! in_chromium {
    ($(#[$doc:meta])* $test:ident) => {
        $(#[$doc])*
        mod $test {
            #[tokio::test(flavor = "multi_thread")]
            async fn chromium() {
                super::$test($crate::harness::Engine::Chromium).await
            }
        }
    };
}

/// `in_webkit!(test)`: a test of a phone's page — the group's page, a
/// lend opened from it, its worker — run once, as `test::webkit`, in
/// WebKit as an iPhone opens it, the engine of iOS Safari.
macro_rules! in_webkit {
    ($(#[$doc:meta])* $test:ident) => {
        $(#[$doc])*
        mod $test {
            #[tokio::test(flavor = "multi_thread")]
            async fn webkit() {
                super::$test($crate::harness::Engine::Webkit).await
            }
        }
    };
}

impl Engine {
    /// The name the browser halves take in their args' `engine`.
    pub fn as_str(self) -> &'static str {
        match self {
            Engine::Chromium => "chromium",
            Engine::Webkit => "webkit",
        }
    }
}

/// The run's one Chromium and one WebKit, which every driver connects to
/// for a context of its own (web/tests/browsers.mjs; page.mjs `launch`):
/// their addresses, as each driver is given them in `SVRNGS_BROWSERS`,
/// started at the first ask and closed as this binary exits, its pipe to
/// them closing. A browser a test, some 250 a gate, crashed macOS's
/// pasteboard daemon, which each browser joins as it starts and leaves as
/// it quits (`-[_CFPasteboardStore dealloc]`, ~/Library/Logs/
/// DiagnosticReports/pboard-*.ips, 27 from 2026-10-21 to 25), and with it
/// every program on the Mac that touched the clipboard.
pub fn browsers() -> &'static str {
    static SHARED: OnceLock<(std::process::Child, String)> = OnceLock::new();
    let (_, said) = SHARED.get_or_init(|| {
        let mut run = std::process::Command::new("node")
            .arg(repo().join("web/tests/browsers.mjs"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("node runs web/tests/browsers.mjs");
        let mut line = String::new();
        let out = std::io::BufReader::new(run.stdout.take().unwrap());
        std::io::BufRead::read_line(&mut { out }, &mut line).expect("browsers.mjs answers");
        assert!(
            line.starts_with('{'),
            "web/tests/browsers.mjs did not start Chromium and WebKit, so no page can be tested: {line:?}; install them with: npm ci --prefix web && npx --prefix web playwright install chromium webkit"
        );
        (run, line.trim().to_string())
    });
    said
}

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The page, built by `scripts/build-web.sh` once per run of this binary.
pub fn site() -> &'static Path {
    static SITE: OnceLock<PathBuf> = OnceLock::new();
    SITE.get_or_init(|| {
        let out = repo().join("target/web/site");
        let built = std::process::Command::new(repo().join("scripts/build-web.sh"))
            .arg("--out")
            .arg(&out)
            .output()
            .expect("scripts/build-web.sh runs");
        assert!(
            built.status.success(),
            "scripts/build-web.sh failed, so the page cannot be tested:\n{}",
            String::from_utf8_lossy(&built.stderr)
        );
        out
    })
}

/// The relay's address, `http://127.0.0.1:<port>/`.
pub fn relay_url(relay: &Server) -> String {
    format!(
        "http://{}/",
        relay.http_addr().expect("the relay serves HTTP")
    )
}

/// The relay's, the doors' and the nodes' debug lines, every test's that
/// this binary runs, kept in `target/traces/<binary>.log`, so a miss in a
/// full check can be read after (bash-film-load): which ranges reached a
/// door, and whether the relay dropped a browser's socket.
fn traced() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let kept = repo().join("target/traces");
        std::fs::create_dir_all(&kept).unwrap();
        let log =
            std::fs::File::create(kept.join(format!("{}.log", env!("CARGO_CRATE_NAME")))).unwrap();
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new(
                "door=debug,carriage=debug,lend=debug,transport=debug,iroh_relay=debug",
            ))
            .with_writer(std::sync::Mutex::new(log))
            .with_ansi(false)
            .init();
    });
}

/// iroh's relay server on 127.0.0.1, plain HTTP: no TLS, no QUIC.
pub async fn relay() -> Server {
    traced();
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    Server::spawn(config)
        .await
        .expect("a relay starts on 127.0.0.1")
}
