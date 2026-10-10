# Reuse indexed frames in the shared golden widening

Item 1644, Codex (GPT-6), base `d3c3f625`. Follow-through on
[the shared widening improvement](2026-10-10-frame-reuse.md), on the
investigation branch with the commander paused.

## Scope and contract

`widen_civilians` serves 48 golden tests, including two paired controls:
50 capture walks. It read each frame through `frame_state`, then read it
again for raw fields and a second `Log`. Reuse the first indexed slice and
its eager tree for typed state and auxiliary comparisons. The existing
`IndexedCapture::frame_from_log` enforces exactly one matching frame.

All code outside this helper remains byte-identical, including every caller
and its assertions. The simulation, staging, frame selection, raw near/goods/
ammo readers, turret comparisons, whole-record comparisons and leader checks
are unchanged. Other golden readers are outside this bounded change. No
capture was shortened, merged, recaptured, converted or deleted.

The tree lives for one frame. No cache crosses tests or recording scopes.
As before, indexed sources must be finalized; this is not a snapshot of a
concurrently edited capture. Disk usage of the evidence corpus is unchanged.

## Verification method

`RON_VERIFY_GOLDEN_FRAME_REUSE=1` additionally runs the legacy two-read walk
and compares the complete report: first-difference keys, frames and values;
the full missing-leader-field set; and counts of blocks, record rows, leader
rows, goods, unreadable goods, ammo and no-goods blocks. Both paths retain
all original internal assertions. A missing fixture cannot print success.
The caller still receives its original difference map.

All 48 callers are selected explicitly for the migration audit, using two
test threads under the 20 GiB cap. The final gate leaves the audit flag unset:
only the optimized path may satisfy normal comparison coverage. Independent
review remains owed; this is executable differential evidence, not a blind
review.

The timing group is chapters 35, 36 and 37, three expensive consumers from
the historical suite timing inventory. Before/after use fresh processes,
raw captures, one test thread, identical release debug=1 profiles, and the
20 GiB cap. Two samples per version alternate, without overlapping builds
or other replays. Compare every chapter value/summary diagnostic after
removing only libtest's test-name prefix.

## Evidence and reproduction

Artifacts: `target/golden-reuse-1644/`. `rondata-before` and
`golden.before.rs` preserve the baseline; `measure.py`, `before-*.log`,
`after-*.log`, `measure.log` and `timings.json` record the measured group.
`callers.json`, `audit.py`, `audit.log` and `audit-summary.json` identify and
verify the complete caller set. Scripts create fresh outputs exclusively;
use a new artifact directory for another experiment.

Build: `CARGO_PROFILE_RELEASE_DEBUG=1 tools/memcap.sh 20 cargo test -p rondata --release --lib --no-run`.
Set `RON_INSTALL=/Users/rf-studio/code/fun/attrition/game` for replays.
Run the release test executable with `--exact --nocapture` and the test
names in `callers.json`, setting the audit flag only for differential runs.

## Results

Alternating samples, seconds: before **68.80 / 69.08**, after **65.12 /
65.47**. Median **68.94 → 65.295 s**, a **5.3% reduction** for the complete
three-test group (which also includes the unchanged pool replays). Every
one of **343 value/summary diagnostic lines** matches in all four runs.
Peak tree RSS: before **4438 / 4433 MiB**, after **4431 / 4387 MiB**;
essentially unchanged. A queue-ledger paperwork check overlapped part of
the measured sequence; no build or second replay overlapped it. Two samples
per version are evidence of a modest local gain, not a precise suite forecast.

The complete audit **passed: 48 tests, 50 complete reports equal**, including
both control pairs; **343.53 s**, peak **6606 MiB** with two test threads.
All-target clippy, formatting, diff checks and repository guards passed.
The final required-fixture gate on the committed tree is pending.
No score, value floor,
draw pin or coverage requirement changes: East Indies coverage 3395,
Great Sahara coverage 2323. The commander remains paused.

## Next measured candidate

Source inspection identifies 32 tests that call both `widen_civilians` and
`widen_pool`. Each helper currently restages and ticks the simulation. A
shared pass could retain both complete reports and their independent pins,
but its speed benefit and equivalence are unmeasured. The candidate list is
`target/golden-reuse-1644/pool-followup-candidates.json`. Do not treat matching
run labels alone as proof: compare stem, window, selected player, staging,
frame semantics and all output counters before sharing execution.
