#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Replay RON_TURN_PROBE field captures outside Wine; emit turn_oracle_check rows.

Usage: uv run tools/explore/replay_live_turn.py INSTALL TRACE > /tmp/live-turn.txt
Reconstructs normal-member fields, not an arbitrary memory snapshot. No imports
are emulated. Missing input records, mismatches, and incomplete calls fail.
"""
import argparse
import importlib.util
from pathlib import Path
import struct
import sys
import time

spec = importlib.util.spec_from_file_location('callfn', Path(__file__).resolve().parents[1] / 'emu/callfn.py')
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('install')
    ap.add_argument('trace')
    ap.add_argument('--mutations', type=int, default=0)
    args = ap.parse_args()
    assert args.mutations >= 0
    start = time.perf_counter()
    raw = Path(args.trace).read_bytes()
    assert len(raw) >= 32 and len(raw) % 32 == 0, 'incomplete trace'
    header = struct.unpack('<8I', raw[:32])
    assert header[0:3] == (0x544e4f52, 2, 0x400000), 'unexpected trace format'
    m = c.Machine(c.Image(Path(args.install) / 'riseofnations.exe'))
    guy, unit, typ, constants, slots = (0x10000000 + i * 4096 for i in range(5))
    m.uc.mem_map(guy, 5 * 4096)

    def put(address, value):
        m.uc.mem_write(address, struct.pack('<I', value & 0xffffffff))

    put(0xc0aec0, slots)
    put(0xc061e4, constants)
    put(slots, unit)
    put(unit + 0x18, typ)
    put(typ + 0x304, 1)
    pending = []
    count = skipped = 0
    unique = set()
    for r in struct.iter_unpack('<8I', raw[32:]):
        if r[0] == 5 and r[1] in (2, 5, 14):
            raise AssertionError(f'trace health failure: {r}')
        if r[0] == 7 and r[1] == 8:
            pending.append({'self': r[2], 'mode': r[3], 'frame': r[7]})
        elif r[0] == 5 and r[1] in (110, 111, 113):
            assert pending and pending[-1]['self'] == r[2], 'unpaired field record'
            assert pending[-1]['frame'] == r[7], 'field frame changed'
            assert r[1] not in pending[-1], 'duplicate fields'
            pending[-1][r[1]] = r[3:7]
        elif r[0] == 8 and r[1] == 8:
            assert pending, 'return without call'
            p = pending.pop()
            assert p['frame'] == r[7], 'return frame changed'
            if 113 in p:
                skipped += 1
                continue
            assert 110 in p and 111 in p, 'missing captured fields'
            rate, squad, last, avg = p[110]
            flags, masks, turn, bonus = p[111]
            put(typ + 0x2c4, rate)
            put(unit + 0x68, masks)
            put(constants + 8, turn)
            put(constants + 12, bonus)
            put(guy + 0x80, last)
            put(guy + 0x84, avg)
            m.uc.mem_write(guy + 0x9a, struct.pack('<H', flags))
            answer = m.call(0x5de340, (c.signed(p['mode']),), ecx=guy)
            assert answer == r[2], f'live/replay mismatch frame {r[7]}: {answer} != {r[2]}'
            fields = (rate, (masks >> 19) & 1, (flags >> 4) & 1,
                      c.signed(last), c.signed(avg), c.signed(p['mode']), turn, bonus)
            unique.add(fields)
            print(*fields, answer)
            count += 1
    assert not pending and count, 'no complete replayable calls'
    # Seeded mutation of the live scalar states. Explicit LCG keeps selection
    # reproducible without depending on Python's random implementation.
    corpus = sorted(unique)
    state = 424242
    choices = ([0, 255, 256, 0x20000000, 0x80000000, 0xffffffff],
               [0, 1], [0, 1], [0, 1, 25], [0, 3, 4, 25, 255, 1023],
               [0, 1], [1, 64, 256, 512], [1, 2, 4])
    def draw():
        nonlocal state
        state = (state * 1664525 + 1013904223) & 0xffffffff
        return state >> 16
    for _ in range(args.mutations):
        fields = list(corpus[draw() % len(corpus)])
        for _ in range(1 + draw() % 4):
            slot = draw() % 8
            fields[slot] = choices[slot][draw() % len(choices[slot])]
        rate, pack, stopped, last, avg, mode, turn, bonus = fields
        put(typ + 0x2c4, rate)
        put(unit + 0x68, pack << 19)
        put(guy + 0x80, last)
        put(guy + 0x84, avg)
        m.uc.mem_write(guy + 0x9a, struct.pack('<H', stopped << 4))
        put(constants + 8, turn)
        put(constants + 12, bonus)
        print(*fields, m.call(0x5de340, (mode,), ecx=guy))
    print(f'mutations={args.mutations} live_calls={count} unique_inputs={len(unique)} excluded_calls={skipped} '
          f'elapsed_s={time.perf_counter()-start:.6f}', file=sys.stderr)


if __name__ == '__main__':
    main()
