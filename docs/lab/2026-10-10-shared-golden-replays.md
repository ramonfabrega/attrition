# Share golden unit/leader and group-pool execution

Item 1645, Codex (GPT-6), base `8668a659`, continuing the user-authorized
optimization work. One worker, commander paused. Builds on
[frame reuse](2026-10-10-golden-frame-reuse.md).

## Contract

32 golden tests called `widen_civilians` then `widen_pool`. Every candidate
used the same run, script stem, frame window and player (who=0). Each helper
stood up the same scenario, staged the script, ticked to the same window,
and read the same indexed frames. The new `widen_civilians_and_pool` keeps
one staged simulation and observes both reports from each frame's eager tree.

The civilian observer still compares units, figures, buildings, cities,
leaders, goods and optional ammo. `PoolWidening::observe` extracts the old
pool comparison loop: 512 records required in each block, all 64 selected
player slots in both directions, membership, scalar fields and member
positions/angles. The report retains the first-difference map and block,
record and row counts. Every observed frame asserts agreement between the
civilian loop clock and the simulation clock used by the old pool walk.

The 32 caller bodies are unchanged apart from replacing their two helper
invocations with one returning both maps (verified after whitespace
normalization). All original pins, diagnostics and maps remain. The pool
summary prints earlier now; diagnostic equality is compared as a multiset,
including repeated lines. No sim logic, parser, source selection, capture,
window, script, score or floor changes. No global or cross-test cache.

## Differential validation

`RON_VERIFY_SHARED_GOLDEN=1` also executes independent civilian and pool
walks and asserts both complete reports equal the shared walk. The civilian
reference uses the legacy two-read parser path; pool uses its original lazy
parser path. Compare exact keys, values, first-difference frames, missing
leader-field names, all seven civilian counters and all three pool counters.
The prior `RON_VERIFY_GOLDEN_FRAME_REUSE` flag remains supported.

All 32 callers must finish their audit: require 32 distinct success markers,
64 reports, and all original test assertions. A missing capture cannot
produce a marker. Normal release gates leave every audit flag unset so
legacy execution cannot satisfy comparison coverage. The extracted pool
comparator is shared by audit paths; retained baseline-binary diagnostics
and the source-equivalence check separately constrain that extraction.
Independent review remains owed.

## Measurement and fixtures

Chapters 35–37, two alternating fresh-process samples per version, raw
fixtures, one test thread, release debug=1, 20 GiB cap. Retain the old binary
before rebuilding; do not overlap timing with another build or replay.
Compare all chapter value and count diagnostics with multiplicity preserved.
Before **64.43 / 64.63 s**, shared **48.90 / 48.24 s**: median
**64.53 → 48.57 s**, **24.7% less time**. All **343 diagnostic lines**
match as multisets in all four runs. Peak tree RSS before **4432 / 4438 MiB**,
after **4520 / 4358 MiB**: essentially flat across these small samples.
No build or second replay overlapped timings. The complete audit **passed:
32 tests, 64 complete reports equal**, **285.46 s**, peak **6019 MiB** with
two test threads. All-target clippy and repository guards passed; full gate
on the implementation commit passed below.

The existing 32 window constants imply **32 repeated setups and 56,588
repeated simulation ticks removed** from normal execution. This is a count
of loop iterations removed, not a claim about instructions or draws removed
from the simulated scenarios. See `saved-ticks.json` in the artifact directory.
All scenarios still run their full original windows and both comparisons.

The release fixture recorder covers `testenv::dump`, but `golden` has a
separate lookup. In addition to the required-fixture gate and complete audit
markers, explicitly verify the 32 golden dumps and 32 traces are nonempty,
and check their size/mtime manifest again after the gate. Do not claim the
archived-fixture report alone proves golden-corpus completeness. Removing
32 duplicate setups will reduce repeated archived sibling fixture requests;
the unique fixture set must remain unchanged.

## Evidence and reproduction

Artifacts: `target/golden-shared-1645/`: retained `rondata-before`,
`golden.before.rs`, migration script and caller source check, `callers.json`,
`measure.py`, timing logs/results, `audit.py`, audit log/summary, and golden
fixture manifests. Scripts create new output files exclusively; use a fresh
artifact directory for a rerun.

Build with `CARGO_PROFILE_RELEASE_DEBUG=1 tools/memcap.sh 20 cargo test -p rondata --release --lib --no-run`.
For replays set `RON_INSTALL=/Users/rf-studio/code/fun/attrition/game` and
run the resulting test binary with `--exact --nocapture` plus the names in
`callers.json`. Audit with `RON_VERIFY_SHARED_GOLDEN=1`, two test threads;
normal gate with that flag unset and `--require-fixtures --test-threads 4`.

Scores remain East Indies coverage 3395 and Great Sahara coverage 2323;
no value-floor or draw-pin movement. Original captures remain intact.


## Full release verdict

The full required-fixture gate on **`e7eb12f3` passed, exit 0**, all six
steps; the tracked tree remained frozen. Offline 277, fixed 13, rondata
**778 passed / five existing ignored**, sim **1476 passed**, three doc tests.
All 32 consolidated tests and parsed-field coverage passed with legacy audit
flags disabled.

Rondata **520.22 s**, preceding gate **542.66 s** (−22.44 s / 4.1%). This
single four-thread pair is an observation, not a repeated suite benchmark.
Sim 2.22 s. Peak tree/largest process **13526 / 13495 MiB**, preceding
**14175 / 14143 MiB**, under the 20 GiB cap. Focused memory stayed roughly
flat; no causal suite-memory improvement is established.

Fixture audit: **2765 requests, 386 unique, zero missing**. Compared with
2893 before, exactly one request for each of four sibling dumps is removed
from each of the 32 consolidated tests: **128 duplicates**, with identical
fixture/test-pair coverage. The separate **64 golden dump/trace files remain
nonempty with unchanged sizes and mtimes**. This does not claim completeness
of unrequested corpus files. Evidence: `release-gate.log`, `gate-report/`,
`fixture-set-comparison.json`, and `golden-fixtures-{before,after}.json` in
`target/golden-shared-1645/`. About 72 GiB remains free.

Next experiment: bounded initial-state loading in `stage_script`, reusing
the established indexed reader. Verify complete Initial fields and cold/warm
behavior; prior road/leader migrations show memory benefits can trade against
cold time. No such gain is claimed for golden setup yet. Independent review
remains owed; commander remains paused.
