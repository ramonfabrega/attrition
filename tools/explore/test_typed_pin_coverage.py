import unittest
from typed_pin_coverage import parse_pin,compare_pin

class PinTests(unittest.TestCase):
    source='const UNREAD: &[(&str, &str)] = &[\n // old count 999\n ("WORLD", "a b c"),\n];'
    def test_comments_are_not_counts_and_absence_is_not_agreement(self):
        r=dict(snapshot_sha256='toy',frame=1,rows=[dict(logger_path='WORLD',key='a',status='exact_integer_match'),dict(logger_path='WORLD',key='b',status='integer_mismatch')])
        out=compare_pin(self.source,r)
        self.assertEqual(out['pin_keys'],3)
        self.assertEqual(out['counts'],dict(all_observed_occurrences_match=1,not_fully_matched=1,absent=1))
        r['rows'].append(dict(logger_path='WORLD',key='a',status='unmapped'))
        self.assertNotIn('all_observed_occurrences_match',compare_pin(self.source,r)['counts'])
    def test_unknown_syntax_and_duplicate_keys_refuse(self):
        for source in (self.source.replace('"a b c"','concat!("a", "b")'),self.source.replace('a b c','a a'),self.source.replace('("WORLD", "a b c"),','')):
            with self.assertRaises(ValueError):parse_pin(source)
    def test_comment_marker_inside_string_is_preserved(self):
        self.assertEqual(parse_pin(self.source.replace('WORLD','WORLD//FOO'))[0][0],'WORLD//FOO')

if __name__=='__main__':unittest.main()
