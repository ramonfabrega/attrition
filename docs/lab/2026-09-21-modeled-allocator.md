# An explicit allocation model for retained-state replay

This follow-up is independent of the adoption decision on draft PR #4. That
review remains on `codex/jev-lab`; this experiment starts from `a4f996e` on
`codex/replay-allocator-lab`. It changes no simulation code or main-loop score.
No live capture or TypeSafe request is needed.

## Contract and evidence boundary

L63 stopped at the owned executable's `malloc` import. The opt-in
`tools/explore/modeled_allocator.py` supplies an authored one-byte return
instruction at that verified target, sets EAX to a fresh allocation, and lets
cdecl return normally. It preserves the other registers and flags. This is a
chosen success model, not evidence about the native allocator's pointer choices,
metadata, failure behavior, caller-saved clobbers, or timing.

The model uses an explicit arena, 16-byte alignment, an allocation-count limit,
and a capacity limit. Only the exact requested bytes are addressable. Allocation
bytes remain uninitialized until guest writes them during that attempt; alignment
gaps and unused capacity cannot be read or written. It refuses zero-size
requests, exhausted limits, malformed stack reads, and undeclared return targets.
Other runtime services remain unavailable. Each run resets allocations, guest
memory, initialization tracking and CPU context through the existing strict
runner. The base `BoundedCall` and `ThreadCall` are unchanged.

The external probe revalidates the complete retained L62 packet, checks its
payload hash, checks the captured import target against L63's identified import,
and chooses a one-MiB arena wholly inside a captured `MEM_FREE` range. All region
overlap checks also apply to the service, arena, stack and descriptor table.
Missing captured data still ends the attempt; the next attempt starts fresh
with an additional captured page fragment. It never fills missing data with
zeros or treats the allocation model as captured evidence.

Eleven authored x86 tests cover cdecl stack cleanup and preserved registers,
multiple allocations, exact bounds, unwritten and partially written reads,
reset after success and failure, quotas, missing stack bytes, malformed returns,
undeclared service/code targets and region overlap. The allocation guards have
been exercised by deliberate violations, not merely by successful replay.

## Results

The first bounded run reached 10,000 instructions after six modeled allocations
and 34 captured-data additions. Raising the instruction budget to 100,000
reached that limit after 54 allocations totaling 1,284 requested bytes and 39
data additions (153,952 captured bytes). Neither is a completed search or a
fidelity measurement. A relocated-arena control at `0x19710000` instead of
`0x184e0000` has exactly
the same dependency sequence, allocation sizes, call sites and relative offsets
at that limit. Its raw write digest differs; pointer-bearing writes have not
been compared semantically, so this is not an output-equivalence claim.

With a one-million-instruction limit and 1,024-allocation limit, execution
instead reaches a real missing-service boundary at **128,717 instructions**.
It performs **72 modeled allocations totaling 1,756 requested bytes**, with
39 captured-data additions totaling **153,952 bytes**. No uninitialized heap
read or allocation-bounds violation was encountered along that prefix.

The next missing instruction is `0x7b951760`. The original call at `0x681fef`
goes through thunk `0x55e0b2`, which jumps through import slot `0xac5424`.
The owned PE import table names that slot **`VCRUNTIME140.dll!memset`**.
The stack return address is `0x681ff4`. This service remains unmodeled; no A*
return was reached. The attempt records 26,549 writes with SHA-256
`5c989bcc84137328d8c6fda8290e1680c7053c6c1ddb040bd99de54746b01fa9`.
A fresh process reproduces the entire dependency sequence, allocation transcript,
terminal arguments and write digest. A third process perturbs all eight XMM and
physical x87 registers plus MXCSR/FPCW; it reproduces those observations exactly.
This is prefix-specific evidence, not a general extended-state import. The
unmodeled call is `memset(0x35445218, 0, 96)`; the captured import-slot value also
matches the refused DLL target.

FXSAVE is still not imported. The retained native A* call returns 1, which can
provide a scalar comparison if replay reaches that boundary. Full mutated-state
equivalence requires a stronger comparison than a matching return value.

## Retention, validation and next step

Authored experiment sources, six observations, import identification and a
hash manifest are retained outside git at
`/Users/rf-studio/ron-data/lab-experiments/2026-09-21-modeled-allocator`.
They bind the L62 payload SHA-256
`d0b4ce6ca20fcf2c1e17f6d385d48150cf84660c706ae944c3b372897824756f`
and the owned original executable. Packet-derived observations remain external.

Seventy focused lab tests pass, including all eleven new allocation tests and
the retained-context replay checks. The full repository gate passed at
`/tmp/modeled-allocator-gate`: 332 rondata tests, 855 sim tests, 13 fixed tests,
three doc tests, and all 782 requested fixtures present; clippy, formatting,
data survey and paperwork guards passed. Peak tree RSS was 8,442 MiB with two
test workers. Final validation-note edits receive the paperwork guard.

The next bounded experiment is an explicitly tested memory-fill service whose
writes still pass the same byte guards. It should not expand into an implicit
Windows runtime: each new service needs a contract and refusal tests. A native
return comparison and affected-state comparison still stand between a modeled
search completion and a faithful replay claim. No capture slot is currently
needed. Fable's review can proceed independently of this follow-up.
