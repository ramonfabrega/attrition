# Fixed ten-pair startup cohort

## Result

**6/10 scheduled pairs succeeded; 15/19 launched maps succeeded.**
This is an observed batch result, not a long-term reliability estimate. There
were no retries: the runner stops a pair at its first failure, so an unlaunched
second map remains unattempted. Skipping the intro does not solve startup
reliability. No simulation completion counter moved.

| Pair | Great Lakes (14) | East Indies (18) |
| --- | --- | --- |
| 1 | Pass | Pass |
| 2 | Pass | Pass |
| 3 | Pass | Pass |
| 4 | Timeout (180 s) | Not attempted |
| 5 | Pass | Pass |
| 6 | Pass | Fail (exit 40) |
| 7 | Pass | Timeout (180 s) |
| 8 | Pass | Pass |
| 9 | Pass | Pass |
| 10 | Pass | Fail (exit 40) |

Every successful map passed the strict lifecycle, identity/seed, frame coverage,
and profile-restoration checks. Against the earlier independent 1400-frame
baseline, each also matched all **19 logged bodies** (including closing frame
1401) and all **1401 frame/seed pairs**. These are the complete observed
projections, not complete native-state parity. Every attempted map restored
and byte-verified all five profile files.

## Protocol and retained evidence

The batch was declared as ten pair attempts before running. Each invokes
`tools/explore/unattended_capture.py INSTALL NEW_PAIR_DIR PROFILE --end-frame 1400`
with its defaults: seed 12345, timeout 180 seconds, map 14 then 18,
`-automation +skipIntro`. Each map starts a fresh process. Before the next
scheduled pair, verify restoration again and require the game to be closed;
stop the cohort on unsafe cleanup. Never replace failed trials with successes.
The live lane stayed with the lab; heavy release tests ran after the batch.

Artifacts: `/tmp/attrition-reliability-ten-pairs/`, with incremental
`cohort.json`, per-pair output/logs, and `comparison.json`. Baseline:
`/tmp/attrition-unattended-1400/map-14` and `map-18`. Compare each successful
map with `tools/explore/compare_unattended.py BASELINE_MAP NEW_MAP`.
The local orchestrator and comparison recipe are
`/tmp/run-attrition-ten-pairs.py` and `/tmp/compare-attrition-cohort.py`.
These local artifacts are ephemeral and contain installed-game data; rerun
before durable acceptance. No executable, dump, or game data is committed.

## Failure localization

- Pair 4, map 14: timeout after 180 seconds. Trace: 17 complete records, 0 simulation frames; driver event counts `{'175': 1}`, fault addresses `[]`. Settings restored.
- Pair 6, map 18: ValueError: process failed: 40. Trace: 21 complete records, 0 simulation frames; driver event counts `{'175': 1, '176': 1, '177': 2}`, fault addresses `['0x7bf21139']`. Settings restored.
- Pair 7, map 18: timeout after 180 seconds. Trace: 17 complete records, 0 simulation frames; driver event counts `{'175': 1}`, fault addresses `[]`. Settings restored.
- Pair 10, map 18: ValueError: process failed: 40. Trace: 21 complete records, 0 simulation frames; driver event counts `{'175': 1, '176': 1, '177': 2}`, fault addresses `['0x7bf21139']`. Settings restored.

`tools/explore/startup_evidence.py TRACE` reproduces those bounded diagnostics.
It accepts partial evidence for inspection, refuses to interpret unknown
headers, and never issues a success/lifecycle verdict. Three authored tests
cover truncation, bounded repeated faults, unsupported headers, and frame
counts that must not be mistaken for continuity.

The startup trace proves hook installation but does not yet localize a hang
within the interval before the first menu. Fault addresses name the observed
instruction only; they do not establish why it faulted. A sampled timeout
process was using approximately two CPU cores. This records an active stall;
no root cause is claimed from that snapshot.

## Next experiment

Prioritize paired startup diagnostics before the canonical scenario adapter:
record coarse native initialization entry/exit witnesses and collect a bounded
stack sample on a subsequent stalled launch before terminating it. Keep the
same fixed-attempt denominator and retain the successful path as a comparison.
Separate Wine startup, renderer/device initialization, and engine initialization
before selecting a bypass. A causal fix needs a fresh cohort and the same
observed-state comparison. Full native A* resumption remains parked.

The adapter then needs distinct witnesses for command acceptance, package
emission, processing frame, and resulting orders/positions. An issuer return
alone is insufficient (lab ledger L08). Full renderer independence remains an
objective, not an outcome of this cohort.
