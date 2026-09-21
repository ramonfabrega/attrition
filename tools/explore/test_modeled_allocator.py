"""Authored x86 tests for the allocation model's ABI and refusal boundaries."""
import struct
import unittest
from bounded_call import Region
from modeled_allocator import AllocatorCall

ENTRY, SERVICE, ARENA, ESP, STOP = 0x100000, 0x500000, 0x600000, 0x300100, 0x400000
REGS = (1, 2, 3, ESP, 4, 5, 6, 7, 0x202)

def word(n):
    return struct.pack('<I', n)

def allocation(offset=0, size=4):
    # push size; call service; add esp,4
    return b'\x68'+word(size)+b'\xe8'+struct.pack('<i', SERVICE-(ENTRY+offset+10))+b'\x83\xc4\x04'

def runner(code, **kwargs):
    return AllocatorCall([
        Region('code', ENTRY, code, executable=True),
        Region('return', ESP, word(STOP)),
        Region('stack', ESP-256, bytes(256), writable=True, scratch=True),
        Region('fs', 0x70000000, word(0xffffffff), writable=True)], STOP,
        malloc_target=kwargs.pop("malloc_target", SERVICE), arena_address=ARENA, arena_size=kwargs.pop('arena_size', 64),
        fs_address=0x70000000, gdt_address=0x71000000, **kwargs)

class AllocatorTest(unittest.TestCase):
    def test_cdecl_and_reset(self):
        r = runner(allocation()+b'\xc7\x00'+word(123)+b'\x8b\x00\xc3')
        first = r.run(ENTRY, REGS)
        self.assertEqual(first[0][7], 123)
        self.assertEqual(first[0][3], ESP+4)
        for i in (0, 1, 2, 4, 5, 6):
            self.assertEqual(first[0][i], REGS[i])
        self.assertEqual(r.allocations, [dict(address=ARENA, size=4, return_address=ENTRY+10)])
        self.assertEqual(first, r.run(ENTRY, REGS))
        self.assertEqual(len(r.allocations), 1)
        self.assertEqual(first, runner(allocation()+b'\xc7\x00'+word(123)+b'\x8b\x00\xc3').run(ENTRY, REGS))

    def test_multiple_aligned_allocations(self):
        r = runner(allocation(size=3)+allocation(13, 5)+b'\xc3')
        r.run(ENTRY, REGS)
        self.assertEqual([(a['address'], a['size']) for a in r.allocations], [(ARENA, 3), (ARENA+16, 5)])

    def test_unwritten_and_partial_reads_refuse(self):
        for tail in (b'\x8b\x00\xc3', b'\xc6\x00\x01\x8b\x00\xc3'):
            with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
                runner(allocation()+tail).run(ENTRY, REGS)

    def test_previous_run_does_not_initialize_heap(self):
        # ECX decides whether to write this run; the same object is reused.
        r = runner(allocation()+b'\x85\xc9\x74\x06\xc7\x00'+word(123)+b'\x8b\x00\xc3')
        r.run(ENTRY, REGS)
        registers = list(REGS); registers[6] = 0
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            r.run(ENTRY, registers)

    def test_allocation_bounds_and_unused_arena(self):
        for code in (allocation()+b'\xc7\x40\x01'+word(1)+b'\xc3',
                     b'\xa3'+word(ARENA)+b'\xc3',
                     allocation(size=3)+b'\xa3'+word(ARENA+3)+b'\xc3'):
            with self.assertRaisesRegex(ValueError, 'outside modeled allocation'):
                runner(code).run(ENTRY, REGS)

    def test_quotas_and_zero_size(self):
        for code, opts, reason in [
            (allocation(size=0), {}, 'zero-size'),
            (allocation(size=65), {}, 'arena exhausted'),
            (allocation()+allocation(13), {'max_allocations': 1}, 'quota exhausted')]:
            with self.assertRaisesRegex(ValueError, reason):
                runner(code+b'\xc3', **opts).run(ENTRY, REGS)

    def test_service_argument_is_checked(self):
        # Jump to service with return on stack but no declared size argument.
        code = b'\xe9'+struct.pack('<i', SERVICE-ENTRY-5)
        with self.assertRaisesRegex(ValueError, 'undeclared access'):
            runner(code).run(ENTRY, REGS)

    def test_other_service_and_arena_code_refuse(self):
        for target in (SERVICE+1, ARENA):
            with self.assertRaises(ValueError):
                runner(b'\xe9'+struct.pack('<i', target-ENTRY-5)).run(ENTRY, REGS)

    def test_return_target_and_uninitialized_argument_refuse(self):
        # A guest pushes an invalid return then jumps to the modeled service.
        code = b'\x68'+word(4)+b'\x68'+word(ARENA)+b'\xe9'+struct.pack('<i', SERVICE-ENTRY-15)
        with self.assertRaisesRegex(ValueError, 'return target'):
            runner(code).run(ENTRY, REGS)
        # Move ESP into unwritten stack scratch and enter the service.
        code = b'\x83\xec\x08\xe9'+struct.pack('<i', SERVICE-ENTRY-8)
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            runner(code).run(ENTRY, REGS)

    def test_failure_then_reset_is_clean(self):
        r = runner(allocation()+b'\xc3')
        r.run(ENTRY, REGS)
        # Starting at the service checks undeclared size above the return.
        with self.assertRaises(ValueError):
            r.run(SERVICE, REGS)
        self.assertEqual(r.allocations, [])
        r.run(ENTRY, REGS)
        self.assertEqual(r.allocations[0]['address'], ARENA)

    def test_overlap_refuses(self):
        with self.assertRaisesRegex(ValueError, 'overlapping'):
            runner(b'\xc3', arena_size=64, malloc_target=ARENA)

if __name__ == '__main__':
    unittest.main()
