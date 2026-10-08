"""Each person's own Jellyfin tokens, after a pair through the middleware
(check.sh `pair` with MW=1; mw-2's row): the middleware signs each person
in and keeps their token, so no browser holds it to be read; Jellyfin's
own database does. Each person's account is the one they were signed in as
afterwards (drive.mjs `signed_in_after`), else the one holding their
browser's device; their tokens are every device's of that account. Written
one per line to <dump>/<user>/token, for scan.py, which counts a person's
own apart and searches the other's bytes for them. Read right after the
pair: the next one deletes the accounts, and their devices go with them.

usage: python3 tokens.py <jellyfin.db> <pair json> <dump dir>
"""
import json
import sqlite3
import sys
from pathlib import Path

db, run, dump = sys.argv[1], json.loads(Path(sys.argv[2]).read_text()), Path(sys.argv[3])
con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
said = {}
for user, p in run.get("people", {}).items():
    name = p.get("signed_in_after")
    row = con.execute("SELECT Id, Username FROM Users WHERE Username = ?", (name,)).fetchone() if name else None
    if not row and p.get("device"):
        row = con.execute(
            "SELECT u.Id, u.Username FROM Devices d JOIN Users u ON u.Id = d.UserId WHERE d.DeviceId = ?",
            (p["device"],)).fetchone()
    if not row:
        said[user] = "no account found: no token to search for"
        continue
    tokens = [t for (t,) in con.execute("SELECT AccessToken FROM Devices WHERE UserId = ?", (row[0],))]
    (dump / user).mkdir(parents=True, exist_ok=True)
    (dump / user / "token").write_text("\n".join(tokens) + "\n")
    said[user] = f"{row[1]}: {len(tokens)} token(s)"
print(f"tokens: {json.dumps(said)}")
