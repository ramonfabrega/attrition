# Third-pair repeated replay — 2026-10-16

User-authorized sequential follow-up to the shared runner and setup persistence.
Branch `codex/astra-windowed-reader`, starting tip679ffe1b; commander paused.
No subagents, captures, score changes, or simulation-rule changes. Goal: reduce
repeated setup/ticks while preserving all comparisons, within the10× suite effort.

Evidence root: `/tmp/attrition-third-pair-20261016-01/`. Original release binary
preserved as `baseline-rondata`. Profiling-only original-order run: `profile.log`,
`profile-fixtures`, `baseline-profile.json`. All24 windows pass; setup9.605s,
ticking24.727s, summed window time35.860s; unit-test36.57s. The external memory
wrapper reports38.56real/36.38CPU and3309830144bytes maximum RSS. Monitor polling
adds wall time after test completion, so keep test and wrapper times distinct.

Implementation moves unchanged setup into `third_pair/replay.rs`, sorts compatible
windows forward, retains one Built (never clones it), and keeps all existing
projectile/group-dumper checks untouched. Source/trace requests still occur for
every window. Future observation cutoffs are the main semantic hazard: initial
24-window audit matched the full Built representation at every compared frame,
including RNG history and state, with original cutoff/from-zero reconstructions.
It avoided233874 repeated ticks. `audit.log` passes in60.27testseconds; this includes
fresh reference walks and full-state comparisons and is not a speed measurement.
A compile-only borrow error during helper extraction was corrected before this
run; retained `candidate-build.log` and `candidate-build2.log` show the history.

The first baseline profile is instrumented, same algorithm; planned alternating
same-binary disabled/enabled runs avoid attributing compiler differences to reuse.
`measure.py` retains each mode's exact fixture rows and full projectile summaries,
asserts equality, then repeats the whole-state audit on the final binary. Logs
and parsed results remain outside git because complete oracle differences are
local evidence. Focused3 helper tests passed (`replay-unit.log`), clippy and
formatting passed. The mutation test deliberately catches assertion failures;
passing it means every intended wrong state was rejected.

Final measurements, implementation commit and complete required-fixture gate:
pending. Pins/scoreboard unchanged. Independent review debt: source metadata
boundary, observation horizon equivalence and window scheduling; executable
differential evidence is not an independent reviewer. Commander remains paused.


## Same-binary measurements and focused validation

Alternating runs, each exits0 (external wrapper seconds / CPU seconds / RSS bytes):
- off1:40.59 /38.29 /4569399296; test38.41s.
- on1:6.11 /4.97 /790822912; test5.02s.
- off2:36.57 /36.65 /3618799616; test36.51s.
- on2:6.10 /4.99 /818642944; test4.97s.

Thus7.3–7.7× less CPU,77–83% lower peak RSS, and7.3–7.7× less test wall time
in these warm repeats. Monitor polling accounts for extra wrapper wall time.
256890 from-zero ticks become23016, with233874 avoided (91.0%). All24 windows
and360 compared frames remain. Exact96 fixture request rows, all24 complete
projectile count/difference/unmodelled summaries match the original algorithm,
same-binary disabled mode, reuse mode and final audit. Missing requests:0.

`audit-final.log` on the final binary passed its full-state comparisons at all360
frames. No later simulation state is persisted or shared beyond this one test.
Three mutation/key/overlap helper tests pass. `cargo clippy --all-targets`,
`cargo fmt --all -- --check`, `git diff --check` and `zsh tools/guard.sh --built`
all exited0. Logs: `replay-unit.log`, `clippy-final.log`, `prelanding-guard.log`.
Committed summary: `docs/audit/2026-10-16-third-pair-replay.json`.

Implementation is ready for commit and the required ordinary full gate. The
shared runner and its validation coverage did not change, so this landing uses
one complete required-fixture gate rather than repeating both full gate modes.
No fixture assertions or comparison coverage are removed by that choice.


## Full release run and isolated bookkeeping correction

Implementation `4627c92c` was frozen for:
`python3 tools/release_gate.py /Users/rf-studio/ron-golden/ch80/map-14 --test-threads 4 --require-fixtures --report-dir /tmp/attrition-third-pair-20261016-01/release-gate`.
Actual gate exit1, recorded in `release-gate.exit`: five of six phases ran;
guard was not reached. The entire no-fail-fast release suite ran: fixed13,
rondata883 passed/1failed/6ignored, sim2585, rondata doctests3. The sole failure
was `ledger::tests::every_differ_module_is_on_the_ledger`: the new helper file
needed registration. This is not a passed full gate. All replay/value tests
passed. Fixture TSV multiset exactly matches the preceding complete gate:
3276requests,419distinct,0missing. `release_completed` is false because Cargo
reported that one failure; the complete per-binary test outcomes are retained.

Exact corrective diff: add `diff/third_pair/replay.rs` to `ledger.rs`'s
NOT_A_DIFFER with the reason that it contains setup/reuse and fresh-state audit;
all original-record comparisons remain in third_pair.rs. No parser, simulation,
window, comparison or input-coverage logic changes. Per AGENTS combined-validation
exception, rerun all ledger tests, guard, clippy and formatting, and verify fixture
completeness instead of rerunning all game data. Correction verdict pending.

The failed gate took400.66wall/1410.96CPU seconds, sampled tree10071MiB;
rondata308.53s. Because the guard phase was not reached and this is one run,
these are recorded measurements rather than a clean passed-full-gate speed ratio.
The focused alternating runs establish the7.3–7.7× CPU reduction for this item.


## Correction verdict and four-suite resource result

All3 ledger tests passed after the four-line registration (`ledger-correction.log`,
exit0). `cargo clippy --all-targets`, `cargo fmt --all -- --check` and
`zsh tools/guard.sh --built` passed (`correction-clippy.log`, `correction-guard.log`).
This is combined validation of the complete release run plus the isolated ledger
correction, not a claim that the original full gate passed. No simulation or
comparison logic changed after the full run. All fixture requests remain present.

The original eight-test/four-suite contention experiment was copied to this
item's evidence root as `contention.py`; `contention-compare.py` executes it twice
sequentially. Both modes use the same corrected binary, same shared16CPU/32GiB
admission, same eight tests per suite, same longest-first hints, same warm sources,
and same outer40GiB memory monitor. Only RON_DISABLE_THIRD_PAIR_REUSE differs.
Both inherit identical nice policy (outer monitor plus inner supervisors).

| Four-suite sample | Fresh replay | Reuse |
| --- | ---: | ---: |
| Sample elapsed seconds |52.444|52.305|
| Whole wrapper CPU seconds |481.34|326.81|
| Sampled tree peak MiB |18503|11896|
| Test executions passed |32|32|
| Fixture requests / missing |1832 /0|1832 /0|

Exact fixture multisets match. CPU falls32.1% and sampled memory35.7%; elapsed
is essentially unchanged. One paired sample, not a universal concurrency claim.
All resource leases released after completion. Full outputs and per-test results
are in `contention-isolated-off/`, `contention-isolated-on/`, both outer `.log`
files and `contention-comparison.json`. Wrapper wall53.33/53.84s includes monitor
polling; use the sample elapsed above for work completion. No corpus cleanup or
capture configuration changes were made.

The remaining critical path in all four reuse suites is
`every_parsed_field_is_compared_by_the_instrument_or_pinned`:44.90–46.29CPU seconds,
47.53–52.26elapsed including admission. Third-pair savings free resources but do
not shorten that path. Next measured candidate is profiling that coverage walk's
setup, resumed ticks, frame decoding and comparison stages; retain every one of
its windows, field sets and assertions. No implementation for it is claimed here.

For the Loop: third-pair24-window replay uses about7.5× less CPU and77–83% less
peak RSS individually; four-suite sample uses32% less CPU/36% less sampled memory,
with no meaningful wall improvement. Scores and value diffs unchanged.10× full
suite target unmet. Combined validation complete; independent review debt remains.
Commander paused; this branch is not merged into its checkout.

Final paperwork: `zsh tools/guard.sh --built` exit0 after the final queue/spec
updates (`final-paperwork-guard.log`); formatting, diff, document links and audit
JSON checks passed. Only the ledger registration and documentary evidence follow
the fully exercised implementation commit.
