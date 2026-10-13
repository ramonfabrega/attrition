#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_wall_data_is_defensive.py — `WallData::is_defensive@00473650` under unicorn (item 2083, hand-written on
tools/emu/hooks.py through `sweep_build_type_data_is_defensive.py`'s `run`).

    uv run tools/emu/sweep_wall_data_is_defensive.py <install>/riseofnations.exe

`__thiscall` on a wall, no stack arguments, plain `ret`: the type-record body of `BuildTypeData::is_defensive`
inlined on the record at `this+0x18` — the three `is` asks go to the record's slot `+0x60` with `ecx` the record
(a call's `this` below is the record), and `attack` is the record's `+0x1e8`. Both objects' bytes are random and
held unwritten. Rows as the type record's sweep.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sweep_build_type_data_is_defensive import run  # noqa: E402

if __name__ == "__main__":
    sys.exit(run(sys.argv, 0x00473650, 0x18, 2085))
