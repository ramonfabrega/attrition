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
