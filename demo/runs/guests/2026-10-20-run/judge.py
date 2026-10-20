#!/usr/bin/env python3
"""The verdicts of a take of run.py here (PREDICTIONS.md §4), one a
prediction, each passed, failed, could-not-judge or never-ran (AGENTS.md 6),
with what it read. A step that did not run, or a reading that is absent, is
could-not-judge, never a pass.

    python3 demo/runs/guests/2026-10-20-run/judge.py <a take's summary.json>   # its verdicts.json beside it"""
import json, re, sys
from pathlib import Path

PASSED, FAILED, CANNOT, NEVER = "passed", "failed", "could-not-judge", "never-ran"
HOURS = 3600
WORDS = re.compile(r"\b(record|key|fold|standing|signer|actor)s?\b", re.I)
REFUSED = "This link lets nobody in. Ask for a new one."

path = Path(sys.argv[1])
s = json.loads(path.read_text())
steps = s.get("steps", {})


def step(k): return steps.get(k)


def ran(*ks): return all(step(k) and not step(k).get("failed") and not step(k).get("error") for k in ks)


def text(k):
    """What the page said at the end of step `k`."""
    st = step(k) or {}
    return (st.get("done") or [{}])[-1].get("text", "") if "done" in st else st.get("text", "")


def finished(k):
    """When step `k` ended, in unix seconds."""
    st = step(k) or {}
    return st["started"] + st.get("ms", 0) / 1000 if st else None


def verdict(ok, cannot=False):
    return CANNOT if cannot else PASSED if ok else FAILED


v = {}
written = {w["id"]: w for ws in (s.get("visits_written") or {}).values() for w in ws if "id" in w}
heads = s.get("heads") or {}

# 1. Kim makes a link at the 48 hours it starts at.
link = s.get("link_1") or {}
one = (step("1") or {}).get("done") or []
made = next((d["ms"] - one[i - 1]["ms"] for i, d in enumerate(one) if "Make a link" in json.dumps(d["step"]) and i), None)
v[1] = {"verdict": verdict(link.get("length") == {"chosen": "48 hours", "offered": ["48 hours", "24 hours", "6 hours"]}
                           and link.get("on_page") and made is not None and made <= 5000, cannot=not ran("1")),
        "length": link.get("length"), "on_page": link.get("on_page"), "made_ms": made}

# 2. Sam watches Kim's film within 5 minutes, told as Kim's guest.
sam = s.get("sam") or {}
visit = sam.get("visit") or {}
key = (written.get(visit.get("id")) or {}).get("key")
kim = [h for h in heads.get("kim", []) if key and h.get("x-svrngs-key") == key]
length = visit["until"] - visit["at"] if visit.get("until") and visit.get("at") else None
told = sorted({h.get("x-svrngs-person") for h in kim})
ok2 = (ran("2-visit", "2-watch") and step("2-visit").get("asked") and not step("2-visit").get("askedAgain")
       and step("2-visit").get("status", "").startswith("You’re Kim’s guest until") and step("2-visit").get("listed") == ["Kim’s jellyfin"]
       and length is not None and 48 * HOURS - 60 <= length <= 48 * HOURS and sam.get("film_ms") is not None and sam["film_ms"] <= 300_000
       and told == ["Sam (Kim's guest)"] and all(h.get("x-svrngs-key") != (s.get("computers") or {}).get("Kim") and (h.get("x-svrngs-user") or "").endswith(".guest") for h in kim))
v[2] = {"verdict": verdict(ok2, cannot=not (step("2-visit") and step("2-watch") and key)), "status": (step("2-visit") or {}).get("status"),
        "listed": (step("2-visit") or {}).get("listed"), "length_s": length, "film_ms": sam.get("film_ms"), "in_ms": sam.get("in_ms"),
        "opened_ms": sam.get("opened_ms"), "played": sam.get("played"), "requests": len(kim), "told": told,
        "users": sorted({h.get("x-svrngs-user") for h in kim})}

# 3. Bo's Jellyfin refuses Sam, said which fact.
forge = step("3-forge") or {}
bo_sam = [h for h in heads.get("bo", []) if key and h.get("x-svrngs-key") == key]
# The page begins the door's words with a capital: "Jellyfin is not …".
v[3] = {"verdict": verdict("jellyfin is not kim's to open to you" in forge.get("text", "").lower() and forge.get("video") == 0 and not bo_sam
                           and "Bo" not in " ".join((step("2-visit") or {}).get("listed") or []),
                           cannot=s.get("bo_holds_sam_ms") is None or not forge or forge.get("error")),
        "bo_holds_sam_ms": s.get("bo_holds_sam_ms"), "said": forge.get("text", "")[:300], "video": forge.get("video"), "bo_requests": len(bo_sam)}

# 4. Every home page shows Sam visiting, within 30 s, Stop on Kim's alone, one `until`.
homes4 = {w: text(f"4-{w}") for w in ("ana", "kim", "bo")}
untils = sam.get("until_on") or {}
v[4] = {"verdict": verdict(s.get("home_4_ms") is not None and s["home_4_ms"] <= 30_000 and "Stop Sam’s visit" in homes4["kim"]
                           and not any("Stop Sam’s visit" in homes4[w] for w in ("ana", "bo")) and len(set(untils.values())) == 1 and None not in untils.values(),
                           cannot=not ran("4-ana", "4-kim", "4-bo")),
        "all_ms": s.get("home_4_ms"), "until_on": untils,
        "said": {w: next((l for l in t.splitlines() if l.startswith("Sam, Kim’s guest")), None) for w, t in homes4.items()}}

# 5. Kim's visit until tonight, made like the first.
jo = s.get("jo") or {}
jv = jo.get("visit") or {}
jlen = jv["until"] - jv["at"] if jv.get("until") and jv.get("at") else None
offered = next((d.get("value") for d in (step("5") or {}).get("done", []) if "options" in d["step"]), None) or {}
late = [round(finished(f"5-{w}") - jo["in"], 1) for w in ("ana", "kim", "bo") if finished(f"5-{w}") and jo.get("in")]
v[5] = {"verdict": verdict("6 hours" in (offered.get("offered") or []) and (step("5-visit") or {}).get("status", "").startswith("You’re Kim’s guest until")
                           and jlen is not None and 6 * HOURS - 60 <= jlen <= 6 * HOURS and ((step("5-watch") or {}).get("played") or {}).get("ms") is not None
                           and ran("5-ana", "5-kim", "5-bo") and len(late) == 3 and max(late) <= 30,
                           cannot=not (step("5-visit") and step("5-watch"))),
        "offered": offered, "length_s": jlen, "played": (step("5-watch") or {}).get("played"), "home_s": dict(zip(("ana", "kim", "bo"), late))}

# 6. A third guest is a guest on no copy, then or later.
mx = s.get("max") or {}
later = [k for k in steps if re.match(r"^[6789]-(ana|kim|bo)$", k)]
shown = sorted(k for k in later if "Max, Ana’s guest" in text(k))
listed = {"at 6": mx.get("visiting"), "at 7": {w: r.get("on") for w, r in (s.get("stop_7") or {}).items()},
          "at 9": (s.get("lee") or {}).get("visiting"), "at the end": mx.get("visiting_end")}
on = sorted(f"{at} {w}" for at, ws in listed.items() for w, gs in (ws or {}).items() if "Max" in (gs or []))
held = mx.get("held") or {}
v[6] = {"verdict": verdict((step("6-visit") or {}).get("status") == REFUSED and (step("6-visit") or {}).get("listed") == []
                           and (mx.get("written") or {}).get("counted") == "false" and all(held.get(w) is not None and held[w] <= 30_000 for w in ("ana", "kim", "bo"))
                           and not on and not shown,
                           cannot=s.get("ana_holds_jo_ms") is None or not step("6-visit")),
        "ana_holds_jo_ms": s.get("ana_holds_jo_ms"), "status": (step("6-visit") or {}).get("status"), "written": mx.get("written"),
        "held_ms": held, "listed_max": on, "pages_showing_max": shown}

# 7. Kim's Stop ends Sam's visit on every copy, within 30 s.
stop = s.get("stop_7") or {}
v[7] = {"verdict": verdict(all((stop.get(w) or {}).get("ms") is not None and stop[w]["ms"] <= 30_000 for w in ("ana", "kim", "bo"))
                           and all("Jo" in (stop[w].get("on") or []) and "Max" not in (stop[w].get("on") or []) for w in stop)
                           and not any("Sam, Kim’s guest" in text(f"7-{w}") for w in ("ana", "kim", "bo")) and "Kim ended your visit" in text("7-sam"),
                           cannot=not ran("7") or not stop),
        "ms": {w: r.get("ms") for w, r in stop.items()}, "on": {w: r.get("on") for w, r in stop.items()}, "sam_said": text("7-sam")[:200]}

# 8. Tonight's visit ends then, on every copy.
tn = s.get("tonight") or {}
if not tn.get("ran"):
    v[8] = {"verdict": NEVER, "why": "TRIAL" if s.get("trial") else "no until for Jo's visit"}
else:
    ended = tn.get("ended") or {}
    v[8] = {"verdict": verdict(all((ended.get(w) or {}).get("ms") is not None and ended[w]["ms"] <= 60_000 for w in ("ana", "kim", "bo"))
                               and not any("Jo, Kim’s guest" in text(f"8-{w}") for w in ("ana", "kim", "bo"))
                               and "your visit with Kim is over: its time is up" in text("8-jo"), cannot=not ran("8-jo")),
            "until": tn.get("until"), "ms": {w: r.get("ms") for w, r in ended.items()}, "on": {w: r.get("on") for w, r in ended.items()}, "jo_said": text("8-jo")[:200]}

# 9. Kim's leave ends Lee's visit on every copy, within 30 s.
lv = s.get("leave_9") or {}
v[9] = {"verdict": verdict((step("9-visit") or {}).get("status", "").startswith("You’re Kim’s guest until") and ((step("9-watch") or {}).get("played") or {}).get("ms") is not None
                           and all((lv.get(w) or {}).get("ms") is not None and lv[w]["ms"] <= 30_000 for w in ("ana", "bo"))
                           and not any("Visiting now" in text(f"9-{w}") for w in ("ana", "bo"))
                           and "Kim is no longer a member, so your visit ended" in text("9-lee"),
                           cannot=not ran("9-leave") or not step("9-lee")),
        "ms": {w: r.get("ms") for w, r in lv.items()}, "on": {w: r.get("on") for w, r in lv.items()}, "lee_said": text("9-lee")[:200],
        "visiting_said": {w: [l for l in text(f"9-{w}").splitlines() if "’s guest, until" in l] for w in ("ana", "bo")}}

# 10. Words, on every page svrngs draws (the stand-in's page says nothing).
reads = []
for k, st in steps.items():
    for d in st.get("done") or [{"text": st.get("text", "")}]:
        reads.append((k, d.get("text", "")))
said = sorted({f"{k}: {m.group(0)}" for k, t in reads for m in WORDS.finditer(t)})
v[10] = {"verdict": verdict(not said, cannot=not reads), "reads": len(reads), "said": said}

# 11. Every page drawn, every step finished, no page error.
failed = sorted(k for k, st in steps.items() if st.get("failed") or st.get("error"))
errors = sorted({e for st in steps.values() for e in st.get("errors") or []})
v[11] = {"verdict": verdict(not failed and not errors, cannot=not steps), "steps": len(steps), "failed": failed, "errors": errors}

# 12. A visit with real people: the operator's.
v[12] = {"verdict": NEVER, "why": "the operator's measure, set before it (STATE.md, \"The measure\")"}

(path.parent / "verdicts.json").write_text(json.dumps(v, indent=1, ensure_ascii=False) + "\n")
for n, r in v.items(): print(n, r["verdict"])
