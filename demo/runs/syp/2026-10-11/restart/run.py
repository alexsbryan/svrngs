#!/usr/bin/env python3
"""syp-6's S3 rerun (ralph/next/syp/STATE.md, syp-6-restart-and-retry (b)):
syp-5's driver, demo/runs/syp/2026-10-10/rerun/run.py, loaded as a module
and run as `run.py s3 <n>` runs it, its setup and S3 alone, the host's load
judged by perf-1's rule; its data here in run-s3-<n>/, beside syp-5's, not
in syp-5's directory. Each node's log at SYP5_LOG, else this file's LOG:
sy's levels with `folder` at debug, so a fetch's dial says the addresses
iroh holds for its holder (31750ee). W is the scratch dir (else
/tmp/syp6-run), removed after. scripts/check.sh runs none of this.

  run.py <n>    one S3 run, into run-s3-<n>/"""
import importlib.util, json, os, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
LOG = "folder=debug,rail:folder=debug,transport=warn,node=debug"
n = sys.argv[1]
os.environ.setdefault("SYP5_LOG", LOG)
os.environ.setdefault("W", "/tmp/syp6-run")
sys.argv = [sys.argv[0], "s3", n]
spec = importlib.util.spec_from_file_location("rerun", HERE.parents[1] / "2026-10-10/rerun/run.py")
rerun = importlib.util.module_from_spec(spec); spec.loader.exec_module(rerun)
sy = rerun.sy
sy.OUT = HERE / f"run-s3-{n}"
try: rerun.main()
finally:
    [sy.down(w) for w in list(sy.nodes)]
    (sy.OUT / "summary.json").write_text(json.dumps(sy.summary, indent=1))
    subprocess.run(["rm", "-rf", str(sy.W)])
