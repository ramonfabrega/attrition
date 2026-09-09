#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Authored x86 fixtures: nested calls, pointer writes, reset and byte guards."""
import struct
import unittest
from bounded_call import BoundedCall, Region

ENTRY, NESTED, PTR, OUT, ESP, STOP = 0x100000, 0x101000, 0x200000, 0x200010, 0x300010, 0x400000

def word(n):
    return struct.pack('<I', n)


def fixture():
    # call nested; ret. Nested: load output pointer, increment its word; ret.
    code = b'\xe8'+struct.pack('<i', NESTED-ENTRY-5)+b'\xc3'
    nested = b'\xa1'+word(PTR)+b'\xff\x00\xc3'
    return [Region('code', ENTRY, code, executable=True),
            Region('nested', NESTED, nested, executable=True),
            Region('pointer', PTR, word(OUT)), Region('output', OUT, word(10), writable=True),
            Region('return', ESP, word(STOP)),
            Region('scratch', ESP-4, bytes(4), writable=True, scratch=True)]

REGS = (1, 2, 3, ESP, 4, 5, 6, 7, 0x202)


class BoundedTest(unittest.TestCase):
    def test_nested_pointer_mutation_resets_between_calls(self):
        r = BoundedCall(fixture(), STOP)
        first = r.run(ENTRY, REGS)
        self.assertEqual(dict(first[1])['output'], word(11))
        r.run(ENTRY, REGS, {'output': word(100)})
        self.assertEqual(first, r.run(ENTRY, REGS))
        self.assertEqual(first, BoundedCall(fixture(), STOP).run(ENTRY, REGS))

    def test_scratch_cannot_supply_implicit_zero_or_previous_call_bytes(self):
        layout = fixture()
        layout[0] = Region('code', ENTRY, b'\xa1'+word(ESP-4)+b'\xc3', executable=True)
        r = BoundedCall(layout, STOP)
        for _ in range(2):
            with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
                r.run(ENTRY, REGS)

    def test_previous_call_stack_writes_do_not_initialize_next_call(self):
        layout = fixture()
        code = b'\x85\xc0\x74\x06\xe8'+struct.pack('<i', NESTED-ENTRY-9)+b'\xc3\xa1'+word(ESP-4)+b'\xc3'
        layout[0] = Region('code', ENTRY, code, executable=True)
        r = BoundedCall(layout, STOP)
        r.run(ENTRY, REGS)  # nested CALL writes the scratch return address
        regs = list(REGS); regs[7] = 0
        with self.assertRaisesRegex(ValueError, 'uninitialized scratch'):
            r.run(ENTRY, regs)

    def test_missing_pointer_fails_even_on_mapped_output_page(self):
        with self.assertRaisesRegex(ValueError, 'undeclared access'):
            BoundedCall([r for r in fixture() if r.name != 'pointer'], STOP).run(ENTRY, REGS)

    def test_pointer_cannot_authorize_adjacent_output(self):
        with self.assertRaisesRegex(ValueError, 'undeclared access'):
            BoundedCall(fixture(), STOP).run(ENTRY, REGS, {'pointer': word(OUT+4)})

    def test_read_only_output_rejected(self):
        layout = fixture(); layout[3] = Region('output', OUT, word(10))
        with self.assertRaises(ValueError):
            BoundedCall(layout, STOP).run(ENTRY, REGS)

    def test_jump_to_padding_on_executable_page_rejected(self):
        layout = fixture(); layout[0] = Region('code', ENTRY, b'\xeb\x20', executable=True)
        with self.assertRaisesRegex(ValueError, 'undeclared instruction'):
            BoundedCall(layout, STOP).run(ENTRY, REGS)

    def test_budget_is_enforced(self):
        layout = fixture(); layout[0] = Region('code', ENTRY, b'\xeb\xfe', executable=True)
        with self.assertRaisesRegex(ValueError, 'budget exhausted'):
            BoundedCall(layout, STOP, budget=8).run(ENTRY, REGS)

    def test_starting_at_stop_is_not_a_successful_call(self):
        with self.assertRaisesRegex(ValueError, 'entry is not declared code'):
            BoundedCall(fixture(), STOP).run(STOP, REGS)

    def test_overlap_rejected(self):
        with self.assertRaisesRegex(ValueError, 'overlapping'):
            BoundedCall(fixture()+[Region('alias', OUT, bytes(1))], STOP)

    def test_code_override_rejected(self):
        with self.assertRaisesRegex(ValueError, 'unknown/code/scratch'):
            BoundedCall(fixture(), STOP).run(ENTRY, REGS, {'code': bytes(6)})


if __name__ == '__main__':
    unittest.main()
