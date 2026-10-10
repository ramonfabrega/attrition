# Nine Great Lakes windows, one execution

Item 1642, Codex (GPT-6), starting at `19eea5c9`. User-authorized execution
consolidation, one worker; commander remains paused. This extends
[item 1640's four-window consolidation](2026-10-09-capture-execution.md).

## Contract

The existing shared run202/211/218/226 walk already simulates from the same
run53 start through block 17350. Five separately registered tests repeat
prefixes of it: run163, run174, run178, run192 and run196. They all compare
from block 11400, read player-1 pool lists from 11800, use the same siblings,
and disable the additional group-record mode. Their endpoints are 12399,
12899, 14899, 15039 and 15232. Their six-, seven-, eight-, nine- and ten-entry
capture chains are exact prefixes of `great_lakes_word_chain()`.

Across these six old executions, the loop bounds imply **87818 sim ticks**
and **19424 compared blocks**; the shared execution uses **17350 ticks** and
**5951 compared blocks**. These are counts from the validated window bounds,
not profiler estimates. The reports still cover all nine original windows.

The combined test is now
`diff::harness::tests::great_lakes_run163_through_run226_are_widened_whole`.
It takes five more cumulative snapshots during the existing walk and passes
them to the original assertion functions. All nine assertion bodies remain
byte-for-byte identical (69 pin expressions). Each report retains its own endpoint, observation
points, first parting rows, missing-field set, block and leader-row counts,
animation changes, housed count, standing rows and army-list field. Every
assertion group runs even if an earlier group fails; failures name the group.
No sim code, source reader, capture, comparison function or floor changes.

The five former test names and the previous combined test name no longer
register individual tests. Exact-name invocations must use the combined name.
Historical journals and timing records remain unchanged. The new test always
runs all nine assertion groups. This reduces registered test count by five,
not assertion coverage.

## Equivalence and failure checks

`RON_VERIFY_SHARED_WIDENING=1` additionally replays each original window
independently. The first five use their original shorter chain prefixes;
the last four retain their existing full chain. The audit compares selected
source entries at every block (including seams), then compares the complete
`Widened` value, including fields not pinned by the original tests.

For each report, three fault injections must be rejected: an absent field,
one fewer compared block, and an altered value row. run163/run174/run178
mutate existing first-parting values pinned verbatim; the other six add an
unexpected standing row at a pinned observation point. The audit is opt-in
so ordinary gates do not restore the redundant execution being removed.

Pre-migration structure verification resolves the old local constant aliases
and compares their chain entries, endpoints, pool threshold and observation
points against the new requests. It also checks all nine assertion bodies
for exact text equality. Evidence is retained in
`target/consolidation-1642/{harness.before.rs,check_structure.py,structure.json}`.

## Measurement and validation

The baseline is the five original tests plus the existing four-window shared
test, in one release process with one test thread. Both measurements use
`CARGO_PROFILE_RELEASE_DEBUG=1`, the same installation and raw captures, and
`tools/memcap.sh 20`. No builds ran concurrently with timed replays.

| Execution | Test elapsed | Peak process/tree MiB |
|---|---:|---:|
| Before: six executions, nine assertion groups | 154.44 s | 1605 |
| After: one execution, nine assertion groups | 54.39 s | 1392 |

This is **100.05 s / 64.8% less time**, or **2.84× faster**, for the selected
group, and 213 MiB less peak memory (13.3%). It is one before/after pair,
not a statistical estimate or a whole-suite speed claim. Both runs passed.
The before binary is retained as `target/consolidation-1642/rondata-baseline`;
logs are `baseline.log`, `shared.log`, `build-measurement.log`.
The first compile (`build.log`) used the normal release profile; the timed
after run used the matching debuginfo profile from the second compile.

The executable migration audit **passed, exit 0**: all nine complete reports
are equal to independent replays, all per-block source selections match, and
all **27 injected faults** were rejected. `equivalence.log` contains expected
caught panic diagnostics for those faults; the enclosing test passed. Audit
elapsed 277.76 s, peak 1603 MiB; this is not a performance comparison because
lint/guard validation overlapped part of it. All-target clippy (`-D warnings`),
`tools/guard.sh`, formatting, diff and local-link checks passed. Structure
checks remain reproducible from the retained script and pre-edit source.

Reproduction after building the rondata release test binary:

```sh
RON_INSTALL=/Users/rf-studio/code/fun/attrition/game \
  tools/memcap.sh 20 cargo test --release -p rondata \
  diff::harness::tests::great_lakes_run163_through_run226_are_widened_whole \
  -- --exact --test-threads 1
# Add RON_VERIFY_SHARED_WIDENING=1 for the independent replay/fault audit.
```

Draw scores remain East Indies coverage 3395 and Great Sahara coverage 2323;
value floors are unchanged. Single-agent executable evidence does not settle
independent review debt.

## Full release verdict

Committed **`6443a80e`**, tracked tree frozen: required-fixture gate **exit 0**,
all six steps ran. Offline 277; fixed 13; rondata **777 passed**, five existing
ignored; sim **1476 passed**; three doc tests. Broad key/field coverage and the
new shared test passed. Registered test count falls by five because those
assertion groups now execute inside the combined test; none was dropped.

Rondata suite elapsed **566.31 s**, compared with the previous gate's
**592.85 s** (26.54 s / 4.5% shorter). These are two observed four-thread runs,
not a controlled repeated suite benchmark. Peak tree/largest-process RSS
**12885/12852 MiB**, versus **12935/12902 MiB** previously: no material
whole-suite memory reduction established. Sim elapsed 2.25 s.

Fixture audit: **2893 requests across 386 unique fixtures, zero missing**.
The fixture-name set exactly matches the previous landing's 386; repeated
requests fell from 2963 by 70. This covers observed requests, not every corpus
file. Evidence: `target/consolidation-1642/release-gate.log`, `gate-report/`,
`fixture-set-comparison.json`. No archive overrides were used.

The next candidate is read/parse reuse within the remaining walks. Later
Great Lakes windows have different comparison starts and deliberate gaps;
sharing them requires separate window accumulators and its own equivalence
proof. Neither change is included here.
