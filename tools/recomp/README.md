# tools/recomp — original functions as native C

A static recompiler for functions of `riseofnations.exe`, one C function per
original function, verified against the emulator rung (`tools/emu/callfn.py`)
row for row — on a seeded sweep, and on every object of a captured frame.
The findings and the measurements are in `docs/lab/RECOMP-REVIEW.md`.
Everything here takes an install path and generates under `target/recomp/`;
nothing from the install is committed.

    uv run tools/recomp/lift.py     <exe> 0046cff0 00688310 005de340
    uv run tools/recomp/difftest.py <exe> diff
    uv run tools/recomp/difftest.py <exe> frame <frame-snapshot.bin> <typed-state.json> --table turn-speed-table.txt
    RON_TURN_TABLE=turn-speed-table.txt cargo test -p sim the_original_on_a_real_frame
    uv run tools/recomp/scan.py     <exe> --docs docs

- `lift.py` — decodes each function (capstone, declared inline) and every
  function it calls directly, emits `lifted.c` against `rt/recomp.h`, and
  builds `librecomp.dylib` with clang. An instruction outside its integer
  subset stops it and names itself.
- `difftest.py` — `callfn.py`'s own sweep through unicorn and through the
  native build (`diff`; `sweep` prints the native table alone in
  `callfn.py`'s format), and `frame`: the lab's end-frame snapshot loaded
  into both machines and `GuyData::turn_speed` called on every guy of every
  active unit, both modes, with `--table` writing each call's inputs and
  answer for the crate's port to assert.
- `snapshot.py` — the lab's `frame-snapshot-v1` stream as guest memory.
- `scan.py` — lifts and compiles every function a set of documents cites,
  and groups the failures by the instruction that stopped them.
- `image.py` — the PE's sections, imports and function table (the linker
  map; the PDB on request).
- `rt/recomp.h`, `rt/runtime.c` — the guest context, SSE semantics in x86's
  terms, a `PROT_NONE` reservation opened range by range, and the trap or
  fault that comes back as a message naming the guest address.
