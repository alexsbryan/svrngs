"""The touched tier's plans (scripts/touched.py), each for a change set whose
reach is known: what it runs, and above all what it may not skip. A step
running named test binaries is folded into `tests`, which runs their tests
docs/seams.tsv does not call fragile, so a plan names binaries, not steps."""
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
# The binaries that build the phone's site (harness.rs `site`).
SITE = {"browser", "conformance", "join_page", "pages", "scenes"}
EVERY = {"app laws", "clippy", "page integration", "tests", "typed lint", "types", "web rustfmt and clippy"}


def plan(*files):
    """The steps planned for `files`, and the `tests` selector."""
    out = subprocess.run([sys.executable, "scripts/touched.py", "--files", *files], cwd=ROOT,
                         capture_output=True, text=True, check=True).stdout
    steps, tests = out.split("SVRNGS_STEPS='")[1].split("' SVRNGS_RUN=")[0].split("' SVRNGS_TESTS='")
    return set(filter(None, steps.split("\n"))), tests


def binaries(tests):
    return {t.strip() for t in tests.split("--test ")[1:]}


class Plans(unittest.TestCase):
    def test_a_driver_reaches_the_binary_naming_it_alone(self):
        self.assertEqual(plan("web/tests/keeps-going.mjs"), ({"tests"}, "-p svrngs-node --test browser"))

    def test_a_helper_every_driver_reaches_reaches_every_binary_driving_them(self):
        steps, tests = plan("web/tests/page.mjs")
        self.assertIn("tests", steps)
        self.assertLessEqual(SITE, binaries(tests))

    def test_a_law_or_its_helper_is_the_cheap_tier_s(self):
        self.assertEqual(plan("web/tests/pages/enter.test.mjs", "web/tests/pages/layout.mjs"), (set(), ""))
        # One whose own import() could load anything, and only laws reach.
        self.assertEqual(plan("web/tests/pages/before.mjs"), (set(), ""))

    def test_the_phone_s_page_reaches_every_binary_building_the_site_and_the_types(self):
        steps, tests = plan("web/page/enter.js")
        self.assertEqual(steps, {"tests", "types", "typed lint"})
        self.assertLessEqual(SITE, binaries(tests))

    def test_core_reaches_its_dependents_and_the_wasm(self):
        steps, tests = plan("crates/svrngs-core/src/pass.rs")
        self.assertEqual(steps, {"web rustfmt and clippy", "clippy", "tests"})
        self.assertTrue(tests.startswith("-p svrngs -p svrngs-core -p svrngs-node -p svrngs-runtime --lib --bins --test"))
        self.assertLessEqual(SITE | {"canaries", "scale", "carried"}, binaries(tests))

    def test_the_command_s_crate_leaves_the_node_s_binaries_out(self):
        steps, tests = plan("crates/svrngs/src/main.rs")
        self.assertEqual(binaries(tests) & {"browser", "canaries", "conformance", "carried"}, set())
        self.assertTrue(tests.startswith("-p svrngs --lib --bins --test"))
        self.assertLessEqual({"scenes", "verbs", "n0"}, binaries(tests))

    def test_a_test_file_reaches_its_binary_alone(self):
        self.assertEqual(plan("crates/svrngs-node/tests/browser/keep_going.rs"), ({"tests", "clippy"}, "-p svrngs-node --test browser"))
        self.assertEqual(plan("crates/svrngs/tests/scenes/main.rs"), ({"tests", "clippy"}, "-p svrngs --test scenes"))
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
        for path in ["web/tests/run.mjs", "crates/svrngs-node/tests/browser/load.rs"]:
            with self.subTest(path=path):
                self.assertEqual(plan(path), plan("Cargo.lock"))

    def test_the_gate_itself_and_a_file_nothing_places_run_every_step(self):
        for path in ["scripts/check.sh", "Cargo.lock", "notes-nothing-names.txt"]:
            with self.subTest(path=path):
                steps, tests = plan(path)
                self.assertEqual(steps, EVERY)
                self.assertTrue(tests.startswith("-p give -p give-fold -p svrngs -p svrngs-core -p svrngs-node -p svrngs-runtime --lib --bins"))
                self.assertLessEqual(SITE | {"canaries", "scale", "n0", "cases"}, binaries(tests))
                self.assertNotIn("xenia", binaries(tests))  # no step of the gate runs it
        self.assertLessEqual(SITE, binaries(plan("web/tests/nothing-names-this.mjs")[1]))


if __name__ == "__main__":
    unittest.main()
