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


class TheSplitColumn(unittest.TestCase):
    # Loop 3093 (1725): a sweep that asserts a raw word backs the layout;
    # a function a document diffs or the crate spells is port-backed.
    def test_port_or_layout(self):
        by_doc, by_sweep, spelled = {0x10}, {0x10, 0x20, 0x30}, {'Unit::hits'}
        self.assertTrue(census.is_port_backed(0x10, 'Unit::move_step', by_doc, by_sweep, spelled))
        self.assertTrue(census.is_port_backed(0x20, 'Unit::hits', by_doc, by_sweep, spelled))
        self.assertFalse(census.is_port_backed(0x30, 'Unit::is_tank', by_doc, by_sweep, spelled))
        self.assertFalse(census.is_port_backed(0x40, 'Unit::hits', by_doc, by_sweep, spelled))

    def test_the_line_leads_with_the_total(self):
        self.assertEqual(census.census_line(1459, 900, 3611),
                         'Census: simulation backed 1459 of 3611 · port 900 · layout 559')


class TheUnbackedList(unittest.TestCase):
    # Parked 1703 (four scratch listers) and 1725 (offset identities
    # counted as backed): the lister ranks the never-backed simulation
    # functions shortest first and marks a port consumer and an operator.
    def test_shortest_first_with_the_marks(self):
        sim = M + 'game\\unit.cpp'
        entries = [
            (0x10, 'Unit::is_tank', sim, 300),
            (0x20, 'Unit::get_captain', sim, 120),
            (0x30, 'TCoord::operator<', M + 'game\\coord.h', 90),
            (0x40, 'Unit::move_step', sim, 5000),
            (0x50, 'Render::draw', M + 'game\\rendersystem.cpp', 10),
            (0x60, 'Unit::hits', sim, 80),
        ]
        # Loop 2286 (the thirty-fourth pass): `port` needs the crate to spell
        # the function's own `Class::method`; a name match alone is `name`.
        rows = census.rank_unbacked(entries, backed={0x40}, port={'is_tank', 'hits'},
                                    spelled={'Unit::hits'})
        self.assertEqual([(r[2], r[3]) for r in rows],
                         [('Unit::hits', 'port'), ('TCoord::operator<', 'operator'),
                          ('Unit::get_captain', '-'), ('Unit::is_tank', 'name')])
        self.assertEqual(rows[0][0], 80)
        self.assertEqual(rows[1][4], 'coord.h')

    def test_a_camel_case_method_is_its_snake_case_port(self):
        self.assertEqual(census.snake('getCaptain'), 'get_captain')
        self.assertEqual(census.snake('is_tank'), 'is_tank')


class TheByteColumn(unittest.TestCase):
    # The count weighs a two-line getter as much as `PathFinder::find_tpath`:
    # a reader measured the backed functions at 43.8 % of the count and
    # about 23.5 % of the decompile's bytes. The byte column says which.
    SIM = M + 'game\\unit.cpp'
    REN = M + 'game\\rendersystem.cpp'

    def rows(self):
        names = {0x10: 'Unit::a', 0x20: 'Unit::b', 0x30: 'Unit::c', 0x40: 'Render::d'}
        sizes = {0x10: 100, 0x20: 300, 0x30: 600, 0x40: 50}
        source = {0x10: self.SIM, 0x20: self.SIM, 0x30: self.SIM, 0x40: self.REN}
        return census.layer_rows(names, sizes, source, by_doc={0x20}, by_sweep={0x10, 0x40},
                                 spelled={'Unit::a'})

    def test_each_layer_carries_its_bytes_and_its_backed_bytes(self):
        rows, sim_files, _unknown = self.rows()
        sim = rows['simulation']
        self.assertEqual((sim['functions'], sim['backed'], sim['port']), (3, 2, 2))
        self.assertEqual((sim['bytes'], sim['backed_bytes']), (1000, 400))
        self.assertEqual((rows['engine']['bytes'], rows['engine']['backed_bytes']), (50, 50))
        self.assertEqual(sim_files['unit.cpp'], 3)

    def test_the_bytes_line_follows_the_census_line_unchanged(self):
        import io
        from contextlib import redirect_stdout
        rows, sim_files, unknown = self.rows()
        out = io.StringIO()
        with redirect_stdout(out):
            census.print_layers(rows, sim_files, unknown, False, size_note="bytes: the test's own")
        lines = out.getvalue().splitlines()
        at = lines.index('Census: simulation backed 2 of 3 · port 2 · layout 0')
        self.assertEqual(lines[at + 1], 'Census bytes: simulation 400 of 1000 bytes backed · 40.0%')
        self.assertIn("# bytes: the test's own", lines)
        head = next(l for l in lines if l.startswith('layer'))
        for col in ('bytes', 'backed bytes', 'byte share'):
            self.assertIn(col, head)
        sim = next(l for l in lines if l.startswith('simulation '))
        self.assertTrue(sim.rstrip().endswith('40.0%'), sim)

    def test_the_json_carries_the_bytes(self):
        import io
        import json
        from contextlib import redirect_stdout
        rows, sim_files, unknown = self.rows()
        out = io.StringIO()
        with redirect_stdout(out):
            census.print_layers(rows, sim_files, unknown, True, size_note='bytes: decompile')
        got = json.loads(out.getvalue())
        self.assertEqual((got['simulation_bytes'], got['simulation_backed_bytes']), (1000, 400))
        self.assertEqual(got['bytes_source'], 'bytes: decompile')
        self.assertEqual(got['layers']['simulation']['backed_bytes'], 400)
        # `rondata::census`'s reader matches `"simulation_backed":` whole.
        self.assertEqual(got['simulation_backed'], 2)
        self.assertEqual(out.getvalue().count('"simulation_backed":'), 1)

    def test_a_symbol_size_in_the_index_wins_over_the_decompile_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, 'funcs', 'Unit').mkdir(parents=True)
            Path(tmp, 'funcs', 'Unit', 'a@00000010.c').write_text('x' * 70)
            Path(tmp, 'INDEX.tsv').write_text(
                '00000010\tUnit::a\tfuncs/Unit/a@00000010.c\n'
                '00000020\tUnit::b\tfuncs/Unit/b@00000020.c\t42\n'
                '00000030\tUnit::c\tfuncs/Unit/c@00000030.c\n')
            got = census.index_entries(str(Path(tmp, 'INDEX.tsv')))
        self.assertEqual([(e[0], e[3], e[4]) for e in got],
                         [(0x10, 70, 'decompile'), (0x20, 42, 'symbol'), (0x30, 0, 'decompile')])
        from collections import Counter
        self.assertIn('decompile file', census.size_note(Counter(['decompile'])))
        self.assertIn('no symbol size', census.size_note(Counter(['decompile'])))
        both = census.size_note(Counter(['decompile', 'symbol', 'symbol']))
        self.assertIn('symbol', both)
        self.assertIn('2', both)

    def test_the_unbacked_list_by_bytes_is_largest_first(self):
        entries = [(0x10, 'Unit::is_tank', self.SIM, 300), (0x20, 'Unit::get_captain', self.SIM, 120),
                   (0x40, 'Unit::move_step', self.SIM, 5000), (0x60, 'Unit::hits', self.SIM, 80)]
        short = census.rank_unbacked(entries, backed=set(), port=set())
        big = census.rank_unbacked(entries, backed=set(), port=set(), by='bytes')
        self.assertEqual([r[0] for r in short], [80, 120, 300, 5000])
        self.assertEqual([r[0] for r in big], [5000, 300, 120, 80])


if __name__ == '__main__':
    unittest.main()
