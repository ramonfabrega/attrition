# Separate correction policies and observe Gaia

## Finding

L51's null result was too narrow to explain its own intervention. The viewer
linked Rust state only for units inside the player comparator's scope; Gaia
rows had original state but no Rust state. Meanwhile, `Built::tick` uses each
future figure record both to reseat Gaia animals and to install figure clocks.
Removing those records therefore changes two mechanisms, not clocks alone.

The viewer now links non-player units too, while keeping their comparator scope
false. Official comparison counts and fidelity floors are unchanged. It also
exports typed integer clock fields alongside complete debug records. The branch
report identifies changed units and field values, including changes outside
player comparison scope. A Gaia-only authored regression protects that distinction.

## Controlled policies

`ReplaySession::fork_without_future_clocks` clones the same owned checkpoint and
redacts future `cur_time` payloads. `guy_of` then refuses clock installation;
identities, positions, goals, animation-category predicates, Gaia reseating and
seed corrections remain intact. Clearing entire guy arrays would also change a
walking-category predicate, so the implementation preserves those arrays.
The test restores only `cur_time` and the provenance note and requires the whole
Built debug representation and Report to equal the original checkpoint.
Both intervention methods refuse a vacuous treatment. The frame-95 clocks-only
branch suppresses 545 eligible clock payloads across ten future records.

The existing whole-record policy remains available and explicitly documents its
combined effect. `debug_view --reader checkpoint --corrections compare-clocks`
writes a standard control and `.without-future-clocks.html` treatment. Both
branches restore the same checkpoint in the same process.

## Results

Run69, with the existing trace and four sibling setup inputs:

| Treatment | First changed unit state | Last changed frame | Clock / position / player / RNG / comparator change |
| --- | --- | --- | --- |
| Remove whole future figure records | 95 | 105 (11 affected frames) | None through 3000 |
| Suppress future clock installation only | None through 3000 | None | None through 3000 |

The retained test repeats this experiment from source index 0 and index 94,
following both variants through frame 3000. Each frame compares the complete
unit vector as well as clocks, positions, player units, RNG seed and FrameResult.
Both checkpoints give the table above. This is not whole-Sim equality: world,
player economics and other hidden subsystems are not independently compared
beyond what FrameResult includes. Initial setup still uses the usual corrected
inputs, even at index 0; this does not test an entirely uncorrected simulation.

At frame 95, Gaia unit `8/0` stays at `(17493, 26594)` in both branches.
The standard control has an empty path; whole-record removal retains a segment
to `(17544, 26568)`. Clock fields agree. `reseat_animal` resets movement/orders
and reconstructs a goal when needed, which explains why suppressing reseating
can expose a path difference without a position or clock difference.
The city `0/2000` peasant-distance mismatch survives both policies. No fidelity
score moved, and no correction support was removed from the standard replay.

## Reproduction and limits

```sh
python3 tools/viewer/branch_experiment.py "$RON_INSTALL" "$RON_GAMELOG_DIR" /tmp/new-figures figures
python3 tools/viewer/branch_experiment.py "$RON_INSTALL" "$RON_GAMELOG_DIR" /tmp/new-clocks clocks
```

Each command checks all 130 exported records, frames 95–224, against its own
independent indexed control, verifies original records and source indices,
and hashes six external inputs and the exporter binary before/after. Install
contents are not content-bound. Actual receipts:
`/tmp/attrition-visible-figures-final-20260910/result.json` and
`/tmp/attrition-visible-clocks-final-20260910/result.json`.
Long continuation evidence: `/tmp/intervention-start-probe.log` and the retained
`run69_intervention_effects_are_visible_beyond_player_comparisons` test.

The next step is to measure which corrections overwrite state across existing
captures, then choose a continuation where clocks actually alter a reached state
transition. This case is now a controlled negative for clocks and a
positive for reseating observability; it cannot establish universal redundancy.
The runtime drafts remain paused. No fresh native capture was required.

The Gaia visibility regression was deliberately run with the old scope mask
restored: it failed on the missing Rust coordinates, then the correct source was
restored. Evidence: `/tmp/gaia-visibility-negative.log`. Python checks also reject
floating-point, boolean, negative and overflowing unsigned clock values.

## Full gate

The explicit-install monitored four-thread gate passed 285 rondata tests
(one ignored, zero filtered), 824 sim, 13 fixed, three doctests and 41 Python
checks, plus install survey, clippy, fmt and paperwork guards. Rondata took
126.94 seconds; sampled process-tree peak was 13,364 MiB under 20 GiB.
All 571 observed fixture requests were present. The new continuation test uses
existing captures; no new fixture dependency or suite speedup is claimed.
Evidence: `/tmp/gaia-observability-gate.log` and
`/tmp/attrition-gate-gaia-observability-20260910/fixture-coverage.json`.
