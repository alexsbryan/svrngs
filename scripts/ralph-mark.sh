#!/usr/bin/env bash
# ported from commonwealth-ai scripts/ralph-mark.sh@8c4c66adc
# Mark a queue row done: rewrite its line to [x] with the commit, and commit the
# state file alone with the subject ralph.py counts. Usage: scripts/ralph-mark.sh <unit-id> <short-sha> [state-file]
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

unit="${1:-}"; sha="${2:-}"; state="${3:-${RALPH_STATE:-}}"
[ -n "$unit" ] && [ -n "$sha" ] || { echo "usage: scripts/ralph-mark.sh <unit-id> <short-sha> [state-file]" >&2; exit 2; }
[ -n "$state" ] || { echo "ralph-mark: no state file — pass it as the third argument, or run under scripts/ralph.py, which sets RALPH_STATE" >&2; exit 2; }
git cat-file -e "${sha}^{commit}" 2>/dev/null || { echo "ralph-mark: $sha is not a commit in this repo" >&2; exit 2; }

n=$(grep -c -E "^- \[[~ ]\] ${unit} — depends" "$state" || true)
[ "$n" = "1" ] || { echo "ralph-mark: expected one open row for $unit in $state, found $n" >&2; exit 3; }
# python, not `sed -i -E`: BSD sed reads `-E` as the BACKUP SUFFIX, so on macOS
# the edit landed and left a `STATE.md-E` beside it on every mark (2026-09-19).
python3 - "$state" "$unit" "$sha" <<'PYEOF'
import re, sys
path, unit, sha = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
new, n = re.subn(rf"(?m)^- \[[~ ]\] {re.escape(unit)} — depends", f"- [x] {unit} {sha} — depends", text)
if n != 1:
    sys.exit(f"ralph-mark: rewrite matched {n} rows")
open(path, "w", encoding="utf-8").write(new)
PYEOF
grep -q -E "^- \[x\] ${unit} ${sha} — depends" "$state" || { echo "ralph-mark: rewrite did not land" >&2; exit 3; }
git add -- "$state"
git commit -q -m "ralph: ${unit} done" -- "$state"
echo "marked ${unit} ${sha}: $(git log --oneline -1)"
