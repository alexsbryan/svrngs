//! What the browser tests share — this binary's and the command's joining
//! test (crates/svrngs/tests/join_page.rs), which includes this file by
//! path: the page built once per run, and iroh's relay on 127.0.0.1.

use std::net::SocketAddr;
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

/// The page, built by `scripts/build-web.sh` into target/web/site once per
/// run of this binary.
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

/// The name the tests serve the group's page at, each app at a name under
/// it (web/tests/page.mjs `PAGE_HOST`, which reaches it through a proxy).
pub const PAGE_HOST: &str = "group.svrnme.test";

/// The relay's address, `http://127.0.0.1:<port>/`: its front's ([`relay`]).
pub fn relay_url(relay: &Server) -> String {
    format!("http://{}/", relay_at(relay))
}

/// Where the relay is reached: its front ([`relay`]).
pub fn relay_at(relay: &Server) -> SocketAddr {
    let at = relay.http_addr().expect("the relay serves HTTP");
    *FRONTS.lock().unwrap().get(&at).unwrap_or(&at)
}

/// Each relay's front, by the relay's own address.
static FRONTS: std::sync::Mutex<std::collections::BTreeMap<SocketAddr, SocketAddr>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());

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

/// iroh's relay server on 127.0.0.1, plain HTTP: no TLS, no QUIC, behind a
/// front at the address [`relay_url`] gives. The front passes each
/// connection on as it comes, but ends TLS for one that opens with it
/// (0x16, a TLS handshake): a group's page over TLS asks the relay over TLS,
/// since the tests' page upgrades its insecure requests
/// (`upgrade-insecure-requests`, web/tests/page.mjs), and WebKit refuses a
/// TLS page any plain request to 127.0.0.1, as mixed content
/// (demo/runs/security/2026-11-02-webkit-partition/README.md). Nodes ask it
/// in the clear, as before.
pub async fn relay() -> Server {
    traced();
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    let server = Server::spawn(config)
        .await
        .expect("a relay starts on 127.0.0.1");
    let inner = server.http_addr().expect("the relay serves HTTP");
    let front = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    FRONTS
        .lock()
        .unwrap()
        .insert(inner, front.local_addr().unwrap());
    let ended = tokio_rustls::TlsAcceptor::from(tls());
    tokio::spawn(async move {
        while let Ok((mut asked, _)) = front.accept().await {
            let ended = ended.clone();
            tokio::spawn(async move {
                let (mut first, mut relay) =
                    ([0u8; 1], tokio::net::TcpStream::connect(inner).await?);
                if asked.peek(&mut first).await? == 1 && first[0] == 0x16 {
                    let mut asked = ended.accept(asked).await?;
                    tokio::io::copy_bidirectional(&mut asked, &mut relay).await?;
                } else {
                    tokio::io::copy_bidirectional(&mut asked, &mut relay).await?;
                }
                Ok::<_, std::io::Error>(())
            });
        }
    });
    server
}

/// The front's TLS: a certificate for 127.0.0.1 made once a run, which the
/// tests' browsers are told to take (page.mjs `launch`).
fn tls() -> std::sync::Arc<rustls::ServerConfig> {
    static TLS: OnceLock<std::sync::Arc<rustls::ServerConfig>> = OnceLock::new();
    TLS.get_or_init(|| {
        let made = rcgen::generate_simple_self_signed(vec!["127.0.0.1".into(), "localhost".into()])
            .unwrap();
        let key = rustls::pki_types::PrivatePkcs8KeyDer::from(made.signing_key.serialize_der());
        let provider = std::sync::Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![made.cert.der().clone()], key.into())
            .unwrap();
        std::sync::Arc::new(config)
    })
    .clone()
}
