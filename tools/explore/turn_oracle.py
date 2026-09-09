#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""Exploration: execute a state-reading function without launching the game.

Usage: uv run tools/explore/turn_oracle.py INSTALL > /tmp/turn-oracle.txt
Output is derived oracle data and must stay outside the repository.
Layouts: local PDB type export; singleton load sites: executable listing.
This builds a normal squad member's object graph, not a whole game state.
"""
import importlib.util
import itertools
from pathlib import Path
import struct
import sys
import time

source = Path(__file__).resolve().parents[1] / 'emu/callfn.py'
spec = importlib.util.spec_from_file_location('callfn', source)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
from unicorn import UC_HOOK_BLOCK, UC_HOOK_MEM_WRITE

start = time.perf_counter()
m = module.Machine(module.Image(Path(sys.argv[1]) / 'riseofnations.exe'))
# Chosen fixture allocations. The four objects occupy separate pages.
guy, unit, typ, constants, slots = (0x10000000 + i * 0x1000 for i in range(5))
m.uc.mem_map(guy, 0x5000)
def put(address, value):
    m.uc.mem_write(address, struct.pack('<I', value & 0xffffffff))

# Absolute operands confirmed by llvm-objdump at the function's load sites.
put(0xc0aec0, slots)  # owner zero's unit pointer array
put(0xc061e4, constants)
put(slots, unit)  # object zero
put(unit + 0x18, typ)
put(typ + 0x304, 1)  # guy zero is inside the squad
# Guy.who, .o, .guy_num are zero in the fresh fixture.
blocks, writes = set(), set()
m.uc.hook_add(UC_HOOK_BLOCK, lambda uc, addr, size, _: blocks.add(addr))
m.uc.hook_add(UC_HOOK_MEM_WRITE, lambda uc, access, addr, size, value, _: writes.add(addr))
count = 0
# Cross the scale/truncation and stop/pack/mode boundaries, including signed
# type angles. Negative average speed is outside this experiment's domain.
for rate, pack, stopped, last, avg, mode, turn, bonus in itertools.product(
    [0, 255, 256, 0x20000000, 0x80000000, 0xffffffff],
    [0, 1], [0, 1], [0, 25], [0, 3, 4, 25, 255], [0, 1], [1, 256], [2]
):
    put(typ + 0x2c4, rate)
    put(unit + 0x68, pack << 19)
    put(constants + 8, turn)
    put(constants + 12, bonus)
    put(guy + 0x80, last)
    put(guy + 0x84, avg)
    m.uc.mem_write(guy + 0x9a, struct.pack('<H', stopped << 4))
    result = m.call(0x5de340, (mode,), ecx=guy)
    print(rate, pack, stopped, last, avg, mode, turn, bonus, result)
    count += 1
outside_stack = [a for a in writes if not module.STACK_BASE <= a < module.STACK_BASE + module.STACK_SIZE]
print(f'calls={count} elapsed_s={time.perf_counter()-start:.6f} '
      f'blocks={len(blocks)} nonstack_write_addresses={len(outside_stack)}', file=sys.stderr)
