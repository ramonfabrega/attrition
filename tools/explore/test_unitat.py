"""`tools/gamelog/unitat.py`: one unit out of a dump, by name, in list order
(parked 1981, 1739 and 1601 (d), the thirty-first pass: four lanes wrote the
same scratch parsers)."""
import io
import sys
import tempfile
import unittest
from unittest.mock import patch
from contextlib import redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/gamelog'))
import unitat  # noqa: E402

UNIT = """  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 65
     o 27
     who 1
     x_internal 6542
     y_internal 19797
     z_internal 34
    damage 0
    inside_down -1
    up -1
    down -1
    uid 6
    myhits {hits}
    near_o -1
    near_who -1
   collide_frame 311
   orders_x 4248
   orders_y 3480
   group 90
   inside_up {inside_up}
   idle 0
   stance 1
   BEGIN STACK<TYPE>
    size 50
    length {plen}
    increment 10
    BEGIN PATHDATA
     to_x 4224
     to_y 3456
     tolerance 0
     flags 1
    BEGIN PATHDATA
     to_x 4248
     to_y 5016
     tolerance 0
     flags 4
{extra}   length 2
   type 1
   metric 0
   BEGIN MOVEORDER
    BEGIN UNITORDER
     flags 1
    x 4248
    y 3480
    dest 1
    tolerance 0
    dest_x 6552
    dest_y 18840
   BEGIN ATTACKORDER
    BEGIN UNITORDER
     flags 0
    target_o 12
    target_who 0
   length 2
   size 2
   increment 1
   BEGIN GUY
    type 77
    x 6542
    y 19797
    who 1
    o 27
"""

OTHER = """  BEGIN UNITDATA
   BEGIN OBJECT
    BEGIN SUBOBJECT
     flags 65
     o 28
     who 1
     x_internal 1
     y_internal 2
    myhits 5
   BEGIN GUY
    type 80
"""


LAST = """    BEGIN PATHDATA
     to_x 6552
     to_y 18840
     tolerance 384
     flags 0
"""


def dump(path):
    frames = [(3131, UNIT.format(hits=130, inside_up=-1, plen=3, extra=LAST) + OTHER),
              (3132, UNIT.format(hits=130, inside_up=-1, plen=3, extra=LAST)),
              (3133, UNIT.format(hits=120, inside_up=2001, plen=2, extra=''))]
    text = ' MAP_STYLE 18\n' + ''.join(f' BEGIN FRAME {n}\n{body}' for n, body in frames) + ' BEGIN FRAME 3134\n GameInfo closing\n'
    path.write_text(text)


GROUP_MOVE = """   length 1
   type 19
   metric 0
   BEGIN GroupMoveOrder
    BEGIN UNITORDER
     flags 1
    x 100
    y 200
"""


class OneUnit(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.path = Path(self.tmp.name) / 'gamelog.txt'
        dump(self.path)
        enums = Path(self.tmp.name) / 'enums'
        enums.mkdir()
        (enums / 'TypeIndex.txt').write_text('enum /rise.pdb/TypeIndex  // 2 values\n  77\t0x4d\tBMP\n  80\t0x50\tFREIGHTER\n')
        self.enums = enums

    def tearDown(self):
        self.tmp.cleanup()

    def run_tool(self, *argv):
        out = io.StringIO()
        with redirect_stdout(out):
            rc = unitat.main([str(self.path), *argv, '--enums', str(self.enums)])
        return rc, out.getvalue()

    def test_a_group_move_order_is_an_order(self):
        # Loop 2145: the dump spells a group move `BEGIN GroupMoveOrder`,
        # mixed case, and the printer kept the children ending in `ORDER`;
        # run827's block 662 printed `0/6` as `orders 1 … FOLLOWORDER` over a
        # `length 2` that held a type-19 GroupMoveOrder, and 2131 read it as
        # "the original queued nothing".
        self.path.write_text(' MAP_STYLE 18\n BEGIN FRAME 662\n'
                             + UNIT.format(hits=130, inside_up=-1, plen=3, extra=LAST).replace(
                                 '   BEGIN GUY', GROUP_MOVE + '   BEGIN GUY')
                             + ' BEGIN FRAME 663\n GameInfo closing\n')
        rc, out = self.run_tool('662', '1/27')
        self.assertEqual(rc, 0, out)
        self.assertIn('orders 3', out)
        self.assertIn('[2] GroupMoveOrder', out)
        self.assertIn('x=100', out)

    def test_one_frame_names_the_type_the_path_s_ends_and_the_orders_bottom_first(self):
        rc, out = self.run_tool('3131', '1/27')
        self.assertEqual(rc, 0, out)
        self.assertIn('f3131 1/27', out)
        self.assertIn('type 77 BMP', out)
        self.assertIn('hits 130', out)
        self.assertIn('inside_up -1', out)
        self.assertIn('path 3:', out)
        self.assertIn('[0] (4224,3456) f1', out)
        self.assertIn('[2] (6552,18840) f0', out)
        # The dump prints the ring from its anchor, the bottom, and the
        # current order last (`docs/AI.md` §206; Loop 2059: the printer once
        # reversed them and marked the bottom `<top>`, which cost 2055 a
        # misreading). The printer keeps the dump's order and marks the
        # current order.
        self.assertLess(out.index('[0] MOVEORDER'), out.index('[1] ATTACKORDER'))
        self.assertIn('[1] ATTACKORDER', out)
        self.assertIn('dest_x=6552', out)
        self.assertIn('<current>', out)
        self.assertNotIn('<top>', out)
        self.assertIn('bottom first', out)
        self.assertNotIn('1/28', out)

    def test_a_window_prints_the_first_frame_whole_and_then_what_changed(self):
        rc, out = self.run_tool('3131-3133', '1/27')
        self.assertEqual(rc, 0, out)
        self.assertEqual(out.count('f3131 1/27'), 1)
        self.assertNotIn('f3132', out)
        self.assertIn('f3133', out)
        self.assertIn('hits 120', out)
        self.assertIn('inside_up 2001', out)
        self.assertIn('path 2:', out)
        # The window's later frames carry only what changed, never the header again.
        self.assertNotIn('f3133 1/27 uid', out)

    def test_several_units_are_read_in_one_pass_over_the_dump(self):
        # Loop 2896 (2881's line, the thirty-ninth pass): about 6 s a unit on
        # run880's 1.2 GB dump, over 100 s for a cast of eighteen, each a
        # scan of its own. Several units, comma-separated, share one pass;
        # an absent one is named and the exit is 2 only when all are.
        passes = []
        real = unitat.units

        def counted(path, lo, hi):
            passes.append(path)
            return real(path, lo, hi)

        with patch.object(unitat, 'units', counted):
            rc, out = self.run_tool('3131-3133', '1/27,1/28,2/27')
        self.assertEqual(rc, 0, out)
        self.assertEqual(len(passes), 1, 'one pass over the dump for every unit asked')
        self.assertIn('f3131 1/27', out)
        self.assertIn('f3131 1/28', out)
        self.assertIn('2/27 absent on frames 3131-3133', out)
        self.assertIn('f3133 1/27 changed:', out)

    def test_an_absent_unit_exits_non_zero(self):
        rc, out = self.run_tool('3131', '2/27')
        self.assertNotEqual(rc, 0)
        self.assertIn('absent', out)

    def test_all_prints_every_path_entry(self):
        rc, out = self.run_tool('3131', '1/27', '--all')
        self.assertIn('[1] (4248,5016) f4', out)


if __name__ == '__main__':
    unittest.main()
