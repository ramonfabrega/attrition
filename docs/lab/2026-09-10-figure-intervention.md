# A controlled figure-correction experiment

## Hypothesis and isolation

Could future figure-clock corrections explain an existing replay discrepancy?
The harness installs figures inside its seed-correction block. Removing seed
records would therefore remove both reseeding and figure corrections, confounding
the experiment. This intervention removes only future `frame_guys` records from
a cloned ReplaySession. Past state, seed records, input cursor and other replay
state remain unchanged. The branch carries a `LAB INTERVENTION` provenance note;
the standard replay is not modified. A branch with no eligible future records
is refused rather than called an experiment.

`debug_view --reader checkpoint --corrections compare-figures` restores the same
owned checkpoint twice per requested window. The control artifact has the normal
name; treatment adds `.without-future-figures.html`. Both use the same open
capture handle. The optional `without-future-figures` mode exports treatment
alone; `standard` remains the default. These are lab interventions, not new
fidelity modes. Existing window bounds, metadata checks and output refusal apply.
A failure can leave a partial control artifact; no successful experiment receipt
is written in that case.

## What happened

At the frame-1900 demo checkpoint the method refused: no future figure records
remain. This says that a future-figure ablation at that point is vacuous, not
that earlier corrections could not influence its state. The refusal and partial
control artifact remain in `/tmp/figure-branches-1900.log` and its named HTML.

The active experiment instead checkpoints before source frame 95 (sim frame 94).
It removes ten future figure records, retaining seed corrections, and compares
130 records through frame 224. **Every exported record remains identical**,
including all 1,704 paired positions, RNG values and typed comparator rows.
At frame 95, city `0/2000` still reports `peasant_dist` original 1, Rust 0.
That mismatch survives this intervention; these future figure inputs do not
explain it over this interval. This is a negative local result, not proof that
figure corrections never matter, that all hidden state agrees, or that the
city mechanic's root cause has been found. No fidelity score moved.

The one-command verifier also runs an independent standard indexed replay and
requires the control's complete records to equal it. Both branches restore one
checkpoint in one exporter process. All six capture/trace inputs and the actual
exporter binary are SHA-256 checked before/after. Install contents remain
external and unbound. The result explicitly allows either a first changed
record or a verified absence of change; it never fabricates a divergence.

```sh
python3 tools/viewer/branch_experiment.py "$RON_INSTALL" "$RON_GAMELOG_DIR" /tmp/new-figure-experiment
```

The actual successful run is `/tmp/attrition-figure-experiment-20260910`.
`result.json` records the intervention, commands, hashes and null first changed
record. Control and treatment HTML remain outside git. Evidence also includes
`/tmp/figure-experiment-final.log` and the earlier `/tmp/figure-branches-95.log`.

## Checkpoint components

The existing run69 checkpoint test additionally timed individual clone+drop
operations, with optimization prevented, at its source-index-1899 checkpoint:

| Component | Clone + drop |
| --- | ---: |
| Sim (mutable state and its owned tables together) | 518.5 microseconds |
| Accumulated Report, 1,899 records | 708.958 microseconds |
| Seed and figure correction vectors | 38.458 microseconds |

The full checkpoint clone measured 1.223 ms. Component numbers are separate
single operations, include dropping their temporary copies, and are not additive
allocation profiles. No mutable/immutable table split is claimed: sim owns both.
The seed vector's allocated capacity accounts for 224 bytes and the recursively
counted figure vectors for 326,288 bytes. Sim's inline struct is 5,720 bytes,
which excludes all owned heaps and is **not** its retained footprint. Report
nested allocations and allocator overhead are not counted. The whole-process
RSS from L49 likewise is not a snapshot-size estimate.

This evidence points first to report-prefix retention if clone cost ever needs
optimization, rather than assuming correction tables dominate. At about a
millisecond today, improving experimental coverage is more valuable than an
unmeasured copy-on-write rewrite. Log: `/tmp/figure-branch-tests.log`.

## Checks and limits

The session test verifies vacuous refusal and that normalizing only removed
figure records and the intervention note restores the exact original Built
debug representation and Report. Seed records remain equal. Existing recorded
continuation tests still pass. Python authored tests distinguish an equal branch
from a changed field and refuse branches with different source indices; both
positive and negative reporting paths are exercised in the offline gate.

The concrete experiment is observational over one interval and the fields the
viewer exports. Next, use a capture interval that exercises an actual correction
of divergent state, or investigate the surviving city residue with a separate
single-policy experiment. Do not generalize this null result or remove correction
support on its strength. Native/runtime drafts and the normal queue stay untouched.

## Full gate

The explicit-install monitored four-thread gate passed 283 rondata tests
(one ignored, zero filtered), 824 sim tests, 13 fixed tests, three doctests
and 39 Python tests, plus install survey, clippy, fmt and paperwork guards.
Rondata took 122.49 seconds; sampled tree peak was 11,647 MiB under 20 GiB.
All 565 observed fixture requests were present. No new capture dependency or
suite-level performance claim is introduced. Evidence:
`/tmp/figure-experiment-gate.log` and
`/tmp/attrition-gate-figure-experiment-20260910/fixture-coverage.json`.
