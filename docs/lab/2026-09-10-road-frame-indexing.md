# Bound the market-road comparison to one frame

## Change and coverage

After L45/L46 bounded construction inputs, the market-road regression still
loaded run72 in full to inspect frame 4804. It now opens the existing indexed
reader, selects the first frame with that number, and reads only its wrapped
frame text. The old lookup also selected the first matching frame. WORLD
lookup, frame-specific height extraction, simulation and every tile assertion
are unchanged. Missing-frame behavior remains an explicit skip; unreadable
or changed indexed sources fail through the reader's existing I/O validation.

A new retained-corpus regression compares the complete ordered WORLD field
list and every frame-specific height entry between whole-text and bounded
parsing for this frame. It requires nonempty fields and more than 1,000 height
entries, and passed in 0.80 seconds. This covers exactly the inputs consumed by
`world_from`; it is not a claim about every other subsystem in the capture.
The test adds one fixture request, not a new capture dependency.
Evidence: `/tmp/road-frame-equality.log`.

The indexed reader still scans the file to build its offsets on a cache miss.
This reduces retained text, not all I/O. No parser, simulation, cache policy,
unsafe code or dependency changes. Finalized-source metadata identity retains
its existing limitations; this is not content-hash binding.

## Isolated measurements

The baseline is branch tip `cbdfc64`, with both earlier road memory changes.
Clean release binaries ran sequentially in before/after/after/before order,
one test thread, explicit RON_INSTALL and fresh process caches. Build time is
excluded. All four runs passed with no fixture skips. RSS is the process peak
reported by `/usr/bin/time -l`.

| Sample | Peak RSS bytes | Test seconds |
| --- | ---: | ---: |
| 1-before | 1,027,424,256 | 6.08 |
| 2-after | 571,064,320 | 6.11 |
| 3-after | 597,950,464 | 6.13 |
| 4-before | 1,027,473,408 | 6.07 |

Peak falls a further 42–44%, from about 1.03 GB to 0.57–0.60 GB, while
runtime remains around 6.1 seconds (under 1% higher in these samples).
The three local changes together bring this test from the separately measured
3.21–3.24 GB pre-L45 baseline to about 0.6 GB, roughly an 81–82% reduction.
That cumulative comparison is not a freshly repeated combined experiment.
No universal or whole-suite speedup is claimed.
Logs: `/tmp/road-frame-{1-before,2-after,3-after,4-before}.log`.

No queue item, fidelity score or paused runtime draft was changed. Further
migration should follow measured consumers, preserving independent whole-text
equality checks rather than replacing the reference side with indexed reads.

## Full gate

The explicit-install monitored four-thread gate passed: 280 rondata tests
(one ignored, zero filtered), 824 sim tests, 13 fixed tests, three doctests
and 34 Python tests, plus survey, clippy, fmt and paperwork guards. Rondata
took 122.69 seconds; sampled tree peak was 13,213 MiB under the 20 GiB ceiling.
All 557 fixture requests were present. One added request is the comparison
input equivalence regression. The suite peak is higher than the preceding
single gate run; local RSS gains do not imply a lower peak under every
concurrent schedule.

Evidence: `/tmp/road-frame-gate.log` and
`/tmp/attrition-gate-road-frame-20260910/fixture-coverage.json`.
