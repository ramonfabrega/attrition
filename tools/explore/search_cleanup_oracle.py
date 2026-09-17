#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Execute suspended-search cleanup on bounded, authored recycler graphs.

Synthetic inputs, not natural captures or a Rust path-search fidelity test.
No allocator adapter, payload mapping, game launch or full PE mapping is used.
Output contains original-derived observations; keep it outside the repository.
"""
import argparse
import hashlib
import json
from pathlib import Path
import random
import struct
import time
from bounded_call import BoundedCall, Region
from replay_capsule import image_bytes, require

ENTRY, STOP, UNIT, ESP = 0x5e3920, 0x40000000, 0x10000000, 0x20001000
TREES, NODES, PAYLOADS, ARRAYS = 0x10001000, 0x10002000, 0x30000000, 0x10003000
MAX_NODES = 31
POOLS = (0xc8d810, 0xc8d950, 0xc8d860, 0xc8d880, 0xc8d9a0,
         0xc8d9b0, 0xc8da70)
# Local listing bounds; omitted callees (including pool growth) must fail closed.
CODE = ((ENTRY, 0x2a2), (0x46da60, 0x69), (0x46db10, 0x70),
        (0x46db80, 0x70), (0x46de10, 0x70), (0x46dbf0, 0x70),
        (0x687c10, 0x64), (0x4550c0, 0x9d))
REGS = (0x11111111, 0x22222222, 0x33333333, ESP, 0x44444444,
        0x55555555, UNIT, 0x66666666, 0x202)
RESET_OFFSETS = (0x118, 0x11c, 0x128, 0x12c, 0x138, 0x13c, 0x140, 0x144, 0x148)


def words(*values):
    return struct.pack('<' + 'I'*len(values), *(v & 0xffffffff for v in values))


def put(data, offset, value):
    struct.pack_into('<I', data, offset, value)


def layout(image):
    result = [Region(f'code_{addr:x}', addr, image_bytes(image, addr, size), executable=True)
              for addr, size in CODE]
    result += [Region('unit', UNIT+0x104, bytes(0x48), writable=True),
               Region('nodes', NODES, bytes(MAX_NODES*20), writable=True),
               Region('return', ESP, words(STOP)),
               Region('scratch', ESP-1024, bytes(1024), writable=True, scratch=True)]
    for i in range(5):
        result.append(Region(f'tree{i}', TREES+i*32, bytes(28 if i in (0, 4) else 24), writable=True))
    for i, address in enumerate(POOLS):
        # list, capacity, length; increment is deliberately inaccessible.
        result += [Region(f'pool{i}', address, bytes(12), writable=True),
                   Region(f'array{i}', ARRAYS+i*256, bytes(256), writable=True)]
    return result


def fixture(count, shape, mask, occupied, seed):
    require(0 <= count <= MAX_NODES and shape in ('balanced', 'left', 'right'), 'bad graph')
    require(0 <= mask < 16 and 0 <= occupied <= 3, 'bad pool fixture')
    rng = random.Random(seed)
    data = {'unit': bytearray(words(*(rng.getrandbits(32) for _ in range(18)))),
            'nodes': bytearray(MAX_NODES*20)}
    edges = []
    for i in range(count):
        left = 2*i+1 if shape == 'balanced' else i+1 if shape == 'left' else count
        right = 2*i+2 if shape == 'balanced' else i+1 if shape == 'right' else count
        edges.append((left if left < count else None, right if right < count else None))
    parent = {child: i for i, pair in enumerate(edges) for child in pair if child is not None}
    def ptr(i):
        return 0 if i is None else NODES+i*20
    for i, (left, right) in enumerate(edges):
        data['nodes'][i*20:i*20+20] = words(ptr(left), ptr(right), ptr(parent.get(i)),
                                           PAYLOADS+i*64, rng.getrandbits(32))
    active = [True] + [bool(mask & (1 << i)) for i in range(4)]
    for i in range(5):
        put(data['unit'], i*4, TREES+i*32 if active[i] else 0)
        data[f'tree{i}'] = bytearray(28 if i in (0, 4) else 24)
    if count:
        data['tree0'][:] = words(PAYLOADS, 123, count, NODES, NODES, 0, 1)
    for i in range(7):
        data[f'pool{i}'] = bytearray(words(ARRAYS+i*256, 64, occupied))
        data[f'array{i}'] = bytearray(words(*(0xa5000000+j for j in range(64))))
    expected = {name: bytearray(value) for name, value in data.items()}
    for i in range(5):
        put(expected['unit'], i*4, 0)
    for offset in RESET_OFFSETS:
        put(expected['unit'], offset-0x104, 0)
    if count:
        for offset in (8, 12, 16, 20, 24):
            put(expected['tree0'], offset, 0)
    # Declarative ownership check: each reachable node and payload appears once.
    # Order is observed separately, rather than implementing native recursion here.
    for i in range(count):
        for offset in (0, 4, 12):
            put(expected['nodes'], i*20+offset, 0)
    return {k: bytes(v) for k, v in data.items()}, expected, active


def verify(result, expected, active, count, occupied):
    regs, state, writes = result
    require(all(regs[i] == REGS[i] for i in (0, 1, 2, 4)), 'callee-saved register changed')
    require(regs[3] == ESP+4, 'stack imbalance')
    state = dict(state)
    require(state['return'] == words(STOP), 'return address changed')
    for i in range(7):
        n = int(active[i]) if i < 5 else count
        pool = expected[f'pool{i}']
        put(pool, 8, occupied+n)
        actual = state[f'array{i}']
        slots = struct.unpack_from('<'+'I'*n, actual, occupied*4)
        targets = [TREES+i*32] if i < 5 and n else [
            (NODES+j*20 if i == 5 else PAYLOADS+j*64) for j in range(n)]
        require(sorted(slots) == sorted(targets), f'ownership mismatch: pool {i}')
        # Preserve both existing entries and unused capacity byte for byte.
        expected[f'array{i}'][occupied*4:(occupied+n)*4] = actual[occupied*4:(occupied+n)*4]
    for name, value in expected.items():
        require(state[name] == value, f'unexpected complete-record change: {name}')
    require(writes, 'cleanup wrote nothing')


def run(image, cases):
    regions = layout(image)
    runner = BoundedCall(regions, STOP, budget=20000)
    samples = []
    started = time.perf_counter()
    peak_instructions = 0
    for i in range(cases):
        count, shape, mask, occupied = i % 32, ('balanced', 'left', 'right')[(i//32) % 3], (i//96) % 16, (i//1536) % 4
        inputs, expected, active = fixture(count, shape, mask, occupied, i)
        result = runner.run(ENTRY, REGS, inputs)
        verify(result, expected, active, count, occupied)
        peak_instructions = max(peak_instructions, runner.instructions)
        # Retain digests, not complete scratch/write histories, across the sweep.
        samples.append(hashlib.sha256(repr(result).encode()).digest())
    reused_seconds = time.perf_counter()-started
    started = time.perf_counter()
    for i in reversed(range(cases)):
        inputs, _, _ = fixture(i % 32, ('balanced', 'left', 'right')[(i//32) % 3], (i//96) % 16, (i//1536) % 4, i)
        result = runner.run(ENTRY, REGS, inputs)
        require(hashlib.sha256(repr(result).encode()).digest() == samples[i], 'reset/order dependence')
    reverse_seconds = time.perf_counter()-started
    started = time.perf_counter()
    for i in range(cases):
        inputs, _, _ = fixture(i % 32, ('balanced', 'left', 'right')[(i//32) % 3], (i//96) % 16, (i//1536) % 4, i)
        result = BoundedCall(regions, STOP, budget=20000).run(ENTRY, REGS, inputs)
        require(hashlib.sha256(repr(result).encode()).digest() == samples[i], 'fresh mismatch')
    fresh_seconds = time.perf_counter()-started
    inputs, _, _ = fixture(31, 'left', 15, 3, 7)
    failures = []
    # Remove every data/code dependency used by the maximal fixture, individually.
    # Unused code and read-free payload storage are not credited as controls.
    for name in (r.name for r in regions):
        reduced = [r for r in regions if r.name != name]
        try:
            BoundedCall(reduced, STOP, budget=20000).run(ENTRY, REGS, {k: v for k, v in inputs.items() if k != name})
        except ValueError:
            failures.append(name)
        else:
            raise ValueError(f'missing dependency accepted: {name}')
    # Prove the complete-record and ownership checks reject corrupted native output.
    baseline = runner.run(ENTRY, REGS, inputs)
    corruptions = [('unit', 0x20, 99), ('nodes', 12, PAYLOADS),
                   ('tree0', 8, 31), ('pool6', 8, 3),
                   ('array6', 12, 0), ('array6', 0, 0),
                   ('array6', 252, 0), ('return', 0, 0)]
    rejected_outputs = []
    for name, offset, value in corruptions:
        altered = dict(baseline[1])
        changed = bytearray(altered[name])
        put(changed, offset, value)
        altered[name] = bytes(changed)
        _, expected, active = fixture(31, 'left', 15, 3, 7)
        try:
            verify((baseline[0], tuple(altered.items()), baseline[2]), expected, active, 31, 3)
        except ValueError:
            rejected_outputs.append(f'{name}+{offset}')
        else:
            raise ValueError(f'corrupted output accepted: {name}+{offset}')
    # Full pool must fail at an undeclared allocator dependency, never silently grow.
    growth_failure = {}
    for i in range(7):
        full = dict(inputs)
        full[f'pool{i}'] = words(ARRAYS+i*256, 3, 3)
        try:
            runner.run(ENTRY, REGS, full)
        except ValueError as error:
            growth_failure[f'pool{i}'] = str(error)
        else:
            raise ValueError(f'pool {i} growth escaped bounds')
        require(runner.run(ENTRY, REGS, inputs) == baseline, 'failed call poisoned reset state')
    return {'schema': 1, 'source_sha256': hashlib.sha256(image.read_bytes()).hexdigest(),
            'cases': cases, 'declared_bytes': sum(len(r.data) for r in regions),
            'mapped_bytes': runner.mapped_bytes, 'payload_bytes_mapped': 0,
            'peak_instructions': peak_instructions, 'reused_seconds': reused_seconds,
            'reverse_seconds': reverse_seconds, 'fresh_seconds': fresh_seconds,
            'missing_dependency_controls': failures, 'growth_rejected': growth_failure,
            'corrupted_output_controls': rejected_outputs,
            'scope': 'synthetic open-tree cleanup; other containers empty; no pool growth'}


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install', type=Path)
    ap.add_argument('--cases', type=int, default=6144)
    args = ap.parse_args()
    require(1 <= args.cases <= 6144, 'cases must be between 1 and 6144')
    print(json.dumps(run(args.install/'riseofnations.exe', args.cases), indent=2))


if __name__ == '__main__':
    main()
