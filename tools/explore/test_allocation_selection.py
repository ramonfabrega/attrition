import unittest
from allocation_selection import select


def row(base, size=4096, allocation=None, state=0x1000, protect=4, kind=0x20000):
    return (base, base if allocation is None else allocation, 0, size, state, protect, kind)


class SelectionTests(unittest.TestCase):
    def setUp(self):
        self.rows = [row(0x400000, kind=0x1000000),
                     row(0x401000, allocation=0x400000, protect=0x20, kind=0x1000000),
                     row(0x1000000), row(0x1001000, allocation=0x1000000),
                     row(0x2000000), row(0x3000000), row(0x5000000)]
        self.kw = dict(main=0x400000, observer=0x3000000,
                       stack=(0x5000000, 0x5001000), roots={'unit': 0x1000020}, budget=128*1024**2)

    def test_deduplicates_roots_and_includes_all_eligible_allocation_spans(self):
        self.kw['roots']['type'] = 0x1001020
        result = select(self.rows, **self.kw)
        self.assertEqual(result['bytes'], 3*4096)
        self.assertEqual(result['allocations'], [0x400000, 0x1000000])
        self.assertFalse(result['closure_established'])

    def test_cap_boundary(self):
        self.kw['budget'] = 3*4096
        self.assertEqual(select(self.rows, **self.kw)['bytes'], self.kw['budget'])
        self.kw['budget'] -= 1
        with self.assertRaisesRegex(ValueError, 'exceeds'):
            select(self.rows, **self.kw)

    def test_absent_and_ineligible_roots(self):
        for address in (0, 0x401000, 0x3000000, 0x5000000, 0x1002000):
            with self.subTest(address=address), self.assertRaises(ValueError):
                select(self.rows, **{**self.kw, 'roots': {'world': address}})
        for state, protect, kind in ((0x2000,4,0x20000), (0x1000,0x104,0x20000),
                                     (0x1000,1,0x20000), (0x1000,4,0x40000)):
            altered = self.rows[:2] + [row(0x1000000,state=state,protect=protect,kind=kind)] + self.rows[3:]
            with self.subTest(state=state,protect=protect,kind=kind), self.assertRaises(ValueError):
                select(altered, **self.kw)

    def test_does_not_follow_unknown_pointer_or_claim_closure(self):
        result = select(self.rows, **self.kw)
        self.assertNotIn(0x2000000, result['allocations'])
        with self.assertRaises(ValueError):
            select(self.rows, **{**self.kw, 'roots': {}})

    def test_rejects_wrap_overlap_and_missing_image(self):
        for rows in (self.rows+[row(0xfffff000,size=8192)], self.rows+[self.rows[2]], self.rows[2:]):
            with self.assertRaises(ValueError):
                select(rows, **self.kw)


if __name__ == '__main__':
    unittest.main()
