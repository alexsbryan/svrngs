#!/usr/bin/env python3
"""syp-5's rerun of sy-1's run on production (ralph/next/syp/STATE.md,
syp-5-rerun-and-merge (a); README.md §1 here): demo/runs/sy/2026-10-09/run.py
loaded as a module, its setup, s1-s4, m1 and m2 as they ran, the data in
run/ beside this file. Changes to sy's, said:
  - `share --folder` names its group, `--group Shared` (syp-4 a), as
    ../production/run.py asks;
  - the readiness probe asks GET /v1/groups, the endpoint sy's named gone
    (syp-1), as ../no-stream/run.py asks;
  - no plant phase: a plant is a test's red run (fence 3), run apart by
    plants.py, its output in plants/;
  - each phase's load, cores, swap and free disk at its start and end, by
    ../production/run.py's `host`, and whether perf-1's load rule judges it;
  - M2 also measures each member's store of each version, in bytes.
W is the scratch dir (else /tmp/syp5-run), removed after. scripts/check.sh
runs none of this."""
import importlib.util, json, os, subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)
    return mod


sy = load("sy", HERE.parents[2] / "sy/2026-10-09/run.py")
prod = load("prod", HERE.parent / "production/run.py")  # its host, judged and stored; it sets sy's sync and api
sy.OUT, sy.W = HERE / "run", Path(os.environ.get("W", "/tmp/syp5-run"))
prod.sy = sy
sy.api = lambda who, path, body=None, asked=sy.api: asked(who, "/v1/groups" if path == "/v1/folders" else path, body)
sy.sync = prod.sync
judgements = {}


def phased(name, run):
    """Run one of sy's phases, the host before and after, judged by perf-1's load rule."""
    before = prod.host(); sy.say("load", at="start", of=name, **before)
    run()
    after = prod.host(); sy.say("load", at="end", of=name, **after)
    judgements[name] = prod.judged(before, after) or "judged"


def m2():
    sy.m2()
    v2 = sy.summary["M2"]
    first = [l for l in sy.OUT.joinpath("events.jsonl").read_text().splitlines() if '"path": "big.bin"' in l]
    v1, v2h = json.loads(first[0])["blake3"], json.loads(first[1])["blake3"]
    t_drop = sy.wait(lambda: all(prod.stored(w, v1) == 0 for w in ("ana", "ben", "cy")), 30)
    v2["stores"] = {w: {"first": prod.stored(w, v1), "edit": prod.stored(w, v2h)} for w in ("ana", "ben", "cy")}
    v2["dropped_s"] = t_drop
    sy.say("measured", stores=v2["stores"], dropped_s=t_drop)


def main():
    subprocess.run(["cargo", "build", "--offline", "-q", "-p", "svrngs"], cwd=sy.REPO, check=True)
    subprocess.run(["rm", "-rf", str(sy.W), str(sy.OUT)]); sy.OUT.mkdir(parents=True)
    for w in ("ana", "ben", "cy", "eve"): sy.data(w).mkdir(parents=True); sy.disk(w).mkdir(parents=True)
    sy.say("load", at="start", of="run", **prod.host())
    planted = bytearray(os.urandom(1 << 20))
    for at in range(0, len(planted), 4096): planted[at:at + len(sy.MARK)] = sy.MARK
    (sy.disk("ana").parent / "beside.bin").write_bytes(planted); (sy.disk("ana") / "link.bin").symlink_to("../beside.bin")
    sy.up("ana", "new", "Shared", "--as", "Ana")  # sy's main to its group (run.py:181-187)
    sy.wait(lambda: "svrngs://join/" in (sy.data("ana") / "out").read_text(), 60)
    link = next(w for w in (sy.data("ana") / "out").read_text().split() if w.startswith("svrngs://join/"))
    for w in ("ben", "cy"):
        sy.up(w, "join", link, "--as", w.title())
        if sy.wait(lambda: "You're in" in (sy.data(w) / "out").read_text(), 60) is None: raise SystemExit(f"{w} did not join")
    sy.KEYS, sy.GROUP = sy.keys(); sy.say("keys", keys=sy.KEYS, group=sy.GROUP)
    for name, run in (("S1", lambda: sy.s1("", "")), ("S2", lambda: sy.s2("", "")),
                      ("S3", lambda: sy.s3("", "")), ("S4", lambda: sy.s4("", "")),
                      ("M1", lambda: sy.m1(link)), ("M2", m2)):
        phased(name, run)
    sy.summary["load"] = judgements
    sy.say("load", at="end", of="run", **prod.host())


if __name__ == "__main__":
    try: main()
    finally:
        [sy.down(w) for w in list(sy.nodes)]
        (sy.OUT / "summary.json").write_text(json.dumps(sy.summary, indent=1))
        subprocess.run(["rm", "-rf", str(sy.W)])
