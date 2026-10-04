//! The svrngs host: one process that keeps a group's journal on this computer
//! and lets the chat write to it (DESIGN D7 E0). `home` finds the files,
//! `store` reads and appends the group's directory, `journal` is its one
//! writer, `runner` runs the apps' handlers for it, and `bridge` is the
//! loopback surface the chat adapter calls.

pub mod bridge;
pub mod home;
pub mod journal;
pub mod runner;
pub mod store;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use tokio::sync::oneshot;
use tracing::info;

pub use home::Home;

/// This computer's clock, in Unix seconds: the one read of it.
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// The bridge's sources, in the checkout this command was built from: E0
/// installs svrngs from a checkout (demo/d2-your-own-group.md), so they stay
/// beside it.
pub fn bridge_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bridge")
}

/// A running host: the journal actor and the bridge append in front of it.
pub struct Host {
    addr: SocketAddr,
    shutdown: oneshot::Sender<()>,
    server: tokio::task::JoinHandle<std::io::Result<()>>,
    actor: std::thread::JoinHandle<()>,
}

impl Host {
    /// Open the home's group and listen on loopback `port` (0 for any).
    pub async fn start(home: &Home, port: u16) -> anyhow::Result<Self> {
        let id = home.the_group()?.context("this computer keeps no group yet")?;
        let (dir, journal) = store::GroupDir::open(&home.group_dir(&id), now())?;
        let keys = home.keys(&id)?;
        let token = home.token()?;
        let runner = runner::Runner::built_in()?;
        let commands = runner.commands().into_iter().cloned().collect();
        let (handle, actor) = journal::spawn(dir, journal, keys.bridge, now, runner);
        let listener = bridge::bind(port).await?;
        let addr = listener.local_addr()?;
        let (shutdown, stop) = oneshot::channel::<()>();
        let app = bridge::router(handle, token, commands);
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = stop.await;
                })
                .await
        });
        info!(group = %id, %addr, "host:start");
        Ok(Self {
            addr,
            shutdown,
            server,
            actor,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Stop listening, then let the journal actor finish its last write.
    pub async fn stop(self) -> anyhow::Result<()> {
        let _ = self.shutdown.send(());
        self.server.await??;
        let actor = self.actor;
        tokio::task::spawn_blocking(move || actor.join())
            .await?
            .map_err(|_| anyhow::anyhow!("the journal actor panicked"))?;
        info!(addr = %self.addr, "host:stop");
        Ok(())
    }
}
