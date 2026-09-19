#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Modeled thread context, not captured replay or a Windows exception runtime.

Load original code only from the user's install. Refuse the first unavailable
dependency; never map it on demand. Keep emitted observations outside git.
"""
import argparse
import json
from pathlib import Path
import struct
from unicorn import UC_HOOK_MEM_INVALID, x86_const as x
from bounded_call import BoundedCall, Region
from replay_capsule import digest, image_bytes, require


def word(value):
    return struct.pack('<I', value)


class ThreadCall(BoundedCall):
    """An explicit four-byte FS slot plus a host-authored 32-bit descriptor.

    Byte guards, not the CPU's segment limit, are the access authority. No
    other TEB fields, TLS slots, exception dispatch, or captured SIMD state.
    """
    def __init__(self, regions, stop, *, fs_address, gdt_address, budget=256):
        require(4096 <= fs_address <= 2**32-4, 'invalid FS address')
        slot = next((r for r in regions if r.address == fs_address and len(r.data) == 4), None)
        require(slot is not None and slot.writable and not slot.scratch and not slot.executable,
                'FS needs an explicit writable four-byte input')
        # Present, accessed, writable data; byte granularity, 32-bit operand size.
        descriptor = struct.pack('<HHBBBB', 3, fs_address & 65535,
                                 (fs_address >> 16) & 255, 0x93, 0x40, fs_address >> 24)
        # Loading segments requires an explicit flat 32-bit stack descriptor;
        # leaving SS at Unicorn's default truncates stack addresses to 16 bits.
        flat = struct.pack('<HHBBBB', 65535, 0, 0, 0x93, 0xcf, 0)
        gdt = Region('_thread_gdt', gdt_address, bytes(8) + descriptor + flat)
        regions = [*regions, gdt]
        require(all(r.address >= 4096 for r in regions), 'null page must remain unmapped')
        super().__init__(regions, stop, budget)
        self.uc.mem_write(gdt.address, gdt.data)
        self.uc.reg_write(x.UC_X86_REG_GDTR, (0, gdt.address, len(gdt.data)-1, 0))
        self.uc.reg_write(x.UC_X86_REG_FS, 8)
        for register in (x.UC_X86_REG_SS, x.UC_X86_REG_DS, x.UC_X86_REG_ES):
            self.uc.reg_write(register, 16)
        self.initial_context = self.uc.context_save()

    def run(self, entry, registers, inputs=None):
        require('_thread_gdt' not in (inputs or {}), 'descriptor is host configuration')
        return super().run(entry, registers, inputs)


ENTRY, STOP, ESP, FS, GDT = 0x682f30, 0x400000, 0x300100, 0x70000000, 0x71000000


def probe(image, with_thread, table=None):
    # Authored arguments are sufficient only for this unconditional prologue.
    # No captured graph, game global or live thread value is being claimed.
    regions = [Region('code', ENTRY, image_bytes(image, ENTRY, 256), executable=True),
               Region('arguments', ESP, word(STOP) + bytes(32)),
               Region('scratch', ESP-4096, bytes(4096), writable=True, scratch=True)]
    if table is not None:
        require(with_thread and len(table) > 0 and len(table) % 8 == 0, 'invalid table probe input')
        regions += [Region('coordinate_table', 0x2000000, table),
                    Region('coordinate_center', 0xcae5fc, word(0x2000000+len(table)//2))]
    if with_thread:
        regions.append(Region('seh_head', FS, word(0xffffffff), writable=True))
        runner = ThreadCall(regions, STOP, fs_address=FS, gdt_address=GDT)
    else:
        runner = BoundedCall(regions, STOP)
    faults = []

    def fault(uc, kind, address, size, value, _):
        faults.append({'kind': kind, 'address': hex(address), 'size': size})
        return False

    runner.uc.hook_add(UC_HOOK_MEM_INVALID, fault)
    registers = (0, 0, 0, ESP, 0, 0, 0, 0, 0x202)
    try:
        runner.run(ENTRY, registers)
    except ValueError as error:
        refusal = str(error)
    else:
        raise ValueError('unexpected full-call success')
    expected_pc, expected_address = (0x682f54, '0xcae5fc') if with_thread else (0x682f3a, '0x0')
    if table is not None:
        expected_pc, expected_address = 0x682f77, '0xc0aec0'
    pc = runner.uc.reg_read(x.UC_X86_REG_EIP)
    require(pc == expected_pc and len(faults) == 1 and faults[0]['address'] == expected_address
            and faults[0]['size'] == 4 and 'READ_UNMAPPED' in refusal,
            f'unexpected dependency boundary: {pc:x}, {faults}, {refusal}')
    if with_thread:
        require(bytes(runner.uc.mem_read(FS, 4)) == word(ESP-16), 'SEH link not installed')
        require(bytes(runner.uc.mem_read(ESP-16, 4)) == word(0xffffffff), 'old SEH head not saved')
        require(runner.uc.reg_read(x.UC_X86_REG_XMM0) == 0, 'prefix did not clear XMM0')
    return {'modeled_thread': with_thread, 'table_supplied': table is not None,
            'fault_pc': hex(pc), 'faults': faults,
            'attempted_instructions': runner.instructions, 'mapped_bytes': runner.mapped_bytes,
            'refusal': refusal, 'null_page_mapped': any(a == 0 for a, _, _ in runner.uc.mem_regions())}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    args = parser.parse_args()
    image = args.install / 'riseofnations.exe'
    results = [probe(image, False), probe(image, True)]
    require(not any(r['null_page_mapped'] for r in results), 'null page mapped')
    print(json.dumps({'schema': 1, 'source_sha256': digest(image),
                      'claim': 'modeled prologue dependency experiment; not captured replay',
                      'results': results}, indent=2))


if __name__ == '__main__':
    main()
