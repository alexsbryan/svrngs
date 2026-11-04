#!/usr/bin/env python3
"""Judge oidc-jellyfin's five bars from the run's raw evidence (AGENTS.md 6).

    judge.py <data dir>

Reads <data>/raw.json, writes <data>/verdicts.json, prints one line a bar.
A bar whose evidence did not happen is could-not-judge, never a pass.
"""
import json
import sys
from pathlib import Path

data = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
raw = json.loads((data / "raw.json").read_text()) if (data / "raw.json").exists() else {}
people = raw.get("people", {})
verdicts = {}


def bar(name, verdict, why):
    verdicts[name] = {"verdict": verdict, "why": why}
    print(f"{name:<8} {verdict:<16} {why}")


def member(who, account, engine=None):
    p = people.get(who)
    if not p:
        return "could-not-judge", f"{who}'s step is not in the data"
    missing = []
    if not p.get("sso_offered"):
        missing.append("the login page did not offer the lender's SSO button")
    if p.get("signed_in_ms") is None:
        missing.append(f"no sign-in: {p.get('stopped') or p.get('home') or p.get('url')}")
    if not (p.get("jellyfin_name") or "").startswith(account):
        missing.append(f"the account is {p.get('jellyfin_name')!r}, not {account}*")
    if p.get("played_ms") is None:
        missing.append(f"the film did not play: {p.get('why')}")
    if engine and engine not in (p.get("engine") or ""):
        missing.append(f"the engine is {p.get('engine')!r}, not {engine}")
    if missing:
        return "failed", "; ".join(missing)
    return "passed", (
        f"sign-in {p['signed_in_ms']} ms after the press, no typing, "
        f"as {p['jellyfin_name']}, film in {p['played_ms']} ms"
    )


bar("alex", *member("alex", "Alex-", engine="chromium"))
bar("bo", *member("bo", "Bo-", engine="webkit"))

apart = raw.get("bars", {}).get("apart")
if not apart:
    bar("apart", "could-not-judge", "no two-person evidence in the data")
else:
    alex, bo = people.get("alex", {}), people.get("bo", {})
    good = (
        apart.get("distinct_users")
        and apart.get("distinct_tokens")
        and apart.get("alex") == alex.get("jellyfin_name")
        and apart.get("bo") == bo.get("jellyfin_name")
        and apart.get("alex_me") == alex.get("jellyfin_name")
        and apart.get("bo_me") == bo.get("jellyfin_name")
        and alex.get("jellyfin_name") != bo.get("jellyfin_name")
    )
    why = (
        f"Alex {apart.get('alex')} ({apart.get('alex_me')} by token), "
        f"Bo {apart.get('bo')} ({apart.get('bo_me')} by token); "
        f"distinct users {apart.get('distinct_users')}, distinct tokens {apart.get('distinct_tokens')}"
    )
    bar("apart", "passed" if good else "failed", why)

cy = people.get("cy")
if not cy:
    bar("removed", "could-not-judge", "Cy's step is not in the data")
else:
    answers_before = (raw.get("cy_before") or {}).get("kim_authorize_answers")
    answers_after = cy.get("kim_authorize_answers_after")
    new_cy = [n for n in cy.get("new_users", []) if str(n).startswith("Cy-")]
    missing = []
    if cy.get("sso_offered"):
        missing.append("Jellyfin's login page was offered")
    if not (cy.get("words") or "").strip():
        missing.append("the refusal said nothing")
    if new_cy:
        missing.append(f"Jellyfin made {new_cy}")
    if answers_before is not None and answers_after is not None and answers_after != answers_before:
        missing.append(f"the door minted {answers_after - answers_before} codes for Cy")
    bar(
        "removed",
        "failed" if missing else "passed",
        "; ".join(missing) if missing
        else f"refused in the door's words: {cy.get('words', '')[:160]!r}; "
             f"no Cy- user; code minted: {answers_after != answers_before}",
    )

planted = raw.get("bars", {}).get("planted")
if not planted:
    bar("planted", "could-not-judge", "the planted ask is not in the data")
else:
    claims = planted.get("claims") or {}
    good = (
        planted.get("first_status") == 200
        and str(claims.get("preferred_username", "")).startswith("Bo-")
        and claims.get("name") == "Bo"
        and not str(claims.get("preferred_username", "")).startswith("Alex-")
        and planted.get("second_status") == 400
        and planted.get("second_error") == "invalid_grant"
    )
    bar(
        "planted",
        "passed" if good else "failed",
        (f"the code named {claims.get('preferred_username')!r} ({claims.get('name')!r}), "
         f"not Alex; a second redemption answered {planted.get('second_status')} "
         f"{planted.get('second_error')}")
        if good
        else (f"captured {planted.get('captured')!r}; claims {claims!r}; "
              f"second {planted.get('second_status')} {planted.get('second_error')}; "
              f"{planted.get('stopped', '')}"),
    )

(data / "verdicts.json").write_text(json.dumps(verdicts, indent=2) + "\n")
sys.exit(0 if all(v["verdict"] == "passed" for v in verdicts.values()) else 1)
