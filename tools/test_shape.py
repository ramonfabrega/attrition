"""`tools/shape.py` — the structure track's six counts (item 4008).

A small crate in a temporary directory, with each count's edge on it:

(a) `flat` counts the root's `.rs` files only, and not `air.rs`, which
    is `air/`'s module file;
(b) `test-tails` counts a production file's `#[cfg(test)] mod … { … }`
    lines, attribute to closing brace, and never a test-mounted file's —
    neither one `lib.rs` mounts under `#[cfg(test)]` (with or without
    `#[path]`), nor one under a test-mounted directory;
(c) `sweep-lines` counts `wc -l` lines under `sweep/`;
(d) `names` counts plain-`pub` const and static names defined twice or
    more, outside `sweep/`, across both crates;
(e) `fields` counts `Sim`'s four-space fields, not its doc comments,
    attributes or a nested struct's fields;
(f) `largest` is the longest production file by `wc -l`, a test file
    excluded however long — `air.rs`'s `#[cfg(test)] mod tests;` makes
    `air/tests.rs` one;
(g) the line rounds the two line counts to thousands, half up;
(h) `largest` counts lines of code: the rows strictly inside an
    item-level `const`/`static` array literal are not counted, its two
    bracket lines are, and `--json`'s `tables` is the rows left out (the
    seventy-fifth pass's ruling);
(i) `flat` does not count a root `foo.rs` beside a `foo/` directory —
    that directory's module file (the seventy-fifth pass's ruling).
"""
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import shape  # noqa: E402

LIB = '''pub mod orders;
pub mod air;
#[cfg(test)]
#[path = "harness_tests.rs"]
mod tests;
#[cfg(test)]
mod sweep;
#[cfg(test)]
mod guard;

pub struct Sim {
    /// The tick.
    pub frame: u32,
    #[doc(hidden)]
    pub(crate) rng: u64,
    units: Vec<u8>,
}

pub struct Other {
    pub not_a_sim_field: u8,
}
'''

ORDERS = '''pub const ONE: i32 = 1;
pub const TWO: i32 = 2;
pub(crate) const PRIVATE: i32 = 3;
fn f() {}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {}
}
'''

FILES = {
    'sim/src/lib.rs': LIB,
    'sim/src/orders.rs': ORDERS,
    'sim/src/air.rs': 'pub const ONE: i32 = 1;\npub static TWO: i32 = 2;\n#[cfg(test)]\nmod tests;\n',
    'sim/src/air/tests.rs': '\n' * 40,
    'sim/src/air/think.rs': 'pub const PRIVATE: i32 = 3;\n',
    'sim/src/harness_tests.rs': '\n' * 50 + '#[cfg(test)]\nmod inner {\n}\n',
    'sim/src/guard.rs': '#[cfg(test)]\nmod tests {\n}\n',
    'sim/src/sweep.rs': 'mod one;\n',
    'sim/src/sweep/one.rs': 'pub const ONE: i32 = 1;\nfn s() {}\n',
    'rondata/src/lib.rs': 'pub const TWO: i32 = 2;\npub const THREE: i32 = 3;\n',
}


def crate(files):
    """A temporary directory holding `files`, and its `sim/src`."""
    tmp = tempfile.TemporaryDirectory()
    root = Path(tmp.name)
    for rel, text in files.items():
        path = root / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    return tmp, root / 'sim/src'


TABLE = ('fn a() {}\nfn b() {}\n'
         'pub const T: &[(i32, i32)] = &[\n'
         + ''.join(f'    ({i}, {i}),\n' for i in range(10))
         + '];\nfn c() {}\n')


class RulingsTest(unittest.TestCase):
    """The seventy-fifth pass's two rulings on the counters."""

    def test_largest_leaves_out_a_table_s_rows(self):
        tmp, sim = crate({'sim/src/lib.rs': TABLE})
        try:
            self.assertEqual(shape.lines(sim / 'lib.rs'), 15)
            self.assertEqual(shape.largest(sim), 5)
            self.assertEqual(shape.tables(sim), 10)
        finally:
            tmp.cleanup()

    def test_flat_leaves_out_a_directory_s_module_file(self):
        tmp, sim = crate({
            'sim/src/lib.rs': 'pub mod foo;\npub mod bar;\n',
            'sim/src/foo.rs': 'pub mod x;\n',
            'sim/src/foo/x.rs': 'fn x() {}\n',
            'sim/src/bar.rs': 'fn bar() {}\n',
        })
        try:
            self.assertEqual(shape.flat(sim), 2)
        finally:
            tmp.cleanup()


class ShapeTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        for rel, text in FILES.items():
            path = root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        self.sim = root / 'sim/src'
        self.rondata = root / 'rondata/src'

    def tearDown(self):
        self.tmp.cleanup()

    def test_the_six_counts(self):
        self.assertEqual(shape.measure(self.sim, self.rondata), {
            'flat': 4,
            'test-tails': 5,
            'sweep-lines': 3,
            'names': 2,
            'fields': 3,
            'largest': 21,
            'tables': 0,
        })

    def test_the_line(self):
        counts = {'flat': 203, 'test-tails': 64958, 'sweep-lines': 148079,
                  'names': 130, 'fields': 92, 'largest': 14222}
        self.assertEqual(
            shape.line(counts),
            'Structure: flat 203 · test-tails 65k · sweep-lines 148.1k · '
            'names 130 · fields 92 · largest 14222')
        self.assertEqual(shape.thousands(149950), '150k')
        self.assertEqual(shape.thousands(65172), '65.2k')
        self.assertIn('test-tails 64958', shape.line(counts, exact=True))


FLOORS = '''/// The pin.
pub const STRUCTURE: [(&str, u64); 6] = [
    ("flat", 4),
    ("test-tails", 5),
    ("sweep-lines", 3),
    ("names", 2),
    ("fields", 3),
    ("largest", 21),
];

pub const STRUCTURE_LINE_BEFORE_SHAPE: Option<&str> = None;
'''


class CheckTest(unittest.TestCase):
    """`--check` (Loop 4070): the counts against floors.rs's `STRUCTURE`,
    so the gate's first step and the reflex hold the structure pin with
    nothing built."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        for rel, text in FILES.items():
            path = root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        self.sim = root / 'sim/src'
        self.rondata = root / 'rondata/src'

    def tearDown(self):
        self.tmp.cleanup()

    def test_the_pin_reads_with_its_digit_separators(self):
        self.assertEqual(shape.pinned(FLOORS.replace('("largest", 21)', '("largest", 3_721)')),
                         [('flat', 4), ('test-tails', 5), ('sweep-lines', 3), ('names', 2),
                          ('fields', 3), ('largest', 3721)])

    def test_the_tree_that_is_the_pin_passes(self):
        self.assertEqual(shape.check(shape.measure(self.sim, self.rondata), shape.pinned(FLOORS)), [])

    def test_an_inline_test_tail_in_a_production_file_is_refused(self):
        # 4063's shape: a new production file ending in a `#[cfg(test)] mod
        # tests { … }` tail, which `test-tails` counts and the pin holds.
        (self.sim / 'starting.rs').write_text(
            'pub fn place() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n')
        moved = shape.check(shape.measure(self.sim, self.rondata), shape.pinned(FLOORS))
        self.assertEqual(moved, ['flat: the tree has 5, STRUCTURE pins 4 (+1)',
                                 'test-tails: the tree has 10, STRUCTURE pins 5 (+5)'])

    def test_an_unreadable_pin_is_an_error_not_agreement(self):
        for text in ('', FLOORS.replace('("names", 2),\n', ''),
                     FLOORS.replace('("names", 2)', '("names", NAMES)')):
            with self.assertRaises(ValueError):
                shape.pinned(text)

    def test_the_tree_s_own_pin_reads(self):
        self.assertEqual([k for k, _ in shape.pinned(shape.FLOORS.read_text())], list(shape.ORDER))


if __name__ == '__main__':
    unittest.main()
