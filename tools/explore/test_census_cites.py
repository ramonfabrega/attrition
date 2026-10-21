"""The census's per-document count of never-entered cites (Loop 3415, the
forty-eighth pass): `--pin --never --cites DOC` prints the functions DOC
cites that no pinned trace entered, net of `rondata::blind::RESIDUE`, marks
the ones no other document cites `only here`, and ends on the line a pass
quotes when it books a reading item — so the booking and the lane count one
set the same way (3408, 3409, 3412 each counted theirs differently)."""
import importlib.util
import os
import pathlib
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('census', ROOT / 'tools' / 'census.py')
census = importlib.util.module_from_spec(spec)
spec.loader.exec_module(census)

RESIDUE_RS = '''
pub const TRACES: &[&str] = &[
    "run1.log",
];

pub const RESIDUE: &[(u32, &str)] = &[
    (
        0x0040_3000,
        "Foo::dead: a vtable slot no instruction dispatches (+0x1ac, 0x0050_0000 in prose)",
    ),
    (0x0040_5000, "Bar::gone: one line"),
];

pub const OTHER: &[(u32, &str)] = &[
    (0x0040_6000, "not residue"),
];
'''


class TheCitesCount(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        d = self.tmp.name
        os.makedirs(os.path.join(d, 'docs', 'journal'))
        with open(os.path.join(d, 'docs', 'A.md'), 'w') as f:
            f.write('Foo::live@00401000 Foo::dead@00403000 Bar::shared@00404000\n'
                    'Bar::gone@00405000 Baz::alone@00406000 Qux::entered@00402000\n'
                    'Nope::absent@00409000\n')
        with open(os.path.join(d, 'docs', 'journal', 'B.md'), 'w') as f:
            f.write('Bar::shared@\n  00404000 and Bar::gone@00405000 and Qux::entered@00402000\n')
        self.docs = os.path.join(d, 'docs')
        self.rs = os.path.join(d, 'blind.rs')
        with open(self.rs, 'w') as f:
            f.write(RESIDUE_RS)
        self.name_of = {0x401000: 'Foo::live', 0x402000: 'Qux::entered', 0x403000: 'Foo::dead',
                        0x404000: 'Bar::shared', 0x405000: 'Bar::gone', 0x406000: 'Baz::alone'}
        self.seen = {0x402000}

    def tearDown(self):
        self.tmp.cleanup()

    def test_residue_is_read_from_its_own_const_only(self):
        self.assertEqual(census.residue_addrs(self.rs), {0x403000, 0x405000})

    def test_cites_are_kept_per_document_and_their_union_is_cited_in(self):
        per = census.cited_per_doc(self.docs)
        a = os.path.realpath(os.path.join(self.docs, 'A.md'))
        b = os.path.realpath(os.path.join(self.docs, 'journal', 'B.md'))
        self.assertEqual(per[a], {0x401000, 0x402000, 0x403000, 0x404000, 0x405000, 0x406000, 0x409000})
        self.assertEqual(per[b], {0x402000, 0x404000, 0x405000})
        self.assertEqual(set().union(*per.values()), census.cited_in(self.docs))

    def test_the_counts_and_the_marks(self):
        per = census.cited_per_doc(self.docs)
        residue = census.residue_addrs(self.rs)
        rows, n, m, k = census.doc_never(os.path.join(self.docs, 'A.md'), per, self.name_of,
                                         self.seen, residue)
        marks = {name: mark for _, name, mark in rows}
        # Entered (Qux) and not in the export (Nope) are not never.
        self.assertEqual(marks, {'Foo::live': 'only here', 'Foo::dead': 'residue',
                                 'Bar::shared': '-', 'Bar::gone': 'residue', 'Baz::alone': 'only here'})
        self.assertEqual((n, m, k), (5, 3, 2))
        self.assertEqual(census.cites_line('docs/A.md', n, m, k),
                         'cites docs/A.md: 5 never, 3 net of residue, 2 only here')
        b_rows, bn, bm, bk = census.doc_never(os.path.join(self.docs, 'journal', 'B.md'), per,
                                              self.name_of, self.seen, residue)
        self.assertEqual((bn, bm, bk), (2, 1, 0))
        self.assertEqual({name: mark for _, name, mark in b_rows},
                         {'Bar::shared': '-', 'Bar::gone': 'residue'})

    def test_a_document_not_found_is_refused(self):
        per = census.cited_per_doc(self.docs)
        with self.assertRaises(SystemExit):
            census.doc_never(os.path.join(self.docs, 'MISSING.md'), per, self.name_of, self.seen, set())


if __name__ == '__main__':
    unittest.main()
