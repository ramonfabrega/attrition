# Decode one indexed frame without a redundant lazy pass

## Change and equivalence

`IndexedCapture::frame_state` read exactly one indexed frame, lazily indexed
its contents, then immediately walked the whole frame to decode owned state.
It now uses the same eager parser adopted for bounded observation chunks.
The check for exactly one matching frame, source metadata checks, duplicate
label handling, and parser arena-size bound remain unchanged. Whole-capture
parsing stays lazy; there is no cache or format change.

All ten indexed-reader tests passed. A separate temporary differential test
compared every field of all 726 decoded Frame records in run44, run25, run27,
run20, and run13 against the previous lazy decoding path. All matched. This
was complete decoded Frame equality, not a comparison of clock fields alone.
The temporary probe was removed; its source and result remain outside git at
`/tmp/frame-decode-temporary-test.txt` and
`/tmp/frame-decode-corpus-equality.log`. Existing indexed full-record and
complete replay-report regressions continue to run in the gate.

This proves equality of the decoded representation for those records, not
that the decoder covers every original engine field. No gameplay rules,
field mappings, or divergence floors change.

## Isolated measurement

Clean before/after release binaries ran the existing frozen-figure-clock test,
which visits those five captures. Each fresh process used explicit RON_INSTALL
and macOS `/usr/bin/time -l`. Samples ran sequentially before/after/after/before,
with no concurrent builds. Compilation is excluded.

| Sample | Version | Peak RSS, bytes | Test seconds | Process wall seconds |
| --- | --- | ---: | ---: | ---: |
| 1 | Before | 112,246,784 | 4.32 | 4.32 |
| 2 | After | 95,371,264 | 3.64 | 3.64 |
| 3 | After | 84,639,744 | 3.65 | 3.65 |
| 4 | Before | 84,148,224 | 4.32 | 4.31 |

This workload improves by about 16%, with RSS varying across samples. It does
not establish a universal memory improvement or full-suite speedup. Logs:
`/tmp/frame-decode-clean-{1-before,2-after,3-after,4-before}.log`. An earlier
exploratory batch overlapped lint briefly and is excluded from this table.
No original process was launched; the runtime experiment remains paused.

## Full gate

The explicit-install gate passed 27 Python tests, 275 rondata tests (one
ignored), 824 sim, 13 fixed and three doctests, plus survey, clippy, formatting
and guards. All 550 fixture requests were present. Rondata test time was
240.68 seconds; sampled process-tree peak was 9809 MiB under 20 GiB. This
run does not demonstrate a full-suite speedup or memory reduction.
Log: `/tmp/indexed-frame-decode-final-gate.log`; fixture report:
`/tmp/attrition-gate-indexed-frame-decode-20260909/fixture-coverage.json`.
