# Core-loop cost investigation, 2026-10-09

User-authorized outside assessment by Codex (GPT-6), starting from `ecc19ef0`.
The commander and Fable loop remain paused. The objective is full-fidelity
simulation/rules, with more verified progress per hour and dollar; no renderer
acceleration or relaxation of evidence standards was requested.

## Scope and fidelity boundary

This is an investigation, not a mechanic landing. No score or value pin moves.
No capture, Wine configuration, shared executable, or original data was changed.
Temporary test-harness experiments were confined to this checkout and restored.
No original asset or derived proprietary state belongs in this report.

The XML boundary is explicit in DECISIONS 11: the sim receives typed tuning,
while `rondata` reads/checks the owned install's XML. Avoiding decompiler-body
transcription does not forbid reading rules data or implementing its semantics.
DECISIONS 20 similarly loads the owned AI scripts into our own interpreter.
Avoiding the original engine's design does not require recalculating expensive
derived quantities on every query; caches may be appropriate if all writers and
invalidation rules are established and tested.

The current loop already schedules AI and rules work concurrently, and item
1569 admitted a three-lane click-free capture pool. Do not diagnose an obsolete
single-capture bottleneck. The lane logs since October 7 show peak two held
lanes and zero waits at the pool cap (20 recorded captures). This is that
window's telemetry, not proof that capture contention can never recur.

The project has prior native profiling and performance work, including
`2026-09-09-native-profile.md`, streaming readers, shared text, and item 1571's
index-cache fix. The parked September statement that nothing was profiled is
historical, not a description of this tree.

## Baseline: three different costs

Built the current rondata test binary locally with
`CARGO_PROFILE_RELEASE_DEBUG=1 cargo test --offline --release -p rondata --no-run`.
Compilation took 52.37 s, outside test timing. Tests used the owned install,
one libtest thread, `tools/memcap.sh 20`, fixture audits, `RON_INDEX_STATS`,
`RON_READ_STATS`, and a ten-second native sample beginning approximately three
seconds after launch. Each was a separate process, with a cold application
index cache; filesystem caches were uncontrolled. These are not full-suite
throughput measurements, nor pure unsampled timings.

| exact test suffix | libtest time | sampled tree peak | index builds | index build time |
|---|---:|---:|---:|---:|
| `run226_s_word_frame_is_widened_whole` | 53.70 s | 1,393 MiB | 15 | 15.531 s |
| `every_key_the_dump_prints_is_read_or_pinned` | 118.22 s | 74 MiB | 171 | 88.112 s |
| `every_parsed_field_is_compared_by_the_instrument_or_pinned` | 112.81 s | 2,501 MiB | 39 | 10.145 s |

All three reported exactly one pass. Every fixture request recorded by their
audits was present. This is not a complete-corpus audit: in particular, golden
paths have a separate lookup. Initial sandboxed attempts exited 125 because
process sampling was unavailable; no test ran in those attempts. The authorized
escalated runs preserved the memory cap and sampled successfully.

The early run226 sample is dominated by index scanning. The coverage-fields
sample includes simulation ticks, `land_size`, `region_size`, `wonders_held`,
collision queries, parsing, and allocation. Sample frequencies are neither
whole-test CPU percentages nor interchangeable with the instrumented index
timers. The idle libtest main thread must be excluded from any denominator.

Item 1571 already tested persistent index sidecars: the four cold standalone
Great Lakes tests improved about 1.42x, below that brief's 1.5x threshold, and
the prototype was not adopted. It subsequently fixed in-process eviction,
reducing a whole rondata suite from 641 to 611 s. Re-proposing sidecars as a
new discovery would be wrong. Faster index scanning or persistent indexes
should now be judged on end-to-end daily savings and operational complexity.

## Accumulation: retain assertions, share execution

Four Great Lakes widenings all start comparing at block 11,400 and finish at
15,440, 15,859, 16,711, and 17,350. Their tests independently run the same
`widen_on_siblings` machinery:

- Compared block visits: 19,764, versus 5,951 in the union (3.32x).
- Simulation ticks: 65,360, versus 17,350 for one shared prefix (3.77x).

These are static work counts, not measured speedups. Setup differences,
recording side effects, diagnostic windows, and per-test assertions must still
be preserved. A union walk must retain enough per-frame events to derive each
interval's first differences; caching only one final `firsts` map loses
information. The thread-local parser/comparison coverage recorders must still
observe all the evidence. Reusing a cached return value blindly is unsafe for
the coverage claim even if ordinary test assertions pass.

The reader has a second duplication inside each compared frame: `frame_state`
reads and eagerly parses it, then the widening reads it again into a lazy Log
for groups, leaders, and other comparisons. The local experiment below measures
reusing the eager Log, with the same one-frame/number checks and all original
assertions. It changes neither sim behavior nor comparison coverage by design;
its focused tests, rather than that design claim, determine the outcome.

The instrumented control passed run226 in 53.47 s; reuse passed in 40.92 s
(23.5% less elapsed time, 1.31x throughput for this standalone test). Sampled
peaks were 1,396 and 1,395 MiB. All 86 diagnostic value/summary lines were
identical, including the widening counts and standing/first-difference rows.
The earlier sampled baseline of 53.70 s is consistent with the instrumented
control; this is still a single paired experiment, not a variance study.

| run226 phase | control | reuse |
|---|---:|---:|
| setup, including indexes | 16.761 s | 16.332 s |
| simulation ticks | 0.786 s | 0.803 s |
| typed frame decoding | 14.606 s | 15.248 s |
| core comparison | 1.568 s | 1.617 s |
| auxiliary reads/parsing/comparisons | 19.706 s | 6.873 s |

The temporary, unapplied [experiment patch](2026-10-09-frame-reuse.patch)
retains the exact source changes: timers plus an opt-in
`RON_PROBE_REUSE_FRAME=1` branch. It is not production code. No assertion is
removed; reuse still requires one decoded frame with the indexed frame number.
Apply only to the recorded baseline in an isolated checkout; without the env
variable it measures the original path. Build once, then run both modes in
separate processes. Restore the patch after the experiment.

The broader coverage-field check also passed under reuse: 111.30 s versus
112.81 s in the earlier sampled baseline, with sampled peaks 2,490 versus
2,501 MiB. All 9,828 diagnostic value/summary rows match exactly. The roughly
1.3% elapsed difference is not a meaningful speed claim from this comparison
(the baseline was sampled and this run was not).

The 38 instrumented walks inside that check simulated **300,067 ticks** to
compare **4,430 captured blocks**. Summed phases: setup 37.680 s, ticking
45.120 s, typed decoding 12.317 s, core comparisons 6.497 s, auxiliary work
5.321 s, total instrumented spans 107.094 s. These spans cover the instrumented
helper, not every operation in the 111.30-second test. They justify investigating
shared setup and replay prefixes. The parse-reuse win on run226 does not
transfer automatically to a check dominated by many short windows.

Both modes' audits recorded zero missing fixture requests. The harness source
was restored byte-for-byte to the starting Git version after the experiment,
and the preserved patch passes `git apply --check` on that restored source.

## Full-file compression, original evidence preserved

Read-only experiment: read 4 MiB at a time, encode each chunk as an independent
gzip level-1 member, decode it immediately, require exact byte equality, and
discard both buffers. SHA-256 covers the entire source; size/mtime are checked
before and after. No compressed archive was persisted or substituted into a
reader. Measurements are single runs on this machine with warm/unknown OS
caches; timings exclude archive writes and container/index implementation.

| source | original MiB | compressed MiB | ratio | encode | decode |
|---|---:|---:|---:|---:|---:|
| run721 detail log | 619.80 | 29.79 | 20.80x | 0.912 s | 0.118 s |
| run717 ammo log | 112.48 | 6.13 | 18.35x | 0.177 s | 0.024 s |
| run712 snapshot | 888.95 | 224.46 | 3.96x | 4.337 s | 0.653 s |

Every byte round-tripped on all three. This supports an indexed compressed
archive experiment, not a 20x prediction for the entire corpus or an adoption
decision. Independent chunks preserve potential random access, but readers
need an offset mapping, checksums, failure handling, and bounded decompression.
Frame-aligned chunks may be preferable; that was not tested. Converting only
cold evidence first would limit migration risk. No deletion is authorized by
these results, and no backup of the whole corpus is necessary for a prototype.

Disk began at about 81 GiB free. The earlier metadata inventory measured about
191 GiB under `ron-data`, including 125.27 GiB of `.txt` logical lengths,
22.27 GiB of `.log`, and 24.53 GiB in 31 named snapshots. These logical sizes
are not unique physical blocks: APFS clones/hardlinks and duplicate contents
must be accounted for before estimating reclaimable capacity.

## Capture time is not disk time

Existing receipt evidence, no new games launched:

| run | detail window as staged | launch to exit | tracer build |
|---|---|---:|---:|
| 712, includes snapshot at 1702 | [1701,1705] | 33.47 s | 9.38 s |
| 716 | [3137,3394] | 2019.56 s | 9.28 s |
| 717, extra ammo detail | [3104,3142] | 334.58 s | 9.31 s |
| 721 | [3389,3646] | 1604.44 s | 11.27 s |

These are different frames, detail settings, and fast-forward settings, not a
controlled logger benchmark. Still, setup compilation alone cannot explain
the long detailed runs. Post-capture compression does not remove the original's
formatting/logging cost. A useful future controlled experiment would hold lobby,
frame, seed, instrumentation and fast-forward constant, vary detail window
width, and measure original logger formatting versus writes before changing
buffering or capture representation. The September 9 methodology report
already read the logger: wide-character formatting, indentation writes, and
many subsystem flushes, with the keep-open flag set. A fresh read of
`GameLog::init@00933190`, `Log::init@00a3baa0` and `Log::say@00a3ab00` /
`00a3b160` agrees with that narrower claim. The older runbook's "flushes per
line" wording does not establish a file reopen/close on every ordinary integer
field. This is source-reading evidence, still provisional and owed independent
review; no logger speedup was measured. That experiment would stage shared Wine
state and therefore needs its own explicit execution plan and backup/restore.

## Priority after this investigation

1. Reduce repeated parsing/replay while preserving every range assertion and
   coverage side effect. Evaluate union walks on the known overlapping family.
2. Build an opt-in indexed compressed archive reader against a small retained
   original sample, with exact replay and corruption checks. Do not migrate the
   corpus before those checks and random-access measurements pass.
3. Profile and optimize derived region/land queries: `region_size` scans the
   region grid; `land_size` scans cells separately per land region. Measure the
   effect before introducing caches, and test all region mutation paths.
4. Schedule focused item checks and one full required-fixture gate per bounded
   integration batch, recording the regression-isolation cost as well as time.
   No gate policy is changed here.
5. Increase direct function/subsystem differential evidence using existing
   snapshots and synthetic inputs. Track matched branches separately from
   merely cited, entered, or discrepancy-backed functions. Keep full-game
   replay checks for their integration role.

## Local evidence and reproduction

Artifacts are retained under this checkout's
`target/core-loop-investigation/`: `build.log`, `profiled/` (test logs, samples,
fixture audits and index statistics), `codec_probe.py`, `codec-results.json`,
and the temporary phase/reuse experiment and its logs. They are ignored local
evidence, not original data exports committed to git. The install is
`/Users/rf-studio/code/fun/attrition/game`; captures are read from their existing
`/Users/rf-studio/ron-data` and `ron-golden` locations.

The existing `tools/explore/profile_native.py` can reproduce a native sample
of any exact test in the table. Run it under `tools/memcap.sh 20`, use a fresh
output directory in this checkout, and export `RON_INSTALL`. The exact
test names are under `diff::harness::tests::` (run226) and `diff::coverage::`
(both coverage checks). Fixtures remain read-only.

This is single-agent executable evidence and source inspection, not an
independent review. There is no blind-review ratification or complete-sim claim.

## Validation and handoff

Investigation only: draw-stream scores unchanged, value-difference assertions
unchanged, no new mechanic or floor. Three baseline focused tests passed; the
instrumented control and two reuse tests passed, with the exact diagnostics
comparisons above. No complete release gate was run: the final active sim,
parser, harness, and comparison code are identical to `ecc19ef0`. The patch is
an unapplied experiment artifact, not an adopted optimization. Report/link,
patch-application, and queue-ledger checks are the relevant final checks.

The local target and reports together occupy about 313 MiB; the volume still
has about 81 GiB free. No capture, backup, configuration, executable, or archive
outside this checkout was written. The investigation branch is separate from
main and from the paused commander's branch. The next useful implementation is
one bounded execution-sharing experiment, preserving all four Great Lakes
interval assertions; alternatively, an opt-in compressed-reader prototype can
address storage without staging another game. Neither is started by this report.
