// SPDX-License-Identifier: AGPL-3.0-or-later
//! `svrngs` as a person runs it, but meeting where the test says: the test
//! binary run again as [`main`], which is src/main.rs's `main` but for the
//! build's relays, one on 127.0.0.1 ([`build_relay`]) in place of
//! `reach(SHIPPED)`'s (docs/CHECKS.md, harness 8). The shipped binary
//! meets svrngs's relay on the internet, n0's behind it, so a test that
//! spawned it leaned on a service off this host; `tests/shipped.rs` pins
//! what `reach(SHIPPED)` is. Each binary that includes this file names [`main`] in an
//! ignored test called [`NAME`]. (Allowed dead: each binary uses a part.)

#![allow(dead_code)]

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use svrngs::cli;
use svrngs_node::iroh::RelayConfig;

/// The ignored test that runs [`main`].
pub const NAME: &str = "svrngs_here";
/// Where [`main`] reads the build's relay: set by [`svrngs`] and
/// [`installed`], never by a person.
const BUILD: &str = "SVRNGS_TEST_BUILD_RELAY";
/// What libtest says on stdout before [`main`] runs, in quiet mode: the
/// test's own lines follow it, and nothing after, since [`main`] exits.
pub const HEADER: [&str; 2] = ["", "running 1 test"];

/// The build's relay for this test binary: a plain-HTTP relay on
/// 127.0.0.1, as `svrngs relay --at` serves one, started on first use on
/// a runtime of its own and kept until the binary ends, so every svrngs a
/// test starts meets there and none at n0.
pub fn build_relay() -> &'static str {
    static AT: OnceLock<String> = OnceLock::new();
    AT.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async move {
                let mut config = iroh_relay::server::ServerConfig::default();
                config.relay = Some(iroh_relay::server::RelayConfig::new(([127, 0, 0, 1], 0)));
                let relay = iroh_relay::server::Server::spawn(config)
                    .await
                    .expect("the build's relay starts on 127.0.0.1");
                let at = relay.http_addr().expect("the relay serves HTTP");
                tx.send(format!("http://{at}/")).unwrap();
                std::future::pending::<()>().await;
            });
        });
        rx.recv().expect("the build's relay said where it is")
    })
}

/// `svrngs`, to which a test adds the arguments a person types: this
/// binary, running [`NAME`] alone, meeting at [`build_relay`].
pub fn svrngs() -> Command {
    let mut c = Command::new(std::env::current_exe().unwrap());
    c.args(["--exact", "--ignored", "--nocapture", "--quiet", NAME, "--"])
        .env(BUILD, build_relay());
    c
}

/// A program named `svrngs` in `dir` that is [`svrngs`], for a test whose
/// svrngs is placed and started by a script with an environment of its
/// own (install.sh, a stub launchd): the relay and this binary written in.
pub fn installed(dir: &Path) -> PathBuf {
    installed_at(dir, build_relay())
}

/// [`installed`], meeting at `relay` in place of [`build_relay`]: the
/// scenes' is the browser harness's relay, behind its TLS front, so a
/// phone's page can reach a computer's home relay there (scenes/main.rs).
pub fn installed_at(dir: &Path, relay: &str) -> PathBuf {
    installed_from(dir, &std::env::current_exe().unwrap(), relay)
}

/// [`installed_at`], of `exe` in place of this binary: a test binary of
/// another build that names its own [`main`] in an ignored [`NAME`], as
/// v1.13.3's do, whose copy of this file is this one but for this function
/// (scenes/mixed.rs, the last release).
pub fn installed_from(dir: &Path, exe: &Path, relay: &str) -> PathBuf {
    let path = dir.join("svrngs");
    let script = format!(
        "#!/bin/sh\n{BUILD}='{relay}' exec '{}' --exact --ignored --nocapture --quiet {NAME} -- \"$@\"\n",
        exe.display()
    );
    // Written beside it, then put in place by `install`, so this process
    // never holds the program open for writing: a child another test thread
    // forks meanwhile would inherit that handle until it execs, and Linux
    // refuses to run a file open for writing ("Text file busy").
    let source = dir.join("svrngs.in");
    std::fs::write(&source, script).unwrap();
    let put = std::process::Command::new("install")
        .args(["-m", "0755"])
        .arg(&source)
        .arg(&path)
        .status()
        .unwrap();
    assert!(put.success(), "install {}: {put}", path.display());
    path
}

/// libtest's [`HEADER`] read off a [`svrngs`]'s stdout, so what is read
/// next is svrngs's own; anything else there is a failure, never skipped.
pub fn past_header(out: &mut impl BufRead) {
    for expected in HEADER {
        let mut line = String::new();
        out.read_line(&mut line).unwrap();
        assert_eq!(line.trim_end_matches('\n'), expected, "libtest's header");
    }
}

/// src/main.rs's `main`, line for line but for the build's relays, the
/// arguments after `--`, and an exit in place of a return, so libtest
/// says nothing after svrngs; and svrngs, where it starts svrngs itself
/// (cli.rs `restart`), starts this binary as [`installed`] runs it.
pub fn main() -> ! {
    let before = std::env::args().skip(1).take_while(|a| a != "--");
    let _ = cli::STARTS_AS.set(before.chain(["--".into()]).collect());
    let typed = std::env::args().skip_while(|a| a != "--").skip(1);
    let command = match cli::parse(std::iter::once("svrngs".to_string()).chain(typed)) {
        Ok(command) => command,
        Err(why) => {
            eprintln!("svrngs: {why}");
            eprintln!();
            eprintln!("{}", cli::USAGE);
            std::io::stdout().flush().unwrap();
            std::process::exit(1);
        }
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new(match &command {
                    cli::Verb::Relay(_) => svrngs::relay::FILTER,
                    _ => "error",
                })
            }),
        )
        .with_writer(std::io::stderr)
        .init();
    let code = run(command);
    std::io::stdout().flush().unwrap();
    std::process::exit(code.into());
}

fn run(command: cli::Verb) -> u8 {
    let root = match cli::data_dir() {
        Ok(r) => r,
        Err(e) => return refuse(e),
    };
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("svrngs: cannot start on this computer: {e}");
            return 3;
        }
    };
    let relay = RelayConfig {
        relay_urls: vec![std::env::var(BUILD).expect("the build's relay is named")],
        n0_services: false,
        loopback_only: false,
    };
    match runtime.block_on(cli::run(command, root, &relay)) {
        Ok(()) => 0,
        Err(e) => refuse(e),
    }
}

fn refuse(e: cli::Refusal) -> u8 {
    eprintln!("svrngs: {e}");
    e.exit_code()
}
