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

    def test_the_census_document_gives_the_flag_as_its_recipe(self):
        text = (ROOT / 'docs/CENSUS.md').read_text()
        self.assertIn('tools/census.py --top 30 --never --pin', text)
        self.assertNotIn('$(for f in', text,
                         'the loop splits on the archive path\'s spaces (parked 942)')


if __name__ == '__main__':
    unittest.main()
