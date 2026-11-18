"""One Node for svrngs: the major version .nvmrc names is the one the gate
runs, and the one every page, README, build script and test message asks a
person for, "Node 24 or later" (docs/PLAN.md §1, "Examples, a ladder"; the
operator, 2026-11-18: "update everything to node 24"). Node's own LTS
schedule is why 24: github.com/nodejs/release, 20 ended 2026-04-30."""
import re
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
WANT = (ROOT / ".nvmrc").read_text().strip()
# "Node <n> or later" or "Node.js <n>+", in any case: what a person is asked for.
ASKED = re.compile(r"\bnode(?:\.js)?\s?v?(\d+)(?:\+|\s+or\s+later)", re.IGNORECASE)
# What is recorded or vendored says what it said then.
KEPT = ("demo/runs/", "ralph/", "web/node_modules/", "docs/principles/")


def tracked():
    out = subprocess.run(["git", "ls-files", "-co", "--exclude-standard", "-z"], cwd=ROOT,
                         capture_output=True, check=True).stdout.decode()
    return [p for p in out.split("\0") if p and not p.startswith(KEPT) and "/vendor/" not in p]


class OneNode(unittest.TestCase):
    def test_the_gate_runs_the_node_nvmrc_names(self):
        got = subprocess.run(["node", "--version"], capture_output=True, text=True).stdout.strip()
        self.assertTrue(got.startswith(f"v{WANT}."),
                        f"node is {got or 'missing'}; .nvmrc names {WANT}: `nvm install` in the checkout")

    def test_every_ask_for_node_names_nvmrcs(self):
        wrong = []
        for rel in tracked():
            path = ROOT / rel
            if not path.is_file() or path.suffix in {".lock", ".png", ".svg", ".woff2", ".gguf"}:
                continue
            text = path.read_text(errors="ignore")
            for m in ASKED.finditer(text):
                if m.group(1) != WANT:
                    wrong.append(f"{rel}:{text.count(chr(10), 0, m.start()) + 1}: {m.group(0)}")
        self.assertEqual(wrong, [], f".nvmrc names Node {WANT}; these ask for another")


if __name__ == "__main__":
    unittest.main()
