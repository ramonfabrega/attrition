# Harness test compilation boundaries

Sequential continuation from `fd360bbb` on `codex/astra-windowed-reader`.
Commander remains paused. Independent review remains owed.
Evidence: `/tmp/attrition-harness-boundary-20261016-01/`.
[Audit](../audit/2026-10-16-harness-boundary.json).

## Boundary and invariants

The24548-line harness mixes production comparison code, shared widening helpers
and large replay tests. Move85 ordinary nested test bodies into nine child files,
in original source order, ten per group except the last five. Keep original
`#[test]` wrappers, names, attributes (including the ignored diagnostic), private
helper APIs, fixtures, recorder, Cargo targets, profiles and runner unchanged.
The parent becomes8433 lines. Parameterized widening helpers and the top-level
label guard remain in place.

Each file has an inner `bodies` module and re-exports its functions only to the
parent test module. The extra scope preserves the original indentation exactly,
including whitespace inside string literals. All85 body byte hashes match the
original. Replacing wrappers with their bodies and removing module registrations
reconstructs the entire original file, including helpers and documentation.
`equivalence.py` and `body-equivalence.json` retain that proof. No assertions,
floors, comparison logic, simulation behavior or expected values change.

New harness tests use a same-named wrapper and body in an existing group. Keep
original docs and attributes on the wrapper. Python registration tests require
all source files to be registered, every test to have exactly one correctly named
body, and no body to be a second libtest test. Authored negatives reject missing,
unmounted, duplicate and misdirected bodies. The compiler checks visibility.

## Experiment and decision

`measure.py SOURCE LABEL` warms, changes only run57's assertion diagnostic,
builds every workspace release target without running tests, restores the source
in `finally`, rebuilds, saves the binary and inventories test names. Command:
`/usr/bin/time -l zsh tools/check.sh test --workspace --release --no-run
--message-format=json`. Existing eight-job/shared32GiB admission is unchanged.
Admission waits were below0.01s. These are local warm edit proxies; no cold-build,
shared-helper edit, concurrent-worktree or full-suite speedup is inferred.

| Layout | Wall s | CPU s | Sampled tree MiB |
|---|---:|---:|---:|
| Original |19.52|52.64|1566|
| Nine groups, first measurement |15.53|21.14|1232|
| Seventeen groups |15.17|20.40|1226|
| Eighty-five one-case groups |15.29|20.45|1216|
| Nine groups, final guard integration |15.30|20.95|1209|

Retain nine groups: additional partitions do not establish a useful improvement.
The final edit is22% faster, consumes60% less CPU and23% less sampled memory.
This is a smaller elapsed-time benefit than golden's chapter split; extending
that result blindly to this family would have overclaimed. First compilation of
the final layout takes19.24s/100.24CPU/1676MiB; it is not a cold-build comparison.
No result claims that whole-suite10× is achieved.

## Source guards and failure evidence

The existing logical-source reader now accepts nested wrappers and reads harness
case bodies from the current frozen checkout, like golden. It checks exact target
names, single-call wrappers, unique bodies and complete consumption. Ledger and
floor guards use the reconstructed parent at the original body positions; child
sources are on the ledger inventory. This preserves per-test field counting and
avoids embedding changed child source text in a shared compiler unit. Other
source guards still inspect all applicable files normally.

Focused receipts:
-11 runtime-policy tests, including two new harness registration checks.
-3 ledger,22 floors,2 source reconstruction,2 slot guards and1 label guard pass.
-Entire printed widening ledger equals the original saved binary:25 records,
 328 fields,9 uncompared and31 single-capture; floors remain10/36.
-All original workspace test names retained; one Rust source guard added.
-Change the moved live-building assertion to expect the dead holder: original
 exact test name fails101 with an equality failure, restores to exit0.
-Remove `Pins::hold()` from the moved run80 widening body: already-built source
 guard fails101 and names the missing hold; restore passes. This proves the
 guard reads actual moved checks rather than accepting the wrapper.

`validate.py`, `fault-*.log`, `source-fault-*.log`, `*-focused.log` and
`focused-equivalence.json` retain commands and receipts. No score, draw-stream,
value-diff or fixture change is intended. Full required-fixture gate is pending
on the implementation commit; this checkpoint is not a completed landing.

## Review debt and next evidence

Fable should review the nested module visibility, source reconstruction and the
retained name/fixture evidence; this session does not claim independent review.
After the full gate, choose the next item from measured admission and runtime
costs. Finer harness partitions have diminishing returns. Global10× and sub-minute,
sub-GiB whole-suite operation remain unmet, and are not implied by build savings.

## Initial gate interruption

The complete-gate attempt on `66daa64e` exits1 after two phases: all502 offline
tests pass (three unchanged skips), then Clippy rejects the original run90
constant arithmetic equality now located in a helper rather than a `#[test]`
function. No release tests ran. Add a function-scoped `#[expect(clippy::eq_op)]`
with the coordinate-check rationale; the assertion and all body bytes remain
unchanged. This is not combined validation: a complete six-phase gate will run
again on the correction commit. Original receipts remain at `gate/` and
`gate.log`; the rerun uses `gate-final/` and `gate-final.log`.

## Complete verdict

All six required-fixture gate phases pass on `5491fe87`, exit0. Tree stayed clean
and frozen throughout. `gate-final.log`, `gate-final.exit`, `gate-final/` and
`gate-equivalence.json` hold the actual receipts. This is a complete gate, not
combined validation of the earlier interrupted attempt.

-502 offline tests, three unchanged skips; Clippy, formatting and survey pass.
-Rondata892 pass, six unchanged ignored; sim2586, fixed13, doctests3 pass.
-All original workspace names preserved exactly, plus the one source guard.
-Exact3276 fixture-request multiset equals the previous full replay run;
 419 unique fixtures, zero missing. Original ignored names preserved.
-Whole gate271.28s/1472.54CPU/8864MiB sampled tree peak, versus the last successful
 six-phase pre-golden gate255.00s/1396.42CPU/8369MiB. These are unpaired complete
 runs with different compile work: all three measures rise, so no full-suite
 speedup or memory reduction is claimed.
-Rondata execution161.22s/1158.05CPU versus160.34s/1149.77CPU in the preceding
 golden run (whose three source guards failed). Runtime remains roughly flat.
-Score/pins, draw-stream and value comparisons: no movement; no fidelity change.

The new runtime ranking (`runtime-ranking.json`) attributes298.56CPU to golden,
242.02 to harness,118.54 to second-pair and72.79 to coverage. Golden+harness are
47% of rondata CPU. Next investigate repeated golden/harness replay work, beginning
with profiling and equivalence evidence; additional harness file splitting has
not earned further complexity. This ranking does not prove replay reuse is safe
or predict a particular gain. Independent review and the global10× target remain
open.
