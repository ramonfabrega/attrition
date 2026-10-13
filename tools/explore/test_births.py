"""`tools/gamelog/births.py`: a birth names the nearest building of its
owner (the twenty-eighth pass, from the race's Loop lines: two of the four
arms found the Terra Cotta Army by a hand-written radius scan over
`BUILDDATA` — oh 1654, om 1659)."""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BIRTHS = ROOT / 'tools/gamelog/births.py'


def dump(frames):
    out = []
    for n, recs in frames:
        out.append(f'BEGIN FRAME {n}')
        for kind, fields in recs:
            out.append(f'BEGIN {kind}')
            out += [f'{k} {v}' for k, v in fields]
            out.append(f'END {kind}')
        out.append('END FRAME')
    return '\n'.join(out) + '\n'


class ABirthNamesItsNearestBuilding(unittest.TestCase):
    def test_the_plus_line_names_the_owner_s_nearest_building(self):
        far = [('who', 1), ('o', 2001), ('uid', 7), ('x_internal', 1920), ('y_internal', 1920), ('orig_type', 300)]
        near = [('who', 1), ('o', 2042), ('uid', 8), ('x_internal', 38400), ('y_internal', 40320), ('orig_type', 59)]
        other = [('who', 2), ('o', 2003), ('uid', 9), ('x_internal', 38592), ('y_internal', 40320), ('orig_type', 61)]
        born = [('who', 1), ('o', 235), ('uid', 325), ('x_internal', 38592), ('y_internal', 40512), ('myhits', 100)]
        text = dump([(10, [('BUILDDATA', far), ('BUILDDATA', near), ('BUILDDATA', other)]),
                     (11, [('BUILDDATA', far), ('BUILDDATA', near), ('BUILDDATA', other), ('UNITDATA', born)])])
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'dump.txt'
            path.write_text(text)
            out = subprocess.run([sys.executable, str(BIRTHS), str(path)], capture_output=True, text=True, check=True).stdout
        plus = [line for line in out.splitlines() if line.startswith('  + UNITDATA')]
        self.assertEqual(len(plus), 1, out)
        self.assertIn('near=1/2042 otype=59 d=1.4', plus[0])


if __name__ == '__main__':
    unittest.main()
