# envdiff.py <site-packages>: compare an installed env with its own RECORDs.
# Prints files a RECORD lists that are gone (the program removed them) and files
# no RECORD lists (the program, or Python, wrote them), ignoring __pycache__.
import sys, os, csv, glob
sp = sys.argv[1]
listed = set()
for rec in glob.glob(os.path.join(sp, "*.dist-info", "RECORD")):
    for row in csv.reader(open(rec)):
        if row: listed.add(os.path.normpath(os.path.join(sp, row[0])))
missing = sorted(p for p in listed if not os.path.lexists(p) and "__pycache__" not in p)
present = set()
for d, dirs, files in os.walk(sp):
    dirs[:] = [x for x in dirs if x != "__pycache__"]
    for f in files: present.add(os.path.normpath(os.path.join(d, f)))
extra = sorted(p for p in present - listed if not p.endswith(".pyc"))
print("listed:", len(listed), "present:", len(present))
print("removed since install:", [os.path.relpath(p, sp) for p in missing])
print("not in any RECORD:", [os.path.relpath(p, sp) for p in extra][:40], "count", len(extra))
