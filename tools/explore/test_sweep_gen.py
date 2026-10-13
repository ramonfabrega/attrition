"""`tools/sweep_gen.py`: a sweep's two files from one line (parked 1706,
the twenty-ninth pass). Each shape's script must parse as Python and its
test must name the function, the script and the port; a function spelled
any other way is refused."""
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import sweep_gen  # noqa: E402


class Shapes(unittest.TestCase):
    def check(self, shape, spec, **kw):
        sp, script, tp, test, mod = sweep_gen.generate(shape, spec, item=1736, **kw)
        compile(script, str(sp), 'exec')
        name, va = sweep_gen.parse(spec)
        self.assertIn(f'{name}@{va:08x}', test)
        self.assertIn(f'sweep_{sweep_gen.snake(name)}.py', test)
        self.assertEqual(sp.name, f'sweep_{sweep_gen.snake(name)}.py')
        self.assertEqual(tp.name, f'{sweep_gen.snake(name)}.rs')
        self.assertEqual(mod, f'mod {sweep_gen.snake(name)};')
        self.assertIn('#[test]', test)
        return script, test

    def test_a_constant(self):
        script, test = self.check('const', 'FollowOrder::get_type@00486c20', port='7')
        self.assertIn('ecx=row[0]', script)
        self.assertIn('let got: i64 = 7;', test)

    def test_a_field_through_the_type_pointer(self):
        script, test = self.check('field', 'ObjectData::has_range@00646c10',
                                  via=0x18, off=0x1fc, port='i64::from(v)')
        self.assertIn('OBJ + 0x18', script)
        self.assertIn('TYPE + 0x1fc', script)
        self.assertIn('let got: i64 = i64::from(v);', test)
        self.assertIn('n >= 200', test)

    def test_a_field_on_the_object_itself(self):
        script, _ = self.check('field', 'Unit::hits@00612340', off=0x40, port='i64::from(v)')
        self.assertIn('OBJ + 0x40', script)
        self.assertNotIn('TYPE +', script)

    def test_a_range_predicate(self):
        script, test = self.check('range', 'TypeData::is_bonus_type@004705b0',
                                  port='crate::type_index::is_bonus_type(t)')
        self.assertIn('TYPE + 0x4', script)
        self.assertIn('range(-2, 0x340)', script)
        self.assertIn('let got: bool = crate::type_index::is_bonus_type(t);', test)

    def test_no_port_is_a_todo_that_fails(self):
        _, test = self.check('const', 'AttackOrder::get_type@00486c30')
        self.assertIn('todo!("the port of AttackOrder::get_type")', test)

    def test_an_operator_is_a_name(self):
        _, test = self.check('range', 'TCoord::operator<@00472b30', port='t < 0')
        self.assertIn('TCoord::operator<@00472b30', test)
        self.assertEqual(sweep_gen.snake('TCoord::operator<'), 'tcoord_operator')

    def test_a_function_spelled_another_way_is_refused(self):
        with self.assertRaises(ValueError):
            sweep_gen.generate('const', 'get_type', item=1)
        with self.assertRaises(ValueError):
            sweep_gen.generate('const', 'Unit::hits@612340', item=1)

    def test_a_generated_sweep_is_written_once(self):
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            try:
                self.assertEqual(sweep_gen.main(['const', 'X::y@00401000', '--item', '1']), 0)
                self.assertEqual(sweep_gen.main(['const', 'X::y@00401000', '--item', '1']), 2)
                self.assertTrue((Path(tmp) / 'sweep_x_y.py').exists())
                self.assertTrue((Path(tmp) / 'x_y.rs').exists())
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'


if __name__ == '__main__':
    unittest.main()
