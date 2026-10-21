# Each top-level `function`/`const`/`let`/`class` of the page's JavaScript,
# and how many times its name is said in the page's files (html included):
# a name said once is defined and never used. The one check a page without
# a compiler allows (start-economize's survey §1, by name).
import re, subprocess, sys
fs = subprocess.run(["git", "ls-files", "web/*.js", "web/*.html", "crates/svrngs-node/src/home/*"],
                    capture_output=True, text=True).stdout.split()
fs = [f for f in fs if not f.startswith("web/tests/")]
text = {f: open(f).read() for f in fs}
alone = []
for f in fs:
    if not f.endswith(".js"): continue
    for m in re.finditer(r"^(?:export\s+)?(?:async\s+)?(?:function\*?|const|let|class)\s+(\w+)", text[f], re.M):
        n = m.group(1)
        said = sum(len(re.findall(r"\b" + re.escape(n) + r"\b", t)) for t in text.values())
        if said < 2: alone.append(f"{f}: {n} said {said}")
print(len(fs), "files")
print("\n".join(alone) or "every top-level name is said again")
