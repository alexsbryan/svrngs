#!/usr/bin/env python3
"""sy-1's four plants as tests' red runs (README §2 here; the queue's fence 3):
each guard taken out of product code by a temporary edit, its test run and
watched, the edit restored, then every guard's test run green. Output in
plants/<plant>.out. The tree must be clean at the start; it is restored
after each edit, and checked so at the end. scripts/check.sh runs none of
this."""
import subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[4]; OUT = HERE / "plants"
SYNCED, FOLDER, NODE = (REPO / "crates/svrngs-node/src" / f for f in ("synced.rs", "folder.rs", "node.rs"))
SERVED = "a_synced_file_is_served_by_a_member_who_did_not_write_it_and_to_no_one_outside"
PLANTS = [
    ("deaf", SYNCED, "        let sent = changes.len();\n",
     "        changes.clear();\n        let sent = changes.len();\n",
     "a_change_by_any_member_reaches_another_s_directory"),
    ("own-only", FOLDER, "Ok(read) if read.hashes.contains(hash) =>",
     "Ok(read) if read.placed.iter().any(|p| p.hash == *hash && p.entry.key == self.host.actor()) =>", SERVED),
    ("later-wins", FOLDER, "            .layout()\n            .into();\n",
     "            .layout()\n            .into_iter()\n            .filter(|p| !p.beside)\n            .collect::<Vec<_>>()\n            .into();\n",
     "both_sides_of_a_conflict_are_kept"),
    ("open-gate", NODE, "    if let Some((namespace, _)) = rail.standing_anywhere(&actor) {\n",
     "    if true {\n        return Alpn::of(alpn);\n    }\n    if let Some((namespace, _)) = rail.standing_anywhere(&actor) {\n", SERVED),
]


def test(*names):
    run = subprocess.run(["cargo", "test", "--offline", "-p", "svrngs-node", "--test", "folders", "--", "--exact", *names],
                         cwd=REPO, capture_output=True, text=True)
    return run.returncode, run.stdout + run.stderr


def main():
    if subprocess.run(["git", "diff", "--quiet", "--", "crates"], cwd=REPO).returncode: sys.exit("crates/ is not clean")
    OUT.mkdir(exist_ok=True)
    head = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=REPO, capture_output=True, text=True).stdout.strip()
    for plant, path, guard, out, name in PLANTS:
        src = path.read_text()
        if src.count(guard) != 1: sys.exit(f"{plant}: the guard is not found once in {path}")
        try:
            path.write_text(src.replace(guard, out)); code, said = test(name)
        finally: path.write_text(src)
        (OUT / f"{plant}.out").write_text(f"at {head}, {plant} planted in {path.relative_to(REPO)}: exit {code}\n{said}")
        print(plant, name, "exit", code, "red" if code else "NOT RED", flush=True)
    code, said = test(*sorted({p[4] for p in PLANTS}))
    (OUT / "green.out").write_text(f"at {head}, every guard in: exit {code}\n{said}")
    print("green", "exit", code, flush=True)
    if subprocess.run(["git", "diff", "--quiet", "--", "crates"], cwd=REPO).returncode: sys.exit("crates/ not restored")


if __name__ == "__main__":
    main()
