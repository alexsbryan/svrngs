//! `svrngs start [--home DIR]`: set this computer up for a group's chat, in
//! DESIGN C11's five steps (`start.rs`).
//!
//! `svrngs serve [--home DIR] [--name NAME] [--port N]`: keep this computer's
//! group, starting one named NAME if there is none, and listen for the chat.
//! Once `svrngs start` has set up a chat, `serve` runs the bridge too, and
//! stops when it stops, so whatever restarts `serve` restarts both.

use std::net::SocketAddr;

use anyhow::{Context, bail};
use svrngs::home::Chat;
use svrngs::{Home, Host, bridge};
use tokio::process::{Child, Command};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const USAGE: &str = "usage: svrngs start [--home DIR] | svrngs serve [--home DIR] [--name NAME] [--port N]";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // `start` talks to a person, so its log stays quiet unless RUST_LOG asks.
    let quiet = if args.first().is_some_and(|c| c == "start") {
        "warn"
    } else {
        "info"
    };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(quiet)))
        .with_writer(std::io::stderr)
        .init();
    match args.split_first() {
        Some((cmd, rest)) if cmd == "serve" => serve(rest).await,
        Some((cmd, rest)) if cmd == "start" => start(rest),
        _ => bail!(USAGE),
    }
}

fn start(args: &[String]) -> anyhow::Result<()> {
    let home = match args {
        [] => Home::from_env()?,
        [flag, dir] if flag == "--home" => Home::new(dir),
        _ => bail!(USAGE),
    };
    svrngs::start::run(&home)
}

async fn serve(args: &[String]) -> anyhow::Result<()> {
    let (mut home, mut name, mut port) = (Home::from_env()?, None, bridge::DEFAULT_PORT);
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let value = it.next().with_context(|| format!("{flag} needs a value; {USAGE}"))?;
        match flag.as_str() {
            "--home" => home = Home::new(value),
            "--name" => name = Some(value.clone()),
            "--port" => port = value.parse().with_context(|| format!("--port {value}"))?,
            _ => bail!("unknown flag {flag}; {USAGE}"),
        }
    }
    if home.the_group()?.is_none() {
        let name = name.context("this computer keeps no group yet: name one with --name")?;
        home.create_group(&name, svrngs::now())?;
    }
    let host = Host::start(&home, port).await?;
    println!("svrngs is listening for the chat on {}", host.addr());
    println!("the chat's token is in {}", home.token_file().display());
    let mut chat = match home.chat()? {
        Some(chat) => Some(start_bridge(&home, &chat, host.addr())?),
        None => {
            info!("serve:no_chat");
            None
        }
    };
    tokio::select! {
        r = tokio::signal::ctrl_c() => {
            r?;
            info!("serve:interrupted");
        }
        () = terminated() => info!("serve:terminated"),
        status = exited(&mut chat) => {
            warn!(?status, "serve:bridge_exited");
            host.stop().await?;
            bail!("the bridge stopped ({status:?}); see the lines above");
        }
    }
    host.stop().await
}

/// Run `bridge/main.mjs` for `chat`, against the host at `addr`.
fn start_bridge(home: &Home, chat: &Chat, addr: SocketAddr) -> anyhow::Result<Child> {
    let token = home
        .discord_token()?
        .context("a chat is set up but no Discord token is kept: run svrngs start again")?;
    let main = svrngs::bridge_dir().join("main.mjs");
    let child = Command::new("node")
        .arg(&main)
        .env("SVRNGS_HOST", format!("http://{addr}"))
        .env("SVRNGS_TOKEN_FILE", home.token_file())
        .env("SVRNGS_DISCORD_TOKEN", token)
        .env("SVRNGS_CHANNEL", &chat.channel)
        .env("SVRNGS_OWNER", &chat.owner)
        .kill_on_drop(true)
        .spawn()
        .with_context(|| format!("node {}: is Node.js 20 or later on PATH?", main.display()))?;
    info!(channel = %chat.channel, pid = ?child.id(), "serve:bridge_started");
    Ok(child)
}

/// The bridge's exit, or never when there is no bridge.
async fn exited(chat: &mut Option<Child>) -> std::io::Result<std::process::ExitStatus> {
    match chat {
        Some(child) => child.wait().await,
        None => std::future::pending().await,
    }
}

/// SIGTERM, which a service manager sends to stop `serve`.
async fn terminated() {
    #[cfg(unix)]
    if let Ok(mut s) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        s.recv().await;
        return;
    }
    std::future::pending::<()>().await
}
