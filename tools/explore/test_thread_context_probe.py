#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Authored instructions only: FS state must not authorize missing bytes."""
import unittest
from unicorn import x86_const as x
from bounded_call import Region
from thread_context_probe import ThreadCall, word

ENTRY, STOP, ESP, FS, GDT = 0x100000, 0x200000, 0x300100, 0x70000000, 0x71000000
REGS = (0, 0, 0, ESP, 0, 0, 0, 0, 0x202)


def runner(code, head=0x12345678):
    regions = [Region('code', ENTRY, code, executable=True),
               Region('return', ESP, word(STOP)),
               Region('scratch', ESP-16, bytes(16), writable=True, scratch=True),
               Region('seh', FS, word(head), writable=True)]
    return ThreadCall(regions, STOP, fs_address=FS, gdt_address=GDT)


class ThreadTest(unittest.TestCase):
    def test_fs_mutation_and_context_reset(self):
        # mov eax,fs:[0]; inc dword fs:[0]; ret
        r = runner(bytes.fromhex('64a10000000064ff0500000000c3'))
        first = r.run(ENTRY, REGS)
        self.assertEqual(first[0][7], 0x12345678)
        self.assertEqual(dict(first[1])['seh'], word(0x12345679))
        changed = r.run(ENTRY, REGS, {'seh': word(9)})
        self.assertEqual(changed[0][7], 9)
        # Corrupt hidden segment state; run() must restore the descriptor cache.
        r.uc.reg_write(x.UC_X86_REG_FS, 0)
        self.assertEqual(first, r.run(ENTRY, REGS))
        self.assertFalse(any(a == 0 for a, _, _ in r.uc.mem_regions()))

    def test_fs_padding_is_not_authorized(self):
        with self.assertRaises(ValueError):
            runner(bytes.fromhex('64a104000000c3')).run(ENTRY, REGS)

    def test_stack_link_round_trip(self):
        # Save old FS head, link stack, pop and restore it; ret.
        r = runner(bytes.fromhex('64ff3500000000648925000000005864a300000000c3'))
        result = r.run(ENTRY, REGS)
        self.assertEqual(dict(result[1])['seh'], word(0x12345678))
        self.assertIn((FS, 4, ESP-4), result[2])
        self.assertEqual(result[0][3], ESP+4)

    def test_plain_null_read_still_fails(self):
        with self.assertRaisesRegex(ValueError, 'READ_UNMAPPED'):
            runner(bytes.fromhex('a100000000c3')).run(ENTRY, REGS)

    def test_thread_pointer_does_not_authorize_pointee(self):
        with self.assertRaisesRegex(ValueError, 'READ_UNMAPPED'):
            runner(bytes.fromhex('64a1000000008b00c3')).run(ENTRY, REGS)

    def test_descriptor_cannot_be_overridden(self):
        with self.assertRaisesRegex(ValueError, 'host configuration'):
            runner(b'\xc3').run(ENTRY, REGS, {'_thread_gdt': bytes(16)})

    def test_scratch_still_needs_a_write(self):
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            runner(b'\xa1' + word(ESP-4) + b'\xc3').run(ENTRY, REGS)

    def test_simd_context_resets(self):
        # pcmpeqd xmm0,xmm0; ret. XMM0 was deliberately changed after construction.
        r = runner(bytes.fromhex('660f76c0c3'))
        initial = r.uc.reg_read(x.UC_X86_REG_XMM1)
        r.uc.reg_write(x.UC_X86_REG_XMM1, 123)
        r.run(ENTRY, REGS)
        self.assertEqual(r.uc.reg_read(x.UC_X86_REG_XMM0), 2**128-1)
        self.assertEqual(r.uc.reg_read(x.UC_X86_REG_XMM1), initial)


if __name__ == '__main__':
    unittest.main()
