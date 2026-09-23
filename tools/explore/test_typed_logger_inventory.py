import tempfile,unittest
from pathlib import Path
from typed_logger_inventory import extract,inventory

class InventoryTests(unittest.TestCase):
    def get(self,text,**kwargs):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'log';p.write_bytes(text.encode());return extract(p,24,**kwargs)
    def test_duplicates_empty_values_and_ambiguous_lines_stay_visible(self):
        text=' BEGIN FRAME 24\r\n  BEGIN WORLD\r\n   x 3\r\n   x 3\r\n  flags 7\r\n  Napata\r\n\r\n BEGIN FRAME 25\r\n'
        r=inventory(self.get(text),24)
        self.assertEqual((r['total_lines'],r['observable_occurrences'],r['token_counts']['x']),(6,4,2))
        self.assertEqual(r['rows'][2]['ownership'],'ambiguous_indentation')
        self.assertEqual(r['rows'][3]['status'],'unclassified_text')
        self.assertFalse(r['logger_parity_established']);self.assertEqual(r['matched_occurrences'],0)
    def test_missing_truncated_empty_or_duplicate_frame_refuses(self):
        for text in ('',' BEGIN FRAME 24\n x 1\n',' BEGIN FRAME 24\n BEGIN FRAME 25\n',
                     ' BEGIN FRAME 24\n x 1\n BEGIN FRAME 25\n BEGIN FRAME 24\n x 2\n BEGIN FRAME 25\n'):
            with self.subTest(text=text),self.assertRaises(ValueError):self.get(text)
        with self.assertRaises(ValueError):inventory(self.get(' BEGIN FRAME 24\n  BEGIN WORLD\n BEGIN FRAME 25\n'),24)
    def test_bounds(self):
        text=' BEGIN FRAME 24\n x 1\n y 2\n BEGIN FRAME 25\n'
        with self.assertRaises(ValueError):self.get(text,max_lines=1)
        with self.assertRaises(ValueError):self.get(text,max_bytes=2)

if __name__=='__main__':unittest.main()
