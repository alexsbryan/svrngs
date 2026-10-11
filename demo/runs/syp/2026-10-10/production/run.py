#!/usr/bin/env python3
"""syp-4-production's reruns (ralph/next/syp/STATE.md): sy-1's M2, S2 and M1
by demo/runs/sy/2026-10-09/run.py, its setup and its functions, each with
the change syp-4 asks of it, the data in run-<part>/ beside this file.
Changes to sy's, said: its readiness probe asks GET /v1/groups, as syp-1's
rerun does (../no-stream/run.py), the endpoint it named being gone; `share
--folder` names its group, `--group Shared`, since syp-4 (a); and each part
measures what its row asks:
  m2  (b) each member's store, in bytes, for the first version and the edit's;
  s2  (d) S2 with Ben's node stopped and started before Cy's start;
  m1  (f) M1, 1,000 files reaching a member who joins later.
No plant here: each is a test's red run (fence 3). W is the scratch dir
(else /tmp/syp4-run). scripts/check.sh runs none of this."""
import importlib.util, json, os, signal, subprocess, sys, time
from pathlib import Path
import blake3

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("sy", HERE.parents[2] / "sy/2026-10-09/run.py")
sy = importlib.util.module_from_spec(spec); spec.loader.exec_module(sy)
PART = sys.argv[1] if len(sys.argv) > 1 else ""
sy.OUT, sy.W = HERE / f"run-{PART}", Path(os.environ.get("W", "/tmp/syp4-run"))
asked = sy.api
sy.api = lambda who, path, body=None: asked(who, "/v1/groups" if path == "/v1/folders" else path, body)


def sync(who, plant=""):
    """who's node up and syncing their Shared in the group Shared (syp-4 a)."""
    if who not in sy.nodes: sy.up(who)
    run = subprocess.run([sy.B, "share", "--folder", str(sy.disk(who)), "--group", "Shared"], capture_output=True,
                         text=True, env={**os.environ, "SVRNGS_DIR": str(sy.data(who))})
    sy.say("synced", who=who, answer=run.stdout.strip() or run.stderr.strip())


sy.sync = sync


def host():
    out = lambda *a: subprocess.run(a, capture_output=True, text=True).stdout.strip()
    return {"uptime": out("uptime"), "cores": out("sysctl", "-n", "hw.ncpu"), "swap": out("sysctl", "vm.swapusage"),
            "df": out("df", "-g", "/").splitlines()[-1], "head": out("git", "-C", str(sy.REPO), "rev-parse", "--short", "HEAD")}


def setup():
    """sy's main to its group (run.py:171-185), the link kept for M1."""
    subprocess.run(["cargo", "build", "--offline", "-q", "-p", "svrngs"], cwd=sy.REPO, check=True)
    subprocess.run(["rm", "-rf", str(sy.W), str(sy.OUT)]); sy.OUT.mkdir(parents=True)
    for w in ("ana", "ben", "cy", "eve"): sy.data(w).mkdir(parents=True); sy.disk(w).mkdir(parents=True)
    sy.say("load", **host())
    sy.up("ana", "new", "Shared", "--as", "Ana")
    sy.wait(lambda: "svrngs://join/" in (sy.data("ana") / "out").read_text(), 60)
    link = next(w for w in (sy.data("ana") / "out").read_text().split() if w.startswith("svrngs://join/"))
    for w in ("ben", "cy"):
        sy.up(w, "join", link, "--as", w.title())
        if sy.wait(lambda: "You're in" in (sy.data(w) / "out").read_text(), 60) is None: raise SystemExit(f"{w} did not join")
    sy.KEYS, sy.GROUP = sy.keys(); sy.say("keys", keys=sy.KEYS, group=sy.GROUP)
    return link


def stored(who, digest):
    """The bytes who's store holds of digest on disk: its data file, past 16 KiB never inlined."""
    p = sy.data(who) / "blobs" / "data" / f"{digest}.data"
    return p.stat().st_size if p.exists() else 0


def m2():
    """sy's M2 (run.py:216-224), then each store's bytes of each version, given 30 s to drop the first."""
    sy.phase("M2"); [sync(w) for w in ("ana", "ben", "cy")]
    big = bytearray(os.urandom(100 << 20))
    v1 = sy.write("ana", "big.bin", bytes(big)); t1 = sy.wait(sy.holds(["ben", "cy"], "big.bin", v1), 600)
    big[50 << 20:51 << 20] = os.urandom(1 << 20); v2 = sy.write("ana", "big.bin", bytes(big))
    t2 = sy.wait(sy.holds(["ben", "cy"], "big.bin", v2), 600)
    whos = ("ana", "ben", "cy")
    t_drop = sy.wait(lambda: all(stored(w, v1) == 0 for w in whos), 30)
    stores = {w: {"first": stored(w, v1), "edit": stored(w, v2)} for w in whos}
    sy.summary["M2"] = {"first_s": t1, "edit_s": t2, "dropped_s": t_drop, "stores": stores, "v1": v1, "v2": v2}
    sy.say("measured", **sy.summary["M2"])
    passed = t2 is not None and all(s["first"] == 0 and s["edit"] == 100 << 20 for s in stores.values())
    sy.bar("M2-store", {"stores": stores, "dropped_s": t_drop}, passed)


def s2():
    """sy's S2 (run.py:102-111), Ben's node stopped by SIGTERM and started again before Cy's start."""
    sy.phase("S2"); [sy.down(w) for w in "ana ben cy".split()]
    sync("ana"); sync("ben"); sync("cy"); time.sleep(3); sy.down("cy")
    a = sy.write("ana", "s2.bin", os.urandom(1 << 16))
    sy.wait(sy.holds(["ben"], "s2.bin", a), 30); sy.down("ana", signal.SIGKILL)
    sy.down("ben"); sync("ben")
    t0 = time.time(); sync("cy")
    sy.wait(sy.holds(["cy"], "s2.bin", a), 10 - (time.time() - t0))
    t = round(time.time() - t0, 2) if sy.holds(["cy"], "s2.bin", a)() else None
    by = [w for w, k in sy.KEYS.items() if f"fetched, verified hash={a} holder={k}" in (sy.OUT / "cy.log").read_text()]
    sy.bar("S2-restarted", {"cy_s": t, "hash": a, "from": by}, t is not None and by == ["ben"])


def main():
    link = setup()
    {"m2": m2, "s2": s2, "m1": lambda: sy.m1(link)}[PART]()
    sy.say("load", **host())
    (sy.OUT / "summary.json").write_text(json.dumps(sy.summary, indent=1))


if __name__ == "__main__":
    try: main()
    finally:
        [sy.down(w) for w in list(sy.nodes)]
        subprocess.run(["rm", "-rf", str(sy.W)])
