"""Each person's own Jellyfin tokens, after Movie Night's runs (check.sh
`judge`): each lender's middleware makes the person an account on their
first request, `<name>-<16 hex of their key>` (apps/jellyfin/mw/src/
viewer.rs `user_name`), and the door holds its token; no browser does.
Jellyfin's own databases, Sam's and Dee's, hold every device's token of
that account. Written one per line to <scratch>/dump/<run>/<person>/token,
for apps/jellyfin/check/scan.py, which counts a person's own apart and
searches every other person's bytes for them. Forked from
apps/jellyfin/check/tokens.py, which reads one database for one pair.

usage: python3 tokens.py <scratch dir>
"""
import json
import sqlite3
import sys
from pathlib import Path

root = Path(sys.argv[1])
dbs = {who: sqlite3.connect(f"file:{root}/jf-{who}/data/data/jellyfin.db?mode=ro", uri=True) for who in ("sam", "dee")}
said = {}
for person in sorted(p for p in (root / "dump").glob("*/*") if p.is_dir()):
    tokens, where = [], []
    for who, con in dbs.items():
        for uid, name in con.execute("SELECT Id, Username FROM Users WHERE lower(Username) LIKE ?", (f"{person.name}-%",)):
            found = [t for (t,) in con.execute("SELECT AccessToken FROM Devices WHERE UserId = ?", (uid,))]
            tokens += found
            where.append(f"{who}: {name} {len(found)}")
    if tokens:
        (person / "token").write_text("\n".join(tokens) + "\n")
    said[f"{person.parent.name}/{person.name}"] = where or ["no account: no token to search for"]
print(f"tokens: {json.dumps(said)}")
