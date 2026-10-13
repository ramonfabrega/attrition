"""The census reads the traces the blind list pins (parked 938, 939, 942).

Its recipe named three traces for nine days while eighty-six more stood
beside them, so the entered column was a sample (7,180 of 7,666, item 923);
and the recipe that replaced it, a `$(for …)` over run names, splits on the
spaces in this machine's archive path and has never run as written (item
935). `--pin` reads `rondata::blind::TRACES` from the source and joins each
name to the archive directory itself, so the recipe is one flag and the
list is the pin's.
"""
import importlib.util
import os
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('census', ROOT / 'tools/census.py')
census = importlib.util.module_from_spec(spec)
spec.loader.exec_module(census)


class PinnedTraces(unittest.TestCase):
    def test_the_list_is_the_pin_s_in_the_pin_s_order(self):
        names = census.pinned_traces()
        source = (ROOT / 'crates/rondata/src/blind.rs').read_text()
        body = source[source.index('pub const TRACES'):]
        body = body[:body.index('\n];')]
        self.assertEqual(names, re.findall(r'"(rontrace-[^"]+\.log)"', body))
        self.assertEqual(names[0], 'rontrace-run29.log')
        self.assertGreaterEqual(len(names), 46)
        self.assertEqual(len(names), len(set(names)))

    def test_a_comment_that_names_a_trace_is_not_a_trace(self):
        text = ('pub const TRACES: &[&str] = &[\n'
                '    "rontrace-run29.log",\n'
                '    // "rontrace-run53.log" was the recipe\'s, and enters nothing run29 does not\n'
                '    "rontrace-run16.log", // ch1\n'
                '];\n')
        with tempfile.TemporaryDirectory() as tmp:
            pin = Path(tmp) / 'blind.rs'
            pin.write_text(text)
            self.assertEqual(census.pinned_traces(pin),
                             ['rontrace-run29.log', 'rontrace-run16.log'])

    def test_an_archive_path_with_spaces_is_one_path_a_trace(self):
        with tempfile.TemporaryDirectory() as tmp:
            archive = Path(tmp) / 'Microsoft Games' / 'Rise of Nations' / 'Logs'
            archive.mkdir(parents=True)
            for name in ('rontrace-run29.log', 'rontrace-run16.log'):
                (archive / name).touch()
            with patch.object(census, 'pinned_traces',
                              return_value=['rontrace-run29.log', 'rontrace-run16.log']):
                logs = census.pinned_logs(str(archive))
            self.assertEqual(logs, [str(archive / 'rontrace-run29.log'),
                                    str(archive / 'rontrace-run16.log')])

    def test_a_pinned_trace_that_is_not_on_disk_is_named(self):
        with tempfile.TemporaryDirectory() as tmp, \
                patch.object(census, 'pinned_traces', return_value=['rontrace-run29.log']), \
                self.assertRaises(SystemExit) as stop:
            census.pinned_logs(tmp)
        self.assertIn('rontrace-run29.log', str(stop.exception))

    def test_an_operator_s_citation_is_counted(self):
        # Parked 1722: the pattern could not spell `operator<` or
        # `operator[]`, so 1708 cited `tcoord::operator@00472b30` as a
        # workaround and every operator was uncountable as backed.
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, 'X.md').write_text(
                '`TCoord::operator<@00472b30` and `Objects::operator[]@00512340` '
                'and `Unit::operator=@00612340`, and `plain@00401000`.\n')
            self.assertEqual(census.cited_in(tmp), {0x472b30, 0x512340, 0x612340, 0x401000})

    def test_the_unbacked_list_skips_the_dead_weight_by_default(self):
        # Parked 1861, the thirtieth pass: destructors, `close`/`init`/
        # `clear`, loggers and `dynamic_initializer`s fill the shortest-first
        # list and every sweep lane skipped them by hand, ~300 bytes deep.
        entries = [(0x10, 'Army::~Army', 'c:\\rise\\game\\army.cpp', 93),
                   (0x20, 'Units::init', 'c:\\rise\\game\\units.cpp', 94),
                   (0x30, 'Guy::close', 'c:\\rise\\game\\guy.cpp', 90),
                   (0x40, 'Lands::clear', 'c:\\rise\\game\\lands.cpp', 110),
                   (0x50, 'Storm::registration', 'c:\\rise\\game\\storm.cpp', 110),
                   (0x60, 'LeaderData::log_data', 'c:\\rise\\game\\leader.cpp', 120),
                   (0x70, "`dynamic initializer for 'x''", 'c:\\rise\\game\\x.cpp', 50),
                   (0x80, 'Cliff::close_class', 'c:\\rise\\game\\cliff.cpp', 98),
                   (0x90, 'World::adjust_fog', 'c:\\rise\\game\\world.cpp', 106),
                   (0xa0, 'BorderWData::BorderWData', 'c:\\rise\\game\\borders.cpp', 140),
                   (0xb0, 'TerrainOil::one_time_init', 'c:\\rise\\game\\terrainoil.cpp', 127)]
        with patch.object(census, 'layer_of', return_value='simulation'):
            rows = census.rank_unbacked(entries, set(), set())
        marks = {name: mark for _, _, name, mark, _ in rows}
        for name in ('Army::~Army', 'Units::init', 'Guy::close', 'Lands::clear', 'Storm::registration',
                     'LeaderData::log_data', "`dynamic initializer for 'x''", 'Cliff::close_class',
                     'BorderWData::BorderWData', 'TerrainOil::one_time_init'):
            self.assertEqual(marks[name], 'skip', name)
        self.assertEqual(marks['World::adjust_fog'], '-')
        self.assertEqual([r[2] for r in census.worth_sweeping(rows)], ['World::adjust_fog'])

    def test_the_pdb_default_resolves_the_main_checkout_from_a_worktree(self):
        # Parked 1792: `../game` is absent under `.claude/worktrees/`, so
        # every lane passed `--pdb <main checkout>/game/sbl/rise.pdb` by hand.
        path = census.default_pdb(str(ROOT / 'tools'))
        self.assertTrue(path.endswith(os.path.join('game', 'sbl', 'rise.pdb')), path)
        self.assertNotIn('.claude/worktrees', path.replace(str(ROOT), '') if '.claude/worktrees' in str(ROOT) else '')
        with tempfile.TemporaryDirectory() as tmp:
            Path(tmp, 'game', 'sbl').mkdir(parents=True)
            Path(tmp, 'game', 'sbl', 'rise.pdb').write_text('')
            self.assertEqual(census.default_pdb(os.path.join(tmp, 'tools')),
                             os.path.join(tmp, 'tools', '..', 'game', 'sbl', 'rise.pdb'))

    def test_the_census_document_gives_the_flag_as_its_recipe(self):
        text = (ROOT / 'docs/CENSUS.md').read_text()
        self.assertIn('tools/census.py --top 30 --never --pin', text)
        self.assertNotIn('$(for f in', text,
                         'the loop splits on the archive path\'s spaces (parked 942)')


if __name__ == '__main__':
    unittest.main()
