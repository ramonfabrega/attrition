# Select clock-bearing units before decoding observations

Replay initialization now rejects units without clock fields before constructing
full typed unit records. In a paired run69 measurement, source loading plus
3,000-record replay falls from 3.579 to 2.844 seconds for the in-memory reader
and from 5.100 to 4.396 seconds for the indexed reader. Complete reports remain
identical. No simulation arithmetic, oracle correction policy or queue score
changes.

## Consumer boundary

The phase probe from `2026-09-09-replay-setup-ranges.md` showed zero retained
clock frames on run69. Nevertheless, observation collection decoded every unit,
building and leader on every frame, converted units into figure rows, and then
rejected rows without clocks. This work was separate from the frame comparison
pass, which still needs complete records.

The new observation path shares the complete parser's unit-block iterator:
direct UNITDATA children first, then ANIMALDATA wrappers' UNITDATA children.
The first direct FULL DUMP child supplies records when present, as before.
Building and leader decoding is omitted from observation collection because
neither product was consumed there. The initial state and frame-comparison
paths continue to decode complete records.

For replay-only observations, a unit is decoded only when a direct GUY child
has integer-valued `cur_time`, `end_time` and `cur_anim`, matching the existing
`Guy::has_clock` predicate and the existing integer accessor. Zero is a value;
a malformed or absent field is not. A selected unit still uses the complete
`unit_of` decoder: every figure, including unclocked figures, position, order
and goal needed by the old FrameUnit conversion is retained. Object validity
checks still run there. The final typed clock filter remains in place.

The public full initializer bypasses this prefilter so its figure-position
audit series remains complete. Animation-length traversal and frame-seed
collection still run for all observed blocks, independently of clock selection.
This is not a prefix-only initializer or a new interpretation of original data.

## Measurement and evidence

A saved binary from `c4fe241` and the new binary ran sequentially with the same
467,769,059-byte main run69 capture and install, with no siblings, trace or
recording. Timers include source loading/indexing and initialization/replay,
excluding install loading and buffered report serialization. `/usr/bin/time -l`
measured process peak RSS; a 20 GiB watchdog covered the experiment. A short
clippy check overlapped the beginning of the experiment; these single paired
samples are indicative, not controlled statistical estimates.

| reader | revision | load + replay seconds | peak RSS bytes |
| --- | --- | ---: | ---: |
| memory | previous | 3.579 | 566,902,784 |
| memory | selective | 2.844 | 568,082,432 |
| indexed | previous | 5.100 | 114,343,936 |
| indexed | selective | 4.396 | 115,752,960 |

The observed time reductions are approximately 21% and 14%; there is no claimed
memory improvement. Both complete 3,000-record reports compare byte for byte
against the saved baseline. Additional baseline/new indexed reports match on
run13 (10 records), run20 (5 records) and run82 (20 records), covering captures
with clocks and full dumps. Those verification runs overlapped the release
gate, so their timings are not used as performance evidence. Clock-heavy inputs
still decode selected units and now perform a preliminary clock-field check;
a universal speedup is not established.

The synthetic differential test compares selected rows to the complete-record
parser followed by its typed clock filter. It exercises missing, incomplete,
malformed and zero-valued clocks; mixed clocked/unclocked figures; animal
wrappers; direct and nested FULL DUMP records; and the full-audit bypass.
The canonical four-sibling viewer export also retains identical full records
and notes across 30 frames and 450 paired positions. Existing complete-setup
and whole-record fidelity tests remain the gate.

Reproduce the report comparison with `replay_memory` as documented in
`2026-09-09-indexed-replay.md`, choosing `memory` or `indexed` and a fresh output
path outside git. The new behavior applies to both readers through their shared
observation accumulator. No default reader or test-concurrency setting changed.

Validation: the full release gate passed 269 rondata tests (one ignored),
821 sim, 13 fixed and three doctests. Rondata took 228.40 s; sampled process-tree
peak was 8,881 MiB under the 20 GiB ceiling. Clippy with warnings denied,
formatting, seven paperwork guards and the real-install survey passed. The suite
run overlapped report verification and viewer export, so its elapsed time is
reported as gate evidence rather than a controlled throughput comparison.
