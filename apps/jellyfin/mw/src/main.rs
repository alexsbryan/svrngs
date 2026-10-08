// SPDX-License-Identifier: AGPL-3.0-or-later
//! Jellyfin's middleware (demo/runs/mw/2026-10-08/README.md; DESIGN §9
//! :1133): a program the lender runs on their own computer, in front of
//! their Jellyfin, and lends with `svrngs share <its port>`. Each person the
//! door names is made a read-only Jellyfin user of their own on their first
//! request, and every request is signed in as its person on the client's
//! own device. svrngs neither ships, starts nor supervises it.
//!
//!   jellyfin-mw [--listen 127.0.0.1:<port>] [--jellyfin 127.0.0.1:8096]
//!
//! The lender gives it, once, on standard input, an API key from Jellyfin's
//! Dashboard, API Keys: the only thing it makes accounts with. It prints
//! `listening on port <port>` and serves until stopped.
//!
//! jellyfin-web's page it answers carries the person's stored sign-in
//! (page.rs), and their token is kept out of what Jellyfin answers and put
//! back in what it is asked (swap.rs), so a person opens it from svrngs's
//! list signed in, with nothing typed, and holds no token.
//!
//! JELLYFIN_MW_PLANT names a failure its check plants, never set otherwise:
//! `one-user`, every person signed in as one account; `let-keyless`, a
//! request with no `x-svrngs-key` let through as that account;
//! `no-sign-in`, the page passed on without its stored sign-in;
//! `lender-key`, the page's stored sign-in holding the lender's key for
//! its token; `one-device`, every person one account on one device.

mod forward;
mod page;
mod people;
mod person;
mod swap;
mod viewer;

use std::io::BufRead;
use std::net::SocketAddr;

/// A failure the check plants, to watch it caught.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Plant {
    OneUser,
    LetKeyless,
    NoSignIn,
    LenderKey,
    OneDevice,
}

fn arg(args: &[String], name: &str, default: &str) -> Result<SocketAddr, String> {
    let raw = match args.iter().position(|a| a == name) {
        Some(i) => args.get(i + 1).ok_or(format!("{name} needs an address"))?.as_str(),
        None => default,
    };
    let addr: SocketAddr = raw
        .parse()
        .map_err(|_| format!("{name} `{raw}` is not an address, as 127.0.0.1:8096"))?;
    if !addr.ip().is_loopback() {
        return Err(format!(
            "{name} `{raw}` is not on this computer's loopback: the middleware serves and asks there alone"
        ));
    }
    Ok(addr)
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let listen = arg(&args, "--listen", "127.0.0.1:0")?;
    let jellyfin = arg(&args, "--jellyfin", "127.0.0.1:8096")?;
    let plant = match std::env::var("JELLYFIN_MW_PLANT").ok().as_deref() {
        None | Some("") => None,
        Some("one-user") => Some(Plant::OneUser),
        Some("let-keyless") => Some(Plant::LetKeyless),
        Some("no-sign-in") => Some(Plant::NoSignIn),
        Some("lender-key") => Some(Plant::LenderKey),
        Some("one-device") => Some(Plant::OneDevice),
        Some(other) => {
            return Err(format!(
                "JELLYFIN_MW_PLANT `{other}`: one-user, let-keyless, no-sign-in, lender-key or one-device"
            ))
        }
    };
    let mut key = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut key)
        .map_err(|e| format!("the key could not be read from standard input: {e}"))?;
    let key = key.trim();
    if key.is_empty() || !key.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err("give it, on standard input, an API key from Jellyfin's Dashboard, API Keys".into());
    }
    let elevated = vec![("authorization".to_string(), format!("MediaBrowser Token=\"{key}\""))];
    let page_token = match plant {
        Some(Plant::LenderKey) => key.to_string(),
        _ => swap::STAND_IN.to_string(),
    };
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("no runtime: {e}"))?;
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(listen)
            .await
            .map_err(|e| format!("{listen} could not be listened on: {e}"))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        if let Some(p) = plant {
            tracing::warn!(plant = ?p, "mw: a planted failure is on — a check's run, never a lend");
        }
        let server = page::Server::read(jellyfin).await?;
        println!("listening on port {port}, in front of Jellyfin at {jellyfin}");
        let people = people::People::start(jellyfin, elevated);
        forward::serve(
            listener,
            forward::At {
                jellyfin,
                server,
                people,
                page_token,
                plant,
            },
        )
        .await;
        Ok(())
    })
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    if let Err(why) = run() {
        eprintln!("jellyfin-mw: {why}");
        std::process::exit(2);
    }
}
