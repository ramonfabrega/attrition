# Isolate individual Gaia reseating writes

## Outcome

The seven writes found by L53 are one animal (`8/0`) on sim ticks 94–100.
Suppressing six individually changes unit state for one source frame each.
Suppressing tick 99 changes state for **556 source frames, 100–655**, even though
positions, clocks, player-unit state, RNG seed and the existing FrameResult
remain identical throughout the remaining capture.

This longer difference includes an original-observed fidelity loss. At source
frame 106, the dump's own unit angle is `756678656`; the standard control matches,
while the tick-99 treatment has `738852864`. These are the existing raw integer
angle representation, compared directly as the unit comparator does. No inferred
conversion or decompiler reading is needed. The retained test asserts control
agreement and treatment disagreement with that field.

Thus a correction can matter to Gaia fidelity while leaving the player comparator
unchanged. This is evidence for retaining the standard correction, not removing it.
The experiment changes no sim rule or headline floor.

## Isolation

`ReplaySession::fork_without_gaia_reseat(ReseatKey { frame, who, o })` clones an
owned checkpoint and installs a one-write skip selector. It skips only the
`reseat_animal` call at that key; the same unit's clock installation, every other
unit and frame, and seed corrections retain their standard paths. A hit counter
must equal one after continuation. The method refuses a past, non-Gaia,
ambiguous or seed-gated-out key and refuses stacking another skip selector.
Structural eligibility is checked at fork; actual execution is checked by hits.

The audit retains up to 128 changed reseat keys in execution order and counts
omissions explicitly. The experiment refuses an incomplete list. This bounds
identity retention without silently treating truncation as full coverage.
Suppressed calls do not count as attempted writes in an enabled audit.

All seven treatments start from the same checkpoint before sim tick 94. At fork,
normalizing only the selector and its provenance note must restore exact Built
and Report equality. Each is run beside its own uninterrupted control through
all **3,001 source records, ending at frame 3001**. Prior L52/L53 experiments
intentionally stopped at 3000; they had not covered the final record.

## Measured effects

| Skipped sim tick | Unit | First changed source frame | Last changed source frame | Changed frames |
| ---: | --- | ---: | ---: | ---: |
| 94 | 8/0 | 95 | 95 | 1 |
| 95 | 8/0 | 96 | 96 | 1 |
| 96 | 8/0 | 97 | 97 | 1 |
| 97 | 8/0 | 98 | 98 | 1 |
| 98 | 8/0 | 99 | 99 | 1 |
| 99 | 8/0 | 100 | 655 | 556 |
| 100 | 8/0 | 101 | 101 | 1 |

The first differences include the retained path and move-order flags, angle and
waypoint status that reseating normally reconstructs. In the long-lived branch,
from frames 102–655 the complete Unit difference is confined to four movement
fields: `facing`, `frame_facing`, `heading`, `des_angle`. Normalizing only those
four fields restores Unit equality on every one of those frames; a test protects
that observation. Position-only or clock-only comparison cannot detect it.

At frame 3001 all seven branches converge to the control's complete Built state
(after removing only the skip selector and its provenance note). Their complete
Reports also agree after removing that same provenance note. This is final-state
convergence, not hidden-state equality throughout the interval. Unit membership
is included in the clock, position and player-state comparison projections.

## Checks and reproduction

```sh
RON_INSTALL=/path/to/owned/game cargo test --release -p rondata \
  run69_single_reseat_interventions_follow_the_remaining_capture -- --nocapture
```

This uses the existing run69 capture, trace and four canonical siblings. No Wine
launch or new capture is needed. Source metadata is validated; this retained
fixture test is not cryptographic install or capture binding. Outputs stay outside
git. The test pins the seven keys, hit counts, changed-frame intervals, unchanged
projections, angle-only residue, original-angle witness and final convergence.
It also checks ambiguous and seed-disabled refusal. A separate authored test
checks identity-list truncation reporting.

Evidence: `/tmp/single-reseat-final-test.log` (individual outcomes and original
angle witness), `/tmp/single-reseat-detail.log` (full unit snapshots used to locate
the four angle fields), and the retained test. Intermediate probes exposed a
3,001-record capture and a provenance-only Report difference; the final check
handles both explicitly instead of suppressing other report differences.

## What remains open

The test proves that tick 99 matters to the observed heading at frame 106. It
does not establish that all seven writes are individually necessary, that any
are universally redundant, or why the next state transition at frame 656 erases
the orientation difference. It also does not test simultaneous removal: effects
of individual interventions need not add together.

The next useful direction is to promote non-player orientation into a broader
optional diagnostic diff, keeping it separate from the established player score.
The original already prints the needed angle in this capture. That gives the
lab an oracle for state that its current headline intentionally excludes.

## Full gate

The explicit-install monitored four-thread gate passed 288 rondata tests
(one ignored, zero filtered), 824 sim, 13 fixed, three doctests and 41 Python
checks, plus install survey, clippy, fmt and paperwork guards. All 577 observed
fixture requests were present. Rondata took 129.37 seconds; sampled process-tree
peak was 12,765 MiB under 20 GiB. These are validation measurements, not a speedup
claim. Evidence: `/tmp/single-reseat-gate.log` and
`/tmp/attrition-gate-single-reseat-20260910/fixture-coverage.json`.
