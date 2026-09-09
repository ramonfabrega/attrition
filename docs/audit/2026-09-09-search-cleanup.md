# A bounded allocator boundary for suspended-search cleanup

## Result and scope

The original's suspended-search cleanup can run on a small authored pointer
graph without mapping the whole executable, a heap, or the PathNode payloads.
`tools/explore/search_cleanup_oracle.py` executes the original instructions;
it does not emulate allocator calls or translate the decompiled cleanup body.

This is a synthetic open-tree teardown experiment. It does not establish that
these inputs occur naturally, capture a suspended search, resume one, or compare
Rust search results. `crates/sim/src/path.rs` still explicitly discards suspended
search state; this work does not change that seam or move a parity score.

## Evidence and checks

Fixture layouts are tied to PDB records and instruction operands in
`docs/FORMATS.md`, “Suspended-search cleanup fixture”. The reading selected a
bounded branch: open trees of zero through 31 nodes; balanced, left-chain and
right-chain shapes; every combination of the four optional empty containers;
and four initial recycler occupancies. The default 6,144 cases cover that
Cartesian product. All node identities are unique and payload pointers are
opaque authored tokens. Metrics are randomized sentinel words, not evidence of
a valid priority ordering or a reachable search state. No payload object is mapped.

The checker verifies every declared non-scratch byte, preserving unrelated unit
words, existing pool entries and unused pool capacity. Every reachable tree node
and payload must enter its respective pool exactly once. Every active container
must enter its own pool exactly once. Node links/data and specified unit state
must clear, and callee-saved registers and stack balance must survive. Recycler
append order is not prescribed by the ownership assertion; full registers,
memory, scratch and write histories must nevertheless match reverse-order reset
runs and independently constructed fresh emulators for every case. Only digests
are retained across the sweep, not the full histories.

Each of the 31 declared regions is removed individually from a maximal fixture
and must cause failure. Each of the seven recycler pools is separately made
full and must fail when it tries to read the undeclared growth increment. After
each failure, the reused engine must reproduce the entire clean baseline. Corruptions of native output must fail the ownership and
complete-record checker: an unrelated unit word, retained node data, tree length,
pool length, missing returned payload, existing pool entry, unused capacity and
return address. Those controls are run against real successful native output.

## Architectural implication and limits

The original's recycler bookkeeping can be included in explicit reset state.
The engine's allocator need not become a host allocator, and payload objects
need not be copied merely because their pointers are returned to a pool. That
is a concrete way to constrain capture size and make reuse falsifiable.

This does not yet measure savings against a live suspended-search capture or
whole-game memory. Pool growth, nonempty closed/reference/valid/block trees,
shared or cyclic ownership, search expansion and live capture perturbation are
outside the contract. Bounds reject unexpected accesses; no missing dependency
is filled with guessed data. Next, capture the natural graph with explicit size
limits and compare entry/exit ownership before widening to search resumption.

## Reproduction

```sh
uv run --offline tools/explore/search_cleanup_oracle.py /path/to/install \
  > /tmp/search-cleanup-report.json
```

Reports contain executable identity and original-derived observations and stay
outside the repository. The probe never launches the game or changes settings.

## Measurements and retained evidence

`/tmp/search-cleanup-full.json` records the complete 6,144-case sweep: 5,208
declared bytes in 45,056 mapped bytes (44 KiB), no mapped payload bytes, and
a maximum of 1,851 executed instructions. The checked initial sweep took
11.459 seconds, reverse reset sweep 11.320 seconds, and fresh-engine sweep
15.425 seconds. These are single local wall measurements including fixture and
result processing; fresh/reverse sweeps compare full-result digests, while the
initial sweep also runs semantic checks. They are not isolated emulator timing,
host RSS measurements or whole-game speed comparisons.

`/tmp/search-cleanup-controls-final.json` records the expanded 31 missing-region,
eight corrupted-output and seven growth/recovery controls. Its one-case sweep
is only a smoke run; its controls always use the maximal 31-node fixture.
The complete sweep preceded this expansion of controls; fixture generation and
semantic verification were unchanged. Both reports identify the executable by
SHA-256. No original bytes or generated report enters the repository.

## Next capture contract

The existing `live_path_capsule.h` rejects any nonzero open-list pointer, so
its retained witness cannot validate this branch. Simply relaxing that check
would produce an incomplete capsule. A separate hook at cleanup entry needs
the five container headers, bounded reachable nodes and each recycler's
list/capacity/length plus affected slots, followed by an exit ownership witness.
It must count rejected candidates by reason: unsupported nonempty containers,
node/stack limits, graph aliases/cycles and insufficient pool headroom. A run
with only rejections is coverage information, never a passing replay.

Start with a size census and the exact subset proven here. Preserve identities
across capture and replay, exclude dead live scratch as in the prior capsule,
and require a control run's full logged frames/RNG before calling interception
transparent. Expand nonempty closed/reference trees only after their distinct
ownership rules are independently exercised; copying all pointer-reachable
payloads would discard the narrow dependency boundary this experiment measures.

Validation: full release gate passes (269 rondata, one ignored; 822 sim;
13 fixed; three doctests), rondata 231.10 seconds, process-tree peak 9,041 MiB
under 20 GiB. Clippy with warnings denied, formatting, documentation guards,
the install survey and 11 bounded-executor tests pass. The release log is
`/tmp/search-cleanup-release.log`.
