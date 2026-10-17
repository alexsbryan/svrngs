#!/usr/bin/env python3
"""gf-1's bars, judged from the run's data (demo/runs/gf/2026-10-09/README.md
§3 the bars, §4 how each is measured). Four verdicts (AGENTS.md 6): passed,
failed, could-not-judge, never-ran; an absence is said, never defaulted.

    analyze.py OUT W
    analyze.py scan OUT DIR NAME   the scan of DIR, to OUT/scan-NAME.json

OUT is the run's data (run.sh): the three nodes' logs, the driver's marks and
results, events.jsonl, the probes; W the scratch dir, whose Ben data dir and
media the scan reads. Writes OUT/summary.json; prints the summary.
"""
import json
import os
import re
import sys
from datetime import datetime

import blake3

MB = 1_000_000
MARKER = b"GF-PLANTED-BESIDE-THE-FOLDER-0001"


def scan(root, planted, skip=()):
    """Every file under `root`, read whole, searched for the planted file's
    marker and its hash, hex and raw: B7's scan."""
    hits, files, total = [], 0, 0
    needles = [MARKER] + ([planted.encode(), bytes.fromhex(planted)] if planted else [])
    for d, dirs, names in os.walk(root):
        dirs[:] = [x for x in dirs if os.path.join(d, x) not in skip]
        for n in names:
            p = os.path.join(d, n)
            if os.path.islink(p) or not os.path.isfile(p):
                continue
            b = open(p, "rb").read()
            files += 1
            total += len(b)
            if any(x in b for x in needles):
                hits.append(os.path.relpath(p, root))
    return {"root": root, "files": files, "bytes": total, "hits": hits}


if sys.argv[1] == "scan":
    out, root, name = sys.argv[2:5]
    planted = [json.loads(l)["blake3"] for l in open(os.path.join(out, "media.jsonl"))
               if json.loads(l)["path"].endswith("beside.bin")][0]
    found = scan(root, planted, skip=tuple(os.path.join(root, x) for x in sys.argv[5:]))
    json.dump(found, open(os.path.join(out, f"scan-{name}.json"), "w"), indent=1)
    print(json.dumps(found))
    sys.exit(0)
OUT, W = sys.argv[1], sys.argv[2]
JF_UA = re.compile(r"Lavf|ffmpeg|ffprobe|Jellyfin", re.I)


def rd(name):
    p = os.path.join(OUT, name)
    return [json.loads(l) for l in open(p) if l.strip()] if os.path.exists(p) else []


def jload(name):
    p = os.path.join(OUT, name)
    return json.load(open(p)) if os.path.exists(p) else None


# --- the nodes' logs: one record per `folder:` line --------------------------
FIELD = re.compile(r'(\w+)=("(?:[^"\\]|\\.)*"|.*?)(?= \w+=|$)')
MSGS = ["served", "fetched, verified", "a holder gave nothing of the range",
        "refused — no standing folder of this computer's names the hash",
        "serving — a standing folder of this computer's names the hash", "sent",
        "refused — no standing folder names the hash", "refused — no holder gave the range, verified",
        "refused mid-answer", "the endpoint listens on loopback", "written to the record"]


def log_of(who):
    out = []
    p = os.path.join(OUT, f"{who}.log")
    if not os.path.exists(p):
        return out
    for line in open(p, errors="replace"):
        m = re.match(r"^(\S+Z)\s+\w+\s+folder: folder: (.*)$", line.rstrip("\n"))
        if not m:
            continue
        rest = m.group(2)
        msg = next((x for x in sorted(MSGS, key=len, reverse=True) if rest.startswith(x)), None)
        if msg is None:
            continue
        rec = {"t": datetime.fromisoformat(m.group(1)).timestamp(), "who": who, "msg": msg}
        for k, v in FIELD.findall(rest[len(msg):].strip()):
            rec[k] = v[1:-1] if v.startswith('"') else v
        for k in ("start", "end", "sent", "ms", "payload", "other", "last"):
            if k in rec:
                rec[k] = int(rec[k])
        out.append(rec)
    return out


logs = {who: log_of(who) for who in ("ana", "ben", "cy")}
ben = logs["ben"]
holders_log = logs["ana"] + logs["cy"]
media = {os.path.basename(m["path"]): m for m in rd("media.jsonl")}
MP4, MKV = media.get("exp.mp4", {}).get("blake3"), media.get("exp.mkv", {}).get("blake3")
PLANTED = media.get("beside.bin", {}).get("blake3")
source = {h: os.path.join(W, "media/ana/films", n) for n, h in
          (("exp.mp4", MP4), ("exp.mkv", MKV))}
if PLANTED:
    source[PLANTED] = os.path.join(W, "media/ana/beside.bin")
listed = jload("setup-ben-folders.json") or {"folders": []}
KEY = {f["person"]: f["key"] for f in listed["folders"]}
WHO = {v: k for k, v in KEY.items()}
events = rd("events.jsonl")
marks = rd("marks.jsonl")
results = rd("results.jsonl")
state = jload("state.json") or {}


def event(phase, text):
    return next((e["t"] for e in events if e["phase"] == phase and e["event"].startswith(text)), None)


def window(run, browser, file):
    s = next((m for m in marks if m.get("run") == run and m["browser"] == browser and m["file"] == file
              and m["phase"] == "start"), None)
    e = next((m for m in marks if m.get("run") == run and m["browser"] == browser and m["file"] == file
              and m["phase"] == "end"), None)
    return (s["t"], e["t"], s["ua"]) if s and e else None


def result(run, browser, file):
    return next((r for r in results if r.get("phase") == run and r["browser"] == browser
                 and r["file"] == file), None)


def phase_span(phase):
    ts = [e["t"] for e in events if e["phase"] == phase]
    nxt = [e["t"] for e in events if ts and e["t"] > max(ts) and e["phase"] != phase]
    return (min(ts), min(nxt) if nxt else float("inf")) if ts else None


# --- ranges: what each holder sent, and what the endpoint wrote ---------------
def union(spans):
    out = []
    for a, b in sorted(s for s in spans if s[1] > s[0]):
        if out and a <= out[-1][1]:
            out[-1][1] = max(out[-1][1], b)
        else:
            out.append([a, b])
    return out


def overlap(xs, ys):
    i = j = n = 0
    while i < len(xs) and j < len(ys):
        a, b = max(xs[i][0], ys[j][0]), min(xs[i][1], ys[j][1])
        n += max(0, b - a)
        if xs[i][1] < ys[j][1]:
            i += 1
        else:
            j += 1
    return n


def chunk_spans(ranges):
    """`ChunkRangesSeq([(0, RangeSet{1024..2048, 3000..}), ...])`'s blob ranges, in bytes."""
    m = re.search(r"\(0, RangeSet\{([^}]*)\}\)", ranges or "")
    spans = []
    for part in (m.group(1).split(",") if m and m.group(1).strip() else []):
        a, _, b = part.strip().partition("..")
        spans.append((int(a) * 1024, int(b) * 1024 if b else 1 << 62))
    return spans


def uploads(hash_, t0, t1):
    """Each request a holder answered for `hash_` between t0 and t1: its spans
    cut at the last offset sent, and payload and other bytes sent."""
    out = []
    for who, recs in (("ana", logs["ana"]), ("cy", logs["cy"])):
        asked = {(r["conn"], r["request"]): r for r in recs if r["msg"].startswith("serving")}
        for r in recs:
            if r["msg"] != "sent" or r.get("hash") != hash_ or not (t0 <= r["t"] <= t1):
                continue
            a = asked.get((r["conn"], r["request"]))
            spans = chunk_spans(a["ranges"]) if a else []
            spans = [(x, min(y, r["last"])) for x, y in spans if x < r["last"]]
            out.append({"who": who, "t": r["t"], "spans": spans, "payload": r["payload"],
                        "other": r["other"], "done": r["done"], "asked_at": a["t"] if a else None})
    return out


def served(hash_, t0, t1, ua=None):
    return [r for r in ben if r["msg"] == "served" and r.get("hash") == hash_ and t0 <= r["t"] <= t1
            and (ua is None or r.get("ua") == ua)]


verdict, data = {}, {}

# --- B3: the scan fetches nothing ---------------------------------------------
if state.get("scan_window"):
    s0, s1 = state["scan_window"]
    at_holders = [r for r in holders_log if s0 <= r["t"] <= s1 and r["msg"] in ("sent",) or
                  (s0 <= r["t"] <= s1 and r["msg"].startswith(("serving", "refused — no standing folder of")))]
    at_endpoint = [r for r in ben if s0 <= r["t"] <= s1 and r["msg"] == "served"]
    data["B3"] = {"scan_s": round(s1 - s0, 1), "holder_requests": len([r for r in at_holders if r["msg"] != "sent"]),
                  "endpoint_answers": len(at_endpoint),
                  "endpoint_uas": sorted({r.get("ua", "") for r in at_endpoint})}
    verdict["B3"] = "passed" if data["B3"]["holder_requests"] == 0 else "failed"
else:
    verdict["B3"] = "never-ran"


# --- B2, B4, B5: Chromium's MP4 in A; WebKit's beside, as data ----------------
def play_numbers(run, browser, file, hash_, since):
    w, r = window(run, browser, file), result(run, browser, file)
    if not w or not r:
        return None
    t0, t1, ua = w
    ups = uploads(hash_, since, t1)
    sent_spans = union([s for u in ups for s in u["spans"]])
    holder_total = sum(u["payload"] + u["other"] for u in ups)
    browser_spans = union([(x["start"], x["start"] + x["sent"]) for x in served(hash_, since, t1, ua)])
    to_browser = overlap(browser_spans, sent_spans)
    net = [n["t"] for n in r.get("net", []) if f":{18733}/" in n["url"] and hash_ in n["url"]]
    first_browser = min(net) if net else None
    probe = [u for u in uploads(hash_, t0, first_browser or t1)] if first_browser else []
    jf = [x for x in served(hash_, t0, t1) if JF_UA.search(x.get("ua", ""))]
    return {
        "window": [round(t0, 3), round(t1, 3)], "ua": ua,
        "holder_sent_bytes": holder_total, "holder_payload_bytes": sum(u["payload"] for u in ups),
        "holder_requests": len(ups), "holders": sorted({u["who"] for u in ups}),
        "browser_bytes_of_what_holders_sent": to_browser,
        "browser_bytes_written": sum(e - s for s, e in browser_spans),
        "share": round(to_browser / holder_total, 4) if holder_total else None,
        "probe_holder_bytes": sum(u["payload"] + u["other"] for u in probe) if first_browser else None,
        "jellyfin_answers": len(jf), "jellyfin_bytes_written": sum(x["sent"] for x in jf),
        "jellyfin_uas": sorted({x.get("ua") for x in jf}),
        "first": r.get("first"), "seeks_ms": [s.get("ms") for s in r.get("seeks", [])],
        "play_method": r.get("playMethod"), "src": (r.get("video") or {}).get("currentSrc"),
        "error": r.get("error"),
    }


if state.get("scan_window"):
    scan_start = state["scan_window"][0]
    c = play_numbers("A", "chromium", "mp4", MP4, scan_start)
    data["A_chromium_mp4"] = c
    wk_since = window("A", "webkit", "mp4")
    data["A_webkit_mp4"] = play_numbers("A", "webkit", "mp4", MP4, wk_since[0]) if wk_since else None
    mkv_since = window("A", "webkit", "mkv")
    data["A_webkit_mkv"] = play_numbers("A", "webkit", "mkv", MKV, mkv_since[0]) if mkv_since else None
    if data["A_webkit_mkv"]:
        data["A_webkit_mkv"]["file_size"] = media["exp.mkv"]["size"]
    if c is None:
        verdict.update({"B2": "never-ran", "B4": "never-ran", "B5": "never-ran"})
    else:
        verdict["B2"] = ("could-not-judge" if c["share"] is None else
                         "passed" if c["share"] >= 0.95 else "failed")
        s = c["seeks_ms"]
        verdict["B4"] = ("could-not-judge" if len(s) != 3 else
                         "passed" if all(x is not None and x <= 2000 for x in s) else "failed")
        verdict["B5"] = ("could-not-judge" if c["probe_holder_bytes"] is None else
                         "passed" if c["probe_holder_bytes"] <= 10 * MB else "failed")
else:
    verdict.update({"B2": "never-ran", "B4": "never-ran", "B5": "never-ran"})


# --- B6: a holder asleep -----------------------------------------------------
def asleep(run):
    r, w = result(run, "chromium", "mp4"), window(run, "chromium", "mp4")
    if not r or not w:
        return None
    fetched = [x for x in ben if x["msg"] == "fetched, verified" and w[0] <= x["t"] <= w[1]]
    gave_nothing = [x for x in ben if x["msg"] == "a holder gave nothing of the range" and w[0] <= x["t"] <= w[1]]
    after = jload(f"{run}-ben-folders.json") or {}
    return {"playing_ms": (r.get("first") or {}).get("playing_ms"), "first": r.get("first"),
            "fetched_from": sorted({WHO.get(x["holder"], x["holder"]) for x in fetched}),
            "gave_nothing": [(WHO.get(x["holder"], x["holder"]), x.get("why")) for x in gave_nothing][:6],
            "listed_folders_of": sorted({f["person"] for f in after.get("folders", [])}),
            "unanswered": [WHO.get(k, k) for k in after.get("unanswered", [])]}


data["B6_plant"], data["B6"] = asleep("B-plant"), asleep("B")
b6 = data["B6"]
if b6 is None:
    verdict["B6"] = "never-ran"
elif b6["playing_ms"] is None:
    verdict["B6"] = "failed"
else:
    ok = (b6["playing_ms"] <= 5000 and "Cy" in b6["fetched_from"] and "Ana" in b6["unanswered"]
          and "Ana" in b6["listed_folders_of"])
    verdict["B6"] = "passed" if ok else "failed"


# --- B1: verified, or refused by name ----------------------------------------
def b3sum(path, start, n):
    with open(path, "rb") as f:
        f.seek(start)
        return blake3.blake3(f.read(n)).hexdigest()


every = [r for r in ben if r["msg"] == "served"]
unmatched = [r for r in every if r["sent"] and (r["hash"] not in source or
             b3sum(source[r["hash"]], r["start"], r["sent"]) != r["blake3"])]
data["B1_served_ranges"] = {"answers": len(every), "bytes": sum(r["sent"] for r in every),
                            "not_the_source_s_bytes": [{k: r.get(k) for k in ("t", "hash", "start", "sent")}
                                                       for r in unmatched]}
dspan = phase_span("D")
dr = result("D", "chromium", "mp4")
if dspan and dr:
    refusals = [r for r in ben if r["msg"].startswith("refused — no holder") and dspan[0] <= r["t"] <= dspan[1]]
    named_ana = [r for r in refusals if KEY.get("Ana", "?") in r.get("why", "")]
    flip = int(next((e["event"].split(" at ")[1].split(",")[0] for e in events
                     if "flipped at" in e["event"]), "500000"))
    d_served = [r for r in served(MP4, dspan[0], dspan[1]) if r["start"] <= flip < r["start"] + r["sent"]]
    played = any(e["e"] == "playing" for e in dr.get("events", []))
    data["B1"] = {"refusals": len(refusals), "naming_ana": len(named_ana),
                  "why": [r.get("why") for r in refusals][:3],
                  "statuses_to_browser": [n["url"][-40:] for n in dr.get("net", [])][:0],
                  "served_the_flipped_byte": len(d_served), "browser_played": played,
                  "browser_error": (dr.get("video") or {}).get("error"), "first": dr.get("first"),
                  "events": [e["e"] for e in dr.get("events", [])][:20]}
    ok = named_ana and not d_served and not played and not unmatched
    verdict["B1"] = "passed" if ok else "failed"
else:
    verdict["B1"] = "never-ran"
plant_range = rd("D-plant-range.jsonl")
dp = result("D-plant", "chromium", "mp4")
if dp:
    flip = 500000
    covering = [r for r in plant_range if r["status"] == 206 and r["content_range"] and
                int(r["content_range"].split()[1].split("-")[0]) <= flip <
                int(r["content_range"].split()[1].split("-")[0]) + r["bytes"]]
    data["B1_plant"] = {"requests_serving_the_flipped_byte": len(covering),
                        "uas": sorted({r["ua"] for r in covering}),
                        "browser_played": any(e["e"] == "playing" for e in dp.get("events", [])),
                        "first": dp.get("first")}


# --- B7: only what is shared leaves a holder ---------------------------------
def stop(run):
    t_ret, t_drop = event(run, "unshare returned"), event(run, "ben's list no longer")
    if t_ret is None:
        return None
    t_end = phase_span(run)[1]
    probes = rd(f"{run}-probe.jsonl")
    after = [p for p in probes if p["t"] >= t_ret]
    mine = [r for r in logs["ana"] if r.get("hash") == MKV and t_ret <= r["t"] < t_end]
    served_after = [r for r in mine if r["msg"].startswith("serving")]
    refused_after = [r for r in mine if r["msg"].startswith("refused")]
    return {"ben_list_drop_s": round(t_drop - t_ret, 3) if t_drop else None,
            "probe_statuses_after": [(round(p["t"] - t_ret, 3), p["status"]) for p in after][:40],
            "ana_served_mkv_after_s": [round(r["t"] - t_ret, 3) for r in served_after][:10],
            "ana_refused_mkv_after_s": [round(r["t"] - t_ret, 3) for r in refused_after][:10]}


data["B7_stop_plant"], data["B7_stop"] = stop("C-plant"), stop("C")
# Ben's data dir as it stood before the plant P (run.sh), and P's store.
data["B7_scan"] = jload("scan-ben.json")
plant_store = os.path.join(W, "ben", "blobs-P-plant")
data["B7_scan_plant"] = scan(plant_store, PLANTED) if os.path.isdir(plant_store) else None
share_a = jload("ana-share.json") or {}
share_p = jload("P-plant-share.json") or {}
planted_asked = [r for r in logs["ana"] if r.get("hash") == PLANTED and r["msg"].startswith("serving")
                 and r["t"] < (event("P-plant", "ana shared again") or float("inf"))]
data["B7"] = {"act_names": [f["path"] for f in share_a.get("files", [])],
              "plant_act_names": [f["path"] for f in share_p.get("files", [])],
              "planted_served_by_ana_before_the_plant": len(planted_asked),
              "planted_bytes_served_by_ben": len([r for r in every if r["hash"] == PLANTED
                                                  and r["t"] < (event("P-plant", "ana shared again") or float("inf"))])}
s7, sc = data.get("B7_stop"), data.get("B7_scan")
if not s7 or not sc:
    verdict["B7"] = "never-ran"
else:
    beside_ok = ("link.bin" not in data["B7"]["act_names"] and not sc["hits"]
                 and not data["B7"]["planted_served_by_ana_before_the_plant"]
                 and not data["B7"]["planted_bytes_served_by_ben"])
    stop_ok = not s7["ana_served_mkv_after_s"]
    seen = bool(s7["ana_refused_mkv_after_s"])
    verdict["B7"] = ("failed" if not (beside_ok and stop_ok) else "passed" if seen else "could-not-judge")
    verdict["B7_note"] = ("the stop's refusal at Ana's node was seen" if seen else
                          "no ask reached Ana's node between her stop and Ben's record dropping it")

summary = {"verdict": verdict, "data": data, "keys": KEY, "media": list(media.values()),
           "jellyfin_version": state.get("version")}
json.dump(summary, open(os.path.join(OUT, "summary.json"), "w"), indent=1, default=str)
for bar in ("B1", "B2", "B3", "B4", "B5", "B6", "B7"):
    print(f"{bar}: {verdict.get(bar)}")
print(json.dumps(data, indent=1, default=str)[:20000])
