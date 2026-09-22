"""Copy service source authority, initialization, byte bounds and cdecl fixtures."""
import struct
import unittest
from bounded_call import Region
from modeled_memory_copy import MemoryCopyCall

ENTRY, MALLOC, FILL, COPY = 0x100000, 0x500000, 0x501000, 0x502000
SRC, OUT, ESP, STOP, ARENA = 0x200000, 0x200100, 0x300100, 0x400000, 0x600000
REGS = (1, 2, 3, ESP, 4, 5, 6, 7, 0x202)

def word(n):
    return struct.pack('<I', n)

def call_copy(destination=OUT, source=SRC, count=4, offset=0):
    return (b'\x68'+word(count)+b'\x68'+word(source)+b'\x68'+word(destination)+
            b'\xe8'+struct.pack('<i', COPY-(ENTRY+offset+20))+b'\x83\xc4\x0c')

def runner(code, *, source=None, output=None, **kwargs):
    return MemoryCopyCall([
        Region('code', ENTRY, code, executable=True),
        source or Region('source', SRC, b'abcdefgh'),
        output or Region('output', OUT, bytes(8), writable=True, scratch=True),
        Region('return', ESP, word(STOP)),
        Region('stack', ESP-256, bytes(256), writable=True, scratch=True),
        Region('fs', 0x70000000, word(0xffffffff), writable=True)], STOP,
        malloc_target=MALLOC, memset_target=FILL, memcpy_target=COPY,
        arena_address=ARENA, arena_size=64, fs_address=0x70000000,
        gdt_address=0x71000000, **kwargs)


class MemoryCopyTest(unittest.TestCase):
    def test_copy_cdecl_and_byte_stores(self):
        r = runner(call_copy()+b'\xc3'); result = r.run(ENTRY, REGS)
        self.assertEqual(dict(result[1])['output'], b'abcd'+bytes(4))
        self.assertEqual(result[0][7], OUT)
        self.assertEqual(result[0][3], ESP+4)
        for i in (0, 1, 2, 4):
            self.assertEqual(result[0][i], REGS[i])
        self.assertEqual([w for w in r.writes if OUT <= w[0] < OUT+8],
                         [(OUT+i, 1, value) for i, value in enumerate(b'abcd')])
        self.assertEqual(r.copies, [dict(destination=OUT, source=SRC, count=4,
                                        return_address=ENTRY+20)])

    def test_uninitialized_source_refuses(self):
        r = runner(call_copy()+b'\xc3', source=Region('source', SRC, bytes(8), writable=True, scratch=True))
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            r.run(ENTRY, REGS)

    def test_source_and_destination_bounds(self):
        for destination, source in ((OUT, SRC+6), (OUT+6, SRC), (ARENA, SRC), (OUT, ARENA)):
            with self.assertRaisesRegex(ValueError, 'undeclared access|outside modeled allocation'):
                runner(call_copy(destination, source)+b'\xc3').run(ENTRY, REGS)

    def test_readonly_destination_refuses(self):
        with self.assertRaisesRegex(ValueError, 'read-only write|WRITE_PROT'):
            runner(call_copy()+b'\xc3', output=Region('output', OUT, bytes(8))).run(ENTRY, REGS)

    def test_overlap_overflow_and_zero_count(self):
        for destination, source, count, reason in ((SRC+1, SRC, 4, 'overlapping'),
                (SRC, SRC, 4, 'overlapping'), (OUT, 0xfffffffe, 4, 'overflow')):
            with self.assertRaisesRegex(ValueError, reason):
                runner(call_copy(destination, source, count)+b'\xc3').run(ENTRY, REGS)
        r = runner(call_copy(0, 0, 0)+b'\xc3')
        self.assertEqual(r.run(ENTRY, REGS)[0][7], 0)

    def test_caps_and_direction(self):
        for code, opts, regs, reason in (
            (call_copy(count=5), {'max_copy_bytes': 4}, REGS, 'byte cap'),
            (call_copy()+call_copy(offset=23), {'max_copies': 1}, REGS, 'call cap'),
            (call_copy(), {}, (*REGS[:-1], REGS[-1] | 0x400), 'direction flag')):
            with self.assertRaisesRegex(ValueError, reason):
                runner(code+b'\xc3', **opts).run(ENTRY, regs)

    def test_reset_and_unwritten_tail(self):
        code = call_copy()+b'\xc3'; r = runner(code)
        first = r.run(ENTRY, REGS)
        self.assertEqual(first, r.run(ENTRY, REGS))
        self.assertEqual(first, runner(code).run(ENTRY, REGS))
        self.assertEqual(len(r.copies), 1)
        r = runner(code+b'\xa1'+word(OUT+4)+b'\xc3')
        r.run(ENTRY, REGS)
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            r.run(ENTRY+len(code), REGS)
        self.assertEqual(r.copies, [])

    def test_partial_failure_reset_and_mid_service_entry(self):
        code = call_copy(OUT+6)+b'\xc3'; r = runner(code+b'\xa1'+word(OUT+4)+b'\xc3')
        with self.assertRaisesRegex(ValueError, 'undeclared access'):
            r.run(ENTRY, REGS)
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            r.run(ENTRY+len(code), REGS)
        for target, reason in ((COPY, 'undeclared access'), (COPY+1, 'entry inside')):
            with self.assertRaisesRegex(ValueError, reason):
                runner(b'\xe9'+struct.pack('<i', target-ENTRY-5)).run(ENTRY, REGS)

if __name__ == '__main__':
    unittest.main()
