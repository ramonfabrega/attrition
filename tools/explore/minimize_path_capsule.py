#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Find/reduce a semantic path mismatch against a persistent production Rust worker.

The bound natural capsule remains outside git. Output is CREATE_NEW outside the
repository. Infrastructure failures abort; only same-kind semantic differences
are minimized. No source code or executable bytes are exported.
"""
import argparse
import json
from pathlib import Path
import random
import struct
import time
from bounded_call import BoundedCall
from replay_capsule import require, digest
from replay_path_capsule import replay, word
from path_counterexample import Worker, validate, mismatch, reduce_case, neighbors


class NativeOracle:
    def __init__(self, capsule):
        self.c = capsule
        self.runner = BoundedCall(capsule.regions, capsule.stop, budget=1024)
        self.calls = 0
        self.executions = 0

    def evaluate(self, case, fresh=False):
        validate(case)
        c = self.c
        require(len(case) <= c.count, 'case exceeds captured capacity')
        data = b''.join(struct.pack('<3IB3x', *row) for row in case)
        data += bytes(len(c.path_before)-len(data))
        inputs = {'length': word(len(case)), 'path': data}
        result = self.runner.run(c.entry, c.before_regs, inputs)
        self.calls += 1
        self.executions += 1
        memory = dict(result[1]); remaining = struct.unpack('<I', memory['length'])[0]
        require(0 <= remaining <= len(case), 'invalid native survivor count')
        require(all(memory[name] == value for name, value in c.expected.items() if name not in ('path', 'length')),
                'native changed unrelated state')
        require(memory['path'] == data, 'native changed waypoint bytes')
        require(all(result[0][r] == c.before_regs[r] for r in (0, 1, 2, 4, 6)) and
                result[0][3] == c.before_regs[3]+4, 'native violated ABI')
        writes = tuple(w for w in result[2] if w[0] == c.self_+0xc0)
        require(writes == tuple((c.self_+0xc0, 4, n) for n in range(len(case)-1, remaining-1, -1)),
                'native violated write contract')
        if fresh:
            self.executions += 1
            require(BoundedCall(c.regions, c.stop, budget=1024).run(c.entry, c.before_regs, inputs) == result,
                    'fresh native result differs')
        return case[:remaining]


def candidates(capacity, count):
    rng = random.Random(12345)
    for i in range(count):
        length = i % (capacity+1)
        yield tuple((rng.getrandbits(32), rng.getrandbits(32), rng.getrandbits(32),
                     (rng.randrange(256) & 254) | ((i//(capacity+1) >> j) & 1)) for j in range(length))


def run(install, capture, executable, count, budget):
    require(count > 0 and budget > 0, 'positive search and reduction budgets required')
    c, identity = replay(install, capture)
    native = NativeOracle(c)
    worker_hash = digest(executable)
    report = {'schema': 1, 'source_sha256': identity['source_sha256'],
              'capsule_sha256': identity['capsule_sha256'], 'worker_sha256': worker_hash,
              'seed': 12345, 'capacity': c.count}
    with Worker(executable) as rust:
        for i, case in enumerate(candidates(c.count, count)):
            a, b = native.evaluate(case), rust.evaluate(case)
            kind = mismatch(a, b)
            if kind is None:
                continue
            initial = case
            def fails(candidate):
                return mismatch(native.evaluate(candidate), rust.evaluate(candidate)) == kind
            reduced, stats = reduce_case(case, fails, budget)
            a, b = native.evaluate(reduced, fresh=True), rust.evaluate(reduced)
            require(mismatch(a, b) == kind, 'final mismatch changed kind')
            # Every final certification query gets its own Rust process and
            # a fresh original engine. No prior query can mask a neighbor.
            fresh_requests = 0
            with Worker(executable) as fresh_rust:
                require(fresh_rust.evaluate(reduced) == b, 'fresh Rust result differs')
                fresh_requests += fresh_rust.calls
            certified = 0
            for candidate in neighbors(reduced):
                with Worker(executable) as fresh_rust:
                    require(mismatch(native.evaluate(candidate, fresh=True), fresh_rust.evaluate(candidate)) != kind,
                            'fresh minimality certificate failed')
                    fresh_requests += fresh_rust.calls
                certified += 1
            require(digest(executable) == worker_hash, 'worker executable changed during search')
            return {**report, 'status': 'mismatch', 'searched': i+1, 'kind': kind,
                    'initial': initial, 'minimal': reduced, 'original_result': a, 'rust_result': b,
                    'reduction': stats, 'fresh_certified_neighbors': certified,
                    'native_queries': native.calls, 'search_native_executions': native.executions,
                    'worker_requests': rust.calls+fresh_requests, 'worker_processes': 2+certified}
    require(digest(executable) == worker_hash, 'worker executable changed during search')
    return {**report, 'status': 'no_mismatch', 'searched': count, 'native_queries': native.calls,
            'search_native_executions': native.executions, 'worker_requests': rust.calls, 'worker_processes': 1}


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install', type=Path); ap.add_argument('capture', type=Path)
    ap.add_argument('--worker', type=Path, required=True)
    ap.add_argument('--cases', type=int, default=4096)
    ap.add_argument('--budget', type=int, default=10000)
    ap.add_argument('--output', type=Path, required=True)
    args = ap.parse_args()
    output = args.output.resolve()
    require(not output.is_relative_to(Path(__file__).resolve().parents[2]), 'reports must stay outside the repository')
    require(not output.exists(), 'output exists')
    started = time.perf_counter()
    report = run(args.install, args.capture, args.worker, args.cases, args.budget)
    report['elapsed_seconds'] = time.perf_counter()-started
    with output.open('x') as file:
        json.dump(report, file, indent=2); file.write('\n')
    print(json.dumps({'status': report['status'], 'searched': report['searched'], 'output': str(output)}))
