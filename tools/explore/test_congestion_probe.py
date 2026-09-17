import unittest
from unittest.mock import patch
from pathlib import Path
from test_search_census import fixture as census_fixture
from congestion_probe import scenario, schedule, report


def fixture():
    rows = [(2, f, f*17, 0, 0, 0, 0, f) for f in range(1401)]
    for i, f in enumerate([37]+list(range(150,182))+[1400]):
        rows.append((5,9,f,i,0,1,0,f))
    for i in range(32):
        rows.append((5,140,i%2,i+1,i+100,3000,32000,210))
    rows.append((5,141,16,16,3000,32000,200,210))
    for i in range(10):
        f=220+200*(i//2)+i%2
        rows.append((5,142,i%2,16,10,67,f,f))
        direction = 1 if (i%2+i//2)%2 else -1
        rows.append((5,143,i%2,3000+direction*1536,32000,0,0,f))
    return rows


class ScenarioTests(unittest.TestCase):
    def test_projection_detects_drift_and_ignores_process_addresses(self):
        rows = fixture()
        metadata = census_fixture()
        rows = metadata[:2]+rows[:8]+metadata[3:]+rows[8:]
        def get(data):
            with patch('congestion_probe.records', return_value=iter(data)):
                return report(Path('authored'))
        baseline = get(rows)
        changed = list(rows)
        i = next(i for i,r in enumerate(rows) if r[0] == 2)
        r = list(changed[i]); r[2] += 1; changed[i] = tuple(r)
        self.assertNotEqual(get(changed)['projections']['frames'], baseline['projections']['frames'])
        changed = list(rows)
        i = next(i for i,r in enumerate(rows) if r[:2] == (5,131))
        r = list(changed[i]); r[5] += 1; changed[i] = tuple(r)
        self.assertNotEqual(get(changed)['projections']['metadata'], baseline['projections']['metadata'])
        changed = []
        for r in rows:
            row = list(r)
            if r[:2] == (5,130): row[3] += 4096
            if r[:2] == (5,131): row[4] += 4096; row[6] += 4096
            if r[:2] == (5,132): row[4] += 4096
            changed.append(tuple(row))
        self.assertEqual(get(changed), baseline)

    def test_complete(self):
        r=scenario(fixture())
        self.assertEqual(len(r['orders']),10)
        self.assertEqual(len(schedule().splitlines()),34)
        self.assertIn('181 !add hoplite who=0 17,173',schedule())

    def test_each_missing_receipt(self):
        rows=fixture()
        for i, row in enumerate(rows):
            if row[0] == 5:
                with self.subTest(tag=row[1], index=i), self.assertRaises(ValueError):
                    scenario(rows[:i]+rows[i+1:])

    def test_failures(self):
        for tag in (10,144):
            with self.assertRaises(ValueError):
                scenario(fixture()+[(5,tag,1,0,0,0,0,220)])
        with self.assertRaises(ValueError):
            scenario(fixture()[1:])

    def test_rejected_packet_and_duplicate_identity(self):
        rows=fixture()
        for tag, field, value in ((142,5,10),(140,3,2),(143,3,0),(9,5,0)):
            changed=list(rows)
            i=next(i for i,r in enumerate(rows) if r[:2] == (5,tag))
            row=list(changed[i]); row[field]=value; changed[i]=tuple(row)
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                scenario(changed)


if __name__ == '__main__':
    unittest.main()
