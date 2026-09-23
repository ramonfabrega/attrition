# Static recompilation of the original: step 1 measured, step 3 counted

**Status: step 1 done and diff-verified; step 3's mechanical share counted;
steps 2 and 4 not started. 2026-09-22.** The spike ran on the charter lore
sent on 2026-09-22 (worktree `recomp-spike`, independent of the commander's
loop, score-neutral). The charter asked: can `riseofnations.exe` be statically
recompiled, one function at a time, into native arm64 code, using the PDB for
function boundaries, so that an original function runs in the same process as
this crate's port? Step 0 (prior art) and the image findings were an Opus 5.5
session's; it stopped while writing the lifter. Fable 5.1 wrote the lifter,
the runtime and the harness, ran step 1, and counted step 3, in one session.

## Step 1: two functions, natively, against the emulator rung

`tools/recomp/lift.py` turns each function into one C function
(`void f_<va>(cpu_t *c)`: registers in C locals, the guest's memory a flat
4 GiB reservation, `goto` per branch, a direct C call per direct `call`,
flags computed eagerly and left to the C compiler to drop);
`tools/recomp/rt/runtime.c` maps the guest space and turns a trap into a
message; `tools/recomp/difftest.py` drives `tools/emu/callfn.py`'s **own**
sweep — the same `sweep_calls` generator, so the same 3,516 calls in the
same order — through unicorn and through the native build, and compares
every row.

| measurement | value | how |
|---|---|---|
| Rows agreeing, `vector_dist@0046cff0` and `get_estimate@00688310` | **3,516 of 3,516** | `uv run tools/recomp/difftest.py <exe> diff` after `uv run tools/recomp/lift.py <exe> 0046cff0 00688310` |
| The diff can fail | yes: one constant of the lifted C changed (`0xea60` → `0xea61`, the 60,000 guard) gave **18 disagreeing rows** and exit 1 | the same command on the altered build |
| Time for the 3,516 calls, excluding process start | native **5–7 ms**, unicorn **22–40 ms** | printed by `difftest.py`; both are dwarfed by `uv`'s six seconds |
| `callfn.py`'s table after the refactor that exposed `sweep_calls` | byte-identical (`md5 9c0f7dd7…`), and `crates/sim`'s `the_emulated_original_agrees_on_every_row` passes in release against the install | `uv run tools/emu/callfn.py <exe> sweep`, before and after |

Both functions are integer-only (`vector_dist` is the hypotenuse, `get_estimate`
the A\* heuristic with a `ret 0x24`), so step 1 verifies the integer subset,
the call/return protocol through the guest stack, `div`/`idiv` and the
flags a `jle`/`jb`/`jne` reads. It verifies nothing about floats.

## Step 3: what share of the cited functions lifts mechanically

`tools/recomp/scan.py --docs docs` takes every `name@<8 hex>` citation in
the top-level `docs/*.md` (as the paperwork guard reads them), lifts each
function on its own — callees named, not followed — and compiles each
lift to an object. Seven seconds.

| | count |
|---|---|
| addresses cited, in `.text` | 771 (of 779 cited) |
| **lifted and compiled** | **555 (72 %)** |
| stopped at `fs:` — all 149 are the SEH prologue's `mov eax, fs:[0]` | 149 |
| stopped at an SSE instruction | 39 |
| stopped at `bt`/`bts`/`btr` (19), `rol` (4), a string op — `rep stosd`, `movsd es:[edi]` (5) | 28 |
| x87, undecodable, ran into the next function, discovery looping | 0 each |

Two readings of the table. **The subset is small and the residue is
shallow**: the 28 integer stops are five instructions, an hour's work, and
the 149 SEH stops are one addressing mode (`fs:` → a guest TIB at
`c->fs_base`, three memory operations in the prologue and epilogue) — for
*lifting*; running a function that then throws is a different matter. **A
share of the 39 SSE stops are data moves, not arithmetic**: `xorps xmm0,
xmm0` and `movaps`/`movups` of sixteen bytes are how this compiler zeroes
and copies structs (`PathFinder::find_wpath`, `PathFinder::init`,
`Army::init`); the count of the ones that do float arithmetic was not taken.

"Lifted and compiled" is not "runs correctly": only the two functions of
step 1 are diff-verified. The per-function rows are in
`target/recomp/scan.tsv` after a run.

## What changed our understanding (the image findings, kept and corrected)

| Finding | Evidence | What it does not establish |
|---|---|---|
| The game's floating point is SSE2, not x87. | A mnemonic histogram over the whole `.text` (`llvm-objdump`): about 12.5k `movss`, 3.9k `mulss`, 2.4k `addss`, against about 700 x87 hits in total, many of them data decoded as code by the linear sweep. No `ldmxcsr` in `.text`. **And the scan above: zero x87 among the 771 cited functions.** | Which functions hold the real x87 hits. The executable also imports `_set_SSE2_enable` and `_except1` from `api-ms-win-crt-math`, so the C runtime's own maths can take an x87 path at run time; MXCSR as set by DLLs was not read. |
| So bit-exactness is mostly SSE edge cases, not 80-bit precision. | SSE add, sub, mul, div and sqrt are exactly specified IEEE operations on both x86 and AArch64. The differences to handle are the NaN a result carries, the `minss`/`maxss` operand order, out-of-range truncation (`0x80000000`), and FMA contraction (`-ffp-contract=off`). `rt/recomp.h`'s helpers say each in x86's terms, and 17 edge cases of them were checked against the SDM's answers on this machine. | No float instruction has been lifted; the helpers have no caller yet. |
| Internal functions use link-time custom conventions. | `Vector<float>::norm@00420870` passes a float in `xmm0` to `sqrtf@0041e6f0` — a `sqrtf` built into the executable — and gets its result back in `xmm0`, with no x87 return. | A machine-level recompiler keeps the XMM registers in its context, so this is transparent to it. A Rust caller still needs each function's convention, as `tools/emu/callfn.py` already records. |
| The transcendentals are not in the executable. | The import directory (read by `tools/recomp/image.py`): `_libm_sse2_{acos,asin,atan,cos,pow,sin,sqrt,tan}_precise` from `api-ms-win-crt-math-l1-1-0.dll`. | Under the captures, Wine's ucrtbase answers these calls, not Microsoft's. Neither implementation was read. |
| No existing 32-bit x86 PE recompiler is worth building on. | [Prior-art survey](2026-09-22-recomp-prior-art.md), 17 projects with URLs — read with its header note: its x87 premise is the one the first row overturns, and the build is Visual Studio 2015 or later (`.gfids`, `ucrtbase`), not MSVC 7.1. | The survey read sources and READMEs; it built none of them. |
| The linker map is the function table the lifter needs. | 63,427 `f` symbols in `sbl/rise_z.map`, every one inside `.text`; the lifter bounds discovery by the next symbol and stops on a tail `jmp` into another symbol. The PDB's 22,199 `S_*PROC32` records with code sizes are read only on request (`Functions(…, cache_dir)`), since `llvm-pdbutil`'s dump is slow and large. | Jump tables (`jmp [reg*4+table]`) stop discovery at the indirect jump; none of the 771 needed one to reach its `ret`s, but a `switch` lowered that way would lift its dispatch as a run-time trap. |

## The challenge question

The charter asked whether an existing emulator gives the same in-process
oracle more cheaply. Unicorn is the emulator rung (`docs/EMULATOR.md`),
reached through `tools/emu/callfn.py` out of process; linking it in-process
would add a GPL-2 crate to an MIT/Apache repository, and Ramon declined that
pivot. Step 1 now puts a number on the difference: the native build answers
the sweep in 5–7 ms against unicorn's 22–40 ms, and both are hidden behind
`uv`'s six-second start. **For a fixture of a few thousand calls the two are
equivalent**; the native build's case is the in-process one — a Rust test
calling the original directly, with no subprocess — and that is not built.

## What exists on the branch

- `tools/recomp/lift.py` — the lifter: capstone (declared inline, like
  `callfn.py`'s unicorn), the integer subset, an error naming the address and
  instruction of anything outside it. Emits `lifted.c` and builds
  `librecomp.dylib` with clang under `target/recomp/`.
- `tools/recomp/rt/recomp.h`, `rt/runtime.c` — the guest context, memory
  and SSE helpers; the reservation, `rc_call`, and a trap that comes back as
  a message.
- `tools/recomp/difftest.py` — `callfn.py`'s sweep through both machines;
  `sweep` alone prints the native table in `callfn.py`'s format, so
  `RON_EMU_TABLE` can point at it.
- `tools/recomp/scan.py` — the step-3 count.
- `tools/recomp/image.py` — the PE, its imports, the function table.
- Nothing from the install, and nothing generated from it, is committed.

## What is not established

- Any float behaviour: no SSE instruction is lifted; the helpers are untested
  against a real function. The first float function through `difftest.py`
  is what would test them, and unicorn's SSE (QEMU softfloat) is the
  reference — not Rosetta's, which runs the captures.
- `af` after a logic instruction is left as it was, and the flags after a
  shift by zero are left as they were, which is x86's rule; `of` after a
  multi-bit shift is emitted as if the count were one (x86 leaves it
  undefined). Nothing in the sweep reads any of these.
- Sub-32-bit `mul`, `div`, `push` and `pop`, `fs:` addressing, indirect
  jumps and the string instructions stop the lift by design.
- The guest space is all mapped: a wild read returns zeros instead of
  faulting, where unicorn names the address. A function that reads a
  singleton this harness did not lay out is therefore wrong quietly here and
  loudly there — step 2 needs the fault back (a `PROT_NONE` reservation with
  the sections and the stack mapped over it).
- Step 2 (`GuyData::turn_speed@005de340` on laid-out state) and step 4 (how
  far a frame runs) were not attempted.
- The scan counts functions the specification cites, not the executable's
  22,199; the share over all of `.text` is not known.

## Adoption

**Pilot-ready as a lab tool; nothing for the main loop yet.** It moves no
score and touches no queue file. What it is good for now: the same fixture
rung as `callfn.py`, faster, and a C rendering of any integer function that
is sometimes easier to read than the decompile (every instruction is
commented with its address). What would make it worth more than that, in
order of cost: the five integer instructions and the `fs:` prologue (an hour,
unlocks 177 of the 216 stops for lifting); the first float function through
the diff (tests the header's helpers, which is the whole bit-exactness
question); a `PROT_NONE` reservation so a missing singleton faults; then step
2. The per-function differential testing the lab has queued needs none of
this — it runs on either machine.
