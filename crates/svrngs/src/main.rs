//! `svrngs serve [--home DIR] [--name NAME] [--port N]`: keep this computer's
//! group, starting one named NAME if there is none, and listen for the chat.

use anyhow::{Context, bail};
use svrngs::{Home, Host, bridge};
use tracing::info;
use tracing_subscriber::EnvFilter;

const USAGE: &str = "usage: svrngs serve [--home DIR] [--name NAME] [--port N]";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.split_first() {
        Some((cmd, rest)) if cmd == "serve" => serve(rest).await,
        _ => bail!(USAGE),
    }
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
    tokio::signal::ctrl_c().await?;
    info!("serve:interrupted");
    host.stop().await
}
