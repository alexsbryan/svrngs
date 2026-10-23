#!/usr/bin/env python3
"""The evening review: the gates a queue's ready rows would hit tonight, asked in daylight.

    python3 preshift.py <queue> [--root DIR] [--state F] [--charter F] [--lean F] [--json]

<queue> is a directory under ralph/next/. For every ready row of its STATE.md
(an open `[ ]` or in-flight `[~]` row whose depends are all `[x]`, by ralph's
own grammar, commonwealth-ai scripts/ralph.py:57-60 and `State.current`), it
reports, each with its file:line:

1. fences: the row's words against the terms of CHARTER.md's reservations
   ("Leave for the operator", "What this queue does not decide");
2. ceiling headroom: lean_check.py's counts against the charter's budget
   fence, and the row's stated estimate against what is left; a row with no
   estimate is said as such, never given one;
3. premises: each path the row names exists (or, for one it makes, its
   parent does), each cited line is inside its file, each command's script
   exists, and no command the row names is on the worker's deny list
   (queue.toml `settings`);
4. operator-present rows: HUMAN- rows and rows that need people or the
   operator there.

Groups: "Needs you present", "Will park tonight (ask now)", "Could not
judge (why)", "Clear". Exit 0 no gate found, 3 any row to ask about now or
needing the operator present, 4 could-not-judge only, 2 never ran (no
STATE.md). A charter number is read as a lean_check cap: a count must stay
below it (scripts/lean_check.py `n >= cap` is broken), so what is left under
a ceiling C at count n is C - 1 - n.
"""
import argparse
import ast
import datetime
import fnmatch
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# ralph's row grammar, copied from commonwealth-ai scripts/ralph.py:57-60, never imported.
HASH_RE = re.compile(r"^[0-9a-f]{7,40}$")
ROW_RE = re.compile(r"^- \[(?P<mark>[x~ ])\] (?P<head>.*?) — depends \[(?P<deps>[^\]]*)\](?P<rest>.*)$")
LEAN_RE = re.compile(r"^(\S+)\s+(\d+) / (\d+) lines$")
SCOPE_NAME = {".": "repository", "crates/svrngs-core": "svrngs-core"}
GROUPS = [("present", "Needs you present (do not run tonight)"),
          ("ask-now", "Will park tonight (ask now)"),
          ("could-not-judge", "Could not judge (why)"),
          ("clear", "Clear")]
FENCE_HEAD = re.compile(r"^##\s+(Leave for the operator|What this queue does not decide)\b")
INLINE_FENCE = re.compile(r"^\*\*Leave for the operator\*\*")
# Words a fence item names that would match nearly any row; the item is then
# checked by its other terms, or said as not checked.
STOP_TERMS = {"git", "tools", "anything", "budget", "the budget", "any change", "change", "code"}
NEGATED_BEFORE = re.compile(r"\b(never|no|not|nor|without|unchanged|keeps?)\b[^.;]*$", re.I)
NEGATED_AFTER = re.compile(r"^[^.;]{0,25}\b(unchanged|waits?|stays?|never-ran|untouched)\b", re.I)
CONDITIONAL = re.compile(r"\b(if|unless|when|past|should|where|whose)\b", re.I)
PRESENT = [re.compile(p, re.I) for p in (
    r"\bthe operator (?:runs|holds|plays|presses|taps|watches|is there)\b", r"\boperator runs\b",
    r"\blive demo\b", r"\bwith (?:real )?people\b", r"\bpeople in the room\b", r"\bin person\b",
    r"\bthe operator present\b", r"\bwith the operator\b")]
COMMANDS = {"python3", "python", "node", "cargo", "npm", "npx", "git", "podman", "docker", "sudo",
            "sh", "bash", "curl", "uv"}
EXT = r"(?:rs|js|mjs|ts|py|sh|md|toml|json|txt|html|css|log|tsv|patch)"
BARE_PATH = re.compile(
    r"(?<![\w/.@#-])((?:~/)?[\w.-]+(?:/[\w.*<>-]+)+/?|[\w-]+\." + EXT + r")(?::(\d+)(?:-(\d+))?)?(?![\w/])")


def finding(kind, group, say, *evidence):
    return {"kind": kind, "group": group, "say": say, "evidence": [e for e in evidence if e]}


def where(path, root):
    p = Path(path).resolve()
    try:
        return p.relative_to(root).as_posix()
    except ValueError:
        return str(path)


# --- STATE.md -----------------------------------------------------------------

def parse_rows(state):
    lines = state.read_text().splitlines()
    rows = []
    for n, line in enumerate(lines, 1):
        m = ROW_RE.match(line)
        if not m:
            continue
        tokens = m.group("head").split()
        row_id = next((t for t in tokens if not HASH_RE.fullmatch(t)), None)
        commit = next((t for t in tokens if HASH_RE.fullmatch(t)), None)
        block = [line]
        for more in lines[n:]:  # ralph.py `State.block`: continuation up to the next row or heading
            if ROW_RE.match(more) or more.startswith("#"):
                break
            block.append(more)
        rows.append({"id": row_id, "mark": m.group("mark"), "commit": commit, "line": n,
                     "deps": [d.strip() for d in m.group("deps").split(",") if d.strip()],
                     "text": " ".join(s.strip() for s in block)})
    return rows


def segments(text):
    """The row's body (what it does and checks) apart from its `read:` list."""
    parts = re.split(r" — (?=read:|check:)", text)
    body = [p for p in parts if not p.startswith("read:")]
    read = [p[5:] for p in parts if p.startswith("read:")]
    return " — ".join(body), " ".join(read)


# --- CHARTER.md ---------------------------------------------------------------

def fence_items(charter):
    """Each reservation item, (line, text), from the shapes the charters use:
    a `## Leave for the operator` (or `## What this queue does not decide`)
    section's bullets; a `**Leave for the operator** (...): a; b; c.`
    paragraph; and a `## The budget` section's first paragraph."""
    lines = charter.read_text().splitlines()
    items, section, cur = [], None, None

    def close():
        nonlocal cur
        if cur:
            items.append((cur[0], " ".join(cur[1]).strip()))
        cur = None

    i = 0
    while i < len(lines):
        line = lines[i]
        n = i + 1
        if line.startswith("## "):
            close()
            section = ("fence" if FENCE_HEAD.match(line)
                       else "budget" if re.match(r"^##\s+The budget\b", line) else None)
            i += 1
            continue
        if INLINE_FENCE.match(line):
            close()
            para, starts = [], []
            while i < len(lines) and lines[i].strip():
                starts.append((sum(len(p) + 1 for p in para), i + 1))
                para.append(lines[i].strip())
                i += 1
            text = " ".join(para)
            body_at = text.find("):") + 2 if "):" in text else text.find(":") + 1
            pos = body_at
            for piece in text[body_at:].split(";"):
                line_of = max(ln for off, ln in starts if off <= pos + len(piece) - len(piece.lstrip()))
                if piece.strip():
                    items.append((line_of, piece.strip().rstrip(".")))
                pos += len(piece) + 1
            continue
        if section == "fence":
            if line.startswith("- "):
                close()
                cur = (n, [line[2:]])
            elif cur and line.startswith("  ") and line.strip():
                cur[1].append(line.strip().lstrip("- "))
            elif line.strip() and not line.startswith("- "):
                close()
                if items and not line.startswith("Write "):
                    section = None
        elif section == "budget" and line.strip():
            para_start = n
            para = []
            while i < len(lines) and lines[i].strip():
                para.append(lines[i].strip())
                i += 1
            items.append((para_start, "the budget: " + " ".join(para)))
            section = None
            continue
        i += 1
    close()
    return items


def terms_of(text):
    """The words a fence item can be matched by: its backticked names, its
    bold lead, and its short comma-separated phrases. A judgment longer than
    three words is no term: said as not checked."""
    found = set(re.findall(r"`([^`]+)`", text))
    bold = re.findall(r"\*\*([^*]+)\*\*", text)
    plain = re.sub(r"`[^`]+`|\*\*[^*]+\*\*|\([^)]*\)", " ", text)
    for chunk in [c for b in bold + [plain] for c in re.split(r",\s+|\s+or\s+|:\s+|;\s+", b)]:
        chunk = re.sub(r"^(?:and|a|an|any|the|for)\s+", "", chunk.strip(" .:;,'\"“”"), flags=re.I).strip()
        if len(chunk.split()) > 3 and " of " in chunk:  # "any widening of the trust floor": its object
            chunk = re.sub(r"^(?:the|a|an)\s+", "", chunk.rsplit(" of ", 1)[1], flags=re.I)
        if chunk and len(chunk.split()) <= 3 and len(chunk) >= 4:
            found.add(chunk)
    found.discard("NEEDS_HUMAN.md")
    return sorted(t for t in found if t.lower() not in STOP_TERMS)


def term_re(term):
    words = term.split()
    last = re.sub(r"(ing|ies|es|s)$", "", words[-1]) if words[-1].isalpha() and len(words[-1]) > 5 else words[-1]
    stem = r"\s+".join([re.escape(w) for w in words[:-1]] + [re.escape(last) + (r"\w*" if last != words[-1] else "")])
    return re.compile(r"(?<![\w-])" + stem + r"(?![\w-])" if last == words[-1] else r"(?<![\w-])" + stem, re.I)


def parse_ceiling(text):
    """The budget fence's ceiling, per scope, in the shapes the charters use:
    relative ("+150 repo, +60 core", "+25 over the pass's cap, any
    svrngs-core line"), absolute ("past 16,925 for the repository or 3,076
    for svrngs-core", "past 15,350, or svrngs-core's past 3,000") or the caps
    as they stand ("past the caps", "raising the size budget", "Zero")."""
    clause, _, qualifier = text.partition(" — but for")
    out = {"qualifier": ("but for" + qualifier).strip() if qualifier else ""}
    if re.search(r"^the budget:\s*Zero\b", clause, re.I):
        return {**out, "kind": "caps"}
    if re.search(r"count it started|started at", clause):  # a per-row allowance, over a count no file holds
        return {**out, "kind": "unparsed", "why": "a per-row allowance over the count the row starts at"}
    rel = re.findall(r"\+(\d[\d,]*)\s*(repo(?:sitory)?|(?:svrngs-)?core)?", clause)
    if rel:
        out["kind"] = "relative"
        out["delta"] = {}
        for num, unit in rel:
            scope = "crates/svrngs-core" if unit and "core" in unit else "."
            out["delta"].setdefault(scope, int(num.replace(",", "")))
        if re.search(r"\bany svrngs-core line\b|\bcore unchanged\b", clause):
            out["delta"].setdefault("crates/svrngs-core", 0)
        out["base"] = "pass" if re.search(r"\bpass\b|economiz", text) else None
        return out
    nums = {}
    for sub in re.split(r",\s+|\s+or\s+", clause):
        for m in re.finditer(r"(?<![+\d,-])(\d{1,2},\d{3}|\d{4,5})(?![\d,])(?!-\d)(?!\s*(?:MB|KB|s\b))", sub):
            scope = "crates/svrngs-core" if "core" in sub else "."
            nums.setdefault(scope, int(m.group(1).replace(",", "")))
    if nums:
        return {**out, "kind": "absolute", "ceiling": nums}
    if re.search(r"past the ceilings?\b", clause, re.I):  # stated in the charter's decisions
        return {**out, "kind": "elsewhere"}
    if re.search(r"past the caps|raising the (?:size )?budget|\ba raise\b", clause, re.I):
        return {**out, "kind": "caps"}
    return {**out, "kind": "unparsed"}


def pass_base(rows, git_dir):
    """The caps an economizing pass left: lean_check's BUDGET at the commit
    the queue's done `*economize*` row names."""
    done = [r for r in rows if "economize" in (r["id"] or "") and r["mark"] == "x" and r["commit"]]
    if not done:
        return None, "no done economize row with a commit in STATE.md"
    r = subprocess.run(["git", "-C", str(git_dir), "show", f"{done[-1]['commit']}:scripts/lean_check.py"],
                       capture_output=True, text=True)
    m = re.search(r"^BUDGET = (\{.*\})", r.stdout, re.M) if r.returncode == 0 else None
    if not m:
        return None, f"git show {done[-1]['commit']}:scripts/lean_check.py gave no BUDGET (exit {r.returncode})"
    return ast.literal_eval(m.group(1)), f"scripts/lean_check.py@{done[-1]['commit']} BUDGET ({done[-1]['id']})"


# --- the lean count -----------------------------------------------------------

def lean_reading(root, lean_file):
    if lean_file:
        text, source, code = Path(lean_file).read_text(), where(lean_file, root), None
    else:
        r = subprocess.run([sys.executable, str(root / "scripts" / "lean_check.py"), str(root)],
                           capture_output=True, text=True)
        text, source, code = r.stdout, "python3 scripts/lean_check.py (run now)", r.returncode
    counts = {}
    for line in text.splitlines():
        m = LEAN_RE.match(line.strip())
        if m:
            counts[m.group(1)] = (int(m.group(2)), int(m.group(3)))
    broken = [l for l in text.splitlines() if l.startswith("BROKEN")]
    return counts, source, code, broken


def headroom(ceiling, ceiling_at, counts, lean_source, rows, git_dir):
    """What is left under the ceiling, per scope, or why it is not known."""
    out, problems = {}, []
    base, base_from = ({}, "")
    if ceiling["kind"] == "relative":
        if ceiling.get("base") != "pass":
            problems.append(f"the budget fence is relative ({ceiling_at}) but names no base this sweep knows")
        else:
            base, base_from = pass_base(rows, git_dir)
            if base is None:
                problems.append(f"the budget fence is relative to the economizing pass ({ceiling_at}): {base_from}")
    for scope in SCOPE_NAME:
        if scope not in counts:
            problems.append(f"{lean_source} gave no count for {scope}")
            continue
        n, cap = counts[scope]
        entry = {"count": n, "cap": cap, "evidence": [ceiling_at, lean_source]}
        if ceiling["kind"] == "caps":
            entry["ceiling"], entry["reading"] = cap, "the caps as they stand"
        elif ceiling["kind"] == "absolute" and scope in ceiling["ceiling"]:
            entry["ceiling"], entry["reading"] = ceiling["ceiling"][scope], "the charter's number"
        elif ceiling["kind"] == "relative" and base and scope in base and scope in ceiling["delta"]:
            entry["ceiling"] = base[scope] + ceiling["delta"][scope]
            entry["reading"] = f"{base[scope]} + {ceiling['delta'][scope]} over {base_from}"
            entry["evidence"].append(base_from)
        else:
            entry["ceiling"] = None
            entry["reading"] = f"the budget fence states no ceiling for {SCOPE_NAME[scope]}"
        entry["left"] = None if entry["ceiling"] is None else entry["ceiling"] - 1 - n
        out[scope] = entry
    return out, problems


# --- a row's premises ---------------------------------------------------------

class Tree:
    def __init__(self, root):
        self.root = root
        r = subprocess.run(["git", "-C", str(root), "ls-files", "-co", "--exclude-standard"],
                           capture_output=True, text=True)
        self.files = r.stdout.splitlines() if r.returncode == 0 else [
            p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file()]

    def resolve(self, raw, docs):
        """(absolute paths it names, how) or ([], why not)."""
        if raw.startswith("~/"):
            p = Path(os.path.expanduser(raw))
            return ([p], "exists") if p.exists() else ([], "missing")
        base = Path(os.path.expanduser("~/dev/svrn-docs")) if docs else self.root
        if "<" in raw:
            fixed = raw.split("<", 1)[0].rstrip("/")
            fixed = fixed.rsplit("/", 1)[0] if "/" in fixed else ""
            p = base / fixed
            return ([p], "made tonight, under an existing dir") if p.is_dir() else ([], f"{fixed}/ is missing")
        if "*" in raw:
            hits = sorted(base.glob(raw))
            return (hits, "glob") if hits else ([], "the glob matches nothing")
        p = base / raw
        if p.exists():
            return [p], "exists"
        if not docs:
            hits = [self.root / f for f in self.files if f.endswith("/" + raw.rstrip("/")) or f == raw]
            if hits:
                return hits, "found by its tail"
        if "/" in raw.rstrip("/") and p.parent.is_dir():
            return [p], "new, its parent exists"
        return [], "missing"


def mentions(text):
    """(token, line-from, kind, in-backticks, offset) for each path or command the text names."""
    out = []
    for m in re.finditer(r"`([^`]+)`", text):
        tok = m.group(1).strip()
        first = tok.split()[0] if tok.split() else ""
        if first in COMMANDS or first.startswith(("scripts/", "./scripts/")) and " " in tok:
            out.append((tok, None, "command", m.start()))
            continue
        pm = BARE_PATH.fullmatch(tok)
        if pm:
            out.append((pm.group(1), pm.group(2), "path", m.start()))
    blanked = re.sub(r"`[^`]+`", lambda m: " " * len(m.group(0)), text)
    for m in BARE_PATH.finditer(blanked):
        tok = m.group(1).rstrip(".,;:")
        if tok.startswith("/") or "://" in tok or not ("/" in tok or re.search(r"\." + EXT + "$", tok)):
            continue
        last = tok.rstrip("/").rsplit("/", 1)[-1]
        if "/" in tok and not ("." in last or tok.endswith("/") or "*" in tok or "<" in tok):
            continue
        out.append((tok, m.group(2), "path", m.start()))
    return out


def premises(row, tree, settings, state_at):
    findings = []
    body, read = segments(row["text"])
    deny = settings.get("deny", [])
    dirs = [Path(os.path.expanduser(d)) for d in settings.get("dirs", [])]
    for text, where_ in ((body, "body"), (read, "read")):
        for tok, line_from, kind, at in mentions(text):
            if kind == "command":
                for pat in deny:
                    m = re.fullmatch(r"Bash\((.*)\)", pat)
                    if m and fnmatch.fnmatchcase(tok, m.group(1)):
                        findings.append(finding("denied-command", "ask-now",
                                                f"names `{tok}`, which the worker's deny list refuses ({pat})",
                                                state_at, settings.get("at")))
                args = [a for a in tok.split()[1:] if "/" in a and not a.startswith("-")]
                script = tok.split()[0] if tok.split()[0].startswith(("scripts/", "./scripts/")) else (args[0] if args else None)
                if script and not (tree.root / script.lstrip("./")).exists() and "<" not in script:
                    findings.append(finding("premise-missing", "ask-now",
                                            f"names `{tok}`, whose `{script}` does not exist", state_at))
                continue
            docs = text[max(0, at - 12):at].rstrip(" `").endswith("svrn-docs")
            hits, how = tree.resolve(tok, docs)
            if not hits:
                group = "ask-now"
                findings.append(finding("premise-missing", group,
                                        f"names `{tok}` ({'read' if where_ == 'read' else 'to change or make'}): {how}"
                                        + (" (looked in ~/dev/svrn-docs)" if docs else ""), state_at))
                continue
            if where_ == "read" and how.startswith("new"):
                findings.append(finding("premise-missing", "ask-now",
                                        f"reads `{tok}`, which does not exist yet", state_at))
            if line_from and len(hits) == 1 and hits[0].is_file():
                have = len(hits[0].read_text(errors="replace").splitlines())
                if int(line_from) > have:
                    findings.append(finding("premise-line", "ask-now",
                                            f"cites `{tok}:{line_from}`, past the {have} lines "
                                            f"{where(hits[0], tree.root)} has now", state_at))
            if where_ == "body" and dirs is not None:
                p = hits[0].resolve()
                inside = [d for d in [tree.root] + dirs if p == d.resolve() or d.resolve() in p.parents]
                writes = re.search(r"\b(write|rewrite|edit|change|delete|move)\w*\b[^.;]{0,40}$",
                                   text[max(0, at - 60):at], re.I)
                if not inside and writes:
                    findings.append(finding("not-writable", "ask-now",
                                            f"would change `{tok}`, outside the worker's tree and its "
                                            f"additionalDirectories", state_at, settings.get("at")))
                for pat in deny:
                    m = re.fullmatch(r"(?:Edit|Write|MultiEdit)\((.*)\)", pat)
                    if m and fnmatch.fnmatchcase(where(p, tree.root), m.group(1)) and writes:
                        findings.append(finding("not-writable", "ask-now",
                                                f"would change `{tok}`, which the deny list refuses ({pat})",
                                                state_at, settings.get("at")))
    return findings


# --- a row's estimate ---------------------------------------------------------

def estimates(text):
    """Each line estimate the row states: {scope: (lines, the words)}."""
    found = {}
    for m in re.finditer(r"\+?(\d[\d,]*)(?:\s*(?:to|-)\s*\+?(\d[\d,]*))?\s+(?:of them\s+)?(?:more\s+)?"
                         r"(?:(repository|repo|svrngs-core|core)\s+)?lines?\b", text, re.I):
        before = text[max(0, m.start() - 45):m.start()]
        if not re.search(r"estimat|\babout\b|~", before, re.I) or re.search(r"up to\s*$|at most\s*$|past\s*$", before):
            continue
        n = int((m.group(2) or m.group(1)).replace(",", ""))
        scope = "crates/svrngs-core" if m.group(3) and "core" in m.group(3).lower() else "."
        found.setdefault(scope, (n, m.group(0)))
    m = re.search(r"line estimate[^(]{0,40}\(about (\d[\d,]*)", text)
    if m:
        found.setdefault(".", (int(m.group(1).replace(",", "")), m.group(0)))
    return found


def ceiling_findings(row, room, room_problems, state_at):
    head = re.split(r"\s\(a\)\s", row["text"], maxsplit=1)[0]
    if re.search(r"nothing to product|not counted", head):
        said = re.search(r"nothing to product[^;)]*|not counted", head).group(0)
        return [finding("estimate-zero", None, f"states it adds nothing counted (\"{said}\")", state_at)]
    if room_problems:
        return [finding("ceiling-unknown", "could-not-judge", "the ceiling is not known: " + "; ".join(room_problems),
                        state_at)]
    est = estimates(row["text"])
    left = {s: e["left"] for s, e in room.items()}
    said = ", ".join(f"{'none' if v is None else v} left under {SCOPE_NAME[s]}'s ceiling" for s, v in left.items())
    if not est:
        zero = [SCOPE_NAME[s] for s, v in left.items() if v is not None and v <= 0]
        return [finding("no-estimate", "could-not-judge",
                        f"no estimate stated; {said}" + (f"; any line in {' or '.join(zero)} exceeds it" if zero else ""),
                        state_at, *sorted({e for r in room.values() for e in r["evidence"]}))]
    out = []
    for scope, (n, words) in est.items():
        have = left.get(scope)
        if have is None:
            out.append(finding("ceiling-unknown", "could-not-judge",
                               f"estimates {n} {SCOPE_NAME[scope]} lines (\"{words}\"), and no ceiling is stated there",
                               state_at))
        elif n > have:
            out.append(finding("over-headroom", "ask-now",
                               f"estimates {n} {SCOPE_NAME[scope]} lines (\"{words}\"), over the {have} left",
                               state_at, *room[scope]["evidence"]))
        else:
            out.append(finding("within-headroom", None,
                               f"estimates {n} {SCOPE_NAME[scope]} lines, within the {have} left", state_at))
    return out


# --- the sweep ----------------------------------------------------------------

def sweep(args):
    root = Path(args.root).resolve()
    qdir = root / "ralph" / "next" / args.queue
    state = Path(args.state) if args.state else qdir / "STATE.md"
    charter = Path(args.charter) if args.charter else qdir / "CHARTER.md"
    if not state.is_file():
        print(f"preshift: never-ran: no {state}", file=sys.stderr)
        sys.exit(2)
    st, ch = where(state, root), where(charter, root)
    report = {"queue": args.queue, "state": st, "charter": ch, "queue_findings": [], "rows": [], "fences": []}
    rows = parse_rows(state)
    marks = {r["id"]: r["mark"] for r in rows}
    parked = {p.stem for p in (qdir / "ctl" / "parked").glob("*.md")} if not args.state else set()

    # The charter's fences.
    items = fence_items(charter) if charter.is_file() else []
    if not charter.is_file():
        report["queue_findings"].append(finding("no-charter", "could-not-judge", f"no {ch}: no fence is known"))
    budget = [(n, t) for n, t in items if re.search(r"\bbudget\b|\bceiling", t, re.I)]
    budget.sort(key=lambda it: not re.match(r"^(?:\*\*)?(?:the budget|raising the (?:size )?budget|"
                                            r"this branch's budget)", it[1], re.I))
    for n, t in items:
        terms = [] if (n, t) in budget else terms_of(t)
        report["fences"].append({"line": n, "text": t, "terms": terms,
                                 "checked": "by the ceiling" if (n, t) in budget else
                                 ("by its terms" if terms else "not checked: a judgment, no word to match")})
    ceiling, ceiling_at = {"kind": "unparsed"}, None
    if budget:
        ceiling_at = f"{ch}:{budget[0][0]}"
        ceiling = parse_ceiling(budget[0][1])
        if ceiling["kind"] == "elsewhere":  # "past the ceilings": the decided "**The budget.** "+150 repo, ..."
            lines = charter.read_text().splitlines()
            said = next((i for i, l in enumerate(lines) if re.match(r'^- \*\*(?:The budget|"\+\d)', l)), None)
            if said is None:
                ceiling = {"kind": "unparsed", "why": "it names ceilings the charter states nowhere this sweep reads"}
            else:
                ceiling = parse_ceiling(" ".join(l.strip() for l in lines[said:said + 3]))
                ceiling_at += f" and {ch}:{said + 1}"
        report["ceiling"] = {"at": ceiling_at, "text": budget[0][1], **ceiling}
        if ceiling.get("qualifier"):
            report["queue_findings"].append(finding("ceiling-qualifier", None,
                                                    f"the budget fence's qualifier is not applied: \"{ceiling['qualifier']}\"",
                                                    ceiling_at))
        if len(budget) > 1:
            report["queue_findings"].append(finding("ceiling-second", None,
                                                    "more budget items than one; the first is read",
                                                    *[f"{ch}:{n}" for n, _ in budget[1:]]))
    counts, lean_source, lean_code, broken = lean_reading(root, args.lean)
    if broken:
        report["queue_findings"].append(finding("lean-broken", "ask-now",
                                                f"lean_check is broken now: {'; '.join(broken)}", lean_source))
    room_problems = []
    if not budget:
        room_problems.append(f"{ch} has no budget item in its fences")
    elif ceiling["kind"] == "unparsed":
        room_problems.append(f"{ceiling_at}'s budget item has no ceiling this sweep can read"
                             f"{' (' + ceiling['why'] + ')' if ceiling.get('why') else ''}: \"{budget[0][1]}\"")
    room, more = ({}, []) if room_problems else headroom(ceiling, ceiling_at, counts, lean_source, rows,
                                                         Path(args.git or root))
    room_problems += more
    report["headroom"] = room
    if room_problems:
        report["queue_findings"].append(finding("ceiling-unknown", "could-not-judge", "; ".join(room_problems),
                                                ceiling_at or ch))

    # The worker's permissions.
    settings = {"deny": [], "dirs": []}
    toml = qdir / "queue.toml"
    m = re.search(r'^settings\s*=\s*"([^"]+)"', toml.read_text(), re.M) if toml.is_file() else None
    if m and (root / m.group(1)).is_file():
        perms = json.loads((root / m.group(1)).read_text()).get("permissions", {})
        settings = {"deny": perms.get("deny", []), "dirs": perms.get("additionalDirectories", []),
                    "at": m.group(1)}
    else:
        report["queue_findings"].append(finding("no-settings", "could-not-judge",
                                                "no worker permission file found (queue.toml `settings`): "
                                                "what the worker may run or write is not judged",
                                                where(toml, root)))
        settings["dirs"] = None

    tree = Tree(root)
    for row in rows:
        if row["mark"] == "x":
            continue
        unknown = [d for d in row["deps"] if d not in marks]
        if any(marks.get(d) != "x" for d in row["deps"] if d not in unknown):
            continue
        at = f"{st}:{row['line']}"
        fs = [finding("unknown-depends", "could-not-judge", f"depends on {d}, which {st} has no row for", at)
              for d in unknown]
        if row["id"] in parked:
            fs.append(finding("parked", "ask-now", "already parked: its package waits for you",
                              where(qdir / "ctl" / "parked" / f"{row['id']}.md", root)))
        if row["id"].startswith("HUMAN-"):
            fs.append(finding("operator-present", "present", "a HUMAN- row: the operator's act, never a unit", at))
        body, _ = segments(row["text"])
        for pat in PRESENT:
            for m in pat.finditer(body):
                if NEGATED_AFTER.search(body[m.end():m.end() + 40]):
                    continue
                fs.append(finding("operator-present", "present", f"needs people there: \"{m.group(0)}\"", at))
        for m in re.finditer(r"NEEDS_HUMAN\.md", body):
            start = max(body.rfind(s, 0, m.start()) for s in (". ", "; ", " — ", ") "))
            stop = min([i for i in (body.find(". ", m.end()), body.find(" — ", m.end())) if i >= 0] or [len(body)])
            clause = body[start + 1:stop].strip()
            if CONDITIONAL.search(clause):
                fs.append(finding("conditional-park", "could-not-judge",
                                  f"parks if its condition holds, known only at the run: \"{clause[:140]}\"", at))
            else:
                fs.append(finding("planned-park", "ask-now", f"plans to stop for you: \"{clause[:140]}\"", at))
        for fence in report["fences"]:
            for term in fence["terms"]:
                for m in term_re(term).finditer(body):
                    if (NEGATED_BEFORE.search(body[max(0, m.start() - 40):m.start()])
                            or NEGATED_AFTER.search(body[m.end():m.end() + 40])):
                        continue
                    fs.append(finding("fence", "ask-now",
                                      f"names \"{m.group(0)}\", a term of the reservation \"{fence['text'][:90]}\"",
                                      at, f"{ch}:{fence['line']}"))
                    break
        fs += ceiling_findings(row, room, room_problems, at)
        fs += premises(row, tree, settings, at)
        gates = [f["group"] for f in fs if f["group"]]
        group = next((g for g, _ in GROUPS if g in gates), "clear")
        report["rows"].append({"id": row["id"], "line": row["line"], "in_flight": row["mark"] == "~",
                               "group": group, "findings": fs})
    groups = {r["group"] for r in report["rows"]} | {f["group"] for f in report["queue_findings"] if f["group"]}
    report["exit"] = 3 if groups & {"ask-now", "present"} else 4 if "could-not-judge" in groups else 0
    return report


def markdown(report):
    out = [f"# Evening review: {report['queue']}, {datetime.date.today().isoformat()}", "",
           f"Read {report['state']} and {report['charter']}. "
           f"{len(report['rows'])} ready row(s)." + ("" if report["rows"] else " Nothing runs tonight."), ""]
    if report.get("headroom"):
        out.append("**Headroom** (a charter number read as a lean_check cap: left = ceiling - 1 - count):")
        for scope, e in report["headroom"].items():
            out.append(f"- {SCOPE_NAME[scope]}: {e['count']} counted, ceiling "
                       f"{e['ceiling'] if e['ceiling'] is not None else 'not stated'} ({e['reading']}), "
                       f"{'unknown' if e['left'] is None else e['left']} left — {', '.join(e['evidence'])}")
        out.append("")
    for g, title in GROUPS:
        rows = [r for r in report["rows"] if r["group"] == g]
        queue = [f for f in report["queue_findings"] if f["group"] == g]
        out.append(f"## {title} ({len(rows) + len(queue)})")
        out.append("")
        for f in queue:
            out.append(f"- the queue: {f['say']} — {', '.join(f['evidence']) or 'no file'}")
        for r in rows:
            out.append(f"- **{r['id']}** ({report['state']}:{r['line']}{', in flight' if r['in_flight'] else ''})")
            for f in r["findings"]:
                mark = f["group"] or "note"
                out.append(f"  - {mark}: {f['say']} — {', '.join(f['evidence'])}")
        if not rows and not queue:
            out.append("- none")
        out.append("")
    notes = [f for f in report["queue_findings"] if not f["group"]]
    out.append("## What the sweep read of the charter")
    out.append("")
    if report.get("ceiling"):
        c = report["ceiling"]
        out.append(f"- ceiling ({c['at']}, read as {c['kind']}): \"{c['text'][:160]}\"")
    for f in notes:
        out.append(f"- {f['say']} — {', '.join(f['evidence'])}")
    for fence in report["fences"]:
        how = fence["checked"] + (f": {', '.join(fence['terms'])}" if fence["terms"] else "")
        out.append(f"- {report['charter']}:{fence['line']} {how} — \"{fence['text'][:100]}\"")
    out.append("")
    out.append(f"Exit {report['exit']}.")
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("queue")
    ap.add_argument("--root", default=Path(__file__).resolve().parents[4])
    ap.add_argument("--state", help="a STATE.md in place of the queue's, such as one from git show")
    ap.add_argument("--charter", help="a CHARTER.md in place of the queue's")
    ap.add_argument("--lean", help="a saved lean_check.py reading in place of running it")
    ap.add_argument("--git", help="the git tree commits are read from (default --root)")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    report = sweep(args)
    print(json.dumps(report, indent=2) if args.json else markdown(report))
    return report["exit"]


if __name__ == "__main__":
    sys.exit(main())
