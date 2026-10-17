# Golden replay test compilation boundaries

Sequential user-authorized continuation from `0dacafe4` on
`codex/astra-windowed-reader`. Commander remains paused; independent review owed.
Evidence: `/tmp/attrition-replay-boundary-20261016-01/`.
[Audit](../audit/2026-10-16-replay-boundary.json).

## Why this boundary

Replay tests share private comparison instrumentation and a9838-line testkit
that reaches across families. Moving these directly into integration targets
would require a larger support-API refactor. Instead retain the entire rondata
crate, its private APIs, single compared recorder, fixture audit, runner and
Cargo targets. Split only the golden test bodies into ten ordinary child modules
by chapter decade, plus setup contracts. Original `#[test]` functions remain in
`diff::golden` and call their same-named body exactly once. This preserves libtest
names, attributes, thread names, fixture labels, exact filters and failure
propagation. Nothing caches a verdict or shares mutable state between tests.

All211 original bodies are byte-identical; reversing the wrapper substitutions
and module registrations reproduces every original helper and doc comment
(`body-equivalence.json`). Helpers and pins remain in `golden.rs`; extracted
bodies are under `diff/golden/cases_*.rs`. No production changes, public API
widening, assertion changes, expected-value changes or fixture relocation.

New golden tests should use a same-named wrapper and body in the appropriate
chapter module. The runtime-policy tests require every wrapper to call its own
body, exactly one registration per source and no orphan or duplicate bodies;
authored negative cases reject missing, misdirected, duplicate and unmounted
registrations. The ordinary Rust compiler checks visibility and calls.

## Experiments and rejected alternatives

All builds use `/usr/bin/time -l zsh tools/check.sh test --workspace --release
--no-run --message-format=json`, the existing8-job/shared32GiB admission policy,
and incremental compilation. Scripts restore exact sources and manifest in
`finally`. Logs include actual child exits and admitted wait/run time.

- Doubling only rondata's codegen units256→512: shared-helper edit11.52→10.22s,
  CPU24.23→24.05s, sampled tree1269→1399MiB. Reject: no CPU gain, more memory.
- One explicit test-body module: single-case edit10.27→2.03s, CPU24.22→2.75s.
  The11MiB sampled peak misses a short build and is not a memory claim.
-211 macro-generated modules:16.23s/55.73CPU/1551MiB for that edit;512 compiler
  partitions still16.06s/38.12CPU/1384MiB. Reject both. No macro retained.
-211 explicit modules:8.57s/9.54CPU/1204MiB. CPU improves but wall benefit is
  small; prefer the simpler ten chapter groups below. These measurements do not
  isolate a compiler-internal cause; macro invalidation/partition merging were
  hypotheses, not established explanations.

## Retained measurements

`case-measure.py` changes only the setup contract's assertion diagnostic;
`group-measure.py` makes the same edit in the extracted file. `measure.py`
changes the shared `walk_script` diagnostic. Restored binaries are retained.

| Edit/layout | Wall s | CPU s | Sampled tree MiB |
|---|---:|---:|---:|
| Setup case, original |10.27|24.22|1444|
| Setup case, chapter groups |3.12|3.69|785|
| Setup case, repeated |3.01|3.66|828|
| Shared helper, original |11.52|24.23|1269|
| Shared helper, chapter groups |9.85|24.89|1130|

The case rebuild is3.3–3.4× faster, uses about85% less CPU and43–46% less sampled
memory. Shared-helper CPU is essentially flat. These are local warm incremental
edit proxies, not cold-build, arbitrary sim-edit, multi-worktree or whole-suite
speedup claims. The first compile of the new layout takes19.72s/94.24CPU/1679MiB;
that is a one-time layout change, not a paired cold-build result. Compiler profile
and runner budgets stay unchanged.

## Validation and review debt

The saved original and grouped binaries list exactly the same896 rondata tests,
including the existing ignored tests (`manifest-equivalence.json`).
`fault.py` deliberately changes the setup contract's expected count0→1:
selection through its original exact Cargo test name fails with exit101 and the
equality assertion, then passes after exact source restoration (`fault-red.log`,
`fault-restored.log`). A skip or zero selected tests cannot satisfy this proof.
Runtime-policy tests9/9 pass, including the new registration failure cases.

Largest chapter-group control (`chapter-measure.py`,54 cases): the same chapter-one
diagnostic edit10.44→4.38s/CPU24.57→6.42s/sampled tree1400→905MiB. This is2.4×
faster,74% less CPU and35% less memory. Complete committed required-fixture
validation remains pending. Implementation will be committed before the full gate;
the tracked tree will remain frozen during it. Scores, pins and compared values
do not move. No independent audit, merge or commander restart is claimed.

Pre-commit guard passed (`guard.log`, exit0,4.51s admitted execution). Final
runtime-policy tests9/9, formatting, whitespace, local links, audit JSON and
exact body/helper/doc equivalence checks passed. Full gate remains pending.
