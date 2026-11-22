#!/usr/bin/env python3
"""The .strm writer (gf-1; demo/runs/gf/2026-10-09/README.md §2): one stub
per film the group's folders name, each holding that film's URL at the
meeting point's endpoint, so Jellyfin plays it by HTTP range from there.

    SVRNGS_DIR=<meeting point's data dir> svrngs ask /v1/folders | strm.py LIBRARY_DIR

On stdin, what the meeting point's svrngs answers at GET /v1/folders,
asked by `svrngs ask`, which never sends its token: the endpoint and
every standing folder. A hash two holders share is one film, one stub. A film is named
by its kind, as the first run named its two (ExpMp4, ExpMkv), so the
driver finds it. Prints each stub and its URL; exits 1 when no folder
names a film, never writing an empty library.
"""
import json
import os
import sys

TYPES = {".mp4": "video/mp4", ".mkv": "video/x-matroska"}

library = sys.argv[1]
listed = json.load(sys.stdin)
endpoint = listed["endpoint"]
films = {}
for folder in listed["folders"]:
    for f in folder["files"]:
        ext = os.path.splitext(f["path"])[1].lower()
        if ext in TYPES:
            films.setdefault(f["hash"], (f["path"], ext))
if not films:
    sys.exit(f"no folder names a film: {json.dumps(listed)}")
for hash_, (path, ext) in sorted(films.items(), key=lambda x: x[1][1]):
    name = f"Exp{ext[1:].capitalize()} (2026)"
    url = f"{endpoint}{hash_}/{os.path.basename(path)}?type={TYPES[ext].replace('/', '%2F')}"
    os.makedirs(os.path.join(library, name), exist_ok=True)
    with open(os.path.join(library, name, f"{name}.strm"), "w") as out:
        out.write(url + "\n")
    print(json.dumps({"stub": name, "url": url, "hash": hash_, "path": path}))
