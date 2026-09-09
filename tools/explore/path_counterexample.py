"""Semantic path cases, bounded worker IPC, and deterministic reduction.

No game files or emulator dependency. Cases are tuples of (x, y, tolerance,
flags) unsigned words. Minimal means the tested deletion/bit-clear neighborhood,
not a globally smallest example.
"""
import math
import os
from pathlib import Path
import selectors
import select
import subprocess
import tempfile
import time


class OracleFailure(RuntimeError):
    pass


def validate(case):
    if len(case) > 64 or any(len(row) != 4 or any(type(v) is not int or not 0 <= v <= 0xffffffff for v in row)
                            or row[3] > 255 for row in case):
        raise ValueError('invalid semantic path case')


def encode(case):
    validate(case)
    return ' '.join(map(str, [len(case), *(v for row in case for v in row)]))+'\n'


def decode(line):
    try:
        values = [int(v) for v in line.split()]
        if not values or not 0 <= values[0] <= 64 or len(values) != 1+4*values[0]:
            raise ValueError('invalid row count')
        case = tuple(tuple(values[i:i+4]) for i in range(1, len(values), 4))
        validate(case)
        return case
    except ValueError as error:
        raise OracleFailure(f'malformed worker output: {error}') from error


class Worker:
    def __init__(self, executable, timeout=5):
        self.stderr = tempfile.TemporaryFile()
        try:
            self.process = subprocess.Popen([str(Path(executable).resolve(strict=True))],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=self.stderr, bufsize=0)
        except Exception:
            self.stderr.close()
            raise
        os.set_blocking(self.process.stdin.fileno(), False)
        os.set_blocking(self.process.stdout.fileno(), False)
        self.timeout = timeout
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.calls = 0

    def evaluate(self, case):
        message = encode(case).encode('ascii')
        end = time.monotonic()+self.timeout
        sent = 0
        while sent < len(message):
            if not select.select([], [self.process.stdin], [], max(0, end-time.monotonic()))[1]:
                raise OracleFailure('worker input timeout')
            try:
                sent += os.write(self.process.stdin.fileno(), message[sent:])
            except BlockingIOError:
                continue
            except OSError as error:
                raise OracleFailure('worker input pipe failed') from error
        response = b''
        while b'\n' not in response:
            if not self.selector.select(max(0, end-time.monotonic())):
                raise OracleFailure('worker response timeout')
            try:
                chunk = os.read(self.process.stdout.fileno(), 8192)
            except BlockingIOError:
                continue
            if not chunk:
                raise OracleFailure('worker exited without a complete response')
            response += chunk
            if len(response) > 8192:
                raise OracleFailure('oversized worker response')
        line, extra = response.split(b'\n', 1)
        if extra:
            raise OracleFailure('unsolicited worker output')
        try:
            result = decode(line.decode('ascii'))
        except UnicodeError as error:
            raise OracleFailure('non-ASCII worker output') from error
        self.calls += 1
        return result

    def close(self):
        self.selector.close()
        self.process.stdin.close()
        try:
            self.process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        self.process.stdout.close()
        self.stderr.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()


def mismatch(original, rust):
    if len(original) != len(rust):
        return 'survivor_count'
    for a, b in zip(original, rust):
        for name, x, y in zip(('x', 'y', 'tolerance', 'flags'), a, b):
            if x != y:
                return name
    return None


def neighbors(case):
    """Every single-waypoint deletion, whole-field zero, and set-bit clear."""
    for i in range(len(case)):
        yield case[:i]+case[i+1:]
    seen = set()
    for i, row in enumerate(case):
        for j, value in enumerate(row):
            if not value:
                continue
            for smaller in [0, *(value & ~(1 << b) for b in reversed(range(value.bit_length())) if value & (1 << b))]:
                r = list(row); r[j] = smaller
                candidate = case[:i]+(tuple(r),)+case[i+1:]
                if candidate not in seen:
                    seen.add(candidate)
                    yield candidate


def reduce_case(case, fails, budget=10000):
    validate(case)
    cache = {}
    def check(candidate):
        if candidate not in cache:
            if len(cache) >= budget:
                raise OracleFailure('reduction evaluation budget exhausted')
            # Exceptions deliberately propagate. A broken oracle is not a mismatch.
            result = fails(candidate)
            if type(result) is not bool:
                raise TypeError('failure predicate must return bool')
            cache[candidate] = result
        return cache[candidate]
    if not check(case):
        raise ValueError('starting case does not preserve the requested mismatch')
    original = case
    while True:
        start = case
        partitions = 2
        while len(case) >= 2:
            width = math.ceil(len(case)/partitions)
            for at in range(0, len(case), width):
                candidate = case[:at]+case[at+width:]
                if check(candidate):
                    case = candidate
                    partitions = max(2, partitions-1)
                    break
            else:
                if partitions >= len(case):
                    break
                partitions = min(len(case), partitions*2)
        for candidate in neighbors(case):
            if check(candidate):
                case = candidate
                break
        if case == start:
            break
    certified = tuple(neighbors(case))
    if any(check(candidate) for candidate in certified):
        raise OracleFailure('reduction did not reach a fixed point')
    return case, {'evaluations': len(cache), 'certified_neighbors': len(certified),
                  'initial_waypoints': len(original), 'final_waypoints': len(case),
                  'initial_set_bits': sum(v.bit_count() for row in original for v in row),
                  'final_set_bits': sum(v.bit_count() for row in case for v in row),
                  'minimality': 'single waypoint deletion, field zero, set-bit clear'}
