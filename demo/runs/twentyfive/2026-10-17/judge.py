#!/usr/bin/env python3
"""The 25-computer run's verdicts: reads <data>/summary.json, judges each
line of PREDICTIONS.md §2 with AGENTS.md 6's four verdicts, writes
<data>/verdicts.json and prints a table.

    python3 judge.py <data dir>

Each bound below is its PREDICTIONS.md line's number, in one place.
"""
import ipaddress
import json
import statistics
import sys
from pathlib import Path

JOIN_EACH_S, JOIN_MEDIAN_S = 10, 3                       # line 1
LIT_S = 15                                               # line 2
LEND_S, LEND_MEDIAN_S = 5, 2                             # line 3
SETTLE_S = {"burst": 30, "stagger": 15}                  # line 4
READ_P50_MS, READ_P95_MS, READ_CAP_MS = 400, 1500, 3000  # line 5
MATRIX_S = 60                                            # line 6
VISITS_EACH, VISIT_SAME_P50_MS, VISIT_FAR_P50_MS = 3, 100, 200  # line 7
CPU_FOUNDER_STEADY, CPU_OTHER_STEADY, RSS_MIB, CPU_FOUNDER_BURST = 25, 5, 80, 100  # line 8

PASSED, FAILED, CNJ, NEVER = "passed", "failed", "could-not-judge", "never-ran"


def pct(xs, p):
    xs = sorted(xs)
    if not xs:
        return None
    return xs[min(len(xs) - 1, int(round(p / 100 * (len(xs) - 1))))]


def r1(x):
    return None if x is None else round(x, 2)


def main(data):
    s = json.loads((Path(data) / "summary.json").read_text())
    v = {}
    if s.get("pre", {}).get("never_ran"):
        why = s["pre"]["never_ran"]
        for i in range(10):
            v[str(i)] = {"verdict": NEVER, "why": why}
        return finish(data, v)
    v["0"] = {"verdict": PASSED, "n0": {k: x for k, x in s["pre"].items() if k.startswith("n0_from")},
              "founded": s.get("founded")}
    plan = s.get("plan") or []
    founder = next((p["name"] for p in plan if p.get("founder")), None)
    joiners = [p["name"] for p in plan if not p.get("founder")]
    machine = {p["name"]: p["machine"] for p in plan}
    ev = [e for k in s if k.startswith("events_") for e in s[k]]
    first = lambda kind, who: min((e["t"] for e in ev if e["event"] == kind and e.get("who") == who), default=None)  # noqa: E731
    start = {w: first("join-start", w) for w in joiners}
    joined = {w: first("joined", w) for w in joiners}
    shared = {w: min((e["t"] for e in ev if e["event"] == "shared" and e.get("who") == w and e.get("code") == 0), default=None)
              for w in [founder] + joiners}
    wall = s.get("wall") or []

    # 1. Every join gets in.
    if not ev:
        v["1"] = {"verdict": CNJ, "why": "no fleet events came back"}
    else:
        took = {w: joined[w] - start[w] for w in joiners if joined[w] is not None and start[w] is not None}
        out = sorted(set(joiners) - set(took))
        med = statistics.median(took.values()) if took else None
        ok = not out and max(took.values()) <= JOIN_EACH_S and med <= JOIN_MEDIAN_S
        v["1"] = {"verdict": PASSED if ok else FAILED, "in": len(took), "of": len(joiners), "not_in": out,
                  "median_s": r1(med), "max_s": r1(max(took.values()) if took else None),
                  "each_s": {w: r1(t) for w, t in sorted(took.items(), key=lambda x: x[1])}}

    def first_on_wall(key, who):
        return min((l["t"] for l in wall if who in (l.get(key) or [])), default=None)

    # 2. Each person lit on the wall.
    if not wall:
        v["2"] = {"verdict": CNJ, "why": "no wall reads came back"}
        v["3"] = dict(v["2"])
    else:
        lit = {w: first_on_wall("lit", w) for w in joiners}
        took = {w: lit[w] - start[w] for w in joiners if lit[w] is not None and start[w] is not None}
        never = sorted(w for w in joiners if lit[w] is None)
        ok = not never and took and max(took.values()) <= LIT_S
        v["2"] = {"verdict": PASSED if ok else FAILED, "never_lit": never,
                  "median_s": r1(statistics.median(took.values()) if took else None),
                  "max_s": r1(max(took.values()) if took else None)}
        # 3. Each lend on the wall, from its share returning.
        listed = {w: first_on_wall("listed", w) for w in [founder] + joiners}
        took = {w: max(0.0, listed[w] - shared[w]) for w in listed if listed[w] is not None and shared[w] is not None}
        never = sorted(w for w in [founder] + joiners if listed[w] is None)
        unshared = sorted(w for w in [founder] + joiners if shared[w] is None)
        med = statistics.median(took.values()) if took else None
        ok = not never and not unshared and max(took.values()) <= LEND_S and med <= LEND_MEDIAN_S
        v["3"] = {"verdict": PASSED if ok else FAILED, "never_listed": never, "share_failed": unshared,
                  "median_s": r1(med), "max_s": r1(max(took.values()) if took else None)}

    # 4. The wall settles.
    settled = next((l["t"] for l in wall if l.get("settled")), None)
    last_start = max((t for t in start.values() if t is not None), default=None)
    if not wall or last_start is None:
        v["4"] = {"verdict": CNJ, "why": "no wall reads or no join started"}
    elif settled is None:
        end = wall[-1]
        v["4"] = {"verdict": FAILED, "why": "never settled within the wall's ceiling",
                  "last_read": {k: end.get(k) for k in ("lit", "listed", "unanswered", "unheard", "refused", "older") if k in end}}
    else:
        dt = settled - last_start
        v["4"] = {"verdict": PASSED if dt <= SETTLE_S[s["mode"]] else FAILED, "after_last_join_s": r1(dt),
                  "bound_s": SETTLE_S[s["mode"]]}

    # 5. Wall reads once settled.
    after = [l["lends_ms"] for l in wall if settled and l["t"] >= settled and l.get("lends_status") == 200]
    if not after:
        v["5"] = {"verdict": CNJ, "why": "no wall read after settling"}
    else:
        p50, p95, mx = pct(after, 50), pct(after, 95), max(after)
        ok = p50 <= READ_P50_MS and p95 <= READ_P95_MS and mx < READ_CAP_MS
        v["5"] = {"verdict": PASSED if ok else FAILED, "reads": len(after), "p50_ms": p50, "p95_ms": p95, "max_ms": mx,
                  "all_reads": len(wall), "p50_ms_whole_run": pct([l["lends_ms"] for l in wall], 50)}

    # 6. Every computer sees the whole group.
    rounds = s.get("matrix") or []
    last_share = max((t for t in shared.values() if t is not None), default=None)
    if not rounds:
        v["6"] = {"verdict": CNJ, "why": "no matrix probe came back"}
    else:
        whole = next((r for r in rounds if r["whole"]), None)
        short = [{"who": r["who"], "lent": len(r.get("lent") or []), "people": len(r.get("people") or []),
                  **{k: r.get(k) for k in ("unanswered", "unheard", "refused", "older") if r.get(k)}}
                 for r in rounds[-1]["rows"] if len(r.get("lent") or []) != len(plan) or any(
                     r.get(k) for k in ("unanswered", "unheard", "refused", "older"))]
        if whole and last_share is not None:
            dt = whole["t"] - last_share
            v["6"] = {"verdict": PASSED if dt <= MATRIX_S else FAILED, "after_last_share_s": r1(dt), "rounds": len(rounds)}
        else:
            v["6"] = {"verdict": FAILED, "why": "some computer still short at the ceiling", "short": short, "rounds": len(rounds)}

    # 7. Every greeting names the right person.
    visits = [dict(x, by=r["who"]) for r in s.get("visits") or [] for x in r.get("visits") or []]
    if not visits:
        v["7"] = {"verdict": CNJ, "why": "no visit was made"}
    else:
        wrong = [x for x in visits if not x["right"]]
        same = [x["ms"] for x in visits if not x["far"] and x["status"] == 200]
        far = [x["ms"] for x in visits if x["far"] and x["status"] == 200]
        want = VISITS_EACH * len(plan)
        ok = not wrong and len(visits) == want and (not same or pct(same, 50) <= VISIT_SAME_P50_MS) \
            and (not far or pct(far, 50) <= VISIT_FAR_P50_MS)
        v["7"] = {"verdict": PASSED if ok else FAILED, "visits": len(visits), "wanted": want, "right": len(visits) - len(wrong),
                  "wrong": [{k: x.get(k) for k in ("by", "to", "status", "owner", "person_said", "error")} for x in wrong][:20],
                  "same_p50_ms": pct(same, 50), "far_p50_ms": pct(far, 50)}

    # 8. CPU and memory.
    windows = [(k.split("_", 1)[1], w) for k in s if k.startswith("samples_") for w in s[k]]
    if not windows or settled is None:
        v["8"] = {"verdict": CNJ, "why": "no samples, or the wall never settled"}
    else:
        first_start = min((t for t in start.values() if t is not None), default=settled)
        last_in = max((t for t in joined.values() if t is not None), default=settled)
        steady = [(m, row) for m, w in windows if w["t0"] >= settled for row in w["rows"]]
        burst = [(m, row) for m, w in windows if w["t1"] >= first_start and w["t0"] <= last_in for row in w["rows"]]
        if not steady:
            v["8"] = {"verdict": CNJ, "why": "no sample window after settling"}
        else:
            f_steady = max((r["cpu_pct"] for _, r in steady if r["who"] == founder), default=None)
            o_steady = max((r["cpu_pct"] for _, r in steady if r["who"] != founder), default=None)
            f_burst = max((r["cpu_pct"] for _, r in burst if r["who"] == founder), default=None)
            rss = max(r["rss_mib"] for _, r in steady + burst)
            ok = (f_steady is not None and f_steady <= CPU_FOUNDER_STEADY and (o_steady or 0) <= CPU_OTHER_STEADY
                  and rss <= RSS_MIB and (f_burst is None or f_burst <= CPU_FOUNDER_BURST))
            loads = {m: max(w["load"] for mm, w in windows if mm == m) for m, _ in windows}
            v["8"] = {"verdict": PASSED if ok else FAILED, "founder_steady_pct": f_steady, "other_steady_max_pct": o_steady,
                      "founder_burst_pct": f_burst, "rss_max_mib": rss, "load_max": loads}

    # 9. Mac to Fedora.
    if set(machine.values()) != {"mac", "linux"}:
        v["9"] = {"verdict": NEVER, "why": "one machine only: no pair across two"}
    else:
        keys = {}
        for l in wall:
            keys.update(l.get("keys") or {})
        short = {k[:10]: p for k, p in keys.items()}
        rows = [r for m in (s.get("paths") or {}).values() for r in m]
        if not rows:
            v["9"] = {"verdict": CNJ, "why": "no path::selected line in any log"}
        else:
            last = {}
            for r in rows:
                to = short.get(r["remote"])
                if to and machine.get(r["who"]) and machine.get(to) and machine[r["who"]] != machine[to]:
                    last[(r["who"], to)] = r["path"]
            pairs = [(a, b) for a in machine for b in machine if machine[a] != machine[b]]
            missing = [f"{a}->{b}" for a, b in pairs if (a, b) not in last]
            kinds = {}
            for p in last.values():
                kinds[kind(p)] = kinds.get(kind(p), 0) + 1
            v["9"] = {"verdict": PASSED if not missing else FAILED, "pairs": len(pairs), "with_a_path": len(last),
                      "missing": missing[:20], "kinds": kinds,
                      "note": "tailscale is direct for reach only; the venue has none"}
    return finish(data, v)


def kind(path):
    if path.startswith("Relay("):
        return "relay"
    if path.startswith("Ip("):
        remote = path[3:-1].split("->")[-1]
        host = remote.rsplit(":", 1)[0].strip("[]")
        try:
            ip = ipaddress.ip_address(host)
        except ValueError:
            return "ip?"
        if ip.is_loopback:
            return "loopback"
        if ip in ipaddress.ip_network("100.64.0.0/10"):
            return "tailscale"
        if ip.is_private:
            return "lan"
        return "public"
    return "other"


def finish(data, v):
    (Path(data) / "verdicts.json").write_text(json.dumps(v, indent=1))
    names = ["preconditions", "every join in", "lit on the wall", "lend on the wall", "wall settles",
             "wall reads settled", "matrix whole", "greetings right", "cpu and memory", "mac to fedora"]
    for i, name in enumerate(names):
        x = v.get(str(i), {"verdict": NEVER})
        detail = {k: y for k, y in x.items() if k not in ("verdict", "each_s", "wrong", "short", "missing", "n0", "founded")}
        print(f"{i}. {name:20s} {x['verdict']:16s} {json.dumps(detail)[:160]}")
    return v


if __name__ == "__main__":
    main(sys.argv[1])
