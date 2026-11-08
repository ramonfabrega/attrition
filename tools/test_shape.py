"""`tools/shape.py` — the structure track's six counts (item 4008).

A small crate in a temporary directory, with each count's edge on it:

(a) `flat` counts the root's `.rs` files only;
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
(g) the line rounds the two line counts to thousands, half up.
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
    'sim/src/sweep/mod.rs': 'mod one;\n',
    'sim/src/sweep/one.rs': 'pub const ONE: i32 = 1;\nfn s() {}\n',
    'rondata/src/lib.rs': 'pub const TWO: i32 = 2;\npub const THREE: i32 = 3;\n',
}


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
            'flat': 5,
            'test-tails': 5,
            'sweep-lines': 3,
            'names': 2,
            'fields': 3,
            'largest': 21,
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


if __name__ == '__main__':
    unittest.main()
