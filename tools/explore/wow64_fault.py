#!/usr/bin/env python3
"""Check a reported read against both decodes of an installed mov edx operand.

Only handles the six-byte 8b/15 disp32 form. It does not infer active CPU mode.
Input is an installed PE32+ module; no module bytes are stored in the repo.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct


def instruction(data, rva):
    if data[:2] != b'MZ': raise ValueError('not a PE image')
    try:
        pe = struct.unpack_from('<I', data, 60)[0]
        if data[pe:pe+4] != b'PE\0\0': raise ValueError('bad PE signature')
        machine, sections = struct.unpack_from('<HH',data,pe+4)
        optional_size = struct.unpack_from('<H',data,pe+20)[0]
        magic = struct.unpack_from('<H',data,pe+24)[0]
        if machine != 0x8664 or magic != 0x20b: raise ValueError('requires x86-64 PE32+')
        for i in range(sections):
            offset=pe+24+optional_size+40*i
            virtual_size, start, raw_size, raw = struct.unpack_from('<4I',data,offset+8)
            if start <= rva and rva+6 <= start+min(virtual_size,raw_size):
                offset=raw+rva-start
                if offset+6 > len(data): raise ValueError('truncated section')
                return data[offset:offset+6]
    except struct.error as error:
        raise ValueError('truncated PE headers') from error
    raise ValueError('instruction is not within a file-backed section')


def evidence(data, loaded_base, pc, access):
    if min(loaded_base,pc,access)<0 or pc < loaded_base: raise ValueError('invalid addresses')
    code=instruction(data,pc-loaded_base)
    if code[:2] != b'\x8b\x15': raise ValueError('unsupported instruction form')
    absolute=struct.unpack_from('<I',code,2)[0]
    relative=(pc+6+struct.unpack_from('<i',code,2)[0]) & 0xffffffffffffffff
    return {'module_sha256':hashlib.sha256(data).hexdigest(), 'rva':hex(pc-loaded_base),
            'load_base':hex(loaded_base),'pc':hex(pc),'reported_access':hex(access),
            'i386_effective_address':hex(absolute),'x86_64_effective_address':hex(relative),
            'access_matches_i386':access==absolute,'access_matches_x86_64':access==relative,
            'active_cpu_mode_established':False}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('module',type=Path)
    for name in ('loaded_base','pc','access'): parser.add_argument(name,type=lambda s:int(s,0))
    args=parser.parse_args()
    print(json.dumps(evidence(args.module.read_bytes(),args.loaded_base,args.pc,args.access),indent=2))


if __name__=='__main__': main()
