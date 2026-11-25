"""The clinic's data: its EHR export (30 synthetic session notes, no real
person) and its client records, migrated from an old system into SQLite.
Three things are wrong here and nowhere at the vendor:
  - 3 notes from a template whose date line has the client's name typed in;
  - 2 clients whose migrated record has no plan id (NULL);
  - 3 notes whose minutes were written "50 min".
  python3 make-clinic.py <inbox dir> <clients.db>
"""
import os, random, sqlite3, sys
inbox, db = sys.argv[1], sys.argv[2]
os.makedirs(inbox, exist_ok=True)
random.seed(11)
first = ["Avery", "Jordan", "Riley", "Casey", "Morgan", "Quinn", "Rowan", "Sage", "Taylor", "Emerson"]
last = ["Lindqvist", "Okafor", "Marchetti", "Haddad", "Novak", "Ferreira", "Tanaka", "Byrne", "Kowalski", "Mendez"]
con = sqlite3.connect(db); con.execute("create table clients (client_id text primary key, name text, payer text, plan_id text)")
for i in range(30):
    name = f"{random.choice(first)} {random.choice(last)}"
    cid = f"TEST-{1000 + i}"
    con.execute("insert into clients values (?,?,?,?)", (cid, name, random.choice(["Aetna", "BCBS", "Cigna"]), None if i in (8, 22) else f"pl-{random.randint(10000, 99999)}"))
    day = f"2026-11-{1 + i % 28:02d}"
    date = f"Nov {1 + i % 28}, 2026 - {name}, telehealth" if i in (6, 19, 25) else day
    minutes = "50 min" if i in (3, 14, 27) else random.choice(["30", "45", "53", "60"])
    body = (f"Session note (synthetic)\nDate of service: {date}\nClient ID: {cid}\nCPT: {random.choice(['90832', '90834', '90837'])}\n"
            f"Diagnosis: {random.choice(['F41.1', 'F32.1', 'F43.10'])}\nMinutes: {minutes}\n"
            f"{name.split()[0]} said “the week was hard”; practiced grounding, will continue CBT.\n")
    open(os.path.join(inbox, f"{day} {name} - session.txt"), "w").write(body)
con.commit()
