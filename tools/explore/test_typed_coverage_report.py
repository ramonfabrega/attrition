import unittest
from typed_coverage_report import summarize

class CoverageTests(unittest.TestCase):
    def fixture(self):
        rows=[dict(status='exact_integer_match',logger_path='WORLD',address=100,pdb_path='grid[0]') for _ in range(3)]
        rows.append(dict(status='unmapped',logger_path='WORLD'))
        return dict(snapshot=dict(sha256='toy')),dict(snapshot_sha256='toy',rows=rows,observable_occurrences=4,matched_occurrences=3,status_counts=dict(exact_integer_match=3,unmapped=1))
    def test_repetitions_do_not_inflate_storage_coverage(self):
        s,r=self.fixture();out=summarize(s,r)
        self.assertEqual(out['matched_occurrences'],3);self.assertEqual(out['compared_storage_references'],1)
        self.assertEqual(out['repeated_compared_occurrences'],2);self.assertIsNone(out['live_allocation_coverage'])
    def test_one_bad_copy_invalidates_storage_agreement(self):
        s,r=self.fixture();r['rows'][1]['status']='integer_mismatch';r['matched_occurrences']=2
        r['status_counts']=dict(exact_integer_match=2,integer_mismatch=1,unmapped=1)
        out=summarize(s,r);self.assertEqual(out['distinct_agreeing_storage_references'],0)
        self.assertEqual(out['distinct_mismatching_storage_references'],1)
    def test_missing_identity_or_false_total_refuses(self):
        s,r=self.fixture();del r['rows'][0]['address']
        with self.assertRaises(ValueError):summarize(s,r)
        s,r=self.fixture();r['matched_occurrences']=4
        with self.assertRaises(ValueError):summarize(s,r)

if __name__=='__main__':unittest.main()
