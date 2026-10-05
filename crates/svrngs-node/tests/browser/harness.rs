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

impl Engine {
    /// The name the browser halves take in their args' `engine`.
    pub fn as_str(self) -> &'static str {
        match self {
            Engine::Chromium => "chromium",
            Engine::Webkit => "webkit",
        }
    }
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

/// iroh's relay server on 127.0.0.1, plain HTTP: no TLS, no QUIC.
pub async fn relay() -> Server {
    let mut config = ServerConfig::default();
    config.relay = Some(RelayServer::new(([127, 0, 0, 1], 0)));
    Server::spawn(config)
        .await
        .expect("a relay starts on 127.0.0.1")
}
