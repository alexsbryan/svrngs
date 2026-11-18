#!/usr/bin/env python3
# The stand-in's database (standin.mjs): Magnets' schema.sql in SQLite, the
# one Python carries. node:sqlite would keep the stand-in in one language,
# but it starts at Node 22.13 and svrngs's tests run on Node 20 or later
# (README.md). Each line in is a JSON list of statements, {sql, params},
# run as one transaction; each line out is {"answers": [...]}, a statement's
# rows or its count of rows changed, or {"error": <SQLite's words>} with
# nothing of the list kept.
import json
import sqlite3
import sys

db = sqlite3.connect(":memory:", isolation_level=None)
with open(sys.argv[1]) as schema:
    db.executescript(schema.read())
for line in sys.stdin:
    try:
        db.execute("BEGIN")
        answers = []
        for statement in json.loads(line):
            cursor = db.execute(statement["sql"], statement.get("params", []))
            if cursor.description:
                names = [c[0] for c in cursor.description]
                answers.append({"rows": [dict(zip(names, row)) for row in cursor]})
            else:
                answers.append({"count": cursor.rowcount})
        db.execute("COMMIT")
        print(json.dumps({"answers": answers}), flush=True)
    except (sqlite3.Error, KeyError, TypeError, ValueError) as e:
        if db.in_transaction:
            db.execute("ROLLBACK")
        print(json.dumps({"error": str(e)}), flush=True)
