# Static recompilation of the original: findings, on hold

**Status: held at step 1, 2026-09-22.** This spike ran on the charter lore sent
on 2026-09-22 (Opus 5.5, worktree `recomp-spike`, independent of the
commander's loop, score-neutral). The charter asked: can `riseofnations.exe` be
statically recompiled, one function at a time, into native arm64 code, using
the PDB for function boundaries, so that an original function runs in the same
process as this crate's port? Step 0 (prior art) is done. Step 1 stopped while
the x86→C lifter was being written, and Ramon put the spike on hold. Nothing
was recompiled or run, so no row of any sweep is measured here.

## What changed our understanding

| Finding | Evidence | What it does not establish |
|---|---|---|
| The game's floating point is SSE2, not x87. | A mnemonic histogram over the whole `.text` (`llvm-objdump`): about 12.5k `movss`, 3.9k `mulss`, 2.4k `addss`, against about 700 x87 hits in total, many of them data decoded as code by the linear sweep. No `ldmxcsr` in `.text`. | Which functions hold the real x87 hits (no per-function count was taken). MXCSR as set at run time by DLLs was not read. |
| So bit-exactness is mostly SSE edge cases, not 80-bit precision. | SSE add, sub, mul, div and sqrt are exactly specified IEEE operations on both x86 and AArch64. The differences to handle are the NaN a result carries, the `minss`/`maxss` operand order, out-of-range truncation (`0x80000000`), and FMA contraction (`-ffp-contract=off`). | Not tested: no recompiled code ran. |
| Internal functions use link-time custom conventions. | For example, `0x420870` passes a float in `xmm0` to `0x41e6f0` and gets its result back in `xmm0`, with no x87 return. | A machine-level recompiler keeps the XMM registers in its context, so this is transparent to it. A Rust caller still needs each function's convention, as `tools/emu/callfn.py` already records. |
| The transcendentals are not in the executable. | The linker map imports `__libm_sse2_{sin,cos,tan,acos,asin,atan,pow,sqrt}_precise` from ucrtbase (`api-ms-win-crt-math`). | Under the captures, Wine's ucrtbase answers these calls, not Microsoft's. Neither implementation was read. |
| No existing 32-bit x86 PE recompiler is worth building on. | [Prior-art survey](2026-09-22-recomp-prior-art.md), 17 projects with URLs. SR/SRW and pcrecomp come closest; neither is float-exact and neither fits an in-process oracle. | The survey read sources and READMEs; it built none of them. |
| The PDB and the map give the function table. | 22,199 `S_*PROC32` records with code sizes; 63,427 `f` symbols in `sbl/rise_z.map`. | Jump tables, and functions with no size in the PDB, need discovery; that part of the lifter was not written. |

## The challenge question

The charter asked whether an existing emulator gives the same in-process oracle
more cheaply. Unicorn is already the emulator rung (`docs/EMULATOR.md`),
reached through `tools/emu/callfn.py` out of process. Linking it in-process
would add a GPL-2 crate (`unicorn-engine`) to an MIT/Apache repository. Ramon
declined that pivot: it duplicates what the rung already does.

## What exists on the branch

- `tools/recomp/rt/recomp.h`: the runtime header generated code would target. It
  defines the guest context, a flat 4 GiB guest space addressed through a base
  pointer, and x86's SSE semantics as named above. Nothing compiled against it
  yet.
- `tools/recomp/image.py`: the PE sections, the import directory, and the
  function table from the map and the PDB. It reads the install on demand.
- No lifter, no generated code, no tests. Nothing from the install is committed.

## Adoption

**Park, as held.** Nothing here moves a score or changes the main branch's
tools. If the spike resumes, the next step is still step 1 as the charter
wrote it: `vector_dist@0046cff0` and `get_estimate@00688310` against the
existing sweep.
