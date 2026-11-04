import datetime
import unittest
from scripts.advisories import exceptions


class Exceptions(unittest.TestCase):
    def test_expired_and_unexplained_exceptions_fail_closed(self):
        today = datetime.date(2026, 11, 2)
        for item in ["RUSTSEC-2024-0000", {"id": "RUSTSEC-2024-0000"},
                     {"id": "RUSTSEC-2024-0000", "reason": "review by 2026-11-02: due today"},
                     {"id": "RUSTSEC-2024-0000", "reason": "review by 2026-11-01: expired"}]:
            with self.assertRaises(ValueError):
                exceptions({"advisories": {"ignore": [item]}}, today)
        exceptions({"advisories": {"ignore": [{"id": "RUSTSEC-2024-0000", "reason": "review by 2026-12-01: explicit reason"}]}}, today)


if __name__ == "__main__":
    unittest.main()
