# Measure actual correction overwrites

## Why

A correction payload is not evidence that the replay needs it. L52 found 545
eligible future clock payloads but no change after suppressing their installation.
The existing installation count counted calls, including no-op writes. Choosing
an experiment needs the state before and after each correction, not that count.

## Implementation

`Built.correction_audit` is an opt-in diagnostic accumulator, off by default.
`ReplaySession::enable_correction_audit` starts a fresh interval; checkpoints
retain the accumulator. Normal correction policy and sim code are unchanged.
The audit counts attempted writes, actual changes and the first changed sim frame:

- Seed: compare the post-tick word immediately before reseeding against the word
  installed. This excludes draws before initial setup and any uncorrected ticks.
- Gaia reseating: compare the complete Unit immediately around `reseat_animal`,
  before clock installation. Position agreement alone cannot hide order/path resets.
- Clock: compare the actual Guy before and after `set_guy`, including insertion
  and the follow state that the setter preserves. Missing clock payloads are not
  counted as attempted installations.

Unlinked and walking-predicate-skipped unit records are counted separately.
Each family retains only its first changed witness, rather than a per-tick log.
The Gaia witness prints position/path/orders; the change predicate covers the
whole Unit, including fields not printed in that short witness. Auditing clones
Gaia units at reseating calls; disabled auditing does not make these copies.
No speed or whole-heap-size claim is made.

The release example `correction_audit` uses indexed setup and observations for
primary and sibling captures. It runs the ordinary ReplaySession with the
optional recording stream, paired with an unaudited clone. Every FrameResult
must agree. At completion, removing only the audit field must restore exact
Built debug equality and complete Report equality. Source and sibling metadata
are validated around the operation. It prints success only after these checks.
This is a measurement tool, not a policy that removes corrections.

## Measured saved captures

Counts are **changed / attempted** writes, not distinct units or changed frames.

| Primary capture | Records (last source frame) | Seeds | Gaia reseats | Clocks | Predicate skips |
| --- | ---: | ---: | ---: | ---: | ---: |
| run69 Great Lakes | 3000 (3000) | 0 / 14 | 7 / 560 | 0 / 748 | 12 |
| run33 Great Lakes | 1800 (1800) | 0 / 14 | 7 / 560 | 0 / 748 | 12 |
| run39 East Indies | 1800 (1800) | 0 / 1 | 0 / 104 | 0 / 116 | 1 |

All three paired controls agree; none has unlinked correction units. Run69 and
run33 use the same four canonical sibling correction inputs, so these are not
independent confirmations of the same seven writes. Run39 uses run38's start
dump. Each command also supplies its corresponding trace. Trace availability
does not imply every tick is forcibly reseeded: only `frame_seeds` entries drive
that block in the existing harness.

The first actual change is sim frame 94, visible as source frame 95 after the
tick: Gaia unit `8/0`, unchanged position `(17493,26594)`, path segment to
`(17544,26568)` cleared, and its move order reconstructed with different flags,
angle and waypoint status. This independently locates L52's effect at the
reseating write itself. Seven writes and eleven affected continuation frames
measure different things; they are not contradictory counters.

These intervals give no positive clock-overwrite candidate. They do not prove
that later captures, other setups or future code never require those corrections.
Initial state is still enriched from the ordinary external inputs. No fidelity
floor or completion score moved.

## Reproduction and evidence

```sh
cargo run --release -p rondata --example correction_audit -- \
  "$RON_INSTALL" "$CAPTURE" 1800 --sibling "$SIBLING" --trace "$TRACE"
```

Repeat `--sibling` for the required setup family; `--recording REC` is optional.
A positive record count within the indexed capture is required; no silent short
window is accepted. Keep capture-derived stdout outside git.

Final commands, external-input and executable SHA-256 hashes, and paired results
are under `/tmp/attrition-correction-audit-final-20260910/run{69,33,39}.{json,txt}`.
The runner verified these hashes before and after each command. The CLI itself
performs metadata checks, not cryptographic binding; install contents remain
unbound. This is a three-capture survey, not an exhaustive corpus audit.

The retained run69 test checks that auditing preserves final Built and Report,
observes real reseat changes, and sees attempted but unchanged clocks. The authored
clock test distinguishes insertion, a no-op write and a later overwrite, retaining
the first witness only. It exercises the same installation helper as the harness.

## Next investigation

Investigate the seven actual Gaia reseating writes: whether each is necessary
state repair or an incidental reset introduced by the harness. Compare changes
one write at a time from a checkpoint and follow RNG and player effects through
the next correction boundary. The current audit establishes where to intervene;
it does not establish that those resets should be removed.

## Full gate

The explicit-install monitored four-thread gate passed 286 rondata tests
(one ignored, zero filtered), 824 sim, 13 fixed, three doctests and 41 Python
checks, plus install survey, clippy, fmt and paperwork guards. All 571 observed
fixture requests were present. Rondata took 124.52 seconds; sampled process-tree
peak was 12,192 MiB under 20 GiB. These are validation measurements, not an audit
performance benchmark. Evidence: `/tmp/correction-overwrites-gate.log` and
`/tmp/attrition-gate-correction-overwrites-20260910/fixture-coverage.json`.
