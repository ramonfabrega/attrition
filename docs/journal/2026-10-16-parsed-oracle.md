# Parsed oracle experiment — 2026-10-16

User objective: 10× lower suite time with lower CPU and memory, preserving all
checks. This continues the original reader brief's parsed-store step after the
replay/trace batch (`fdcc8383`, pushed). The paused loop remains paused. No sim,
pins, scoreboards or value/word targets change. This is one sequential agent;
independent review remains owed.

## Hypothesis and scope

Persistent decoded source observations avoid repeated text parsing and key
scans. First use exact source + frame position, rather than merging captures by
lobby: capture detail and duplicate labels must not be conflated. Cache only
owned Frame records, pool groups and normalized original leader rows. Every
simulation tick, per-field comparison, checkpoint and report still executes.
Integration starts in the generic widening harness; specialized walkers and
raw-key coverage stay on original text. This is not yet a suite-wide migration.

Codec schemas exhaustively destructure and reconstruct all 13 owned record
types (268 fields); adding a field fails compilation until included. Encoding
uses little-endian integers, strict options, bounded vectors/strings and maps,
then the existing safe LZ4 dependency. Decode checks UTF-8, duplicate map keys,
complete consumption, 32 MiB encoded/decompressed bounds and a 64 MiB allocation
budget. Full identity and an accidental-corruption checksum accompany payloads;
the checksum is not a security authentication scheme. Cache misses/corruption
fall back to parsing. Existing files are never overwritten. Publication uses a
unique create-new temporary file and hard-link to an absent final name; only
this invocation's temporary file is removed.

Identity includes held source path/length/mtime, frame position and byte range,
plus a fingerprint of parser/normalizer/index/codec sources. Source metadata is
validated before and after use. Read-key instrumentation disables cache use.
`RON_VERIFY_ORACLE=1` checks every warm decoded frame against fresh whole-record
parsing, and independently compares complete widening reports/checkpoints and
compared-field sets against the uncached legacy path. An ordinary optimized run
must follow audit; audit reads cannot supply otherwise absent coverage.

## First measurement

Evidence root: `/tmp/attrition-replay-batch-20261016-01/` (retained).
`oracle-prototype-run583.log`: 128 frames of
`gamelog-run583-islands-toughest-16754.txt`, explicit ignored experiment
`diff::oracle::decoded_frame_experiment` with `RON_ORACLE_SOURCE` set to the
fixture. Whole Oracle structs round-trip equal for every frame.

- Parse: 669.604 ms; compress/encode: 52.090 ms; decompress/decode: 78.413 ms.
- Decode stage 8.54× faster than parsing; this excludes disk cache lookup and
  does not claim a whole-suite speedup.
- Text 396,127,345 bytes; binary 74,715,115; compressed 12,281,692 bytes.

`oracle-tests-final.log`: three tests passed, one explicit fixture experiment
ignored. Negatives cover every truncation of a synthetic record/entry, bit
corruption, oversized lengths/decompression, invalid Option tags, duplicate map
keys, allocation limits, stale identity, duplicate frame labels, source changes,
existing-file preservation and read-key instrumentation bypass. Implementation
is experimental and uncommitted; full report audit, performance evaluation,
bookkeeping guards and required-fixture gate are pending.

## Cold integrated coverage

First complete optimized coverage pass (`oracle-cold.log/.time`) passed: 88.40 s
real, 77.43 s user, 10.06 s system, 2,075,394,048 B peak RSS and
1,960,870,662,261 retired instructions. All 62 generic widenings ran; 9,879
misses and 9,879 successful publications, zero hits. Retained experimental
cache `/tmp/attrition-replay-batch-20261016-01/oracle-cache-v1` has 9,879 entries
and 605,072,218 payload-file bytes. No source capture was changed.

This run used `--nocapture` to observe all counts, so its output/syscall behavior
differs from the earlier ordinary timing. Subsequent paired warm/off measurements
use `--show-output`, preserving libtest's buffered capture and exposing totals
at completion. Cold population cost must remain explicit; it is not a speedup.

## Paired warm/off result and allocation follow-up

The same experimental binary, same fixture installation, one test thread and
`--show-output`: warm `oracle-warm.log/.time` passed in 41.63 s real (40.52 user,
1.01 system), with 910,925,894,338 instructions and exactly 9,879 hits/0 misses.
Cache off `oracle-off.log/.time` passed in 75.94 s (74.06 user, 1.82 system),
1,765,420,746,352 instructions. Warm improves elapsed 1.82×, CPU 1.83× and
instructions 1.94×. But peak RSS regressed from 1,798,602,752 to 2,448,769,024 B.
Do not hide that regression or treat the prototype as an accepted memory win.

Follow-up hypothesis: repeated decode allocations retain extra heap pages.
Reuse compressed/decompressed scratch buffers per widening, and allocate every
decoded vector once at its validated exact length. The previous fallible
iterator collection could grow capacities geometrically; the decoder's memory
budget now corresponds to the explicitly requested vector capacity. Maximum
decoded frame in the first cache was 3,112,046 B; total decoded entries 3.666 GB.
No frame is retained between comparisons. Both scratch buffers remain bounded
by the entry limit. New source fingerprints intentionally invalidate prototype
entries; the first cache is retained as evidence, never rewritten or reused
under a new schema without parsing source again.

Byte-buffer/exact-capacity experiment: `oracle-buffer-cold.log/.time` passed
82.92 s (75.49 user, 6.88 system), RSS 1,468,907,520 B; all 9,879 entries cold.
`oracle-buffer-warm.log/.time` passed 44.61 s (43.39 user, 1.13 system),
905,020,839,582 instructions, but RSS 2,590,359,552 B. The allocation hypothesis
was not solved by byte buffers/exact vector capacity alone. This is not a memory
win and is not sufficient evidence to accept default cache behavior.

Next experiment reuses owned record vectors in place. Every field is rewritten
by the exhaustive decoder; vectors truncate on shorter input, Options are
replaced, maps are rebuilt, strings reuse capacity. A failed partial decode is
never exposed: the caller reparses the original and replaces the partial value.
A retained-capacity estimate includes nested vectors/strings/maps and resets
records exceeding 16 MiB before the next 64 MiB-budgeted decode, preventing
capacity from accumulating indefinitely at different record positions. Tests
include populated → empty → populated decoding to detect stale values. This
experiment is still pending measurement and full report audit.

In-place candidate: `oracle-reuse-final-tests.log` passed all three codec/store
checks (the explicit benchmark remains ignored). `oracle-reuse-cold.log/.time`
passed 86.71 s real (79.37 user, 6.17 system), RSS 1,794,818,048 B; 9,879 new
entries published. `oracle-reuse-warm.log/.time` passed 43.36 s (41.99 user,
1.11 system), 897,833,268,288 instructions, RSS 2,073,493,504 B; all 9,879 hits,
zero misses. Reuse reduced observed warm RSS versus the prior candidate, but
it remains 15.3% above the 1.799 GB cache-off measurement. This is not a stable
memory win; whole-suite timing and peak overlap remain to be measured.

Default entries now live in `.ron-oracle-v1` beside each canonical capture;
`RON_DISABLE_ORACLE_CACHE=1` opts out. Prior experimental generations remain
intact, including the explicit temporary-root prototype. The on-disk format is
not a public interchange format; parser/schema changes select new entries.
`oracle-final-clippy.log` and `oracle-final-fmt.log` report successful lint and
format checks. The full warm differential audit is running; no gate verdict
or implementation landing is claimed yet.

## Whole-record and complete-report audit

`oracle-whole-audit.log/.time`: exit 0, 242.89 s real. All 62 generic windows
matched complete fresh reports, checkpoints and independently recorded
compared-field sets. Exactly 9,879 cache hits, zero misses/writes; each hit was
also compared as a whole Oracle (all owned records, groups and normalized
leader rows) against parsing its original source frame. Fresh walks used the
uncached legacy reader and restarted the simulation independently. Prefix
compatibility, scores and pins did not change. This is executable differential
evidence, not an independent human/model review; that review debt remains.

`oracle-guard.log`: `tools/guard.sh --built` and `git diff --check` completed
exit 0; the pre-existing item 7 queue-ledger note remains. The ordinary warm
coverage pass after the audit is running separately. The subsequent full gate
will measure aggregate CPU/wall time and process-tree RSS; the single-test
memory regression is still disclosed above.

Ordinary post-audit pass (`oracle-final-ordinary.log/.time`) exited 0: 40.94 s
real, 39.55 user, 0.96 system, 896,108,266,010 instructions; all 9,879 hits and
zero misses/writes. RSS was 2,493,399,040 B. Thus the in-place warm runs range
2.07–2.49 GB: the lower first peak did not establish a repeatable memory fix.
Time/CPU gains are clear; memory benefit is not. The implementation is ready
for a committed full-suite measurement, with that limitation left explicit.

For the Loop: paused; source-keyed parsed observations now replace repeated
parsing in generic widenings, preserving all reports/field sets over 62 audited
windows. Coverage 75.94 s off → 40.94–43.36 s warm; cold population 86.71 s.
No score, value, word or pin movement. Full required-fixture gate pending.
