# Wave 4 brief: one section of `docs/LEADERS.md` from one family's verdicts

You write one numbered section of a new specification, `docs/LEADERS.md`,
for item 4152 of the attrition project. You inherit `CLAUDE.md`, which has
the rules. Your spawn line names a **slug**, its **section number N** and
**title**.

**Read first:**
- `docs/LEADERS.md`'s header (the whole file as it stands) — its conventions
  and shared facts are given there and not repeated in your section;
- `docs/audit/2026-11-11-item-4152/<slug>-verdicts.md` — **your one source**.

The readings (`<slug>-reading.md`, `<slug>-second.md`) are background you may
open only to recover a detail a verdict cites but does not restate. Where a
verdict corrected a reading, the verdict wins, always. Do not re-read the
decompile; the adjudication is done.

## What to write

`docs/audit/2026-11-11-item-4152/<slug>-section.md`, containing exactly one
`## N. <title>` section (subsections `###` allowed). Write it in one go, or in
parts as you like; write only that file, do not commit, and do not edit
`docs/LEADERS.md` (the row splices it in).

The section, in this order:

1. **Leaves.** Each leaf, its offset and type, one line each.
2. **The rule, per leaf (or per shared rule)** — what a builder ports:
   writers (`Name (00xxxxxx)`, live sites only, with dead bodies named as
   dead), the event, the predicate (exemptions first), the step in integers
   at the original's scale, the reset, and the readers that change a
   decision. Plain prose and short lists; a formula as a formula.
3. **In the crate.** Where each event already lives in `crates/sim`
   (function and file, as the verdicts confirmed), what is a stub or SEAM
   today, and any crate trap the verdicts name (a flag passed wrongly, a
   mislabelled comment, an early return in the wrong place). Name the
   checksum arm and the `LEADERS_HELD_OUT` count a port moves, if the
   verdicts give it.
4. **Reached by a plain game.** Which writers a skirmish against the
   computer reaches; which sit behind triggers, Conquer the World or
   multiplayer.
5. **How established, and how confident.** "Two readings and an
   adjudication (`docs/audit/2026-11-11-item-4152/<slug>-*.md`); no diff backs
   it yet." Then the verdict counts (confirmed / corrected / open), the
   corrections that changed a rule, and any kept-dump value the verdicts
   cite as a witness (say it is not a pinned diff).
6. **Not established.** Every open verdict, and what would settle it.

## Rules

- Every shell call starts with
  `export PATH=$HOME/.cargo/bin:$HOME/.bun/bin:/opt/homebrew/bin:$PATH`.
  Never a bare `cat`, `read` or `tee` with no operand; read by `grep -n` and
  `sed -n 'A,Bp'`.
- **Cite a function as `Name (00xxxxxx)`, never `Name@00xxxxxx`.**
- **The section must stay under 12,000 characters** (a guard holds every
  `## ` section of a specification under 16,000, and an over-ceiling section
  may only shrink). Check with `wc -c`. Concision over completeness of
  narrative: the verdict file keeps the evidence; the section keeps the rule.
- No story of the readings (who said what); state the rule and its standing.
- No decompiled body transcribed; your own words and integers.
- Hand back one line: the file, its `wc -c`, and anything in the verdicts you
  could not turn into a rule.
