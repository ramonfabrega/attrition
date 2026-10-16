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

Thus7.4–7.7× less CPU,77–83% lower peak RSS, and7.3–7.7× less test wall time
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
