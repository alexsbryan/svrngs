#!/usr/bin/env python3
"""preshift.py held to the park it exists to see coming (2026-10-19, guests-link).

The guests queue's STATE.md and CHARTER.md as they stood at 65c7fbb3, the
commit before guests-link (b) parked over its ceiling (a3212c59,
demo/runs/guests/2026-10-19-link/NEEDS_HUMAN.md), are read from git; the
lean_check reading at that commit is fixtures/lean-65c7fbb3.txt, taken from
`git archive 65c7fbb3`. Run: python3 demo/runs/preshift/2026-10-23/test_preshift.py
"""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
SCRIPT = HERE / "preshift.py"
PARK = "65c7fbb3"
LEAN = HERE / "fixtures" / f"lean-{PARK}.txt"
# The planted estimate: what the park measured, stated in the row beforehand.
PLANT = " — an estimate of about 118 repository lines, 7 of them svrngs-core lines"


def at_park(path):
    return subprocess.run(["git", "-C", str(ROOT), "show", f"{PARK}:{path}"],
                          capture_output=True, text=True, check=True).stdout


def sweep(state_text, charter_text):
    with tempfile.TemporaryDirectory() as d:
        state, charter = Path(d) / "STATE.md", Path(d) / "CHARTER.md"
        state.write_text(state_text)
        charter.write_text(charter_text)
        r = subprocess.run([sys.executable, str(SCRIPT), "guests", "--root", str(ROOT),
                            "--state", str(state), "--charter", str(charter),
                            "--lean", str(LEAN), "--json"], capture_output=True, text=True)
        try:
            return r.returncode, json.loads(r.stdout)
        except json.JSONDecodeError:
            raise AssertionError(f"exit {r.returncode}, no JSON: {r.stdout[:300]} {r.stderr[-600:]}")


def row(report, row_id):
    found = [r for r in report["rows"] if r["id"] == row_id]
    if not found:
        raise AssertionError(f"{row_id} not among the ready rows: {[r['id'] for r in report['rows']]}")
    return found[0]


class TheGuestsLinkPark(unittest.TestCase):
    def setUp(self):
        self.state = at_park("ralph/next/guests/STATE.md")
        self.charter = at_park("ralph/next/guests/CHARTER.md")

    def test_the_ceiling_is_read_and_its_headroom_is_what_the_park_found(self):
        _, report = sweep(self.state, self.charter)
        h = report["headroom"]
        # NEEDS_HUMAN.md: "The operator's ceiling leaves 24 in the repository and none in svrngs-core".
        self.assertEqual((h["."]["ceiling"], h["."]["left"]), (16728, 24), h)
        self.assertEqual((h["crates/svrngs-core"]["ceiling"], h["crates/svrngs-core"]["left"]), (3069, 0), h)
        self.assertTrue(any(e.endswith("CHARTER.md:79") for e in h["."]["evidence"]), h)

    def test_the_row_as_written_is_flagged_no_estimate_never_defaulted(self):
        code, report = sweep(self.state, self.charter)
        link = row(report, "guests-link")
        kinds = [f["kind"] for f in link["findings"]]
        self.assertIn("no-estimate", kinds, link)
        self.assertNotEqual(link["group"], "clear", link)
        self.assertNotEqual(code, 0)

    def test_the_row_with_the_parks_own_measure_will_park(self):
        planted = "\n".join(l + PLANT if l.startswith("- [ ] guests-link ") else l
                            for l in self.state.splitlines())
        code, report = sweep(planted, self.charter)
        link = row(report, "guests-link")
        over = [f for f in link["findings"] if f["kind"] == "over-headroom"]
        self.assertEqual(len(over), 2, link)  # 118 over 24, and 7 over 0
        self.assertEqual(link["group"], "ask-now", link)
        self.assertEqual(code, 3)


class EachGateOnItsPlant(unittest.TestCase):
    """Each other gate on a planted row, against the same charter."""

    def test_each_plant_lands_in_its_group(self):
        rows = [
            "- [ ] plant-paths — depends [] — ONE ROW: change `crates/svrngs-node/src/nosuch/x.rs` "
            "and `scripts/lean_check.py:9999` — read: `docs/NOSUCH.md`",
            "- [ ] HUMAN-plant-demo — depends [] — the operator's act",
            "- [ ] plant-people — depends [] — ONE ROW (nothing to product): a live demo at the table",
            "- [ ] plant-denied — depends [] — ONE ROW (nothing to product): `docker run x`",
            "- [ ] plant-waits — depends [plant-paths] — not ready, so not swept",
        ]
        code, report = sweep("# plants\n\n" + "\n".join(rows) + "\n", at_park("ralph/next/guests/CHARTER.md"))
        got = {r["id"]: (r["group"], sorted({f["kind"] for f in r["findings"] if f["group"]}))
               for r in report["rows"]}
        self.assertEqual(got, {
            # No economize row in these plants, so the charter's relative ceiling has no base.
            "plant-paths": ("ask-now", ["ceiling-unknown", "premise-line", "premise-missing"]),
            "HUMAN-plant-demo": ("present", ["ceiling-unknown", "operator-present"]),
            "plant-people": ("present", ["operator-present"]),
            "plant-denied": ("ask-now", ["denied-command"]),
        })
        self.assertEqual(code, 3)


if __name__ == "__main__":
    unittest.main(verbosity=2)
