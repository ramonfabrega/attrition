# Coverage walk — 2026-10-16

User-authorized sequential performance work on `codex/astra-windowed-reader`,
starting at `02325d9e`. Commander remains paused. No simulation changes, captures,
new dependencies, pin changes or altered fixture requirements. The whole-suite
10× target remains unmet; this item targets the remaining coverage critical path.

Evidence: `/tmp/attrition-coverage-walk-20261016-01/`. Baseline `baseline.log`
passes all 62 windows: 44.69 wall / 42.44 CPU seconds, 2269937664 bytes peak RSS.
Optional `RON_PROFILE_REPLAY` stage timers identify decoding, direct values,
groups and leaders/gaia separately. Diagnostic `stages.log`: summed 41.078s,
setup 4.125s, ticking 14.167s, decoding 6.002s, values 13.651s, groups 0.667s,
leaders/gaia 1.992s. Native `sample.txt` overlaps this diagnostic run, so this
is localization evidence rather than an uncontaminated speed comparison.
2958 of 11586 worker samples enter `compared::note` (about 25.5%).

The first experiment replaces the repeated ordered-string pair lookup with a
HashSet of immutable static string identities (pointer and length). Final labels
are still sorted and deduplicated by text; no hash is persisted, and full identity
equality handles hash collisions. Same text at different addresses is allowed
internally, then canonicalized at the boundary. `RON_DISABLE_FAST_COMPARE_NOTES`
retains the previous pair tree; `RON_VERIFY_COMPARE_NOTES` records both and checks
the complete resulting label sets. All presence checks and note call sites stay.
First controlled runs: fast 37.38 / 35.79 CPU seconds, legacy 40.99; audit passes
at 45.40. All four runs request the exact same 338 fixture rows. RSS varies from
2223144960 to 2770108416 bytes: this trial establishes no memory reduction.

The second experiment defers direct-row label allocation until a value differs.
A static base and optional guy index preserve each original label. All numeric
rows, row counts and field registration still execute, including matching rows.
`RON_DISABLE_LAZY_ROW_LABELS` eagerly materializes every label and looks up its
field through the original indexed string. `RON_VERIFY_ROW_LABELS` reruns each
frame with that reference mode and compares the complete FrameResult, compared
row count, divergence map (keys and values) and independently registered fields.
Ordinary runs after the audit must pass without its reference registrations.
The reference retains eager allocation but has a different temporary vector
layout; the untouched baseline remains the pre-change timing anchor.

Focused tests cover duplicate/static-prefix strings, nested/disabled recording,
a deliberately dropped registration rejected by the audit, and deferred versus
original label formatting/field mapping (including unknown names and extreme
indices). An early manual invocation raced the rebuild and used an older binary
with the initial source-scanning test bug; the completed candidate's four focused
tests pass (`focused.log`). The test now locates the final mapping declaration.

Implementation files: `diff.rs` recorder; `diff/harness.rs` labels, full-frame
audit and optional stage timers. Final measurements, contention evidence and
release verdict are pending. Independent review remains owed for static-identity
canonicalization and deferred-label equivalence. No self-ratification.

For the Loop: paused; pins and scoreboard unchanged. Review this branch's
performance evidence separately from the commander's unchanged main branch.

## Controlled result

`rows-measure.py`: alternating same-binary eager/tree and lazy/hash modes,
warm existing caches; each exit0, exact338 fixture requests, zero missing.
Wall / CPU seconds / maximum resident bytes:

| Mode | Wall | CPU | RSS bytes |
| --- | ---: | ---: | ---: |
| off1 | 48.76 | 44.59 | 1974059008 |
| on1 | 34.57 | 33.95 | 2063908864 |
| off2 | 44.69 | 42.95 | 1984086016 |
| on2 | 34.53 | 33.42 | 1593163776 |
| audit | 56.84 | 56.73 | 2205696000 |

CPU falls21–25%; memory ranges overlap (no reliable isolated memory claim).
The initial unchanged baseline is42.44CPU, so its comparison is about20–21%.
All five runs also match all14748 original diagnostic lines, excluding timing
and monitor output (`compare-output.py`, `output-equivalence.log`). One memcap
line interleaved inside a diagnostic; the extractor removes that exact monitor
line including its newline, retaining the surrounding application text.
The audit compares full per-frame results and divergence maps, not only stdout.
`rows-on1` summed values4.567s vs diagnostic baseline13.651s; ticking13.879s and
decoding5.713s now dominate. No record, fixture, value assertion or raw-key check
was removed. Four focused tests pass; full gate pending.

## Four competing suites

`contention.py` / `contention-compare.py` use the previous eight-test sample,
four processes, each using the production isolated runner,16 CPU slots and
32GiB reserved memory, eight workers per suite. Identical binary/cache/resource
policy; both optimizations disabled together versus enabled. This is a sample,
not four complete release suites. All32 tests pass in each mode; exact1832
fixture-request multisets match, zero missing. Wall is runner makespan; peak is
sampled across the entire process tree, not a sum of individual maxima:

| Mode | Wall seconds | CPU seconds | Peak tree MiB |
| --- | ---: | ---: | ---: |
| off | 53.408 | 341.49 | 14031 |
| on | 44.661 | 291.33 | 13195 |

About16% lower elapsed,15% lower CPU,6% lower peak tree memory. A single pair
of contention runs is evidence of this workload, not a universal memory bound.
Clippy, formatting and `tools/guard.sh --built` pass before this experiment.

## Six competing suites and landing checkpoint

Same eight-test sample/resource policy, six simultaneous suites (`six/` scripts
and logs): all48 tests pass in each mode, exact2748 fixture-request rows match,
zero missing. Baseline60.166wall /507.90CPU /15302MiB peak tree; optimized
49.525wall /438.95CPU /15331MiB. About18% lower elapsed and14% lower CPU;
peak memory is effectively unchanged (0.2% higher). Shared reservations are
empty afterward. Thus CPU and throughput improve consistently across4–6 suites;
this change does not establish a consistent memory reduction at six suites.

The implementation checkpoint is pending the complete required-fixture gate.
The remaining measured coverage costs are ticking (~14s), decoding (~6s), setup
(~4s) and values (~5s). Further gains need new measurements there; the original
reader profile no longer describes the current bottleneck. No10× whole-suite
claim. Source-based review remains provisional until the later independent steer.
