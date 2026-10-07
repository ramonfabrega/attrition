"""`tools/census.py --layers`: the layer rule and the backed column (item 1467).

Each rule is written down in the tool's docstring; these hold it to
fixtures — a source path per layer, a coverage section's spans, a sweep
test's bounds — and never read the export, the PDB or `docs/`.
"""
import importlib.util
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('census', ROOT / 'tools/census.py')
census = importlib.util.module_from_spec(spec)
spec.loader.exec_module(census)
M = 'e:\\agent\\_work\\2\\s\\main\\'


class TheLayerRule(unittest.TestCase):
    def test_each_rule_in_order(self):
        cases = {
            'c:\\program files (x86)\\microsoft visual studio 14.0\\vc\\include\\xmemory': 'engine',
            M + 'zlib\\inflate.c': 'engine',
            M + 'basic\\array.h': 'engine',
            M + 'bighuge\\bhfile.cpp': 'engine',
            M + 'game\\minizip\\ioapi.c': 'engine',
            M + 'game\\script\\lexer.cpp': 'AI',
            M + 'game\\leaders.cpp': 'AI',
            M + 'game\\scriptfunctions.cpp': 'AI',
            M + 'game\\ifacemain.cpp': 'interface',
            M + 'game\\conquestwin.cpp': 'interface',
            M + 'game\\scripteditbox.cpp': 'interface',
            M + 'game\\statwin.cpp': 'interface',
            M + 'game\\rendersystem.cpp': 'engine',
            M + 'game\\gamelog.cpp': 'engine',
            M + 'game\\unit.cpp': 'simulation',
            M + 'game\\orders.h': 'simulation',
            M + 'game\\pathfinder.cpp': 'simulation',
            M + 'stray.cpp': 'unknown',
            None: 'unknown',
        }
        for path, layer in cases.items():
            with self.subTest(path=path):
                self.assertEqual(census.layer_of(path), layer)


INDEX = {'Unit::move_step': [0x5f0000], 'Guy::move': [0x600000], 'Wall::init': [0x610000],
         'Twin::a': [0x620000, 0x620010]}


class TheBackedColumn(unittest.TestCase):
    def test_a_coverage_section_s_diff_span_is_backed_and_nothing_else(self):
        doc = '''# Thing

Cited outside coverage: `x@005e0000`.

## 3. Coverage

**Diff-backed** — run10, field for field:

- the walk `Unit::move_step`, and `f@00610000` on every frame.

**Listing-backed**: `h@00640000` — a listing closes the span as a reading does.

**Reading-only**: `Guy::move`, and `g@00630000`.

**Dump-backed**: `Twin::a` names two functions and counts as neither.

## 4. Next

**Diff-backed**: `y@00650000` is outside a coverage section.
'''
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, 'THING.md').write_text(doc)
            got = census.doc_backed(tmp, INDEX)
        self.assertEqual(got, {0x5f0000, 0x610000})

    def test_a_sweep_is_an_emulator_test_s_own_citations(self):
        src = '''
fn helper() { /* `z@00700000` under tools/emu/ */ }

/// The original under the emulator (`tools/emu/callfn.py`): `Wall::init`.
#[test]
fn the_emulated_wall() {
    let a = "s@00710000";
}

/// A plain unit test of `Guy::move`.
#[test]
fn a_plain_test() {
    let b = "t@00720000";
}
'''
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, 'wall.rs').write_text(src)
            Path(tmp, 'docs_guard.rs').write_text(
                '/// EMULATOR table\n#[test]\nfn guard() { let c = "u@00730000"; }\n')
            got = census.sweep_backed(tmp, INDEX)
        self.assertEqual(got, {0x610000, 0x710000})


if __name__ == '__main__':
    unittest.main()
