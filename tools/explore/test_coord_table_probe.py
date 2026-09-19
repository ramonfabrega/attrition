#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Install-dependent negative controls; no executable bytes are stored here.

Usage: uv run tools/explore/test_coord_table_probe.py /path/to/game
"""
from pathlib import Path
import sys
import unittest
from dataclasses import replace
from unittest.mock import patch
import coord_table_probe as probe

IMAGE = None


class TableTest(unittest.TestCase):
    def test_original_initializer(self):
        report, _ = probe.initialize(IMAGE, 1)
        self.assertEqual(report['entries_checked'], 48)

    def test_missing_allocation_import(self):
        real = probe.BoundedCall

        def missing(regions, *args, **kwargs):
            return real([r for r in regions if r.name != 'malloc_import'], *args, **kwargs)

        with patch.object(probe, 'BoundedCall', side_effect=missing):
            with self.assertRaisesRegex(ValueError, 'READ_UNMAPPED'):
                probe.initialize(IMAGE, 1)

    def test_undersized_allocation(self):
        real = probe.BoundedCall

        def smaller(regions, *args, **kwargs):
            regions = [replace(r, data=r.data[:-4]) if r.name == 'allocation' else r for r in regions]
            return real(regions, *args, **kwargs)

        with patch.object(probe, 'BoundedCall', side_effect=smaller):
            with self.assertRaisesRegex(ValueError, 'undeclared access'):
                probe.initialize(IMAGE, 1)

    def test_corrupted_outputs_and_missing_write_are_rejected(self):
        real = probe.BoundedCall.run
        for mutation, message in [('negative', 'floor'), ('last', 'floor'),
                                  ('center', 'pointers'), ('unwritten', 'unwritten')]:
            with self.subTest(mutation=mutation):
                def altered(runner, entry, *args, **kwargs):
                    regs, outputs, writes = real(runner, entry, *args, **kwargs)
                    if entry == probe.RESUME:
                        outputs = dict(outputs)
                        if mutation == 'unwritten':
                            runner.initialized.discard(probe.BUFFER+100)
                        elif mutation == 'center':
                            outputs['center'] = probe.word(probe.BUFFER)
                        else:
                            data = bytearray(outputs['allocation'])
                            data[0 if mutation == 'negative' else -1] ^= 1
                            outputs['allocation'] = bytes(data)
                        outputs = tuple(outputs.items())
                    return regs, outputs, writes

                with patch.object(probe.BoundedCall, 'run', altered):
                    with self.assertRaisesRegex(ValueError, message):
                        probe.initialize(IMAGE, 1)


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    IMAGE = Path(sys.argv.pop())/'riseofnations.exe'
    if not IMAGE.is_file():
        raise SystemExit('Owned executable is required; missing is not a passing test')
    unittest.main()
