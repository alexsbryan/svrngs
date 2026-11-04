#!/usr/bin/env python3
"""Read back this release's public bytes, policies and hidden-file refusals."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
from urllib.error import HTTPError
from urllib.parse import quote, urlsplit, urlunsplit
from urllib.request import Request, urlopen


def require_equal(expected, actual, label):
    if expected != actual:
        raise RuntimeError(f"{label}: expected {expected!r}, got {actual!r}")


def self_test():
    for label, expected, bad in [
        ("bytes", b"release", b"corrupt"),
        ("digest", hashlib.sha256(b"release").hexdigest(), hashlib.sha256(b"corrupt").hexdigest()),
        ("policy", "frame-ancestors 'none'", "frame-ancestors *"),
        ("hidden file", 404, 200),
    ]:
        try:
            require_equal(expected, bad, label)
        except RuntimeError:
            pass
        else:
            raise RuntimeError(f"instrument accepted the planted {label}")
        require_equal(expected, expected, label)
    print("Instrument plants rejected: corrupt bytes, wrong digest, widened policy, exposed hidden file.")


def fetch(url):
    with urlopen(Request(url, headers={"Accept-Encoding": "identity"}), timeout=120) as response:
        return response.read(), response.headers


if sys.argv[1:] == ["--self-test"]:
    self_test()
    raise SystemExit(0)

site, record = (Path(argument).resolve() for argument in sys.argv[1:])
publication = json.loads((record / "publish.json").read_text())
require_equal("passed", publication["state"], "publication")
require_equal(Path(publication["site"]).resolve(), site, "published site")
base = "https://group.svrnme.sh/"
result = {"source": publication["source"], "checked": datetime.now(timezone.utc).isoformat(), "base": base, "state": "running"}
try:
    self_test()
    expected_sums = (site / "get/SHA256SUMS").read_bytes()
    live_sums, _ = fetch(base + "get/SHA256SUMS")
    require_equal(expected_sums, live_sums, "live SHA256SUMS")
    (record / "SHA256SUMS").write_bytes(expected_sums)
    (record / "live-sums.txt").write_bytes(live_sums)
    sums = {name: digest for digest, name in (line.split() for line in expected_sums.decode().splitlines())}
    expected_archives = {
        f"{program}-{platform}.tar.gz"
        for program in ("svrngs", "give")
        for platform in ("darwin-aarch64", "darwin-x86_64", "linux-x86_64", "linux-aarch64")
    }
    require_equal(expected_archives, set(sums), "eight supported downloads")

    def checked(path):
        relative = path.relative_to(site).as_posix()
        local_digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if relative.startswith("get/") and path.name in sums:
            require_equal(sums[path.name], local_digest, relative + " local manifest")
        data, _ = fetch(base + quote(relative, safe="/"))
        digest = hashlib.sha256(data).hexdigest()
        require_equal(local_digest, digest, relative + " public bytes")
        return {"path": relative, "bytes": len(data), "sha256": digest}

    public = [
        path for path in sorted(site.rglob("*"))
        if path.is_file() and path.name != "vercel.json"
        and not any(part.startswith(".") for part in path.relative_to(site).parts)
    ]
    with ThreadPoolExecutor(max_workers=4) as pool:
        result["files"] = list(pool.map(checked, public))
    result["archives"] = sums

    config = json.loads((site / "vercel.json").read_text())
    def policy(source):
        rule = next(rule for rule in config["headers"] if rule["source"] == source)
        return next(header["value"] for header in rule["headers"] if header["key"].lower() == "content-security-policy")

    result["policies"] = {}
    for path, source in [("", "/(.*)"), ("sign.html", "/sign.html")]:
        _, headers = fetch(base + path)
        actual = headers.get("Content-Security-Policy")
        require_equal(policy(source), actual, path + " frame policy")
        result["policies"][path or "/"] = actual
    parts = urlsplit(base)
    wildcard = urlunsplit((parts.scheme, "release-" + publication["source"][:8] + "." + parts.netloc, "/", "", ""))
    data, headers = fetch(wildcard)
    require_equal((site / "index.html").read_bytes(), data, "wildcard page")
    require_equal(policy("/(.*)"), headers.get("Content-Security-Policy"), "wildcard frame policy")
    result["wildcard"] = wildcard

    result["hidden"] = {}
    for path in (".env.local", ".vercel/project.json", "vercel.json"):
        try:
            fetch(base + path)
            status = 200
        except HTTPError as error:
            status = error.code
        require_equal(404, status, path)
        result["hidden"][path] = status

    result["native_smoke"] = {}
    with tempfile.TemporaryDirectory(prefix="native-smoke-", dir=record) as temporary:
        home = Path(temporary)
        for program, arguments, expected_exit, words in [
            ("svrngs", ["--help"], 0, "Give it again for each relay"),
            ("give", [], 2, "usage: give check"),
        ]:
            name = f"{program}-darwin-aarch64.tar.gz"
            data, _ = fetch(base + "get/" + name)
            require_equal(sums[name], hashlib.sha256(data).hexdigest(), name + " smoke digest")
            with tarfile.open(fileobj=io.BytesIO(data)) as archive:
                require_equal([program], archive.getnames(), name + " contents")
                executable = home / program
                executable.write_bytes(archive.extractfile(program).read())
            executable.chmod(0o700)
            smoke = subprocess.run(
                [str(executable), *arguments], capture_output=True, text=True, timeout=30,
                env={**os.environ, "HOME": str(home), "SVRNGS_DIR": str(home / "data")},
            )
            require_equal(expected_exit, smoke.returncode, program + " startup exit")
            require_equal(True, words in smoke.stdout + smoke.stderr, program + " startup words")
            result["native_smoke"][program] = {"exit": smoke.returncode, "contains": words}
    result["state"] = "passed"
except Exception as error:
    result.update(state="failed", error=str(error))
finally:
    (record / "verification.json").write_text(json.dumps(result, indent=2) + "\n")
print(f"Release verification: {result['state']}; {len(result.get('files', []))} public files checked.")
if result["state"] != "passed":
    raise SystemExit(result["error"])
