#!/usr/bin/env python3
# The stand-in's database (standin.mjs): Saturday's schema.sql in SQLite, the
# one Python carries, copied from Magnets' (apps/magnets/tests/sqlite.py),
# which says node:sqlite could take its place on Node 24 or later and keep
# the stand-in in one language. Each line in is a
# JSON object, and each line out its answer:
#   {"run": [{sql, params}, ...]}  run as one transaction, nothing checked:
#                                  {"answers": [...]}, each statement's
#                                  {"rows": [...]} or its {"count": <n>}
#   {"read": {sql, params}}        one statement, run only if SQLite's
#                                  authorizer finds it only reads: {"rows"}
# or {"refused": "not-a-read" | "sql", "words": <SQLite's>}, with nothing of
# it kept. A BLOB crosses as {"base64": "..."}, in params and in rows.
import base64
import json
import sqlite3
import sys

# What a read may do, as SQLite's authorizer names each part of a statement
# while it compiles it (sqlite.org/c3ref/c_alter_table.html): select, read a
# column, call a function, recurse in a CTE. Anything else is not a read: an
# INSERT, a write behind a WITH or before a RETURNING, a CREATE, a BEGIN, a
# PRAGMA (even as pragma_table_info). Setting an authorizer expires every
# statement prepared before it, so one Python cached as a write is compiled
# again, and asked again, as a read.
READS = {sqlite3.SQLITE_SELECT, sqlite3.SQLITE_READ, sqlite3.SQLITE_FUNCTION, sqlite3.SQLITE_RECURSIVE}
denied = []


def reading(action, *_):
    if action in READS:
        return sqlite3.SQLITE_OK
    denied.append(action)
    return sqlite3.SQLITE_DENY


def bound(params):
    return [base64.b64decode(p["base64"], validate=True) if isinstance(p, dict) else p for p in params]


def answer(cursor):
    if not cursor.description:
        return {"count": cursor.rowcount}
    names = [c[0] for c in cursor.description]
    shown = lambda v: {"base64": base64.b64encode(v).decode()} if isinstance(v, bytes) else v
    return {"rows": [{n: shown(v) for n, v in zip(names, row)} for row in cursor]}


def run(statements):
    db.execute("BEGIN")
    answers = [answer(db.execute(s["sql"], bound(s.get("params", [])))) for s in statements]
    db.execute("COMMIT")
    return {"answers": answers}


def read(statement):
    db.set_authorizer(reading)
    try:
        cursor = db.execute(statement["sql"], bound(statement.get("params", [])))
    finally:
        db.set_authorizer(None)
    if not cursor.description:
        raise ValueError("it holds no statement")
    return answer(cursor)


db = sqlite3.connect(":memory:", isolation_level=None)
with open(sys.argv[1]) as schema:
    db.executescript(schema.read())
for line in sys.stdin:
    asked = json.loads(line)
    denied.clear()
    try:
        out = read(asked["read"]) if "read" in asked else run(asked["run"])
    except (sqlite3.Error, KeyError, TypeError, ValueError) as e:
        if db.in_transaction:
            db.execute("ROLLBACK")
        out = {"refused": "not-a-read" if denied else "sql", "words": str(e)}
    print(json.dumps(out), flush=True)
