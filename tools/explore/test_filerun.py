"""`tools/gamelog/filerun.py`: a windowed capture's dump and trace filed
under the harness's names (Loop 2933, the thirty-ninth pass: a hand step
in every journal of the AI word)."""
import io
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools/gamelog'))
import filerun  # noqa: E402


class FileRun(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        base = Path(self.tmp.name)
        self.golden, self.logs = base / 'golden', base / 'Logs'
        self.logs.mkdir()
        d = self.golden / 'run893c' / 'map-18'
        d.mkdir(parents=True)
        (d / 'gamelog.txt').write_text(' MAP_STYLE 18\n BEGIN FRAME 6579\n')
        (d / 'rontrace.log').write_bytes(b'RONT' + bytes(28))

    def tearDown(self):
        self.tmp.cleanup()

    def test_the_two_files_land_under_the_pinned_names_and_are_never_overwritten(self):
        out = io.StringIO()
        pairs = filerun.file_run(self.golden, self.logs, 'run893c', 'eastindies-german-diff3-window-6579-6837', out=out)
        dump = self.logs / 'gamelog-run893c-eastindies-german-diff3-window-6579-6837.txt'
        trace = self.logs / 'rontrace-run893c.log'
        self.assertEqual([dst for _, dst in pairs], [dump, trace])
        self.assertEqual(dump.read_text(), ' MAP_STYLE 18\n BEGIN FRAME 6579\n')
        self.assertEqual(trace.read_bytes()[:4], b'RONT')
        self.assertIn(str(dump), out.getvalue())
        with self.assertRaises(filerun.Refused):
            filerun.file_run(self.golden, self.logs, 'run893c', 'eastindies-german-diff3-window-6579-6837', out=out)
        filerun.file_run(self.golden, self.logs, 'run893c', 'eastindies-german-diff3-window-6579-6837',
                         force=True, out=out)
        self.assertEqual(len(list(self.logs.glob('rontrace-run893c.log.replaced-*'))), 1)

    def test_a_filed_capture_is_one_file_under_two_names(self):
        # Item 4600a, the pass of 2026-11-23, second (DECISIONS 155 §6): a
        # copy held every filed capture twice on one volume (1,785 twins,
        # 19.2 GiB on disk). A hard link is one file, and the archive's
        # compression keeps the link (afsctool, measured the same day).
        out = io.StringIO()
        pairs = filerun.file_run(self.golden, self.logs, 'run893c', 'tag', out=out)
        for src, dst in pairs:
            self.assertEqual(src.stat().st_ino, dst.stat().st_ino, f'{dst.name} is a second copy')
        self.assertIn('linked', out.getvalue())

    def test_a_link_that_fails_is_a_copy_and_says_so(self):
        out = io.StringIO()

        def no_link(src, dst):
            raise OSError(18, 'Cross-device link')

        pairs = filerun.file_run(self.golden, self.logs, 'run893c', 'tag', out=out, link=no_link)
        for src, dst in pairs:
            self.assertNotEqual(src.stat().st_ino, dst.stat().st_ino)
            self.assertEqual(src.read_bytes(), dst.read_bytes())
        self.assertIn('copied', out.getvalue())

    def test_a_run_with_two_maps_wants_map_and_a_missing_run_refuses(self):
        (self.golden / 'run893c' / 'map-14').mkdir()
        with self.assertRaises(filerun.Refused):
            filerun.map_dir(self.golden, 'run893c')
        self.assertEqual(filerun.map_dir(self.golden, 'run893c', 18).name, 'map-18')
        with self.assertRaises(filerun.Refused):
            filerun.map_dir(self.golden, 'run999')
        self.assertEqual(filerun.main(['run999', 'x', '--golden', str(self.golden), '--logs', str(self.logs)]), 2)

    def test_a_playback_s_dump_is_its_gamelog_p(self):
        # Item 3428: a recorded game's playback writes `gamelog_p.txt`.
        d = self.golden / 'run1036' / 'map-18'
        d.mkdir(parents=True)
        (d / 'gamelog_p.txt').write_text(' MAP_STYLE 18\n')
        (d / 'rontrace.log').write_bytes(b'RONT')
        pairs = filerun.plan(self.golden, self.logs, 'run1036', 'playback')
        self.assertEqual(pairs[0][0].name, 'gamelog_p.txt')
        (d / 'gamelog.txt').write_text(' MAP_STYLE 18\n')
        self.assertEqual(filerun.plan(self.golden, self.logs, 'run1036', 'playback')[0][0].name, 'gamelog.txt')

    def test_a_dry_run_writes_nothing(self):
        out = io.StringIO()
        filerun.file_run(self.golden, self.logs, 'run893c', 't', dry_run=True, out=out)
        self.assertEqual(list(self.logs.iterdir()), [])
        self.assertIn('-> ', out.getvalue())


if __name__ == '__main__':
    unittest.main()
