// SPDX-License-Identifier: AGPL-3.0-or-later
//! The one test that meets a relay on the internet (docs/CHECKS.md, seam
//! R): the shipped `svrngs`, whose main passes `cli::reach(cli::SHIPPED)`
//! (src/main.rs), is met at svrngs's relay, `relay.svrnme.sh`, by a second
//! computer. Every other test's svrngs meets at a relay on 127.0.0.1
//! (common/as_main.rs), so that relay is judged here alone, in check.sh's
//! `internet` step; `tests/shipped.rs` pins what `reach(SHIPPED)` is, and
//! n0's behind it, with no socket.
//!
//! The gate judges this commit's code, and whether the relay is up is not
//! a property of the code: before any svrngs runs it is asked by iroh alone
//! ([`ours_answers`]), its name resolved and connected to, and an endpoint
//! of iroh's own with it as its one relay given a home there. When it does
//! not answer, the claim cannot be judged: why is written to
//! `target/could-not-judge/internet.step` (tests/common/unjudged.rs in
//! svrngs-node, every test binary's one way), which check.sh says as the
//! step's verdict, never a pass and never a failure. When a svrngs fails
//! and the relay is then asked again and does not answer, the same.
//! Otherwise a failure is ours, and fails. The relay's silence is a failure
//! where it is owed: `scripts/release.sh` refuses to publish a build
//! at svrngs's while it does not answer (DESIGN §7).
//!
//! A test target of its own, `test = false` (crates/svrngs/Cargo.toml), so
//! a plain `cargo test` meets no relay off this host.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

/// svrngs's relay as the operator hosts it, the test's own spelling, apart
/// from `cli::DEFAULT_RELAY`.
const OURS: &str = "relay.svrnme.sh";

/// How long the relay may take to resolve, and then to take a connection,
/// and iroh's own endpoint to be met there. With [`SAM`], [`BO`] and the
/// relay asked twice, the test ends in under 200 s whatever the relay
/// does, within check.sh's limit for the step.
const ASKED: Duration = Duration::from_secs(5);
const MET: Duration = Duration::from_secs(20);
/// How long Sam's `svrngs new` may take to print its link (it waits
/// `ONLINE_WAIT`, 10 s, for a home relay first), and Bo's join to end.
const SAM: Duration = Duration::from_secs(45);
const BO: Duration = Duration::from_secs(90);

/// Whether svrngs's relay answers, asked by iroh and the host alone, no
/// svrngs code in play: its name resolved and connected to on 443, and an
/// endpoint with it as iroh's one relay online there within [`MET`]. `Err`
/// says what did not answer.
async fn ours_answers() -> Result<(), String> {
    let resolved = tokio::time::timeout(ASKED, tokio::net::lookup_host((OURS, 443)));
    let addr = match resolved.await {
        Err(_) => return Err(format!("{OURS}: no answer to its name in {ASKED:?}")),
        Ok(Err(e)) => return Err(format!("{OURS}: its name did not resolve: {e}")),
        Ok(Ok(mut addrs)) => addrs
            .next()
            .ok_or_else(|| format!("{OURS}: resolved to nothing"))?,
    };
    match tokio::time::timeout(ASKED, tokio::net::TcpStream::connect(addr)).await {
        Err(_) => return Err(format!("{OURS} ({addr}): no connection in {ASKED:?}")),
        Ok(Err(e)) => return Err(format!("{OURS} ({addr}): connecting failed: {e}")),
        Ok(Ok(_)) => eprintln!("internet: {OURS} ({addr}): connected"),
    }
    let url: iroh::RelayUrl = format!("https://{OURS}/").parse().unwrap();
    let endpoint = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .relay_mode(iroh::RelayMode::custom([url]))
        .bind()
        .await;
    let endpoint = endpoint.map_err(|e| format!("iroh's own endpoint did not start: {e}"))?;
    let online = tokio::time::timeout(MET, endpoint.online()).await;
    endpoint.close().await;
    online.map_err(|_| format!("iroh's own endpoint had no home at {OURS} in {MET:?}"))
}

// The claim not judged, `why`: said where check.sh says it as the
// `internet` step's verdict, the one way every test binary says it.
#[path = "../../svrngs-node/tests/common/unjudged.rs"]
mod unjudged;
use unjudged::unjudged;

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
        .env(
            "RUST_LOG",
            "info,cli=debug,relay=debug,tiers=debug,iroh::_events=debug",
        )
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

/// Sam makes a group with the shipped svrngs, met at the build's relay
/// alone (no `--relay`), and Bo's computer, knowing nothing but Sam's link,
/// joins by it: Bo's shipped svrngs reaches Sam's through svrngs's relay,
/// since nothing else tells it where Sam is (no relay of the group's, no
/// address lookup, no local discovery in svrngs).
async fn met_at_ours(dir: &Path) -> Result<(), String> {
    let ours = format!("https://{OURS}/");
    let sam_home = dir.join("sam");
    let mut sam = shipped(&sam_home, "sam", &["new", "house", "--as", "Sam"]);
    let mut sam = sam
        .spawn()
        .map_err(|e| format!("the shipped svrngs runs: {e}"))?;
    let sam_log = sam_home.join("svrngs.log");
    let link = link_of(&mut sam, SAM).await;
    let traced = std::fs::read_to_string(&sam_log).unwrap_or_default();
    let (raw, link) = link.map_err(|why| format!("{why}; its log is {}", sam_log.display()))?;
    // Sam's computer waited for a home relay as it made the group (group.rs
    // `Asking::made`, `Host::online`), and was met at one, as the command's
    // trace of the answer says (cli.rs `new_running`).
    let made = "new — made by the svrngs running here, which had reached a relay or not";
    let waited = traced.lines().find(|l| l.contains(made));
    if !waited.is_some_and(|l| l.contains("online=true")) {
        return Err(format!(
            "Sam's computer was met at no home relay before its link: {waited:?}; its log is {}",
            sam_log.display()
        ));
    }
    // The link carries Sam's home first (svrngs-core relay.rs `carried`):
    // svrngs's relay, not n0's emergency behind it.
    let carried: Vec<String> = link.token.relays.iter().map(ToString::to_string).collect();
    if carried.first() != Some(&ours) {
        return Err(format!(
            "Sam's link carries {carried:?}, not first {ours}; its log is {}",
            sam_log.display()
        ));
    }
    eprintln!("internet: Sam's computer met at {ours}");

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
    eprintln!("internet: Bo's computer joined Sam's house by its link, through {ours}: {said}");
    let _ = sam.kill().await;
    Ok(())
}

/// **The shipped svrngs is met at svrngs's relay**: Sam's computer, with
/// the build's relays as the shipped binary has them, is met at a home at
/// `relay.svrnme.sh`, and Bo's computer, knowing only Sam's link, joins by
/// it. Judged only once that relay answers iroh alone ([`ours_answers`]);
/// a failure with the relay gone silent since is could-not-judge, said
/// with both reasons. Planted (named in its commit): `cli::DEFAULT_RELAY`
/// naming another relay fails it here, ours answering; and with no network
/// but loopback (`unshare -rn`), could-not-judge.
#[tokio::test(flavor = "multi_thread")]
async fn the_shipped_svrngs_is_met_at_svrngs_s_relay_by_a_second_computer() {
    if let Err(why) = ours_answers().await {
        return unjudged(&format!(
            "svrngs's relay did not answer before any svrngs ran: {why}"
        ));
    }
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/internet");
    let _ = std::fs::remove_dir_all(&dir);
    let dir: PathBuf = dir;
    let Err(why) = met_at_ours(&dir).await else {
        return;
    };
    match ours_answers().await {
        Err(gone) => unjudged(&format!(
            "{why}; and svrngs's relay, asked again, did not answer: {gone}"
        )),
        Ok(()) => panic!("{why}; svrngs's relay answers iroh alone, so this is svrngs's"),
    }
}
