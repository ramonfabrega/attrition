#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_farms_grow.py — `Farms::grow@008d91c0` under unicorn, one row per input.

    uv run tools/emu/sweep_farms_grow.py <install>/riseofnations.exe [SEED] [N]

`__thiscall Farms::grow(this, farm, dy, dx)`, `ret 0xc`: it reads the farm
table through the global `FarmStruct *` at `0x00c0a914` (`FarmsData+0x14`'s
`Array::list`), stride `0xc0`, writing `status[dx][dy]` (`+0xac`) and
`percent[dx][dy]` (`+0x8`, a float). The script points that global at a page
it maps, seeds one cell, calls, and reads the cell back.

A row is

    <farm> <dy> <dx> <state0> <n0> -> <state1 * 2^32 + percent1's f32 bits>

where `(state0, n0)` is the cell before the call in `crates/sim/src/farms.rs`'
own representation — `n0` adds of `0.005f` from zero for an empty or growing
cell, the exact `1.0f` for a ripe one (`n0` = 200), and `1.0f` less
`(200 − n0) / 2` subtractions of `0.01f` for a cut one — and the float the
original is seeded with is built from it here, step by step in single
precision.

The comparisons the function makes are `dy < 4` and `dx < 4` (signed) and
`1.0f < percent + 0.005f`; a row with `dy` or `dx` at or past 4 prints the
count of bytes of the farm table the call changed (the guard says 0).
"""
import random
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from callfn import Image, Machine  # noqa: E402

GROW = 0x008D91C0
FARMS_LIST = 0x00C0A914
TABLE, FARMS, STRIDE = 0x1000_0000, 64, 0xC0
PERCENT, STATUS = 0x8, 0xAC
EMPTY, GROWING, RIPE, CUT = 0, 1, 2, 3


def f32(x):
    return struct.unpack("<f", struct.pack("<f", x))[0]


ADD, SUB = f32(0.005), f32(0.01)


def seed_float(state, n):
    """The cell's float for the port's `(state, adds)`, built as the
    original builds it: one single-precision operation at a time."""
    if state == RIPE:
        return 1.0
    if state == CUT:
        v = 1.0
        for _ in range((200 - n) // 2):
            v = f32(v - SUB)
        return v
    v = 0.0
    for _ in range(n):
        v = f32(v + ADD)
    return v


def inputs(seed, n):
    rng = random.Random(seed)
    rows = []
    # Every growing count the port can hold, on a walk of cells and farms.
    for k in range(201):
        rows.append((k % 7, k % 4, (k // 4) % 4, GROWING, k))
    rows.append((0, 0, 0, EMPTY, 0))
    for dy in range(4):
        for dx in range(4):
            rows.append((3, dy, dx, EMPTY, 0))
            rows.append((5, dy, dx, RIPE, 200))
    # Every cut cell `inc_time` keeps (k = 0..100 subtractions).
    for k in range(101):
        rows.append((k % 11, (k + 1) % 4, (k + 2) % 4, CUT, 200 - 2 * k))
    # The guard's edges.
    for dy, dx in [(3, 3), (4, 0), (0, 4), (4, 4), (5, 2), (2, 7), (100, 1), (1, 0x7FFF_FFFF)]:
        rows.append((1, dy, dx, GROWING, 40))
    # A seeded random set over the reachable states.
    for _ in range(n):
        st = rng.choice([EMPTY, GROWING, GROWING, RIPE, CUT])
        n0 = {EMPTY: 0, GROWING: rng.randint(0, 200), RIPE: 200, CUT: 200 - 2 * rng.randint(0, 100)}[st]
        rows.append((rng.randint(0, FARMS - 1), rng.randint(0, 3), rng.randint(0, 3), st, n0))
    return rows


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    seed = int(argv[2]) if len(argv) > 2 else 1575
    n = int(argv[3]) if len(argv) > 3 else 300
    m = Machine(Image(argv[1]))
    size = FARMS * STRIDE
    m.uc.mem_map(TABLE, (size + 0xFFF) & ~0xFFF)
    m.uc.mem_write(FARMS_LIST, struct.pack("<I", TABLE))
    rng = random.Random(seed ^ 0x5EED)
    for farm, dy, dx, st, n0 in inputs(seed, n):
        # Noise everywhere, so a stray write shows.
        before = bytes(rng.getrandbits(8) for _ in range(size))
        m.uc.mem_write(TABLE, before)
        inside = dy < 4 and dx < 4
        rec = TABLE + farm * STRIDE
        if inside:
            m.uc.mem_write(rec + STATUS + dx * 4 + dy, bytes([st]))
            m.uc.mem_write(rec + PERCENT + (dx * 4 + dy) * 4, struct.pack("<f", seed_float(st, n0)))
            before = bytes(m.uc.mem_read(TABLE, size))
        m.call(GROW, (farm, dy, dx), ecx=TABLE)
        after = bytes(m.uc.mem_read(TABLE, size))
        changed = [i for i in range(size) if before[i] != after[i]]
        if not inside:
            print(f"{farm} {dy} {dx} {st} {n0} -> {len(changed)}")
            continue
        s_off = farm * STRIDE + STATUS + dx * 4 + dy
        p_off = farm * STRIDE + PERCENT + (dx * 4 + dy) * 4
        stray = [i for i in changed if i != s_off and not p_off <= i < p_off + 4]
        assert not stray, f"stray writes at {stray[:4]} for {(farm, dy, dx)}"
        state1 = after[s_off]
        bits1 = struct.unpack_from("<I", after, p_off)[0]
        print(f"{farm} {dy} {dx} {st} {n0} -> {(state1 << 32) | bits1}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
