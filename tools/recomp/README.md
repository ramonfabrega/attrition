# tools/recomp — original functions as native C

A static recompiler for functions of `riseofnations.exe`, one C function per
original function, verified against the emulator rung (`tools/emu/callfn.py`)
row for row — on a seeded sweep, and on every object of a captured frame.
The findings and the measurements are in `docs/lab/RECOMP-REVIEW.md`.
Everything here takes an install path and generates under `target/recomp/`;
nothing from the install is committed.

    uv run tools/recomp/lift.py     <exe> 0046cff0 00688310 005de340 00420870 005ea390
    uv run tools/recomp/difftest.py <exe> diff
    uv run tools/recomp/difftest.py <exe> frame <frame-snapshot.bin> <typed-state.json> --table turn-speed-table.txt
    RON_TURN_TABLE=turn-speed-table.txt cargo test -p sim the_original_on_a_real_frame
    uv run tools/recomp/scan.py     <exe> --docs docs

- `lift.py` — decodes each function (capstone, declared inline) and every
  function it calls directly, emits `lifted.c` against `rt/recomp.h`, and
  builds `librecomp.dylib` with clang. Integer and scalar-SSE subsets,
  `fs:` as a guest TIB, imports through the IAT as `rc_import`; an
  instruction outside the subsets stops it and names itself.
- `difftest.py` — `callfn.py`'s own sweep through unicorn and through the
  native build (`diff`; `sweep` prints the native table alone in
  `callfn.py`'s format), and `frame`: the lab's end-frame snapshot loaded
  into both machines, `GuyData::turn_speed` called on every guy of every
  active unit in both modes (`--table` writes each call's inputs and answer
  for the crate's port to assert), `Vector<float>::norm` on every guy's
  vector and on 1,088 chosen vectors, and `Unit::air_turn_speed` on every
  unit with 22 chosen banks — the float diff. The unicorn machine has the
  imports stubbed with the same semantics `rt/runtime.c` gives them.
- `difftest.py … run <snapshot> --entry <va> --this <va> --dump <va>+<n>` —
  one function on the packet in both machines (the same TEB as `fs:`, a
  fresh stack, the imports stubbed alike), the named memory compared byte
  for byte afterwards.
- `step4.py` — the explorer: a function on the packet under unicorn with
  `fs:` on the capturing thread's TEB and a stub table that answers the
  clock, heap and thread imports; reports what stopped it, what ran, the
  exact functions it called (and whether they lift), `--watch` for who
  writes a range, `--find` for the instruction that first produces a
  value. Step 4 of the charter, and how item 327 was read.
- `get_position.py` — `GraphicPieces::get_position@0090b750` on a packet
  (packed SSE, so it does not lift): a piece's `AttachPos` entries and
  `RData +0x88` scale (`entries`), the non-pivot arm at every whole degree
  (`sweep`), and the pivot arm on every facing and turret step (`pivot`).
  How a release node's rows are read for `sim::launch` and `sim::pivot`
  (items 602, 1194, 1257).
- `snapshot.py` — the lab's `frame-snapshot-v1` stream as guest memory.
- `scan.py` — lifts and compiles every function a set of documents cites,
  and groups the failures by the instruction that stopped them.
- `image.py` — the PE's sections, imports and function table (the linker
  map; the PDB on request).
- `rt/recomp.h`, `rt/runtime.c` — the guest context, SSE semantics in x86's
  terms, a `PROT_NONE` reservation opened range by range, and the trap or
  fault that comes back as a message naming the guest address.
