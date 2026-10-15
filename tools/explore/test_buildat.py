"""`tools/gamelog/buildat.py`: a building's queue across frames (Loop 2459,
three reaches — 2412's scratch script, 2491's and 2526's hand reads of
`BUILDQUEUE`'s counters across every trainer of a player; Loop 2407: the
dump prints every slot and the ones past `queued` keep their last entry).
"""
import io
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/gamelog'))
import buildat  # noqa: E402

SLOT = """    queue[scan].type {ty}
    queue[scan].job_counter {jc}
    queue[scan].cost[0] 0
    queue[scan].cost[1] 0
    queue[scan].cost[2] 0
    queue[scan].good[0] 0
    queue[scan].good[1] 0
    queue[scan].good[2] 0
"""

BUILDING = """  BEGIN BUILDDATA
   BEGIN WALLDATA
    BEGIN OBJECT
     BEGIN SUBOBJECT
      flags 7
      o {o}
      who {who}
      x_internal 6240
      y_internal 7008
     damage 0
     uid 3
     myhits 1560
    hold_frames 0
    job_counter {head}
   queued {queued}
   orig_type {ty}
   BEGIN BUILDQUEUE
    queue_size 3
{slots}"""


def building(who, o, ty, queued, head, slots):
    text = ''.join(SLOT.format(ty=t, jc=j) for t, j in slots)
    return BUILDING.format(who=who, o=o, ty=ty, queued=queued, head=head, slots=text)


def dump(path, crlf=False):
    # 1/2002 trains a 579 whose counter falls, with a stale 77 in the slot
    # past `queued`; 1/2003's queue is empty and stays so; 0/2000 is another
    # player's.
    frames = [
        (3296, building(1, 2002, 435, 1, 329200, [(579, 329200), (77, 5), (0, 0)])
         + building(1, 2003, 414, 0, 0, [(-1, 0), (0, 0), (0, 0)])
         + building(0, 2000, 414, 1, 10, [(100, 10), (0, 0), (0, 0)])),
        (3297, building(1, 2002, 435, 1, 329100, [(579, 329100), (77, 5), (0, 0)])
         + building(1, 2003, 414, 0, 0, [(-1, 0), (0, 0), (0, 0)])),
        (3298, building(1, 2002, 435, 1, 329100, [(579, 329100), (77, 5), (0, 0)])
         + building(1, 2003, 414, 0, 0, [(-1, 0), (0, 0), (0, 0)])),
        (3299, building(1, 2002, 435, 0, 0, [(579, 329100), (77, 5), (0, 0)])
         + building(1, 2003, 414, 0, 0, [(-1, 0), (0, 0), (0, 0)])),
    ]
    text = ' MAP_STYLE 18\n' + ''.join(f' BEGIN FRAME {n}\n{body}' for n, body in frames) \
        + ' BEGIN FRAME 3300\n GameInfo closing\n'
    if crlf:
        text = text.replace('\n', '\r\n')
    path.write_text(text)


class Queues(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.path = Path(self.tmp.name) / 'gamelog.txt'
        dump(self.path)
        enums = Path(self.tmp.name) / 'enums'
        enums.mkdir()
        (enums / 'TypeIndex.txt').write_text(
            'enum /rise.pdb/TypeIndex  // 3 values\n  435\t0x1b3\tBARRACKS\n  579\t0x243\tMUSKETEER\n  414\t0x19e\tCITY\n')
        self.enums = enums

    def tearDown(self):
        self.tmp.cleanup()

    def run_tool(self, *argv):
        out = io.StringIO()
        with redirect_stdout(out):
            rc = buildat.main([str(self.path), *argv, '--enums', str(self.enums)])
        return rc, out.getvalue()

    def test_one_building_prints_on_change_and_cuts_at_queued(self):
        rc, out = self.run_tool('3296-3299', '1/2002')
        self.assertEqual(rc, 0, out)
        lines = out.splitlines()
        # The first frame whole, then only the frames the queue changed on:
        # 3297 (the counter), 3299 (emptied); 3298 is silent.
        self.assertEqual([l.split()[0] for l in lines], ['f3296', 'f3297', 'f3299'])
        self.assertIn('1/2002 435 BARRACKS queued 1 [579 MUSKETEER:329200]', lines[0])
        self.assertNotIn('77', out, 'the slot past `queued` is a stale entry (Loop 2407)')
        self.assertIn('queued 0 []', lines[2])

    def test_a_player_s_trainers_are_every_building_with_a_queue(self):
        rc, out = self.run_tool('3296-3297', '1')
        self.assertEqual(rc, 0, out)
        self.assertIn('1/2002', out)
        self.assertNotIn('1/2003', out, 'an empty queue that stays empty is not a trainer')
        self.assertNotIn('0/2000', out)

    def test_a_building_absent_on_every_frame_exits_2(self):
        rc, out = self.run_tool('3296-3299', '1/9999')
        self.assertEqual(rc, 2)

    def test_a_crlf_dump_reads_the_same(self):
        dump(self.path, crlf=True)
        rc, out = self.run_tool('3296', '1/2002')
        self.assertEqual(rc, 0, out)
        self.assertIn('[579 MUSKETEER:329200]', out)


if __name__ == '__main__':
    unittest.main()
