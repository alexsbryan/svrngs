//! The host's load, and the bar past which a seek that missed is not
//! judged: one home for the two binaries that time seeks, the conformance
//! canary (crates/svrngs-node/tests/conformance.rs), the browser tests
//! (bash-film-load; AGENTS.md 8) and the films scene
//! (crates/svrngs/tests/scenes/films.rs), each including this file by path.

/// The load per core past which a seek that missed is not judged (chk-1;
/// operator, 2026-10-10, "Could-not-judge above a load bar"): sup-4's
/// check, sy-1's three on the branch `folders` and REVIEW-audit-sup-auto-2's
/// each failed only `on_a_phone_with_the_wildcard::chromium`'s seeks, at
/// load averages of about 40-130 on this host's 12 cores, while the code
/// they ran changed nothing near it and passed alone or at a quieter moment.
pub const BUSY: f64 = 3.0;

/// The clock and this host's load, `uptime`'s line after the UTC time:
/// every run that times seeks records it at its start and end (the u5
/// queue's rule; u5-4's one miss came at a load of 33.6 on 12 cores).
pub async fn load() -> String {
    let out = tokio::process::Command::new("sh")
        .args(["-c", "echo $(date -u +%FT%TZ) $(uptime)"])
        .output()
        .await
        .expect("sh runs date and uptime");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// This host's cores, which `BUSY` is counted against.
pub fn cores() -> usize {
    std::thread::available_parallelism()
        .expect("this host's cores")
        .get()
}

/// `could-not-judge` with the load, when uptime's first average in `load`
/// (a `load()` line) is past `BUSY` times `cores`; else `None`. A load that
/// does not read is never past the bar: the miss fails, as before chk-1.
pub fn past_busy(load: &str, cores: usize) -> Option<String> {
    // `load averages: 1.5 1.2 1.0` on macOS, `load average: 1.5, 1.2, 1.0` on Linux.
    let first = load
        .split_once("average: ")
        .or_else(|| load.split_once("averages: "))
        .and_then(|(_, a)| a.split([' ', ',']).next()?.parse::<f64>().ok())?;
    (first > BUSY * cores as f64)
        .then(|| format!("could-not-judge, load {first} on {cores} cores, past {BUSY} a core"))
}

/// Demo A's seek bar on the film's seeks, each playing again within five
/// seconds, judged as the conformance canary judges it (`past_busy`): `Ok(None)`
/// when each did; when one missed, on a host whose load at the run's start
/// was past `load::BUSY` a core, `Ok(Some(could-not-judge …))`, never a
/// pass; and on a host under it, `Err` naming the misses, which fails.
pub fn seeks_judged(
    seeks: &[serde_json::Value],
    load: &str,
    cores: usize,
) -> Result<Option<String>, String> {
    let missed: Vec<String> = seeks
        .iter()
        .filter(|s| s["ms"].as_u64().is_none_or(|ms| ms > 5_000))
        .map(serde_json::Value::to_string)
        .collect();
    if missed.is_empty() {
        return Ok(None);
    }
    past_busy(load, cores)
        .map(Some)
        .ok_or_else(|| missed.join(", "))
}
