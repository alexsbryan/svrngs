// SPDX-License-Identifier: AGPL-3.0-or-later
//! The one test that meets n0 (docs/CHECKS.md, seam R): the shipped
//! `svrngs`, whose main passes `cli::reach(cli::SHIPPED)` (src/main.rs),
//! is met at n0's relays and lookup on the internet. Every other test's
//! svrngs meets at a relay on 127.0.0.1 (common/as_main.rs), so n0 is
//! judged here alone, in check.sh's `n0` step, and `tests/shipped.rs`
//! pins what `reach(SHIPPED)` is with no socket.
//!
//! n0 is a service the gate does not own, so before any svrngs runs n0 is
//! asked by iroh alone ([`n0_answers`]): each of n0's relays, as iroh
//! lists them, resolved and connected to, and an endpoint of iroh's own
//! with n0's presets given a home relay. When it does not answer, the
//! claim cannot be judged: why is written to
//! `target/could-not-judge/n0.step`, which check.sh says as the step's
//! verdict, never a pass and never a failure. When a svrngs fails and n0
//! is then asked again and does not answer, the same. Otherwise a failure
//! is ours, and fails.
//!
//! A test target of its own, `test = false` (crates/svrngs/Cargo.toml), so
//! a plain `cargo test` never meets n0.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

/// n0's relays as iroh itself lists them, the test's own reading of n0
/// (`iroh-1.0.2/src/defaults.rs`), apart from svrngs's
/// (svrngs-node relay.rs `n0_relays`), each host without its trailing dot.
fn n0_hosts() -> Vec<String> {
    let map = iroh::defaults::prod::default_relay_map();
    let urls: Vec<iroh::RelayUrl> = map.urls();
    let hosts = urls.iter().filter_map(|u| u.host_str().map(String::from));
    hosts.map(|h| h.trim_end_matches('.').to_string()).collect()
}

/// How long each relay of n0's may take to resolve and to take a
/// connection, all asked at once, and iroh's own endpoint to be met at one.
/// With [`SAM`], [`BO`] and n0 asked twice, the test ends in under 200 s
/// whatever n0 does, within check.sh's limit for the step.
const ASKED: Duration = Duration::from_secs(5);
const MET: Duration = Duration::from_secs(20);
/// How long Sam's `svrngs new` may take to print its link (it waits
/// `ONLINE_WAIT`, 10 s, for a home relay first), and Bo's join to end.
const SAM: Duration = Duration::from_secs(45);
const BO: Duration = Duration::from_secs(90);

/// One relay of n0's, `host`, resolved and connected to on 443: what was
/// heard, and whether it took the connection.
async fn asked(host: String) -> (String, bool) {
    let resolved = tokio::time::timeout(ASKED, tokio::net::lookup_host((host.as_str(), 443)));
    let addr = match resolved.await {
        Err(_) => return (format!("{host}: no answer to its name in {ASKED:?}"), false),
        Ok(Err(e)) => return (format!("{host}: its name did not resolve: {e}"), false),
        Ok(Ok(mut addrs)) => match addrs.next() {
            None => return (format!("{host}: resolved to nothing"), false),
            Some(addr) => addr,
        },
    };
    match tokio::time::timeout(ASKED, tokio::net::TcpStream::connect(addr)).await {
        Err(_) => (
            format!("{host} ({addr}): no connection in {ASKED:?}"),
            false,
        ),
        Ok(Err(e)) => (format!("{host} ({addr}): connecting failed: {e}"), false),
        Ok(Ok(_)) => (format!("{host} ({addr}): connected"), true),
    }
}

/// Whether n0 answers, asked by iroh and the host alone, no svrngs code in
/// play: some relay of n0's resolved and connected to on 443, and an
/// endpoint with iroh's n0 presets online at a home relay within [`MET`].
/// `Err` says what did not answer, each relay's resolve or connect.
async fn n0_answers() -> Result<(), String> {
    let mut each = tokio::task::JoinSet::new();
    for host in n0_hosts() {
        each.spawn(asked(host));
    }
    let (heard, took): (Vec<String>, Vec<bool>) = each.join_all().await.into_iter().unzip();
    let heard = heard.join("; ");
    eprintln!("n0: {heard}");
    if !took.contains(&true) {
        return Err(format!("no relay of n0's answered: {heard}"));
    }
    let endpoint = iroh::Endpoint::bind(iroh::endpoint::presets::N0).await;
    let endpoint = endpoint.map_err(|e| format!("iroh's own endpoint did not start: {e}"))?;
    let online = tokio::time::timeout(MET, endpoint.online()).await;
    endpoint.close().await;
    online.map_err(|_| {
        format!("iroh's own endpoint, with n0's presets, had no home relay of n0's in {MET:?} ({heard})")
    })
}

/// The claim not judged, `why`: said on stderr, and written where check.sh
/// says it as the `n0` step's verdict (scripts/check.sh `step`).
fn unjudged(why: &str) {
    eprintln!("could-not-judge: {why}");
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/could-not-judge");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("n0.step"), format!("{why}\n")).unwrap();
}

/// The shipped `svrngs <args>` in `home`, as a person's terminal runs it:
/// nothing of this computer's environment but its `PATH`, every decision
/// of the command's and the relays' traced to `home/svrngs.log`.
fn shipped(home: &Path, user: &str, args: &[&str]) -> Command {
    std::fs::create_dir_all(home).unwrap();
    let log = std::fs::File::create(home.join("svrngs.log")).unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_svrngs"));
    c.args(args)
        .env_clear()
        .env("HOME", home)
        .env("USER", user)
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("RUST_LOG", "info,cli=debug,relay=debug,iroh::_events=debug")
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(log)
        .kill_on_drop(true);
    c
}

/// The link Sam's `svrngs new` prints, as printed and read, within
/// `within`. What it prints after is drained, so it never writes to a
/// closed pipe: on the first run of this test it did, and died of it, and
/// Bo was told Sam's computer "cannot be reached".
type Printed = (String, svrngs_node::link::Link);
async fn link_of(sam: &mut Child, within: Duration) -> Result<Printed, String> {
    let out = sam.stdout.take().unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let mut lines = tokio::io::BufReader::new(out).lines();
        let mut seen = Vec::new();
        let mut tx = Some(tx);
        while let Ok(Some(line)) = lines.next_line().await {
            match svrngs_node::link::Link::parse(&line) {
                Some(link) if tx.is_some() => {
                    let printed = (line.trim().to_string(), link);
                    let _ = tx.take().unwrap().send(Ok(printed));
                }
                _ => seen.push(line),
            }
        }
        if let Some(tx) = tx {
            let _ = tx.send(Err(seen));
        }
    });
    match tokio::time::timeout(within, rx).await {
        Ok(Ok(Ok(link))) => Ok(link),
        Ok(Ok(Err(seen))) => Err(format!("Sam's `svrngs new` ended with no link: {seen:?}")),
        Ok(Err(_)) => Err("Sam's output was not read".into()),
        Err(_) => Err(format!("Sam's `svrngs new` printed no link in {within:?}")),
    }
}

/// Sam makes a group with the shipped svrngs, met at n0 alone (no
/// `--relay`), and Bo's computer, knowing nothing but Sam's link, joins by
/// it: Bo's shipped svrngs reaches Sam's through n0, since nothing else
/// tells it where Sam is (no relay of the group's, no address on this
/// host, no local discovery in svrngs).
async fn met_at_n0(dir: &Path) -> Result<(), String> {
    let n0 = n0_hosts();
    let sam_home = dir.join("sam");
    let mut sam = shipped(&sam_home, "sam", &["new", "house", "--as", "Sam"]);
    let mut sam = sam
        .spawn()
        .map_err(|e| format!("the shipped svrngs runs: {e}"))?;
    let sam_log = sam_home.join("svrngs.log");
    let link = link_of(&mut sam, SAM).await;
    let traced = std::fs::read_to_string(&sam_log).unwrap_or_default();
    let (raw, link) = link.map_err(|why| format!("{why}; its log is {}", sam_log.display()))?;
    // Sam's computer waited for a home relay (cli.rs `wait_online`), and
    // was met at one, as its own trace says.
    let waited = traced.lines().find(|l| l.contains("waited for a relay"));
    if !waited.is_some_and(|l| l.contains("online=true")) {
        return Err(format!(
            "Sam's computer was met at no home relay before its link: {waited:?}; its log is {}",
            sam_log.display()
        ));
    }
    // The link carries Sam's home first (svrngs-core relay.rs `carried`):
    // a relay of n0's, as iroh lists them.
    let carried: Vec<String> = link.token.relays.iter().map(ToString::to_string).collect();
    let home = carried.first().cloned().unwrap_or_default();
    if !n0.iter().any(|h| home == format!("https://{h}/")) {
        return Err(format!(
            "Sam's link carries {carried:?}, not first a relay of n0's {n0:?}"
        ));
    }
    eprintln!("n0: Sam's computer met at {home}");

    let bo_home = dir.join("bo");
    let args = ["join", raw.as_str(), "--as", "Bo", "--no-run"];
    // Spawned, not `output()`, which would take its stderr off its log.
    let joined = shipped(&bo_home, "bo", &args).spawn();
    let joined = joined.map_err(|e| format!("the shipped svrngs runs: {e}"))?;
    let joined = joined.wait_with_output();
    let joined = tokio::time::timeout(BO, joined).await;
    let bo_log = bo_home.join("svrngs.log");
    let joined = joined
        .map_err(|_| {
            format!(
                "Bo's join did not end in {BO:?}; its log is {}",
                bo_log.display()
            )
        })?
        .map_err(|e| format!("the shipped svrngs runs: {e}"))?;
    let said = String::from_utf8_lossy(&joined.stdout);
    if !joined.status.success() || !said.contains("You're in house, as Bo.") {
        return Err(format!(
            "Bo's computer did not join by Sam's link ({}): {said}; logs {} and {}",
            joined.status,
            bo_log.display(),
            sam_log.display()
        ));
    }
    eprintln!("n0: Bo's computer joined Sam's house by its link, through n0: {said}");
    let _ = sam.kill().await;
    Ok(())
}

/// **The shipped svrngs is met at n0**: Sam's computer, with n0's relays
/// and lookup as the shipped binary has them, is met at a home relay of
/// n0's, and Bo's computer, knowing only Sam's link, joins by it. Judged
/// only once n0 answers iroh alone ([`n0_answers`]); a failure with n0
/// gone silent since is could-not-judge, said with both reasons.
/// Planted (named in its commit): `cli::reach`'s n0 arm given a relay
/// that is not n0's fails it here, n0 answering; and with every
/// connection off this host denied (`sandbox-exec`), could-not-judge.
#[tokio::test(flavor = "multi_thread")]
async fn the_shipped_svrngs_is_met_at_n0_by_a_second_computer() {
    if let Err(why) = n0_answers().await {
        return unjudged(&format!("n0 did not answer before any svrngs ran: {why}"));
    }
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/n0");
    let _ = std::fs::remove_dir_all(&dir);
    let dir: PathBuf = dir;
    let Err(why) = met_at_n0(&dir).await else {
        return;
    };
    match n0_answers().await {
        Err(gone) => unjudged(&format!(
            "{why}; and n0, asked again, did not answer: {gone}"
        )),
        Ok(()) => panic!("{why}; n0 answers iroh alone, so this is svrngs's"),
    }
}
