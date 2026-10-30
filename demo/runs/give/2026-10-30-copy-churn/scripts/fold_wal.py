# usage: python3 -I fold_wal.py <app-dir> <db-name>
# A measure of one candidate for (c), not a build of it: for each copy
# snaps/<k>.tar, take the SQLite database and its -wal out of the tar (not the
# -shm, which SQLite rebuilds from the WAL), open it in a scratch directory,
# fold the WAL in (`PRAGMA wal_checkpoint(TRUNCATE)`), close it, and hash the
# database alone. Between consecutive copies, count the database pages that
# differ. This is what a copy would carry if the WAL were folded into the
# database in the copy, never in the app's own volume.
import hashlib
import os
import sqlite3
import sys
import tarfile
import tempfile

app_dir, db = sys.argv[1], sys.argv[2]
pages, prev = [], None
for k in range(7):
    with tempfile.TemporaryDirectory() as d, tarfile.open(os.path.join(app_dir, "snaps", f"{k}.tar")) as t:
        for suffix in ("", "-wal"):
            try:
                m = t.getmember(db + suffix)
            except KeyError:
                continue
            with open(os.path.join(d, db + suffix), "wb") as f:
                f.write(t.extractfile(m).read())
        c = sqlite3.connect(os.path.join(d, db))
        size = c.execute("PRAGMA page_size").fetchone()[0]
        c.execute("PRAGMA wal_checkpoint(TRUNCATE)")
        ok = c.execute("PRAGMA integrity_check").fetchone()[0]
        c.close()
        raw = open(os.path.join(d, db), "rb").read()
    cur = [raw[i:i + size] for i in range(0, len(raw), size)]
    line = f"{k}: {db} folded {len(raw)} bytes, page {size}, integrity {ok}, sha256 {hashlib.sha256(raw).hexdigest()[:12]}"
    if prev is not None:
        n = sum(a != b for a, b in zip(prev, cur)) + abs(len(prev) - len(cur))
        line += f"; vs {k - 1}: {n} pages differ = {n * size} bytes"
    print(line)
    prev = cur
