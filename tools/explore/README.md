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
