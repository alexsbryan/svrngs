#!/usr/bin/env python3
"""syp-1's rerun of sy-1's S1-S4 (ralph/next/syp/STATE.md, syp-1-no-stream),
each with its plant first, by demo/runs/sy/2026-10-09/run.py as it ran:
its setup and its s1-s4, here without M1 and M2, the data in run/ beside
this file, not in the sy run's own. One change, said: run.py's readiness
probe asked GET /v1/folders, which syp-1 dropped with the endpoint it named,
so it asks GET /v1/groups, the record API's, in its place. W is the scratch
dir (else /tmp/syp1-run)."""
import importlib.util, json, os, subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("sy", HERE.parents[2] / "sy/2026-10-09/run.py")
sy = importlib.util.module_from_spec(spec); spec.loader.exec_module(sy)
sy.OUT, sy.W = HERE / "run", Path(os.environ.get("W", "/tmp/syp1-run"))
asked = sy.api
sy.api = lambda who, path, body=None: asked(who, "/v1/groups" if path == "/v1/folders" else path, body)


def main():
    """sy's main to S4, as it ran (run.py:171-190)."""
    subprocess.run(["cargo", "build", "--offline", "-q", "-p", "svrngs"], cwd=sy.REPO, check=True)
    subprocess.run(["rm", "-rf", str(sy.W), str(sy.OUT)]); sy.OUT.mkdir(parents=True)
    for w in ("ana", "ben", "cy"): sy.data(w).mkdir(parents=True); sy.disk(w).mkdir(parents=True)
    sy.say("load", uptime=subprocess.run(["uptime"], capture_output=True, text=True).stdout.strip(),
           head=subprocess.run(["git", "-C", str(sy.REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip())
    planted = bytearray(os.urandom(1 << 20))
    for at in range(0, len(planted), 4096): planted[at:at + len(sy.MARK)] = sy.MARK
    (sy.disk("ana").parent / "beside.bin").write_bytes(planted); (sy.disk("ana") / "link.bin").symlink_to("../beside.bin")
    sy.up("ana", "new", "Shared", "--as", "Ana")
    sy.wait(lambda: "svrngs://join/" in (sy.data("ana") / "out").read_text(), 60)
    link = next(w for w in (sy.data("ana") / "out").read_text().split() if w.startswith("svrngs://join/"))
    for w in ("ben", "cy"):
        sy.up(w, "join", link, "--as", w.title())
        if sy.wait(lambda: "You're in" in (sy.data(w) / "out").read_text(), 60) is None: raise SystemExit(f"{w} did not join")
    sy.KEYS, sy.GROUP = sy.keys(); sy.say("keys", keys=sy.KEYS, group=sy.GROUP)
    sy.s1("-plant", "deaf"); (sy.disk("ana") / "s1-plant.txt").unlink(missing_ok=True); sy.s1("", "")
    sy.s2("-plant", "own-only"); sy.s2("", "")
    sy.s3("-plant", "later-wins"); sy.s3("", "")
    sy.s4("-plant", "open-gate"); sy.s4("", "")
    (sy.OUT / "summary.json").write_text(json.dumps(sy.summary, indent=1))
    [sy.down(w) for w in list(sy.nodes)]


if __name__ == "__main__":
    try: main()
    finally: [sy.down(w) for w in list(sy.nodes)]
