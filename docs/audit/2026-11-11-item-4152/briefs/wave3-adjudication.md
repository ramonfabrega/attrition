# Wave 3 brief: adjudicating one leaf family of the leaders record

You are the adjudicator for item 4152 of the attrition project, a Rust
reimplementation of Rise of Nations. You inherit `CLAUDE.md`, which has the
rules. Your spawn line names a **slug**, or two. For each slug, two readers
wrote up the same leaves of `LeaderData` independently:

- `docs/audit/2026-11-11-item-4152/<slug>-reading.md` is the first reading;
- `<slug>-second.md` is the blind second, written without seeing the first.

That directory's `README.md`, table "The families", gives the family's
leaves and the writers an earlier listing-only pass named. Treat those
writers as a claim too, not as ground truth.

The worktree is `/Users/rf-studio/code/fun/attrition/.claude/worktrees/att-4152`.

## What to produce

Write `docs/audit/2026-11-11-item-4152/<slug>-verdicts.md`, one per slug.
**Append per leaf as you settle it, never all at the end.** The account's
wall can kill you mid-run, and what is on disk is what survives. Put a
header in the file in your first minutes. Write only your verdict file(s),
don't commit, and edit nothing else.

For each leaf:

1. **Line the two readings up claim by claim.** The claims are: each
   writer, its predicate, its step, its reset, its readers, its crate site,
   and whether a plain game reaches it. Mark each claim *agree*, *disagree*
   or *one-sided* (only one reader made it).
2. **Settle every disagreement and every one-sided claim** against the
   decompiled function (`~/ghidra-projects/decomp/funcs/...`, read by
   `grep -n` and line range) or the listing
   (`/opt/homebrew/opt/llvm/bin/llvm-objdump -d --no-show-raw-insn --start-address=0x... --stop-address=0x... /Users/rf-studio/code/fun/attrition/game/riseofnations.exe`).
   Give each a verdict: **confirmed**, **corrected** (state the right rule)
   or **open** (state what would settle it), with **the deciding line**.
   That is the file and line number, or the instruction address, that
   decides it.
3. **Check the predicates first, even where the two agree.** Earlier
   readings have gone wrong on which kinds or slots are exempt, which step a
   multiplier belongs to, which array an index reads, and signed against
   unsigned. Two readers can agree on a wrong predicate. Spot-check each
   agreed predicate against the body.
4. **Check the writer list for completeness.** Grep both nets yourself: the
   export's `->name` and `field_0x<off>`, then the listing's displacement
   off the leaders array `0xe3a390 + i·0x6eec` or off a `Leader*`. A writer
   neither reader found is a finding.
5. **Check the crate site.** Confirm that the Rust function each reader
   named exists (`grep -n` in `crates/sim/src`) and is the event's home.
6. **End the leaf with the rule as a builder should port it**, in integers
   at the original's scales: writers, predicate, step, reset, crate site,
   and whether a plain game reaches it, followed by the confidence and what
   stays open.

Close each file with a **verdict table**: leaf | claims confirmed /
corrected / open | the rule in one line.

## Rules

- Start every shell call with
  `export PATH=$HOME/.cargo/bin:$HOME/.bun/bin:/opt/homebrew/bin:$PATH`.
- Never run a bare `cat`, `read` or `tee` with no operand. Read source by
  `grep -n` and `sed -n 'A,Bp' file`.
- **Cite a function as `Name (00xxxxxx)`, never `Name@00xxxxxx`.** A guard
  that walks `docs/**` checks the `@` form.
- `types.txt` names a `field_0xNN` and `vtables.txt` names a vtable slot.
  The decompiler's traps are in `tools/ghidra/README.md` "Traps"
  (`sed -n '85,206p'`).
- Read decompiled bodies, but never transcribe one. Write the rule in your
  own words. No Ghidra headless and no run of the original.
- When you hand back, give one paragraph per slug with its counts
  (confirmed / corrected / open), the most important correction, and your
  sense of the tokens and time it took.
