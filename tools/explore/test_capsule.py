#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Strict replay guards with authored machine-code fixtures; no game files."""
import importlib.util
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace
import unittest

spec = importlib.util.spec_from_file_location('capsule', Path(__file__).with_name('replay_capsule.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


def fixture():
    obj, seed, stack, stop = 0x10000100, 0x20000010, 0x30000100, 0x40000000
    # Authored fixture: load seed; store zero length; store seed; return.
    code = b'\xa1' + struct.pack('<I', seed) + b'\x66\xc7\x41\x10\0\0\x89\x81\x14\x02\0\0\xc3'
    before = bytearray(b'\x5a' * 0x218)
    after = bytearray(before)
    struct.pack_into('<H', after, 16, 0)
    struct.pack_into('<I', after, 0x214, 42)
    regs = (1, 2, 3, stack, 4, 5, obj, 6, 0x202)
    expected = list(regs); expected[3] += 4; expected[7] = 42
    return SimpleNamespace(entry=m.ENTRY, stop=stop, self_=obj, seed_before=42,
        before_regs=regs, after_regs=tuple(expected), regions=[
            ('code', m.ENTRY, code, code), ('package', obj, bytes(before), bytes(after)),
            ('game_seed', seed, struct.pack('<I', 42), struct.pack('<I', 42)),
            ('return', stack, struct.pack('<I', stop), struct.pack('<I', stop))])


class CapsuleTest(unittest.TestCase):
    def test_fresh_instances_match_registers_memory_and_writes(self):
        c = fixture()
        a, b = m.Replay(c).run(), m.Replay(c).run()
        self.assertEqual(a, b)
        self.assertEqual(a['instructions'], 4)
        self.assertEqual(len(a['writes']), 2)

    def test_missing_dependency_fails(self):
        with self.assertRaises(ValueError):
            m.Replay(fixture(), omit='game_seed').run()

    def test_same_mapped_page_does_not_authorize_adjacent_reads(self):
        c = fixture()
        name, addr, code, _ = c.regions[0]
        code = b'\xa1' + struct.pack('<I', c.regions[2][1]+4) + code[5:]
        c.regions[0] = (name, addr, code, code)
        with self.assertRaisesRegex(ValueError, 'uncaptured read'):
            m.Replay(c).run()

    def test_writes_require_explicit_permission(self):
        with self.assertRaises(ValueError):
            m.Replay(fixture(), writable=False).run()

    def test_unchanged_bytes_are_compared_too(self):
        c = fixture()
        name, addr, before, after = c.regions[1]
        after = bytearray(after); after[100] ^= 1
        c.regions[1] = (name, addr, before, bytes(after))
        with self.assertRaisesRegex(ValueError, 'live output mismatch'):
            m.Replay(c).run()

    def test_register_mismatch_fails(self):
        c = fixture(); regs = list(c.after_regs); regs[0] ^= 1; c.after_regs = tuple(regs)
        with self.assertRaisesRegex(ValueError, 'EDI'):
            m.Replay(c).run()

    def test_unbounded_execution_fails(self):
        c = fixture(); c.regions[0] = ('code', c.entry, b'\xeb\xfe', b'\xeb\xfe')
        with self.assertRaisesRegex(ValueError, 'budget exhausted'):
            m.Replay(c).run()

    def test_data_cannot_be_executed(self):
        c = fixture(); c.regions[0] = ('code', c.entry, b'\xff\xe1', b'\xff\xe1')
        with self.assertRaises(ValueError):
            m.Replay(c).run()

    def test_uncaptured_code_on_an_executable_page_fails(self):
        c = fixture()
        code = b'\xe9' + struct.pack('<i', 35)  # jump to entry + 40, same page
        c.regions[0] = ('code', c.entry, code, code)
        with self.assertRaisesRegex(ValueError, 'uncaptured instruction/import'):
            m.Replay(c).run()

    def test_changed_executable_identity_fails_before_replay(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); install = root / 'install'; output = root / 'output'
            install.mkdir(); output.mkdir()
            (install / 'riseofnations.exe').write_bytes(b'authored fixture')
            for name in ['riseofnations.exe', 'riseofnations_trace.exe', 'rontrace.dll']:
                (output / name).write_bytes(b'authored fixture')
            m.bind(install, output)
            (install / 'riseofnations.exe').write_bytes(b'changed fixture')
            with self.assertRaisesRegex(ValueError, 'image identity mismatch: source'):
                m.replay(install, output)

    def test_empty_capture_is_not_a_pass(self):
        with self.assertRaisesRegex(ValueError, 'truncated'):
            m.Capsule(b'')


if __name__ == '__main__':
    unittest.main()
