# Public-API sweep extraction

User-authorized sequential continuation from `1273edde`, on
`codex/astra-windowed-reader`. Commander remains paused; no lanes or independent
review launched. Evidence: `/tmp/attrition-public-sweeps-20261016-01/`.
[Audit](../audit/2026-10-16-public-sweeps.json).

## Boundary and evidence

The preceding extraction used absence of `crate::` as a conservative boundary.
That syntax alone does not imply a private dependency. A temporary integration
target re-exported `sim::*` and mounted the remaining309 sweep modules unchanged.
The compiler rejected108 modules initially and5 more after resolving the first
errors; the remaining196 compiled (`check.jsonl`, `public-check.jsonl`,
`public-final.log`). Rejections include shared test-only helpers as well as
private functions/methods. They remain in the library; this is not a claim that
all113 require private production APIs. The authored probe was saved in the
evidence directory then removed from Cargo discovery.

Move those196 modules into the existing four alphabetical partitions. The test
entry points re-export only the library's existing public API, preserving the
original `crate::` paths. No production visibility or code changes. All1046
registered sweep bodies, plus the pre-existing unregistered file with spaces in
its name, are byte-identical to HEAD (`body-hashes.json`, Git batch comparison).
No helpers, floors, expected values, fixtures or oracle scripts change.

The registration guard still enforces exactly one owner and correct Cargo
mounts. For a partition containing `crate::`, it now requires the public API
bridge; Rust compilation enforces actual privacy. New generated `crate::` tests
still conservatively start in the library; promotion requires compilation.
Regeneration preserves ownership. A new guard case fails without the bridge and
passes with it. Private access still fails compilation, as the initial probes
show. This replaces the old textual refusal, not Rust's visibility checks.

## Measurements

Sequential release incremental256 builds with the existing shared admission:

```sh
python3 /tmp/attrition-public-sweeps-20261016-01/measure.py LABEL
```

The script warms `zsh tools/check.sh test --workspace --release --no-run
--message-format=json`, changes only the diagnostic in
`build_type_corner_tile.rs`, measures `/usr/bin/time -l`, and restores exact
source bytes in `finally`. Both layouts use the same command and diagnostic edit.

| Layout | Wall seconds | CPU seconds | Sampled tree MiB |
|---|---:|---:|---:|
| Prior library |14.37|16.59|1071|
| Extracted |2.47|13.98|404|
| Extracted repeat |2.47|14.00|400|

About5.8× faster rebuilding this test; CPU−16%, sampled memory−62%. This extends
the cheap edit boundary to196 more tests; it is not a whole-suite speedup or a
large CPU gain. A control edit in the previously extracted
`world_data_has_mountain_tcoords.rs` remains2.15s/2.76CPU/229MiB versus the prior
2.21–2.78s/2.76–2.83CPU/212–268MiB: no observed cost transfer on that control.
No cold-build, arbitrary simulation-edit or contended-worktree gain is claimed.

## Correctness and landing

Saved Cargo manifests list exactly the same complete workspace test-name
multiset before and after. Sim remains2586: library1654 and partitions251/270/
190/221. The196 promoted tests retain their names and bodies; there are no extra
Cargo targets. The existing `--lib` limitation remains: use ordinary `cargo test
-p sim` or the release gate for the complete sim suite.

`fault.py` adds1 to the expected corner-tile oracle value. Ordinary cross-target
Cargo selection of its exact name fails with exit101 and the equality assertion,
then passes after restoring the exact source (`fault-red.log`,
`fault-restored.log`). This proves it executes and can fail, rather than passing
through fixture skipping or zero selected tests. Generator tests:109 run with1
unchanged skip (`tool-tests.log`). Formatting passes. All probe sources restored.

Implementation will be committed before the full required-fixture gate; its
verdict is pending. No pin, score or compared-value movement. Independent Fable
review remains owed; no merge or commander restart.

Pre-commit guard passed (`guard.log`, exit0,6.47s admitted execution). Formatting,
diff whitespace, audit JSON and local documentation links pass.

## Frozen committed-tree verdict

Implementation `f4e683b6` was committed before this complete run; no tracked file
changed while it ran:

```sh
/usr/bin/time -l zsh tools/memcap.sh 40 python3 tools/release_gate.py /Users/rf-studio/ron-golden/ch80/map-14 --test-threads 4 --require-fixtures --report-dir /tmp/attrition-public-sweeps-20261016-01/gate
```

Actual exit0 (`gate.exit`), six of six phases passed. Offline498 tests ran with3
unchanged skips. Sim2586 pass across all five targets,0 ignored/filtered;
rondata890 pass,6 unchanged ignored,0 filtered; fixed13 and doctests3 pass.
`compare.py` and `gate-equivalence.json` verify the full workspace test-name
multiset, no duplicate sim names, unchanged ordered rondata manifest/ignored
list, and exact3276 fixture requests (419 unique,0 missing). All comparisons
use the actual newly built executables and recorded fixture requests.

Complete gate255.00s wall/1396.42CPU/8369MiB sampled tree versus the preceding
262.15s/1443.50CPU/8360MiB. Rondata159.20s/1144.01CPU versus164.91s/1170.41CPU.
These single full runs show no regression and essentially unchanged peak memory;
they do not establish a causal whole-suite runtime improvement. The retained
benefit is the measured edit/rebuild boundary. Scores, pins and compared values
remain unchanged; whole-suite10× is still unmet. Independent review is owed.

The verdict follow-up changes bookkeeping only; guard, formatting, links, audit
JSON and diff checks validate it without another full game-data run.

Verdict guard passed (`guard-verdict.log`, exit0). Formatting, local documentation
links, audit JSON and diff whitespace checks also passed.
