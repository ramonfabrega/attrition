#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Execute the owned coordinate initializer across an explicit malloc boundary.

No allocator code is stubbed. Phase one stops at its requested allocation;
phase two starts with a declared successful allocation and original saved state.
This models malloc success, not allocation failure or the game's live extent.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from bounded_call import BoundedCall, Region
from replay_capsule import image_bytes, digest, require
from thread_context_probe import word, probe

ENTRY, RESUME, MALLOC, STOP = 0x681db0, 0x681dd2, 0x500000, 0x400000
ESP, BUFFER, ORIGIN, CENTER, IAT = 0x300100, 0x2000000, 0xcab3ac, 0xcae5fc, 0xac54f0


def initialize(image, size):
    require(type(size) is int and 1 <= size <= 4096, 'probe size outside 1..4096')
    code = Region('code', ENTRY, image_bytes(image, ENTRY, 0x7d), executable=True)
    first = BoundedCall([code, Region('malloc_import', IAT, word(MALLOC)),
                         Region('return', ESP, word(STOP)),
                         Region('scratch', ESP-16, bytes(16), writable=True, scratch=True)], MALLOC)
    before = (0x11223344, 0x55667788, 0, ESP, 0x1234, 0, size, 0, 0x202)
    regs, memory, _ = first.run(ENTRY, before)
    saved = dict(memory)['scratch']
    requested = struct.unpack_from('<I', saved, 4)[0]
    require(regs[3] == ESP-16 and struct.unpack_from('<I', saved)[0] == RESUME,
            'unexpected allocation call stack')
    require(requested == size*192, 'unexpected allocation extent')
    # The external allocator returns a fresh exact-sized buffer and preserves
    # nonvolatile registers. ECX/EDX are clobbered to detect accidental reliance.
    resumed = list(regs)
    resumed[3], resumed[7] = ESP-12, BUFFER
    resumed[5], resumed[6] = 0xdeadbeef, 0xa5a5a5a5
    regions = [code, Region('return', ESP, word(STOP)),
               Region('saved_stack', ESP-12, saved[4:]),
               Region('allocation', BUFFER, bytes(requested), writable=True, scratch=True),
               Region('origin', ORIGIN, bytes(4), writable=True, scratch=True),
               Region('center', CENTER, bytes(4), writable=True, scratch=True)]
    second = BoundedCall(regions, STOP, budget=size*500+100)
    after, outputs, writes = second.run(RESUME, resumed)
    outputs = dict(outputs)
    require(after[3] == ESP+4 and all(after[i] == before[i] for i in (0, 1, 2, 4)),
            'callee-saved state or stack mismatch')
    require(outputs['origin'] == word(BUFFER) and outputs['center'] == word(BUFFER+requested//2),
            'unexpected table pointers')
    require(all(a in second.initialized for a in range(BUFFER, BUFFER+requested)),
            'initializer left allocation bytes unwritten')
    half = size*24
    values = struct.unpack('<'+'i'*(half*2), outputs['allocation'])
    # Independent mathematical specification: floor division, including negatives.
    expected = tuple(index//3 for index in range(-half, half))
    require(values == expected, 'table differs from floor(index / 3)')
    report = {'size': size, 'allocation_bytes': requested, 'entries_checked': len(values),
              'index_first': -half, 'index_last': half-1,
              'instructions': first.instructions+second.instructions,
              'table_sha256': hashlib.sha256(outputs['allocation']).hexdigest(),
              'all_bytes_written': True, 'allocator': 'explicit successful-allocation boundary'}
    return report, outputs['allocation']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('install', type=Path)
    parser.add_argument('--sizes', type=int, nargs='+', default=[1, 2, 17, 128, 400, 1024])
    args = parser.parse_args()
    image = args.install/'riseofnations.exe'
    runs = [initialize(image, size) for size in args.sizes]
    print(json.dumps({'schema': 1, 'source_sha256': digest(image),
                      'runs': [run[0] for run in runs],
                      'search_frontier': probe(image, True, runs[-1][1])}, indent=2))


if __name__ == '__main__':
    main()
