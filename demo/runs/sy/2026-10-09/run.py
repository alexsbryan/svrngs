#!/usr/bin/env python3
"""sy-1's run, rerunnable (README §2-§4): Ana, Ben and Cy, three svrngs data
dirs on this host through iroh's public relays, in one group, each syncing a
directory named Shared; Dee, a node outside it (crates/svrngs-node/tests/
sy_dee.rs); Eve, who joins later. Each bar's plant first, its guard out
(SVRNGS_GF_PLANT), then the bar. Data in run/; W is the scratch dir (else
/tmp/sy-run). svrngs is built first. scripts/check.sh runs none of this."""
import json, os, re, signal, subprocess, sys, time, urllib.request
from pathlib import Path
import blake3

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "run"; W = Path(os.environ.get("W", "/tmp/sy-run")); B = REPO / "target/debug/svrngs"
MARK = b"SY-PLANTED-BESIDE-THE-FOLDER-01"
nodes, summary, PHASE = {}, {}, ["setup"]


def say(event, **kw):
    line = {"t": round(time.time(), 3), "phase": PHASE[0], "event": event, **kw}
    print(json.dumps(line), flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(json.dumps(line) + "\n")


def disk(who): return W / "disk" / who / "Shared"
def data(who): return W / "data" / who


def up(who, *args, plant=""):
    env = {**os.environ, "SVRNGS_DIR": str(data(who)), "SVRNGS_GF_PLANT": plant, "NO_COLOR": "1",
           "RUST_LOG": "folder=info,rail:folder=debug,transport=warn"}
    (data(who) / "api").unlink(missing_ok=True)
    nodes[who] = subprocess.Popen([B, *args], env=env, stdout=open(data(who) / "out", "a"),
                                  stderr=open(OUT / f"{who}.log", "a"))
    if not wait(lambda: api(who, "/v1/folders"), 60): sys.exit(f"{who}'s node did not come up")
    say("up", who=who, plant=plant, args=[a if "join/" not in a else "<the link>" for a in args])


def down(who, sig=signal.SIGTERM):
    if who in nodes: p = nodes.pop(who); p.send_signal(sig); p.wait(); say("down", who=who, sig=sig.name)


def api(who, path, body=None):
    try:
        at = (data(who) / "api").read_text().strip()
        req = urllib.request.Request(at + path, data=body and json.dumps(body).encode(),
                                     headers={"content-type": "application/json"})
        return json.load(urllib.request.urlopen(req, timeout=5))
    except Exception: return None


def sync(who, plant=""):
    """who's node up and syncing their Shared, as `share --folder --both-ways` asks."""
    if who not in nodes: up(who, plant=plant)
    run = subprocess.run([B, "share", "--folder", str(disk(who)), "--both-ways"], capture_output=True,
                         text=True, env={**os.environ, "SVRNGS_DIR": str(data(who))})
    say("both-ways", who=who, answer=run.stdout.strip() or run.stderr.strip())


def state(d):
    return {str(p.relative_to(d)): blake3.blake3(p.read_bytes()).hexdigest()
            for p in sorted(d.rglob("*")) if p.is_file() and not p.is_symlink()}


def wait(cond, bound):
    t0 = time.time()
    while time.time() - t0 < bound:
        if cond(): return round(time.time() - t0, 2)
        time.sleep(0.1)
    return None


def holds(whos, path, digest):
    """Every one of whos holds path at digest (None: holds nothing there)."""
    return lambda: all(state(disk(w)).get(path) == digest for w in whos)


def write(who, path, blob):
    p = disk(who) / path; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(blob)
    say("wrote", who=who, path=path, size=len(blob), blake3=blake3.blake3(blob).hexdigest())
    return blake3.blake3(blob).hexdigest()


def bar(name, numbers, passed):
    summary[name] = {**numbers, "verdict": "passed" if passed else "failed"}
    say("bar", bar=name, **summary[name])


def phase(name): PHASE[0] = name; say("phase")


def s1(tag, plant):
    phase(f"S1{tag}"); [down(w) for w in "ana ben cy".split()]
    for w in ("ana", "ben", "cy"): sync(w, plant if w == "ana" else "")
    a = write("ana", f"s1{tag}.txt", os.urandom(4096))
    t_add = wait(holds(["ben", "cy"], f"s1{tag}.txt", a), 10)
    if t_add is None: return bar(f"S1{tag}", {"add_s": None}, False)
    b = write("ben", f"s1{tag}.txt", os.urandom(5000))
    t_edit = wait(holds(["ana", "cy"], f"s1{tag}.txt", b), 10)
    (disk("cy") / f"s1{tag}.txt").unlink(); say("deleted", who="cy", path=f"s1{tag}.txt")
    t_del = wait(holds(["ana", "ben"], f"s1{tag}.txt", None), 10)
    bar(f"S1{tag}", {"add_s": t_add, "edit_s": t_edit, "delete_s": t_del}, None not in (t_edit, t_del))


def s2(tag, plant):
    phase(f"S2{tag}"); [down(w) for w in "ana ben cy".split()]
    sync("ana"); sync("ben", plant); sync("cy"); time.sleep(3); down("cy")
    a = write("ana", f"s2{tag}.bin", os.urandom(1 << 16))
    wait(holds(["ben"], f"s2{tag}.bin", a), 30); down("ana", signal.SIGKILL)
    t0 = time.time(); sync("cy")  # from Cy's start, her command included
    wait(holds(["cy"], f"s2{tag}.bin", a), 10 - (time.time() - t0))
    t = round(time.time() - t0, 2) if holds(["cy"], f"s2{tag}.bin", a)() else None
    by = re.findall(rf"fetched, verified hash={a} holder=(\w+)", (OUT / "cy.log").read_text())
    bar(f"S2{tag}", {"cy_s": t, "hash": a, "from": [w for w, k in KEYS.items() if k in by]}, t is not None)


def s3(tag, plant):
    phase(f"S3{tag}"); [down(w) for w in "ana ben cy".split()]
    for w in ("ana", "ben", "cy"): sync(w, plant)
    v0 = write("ana", f"s3{tag}.txt", f"first version{tag}\n".encode())
    wait(holds(["ben", "cy"], f"s3{tag}.txt", v0), 30); time.sleep(2); down("ben")
    b = write("ben", f"s3{tag}.txt", f"Ben's edit{tag}, made while his node was stopped\n".encode())
    a = write("ana", f"s3{tag}.txt", f"Ana's edit{tag}, made while Ben's node was stopped\n".encode())
    wait(holds(["cy"], f"s3{tag}.txt", a), 30); sync("ben", plant)
    both = lambda: all({a, b} <= set(state(disk(w)).values()) for w in ("ana", "ben", "cy"))
    t = wait(both, 30)
    mine = lambda p: p == f"s3{tag}.txt" or p.startswith(f"s3{tag} (")
    found = {w: {p: h for p, h in state(disk(w)).items() if mine(p)} for w in ("ana", "ben", "cy")}
    bar(f"S3{tag}", {"both_s": t, "ana_edit": a, "ben_edit": b, "found": found}, t is not None)


def dee(tag, member, group, digest):
    d = W / f"dee{tag}"; d.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, "SY_DEE_DIR": str(d), "SY_MEMBER": member, "SY_GROUP": group, "SY_HASH": digest}
    run = subprocess.run(["cargo", "test", "--offline", "-q", "-p", "svrngs-node", "--test", "sy_dee", "--",
                          "--ignored", "--nocapture"], cwd=REPO, env=env, capture_output=True, text=True)
    (OUT / f"dee{tag}.out").write_text(run.stdout + run.stderr)
    return [json.loads(l[7:]) for l in run.stdout.splitlines() if l.startswith("SY-DEE ")], d


def scan(dirs, needles):
    hits = []
    for d in dirs:
        for p in [p for p in Path(d).rglob("*") if p.is_file() and not p.is_symlink()]:
            blob = p.read_bytes()
            hits += [{"file": str(p), "needle": n.hex()[:16]} for n in needles if n in blob]
    return hits


def s4(tag, plant):
    """A file Ana writes now: after S2's kill -9 her store lacked s2.bin's bytes (run of 20:28)."""
    phase(f"S4{tag}"); down("ana"); sync("ana", plant); blob = os.urandom(1 << 16)
    digest, first = write("ana", f"s4{tag}.bin", blob), blob[:64]; wait(holds(["ben", "cy"], f"s4{tag}.bin", digest), 30)
    answers, d = dee(tag, KEYS["ana"], GROUP, digest)
    hits = scan([d], [digest.encode(), first, bytes.fromhex(digest)])
    beside = scan([data(w) for w in ("ben", "cy")] + [disk(w) for w in ("ben", "cy")] + [d], [MARK])
    given = any("ops" in a["answer"] and a["answer"]["ops"] for a in answers) or any(
        "payload" in a["answer"] for a in answers)
    bar(f"S4{tag}", {"answers": answers, "dee_hits": len(hits), "beside_hits": len(beside),
                     "hits": hits[:5]}, not given and not hits and not beside)


def keys():
    """Each person's key and the group's id, from Ana's journal: her Admits."""
    ring = next((data("ana") / "rings").iterdir()); out = {}
    for line in (ring / "ring_oplog.jsonl").read_text().splitlines():
        m = re.search(r'"op":"admit","person":"(\w+)","key":"([0-9a-f]{64})"', line)
        if m: out[m[1].lower()] = m[2]; out["ana"] = re.search(r'"actor":"([0-9a-f]{64})"', line)[1]
    return out, ring.name


def main():
    global KEYS, GROUP
    subprocess.run(["cargo", "build", "--offline", "-q", "-p", "svrngs"], cwd=REPO, check=True)
    subprocess.run(["rm", "-rf", str(W), str(OUT)]); OUT.mkdir(parents=True)
    for w in ("ana", "ben", "cy", "eve"): data(w).mkdir(parents=True); disk(w).mkdir(parents=True)
    say("load", uptime=subprocess.run(["uptime"], capture_output=True, text=True).stdout.strip(),
        head=subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip())
    planted = bytearray(os.urandom(1 << 20))
    for at in range(0, len(planted), 4096): planted[at:at + len(MARK)] = MARK
    (disk("ana").parent / "beside.bin").write_bytes(planted); (disk("ana") / "link.bin").symlink_to("../beside.bin")
    up("ana", "new", "Shared", "--as", "Ana")
    wait(lambda: "svrngs://join/" in (data("ana") / "out").read_text(), 60)
    link = next(w for w in (data("ana") / "out").read_text().split() if w.startswith("svrngs://join/"))
    for w in ("ben", "cy"):
        up(w, "join", link, "--as", w.title())
        if wait(lambda: "You're in" in (data(w) / "out").read_text(), 60) is None: sys.exit(f"{w} did not join")
    KEYS, GROUP = keys(); say("keys", keys=KEYS, group=GROUP)
    s1("-plant", "deaf"); (disk("ana") / "s1-plant.txt").unlink(missing_ok=True); s1("", "")
    s2("-plant", "own-only"); s2("", "")
    s3("-plant", "later-wins"); s3("", "")
    s4("-plant", "open-gate"); s4("", "")
    m1(link); m2()
    (OUT / "summary.json").write_text(json.dumps(summary, indent=1))
    [down(w) for w in list(nodes)]


def journal(who): return max(data(who).rglob("ring_oplog.jsonl"), key=lambda p: p.stat().st_size)


def m1(link):
    phase("M1"); [sync(w) for w in ("ana", "ben", "cy") if w not in nodes]
    files = {f"m1/{i:04}.bin": os.urandom(4096) for i in range(1000)}
    for p, blob in files.items(): (disk("ana") / p).parent.mkdir(exist_ok=True); (disk("ana") / p).write_bytes(blob)
    want = {p: blake3.blake3(b).hexdigest() for p, b in files.items()}
    whole = lambda w: lambda: (lambda st: all(st.get(p) == h for p, h in want.items()))(state(disk(w)))
    t_members = wait(lambda: whole("ben")() and whole("cy")(), 600)
    up("eve", "join", link, "--as", "Eve"); wait(lambda: "You're in" in (data("eve") / "out").read_text(), 60)
    t0 = time.time(); sync("eve"); wait(whole("eve"), 600)
    t_eve = round(time.time() - t0, 2) if whole("eve")() else None
    j = journal("eve"); before = j.stat().st_size; act = max(len(l) for l in j.read_bytes().splitlines())
    h = write("ana", "m1/0001.bin", os.urandom(4096)); t_one = wait(holds(["eve"], "m1/0001.bin", h), 60)
    summary["M1"] = {"ben_cy_s": t_members, "eve_s": t_eve,
                     "eve_journal_bytes": before, "act_line_bytes": act, "one_change_s": t_one,
                     "one_change_bytes": journal("eve").stat().st_size - before}
    say("measured", **summary["M1"])


def m2():
    phase("M2"); big = bytearray(os.urandom(100 << 20))
    v1 = write("ana", "big.bin", bytes(big)); t1 = wait(holds(["ben", "cy"], "big.bin", v1), 600)
    big[50 << 20:51 << 20] = os.urandom(1 << 20); v2 = write("ana", "big.bin", bytes(big))
    t2 = wait(holds(["ben", "cy"], "big.bin", v2), 600)
    got = {w: sum(int(l.split(" payload=")[1].split()[0]) for l in (OUT / f"{w}.log").read_text().splitlines()
                  if "fetched, verified" in l and f"hash={v2}" in l) for w in ("ben", "cy")}
    summary["M2"] = {"first_s": t1, "edit_s": t2, "received_bytes": got, "edit_bytes": 1 << 20}
    say("measured", **summary["M2"])


if __name__ == "__main__":
    main()
