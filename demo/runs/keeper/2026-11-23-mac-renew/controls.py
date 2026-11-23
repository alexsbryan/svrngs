"""Instrument only an archived source tree on beefymac, never the checkout."""
from pathlib import Path
import sys

scratch = Path.home() / 'svrngs-keeper-scratch'
repo = scratch / 'repo'
assert Path.cwd() == repo and not (repo / '.git').exists()
src = repo / 'crates'
mode = sys.argv[1]
assert mode in ('test', 'plant')


def change(path, old, new, count=1):
    path = src / path
    text = path.read_text()
    assert text.count(old) == count, (path, old)
    path.write_text(text.replace(old, new))


# Exercise the installed Share path with a distinct login label, not the
# installed operator agent or its files.
for path in ('svrngs-node/src/job/mac.rs', 'svrngs/src/cli.rs'):
    change(path, 'sh.svrnme.svrngs', 'svrngs-keeper-test-mac-renew-door',
           count=1 if path.endswith('mac.rs') else 2)

bound = 'svrngs-node/src/door/bound.rs'
change(bound, '    let fresh = || {\n        let another', '''    if let Some(signal) = std::env::var_os("SVRNGS_TEST_RACE").map(std::path::PathBuf::from).filter(|p| p.exists()) {
        std::fs::write(signal.with_extension("judged"), []).unwrap();
        for _ in 0..500 {
            if !signal.exists() { break; }
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(!signal.exists(), "withdrawal test did not release the door");
    }
    let fresh = || {
        let another''')
change(bound, '    let renewed = stands.then_some(()).ok_or(Unsent::Held(hold));', '''    if let Some(signal) = std::env::var_os("SVRNGS_TEST_RACE").map(std::path::PathBuf::from).filter(|p| p.with_extension("judged").exists()) {
        std::fs::write(signal.with_extension("decided"), if stands { "carried" } else { "refused" }).unwrap();
    }
    let renewed = stands.then_some(()).ok_or(Unsent::Held(hold));''')

# The kernel must be sampled after, not during, the fixture's local FD handoff.
change('svrngs-node/src/uid.rs', '    let scan = scanned(me, raw, port, conn.map(|c| (c.peer, c.local)))?;', '''    if conn.is_some() && std::env::var_os("SVRNGS_TEST_RACE").is_some() {
        std::thread::sleep(std::time::Duration::from_millis(60));
        if let Some(route) = std::env::var_os("SVRNGS_TEST_HANDOFF").map(std::path::PathBuf::from).filter(|p| p.exists()) {
            let ready = route.with_extension("ready");
            for _ in 0..500 {
                if ready.exists() { break; }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert!(ready.exists(), "fixture did not finish the FD handoff");
        }
    }
    let scan = scanned(me, raw, port, conn.map(|c| (c.peer, c.local)))?;''')

if mode == 'plant':
    change(bound, 'pub(super) fn job_approval', '''fn planted(case: &str) -> bool {
    std::env::var("SVRNGS_TEST_CONTROL").is_ok_and(|found| found == case)
}

pub(super) fn job_approval''')
    change(bound, '.filter(|a| !a.held);', '.filter(|a| !a.held || planted("held"));')
    change(bound, '.filter(|job| job.keep == *keep);', '.filter(|job| job.keep == *keep || planted("changed"));')
    change(bound, '    let stands = carried && matches!(kept, Some(Ok(true)));',
           '    let stands = (carried && matches!(kept, Some(Ok(true)))) || planted("withdrawn");')
    change(bound, 'if !on.into_iter().all', 'if !planted("holder") && !on.into_iter().all')
    change(bound, 'if up.contains(root) =>', 'if up.contains(root) || planted("taker") =>')
    change(bound, '    binding::check(Some(&approved), read, took)',
           '    let took = if planted("taker") { &Took::Datagram } else { took };\n    binding::check(Some(&approved), read, took)')
