# Release rebuild cost — 2026-10-16

User-authorized sequential performance exploration, starting `1156437a` on
`codex/astra-windowed-reader`. Fable remains paused; no subagents or other
worktrees are used. Evidence root: `/tmp/attrition-build-cost-20261016-01/`.
No capture, fixture, sim mechanic, comparison, score or value pin changes.

## Question and controlled probes

The user's workers edit different revisions; a no-op build or shared test verdict
is not the relevant measurement. Measure compiler work after small changes before
splitting crates. Three cumulative, reversible probes:

1. Change one assertion diagnostic in `diff/third_pair.rs` (test-only).
2. Initialize `FrameResult` with default then assign its frame in `diff/harness.rs`.
3. Return a named default local from `no_profile` in `sim/fight.rs`.

These change compiler input without changing passing behavior. They are small
behavior-preserving proxies, not a claim about arbitrary new algorithms or type
changes. The first probe recompiles only the rondata test library; harness edits
also rebuild its production library and dependent examples; sim edits rebuild
sim and rondata test/production targets and dependent examples. Nothing is skipped.
`measure.py` records original files, applies one edit, waits for the actual build
exit, and restores all originals in `finally`; `repeat.py` repeats the sequence.
Their source, originals, restoration hashes, JSON Cargo artifacts and per-build
logs remain under the evidence root. Restored source bytes equal Git HEAD.

Each build runs `/usr/bin/time -l zsh tools/check.sh test --workspace --release
--no-run --timings --message-format=json`: shared32GiB reservation,8 Cargo jobs,
nice+10. This is compilation only, not a passing test-suite claim. Baseline uses
the starting manifest (incremental off,16 codegen units). Candidates use separate
fresh `CARGO_TARGET_DIR`s and child-local `CARGO_INCREMENTAL=1`, with explicit
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16` or256. Inputs/corpus are not copied.
Toolchain: rustc1.97.1/LLVM22.1.6,aarch64-apple-darwin. No competing experiment.

Cargo changes the default partition count with incremental compilation, so the
16-partition control isolates caching from finer partitioning. See the primary
[Cargo profile documentation](https://doc.rust-lang.org/cargo/reference/profiles.html).
`build-summary.json` and the tracked audit retain every result and rebuilt target.

## Build measurements

Ranges below are two sequential runs, not confidence intervals. CPU is user plus
system including the supervisor; memory is sampled process-tree RSS in MiB.

| Edit | Original wall s | Selected wall s | Original CPU s | Selected CPU s | Original peak MiB | Selected peak MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Test diagnostic |24.72–25.75|16.47–16.84|103.46–107.89|19.59–20.00|2094–2150|1245–1271|
| Harness |25.15–27.36|18.83–18.84|138.60–146.58|89.87–89.98|2104–2149|1616–1701|
| Simulation |43.98–46.87|18.61–19.25|276.56–289.04|108.49–108.80|4013–4288|2344–2586|

Selected: incremental enabled,256 partitions. The16-partition caching control
was17.95/19.50/19.12s wall and78.02/107.12/109.95 CPU seconds for the same edits.
The extra partitioning mainly saves CPU in the test-only edit; simulation-edit
wall gains largely come from incremental reuse. No-op builds stay0.10–0.13s.
The selected fresh cache initially takes49.71s/331.94 CPU seconds/3269MiB peak;
this has no paired cold baseline and is not advertised as a cold-build win.
Both candidate caches occupy about1.7GiB each after the first sequence. This is
additional local disk usage, separate from RAM; no cross-worktree reuse assumed.

## Runtime screen

Copied each experiment's final binary to a distinct retained path. Those binaries
include the same three behavior-preserving probes. `runtime.py` runs the existing
eight-test sample through the production isolated runner,8 workers and unchanged
allocator/admission policy. The names and explicit exclusions are in manifests;
this is a sample, not the full suite. All8 pass in each mode, and the exact458
fixture requests match with none missing:

| Profile | Sample wall s | Test CPU s |
| --- | ---: | ---: |
| Original |35.39|65.14|
| Incremental16 |32.35|60.48|
| Incremental256 |32.47|60.01|

Original/selected sampled process-tree peaks:5242/4859MiB. This single runtime
screen finds no regression but does not establish a general execution speedup.
The full committed-tree gate must validate the actual restored sources/profile.

## Selected change and validation

Set `[profile.release] incremental=true` and `codegen-units=256`. Optimization
level3, explicit overflow checks, unwind behavior, test assertions, test selection
and all fidelity checks stay intact. Cargo caches compiler intermediates, not
verdicts. No crate extraction, shared target directory, install relocation or
process-wide compiler wrapper is introduced. Cargo's ordinary environment
controls remain available. To reproduce the original compilation policy:

```sh
CARGO_INCREMENTAL=0 CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 zsh tools/check.sh test --workspace --release --no-run
```

The new policy regression fails on the old manifest (`policy-red.log`) and passes
with the selected profile (`policy-green.log`). All temporary Rust edits restored.
Offline checks, guard and full committed required-fixture gate are pending here;
implementation will be committed before the full gate and the actual result will
be recorded separately. Independent Fable review remains owed. Full-suite10× is
unmet; these measurements address the build component after small edits.

## Next boundary

The remaining16–19s per small build and replay execution still cost time. Do not
infer that file splitting fixes this: a new compilation boundary must isolate
specific dependencies and reduce measured rebuild work. First observe worker
wait/execution receipts with this policy adopted; then consider extracting test
families or a harness crate using measured invalidation and preserved discovery.

Pre-commit checks:492 offline tests pass,3 unchanged skips (`offline.log`),
7 focused runtime/build policy tests pass (`policy-green.log`); the complete
`guard.sh --built` passes (`guard.log`, exit0). Shell/tool behavior is unchanged.
Formatting, audit JSON parsing, local documentation links and `git diff --check`
pass. Full required-fixture gate remains pending the implementation commit.
