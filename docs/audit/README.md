# Audits: the second reading

Each mechanic document in `docs/` was written by one reader from the
decompiled original. The files here are the record of a **blind second
reading**: an independent reader re-derives the mechanic from the same
decompile export without seeing the document or the implementation, and a
third reader adjudicates every disagreement back to the decompiled function
and records a verdict — A (the document) right, B (the second reading) right,
both wrong, or genuinely ambiguous — with the function and the expression that
settles it.

They are reading notes in the same sense as the documents: short expressions
are quoted where a verdict turns on one, nothing is transcribed, and nothing
here is a source to implement from. What they are for:

- **A list of corrections the document and implementation owe.** Each file ends
  with "must change". Until a correction lands, the document is the one that is
  wrong, and it should say so in its own "second reading" note.
- **A record of what is now doubly confirmed**, so a later reader knows which
  claims two people reached independently.
- **Closed open questions**, where the second reader settled something the
  first had listed as not established.

The first pass (2026-08-20) covered the seven mechanics then implemented. Its
headline: arithmetic doubly confirmed almost everywhere; predicates, scopes and
wiring wrong in several places — which unit kinds are exempt, which step a
multiplier belongs to, which building a rule applies to. These are exactly the
errors that tests written from the same reading cannot catch, which is why the
second reading is now part of a mechanic's definition of done (`CLAUDE.md`,
working agreement).

How it was run: six `lean` subagents on Fable 5, one per mechanic, given the
entry points and the traps but not the documents, writing to a scratch
directory; then one adjudicator per report with both readings and the export.
About an hour of wall clock for all seven, because `tools/ghidra/export.sh`
had already turned the decompile into files.

The tech tree (`2026-08-20-tech.md`, the same day, the eighth mechanic) was
read by two blind readers at once — one over the predicates, one over
`gain_tech` and the state — because the mechanic is twice the size of the
others. Its headline was different in kind: the predicates were doubly
confirmed branch by branch, and what the first reading had missed was in the
**loaders** — a derived prerequisite (`UnitType::init` giving every combat
unit its age's Military epoch) and derived back-links that no amount of
reading the consumers reveals. Read the `init` of every type the mechanic
touches, not only the functions that ask about it.

Combat (`2026-08-20-combat.md`, the ninth mechanic, later the same day) was
read by two blind readers split by function rather than by half — the damage
pipeline and the firing path — because the mechanic is the largest yet and the
two halves share almost nothing but `do_damage`. Its headline: the formula was
doubly confirmed step by step and the **edges of the reading** were where the
first draft was wrong — a magic-number division read as `/96` instead of
`/192`, a `goto` inverted, an enum value of 1 read as a tag, a branch not read
to its end, a block read and not written down. Five corrections, four
additions, all landed the same day.

Cities and buildings (`2026-08-20-cities.md`, the tenth mechanic, later the
same day) was read by **five** readers in parallel for the first reading —
one per sub-area: placement, construction, city levels, garrisons, capture —
and then by five blind readers split the same way, all on Fable 5. Its
headline differs again: the predicates were doubly confirmed almost
everywhere, and what the second reading found was at the **edges of the first
reading's scope** — a caller nobody grepped for (`Leader::calc_wall_stats`
re-bakes every unfinished building's clock, so "frozen at placement" was
wrong), a table taken on trust (`even_circle_init` builds the city radius
mask with a rounded `sqrtf`, not the octagonal metric), a gate read with its
sense inverted (the building's own attrition is every 32 frames, 16 only
under rush rules before war), an early `return` ten lines into a long
function. Seven corrections, all landed the same day. Two disagreements were
settled neither way but in the listing or the PE: the `do_construct`
argument, `capture_strength`, and the type-vtable slot `+0xfc`. The lesson:
grep the writers of every field you call frozen, and the callers of every
function you call once-only.

Orders (`2026-08-21-orders.md`, the eleventh mechanic) is the first audit in
this directory that is **partial, and says so in its own first paragraph**.
Seven readers took the blind side, one per sub-area, split as the first
reading was — and, for the first time, on a **different model from the first
reading**: Opus 5 blind against Fable 5's document, the trial the model-split
note had left pending. That half worked: all seven finished, 355 numbered
claims, 64–83 KB apiece. The adjudication did not. Six of the seven
adjudicators, on Fable per the split, hit the account's Fable limit within
about ten minutes of each other and died; five had read both readings and
written no verdict. Only R5, the start of a game, survives — 37 verdicts,
seven corrections, all landed.

Three lessons, all cheap next time:

- **A fan-out of adjudicators is a quota commitment, not just a token
  cost.** Seven long Fable agents in flight is enough to exhaust a day's
  limit, and when the wall arrives it takes every one of them at once. Land
  them in two or three waves, and let each wave finish before the next
  starts, so a wall costs one wave and not a run.
- **A failed agent's notification carries no usage figures**, so the waste
  can only be bounded by wall clock. The instruction to readers to write
  their file *section by section* is what saved the blind side; the
  adjudicators had the same instruction and five of them still died before
  the first verdict, because a verdict costs a lot of reading before it
  costs a line of output. Tell an adjudicator to append **each table row as
  it is settled**, not each section.
- **Keep the raw reports somewhere durable from the start.** They were in a
  job scratch directory that is deleted with the job; 2.2 M subagent tokens
  of reading survived only because they were copied out. They live at
  `~/ghidra-projects/reading/<mechanic>-<date>/` now, which is where the
  next mechanic's should be written directly.

What the six missing adjudications owe is listed at the end of
`2026-08-21-orders.md`, so the disagreements are not lost with the agents
that were reading them; the reports and briefs to resume from are at
`~/ghidra-projects/reading/orders-2026-08-21/`.
