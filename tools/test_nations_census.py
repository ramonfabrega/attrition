"""`tools/nations_census.py --check` (Loop 3972): the generator's tables
against the ones on the tree, every differing row named, and a skip — never
a failure — on a machine without the decompile export or the install.

On `864dd6aec` the generator rewrote 24 rows no item had touched: the table
had drifted from what the tool emits, so a lane that ran it to re-pin one
row carried the rest. The check is the guard that the round trip is empty."""
import importlib.util
import io
import pathlib
import tempfile
import unittest
from contextlib import redirect_stdout

ROOT = pathlib.Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('nations_census', ROOT / 'tools' / 'nations_census.py')
nc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(nc)

DOC = """# Nations

prose that is never generated
<!-- GENERATED:hand -->
| nation | arm | original | status | witness |
|---|---|---|---|---|
| Japanese | carriers train faster | `ObjectData::train_time@006508c0` | modelled at lib.rs | `japanese_carrier_speed` |
| Lakota | raze speed | `ObjectData::train_time@006508c0` | modelled at queue_disband.rs | `lakota_raze_speed` |

more prose
<!-- GENERATED:consts1 -->
| constant | rules | the export reads it in | status | witness |
|---|---:|---|---|---|
| `INCA_REFUND` | 25 | Object::take_damage | modelled at fight.rs | `inca_refund` |
| `AZTEC_X` | 1 | no reader in the export | carried, unread | `aztec_x` |
"""

GEN = """<!-- GENERATED:hand -->
| nation | arm | original | status | witness |
|---|---|---|---|---|
| Japanese | carriers train faster | `ObjectData::train_time@006508c0` | modelled at lib.rs | `japanese_carrier_speed` |
| Lakota | raze speed | `ObjectData::train_time@006508c0` | not modelled | `lakota_raze_speed` |
<!-- GENERATED:consts1 -->
| constant | rules | the export reads it in | status | witness |
|---|---:|---|---|---|
| `INCA_REFUND` | 25 | Object::take_damage | modelled at plunder.rs | `inca_refund` |
| `MAYA_Y` | 2 | no reader in the export | not carried | `maya_y` |"""


class TheDrift(unittest.TestCase):
    def test_every_differing_row_is_named_by_its_block_and_key(self):
        rows = nc.drift(DOC, GEN)
        self.assertEqual([(b, k) for b, k, _, _ in rows], [
            ('hand', 'Lakota | raze speed'),
            ('consts1', '`INCA_REFUND`'),
            ('consts1', '`AZTEC_X`'),
            ('consts1', '`MAYA_Y`'),
        ])
        lakota = rows[0]
        self.assertIn('modelled at queue_disband.rs', lakota[2])
        self.assertIn('not modelled', lakota[3])
        # A row only the tree holds, and one only the generator emits.
        self.assertIsNone(rows[2][3])
        self.assertIsNone(rows[3][2])

    def test_a_round_trip_that_is_empty_has_no_drift(self):
        self.assertEqual(nc.drift(nc.splice(DOC, GEN), GEN), [])

    def test_the_header_rows_never_count(self):
        gen = GEN.replace('| constant | rules |', '| constant | rule |')
        self.assertNotIn('|---', ''.join(k for _, k, _, _ in nc.drift(nc.splice(DOC, GEN), gen)))

    def test_a_repeated_key_never_hides_its_second_row(self):
        twice = '| Lakota | raze speed | `ObjectData::train_time@006508c0` | not modelled | `lakota_raze_speed` |'
        tree = nc.splice(DOC, GEN)
        gen = GEN.replace(twice, twice + '\n' + twice.replace('not modelled', 'modelled at lib.rs'))
        tree = tree.replace(twice, twice + '\n' + twice)
        self.assertEqual([(b, k) for b, k, _, _ in nc.drift(tree, gen)], [('hand', 'Lakota | raze speed #2')])

    def test_a_block_the_tree_lacks_is_a_drift(self):
        gen = GEN + '\n<!-- GENERATED:consts2 -->\n| constant | rules |\n|---|---:|\n| `TURK_Z` | 3 |'
        rows = nc.drift(nc.splice(DOC, GEN), gen)
        self.assertEqual([(b, k) for b, k, _, _ in rows], [('consts2', None)])


class TheCheck(unittest.TestCase):
    def run_check(self, export, rules, doc):
        out = io.StringIO()
        with redirect_stdout(out):
            code = nc.check(export, rules, doc, generate=lambda e, r: GEN)
        return code, out.getvalue()

    def test_drift_exits_one_and_names_every_row(self):
        with tempfile.TemporaryDirectory() as d:
            d = pathlib.Path(d)
            (d / 'funcs').mkdir()
            (d / 'rules.xml').write_text('<RULES/>')
            (d / 'NATIONS.md').write_text(DOC)
            code, out = self.run_check(d, d / 'rules.xml', d / 'NATIONS.md')
        self.assertEqual(code, 1)
        self.assertIn('4 rows differ', out)
        self.assertIn('Lakota | raze speed', out)
        self.assertIn('`MAYA_Y`', out)

    def test_an_empty_round_trip_exits_zero(self):
        with tempfile.TemporaryDirectory() as d:
            d = pathlib.Path(d)
            (d / 'funcs').mkdir()
            (d / 'rules.xml').write_text('<RULES/>')
            (d / 'NATIONS.md').write_text(nc.splice(DOC, GEN))
            code, out = self.run_check(d, d / 'rules.xml', d / 'NATIONS.md')
        self.assertEqual(code, 0, out)
        self.assertIn('round trip is empty', out)

    def test_no_export_or_no_install_skips_rather_than_fails(self):
        with tempfile.TemporaryDirectory() as d:
            d = pathlib.Path(d)
            (d / 'NATIONS.md').write_text(DOC)
            code, out = self.run_check(d / 'no-export', d / 'no-rules.xml', d / 'NATIONS.md')
        self.assertEqual(code, 0)
        self.assertIn('skipped', out)


if __name__ == '__main__':
    unittest.main()
