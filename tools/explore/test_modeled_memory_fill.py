"""Authored x86 fill calls, initialization, refusal, ABI and reset checks."""
import struct
import unittest
from bounded_call import Region
from modeled_memory_fill import MemoryFillCall

ENTRY, MALLOC, FILL, ARENA = 0x100000, 0x500000, 0x501000, 0x600000
OUT, ESP, STOP = 0x200000, 0x300100, 0x400000
REGS = (1, 2, 3, ESP, 4, 5, 6, 7, 0x202)

def word(n):
    return struct.pack('<I', n)

def call_fill(destination=OUT, value=0x1234, count=4, offset=0):
    # cdecl arguments in reverse order; caller owns cleanup.
    return (b'\x68'+word(count)+b'\x68'+word(value)+b'\x68'+word(destination)+
            b'\xe8'+struct.pack('<i', FILL-(ENTRY+offset+20))+b'\x83\xc4\x0c')

def runner(code, *, output=None, **kwargs):
    return MemoryFillCall([
        Region('code', ENTRY, code, executable=True),
        output or Region('output', OUT, bytes(8), writable=True, scratch=True),
        Region('return', ESP, word(STOP)),
        Region('stack', ESP-256, bytes(256), writable=True, scratch=True),
        Region('fs', 0x70000000, word(0xffffffff), writable=True)], STOP,
        malloc_target=MALLOC, memset_target=FILL, arena_address=ARENA, arena_size=64,
        fs_address=0x70000000, gdt_address=0x71000000, **kwargs)


class MemoryFillTest(unittest.TestCase):
    def test_fill_contract_and_cdecl(self):
        r = runner(call_fill()+b'\xc3')
        result = r.run(ENTRY, REGS)
        self.assertEqual(dict(result[1])['output'], b'\x34'*4+bytes(4))
        self.assertEqual(result[0][7], OUT)
        self.assertEqual(result[0][3], ESP+4)
        for i in (0, 1, 2, 4):  # EDI, ESI, EBP, EBX
            self.assertEqual(result[0][i], REGS[i])
        self.assertEqual([w for w in r.writes if OUT <= w[0] < OUT+8],
                         [(OUT+i, 1, 0x34) for i in range(4)])
        self.assertEqual(r.fills, [dict(destination=OUT, value=0x34, count=4,
                                       return_address=ENTRY+20)])
        self.assertFalse(r.fill_active)

    def test_heap_fill_initializes_only_written_bytes(self):
        # push 8; call malloc; add esp,4; then memset the known model address.
        alloc = b'\x6a\x08\xe8'+struct.pack('<i', MALLOC-ENTRY-7)+b'\x83\xc4\x04'
        for offset, succeeds in ((0, True), (4, False)):
            code = alloc+call_fill(ARENA, count=4, offset=len(alloc))+b'\xa1'+word(ARENA+offset)+b'\xc3'
            r = runner(code)
            if succeeds:
                self.assertEqual(r.run(ENTRY, REGS)[0][7], 0x34343434)
            else:
                with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
                    r.run(ENTRY, REGS)

    def test_zero_count_does_not_touch_null_destination(self):
        r = runner(call_fill(0, count=0)+b'\xc3')
        self.assertEqual(r.run(ENTRY, REGS)[0][7], 0)
        self.assertEqual(r.fills[0]['count'], 0)

    def test_refuse_readonly_and_code_destinations(self):
        for destination, output in ((OUT, Region('output', OUT, bytes(8))), (ENTRY, None)):
            with self.assertRaisesRegex(ValueError, 'read-only write|WRITE_PROT'):
                runner(call_fill(destination)+b'\xc3', output=output).run(ENTRY, REGS)

    def test_byte_bounds_and_unallocated_arena(self):
        for destination, count, reason in ((OUT+6, 3, 'undeclared access'),
                                           (ARENA, 1, 'outside modeled allocation')):
            with self.assertRaisesRegex(ValueError, reason):
                runner(call_fill(destination, count=count)+b'\xc3').run(ENTRY, REGS)

    def test_caps_and_direction_flag(self):
        for code, options, regs, reason in (
            (call_fill(count=5), {'max_fill_bytes': 4}, REGS, 'byte cap'),
            (call_fill()+call_fill(offset=23), {'max_fills': 1}, REGS, 'call cap'),
            (call_fill(), {}, (*REGS[:-1], REGS[-1] | 0x400), 'direction flag')):
            with self.assertRaisesRegex(ValueError, reason):
                runner(code+b'\xc3', **options).run(ENTRY, regs)

    def test_reset_and_fresh_match(self):
        code = call_fill()+b'\xc3'; r = runner(code)
        first = r.run(ENTRY, REGS)
        self.assertEqual(first, r.run(ENTRY, REGS))
        self.assertEqual(len(r.fills), 1)
        self.assertEqual(first, runner(code).run(ENTRY, REGS))
        # Re-enter a second code entry which only reads the untouched half.
        tail = ENTRY+len(code)
        r = runner(code+b'\xa1'+word(OUT+4)+b'\xc3')
        r.run(ENTRY, REGS)
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            r.run(tail, REGS)
        self.assertEqual(r.fills, [])

    def test_missing_arguments_and_mid_service_entry(self):
        for target, reason in ((FILL, 'undeclared access'), (FILL+1, 'entry inside')):
            code = b'\xe9'+struct.pack('<i', target-ENTRY-5)
            with self.assertRaisesRegex(ValueError, reason):
                runner(code).run(ENTRY, REGS)

    def test_partial_failure_does_not_initialize_next_run(self):
        failing = call_fill(OUT+6, count=3)+b'\xc3'
        r = runner(failing+b'\xa1'+word(OUT+4)+b'\xc3')
        with self.assertRaisesRegex(ValueError, 'undeclared access'):
            r.run(ENTRY, REGS)
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            r.run(ENTRY+len(failing), REGS)

if __name__ == '__main__':
    unittest.main()
