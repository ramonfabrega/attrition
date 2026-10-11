# REA follow-up — select the question before bounding the evidence

User asked to continue extracting useful techniques/tools without the agentic
layer. Single-worker exploration from `4d8fcb0f`; commander remains paused.
This supplements [the initial assessment](2026-10-11-rea-technique-trial.md).
No simulation implementation, capture, fixture, score or comparison changed.

## What was missing, and what was already present

REA's native value trace exposes Ghidra high-p-code definitions and uses, then
composes recovered arguments/returns across resolved calls. Our existing exports
provide pseudocode, types and vtables, but no selected data-flow slices. This
was the strongest additional capability to test. Existing packet emulation,
write watches, original-function probes and assembly remain the executable
checks; a graph does not replace them.

The reviewed `resolveNativeCallTargets` uses Ghidra's existing static call
references. It explicitly leaves reference-less computed targets unresolved;
it is not a general vtable/closure resolver. Our PDB vtable exports and emulator
hooks already cover much of this need. No new engine is justified by that name.

Sources remain pinned to upstream `d802304c542b20bc86e22cc492718e89a8a5579b`:
[bridge](https://github.com/morluto/rea/blob/d802304c542b20bc86e22cc492718e89a8a5579b/bridge/ghidra/ReaGhidraBridge.java),
[value-trace composition](https://github.com/morluto/rea/blob/d802304c542b20bc86e22cc492718e89a8a5579b/src/application/native/NativeValueTrace.ts),
[provider contract](https://github.com/morluto/rea/blob/d802304c542b20bc86e22cc492718e89a8a5579b/docs/ghidra-provider.md).
These source conclusions are provisional pending independent review; the
local measurements below are observations, not a REA installation benchmark.

## Executed experiment

A small original scratch Java script opens our existing Ghidra project with
`-noanalysis -readOnly`, decompiles only requested functions, selects high-p-code
operations at an exact instruction, and walks their input definitions. Each
query is bounded at 128 nodes / depth 16 and reports budget stops. Complete
outputs remain ignored; no decompiled code was transcribed or committed.

The known caravan predicate is the positive control. The unresolved ammo
question supplies the branch before the two RNG calls. A non-instruction
address inside the branch encoding is the no-result control.

| query | observation |
|---|---|
| Caravan return | 4 nodes, no budget stop; load offset 0x2b8 and mask 8 agree with prior original-byte listing |
| Ammo function | 11,987 high-p-code operations in this saved project |
| Ammo gate location | Branch seed is operation 6,335 in iterator order |
| Initial gate dependency slice | 38 nodes, four depth-budget stops; incomplete |
| Initial RNG-site query, all operations at that address | 64 seeds, 128-node cap hit, 78 budget stops |
| Refined gate query: CBRANCH only, stop at INDIRECT | 11 nodes, five explicitly unresolved call-effect boundaries, no budget stop |
| Refined RNG query: CALL only | One node; target and arguments 0 / 0xffff agree with prior assembly |
| Invalid-address query | Zero seeds/nodes, explicit unknown; no guessed nearest instruction |

The refined invocation completed in **6.11 s**, exit 0, including headless
startup and four decompilations; output is **3,217 bytes**. Internal decompilation
times: 1,993 ms for the first tiny control, then 397 / 330 / 319 ms for the ammo
queries. These are one-run observations, not a cold/warm benchmark or a claim
of agent-time savings. No peak-RSS measurement was made; the installed headless
launcher defaults to a 2 GiB Java heap (not a total process-memory bound).

## Lessons that change how we investigate

1. **Select the exact instruction and operation before applying a size cap.**
   REA's reviewed bridge retains the first 3,000 operations. Applying that
   selection to our measured graph omits the gate at 6,335. A question-specific
   slice reaches it with a small result. This does not demonstrate what a fresh
   REA import would produce, nor that its full pseudocode/assembly omit the gate.
2. **An address alone can still be too broad.** One RNG instruction had 64
   associated high-p-code operations, including synthetic effects. Selecting
   CALL reduced that query to the one relevant operation. Branch queries select
   CBRANCH, return queries RETURN; ambiguous or missing seeds stay visible.
3. **Stop at uncertainty rather than rendering it as a complete explanation.**
   The refined gate graph stops at five INDIRECT nodes. Those are possible
   side effects, not proven copies of the incoming value. LOAD address tracing
   does not establish memory contents or aliases. No budget stop is not the
   same thing as a fully resolved value.
4. **Use the graph to locate evidence, not to reconstruct machine control flow
   from printed high-p-code addresses.** Decompiler normalization and stack
   coordinates require interpretation. Assembly and an executable probe remain
   the authority for a branch's polarity, physical stack slot and runtime path.

The original assembly and existing COMBAT §9.1 already describe two scatter
RNG draws when the radius exceeds one. This trial adds no new mechanic claim.
It reinforces the current alternatives: ours can omit the shot, or reach
initialization with different state. A static slice cannot name the firing
actor or settle which alternative occurs on tick 3439.

Adoption: keep exact-instruction, opcode-specific slices as an occasional
reading aid when pseudocode loses a value's origin. Do not add REA orchestration,
a graph database, a whole-program export or a second analysis engine. The
one-off script remains scratch until repeat use earns a supported tool. The
next productive work is the existing frame-3439 release/state comparison,
not increasing the static graph's limits.

## Shared-state accounting and reproduction

All new scripts, logs, evidence and isolated Java user settings are retained in
`target/rea-value-flow/`. The first sandbox attempt exited 1 because Ghidra could
not open the external project; the approved retry and subsequent queries exited
0. This was a filesystem permission issue, not macOS consent or an engine fault.

Despite program `-readOnly`, Ghidra rotates `idata` and `versioned` project index
files. The first run changed their timestamps and rotated their previous `.bak`
files; that was not anticipated by the flag and was detected by the before/after
inventory. No pre-first backup of those older `.bak` contents was made. Before
subsequent opens, all small project metadata files were copied to distinct
ignored backup directories. Active index contents match the pre-first active
indexes recovered from the first rotation. Program database, program properties
and other non-index files retain their original sizes and modification times;
this is not a full before/after hash of the database. No index restoration was
attempted and no temporary program edits were made. Future read-only headless
trials should preserve these index files before the first open too.

The program's recorded import SHA-256 matches the installed executable. This
improves artifact identity over unbound text alone; it does not prove the age
or derivation of every previously exported file.

Reproduction: `target/rea-value-flow/run_refined.py` records the exact invocation
in `refined-run.json`; use new output/backup names before another run. The current
`ValueSlice.java` takes `entry:instruction:OPCODE` selectors. Earlier unfiltered
script is retained as `ValueSlice-before-refinement.java.txt`.
`python3 target/rea-value-flow/verify.py` verifies the retained results, controls,
identity, explicit unresolved boundaries, truncation and shared-state accounting.

Validation: original-byte comparisons from the first trial, positive/negative
controls and verification script passed. `tools/guard.sh` and `git diff --check`
exited 0; pinned and local links resolved. No release gate or fidelity replay
is claimed.
Coverage remains **3439**, **23 vs 25** draws; run721 remains **951 keys / 146
standing** from the prior validated landing. Independent review remains owed.
