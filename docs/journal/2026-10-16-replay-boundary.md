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

## Full run and source-guard correction

Implementation `e8404b99` was committed before the complete run; tracked files
were frozen until its actual exit:

```sh
/usr/bin/time -l zsh tools/memcap.sh 40 python3 tools/release_gate.py /Users/rf-studio/ron-golden/ch80/map-14 --test-threads 4 --require-fixtures --report-dir /tmp/attrition-replay-boundary-20261016-01/gate
```

**Actual gate exit1**, retained in `gate.exit`: five phases reached; final guard
was not reached. Offline500 tests ran with3 skips; lint, format and survey passed.
Every release test ran: sim2586/fixed13/doctests3 pass; rondata887 pass,3 fail,
6 unchanged ignored. All211 golden cases pass. Exact3276 fixture requests
(419 unique,0 missing) equal the preceding full gate. The three failures are:

- `diff::floors::tests::a_widening_s_test_holds_its_pins_under_the_guard`
- `ledger::tests::the_widening_ledger_counts_what_nothing_compares`
- `ledger::tests::every_differ_module_is_on_the_ledger`

They scan source: the pin check saw wrappers without pin holds, and the ledger
omitted the new body files. This caught missing structural integration; these
failures are not ignored or reclassified as passing. The failed whole run took
251.77s/1401.93CPU/7775MiB sampled tree; rondata160.34s. Since the last guard phase
was not reached, these are not a successful whole-gate speedup measurement.

Correction `a30a36bd` changes only source-guard plumbing and adds one authored
source-guard test. `testenv/source.rs` substitutes each wrapper's original body
at the same logical location, refusing missing, duplicate, orphan or misdirected
bodies. Floors runs its unchanged pin checks on that expansion; ledger runs its
unchanged field/site checks on it and accounts for all registered case files.
No simulation, replay, comparison function, expectation, floor or pin changes.

Golden source checks now read the current checkout, consistently with floors'
existing source guards. They require that source tree, and validation freezes it.
The ledger no longer embeds the whole golden source in another compiler module
with `include_str!`; that would reintroduce invalidation on every case edit.
Other ledger source inputs remain unchanged. This is a source-check input change,
not a change to the comparison recorder or which fields are held.

The ledger's entire printed result equals the saved original binary exactly
(`ledger-equivalence.txt`):25 records/328 fields,9 uncompared and31 single-capture;
existing pins10/36 stay unchanged. Removing a `Pins::hold()` from an extracted
case makes the existing pin guard fail, then restoring it passes
(`source-fault.py`, `source-fault-red.log`, `source-fault-restored.log`).

The final corrected layout still measures:

| Edit | Original wall/CPU/MiB | Corrected wall/CPU/MiB |
|---|---:|---:|
| Setup case |10.27 /24.22 /1444|2.99 /3.50 /818|
| Chapter-one case |10.44 /24.57 /1400|4.35 /6.35 /803|

This is2.4–3.4× faster rebuilding those cases,74–86% less CPU and about43% less
sampled memory, with all source checks restored. The shared-helper measurement
above predates this guard correction; no extra improvement is inferred for it.

## Combined validation

Use AGENTS.md's explicit source-structure/paperwork correction allowance to reuse
the completed replay results. **This is combined validation, not a claim that
the original full gate passed.** The exact corrective diff is
`git diff e8404b99 a30a36bd`; all211 replay bodies remain untouched.

Affected ledger tests3/3, floors22/22 and the new source contract1/1 pass.
Offline500 tests rerun with3 unchanged skips; original lint command and formatting
pass. All original896 rondata names/ignored states are retained, plus exactly one
new guard test (897 discovered,6 ignored); sim2586 names unchanged. The frozen
release fixture audit still has the exact3276 requests and0 missing.
`compare.py`/`gate-equivalence.json` retain the original failed summary, fixture
equality and the explicitly named new guard rather than hiding a test-set change.
Final paperwork guard/link/diff checks remain to be recorded below.

Keep the ten chapter modules. Independent Fable review remains owed, particularly
the source-check reconstruction. Scores, pins, draw/value comparisons stay fixed;
whole-suite10× remains unmet. Next measure a similar boundary in another replay
family. No merge or commander restart performed.

Final combined-verdict checks passed: complete guard exit0
(`guard-corrected.log`), formatting, whitespace, audit JSON and local links.
No second complete corpus run is claimed.
