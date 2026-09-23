# Static recompilation of the original: steps 1 and 2 measured, the first float function exact, step 3 counted

**Status: steps 1 and 2 done and diff-verified; the first float function
bit-exact against unicorn on 1,088 chosen vectors; step 3's mechanical
share counted at 91 %; step 4 not started. 2026-09-22.** The spike ran on
the charter lore sent on 2026-09-22 (worktree `recomp-spike`, independent of
the commander's loop, score-neutral). The charter asked: can
`riseofnations.exe` be statically recompiled, one function at a time, into
native arm64 code, using the PDB for function boundaries, so that an
original function runs in the same process as this crate's port — with
floats bit-exact? Step 0 (prior art) and the image findings were an Opus 5.5
session's; it stopped while writing the lifter. Fable 5.1 wrote the lifter,
the runtime and the harness, ran steps 1 and 2 and the float function, and
counted step 3, in one session, with lore's read on the order.

## Step 1: two functions, natively, against the emulator rung

`tools/recomp/lift.py` turns each function into one C function
(`void f_<va>(cpu_t *c)`: registers in C locals, the guest's memory a flat
4 GiB reservation, `goto` per branch, a direct C call per direct `call`,
flags computed eagerly and left to the C compiler to drop);
`tools/recomp/rt/runtime.c` reserves the guest space and turns a trap into a
message; `tools/recomp/difftest.py diff` drives `tools/emu/callfn.py`'s
**own** sweep — the same `sweep_calls` generator, so the same 3,516 calls in
the same order — through unicorn and through the native build, and compares
every row.

| measurement | value | how |
|---|---|---|
| Rows agreeing, `vector_dist@0046cff0` and `get_estimate@00688310` | **3,516 of 3,516** | `uv run tools/recomp/difftest.py <exe> diff` after `uv run tools/recomp/lift.py <exe> 0046cff0 00688310` |
| The diff can fail | yes: one constant of the lifted C changed (`0xea60` → `0xea61`, the 60,000 guard) gave **18 disagreeing rows** and exit 1 | the same command on the altered build |
| Time for the 3,516 calls, excluding process start | native **5–9 ms**, unicorn **22–43 ms** | printed by `difftest.py`; both are dwarfed by `uv`'s six seconds |
| `callfn.py`'s table after the refactor that exposed `sweep_calls` | byte-identical (`md5 9c0f7dd7…`), and `crates/sim`'s `the_emulated_original_agrees_on_every_row` passes in release against the install | `uv run tools/emu/callfn.py <exe> sweep`, before and after |

Both functions are integer-only (`vector_dist` is the hypotenuse, `get_estimate`
the A\* heuristic with a `ret 0x24`), so step 1 verifies the integer subset,
the call/return protocol through the guest stack, `div`/`idiv` and the
flags a `jle`/`jb`/`jne` reads. It verifies nothing about floats.

## Step 2: a function that reads the object graph, on a real frame, three ways

The charter's step 2 is `GuyData::turn_speed@005de340` on laid-out or typed
state. Lore's read (2026-09-22) was to take the lab's **typed end-frame
snapshot** rather than lay the ~12 fields out by hand: the end-frame
collector on `codex/typed-state-oracle` took one packet of the Great Lakes
market run at logger frame 11,186 (trace tick 11,185) — 177 ranges,
843,001,856 bytes of the process's private data and main-image data at the
addresses the game had them, 127 units marked active by the lab's decode.
`tools/recomp/snapshot.py` reads that stream's range table;
`difftest.py frame` maps every range into both machines beside the image
and calls `turn_speed` on **every guy of every active unit, in both modes**.

| measurement | value | how |
|---|---|---|
| Units, guys, calls | 127 units, 134 guys, **268 calls** (two modes) | `uv run tools/recomp/difftest.py <exe> frame <frame-snapshot.bin> <typed-state.json> --table …` |
| Native against unicorn, on the same memory | **268 of 268 agree, 0 traps** | the same command |
| The crate's port against the original's answers | **264 of 264 modelled rows agree**; the other 4 are guys past their squad with no track offset, a shape the port has no function for — counted, not asserted | `RON_TURN_TABLE=… cargo test -p sim the_original_on_a_real_frame_agrees_on_every_guy` (`crates/sim/src/movement.rs`; skips with a message when the table is absent) |
| That test can fail | yes: one answer altered in a copy of the table fails it on that line | the same test on the altered copy |
| Loading 843 MB | natively **0.08 s** (`mprotect` and `memmove`), into unicorn 0.09 s | printed by `difftest.py` |
| The rows' spread | 17 distinct answers; 204 of 268 are the instant-turn `0x80000000` (250 guys carry the flag, 224 are stopped), 12 are the crew quarter turn, the rest the divided rate; no packed unit on this frame | `awk` over the table |

Three readers, two independent of the port: the lifted C and unicorn agree
on the original's answer, and the port agrees with both. The self-checks the
driver runs on the way — each unit's `units` band slot points back at it,
and each guy's `who`/`o` are its unit's — passed on all 127 and 134.

**Provenance.** The snapshot, its decode (`typed-state.json`, which names
the active units) and the collector are the Codex lab's artifacts, on
`codex/typed-state-oracle` and its successor branch, not on `main`; the
packet is non-atomic across threads (the lab says so) and the lab has not
established full logger parity for it. Neither matters for a per-guy getter,
and the numbers above are on that packet only. The table the third reader
asserts is written to `~/ron-data/lab-experiments/2026-09-22-typed-state-market/recomp/`,
outside git like everything derived from the original.

## The float question: `Vector<float>::norm`, bit for bit

The charter's hard constraint was that floats be bit-exact. The first float
function through the diff is `Vector<float>::norm@00420870` — in-place
normalisation: `mulss`/`addss` for the sum of squares, `ucomiss` against 0
and 1.0 through the `lahf; test ah, 0x44; jnp` idiom, then `sqrtf@0041e6f0`
(which is **not** in-image arithmetic: `cvtss2sd`, a `call` to the thunk
`0x00a571b0`, which is `jmp dword ptr [0xac5530]`, the IAT slot of
`_libm_sse2_sqrt_precise`, then `cvtsd2ss`), a `divss` and three `mulss`
stores. The lifter gained the scalar-SSE subset and the 128-bit moves (the
XMM registers live in the context, not in locals), `lahf`/`sahf`, and
**import stubs**: a `call` or `jmp` through an IAT slot becomes
`rc_import(c, name)`, and the unicorn machine points every IAT slot at a
page of `ret`s with a code hook — both give `_libm_sse2_sqrt_precise` the
same semantics (host IEEE double sqrt, x86's indefinite NaN for a negative),
so the diff tests the lifted code around the import and never the import.

| measurement | value | how |
|---|---|---|
| On the frame's own 134 `last_norm` vectors | 134 of 134 agree — but every one is an axis unit vector, so all take the early exit; this row verifies the loads, the sum and the compare idiom, not the tail | `difftest.py frame`, the `norm` rows |
| **On 1,088 chosen vectors** — every triple from {±0, ±1, 0.5, 3, 1e-20, 1e-38, 1.5e-45, 1e20, 3e38, ±inf, NaN} × {0, 1, 1e20} × the same, plus 500 seeded random vectors with magnitudes from 1e-30 to 1e30 | **1,088 of 1,088 agree, bit for bit, 0 traps** | the `normv` rows: the vector is written to a scratch slot on both machines before each call, the row is the twelve bytes after it |
| The float diff can fail | yes: the lifted `divss` turned into a multiply gives **882 disagreeing rows** of 1,088 (the 206 that still agree are the early exits) and exit 1 | the same command on the altered build |
| Time | 1,490 calls (both families and the sweep) in 4 ms natively, 37 ms under unicorn | printed by `difftest.py` |

So on the scalar single-precision path — `mulss`, `addss`, `divss`,
`ucomiss` (ordered, unordered), `cvtss2sd`, `cvtsd2ss`, the NaN a result
carries, overflow to infinity and the reciprocal of infinity, denormal
inputs (`1.5e-45`, `1e-38`) — the C the lifter emits, compiled with
`-ffp-contract=off` and `rt/recomp.h`'s fix-ups, is bit-identical to
unicorn's x86 model (QEMU 5.0.1 softfloat) on this machine. That is the
answer to "where does x87 extended precision make bit-exactness hard":
nowhere on this path, because the path has no x87.

Two candidates were read and not taken: `Unit::bank_aircraft@005e9520`
(lore's suggestion — SSE only, but it makes two virtual calls, which the
lifter turns into run-time dispatch to functions it has not lifted), and
`Unit::air_turn_speed@005ea390` (one `cvttss2si` on the first guy's `bank`
and integer arithmetic otherwise; a cheap second, not run). Lore's reading
of the callers, not checked here: the CRT's transcendental imports are
reached only through the `sinf`/`cosf`/`tanf`/`acosf`/`atanf`/`powf`
wrappers, whose callers are the `fast_*_to_sine/cosine` table fills (once,
at init; the tables are main-image data the snapshot holds), the script
VM's `^`, and rendering — so a gameplay function that reaches a DLL
mid-frame was not found, and `Ammo::init@0067bbf0`'s half-angle quaternion
is the one gameplay-adjacent user of the DLL-built sine table.

## Step 3: what share of the cited functions lifts mechanically

`tools/recomp/scan.py --docs docs` takes every `name@<8 hex>` citation in
the top-level `docs/*.md` (as the paperwork guard reads them), lifts each
function on its own — callees named, not followed — and compiles each
lift to an object. Eight seconds.

| | first subset | after `bt`/`bts`/`btr`/`btc`, `rol`/`ror`, `rep stos`/`movs`, `fs:` as a guest TIB |
|---|---|---|
| addresses cited, in `.text` | 771 (of 779 cited) | 771 |
| **lifted and compiled** | 555 (72 %) | **698 (91 %)** |
| stopped at `fs:` — the SEH prologue's `mov eax, fs:[0]` | 149 | 0 |
| stopped at an SSE instruction | 39 | 70 |
| stopped at `bt`/`bts`/`btr`, `rol`, a string op | 28 | 0 |
| stopped at a `lock` prefix (interlocked ops) | — | 3 |
| x87, undecodable, ran into the next function, discovery looping | 0 each | 0 each |

The residue is now one thing: **SSE, 70 functions**, and a share of those
are data moves, not arithmetic — `xorps xmm0, xmm0` and `movaps`/`movups`
of sixteen bytes are how this compiler zeroes and copies structs
(`PathFinder::find_wpath`, `PathFinder::init`, `Army::init`); the count of
the ones that do float arithmetic was not taken. "Lifted and compiled" is
not "runs correctly": three functions are diff-verified. The per-function
rows are in `target/recomp/scan.tsv` after a run.

## What changed our understanding (the image findings, kept and corrected)

| Finding | Evidence | What it does not establish |
|---|---|---|
| The game's floating point is SSE2, not x87. | A mnemonic histogram over the whole `.text` (`llvm-objdump`): about 12.5k `movss`, 3.9k `mulss`, 2.4k `addss`, against about 700 x87 hits in total, many of them data decoded as code by the linear sweep. No `ldmxcsr` in `.text`. **And the scan: zero x87 among the 771 cited functions.** | Which functions hold the real x87 hits. The executable also imports `_set_SSE2_enable` and `_except1` from `api-ms-win-crt-math`, so the C runtime's own maths can take an x87 path at run time; MXCSR as set by DLLs was not read. |
| So bit-exactness is mostly SSE edge cases, not 80-bit precision. | SSE add, sub, mul, div and sqrt are exactly specified IEEE operations on both x86 and AArch64. The differences to handle are the NaN a result carries, the `minss`/`maxss` operand order, out-of-range truncation (`0x80000000`), and FMA contraction (`-ffp-contract=off`). `rt/recomp.h`'s helpers say each in x86's terms; 17 edge cases of them were checked against the SDM's answers, and **the section above puts `norm`'s path through them on 1,088 vectors against unicorn**. | `minss`/`maxss`, the truncations and the double-precision arithmetic are lifted but no verified function uses them yet. |
| Internal functions use link-time custom conventions. | `Vector<float>::norm@00420870` passes a float in `xmm0` to `sqrtf@0041e6f0` and gets its result back in `xmm0`, with no x87 return. `sqrtf` itself widens to double and calls the CRT's `_libm_sse2_sqrt_precise` through the thunk at `0x00a571b0`; it is not in-image `sqrtss`. | A machine-level recompiler keeps the XMM registers in its context, so this is transparent to it. A Rust caller still needs each function's convention, as `tools/emu/callfn.py` already records. |
| The transcendentals are not in the executable. | The import directory (read by `tools/recomp/image.py`): `_libm_sse2_{acos,asin,atan,cos,pow,sin,sqrt,tan}_precise` from `api-ms-win-crt-math-l1-1-0.dll`. | Under the captures, Wine's ucrtbase answers these calls, not Microsoft's. Neither implementation was read. |
| No existing 32-bit x86 PE recompiler is worth building on. | [Prior-art survey](2026-09-22-recomp-prior-art.md), 17 projects with URLs — read with its header note: its x87 premise is the one the first row overturns, and the build is Visual Studio 2015 or later (`.gfids`, `ucrtbase`), not MSVC 7.1. | The survey read sources and READMEs; it built none of them. |
| The linker map is the function table the lifter needs. | 63,427 `f` symbols in `sbl/rise_z.map`, every one inside `.text`; the lifter bounds discovery by the next symbol and stops on a tail `jmp` into another symbol. The PDB's 22,199 `S_*PROC32` records with code sizes are read only on request (`Functions(…, cache_dir)`), since `llvm-pdbutil`'s dump is slow and large. | Jump tables (`jmp [reg*4+table]`) stop discovery at the indirect jump; none of the 771 needed one to reach its `ret`s, but a `switch` lowered that way would lift its dispatch as a run-time trap. |

## The challenge question

The charter asked whether an existing emulator gives the same in-process
oracle more cheaply. Unicorn is the emulator rung (`docs/EMULATOR.md`),
reached through `tools/emu/callfn.py` out of process; linking it in-process
would add a GPL-2 crate to an MIT/Apache repository, and Ramon declined that
pivot. Steps 1 and 2 put numbers on the difference: the native build answers
the sweep in 5–9 ms against unicorn's 22–43 ms, and the frame's 268 calls in
1 ms against 2 ms; both are hidden behind `uv`'s six-second start. **For a
fixture of a few thousand calls the two are equivalent**; the native build's
case is the in-process one — a Rust test calling the original directly,
with no subprocess — and that is not built. What step 2 adds to the
question: the snapshot loads in 0.08 s either way, so a per-function
differential test on real frames is cheap on both machines.

## What exists on the branch

- `tools/recomp/lift.py` — the lifter: capstone (declared inline, like
  `callfn.py`'s unicorn), the integer subset, the scalar-SSE subset and
  the 128-bit moves, `fs:` as the guest TIB, imports through the IAT as
  `rc_import`, an error naming the address and instruction of anything
  outside it. Emits `lifted.c` and builds `librecomp.dylib` with clang
  under `target/recomp/`.
- `tools/recomp/rt/recomp.h`, `rt/runtime.c` — the guest context, memory
  and SSE helpers; a `PROT_NONE` reservation with `rc_map` to open the
  ranges a harness lays out, `rc_call`, the import stubs, and a trap or a
  fault that comes back as a message naming the guest address.
- `tools/recomp/snapshot.py` — the lab's `frame-snapshot-v1` stream as
  guest memory: the range table and where each range's bytes sit.
- `tools/recomp/difftest.py` — `callfn.py`'s sweep through both machines
  (`diff`, `sweep`), and the frame driver (`frame`, `--table`): `turn_speed`
  on every guy, `norm` on every guy's vector, and `norm` on the chosen
  vectors; the unicorn machine with the imports stubbed and the registers
  zeroed per call.
- `tools/recomp/scan.py` — the step-3 count.
- `tools/recomp/image.py` — the PE, its imports, the function table.
- `crates/sim/src/movement.rs` — the one touch outside `tools/` and
  `docs/lab/`: the `RON_TURN_TABLE` test, which skips when the table is
  absent and changes nothing else.
- Nothing from the install, and nothing generated from it, is committed.

## What is not established

- Float behaviour beyond `norm`'s path: the double-precision arithmetic,
  `minss`/`maxss`, the truncating and rounding conversions and the packed
  bitwise ops are lifted but unexercised by a verified function; packed
  arithmetic (`addps` and kin), `shufps`, `cmpss` and x87 stop the lift.
  MXCSR is assumed at its default (round to nearest, no FTZ/DAZ) on both
  machines; the game's own MXCSR at the capture boundary was not read.
- The reference for floats is unicorn's x86 model (QEMU 5.0.1 softfloat),
  not silicon and not Rosetta, which runs the captures. The two machines
  give the sqrt import identical semantics by construction, so the CRT's
  own `sqrt` (Wine's under the captures) is outside the diff.
- `af` after a logic instruction is left as it was, and the flags after a
  shift by zero are left as they were, which is x86's rule; `of` after a
  multi-bit shift is emitted as if the count were one (x86 leaves it
  undefined); `rol`/`ror` by a count that is a multiple of the width leave
  `cf` as it was, where x86 sets it. Nothing verified reads any of these.
- Sub-32-bit `mul`, `div`, `push` and `pop`, `lock`, `repe`/`repne` and
  indirect jumps stop the lift by design; an indirect `call` (a vtable) is
  dispatched at run time and traps unless its target was lifted.
- The fault's granularity is the host page, 16 KiB on Apple Silicon
  against the guest's 4 KiB: a read within the same 16 KiB as a mapped range
  does not fault. A function that lifts because its SEH prologue now writes
  the guest TIB runs wrong if it then throws.
- The harness's stack (`0x7ff00000`, `callfn.py`'s constant) sits inside a
  range the snapshot also holds (`0x7fde0000+0x200000`, a thread's stack);
  the native machine maps the snapshot over it and the harness's frame
  overwrites a few hundred bytes of it per call, unicorn refuses the two
  overlapping ranges. Harmless for `turn_speed`; a function that reads that
  stack needs the harness's stack moved.
- The active-unit list comes from the lab's decode, not from the packet's
  `units` bands; the driver checks each unit against its band but does not
  enumerate the bands itself.
- Step 4 (how far a frame runs with imports stubbed) was not attempted.
- The scan counts functions the specification cites, not the executable's
  22,199; the share over all of `.text` is not known.

## Adoption

**Pilot-ready as a lab tool; one candidate for the main loop; the
charter's float question answered on its first function.** It moves no
score and touches no queue file. What it is good for now: a fixture rung on
real frames — an original function called on every object of a captured
frame in a millisecond, natively or under unicorn, with the port asserted
beside it — which is the per-function differential testing the lab has
queued, built and run once; and, for floats, a place where the C a lifter
emits and QEMU's x86 model can be put side by side on chosen inputs, which
is what settles a float residue without a capture. The candidate:
`RON_TURN_TABLE`'s test is the shape every such function would get. What
would make the tool worth more, in order of cost: a second float function
whose path has `minss`/`maxss`, a truncation or double arithmetic
(`air_turn_speed` is a cheap start); enumerating the `units` bands from the
packet so the driver needs no decode; the harness's stack moved off the
snapshot; lifting a vtable's targets so a virtual call dispatches; then
step 4.
