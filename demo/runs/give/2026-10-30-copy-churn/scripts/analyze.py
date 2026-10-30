# usage: python3 -I analyze.py <app-dir> <out-dir>
# Reads <app-dir>/snaps/0..6.tar and <app-dir>/det/*.tar. Writes
# <out-dir>/manifests/<k>.txt (one entry per line, in the tar's own order:
# "path size sha256" for a file, "path/ dir", "path -> target" for a link)
# and <out-dir>/windows.json: per window between consecutive copies, the
# files added, removed and changed in content, the entries whose header alone
# changed (mtime, mode, owner), whether the entry order changed, the bytes of
# the changed and added files, and of the 4 KiB blocks that differ inside
# each changed file (fixed offsets).
import hashlib
import json
import os
import sys
import tarfile

app_dir, out_dir = sys.argv[1], sys.argv[2]
BLOCK = 4096
os.makedirs(os.path.join(out_dir, "manifests"), exist_ok=True)


def sha_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def read(path):
    """The tar's entries in order: name -> (kind, size, mtime, mode, uid, gid, link, sha)."""
    entries = {}
    with tarfile.open(path) as t:
        for m in t:
            name = m.name  # as the tar names it; podman writes them one way
            if m.isreg():
                h = hashlib.sha256()
                f = t.extractfile(m)
                for chunk in iter(lambda: f.read(1 << 20), b""):
                    h.update(chunk)
                entries[name] = ("f", m.size, m.mtime, m.mode, m.uid, m.gid, "", h.hexdigest())
            elif m.isdir():
                entries[name] = ("d", 0, m.mtime, m.mode, m.uid, m.gid, "", "")
            else:
                entries[name] = ("l" if m.issym() else m.type.decode(), 0, m.mtime, m.mode, m.uid, m.gid,
                                 m.linkname, "")
    return entries


def blocks_differ(tar_a, tar_b, name):
    """How many fixed 4 KiB blocks of `name` differ between the two tars (a block past
    the shorter file's end counts as differing)."""
    with tarfile.open(tar_a) as ta, tarfile.open(tar_b) as tb:
        fa, fb = ta.extractfile(ta.getmember(entry_name(ta, name))), tb.extractfile(tb.getmember(entry_name(tb, name)))
        n = 0
        while True:
            a, b = fa.read(BLOCK), fb.read(BLOCK)
            if not a and not b:
                return n
            n += a != b


def entry_name(t, name):
    for cand in (name, "./" + name):
        try:
            t.getmember(cand)
            return cand
        except KeyError:
            pass
    raise KeyError(name)


def manifest(entries, path):
    with open(path, "w") as f:
        for name, (kind, size, *_rest) in entries.items():
            sha = _rest[-1]
            if kind == "f":
                f.write(f"{name} {size} {sha}\n")
            elif kind == "d":
                f.write(f"{name}/ dir\n")
            else:
                f.write(f"{name} -> {_rest[-2]}\n")


def diff(a, b, tar_a, tar_b):
    fa = {k: v for k, v in a.items() if v[0] == "f"}
    fb = {k: v for k, v in b.items() if v[0] == "f"}
    added = sorted(set(fb) - set(fa))
    removed = sorted(set(fa) - set(fb))
    changed = sorted(k for k in set(fa) & set(fb) if fa[k][7] != fb[k][7])
    header_only = sorted(k for k in set(a) & set(b)
                         if a[k][7] == b[k][7] and a[k][:7] != b[k][:7])
    common = [k for k in a if k in b]
    order_changed = common != [k for k in b if k in a]
    blocks = {k: blocks_differ(tar_a, tar_b, k) for k in changed}
    return {
        "added": [(k, fb[k][1]) for k in added],
        "removed": [(k, fa[k][1]) for k in removed],
        "changed": [(k, fa[k][1], fb[k][1], blocks[k]) for k in changed],
        "header_only": [(k, [f for f, x, y in zip(("kind", "size", "mtime", "mode", "uid", "gid", "link"),
                                                     a[k][:7], b[k][:7]) if x != y]) for k in header_only],
        "order_changed": order_changed,
        "bytes_added": sum(fb[k][1] for k in added),
        "bytes_changed": sum(fb[k][1] for k in changed),
        "bytes_changed_blocks": sum(blocks.values()) * BLOCK,
    }


snaps = [os.path.join(app_dir, "snaps", f"{k}.tar") for k in range(7)]
det = sorted(os.path.join(app_dir, "det", n) for n in os.listdir(os.path.join(app_dir, "det")))
read_all = {}
copies = []
for p in snaps + det:
    e = read_all[p] = read(p)
    label = os.path.relpath(p, app_dir).replace("/", "-").removesuffix(".tar")
    manifest(e, os.path.join(out_dir, "manifests", f"{label}.txt"))
    files = [v for v in e.values() if v[0] == "f"]
    copies.append({"tar": os.path.relpath(p, app_dir), "bytes": os.path.getsize(p), "sha256": sha_file(p),
                   "entries": len(e), "files": len(files), "file_bytes": sum(v[1] for v in files),
                   "manifest_bytes": os.path.getsize(os.path.join(out_dir, "manifests", f"{label}.txt"))})
windows = []
for i in range(6):
    d = diff(read_all[snaps[i]], read_all[snaps[i + 1]], snaps[i], snaps[i + 1])
    d["window"] = f"{i}->{i + 1}"
    d["tar_identical"] = copies[i]["sha256"] == copies[i + 1]["sha256"]
    windows.append(d)
pairs = [("det/export-a.tar", "det/export-b.tar"), ("det/copy-a.tar", "det/copy-b.tar"),
         ("det/export-a.tar", "det/copy-a.tar"), ("snaps/6.tar", "det/export-a.tar")]
dets = []
for x, y in pairs:
    px, py = os.path.join(app_dir, x), os.path.join(app_dir, y)
    if px in read_all and py in read_all:
        d = diff(read_all[px], read_all[py], px, py)
        d["pair"] = f"{x} vs {y}"
        d["tar_identical"] = sha_file(px) == sha_file(py)
        dets.append(d)
json.dump({"copies": copies, "windows": windows, "determinism": dets},
          open(os.path.join(out_dir, "windows.json"), "w"), indent=1)
print(json.dumps({"copies": [(c["tar"], c["bytes"], c["sha256"][:12], c["files"]) for c in copies]}))
for w in windows + dets:
    print(w.get("window") or w["pair"], "identical" if w["tar_identical"] else "differs",
          "added", len(w["added"]), w["bytes_added"], "removed", len(w["removed"]),
          "changed", len(w["changed"]), w["bytes_changed"], "blocks", w["bytes_changed_blocks"],
          "header-only", len(w["header_only"]), "order", w["order_changed"])
    for c in w["changed"]:
        print("   changed", c)
    for c in w["added"][:20]:
        print("   added", c)
    for c in w["removed"][:20]:
        print("   removed", c)
    for c in w["header_only"][:20]:
        print("   header", c)
