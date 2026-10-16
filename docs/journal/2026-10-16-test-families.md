# Separate sweep test families — 2026-10-16

User-authorized sequential exploration, starting `a9d6b67d` on
`codex/astra-windowed-reader`. Fable stays paused; no subagents, new worktrees,
fixture relocations or captures. Evidence root:
`/tmp/attrition-test-families-20261016-01/`.

## Boundary chosen

The rondata replay families share private harness APIs and a large testkit;
moving their files alone would not create compilation boundaries. The sim sweep
family has a narrower first boundary:737 of its1046 modules have no `crate::`
references and use only `emu_rows`, `row` and `tail_row`. They contain736 tests.
Conservative textual classification also retains modules whose comments mention
`crate::`; no simulation APIs are made public to force a split.

First extracted those737 registrations into one integration target. That compiled
without API changes and reduced the measured sweep edit from21.95→9.69s. Then
partitioned that independent family alphabetically into a–f,g–o,p–t,u–z. Four
Cargo integration entry points mount modules named `sweep`, preserving every
existing `sweep::<module>::<test>` name. Test bodies remain at their original
`crates/sim/src/sweep/*.rs` paths, preserving census and generator source discovery.

`mod.rs` retains private-API tests and its row-parser assertion. Shared oracle
functions moved unchanged into `sweep/support.rs`; the install-path helper moved
unchanged into `test_support/install.rs`, mounted by both lib and integration
targets. All targets keep the same sim package manifest directory, install
fallbacks, subprocess command, outputs, minimum-row and whole-window assertions.
A narrow dead-code allowance covers the shared tail-row helper in partitions
that do not call it. No check or sim behavior changes.

## Discovery and registration safety

Before/after real libtest discovery gives the same2586 names exactly once:
1850 library +207 a–f +224 g–o +159 p–t +146 u–z. `partition-manifest-equivalence.json`
records the equality and0 duplicates. Baseline and candidate executables and
lists are retained. `sweep-body-hashes.json` verifies all1046 original module
bodies byte-equivalent after decoding to Git HEAD; all temporary probes restored.
The one historically present filename that is not a valid Rust module identifier
remains untouched and outside the original1046 registered modules.

The generator now routes new independent tests to these stable partitions;
crate references keep new tests in the library. Existing registration ownership
is preserved on regeneration. If a standalone port gains a crate reference,
the guard fails with instructions to move its registration back into `mod.rs`.
The normal guard and offline gate check that each validly named sweep source
has exactly one registration, each partition is mounted, alphabetical ownership
is correct, and Cargo auto-discovery is not disabled or overridden. Unknown or
duplicate standalone registry content fails before the generator writes its two outputs;
no unrecognized lines are discarded. Generator shape/port rendering is unchanged.

Tests reject missing/duplicate/wrong-partition registrations and disabled Cargo
targets; authored generator creation/regeneration preserves ownership. These
checks failed before implementation (`registration-red.log`) and pass after.
The four known integration targets retain the original sim target's requested
thread width/4GiB admission; unknown targets retain their conservative32GiB policy.
All Cargo-discovered targets still execute through the existing release runner.

`cargo test -p sim` continues to include every sim test. `--lib` or running only
the sim library executable now excludes the independent sweeps and is **not** a
complete sim-suite claim. The gate/guard use Cargo discovery and no `--lib`.
For a selected partition use e.g. `zsh tools/check.sh test -p sim --release
--test sweep_u_z FILTER`; a complete landing still owes the normal full gate.

## Measured compilation

Every build uses `/usr/bin/time -l zsh tools/check.sh test --workspace --release
--no-run --message-format=json`, unchanged incremental release policy and shared
admission, with sequential runs. `baseline.py`, `extract.py`, `split-measure.py`,
`partition-measure.py`, `final-build-probes.py` and `sim-edit.py` retain reproduction
steps, backups and actual exits. The edit changes only an assertion diagnostic
in `world_data_has_mountain_tcoords.rs`; it is restored after each run.

| Layout | Edit wall s | Build CPU s | Sampled tree MiB |
| --- | ---: | ---: | ---: |
| Original sim test library |21.95|103.49|1971|
| One standalone family |9.69|42.08|728|
| Four partitions, first |2.21|2.76|212|
| Four partitions, repeat |2.78|2.83|268|

This is about8–10× wall speedup and97% less CPU **for this sweep edit**, not a
10× whole suite. An authored new test plus its registration in the existing
u–z partition builds in0.60s/1.06CPU; its sampled peak19MiB under-samples such a
short build, so no new-test memory reduction is claimed. That probe is removed
and the exact registration restored. Artifacts identify only the affected
partition as rebuilt for each sweep probe; unchanged targets are reused.

The same small behavior-preserving sim-function edit from the previous journal
builds in17.87s/76.52CPU/1945MiB, compared with the prior18.61–19.25s/108.49–108.80CPU/
2344–2586MiB. This is one current sample, not an arbitrary-simulation-edit promise.
The split does not show a rebuild-cost transfer onto sim edits in that sample.
Cold compilation and source edits in private-API sweep modules are not claimed
to gain8–10×. The independent binaries still depend on the sim package's normal
Cargo graph; no cross-worktree compiler or test verdict cache is introduced.

## Failure proof and validation

`fault.py` changes the extracted world test's expected oracle answer by+1,
runs its exact name through ordinary `cargo test -p sim`, and requires exit101
and the failed equality. It then restores exact source bytes and requires the
same command to pass. `fault-red.log` and `fault-restored.log` retain both actual
verdicts; a missing install or zero selected tests could not satisfy this proof.

Focused tooling121 tests pass,1 unchanged skip (`tool-tests.log`), and the full
offline suite497 passes,3 unchanged skips (`offline.log`). Subsequent Cargo-target
registration refusal cases pass (`registration-final2.log`). Formatting and
registration validation pass after formatting. Complete guard and full committed
required-fixture gate are pending; implementation will be committed before that
gate, with the actual verdict recorded separately. No score/value/pin movement.
Independent Fable review remains owed.

## Next

This establishes a narrow compilation boundary without making private APIs
public. After review, consider other genuinely independent families; avoid
mechanically exposing the replay harness just to move test files. The whole
suite still performs its full simulation/fixture work. Adoption and measured
contended behavior remain separate from these single-build measurements.

Final pre-commit tooling run:122 tests pass,1 unchanged skip (`tool-tests-final.log`).
The additional authored case proves malformed registry content returns exit2
before either generated output is created. The complete guard passed before
that additional case (`guard.log`); final guard receipt is `guard-final.log`, exit0 (4.23s admitted execution).
Formatting/diff checks pass;16 local documentation links resolve and audit JSON
parses. The full committed-tree gate remains pending.
