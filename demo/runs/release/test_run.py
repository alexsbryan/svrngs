"""Positive and negative controls for the release verdicts."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from run import MAC, Run, Verdict, identifies, judge, mac_verdicts


class ReleaseTests(unittest.TestCase):
    def test_failing_driver_makes_named_tier_red(self):
        with tempfile.TemporaryDirectory() as tmp:
            run = Run(Path(tmp), 'abc')
            with patch.object(run, 'clean', return_value=True):
                run.step('linux-ship', 'archive', [sys.executable, '-c', 'raise SystemExit(7)'])
            self.assertEqual(run.exit_code(), 1)
            self.assertIn('failed linux-ship: commit=abc build=archive; exit=7',
                          (run.logs / 'summary.txt').read_text())

    def test_a_successful_control_makes_tier_green(self):
        with tempfile.TemporaryDirectory() as tmp:
            run = Run(Path(tmp), 'abc')
            with patch.object(run, 'clean', return_value=True):
                code = 7 if os.environ.get('PLANT_RELEASE_DRIVER_FAILURE') else 0
                run.step('linux-ship', 'archive', [sys.executable, '-c', f'raise SystemExit({code})'])
            self.assertEqual(run.exit_code(), 0, (run.logs / 'summary.txt').read_text())

    def test_host_absence_and_wrong_candidate_are_never_green(self):
        log = 'release-candidate abc\n' + ''.join(
            f'release-case {case} 0 darwin/debug\n' for case in (*MAC, 'mac-cleanup'))
        self.assertTrue(all(v == Verdict.PASSED for _, v, _ in mac_verdicts(0, log, 'abc')))
        self.assertTrue(all(v == Verdict.FAILED for _, v, _ in mac_verdicts(0, log, 'def')))
        self.assertTrue(all(v == Verdict.MISSING for _, v, _ in mac_verdicts(255, 'ssh: unreachable', 'abc')))
        self.assertFalse(identifies('svrngs 1.2.3 (abc-dirty)', 'abc'))

    def test_inner_case_is_not_masked_by_driver_exit(self):
        self.assertEqual(judge(0, '{"event":"case","verdict":"never-ran"}'), Verdict.MISSING)
        self.assertEqual(judge(0, '{"event":"case","verdict":"could-not-judge"}'), Verdict.UNJUDGED)
        self.assertEqual(judge(0, '{"event":"case","verdict":"failed"}'), Verdict.FAILED)


if __name__ == '__main__':
    unittest.main()
