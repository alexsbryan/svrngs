#!/usr/bin/env python3
"""Judge oidc-forgejo's five bars from the run's raw evidence (AGENTS.md 6).

    judge.py <data dir>

Reads <data>/raw.json, writes <data>/verdicts.json, prints one line a bar.
A bar whose evidence did not happen is could-not-judge, never a pass.
"""
import json
import re
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
    login = (p.get("user") or {}).get("login") or ""
    missing = []
    if not p.get("sso_offered"):
        missing.append("the login page did not offer the lender's svrngs source")
    if p.get("signed_in_ms") is None:
        missing.append(f"no sign-in: {p.get('stopped') or p.get('page') or p.get('url')}")
    if not re.fullmatch(rf"{account}-[0-9a-f]{{16}}", login, re.I):
        missing.append(f"the account is {login!r}, not {account}-<16 hex>")
    if p.get("stayed") is not True:
        missing.append(f"the browser left the lend's origin: {p.get('url')!r}")
    if p.get("typed") is not False or p.get("local_login_posts"):
        missing.append(f"the local sign-in form was used ({p.get('local_login_posts')} posts)")
    if engine and engine not in (p.get("engine") or ""):
        missing.append(f"the engine is {p.get('engine')!r}, not {engine}")
    if missing:
        return "failed", "; ".join(missing)
    return "passed", (
        f"sign-in {p['signed_in_ms']} ms after the press, no typing, "
        f"as {login}, on the lend's own origin"
    )


bar("alex", *member("alex", "alex", engine="chromium"))
bar("bo", *member("bo", "bo", engine="webkit"))

apart = raw.get("bars", {}).get("apart")
users = raw.get("forgejo", {}).get("users")
if not apart or not isinstance(users, list):
    bar("apart", "could-not-judge", "no two-person evidence in the data")
else:
    logins = {u.get("login") for u in users}
    good = (
        apart.get("distinct_users")
        and apart.get("distinct_logins")
        and apart.get("alex") in logins
        and apart.get("bo") in logins
        and apart.get("alex") == (people.get("alex", {}).get("user") or {}).get("login")
        and apart.get("bo") == (people.get("bo", {}).get("user") or {}).get("login")
    )
    why = (
        f"Alex {apart.get('alex')}, Bo {apart.get('bo')}; "
        f"distinct users {apart.get('distinct_users')}, distinct logins {apart.get('distinct_logins')}; "
        f"Forgejo's own list holds both: {sorted(l for l in logins if l)}"
    )
    bar("apart", "passed" if good else "failed", why)

cy = people.get("cy")
if not cy:
    bar("removed", "could-not-judge", "Cy's step is not in the data")
else:
    answers_before = (raw.get("cy_before") or {}).get("kim_authorize_answers")
    answers_after = cy.get("kim_authorize_answers_after")
    new_cy = [n for n in (cy.get("new_users") or []) if str(n).lower().startswith("cy-")]
    missing = []
    if cy.get("sso_offered"):
        missing.append("Forgejo's login page was offered")
    if not (cy.get("words") or "").strip():
        missing.append("the refusal said nothing")
    if new_cy:
        missing.append(f"Forgejo made {new_cy}")
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
    username = str(claims.get("preferred_username", ""))
    good = (
        planted.get("first_status") == 200
        and username.lower().startswith("bo-")
        and claims.get("name") == "Bo"
        and not username.lower().startswith("alex-")
        and planted.get("second_status") == 400
        and planted.get("second_error") == "invalid_grant"
    )
    bar(
        "planted",
        "passed" if good else "failed",
        (f"the code named {username!r} ({claims.get('name')!r}), "
         f"not Alex; a second redemption answered {planted.get('second_status')} "
         f"{planted.get('second_error')}")
        if good
        else (f"captured {planted.get('captured')!r}; claims {claims!r}; "
              f"second {planted.get('second_status')} {planted.get('second_error')}; "
              f"{planted.get('stopped', '')}"),
    )

(data / "verdicts.json").write_text(json.dumps(verdicts, indent=2) + "\n")
sys.exit(0 if all(v["verdict"] == "passed" for v in verdicts.values()) else 1)
