# attrition — direct Codex work

## Scope and precedence

This is the working agreement for direct work in this Codex checkout, authorized
by the user on 2026-10-01. Read `docs/QUEUE.md` first, then `CLAUDE.md` for the
project architecture, evidence standards, conventions, and tools. Apply those
engineering rules with the explicit overrides below. `CLAUDE.md` remains the
Claude commander's operating agreement; this file is deliberately a regular
file rather than a symlink to it.

This session is one agent working directly with the user on one agreed queue
item. No subagents, parallel agent lanes, commander loop, automatic refill,
model trial, or scheduled continuation. The paused Claude loop stays paused.
The user may extend the scope later; a nonempty queue alone does not extend it.

## Autonomy and completion

Carry the agreed item end to end: establish the baseline, investigate, falsify
hypotheses, implement what the evidence supports, add differential assertions,
update the specification and bookkeeping, validate, and commit. Do not stop at
an investigation report, proposed fix, or passing focused test when more of the
agreed item remains. Do not ask whether to continue between these steps.

Routine implementation choices, reversible local experiments, captures needed
for the item, and fixes to local tooling are within scope. Use the available
computer resources, respecting the repository's memory limits and capture lock.
Record assumptions and limitations; uncertainty within a mechanic is a reason
to measure, not automatically a reason to ask. The user permits adaptations to
this file as experience warrants; explain substantive changes in the handoff.

Ask when a necessary decision changes project direction or scope, when an
external or irreversible action needs authorization, or when a genuine blocker
requires the user. Use the environment's permission mechanism for operations
outside the sandbox. Distinguish a sandbox denial, a transient tool failure,
and a macOS consent issue; use an established authorized remedy when available.
Report unresolved blockers promptly with the action the user must take. Never
silently replace missing evidence with an assumption or weaken a check to pass.

An asynchronous process does not end the work. Use the tool's process/session
handle and bounded waits, inspect its actual exit status, and continue after
completion. Use `tools/gamelog/waitrun.sh` for detached captures as documented.
Claude's Monitor, idle subscription, automatic re-invocation, `/clear`, and
"the turn ends" instructions do not apply here. Avoid busy polling; give useful
progress updates during long work. Context compaction is not an item boundary:
preserve the state needed to resume and continue the same item.

Finish at the agreed item's boundary, or report a genuine blocker or explicit
user pause. A final report states what changed, the measured score and value
diffs (including no movement), validation, commit, and unresolved questions.
Do not claim a gate passed, a commit landed, or a push happened before verifying
that action. Completing an item does not start the next one.

## Evidence and review

The project's hard constraints are unchanged: no floating point in the sim,
no engine dependency in the sim, no original assets or derived proprietary
exports in git, no transcription of decompiler bodies, and evidence for format
claims. Preserve the diff-first method, whole-record and whole-cast comparisons,
widening and coverage requirements, hypothesis killers, seam checks, and
measured floors. A draw-stream improvement needs the corresponding value diff.
Keep existing floors; report an unmet target honestly rather than moving it
backward or declaring an unmeasured result.

No model assignments or model-admission matrix from the Claude harness govern
this user-authorized run. Attribute work only to the model identity actually
available in the session, never to a Claude role or an inferred model variant.

No-subagents also means no pretend independent audit. A claim established only
by reading remains explicitly provisional until the required blind review is
performed later; mark its evidence, uncertainty, and review debt in the relevant
specification and item journal. Prefer executable differential evidence where
possible. A later Fable steer can review the commits, outcomes, methodological
changes, and open review debt. Do not self-ratify that debt or launch a steer.

## Local data and shared tooling

Keep using the established install, Wine prefix, ignored directories, captures,
packets, decompile exports, and local tool dependencies at their documented
paths. Read the relevant runbook and resolve symlinks before writing. A git
worktree isolates tracked edits, not shared data outside it. Do not relocate or
duplicate the data corpus, or rewrite tools merely to fit the sandbox; use the
environment's permission mechanism for necessary writes outside this checkout.
Routine local staging required by the documented capture flow is within scope.

Ignored, untracked, and outside-repository files are not disposable. Preserve
pre-existing files and evidence: no blanket cleanup, `git clean`, pruning,
deletion, or destructive overwrite without explicit user authorization. Check
that a new run/output path is unused and keep experiment outputs distinct.
Before a tool replaces a shared configuration, executable, or rolling log,
inspect its write behavior and preserve the existing contents using the
runbook's backup/archive flow (or a distinct backup if none exists). Restore
temporary configuration changes after the experiment and record any intentional
shared-state change. Never remove capture locks by hand.

Scratch files created by this session may be cleaned up only when known to be
disposable and unused; retain captures, packets, logs, and other evidence cited
by a result. Record durable evidence paths and reproduction commands in the
item journal/run ledger so a later steer can find them. Keep proprietary data
out of git even when it is essential to the work.

## Bookkeeping and validation

This single session owns both implementation and the bookkeeping normally split
between worker and commander. Update pins and queue scoreboard together. Write
the item's own journal and specification; maintain queue, parked items, and run
ledger when the work requires it, following their existing guards and numbering
rules. Preserve the paused-loop handoff and clearly distinguish this branch's
work from anything merged into the commander's branch. Do not rewrite historical
journals or claim the commander resumed. Keep findings out of instruction files.

Use ordinary git on the current branch. The `ccc` merge/book/spawn/push/reap
chain, worker brief dispatch, lane-only file ownership, clear cadence, and
mandatory push do not apply. Commit locally; pushing, merging into main, or
resuming another checkout is a separate user-directed action.

For an implementation landing, commit the work before the full release gate,
then record its actual verdict in a follow-up commit. Run
`python3 tools/release_gate.py <install> --test-threads 4` without `--lane`,
logging to a file without a pipe that hides the exit status. Do not edit the
validated tree while the gate runs. Use focused checks and `tools/guard.sh`
during development; do not repeatedly run the full gate without a new reason.
An instruction-only adaptation needs diff/link checks and relevant paperwork
checks, not a game-data release run; state that limited validation explicitly.
