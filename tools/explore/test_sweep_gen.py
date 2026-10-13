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
sys.path.insert(0, str(ROOT / 'tools/emu'))
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

    def test_a_global_cell_shape(self):
        # Parked 1761, the thirtieth pass: eight lanes hand-wrote "the
        # pointer in a global cell, a word at +OFF of the record it names".
        script, test = self.check('cell', 'GameAccessConst::get_scary_console_leader@005833e0',
                                  cell=0xc06210, off=0x298, port='i64::from(v)')
        self.assertIn('CELL = 0x00c06210', script)
        self.assertIn('REC + 0x298', script)
        self.assertIn('from sweeplib import', script)
        self.assertIn('let got: i64 = i64::from(v);', test)

    def test_a_virtual_call_shape(self):
        # Five lanes laid a fake vtable whose slot answers the row's value;
        # the generated script uses `tools/emu/hooks.py`'s Harness.
        script, test = self.check('vcall', 'OrderList::get_data_type@0046ed70',
                                  slot=0x10, nargs=0, port='i64::from(v)')
        self.assertIn('from hooks import Harness', script)
        self.assertIn('0x10: ("slot", 0, ', script)
        self.assertIn('h.calls', script)
        self.assertIn('let got: i64 = i64::from(v);', test)

    def test_an_out_pointer_shape(self):
        # `WCoord::traverse_y` and `get_tcorner` store through `[esp+4]`.
        script, test = self.check('outptr', 'WCoord::traverse_y@0046eed0', off=0x0, port='i64::from(v) >> 2')
        self.assertIn('OUT', script)
        self.assertIn('OBJ + 0x0', script)
        self.assertIn('let got: i64 = i64::from(v) >> 2;', test)

    def test_the_generated_script_s_helpers_are_one_module(self):
        # Fourteen lanes threw away the same `stub`/`argstub`/`setup()`.
        import sweeplib
        self.assertEqual(sweeplib.stub_bytes(7), b'\xb8\x07\x00\x00\x00\xc3')
        self.assertEqual(sweeplib.stub_bytes(-1, ret=8), b'\xb8\xff\xff\xff\xff\xc2\x08\x00')
        self.assertTrue(sweeplib.argstub_bytes(5).endswith(b'\xc2\x08\x00'))
        self.assertIn('setup', dir(sweeplib))
        # Parked 1867: `Guy::set_bank`/`set_pitch` take a float in `xmm1`;
        # the seam is a float's bits in a 128-bit register's low lane.
        self.assertEqual(sweeplib.xmm_bits(1.0), 0x3f800000)
        self.assertEqual(sweeplib.xmm_bits(-2.5), 0xc0200000)
        self.assertEqual(sweeplib.xmm_bits(0x42280000, raw=True), 0x42280000)

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
