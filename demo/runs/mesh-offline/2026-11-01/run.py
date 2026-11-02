#!/usr/bin/env python3
"""mesh-offline's run (PREDICTIONS.md here): a group on the shipped svrngs
binary, n0's lookup on as it ships, with the internet cut by modem-off.sb
(every svrngs and every lent app under `sandbox-exec`), meeting at
`svrngs relay --at 127.0.0.1:<port> --open`. The profile is checked against
planted fetches first. Sam makes the group; Ana and Alex join; a file and a
lend of Ana's; a cold restart of everything; a file and a lend of Alex's;
then Dee joins by a link made after the restart. Every remote address each
process opened is sampled with lsof; each trace is read at the end for n0's
lookups and how fast they failed.

    BIN=<svrngs built at 227193b1> W=<scratch, absent> python3 demo/runs/mesh-offline/2026-11-01/run.py

Data in data/ beside this file, the group's links never in it.
scripts/check.sh runs none of this."""
import datetime, gzip, hashlib, json, os, re, signal, socket, subprocess, sys, threading, time, urllib.error, urllib.request
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "data"; W = Path(os.environ["W"]); BIN = os.environ["BIN"]; SB = HERE / "modem-off.sb"
GROUP = "mesh"
LOG = ("info,cli=debug,transport=debug,join=debug,group=debug,folder=info,door=debug,origin=debug,surface=debug,"
       "svrngs_node=debug,iroh::address_lookup=trace,iroh::socket::remote_map=debug,iroh::net_report=debug")
procs, pids, summary, hidden, started = {}, {}, {}, [], {}
lock = threading.Lock()


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}))
    print(line, flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def wait(cond, bound, every=0.1):
    t0 = time.time()
    while time.time() - t0 < bound:
        try: got = cond()
        except Exception: got = None
        if got: return got
        time.sleep(every)
    return None


def timed(cond, bound, since=None):
    """ms from `since` (else now) until `cond` holds, or None at `bound` s."""
    t0 = since or time.time()
    return round((time.time() - t0) * 1000) if wait(cond, bound - (time.time() - t0)) else None


def boxed(cmd): return ["sandbox-exec", "-f", str(SB), *map(str, cmd)]


def free_port():
    s = socket.socket(); s.bind(("127.0.0.1", 0)); p = s.getsockname()[1]; s.close(); return p


def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()


# --- each computer: a HOME and data dir of its own, under the profile -------
def home(who): return W / who
def data(who): return home(who) / ".svrngs"
def folder(who): return home(who) / "svrngs" / GROUP
def env_of(who, log=LOG):
    e = {k: v for k, v in os.environ.items() if k not in ("SVRNGS_DIR", "RUST_LOG")}
    return {**e, "HOME": str(home(who)), "USER": who, "SVRNGS_DIR": str(data(who)), "NO_COLOR": "1", "RUST_LOG": log}


def start(who, *args, log=LOG):
    """`svrngs <args>` as `who` types it, kept running, under the profile."""
    home(who).mkdir(parents=True, exist_ok=True); (W / "logs").mkdir(exist_ok=True)
    n = started[who] = started.get(who, 0) + 1
    out, err = W / "logs" / f"{who}-{n}.out", W / "logs" / f"{who}-{n}.log"
    p = subprocess.Popen(boxed([BIN, *args]), env=env_of(who, log), stdin=subprocess.DEVNULL,
                         stdout=open(out, "w"), stderr=open(err, "w"))
    with lock: procs[who] = p; pids[p.pid] = who
    t = time.time()
    say("started", who=who, n=n, args=[redact(a) for a in args], pid=p.pid)
    return t, out


def stop(who):
    """Ctrl-C, as a person stops it; SIGKILL at 20 s, said."""
    p = procs.pop(who, None)
    if not p or p.poll() is not None: return None
    t = time.time(); p.send_signal(signal.SIGINT)
    try: p.wait(20); killed = False
    except subprocess.TimeoutExpired: p.kill(); p.wait(); killed = True
    with lock: pids.pop(p.pid, None)
    r = {"who": who, "exit": p.returncode, "ms": round((time.time() - t) * 1000), "killed": killed}
    say("stopped", **r)
    return r


def http(url, headers=None, timeout=15):
    req = urllib.request.Request(url, headers=headers or {})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: return r.status, r.read().decode(errors="replace")
    except urllib.error.HTTPError as e: return e.code, e.read().decode(errors="replace")
    except Exception as e: return None, str(e)


def base(who): return (data(who) / "api").read_text().strip().rstrip("/")


def api(who, path):
    try: token = (data(who) / "token").read_text().strip()
    except Exception: return None
    status, body = http(base(who) + path, {"Authorization": f"Bearer {token}"})
    if status != 200: return None
    try: return json.loads(body)
    except Exception: return None


def gid(who):
    g = api(who, "/v1/groups")
    return g and g.get("groups") and g["groups"][0]["id"]


def lends(who):
    r = api(who, f"/v1/lends?namespace={GID}")
    return r and {l.get("name"): l.get("origin") for l in r.get("lent", [])}, r


def listed(who, name):
    got, _ = lends(who)
    return got and got.get(name)


def fetch(who, origin, words):
    """GET / at the lend's origin on `who`'s own listener, as a program on
    that computer asks it: with the computer's token (svrngs-node
    origin.rs:164-165; take 1 sent none, and every fetch was refused at
    origin.rs:189, its README says)."""
    host = re.match(r"https?://([^/]+)/", origin)[1]
    token = (data(who) / "token").read_text().strip()
    status, body = http(f"http://127.0.0.1:{base(who).rsplit(':', 1)[1]}/", {"Host": host, "Authorization": f"Bearer {token}"})
    return status == 200 and words in body


def lend_seen(who, name, words, bound, since):
    """(ms listed, ms fetched) on `who`'s computer, each from `since`."""
    origin = wait(lambda: listed(who, name), bound - (time.time() - since))
    at_listed = round((time.time() - since) * 1000) if origin else None
    at_fetched = timed(lambda: fetch(who, origin, words), bound, since) if origin else None
    _, said = lends(who)
    return {"listed_ms": at_listed, "fetched_ms": at_fetched,
            "unlisted": said and {k: said.get(k) for k in ("unanswered", "unheard", "refused", "older")}}


def each(fn, whos):
    """fn(who) for every one of `whos` at once, so no wait eats another's
    bound (take 1 waited on them one after another from one start)."""
    got = {}
    ts = [threading.Thread(target=lambda w=w: got.__setitem__(w, fn(w))) for w in whos]
    for t in ts: t.start()
    for t in ts: t.join()
    return {w: got.get(w) for w in whos}


def joined(out):
    return re.search(r"You.re in .*", out.read_text())


def link_in(out):
    m = re.search(r"svrngs://join/([A-Za-z0-9_-]+)", out.read_text())
    if m and m[1] not in hidden: hidden.append(m[1])
    return m and m[0]


def put_file(who, name):
    p = folder(who) / name
    p.write_bytes(os.urandom(1_000_000))
    return sha(p), time.time()


def arrived(who, name, want):
    p = folder(who) / name
    return p.is_file() and sha(p) == want


def app(name, words):
    """A page on loopback, served under the profile, as the lend's app."""
    d = W / "apps" / name; d.mkdir(parents=True, exist_ok=True)
    (d / "index.html").write_text(f"<!doctype html><title>{name}</title><h1>{words}</h1>\n")
    port = free_port()
    p = subprocess.Popen(boxed(["python3", "-m", "http.server", port, "--bind", "127.0.0.1", "--directory", d]),
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    procs[f"app-{name}"] = p
    with lock: pids[p.pid] = f"app-{name}"
    wait(lambda: http(f"http://127.0.0.1:{port}/", timeout=2)[0] == 200, 10)
    return port


def share(who, port, name):
    t = time.time()
    r = subprocess.run(boxed([BIN, "share", port, "--kind", "page", "--name", name]),
                       env=env_of(who, "cli=debug"), input="\n", capture_output=True, text=True, timeout=60)
    out = {"who": who, "name": name, "exit": r.returncode, "ms": round((time.time() - t) * 1000),
           "said": r.stdout.strip()[-400:], "trace": r.stderr.strip()[-600:]}
    say("share", **out)
    return out, time.time()


# --- the instrument: modem off, checked against plants ----------------------
def probe(name, cmd, inside=True, timeout=30):
    t = time.time()
    try: r = subprocess.run(boxed(cmd) if inside else list(map(str, cmd)), capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired: return {"name": name, "inside": inside, "exit": None, "ms": timeout * 1000, "said": "timed out"}
    return {"name": name, "inside": inside, "cmd": list(map(str, cmd)), "exit": r.returncode,
            "ms": round((time.time() - t) * 1000), "said": (r.stdout + r.stderr).strip()[-300:]}


def instrument(control_port):
    curl = ["curl", "-sS", "-m5", "-o", "/dev/null", "-w", "%{http_code}"]
    udp = ("import socket,time;t=time.time()\ntry:\n socket.socket(socket.AF_INET,socket.SOCK_DGRAM)"
           ".sendto(b'x',('100.100.100.100',53));print('sent')\nexcept OSError as e:print(repr(e))\n"
           "print(round((time.time()-t)*1000),'ms')")
    got = [probe("curl a name", curl + ["https://example.com"]),
           probe("curl an address", curl + ["https://1.1.1.1"]),
           probe("dig", ["dig", "+time=2", "+tries=1", "example.com"]),
           probe("host", ["host", "-W", "2", "example.com"]),
           probe("udp to this host's resolver", ["python3", "-c", udp]),
           probe("curl loopback (control)", curl + [f"http://127.0.0.1:{control_port}/"]),
           probe("curl a name, outside the profile", curl + ["https://example.com"], inside=False)]
    p = {g["name"]: g for g in got}
    verdict = {
        "curl a name": p["curl a name"]["exit"] != 0 and "resolve" in p["curl a name"]["said"],
        "curl an address": p["curl an address"]["exit"] != 0,
        "dig": "no servers could be reached" in p["dig"]["said"] or p["dig"]["exit"] != 0,
        "host": p["host"]["exit"] != 0,
        "udp to this host's resolver": "sent" not in p["udp to this host's resolver"]["said"].split("\n")[0],
        "curl loopback (control)": p["curl loopback (control)"]["said"].endswith("200"),
        "outside": p["curl a name, outside the profile"]["said"].endswith("200"),
    }
    summary["instrument"] = {"probes": got, "held": verdict}
    say("instrument", held=verdict)
    if not all(verdict.values()): sys.exit(f"the instrument is not modem off: {verdict}")


# --- every remote address each process opened -------------------------------
def sampler(stop_ev):
    seen = set(); n = 0
    while not stop_ev.is_set():
        with lock: watched = dict(pids)
        if watched:
            r = subprocess.run(["lsof", "-i", "-n", "-P", "-a", "-p", ",".join(map(str, watched))], capture_output=True, text=True)
            n += 1
            for line in r.stdout.splitlines()[1:]:
                f = line.split()
                if len(f) < 9: continue
                name = " ".join(f[8:])
                remote = name.split("->", 1)[1].split()[0] if "->" in name else None
                key = (watched.get(int(f[1]), "?"), f[7], remote or name.split()[0])
                if key not in seen:
                    seen.add(key)
                    with open(OUT / "lsof.jsonl", "a") as out:
                        out.write(json.dumps({"t": round(time.time(), 3), "who": key[0], "pid": int(f[1]), "proto": key[1],
                                              "remote": remote, "local": None if remote else key[2]}) + "\n")
        summary["lsof_samples"] = n
        stop_ev.wait(2)


# --- reading the traces -----------------------------------------------------
ANSI = re.compile(r"\x1b\[[0-9;]*m")
STAMP = re.compile(r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+)Z")


def at(line):
    m = STAMP.match(line)
    if not m: return None
    s = m[1][:26]
    return datetime.datetime.strptime(s, "%Y-%m-%dT%H:%M:%S.%f").replace(tzinfo=datetime.timezone.utc).timestamp()


def lines_of(path): return [ANSI.sub("", l) for l in path.read_text(errors="replace").splitlines()] if path.exists() else []


def first_round(path, since, reached):
    for l in lines_of(path):
        m = re.search(r"ring sync: round .*?peers_reached=(\d+) peers_unreachable=(\d+)", l)
        if m and int(m[1]) >= reached and at(l): return round((at(l) - since) * 1000)
    return None


def bare_dials(ls):
    """Each dial of a bare key (svrngs-node iroh.rs:283-289): ms to its
    "a dial, answered or not" line for that peer, and that line's error."""
    out = []
    for i, l in enumerate(ls):
        m = re.search(r"a bare key — dialled through.*?peer=(\w+)", l)
        if not (m and at(l)): continue
        for l2 in ls[i + 1:]:
            if "a dial, answered or not" in l2 and f"peer={m[1]}" in l2 and at(l2):
                e = re.search(r"error=(.*)$", l2)
                out.append({"peer": m[1], "ms": round((at(l2) - at(l)) * 1000, 1), "error": e and e[1][:160]})
                break
    return out


def read_trace(path):
    ls = lines_of(path)
    def pairs(begin, end, key=None):
        """ms from each `begin` line to the next `end` line (of the same `key` group)."""
        out, open_ = [], {}
        for l in ls:
            k = (re.search(key, l)[1] if key and re.search(key, l) else None)
            if begin in l and at(l): open_[k] = at(l)
            elif end in l and k in open_ and at(l): out.append(round((at(l) - open_.pop(k)) * 1000))
        return out
    find = lambda s: [l for l in ls if s in l]
    pk_fail = find("Failed to publish to pkarr")
    dns_fail = find("DNS lookup failed")
    rounds = [tuple(map(int, m.groups())) for l in ls
              for m in [re.search(r"ring sync: round .*?peers_reached=(\d+) peers_unreachable=(\d+)", l)] if m]
    warns = Counter(re.sub(r"\s+", " ", l.split(": ", 1)[-1])[:110] for l in ls if " WARN " in l or " ERROR " in l)
    return {
        "lines": len(ls),
        "relays_line": [l[l.find("cli:"):][:300] for l in find("the relays this computer meets its groups at")][:1],
        "egress": [l[l.find("n0_services"):][:200] for l in find("iroh egress posture")][:1],
        "pkarr_attempts": len(find("Publishing endpoint info to pkarr")),
        "pkarr_failures": len(pk_fail),
        "pkarr_fail_ms": pairs("Publishing endpoint info to pkarr", "Failed to publish to pkarr"),
        "pkarr_first_failure": pk_fail[0][-400:] if pk_fail else None,
        "pkarr_retry_after_max": max([float(m[1]) for l in pk_fail for m in [re.search(r"retry_after=(\d+(?:\.\d+)?)s", l)] if m], default=None),
        "dns_started": len(find("starting DNS lookup")),
        "dns_failures": len(dns_fail),
        "dns_fail_ms": pairs("starting DNS lookup", "DNS lookup failed", r"lookup_id=(\w+)"),
        "dns_first_failure": dns_fail[0][-400:] if dns_fail else None,
        "lookup_failed": len(find("Address Lookup failed")),
        "bare_key_dials": [l[-260:] for l in find("a bare key — dialled through")][:5],
        "bare_key_count": len(find("a bare key — dialled through")),
        "bare_key_dial_ms": bare_dials(ls),
        "no_relay_named": len(find("the group names no relay for this computer")),
        "datagram_refused": len(find("failed to send datagram")),
        "folder_fetched": len(find("folder: fetched, verified")),
        "rounds": Counter(rounds).most_common(),
        "n0_words": len([l for l in ls if re.search(r"iroh\.link|pkarr|relay\.n0|n0_dns", l)]),
        "warn_error": warns.most_common(12),
    }


def main():
    if W.exists(): sys.exit(f"{W} exists: give the run a scratch directory of its own")
    W.mkdir(parents=True); OUT.mkdir(exist_ok=True)
    built = subprocess.run(["git", "-C", Path(BIN).parents[2], "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    dirty = subprocess.run(["git", "-C", Path(BIN).parents[2], "status", "--porcelain", "--untracked-files=no"], capture_output=True, text=True).stdout.strip()
    summary["start"] = {"build": built, "build_tree_clean": not dirty, "bin_sha256": sha(BIN), "load": os.getloadavg()[0],
                        "cores": os.cpu_count(), "macos": subprocess.run(["sw_vers", "-productVersion"], capture_output=True, text=True).stdout.strip()}
    say("start", **summary["start"])
    stop_ev = threading.Event()
    global GID
    try:
        # 0. The apps, then the instrument against them.
        notes = app("notes", "Ana's notes, lent offline")
        board = app("board", "Alex's board, lent after the restart")
        instrument(notes)
        threading.Thread(target=sampler, args=(stop_ev,), daemon=True).start()

        # 1. The relay.
        RPORT = free_port(); RELAY = f"http://127.0.0.1:{RPORT}"
        def relay_up():
            t, out = start("relay", "relay", "--at", f"127.0.0.1:{RPORT}", "--open", log="relay=debug,iroh_relay=info")
            said = wait(lambda: "Relaying at" in out.read_text() and out.read_text().strip(), 15)
            return {"said": said, "ms": round((time.time() - t) * 1000) if said else None}
        summary["relay"] = {"at": RELAY, "first": relay_up()}
        say("relay", **summary["relay"])
        if not summary["relay"]["first"]["said"]: sys.exit("the relay said nothing")

        # 2. Sam makes the group; Ana and Alex join by its link.
        t, out = start("sam", "new", GROUP, "--as", "Sam", "--relay", RELAY)
        link = wait(lambda: link_in(out), 60)
        summary["sam"] = {"link_ms": round((time.time() - t) * 1000) if link else None, "said": redact(out.read_text()[-300:])}
        say("made", **summary["sam"])
        if not link: sys.exit(f"Sam's new printed no link: {redact(out.read_text())[-400:]}")
        GID = wait(lambda: gid("sam"), 30)
        for who, name in (("ana", "Ana"), ("alex", "Alex")):
            t, o = start(who, "join", link, "--as", name)
            said = wait(lambda: joined(o), 60)
            summary[who] = {"joined_ms": round((time.time() - t) * 1000) if said else None, "said": said and said[0],
                            "folder": bool(wait(lambda: folder(who).is_dir(), 10))}
            say("joined", who=who, **summary[who])

        # 3. A file of Ana's.
        want1, t = put_file("ana", "before.bin")
        summary["file_before"] = each(lambda w: timed(lambda: arrived(w, "before.bin", want1), 60, t), ("sam", "alex"))
        say("file-before", **summary["file_before"])

        # 4. Ana's lend.
        summary["share_notes"], t = share("ana", notes, "notes")
        summary["notes_before"] = each(lambda w: lend_seen(w, "notes", "Ana's notes", 60, t), ("ana", "sam", "alex"))
        if not summary["notes_before"]["ana"]["fetched_ms"]: sys.exit("Ana's own lend not fetched on her own computer: the fetch is not an instrument")
        say("notes-before", **summary["notes_before"])

        # 7. The cold restart: every computer, then the relay, Ctrl-C.
        time.sleep(5)
        summary["stopped"] = [stop(w) for w in ("sam", "ana", "alex", "relay")]
        time.sleep(3)
        summary["relay"]["again"] = relay_up()
        say("relay-again", **summary["relay"]["again"])
        t_start = {}
        for w in ("sam", "ana", "alex"):
            t_start[w], _ = start(w)
        summary["restart"] = {"api_ms": each(lambda w: timed(lambda: gid(w) == GID, 60, t_start[w]), list(t_start))}
        say("restart-api", **summary["restart"]["api_ms"])
        # 8. Ana's lend, as it was, on the others; a file of Alex's.
        summary["restart"]["notes"] = each(lambda w: lend_seen(w, "notes", "Ana's notes", 120, t_start[w]), ("sam", "alex"))
        say("restart-notes", **summary["restart"]["notes"])
        want2, t = put_file("alex", "after.bin")
        summary["restart"]["file_after"] = each(lambda w: timed(lambda: arrived(w, "after.bin", want2), 120, t), ("sam", "ana"))
        say("restart-file", **summary["restart"]["file_after"])
        # Each first round reaching both others, from the trace, waited for.
        log_of = lambda w: W / "logs" / f"{w}-{started[w]}.log"
        wait(lambda: all(first_round(log_of(w), t_start[w], 2) is not None for w in t_start), 150 - (time.time() - min(t_start.values())), 1)
        summary["restart"]["first_round_both_ms"] = {w: first_round(log_of(w), t_start[w], 2) for w in t_start}
        summary["restart"]["first_round_any_ms"] = {w: first_round(log_of(w), t_start[w], 0) for w in t_start}
        say("restart-rounds", both=summary["restart"]["first_round_both_ms"], any=summary["restart"]["first_round_any_ms"])

        # 9. A new lend, Alex's.
        summary["share_board"], t = share("alex", board, "board")
        summary["board"] = each(lambda w: lend_seen(w, "board", "Alex's board", 60, t), ("sam", "ana"))
        say("board", **summary["board"])

        # 10. Dee, by a link made after the restart.
        summary["sam_for_link"] = stop("sam")
        t, out = start("sam", "link", GROUP)
        link2 = wait(lambda: link_in(out), 60)
        summary["link_again"] = {"ms": round((time.time() - t) * 1000) if link2 else None, "said": redact(out.read_text()[-300:])}
        say("link-again", **summary["link_again"])
        if link2:
            t, o = start("dee", "join", link2, "--as", "Dee")
            said = wait(lambda: joined(o), 60)
            t_in = time.time()
            dee = {"joined_ms": round((t_in - t) * 1000) if said else None, "said": said and said[0]}
            if said:
                seen = each(lambda n: lend_seen("dee", n, {"notes": "Ana's notes", "board": "Alex's board"}[n], 60, t_in), ("notes", "board"))
                dee.update(seen)
                hashes = {"before.bin": want1, "after.bin": want2}
                dee["files"] = each(lambda n: timed(lambda: arrived("dee", n, hashes[n]), 60, t_in), list(hashes))
            else:
                dee["out"] = redact(o.read_text()[-600:])
            summary["dee"] = dee; say("dee", **dee)
        time.sleep(20)
        summary["people"] = api("sam", f"/v1/rail/people?namespace={GID}")
    finally:
        summary["stopped_end"] = [stop(w) for w in list(procs)]
        stop_ev.set(); time.sleep(2.5)
        summary["traces"] = {}
        for f in sorted((W / "logs").glob("*")):
            text = redact(f.read_text(errors="replace"))
            if f.suffix == ".log": summary["traces"][f.stem] = read_trace(f)
            with gzip.open(OUT / f"{f.name}.gz", "wt") as g: g.write(text)
        seen = [json.loads(l) for l in (OUT / "lsof.jsonl").read_text().splitlines()] if (OUT / "lsof.jsonl").exists() else []
        loop = lambda r: r.rsplit(":", 1)[0].strip("[]") in ("127.0.0.1", "::1", "localhost")
        summary["off_host"] = [s for s in seen if s["remote"] and not loop(s["remote"])]
        summary["end"] = {"load": os.getloadavg()[0], "t": round(time.time(), 3)}
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, default=str)))
        say("end", off_host=len(summary["off_host"]), lsof_samples=summary.get("lsof_samples"))


main()
