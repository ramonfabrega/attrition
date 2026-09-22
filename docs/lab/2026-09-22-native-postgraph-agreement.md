# Native saved-search graph agrees on required fields

One authorized native limit-95 call now agrees with the modeled replay across
**all required fields in 539 saved-search graph records**, under the pointer
correspondence committed before capture. This extends the earlier unit/path
witness to tree nodes, PathNodes, opaque CollBlock bytes and occupied recycler
state. It is still not full-byte or full-engine equivalence: **102 model-unknown
bytes remain unverified**, and **four known bytes in an unused recycler slot
differ**. Strict known-byte agreement correctly fails.

Work starts from `5aa8aab` on `codex/saved-tree-lab`, using the unchanged
[L74 collector and adapter](2026-09-22-native-postgraph-preparation.md) and
[L73 comparison contract](2026-09-22-graph-correspondence.md). No implementation
or comparison rule changed after seeing the native result. PR #5 remains frozen;
no main score moved and no external API was used.

## Capture and provenance

Ramon authorized the prepared approximately 30-second run. Map 14, seed 12345,
end frame 1400 and the exact L70 command stream were retained; the only new
tracer option was `RON_RESTORE_POSTGRAPH`. The game ran for 21.02 seconds;
build, launch and restoration took 29.04 seconds in total. It exited zero at
closing frame 1401, all five backed-up settings files were restored, the runner
released its cooperative lock, and Ramon was told the lane was free immediately.
No second capture was run or booked.

At frame 224, the post-graph packet is 26,036 bytes including its paired
unit/path witness; the embedded graph is 25,184 bytes. Acquisition reports
zero milliseconds at the collector's timer resolution, not zero execution cost.
All prefix, input intervention/restoration, return, unit/path and post-graph
receipts validate in order. The native postgraph SHA-256 is
`cfa7e7759a94dc2733763bc37be8b7b029ae089e6983426601d1a919cf474a8f`.
The broad input payload SHA-256 is
`df96361121222179912a6ede1b72e265c8593a4fc64f0e7e0a01cf7f0c5fe6e2`.
Source, staged source, traced executable and tracer hashes were measured and
bound to the runner receipt before replay.

The native function returns -1. All 344 unit bytes and all 160 path-capacity
bytes agree exactly; there is no unit-pointer exception in this intervention.
Path length/capacity remain 1/10. The modeled limit-95 execution takes 165,934
instructions and restores its unchanged control after the experiment. The three
borrowed objects are independently derived from this capture's headers: the
160-byte path and two 256-byte recycler arrays. Old capture addresses were not
reused. No additional native capture was needed to close dependencies.

## Graph result

| observation | measured result |
|---|---:|
| complete records / declared bytes | 539 / 16,528 |
| tree nodes / PathNodes / CollBlocks | 392 / 123 / 4 |
| logical tree lengths | 90, 90, 33, 142, 4 |
| physical tree lengths | 90, 123, 33, 142, 4 |
| paired records whose address differs | 99 |
| relocated typed pointer fields agreeing | 246 |
| bytes known on both sides | 16,426 |
| model-unknown positions | 102 |
| raw differing known bytes | 985 |
| remaining required differences | 0 |
| remaining optional differences | 4 bytes in one word |

Correspondence comes from anchored roles and labeled ownership links, not a
value-fitting search. Every native byte is retained. Model unknowns remain nulls
at +22/+23 in 51 BR-tree nodes even though native values exist at those positions.
Consequently `definedness_equal`, `all_bytes_known` and `known_bytes_agree` are
false; `required_fields_agree` and `native_compared` are true. Those are distinct
claims, and only the last required-field assertion is the successful fidelity
result here.

The four differing bytes are recycler owner 5's first capacity word at
`0x32a40888`. Native stores `0x3521f15c`, model stores `0x17fa0b74`; the derived
map pairs those two observed open-tree nodes. But the recycler's occupied length
is zero. As promised by L73, unused capacity is compared literally rather than
silently pointer-normalized. All four bytes remain in the difference report.
This is the same kind of allocator-history difference exposed by the controlled
relocation experiment, now observed against the original.

## Falsifiers and limits

The unmodified native comparison passes `--require-required-fields`. Applying
the strict known-value assertion to that same result fails. Changing one
PathNode payload byte in a copy of the model report also fails the required-field
assertion while its native unit/path witness is unchanged. This checks that the
new tree evidence is actually tested, rather than inferred from the older path
agreement. Same-run execution and graph observation repeat exactly, observation
leaves the model fingerprint unchanged, and the unchanged control restores.

This is one suspended continuation, with success-path allocator and memory-service
models. It does not establish all mutated game state, arbitrary future calls,
allocator metadata or allocation-failure behavior, red-black balancing or key
ordering beyond the observed records, native limit 96, or imported extended
state. The graph acquisition is bounded and checks the unit view at both ends;
it is not a proof of atomicity against all other threads. Unknown bytes have
not been promoted into evidence, and the opaque records are compared only as
complete byte extents in this witness.

The useful next step is a steering-selected question for this validated call
harness, with one named input/field and a native falsifier. More generic runtime
reconstruction is not required to use this result. Fable decides whether this
method merits a small pilot, further evidence, or parking; this branch is not a
request to merge the main loop's policy or expand the fidelity claim.

Capture and replay data remain outside git:
`/Users/rf-studio/ron-data/lab-captures/2026-09-22-native-postgraph/map-14` and
`/Users/rf-studio/ron-data/lab-experiments/2026-09-22-native-postgraph-agreement`.
The source in git is authored tooling and this report only. The lane is released.

A second fresh replay process reproduces the complete graph, native unit/path
witness, unchanged baseline and all 41 dependency-expansion steps; only elapsed
wall time differs. The expansion adds 161,956 captured bytes; no region is added
during the prepared intervention itself.

The final release gate exits zero: 332 rondata, 855 sim, 13 fixed-point and
three doc tests; all 782 fixture requests present; clippy, formatting, install
survey and paperwork checks pass. Peak process-tree memory is 8,057 MiB.
The final validation-note edit is followed by the fast guard and whitespace
check. No implementation changed during this native validation landing.
