import os, sys, unittest
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import notes

HERE = os.path.join(os.path.dirname(__file__), "..", "fixtures")


class Notes(unittest.TestCase):
    def test_every_fixture_parses_whole(self):
        for f in sorted(os.listdir(HERE)):
            with self.subTest(f):
                r = notes.parse(open(os.path.join(HERE, f), "rb").read())
                self.assertEqual(r["missing"], [])

    def test_shape_says_no_words(self):
        raw = open(os.path.join(HERE, sorted(os.listdir(HERE))[0]), "rb").read()
        s = notes.shape(raw)
        self.assertEqual(set(s) - {"bytes", "bom", "newline", "high_bytes", "first_invalid_byte", "first_invalid_at_fraction"}, set())


if __name__ == "__main__":
    unittest.main()
