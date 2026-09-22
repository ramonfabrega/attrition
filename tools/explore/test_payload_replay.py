"""Authored packet fragments: exact byte authority and arena selection."""
import io
from pathlib import Path
import struct
import unittest
from unittest.mock import patch
from bounded_call import Region, BoundedCall
from payload_replay import CapturedPages, declare_borrowed, mutable_callee_arguments, observe_unit_path, fingerprint, subtract_regions, free_arena, import_target, native_return, explore, original_code, ARENA_SIZE, GDT


class PayloadReplayTest(unittest.TestCase):
    def test_exact_span_and_short_read(self):
        s = [dict(base='0x1000', bytes=8, file_offset=3, inventory_index=0)]
        p = CapturedPages(io.BytesIO(b'---abcdefgh'), s, [(0, 0, 0, 0, 0, 4, 0)])
        self.assertEqual(p.read(0x1002, 3), b'cde')
        self.assertIsNone(p.read(0xfff, 1))
        self.assertIsNone(p.read(0x1007, 2))
        p.stream = io.BytesIO(b'---abcd')
        with self.assertRaisesRegex(ValueError, 'short captured'):
            p.read(0x1004, 4)

    def test_page_addition_subtracts_existing_and_respects_protection(self):
        span = dict(base='0x1000', bytes=4096, file_offset=0, inventory_index=0)
        for protection in (2, 4, 8):
            p = CapturedPages(io.BytesIO(bytes(range(256))*16), [span], [(0, 0, 0, 0, 0, protection, 0)])
            old = [Region('old', 0x1010, bytes(16))]
            added = p.expand(0x1100, 4, old)
            self.assertEqual([(r.address, len(r.data)) for r in added], [(0x1000, 16), (0x1020, 4064)])
            self.assertEqual(added[1].data[:4], bytes(range(32, 36)))
            self.assertEqual([r.writable for r in added], [protection in (4, 8)]*2)
            self.assertTrue(all(not r.executable for r in added))
            self.assertEqual(p.expand(0x2000, 1, old), [])

    def test_multiple_overlaps_and_complete_coverage(self):
        regions = [Region('a', 20, bytes(10)), Region('b', 40, bytes(10))]
        self.assertEqual(subtract_regions(15, 55, regions), [(15, 20), (30, 40), (50, 55)])
        self.assertEqual(subtract_regions(21, 25, regions), [])
        self.assertEqual(subtract_regions(30, 40, regions), [(30, 40)])

    def test_free_arena_requires_whole_free_extent_and_no_overlap(self):
        start = 0x18000000
        rows = [(start, 0, 0, ARENA_SIZE, 0x10000, 1, 0)]
        self.assertEqual(free_arena(rows, []), start)
        for regions, address in (([Region('occupied', start+32, bytes(1))], start),
                                 ([], start+16), ([], start+1)):
            with self.assertRaisesRegex(ValueError, 'captured-free arena'):
                free_arena(rows, regions, address)
        with self.assertRaises(ValueError):
            free_arena([(start, 0, 0, ARENA_SIZE, 0x1000, 4, 0)], [])
        with self.assertRaises(ValueError):
            free_arena([(GDT, 0, 0, ARENA_SIZE, 0x10000, 1, 0)], [], GDT)

    def test_import_target_requires_captured_slot_and_executable_image(self):
        span = dict(base='0x1000', bytes=4, file_offset=0, inventory_index=0)
        for state, protection, kind, success in ((0x1000, 0x20, 0x1000000, True),
                (0x1000, 4, 0x1000000, False), (0x1000, 0x120, 0x1000000, False), (0x10000, 0x20, 0x1000000, False),
                (0x1000, 0x20, 0x20000, False)):
            pages = CapturedPages(io.BytesIO(struct.pack('<I', 0x3000)), [span],
                                  [(0x3000, 0x3000, protection, 4096, state, protection, kind)])
            if success:
                self.assertEqual(import_target(pages, 0x1000), 0x3000)
            else:
                with self.assertRaises(ValueError):
                    import_target(pages, 0x1000)
            with self.assertRaises(ValueError):
                import_target(pages, 0x1001)

    def test_native_return_matches_outer_call_after_receipt(self):
        rows = [(8, 0, 5), (5, 167), (7, 0), (7, 0), (8, 0, 2), (8, 0, 1)]
        with patch('payload_replay.records', return_value=iter(rows)):
            self.assertEqual(native_return(Path('/unused')), 1)
        with patch('payload_replay.records', return_value=iter([(5, 167), (7, 0)])):
            with self.assertRaises(ValueError):
                native_return(Path('/unused'))

    def test_borrowed_objects_require_explicit_captured_writable_bytes(self):
        span = dict(base='0x1000', bytes=8, file_offset=0, inventory_index=0)
        p = CapturedPages(io.BytesIO(b'abcdefgh'), [span], [(0, 0, 0, 0, 0, 4, 0)])
        objects = declare_borrowed(p, [], [(0x1000, 4)])
        self.assertEqual(objects, [Region('borrowed_1000', 0x1000, b'abcd', writable=True)])
        self.assertEqual(declare_borrowed(p, objects, [(0x1000, 4)]), [])
        for regions, requests in ((objects, [(0x1001, 4)]), ([], [(0x1000, 4), (0x1002, 4)]),
                                  ([], [(0x1000, 9)]), ([], [(0x1000, 0)]),
                                  ([], [(0x1000, -1), (0x1000, 1024*1024+1)])):
            with self.assertRaises(ValueError):
                declare_borrowed(p, regions, requests)
        p.rows = [(0, 0, 0, 0, 0, 2, 0)]
        with self.assertRaisesRegex(ValueError, 'read-only'):
            declare_borrowed(p, [], [(0x1000, 4)])

    def test_borrowing_cannot_silently_enable_a_service(self):
        with self.assertRaisesRegex(ValueError, 'explicit free policy'):
            explore(Path('/absent'), Path('/absent'), borrowed=[(0x1000, 4)])

    def test_six_callee_words_are_writable_and_reset(self):
        entry, esp, stop = 0x100000, 0x300000, 0x400000
        code = bytes.fromhex('c7442404 11000000 c7442418 22000000 c21800')
        layout = mutable_callee_arguments([Region('code', entry, code, executable=True),
                  Region('arguments', esp, struct.pack('<13I', stop, *range(12)))])
        r = BoundedCall(layout, stop)
        registers = (1, 2, 3, esp, 4, 5, 6, 7, 0x202)
        first = r.run(entry, registers)
        self.assertEqual(first[0][3], esp+28)
        self.assertEqual(struct.unpack('<6I', dict(first[1])['callee_arguments']), (17, 1, 2, 3, 4, 34))
        self.assertEqual(dict(first[1])['caller_tail'], struct.pack('<6I', *range(6, 12)))
        self.assertEqual(first, r.run(entry, registers))

    def test_argument_mutability_does_not_authorize_return_or_caller_tail(self):
        for offset in (0, 28):
            layout = mutable_callee_arguments([
                Region('code', 0x100000, b'\xc7\x44\x24'+bytes([offset])+struct.pack('<I', 1)+b'\xc3', executable=True),
                Region('arguments', 0x300000, struct.pack('<13I', 0x400000, *range(12)))])
            with self.assertRaisesRegex(ValueError, 'read-only write'):
                BoundedCall(layout, 0x400000).run(0x100000, (1, 2, 3, 0x300000, 4, 5, 6, 7, 0x202))

    def test_post_state_observer_keeps_every_slot_and_refuses_missing_bytes(self):
        def make(capacity=2, length=1, extent=32, scratch=False):
            unit = bytearray(0x158)
            struct.pack_into('<3I', unit, 0xb8, 0x210000, capacity, length)
            r = BoundedCall([Region('code', 0x100000, b'\xc3', executable=True),
                Region('unit', 0x200000, bytes(unit)),
                Region('path', 0x210000, bytes(range(extent)), writable=True, scratch=scratch),
                Region('return', 0x300000, struct.pack('<I', 0x400000))], 0x400000)
            r.run(0x100000, (1, 2, 3, 0x300000, 4, 5, 6, 7, 0x202))
            return r
        result = observe_unit_path(make(), 0x200000)
        self.assertEqual(result['path_slot_bytes'], bytes(range(32)).hex())
        self.assertEqual((result['capacity'], result['length']), (2, 1))
        self.assertFalse(result['native_compared'])
        for kwargs in ({'extent': 16}, {'scratch': True}, {'length': 3}, {'capacity': 4097}):
            with self.assertRaises(ValueError):
                observe_unit_path(make(**kwargs), 0x200000)
        self.assertEqual(observe_unit_path(make(capacity=0, length=0), 0x200000)['path_slot_bytes'], '')

    def test_fingerprint_includes_unwritten_declared_memory(self):
        r = BoundedCall([Region('code', 0x100000, b'\xc3', executable=True),
                        Region('untouched', 0x200000, b'abcd'),
                        Region('return', 0x300000, struct.pack('<I', 0x400000))], 0x400000)
        r.run(0x100000, (1, 2, 3, 0x300000, 4, 5, 6, 7, 0x202))
        before = fingerprint(r, None)
        r.uc.mem_write(0x200000, b'WXYZ')
        after = fingerprint(r, None)
        self.assertNotEqual(before['declared_memory_sha256'], after['declared_memory_sha256'])
        self.assertEqual(before['writes_sha256'], after['writes_sha256'])

    def test_limits_and_image_identity_refuse_before_loading(self):
        for options in ({'budget': 1000001}, {'rounds': 0}, {'services': 'all'}, {'repeat_final': 17}):
            with self.assertRaises(ValueError):
                explore(Path('/absent'), Path('/absent'), **options)
        with self.assertRaisesRegex(ValueError, 'unsupported original'):
            original_code(bytes(512))

if __name__ == '__main__':
    unittest.main()
