"""Replay the ordinary parallel browser step with the relay tiers beside it.

Run inside svrngs-gate from /work. Keep complete output under target/;
the transcript gives a verdict for each name even if another case fails.
"""

import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


BROWSER = ["cargo", "test", "--quiet", "--workspace", "--test", "browser"]
RELAY = ["cargo", "test", "-p", "svrngs-node", "--test", "relay_tiers"]
CLOSE = "webrtc_page::the_channel_closed_mid_film_completes_over_the_relay"
NAV = "navigations::a_navigation_from_another_lend_s_page_is_judged_as_its_fetch"
SEEK = "a_seek_over_the_share_on_a_{}_waits_behind_what_it_lets_be_queued"


def replay():
    os.environ["TOKIO_WORKER_THREADS"] = "4"
    os.environ.pop("RUST_TEST_THREADS", None)
    subprocess.run(["git", "rev-parse", "HEAD"], check=True)
    subprocess.run(["npm", "ci", "--prefix", "web", "--no-audit", "--no-fund"], check=True)
    subprocess.run([*BROWSER, "--no-run"], check=True)
    built = subprocess.check_output([*RELAY, "--no-run", "--message-format=json"], text=True)
    (relay,) = [row["executable"] for line in built.splitlines() if (row := json.loads(line))
                and row.get("executable") and row["target"]["name"] == "relay_tiers"]
    output = Path(tempfile.mkdtemp(prefix="robust-e2e-", dir="target"))
    print(f"{os.cpu_count()} visible CPUs; parallel libtest; logs in {output}", flush=True)
    browser_cases = [f"{CLOSE}::chromium", f"{CLOSE}::webkit", f"{NAV}::webkit"]
    relay_cases = [SEEK.format("pages_dial"), SEEK.format("data_endpoint")]
    streaks = dict.fromkeys([*browser_cases, *relay_cases], 0)
    pretty = ["--format", "pretty", "--show-output"]
    for n in range(1, 11):
        browser_log, relay_log = (output / f"{n:02}-{which}.log" for which in ("browser", "relay"))
        with browser_log.open("w") as b, relay_log.open("w") as r:
            with subprocess.Popen([*BROWSER, "--", "--skip", "playback::", *pretty],
                                  stdout=b, stderr=subprocess.STDOUT) as browser:
                relay_run = subprocess.run([relay, *pretty], stdout=r, stderr=subprocess.STDOUT)
                browser_status = browser.wait()
        for log, cases, status in ((browser_log, browser_cases, browser_status),
                                   (relay_log, relay_cases, relay_run.returncode)):
            text = log.read_text(errors="replace")
            print(f"{log}: exit {status}", flush=True)
            for case in cases:
                passed = re.findall(rf"^test {re.escape(case)} \.\.\. (\w+)$", text, re.M) == ["ok"]
                streaks[case] = streaks[case] + 1 if passed else 0
                print(f"  {case}: {'passed' if passed else 'not passed'} ({streaks[case]} in a row)")
            for line in text.splitlines():
                if (line.startswith(("test result:", "the seek over ", "retrying the WebKit "))
                        or line.endswith(" ... FAILED")):
                    print(f"  {line}")
                if line.startswith("the closed channel's run: "):
                    film = json.loads(line.removeprefix("the closed channel's run: "))["first"]
                    print("  film:", {k: film.get(k) for k in ("closed", "status", "bytes", "sha", "fallbackMs")})
    return 0 if all(streak == 10 for streak in streaks.values()) else 1


if __name__ == "__main__":
    raise SystemExit(replay())
