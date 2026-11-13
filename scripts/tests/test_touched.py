"""The touched tier's plans (scripts/touched.py), each for a change set whose
reach is known: what it runs, and above all what it may not skip."""
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
# `playback` runs part of the browser and conformance binaries, so what
# reaches either reaches it (scripts/check.sh `playback`).
SITE = {"browser", "conformance canary", "joining in a browser", "pages", "playback", "scenes"}


def plan(*files):
    """The steps planned for `files`, and the `tests` selector; None for every step."""
    out = subprocess.run([sys.executable, "scripts/touched.py", "--files", *files], cwd=ROOT,
                         capture_output=True, text=True, check=True).stdout
    if not out:
        return None
    steps, tests = out.split("SVRNGS_STEPS='")[1].split("' SVRNGS_TESTS='")
    return set(filter(None, steps.split("\n"))), tests.rstrip("'\n")


class Plans(unittest.TestCase):
    def test_a_driver_reaches_the_binary_naming_it_alone(self):
        self.assertEqual(plan("web/tests/keeps-going.mjs"), ({"browser", "playback"}, ""))

    def test_a_helper_every_driver_reaches_reaches_every_binary_driving_them(self):
        steps, tests = plan("web/tests/page.mjs")
        self.assertLessEqual(SITE, steps)
        self.assertIn("-p svrngs-node", tests)

    def test_a_law_or_its_helper_is_the_cheap_tier_s(self):
        self.assertEqual(plan("web/tests/pages/enter.test.mjs", "web/tests/pages/layout.mjs"), (set(), ""))
        # One whose own import() could load anything, and only laws reach.
        self.assertEqual(plan("web/tests/pages/before.mjs"), (set(), ""))

    def test_the_phone_s_page_reaches_every_binary_building_the_site_and_the_types(self):
        self.assertEqual(plan("web/page/enter.js"), (SITE | {"types", "typed lint"}, ""))

    def test_core_reaches_its_dependents_and_the_wasm(self):
        steps, tests = plan("crates/svrngs-core/src/pass.rs")
        self.assertLessEqual(SITE | {"canaries", "web rustfmt and clippy", "clippy", "tests"}, steps)
        self.assertEqual(tests, "-p svrngs -p svrngs-core -p svrngs-node -p svrngs-runtime")

    def test_the_command_s_crate_leaves_the_node_s_binaries_out(self):
        steps, tests = plan("crates/svrngs/src/main.rs")
        self.assertEqual(steps & {"browser", "canaries", "conformance canary"}, set())
        self.assertEqual(tests, "-p svrngs")

    def test_a_test_file_reaches_its_binary_alone(self):
        self.assertEqual(plan("crates/svrngs-node/tests/browser/keep_going.rs"), ({"browser", "playback", "clippy"}, ""))
        self.assertEqual(plan("crates/svrngs-node/tests/carried.rs"), ({"tests", "clippy"}, "-p svrngs-node --test carried"))
        self.assertEqual(plan("crates/svrngs-node/tests/seek_judgment.rs"), ({"tests", "clippy"}, "-p svrngs-node --test seek_judgment"))

    def test_what_the_cheap_steps_alone_read_reaches_no_slow_one(self):
        self.assertEqual(plan("docs/DESIGN.md", "scripts/lean_check.py"), (set(), ""))

    def test_node_resource_suites_and_their_helper_reach_node_integration(self):
        for path in ["web/tests/resolver.integration.test.mjs", "web/tests/page-server.integration.test.mjs", "web/tests/resolver-fixture.mjs"]:
            with self.subTest(path=path):
                self.assertEqual(plan(path), ({"page integration"}, ""))

    def test_resolver_sources_reach_the_real_store_contract(self):
        for path in ["resolver/handler.mjs", "resolver/store.mjs"]:
            with self.subTest(path=path):
                self.assertEqual(plan(path), ({"page integration"}, ""))

    def test_pure_question_contracts_do_not_require_native_binaries(self):
        self.assertEqual(plan("web/tests/question.test.mjs"), (set(), ""))
        self.assertEqual(plan("resolver/adapter.test.mjs"), (set(), ""))

    def test_shared_catalog_and_seek_instrument_keep_the_conservative_gate(self):
        self.assertIsNone(plan("web/tests/run.mjs"))
        self.assertIsNone(plan("crates/svrngs-node/tests/browser/load.rs"))

    def test_the_gate_itself_and_a_file_nothing_places_run_every_step(self):
        self.assertIsNone(plan("scripts/check.sh"))
        self.assertIsNone(plan("Cargo.lock"))
        self.assertIsNone(plan("notes-nothing-names.txt"))
        self.assertLessEqual(SITE, plan("web/tests/nothing-names-this.mjs")[0])


if __name__ == "__main__":
    unittest.main()
