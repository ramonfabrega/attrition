#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Sparse original turn_angles oracle; synthetic normal-member inputs, not live captures.

Outputs derived data: redirect outside the repository. Compare with the Rust
turn_angles_oracle_check example. No whole executable mapping or launch.
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct
import sys
import time
from bounded_call import BoundedCall, Region
from replay_capsule import image_bytes, require

ENTRY, SPEED, STOP = 0x5d98c0, 0x5de340, 0x40000000
GUY, UNIT, TYPE, CONSTANTS, SLOTS, OUT = [0x10000000+i*4096 for i in range(6)]
ESP = 0x20000100


def word(n):
    return struct.pack('<I', n & 0xffffffff)


def regions(image):
    # Code extents and operand addresses checked against the local listing.
    # Fields use the same PDB-backed graph as turn_oracle.py. Omitted fields
    # are inaccessible, including the crew track-offset branch.
    return [Region('angles_code', ENTRY, image_bytes(image, ENTRY, 0x86), executable=True),
            Region('speed_code', SPEED, image_bytes(image, SPEED, 0xca), executable=True),
            Region('heading', GUY+0x18, word(0)),
            Region('speeds', GUY+0x80, bytes(8)),
            Region('object', GUY+0x8c, bytes(2)),
            Region('flags', GUY+0x9a, bytes(1)),
            Region('owner_member', GUY+0xa1, bytes(2)),
            Region('registry', 0xc0aec0, word(SLOTS)),
            Region('slot', SLOTS, word(UNIT)),
            Region('unit_type', UNIT+0x18, word(TYPE)),
            Region('pack', UNIT+0x68, word(0)),
            Region('rate', TYPE+0x2c4, word(0)),
            Region('squad', TYPE+0x304, word(1)),
            Region('constants', 0xc061e4, word(CONSTANTS)),
            Region('tuning', CONSTANTS+8, bytes(8)),
            Region('output', OUT, word(0xdeadbeef), writable=True),
            Region('arguments', ESP, bytes(20)),
            Region('scratch', ESP-40, bytes(40), writable=True, scratch=True)]


def arguments(row):
    rate, pack, stopped, last, avg, mode, turn, bonus, start, target, half = row
    return {'heading': word(start), 'speeds': word(last)+word(avg),
            'flags': bytes([stopped << 4]), 'pack': word(pack << 19),
            'rate': word(rate), 'tuning': word(turn)+word(bonus),
            'arguments': b''.join(map(word, (STOP, target, OUT, mode, half)))}


def cases(count):
    rng = random.Random(12345)
    angles = [0, 1, 0x222221f, 0x2222220, 0x2222221, 0x7fffffff, 0x80000000, 0x80000001, 0xffffffff]
    for i in range(count):
        start = rng.getrandbits(32)
        delta = angles[i % len(angles)] if i % 2 else rng.getrandbits(32)
        yield (rng.choice([0, 255, 256, 0x20000000, 0x80000000, 0xffffffff]),
               rng.randrange(2), rng.randrange(2), rng.choice([0, 25]),
               rng.choice([0, 3, 4, 25, 255]), rng.randrange(2),
               rng.choice([1, 256]), 2, start, (start+delta) & 0xffffffff, rng.randrange(2))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install', type=Path)
    ap.add_argument('--cases', type=int, default=4096)
    args = ap.parse_args()
    require(args.cases >= 32, 'at least 32 cases required')
    image = args.install / 'riseofnations.exe'
    layout = regions(image)
    runner = BoundedCall(layout, STOP)
    regs = (0x11111111, 0x22222222, 0x33333333, ESP, 0x44444444, 0x55555555, GUY, 0x66666666, 0x202)
    rows = list(cases(args.cases))
    begin = time.perf_counter()
    results = [runner.run(ENTRY, regs, arguments(row)) for row in rows]
    reused_s = time.perf_counter()-begin
    # Reverse order on the reused engine must match all registers, memory,
    # scratch writes and output; then compare every case against a fresh engine.
    for i in reversed(range(len(rows))):
        require(runner.run(ENTRY, regs, arguments(rows[i])) == results[i], f'order dependence: {i}')
    indices = range(len(rows))
    begin = time.perf_counter()
    for i in indices:
        fresh = BoundedCall(layout, STOP)
        require(fresh.run(ENTRY, regs, arguments(rows[i])) == results[i], f'fresh mismatch: {i}')
    fresh_s = time.perf_counter()-begin
    # This non-snapping, packed Unit-mode case touches every declared region.
    dependency_case = (0x20000000, 1, 0, 0, 25, 0, 256, 2, 0, 0x40000000, 0)
    for omitted in layout:
        reduced = [r for r in layout if r.name != omitted.name]
        allowed_inputs = {k: v for k, v in arguments(dependency_case).items() if k != omitted.name}
        try:
            BoundedCall(reduced, STOP).run(ENTRY, regs, allowed_inputs)
        except ValueError:
            pass
        else:
            raise ValueError(f'missing dependency passed: {omitted.name}')
    for row, (actual, memory, writes) in zip(rows, results):
        require(actual[3] == ESP+20, 'unexpected stack cleanup')
        require(all(actual[i] == regs[i] for i in (0, 1, 2, 4)), 'callee-saved register changed')
        output = struct.unpack('<I', dict(memory)['output'])[0]
        require([w for w in writes if w[0] == OUT] == [(OUT, 4, output)], 'output write contract')
        print(*row, output, actual[7])
    print(json.dumps({'cases': len(rows), 'fresh_checks': len(indices),
        'reverse_checks': len(rows), 'missing_dependency_controls': len(layout), 'declared_bytes': sum(len(r.data) for r in layout),
        'mapped_bytes': runner.mapped_bytes, 'reused_seconds': reused_s,
        'fresh_seconds': fresh_s, 'source_sha256': hashlib.sha256(image.read_bytes()).hexdigest()}), file=sys.stderr)


if __name__ == '__main__':
    main()
