"""Search what a member's browser received, and both computers' svrngs logs,
for Sam's Jellyfin password and every credential Sam's computer held.

usage: python3 scan.py <scratch dir> <out dir>
Writes <out>/scan.json and, per mode, <out>/<mode>.requests.tsv and a HAR
with bodies dropped and credentials masked, <out>/<mode>.har.
"""
import base64, json, re, sys
from pathlib import Path

root, out = Path(sys.argv[1]), Path(sys.argv[2])
sec = root / "secrets"


def body(c):
    t = c.get("text") or ""
    return base64.b64decode(t).decode("latin-1") if c.get("encoding") == "base64" else t


secrets = {
    "sam_password": (sec / "sam_pw").read_text().strip(),
    "viewer_password": (sec / "viewer_pw").read_text().strip(),
    "sam_admin_token": (sec / "admin_token").read_text().strip(),
    "api_key_revoked": (sec / "api_key").read_text().strip(),
}
# Every token Sam's computer declared as the lend's secret, in order
# (dry.sh `mint` appends each), and every token Jellyfin answered the
# browser's sign-in with; a token in neither list is named as unknown.
for i, t in enumerate((sec / "lend_tokens").read_text().split()):
    secrets[f"lend_token_{i + 1}"] = t
handed = []
for har in sorted((root / "runs").glob("*/full.har")):
    for e in json.loads(har.read_text())["log"]["entries"]:
        handed += re.findall(r'"AccessToken"\s*:\s*"([0-9a-f]{32})"', body(e["response"].get("content", {})))
for i, t in enumerate(dict.fromkeys(handed)):
    secrets.setdefault(f"browser_token_{i + 1}", t)
jlog = "".join(p.read_text(errors="replace") for p in (root / "jf/log").glob("*.log"))
logged_out = re.findall(r'Logging out access token "([0-9a-f]{32})"', jlog)
for i, t in enumerate(t for t in dict.fromkeys(logged_out) if t not in secrets.values()):
    secrets[f"unknown_token_{i + 1}"] = t


def hits(text):
    return {k: text.count(v) for k, v in secrets.items() if v and v in text}


def mask(s):
    for k, v in secrets.items():
        if v:
            s = s.replace(v, f"<{k}>")
    return re.sub(r'(Token=\\?")[0-9a-f]{32}', r"\1<a token the browser held>", s)


report = {"secrets_searched": sorted(secrets), "modes": {}, "logs": {}}
for mode_dir in sorted((root / "runs").iterdir()):
    har = json.loads((mode_dir / "full.har").read_text())
    rx = {"response_headers": {}, "response_bodies": {}}
    tx = {"request_headers": {}, "request_bodies": {}}
    received, tokens_received, rows = 0, set(), []
    for e in har["log"]["entries"]:
        rq, rs = e["request"], e["response"]
        b = body(rs.get("content", {}))
        received += len(b)
        for k, n in hits(json.dumps(rs["headers"])).items():
            rx["response_headers"][k] = rx["response_headers"].get(k, 0) + n
        for k, n in hits(b).items():
            rx["response_bodies"][k] = rx["response_bodies"].get(k, 0) + n
        for k, n in hits(json.dumps(rq["headers"])).items():
            tx["request_headers"][k] = tx["request_headers"].get(k, 0) + n
        for k, n in hits((rq.get("postData") or {}).get("text", "")).items():
            tx["request_bodies"][k] = tx["request_bodies"].get(k, 0) + n
        tokens_received |= set(re.findall(r'"AccessToken"\s*:\s*"([0-9a-f]{32})"', b))
        hdr = {h["name"].lower(): h["value"] for h in rq["headers"]}
        path = re.sub(r"^http://127\.0\.0\.1:\d+", "", rq["url"])
        rows.append("\t".join([rq["method"], str(rs["status"]), str(len(b)), hdr.get("range", ""),
                               mask(hdr.get("authorization", "")), mask(path)[:200]]))
        rs.get("content", {}).pop("text", None)
        if rq.get("postData"):
            rq["postData"]["text"] = mask(rq["postData"].get("text", ""))
    name = mode_dir.name
    (out / f"{name}.requests.tsv").write_text(
        "method\tstatus\tbody_bytes\trange\tauthorization_sent\tpath\n" + "\n".join(rows) + "\n")
    (out / f"{name}.har").write_text(mask(json.dumps(har)))
    (out / f"{name}.driver.log").write_text(mask((mode_dir / "driver.log").read_text()))
    report["modes"][name] = {
        "requests": len(rows), "body_bytes_received": received,
        "received": rx, "sent": tx,
        "jellyfin_tokens_handed_to_the_browser": len(tokens_received),
    }
for log in sorted(list((root / "sam").glob("*.log")) + list((root / "sam").glob("*.out"))
                  + list((root / "alex").glob("*.log")) + list((root / "alex").glob("*.out"))):
    report["logs"][str(log.relative_to(root))] = hits(log.read_text(errors="replace"))
report["jellyfin_log_on_sams_computer"] = hits(jlog)
names = {v: k for k, v in secrets.items()}
report["jellyfin_logged_out_in_order"] = [names[t] for t in logged_out]
(out / "scan.json").write_text(json.dumps(report, indent=1) + "\n")
print(json.dumps(report, indent=1))
