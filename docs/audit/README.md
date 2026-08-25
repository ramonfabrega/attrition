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

## How a second reading is run

The rules are `CLAUDE.md`'s ("Every mechanic gets a blind second reading",
"Fan-out rules") and the rationale is `docs/DECISIONS.md` entry 22; this is
the operating checklist, every line of it paid for once in the record below.

1. `grep -n <mechanic> CLAUDE.md`, and read the memory index. A reader
   inherits both. If either names a finding the readers are meant to
   re-derive, move it to `docs/QUEUE.md` or `docs/JOURNAL.md` first — the
   brief cannot undo what the system prompt already delivered.
2. Brief with entry points and the export's structural traps only — facts
   about the export, never claims about the mechanic. Where a run has
   already confirmed a claim, do not spend a reader on it: brief the
   mechanic's never-executed functions (`tools/trace/report.py … blind
   docs/`).
3. Ask each reader to disclose any repository content it was handed, and to
   name, for each claim, the capture that would falsify it — the ini
   category, the frame, the field.
4. Readers write to `~/ghidra-projects/reading/<mechanic>-<date>/` from the
   first finding, section by section. Launch in waves of two or three and
   let each finish; a wall then costs a wave, not a run.
5. Adjudicate in the main thread, or on Opus under the marker discipline:
   append each table row as it is settled; mark anything unsettled
   `FABLE:` rather than guess. A verdict is a function and an expression,
   never a paraphrase.
6. A Fable pass over every marker and every verdict that changed Rust, each
   re-read from its own citation, recorded as "Third pass" in the audit
   file — before the next mechanic builds on it.
7. Every finding that can become an assertion becomes one before the audit
   is closed; the widening of a dumped record is the cheapest and has
   out-produced the reading.

## The record

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

Orders (`2026-08-21-orders.md`, the eleventh mechanic) was run twice, and the
record keeps both attempts.
Seven readers took the blind side, one per sub-area, split as the first
reading was — and, for the first time, on a **different model from the first
reading**: Opus 5 blind against Fable 5's document, the trial the model-split
note had left pending. That half worked: all seven finished, 355 numbered
claims, 64–83 KB apiece. The adjudication did not. Six of the seven
adjudicators, on Fable per the split, hit the account's Fable limit within
about ten minutes of each other and died; five had read both readings and
written no verdict. Only R5, the start of a game, survives — 37 verdicts,
seven corrections, all landed.

The second attempt ran the adjudication on Opus instead, in waves, and it
completed: **A 32 · B 89 · both 165 · neither 18 · open 8** across seven
sub-areas, ten corrections to `crates/sim`, all three queued disagreements
closed without a behavioural check. Its own lesson is in the file, and it is
about hedges: every place the first reading wrote "medium" or "the
decompiler's local is stale", the second found something — twice, the hedge
was pointing straight at the mechanism it had missed.

Three lessons from the failure, all cheap next time:

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

The reports, briefs and adjudications are all at
`~/ghidra-projects/reading/orders-2026-08-21/`. One process change from the
second attempt is worth keeping whatever the model: adjudicators were told to
mark anything they could not settle **`FABLE:`** rather than produce a
verdict they did not believe. Five rows across seven sub-areas carry it —
five honest gaps instead of five plausible errors.

**The pattern's third step, proven on orders (2026-08-23): a ratification
pass.** When an adjudication has run on Opus, the loop is closed by Fable
before the next mechanic builds on it: every `FABLE:` row taken back to the
binary, and every verdict that changed Rust re-verified from its own
citation — the listing re-read where the verdict cites the listing, the
decompile where it cites the decompile. On orders the pass cost about an
hour, retracted nothing, and settled four of the six flagged rows (two of
them from the PDB's own type records, the strongest evidence in the
project). That result is what makes the arrangement safe to repeat: Opus
blind readers, Opus adjudication under the marker-and-append-per-row
discipline, Fable ratification of the markers and the code-changing
verdicts, recorded as a "third pass" section in the audit file.

**The animation clock (`2026-08-24-anim.md`, the same day as the mechanic).**
One Opus blind reader, ~20 minutes, adjudicated by the first reader on
Fable the same hour, while the context that wrote the mechanic was still
loaded — the cheapest adjudication yet, and the argument for running the
second reading before `/clear` rather than in a later session. The
structure was doubly confirmed to the line, and the oracle matched 52/52
both ways. Three verdicts went to the second reading, two of them changing
Rust: a rescale the decompiler prints as a formula and the listing shows to
be an identity (both calls take the old slot), and a flag bit the first
reading had named a boat's crew that the `TypeIndex` enum names a scholar.
The third closed an open question from the dump's own state — a figure's
non-zero `end_time` proves its `set_anim` ran, and `set_anim` there draws —
which the first reading had deferred to a debugger run. One verdict went
against it: a field named from its offset rather than from `types.txt`
(`o_up` for `inside_up`), refuted by the PDB and by the dump in one line
each. The lesson is the cities audit's, again: the readings agree on the
arithmetic and disagree on names, and a name is settled by the type
record, never by the surrounding code.

**The AI (`2026-08-25-ai.md`, the twelfth mechanic, and the largest).** Seven
blind readers on Opus 5, one per sub-area, run in **two waves** — the orders
audit's quota lesson applied deliberately for the first time, and it cost
nothing. Adjudication ran in the **main thread** rather than as a fan-out:
one standard across seven reports, and no repeat of the wall that killed six
of seven adjudicators on orders. 590 numbered claims, 42 rows doubly
confirmed, 31 corrections, two changes to `crates/sim`.

The readers were spent **unevenly on purpose**, which is new. §2.18–§2.20
rested on a single reading (the implementation's) and the trace's coverage
report had all three functions on the never-executed list, so they went in the
first wave; the census, driver and make list had been confirmed behaviourally
by run18 and run19, so they went in the second. That triage is worth repeating
wherever a document's sections have visibly different provenance.

Its headline is a methodological one, and it is uncomfortable: **the blind
protocol leaked, and the project's own habits caused it.** A subagent inherits
the repository's `CLAUDE.md`, and ours narrates each mechanic in the working
agreement — so every reader had this mechanic's headline findings in context
before it read the brief. B1 disclosed it unprompted and precisely; the
affected rows are marked rather than counted. The fix is not to the brief,
which cannot reach the system prompt, but to where the handoff prose lives.
Until that moves, a blind reading of any mechanic `CLAUDE.md` describes is
weaker than it looks.

Three findings show what the pattern is now good for. The one verdict that
went against **both** readings — `compute_site_stats`' coastal ring is centred
on the original cell, not the slid one — was not something the second reader
found; it was something the second reader stated *confidently and wrongly in
passing*, which made it worth checking, and the check found the document and
the implementation both wrong. The one **refutation** went the other way: B7
flagged that `create_units` might feed a type index where a count belongs,
marked it medium, and handed it to whoever owned that function rather than
asserting it — and it was wrong, settled in twenty lines of listing. And
`TypeData::can_pay_cost` reducing with `max` instead of `min` was found by
**two readers independently, in different scopes**, which is the strongest
evidence this method produces.

The recurring lesson recurred three times in one audit: **grep the writers
outside the class you are reading** (B3-b, B5-f, B7-b). In B3-b it had led the
first reading to a confident negative — "nor anywhere" — that was false. The
new lesson is about **transcription**: three of the sharpest findings are
places where the decompiler's C is a faithful-looking lie (an unsigned compare
printed signed, a `% 63` aliasing, a reloaded base register), and in two of
them the implementation was right *because* it had been written from
understanding rather than transcribed. The single place it was wrong is the
place it followed the document's own coordinate bookkeeping.

Still owed, and named here so it is not lost: a guard for the coastal-ring
fix. The existing suite passes either way, so nothing covered it.

*(The pathfinder, commands and recgame audits are still owed a paragraph each
here.)*
