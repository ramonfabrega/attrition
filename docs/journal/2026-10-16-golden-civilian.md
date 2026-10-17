# Golden civilian observation decoding

Sequential continuation from `4d6c0b9d` on `codex/astra-windowed-reader`.
Commander remains paused. Independent review remains owed.
Evidence: `/tmp/attrition-golden-civilian-20261016-01/`.
[Audit](../audit/2026-10-16-golden-civilian.json).

## Baseline and decision

The preceding warm complete rondata run costs145.58s/1037.78CPU; golden is the
largest family at283.12CPU. The retained civilian native sample in
`/tmp/attrition-golden-replay-20261016-01/civilian-sample.txt` locates repeated
`raw_near`, eager parsing and typed readers in the same civilian loop. Its
inclusive stacks overlap and are not additive percentages. Save the exact
preceding executable as `baseline-rondata` and compare three exact windows.

Extend the existing source-keyed oracle with an optional civilian observation:
near map, ordered building cycle/visibility rows, world fog/detection/danger
planes, goods by original index, and optional live ammunition by shooter/slot.
Existing frame, groups and leader records supply the rest. Pool comparisons
consume the same groups through an extracted observation-taking method; the
original raw pool reader remains. Every simulation tick, comparison, floor,
missing-field row, first divergence and report count still executes.

Cache identity distinguishes ordinary, civilian without ammo and civilian with
ammo modes. Civilian identity additionally hashes golden helpers, observation
code and ammo readers. Other callers do not parse civilian observations. Source
metadata, schema checks, bounded decoding, corruption fallback and atomic
publication remain. Old entries are retained; this codec/schema change requires
one-time repopulation. Simulation or moved case-body edits do not change these
observation schemas; changes to the golden helper file do. No passing verdict,
simulation state or comparison result is cached.

The raw-key recorder disables cache use. `RON_DISABLE_GOLDEN_ORACLE` forces the
previous eager parsing path; `RON_DISABLE_GOLDEN_READERS` also bypasses this new
cache. `RON_VERIFY_GOLDEN_READERS` preserves its audit on hits by reparsing the
whole oracle. `RON_VERIFY_ORACLE`, `RON_VERIFY_GOLDEN_FRAME_REUSE` and
`RON_VERIFY_SHARED_GOLDEN` retain whole-observation and complete-report audits.

## Coverage and fail-first evidence

`audit.py` selects all72 cases calling either civilian widening helper. All72
pass with the four audit flags enabled:171.09s/668.69CPU. Complete civilian
reports include every counter and missing/parted row. Combined cases also
compare the complete independent pool report. Raw-key coverage is checked again
in the required gate. Original helper/case assertions and expected values stay.

Three new tests cover auxiliary observation values and codec boundaries, numeric
map bounds, and distinct persistent cache modes. Authored values exercise first
valid near fields, last duplicate goods, missing defaults, live-ammo filtering,
cycle/visibility values, fog byte conversion, malformed fields and danger values.
No-fog world records still skip detection/danger readers. Codec checks cover
truncation, trailing bytes, changing/empty records, duplicate map keys, impossible
lengths and exhausted allocation budget. Numeric-map codecs retain explicit
bounds and duplicate rejection; byte vectors use the existing bounded Vec codec.

`fault2.py` removes danger observations and aliases ammunition modes. Both exact
guards fail101, then pass0 after exact restoration; the restored binary is
byte-identical to `final-rondata`. The first fault driver selected the empty CLI
test binary and ran zero tests: this was rejected, not counted as evidence. The
corrected driver selects the library target and requires one executed test.
All logs, including failed development builds, remain in the evidence directory.

Focused observations/oracle/codec tests, three ledger tests and22 floor tests
pass. The source inventory initially caught the new module missing; it is now
registered as an observation/codec module, with comparisons in golden.rs. The
ledger remains25 records/328 fields,9 uncompared and31 single-capture. All-target
Clippy passes before landing. Score, pins, draw-stream and value floors do not move.

## Local measurements and limits

`measure.py LABEL BINARY` runs exact cases sequentially under one CPU/4096MiB
admission with `/usr/bin/time -l`; RSS is the OS high-water mark. All three cases'
normal diagnostics and exact fixture-request multisets match the saved baseline.
These are individual local measurements, not confidence intervals.

| Window | Baseline wall/CPU s | First-use wall/CPU s | Warm wall/CPU s | Baseline / first-use / warm RSS MiB |
|---|---:|---:|---:|---:|
| Chapter36 civilian/pool |8.89/8.22|12.21/12.10|1.72/1.67|765 /1141 /763|
| Chapter44 ammunition |2.10/2.04|2.90/2.85|0.87/0.84|496 /593 /495|
| Chapter78 late window |1.35/1.29|1.79/1.77|0.76/0.71|457 /511 /439|

Warm CPU falls substantially, while warm memory is roughly flat. First-use cost
and RSS increase because parsing and cache publication are additional work.
This is a warm decoding improvement, not an unconditional memory reduction or
whole-suite10× result. Full-suite effects and cache-repopulation cost must be
reported separately; concurrent-agent capacity is not established by these runs.

## Implementation checkpoint

Complete required-fixture gate pending on the committed implementation. After
that gate, measure all rondata tests warm and compare original test names and
fixture requests. Independent review of cache semantics and coverage remains
owed; this branch is not merged into the paused commander's checkout.
