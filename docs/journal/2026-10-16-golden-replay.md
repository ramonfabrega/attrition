# Repeated golden and East Indies observations

Sequential continuation from `d4e8d083` on `codex/astra-windowed-reader`.
Commander remains paused; independent review remains owed.
Evidence: `/tmp/attrition-golden-replay-20261016-01/`.
[Audit](../audit/2026-10-16-golden-replay.json).

## Measurement and scope

The preceding complete run attributes298.56CPU to golden and242.02 to harness,
47% of rondata CPU. Native samples (`profile.py`, `*-sample.txt`) locate repeated
capture scans and decoding in chapter two near and East Indies widenings.
Inclusive sample stacks overlap; they are localization evidence, not additive
CPU percentages. Simulation replay still executes every tick from its original
start. No simulation result, comparison result or passing-test verdict is cached.

Chapter two near now reads the first matching indexed frame through its trailing
siblings, then uses the original near parser. It uses the established bounded
setup path. Reading just the frame would lose trailing records: the authored
regression covers that seam, duplicate frames and fields, missing fields, CRLF,
and nonmonotonic frame numbers. Every original frame/map assertion remains.

East Indies setup now uses bounded Initial observations for base and sibling.
Frames, groups and leader observations use the existing persistent oracle store;
region rows are a separate optional component with a distinct cache identity.
Other callers do not compute region rows. Existing source/schema invalidation,
codec bounds, corruption rejection, atomic publication and raw-key recording
bypass remain. This schema change causes one-time cache repopulation; old entries
are retained. Shared source caches hold observations, never simulation outcomes.
Every unit, record, leader, region, gaia, near, army-list and floor check remains.

## Differential and fail-first evidence

`audit.py` selects all19 harness and25 second-pair users plus chapter two near.
All45 pass with `RON_VERIFY_EAST_REUSE`, `RON_VERIFY_ORACLE`,
`RON_VERIFY_GOLDEN_SETUP` and `RON_VERIFY_GOLDEN_NEAR` enabled:86.05s/328.88CPU.
This compares complete widening reports plus record-row counts, every retained
Initial field (excluding intentionally omitted frame bodies), cached observations
against raw parsing, and each complete near map. The original parser paths remain
available through `RON_DISABLE_EAST_REUSE` and `RON_DISABLE_GOLDEN_NEAR_INDEX`.
Audit flags deliberately repeat work and are not performance defaults.

Two new authored guards cover near boundaries and region-mode identity/base
observation preservation. `fault.py` makes each fail101 by removing trailing
records or disabling region mode; exact restoration makes both pass0. Existing
oracle corruption, stale-source, duplicate-frame and codec tests also pass.
All-target Clippy passes. Sample normal-mode diagnostics and exact fixture-request
multisets equal the saved baseline binary for all three measured tests.

## Local performance

`measure.py LABEL BINARY` runs each exact test under one CPU/4096MiB admission,
using `/usr/bin/time -l`. RSS below is the OS high-water mark, not the sampler:
short tests can finish between memory samples. These are single local observations,
not a statistical benchmark or a concurrent-suite capacity claim.

| Test | Baseline wall/CPU s | First-use wall/CPU s | Warm wall/CPU s | Baseline → warm RSS MiB |
|---|---:|---:|---:|---:|
| Chapter two near |10.60/10.33|1.03/0.69|0.59/0.59|1410 →398|
| East Indies run227 |7.63/7.64|5.78/5.58|1.95/1.90|381 →256|
| East Indies run299 |4.48/4.43|3.60/3.52|2.11/2.11|384 →262|

Near is about18× faster locally; the two harness samples are3.9× and2.1× warm.
This does not establish whole-suite10×, sub-minute or sub-GiB operation. Warm
observations depend on unchanged source/schema and are still compared against
fresh simulation on every run. Cold parsing/publication cost is reported above.

## Landing status

Implementation checkpoint: complete required-fixture gate pending. No score,
pin, draw-stream or value-floor movement; simulation behavior is unchanged.
The final committed tree must pass the full release gate, preserve the original
fixture-request multiset and test inventory, and receive a separate warm complete
rondata measurement before this landing is reported complete. Fable review of
cache identity, trailing-record semantics and differential coverage remains owed.
