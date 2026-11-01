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

/// Which page a test drives: the node's own choice (src/home/v2/serve.rs),
/// so one enum decides it.
pub use svrngs_node::home::v2::Ui;

/// The page this run drives: `SVRNGS_UI`, read once, v1 the live one when
/// unset (ralph/next/page/CHARTER.md, "What is settled" 6). Any other value
/// stops the run in its words, so a typo never runs v1 as if it were v2.
pub fn ui() -> Ui {
    static UI: OnceLock<Ui> = OnceLock::new();
    *UI.get_or_init(|| match std::env::var("SVRNGS_UI") {
        Err(std::env::VarError::NotPresent) => Ui::V1,
        Ok(v) if v == "v1" => Ui::V1,
        Ok(v) if v == "v2" => Ui::V2,
        other => panic!("SVRNGS_UI is v1 or v2, or unset for v1; it is {other:?}"),
    })
}

/// `node`, its home page the run's [`ui`]: chosen v2 under v2, and left
/// unchosen under v1, as a release leaves it, so a v2-only test
/// (crates/svrngs/tests/page_v2) can still choose it once; then its `/`
/// held to it ([`home_is`]).
pub async fn with_ui(node: svrngs_node::node::Node) -> svrngs_node::node::Node {
    let node = match ui() {
        Ui::V1 => node,
        Ui::V2 => node.ui(Ui::V2),
    };
    home_is(node.api_addr().port(), &node.token(), ui()).await;
    node
}

/// The words `svrngs-ui-v2` marks v2 by, in web/v2/body.html and
/// home/v2/index.html, and the trace a binary that can serve v2 carries
/// (serve.rs; build-release.sh's refusal).
const MARK: &str = "svrngs-ui-v2";

/// Whether `page`, from `whose`, is `ui`'s: it carries [`MARK`] if and only
/// if `ui` is v2, so a v2 run never silently tests v1, nor v1 a v2 page.
pub fn page_is(page: &str, ui: Ui, whose: &str) {
    match (ui, page.contains(MARK)) {
        (Ui::V2, false) => panic!("the run is v2 (SVRNGS_UI=v2), but {whose} is v1's, with no {MARK}, so a pass here would be v1's"),
        (Ui::V1, true) => panic!("the run is v1, but {whose} is v2's, marked {MARK}: SVRNGS_UI is unset or v1 here"),
        _ => {}
    }
}

/// The home page at `port`, as a person's typed address opens it, `token`
/// shown as a program shows it (home.rs `only_itself`), held to `ui`
/// ([`page_is`]): where the harness first hands out its address.
pub async fn home_is(port: u16, token: &str, ui: Ui) {
    let home = format!("svrngs.localhost:{port}");
    let http = reqwest::Client::builder()
        .no_proxy()
        .resolve("svrngs.localhost", ([127, 0, 0, 1], port).into())
        .build()
        .unwrap();
    let asked = http
        .get(format!("http://{home}/"))
        .bearer_auth(token.trim());
    let asked = asked
        .header("sec-fetch-site", "none")
        .header("sec-fetch-mode", "navigate");
    let got = asked
        .send()
        .await
        .unwrap_or_else(|e| panic!("{home}'s home page did not answer: {e}"));
    let page = got
        .text()
        .await
        .unwrap_or_else(|e| panic!("{home}'s home page: {e}"));
    page_is(&page, ui, &format!("the home page at {home}"));
}

/// What a `svrngs` this run starts is given beside its own words for the
/// run's [`ui`]: nothing under v1, as a person runs it; `--ui v2` under v2,
/// which only a binary built with svrngs's test-support takes (cli.rs
/// `parse`), so one built without it stops the run here, in words, and
/// not as a node that never answers. The browser and conformance targets
/// start none.
#[allow(dead_code)]
pub fn ui_args() -> &'static [&'static str] {
    static HELD: std::sync::Once = std::sync::Once::new();
    let bin = option_env!("CARGO_BIN_EXE_svrngs");
    HELD.call_once(|| {
        if let (Ui::V2, Some(bin)) = (ui(), bin) {
            let grep = std::process::Command::new("grep").args(["-qa", MARK, bin]).status();
            assert!(
                grep.expect("grep runs").success(),
                "the run is v2 (SVRNGS_UI=v2), but {bin} was built without svrngs's test-support, so it cannot serve v2's home page: run cargo test with --features svrngs/test-support"
            );
        }
    });
    match ui() {
        Ui::V1 => &[],
        Ui::V2 => &["--ui", "v2"],
    }
}

/// The page of the run's [`ui`] ([`site_of`]).
pub fn site() -> &'static Path {
    site_of(ui())
}

/// The page `ui`, built by `scripts/build-web.sh --ui <ui>` once per run of
/// this binary: v1 into target/web/site, v2 into target/web/site-v2.
pub fn site_of(ui: Ui) -> &'static Path {
    static V1: OnceLock<PathBuf> = OnceLock::new();
    static V2: OnceLock<PathBuf> = OnceLock::new();
    let (once, dir, name) = match ui {
        Ui::V1 => (&V1, "site", "v1"),
        Ui::V2 => (&V2, "site-v2", "v2"),
    };
    once.get_or_init(|| {
        let out = repo().join("target/web").join(dir);
        let built = std::process::Command::new(repo().join("scripts/build-web.sh"))
            .arg("--out")
            .arg(&out)
            .args(["--ui", name])
            .output()
            .expect("scripts/build-web.sh runs");
        assert!(
            built.status.success(),
            "scripts/build-web.sh --ui {name} failed, so the page cannot be tested:\n{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let page = std::fs::read_to_string(out.join("index.html")).expect("the built page");
        page_is(&page, ui, &format!("{}/index.html", out.display()));
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
