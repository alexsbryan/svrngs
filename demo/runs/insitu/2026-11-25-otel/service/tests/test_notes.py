import os, sys, unittest
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
import notes

HERE = os.path.join(os.path.dirname(__file__), "..", "fixtures")


class Notes(unittest.TestCase):
    def test_every_fixture_parses_whole(self):
        for f in sorted(os.listdir(HERE)):
            with self.subTest(f):
                form = notes.parse(open(os.path.join(HERE, f), "rb").read())
                self.assertEqual(form["cpt"].isdigit(), True)


if __name__ == "__main__":
    unittest.main()
