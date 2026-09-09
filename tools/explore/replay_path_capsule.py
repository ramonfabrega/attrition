#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Natural path-segment deletion: bounded original replay and semantic Rust rows.

Reports/rows contain original-derived data and must be redirected outside git.
Scratch is initialized-on-write and compared across emulators, not against live.
"""
import argparse
import json
import random
from pathlib import Path
import struct
from bounded_call import BoundedCall, Region
from replay_capsule import require, digest, image_bytes, read_trace

ENTRY, CLEAR, TAIL = 0x5e31d0, 0x5e3920, 0x5e3bc0

def word(n):
    return struct.pack('<I', n)


class PathCapsule:
    def __init__(self, raw):
        require(len(raw) == 2296, 'truncated or oversized path capsule')
        h = struct.unpack_from('<27I', raw)
        magic, version, self.entry, self.stop, self.frame, self.self_, self.path, self.count, self.caller = h[:9]
        require((magic, version, self.entry) == (0x31504350, 1, ENTRY), 'unsupported path capsule')
        require(0 <= self.frame <= 240 and 1 <= self.count <= 64, 'invalid capture domain')
        self.before_regs, self.after_regs = h[9:18], h[18:27]
        require(self.before_regs[6] == self.self_, 'self differs from ECX')
        esp = self.before_regs[3]
        require(esp % 4 == 0 and self.after_regs[3] == esp+4, 'invalid stack effect')
        self.code = raw[108:188], raw[188:208], raw[208:212]
        self.before, self.after = raw[212:224], raw[224:236]
        ob, oa = struct.unpack_from('<2I', raw, 236)
        self.path_before, self.path_after = raw[244:244+self.count*16], raw[1268:1268+self.count*16]
        stack_return = struct.unpack_from('<I', raw, 2292)[0]
        require(stack_return == self.stop, 'invalid stop slot')
        require(ob == oa == 0, 'suspended search outside contract')
        require(struct.unpack_from('<I', self.before)[0] == self.path and self.path != 0, 'broken path pointer')
        require(struct.unpack_from('<I', self.before, 8)[0] == self.count, 'inconsistent input length')
        self.remaining = struct.unpack_from('<I', self.after, 8)[0]
        require(self.remaining < self.count, 'no observed path deletion')
        self.regions = [Region('code', ENTRY, self.code[0], executable=True),
            Region('clear_head', CLEAR, self.code[1][:17], executable=True),
            Region('clear_tail', TAIL, self.code[2][:2], executable=True),
            Region('path_header', self.self_+0xb8, self.before[:8]),
            Region('length', self.self_+0xc0, self.before[8:], writable=True),
            Region('openlist', self.self_+0x104, word(ob)),
            Region('path', self.path, self.path_before),
            Region('return', esp, word(stack_return)),
            Region('scratch', esp-16, bytes(16), writable=True, scratch=True)]
        self.expected = {'path_header': self.after[:8], 'length': self.after[8:],
                         'openlist': word(oa), 'path': self.path_after, 'return': word(stack_return)}

    def run(self):
        runner = BoundedCall(self.regions, self.stop, budget=1024)
        result = runner.run(self.entry, self.before_regs)
        require(result[0] == self.after_regs, 'live register/flags mismatch')
        memory = dict(result[1]); memory.pop('scratch')
        require(memory == self.expected, 'live semantic memory mismatch')
        # Every semantic store decrements length; temporary stack stores have
        # separate byte bounds and write-before-read enforcement.
        writes = tuple(w for w in result[2] if w[0] == self.self_+0xc0)
        require(writes == tuple((self.self_+0xc0, 4, n) for n in range(self.count-1, self.remaining-1, -1)),
                'unexpected path-length store sequence')
        require(runner.run(self.entry, self.before_regs) == result, 'reset mismatch')
        require(BoundedCall(self.regions, self.stop, budget=1024).run(self.entry, self.before_regs) == result, 'fresh mismatch')
        controls = ('code', 'clear_head', 'clear_tail', 'path_header', 'length', 'openlist', 'path', 'return', 'scratch')
        for name in controls:
            try:
                BoundedCall([r for r in self.regions if r.name != name], self.stop, budget=1024).run(self.entry, self.before_regs)
            except ValueError:
                pass
            else:
                raise ValueError(f'missing dependency passed: {name}')
        return {'frame': self.frame, 'before_count': self.count, 'after_count': self.remaining,
                'instructions': runner.instructions, 'declared_bytes': sum(len(r.data) for r in self.regions),
                'mapped_bytes': runner.mapped_bytes, 'fresh_replays': 2, 'reset_replays': 1,
                'missing_dependency_controls': len(controls), 'semantic_writes': writes,
                'live_scratch_compared': False}

    def row(self):
        return semantic_row(self.count, self.remaining, self.path_before)

    def mutations(self, count):
        require(count >= 0, 'negative mutation count')
        rng = random.Random(12345)
        runner = BoundedCall(self.regions, self.stop, budget=1024)
        rows, retained = [], 0
        for i in range(count):
            length = i % (self.count+1)
            data = bytearray(rng.randbytes(len(self.path_before)))
            for j in range(self.count):
                data[j*16+12] = (data[j*16+12] & 254) | ((i//(self.count+1) >> j) & 1)
            inputs = {'length': word(length), 'path': bytes(data)}
            result = runner.run(self.entry, self.before_regs, inputs)
            require(BoundedCall(self.regions, self.stop, budget=1024).run(self.entry, self.before_regs, inputs) == result,
                    f'mutation fresh mismatch: {i}')
            memory = dict(result[1]); remaining = struct.unpack('<I', memory['length'])[0]
            require(remaining < length if length else remaining == 0, 'invalid mutated length')
            require(all(memory[name] == expected for name, expected in self.expected.items() if name not in ('path', 'length')),
                    'mutation changed unrelated state')
            require(memory['path'] == data, 'waypoint bytes changed')
            require(all(result[0][r] == self.before_regs[r] for r in (0, 1, 2, 4, 6)) and
                    result[0][3] == self.before_regs[3]+4, 'mutation violated ABI')
            writes = tuple(w for w in result[2] if w[0] == self.self_+0xc0)
            require(writes == tuple((self.self_+0xc0, 4, n) for n in range(length-1, remaining-1, -1)),
                    'unexpected mutated store sequence')
            rows.append(semantic_row(length, remaining, data[:length*16]))
            retained += remaining != 0
        return rows, {'synthetic_mutations': count, 'fresh_mutation_checks': count,
                      'mutations_retaining_waypoints': retained, 'seed': 12345}


def semantic_row(count, remaining, data):
    fields = [count, remaining]
    for x, y, tolerance, flags in struct.iter_unpack('<4I', data):
        fields.extend((x, y, tolerance, flags & 255))
    return ' '.join(map(str, fields))


def replay(install, directory):
    identity = json.loads((directory/'capsule-image.json').read_text())
    require(identity['schema'] == 1, 'unsupported image identity')
    for name, path in [('source', install/'riseofnations.exe'), ('staged_source', directory/'riseofnations.exe'),
                       ('traced', directory/'riseofnations_trace.exe'), ('tracer', directory/'rontrace.dll')]:
        require(digest(path) == identity['sha256'][name], f'image identity mismatch: {name}')
    c = PathCapsule((directory/'path-capsule.bin').read_bytes())
    for address, code in zip((ENTRY, CLEAR, TAIL), c.code):
        require(image_bytes(install/'riseofnations.exe', address, len(code)) == code, 'source code mismatch')
    records = read_trace(directory/'rontrace.log')
    require(not any(r[:2] == (5, 127) for r in records), 'path hook failed')
    receipts = [r for r in records if r[:2] == (5, 126)]
    require(len(receipts) == 1 and receipts[0][2:] == (0, c.self_, c.frame, 2296, 2296, c.frame), 'invalid capture receipt')
    return c, {'schema': 1, 'source_sha256': identity['sha256']['source'],
               'capsule_sha256': digest(directory/'path-capsule.bin'), **c.run()}


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install', type=Path); ap.add_argument('directory', type=Path)
    ap.add_argument('--oracle-rows', action='store_true')
    ap.add_argument('--mutations', type=int, default=0)
    args = ap.parse_args()
    capsule, report = replay(args.install, args.directory)
    rows, summary = capsule.mutations(args.mutations)
    print('\n'.join([capsule.row(), *rows]) if args.oracle_rows else json.dumps({**report, **summary}, indent=2))
