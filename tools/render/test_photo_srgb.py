#!/usr/bin/env python3
"""Tests for photo_srgb.py (item 4223): `python3 -m unittest -q test_photo_srgb`
from `tools/render`. `sips` is stubbed; no photograph is read."""
import tempfile
import unittest
import zlib
from pathlib import Path

import photo_srgb


def chunk(kind, body):
    return len(body).to_bytes(4, 'big') + kind + body + b'\0\0\0\0'


def png(tag):
    out = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', bytes(13)) + tag
    return out + chunk(b'IDAT', b'') + chunk(b'IEND', b'')


def icc(name):
    text = name.encode('utf-16-be')
    mluc = b'mluc' + bytes(4) + (1).to_bytes(4, 'big') + (12).to_bytes(4, 'big')
    mluc += b'enUS' + len(text).to_bytes(4, 'big') + (28).to_bytes(4, 'big') + text
    head = bytearray(128)
    table = (1).to_bytes(4, 'big') + b'desc' + (144).to_bytes(4, 'big') + len(mluc).to_bytes(4, 'big')
    return bytes(head) + table + mluc


SRGB_PNG = png(chunk(b'sRGB', b'\0'))
MI_PNG = png(chunk(b'iCCP', b'ICC Profile\0\0' + zlib.compress(icc('Mi monitor'))))


class PhotoSrgb(unittest.TestCase):
    def test_profile_names(self):
        with tempfile.TemporaryDirectory() as d:
            a, b, c = Path(d, 'a.png'), Path(d, 'b.png'), Path(d, 'c.png')
            a.write_bytes(SRGB_PNG)
            b.write_bytes(MI_PNG)
            c.write_bytes(png(b''))
            self.assertEqual(photo_srgb.profile_name(a), photo_srgb.SRGB)
            self.assertEqual(photo_srgb.profile_name(b), 'Mi monitor')
            self.assertIsNone(photo_srgb.profile_name(c))

    def test_srgb_is_left_alone(self):
        with tempfile.TemporaryDirectory() as d:
            a = Path(d, 'a.png')
            a.write_bytes(SRGB_PNG)
            calls = []
            self.assertEqual(photo_srgb.to_srgb(a, runner=lambda *x, **k: calls.append(x)),
                             (photo_srgb.SRGB, None))
            self.assertEqual(calls, [])

    def test_another_profile_is_matched_beside_it(self):
        with tempfile.TemporaryDirectory() as d:
            b = Path(d, 'run-3294.png')
            b.write_bytes(MI_PNG)
            seen = []

            def sips(argv, **_):
                seen.append(argv)
                Path(argv[-1]).write_bytes(SRGB_PNG)

            name, out = photo_srgb.to_srgb(b, runner=sips)
            self.assertEqual((name, out.name), ('Mi monitor', 'run-3294-srgb.png'))
            self.assertEqual(seen[0][:2], ['sips', '-m'])
            self.assertEqual(b.read_bytes(), MI_PNG, 'the grab is kept')
            with self.assertRaises(SystemExit):
                photo_srgb.to_srgb(b, runner=sips)  # never overwrites

    def test_a_match_that_stays_untagged_refuses(self):
        with tempfile.TemporaryDirectory() as d:
            b = Path(d, 'b.png')
            b.write_bytes(MI_PNG)
            with self.assertRaises(SystemExit):
                photo_srgb.to_srgb(b, runner=lambda argv, **_: Path(argv[-1]).write_bytes(MI_PNG))


if __name__ == '__main__':
    unittest.main()
