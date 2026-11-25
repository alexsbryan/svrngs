"""The clinic's client records, migrated from its old system into SQLite."""
import sqlite3


def plan_of(db: str, client_id: str) -> dict:
    row = sqlite3.connect(db).execute("select payer, plan_id from clients where client_id = ?", (client_id,)).fetchone()
    if row is None:
        raise LookupError("no such client")
    payer, plan_id = row
    return {"payer": payer, "plan": plan_id.upper()}
