#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["unicorn==2.1.4"]
# ///
"""sweep_unit_data_is_fleeing.py — `UnitData::is_fleeing@0046efa0` under unicorn (item 2090): the layout and the
rows are `sweep_unit_data_is_idle.py`'s, run on the other entry.

    uv run tools/emu/sweep_unit_data_is_fleeing.py <install>/riseofnations.exe
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sweep_unit_data_is_idle import main  # noqa: E402

if __name__ == "__main__":
    sys.exit(main(sys.argv, fleeing=True))
