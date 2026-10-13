#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_build_queue_data_get_queue.py — `BuildQueueData::get_queue@00630670` under unicorn (item 1819).

    uv run tools/emu/sweep_build_queue_data_get_queue.py <install>/riseofnations.exe

`__thiscall`, `ecx` = the queue data, one stack argument: `queue_size` at +0, the entries behind +4 (20 bytes each); for an index below the size the sign-extended word at entry+4, else -1 — negative indices included, read from the entry before the array. The entry at the index carries the word, every other the word 0x0101.
One row per input, `<args…> -> <result>`.
"""
import os
import random
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from callfn import Image, Machine, signed  # noqa: E402

ENTRY = 0x00630670
OBJ, ARR = 0x10000000, 0x10100000
EDGES = (0, 1, -1, 2, -2, 0x7FFFFFFF, -0x80000000, 0x100, -0x100, 15, 16, 255, 0x8000, -0x8000)


def main(argv):
    m = Machine(Image(argv[1]))
    uc = m.uc
    uc.mem_map(OBJ, 0x20000)
    uc.mem_map(ARR, 0x20000)
    Q = ARR + 0x1000
    for size in (0, 1, 2, 5, 6, -1, 0x7FFFFFFF, -0x80000000):
        for idx in range(-3, 7):
            for v in (0, 1, 0x7FFF, 0x8000, 0xFFFF, 0x1234, 0xFFFE):
                uc.mem_write(OBJ, struct.pack("<i", size))
                uc.mem_write(OBJ + 4, struct.pack("<I", Q))
                for i in range(-3, 8):
                    w = v if i == idx else 0x0101
                    uc.mem_write(Q + i * 20 + 4, struct.pack("<H", w))
                r = signed(m.call(ENTRY, (idx,), ecx=OBJ))
                print(f"{size} {idx} {v} -> {r}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
