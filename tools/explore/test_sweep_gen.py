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

    def test_a_void_is_an_object_held_to_its_bytes(self):
        # Parked 1889 (Loop 1890's second reach): ten empty bodies were a
        # throwaway substitution of 1866's script; a void is `obj` with no
        # answer, every byte of the object held, `esp` checked after `ret N`.
        sp, script, tp, test, mod = sweep_gen.generate_void('Foo::bar@00401000', item=1889, own=2)
        compile(script, str(sp), 'exec')
        self.assertIn('OWN = 2', script)
        self.assertIn('wrote past the window', script)
        self.assertIn('wrong pop', script)
        self.assertIn('let _ = (got, want_eax);', test)
        self.assertIn('an empty body', test)

    def test_an_object_with_a_cell_a_table_a_pointer_argument_and_a_stub(self):
        # Parked 1955, 1967, 1972, 1977 and 1994 (Loop 1890's ninth to
        # thirteenth reaches): a record through a global cell, a table at a
        # fixed address, a buffer a pointer argument names, and a callee
        # stubbed per row, each hand-written again by the sweep lanes.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=1994, win=0x40, bias=[], own=2,
            cells=['0xc061d0:0x200'], at=['0xc06afc:0x100'], ptrargs=['1:0x20'],
            stubs=['slot:0x138:0', '0x00401234:2'], port='rd(&c[0], 4) + rd(&p[0], 0) + i64::from(a[2])')
        compile(script, str(sp), 'exec')
        self.assertIn('CELLS = [(12607952, 512)]', script)
        self.assertIn('AT = [(12610300, 256)]', script)
        self.assertIn('PTRARGS = [(1, 32)]', script)
        self.assertIn("STUBS = [('slot', 312, 0), ('call', 4198964, 2)]", script)
        self.assertIn('from hooks import Harness', script)
        self.assertIn('wrote past the window', script)
        self.assertIn('let mut c: Vec<Vec<u8>>', test)
        self.assertIn('let mut p: Vec<Vec<u8>>', test)
        self.assertIn('rd(&c[0], 4) + rd(&p[0], 0) + i64::from(a[2])', test)
        self.assertIn('window c[{i}]', test)
        # The plain `obj` keeps its row format: one window, no `;`.
        sp, script, tp, test, mod = sweep_gen.generate_obj('Foo::bar@00401000', item=1994, win=0x40, bias=[], own=1)
        self.assertNotIn('CELLS', script)
        self.assertNotIn('Vec<Vec<u8>>', test)

    def test_a_vcall_may_take_the_function_s_own_arguments(self):
        # 1966's `BuildTypeData::has_stance_type` compares the slot's answer
        # with its own `p0`; the plain `vcall` had no `--own`.
        script, test = self.check('vcall', 'BuildTypeData::has_stance_type@00639770',
                                  slot=0x138, nargs=0, own=1, port='i64::from(v == p[0] as i32)')
        self.assertIn('OWN = 1', script)
        self.assertIn('row[1:]', script)
        self.assertIn('let p: &[i64] = &a[1..];', test)

    def test_a_slot_compared_with_the_base_s_address(self):
        # Parked 2021 (Loop 2003's second reach), built by item 2072: the
        # four `UnitData::get_*_stance` getters compare slot +0x104 with the
        # base's address and, equal, call the type record's +0x134 instead.
        sp, script, tp, test, mod = sweep_gen.generate_vbase(
            'UnitData::get_combat_stance@00610a30', item=2072, slot=0x104, base=0x46ced0,
            handed=['0'], via=0x18, via_slot=0x134, port='i64::from(v)')
        compile(script, str(sp), 'exec')
        self.assertIn('BASE = 0x0046ced0', script)
        self.assertIn('h.w32(vt + SLOT, BASE if mode else stub)', script)
        self.assertIn('VIA = 24', script)
        self.assertIn('let handed: &[i64] = &[0];', test)
        self.assertIn('match (mode, true)', test)
        self.assertIn('UnitData::get_combat_stance@00610a30', test)
        # The inline arm that calls nothing (is_missile's field test).
        _, script, _, test, _ = sweep_gen.generate_vbase(
            'UnitTypeData::is_missile@0061d430', item=2072, slot=0xf0, base=0x470860, handed=['0x8000000'])
        self.assertIn('VIA = None', script)
        self.assertIn('match (mode, false)', test)
        self.assertIn('&[134217728]', test)
        # A via with no slot is refused.
        with self.assertRaises(ValueError):
            sweep_gen.generate_vbase('Foo::bar@00401000', item=1, slot=0x10, base=0x401000, via=0x18)

    def test_an_answer_may_set_ecx_on_return(self):
        # Parked 1922 (Loop 1890's sixth reach): `Tribes`, `Lands` and
        # `Herds` read a hooked callee's `ecx` on return (the decompiler's
        # `extraout_ECX`); an answer that is a pair sets both registers.
        import sweeplib
        self.assertEqual(sweeplib.answer_regs(7, 1, []), (7, None))
        self.assertEqual(sweeplib.answer_regs((7, 0x1234), 1, []), (7, 0x1234))
        self.assertEqual(sweeplib.answer_regs(lambda ecx, args: ecx + args[0], 5, [2]), (7, None))
        self.assertEqual(sweeplib.answer_regs(lambda ecx, args: (ecx, args[0]), 5, [2]), (5, 2))

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


    def test_an_obj_from_the_command_line_holds_its_answer(self):
        # Item 2090: `--ret` defaulted to fwd's `none` for every shape, so an
        # `obj` written without it compared no `eax`; eight value-returning
        # sweeps stood on their bytes alone, one of them reading the wrong field.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            try:
                self.assertEqual(sweep_gen.main(['obj', 'X::y@00401000', '--item', '1', '--win', '0x20']), 0)
                self.assertIn('assert_eq!(got, want_eax', (Path(tmp) / 'x_y.rs').read_text())
                self.assertEqual(sweep_gen.main(['obj', 'X::z@00401010', '--item', '1', '--ret', 'none']), 0)
                self.assertNotIn('assert_eq!(got, want_eax', (Path(tmp) / 'x_z.rs').read_text())
                self.assertEqual(sweep_gen.main(['fwd', 'X::w@00401020', '--item', '1',
                                                 '--call', '0x401000:0:this:']), 0)
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'


if __name__ == '__main__':
    unittest.main()
