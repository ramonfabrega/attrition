"""`tools/sweep_gen.py`: a sweep's two files from one line (parked 1706,
the twenty-ninth pass). Each shape's script must parse as Python and its
test must name the function, the script and the port; a function spelled
any other way is refused."""
import contextlib
import inspect
import io
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

    def test_vtat_keep_lays_an_image_function_at_an_unstubbed_slot(self):
        # Item 2900 (parked 2888): `--vtat-keep SLOT:VA` lays the image function at VA at vtable slot SLOT half the rows
        # (a flag after the answers), the slot a stub or not; it needs a stubbed vtable.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2900, win=0x40, bias=[], own=1, stubs=['slot:0x3c:0'],
            vtat_keep=['0x78:0x642510'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('SLOTBASE = [(120, %d)]' % 0x642510, script)
        self.assertIn('slot +0x78 laid as 0x00642510', test)
        for kw in ({'vtat_keep': ['0x78']}, {'vtat_keep': ['0x78:0x642510'], 'stubs': []}):
            kw = {'stubs': ['slot:0x3c:0'], **kw}
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2900, win=0x40, bias=[], own=1, port='0', **kw)

    def test_link_hi_draws_the_short_beside_a_link(self):
        # Item 2900 (parked 2889): `--link-hi pA..B+OFF:V,..|LO..HI` writes the short at OFF after the links.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2900, win=0x40, bias=[], own=1, ptrargs=['0:4'],
            places=['m+4:0:0x40', 'p1+0:0:0x40', 'p2+0:0:0x40'], links=['p1..2+0x2c:2:<'], link_hi=['p1..2+0x2e:0..2'],
            port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("LINKHI = [(1, 2, 46, ('range', 0, 2))]", script)
        self.assertLess(script.index('for lo, hi, off, size, d, terms in LINKS:'),
                        script.index('for lo, hi, off, vals in LINKHI:'))
        self.assertIn('p[lo + k][off:off + 2] = (wdraw(rng, vals) & 0xFFFF)', script)
        self.assertIn('hold a short at +0x2e drawn from 0..2, after the links', test)
        for bad in ('p2..1+0x2e:0..2', 'p1+0x2e:0..2', 'p1..2+0x2e'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2900, win=0x40, bias=[], own=1, port='0', link_hi=[bad])

    def test_place_null_may_force_a_guard_word_on_its_null_rows(self):
        # Item 2900 (Loop 2899): `--place-null K:WIN+OFF=V` sets the word on K's null rows, after every window is drawn.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2900, win=0x100, bias=[], own=0,
            places=['m+0xd8:0:8', 'p0+0:0:8'], place_null=['1:m+0xdc=0'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("NULLFORCE = {1: [('m', 220, 0)]}", script)
        self.assertIn('NULLS = [1]', script)
        self.assertLess(script.index('forced += NULLFORCE.get(k, [])'), script.index('for win, off, v in forced:'))
        self.assertLess(script.index('for win, off, v in forced:'), script.index('callwins = {}'))
        for bad in ('0:m+0xdc', '0:q+0xdc=0', '5:m+0xdc=0'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2900, win=0x100, bias=[], own=0,
                                       places=['m+0xd8:0:8'], place_null=[bad], port='0')

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

    def test_a_stub_may_answer_each_call_in_turn(self):
        # Item 2217 (Loop 2003's eighteenth reach, 2211's line): a callee
        # called more than once a row answered one value for every call, so
        # `determine_roles`' two `is` tests and `check_building_wcoord`'s
        # corner walk could only ever agree. `--seq IDX:N` draws N answers a
        # row, hands them out in call order (the last repeated past N), and
        # prints them after the stubs' answers.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2217, win=0x20, bias=[], own=1,
            stubs=['0x401100:2'], answers=['0:0,1,4'], seq=['0:6'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('SEQ = [(0, 6, None)]', script)
        self.assertIn('for idx, n, terms in SEQ:', script)
        # Drawn from the stub's `--answers` list when it has one.
        self.assertIn('ANSWERS', script[script.index('for idx, n, terms in SEQ:'):])
        self.assertIn('+ [str(v) for s in seqs for v in s]', script)
        self.assertIn('stub 0 answers 6 values in call order', script)
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2217, win=0x20, bias=[], own=0, stubs=['0x401100:2'])
        self.assertIn('SEQ = []', script)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0,
                                   stubs=['0x401100:2'], seq=['1:3'])

    def test_a_stub_s_answer_may_be_held_to_a_range(self):
        # Item 2217: `City::fix_world_vals` indexes `circle_radius` by its
        # callee's answer, and `--answers`' 10% of random words reads before
        # the table. `--answer-range IDX:LO:HI` draws it from the range every
        # row, as `--argrange` does an argument.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2217, win=0x20, bias=[], own=0,
            stubs=['0x401100:1'], answer_range=['0:0:80'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('ANSRANGE = [(0, 0, 80)]', script)
        self.assertIn('for idx, lo, hi in ANSRANGE:', script)
        # Drawn after `--answers`, so the range is the last word.
        self.assertLess(script.index('for idx, vals in ANSWERS:'), script.index('for idx, lo, hi in ANSRANGE:'))
        self.assertIn('stub 0 answers 0..80 every row', script)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0,
                                   stubs=['0x401100:1'], answer_range=['1:0:3'])

    def test_a_global_cell_may_hold_this(self):
        # Item 2217: `World::set_road_at` writes through `this` and scans
        # through `GameAccess::world`, which in the game are one object;
        # `--cell-win` gave the cell a record of its own. `--cell-this VA`
        # points the cell at `this` every row, no window of its own.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2217, win=0x140, bias=[], own=0,
            cell_this=['0xc06188'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('CELLTHIS = [12607880]', script)
        self.assertIn('for va in CELLTHIS:', script)
        self.assertIn('the global cell 0x00c06188 holds `this`', script)
        # A cell is `this` or a record of its own, never both.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0,
                                   cells=['0xc06188:4'], cell_this=['0xc06188'])

    def test_the_stubbed_vtable_may_serve_this_and_a_placed_record(self):
        # Item 2217: `ObjectData::is_cargo` asks `this`'s slot `+0xbc` and then
        # its type record's slots `+0xc` and `+0x12c`; `--vtat` moves the one
        # stubbed vtable off `this`. `--vtat-also WIN+OFF` lays it in the
        # window as well, `this` keeping it.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2217, win=0x20, bias=[], own=0,
            places=['m+0x18:0:0x220'], stubs=['slot:0xbc:0', 'slot:0xc:0'], vtat_also=['p0+0'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("VTALSO = [('p0', 0)]", script)
        self.assertIn('for win, off in VTALSO:', script)
        self.assertIn('VTAT = None', script)
        self.assertIn('the stubbed vtable laid at `p0+0` as well as at `this`', script)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0,
                                   places=['m+0x18:0:8'], vtat_also=['p0+0'])

    def test_a_sparse_table_lays_only_the_slices_of_its_rows(self):
        # Parked 2201 (Loop 2003's seventeenth reach, item 2247): the leaders
        # table is eight rows 0x6eec apart, and `in_unfriendly_territory` reads
        # `+8` and the diplos at `+0x74` of two of them; a cell window of the
        # whole table is 113 KB a row. `--sparse LOC:STRIDE:N:OFF,SIZE[/..]`
        # points LOC at a zeroed table and lays each row's slices as `p`
        # windows, before the places; `@VA` lays them at a table in the image.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2247, win=0x20, bias=[], own=0,
            sparse=['0xc061e0:0x6eec:8:8,0x8c/0x3f8,4'], wbias=['p0..7+*:0,1,2'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("SPARSE = [(None, '0xc061e0', 28396, 8, [(8, 140), (1016, 4)])]", script)
        self.assertIn('for win, loc, stride, n, slices in SPARSE:', script)
        self.assertIn("('p7', None, ('set', [0, 1, 2]))", script)
        self.assertIn('8 rows 0x6eec apart at `0xc061e0`, slices +0x8:0x8c, +0x3f8:0x4 (`p` windows', script)
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2247, win=0x20, bias=[], own=0, sparse=['@0xe6e000:0x6eec:8:0,4'], port='0')
        self.assertIn("SPARSE = [('@', '0xe6e000', 28396, 8, [(0, 4)])]", script)
        # Slices of one row may not overlap, nor outrun the stride.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0,
                                   sparse=['0xc061e0:0x100:2:0,0x20/0x10,8'])
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0,
                                   sparse=['0xc061e0:0x100:2:0xf0,0x20'])

    def test_a_counted_pointer_array_behind_image_words_is_drawn_bounded(self):
        # Parked 2699, 2702 (item 2709): `Herds::start_herds` reads a count at
        # the image word 0xc0a244 and a pointer array at 0xc0a250, each
        # element a record; `--at 0xc0a244:4` draws 32 random bits and the
        # loop never ends. `--gtab CNT:LO..HI:PTR:SIZE` draws the count from
        # the range, lays a head window (CNT..PTR+4) and an array window as
        # `c` windows, and one `p` window of SIZE random bytes an element.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Herds::start_herds@00741d10', item=2709, win=0x20, bias=[], own=0,
            gtab=['0xc0a244:-1..6:0xc0a250:0x40'], stubs=['0x7417f0:0'], record=['0'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('GTAB = [(12624452, -1, 6, 12624464, 64)]', script)
        self.assertIn('for cnt, lo, hi, ptr, esize in GTAB:', script)
        self.assertIn('a counted array', script)
        self.assertIn('let mut c: Vec<Vec<u8>> = wins;', test)
        self.assertIn('wins.split_off(2)', test)
        for bad in ('0xc0a244:-1..6:0xc0a240:0x40', '0xc0a244:6..-1:0xc0a250:0x40', '0xc0a244:0..6:0xc0a250:8'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0, gtab=[bad])

    def test_a_link_field_is_drawn_to_terminate(self):
        # Loop 2003's reach from item 2247 (item 2276): `curr_uber_size`
        # climbs `o_up` to the captain and walks `o_down`; `set_in_danger`,
        # `set_group` and `upgrade_level` walk a link the same way, and a
        # random link can cycle. `--link pA..B+OFF:SIZE:DIR[:T,T..]` draws
        # figure k's (window p(A+k)'s) link from the figures on DIR's side of
        # its own index (`<` below, `>` above) or a terminator, so every
        # walk ends.
        import random
        import sweeplib
        rng = random.Random(1)
        ends = set()
        for _ in range(400):
            n = rng.randint(1, 9)
            for d in '<>':
                links = [sweeplib.link_value(rng, n, k, d, (-1,)) for k in range(n)]
                for k, v in enumerate(links):
                    self.assertTrue(v == -1 or (v < k if d == '<' else k < v < n), (d, k, v, links))
                k, seen = rng.randrange(n), 0
                while k >= 0:
                    k, seen = links[k], seen + 1
                    self.assertLessEqual(seen, n)
                ends.add(seen)
        self.assertGreater(len(ends), 4, 'a walk of every length, not only the first step')
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2276, win=0x20, bias=[], own=0,
            places=['p0+0:0:0x94', 'p0+4:0:0x94', 'p0+8:0:0x94'], cells=['0xc0aec0:0x10'],
            links=['p1..3+0x8e:2:<:-1,-0x8000', 'p1..3+0x90:2:>'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("LINKS = [(1, 3, 142, 2, '<', [-1, -32768]), (1, 3, 144, 2, '>', [-1])]", script)
        self.assertIn('link_value(rng, n, k, d, terms)', script)
        self.assertIn('from sweeplib import link_value', script)
        self.assertIn('figures p1..p3 link at +0x8e (2 bytes) below their own index or to -1, -32768', test)
        for bad in ('p1..3+0x8e:3:<', 'p1..3+0x8e:2:=', 'm+0x8e:2:<', 'p3..1+0x8e:2:<'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], links=[bad])

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

    def test_edx_may_carry_an_integer_beside_ecx(self):
        # Item 2346 (parked 2338, Loop 2003): `add_type_bucket` indexes its
        # byte table by `edx * 0x1ed + ecx`, and a sibling hands `edx` to its
        # callee unset; `obj` set no `edx`. `--edx` draws an integer into
        # `edx` every row, printed after `ecx` (or last when `ecx` is the
        # object), and the harness enters with it.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2346, win=0x20, bias=[], own=0,
            at=['0xed89ce:0xf68'], ecx_int='0..0x1ec', edx='0..7', port='i64::from(a[a.len() - 1])')
        compile(script, str(sp), 'exec')
        self.assertIn("EDX = ('range', 0, 7)", script)
        self.assertIn('edxa = 0 if EDX is None else edx', script)
        self.assertIn('+ ([] if EDX is None else [str(edx)])\n', script)
        self.assertIn('`edx` an integer in 0..7', test)
        # Beside the object in `ecx` as well, and from a list.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2346, win=0x20, bias=[], own=2, stubs=['0x402000:4'], edx='0,1,-1')
        self.assertIn("EDX = ('set', [0, 1, -1])", script)
        self.assertIn('ECXINT = None', script)
        # Without it, `edx` is entered as 0 and nothing is printed.
        _, script, _, test, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2346, win=0x20, bias=[], own=0, stubs=['0x402000:0'])
        self.assertIn('EDX = None', script)
        self.assertNotIn('`edx` an integer', test)
        # The plain obj has no row for it, and the harness takes the register.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x20, bias=[], own=0, edx='1..2')
        hooks = (ROOT / 'tools/emu/hooks.py').read_text()
        self.assertIn('def call(self, entry, stack_args=(), ecx=0, max_insns=200_000, edx=0, ret=\'eax\', xmm=None):', hooks)
        self.assertIn('edx=edx', hooks)

    def test_a_stub_may_write_a_window_word(self):
        # Item 2346 (parked 2338's spare): `Object::kill_contents` loops while
        # its container's head (+0x28) is not negative, and only the callee
        # that kills the head unlinks it; a stub that writes nothing loops
        # until the instruction budget. `--stub-write IDX:WIN+OFF:SIZE:VAL`
        # writes VAL (SIZE bytes) at the window's word each call of stub IDX.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2346, win=0x40, bias=[], own=0, ret='none',
            stubs=['0x402000:1', '0x403000:1'], stub_write=['1:m+0x28:2:-1'])
        compile(script, str(sp), 'exec')
        self.assertIn("STUBWRITE = [(1, 'm', 40, 2, -1)]", script)
        self.assertIn('def win_write(', script)
        self.assertIn('stub 1 writes -1 (2 bytes) at `m+0x28` each call', test)
        for bad in ('1:m+0x28:3:-1', '2:m+0x28:2:-1', '1:q+0x28:2:-1', '1:m:2:-1'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], ret='none',
                                       stubs=['0x402000:1', '0x403000:1'], stub_write=[bad])

    def test_a_cdecl_stub_may_copy(self):
        # Item 2921 (parked 2906): `memcpy` and `memcmp` are import thunks
        # (`jmp [0xac5428]`); a stub of one copied nothing, so the string
        # `CommandManager::issue_rename_city` copies into its command was
        # never under the deref. `--stub-copy IDX` makes a stubbed cdecl callee
        # copy argument 2's bytes from argument 1 to argument 0, as `memcpy`
        # does, and answer the destination.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2921, win=0x40, bias=[], own=0, ret='none',
            stubs=['0x402000:3'], stub_cdecl=['0'], stub_copy=['0'])
        compile(script, str(sp), 'exec')
        self.assertIn('STUBCOPY = [0]', script)
        self.assertIn('def copy_answer(', script)
        self.assertIn('stub 0 copies its argument 2 bytes from argument 1 to argument 0', test)
        for bad in ('1', '-1', 'x'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], ret='none',
                                       stubs=['0x402000:3'], stub_cdecl=['0'], stub_copy=[bad])
        # a stub with fewer than three arguments, or one the caller does not pop, is no memcpy
        for stubs, cd in ((['0x402000:2'], ['0']), (['0x402000:3'], [])):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], ret='none',
                                       stubs=stubs, stub_cdecl=cd, stub_copy=['0'])
        # the copy itself, on a fake memory: n bytes, the destination answered, n capped
        src = script[script.index('def copy_answer('):]
        ns = {}
        exec(src[:src.index('\n\n\n')], ns)
        mem = bytearray(0x40)
        mem[0x10:0x14] = b'abcd'
        rd_ = lambda a, n: bytes(mem[a:a + n])
        wr_ = lambda a, b: mem.__setitem__(slice(a, a + len(b)), b)
        self.assertEqual(ns['copy_answer'](None, rd_, wr_, 0x1000)(0, [0x20, 0x10, 4]), 0x20)
        self.assertEqual(bytes(mem[0x20:0x24]), b'abcd')
        self.assertEqual(bytes(mem[0x24:0x28]), bytes(4))
        mem[0x20:0x24] = bytes(4)
        ns['copy_answer'](None, rd_, wr_, 2)(0, [0x20, 0x10, 4])
        self.assertEqual(bytes(mem[0x20:0x24]), b'ab\0\0')

    def test_a_pointer_into_the_callers_argument_slots_is_sp_plus(self):
        # Item 2921 (parked 2918): `walk_data` hands a stub `&flag` (a local, `sp-0x1`) and
        # `ebp+0xc`, its own argument slot, which printed as the harness's raw stack
        # constant (2147483392). A record token for a word above the held `esp`
        # is `sp+0xN`, N bytes above it, so a port spells the slot and not the constant.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2921, win=0x40, bias=[], own=1, ret='none',
            stubs=['0x402000:2'], record=['0'])
        compile(script, str(sp), 'exec')
        src = script[script.index('def token('):]
        ns = {'STACK_BASE': 0x20000000, 'STACK_SIZE': 0x100000}
        exec(src[:src.index('\n\n\n')], ns)
        top = 0x20000000 + 0x100000 - 0x100
        tok = ns['token']
        self.assertEqual(tok(top - 1, []), 'sp-0x1')
        self.assertEqual(tok(top + 0xc, []), 'sp+0xc')
        self.assertEqual(tok(top, []), 'sp+0x0')
        self.assertEqual(tok(top + 0x40, []), str(top + 0x40))
        self.assertEqual(tok(0x7FFFFFFF, []), '2147483647')  # the integer edge, 0xff over the top
        self.assertEqual(tok(top + 0xc, [('m', top + 0x8, 0x10)]), 'm+0x4')

    def test_the_stack_under_the_call_may_be_cleared_each_row(self):
        # Item 2921 (parked 2906): a `--deref` of a caller's local printed the
        # last row's bytes wherever the function wrote less than the local
        # holds (`issue_rename_city`'s Command, 55 bytes, its string a variable
        # length). `--clean-stack N` zeroes N bytes under the entry `esp`
        # before each row, so a byte the function did not write reads 00.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2921, win=0x40, bias=[], own=0, ret='none',
            stubs=['0x402000:1'], deref=['0:0:55'], clean_stack=0x100)
        compile(script, str(sp), 'exec')
        self.assertIn('STACKCLEAN = 256', script)
        self.assertIn('uc.mem_write(STACK_BASE + STACK_SIZE - 0x100 - STACKCLEAN, bytes(STACKCLEAN))', script)
        self.assertIn('the stack 256 bytes under the call is zeroed before each row', test)
        _, none, _, _, _ = sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], own=0, ret='none', stubs=['0x402000:1'])
        self.assertIn('STACKCLEAN = 0', none)
        # alone, with no stub or window, it still selects the windowed script (the plain obj has no row for it)
        _, alone, _, _, _ = sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], own=0, ret='none', clean_stack=0x40)
        self.assertIn('STACKCLEAN = 64', alone)
        for bad in (-1, 0x20000):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x40, bias=[], ret='none', clean_stack=bad)

    def test_a_stub_may_decrement_a_window_word_each_call(self):
        # Item 2749 (parked 2734): `PathFinder::kill_lists` and `Build::verify_gather_tiles`
        # loop on a count a callee (`remove_current`, `Array::remove`) lowers by
        # one a call; `--stub-write IDX:WIN+OFF:SIZE:dec` is that, where VAL
        # writes one value every call.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2749, win=0x40, bias=[], own=0, ret='none',
            stubs=['0x402000:1', '0x403000:1'], stub_write=['1:m+0x28:4:dec'])
        compile(script, str(sp), 'exec')
        self.assertIn("STUBWRITE = [(1, 'm', 40, 4, 'dec')]", script)
        self.assertIn('stub 1 lowers `m+0x28` (4 bytes) by one each call', test)

    def test_a_callee_may_be_run_not_hooked(self):
        # Item 2731 (parked 2701, 2712, 2720): `Group::equals_group` calls
        # `normalize`, which rewrites `this` before the loop reads it; a hooked
        # `normalize` writes nothing. `--stub run:ADDR:NARGS` leaves the callee
        # unhooked, so the original's own code runs, its callees hooked by the
        # `--stub` lines beside it.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2731, win=0x40, bias=[], own=0, ret='none',
            stubs=['run:0x402000:0', '0x403000:0'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("STUBS = [('run', 4202496, 0), ('call', 4206592, 0)]", script)
        self.assertIn('run{a:x}', script)
        self.assertIn('callee 0x00402000 (0 args) run, not hooked', test)
        # only a `call` stub is handed to the Harness to hook
        self.assertIn('if k == "call"', script)
        with self.assertRaises(ValueError):
            sweep_gen._stub('run:0x402000')

    def test_a_tame_register_draws_no_wild_rows(self):
        # Item 2346 (parked 2338): `add_type_bucket2` increments a byte of a
        # table at `ecx`, and `--ecx-int` drew a random word one row in ten,
        # which faults the index. `--tame` holds `ecx` (and `edx`) to the
        # range or the list every row.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2346, win=0x20, bias=[], own=0,
            at=['0xed9936:0x1ed'], ecx_int='0..0x1ec', tame=True, port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('WILD = 0.0', script)
        self.assertIn('draw_reg(rng, ECXINT, WILD, rand32)', script)
        self.assertIn('every row, no wild word', test)
        _, script, _, test, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2346, win=0x20, bias=[], own=0, at=['0xed9936:0x1ed'], ecx_int='0..0x1ec')
        self.assertIn('WILD = 0.1', script)
        self.assertNotIn('no wild word', test)
        # Tame with no register drawn is refused: there is nothing to tame.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x20, bias=[], own=0,
                                   at=['0xed9936:0x10'], tame=True)
        # And the draw itself: a thousand tame rows stay inside the range,
        # and an untamed thousand do not.
        import random
        import sweeplib
        self.assertIn('from sweeplib import draw_reg', script)
        rng = random.Random(1)
        self.assertTrue(all(0 <= sweeplib.draw_reg(rng, ('range', 0, 0x1ec), 0.0) <= 0x1ec for _ in range(1000)))
        self.assertTrue(all(sweeplib.draw_reg(rng, ('set', [3, 9]), 0.0) in (3, 9) for _ in range(1000)))
        self.assertFalse(all(0 <= sweeplib.draw_reg(rng, ('range', 0, 0x1ec), 0.1) <= 0x1ec for _ in range(1000)))

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

    def test_a_stub_called_twice_may_answer_a_window_a_call(self):
        # Parked 2623, built by item 2633: `UnitData::is_gathering` calls
        # `get_action` on `this` and again on a unit reached through
        # `units`; each answer is an order object whose vtable (slots +0x10
        # and +0xe4 stubbed on one vtable) sits at its offset 0. `--answer-win`
        # repeated for one stub answers a window of its own to each call, in
        # call order, the last repeated past them.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2633, win=0x20, bias=[], own=1,
            stubs=['0x401100:0', 'slot:0x10:0', 'slot:0xe4:0'],
            answer_wins=['0:8', '0:8', '2:0x10'], vtat='p0+0', vtat_also=['p1+0'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('ANSWERWIN = [(0, 8), (0, 8), (2, 16)]', script)
        self.assertIn('stub 0 answers a window of its own to each call', test)
        self.assertIn('callwins', script)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2633, win=8, bias=[], stubs=['0x401100:0'],
                                   answer_wins=['0:8'], seq=['0:2'])

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

    def test_a_word_may_name_a_window_s_own_address(self):
        # Item 2797 (parked 2791): `StripBuilder::check_last_connect` compares `*node->next` with the node's own
        # address: a placed pointer cannot aim at another window (a 2^-32 draw), so `--alias WIN+OFF:TARGET[+K]`
        # lays the word as the address of window TARGET (plus K) half the rows, the other half left as drawn.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2797, win=8, bias=[], places=['m+0:0:16', 'p0+0:0:8', 'p1+4:0:8'],
            alias=['p2+0:p1+0x10'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("ALIAS = [('p2', 0, 'p1', 16, False)]", script)
        self.assertLess(script.index('for win, off, tgt, plus, always in ALIAS:'), script.index('for a_, b in wins:'))
        self.assertIn('the word at `p2+0` laid as the address of `p1+0x10` half the rows', test)
        for bad in (['p2+0'], ['p2+0:q1'], ['p2:p1']):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2797, win=8, bias=[], places=['m+0:0:16'],
                                       alias=bad, port='0')

    def test_an_alias_may_hold_every_row_and_a_stub_may_answer_by_its_ecx(self):
        # Item 2863 (parked 2861): `--alias WIN+OFF:TARGET!` every row; `--answer-by IDX:OBJ0:DAT0:N:LINKOFF` answers
        # the data window of the object the stub's `ecx` is, its links laid before the windows are written;
        # `--vtat-image WIN+OFF:VA` copies the stub slots into the image's vtable at VA.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2863, win=8, bias=[], stubs=['slot:0x3c:0'],
            places=['m+0:0:16', 'p0+0:0:8', 'p1+4:0:8', 'm+4:0:8'],
            alias=['p2+0:p1!'], answer_by=['0:1:3:1:0x2'], vtat_image=['p1+0:0xb42174'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("ALIAS = [('p2', 0, 'p1', 0, True)]", script)
        self.assertIn('ANSWERBY = [(0, 1, 3, 1, 2)]', script)
        self.assertIn("VTIMAGE = [('p1', 0, %d)]" % 0xb42174, script)
        self.assertLess(script.index('for idx, ofirst, dfirst, nobj, loff in ANSWERBY:'), script.index('for a_, b in wins:'))
        self.assertIn('laid as the address of `p1` every row', test)
        self.assertIn('stub 0 answers `p3+j`, the data of the object its `ecx` is in `p1+j`', test)
        for kw in ({'answer_by': ['0:1:3']}, {'answer_by': ['5:1:3:1:2']}, {'vtat_image': ['p1+0']}):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2863, win=8, bias=[], stubs=['slot:0x3c:0'],
                                       places=['m+0:0:16'], port='0', **kw)

    def test_edx_may_name_a_window(self):
        # Item 2801 (parked 2791): `MiscAccess::world_to_screen` is entered with a pointer in `edx` (the decompiler's
        # `in_EDX`) and adds a word to the int it points at; `--edx` draws an integer, so `--edx-win WIN` lays `edx`
        # as the address of window WIN (m, cN or pN), whose bytes the row prints before and after.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2801, win=8, bias=[], own=1, ptrargs=['0:8'], edx_win='p0', port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("EDXWIN = 'p0'", script)
        self.assertIn('`edx` the address of window `p0`', test)
        for bad in ('q0', 'p', '0'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2801, win=8, bias=[], own=1, ptrargs=['0:8'],
                                       edx_win=bad, port='0')
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2801, win=8, bias=[], own=1, ptrargs=['0:8'],
                                   edx_win='p0', edx='0..3', port='0')

    def test_a_float_register_argument_may_be_drawn(self):
        # Item 2807: `get_float_z` takes two floats in `xmm0` and `xmm1` (the decompiler's `in_XMM0_Da`), truncates
        # them to ints; `--xmm N:LO..HI` loads xmm N with the float V/4 for an integer V drawn from the range (quarters,
        # exact in an f32, so the port needs no float: Rust's `V / 4` truncates toward zero as `cvttss2si` does), V
        # printed after the row's other numbers.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2807, win=8, bias=[], own=0, cdecl=True, ptrargs=[],
            xmm=['0:-4000..4000', '1:-4000..4000'], ret='xmm0', port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("XMM = [(0, ('range', -4000, 4000)), (1, ('range', -4000, 4000))]", script)
        self.assertIn('xmm=xmm', script)
        self.assertIn('xmm 0 an integer in -4000..4000, loaded as that over four', test)
        for bad in ('0', '8:1..2', 'x:1..2'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2807, win=8, bias=[], own=0, xmm=[bad], port='0')

    def test_a_stub_may_answer_a_placed_record(self):
        # Item 2241: `CityData::get_pop_value` calls slot +0x3c of
        # `objects[who][o]` and reads `+4` of the record at `+0x18` of the
        # record the slot answers — two levels below the answer, where an
        # `--answer-win` (laid after the places) can lay no pointer. So a
        # place's location may be a stub's answer, `sN`: the stub answers
        # the placed buffer, and a later place reaches into it.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2241, win=0x60, bias=[], stubs=['slot:0x3c:0'],
            places=['0xc0ab84+i8(m,0x5e)*0x1c:i16(m,8)*4:4', 'p0+0:0:4', 's0:0:0x1c', 'p2+0x18:4:4'],
            vtat='p1+0', port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("('s0', '0', '0', 28)", script)
        self.assertIn('answers[int(win[1:])] = (buf - off)', script)
        self.assertIn('stub 0\'s answer points at 28 random bytes less `0` (placed)', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2241, win=8, bias=[], stubs=['slot:0x3c:0'],
                                   places=['s1:0:4'])

    def test_a_place_may_be_drawn_null(self):
        # Item 2253 (Loop 2003, 2241's line): `Unit::update_order` and
        # `UnitData::get_order` guard the pointer at `+0xdc` against null,
        # and a place is never null, so the guard ran one arm. `--place-null
        # K` lays place K's pointer as 0 a third of the rows; the buffer is
        # still drawn so every later window keeps its index, and a line
        # without the flag draws no extra number.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2253, win=0xe0, bias=[],
            places=['m+0xdc:0:8', 'p0+4:0:0x10'], place_null=['0'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('NULLS = [0]', script)
        self.assertIn('if k in NULLS and rng.random() < 1 / 3:', script)
        self.assertIn('null a third of the rows', test)
        sp2, script2, *_ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2253, win=0xe0, bias=[],
            places=['m+0xdc:0:8', 'p0+4:0:0x10'], port='0')
        self.assertIn('NULLS = []', script2)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2253, win=0xe0, bias=[],
                                   places=['m+0xdc:0:8'], place_null=['1'])

    def test_a_ptrarg_may_be_drawn_null(self):
        # Item 2271 (Loop 2003, 2253's line): `UnitType::backup` and its
        # kin guard a pointer argument against null, and a `--ptrarg` is
        # never null, so the guard ran one arm. `--ptrarg-null IDX` lays
        # argument IDX as 0 a third of the rows; its buffer is still drawn
        # so every later window keeps its index, and a line without the
        # flag draws no extra number.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2271, win=0x20, bias=[], own=2,
            ptrargs=['0:0x40', '1:0x10'], ptrarg_null=['1'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('PNULLS = [1]', script)
        self.assertIn('if idx in PNULLS and rng.random() < 1 / 3:', script)
        self.assertIn('argument 1 names 16 random bytes (null a third of the rows)', test)
        sp2, script2, *_ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2271, win=0x20, bias=[], own=2,
            ptrargs=['0:0x40', '1:0x10'], port='0')
        self.assertIn('PNULLS = []', script2)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2271, win=0x20, bias=[], own=2,
                                   ptrargs=['0:0x40'], ptrarg_null=['1'])

    def test_a_stub_may_record_what_it_was_handed(self):
        # Item 2281 (Loop 2003, parked 2272 and 2219): six of 2271's ten end
        # in `pop ebp; jmp` into the base class's twin, and a stubbed tail
        # target answered without a row saying what `ecx` and argument it
        # was handed. `--record IDX` prints stub IDX's calls after ` # `,
        # each `ecx,arg..` with a pointer into a window named by it
        # (`m+0x0`, `p0+0x10`), `-` when it was not called; the test hands
        # the port a `log` and holds it to the row.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2281, win=0x20, bias=[], own=1,
            ptrargs=['0:0x40'], ptrarg_null=['0'], stubs=['0x402000:1'], record=['0'], ret='none',
            port='{ log.push("-".into()); 0 }')
        compile(script, str(sp), 'exec')
        self.assertIn('RECORD = [0]', script)
        self.assertIn('def token(', script)
        self.assertIn('" # "', script)
        self.assertIn('let mut log: Vec<String> = Vec::new();', test)
        self.assertIn('split_once(" # ")', test)
        self.assertIn('stub 0 records its `ecx` and arguments', test)
        # Without the flag the row and the test are as before.
        _, script2, _, test2, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2281, win=0x20, bias=[], own=1,
            ptrargs=['0:0x40'], stubs=['0x402000:1'], ret='none', port='0')
        self.assertIn('RECORD = []', script2)
        self.assertNotIn('split_once(" # ")', test2)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2281, win=0x20, bias=[], own=1,
                                   stubs=['0x402000:1'], record=['1'])
        # The token: a pointer into a window by its name, anything else signed.
        ns = {'STACK_BASE': 0x7FF0_0000, 'STACK_SIZE': 0x10_0000}
        body = script[script.index('def token('):]
        exec(body[:body.index('\n\n\ndef ')], ns)
        spans = [('m', 0x1000, 0x20), ('p0', 0x2000, 0x40)]
        self.assertEqual(ns['token'](0x1000, spans), 'm+0x0')
        self.assertEqual(ns['token'](0x2010, spans), 'p0+0x10')
        self.assertEqual(ns['token'](0xFFFFFFFF, spans), '-1')
        self.assertEqual(ns['token'](0, spans), '0')

    def test_a_record_may_keep_edx_beside_ecx(self):
        # Item 2400 (parked 2369): `City::compute_trade` hands `trade_value`
        # the near end's leader in `edx`, and `--record` kept `ecx` and the
        # arguments only, so a break that hands over the far end's leader
        # was compared by no row. `--record-edx` prints `ecx,edx,arg..`.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2400, win=0x20, bias=[], own=0,
            stubs=['0x402000:1'], record=['0'], record_edx=True, ret='none',
            port='{ log.push("-".into()); 0 }')
        compile(script, str(sp), 'exec')
        self.assertIn('RECORDEDX = True', script)
        self.assertIn('h.edxs', script)
        self.assertIn('h.edxs.clear()', script)  # a row's calls are its own
        self.assertIn('`ecx`, `edx` and arguments', test)
        # Without the flag the row is as before.
        _, script2, _, test2, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2400, win=0x20, bias=[], own=0,
            stubs=['0x402000:1'], record=['0'], ret='none', port='0')
        self.assertIn('RECORDEDX = False', script2)
        self.assertNotIn('`edx` and arguments', test2)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2400, win=0x20, bias=[], own=0,
                                   stubs=['0x402000:1'], record_edx=True)
        hooks = open(sweep_gen.EMU / 'hooks.py').read()
        self.assertIn('self.edxs.append(', hooks)

    def test_a_circular_list_and_a_slot_answered_from_its_own_ecx(self):
        # Item 2400 (parked 2369): `Unit::kill_group_move` walks a circular
        # doubly linked list of orders and asks each order's slots, and both
        # were hand edits of its script. `--ring` lays each node's `prev` as
        # its predecessor on the `next` walk from the head; `--ecx-answer`
        # makes a slot answer from the word at its own `ecx`.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2400, win=0x20, bias=[], own=0, ret='none',
            stubs=['slot:0x14:0', 'slot:0x94:0'], ecx_answer=['0:8', '1:0x18:0x14'],
            ring=['m+0xdc:0..3:4:0:0xe90000:0x10'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("ECXANSWER = [(0, 8, None), (1, 24, 20)]", script)
        self.assertIn("RING = [('m', 220, 0, 3, 4, 0, 15269888, 16)]", script)
        self.assertIn('answers from its own `ecx`', test)
        self.assertIn('a circular list in `c0..c3`', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0, stubs=['slot:0x14:0'],
                                   ecx_answer=['1:8'])
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0, ring=['m+0:3..1:4:0:0:16'])
        # The ring repair, run on four nodes whose `next` chain wraps the
        # head: every `prev` is the predecessor, the head's the last node.
        body = script[script.index('        for hwin, hoff, lo_, hi_, noff, poff, base_, stride_ in RING:'):]
        body = body[:body.index('        for a_, b in wins:')]
        import random, textwrap
        for seed in range(30):
            mem = bytearray(0x100)
            c = [bytearray(16) for _ in range(4)]
            nxt = {0: 2, 2: 1, 1: 3, 3: 0}
            for k, v in nxt.items():
                c[k][4:8] = (0xe90000 + 0x10 * v).to_bytes(4, 'little')
            mem[0xdc:0xe0] = (0xe90000).to_bytes(4, 'little')
            ns = {'RING': [('m', 0xdc, 0, 3, 4, 0, 0xe90000, 16)], 'mem': mem, 'c': c, 'p': [],
                  'rng': random.Random(seed), 'i32': lambda w, o: int.from_bytes(w[o:o + 4], 'little', signed=True)}
            exec(textwrap.dedent(body), ns)
            prev = {k: (int.from_bytes(c[k][0:4], 'little') - 0xe90000) // 16 for k in range(4)}
            if prev == {0: 3, 2: 0, 1: 2, 3: 1}:
                continue
            # An early stop: one later node's prev is the head.
            self.assertEqual(sum(v == 0 for v in prev.values()), 2, prev)
            break
        else:
            self.fail('no seed drew the early stop')

    def test_a_hooked_callee_may_answer_eax_and_ecx(self):
        # Item 2818 (parked 2816): a callee whose `ecx` the caller reads on
        # return (the decompiler's `extraout_ECX`) was a hand edit; a
        # `--ecx-out IDX:V,V,..` stub answers `(eax, ecx)`, the `ecx` drawn
        # from the list and printed after the other numbers on the row.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2818, win=0x20, bias=[], own=0, ret='none',
            stubs=['0x402000:1'], ecx_out=['0:0,5,0x1234'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("ECXOUT = [(0, [0, 5, 4660])]", script)
        self.assertIn('leaves `ecx` as', test)
        self.assertIn('h.answers[names[idx]] = pair_answer(', script)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0,
                                   stubs=['0x402000:1'], ecx_out=['1:5'])

    def test_an_iat_stub_may_be_cdecl(self):
        # Item 2400 (`Objects::add_ammo`): `malloc` is `call [VA]; add esp, 4`,
        # an `iat` stub whose caller pops its argument; `--stub-cdecl` took
        # direct callees only.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2400, win=0x20, bias=[], own=0, ret='none',
            stubs=['iat:0xac54f0:1'], stub_cdecl=['0'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('CDECLSTUBS = [0]', script)
        self.assertIn('h.hooked[sa][:3] + (True,)', script)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2400, win=0x20, bias=[], own=0,
                                   stubs=['slot:0x10:0'], stub_cdecl=['0'])

    def test_a_stub_may_print_the_bytes_an_argument_points_at(self):
        # Item 2379 (parked 2348): `CommandManager::issue_speed_up` and its
        # siblings hand `add_command` the address of a local `Command` whose
        # bytes are gone once the call returns, and `--record` printed only
        # its depth (`sp-0x9`), so a break that writes the wrong command type
        # passed. `--deref IDX:K:N` reads N bytes at stub IDX's argument K at
        # call time, hex, calls joined by `/`, `-` when it was not called,
        # `null` for a null argument, after the record's columns.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2379, win=0x20, bias=[], own=0,
            stubs=['0x402000:0', '0x403000:2'], deref=['1:0:2'], ret='none',
            port='{ log.push("3501".into()); 0 }')
        compile(script, str(sp), 'exec')
        self.assertIn('DEREF = [(1, 0, 2, None)]', script)
        self.assertIn('def read_arg(', script)
        self.assertIn('split_once(" # ")', test)
        self.assertIn('let mut log: Vec<String> = Vec::new();', test)
        self.assertIn('stub 1 prints the 2 bytes its argument 0 points at', test)
        # The wrapper reads at call time and answers as the stub would.
        ns = {}
        body = script[script.index('def read_arg('):]
        exec(body[:body.index('\n\n\ndef ')], ns)
        seen = []
        ans = ns['read_arg'](7, 0, 2, lambda a, n: bytes([a & 0xFF, n]), seen)
        self.assertEqual(ans(0, [0x35, 1]), 7)
        self.assertEqual(ans(0, [0, 1]), 7)
        self.assertEqual(seen, ['3502', 'null'])
        _, script2, _, test2, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2379, win=0x20, bias=[], own=0, stubs=['0x402000:2'], ret='none', port='0')
        self.assertIn('DEREF = []', script2)
        self.assertNotIn('split_once(" # ")', test2)
        for bad in (['1:0:2'], ['0:2:2'], ['0:0:0']):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2379, win=0x20, bias=[], own=0,
                                       stubs=['0x402000:2'], deref=bad)

    def test_a_deref_may_hold_only_the_bytes_the_function_wrote(self):
        # Item 2644 (parked 2636): `UnitOut::issue_come_out` hands `add_command`
        # a 16-byte `Command` local of which five bytes are written; the rest
        # is the emulator's stack residue. `--deref IDX:K:N:W` names the
        # written byte ranges W (`0,1-4`); every other byte prints `00`.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2644, win=0x20, bias=[], own=0,
            stubs=['0x402000:2'], deref=['0:0:4:0,2-3'], ret='none',
            port='{ log.push("31003300".into()); 0 }')
        compile(script, str(sp), 'exec')
        self.assertIn('DEREF = [(0, 0, 4, (0, 2, 3))]', script)
        self.assertIn('the 4 bytes its argument 0 points at (bytes 0, 2, 3 written, the rest 00)', test)
        ns = {}
        body = script[script.index('def read_arg('):]
        exec(body[:body.index('\n\n\ndef ')], ns)
        seen = []
        ans = ns['read_arg'](7, 0, 4, lambda a, n: bytes([0x31, 0x99, 0x33, 0x44]), seen, (0, 2, 3))
        self.assertEqual(ans(0, [0x1000]), 7)
        self.assertEqual(seen, ['31003344'])
        for bad in ('0:0:4:4', '0:0:4:2-1', '0:0:4:x'):
            with self.assertRaises(ValueError):
                sweep_gen.generate_obj('Foo::bar@00401000', item=2644, win=0x20, bias=[], own=0,
                                       stubs=['0x402000:2'], deref=[bad])

    def test_a_stub_may_print_an_absolute_cell_read_at_call_time(self):
        # Item 2513 (parked 2509): `find_wpath_army`, `find_road_restore` and
        # `find_upath_restore` set `pathfinder+0x70`/`+0x84` to 1 around the
        # hooked callee and 0 on return; the `--at` window saw only the 0.
        # `--deref IDX:@VA:N` reads N bytes at the absolute cell VA when stub
        # IDX is called, printed beside the argument derefs.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2513, win=0x20, bias=[], own=0,
            stubs=['0x402000:1'], deref=['0:@0xe85eb0:4'], ret='none',
            port='{ log.push("01000000".into()); 0 }')
        compile(script, str(sp), 'exec')
        self.assertIn("DEREF = [(0, ('@', 15228592), 4, None)]", script)
        self.assertIn('prints the 4 bytes at the cell 0x00e85eb0 when it is called', test)
        ns = {}
        body = script[script.index('def read_arg('):]
        exec(body[:body.index('\n\n\ndef ')], ns)
        seen = []
        ans = ns['read_arg'](7, ('@', 0x1000), 2, lambda a, n: bytes([a >> 8, n]), seen)
        self.assertEqual(ans(0, [0]), 7)
        self.assertEqual(seen, ['1002'])
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2513, win=0x20, bias=[], own=0,
                                   stubs=['0x402000:1'], deref=['0:@0x1000:0'])

    def test_an_obj_may_draw_more_rows(self):
        # Item 2314 (Loop 2003, 2287's line): `Object::valid_target` and
        # `UnitData::is_gathering_at` each had an arm few of 220 rows
        # reached, and 2287 edited the generated `range(220)` by hand.
        # `--rows N` draws N rows, on the plain obj and the windowed one;
        # a line without it draws the 220 it always did.
        for kw in ({}, {'stubs': ['0x402000:1']}):
            _, script, _, test, _ = sweep_gen.generate_obj(
                'Foo::bar@00401000', item=2314, win=0x20, bias=[], own=1, rows=600, port='0', **kw)
            self.assertIn('for i in range(600):', script)
            self.assertNotIn('range(220)', script)
            self.assertIn('600 rows', test)
            # The test asks for every row drawn, not the default's 200 (2314's num_inside).
            self.assertIn('assert!(n >= 600,', test)
            _, script2, _, test2, _ = sweep_gen.generate_obj(
                'Foo::bar@00401000', item=2314, win=0x20, bias=[], own=1, port='0', **kw)
            self.assertIn('for i in range(220):', script2)
            self.assertNotIn('600 rows', test2)
            self.assertIn('assert!(n >= 200,', test2)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2314, win=0x20, bias=[], own=1, rows=50)

    def test_a_record_may_split_by_the_table_row(self):
        # Item 2309 (2298's Loop line, Loop 2003): `Game::defeat_all` walks
        # the image's leaders table (`--sparse @`) and calls two stubs a
        # row; a plain `--record` reads nine rows' calls as one `/`-joined
        # token. `--record IDX@S` splits stub IDX's calls by the row of
        # sparse table S its `ecx` lies in: one token a row, `ecx` written
        # `r+OFF` within it, and a last token for the calls outside it.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2309, win=0x20, bias=[], own=0,
            sparse=['@0xe6e000:0x6eec:3:0,4'], stubs=['0x402000:1', '0x403000:2'],
            record=['0', '1@0'], ret='none', port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('RECORD = [0, 1]', script)
        self.assertIn('ROWSPLIT = {1: 0}', script)
        self.assertIn('after ` # `, split by the rows of sparse table 0', test)
        ns = {'STACK_BASE': 0x7FF0_0000, 'STACK_SIZE': 0x10_0000}
        for fn in ('def token(', 'def by_row('):
            body = script[script.index(fn):]
            exec(body[:body.index('\n\n\ndef ')], ns)
        spans = [('m', 0x1000, 0x20)]
        calls = [(0x100, [5, 0x1004]), (0x100 + 2 * 0x10 + 4, [-1, 0]), (0x77, [1, 2])]
        self.assertEqual(ns['by_row'](calls, 0x100, 0x10, 3, spans),
                         ['r+0x0,5,m+0x4', '-', 'r+0x4,-1,0', '119,1,2'])
        self.assertEqual(ns['by_row']([], 0x100, 0x10, 2, spans), ['-', '-', '-'])
        # A split names a --sparse table that exists, and a recorded stub.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2309, win=0x20, bias=[], own=0,
                                   stubs=['0x402000:1'], record=['0@0'])
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2309, win=0x20, bias=[], own=0,
                                   sparse=['@0xe6e000:0x6eec:3:0,4'], stubs=['0x402000:1'], record=['0@1'])

    def test_a_record_may_name_the_callee_in_call_order(self):
        # Item 2341 (2314's Loop line, parked 2318, Loop 2003):
        # `Unit::add_think_order` calls `clear_partial_path` and then
        # `update_action`, both argument-less on one `ecx`, and `--record`
        # prints each stub's calls in a column of its own, in stub order,
        # so the break that swapped them passed. `--record-order` prints
        # one token a call, in the order the calls were made, each headed
        # by the callee's stub index (`s3:m+0x0`).
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2341, win=0x20, bias=[], own=0,
            stubs=['0x402000:0', '0x403000:0'], record=['0', '1'], record_order=True,
            ret='none', port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('RECORDORDER = True', script)
        self.assertIn('in call order, each headed `sIDX:`', test)
        ns = {'STACK_BASE': 0x7FF0_0000, 'STACK_SIZE': 0x10_0000}
        for fn in ('def token(', 'def in_order('):
            body = script[script.index(fn):]
            exec(body[:body.index('\n\n\ndef ')], ns)
        spans = [('m', 0x1000, 0x20)]
        names = ['a', 'b', 'c']
        calls = [('b', 0x1000, ()), ('c', 0x1000, ()), ('a', 0x1000, (5,)), ('b', 0x77, ())]
        self.assertEqual(ns['in_order'](calls, [0, 1], names, spans),
                         ['s1:m+0x0', 's0:m+0x0,5', 's1:119'])
        self.assertEqual(ns['in_order'](calls, [0], names, spans), ['s0:m+0x0,5'])
        self.assertEqual(ns['in_order']([], [0, 1], names, spans), ['-'])
        # Without the flag the columns are as before.
        _, script2, _, test2, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2341, win=0x20, bias=[], own=0,
            stubs=['0x402000:0', '0x403000:0'], record=['0', '1'], ret='none', port='0')
        self.assertIn('RECORDORDER = False', script2)
        self.assertNotIn('in call order', test2)
        # It orders what --record records, and a row split has its own columns.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2341, win=0x20, bias=[], own=0,
                                   stubs=['0x402000:0'], record_order=True)
        # (Item 2421: an order may now carry a row split; see the next test.)

    def test_a_stub_may_answer_through_its_pointer_argument(self):
        # Item 2325 (2309's Loop line, Loop 2003): `ObjectData::in_a_building`
        # hands a callee the address of a local, and the callee writes the
        # player index there that the function then indexes a table by; a
        # stub answered `eax` alone and left the local as the caller laid it.
        # `--outarg IDX:K:VALS` writes a value drawn from VALS through stub
        # IDX's argument K, printed after the answers, `o` in a place.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2325, win=0x20, bias=[], own=0,
            stubs=['0x402000:1'], outarg=['0:0:0..7'],
            places=['0xc0ab84+o[0]*0x1c:s[0]*4:4'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn("OUTARG = [(0, 0, ('range', 0, 7))]", script)
        self.assertIn('stub 0 writes a value in 0..7 through its argument 0', test)
        ns = {}
        body = script[script.index('def out_answer('):]
        exec(body[:body.index('\n\n\ndef ')], ns)
        wrote = []
        answer = ns['out_answer'](5, 0, 3, lambda a, v: wrote.append((a, v)))
        self.assertEqual(answer(0x99, [0x1000]), 5)
        self.assertEqual(wrote, [(0x1000, 3)])
        # A null pointer argument is not written through, as `if (p) *p = ..`.
        self.assertEqual(answer(0x99, [0]), 5)
        self.assertEqual(wrote, [(0x1000, 3)])
        # An inner answer that is itself a function (`--seq`) still answers.
        answer = ns['out_answer'](lambda e, a: e + 1, 0, 3, lambda a, v: wrote.append((a, v)))
        self.assertEqual(answer(0x99, [0x2000]), 0x9a)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2325, win=0x20, bias=[], own=0,
                                   stubs=['0x402000:1'], outarg=['1:0:0..7'])
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2325, win=0x20, bias=[], own=0,
                                   stubs=['0x402000:1'], outarg=['0:1:0..7'])

    def test_a_stubbed_callee_may_leave_its_arguments_to_the_caller(self):
        # Item 2325: `BuildQueue::un_queue` calls `memcpy` (an import, which
        # the emulator cannot enter) and pops its three arguments itself
        # (`add esp, 0xc`); a stub that popped them too left `esp` twelve
        # bytes high. `--stub-cdecl IDX` hooks stub IDX as `__cdecl`: its
        # arguments are read and recorded, and only the return is popped.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2325, win=0x20, bias=[], own=0,
            stubs=['0x402000:3', '0x403000:1'], stub_cdecl=['0'], record=['0'], ret='none', port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('CDECLSTUBS = [0]', script)
        self.assertIn('stub 0 is `__cdecl`: the caller pops its arguments', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2325, win=0x20, bias=[], own=0,
                                   stubs=['slot:0x10:1'], stub_cdecl=['0'])
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2325, win=0x20, bias=[], own=0,
                                   stubs=['0x402000:3'], stub_cdecl=['1'])
        # The hook pops the return address alone for a `__cdecl` callee.
        src = (ROOT / 'tools/emu/hooks.py').read_text()
        self.assertIn('cdecl', src[src.index('def _on_code'):])

    def test_an_obj_may_read_its_answer_from_xmm0(self):
        # Item 2537 (parked 2534): the five render getters print `in_ST0` in
        # the decompile, and their listings return the float in `xmm0`
        # (`movss xmm0, ...; ret`), which `eax` never reads. `--ret xmm0`
        # holds the low 32 bits of `xmm0` against the port's bits.
        _, script, _, test, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2537, win=0x20, bias=[], own=0, ret='xmm0', port='0',
            cells=['0xc061e8:0x10'])
        self.assertIn("RET = 'xmm0'", script)
        self.assertIn('ret=RET', script)
        self.assertIn('assert_eq!(got, want_eax', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=0x20, bias=[], ret='st1')

    def test_machine_call_returns_xmm0_bits_on_request(self):
        try:
            import callfn
            import unicorn  # noqa: F401
        except ImportError:
            self.skipTest('unicorn is not installed (uv run --with unicorn==2.1.4)')

        class Img:
            base, size = 0x400000, 0x1000
            sections = [('.text', 0, 0x1000, b'\xf3\x0f\x10\x01\xc3')]  # movss xmm0, [ecx]; ret

            def map_into(self, uc):
                uc.mem_map(self.base, self.size)
                uc.mem_write(self.base, self.sections[0][3])

        m = callfn.Machine(Img())
        m.uc.mem_map(0x10000, 0x1000)
        m.uc.mem_write(0x10000, (0x3F800000).to_bytes(4, 'little'))
        self.assertEqual(m.call(0x400000, ecx=0x10000, ret='xmm0'), 0x3F800000)
        self.assertEqual(m.call(0x400000, ecx=0x10000), 0)  # eax, untouched by the getter: the default

    def test_an_obj_may_hold_only_the_low_byte_of_its_answer(self):
        # Item 2336 (parked 2327): `Game::is_game_ready` answers a `bool` by
        # `sete al` over an `eax` whose upper bytes are the game pointer's,
        # so a whole-`eax` compare holds the port to a pointer it never
        # sees. `--ret al` compares the low byte only.
        _, script, _, test, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2336, win=0x20, bias=[], own=0, ret='al', port='0',
            cells=['0xc06210:0x10'])
        self.assertIn('assert_eq!(got & 0xff, want_eax & 0xff', test)
        self.assertNotIn('assert_eq!(got, want_eax', test)
        _, _, _, test, _ = sweep_gen.generate_obj('Foo::bar@00401000', item=2336, win=0x20, bias=[], ret='al')
        self.assertIn('assert_eq!(got & 0xff, want_eax & 0xff', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2336, win=0x20, bias=[], ret='ax')
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            try:
                self.assertEqual(sweep_gen.main(['obj', 'X::y@00401000', '--item', '1', '--ret', 'al']), 0)
                self.assertIn('assert_eq!(got & 0xff, want_eax & 0xff', (Path(tmp) / 'x_y.rs').read_text())
                self.assertEqual(sweep_gen.main(['obj', 'X::z@00401000', '--item', '1', '--ret', 'bogus']), 2)
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_a_stub_may_be_laid_in_an_import_cell(self):
        # Item 2336 (parked 2327): `TurnControl::get_lag_time` calls
        # `timeGetTime` through its IAT cell (`call [0xac5460]`), which no
        # import resolves under the emulator; a hooked address cannot answer
        # an indirect call. `--stub iat:VA:NARGS` lays a stub's address in
        # the cell, so the call lands on the stub and is answered.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2336, win=0x40, bias=[], own=0,
            stubs=['iat:0xac5460:0', '0x402000:1'], answers=['0:5,6'], record=['0'])
        compile(script, str(sp), 'exec')
        self.assertIn("('iat', 11293792, 0)", script)
        self.assertIn('the import cell 0x00ac5460 holds a stub (0 args)', test)
        # The cell is laid every row with the stub's address, and only the
        # direct callee is hooked by its own address.
        self.assertIn('if k == "iat"', script)
        self.assertIn('iatac5460', script.replace('iat{a:x}', 'iatac5460'))
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2336, win=0x20, bias=[],
                                   stubs=['iat:0xac5460'])
        # (Item 2400 lifted the refusal of `--stub-cdecl` on an `iat` stub:
        # `malloc` is `call [VA]; add esp, 4`; see test_an_iat_stub_may_be_cdecl.)

    def test_a_recorded_local_is_named_by_its_depth(self):
        # Item 2336: `MiscAccess::coord_vis@00583aa0` hands its callees the
        # addresses of its own locals; a record printed them as raw stack
        # words, which a port can only copy. A word inside the stack below
        # the caller's `esp` prints as `sp-0xNN`, its depth under it.
        _, script, _, _, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2336, win=0x20, bias=[], own=0,
            stubs=['0x402000:1'], record=['0'])
        ns = {'STACK_BASE': 0x7FF0_0000, 'STACK_SIZE': 0x10_0000}
        body = script[script.index('def token('):]
        exec(body[:body.index('\n\n\ndef ')], ns)
        top = ns['STACK_BASE'] + ns['STACK_SIZE'] - 0x100
        self.assertEqual(ns['token'](top - 0x14, []), 'sp-0x14')
        self.assertEqual(ns['token'](top - 0x2000, []), str(top - 0x2000))
        self.assertEqual(ns['token'](5, []), '5')

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

    def test_a_fwd_gate_writes_a_gate_and_forward_forwarder(self):
        # Item 2443 (parked 2433, Loop 2003): `GroupOut::issue_*` tests a byte
        # of `gamec`, then forwards its arguments to `CommandManager::issue_*`
        # with the caller's stale `ecx` first. One flag writes what seven
        # hand-written `--port`s of item 2421 wrote: the window, the cell, the
        # byte's draw, the stub of N+1 arguments, the record and the port.
        x = sweep_gen.fwd_gate_expand('0x820:0x10:0x9415e0', own=4)
        self.assertEqual(x['stub'], ['0x9415e0:5'])
        self.assertEqual(x['cells'], ['0xc061e8:0x900'])
        self.assertEqual(x['wbias'], ['c0+0x820:0,0x10,0x20,0x30,0xef'])
        self.assertEqual(x['record'], ['0'])
        self.assertEqual(
            x['port'],
            '{ if c[0][0x820] & 0x10 != 0 { log.push("-".into()); } else { log.push(format!('
            '"m+0x0,m+0x0,{},{},{},{}", a[0], a[1], a[2], a[3])); } 0 }')
        # A constant tail (`hotkey`'s zero) is one more stub argument.
        t = sweep_gen.fwd_gate_expand('0x820:0x10:0x941fd0:0', own=1)
        self.assertEqual(t['stub'], ['0x941fd0:3'])
        self.assertIn('"m+0x0,m+0x0,{},0", a[0]', t['port'])
        with self.assertRaises(ValueError):
            sweep_gen.fwd_gate_expand('0x820:0x10', own=1)

    def test_a_cmd_gate_writes_issue_halts_shape(self):
        # Item 2454 (parked 2444, Loop 2003): seven of 2443's ten were
        # `issue_halt`'s shape -- a gamec byte gate, `check_accept_issue`,
        # `add_group`, the command built in a local, `add_command` -- and
        # were hand `--port`s. One flag writes the stubs, the answers, the
        # records, the deref of the local and the port; the `sp-` token is
        # `c + 8 + 4*own` with `c` the local's size rounded up to four.
        x = sweep_gen.cmd_gate_expand('0x820:0x10:2:5:a0', own=1)
        self.assertEqual(x['stub'], ['0x940a70:0', '0x94bb60:1', '0x94bae0:2'])
        self.assertEqual(x['answers'], ['0:0,1', '1:0,1'])
        self.assertEqual(x['record'], ['0', '1', '2'])
        self.assertEqual(x['deref'], ['2:0:5'])
        self.assertEqual(x['wbias'], ['c0+0x820:0,0x10,0x20,0x30,0xef'])
        self.assertIn('log.push("15269768,sp-0x14,5".into()); log.push(format!("02{}", le(a[0])));', x['port'])
        self.assertIn('if a[1] == 0 {', x['port'])
        self.assertIn('if a[2] == 0 {', x['port'])
        # A byte argument, a constant word and a guard before the check.
        y = sweep_gen.cmd_gate_expand('0x820:0x10:0xa:10:a0,a1,b2', own=3)
        self.assertIn('format!("0a{}{}{:02x}", le(a[0]), le(a[1]), a[2] as u8)', y['port'])
        self.assertIn('sp-0x20,10', y['port'])
        k = sweep_gen.cmd_gate_expand('0x820:0x10:0x21:9:a0,k1', own=2)
        self.assertIn('format!("21{}{}", le(a[0]), "01000000")', k['port'])
        g = sweep_gen.cmd_gate_expand('0x820:0x10:5:13:a0,a1,a2::a0>=0,a1>=0', own=3)
        self.assertIn('if a[0] < 0 || a[1] < 0 {', g['port'])
        with self.assertRaises(ValueError):
            sweep_gen.cmd_gate_expand('0x820:0x10:2', own=1)

    def test_a_cmd_obj_reads_a_commands_fields_off_an_object_argument(self):
        # Item 2469 (parked 2466, Loop 2003): `issue_come_out` and both
        # `issue_unqueue` read a `UnitOut`/`WallOut` through argument 0 and
        # write `cmd, zx(byte+9), sx(word+0xa), [a1 | 0], word+0x30`; 2464
        # wrote the three as hand `--port`s. `--cmd-obj CMD:SIZE:FIELD,..`
        # writes the stubs, the deref and the port: `zOFF` a byte widened to
        # a dword, `sOFF` a signed word widened to a dword, `wOFF` a word's
        # two bytes as they lie, `aN` an argument, `kV` a constant.
        x = sweep_gen.cmd_obj_expand('0x31:11:z9,s0xa,w0x30', own=1)
        self.assertEqual(x['stub'], ['0x940a70:0', '0x94bae0:2'])
        self.assertEqual(x['answers'], ['0:0,1'])
        self.assertEqual(x['record'], ['0', '1'])
        self.assertEqual(x['deref'], ['1:0:11'])
        self.assertEqual(x['ptrarg'], ['0:0x40'])
        self.assertIn('if a[1] == 0 {', x['port'])
        self.assertIn('log.push("15269768,sp-0x18,11".into());', x['port'])
        self.assertIn('format!("31{:02x}000000{}{:02x}{:02x}", p[0][0x9], le(w0xa), p[0][0x30], p[0][0x31])', x['port'])
        y = sweep_gen.cmd_obj_expand('0x30:15:z9,s0xa,a1,w0x30', own=2)
        self.assertIn('if a[2] == 0 {', y['port'])
        self.assertIn('sp-0x20,15', y['port'])
        self.assertIn('format!("30{:02x}000000{}{}{:02x}{:02x}"', y['port'])
        z = sweep_gen.cmd_obj_expand('0x30:15:z9,s0xa,k0,w0x30', own=2)
        self.assertIn('"00000000"', z['port'])
        with self.assertRaises(ValueError):
            sweep_gen.cmd_obj_expand('0x30:15:q9', own=1)

    def test_a_cmd_byte_is_a_group_id_and_a_one_byte_command(self):
        # Item 2481 (parked 2472, Loop 2003): `issue_recall`, `_scramble`,
        # `_stop_spell` and `_alarm` take a group id (a plain integer, not a
        # buffer), pass it through `add_group` and hand `add_command` a
        # one-byte local; four hand `--port`s. `--cmd-byte CMD[:SIZE]`.
        x = sweep_gen.cmd_byte_expand('0x23', own=1)
        self.assertEqual(x['stub'], ['0x940a70:0', '0x94bb60:1', '0x94bae0:2'])
        self.assertEqual(x['answers'], ['0:0,1', '1:0,1'])
        self.assertEqual(x['record'], ['0', '1', '2'])
        self.assertEqual(x['deref'], ['2:0:1'])
        self.assertIn('log.push(format!("15269768,{}", a[0]));', x['port'])
        self.assertIn('log.push("15269768,sp-0xd,1".into()); log.push("23".into());', x['port'])
        self.assertIn('if a[1] == 0 {', x['port'])
        self.assertIn('if a[2] == 0 {', x['port'])
        with self.assertRaises(ValueError):
            sweep_gen.cmd_byte_expand('', own=1)

    def test_a_cmd_seh_is_a_leader_checked_command_with_a_feedback_log_arm(self):
        # Item 2486 (parked 2482, Loop 2003): `issue_treaty` and its six
        # siblings build a command from the leader a `LeaderOut` names and
        # argument 1, add it when both leaders' flag bytes are set, and
        # else -- `ignore_always` clear -- run an SEH-framed feedback arm
        # (two `String`s, `Error::report`, two closes). `--cmd-seh
        # CMD:SIZE:FIELD,..:C:IGN:WIDE:NARROW:DW:DN`: the arm's stubs are
        # recorded, the command bytes compared as `--cmd-gate` does.
        x = sweep_gen.cmd_seh_expand('0x25:13:w,a1,a2:0x1c:0xee12d4:0xaf7008:0xaf7178:0x34:0x20', own=3)
        self.assertEqual(x['stub'], ['0x940a70:0', '0x94bae0:2', '0xa1edd0:1', '0xa1d660:1', '0xa2e550:6', '0xa1cf40:0'])
        self.assertEqual(x['answers'], ['0:0,1'])
        self.assertEqual(x['answer_range'], ['4:0:0'])
        self.assertEqual(x['record'], ['0', '1', '2', '3', '4', '5'])
        self.assertEqual(x['deref'], ['1:0:13'])
        self.assertEqual(x['ptrarg'], ['0:0x10'])
        self.assertEqual(x['wbias'][0], 'p0+8:0..3')
        self.assertEqual(x['sparse'], ['@0xe3a390:0x6eec:4:0,4'])
        self.assertEqual(x['argrange'], ['1:0:3'])
        self.assertIn('p1+0:1,1,1,3,0', x['wbias'])
        self.assertIn('13161384,sp-0x34,sp-0x34,0,0,c0+0x8,1', x['port'])
        self.assertIn('log.push("15269768,sp-0x30,13".into());', x['port'])
        self.assertIn('sp-0x48,11497480', x['port'])
        self.assertIn('sp-0x34,11497848', x['port'])
        self.assertIn('format!("25{}{}{}", le(who), le(a[1]), le(a[2]))', x['port'])
        with self.assertRaises(ValueError):
            sweep_gen.cmd_seh_expand('0x25:13:q:0x1c:0xee12d4:0xaf7008:0xaf7178:0x34:0x20', own=3)

    def test_a_chain_is_a_three_level_pointer_walk_through_a_global_cell(self):
        # Item 2495 (parked 2484, Loop 2003): `PathFinder::find_wpath` and its
        # three siblings read `[[[objects + 0x14 + 0x1c*p2] + 4*p3] + 0x10]`.
        # `--chain CELL:OFF:STRIDE:P:I:N:SIZE`: the cell's window laid wide
        # enough for N rows, the row word (argument P picks the row) a place
        # at a table of N words, the table word (argument I picks it) a place
        # at a SIZE-byte record, and both arguments drawn inside 0..N-1.
        x = sweep_gen.chain_expand('0xc0618c:0x14:0x1c:1:2:4:0x18', own=3, cell_index=0, pbase=0)
        self.assertEqual(x['cells'], ['0xc0618c:0x90'])
        self.assertEqual(x['place'], ['c0+0x14+0x1c*a[1]:0:16', 'p0+4*a[2]:0:0x18'])
        self.assertEqual(x['argrange'], ['1:0:3', '2:0:3'])
        y = sweep_gen.chain_expand('0xc0618c:0x14:0x1c:1:2:4:0x18', own=3, cell_index=1, pbase=2)
        self.assertEqual(y['place'], ['c1+0x14+0x1c*a[1]:0:16', 'p2+4*a[2]:0:0x18'])
        with self.assertRaises(ValueError):
            sweep_gen.chain_expand('0xc0618c:0x14:0x1c:1:5:4:0x18', own=3, cell_index=0, pbase=0)

    def test_an_ecx_derived_from_this_prints_as_this_plus_its_offset(self):
        # Item 2464 (parked 2455, Loop 2003): `this + 0x28` handed to a callee
        # lies past the `m` window (WIN bytes) and printed as the absolute
        # address (`536870952`), so a regenerated port needed a hand patch in
        # its kept line. A `this+OFF` span follows the windows: inside the
        # window the token stays `m+OFF`, past it `this+OFF`.
        _, script, _, _, _ = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2464, win=0x20, bias=[], own=0,
            stubs=['0x402000:1'], record=['0'])
        self.assertIn('+ [("this", this, 0x1000)]', script)
        ns = {'STACK_BASE': 0x7FF0_0000, 'STACK_SIZE': 0x10_0000}
        body = script[script.index('def token('):]
        exec(body[:body.index('\n\n\ndef ')], ns)
        spans = [('m', 0x2000_0000, 0x20), ('this', 0x2000_0000, 0x1000)]
        self.assertEqual(ns['token'](0x2000_0010, spans), 'm+0x10')
        self.assertEqual(ns['token'](0x2000_0028, spans), 'this+0x28')

    def test_a_cmd_gate_without_a_gate_is_a_command_manager_issuer(self):
        # Item 2454: `CommandManager::issue_*` is the same shape with no gate:
        # argument 0 is the group (a buffer, handed to `add_group` by name)
        # and a sound call may stand before the check.
        u = sweep_gen.cmd_gate_expand('::0xe:5:a1', own=2)
        self.assertEqual(u['cells'], [])
        self.assertEqual(u['ptrarg'], ['0:0x20'])
        self.assertTrue(u['port'].startswith('{ let le'))
        self.assertIn('"15269768,p0+0x0"', u['port'])
        self.assertIn('if a[2] == 0 {', u['port'])
        self.assertNotIn('c[0]', u['port'])
        s = sweep_gen.cmd_gate_expand('0x820:0x10:0x12:9:a0,k2::: 0x97f770:0x2b'.replace(' ', ''), own=1)
        self.assertEqual(s['stub'][3], '0x97f770:1')
        self.assertEqual(s['record'], ['0', '1', '2', '3'])
        # The emulator prints in stub order and the deref last: the sound
        # entry stands before each arm's final push.
        self.assertIn('log.push("-".into()); log.push("-".into()); log.push("m+0x0,43".into()); log.push("-".into());', s['port'])
        self.assertIn('log.push("m+0x0,43".into()); log.push(format!("12{}{}"', s['port'])
        self.assertEqual(s['port'].count('log.push("-".into());'), 5 + 3 + 2)

    def test_a_vtordisp_thunk_moves_this_by_the_word_before_it(self):
        # Item 2362 (parked 2348, Loop 2003): a `vtordisp` thunk is `sub ecx,
        # [ecx-4]; jmp target`, and `fwd` drew `this` as an address nothing
        # maps, so the thunk's one read faulted. The `vtordisp` token lays
        # `this` in the heap with a drawn displacement at `this-4`, prints it
        # sixth on the row, and holds the callee's `ecx` to `this - disp`.
        sp, script, tp, test, mod = sweep_gen.generate_fwd(
            'MoveOrder::is_pathed_vtordisp@0048898f', item=1, calls=['0x488980:0:vtordisp:'], ret='last')
        compile(script, str(sp), 'exec')
        self.assertIn('VTORDISP = True', script)
        self.assertIn('let disp = a[5];', test)
        self.assertIn('(this as u32).wrapping_sub(disp as u32)', test)
        _, plain, _, ptest, _ = sweep_gen.generate_fwd('X::y@00401000', item=1, calls=['0x401000:0:this:'])
        self.assertIn('VTORDISP = False', plain)
        self.assertNotIn('disp', ptest)

    def test_a_fwd_order_token_names_the_callee_of_each_call(self):
        # Item 2672 (parked 2666): `fwd` printed each call's `ecx` and
        # arguments, not which callee, so five calls on one `this` passed
        # with two of them swapped. `order=True` heads each call `sIDX:`,
        # IDX the callee's place in `--call`, in the script and the test.
        calls = ['0x401000:0:this:', '0x401010:0:this:']
        sp, script, tp, test, mod = sweep_gen.generate_fwd('X::y@00401000', item=1, calls=calls, order=True)
        compile(script, str(sp), 'exec')
        self.assertIn('ORDER = True', script)
        self.assertIn('format!("s{k}:', test)
        _, plain, _, ptest, _ = sweep_gen.generate_fwd('X::y@00401000', item=1, calls=calls)
        self.assertIn('ORDER = False', plain)
        self.assertNotIn('format!("s{k}:', ptest)

    def test_a_walk_may_record_only_its_call_count_and_ends(self):
        # Item 2672 (parked 2667): `Balance::walk_rules_data` calls its walker
        # 0x1ed * 0x1ed times; a row of them is a quarter-million hooked
        # calls. `count_only` prints one record a row: the count, the first
        # and last call, and the sums of the two arguments; the port is an
        # iterator of `(a0, a1)` pairs, and `rows` bounds the rows drawn.
        sp, script, tp, test, mod = sweep_gen.generate_walk(
            'Balance::walk_rules_data@00582cc0', item=1, a0='0xc12bf4', a1='0xc12bf6', count_only=True, rows=3,
            port='(0..4i64).map(|k| (0xc12bf4 + 2 * k, 0xc12bf6 + 2 * k))')
        compile(script, str(sp), 'exec')
        self.assertIn('ROWS = 3', script)
        self.assertIn('COUNT_ONLY = True', script)
        self.assertIn('max_insns=', script)
        self.assertIn('0xc12bf4 + 2 * k', test)
        self.assertIn('sum', test)
        self.assertIn('assert!(n >= 3', test)
        _, plain, _, ptest, _ = sweep_gen.generate_walk('X::y@00401000', item=1, a0='this', a1='this')
        self.assertIn('COUNT_ONLY = False', plain)
        self.assertNotIn('sum', ptest)

    def test_a_free_function_is_a_name(self):
        # Item 2346: `add_type_bucket2@00669140` and `add_type_bucket` are
        # free functions (`types.cpp`), and the name refused them; the census
        # backs one by `name@va` as it backs `get_unique_types@00669120`.
        self.assertEqual(sweep_gen.parse('add_type_bucket2@00669140'), ('add_type_bucket2', 0x669140))
        self.assertEqual(sweep_gen.snake('add_type_bucket2'), 'add_type_bucket2')
        _, test = self.check('const', 'add_type_bucket@00669160', port='0')
        self.assertIn('add_type_bucket@00669160', test)

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

    def test_a_generated_sweep_is_redrawn_in_place(self):
        # Item 2298 (Loop 2003, 2222): a starved draw was redrawn by deleting
        # both files, regenerating, and deduping `mod.rs` by hand, because
        # `--write-mod` appended its line again. `--regen` rewrites both files
        # and leaves `mod.rs` alone; it refuses a sweep that was never written.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            mod = Path(tmp) / 'mod.rs'
            mod.write_text('')
            try:
                line = ['obj', 'X::y@00401000', '--item', '1', '--win', '0x20']
                self.assertEqual(sweep_gen.main(line + ['--regen']), 2)
                self.assertEqual(sweep_gen.main(line + ['--write-mod']), 0)
                self.assertEqual(mod.read_text().count('mod x_y;'), 1)
                self.assertNotIn('ARGRANGE', (Path(tmp) / 'sweep_x_y.py').read_text())
                self.assertEqual(sweep_gen.main(line + ['--argrange', '0:0:2', '--regen']), 0)
                self.assertIn('ARGRANGE = [(0, 0, 2)]', (Path(tmp) / 'sweep_x_y.py').read_text())
                self.assertEqual(sweep_gen.main(line + ['--regen', '--write-mod']), 2)
                self.assertEqual(mod.read_text().count('mod x_y;'), 1)
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_regen_refuses_to_reset_a_hand_written_port(self):
        # Loop 2792 (2783's, 2807's, 2818's and 2844's lines, the thirty-eighth
        # pass): `--regen` with no `--port` dropped a hand-written port to
        # `todo!()` four times in one batch when the keep could not find its
        # block, and each cost a step. A regen that would reset a port that
        # is not `todo!()` refuses and names the three ways on; `--port-file`
        # reads the expression from a file (Loop 2796: a 3 KB port does not
        # fit a command), and an `@file` holds flags one a line.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            (Path(tmp) / 'mod.rs').write_text('')
            try:
                line = ['obj', 'X::y@00401000', '--item', '1', '--win', '0x20']
                self.assertEqual(sweep_gen.main(line + ['--write-mod']), 0)
                tp = Path(tmp) / 'x_y.rs'
                text = tp.read_text()
                todo = 'todo!("the port of X::y")'
                self.assertIn(todo, text)
                # The hand port replaces the todo line whole, and the lane
                # also touched the line above it, so the keep has no anchor.
                at = text.index(todo)
                above = text.rfind('\n', 0, text.rfind('\n', 0, at))
                hand = text[:above] + ' // by hand' + text[above:].replace(todo, '7')
                tp.write_text(hand)
                self.assertEqual(sweep_gen.main(line + ['--argrange', '0:0:2', '--regen']), 2,
                                 'a regen that would reset the port refuses')
                self.assertEqual(tp.read_text(), hand, 'and writes nothing')
                self.assertEqual(sweep_gen.main(line + ['--argrange', '0:0:2', '--regen', '--reset-port']), 0)
                self.assertIn(todo, tp.read_text())
                port = Path(tmp) / 'port.rs'
                port.write_text('{\n    let v = a[0];\n    v + 1\n}\n')
                self.assertEqual(sweep_gen.main(line + ['--argrange', '0:0:2', '--regen', '--port-file', str(port)]), 0)
                self.assertIn('let v = a[0];', tp.read_text())
                self.assertIn('v + 1', (Path(tmp) / 'sweep_x_y.py').read_text().split('# sweep_gen: ')[1],
                              'the kept line records the port text, not the file')
                flags = Path(tmp) / 'x_y.args'
                flags.write_text('# the flags, one a line\n--argrange\n0:0:3\n--regen\n')
                self.assertEqual(sweep_gen.main(line + [f'@{flags}']), 0)
                self.assertIn('ARGRANGE = [(0, 0, 3)]', (Path(tmp) / 'sweep_x_y.py').read_text())
                self.assertIn('let v = a[0];', tp.read_text(), 'the keep found the block this time')
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'
        self.assertEqual(sweep_gen.expand_argfiles(['a', '@/no/such/file', '--x']), ['a', '@/no/such/file', '--x'],
                         'a missing @file is left as it stands')

    def test_regen_given_a_whole_line_keeps_the_flags_the_line_leaves_out(self):
        # Loop 2508 (2507's Loop line; the thirty-fifth pass): `--regen` with
        # a whole line redrew `issue_toggle_pause` without the kept line's
        # `--cell-win` and `--wbias`, so the test named a cell it no longer
        # had; the sweep was deleted and drawn again. A whole line merges
        # the kept line's flags under its own: an option the new line sets
        # wins, one it leaves out is kept.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            (Path(tmp) / 'mod.rs').write_text('')
            try:
                line = ['obj', 'X::y@00401000', '--item', '1', '--win', '0x20']
                self.assertEqual(sweep_gen.main(line + ['--cell-win', '0x500000:0x40', '--write-mod']), 0)
                self.assertEqual(sweep_gen.main(line + ['--argrange', '0:0:2', '--regen']), 0)
                kept = sweep_gen.kept_line(Path(tmp) / 'sweep_x_y.py')
                self.assertIn('--cell-win', kept)
                self.assertIn('0x500000:0x40', kept)
                self.assertIn('--argrange', kept)
                self.assertEqual(kept.count('--item'), 1)
                self.assertEqual(sweep_gen.main(line + ['--cell-win', '0x600000:0x40', '--regen']), 0)
                kept = sweep_gen.kept_line(Path(tmp) / 'sweep_x_y.py')
                self.assertIn('0x600000:0x40', kept)
                self.assertNotIn('0x500000:0x40', kept)
                self.assertIn('--argrange', kept)
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_regen_replaces_an_option_given_in_equals_form_and_never_takes_the_kept_port(self):
        # Loop 2886 (five reaches, the thirty-ninth pass). 2946: `--rows=2400`
        # on the new line left the kept `--rows 1200` merged in after it,
        # so the old value won. 2921: the kept line's `--port` came back
        # over a port edited in the `.rs`. 2900: a merged `--table` grew a
        # second copy of its places in silence — `--fresh` starts clean,
        # and what is merged is said.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            (Path(tmp) / 'mod.rs').write_text('')
            try:
                line = ['obj', 'X::y@00401000', '--item', '1']
                self.assertEqual(sweep_gen.main(line + ['--win', '0x20', '--cell-win', '0x500000:0x40',
                                                        '--port', '0x7a7a', '--write-mod']), 0)
                tp = Path(tmp) / 'x_y.rs'
                tp.write_text(tp.read_text().replace('0x7a7a', '0x7b7b'))
                err = io.StringIO()
                with contextlib.redirect_stderr(err):
                    self.assertEqual(sweep_gen.main(line + ['--win=0x40', '--regen']), 0)
                kept = sweep_gen.kept_line(Path(tmp) / 'sweep_x_y.py')
                self.assertIn('--win=0x40', kept)
                self.assertNotIn('0x20', kept, 'the kept `--win 0x20` is replaced, not merged after the new value')
                self.assertIn('--cell-win', kept)
                self.assertIn('0x7b7b', tp.read_text(), 'the `.rs` port is the one kept')
                self.assertNotIn('0x7a7a', tp.read_text())
                self.assertNotIn('--port', kept)
                self.assertIn('--regen kept from the old line: --cell-win 0x500000:0x40', err.getvalue())
                # Fresh, the windows change and the port block moves: the line names its port.
                self.assertEqual(sweep_gen.main(line + ['--win', '0x20', '--port', '0x7c7c', '--regen', '--fresh']), 0)
                kept = sweep_gen.kept_line(Path(tmp) / 'sweep_x_y.py')
                self.assertNotIn('--cell-win', kept)
                self.assertNotIn('--fresh', kept)
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_the_header_names_the_p_windows_in_the_script_s_order(self):
        # Loop 2892 (three reaches): sparse slices are `p` windows before the
        # places, so a `--wbias p0+…` meant for the first place biased a
        # slice and every later `p` moved by rows × slices. The generated
        # doc comment lists the windows in the order the script numbers them.
        line = sweep_gen.window_order([(0, 0x100)], [], [('@', '0xe3a390', 0x6eec, 4, [(0, 4), (8, 2)])],
                                      [('m', 0, '0', 0x40), ('m', 4, '0', 0x40)], [(1, 0x20)])
        self.assertEqual(line, '`p` windows in order: p0 argument 0, p1..p8 sparse table 0 (4 rows × 2 slice(s), '
                               "row-major), p9..p10 the place(s), in flag order, p11 stub 1's answer window")
        self.assertEqual(sweep_gen.window_order([], [], [], [], []), '`p` windows in order: none')
        out = sweep_gen.generate_obj('X::y@00401000', item=1, win=0x20, bias=(), places=['m+0:0:0x40'])
        self.assertIn('`p` windows in order: p0 the place', out[3])

    def test_regen_merges_the_named_sweep_s_line_not_another_arm_s(self):
        # Item 2801 (`Search::valid_filter`'s 27 arms, `--name valid_filter_caseK`): `--regen` with a whole line
        # found the kept line by the function's token and took the first script that held it, so arm 3's redraw
        # merged arm 0's stubs and port. With `--name` the sweep is that name's script, no other.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            (Path(tmp) / 'mod.rs').write_text('')
            try:
                line = ['obj', 'X::y@00401000', '--item', '1', '--win', '0x20']
                self.assertEqual(sweep_gen.main(line + ['--name', 'x_y_a', '--cell-win', '0x500000:0x40', '--write-mod']), 0)
                self.assertEqual(sweep_gen.main(line + ['--name', 'x_y_b', '--cell-win', '0x600000:0x40', '--write-mod']), 0)
                self.assertEqual(sweep_gen.main(line + ['--name', 'x_y_b', '--argrange', '0:0:2', '--regen']), 0)
                kept = sweep_gen.kept_line(Path(tmp) / 'sweep_x_y_b.py')
                self.assertIn('0x600000:0x40', kept)
                self.assertNotIn('0x500000:0x40', kept)
                self.assertIn('--argrange', kept)
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_write_mod_inserts_its_line_in_sorted_order(self):
        # Item 2379 (Loop 2003; parked 2318, 2375): `--write-mod` appended its
        # `mod` line at `mod.rs`'s tail, and two batches sorted the run by
        # hand. The line goes into the file's last run of `mod` lines at its
        # name's place, the helpers and the first run left as they stand.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            mod = Path(tmp) / 'mod.rs'
            head = '//! doc\n\nmod zz_first_run;\n\nfn helper() {}\n\n'
            mod.write_text(head + 'mod a_b;\nmod y_z;\n')
            try:
                self.assertEqual(sweep_gen.main(['const', 'X::y@00401000', '--item', '1', '--write-mod']), 0)
                self.assertEqual(mod.read_text(), head + 'mod a_b;\nmod x_y;\nmod y_z;\n')
                self.assertEqual(sweep_gen.main(['const', 'Z::z@00401010', '--item', '1', '--write-mod']), 0)
                self.assertEqual(mod.read_text(), head + 'mod a_b;\nmod x_y;\nmod y_z;\nmod z_z;\n')
                self.assertEqual(sweep_gen.main(['const', 'A::a@00401020', '--item', '1', '--write-mod']), 0)
                self.assertEqual(mod.read_text(), head + 'mod a_a;\nmod a_b;\nmod x_y;\nmod y_z;\nmod z_z;\n')
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_a_generated_sweep_keeps_its_own_line(self):
        # Item 2325 (parked 2311, Loop 2003): `--regen` wants the line that
        # drew a sweep, and a lane kept it in its job's tmp directory, gone
        # with the job. The script carries its line, and `--regen` with the
        # script alone redraws it from that line.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            (Path(tmp) / 'mod.rs').write_text('')
            try:
                line = ['obj', 'X::y@00401000', '--item', '1', '--win', '0x20', '--argrange', '0:0:2',
                        '--own', '1', '--port', 'let v = 1;\n0']
                self.assertEqual(sweep_gen.main(line + ['--write-mod']), 0)
                sp = Path(tmp) / 'sweep_x_y.py'
                drawn = sp.read_text()
                self.assertEqual(sweep_gen.kept_line(sp), line)
                sp.write_text('# a lane hand-edited this\n' + drawn.splitlines()[-1] + '\n')
                (Path(tmp) / 'x_y.rs').write_text('')
                self.assertEqual(sweep_gen.main(['--regen', str(sp)]), 0)
                self.assertEqual(sp.read_text(), drawn)
                self.assertIn('let v = 1;', (Path(tmp) / 'x_y.rs').read_text())
                self.assertEqual((Path(tmp) / 'mod.rs').read_text().count('mod x_y;'), 1)
                # The snake name finds the script too.
                self.assertEqual(sweep_gen.main(['--regen', 'x_y']), 0)
                # A whole line given with --regen replaces the kept one.
                self.assertEqual(sweep_gen.main(line + ['--bias', '0:1', '--regen']), 0)
                self.assertEqual(sweep_gen.kept_line(sp), line + ['--bias', '0:1'])
                # A script with no line kept is refused by name.
                sp.write_text('print(1)\n')
                self.assertEqual(sweep_gen.main(['--regen', str(sp)]), 2)
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_a_regen_keeps_the_hand_written_port(self):
        # Item 2368 (parked 2342, Loop 2003): `--regen` with no `--port`
        # rewrote the test's hand-written port block to `todo!()`, and
        # `AnimalData::get_speed` and `Herd::process` each restored it from a
        # copy. A regen without `--port` keeps the block between the
        # generated halves; one given `--port` still replaces it.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            (Path(tmp) / 'mod.rs').write_text('')
            try:
                line = ['obj', 'X::y@00401000', '--item', '1', '--win', '0x20', '--stub', '0x401100:0']
                self.assertEqual(sweep_gen.main(line), 0)
                tp = Path(tmp) / 'x_y.rs'
                hand = ('            let v = crate::x::y(m);\n'
                        '            if v > 0 {\n'
                        '                wr(m, 0, v);\n'
                        '            }\n'
                        '            i64::from(v)')
                tp.write_text(tp.read_text().replace('            todo!("the port of X::y")', hand))
                self.assertEqual(sweep_gen.main(line + ['--rows', '300', '--regen']), 0)
                text = tp.read_text()
                self.assertIn(hand, text)
                self.assertNotIn('todo!', text)
                self.assertIn('n >= ', text)
                self.assertEqual(text.count('let got: i64 = {'), 1)
                # The kept line redraws it too.
                self.assertEqual(sweep_gen.main(['--regen', 'x_y']), 0)
                self.assertIn(hand, tp.read_text())
                # A `--port` given replaces the block.
                self.assertEqual(sweep_gen.main(line + ['--port', '0', '--regen']), 0)
                self.assertNotIn(hand, tp.read_text())
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_a_seq_may_end_on_a_terminating_answer(self):
        # Item 2368 (parked 2342, Loop 2003): `--seq` repeats its last answer
        # forever, so a function that loops while a stub answers inside a set
        # never returns once the stubbed kills change nothing (`Unit::repath`,
        # hand-edited). `--seq IDX:N:T,T..` draws the last of the N answers
        # from the terminators, and answers it past N.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2368, win=0x20, bias=[], own=0,
            stubs=['0x401100:0'], answers=['0:1,2,3'], seq=['0:4:0,7'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('SEQ = [(0, 4, [0, 7])]', script)
        self.assertIn('stub 0 answers 4 values in call order, the last from [0, 7]', script)
        ns = {}
        start = script.index('def seq_values')
        exec(script[start:script.index('\n\n\n', start)], ns)
        import random
        rng = random.Random(1)
        for _ in range(200):
            vals = ns['seq_values'](rng, 4, [1, 2, 3], [0, 7], lambda r: 99)
            self.assertEqual(len(vals), 4)
            self.assertIn(vals[-1], (0, 7))
        # Without terminators the last is drawn as the rest.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2368, win=0x20, bias=[], own=0,
            stubs=['0x401100:0'], seq=['0:3'], port='0')
        self.assertIn('SEQ = [(0, 3, None)]', script)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=1, win=8, bias=[], own=0,
                                   stubs=['0x401100:0'], seq=['0:3:'])

    def test_a_record_order_may_carry_a_row_split(self):
        # Item 2421 (parked 2404, Loop 2003): `PathFinder::calc_road_cost`
        # logs in stub order and walks a sparse table; `--record-order` with
        # `--record IDX@S` writes each call of the split stub headed by its
        # row (`s1:r2+0x4,..`), `ecx` outside the table as a plain token.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2421, win=0x20, bias=[], own=0,
            sparse=['@0xe6e000:0x6eec:3:0,4'], stubs=['0x402000:1', '0x403000:0'],
            record=['0@0', '1'], record_order=True, ret='none', port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('RECORDORDER = True', script)
        ns = {'STACK_BASE': 0x7FF0_0000, 'STACK_SIZE': 0x10_0000}
        for fn in ('def token(', 'def in_order('):
            body = script[script.index(fn):]
            exec(body[:body.index('\n\n\ndef ')], ns)
        spans = [('m', 0x1000, 0x20)]
        names = ['a', 'b']
        calls = [('a', 0x100 + 2 * 0x10 + 4, (5,)), ('b', 0x1000, ()), ('a', 0x77, (1,))]
        self.assertEqual(ns['in_order'](calls, [0, 1], names, spans, {0: (0x100, 0x10, 3)}),
                         ['s0:r2+0x4,5', 's1:m+0x0', 's0:119,1'])
        # A split names a --sparse table that exists.
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj('Foo::bar@00401000', item=2421, win=0x20, bias=[], own=0,
                                   stubs=['0x402000:1'], record=['0@0'], record_order=True)

    def test_a_seq_may_draw_from_a_bounded_range(self):
        # Item 2421 (parked 2404): `--seq` answers from a list or a random
        # word, so an index-like answer (a `Random::get` stub) runs out of
        # its table. `--answer-range IDX:LO:HI` beside `--seq IDX:N` bounds
        # every one of the N answers.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2421, win=0x20, bias=[], own=0,
            stubs=['0x401100:0'], seq=['0:5'], answer_range=['0:2:6'], port='0')
        compile(script, str(sp), 'exec')
        ns = {}
        start = script.index('def seq_values')
        exec(script[start:script.index('\n\n\n', start)], ns)
        import random
        rng = random.Random(3)
        for _ in range(300):
            vals = ns['seq_values'](rng, 5, None, None, lambda r: 999, (2, 6))
            self.assertTrue(all(2 <= v <= 6 for v in vals), vals)
        # Without a range the unbounded draw stays.
        self.assertIn(999, [v for _ in range(50) for v in ns['seq_values'](rng, 5, None, None, lambda r: 999)])
        self.assertIn('ANSRANGE = [(0, 2, 6)]', script)

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

    def test_a_mix_step_names_the_slot_4_callees_stale_ecx(self):
        # parked 2498: Tribes::walk_data hands Array::walk_data the `ecx` the
        # walker's slot 4 left, `pushl %ecx` after the call.
        spec = 'Tribes::walk_data@006efb20'
        _, script, _, test, _ = sweep_gen.generate_mix(
            spec, item=2507, glob=0xc06378,
            steps=['walk4:0x21df4', 'call:0x47e230:2:this+4:p0,stale'])
        compile(script, 'sweep_tribes_walk_data.py', 'exec')
        self.assertIn('STALE = True', script)
        self.assertIn('stale', test)
        _, script0, _, _, _ = sweep_gen.generate_mix(spec, item=2507, glob=0xc06378,
                                                     steps=['walk4:0x21df4'])
        self.assertIn('STALE = False', script0)

    def test_a_stale_ecx_on_a_call_step_after_slot_0_is_the_walkers_and_the_callees(self):
        # parked 2563: PatrolOrder::walk_data does `pushl %ecx` after the slot-0
        # callback and again after a direct callee: both are leftovers the
        # hooked callee must leave (an answer `(eax, ecx)`), slot 0 included.
        spec = 'PatrolOrder::walk_data@00483e80'
        steps = ['walk:this-0x40+d:this-0x3f+d', 'walk:this-0x8:this-0x4',
                 'call:0x483cc0:2:this-0x40:p0,stale', 'call:0x483cc0:2:this-0x24:p0,stale']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2567, vbase=0x44, steps=steps)
        compile(script, 'sweep_patrol_order_walk_data.py', 'exec')
        self.assertIn('STALE = True', script)
        self.assertIn('h.answers["walk"] = (7, sv)', script)
        self.assertIn('h.answers[f"{a:x}"] = (0, sv)', script)
        self.assertIn('calls.push(vec![this - 0x24, walker, stale]);', test)
        _, script0, _, _, _ = sweep_gen.generate_mix(spec, item=2567, vbase=0x44, steps=steps[:2])
        self.assertIn('STALE = False', script0)

    def test_a_mix_may_gate_on_the_short_at_this(self):
        # parked 2536: Army::walk_data walks (this+2, this+0x98) only when the
        # short at `this` is non-zero; `this` must be a real, written object.
        spec = 'Army::walk_data@006f9850'
        steps = ['walk4:0xa78', 'walk:this:this+2', 'gated/walk:this+2:this+0x98']
        _, script, _, test, _ = sweep_gen.generate_mix(
            spec, item=2547, glob=0xc06378, gate_this=True, steps=steps)
        compile(script, 'sweep_army_walk_data.py', 'exec')
        self.assertIn('GATE_THIS = True', script)
        self.assertIn('if gate != 0 {', test)
        self.assertNotIn('if gate == 0 {', test)
        _, script0, _, test0, _ = sweep_gen.generate_mix(
            spec, item=2547, glob=0xc06378, gate=0x10, steps=steps)
        self.assertIn('GATE_THIS = False', script0)
        self.assertIn('if gate == 0 {', test0)

    def test_a_mix_may_read_a_displacement_through_this_minus_k(self):
        # parked 2536: GroupOrder::walk_data reads edx = [[this-0x1c]+4] and
        # walks (this+edx-0x18, this+edx-0x17) then (this-0x18, this-4).
        spec = 'GroupOrder::walk_data@00485580'
        steps = ['walk:this-0x18+d:this-0x17+d', 'walk:this-0x18:this-4']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2547, vbase=0x1c, steps=steps)
        compile(script, 'sweep_group_order_walk_data.py', 'exec')
        self.assertIn('VBASE = 0x1c', script)
        self.assertIn('this - 0x18 + g', test)
        self.assertIn('this - 0x4', test)
        _, script0, _, _, _ = sweep_gen.generate_mix(spec, item=2547, steps=['walk:this:this+2'])
        self.assertIn('VBASE = None', script0)

    def test_a_mix_may_gate_on_a_bit_at_another_offset(self):
        # parked 2548: Player::walk_data walks (this, this+0x39) and calls 0xa1b2d0 only
        # when `testb $1,[this+0x30]`; the gate column is the masked byte.
        spec = 'Player::walk_data@006ee2d0'
        steps = ['walk4:0x17a48', 'walk:this+0x30:this+0x32', 'gated/walk:this:this+0x39']
        _, script, _, test, _ = sweep_gen.generate_mix(
            spec, item=2562, glob=0xc06378, gate_this='0x30:1', steps=steps)
        compile(script, 'sweep_player_walk_data.py', 'exec')
        self.assertIn('GATE_AT = (0x30, 0x1)', script)
        self.assertIn('if gate != 0 {', test)
        _, script0, _, _, _ = sweep_gen.generate_mix(
            spec, item=2562, glob=0xc06378, gate_this=True, steps=steps)
        self.assertIn('GATE_AT = None', script0)

    def test_a_mix_may_read_a_walk_length_from_memory(self):
        # parked 2548: CommandPackage::walk_data walks (this+0x12, this+0x12+sext([this+0x10]))
        # when that short is non-zero; the gate column is the sign-extended short.
        spec = 'CommandPackage::walk_data@0094b740'
        steps = ['walk4:0x30ac', 'walk:this:this+0x12', 'gated/walk:this+0x12:this+0x12+len']
        _, script, _, test, _ = sweep_gen.generate_mix(
            spec, item=2562, glob=0xc06378, gate_this=True, len_at=0x10, steps=steps)
        compile(script, 'sweep_command_package_walk_data.py', 'exec')
        self.assertIn('LEN_AT = 0x10', script)
        self.assertIn('this + 0x12 + gate', test)
        self.assertIn('if gate != 0 {', test)

    def test_a_mix_may_gate_on_a_walker_word_not_equal_to_zero(self):
        # parked 2568: Rivers::walk_data calls 0x8878f0 and 0x886020 (ecx = this)
        # only when [walker+4] != 0, then 0x4a2f80 ungated.
        spec = 'Rivers::walk_data@008890e0'
        steps = ['gated/call:0x8878f0:0:this:', 'gated/call:0x886020:0:this:',
                 'call:0x4a2f80:2:0xc11c10:p0,stale']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2583, gate=4, gate_ne=True, steps=steps)
        compile(script, 'sweep_rivers_walk_data.py', 'exec')
        self.assertIn('GATE = 0x4', script)
        self.assertIn('if gate != 0 {', test)
        self.assertNotIn('if gate == 0 {', test)
        self.assertIn('let stale = g;', test)
        self.assertIn('h.answers["walk"] = (7, sv)', script)
        _, script0, _, test0, _ = sweep_gen.generate_mix(spec, item=2583, gate=4, steps=steps)
        self.assertIn('if gate == 0 {', test0)

    def test_a_mix_stale_may_be_a_column_of_its_own(self):
        # parked 2673: `stale` was `g`, so swapping them in a step passed.
        spec = 'ObjectType::walk_rules_data@0065fba0'
        steps = ['call:0x476610:2:this+0x27c:p0,stale']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2688, gate=8, steps=steps, stale_col=True)
        compile(script, 'sweep_object_type_walk_rules_data.py', 'exec')
        self.assertIn('STALE_COL = True', script)
        self.assertIn('let stale = a[4];', test)
        self.assertNotIn('let stale = g;', test)
        _, script0, _, test0, _ = sweep_gen.generate_mix(spec, item=2688, gate=8, steps=steps)
        self.assertIn('STALE_COL = False', script0)
        self.assertIn('let stale = g;', test0)

    def test_a_mix_may_read_a_second_vbase_displacement(self):
        # parked 2668, 2674: GroupMoveOrder::walk_data reads [[this-0x70]+4] and later [[this-0x20]+4].
        spec = 'GroupMoveOrder::walk_data@004855c0'
        steps = ['walk:this-0x6c+d:this-0x6b+d', 'walk:this-0x6c:this-0x20',
                 'walk:this-0x1c+d2:this-0x1b+d2', 'walk:this-0x1c:this-8', 'walk:this-8:this-4']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2688, vbase=0x70, vbase2=0x20, steps=steps)
        compile(script, 'sweep_group_move_order_walk_data.py', 'exec')
        self.assertIn('VBASE2 = 0x20', script)
        self.assertIn('let d2 = a[4];', test)
        self.assertIn('walker, this - 0x1c + d2, this - 0x1b + d2', test)
        self.assertIn('walker, this - 0x6c + g, this - 0x6b + g', test)
        _, script0, _, _, _ = sweep_gen.generate_mix(spec, item=2688, vbase=0x70, steps=steps[:2])
        self.assertIn('VBASE2 = None', script0)

    def test_a_mix_may_hold_a_second_gate_a_bit_of_this(self):
        # parked 2669, 2675: City::walk_data tests [this+4] & 1, then [walker+8] == 0.
        spec = 'City::walk_data@00489220'
        steps = ['walk:this+4:this+6', 'bit/walk:this+6:this+0x72', 'bit/gated/call:0xa1b2d0:1:this+0x90:p0',
                 'bit/gated/call:0xa1b2d0:1:this+0xa4:p0', 'bit/call:0x489040:2:this+0x74:p0,stale']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2688, gate=8, bit_at='4:1', stale_col=True, steps=steps)
        compile(script, 'sweep_city_walk_data.py', 'exec')
        self.assertIn('BIT_AT = (0x4, 0x1)', script)
        self.assertIn('let bit = a[5];', test)
        self.assertIn('if bit != 0 && gate == 0 {', test)
        self.assertIn('if bit != 0 { calls.push(vec![walker, this + 0x6, this + 0x72]); }', test)
        _, script0, _, _, _ = sweep_gen.generate_mix(spec, item=2688, gate=8, steps=steps[:1])
        self.assertIn('BIT_AT = None', script0)

    def test_a_mix_may_loop_a_count_read_from_this_over_records(self):
        # parked 2689: BuildQueue::walk_data closes/inits on [this] == 0 / != 0 when [walker+4] != 0,
        # then walks the [this]-many 0x14-byte records at [this+4].
        spec = 'BuildQueue::walk_data@006305f0'
        steps = ['walk4:6000', 'walk:this:this+4', 'gated/zero/call:0x6306c0:0:this:', 'gated/nz/call:0x6307d0:1:this:count',
                 'each/walk:elem:elem+0x12']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2698, glob=0xc06378, gate=4, gate_ne=True,
                                                       elems='count:0:4:20', steps=steps)
        compile(script, 'sweep_build_queue_walk_data.py', 'exec')
        self.assertIn("ELEMS = ('count', 0x0, 0x4, 20)", script)
        self.assertIn('let count = a[4];', test)
        self.assertIn('let elems = &a[5..];', test)
        self.assertIn('if gate != 0 && count == 0 {', test)
        self.assertIn('if gate != 0 && count != 0 {', test)
        self.assertIn('calls.push(vec![this, count]);', test)
        self.assertIn('for &e in elems { calls.push(vec![walker, e, e + 0x12]); }', test)

    def test_a_mix_may_assert_the_result_is_the_stack_argument(self):
        # parked 2570: a get_name returns its out-pointer ([ebp+8]), eax == walker.
        spec = 'Surf::get_name@004a5dc0'
        steps = ['call:0xa1d660:1:p0:11879128']
        _, _, _, test, _ = sweep_gen.generate_mix(spec, item=2583, steps=steps, ret_arg=True)
        self.assertIn('assert_eq!(eax, walker', test)
        _, _, _, test0, _ = sweep_gen.generate_mix(spec, item=2583, steps=steps)
        self.assertNotIn('assert_eq!(eax, walker', test0)

    def test_a_mix_token_may_name_the_record_the_global_cell_holds(self):
        # parked 2584: Constants::walk_data hands (rec, rec+0xd40) and (rec+0x804, rec+0x808), rec = *0xc061f0.
        spec = 'Constants::walk_data@0057f910'
        steps = ['walk:rec:rec+0xd40', 'walk:rec+0x804:rec+0x808']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2588, glob=0xc061f0, steps=steps)
        compile(script, 'sweep_constants_walk_data.py', 'exec')
        self.assertIn('REC = True', script)
        self.assertIn('vec![walker, g, g + 0xd40]', test)
        self.assertIn('vec![walker, g + 0x804, g + 0x808]', test)
        _, script0, _, _, _ = sweep_gen.generate_mix(spec, item=2588, glob=0xc061f0, steps=['walk:this:this+2'])
        self.assertIn('REC = False', script0)

    def test_a_mix_may_assert_a_store(self):
        # parked 2586: Dock::walk_data stores -1 (a word) at this+6 when [walker+8]==0 and [walker+4]!=0.
        spec = 'Dock::walk_data@00740c40'
        steps = ['walk:this:this+6', 'walk:this+8:this+0xa', 'gated/walk:this+6:this+8']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2588, gate=8, store='6:0xffff:2:4', steps=steps)
        compile(script, 'sweep_dock_walk_data.py', 'exec')
        self.assertIn('STORE = (0x6, 0xffff, 2, 0x4)', script)
        self.assertIn('after', test)
        self.assertIn('0xffff', test)
        _, script0, _, test0, _ = sweep_gen.generate_mix(spec, item=2588, gate=8, steps=steps)
        self.assertIn('STORE = None', script0)
        self.assertNotIn('after', test0)

    def test_a_mix_may_loop_over_a_table_read_through_a_global(self):
        # parked 2585: Types::walk_data hands (e+0x1e2, e+0x1e3) for each of the 85 words of *(*0xc061b0+0x10)[0x880..0x9d4).
        spec = 'Types::walk_data@00669780'
        steps = ['each/walk:elem+0x1e2:elem+0x1e3']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2593, glob=0xc061b0, elems='table:0x880:0x9d4:4', steps=steps)
        compile(script, 'sweep_types_walk_data.py', 'exec')
        self.assertIn("ELEMS = ('table', 0x880, 0x9d4, 4)", script)
        self.assertIn('for &e in elems', test)
        self.assertIn('vec![walker, e + 0x1e2, e + 0x1e3]', test)
        self.assertIn('assert_eq!(elems.len(), 85', test)
        _, script0, _, test0, _ = sweep_gen.generate_mix(spec, item=2593, glob=0xc061b0, steps=['walk:this:this+2'])
        self.assertIn('ELEMS = None', script0)
        self.assertNotIn('elems', test0)

    def test_a_mix_may_walk_a_linked_list_from_a_field(self):
        # parked 2589: TerrainList::walk_data walks [this+8] by [node+8], handing (node+0x1c, node+0x1e).
        spec = 'TerrainList::walk_data@0084b9e0'
        steps = ['each/walk:elem+0x1c:elem+0x1e']
        _, script, _, test, _ = sweep_gen.generate_mix(spec, item=2593, elems='list:8:8', steps=steps)
        compile(script, 'sweep_terrain_list_walk_data.py', 'exec')
        self.assertIn("ELEMS = ('list', 0x8, 0x8)", script)
        self.assertIn('vec![walker, e + 0x1c, e + 0x1e]', test)
        self.assertNotIn('assert_eq!(elems.len()', test)

    def test_a_mix_token_may_name_the_word_the_global_cells_record_holds(self):
        # parked 2593: OceanVars::get_name copies the string at *(*0xc06378+0x10)+0x163dc: the g column plus a constant.
        spec = 'OceanVars::get_name@0049f320'
        steps = ['call:0xa1d590:1:p0:g+0x163dc']
        _, _, _, test, _ = sweep_gen.generate_mix(spec, item=2593, glob=0xc06378, steps=steps, ret_arg=True)
        self.assertIn('vec![walker, g + 0x163dc]', test)

    def test_an_obj_may_answer_the_out_pointer_it_was_handed(self):
        # parked 2594: the inlined `String` constructors (FarmsOut::get_name and four kin) store a block into the
        # buffer at [esp+4] and answer that pointer: `--ret arg:0` holds eax to argument 0, and the buffer's bytes after are the block.
        spec = 'FarmsOut::get_name@004352b0'
        port = 'let o = &mut p[0]; wr(o, 0, 0xb2468c); o[8] = 5; 0'
        _, script, _, test, _ = sweep_gen.generate_obj(spec, item=2604, win=0x20, bias=[], own=1, ret='arg:0',
                                                        ptrargs=['0:0x14'], port=port)
        compile(script, 'sweep_farms_out_get_name.py', 'exec')
        self.assertIn('assert_eq!(want_eax, i64::from(a[0])', test)
        self.assertNotIn('let _ = (got, want_eax);', test)
        with self.assertRaises(ValueError):
            sweep_gen.generate_obj(spec, item=2604, win=0x20, bias=[], own=1, ret='arg:1', ptrargs=['0:0x14'])

    def test_a_jump_table_draws_its_index_inside_the_table(self):
        # parked 2607: Setup::get_starting_citizens is `jmp *0x5aafac(,%eax,4)` over four arms; `--jump IDX:N`
        # draws the index argument in 0..N-1 (a wild index jumps into the next table's bytes) and is an --own argument.
        x = sweep_gen.jump_expand('1:4', own=2)
        self.assertEqual(x['argrange'], ['1:0:3'])
        for bad in ('1:0', '2:4', '1', '1:4:9'):
            with self.assertRaises(ValueError):
                sweep_gen.jump_expand(bad, own=2)

    def test_a_table_is_rows_of_pointer_arrays_of_linked_records(self):
        # parked 2617 (item 2621): WorldData::get_bottom walks `[0xc0ab84 + 28*v]` (an array of record pointers, indexed by
        # the walk's x) to a record whose i16 at +0x2c is the next x and at +0x2e the next v. `--table
        # VA:STRIDE:ROWS:COLS:SIZE:XOFF:VOFF` lays ROWS pointer arrays at the image's own table, COLS records each, the
        # x link of each drawn below its own column (the walk ends) and the v word inside the rows.
        x = sweep_gen.table_expand('0xc0ab84:0x1c:2:3:0x34:0x2c:0x2e', pbase=1)
        self.assertEqual(x['place'], ['0xc0ab84+0x1c*0:0:12', 'p1+0:0:0x34', 'p1+4:0:0x34', 'p1+8:0:0x34',
                                      '0xc0ab84+0x1c*1:0:12', 'p5+0:0:0x34', 'p5+4:0:0x34', 'p5+8:0:0x34'])
        self.assertEqual(x['link'], ['p2..4+0x2c:2:<:-1,-2', 'p6..8+0x2c:2:<:-1,-2'])
        self.assertEqual(x['wbias'], ['p2..4+0x2e:0..1', 'p6..8+0x2e:0..1'])
        for bad in ('0xc0ab84:0x1c:0:3:0x34:0x2c:0x2e', '0xc0ab84:0x1c:2:3:0x34:0x2c', '0xc0ab84:0x1c:2:3:0x2e:0x2c:0x2e'):
            with self.assertRaises(ValueError):
                sweep_gen.table_expand(bad, pbase=1)

    def test_a_table_record_may_carry_its_row_in_a_byte_and_links_within_its_row(self):
        # parked 2713 (item 2719): UnitData::has_damage's two rows were written by hand -- each record's byte at +9 is
        # its row, its i16 at +0x8e a figure below its own index and at +0x90 above. `--table`'s VOFF `-` leaves the v
        # word alone, `--row-byte OFF[:LOW,..]` draws the word holding OFF as row<<shift | LOW, `--row-link` is `--link`
        # over each row's records.
        x = sweep_gen.table_expand('0xc0aec0:0x1c:2:4:0xa0:0x8e:-', pbase=1, row_byte='9:0,1,1,3',
                                   row_links=['0x90:2:>:-1'])
        self.assertEqual(x['wbias'], ['p2..5+0x8:0x0,0x1,0x1,0x3', 'p7..10+0x8:0x100,0x101,0x101,0x103'])
        self.assertEqual(x['link'], ['p2..5+0x8e:2:<:-1,-2', 'p2..5+0x90:2:>:-1',
                                     'p7..10+0x8e:2:<:-1,-2', 'p7..10+0x90:2:>:-1'])
        with self.assertRaises(ValueError):
            sweep_gen.table_expand('0xc0aec0:0x1c:2:4:0xa0:0x8e:-', pbase=1, row_byte='0xa0')
        with self.assertRaises(ValueError):
            sweep_gen.table_expand('0xc0aec0:0x1c:2:4:0xa0:0x8e:-', pbase=1, row_links=['0x90:3:>'])

    def test_a_pointer_table_is_one_place_for_the_array_and_one_a_record(self):
        # parked 2772 (item 2782): an object list's N records were N `--place` flags and N `--vtat-also`s written by a shell
        # loop. `--ptrtab LOC:N:SIZE[:vt]` lays the array of N record pointers at LOC (a place of 4N bytes) and each record
        # as a SIZE-byte place of the array's window, `vt` laying the stubbed vtable at each record's offset 0 as well.
        x = sweep_gen.ptrtab_expand('0xc0ab84+m[0x4a]*0x1c:3:0x20', pbase=1)
        self.assertEqual(x['place'], ['0xc0ab84+m[0x4a]*0x1c:0:12', 'p1+0:0:0x20', 'p1+4:0:0x20', 'p1+8:0:0x20'])
        self.assertEqual(x['vtat_also'], [])
        y = sweep_gen.ptrtab_expand('c0+8:2:0x20:vt', pbase=4)
        self.assertEqual(y['place'], ['c0+8:0:8', 'p4+0:0:0x20', 'p4+4:0:0x20'])
        self.assertEqual(y['vtat_also'], ['p5+0', 'p6+0'])
        for bad in ('0xc0ab84:0:0x20', '0xc0ab84:3', '0xc0ab84:3:0x20:xx', '0xc0ab84:3:0'):
            with self.assertRaises(ValueError):
                sweep_gen.ptrtab_expand(bad, pbase=1)

    def test_gate_this_does_not_swallow_a_following_positional(self):
        # parked 2569: a bare `--gate-this` before the function name is the flag, not its value.
        self.assertEqual(sweep_gen.split_gate_this(['mix', '--gate-this', 'Foo::bar@00401000']),
                         ['mix', '--gate-bare', 'Foo::bar@00401000'])
        self.assertEqual(sweep_gen.split_gate_this(['mix', '--gate-this', '0x30:1', 'F@1']),
                         ['mix', '--gate-this', '0x30:1', 'F@1'])
        self.assertEqual(sweep_gen.split_gate_this(['mix', 'F@1', '--gate-this']),
                         ['mix', 'F@1', '--gate-bare'])




class FamilyRegistrations(unittest.TestCase):
    def test_new_independent_sweeps_route_to_stable_partitions(self):
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for family in sweep_gen.SWEEP_FAMILIES:
                (root / f'standalone_{family}.rs').write_text(sweep_gen.REGISTRY_MARKER)
            (root / 'mod.rs').write_text('mod existing;\n')
            with patch.object(sweep_gen, 'SWEEP', root):
                for name, family in [('apple', 'a_f'), ('group', 'g_o'), ('path', 'p_t'), ('world', 'u_z')]:
                    self.assertEqual(sweep_gen.sweep_registry(f'mod {name};', 'let got = 1;'), root / f'standalone_{family}.rs')
                self.assertEqual(sweep_gen.sweep_registry('mod private;', 'crate::internal()'), root / 'mod.rs')
                self.assertEqual(sweep_gen.sweep_registry('mod existing;', 'let got = 1;'), root / 'mod.rs')

    def test_generator_writes_new_partition_registration_and_keeps_regeneration_owner(self):
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'mod.rs').write_text('')
            for family in sweep_gen.SWEEP_FAMILIES:
                (root / f'standalone_{family}.rs').write_text(sweep_gen.REGISTRY_MARKER)
            with patch.object(sweep_gen, 'SWEEP', root), patch.object(sweep_gen, 'EMU', root):
                command = ['const', 'World::probe@00401000', '--item', '1', '--port', '7']
                self.assertEqual(sweep_gen.main(command + ['--write-mod']), 0)
                registry = root / 'standalone_u_z.rs'
                registered = registry.read_text()
                self.assertIn('mod world_probe;', registered)
                self.assertEqual((root / 'mod.rs').read_text(), '')
                self.assertEqual(sweep_gen.main(command + ['--regen']), 0)
                self.assertEqual(registry.read_text(), registered)

    def test_generator_refuses_bad_registry_before_creating_files(self):
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'mod.rs').write_text('')
            for family in sweep_gen.SWEEP_FAMILIES:
                (root / f'standalone_{family}.rs').write_text(sweep_gen.REGISTRY_MARKER)
            (root / 'standalone_u_z.rs').write_text(sweep_gen.REGISTRY_MARKER + 'unexpected_content();\n')
            with patch.object(sweep_gen, 'SWEEP', root), patch.object(sweep_gen, 'EMU', root):
                self.assertEqual(sweep_gen.main(['const', 'World::probe@00401000', '--item', '1', '--write-mod']), 2)
                self.assertFalse((root / 'world_probe.rs').exists())
                self.assertFalse((root / 'sweep_world_probe.py').exists())

    def test_registry_insertion_preserves_header_and_rejects_loss(self):
        header = '// helper declarations stay intact\n' + sweep_gen.REGISTRY_MARKER
        text = sweep_gen.insert_standalone(header, 'mod z;')
        text = sweep_gen.insert_standalone(text, 'mod a;')
        self.assertTrue(text.startswith(header))
        self.assertLess(text.index('mod a;'), text.index('mod z;'))
        with self.assertRaises(ValueError):
            sweep_gen.insert_standalone(text, 'mod a;')
        with self.assertRaises(ValueError):
            sweep_gen.insert_standalone(text + 'fn unexpected() {}\n', 'mod b;')

    def test_each_real_sweep_is_registered_once_and_targets_are_mounted(self):
        sweep_gen.validate_sweep_registrations(ROOT / 'crates/sim')

    def test_missing_duplicate_and_wrong_partition_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            folder = root / 'src/sweep'
            folder.mkdir(parents=True)
            (root / 'tests').mkdir()
            (folder / 'mod.rs').write_text('mod private;\n')
            (folder / 'private.rs').write_text('crate::private();')
            (folder / 'apple.rs').write_text('#[test] fn check() {}')
            for family in sweep_gen.SWEEP_FAMILIES:
                (folder / f'standalone_{family}.rs').write_text(sweep_gen.REGISTRY_MARKER)
                (root / 'tests' / f'sweep_{family}.rs').write_text(f'#[path = "../src/sweep/standalone_{family}.rs"]\nmod sweep;\n')
            with self.assertRaises(ValueError):
                sweep_gen.validate_sweep_registrations(root)
            path = folder / 'standalone_a_f.rs'
            path.write_text(sweep_gen.insert_standalone(path.read_text(), 'mod apple;'))
            sweep_gen.validate_sweep_registrations(root)
            # A crate:: path may name a public API. Preserve its body and let
            # Rust enforce visibility through the integration target bridge.
            (folder / 'apple.rs').write_text('crate::public_api();')
            with self.assertRaisesRegex(ValueError, 'public API bridge missing'):
                sweep_gen.validate_sweep_registrations(root)
            entry = root / 'tests/sweep_a_f.rs'
            entry.write_text('pub use sim::*;\n' + entry.read_text())
            sweep_gen.validate_sweep_registrations(root)
            (folder / 'apple.rs').write_text('#[test] fn check() {}')
            manifest = root / 'Cargo.toml'
            manifest.write_text('[package]\nautotests = false\n')
            with self.assertRaises(ValueError):
                sweep_gen.validate_sweep_registrations(root)
            manifest.write_text('[[test]]\nname = "sweep_a_f"\ntest = false\n')
            with self.assertRaises(ValueError):
                sweep_gen.validate_sweep_registrations(root)
            manifest.write_text('')
            (folder / 'mod.rs').write_text('mod private;\nmod apple;\n')
            with self.assertRaises(ValueError):
                sweep_gen.validate_sweep_registrations(root)
            (folder / 'mod.rs').write_text('mod private;\n')
            (folder / 'standalone_u_z.rs').write_text(path.read_text())
            path.write_text(sweep_gen.REGISTRY_MARKER)
            with self.assertRaises(ValueError):
                sweep_gen.validate_sweep_registrations(root)

    def test_a_void_honours_the_obj_windows_and_stubs(self):
        # Item 2887 (parked 2872): `void` ignored `--cell-win` and `--stub`
        # (`Player::end_game` used `obj --ret none`). A void is `obj` with no
        # answer; every window and stub the line names reaches the script.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            (Path(tmp) / 'mod.rs').write_text('mod a_b;\n')
            try:
                line = ['void', 'Foo::bar@00401000', '--item', '2887', '--own', '1',
                        '--cell-win', '0xc061d0:0x80', '--stub', '0x00401234:1',
                        '--place', 'm+0x10:0:0x20', '--place-null', '0']
                import contextlib, io
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    self.assertEqual(sweep_gen.main(line + ['--dry-run']), 0)
                # main hands `ret='eax'`; a void answers nothing whatever it is handed.
                self.assertIn('an empty body', out.getvalue())
                self.assertIn('let _ = (got, want_eax);', out.getvalue())
                self.assertIn('CELLS = [(12607952, 128)]', out.getvalue())
                sp, script, tp, test, mod = sweep_gen.generate_void(
                    'Foo::bar@00401000', item=2887, own=1, cells=['0xc061d0:0x80'],
                    stubs=['0x00401234:1'], places=['m+0x10:0:0x20'], place_null=['0'])
                compile(script, str(sp), 'exec')
                self.assertIn('CELLS = [(12607952, 128)]', script)
                self.assertIn("STUBS = [('call', 4198964, 1)]", script)
                self.assertIn('NULLS = [0]', script)
                self.assertIn('an empty body', test)
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_a_new_sweep_is_registered_without_the_flag(self):
        # Item 2887 (parked 2872): `--write-mod` was added by hand to
        # `TerrainOil::remove_rocky_patch`'s line. A fresh generation
        # registers; `--no-write-mod` and `--dry-run` do not, `--regen`
        # leaves `mod.rs` alone.
        with tempfile.TemporaryDirectory() as tmp:
            sweep_gen.EMU = Path(tmp)
            sweep_gen.SWEEP = Path(tmp)
            mod = Path(tmp) / 'mod.rs'
            mod.write_text('mod a_b;\n')
            try:
                self.assertEqual(sweep_gen.main(['const', 'X::y@00401000', '--item', '1', '--dry-run']), 0)
                self.assertEqual(mod.read_text(), 'mod a_b;\n')
                self.assertEqual(sweep_gen.main(['const', 'X::y@00401000', '--item', '1', '--no-write-mod']), 0)
                self.assertEqual(mod.read_text(), 'mod a_b;\n')
                (Path(tmp) / 'x_y.rs').unlink()
                (Path(tmp) / 'sweep_x_y.py').unlink()
                self.assertEqual(sweep_gen.main(['const', 'X::y@00401000', '--item', '1']), 0)
                self.assertEqual(mod.read_text(), 'mod a_b;\nmod x_y;\n')
                self.assertEqual(sweep_gen.main(['const', 'X::y@00401000', '--item', '1', '--regen']), 0)
                self.assertEqual(mod.read_text(), 'mod a_b;\nmod x_y;\n')
            finally:
                sweep_gen.EMU = ROOT / 'tools/emu'
                sweep_gen.SWEEP = ROOT / 'crates/sim/src/sweep'

    def test_a_place_null_past_the_first_place_nulls_that_place_alone(self):
        # Item 2887 (parked 2885): `--place-null K` for K past 0 lays place K
        # (its order on the line) null, and no other.
        sp, script, tp, test, mod = sweep_gen.generate_obj(
            'Foo::bar@00401000', item=2887, win=0xe0, bias=[],
            places=['m+0xd8:0:8', 'm+0xdc:0:8', 'p1+4:0:0x10'], place_null=['1'], port='0')
        compile(script, str(sp), 'exec')
        self.assertIn('NULLS = [1]', script)
        self.assertIn('the word at `m+0xdc` points at 8 random bytes less `0` (placed, null a third of the rows)', test + script)
        self.assertNotIn('the word at `m+0xd8` points at 8 random bytes less `0` (placed, null', test + script)


if __name__ == '__main__':
    unittest.main()
