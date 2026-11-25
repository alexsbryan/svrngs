"""The clinic's EHR export: 20 synthetic session notes (no real person), as
this site's export writes them. 17 are UTF-8; 3, from the older intake
workstation, are Windows-1252, with its curly quotes and dashes. File names
carry the client's name, as exports here do, so a leak of names would show.

  python3 make-inbox.py <dir>
"""
import os, random, sys
out = sys.argv[1]; os.makedirs(out, exist_ok=True)
random.seed(7)
first = ["Avery", "Jordan", "Riley", "Casey", "Morgan", "Quinn", "Rowan", "Sage", "Taylor", "Emerson"]
last = ["Lindqvist", "Okafor", "Marchetti", "Haddad", "Novak", "Ferreira", "Tanaka", "Byrne", "Kowalski", "Mendez"]
cpts = [("90834", 45), ("90837", 60), ("90832", 30)]
dx = ["F41.1", "F32.1", "F43.10", "F33.0"]
for i in range(20):
    name = f"{random.choice(first)} {random.choice(last)}"
    cpt, mins = random.choice(cpts)
    legacy = i in (4, 11, 17)
    body = (f"Session note — synthetic\nDate of service: 2026-11-{3 + i % 20:02d}\nClient ID: TEST-{1000 + i}\n"
            f"CPT: {cpt}\nDiagnosis: {random.choice(dx)}\nMinutes: {mins}\n"
            f"Client said “the week was hard” – practiced grounding; we’ll continue CBT.\n")
    data = body.replace("\n", "\r\n").encode("cp1252") if legacy else body.encode("utf-8")
    open(os.path.join(out, f"2026-11-{3 + i % 20:02d} {name} - session.txt"), "wb").write(data)
