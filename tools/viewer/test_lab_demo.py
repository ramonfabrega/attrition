import copy
import unittest
from lab_demo import matching_frames, witness, verify_checkpoint_windows


def data():
    row = dict(entity='city:0/2000', field='peasant_dist', original='2', rust='0', assessment='city comparator', weight=1)
    return dict(schema=1, frames=[dict(n=1900, index=1899, issues=1, compared=0,
        positionMismatches=0, units=[], differences=[row])])


class DemoTests(unittest.TestCase):
    def test_preserves_witness_and_complete_records(self):
        broad = data()
        matching_frames(broad, copy.deepcopy(broad))
        self.assertEqual(witness(broad)['original'], '2')
        changed = copy.deepcopy(broad)
        changed['frames'][0]['extra_state'] = 1
        with self.assertRaises(ValueError):
            matching_frames(broad, changed)

    def test_checkpoint_windows_reject_repeated_or_short_windows(self):
        broad = data()
        first = broad['frames'][0]
        broad['frames'] = [dict(copy.deepcopy(first), index=1899+i, n=1900+i) for i in range(9)]
        windows = [dict(schema=1, frames=broad['frames'][i:i+3]) for i in range(0, 9, 3)]
        verify_checkpoint_windows(broad, windows[0], windows)
        with self.assertRaises(ValueError):
            verify_checkpoint_windows(broad, windows[0], [windows[0]] * 3)
        with self.assertRaises(ValueError):
            verify_checkpoint_windows(broad, windows[0], windows[:2])

    def test_refuses_missing_or_agreeing_witness(self):
        for mutation in ['entity', 'rust']:
            changed = data()
            changed['frames'][0]['differences'][0][mutation] = 'other' if mutation == 'entity' else '2'
            with self.assertRaises(ValueError):
                witness(changed)


if __name__ == '__main__':
    unittest.main()
