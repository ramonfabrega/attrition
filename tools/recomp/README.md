# tools/recomp — original functions as native C

A static recompiler for functions of `riseofnations.exe`, one C function per
original function, verified against the emulator rung (`tools/emu/callfn.py`)
row for row. The findings and the measurements are in
`docs/lab/RECOMP-REVIEW.md`. Everything here takes an install path and
generates under `target/recomp/`; nothing from the install is committed.

    uv run tools/recomp/lift.py     <install>/riseofnations.exe 0046cff0 00688310
    uv run tools/recomp/difftest.py <install>/riseofnations.exe diff
    uv run tools/recomp/scan.py     <install>/riseofnations.exe --docs docs

- `lift.py` — decodes each function (capstone, declared inline) and every
  function it calls directly, emits `lifted.c` against `rt/recomp.h`, and
  builds `librecomp.dylib` with clang. An instruction outside its integer
  subset stops it and names itself.
- `difftest.py` — `callfn.py`'s own sweep through unicorn and through the
  native build; prints every disagreeing row and `rows= agree= disagree=`.
  `sweep` prints the native table alone in `callfn.py`'s format.
- `scan.py` — lifts and compiles every function a set of documents cites,
  and groups the failures by the instruction that stopped them.
- `image.py` — the PE's sections, imports and function table (the linker
  map; the PDB on request).
- `rt/recomp.h`, `rt/runtime.c` — the guest context and memory, SSE
  semantics in x86's terms, and the trap that comes back as a message.
