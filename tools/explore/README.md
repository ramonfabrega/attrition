# Methodology exploration probes

These are experiments, not changes to the simulation or its acceptance floors.
Findings and limitations are in
`docs/audit/2026-09-09-methodology-exploration.md`.

- `turn_oracle.py INSTALL` executes a small object-graph fixture in Unicorn.
  Its stdout is derived original-game output: redirect it outside the repo.
  `cargo run -p rondata --release --example turn_oracle_check` consumes the
  table on stdin and compares the existing Rust implementation.
- `trace_health.py LOG_DIRECTORY` scans archived trace framing and recorded
  errors. It reports metadata only. A clean report is not proof of completeness.
- `cargo run -p rondata --release --example trace_acceptance` demonstrates
  permissive parsing of synthetic headers, truncation, and loss records.
- `profile_gate.py` temporarily instruments three reader operations, runs the
  rondata library suite, and restores the source. Run from the repository root
  with `RON_INSTALL` set, working process-sampling permission, and no concurrent
  work in this worktree. It writes `/tmp/attrition-profile-gate.log`.
  Its `EXPLORE phase nanoseconds ...` lines measure wall spans, not CPU time.

No probe launches the game, modifies the install, or writes to existing
captures or the Ghidra project. Oracle tables and full gate output stay local.

- `command_oracle.py INSTALL` executes the original MoveTo issuer and replays
  its touched data pages. Pipe stdout (outside the repo) to
  `cargo run -p rondata --release --example command_oracle_check` to compare
  all packet fields with the existing decoder. It uses synthesized state and
  a bounded imported-memcpy adapter; it does not test live scheduling.
  See `docs/audit/2026-09-09-command-loop-experiment.md` for measured capacity
  hazards and the live permission blocker.

## Live oracle experiments

`docs/audit/2026-09-09-live-oracle-unlocks.md` records the measured live results
and the remaining boundary to true headless execution.

- `live_session.py stage INSTALL NEW_OUTPUT PROFILE_DIRECTORY [--hide-scene]`
  prepares a new isolated output directory, backs up shared settings, and
  stages a bounded run. `restore NEW_OUTPUT` restores backed-up settings after
  the game closes. It rejects an existing output directory.
- `live_move_probe.h` is enabled by `TRACER_DEFS=-DRON_COMMAND_PROBE`: one real
  move at frame 20, with emission and position receipts.
- `live_turn_probe.h` is enabled by `-DRON_TURN_PROBE`: captures normal-member
  turn-speed fields and adds original call/return and render-call witnesses.
- `-DRON_HIDE_SCENE` requires the turn probe and suppresses presentation over
  frames 18–34. Experimental; does not remove graphics startup or service work.
- `compare_live_runs.py CONTROL_DIRECTORY HIDDEN_DIRECTORY` fails unless the
  bounded command, render-suppression, full logged-frame and RNG checks pass.
- `replay_live_turn.py INSTALL TRACE [--mutations N]` replays actual captured
  inputs and emits deterministic mutation rows for `turn_oracle_check`.
  Redirect its output outside the repository.

## Capture memory experiments

`docs/audit/2026-09-09-streaming-captures.md` records separate-process RSS and
complete-record equivalence checks. Build `frame_memory` with Cargo in release
mode, then run `whole`, `indexed`, `indexed-warm`, or `verify` against a finalized
capture. The warm mode excludes initial index construction from its reported
scan time. `verify` intentionally loads the full baseline and is not a memory
benchmark. `python3 tools/explore/test_memcap.py` exercises sampling failures.

## Native call graphs

`docs/audit/2026-09-09-native-profile.md` records the census profiles and the
selective frame lookup they motivated. Build a release test binary with
`CARGO_PROFILE_RELEASE_DEBUG=1`, then run `profile_native.py BINARY EXACT_TEST
NEW_OUTPUT` under `tools/memcap.sh`, with `RON_INSTALL` set. `sample_graph.py`
converts its sample file into a graph and hotspot JSON; `--help` lists the
external FlameGraph checkout and LLVM demangler arguments. Graphs select the
test thread and retain its waiting states, so their denominator is explicit.

`shutdown_memory` similarly compares whole-file and suffix-only closing-state
reads. Its `verify` mode accepts one capture or a directory of finalized
`gamelog-*.txt` archives. See `docs/audit/2026-09-09-shutdown-streaming.md` for
complete-record coverage, the census migration, and limits of the memory claim.


## Strict live-derived call capsules

`docs/audit/2026-09-09-live-call-capsule.md` records the bounded clear-call
experiment, evidence and limitations. `RON_CAPSULE_PROBE` requires
`RON_COMMAND_PROBE`; it invokes the original clear function on a complete copy
of a live command package, preserving the gameplay packet. It is not a natural
call interception or an automatic arbitrary-state capture.

- `uv run replay_capsule.py bind INSTALL OUTPUT` pins staged executable/DLL
  identities before the run; `replay INSTALL OUTPUT` verifies the captured
  call twice in fresh emulators and rejects missing dependencies.
- `python3 compare_capsule_runs.py CONTROL OUTPUT` checks full logged frames,
  frame/RNG records and command/copy receipts against a normal command run.
- `uv run test_capsule.py` exercises strict access, output and execution guards
  with authored fixtures. It needs no game install.

Use full paths under `tools/explore/` when running from the workspace root.
Capture binaries and reports must remain outside the repository. Restore the
backed-up shared settings with `live_session.py restore` after the game closes.

## Sparse pointer-graph calls

`turn_angles_oracle.py INSTALL [--cases 4096]` executes the original angle
mutator and nested turn-speed function on authored normal-member fixtures.
Redirect its rows outside the repo, then pipe them into the release Cargo
example `turn_angles_oracle_check`. It checks fresh versus reset execution,
reverse input order, missing dependencies, pointer writes and stack cleanup.
This is synthetic-state differential testing, not natural live capture.

`bounded_call.py` provides explicit byte regions, initialized-on-write scratch,
immutable code and CPU/data reset. `uv run tools/explore/test_bounded_call.py`
checks it with authored machine code. Bounds, measurements and next live-witness
requirements are in `docs/audit/2026-09-09-bounded-turn-angles.md`.

## Natural order-cache capsule

`RON_ORDER_CAPSULE` intercepts a natural `Unit::update_order` call in frames
0–35, with coverage disabled. It excludes the copy-capsule and render-suppression
experiments. Bind the staged images with `replay_capsule.py bind`, then use
`replay_order_capsule.py INSTALL OUTPUT` and
`compare_order_capsule_runs.py CONTROL OUTPUT`. The first successful witness was
an equal-value refresh: see `docs/audit/2026-09-09-natural-order-capsule.md` for
its scope and the distinction between live entry/exit bytes and replay stores.

`uv run tools/explore/test_order_capsule.py` checks capsule framing;
`python3 tools/explore/test_lobby_focus.py` verifies missing windows and
Automation failures cannot reach lobby clicks, using mocked commands only.

Add `--mutations 4096` to the order replay to poison the three stale cache fields
with deterministic bit patterns. Every result must reconstruct the full live
exit state and match a fresh emulator. These are live-derived synthetic inputs,
not extra natural captures or a Rust order-list differential test.

## Natural path deletion with a Rust comparison

`RON_PATH_CAPSULE` captures a nonempty `Unit::kill_current_path` call with no
suspended search. Bind staged images before launch as for other capsules.
`replay_path_capsule.py INSTALL OUTPUT --mutations 4096 --oracle-rows` emits
semantic rows for the release Cargo example `path_capsule_check`; redirect them
outside the repo and require the producer to exit zero before running the checker.
Omit `--oracle-rows` for the replay/mutation report. The first live sample was
shutdown path deletion, 2 → 0; synthetic lengths range from zero to two.

`compare_natural_capsule_runs.py CONTROL OUTPUT --kind path` checks the standard
bounded run; `--kind order` supports the previous natural capsule too. Eight
framing tests are in `test_path_capsule.py`. The explicit live-scratch exclusion,
code-region bounds and remaining gaps are in
`docs/audit/2026-09-09-natural-path-capsule.md`.

## Semantic counterexample reduction

Build the release example `path_capsule_worker`, then run
`minimize_path_capsule.py INSTALL CAPTURE --worker WORKER --output /tmp/NEW.json`.
It searches deterministic cases, preserving mismatch class while deleting
waypoints and clearing field bits. The final certificate rechecks every smaller
neighbor with fresh original engines and Rust processes. A correct worker returns
`no_mismatch`; infrastructure failures abort. Reports must stay outside the repo.

`python3 tools/explore/test_path_counterexample.py` needs no game or emulator.
The promoted `reduced_path_counterexample` Rust test also runs without original
files. The deliberately injected fault, local minimality limits and measurements
are recorded in `docs/audit/2026-09-09-path-counterexample-reduction.md`.

## Suspended-search recycler boundary

`search_cleanup_oracle.py INSTALL` checks 6,144 authored cleanup graphs against
native complete-record and ownership assertions, reverse-order reset runs and
fresh engines. It maps recycler state explicitly and rejects growth; it does
not map payloads or replace allocator calls. Redirect output outside the repo.
Scope, negative controls and the remaining natural-capture requirement are in
`docs/audit/2026-09-09-search-cleanup.md`.

## Live suspended-search census

`RON_SEARCH_CENSUS` instruments only the A* proxy. On a suspension return it
reads bounded container/pool metadata, with explicit read-failure and cap
receipts. It requires `cover=0` and a `callwin`. Stage a longer run with
`live_session.py stage INSTALL NEW_OUTPUT PROFILE --end-frame 8000 --fast-forward`,
then build with `TRACER_DEFS='-DRON_COMMAND_PROBE -DRON_SEARCH_CENSUS'`, bind its
images and launch normally. The original's fast-forward command runs at frame
37, after the detailed log window; the configured endpoint quits the match.
Close the application and restore the staged settings afterward.

`search_census.py TRACE` rejects incomplete metadata and reports a zero-event
run explicitly. `--archive` scans legacy proxy traces without requiring the
experimental receipts. Reports belong outside the repo. Run
`python3 tools/explore/test_search_census.py` for the protocol tests; compile
`test_live_search_census.c` with host Clang and
`-Wno-int-to-void-pointer-cast` for the callback's mocked-read tests. Coverage
and the successful 8,000-frame census are recorded in
`docs/audit/2026-09-09-search-census.md`.

## Targeted congestion scenario

Stage `live_session.py stage INSTALL OUTPUT PROFILE --end-frame 1400 --fast-forward`,
then write `python3 tools/explore/congestion_probe.py schedule > OUTPUT/rontrace.cmd`.
Build into OUTPUT with `TRACER_DEFS='-DRON_CONGESTION_PROBE -DRON_SEARCH_CENSUS -Werror'`;
bind, launch and restore as above. This macro excludes `RON_COMMAND_PROBE`.
It orders two groups of newly spawned captains through the original issuer;
it does not modify search limits. `congestion_probe.py report TRACE` validates
spawn/order receipts and census metadata. `compare TRACE REPEAT` also requires
matching frame/RNG and search projections, without claiming full-state parity.
Redirect reports outside git. Five Python tests and `test_live_congestion_probe.c`
(host Clang with `-Wno-int-to-pointer-cast -Wno-pointer-to-int-cast`) cover refusal
paths. Two live runs reach 1,787 natural suspensions each; measured graph bounds,
limitations and the next capsule boundary are in
`docs/audit/2026-09-09-congestion-probe.md`.
