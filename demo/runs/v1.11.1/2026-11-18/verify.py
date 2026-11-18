#!/usr/bin/env python3
"""Read back this release's public bytes, page version, network-first page, install line, policies, lend.js's policy and hidden-file refusals."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import io
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
from urllib.error import HTTPError
from urllib.parse import quote, urlsplit, urlunsplit
from urllib.request import Request, urlopen


# What group.svrnme.sh served before this run (V1.10.1's page, read back at
# 2026-11-18T16:06Z, page-before.txt), so the new page must be another version.
PREVIOUS_VERSION = "095d48338161"


def require_new(version):
    if version == PREVIOUS_VERSION:
        raise RuntimeError(f"page version: still {version!r}, the previous release's")


def require_equal(expected, actual, label):
    if expected != actual:
        raise RuntimeError(f"{label}: expected {expected!r}, got {actual!r}")


def require_served_policy(expected, served, label):
    """The one policy an answer is served with is `expected`: none, two, or
    another one is refused. A browser enforces every policy it is sent, so a
    second would narrow it, never widen it; but the host sends one where two
    of vercel.json's rules match (page-before.txt, sign.html), so two is not
    what this release set either."""
    require_equal([expected], served, label)


# lend.js's policy as web/vercel.json sets it, for the self-test's plants.
LEND_POLICY = "default-src 'none'; connect-src 'self' wss: https:; script-src 'self' 'wasm-unsafe-eval'"


def network_first(worker):
    """The worker's `answered` sends a navigation to the network before its
    cache (web/sw.js at 0002248d), so a newer build's invitation gets the
    newer page."""
    match = re.search(r"^async function answered\(request\) \{\n(.*?)^\}", worker, re.MULTILINE | re.DOTALL)
    if not match:
        raise RuntimeError("sw.js: no `answered` function")
    body = match.group(1)
    navigate = body.find("if (request.mode === 'navigate') {")
    network = body.find("await fetch(request)")
    cache = body.find("caches.match(request)")
    if not (0 <= navigate < network < cache):
        raise RuntimeError("sw.js `answered`: a navigation is not sent to the network before the cache")
    return True


CACHE_FIRST = """async function answered(request) {
  const hit = await caches.match(request);
  return hit ?? fetch(request);
}
"""


def self_test():
    for label, expected, bad in [
        ("bytes", b"release", b"corrupt"),
        ("digest", hashlib.sha256(b"release").hexdigest(), hashlib.sha256(b"corrupt").hexdigest()),
        ("policy", "frame-ancestors 'none'", "frame-ancestors *"),
        ("hidden file", 404, 200),
        ("page version", "0123456789ab", PREVIOUS_VERSION),
    ]:
        try:
            require_equal(expected, bad, label)
        except RuntimeError:
            pass
        else:
            raise RuntimeError(f"instrument accepted the planted {label}")
        require_equal(expected, expected, label)
    try:
        require_new(PREVIOUS_VERSION)
    except RuntimeError:
        pass
    else:
        raise RuntimeError("instrument accepted the previous release's page version as new")
    for label, bad in [
        ("V1.10's lend.js, the site's frame policy alone (page-before.txt)", ["frame-ancestors 'none'"]),
        ("a widened lend.js policy", [LEND_POLICY + " 'unsafe-eval'"]),
        ("a lend.js served with no policy", []),
        ("a lend.js served with two policies", ["frame-ancestors 'none'", LEND_POLICY]),
    ]:
        try:
            require_served_policy(LEND_POLICY, bad, "lend.js policy")
        except RuntimeError:
            pass
        else:
            raise RuntimeError(f"instrument accepted {label}")
    require_served_policy(LEND_POLICY, [LEND_POLICY], "lend.js policy")
    try:
        network_first(CACHE_FIRST)
    except RuntimeError:
        pass
    else:
        raise RuntimeError("instrument accepted a cache-first `answered`")
    print("Instrument plants rejected: corrupt bytes, wrong digest, widened policy, exposed hidden file, stale page version, "
          "V1.10's lend.js policy, a widened lend.js policy, none, two, cache-first page.")


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

    # The page version build-web.sh stamped, as publish.log says it, is new
    # and is what the live service worker names.
    built = re.findall(r"^built: .* \(version ([0-9a-f]{12})\)$", (record / "publish.log").read_text(), re.MULTILINE)
    require_equal(1, len(built), "one page version in publish.log")
    version = built[0]
    require_new(version)
    live_worker, _ = fetch(base + "sw.js")
    require_equal({version}, set(re.findall(r"v=([0-9a-f]{12})", live_worker.decode())), "live page version")
    result["page_version"] = {"previous": PREVIOUS_VERSION, "live": version}

    # The live worker is the built one, and it opens the page from the
    # network first, as since V1.8.1 (0002248d).
    require_equal((site / "sw.js").read_bytes(), live_worker, "live sw.js")
    result["network_first"] = network_first(live_worker.decode())

    # install.sh fetches SHA256SUMS and each tarball from the get/ of the
    # page it came from (web/install.sh), so its live copy, read from here,
    # fetches the sums just held equal to the built ones.
    install, _ = fetch(base + "install.sh")
    require_equal((site / "install.sh").read_bytes(), install, "live install.sh")
    for line in ('curl -fsSL "${from}get/SHA256SUMS"', 'curl -fsSL "${from}get/$tarball"'):
        require_equal(True, line in install.decode(), "install.sh fetches " + line)
    result["install"] = {"from": base, "sums": base + "get/SHA256SUMS", "equal_to_built": True}

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

    # lend.js, a phone's worker at each lend's own origin, is served with the
    # policy vercel.json sets for it (5078e98e), at the page's host and at a
    # name under it, where a lend opens; and it is the built lend.js.
    result["lend_policy"] = {}
    for at in (base, wildcard):
        data, headers = fetch(at + "lend.js")
        require_equal((site / "lend.js").read_bytes(), data, at + "lend.js bytes")
        served = headers.get_all("Content-Security-Policy") or []
        require_served_policy(policy("/lend.js"), served, at + "lend.js policy")
        result["lend_policy"][at + "lend.js"] = served

    result["hidden"] = {}
    for path in (".env.local", ".vercel/project.json", ".gitignore", "vercel.json"):
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
