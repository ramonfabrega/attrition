"""Object retirement must catch aliases, partial overlaps and reset leaks."""
import struct
import unittest
from bounded_call import Region
from modeled_lifetime import LifetimeCall

ENTRY, MALLOC, FILL, COPY, FREE = 0x100000, 0x500000, 0x501000, 0x502000, 0x503000
OBJ, ESP, STOP, ARENA = 0x200000, 0x300100, 0x400000, 0x600000
REGS = (1, 2, 3, ESP, 4, 5, 6, 7, 0x202)

def word(n):
    return struct.pack('<I', n)

def call_free(pointer=OBJ, offset=0):
    return b'\x68'+word(pointer)+b'\xe8'+struct.pack('<i', FREE-ENTRY-offset-10)+b'\x83\xc4\x04'

def allocation(offset=0):
    return b'\x6a\x04\xe8'+struct.pack('<i', MALLOC-ENTRY-offset-7)+b'\x83\xc4\x04'

def runner(code, *, borrowed=((OBJ, 8),), **kwargs):
    return LifetimeCall([
        Region('code', ENTRY, code, executable=True),
        Region('borrowed', OBJ, b'abcdefgh', writable=True),
        Region('return', ESP, word(STOP)),
        Region('stack', ESP-256, bytes(256), writable=True, scratch=True),
        Region('fs', 0x70000000, word(0xffffffff), writable=True)], STOP,
        borrowed=borrowed, malloc_target=MALLOC, memset_target=FILL, memcpy_target=COPY,
        free_target=FREE, arena_address=ARENA, arena_size=64,
        fs_address=0x70000000, gdt_address=0x71000000, **kwargs)


class LifetimeTest(unittest.TestCase):
    def test_borrowed_free_and_abi(self):
        r = runner(call_free()+b'\xc3'); result = r.run(ENTRY, REGS)
        self.assertEqual(r.retired, [(OBJ, 8)])
        self.assertEqual(r.frees, [dict(pointer=OBJ, size=8, source='borrowed', return_address=ENTRY+10)])
        self.assertEqual(result[0][:3], REGS[:3])
        self.assertEqual(result[0][4:8], REGS[4:8])
        self.assertEqual(result[0][3], ESP+4)

    def test_retired_reads_writes_and_partial_overlap_refuse(self):
        for tail in (b'\xa1'+word(OBJ), b'\xa3'+word(OBJ+4), b'\xa1'+word(OBJ+7)):
            with self.assertRaisesRegex(ValueError, 'retired object access'):
                runner(call_free()+tail+b'\xc3').run(ENTRY, REGS)

    def test_unknown_interior_and_double_free(self):
        for code, reason in ((call_free(OBJ+1), 'unknown free'),
                             (call_free(ARENA), 'unknown free'),
                             (call_free()+call_free(offset=13), 'double free')):
            with self.assertRaisesRegex(ValueError, reason):
                runner(code+b'\xc3').run(ENTRY, REGS)

    def test_null_free_and_quota(self):
        r = runner(call_free(0)+call_free(0, 13)+b'\xc3')
        r.run(ENTRY, REGS)
        self.assertEqual(r.retired, [])
        self.assertEqual([f['source'] for f in r.frees], ['null', 'null'])
        with self.assertRaisesRegex(ValueError, 'free call cap'):
            runner(call_free(0)+call_free(0, 13)+b'\xc3', max_frees=1).run(ENTRY, REGS)

    def test_modeled_free_and_no_reuse(self):
        code = allocation()+call_free(ARENA, 10)+allocation(23)+b'\xc3'
        r = runner(code); result = r.run(ENTRY, REGS)
        self.assertEqual(result[0][7], ARENA+16)
        self.assertEqual(r.frees[0]['source'], 'modeled')
        self.assertEqual(r.retired, [(ARENA, 4)])
        with self.assertRaisesRegex(ValueError, 'retired object access'):
            runner(allocation()+call_free(ARENA, 10)+b'\xa3'+word(ARENA)+b'\xc3').run(ENTRY, REGS)

    def test_reset_revives_input_not_previous_retirement(self):
        code = call_free()+b'\xc3'; r = runner(code+b'\xa1'+word(OBJ)+b'\xc3')
        first = r.run(ENTRY, REGS)
        self.assertEqual(first, r.run(ENTRY, REGS))
        self.assertEqual(r.run(ENTRY+len(code), REGS)[0][7], int.from_bytes(b'abcd', 'little'))
        self.assertEqual(r.retired, [])
        self.assertEqual(r.frees, [])
        self.assertEqual(first, runner(code+b'\xa1'+word(OBJ)+b'\xc3').run(ENTRY, REGS))

    def test_borrowing_needs_exact_input_extent(self):
        for borrowed in (((OBJ, 4),), ((OBJ+1, 7),), ((OBJ, 8), (OBJ, 8)), ((ARENA, 8),)):
            with self.assertRaises(ValueError):
                runner(b'\xc3', borrowed=borrowed)

    def test_failed_retirement_attempt_resets(self):
        code = call_free()+call_free(offset=13)+b'\xc3'
        r = runner(code+b'\xa1'+word(OBJ)+b'\xc3')
        with self.assertRaisesRegex(ValueError, 'double free'):
            r.run(ENTRY, REGS)
        self.assertEqual(r.run(ENTRY+len(code), REGS)[0][7], int.from_bytes(b'abcd', 'little'))

if __name__ == '__main__':
    unittest.main()
