# Reuse checkpoints for nearby diagnostic windows

## Interface and source boundary

The existing `debug_view` example accepts `--reader checkpoint --windows N`,
where N is 1–8. `--from` selects the first frame label at or after the requested
value, as before; `--count` remains 1–200 records per window. Windows are adjacent
by source index. The first artifact uses the requested output path; subsequent
ones use `<stem>.window-1.html`, `<stem>.window-2.html`, etc. Existing destinations
and a requested range beyond the capture are refused before prefix replay.

The driver constructs an enriched ReplaySession with the same siblings, trace
and recording handling as the other readers. It pays for setup and replay to
the first window once, then restores that checkpoint independently for each
window. Later windows still replay the short interval from the checkpoint to
their own start. This is bounded adjacent-window reuse, not arbitrary seeking
or a persistent snapshot cache. Memory/default indexed modes remain unchanged;
multiple windows require checkpoint mode.

The operation owns one open IndexedCapture throughout: CLI checkpoints cannot
be rebound to a different file. Source metadata is validated after prefix and
before/after each continuation. This is handle identity plus ordinary mutation
detection, not content hashing: same-length writes with restored timestamps are
outside this contract. The one-command Python lab demo additionally hashes all
six capture/trace inputs before/after the complete operation and binds the
exporter binary; its successful result is the end-to-end content check. Install
contents remain external and unbound. Failures may leave partial HTML outputs;
only a successfully verified `result.json` certifies the entire demo.

`Window::resume(start, count, next_record)` initializes the observer's global
source cursor. It rejects a cursor after the requested start. This preserves
frame indices and deep links rather than renumbering a restored suffix from
zero. The ordinary Window constructor still starts at source index zero.

## Complete-record validation

The enhanced `tools/viewer/lab_demo.py` retains the full-input run69 witness,
130-record memory/indexed comparison and uninterrupted focused export. It adds
three three-record checkpoint windows starting at source index 1899. All nine
records must equal the corresponding broad export completely, the first window
must equal the independently replayed focused export, and each window must have
exactly three consecutive records at the expected global indices. This verifies
the comparator rows, raw records and observer state, not just the chosen city
field. Repeated/missing windows and changed records are rejected by authored
Python tests; Rust separately checks cursor initialization and refusal.

The actual run at `/tmp/attrition-checkpoint-export-20260910` passed. The witness
remains frame 1900, city `0/2000`, `peasant_dist`: original 2, Rust 0. This is
existing comparator residue, not a new failing acceptance test or score move.
The final extracted verifier was also run against all saved actual windows.

## Measurements and commands

The checkpoint run logged:

| Stage | Time | HTML bytes |
| --- | ---: | ---: |
| Setup + prefix after primary indexing/sibling parsing | 2.0857 s | — |
| Clone, continuation and write window 0 | 19.369 ms | 912,073 |
| Clone, continuation and write window 1 | 20.664 ms | 911,508 |
| Clone, continuation and write window 2 | 22.312 ms | 911,497 |

These single stage timings include HTML serialization/writes for the windows,
but exclude pre-prefix install loading, primary indexing and sibling parsing.
They are not whole-command latency, snapshot allocation size, or a suite
speedup. All stages ran sequentially with existing captures and no native game.
Whole-command demo timings and input/binary hashes are in its `result.json`.

Reproduce the verified four-mode comparison in one command:

```sh
python3 tools/viewer/lab_demo.py "$RON_INSTALL" "$RON_GAMELOG_DIR" /tmp/new-checkpoint-demo
```

Or add `--reader checkpoint --windows 3 --from 1900 --count 3` to the existing
export command, keeping the exact trace/sibling/recording inputs of that replay.
Every new process still starts from setup. The checkpoint is not serialized;
the HTML is not a resumable simulation. Source binding in the low-level
ReplaySession itself remains the caller's responsibility. No original-format
claim, sim rule, queue item or paused runtime draft changed.

## Full gate

The explicit-install monitored four-thread gate passed 283 rondata tests
(one ignored, zero filtered), 824 sim tests, 13 fixed tests, three doctests
and 37 Python tests, plus install survey, clippy, fmt and paperwork guards.
Rondata took 121.70 seconds; sampled tree peak was 11,744 MiB under 20 GiB.
All 565 fixture requests were present. No suite-level speed or memory claim
is inferred from this single validation run. Logs:
`/tmp/checkpoint-export-gate.log`, `/tmp/checkpoint-export-cursor-test.log`,
`/tmp/checkpoint-export-demo.log`; fixture report:
`/tmp/attrition-gate-checkpoint-export-20260910/fixture-coverage.json`.
