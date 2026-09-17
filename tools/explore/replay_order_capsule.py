#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Replay a natural Unit::update_order witness; output reports outside git.

First bind the staged images with replay_capsule.py bind INSTALL DIRECTORY.
This validates original memory effects, not Rust's different order layout.
"""
import argparse
import json
import random
from pathlib import Path
import struct
from bounded_call import BoundedCall, Region
from replay_capsule import require, digest, image_bytes, read_trace

ENTRY = 0x6179d0

def word(n):
    return struct.pack('<I', n)


class OrderCapsule:
    def __init__(self, raw):
        require(len(raw) == 240, 'truncated or oversized order capsule')
        h = struct.unpack_from('<27I', raw)
        magic, version, self.entry, self.stop, self.frame, self.self_, self.head, self.node, self.caller = h[:9]
        require((magic, version, self.entry) == (0x314f4352, 1, ENTRY), 'unsupported order capsule')
        self.before_regs, self.after_regs = h[9:18], h[18:27]
        self.code = raw[108:172]
        before, after = raw[172:192], raw[192:212]
        hb, ha = struct.unpack_from('<2I', raw, 212)
        nb, na = raw[220:228], raw[228:236]
        stack_return = struct.unpack_from('<I', raw, 236)[0]
        require(0 <= self.frame <= 35, 'outside capture window')
        require(self.before_regs[6] == self.self_, 'self differs from ECX')
        require(struct.unpack_from('<I', before, 16)[0] == self.head and hb == self.node, 'broken pointer graph')
        require(self.head != 0 and self.node != 0, 'empty graph')
        require(stack_return == self.stop, 'stop differs from return slot')
        require(self.after_regs[3] == self.before_regs[3]+4, 'unexpected stack effect')
        self.changed_bytes = sum(a != b for a, b in zip(before, after))
        # Split the contiguous unit snapshot so permission is exact per field.
        self.regions = [Region('code', ENTRY, self.code[:62], executable=True)]
        self.expected = {}
        for name, off, size, writable in [('order', 0, 4, True), ('metric', 4, 1, True),
                ('padding', 5, 3, False), ('node', 8, 4, True),
                ('untouched', 12, 4, False), ('head', 16, 4, False)]:
            self.regions.append(Region(name, self.self_+0xcc+off, before[off:off+size], writable=writable))
            self.expected[name] = after[off:off+size]
        for name, address, b, a in [('previous', self.head+4, word(hb), word(ha)),
                ('node_data', self.node+8, nb, na),
                ('return', self.before_regs[3], word(stack_return), word(stack_return))]:
            self.regions.append(Region(name, address, b)); self.expected[name] = a
        self.expected_writes = ((self.self_+0xd4, 4, self.node),
            (self.self_+0xcc, 4, struct.unpack_from('<I', nb)[0]), (self.self_+0xd0, 1, nb[4]))

    def verify(self):
        a = BoundedCall(self.regions, self.stop)
        result = a.run(self.entry, self.before_regs)
        require(result[0] == self.after_regs, 'live register/flags mismatch')
        require(dict(result[1]) == self.expected, 'live memory mismatch')
        require(result[2] == self.expected_writes, 'live write contract mismatch')
        require(a.run(self.entry, self.before_regs) == result, 'reset replay mismatch')
        require(BoundedCall(self.regions, self.stop).run(self.entry, self.before_regs) == result, 'fresh replay mismatch')
        # Remove each input needed for this path, including same-page fields.
        controls = ('code', 'head', 'previous', 'node_data', 'return')
        for name in controls:
            try:
                BoundedCall([r for r in self.regions if r.name != name], self.stop).run(self.entry, self.before_regs)
            except ValueError:
                pass
            else:
                raise ValueError(f'missing dependency passed: {name}')
        return {'changed_unit_bytes': self.changed_bytes, 'frame': self.frame, 'self': self.self_, 'caller': self.caller,
                'declared_bytes': sum(len(r.data) for r in self.regions),
                'mapped_bytes': a.mapped_bytes, 'instructions': a.instructions,
                'fresh_replays': 2, 'reset_replays': 1, 'missing_dependency_controls': len(controls),
                'writes': result[2], 'before_cache': {r.name: r.data.hex() for r in self.regions if r.writable},
                'after_cache': {r.name: self.expected[r.name].hex() for r in self.regions if r.writable}}


def fuzz_cache(c, count):
    """Poison only stale cached outputs; authoritative list data stays captured.

    This is live-derived synthetic mutation, not additional natural calls.
    Every result must restore the complete live exit state and store sequence.
    """
    require(count > 0, 'cache mutation count must be positive')
    rng = random.Random(12345)
    runner = BoundedCall(c.regions, c.stop)
    baseline = runner.run(c.entry, c.before_regs)
    for i in range(count):
        inputs = {}
        for r in c.regions:
            if r.writable:
                value = 0 if i == 0 else ((1 << (8*len(r.data)))-1 if i == 1 else rng.getrandbits(8*len(r.data)))
                inputs[r.name] = value.to_bytes(len(r.data), 'little')
        actual = runner.run(c.entry, c.before_regs, inputs)
        require(actual == baseline, f'stale cache affected result: mutation {i}')
        require(BoundedCall(c.regions, c.stop).run(c.entry, c.before_regs, inputs) == actual,
                f'cache mutation fresh mismatch: {i}')
    return {'synthetic_cache_mutations': count, 'fresh_mutation_checks': count, 'seed': 12345}


def replay(install, directory, mutations=0):
    require(mutations >= 0, 'negative mutation count')
    identity = json.loads((directory/'capsule-image.json').read_text())
    require(identity['schema'] == 1, 'unsupported image identity')
    for name, path in [('source', install/'riseofnations.exe'), ('staged_source', directory/'riseofnations.exe'),
                       ('traced', directory/'riseofnations_trace.exe'), ('tracer', directory/'rontrace.dll')]:
        require(digest(path) == identity['sha256'][name], f'image identity mismatch: {name}')
    c = OrderCapsule((directory/'order-capsule.bin').read_bytes())
    require(c.code == image_bytes(install/'riseofnations.exe', ENTRY, 64), 'captured code differs from source')
    records = read_trace(directory/'rontrace.log')
    require(not any(r[:2] == (5, 125) for r in records), 'order hook failure')
    receipts = [r for r in records if r[:2] == (5, 124)]
    require(len(receipts) == 1 and receipts[0][2:] == (0, c.self_, c.frame, 240, 240, c.frame), 'invalid capture receipt')
    return {'schema': 1, 'source_sha256': identity['sha256']['source'],
            'capsule_sha256': digest(directory/'order-capsule.bin'), **c.verify(),
            **(fuzz_cache(c, mutations) if mutations else {})}


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install', type=Path); ap.add_argument('directory', type=Path)
    ap.add_argument('--mutations', type=int, default=0)
    args = ap.parse_args()
    print(json.dumps(replay(args.install, args.directory, args.mutations), indent=2))
