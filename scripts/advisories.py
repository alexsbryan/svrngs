#!/usr/bin/env python3
"""Fresh RustSec checks for both lockfiles; exceptions expire, missing tools never pass."""
import datetime
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def exceptions(config, today):
    for item in config.get("advisories", {}).get("ignore", []):
        reason = item.get("reason", "") if isinstance(item, dict) else ""
        due = re.search(r"review by (\d{4}-\d{2}-\d{2}): .+", reason)
        if not due or datetime.date.fromisoformat(due[1]) <= today:
            raise ValueError(f"advisory exception has no future review date and reason: {item}")


def main():
    try:
        exceptions(tomllib.loads((ROOT / "deny.toml").read_text()), datetime.date.today())
    except (ValueError, OSError) as error:
        print(f"failed     advisories: {error}")
        return 1
    if not shutil.which("cargo") or subprocess.run(["cargo", "deny", "--version"], capture_output=True).returncode:
        print("never-ran  advisories: install cargo-deny with cargo install cargo-deny --locked")
        return 3
    failed = False
    for manifest in ["Cargo.toml", "web/Cargo.toml"]:
        result = subprocess.run(["cargo", "deny", "--manifest-path", manifest, "--config", "deny.toml", "--locked", "check", "advisories"], cwd=ROOT)
        print(f"{'failed' if result.returncode else 'passed'}     advisories: {manifest}", flush=True)
        failed |= result.returncode != 0
    return int(failed)


if __name__ == "__main__":
    sys.exit(main())
