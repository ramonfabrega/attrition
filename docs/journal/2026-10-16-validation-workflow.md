# Shared validation workflow — 2026-10-16

User-authorized follow-up on `codex/astra-windowed-reader`, starting `614bd7a9`.
Draft PR10 remains open; Fable/commander stays paused. The user's operational
objective is less validation contention and worker waiting, not a hard count
of simultaneous suites or a promised sub-minute/1GiB result. Evidence root:
`/tmp/attrition-validation-flow-20261016-01/`.

## Audit and decision

Read QUEUE first, then the working agreement, CLAUDE architecture/conventions,
gate/guard/isolated runner, resource governor, Cargo manifests/config and entry
point tests. The existing host governor was enabled only by `--isolated`, which
rejected scoped and lane gates. Ordinary gate release commands bypassed it;
standalone guards and direct Cargo also bypassed it. Different worktree binaries
therefore could still independently saturate the machine even after the earlier
isolated-runner work. No assumption that their revisions or verdicts are equal.

The current host has16 physical cores/128GiB. Its existing shared policy remains
16 CPU slots/32GiB reservation, nice+10; no host config, capture locks, paused
checkout or shared cache policy is replaced. CPU reservations are cooperative,
not a kernel throttle. The selected change makes existing admission operational
at normal entry points, with explicit wait/execution measurement.

## Implementation

- Every gate mode defaults to isolation. `--combined` retains reference combined
  execution. Lane/scoped selection is still computed by the same timing, path,
  widening and required-filter logic; exact exclusions reach all test targets
  and doctests. The per-rondata manifest records selected/ignored/skipped names.
  New/unknown tests remain included and retain the conservative memory reservation.
- All non-dispatch phases acquire shared resources. Isolated dispatch acquires
  per child, avoiding a whole-budget lease held while requesting child leases.
  Combined wide gates now fit the shared32GiB ceiling instead of the old40GiB.
- Standalone guards acquire one conservative compilation-capable reservation.
  Gate-launched guards inherit an explicit child-only cooperative marker to
  avoid nested acquisition. Gates clear an externally inherited marker first.
- `zsh tools/check.sh {test|build|check|clippy|run} ...` supplies the same admission
  for standalone Cargo. Cargo jobs8/default test threads2; CLI arguments survive,
  unsupported commands fail, child failures propagate. The CLAUDE convention
  directs routine Cargo work through it. Raw Cargo/binaries and old checkouts
  remain bypasses; no global shell alias or PATH interception is installed.
- Every gate phase retains a log. Supervisors emit a JSON receipt of admission
  wait and admitted execution; per-test rows retain it alongside CPU/RSS. These
  timings include process launch/cleanup and are not CPU time. No strict fairness,
  short-job priority, hard CPU quota or queue-latency guarantee is introduced.
  Conservative build/guard reservations can increase waiting; measure before
  changing the policy. Do not wrap admitted entry points in a second reservation.

No simulation, comparison, capture, pin, value floor or result cache changes.
No existing test is removed. Full/scoped/lane are still distinct coverage claims;
using the isolated executor does not turn a scoped pass into a full-suite pass.

## Different revisions, large files and crate boundaries

`source-sizes.json` records harness24,548 lines, golden17,885, orders11,272,
testkit9,838 and fight9,334 at the starting tree. Cargo has three workspace
packages. `rondata` depends on `sim`; diff is a public production module, and
many other rondata modules reference sim. A simple file move is not a new Cargo
invalidation boundary. Splitting the harness from the data reader would require
explicit interfaces and a dependency audit, not mechanically moving godfiles.

Release incremental compilation is not enabled in this workspace (Cargo's
release default is false); debug/test defaults differ. Source:
[Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html).
No incremental/profile flags or crate boundaries are changed in this landing.
Previous build logs show recompilation costs, but no controlled small-edit build
comparison has been performed here, so a compile-time gain is not claimed.

Each revision builds/runs its own executables. Shared source observations keep
their existing parser/source identity checks; simulation verdicts and mutable
replays are not shared across revisions. Target directories are not combined.
The next build experiment should separate no-op, test-only, harness and sim edits
and measure compile CPU/RSS/wall separately from execution before choosing a split.

## Validation so far

New entry-point regression tests fail against the starting code (`red.log`):
combined release bypass, missing policy on ordinary/scoped/lane gates and missing
guard admission marker. The exact-selection test fails before implementation
(`selection-red.log`). After implementation, entry tests compare isolated and
combined exclusions for full/scoped/lane modes; preserve new prefix-matching
names; check nested-admission avoidance, guard filter/lane forwarding, standalone
Cargo entry and failures. Real six-process admission tests check concurrent slot
bounds, distinct timing receipts, cancellation/timeout/memory behavior and locks.
Existing no-fail-fast, fixture rejection, lane classification and manifest checks
remain. Offline suite:489 tests pass with3 unchanged skips (`offline-final.log`).
Entry tests40 pass. The first real guard correctly rejected the new shell script
without its executable bit (`guard.log`); chmod fixes that packaging omission.
Corrected guard passed (`guard-fixed.log`, exit 0, 27.98s admitted execution).
Final committed required-fixture gate verdict pending.

## Adoption and limits

This branch is the implementation and review surface; nothing is merged into or
executed by the paused commander. Each active worktree must adopt these tools
and conventions to cooperate. The change is admission and executor adoption,
not a claim that less simulation work is needed or that20 changing revisions
can all finish simultaneously. The user's actual four-worker-plus-commander
workflow is not resumed as a benchmark. Earlier independent review debt remains;
this landing also awaits Fable review. Pins and scoreboard remain unchanged.

## History-reader experiment

A standalone guard still spent most of its time starting Git once per historical
queue body. The history contains 2,208 bodies (1,141 QUEUE + 1,067 PARKED).
`queueledger.py` now exchanges one request/response at a time with `git cat-file
--batch`, preserving the same history traversal, body decoding and all ledger
checks. It fails on missing/wrong-type/truncated responses and checks the child
exit. No historical verdict cache, reduced history or Git configuration writes.
The child-local delta cache is bounded to 16 MiB.

Measured sequentially under the same admission and `/usr/bin/time -l`:

| Reader | Wall seconds | CPU seconds | Peak RSS MiB |
| --- | ---: | ---: | ---: |
| Original `git show` per body | 30.10 | 27.47 | 50.03 |
| Batched, default Git cache | 2.12 | 1.97 | 120.05 |
| Batched, selected 16 MiB cache | 2.13 | 2.08 | 55.38 |

The selected reader is 14.1× faster for this guard component and consumes 92.4%
less CPU; peak RSS increases about 5.3 MiB. This is not a 10× full-suite result.
Logs: `ledger-baseline.log`, `ledger-batched.log`, `ledger-bounded.log` under the
evidence root. All three emit identical ledger output. `ledger-differential.py`
compared every one of the 2,208 bodies with the original `git show` reader in
request order; exit 0, all equal (`ledger-differential.log`, per-body SHA256s in
`ledger-body-audit.json`). New authored-repository tests preserve repeated/empty/
CRLF/UTF8 blobs and prove an unaccounted historical deletion still fails; both
the failure and corrected-journal output equal the original reader. These tests
failed before implementation (`ledger-red.log`) and pass after (`ledger-tests.log`).

## Pre-commit validation

Final offline run:491 tests pass,3 unchanged skips,35.98s
(`offline-ledger-final.log`). Complete standalone `guard.sh --built` passes after
the reader change:4.29s admitted execution versus27.98s before, sampled79MiB
(`guard-ledger-final.log`). These are sequential warm samples, not a contention
benchmark. Shell syntax, `cargo fmt --all --check` and `git diff --check` pass.
No Rust or pin diff. `real-selection-audit.json` also compares the isolated
exact-name selection against real libtest discovery on the prior unchanged
rondata binary:896 total names, selected361/399/375 with no retained filter,
third_pair and coverage filters respectively; all names and their order agree.
This discovery check executes no simulation and does not replace the full gate.

Implementation is ready for its commit; the complete required-fixture gate will
run on that frozen commit, followed by a recorded verdict. No full-gate pass is
claimed at this checkpoint.
