# Ticking and decoded-record allocation — 2026-10-16

User-authorized sequential follow-up on `codex/astra-windowed-reader`, starting
at `dabd6a04`. Commander stays paused. Goal: reduce CPU/memory and contention
without changing comparisons, fixture requirements, pins, simulation results or
bounds. Evidence root: `/tmp/attrition-tick-decode-20261016-01/`; original binary
preserved as `baseline-rondata`. Whole-suite10× remains unmet.

## Baseline and selection

Warm baseline coverage (`baseline.log`): test37.62s, wrapper38.59wall/35.39CPU,
1984020480bytes RSS. The separate diagnostic run (`profile.py`, `profile.log`,
`sample.txt`) samples this process for20s. It is localization evidence, not a
speed comparison. Of15365 worker samples,8884 are under Built::tick and8818
under Sim::tick; harness overhead accounts for less than1% of sampled ticking.
Thus do not rewrite mutable frame-input lookup to optimize the wrong layer.
Wonders-held scans account for627 leaf samples; find-unit-with-radius513.
Decoded maps and their key allocation are also visible in the sample.

## First candidate

`capture/owned_codec.rs`: reuse nested optional heap values, arrays and tuples.
For a map with the same length, compare every incoming key to the existing
ordered keys while decoding every value. Commit decoder consumption only if
all keys match; otherwise run the original map decoder against the original
input and budget. This retains tree nodes and String buffers on stable schemas,
without a temporary index or retained borrowed pointers. Changed/reordered keys
remain accepted when the original accepts them; duplicates remain errors.
`RON_DISABLE_DECODE_REUSE` keeps the original nested-container decode paths.
The32MiB input bound,64MiB decode allocation budget and16MiB retained-capacity
reset are unchanged. No observation or verdict is reused instead of decoded.
`diff/oracle/codec.rs` updates explicit Decoder construction in existing tests.

`sim/ai_sites.rs`: a present seen2 bit already makes was_seen_fog true, whatever
the other true-return arms say. Check that bit before live wonder/unit scans.
If it does not prove true, call the unchanged reference function. All state is
read live on each call; no turn cache or invalidation assumptions. This only
reorders pure predicates and does not approximate visibility or alter draws.

Focused checks: four codec tests pass, including actual key-pointer retention,
complete outputs across grow/shrink/None transitions, changed and reordered keys,
duplicate keys, every truncation, UTF-8 errors and allocation-budget equality.
Two visibility tests pass: original Electronics/Fouché fixture and a matrix of
fog bits, ally masks, reveal modes, Fouché mutations, players and off-grid cells
against the unchanged function. Compile-only method-name resolution and closure
inference errors were corrected; build logs1–3 retain the sequence.

Changing codec source intentionally creates new source-fingerprinted oracle and
setup-observation cache generations. Existing generations and source captures
are preserved. `warmup.log` is cold creation, separate from warm measurements.
No cache fingerprint was weakened to reuse entries across code changes.

Full-record audit and contention are complete below; release validation is pending.
Independent review owed for decoder reuse and pure-predicate reordering. Pins
and scoreboard unchanged; later Fable review remains separate.

For the Loop: paused; this branch's work is not a main-branch landing.

## Warm measurements and the allocator finding

All times are seconds; RSS below is decimal GB. Original is the preserved
`dabd6a04` binary. Both alternating sequences assert identical 338 fixture request
rows (zero missing) and 14,748 complete diagnostic lines, excluding only profile
and resource-monitor output. `measure.py` and `allocator-measure.py` retain exact
commands, raw logs and unabridged fixture/output comparisons under the evidence
root. Cold new-generation creation (`warmup.log`, 85.33 wall/82.07 CPU,
1.803 GB RSS) is excluded from warm comparisons.

| Binary / allocator | Wall | CPU | Peak RSS GB |
|---|---:|---:|---:|
| original / system, repeat 1 | 36.57 | 34.73 | 2.248 |
| candidate / system, reuse disabled | 36.57 | 33.61 | 2.144 |
| candidate / system, repeat 1 | 32.51 | 30.30 | 3.262 |
| original / system, repeat 2 | 38.60 | 35.75 | 2.369 |
| candidate / system, repeat 2 | 34.55 | 31.47 | 2.769 |
| original / large cache off, repeat 1 | 36.56 | 34.66 | 0.856 |
| candidate / large cache off, repeat 1 | 34.55 | 30.67 | 0.855 |
| candidate / space efficient, experiment | 32.52 | 31.28 | 0.772 |
| candidate / large cache off, repeat 2 | 30.47 | 30.31 | 0.851 |
| original / large cache off, repeat 2 | 36.58 | 34.56 | 0.857 |

The initial candidate **regressed memory** under the system allocator. Native
heap and VM probes (`memory-probe.py`, `*-heap.txt`, `*-vmmap.txt`) suspend only
their own child at the same run861 boundary and resume in a finally block.
Both complete successfully. Live malloc data is only about 59 MiB in either
binary, while original/candidate footprints are 1.7/2.2 GiB and peaks 1.9/2.8 GiB.
Most excess residency is marked `MALLOC_LARGE (empty)` or `MALLOC_REALLOC (empty)`.
This distinguishes freed-block retention from live decoded-record growth.

Apple's [xzone allocator source](https://github.com/apple-oss-distributions/libmalloc/blob/main/src/xzone_malloc/xzone_malloc.c)
reads `MallocLargeCache` as a production environment option; zero disables huge
caching and deferred reclamation. The broader space-efficient option also changes
other allocation policies, so the narrower large-cache option is selected.
The source is explanatory; the local macOS 26.3.1 experiments establish the
observed effect, not a promise about all OS versions. No type isolation,
zero-on-free, guard-page or corruption-check setting is disabled.

`tools/test_runtime.py` defaults release-gate and isolated-runner children to
`MallocLargeCache=0` only on macOS, preserves explicit values, and supports
`RON_TEST_ALLOCATOR=system`. It does not mutate the parent's environment or any
host configuration. Effective values are recorded in the gate policy and isolated
result rows. Linux/Windows receive no allocator setting. Four authored policy
and plumbing tests join the offline suite. Of 59 focused Python tests, the first
mock omitted its fixture receipt and was corrected; the next sandboxed run could
not use `ps`. The authorized rerun (`tool-tests3.log`) passes all 59. This is a
sandbox limitation, not a weakened resource check.

With the same bounded allocator, candidate CPU is 30.31–30.67 against
34.56–34.66: about 11–13% saved, with essentially equal 0.85 GB memory. Ticking
falls from about 13.9 to 13.1 seconds. Decoder stage time varies with IO, but
complete-output equality and CPU saving repeat. The allocator alone reduces
original coverage peak memory roughly 62–64%, without an observed CPU penalty.
The combination is the proposed default; code reuse alone is not a memory win.

## Contention protocol

The unchanged eight-test sample covers coverage, third-pair, two chapter walks,
city/world/build walks and a scalar test. Four or six independent suites share
16 CPU slots and 32 GiB of reservations, with eight workers per suite. The
preserved original binary uses `RON_TEST_ALLOCATOR=system`; the candidate uses
the new default. This is a combined code+allocator comparison, not a same-binary
allocator trial. All 32/48 individual tests pass and exact fixture multisets
match (1,832/2,748 requests; zero missing). Reservations and correctness checks
are unchanged. Full-suite results are reported separately.

The initial four-suite sequence (`contention-compare.py`) measured
45.902→38.965 seconds, CPU300.93→316.09 seconds, and sampled tree peak
16,038→10,619 MiB. It saves wall/memory but regresses aggregate CPU5%.
Per-test accounting localizes the increase to chapters35/36 (36.77→51.93 and
40.01→58.56 CPU seconds across four copies); coverage falls153.15→136.64.
The coverage-only warmup did not populate these chapters' new fingerprint
entries. Thus first-use cache creation is a confounder; do not discard this run
or label it a controlled warm regression. A repeat after both generations have
been used separates steady-state performance.

The following six-suite sequence (`six/contention-compare.py`) measures
47.642→48.380 seconds (1.5% slower), CPU442.13→419.07 seconds (5.2% lower),
and sampled tree peak17,936→10,767 MiB (40.0% lower). This establishes a large
contention memory reduction, a modest CPU saving, and no elapsed-time gain in
this sample. It does not establish 10× or a six-complete-suite result.

The warm four-suite repeat (`repeat-four/contention-compare.py`) passes all32
again with the same1,832 requests. Wall38.980→39.379 seconds (+1.0%),
CPU288.75→288.26 seconds (effectively flat), peak11,772→7,330 MiB (−37.7%).
Thus the repeat does **not** support the first run's15% elapsed improvement;
it supports a substantial memory reduction. Aggregate CPU savings are workload
and contention dependent, even though the coverage CPU result repeats in isolation.

## Reproduction and scope

Run from this worktree. `python3 /tmp/attrition-tick-decode-20261016-01/measure.py`
and `allocator-measure.py` describe the alternating single-test protocol (their
exclusive output names prevent overwriting evidence). The three contention
scripts record their eight exact test names, admission settings, child exits and
fixture multisets; use a new output directory for a repeat. The tracked audit
JSON retains binary SHA256 identities and compact measurements, with no captured
record payloads. Native sampling/probe scripts and diagnostic source copies are
local evidence only. Existing captures and cache generations remain untouched.

Full-record differential command, with a fresh fixture-audit directory:

```sh
RON_INSTALL=/Users/rf-studio/ron-golden/ch80/map-14 \
RON_VERIFY_ORACLE=1 RON_PROFILE_REPLAY=1 MallocLargeCache=0 \
RON_FIXTURE_AUDIT_DIR=/tmp/attrition-tick-decode-20261016-01/oracle-audit-fixtures \
/usr/bin/time -l zsh tools/memcap.sh 20 \
target/release/deps/rondata-4337e333078eb556 --exact \
diff::coverage::every_parsed_field_is_compared_by_the_instrument_or_pinned \
--nocapture --test-threads=1
```

No draw-stream score, whole-record value diff, pins, floors or expected coverage
are intentionally changed. The 14,748 diagnostic lines match the original
exactly. Independent review of decoder reuse, predicate reordering and runner
policy remains owed; executable evidence does not self-ratify that review.

## Pre-landing validation

The oracle audit exits0: all62 complete report/checkpoint/field-set comparisons
match fresh uncached from-zero walks, and all decoded cached records match fresh
whole-record parsing (`oracle-audit.log`). It takes241.68 wall/241.39 CPU seconds,
921,403,392 bytes peak RSS. This diagnostic work is not a performance benchmark.
The subsequent ordinary full gate runs with no audit flags, preventing the
reference walk from supplying coverage to the passing result. Focused Rust
checks pass (four codec, two visibility); all59 runner tests, clippy, formatting
and the development guard pass. A final guard covers these documentation updates.
The implementation commit precedes the complete four-thread required-fixture
gate; its actual verdict will be recorded separately.

## First complete gate and the full-workload policy correction

Implementation `33379f68667d942508e41991a3ee102c60ffde74` passed the full gate
(exit0, six of six phases; `gate.log`, report directory `gate/`): fixed13,
rondata890 plus6 ignored, sim2586, doc tests3. All3,276 requests for419 fixtures
are present and the exact request multiset matches the preceding full gate.
Pins, scored words, value divergences and coverage floors do not move.
The first full pass over new codec fingerprints takes481.21 wall/1,625.32 CPU
seconds, sampled tree peak8,553 MiB; rondata360.30 seconds. Against the previous
gate's433.29/1,422.75/9,910, memory falls14% but time/CPU rise11%/14%.
This is a real recorded result, not a suite speedup. Cache creation confounds
comparison with the previous already-populated generation.

To distinguish that cost from steady-state overhead, `full-isolated.py` runs
complete manifests through the production eight-worker runner with warm cache
generations. All requests match the gate exactly; no tests or checks are removed.
The original has887 passing tests; the candidate has890 (three new authored
codec checks); both retain the same6 ignored tests.

| Complete warm rondata | Runner elapsed s | Test CPU s | Wrapper CPU s | Peak tree MiB |
|---|---:|---:|---:|---:|
| Original, system policy | 170.558 | 1227.88 | 1311.37 | 8223 |
| Candidate, all bounded | 174.505 | 1253.99 | 1340.23 | 7992 |
| Candidate, system policy | 167.500 | 1201.01 | 1284.60 | 7658 |
| Candidate, scoped automatic | 166.118 | 1198.29 | 1281.47 | 8608 |

Applying bounded reclamation everywhere costs about4% CPU versus the same
candidate's system policy. Per-test evidence identifies the useful exception:
parsed-field coverage falls from2,210 to786 MiB; the48 third-pair tests average
589→433 MiB. Their combined CPU is nearly flat across allocator modes (coverage
34.58→33.58; third-pair74.54→75.85). Other tests often save little memory and
pay reclamation overhead. Thus the initial broad isolated default is superseded.

`auto` now selects bounded reclamation for that coverage test and the third-pair
group only. Other isolated tests keep system behavior. Combined-process test
children still use bounded reclamation because they contain those walks.
The isolated gate dispatcher preserves the per-child choice. Explicit allocator
values always win; `bounded`/`system` modes provide reproducible forced controls.
This is deterministic workload selection, not a result cache, skipped test,
weaker limit or online timing heuristic. Unknown/renamed tests use system policy;
future additions require measurement.61 focused tooling tests pass, including
actual isolated dispatch and forced overrides (`tool-tests-auto.log`).

The selected complete warm run saves2.4% test CPU and2.6% elapsed against the
original, while sampled aggregate memory is4.7% higher. Scheduling and allocator
residency vary; no full-suite memory improvement is established by that run.
The strong memory result is specifically competing repeated walks.10× remains
unmet. The full required-fixture gate will be repeated on the final committed
policy; the earlier passing gate remains identified separately.

## Selected automatic policy: fresh contention comparison

The final defaults are measured with fresh original/system versus candidate/auto
runs, same eight tests, same limits and warm inputs. `auto-four/` and `auto-six/`
retain every invocation, result, allocation policy and fixture receipt.

| Competing suites | Elapsed s, before → after | CPU s | Peak tree MiB |
|---|---:|---:|---:|
| Four | 41.152 → 41.048 | 295.48 → 277.94 | 13199 → 11221 |
| Six | 56.110 → 51.008 | 453.94 → 422.55 | 14864 → 11937 |

Four: elapsed flat, CPU−5.9%, memory−15.0%. Six: elapsed−9.1%, CPU−6.9%,
memory−19.7%. All32/48 tests pass; exact1,832/2,748 fixture requests match,
zero missing. These are contention samples, not four/six entire suites. The
selected default balances CPU and memory; forcing `bounded` can reclaim more
memory for these repeated walks but was measurably more expensive over the
complete isolated suite. No claim of10×, unchanged peak memory in every schedule,
or a general allocator optimum is made. Independent review remains owed.

The final policy does not change Rust simulation/comparison code from33379f68.
It changes only per-child allocator selection and its tests/documentation.
Nevertheless, the required full gate is repeated on the final committed tree;
that also measures an already-populated fingerprint generation. Its verdict is
pending here and must be recorded after actual completion.

## Final landing verdict

Final implementation **`3b7e6846f3c251a33712322d4c2d167bcddfb71f`** passes the complete required-fixture gate,
exit0, all six phases: offline tooling, clippy, formatting, install survey,
release suites and final guards. Exact command:

```sh
/usr/bin/time -l zsh tools/memcap.sh 40 python3 tools/release_gate.py \
  /Users/rf-studio/ron-golden/ch80/map-14 --test-threads 4 --require-fixtures \
  --report-dir /tmp/attrition-tick-decode-20261016-01/gate-final
```

`gate-final.log` and `gate-final.exit` retain actual completion. Fixed13,
rondata890 passing plus6 unchanged ignored, sim2586 and3 doc tests all pass.
All3,276 requested fixtures are present (419 unique), and the exact request
multiset matches both the first gate and the preceding branch's full gate.
The tree stayed frozen during each full gate. No pins, scored words, expected
value divergences, comparison fields or required fixtures moved.

Final gate427.11 wall/1,438.50 CPU seconds, sampled tree peak8,089 MiB;
rondata313.32 seconds. Previous branch:433.29/1,422.75/9,910; time is roughly
flat (−1.4%), CPU slightly higher (+1.1%), peak memory18.4% lower. These are
observed gate runs, not a controlled claim of general speedup. The earlier
cold-generation481.21/1,625.32/8,553 result remains in the record.

The selected isolated contention sample saves CPU6%/7% and peak memory15%/20%
for four/six competing suites; elapsed is flat/−9%. The complete warm isolated
rondata suite saves about2% CPU/time, with variable aggregate memory (+5% in
that one selected run). All890 effective policy receipts match the intended
49 bounded /841 system choices. Independent review remains owed.10× remains
unmet; neither an individual-test gain nor contention sample is a full-suite10×.

Next measured candidate: live unit/radius scans during ticking. Before a broad
rewrite, profile whole-suite setup/parsing and ticking shares: the complete
isolated evidence shows gains in one coverage test are diluted across hundreds
of other tests. Do not return to frame-input indexing without new evidence.
The commander remains paused; these commits are on the user-directed branch,
not merged into the commander's checkout. A documentation-only verdict commit
follows the passing gate, with diff and paperwork guards checked separately.
