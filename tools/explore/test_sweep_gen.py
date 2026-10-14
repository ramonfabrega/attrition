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

    def test_a_table_placed_so_the_reads_land_in_a_buffer(self):
        # Parked 2113 and 2046 (Loop 2003's tenth reach, item 2122): a world
        # record's cell array, a leader table and a pointer field of `this`
        # pointed at `buffer - (the offset the function adds)` a row, by hand
        # three times; with a filled table, a window's bias held every row,
        # an argument's range, and a slot laid as its base address or a stub.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2122, win=0x40, bias=[], own=2,
            cells=['0xc061d0:0x200'], ptrargs=['1:4'], stubs=['slot:0xc:0'],
            places=['c0+0x134:(i32(c[0],0)*i32(p[0],0)+a[0])*0x1c:0x1c', 'p1+0x4:0x248:4',
                    '0xc0aec0+i8(m,0xa1)*0x1c:i16(m,0x8c)*4:4'],
            wbias=['c0+0x18:0..16', 'p1+0x4:1,2,3', 'm+*:0..9'], fills=['c0:i//3'],
            argrange=['0:0:255'], slotbase=['0xc:0x4707e0'],
            port='rd(&p[1], 0) + i64::from(a[2]) + i64::from(a[3])')
        compile(script, str(sp), 'exec')
        self.assertIn("PLACES = [('c0', '0x134', '(i32(c[0],0)*i32(p[0],0)+a[0])*0x1c', 28), "
                      "('p1', '0x4', '0x248', 4), (None, '0xc0aec0+i8(m,0xa1)*0x1c', 'i16(m,0x8c)*4', 4)]", script)
        self.assertIn("WBIAS = [('c0', 24, ('range', 0, 16)), ('p1', 4, ('set', [1, 2, 3])), ('m', None, ('range', 0, 9))]",
                      script)
        self.assertIn("FILLS = [('c0', 'i//3')]", script)
        self.assertIn('ARGRANGE = [(0, 0, 255)]', script)
        self.assertIn('SLOTBASE = [(12, 4655072)]', script)
        self.assertIn('buf - off', script)
        self.assertIn('placed', test)
        # A placed buffer is one more `p` window, after the pointer arguments.
        self.assertIn('let mut p: Vec<Vec<u8>> = wins.split_off(1)', test)
        # An overload names its own module (parked 2045): two `get_land`s.
        sp, script, tp, test, mod = sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], own=1,
                                                           ptrargs=['0:4'], module='foo_bar_wcoord')
        self.assertEqual((sp.name, tp.name, mod), ('sweep_foo_bar_wcoord.py', 'foo_bar_wcoord.rs', 'mod foo_bar_wcoord;'))
        self.assertIn('emu_rows("sweep_foo_bar_wcoord.py")', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], own=1, places=['q0+4:0:4'])
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], own=1, slotbase=['0xc:0x401000'])

    def test_a_range_bias_draws_its_own_edges(self):
        # Loop 2003's sixteenth reach (item 2170's "for the Loop", built by
        # item 2181): a `--wbias LO..HI` range was drawn uniformly, so a
        # predicate on a boundary inside a wide range (`t < 1` in `-2..3000`)
        # was tested by luck. The draw now takes LO, HI and the small
        # integers inside the range a few rows in ten.
        import random
        import re
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2181, win=0x10, bias=[], own=0,
            cells=['0xc06188:0x20'], wbias=['c0+0:-2..3000'], port='0')
        compile(script, str(sp), 'exec')
        body = re.search(r'^def wdraw\(.*?(?=^\S)', script, re.S | re.M)
        self.assertIsNotNone(body, 'the script draws a bias through `wdraw`')
        env = {}
        exec(body.group(0), env)
        rng = random.Random(1)
        drawn = {env['wdraw'](rng, ('range', -2, 3000)) for _ in range(100)}
        for edge in (-2, -1, 0, 1, 2, 3000):
            self.assertIn(edge, drawn, f'the edge {edge} in a hundred draws')
        self.assertTrue(all(-2 <= v <= 3000 for v in drawn))
        # Edges outside the range are never drawn; a set stays a set.
        drawn = {env['wdraw'](rng, ('range', 5, 9)) for _ in range(100)}
        self.assertEqual(drawn, {5, 6, 7, 8, 9})
        self.assertEqual({env['wdraw'](rng, ('set', [4, 6])) for _ in range(50)}, {4, 6})

    def test_a_cdecl_function_leaves_its_arguments_to_the_caller(self):
        # Loop 2003's sixteenth reach (item 2181): `Caravan::distance@0073d300`
        # is `__cdecl` and ends in a bare `ret`, so the obj shape's esp check
        # (the callee pops its arguments) refused every row; item 2170 left
        # `add_type_bucket` for the same reason. `--cdecl` expects the caller's
        # pop: esp short by four bytes an argument.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2181, win=8, bias=[], own=4,
            cells=['0xc06188:4'], cdecl=True, port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('CDECL = True', script)
        self.assertIn('- (4 * len(args) if CDECL else 0)', script)
        self.assertIn('`__cdecl`', script)
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2181, win=8, bias=[], own=1, cells=['0xc06188:4'])
        self.assertIn('CDECL = False', script)
        # The plain obj has no window to carry it, and refuses it.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=1, cdecl=True)

    def test_an_argument_may_copy_a_word_of_a_placed_window(self):
        # Item 2200: `PathFinder::valid_wcoord@00687da0` answers 0 at once when
        # its two coordinates equal the unit's `avoid_x`/`avoid_y`, which live
        # in a record the function reaches through `this+0x54` — a placed
        # window, laid after the arguments are drawn. `--argrange` keeps the
        # coordinates inside the div_3_table, and two independent draws almost
        # never meet: `--argeq IDX:WIN+OFF` copies the window's word into the
        # argument (45%) once every window is laid.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2200, win=0x58, bias=[], own=2,
            places=['m+0x54:0:0x128'], argrange=['0:0:2000'], argeq=['0:p0+0x120'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("ARGEQ = [(0, 'p0', 288)]", script)
        self.assertIn('for idx, win, off in ARGEQ:', script)
        # It runs after the places, so the copied word is the one the call reads.
        self.assertLess(script.index('for win, loc, expr, size in PLACES:'), script.index('for idx, win, off in ARGEQ:'))
        self.assertLess(script.index('for idx, win, off in ARGEQ:'), script.index('for a_, b in wins:'))
        self.assertIn('argument 0 copies `p0+0x120` (45%)', script)
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2200, win=8, bias=[], own=1, cells=['0xc06188:4'])
        self.assertIn('ARGEQ = []', script)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=1,
                                   cells=['0xc06188:4'], argeq=['1:m+0'])

    def test_ecx_may_be_an_integer_rather_than_the_object(self):
        # Parked 2141 (Loop 2003's twelfth reach, item 2149): `Map::scale_by_axis`
        # and `Map::scale_by_area` take the axis length in `ecx`, an integer,
        # and `obj` laid `ecx` as the object. The integer is the row's last.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2149, win=0x20, bias=[], own=0,
            cells=['0xc06188:0x20'], ecx_int='-4..5000', port='i64::from(a[a.len() - 1])')
        compile(script, str(sp), 'exec')
        self.assertIn("ECXINT = ('range', -4, 5000)", script)
        self.assertIn('ecx=this if ECXINT is None else ecx', script)
        self.assertIn('+ ([] if ECXINT is None else [str(ecx)])', script)
        self.assertIn('`ecx` an integer', test)
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2149, win=0x20, bias=[], own=1, cells=['0xc06188:0x20'], ecx_int='0,1,7')
        self.assertIn("ECXINT = ('set', [0, 1, 7])", script)
        # Without it, `ecx` stays the object.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2149, win=0x20, bias=[], own=0, cells=['0xc06188:0x20'])
        self.assertIn('ECXINT = None', script)
        self.assertNotIn('`ecx` an integer', test)
        # And the shape refuses it on the plain `obj`, which has no row for it.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x20, bias=[], own=0, ecx_int='1..2')

    def test_a_stub_s_answer_may_be_drawn_from_a_list(self):
        # Parked 2141 (Loop 2003's twelfth reach), built by item 2160:
        # `UnitData::is_cloaked` and `WorldData::is_pass_land` compare a
        # callee's answer with 0, which the generic `I32` draws one row in
        # thirty; `--answers IDX:V,..` draws stub IDX's answer from the list.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2160, win=0x20, bias=[], own=0,
            stubs=['0x60b030:0'], answers=['0:0,1'], port='i64::from(a[0] == 0)')
        compile(script, str(sp), 'exec')
        self.assertIn('ANSWERS = [(0, [0, 1])]', script)
        self.assertIn('for idx, vals in ANSWERS:', script)
        self.assertIn('stub 0 answered from [0, 1]', test)
        # It names a stub the line stubs.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x20, bias=[], own=0,
                                   stubs=['0x60b030:0'], answers=['1:0,1'])

    def test_a_placed_window_may_read_the_stubs_answers(self):
        # Item 2160: `ObjectsData::find_town` indexes the cities' table with
        # its callee's answer, so the answers are drawn before the windows
        # are placed, and a `--place` expression reads them as `s`.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2160, win=0x20, bias=[], own=3,
            cells=['0xc061d4:0x100'], stubs=['0x658190:3'], argrange=['2:0:7'],
            places=['c0+0x10+a[2]*0x1c:s[0]*4:4'], port='0')
        compile(script, str(sp), 'exec')
        self.assertLess(script.index('answers = [rand32(rng) for _ in STUBS]'),
                        script.index('for win, loc, expr, size in PLACES:'))
        self.assertIn('"s": answers', script)

    def test_a_slot_of_a_placed_object_may_answer_a_window(self):
        # Parked 2151, built by item 2170: `UnbuiltCity::get_x` calls slot
        # +0xac of `objects[who][o]`, an object reached through two placed
        # windows, and reads `+0x10` of the record the slot answers; the
        # coordinate goes out through the hidden return pointer `[esp+4]`.
        # So the stubbed vtable is laid in a placed window (`--vtat`), and a
        # stub answers the address of a window of its own (`--answer-win`).
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2170, win=8, bias=[], own=1,
            cells=['0xc0618c:0xe0'], ptrargs=['0:4'], stubs=['slot:0xac:0'],
            places=['c0+0x14+i8(m,2)*0x1c:i16(m,0)*4:4', 'p1+0:0:0x10'],
            vtat='p2+0', answer_wins=['0:0x20'], port='a[0] as i64')
        compile(script, str(sp), 'exec')
        self.assertIn("VTAT = ('p2', 0)", script)
        self.assertIn('WIN = 8\n', script)
        self.assertIn('ANSWERWIN = [(0, 32)]', script)
        self.assertLess(script.index('for win, loc, expr, size in PLACES:'),
                        script.index('for idx, size in ANSWERWIN:'))
        self.assertLess(script.index('for idx, size in ANSWERWIN:'),
                        script.index('for a_, b in wins:'))
        self.assertIn('vtable laid at `p2+0`', test)
        self.assertIn('stub 0 answers the address of 32 random bytes', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2170, win=8, bias=[], answer_wins=['0:0x20'])
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2170, win=8, bias=[], stubs=['0x401000:0'],
                                   vtat='p0+0')

    def test_a_record_s_vtable_may_be_a_class_s_half_the_rows(self):
        # Item 2223: `UnbuiltCity::is_started` calls slot +0xac of
        # `objects[who][o]` and compares the record it answers with
        # `Build::vftable`: equal, a flag byte; else the record's own slot
        # +0x50. The stubbed vtable is laid at the object (`--vtat`) and at
        # the record half the rows (`--vtat-half`), the other half left as
        # the record's word was drawn (a `--wbias` of the class's address).
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2223, win=8, bias=[], stubs=['slot:0xac:0', 'slot:0x50:0'],
            places=['0xc0618c:0x14+i8(m,2)*0x1c:4', 'p0+0:i16(m,0)*4:4', 'p1+0:0:0xc'],
            vtat='p2+0', answer_wins=['0:0xc'], vtat_half=['p3+0'], wbias=['p3+0:0xb42174'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("VTATHALF = [('p3', 0)]", script)
        self.assertLess(script.index('if VTAT is not None:'), script.index('for win, off in VTATHALF:'))
        self.assertLess(script.index('for win, off in VTATHALF:'), script.index('for a_, b in wins:'))
        self.assertIn('the stubbed vtable laid at `p3+0` half the rows', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2223, win=8, bias=[], stubs=['0x401000:0'],
                                   vtat_half=['p0+0'])

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

    def test_a_slot_compared_with_the_base_whose_body_is_a_call_of_another_slot(self):
        # Item 2112 (parked 2095's sibling): `BuildTypeData::makes_roads`
        # compares slot +0x64 with `is_city` and, equal, calls its own slot
        # +0x60 handed (0x19e, 0); `ObjectData::isnt` compares the record's
        # +0xf8 with `ObjectTypeData::isnt` and answers `!+0x60(p0, p1)`.
        sp, script, tp, test, mod = sweep_gen.generate_vcallbase(
            'BuildTypeData::makes_roads@00639910', item=2112, slot=0x64, base=0x472bc0, base_slot=0x60,
            handed=['0x19e', '0'], port='i64::from(v)')
        compile(script, str(sp), 'exec')
        self.assertIn('BSLOT = 0x60', script)
        self.assertIn("HANDED = ['414', '0']", script)
        self.assertIn('VIA = None', script)
        self.assertIn('["414", "0"]', test)
        self.assertIn('format!("slot,this{own_s}")', test)
        _, script, _, test, _ = sweep_gen.generate_vcallbase(
            'ObjectData::isnt@00653480', item=2112, slot=0xf8, base=0x661b80, base_slot=0x60,
            handed=['p0', 'p1'], own=2, via=0x18, negate=True)
        self.assertIn('VIA = 24', script)
        self.assertIn('OWN = 2', script)
        self.assertIn('format!("slot,rec{own_s}")', test)
        self.assertIn('todo!(', test)
        # A forwarded argument the function does not take is refused.
        with self.assertRaises(ValueError):
            sweep_gen.generate_vcallbase('Foo::bar@00401000', item=1, slot=0x10, base=0x401000, base_slot=0x14,
                                         handed=['p1'], own=1)

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
