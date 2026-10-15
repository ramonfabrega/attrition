"""`tools/trace/report.py`'s frame arguments (Loop 2512, the thirty-fifth pass).

`draws <frame>` re-reads the whole trace — 277 MB, about a minute — on
every call, and 2460's loop over two hundred frames ran out the foreground
limit. A window is one argument, `lo-hi`, read in one pass.
"""
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('report', ROOT / 'tools/trace/report.py')
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


class FrameArguments(unittest.TestCase):
    def test_a_window_is_one_argument(self):
        self.assertEqual(report.frame_args(['setup', '5', '7-9']), {-1, 5, 7, 8, 9})
        self.assertEqual(report.frame_args(['4859-4859']), {4859})
        with self.assertRaises(ValueError):
            report.frame_args(['9-7'])


if __name__ == '__main__':
    unittest.main()
