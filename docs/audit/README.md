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
"Fan-out rules") and the rationale is `docs/DECISIONS.md` entries 22 and
23; this is the operating checklist, every line of it paid for once in the
record below.

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
   category, the frame, the field. **Hand the readers the captures** —
   the trace logs with `tools/trace/report.py`, and the dumps under
   `Logs/` — as evidence, with the frames the mechanic's functions were
   first entered on. They are the original's output, not the document,
   so a blind reader may read them; a reader who can run its own
   falsifier catches the map fact that two readings of the code agree on
   and both get wrong (transports, B.9: a dock registers in the *sea*).
4. Readers write to `~/ghidra-projects/reading/<mechanic>-<date>/` from the
   first finding, section by section. Launch in waves of two or three and
   let each finish; a wall then costs a wave, not a run.
5. Adjudicate in the main thread, or on Opus under the marker discipline:
   append each table row as it is settled; mark anything unsettled
   `FABLE:` rather than guess. A verdict is a function and an expression,
   never a paraphrase.
6. **If the mechanic's product is arithmetic, implement it now** — before
   the ratification, or in parallel with step 4. A formula cannot be
   checked by reading it again: every citation in a row can be right and
   the result still wrong, and the pass that catches it is the one that
   writes the code and runs the diff. Predicates and call graphs do not
   need this; formulas do. `docs/DECISIONS.md` entry 23, and the group
   orders' "Fourth pass" is where it was paid for.
7. Markers and code-changing verdicts go on the **ratification ledger**;
   the next mechanic does not wait on a pass. Fable ratifies in batches
   over what is marked, in the main thread, each row re-read from its own
   citation, recorded as a numbered pass in the audit file. The batch size
   and cadence are not fixed.
8. Every finding that can become an assertion becomes one before the audit
   is closed; the widening of a dumped record is the cheapest and has
   out-produced the reading. **Grep the dump before booking a reading** —
   more than one open question has been answered by a field the original
   was already printing.

## The ratification ledger

What is owed a Fable pass, as of 2026-08-26. Ratification runs in batches
over this list rather than gating each mechanic on a pass of its own
(`CLAUDE.md`, "Fan-out rules"; `docs/DECISIONS.md` entry 22 as amended),
so this section is the thing that keeps a batch from losing a row. Update
it when a pass lands, not when one is planned.

**Ratified.** `2026-08-21-orders.md`, `2026-08-25-ai.md` and
`2026-08-25-groups.md` each carry a Fable pass recorded in the file. Every
`FABLE:` marker in the groups file is struck through with its answer.

**Two markers are still open, deliberately**, both in the orders audit and
both kept as the pointer to a check rather than as an unsettled verdict:
R2 O1, `do_move`'s attack-retarget block — nothing in `crates/sim` depends
on it, and the `objdump` recipe is in the R2 adjudication — and R4's
`find_gather_tcoords@0063bdc0`, which matters only once the harness builds
a camp itself. Neither blocks a batch; both go into one when the thing
that needs them is being built.

**Owed, oldest first.** Adjudicated on Opus and never ratified:

- the nine of 2026-08-20 — attrition, cities, combat, costs, economy,
  movement, production, supply, tech;
- `2026-08-23-pathfinder.md`, `2026-08-24-anim.md`,
  `2026-08-24-commands.md`, `2026-08-24-recgame.md`;
- `2026-08-25-transport.md`, `2026-08-25-army.md`;
- **`2026-08-25-groups.md`'s "Fourth pass"** — the row that matters most,
  because it *overturns* three verdicts an earlier Fable pass confirmed.
  A ratifier should start there and should be told that a previous
  ratification agreed with the rows now being retracted.
- **`docs/ARMY.md` §11's two corrections and §18's new `FABLE:` marker**
  (2026-08-26, Opus, from the listing at `6f5160`): that `is_engaged` and
  `engagement` test `get_action()` rather than the front order — which a
  run29 diff then confirmed, so it needs no ratification — and that
  `is_map_unit` gates only the loop's **break**, so an army with no
  map-unit target adopts the **last** qualifying unit's target. The
  second is the marked one: it rests on a register spill
  (`6f5324`/`6f5342`) and a fall-through (`6f536b`) and **no run has
  exercised it**. It is implemented in `Sim::army_engagement_seed`, so a
  ratifier is checking live code.
- **`docs/GROUPS.md` §4.4's `find_leader` key** (2026-08-26, Opus,
  listing `0070ccb0`). Implemented and unit-tested; unobserved, because
  every group in every dump so far has one `type_cat` category. The
  capture is named in §13.

**The cheapest way to shorten this list is not a pass.** Most of what is
owed is arithmetic and predicates a capture can settle outright, so
`docs/QUEUE.md` item 13's differential fuzzing retires more of it per hour
than a reading does — and per entry 23, anything on it whose product is a
formula wants its implementation before its ratification anyway.

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

**The pathfinder (`2026-08-23-pathfinder.md`, the same day as the
mechanic).** Two blind readers on Opus 5, split by function — `astar_path`
and its machinery, `calc_cost` and the three wrappers — with the PE listing
allowed for garbled locals; an Opus adjudicator under the marker discipline,
32 verdicts; then the main thread, on Fable, ratified every verdict and
re-read the five behavioural ones from the decompile itself. The structure
was doubly confirmed — the LIFO tie-break, the expansion wheel, the budgets,
the arrival radius, the reconstruction roots — and the corrections were at
the edges again: which branch each cost term is reachable from, a flag both
readings had left uninterpreted (`leaders.flags & 4`), a give-up exit nobody
had transcribed, and the 24 corner-cutting probe offsets, which the
decompiler had mangled and the first reading had refused to trust, settled
byte by byte in the listing. Five behavioural corrections landed in
`path.rs` the same day, and one of them found a bug outside the mechanic:
the gamelog parser had been handing every leader its successor's flags. The
new lesson: **check the project's own established facts before leaving a
flag uninterpreted** — `is_human` had been in `docs/ORDERS.md` §8 the whole
time, and both readings re-derived around it. The working papers are at
`~/ghidra-projects/reports/pathfinder/`, one directory over from where the
later mechanics' live.

**The recorded-game container (`2026-08-24-recgame.md`).** The first audit
of a file format rather than a mechanic, so the blind reader had a second
oracle — the heavengames sample as well as the export. One Opus 5 reader,
adjudicated by the first reader on Fable the same day. The 943-byte header
map came back identical field for field, and the strongest agreement in
the record so far is the package-stream start: the first reading's landmark
and the second's backward dynamic program over all 1.68 MB — two algorithms
with no shared code — arrived at the same byte. The corrections were in the
**writers**, once more: the first reading had the recording path writing
gzip, and the mode bits in `File::open` show it writes raw and gzips only in
`finalize`, so an unfinalized recording is a raw file and the reader now
sniffs for both. Two more: a marker field settled from its use sites, and a
Types stretch the second reader walked to exactly 806 records and then
proved unparseable without the loader's own class order. The blind report
was not kept — it lived in the job's tmp, against the orders audit's third
lesson — and survives only as absorbed into the audit file and the
document.

**The command payload encoding (`2026-08-24-commands.md`, the same day).**
One Opus 5 blind reader in an isolated worktree branched from `main`, which
carried neither the document nor the queue entry — the isolation structural
rather than an instruction — 886 lines, every claim cited and graded;
adjudicated by the first reader on Fable. The 82-entry dispatch table and
every size were derived four times over, each reading from the decode side
and the encode side, and all four coincide entry for entry; the obfuscation
layer — the seed-keyed XOR, the seeded gap bytes, the gate on the network
bit — was proven from the call ordering by each reader independently.
Three of the four corrections were of one kind, **what can actually be
emitted**: the console command's wire size exceeds the package cap, so the
dispatcher handles a command nothing can send; five more commands have no
issuer anywhere in the export; and the walker the first reading had cited
to prove `group` is never serialised was the recording head, while the
save-game walker does write it. Each was settled by a sweep of every
`add_command` caller — the cities lesson with a different verb: grep the
callers of every function you call once-only, and the **emitters of every
command you call sendable**. Its report was also lost with the job's tmp;
from the AI audit on, readers write to `~/ghidra-projects/reading/` from
the first finding.

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
fix. The existing suite passes either way, so nothing covered it. *Landed
2026-08-25 with run20 (`docs/AI.md` §15.8).*

**Transports and docks (`2026-08-25-transport.md`, the same day as the
mechanic).** The first reading in the main thread on Fable from the
decompile and run21's trace; two blind readers on Opus 5 in one wave,
briefed while the mechanic's own capture (run22, a `DUMP_ALL` window at the
frame the trace gave for the first dock) was running; adjudicated on Fable
against both the same hour. 114 claims, six verdicts against the document,
two of them changing Rust — a stale `gull_o` and a `num_coasts` that answers
1 for a sea region — and the rest names and scope: `num_captains` for
`num_units`, an air unit that skips a gate rather than failing it, a caller
list that turned out to be the census alone. The day's two largest facts
came from the capture and from neither reading: a dock registers in the
*sea* region, so the `reg_docks` counter both readings described is never
incremented for it, and owner 9's gull is not in the dump at all. The
lesson is `CLAUDE.md`'s "prefer a diff to a reading" with an edge: which
side of a shore a building's centre falls on is a map fact, and a reading
cannot settle a map fact. The sequence — read, stage the capture from the
trace's frame numbers, spawn the blind readers while it runs, adjudicate
against both — cost under two hours end to end and is the shape to repeat.

**Armies (`2026-08-25-army.md`, the same day as the mechanic).** The first
reading in the main thread on Fable from 5,500 lines of decompile and the
listing; two blind readers on Opus 5 in one wave, launched while three
`DUMP_ALL` windows of a raid on the AI's capital were being captured and
briefed with them; adjudicated on Fable against both the same hour. 142
claims, eleven verdicts against the document, six of them changing Rust —
a direction, a base register, a field twice over, a predicate's shape, a
switch's polarity, and a fold read as a reversal. The day's two lessons:
the readers went to `rise.pdb` itself for the enums the export does not
carry (`ArmyStatus`, `CITY_*`, `OBJECT_*`, the leader flags) and that
settled five inferred names at once — the export should ship them; and
the largest correction was to a claim the first reading had marked
"settled in the listing", read without its base register. The blind
reader re-derives every claim, the settled ones included. The captures
did what the readings could not: two of the three windows end an army
through the one search the sim stands in for, and only the dump could
say so.


**The group orders (`2026-08-25-groups.md`, the same day as the mechanic;
applied 2026-08-26).** The first reading in the main thread on Opus 5; two
blind readers on Opus 5 in one wave; adjudicated in the main thread on
**Opus 5** under the marker discipline, appending each verdict as it was
settled and marking `FABLE:` what could not be. 124 verdict rows, nine
verdicts changing Rust, five markers. The first reading was given **low**
standing by the brief and deserved it, for a reason that has now cost two
sessions: it never opened `rise_z.map`, and it guessed at five vtable slots
the PE names outright — three of the guesses were wrong, and two sections
had built rules on them.

Applying it a day later produced three lessons of its own, and all three
are about **where the evidence already was**.

- **The seam that was never a seam.** The document declared
  `Form::compute`'s slot table unimplementable partly because "no capture
  pins its output". `GroupData::log_data` had been dumping the whole of that
  output — five arrays per member — every frame the category was on, and a
  run with it on was already on disk. `CLAUDE.md`'s rule is "before
  declaring a seam, grep `log_data` for the fields it covers", and it was
  written *because of* this mechanic; the correction is that the rule also
  applies to a seam you are only declaring provisionally.
- **Four of the five `FABLE:` markers were settled in minutes, by the checks
  the audit itself had named.** A semaphore bit that the document called the
  network flag turned out to be "the scenario editor is open"
  (`ConsoleWin::run_cmd` sets it around `ScenarioEditor::init`); a claim
  about an asymmetric restore was confirmed by twelve instructions of
  `llvm-objdump`; a flag the audit thought might be vacuous was
  `unitrules.xml`'s own "flies like a helicopter", and is not; and a role
  bit was named by the type record. **A marker is a question with a costed
  answer, and the cost is usually smaller than the estimate written beside
  it.** Settling one on Opus narrows the Fable pass rather than discharging
  it.
- **The widening found something all three readings had missed, and it was
  outside the class.** `GroupData::facing` reads 1 on live groups in the
  capture, which `compute_form` cannot produce with that semaphore bit
  clear. The third writer is `Unit::kill_current_order@005e2cb0` — a
  different subsystem, which is exactly why a brief scoped to the `Group`
  family could not reach it. And the first `GROUPDATA` assertion written
  from the audit **failed on its first run**: `priority` is 0 on an emptied
  hotkey slot, because `Group::kill`'s `num == 0 → clear(−1)` writes over
  the bit. Two for two on `CLAUDE.md`'s "the first widening failed on its
  first run".

**The group orders' third pass (2026-08-26, the same file).** The first
ratification run **as the session** rather than as a subagent — bank,
`/clear`, Fable in the main thread — and briefed by a **charter** rather
than the list of nine, which is what let it find what it found. The floor
held eight of nine; the ninth verdict's two gates were right and its
*consequence* wrong, because nobody had followed what the first loop did to
the state the second loop reads (`order_type` after `clear_orders`). Its
sharpest finding was outside the floor and inside a function no pass had
opened: `UnitTypeData::get_stance_type`, the predicate every stance
decision resolves to, with its tests in an order the sim had wrong. Two
methods worth keeping: **the PDB's `LF_ONEMETHOD` records name a vtable
slot when `rise_z.map` cannot** (a COMDAT-folded slot has one name per
address in the map and its own in the type stream — `llvm-pdbutil dump
--types`, ten seconds), which closed the last marker and confirmed nine
inferred slots at once; and **delegate the mechanical sweeps, not the
judgment** — two Opus `lean` scanners over the whole export (writers and
readers of every field, callers of every function) cost a quarter hour and
surfaced a fourth `facing` writer, with every hit re-read in the main
thread before it became a claim. The one control the widening dropped is
recorded with its reason: a capture where the leader's heading equals the
move's bearing cannot separate the two, and only the listing can.
