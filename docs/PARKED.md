# Parked

The backlog that does not boot. `docs/QUEUE.md` holds what is in flight and
what is next; this holds everything else, and **a fresh session does not read
it** — the queue's opener is what starts a session, and this file is opened
when a *wave is composed*, which is a different and rarer moment (item 263,
decided 2026-09-07 with Ramon and lore).

Two rules ride with that, and they are the conditions the split was agreed
under:

- **Read this when composing a wave, not at boot.** Once per wave is cheap;
  once per session was the cost being removed.
- **A cross-item constraint does not park.** If an item says something a
  worker on *another* item must obey — that a file is about to be split,
  that a mechanism cannot move a scored word — it is stated in that
  worker's brief, or the item is promoted back to the queue. Two workers in
  one file is the failure this split would otherwise buy.

Numbers here are live: `tools/queueledger.py` reads this file alongside the
queue, so moving an item between the two is not a deletion and never reads
as one. Everything in `docs/QUEUE.md`'s "How to maintain this file" applies
here too, except the item cap — a parked item is not an open one.

## Parked by item 506, 2026-09-22 — a standing gap and two the ceiling found

(514) **`1/income[2:wealth]` 960 against 992, on all 245 blocks of
run117** — `resources[2:wealth]` and `rate[2:wealth]` (60 v 62) say it
twice more. **It names no frame**, which is why it parks rather than
taking the headline: run107's thirty blocks at 9170–9199 carry no income
row at all, so the 32 opens somewhere inside `(9199, 10375]` and nothing
on disk covers it. The AI's two unmodelled caravans are **a candidate,
not a measurement** — 176 is not 32, and the gap is not there at 9199.
A `MISC,LEADERS=2` run over that span is thin enough to take whole and
would date it to the frame; grep the disk first.

(511) **`1/2018 queue[0].cost[0]` on 10782.** Both sides now queue Horse
Archers at the Stable; this crate charges 60 timber / 40 wealth where the
original charges 57 / 38. A **price**, not a ledger.

(512) **`1/28 order:coll` on 10618.**

Both came under the comparison when the widening's ceiling followed the
word 10595 → 10830, and both are named in the test rather than swallowed
into a count. **A number that rises when a word moves is not a
regression** — same rule as (504)/(505).

## Parked by item 497, 2026-09-22 — brought under the line by the word itself

Item 497 moved Great Lakes 10303 → **10582**, and the widening's window
runs *to* the word — so 279 blocks nobody had ever compared came into
scope in one landing. Two counts went **up** as a result and **neither is
497's**: they were standing there the whole time, under a line that had
not reached them.

(504) **`1/35`'s standing position residue, from block 10353.** The
`run100_s_word_frame_is_the_original_s` position residue goes 5 units → 6.

(505) **`0/2000`'s `free`, from block 10400.** The same test's standing
city residue goes 20 fields → 21.

**Both are named in the test** — `RAIDER_SHORT` and `CITY_FREE` — rather
than swallowed into a count, so the day either closes, the pin fails and
says so. They park because neither names the headline's frame: 10583 is a
production frame and these are a raider's position and a city's field,
250 and 180 blocks under it. **A number that rises when a word moves is
not a regression**; promote either only when a takes-chain names it.

## Parked by item 496, 2026-09-22 — a piece that cannot score alone

(501) **`Sim::forget` drops a dead object from every attacker's target
slot on the frame it dies; the original keeps it.** Frame **684**, `order
0/6`/`0/7`/`0/8`, ours `None` theirs `(1,8)`, standing to 695. Invisible
until 496 widened `compare_orders`, and then thirty-six unit-frames of it,
under the word, quiet.

**Why this parks rather than queues:** measured, it moves no word alone —
the word stays 695 and two green pins go red. It is the smallest and least
informative of `docs/COMBAT.md` §43.2's four pieces, and the arm only
scores whole. It is therefore **inside item 502**, not a rival to it; do
not run it on its own on the strength of being true.

## Parked by item 494, 2026-09-22 — the wait table's unlit halves

Item 494 read `LeaderOptions +0x8` as an **index into five waits**
(1→7, 2→0xc, 3→0x11, 4→0x20, 5→0x3e, default→2) where this crate used the
index itself; `init` writes 2 and the wait is therefore 12.
`docs/ORDERS.md` §21. Three parts of that reading no run on disk reaches:

(498) **`think_caravan`'s half of the same table is reading-only.** Moved
to the shared `LeaderOptions::idle_wait` because the two listings are
identical — which is an argument, not a measurement. **No capture on disk
has a human caravan**, so nothing scores it. It is the honest kind of
reading-only claim: stated, not hidden, and `docs/ORDERS.md` §21's
coverage section says so.

(499) **`peasants_wait` is read from run12's `LEADEROPTION` record and
assumed to hold for run100**, which does not enable that category. The
whole of 494's nine-frame word rests on the assumption, and **one
`gamelog.ini` line on a Great Lakes capture settles it outright** — this
is the cheapest open question on file and the second genuine capture
question in twenty-odd landings (the first is (490)). ~~Promote it the
moment a Great Lakes capture is booked for any reason.~~

**Closed 2026-09-22 by item 506, off the disk and for two greps — no
capture was needed at all.**
`gamelog-run34-greatlakes-dumpall-start.txt` is this same game (map 14,
seed 12345, `GAME INFO` identical to run53's and run100's) and prints
`peasants_wait 2` on all eight leader-option records; and
`report.py rontrace-run53.log functions` puts
`CommandPackage::process_leader_options` at frame −1 and never finds
`set_auto_peasant_level` among the 6,937 functions the 24,000-frame
trace enters — so nothing rewrites it mid-game. `docs/ORDERS.md` §21 is
struck and points at both. **The fourth time in one day** that the disk
already held an answer something had booked a capture for; see (508).

(500) **The gate's `(idle − 2) % 5` retry above the threshold is
unexercised.** 494's citizen re-tasks on its first pass of 12, so nothing
in the window measures a retry at all. A second reading or a longer window
would reach it; the residue chase will not.

## Parked by item 489's landing, 2026-09-22 — its takes-chain target closed

(477) **f10235's `1/28 pos`**, ours (4801,30175) theirs (4800,30175), with
`g.x[0]` and `g.des_x[0]` the same — one world unit in x, said three times.
`docs/ORDERS.md` §18.3. It stood in the queue on one clause: that `1/28` is
the word's own squad and the row is **a takes-chain candidate to 10278's
collision**. Item 489 closed 10278 — `1/40` and `1/41` carry no row of any
kind in `[10270, 10307]` — so the chain has nothing left to reach, and the
new word (10294) is the **human's** citizen `0/5`, not the AI's squad.

The rows themselves are unchanged and still pinned, by value, in
`run100_s_word_block_is_every_record_the_dump_carries` ("block 10235 is not
item 477's three rows") — parking loses no measurement. It returns when
something names it: a takes-chain to a live word, or a floor that moves
with it. **Do not re-attach it to 10294 on the strength of proximity** —
that is the hypothesis-as-finding trap of `docs/DECISIONS.md` 42, and the
two units are on opposite sides of the match.

## Parked 2026-09-21, item 471's successor

(474) **A recycled group slot keeps its `order_num`, and this crate's
fresh one does not.** Great Lakes' probe group carries order `id 8192502`
— group 65, frame 8186, **order_num 2** — where this crate writes
`8192801`, group 68, the same frame, **order_num 1**. Two reasons, both
read off the listing and neither modelled:
`Groups::copy_group@006fa690` copies `who`, `num`, `ox`, `oy`, `o_dist`
and `o_angle` into the pool slot and the four member arrays with them, but
**not `order_num`**; and `Groups::push_group@0070f9e0` skips `copy_group`
altogether when `Group::equals_group` matches the pool's *last* slot
(`groups +0x1c`), so a group pushed twice running reuses its seat with
everything on it intact. Either would leave a recycled slot counting on
from where it was.

**Why this parks rather than queues, and the clause matters:** it changes
the group order's `id` and nothing else any comparison reads. `id` is not
compared — the seat numbers are this crate's own and deliberately
uncompared (`docs/ORDERS.md` §16.6, and `Sim::group_id`'s comment says
so) — so closing this moves no word, no floor and no endpoint count. It
is on the record because it is *known* and because the `id` arithmetic is
how frame 8186 was established twice over; it should not be promoted to
the queue on the strength of being true. Promote it only if something
later starts comparing the seat, or if `order_num` turns out to be read by
a mechanism that is scored.

## Loop — the steering pass's, never a worker's

Tooling, guards and the queue's own rules. The Fable pass takes these
(`CLAUDE.md`, "Fan-out rules"); a commander never spawns one. Item 279's
two ledger regexes closed in the third pass, 2026-09-17; the fourth pass,
2026-09-18, ruled six (`docs/audit/2026-09-18-fable-pass-4.md`); the sixth,
2026-09-19, ruled ten — 377, 398, 406, 404, 411, 397, 374, 343, 356, 321 —
each now a guard or a clause (`docs/audit/2026-09-19-fable-pass-6.md`,
DECISIONS 42); the seventh, 2026-09-21, ruled nine — 416, 417, 419, 420,
421, 424, 431, 433, 436 — two guards, six clauses and a DECISIONS
paragraph — and **built 367**, the AI's dump, the same day: run114 and
`docs/AI.md` §52 (`docs/audit/2026-09-21-fable-pass-7.md`, DECISIONS 43);
the eighth, the same evening, ruled eight — 251, 335, 428, 446, 449, 452,
453, 461 — code, five guards, a lock, a fixture and a clause, each made to
fail first, and merged PR #4 whole (458); 375 left for the ordinary list
(`docs/audit/2026-09-21-fable-pass-8.md`, DECISIONS 44); the ninth,
the same evening, ruled three — 467, 468, 469 — three clauses in the
commander's chain, after the loop stopped at its own reap
(`docs/audit/2026-09-21-fable-pass-9.md`, DECISIONS 45); the tenth,
2026-09-22, ruled two — 480 and 488 — both guards, each made to fail
first: struck text no longer counts against a section's ceiling, and
`rondata::diff::coverage` pins every key the dump prints that nothing
reads, 239 on twenty paths on its first run
(`docs/audit/2026-09-22-fable-pass-10.md`, DECISIONS 46).
341 closed 2026-09-18 by item 363.

(452) and of the same family as (449): an instrument that quietly stops
looking at the moment the thing it measures happens. Both are guard
shapes, not mechanics, which is why they are here and not in the queue.

(281) it read `ps rss`, which over-counted the mapping 280 removed — ~1.3 GB
of 260's 8,816 was run58's clean text — and a 2 s poll under-reports a
sawtooth. Still wants the fixture with teeth: past the cap, dead in N
seconds, exit 137.

(341) closed 2026-09-18 by item 363: `live_session.stage()`'s window and
categories are caller-supplied, the golden record's first run used them, and
`docs/RUNS.md` run101–run105 is the evidence. The lane's real constraint was
never the cursor — it is that a launch from inside Claude Code's own process
tree gets no window at all (`nodrv_CreateWindow`, dead in 3.8 s, 0 frames),
so every launch goes through `viadriver.sh` (`docs/ORACLE.md`, "The
click-free lane needs a window").

(513) **A moved word's pin records its value and not its story, and
nothing fails when it does not.** 2026-09-22: `LONG_WORD_GREAT_LAKES`
reads **10817** while its own doc-comment's narrative stops at **10303**.
Items 497 and 506 each moved the word and each changed the constant and
a window — four lines, two of them the value — and neither added the
entry. **Both briefs asked for it in those words**, after item 491 had to
land a second commit for exactly this.

**Why it keeps happening:** the guards check the constant's *value*
against the queue's `Long captures:` line, so a stale comment is invisible
to every check this repo has. The rule lives only in a brief, and
`CLAUDE.md` says a rule that could be a guard and is only prose will be
broken within the week. It was broken twice in one afternoon.

**What the pass should weigh:** a guard that fails when a word constant's
value changes in a commit that does not also add a line to its comment is
cheap and mechanical (the same shape as `a_constant_a_document_names_is_
built_or_pinned`). Against that: the comment is prose and a guard on
prose can be satisfied trivially. The alternative is that the commander
reads the pin at every merge, which is what caught it this time — but
that is a habit, not a check, and it caught it two words late.

**Narrower than it first looked, and the correction is the finding.**
The commander's first reading of this was that the story had not been
written at all. It had: item 506 pinned the new word's eight rows *and*
its draw delta in `run100_s_word_block_is_every_record_the_dump_carries`,
in `harness.rs`, with a paragraph of prose around them. What is missing
is only the entry in the **word constant's own comment** — so the defect
is not a lost measurement but a **split one**: the value lives in
`testkit.rs`, the story in `harness.rs`, and a session reading the pin to
learn what the word is gets the number without the reason.

So the pass's question is sharper than "add a guard": **which of the two
places is meant to carry the story, and should the other point at it?**
A guard that merely demands a comment line would be satisfied by
duplicating prose, which is worse than the split. 491's round trip
happened because its delta was in neither place; today's is in one of two.

**Note also how it was caught** — the commander read the pin at the merge
and found the narrative two words stale. That is a habit rather than a
check, and it produced a wrong first diagnosis before a `git show`
corrected it.

(509) **Filing to this section is the commander's default, and it was
not.** Ramon, 2026-09-22, after having to ask twice: *"it should be your
DEFAULT to tag to fable. idk why i have to ask you every time"*. Both
(507) and (508) were filed **only because he asked** — the commander had
seen its own misread and the three disk-already-had-it cases, narrated
both in chat, and filed neither. A finding that lives in a transcript
has told nobody; that is the same rule `CLAUDE.md` already applies to a
worker's report, and the commander is not exempt from it.

**What the pass should decide**: whether `CLAUDE.md`'s fan-out rules get
a clause making this explicit — *anything the commander would raise at
the steering pass is written to `docs/PARKED.md`'s Loop section in the
turn it is noticed, not mentioned in chat and not carried in context* —
and whether the handoff's `Fable backlog: N Loop items` line is enough
of a forcing function on its own. It was not today: the count sat at 2
through five landings while three fileable findings went unwritten.

The failure is **not** that the items were wrong. It is that a
process-level finding was treated as commentary while a mechanic-level
finding is treated as paperwork, and only the second has a guard behind
it.

(507) **A question about a resource is not a request for it, and the
commander acted on the wrong one.** 2026-09-22, mid-chain: Ramon asked
*"lmk when capture is avail as astra would like to use it (1-3min max)"*.
The commander read that as a request to clear the lane and, without
being asked, messaged **both** live lanes — telling att-506 to kill its
running capture if it had more than three minutes left, and att-502 not
to launch. Ramon's correction, in three messages: *"you dont have to
cancel the agents.. im asking when tis free"*, *"not to stop"*, *"ffs"*.

**What it cost.** att-506's run117 was ~30 seconds from finishing and a
re-take is ~12 minutes; it survived only because both messages arrived
together and it acted on neither. att-502 killed a queued wrapper, then
relaunched on the reversal, then killed run119 35 seconds in when the
*genuine* hold arrived — three reversals in about four minutes, from
three contradictory messages the commander sent in the wrong order.
Nothing was permanently lost (settings restored, aborted output dir
deleted), which is luck rather than design.

**The rule this wants.** A user's question is answered before it is
acted on; the answer to "when is X free" is a time, not an intervention.
An outward action taken on an inferred request is the same failure class
as a mechanism inferred from a draw site — and this file records eight
instances of the latter on the same day. **Whatever the pass writes,
it should not be a clause about capture lanes**: the lane is the
instance, the inference is the defect.

Also worth the pass's attention: **the commander's three messages
reached the lanes out of order relative to its own intent**, and each
lane acted on what it had. If a reversal is ever legitimate, it needs to
be one message that supersedes, not a sequence.

**And the intervention was unnecessary at the mechanism level, not just
at the etiquette level.** Ramon, afterwards: *"astra actually uses the
same tool to bring it up so it knows if lane is taken, so we might not
need to manually ask maybe"*. The lane lock (parked 446, eighth pass)
already arbitrates every launcher that goes through `winelaunch.sh`,
Astra's included — a second launch into a running game refuses and names
the holder, and the lock frees itself on a dead one. So the correct
answer to the whole episode was **to read the lock and report a time**,
which is what was asked for, and to let the lock do the arbitration it
was built for. The pass should check whether anything else in this
repo's operating rules re-implements by hand a thing a guard or a lock
already does.

(503) **A guard aimed at agreement, not at silence.** `coverage` pins
keys the parser never asks for; **nothing pins a key the parser reads and
the comparison then drops.** Item 496 found `compare_orders` reporting an
order's target only when *both* sides named one, so this crate's empty
order read as agreeing with the dump's `ox 8 whom 1` — thirty-six
unit-frames, under the word, green. Item 462 fixed the same hole one level
down in `unit_ids`. **Two instances in one function is a rule, and the
grep is mechanical**: `if let (Some(a), Some(b))`, `zip`, a `?` in a helper
feeding a comparison. Every guard this repo has is aimed at an instrument
that says nothing; this would be the first aimed at one that says *yes*.
Raised by item 496, 2026-09-22; `docs/COMBAT.md` §43.3.1 states the rule.

(508) **Three times in one day the disk already held the answer a
booked capture or a named mechanism was going to buy.** The
"grep the disk first" ordering is earning more than its one line in
`CLAUDE.md` suggests, and the pass should decide whether it is promoted,
sharpened, or made a guard. The three, 2026-09-22:

- **item 497** — `docs/ANIM.md` §4.11 had written its ambiguity down
  *and named its falsifier*: a `GUYS` window over a university holding
  two or more scholars. run100 holds **ten**, in two chains, and had
  printed all ten on every block since the day it was taken. No capture
  needed booking; the reading-only claim was closable from disk.
- **item 502** — §43.5 wanted a capture for its falsifier. It turned out
  to be **two fields of run112** (`new_ord`, `ever_in_range`), already
  on disk, distinguishing an `add_attack_order` from an in-place
  retarget. run119 was booked, killed at 35 seconds, and on the lane's
  own worker's assessment *"may not be worth much any more"* — all it
  would still buy is run118's missing blocks 847–900, which nothing
  open reads.
- **item 506** — the converse, and the control case: the disk **was**
  grepped first, the widening **was** run first (1,257 blocks, 3.4M
  rows), every existing `LEADERS>=2` window on map 14 missed 10582 by
  1,400 frames on one side and 13,000 on the other, and run117 was
  therefore **owed**. It paid: the cause is at 10576, a Setup step that
  spends no draw.

**So the rule is not "do not capture"** — 506 shows a booked capture
earning itself on evidence. It is that the grep comes first and the
booking cites what the disk could not answer. Two of three captures
booked today failed that test; the one that passed it found the item.

**A second pattern the pass may want beside it**: the mechanism named in
a brief has now been wrong **eight** times in one chain (487, 494, 497,
502 and others), twice named by the commander from the draw's callee,
and once — 502 — cited from a field (`near_o`) that *cannot* answer the
question it was cited for, since it is a search footprint written only
when a nearer candidate is seen. `docs/DECISIONS.md` 42 covers the
frame-versus-mechanism half; it does not cover **citing a field whose
write condition makes it silent on the question**. That may be the
sharper rule.

(313) **The landing chain wants one verb.** Merge, gate, push and reap are
one chain by rule since 09-17; ccc has `merge`, `update` and `clear`, and
the reap is a separate command a commander typed after the chain twice and
forgot twice. Filed with ccc as `land <ref>` or `merge --reap`; until it
exists, the chain is one shell line in the commander's brief.

## Parked by the tenth Fable pass, 2026-09-22 — names no score

(476) **f10234's three value rows**: `0/5 order:length` 2/1, `0/5
orders.len` 2/1, `0/2001 gather:gather_down[-1]` 5/2. Booked on the
word's frame when the word was 10234 (`docs/ORDERS.md` §18.3); the word
is 10277 now, the rows are a citizen's flee and a gather field
forty-three blocks under it, and no draw on the word's frame names
them. Returns when a takes-chain to the word does.

(342) **Host choice seats the scholar on the wrong university** — 338's
residue, re-pinned three times, and last `1/55`. Names no frame and no
floor; it stood in the queue through four passes on the strength of
being true. **Re-measure before diagnosing**: the vector `(768, 9984)`
is the claim, never the count. Returns with East Indies (444), whose
word it sits under.

## Parked by item 485, 2026-09-22 — 683's two sides, and two callers

(491) and (492), the two sides of 683 itself, were **promoted to the
queue by the tenth pass** as item 491 — a finding on the headline's
frame books, and the rules slot had stood empty behind a `Golden:` line
that named a landed item.

(490) **`(384, ATTACK3, 16)`'s release node** — the one launch-point
row the chapter-two widening still carries: `damage 1/6` at **680**
against the dump's 679. **Unmeasurable on the disk we have**: the
arrow lands inside its own launch frame, so no `AMMO` block is ever
written for it. Needs a capture with `AMMO` on a slinger volley at
longer range — the first genuine capture question in twenty landings.

(493) **The attrition caller's threshold.** `Sim::attrition_tick`
still kills on the squad-sized `health <= 0` — the same defect item
485 fixed in `Sim::take_damage`, one caller over, on
`Object::take_damage`'s `attrition != 0` arm. `docs/ATTRITION.md`'s,
not COMBAT's. **No capture on disk has an attrition death**, so
nothing scores it today; it parks until one does, or until the
namesake mechanic is worked deliberately.

## Parked by item 491, 2026-09-22 — a rolled arrow's landing

(495) **Where a rolled arrow comes down, and the nine rows behind it.**
Frame **686**: `damage 1/7` ours 0 theirs **19**, `damage_frac 1/7` ours
0 theirs 5, `damage_frame 1/7` ours 0 theirs **685**. run112's `0/8`
fires at 677 at `1/8`; `0/7`'s arrow kills `1/8` on 683; `0/8`'s finds
its target gone, rolls on (`docs/COMBAT.md` §42.2), and comes down two
frames later where `check_hit` finds `1/7` — a flank hit for `19+5/16`.
This crate rolls the arrow and drops it at the `3 × total_time` cap.
Nine more rows follow at **696-698** — `order 0/6`/`0/7`/`0/8` (ours
target `(1,7)`, theirs `(1,6)`), three `angle` and three `recharging` —
because a wounded `1/7` outranks `1/6` on §33's damage weight and the
original's bowmen retarget.

**Why this parks rather than queues, and the clause matters:** it is a
**hard-constraint** problem, not a residue. The landing needs
`v1z × t + sz + GRAV_Z × t² / 2` in IEEE singles — `v1z` is a float the
`AMMO` record itself prints — compared against
`TerrainOut::find_data_z@00866560`, a bilinear interpolation over a
float height surface this crate does not carry (a different quantity
from the integer corner grid `crate::terrain` models). `CLAUDE.md`'s
no-float rule means whoever takes it establishes the original's own
integer scales *first*, the way `combat::flight_time` did for the one
`sqrtf`; it is not an afternoon's residue chase, and it moves no draw —
the rolled arrow spends none on either side.

## Parked by item 481, 2026-09-22 — surfaced by a ceiling, nobody's item

(486) **`order 1/4` at 685 and `pos 1/4` at 686, a citizen forty
thousand units from the engagement.** Guy type 50, `myhits 40`, GATHER
on uid 3. Surfaced when 481 lifted `chapter_two_s_word_frame_is_widened_whole`'s
**ceiling** to 687 — the same shape as item 470's floor hiding `0/10`
at 630 — and re-measured with 481's change reverted and the window
left wide: they stand, so they are pre-existing and nobody's.

Same family as `order 0/5` (`docs/COMBAT.md` §32.5). Parks because it
names no headline frame and no floor; promote it if a chapter-two word
ever reaches 685, or if the §32.5 family turns out to be one cause.

## Parked by item 479, 2026-09-21 — wider than the word it came from

(482) **`Unit::think@005f6e40:104-134` has a second `near_o` reader, and
it fires thirty-one frames in thirty-two.** An idle captain takes
`add_attack_order(near_o, QUEUE_NEW)` when it is in range and **skips
the search entirely**; this crate searches on the phase frames and does
nothing on the others. `docs/COMBAT.md` §37.5.

**Not the current word** — `think` is reached from `do_idle` alone (one
caller, grepped) and `0/9` had a MOVE at its head — which is why it
parks rather than queues. But it changes what **every idle captain in
every capture** does, so it is the widest unlanded thing on file and
should be the first row a wave picks up when a captain's idle behaviour
is next in question. Whoever takes it should expect it to move more than
one word, and should measure before and after on both maps rather than
on the frame that sends them.

## Parked by the eighth Fable pass, 2026-09-21 — a guard a worker can take

(375) **A staged run is indistinguishable from an unstaged one, and the
borrow of a checksum trace is ungated.** Found by item 364, 2026-09-18,
while measuring its own first golden word as an artefact.
`borrow_from_siblings` gates `frame_seeds`/`frame_guys` on
`init.checksums.last().seed`, but borrows the **setup checksum trace
itself** ungated (`if init.checksums.is_empty()`), which makes that gate
compare a borrowed word with itself. Harmless where the donors are the
same game (run6/7/9); not harmless for the golden record, whose GAMEINFO
is **byte-identical to run11's** — map 14, seed 12345, size 2, the same 34
option fields — and whose setup word is the same `1003723497`, because the
script's first line runs at frame 0's `do_frame` entry, *after*
`Game::init`. **No gate on the map, the lobby or the setup word can tell a
staged run from an unstaged one; only the caller knows.**

364's runner does the right thing by hand — keeps the borrowed setup trace
after asserting its last word equals the trace's frame-0 entry word
(`0x3bd39ae9`), and refuses the per-frame records. The guard shape is to
make that the only possible behaviour: a golden run declares itself, and
the borrow refuses per-frame records from a run that did not. Written up
because the next golden capture will hit it and a silently-passing
comparison is what it looks like. Not urgent — see (376) for why.

## Parked by the seventh Fable pass, 2026-09-21 — two words without a widening

(444) **East Indies' word has no widening on file.** `LONG_WORD_EAST_INDIES`
is 9,711, and the last capture on that map compared whole (run90) sits at a
word two thousand frames lower; `rondata::diff::testkit::WIDENINGS` says so
and the guard behind it reads this row. Parks rather than queues because the
map is not the headline — lower map first, DECISIONS 41 — and it is the
first item the AI track takes the day East Indies becomes the lower word:
every record the long capture's game dumps around 9,711, both directions,
before any mechanism.

(445) **Chapter one's word has no widening on file.** `GOLDEN_WORD_CHAPTER_ONE`
is 626 and none of the tests behind it is a whole-cast `compare` over the
word's own frame — they pin the seating, the reach, the hit and the
hand-off; `WIDENINGS` says so. Parks because chapter two's 616 is the lower
chapter and the rules headline; it is the first rules item the day the
chapter two word closes, and it is one probe over
`crate::diff::harness::compare`, the shape 441 used.

## Parked by the sixth Fable pass, 2026-09-19 — a guard's first run

(440) **9582, one `use_market+0x1ed` short.** The AI headline's next
parting, measured by item 432, 2026-09-19, under the probe that gives the
Scholar its two per-city values: ours spends **one** `Leader::use_market
+0x1ed` where the original spends **two** — the market's sell rotation,
the same site as 9382's agreeing pair. Everything else on the frame matches
entry for entry (`make_stuff+0x221` ×2, `+0x63d` ×2, three `set_anim`, two
`Ammo::init`, `Farms::inc_time`), and 9581 and 9583 agree whole.

Parks rather than queues because **it is not reachable on the real tree
until `create_units` computes per-city** (438). It is booked by its frame
and its draw delta with no mechanism named, and it replaces 9518's scholar
birth, which was probe 4's artefact through and through.

(429) **A positive i32 wrap is live wherever `create_units`' tail is
reached.** Measured by item 422, 2026-09-19, on frame 9382:
`out = wm(fac, wm(want, val) / divisor) / 256` with `val 42,000,000`,
`fac 256`, `want 20`, `divisor 25` gives `256 × 33,600,000 =
8,601,600,000`, which wraps as i32 to **11,665,408 — positive** — and
divides to exactly 45,568. The original's product stays inside i32
(`5,755,741 × 256 = 1,473,469,696`) and does not wrap. **The sign is the
defect**: §45 documents this same overflow in the arm next door and guards
it with `out < 0 → 9,999,999`, so a wrap that goes negative becomes the
ceiling and is caught, while one that goes positive becomes a plausible
small number that ranks below a Citizen and is not. 422 pins the wrap as
an assertion on its own frame; this parked the general case as a defect.

**Corrected in place, 2026-09-19, by 430's second probe — the wrap is
fidelity and this item was wrong.** Computing the tail in 64 bits so
nothing wraps costs **2,727 frames**: the word falls 9510 → 6783, because
the original is a 32-bit engine whose own `imul` wraps and this crate
tracks it that far *because* it wraps too. So the overflow is not a
defect; it is a wrong input carried faithfully into a wrong answer, and
the lever is entirely upstream in `val`, `want` or `divisor`.
`offer_value`'s doc comment now says so with the number beside it, and
`LONG_WORD_GREAT_LAKES` pinning 9510 means a well-meant "fix" of the
overflow **fails the floor** rather than passing quietly.

What survives is narrower and still true: **`out < 0 → 9,999,999` catches
only half of a wrap's range**, so a positive wrap is invisible where a
negative one is caught. That is an observability property of 32-bit
arithmetic the original shares, not a defect in the expression, and where
it matters is a question nobody has asked yet.

(427) **Twenty-four frames pass before chapter two's first arrow.**
Observed by item 415, 2026-09-19, and **free to measure from run112, which
is already on the disk** — no capture, no screen. Parks because it names no
score: it is a delay this crate and the original may well share, and
nobody has compared them. Worth an hour the day a combat word lands near
it, and worth nothing before that.

(425) **A name a pinned constant carries for an index has nothing to
disagree with.** Named by item 408 while settling 423, 2026-09-19, and it
is the reason that defect survived from run19 to today. Both sides of the
leader diff generate their key from the **same** `GOODS` array —
`rows()` at `leader.rs:87` and `theirs()` beside it both `format!` the name
in — and the dump prints `bucket` as six bare values, so the index is
positional on both sides and **the label never participates in matching**.
The pinned literals in `PARTS_ON_RUN19/107/111` then check only that the
string matches whatever `GOODS` currently says. So `GOODS` and its pins can
be wrong **together, consistently, forever**: the specifications got it
right, the decompile export got it right, and 858 tests never looked,
because a self-consistent label has nothing to disagree with.

The guard is the check nobody wrote, and it is deliberately **wider than
`GOODS`**: every index-to-name mapping the diff layer pins is compared
against the type record, `GOODS` being the first instance. If others exist
it finds them the day it lands; if none do it costs three lines and says
so. A worker's item, not the pass's — a correctness guard about the port
rather than loop machinery. Make it fail on purpose first.

**Its sibling already exists, which is the argument for building this
one.** Item 422 wrote `assert!(45_568 < 234_782 && 234_782 < 5_755_741)`
— three literals agreeing with each other, unable to fail — one commit
after landing 423, and clippy's `assertions_on_constants` killed it in
ninety seconds because the gate runs `-D warnings`. That class of
decorative check has a working guard. **The label class has none**: no
lint knows that `GOODS[2]` should read "wealth", which is why one sat
wrong from run19 until someone traced a number to the array that named
it. Nothing to build for the assertion case; this item is the gap.

**Measured 2026-09-19 by 423's own fail-on-purpose**, which is what tells
this apart from a load-bearing label: with one row left on the old name,
every value comparison passed — 33,536 field-frames — while the `parting`
equality failed in **exactly one position**, `income[2:wealth]` generated
against `income[2:metal]` pinned. The pin guards the literal against the
generated name; nothing guards the generated name against the truth.

**The same shape one level up, and 423 found it in its own writing**: its
`docs/AI.md` §48.4 said "three pinned constants across three items", an
estimate written into a specification without counting, and the count is
one pinned constant and one assertion key. A number asserted in prose has
nothing to disagree with it either — struck in place and pointed at the
measurement. Whether that generalises to a guard over counted claims in
`docs/` is the pass's to judge; it is named here so it is not lost.

(423) closed 2026-09-19 by its own landing (`docs/journal/2026-09-19-item-423.md`);
the entry stood live a week after, and the eighth pass's number guard is
what found it. **`GOODS` is mislabelled in `diff/leader.rs`.** Found by item 414,
2026-09-19, while tracing what pays for a Scholar. Index 2 is named "metal"
and index 2 is what pays this Scholar's 40 **wealth**:
`sim::economy::Resource` has Wealth 2, Knowledge 3, Metal 4. **No verdict on
any capture is affected** — the comparisons are index against index, so
every number that has ever been checked was checked against its own
counterpart — but the *names* pinned at 2, 3 and 4 in `PARTS_ON_RUN19`,
`PARTS_ON_RUN107` and `PARTS_ON_RUN111` are the wrong resources, and a
reader who trusts them will reason about the wrong good. **Corrected in
place 2026-09-19, before the item ran**: the authority is the type record,
`rise.pdb`'s own `TypeIndex` — `0 FOOD 1 TIMBER 2 WEALTH 3 KNOWLEDGE
4 METAL 5 OIL` — which is `sim::economy::Resource` exactly; and the blast
radius is **six strings in one file, every one at index 2**, not three
constants across three items. There is no wrong row at 3 or 4, because
who=1's knowledge and metal never parted on those windows, and the
seventeen rows at 0, 1 and 5 are already right because those indices agree
between the two orderings. **No document changes**: `docs/AI.md:227` and
`docs/ECONOMY.md:861` already say wealth. Parks because it names no score;
a worker's to take, not the pass's.

(418) **Order coverage is a separate axis, and its unit is a native
issuer.** Found by item 365, 2026-09-19, while designing the golden
record's chapters. The cheat channel **stages state**; it does not order.
So seven of the eight new chapters add no order class the tree already
enters, chapter six adds one, and `bird` — console table case 82,
`Unit::add_air_patrol_order@005e4350` — is the only console command that
issues an order at all. "Every order class" is therefore not something the
chapters can deliver, and `docs/GOLDEN.md` §13 is the order-class-to-issuer
table DECISIONS 41 §5 was waiting for: coverage is counted in native
issuers reached, not chapters run. Parks because it names no score yet; it
returns when one names it.

(412) **Fourteen document-address pairs cite a dead-listed function.**
`docs_guard::a_dead_listed_address_is_cited_only_where_pinned` reads
`docs/EMULATOR.md` §4's fifteen and pins the citations standing on the day
in `DEAD_CITED`: `LeaderData::is_human@006ec170` in ARMY, COMBAT, GROUPS
and ORDERS; `find_wpath_army@00683730` in GROUPS and ORDERS;
`get_estimate@00688310` in PATHFINDER and RUNS; `locked_transport@006d5230`
and `find_dock@0065cfd0` in TRANSPORT; `set_domain@00633390` in COMBAT;
`add_attack_order@00622ce0` in ORDERS; `set_gathered_at@006b46b0` in AI;
`cos_table@00a469f0` in MOVEMENT. Each cites a standalone body the linker
kept while the live copy is inlined in a caller, so the claim rests on
whatever else backs it — run58's dump for `is_human`, a listing for the
rest — and the reconciliation is to say so at the citation and delete the
row. Names no score; a worker takes one document at a time, and the row's
deletion is the landing.

(413) **Seventy-seven constants the specifications name that no crate
carries.** `docs_guard::a_constant_a_document_names_is_built_or_pinned`
pins them by file in `UNBUILT` — AI 20, ORDERS 9, TECH 7, CITIES 6, GOODY 6,
ARMY 5, ECONOMY 5, PRODUCTION 4, GROUPS 3, and one or two in nine more —
and a file may only shrink. Three kinds, to be told apart one file at a
time: a field offset written without its `+` (spell it `+0x..` and the
guard stops reading it); a value named for context the code never needs
(say so beside it); and a constant read and never built, which is the
kind that cost a month at `0x66`. The third kind is an item on whichever
track its frame is on; the other two are a document edit and a lowered
pin. Names no score until a row does.

## Parked by the third Fable pass, 2026-09-17 — names no score

(310) **Nothing decrements the muster on death** — not `by_type`,
`by_group`, `control` or `active`. The original's `Unit::close@0060ee50:235`
undoes all three and this crate has no counterpart. No unit of players 0
or 1 dies in either window, so no diff reaches it, which is why 303 could
land `Sim::track_unit_type` correct in both directions of `set_type` and
still leave this open. Falsified by a window containing a death. AI §37.

(305) **The make list's five building values, and a factor of ten** — the
building producer's rows `MAKE[0]`..`MAKE[4]` still part, with `MAKE[2].cat`
beside them, and two are exactly ten times out: `MAKE[1]` 202500 against
2025000, `MAKE[2]` 165000 against 1012500. A different producer from 302's
`upgrade_units`, and the factor-of-ten shape is as strong an oracle as 302's
factor of two was. Value diff on disk, no capture needed. AI §36. Parked
because 302 and 303 closed make-list rows and moved no word.

(278) **`Unit::work@0060d180:440` is a second `set_new_location(…, 1, 1)`**
and nothing models it — two units of one type within `0x180` are pushed
apart by half their separation and **both snapped**, gated on
`field_0x82 < 0` and the order's `+0x30` vcall (read, never run). Ruled out
for 271's 6937; `Guy::last_pos` makes the signature searchable on every
dump: a unit that moved whose figure has `last_x == x`. run79's squad first.

(220) **TECH §13's twenty range blocks.** (209) **the once-per-game events a
dump install swallows** — a `set_*` whose **return value** drives an
irreversible record; **takes 224**. (224) **`mil_trainers`' other three
writers** (AI §29.4) — a trainer that changes city, upgrades in place or is
captured is filed by neither; referenced since 09-04, booked 09-17. (203)
`mark_behind_tiles`' `0x4` is a building *finishing*; (240) the merchant's
`gather_down`/`special` −1 at 6929, **takes 184**; (216) `create_buildings`
offers a gather building the original does not.

(242) **The crate's group id is not the original's** — `GroupData +0x4 =
64` against `group_id`'s stand-in, the army group's *slot* 1; the other
five agree on all 630 and 237 reports the row without scoring it. (243)
`UnitDump::group` is parsed and **nothing compares it**. (244) **1,428
grouped order records no test windows** — run79's 453, run31's 945. (245)
`focus.sh` matched a concurrent worker's shell on run84.

(255) **`go_to_unit`'s `0x480` and `go_to`'s `MOVE_TO` arm are read and
never run** — every joiner on disk is farther than `0x480` from its army's
`get_unit(0)`, so the falsifier is one born beside its army. (256)
`come_out`'s three `action_move_to` sites stay unreached; (254) its ring
has one sample (7284) and wants a second disembark. (167) run61's two (SYNC
§3.9). All four came off 253, which 271 closed.

(247) **An upgrade is an in-place guy-type change on the standing unit** —
run76's **6737**, three Archers going guy **170 → 177** keeping `(who, o)`
and `group 64`; East Indies' `1/32` does 340 → 341, the danger row moving
by `(110 − 100) / 2`. (181) CARAVAN §7.2–§7.3.

**The widening ledger** (87, DATALAYER §4, §4.1): **18** fields the harness
names nowhere, **46** one capture names — item 308 moved both on 2026-09-17,
`start_dist` off the uncompared list and `uid` onto the single-capture one; blind spot `avg_speed` (210) — it
counts *fields* and cannot see a record the parser never visits, which is
what hid 252's dumps. Uncounted: (88) the blind list — **802 cited, 650
entered, 152 never** since coverage came back 09-08 (ORACLE), not the 101
of 617 this line said for a week; (72)
every `+0xNN` a document pins vs its module; (89) the guard; (35) VISION §7.

(269) **The third axis: a field compared at the wrong width** (DATALAYER
§4.2, from 265). Ten `CityData` counters are `uchar` in the type record and
`i32` in `ai::CityAi` — `busy` and `gatherers` have bare byte writers of
their own — and `pop` is an eleventh on `sim::City`. None has been seen to
wrap; the falsifier is a capture where the sweep's count and the producers'
decrements cross zero. Same question one record up, for every
`char`/`short` of `LeaderData` held as an `i32`. A guard wants the PDB's
widths beside the sim's structs, not a grep.

(270) **The fourth axis: a word compared one bit at a time** (DATALAYER
§4.3, from 267). The ledger scans the differ for a field's *name*, so
`unit_masks` has been on neither list since the packed bit got a row while
one of at least eight modelled bits was actually compared — and the missing
one, `0x100000`, named Great Lakes' run-up cause on its first run. Nine
dumped masks want a per-**bit** census: `unit_masks`, `unit_masks2`,
`guy_flags`, `node_flags`, `city_flags`, `leader_flags`, `leader_flags2`,
`build_flags`, `role`. Same tool as 269.

(257) **Nineteen kept tests compare a torn block** (252): a closing dump is
frame n but for the one unit the quit caught mid-update, and every nested
archive's last `FRAME n` body *is* its closing dump. No live case found,
unaudited; fix is `compare_shutdown`'s n−1 allowance.

(175) **The uber chain past its birth**: `Objects::init_unit` threads
`uber_size` objects (CITIES §4.3); nothing else reads it. Takes (48)
COLLISION §3/§7, (73) `UnitData::group`'s back-pointer, (56) ARMY §13.

(161) **The make-list block is 2,500 frames behind East Indies' word**:
`create_buildings` first runs on 9982 (AI §25), so `building_value`,
`gather_value` and §24.4's arm wait on it. (195) `find_repair_spot`.

(211) **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
doubling; the group cap, gated on `action_type == 0` (219) — it wants a
grouped unit holding no action. (229) a figure in melee does not step its
clock — `unit_masks2 & 0x10` freezes `Guy::inc_time` (ANIM §5), and that
arm waits on a melee frame. **312 names this first** if the 7679 figure is
in melee.

(291) **The AI's caravans are not linked** — `vans.length` 0 against 1 and
`trade_val` 0 against 128 on **both** of player 1's cities, 246 of 246
blocks (287). That names two of 285's eighteen fields and gives them a
mechanism, and wealth is what a market buy spends, so it is a live
candidate for 290's shortfall. (288) the gull's `do_strafe`, unmodelled.
(274) run87's `1/26`. (268) `AnimalData::ox`/`whom`/`aid` carry nothing —
−1 on all forty animals on all 247 blocks. Closed as answered, not open.

(285) **The CITY record parts on every block and nothing asserts it** — 18
fields on 246 of 246 of run89's window, found by widening the whole record,
and **no window test on either map asserts `city_diverged`**. Two of the
eighteen are named now (291, the unlinked caravans). `1/3`'s `(+192, +192)`
`MOVEORDER` was booked here and closed itself when 284 landed the
air-physics fix.

(273) **`refresh_group_order` re-origins on the order's `form_id`, not the
member's list position** — `713ac3` reads `[eax+0x10]` off the `GroupOrder`
the `+0x94` vcall returns, where `Sim::group_refresh_order` uses
`g.list.iter().position(member)`. `do_group_move` step 2 rewrites `form_id`
every frame, so the two agree except where membership changed and the
follower arm has not run: wants a `GROUPS=1` window across a death or a
join in a marching formation. (275) `MoveOrder::facing` still does not
score; 267 split the two mechanisms, so re-read that. **304 is the nearest
live window** to this — a formation ending early on run76.

(315) **run90's `1/7` names no blocker on 7820** — `collide_o` and
`collide_who` read −1 where the original has 6 and 1, with `collide`
itself, the count, agreeing. New on 2026-09-17 with the suspend wiring
(items 301/304), and the window cannot price it: every position, every
angle, every order record and every draw count over those 111 blocks
agrees, and the two Merchant constants are the only parted units left.
Pinned in `run90_s_window_is_east_indies_shuffle` so it cannot move in
silence. Parked because it names no score — the falsifier is a capture
where a missing blocker identity changes a decision.

(316) **Sixty-two other callers of `clear_partial_path` are unchecked**
— item 304 fixed `kill_current_path`'s and no capture reaches the rest.
The `Group::action_*` and `think_carry*` families have **no counterpart
call in this crate at all**, which is the shape 304 turned out to be, one
level up. Parked because it names no score; it comes back the day a
window contains one of them.

(325) **`MAKE[*].city` is ours + 1 on every offer** — this crate's city
array puts the human's at index 0 and the AI's at 1 and 2 where the dump
reads 0 and 1. No offer in run19's window is chosen by the index, so
nothing scores it; the falsifier is an offer whose choice depends on the
city. Item 323, AI §38.

(326) **Three tech `val`s part from before run19's window** — Empire
2,100,000 against 1,800,000, Mercenaries 165,000 against 216,000,
Mathematics 82,500 against 63,000, all `research_techs`' arithmetic. A
value diff on disk, no capture needed; parked because the window that
prints them is not a scoring one. Item 323.

(327) **The Merchant is still not offered at slot 3** (`t 61`, `val
869,565`) — `civilian_value`'s merchant arm reads `known_rares`, which is
still a summed-region seam. Item 323, and a sibling of 291's unlinked
caravans.

(333) **`1/28`'s path stack is 41 where run19 says 42** — an off-by-one
already present at 8186 and belonging to `find_wpath`'s plan near the
goal, found by item 329 and deliberately not closed by it. The chase and
the delay are right either side of it, so nothing on the word depends on
it; the falsifier is a plan whose last leg the count decides. PATHFINDER
§21.

## Parked from the lab, 2026-09-17

(309) **`find_upath`'s pre-walk give-up exit targets the wrong label** —
`00683082` is push-and-return-length where this crate's `break` falls
through to the near test and the search. Unreachable today for a
transport-capable unit and no capture reaches it, so 301 recorded it in
PATHFINDER §18.4 rather than changing it. Parked because its falsifier does
not exist on disk.

(311) **The zero-pop predicate disagrees on paper and nowhere else** — the
original counts a `control_cost == 0` unit only when `is(0x134)` or
`is_gov_hero`; this crate's muster seams apply no test at all and its sweep
skips every zero-pop unit outright. Two different wrong answers that agree
on every capture, because nothing on disk separates them. AI §37. Parked
with 309: a reading-only disagreement with no falsifier.

(298) **The click-free capture lane starts 6 of 10 pairs** — a
`RON_AUTOSTART` tracer build replaces the two menu functions and clicks
Start from inside the modal loop; every success matched 19 logged bodies and
1,401 frame seeds against a hand-driven run. The failures are 180 s timeouts
and exit 40 before the menu; two faults map to the WoW64 transition RVA
0x1139, and moving the hooks off bulk restores (226's family) did not cure
it. The factor-isolation experiment is paused
(`docs/lab/2026-09-09-paused-runtime-experiment.md`). Falsifier: a ten-pair
cohort at 10/10. Lab rows L18, L19, L28, L29.

(299) **A single Gaia reseat correction hides a 556-frame heading
difference** — run69, Great Lakes: of seven actual reseat writes across
three intervals, removing tick 99's leaves this crate's Gaia heading off the
original's for 556 frames, only the control matches at 106, and all seven
converge by 3001 — invisible to the player comparator throughout. Lab rows
L53, L54, `docs/lab/2026-09-10-single-reseat-interventions.md`. Wants a
Gaia heading row in the differ; unscored today.

(300) **The original's command API is unused, and the replay adapter lowers
only `MoveTo`** — `CommandManager` has 68 `issue_*` entries;
`CommandManager::issue_move_to@00941720` builds the command and appends it
to `local_package` after `CommandManager::check_accept_issue@00940a70`,
while `tools/fuzz/scenario.py` teleports. On this side `input::Stream::one`
applies selection and `MoveTo` and skips production, repair and market. Lab
row L08: acceptance is not emission — package capacity can refuse after
selection changes — so an adapter needs acceptance, emission and
processed-frame witnesses. A prerequisite for manufactured falsifiers and
phase 5, not headline work.

## Parked from the queue, 2026-09-07

(277) **An age gained through a cascade is unmodelled** — `Sim::gain_tech`
reads the age gate off the type the call was made with, as the original does,
but the original **recurses into `gain_tech`** per cascaded grant where this
crate flattens the cascade into events, so a cascaded age would take 271's
snap arm there and not here. Parked to seat 290/291 under the cap, on the
ground that **no capture on this disk gains an age other than directly**, so
nothing can falsify it today. It is cheap to unpark: `BuildDump::max_age` is
parsed, so the falsifier is a grep rather than a reading — a block whose
`max_age` moves on research that is not one of the seven ages. Cross-item
constraint, and the reason this is not a silent drop: **271's snap arm is
live code**, so anyone touching `Sim::gain_tech` inherits this question.

(248) **ATTRITION's Territory section is 15,625 bytes of 16,000** and wants a
retelling pass before anything is added. Parked to seat 283; its remaining
terms are **closed** — handicap, temple and fort are unreachable in any game
that runs here — so nothing is waiting on it and the retelling is owed only
to whoever next adds to that section. Read it before re-booking any of the
three. (AI §2.1's `check_explore`.) No cross-item constraint: the size ceiling
is per-section and `docs_guard` enforces it, so a worker cannot trip over this
without being told by the guard itself.

(234) **Four rules of the turn/idle animation neither side has** (ANIM §9),
written and never landed — branch `rescue-234`. Parked by the commander to
seat 276–279 under the item cap, on the item's own words: the limbs move no
score, since §4.7's turning types are in neither scored game. **The two
static assertions are the cheap half** and are what to take first if it
comes back. It carries the ledger's nine `unverified` rows on the way, so
whoever unparks it inherits that audit. Nothing here is a cross-item
constraint: no live worker is in `crates/sim`'s animation code.

## Steering candidates, booked 2026-09-07 with Ramon

Neither of the two below is a mechanic and neither moves a word, which is why both park
rather than take a slot under a cap that stood at 18 of 18 the day they
were booked. Ruled by the second Fable pass the same day
(`docs/audit/2026-09-07-fable-pass-2.md`): 293 closed there, 292 stays,
and 283 came here from the queue to sit behind it.

(292) **Nothing in this tree has ever been profiled** — no bench target, no
criterion, no flamegraph, no samply, and no mention of Instruments anywhere
under `docs/`, `crates/` or `tools/`, grepped 2026-09-07. Every measurement
the gate has ever been given is **memory**: the 15,128 MiB peak at two
threads, the memcap in front of it, and the ratchet items 235, 260, 280 and
283 that came off it. Its **256 seconds** is a wall-clock fact with nothing
under it — no split between parsing a dump and comparing it, which is the
obvious first cut when run89 is 121 MB and run90 72 MB of text. That
ignorance is upstream of more than the gate's runtime: 283 proposes a
`#[global_allocator]` on reasoning about what macOS's allocator keeps, and a
profile is what would settle it rather than argue it. Cheap to start —
one bench target over the differ's parse and one over a sim tick — and it
earns its dependency the moment it prints a number nobody predicted.
**Ruled 2026-09-07**: stays parked with this first cut; the gate's five
minutes is not what the loop was losing time to (293), and the item is
taken by whichever worker next touches the gate's runtime, or with 283.

(283) **The ratchet wants a `#[global_allocator]`, not a mapping** — 280's
+1,563 MiB is *not* live data: a mapping is `munmap`ed at drop, while a
freed `String` of a capture's size is kept by macOS's allocator and cannot
serve the next capture's different size (260's own 5,332-MiB-with-nothing-
alive probe, again). An allocator that returns large blocks recovers most
of it with **no `unsafe` in this tree** — the crate carries it — and closes
235's ratchet for every large owned buffer, not the two that were mapped.
**Parked by the pass behind 292**: the retained memory is not live, not a
score, and not a hazard behind `memcap.sh`; a dependency is earned by a
measurement, and 292's bench is that measurement.

## Parked at the 289/290 merge, 2026-09-07

(296) **Great Lakes' endpoint is eight units short and nobody knows which**
— 290's `get_mod_resource_cap` fix halves the AI's commerce cap on this
lobby's Easiest, which is the right-hand side of every `income < cap` gate
in `create_buildings`, `create_units` and `research_techs`. It took 33
spurious units off the four endpoint rows and on Great Lakes went eight
past the mark: `(1,73)`-`(1,80)`, the last eight object numbers the AI ever
reaches. run53's endpoint is MISC-only, so their positions are known and
their types are not. **The cheapest falsifier is already on disk and was
not spent**: run80 is Great Lakes `LEADERS=9` over [23960, 24000), and that
record carries `num_units` and `num_queued`, which would name exactly which
types the AI is short of. Parked rather than booked because the queue stood
at 18 of 18 and this sits 16,400 frames past a word neither map moved, on
streams that are nobody's. What keeps the change that caused it is that its
own evidence is local and strong — twelve lines of decompile at 006d65b0,
and `rate[0]`, `rate[1]` and `best_good` going from wrong on all 80 blocks
of run84 to right on all 80. Expected in direction, unchased in size, and
290 declined to call it clean rather than papering it over.

**Two senses of one word, kept apart on purpose.** At the endpoint,
`unlinked` is a `(who, o)` the dump has and the simulation does not — a
unit the original built and we did not. In item 291 it is a caravan not
linked to a city. Same word, different counter, and blurring them would
make either number unreadable.

## Parked 2026-09-18, the two-lane session

(372) **The held-out map's 1 is comparable in kind, not in provenance.**
run106 ran on the click-free lane's lobby, not run33/run39's `-config
check.ini`, and its window opens at frame 1 rather than matching the score
runs' shape. Strict comparability wants a run33-shaped capture on map 9 —
a second capture and a second decision, which is why 363 left it. Park,
not bury: the 1 is the generalisation number and the first pass that
reads it should know exactly how it was taken.

(373) **The click-free lane's give-up truncates rather than stops.**
`--timeout` defaults to 180 s and the first held-out attempt returned 593
of 1,900 blocks in a **62 MB file that reads as completely ordinary** —
only `receipt.json` said `success: false`. The silent truncation is the
finding, not the timeout: every check that greps a dump would have passed
on it. In `docs/ORACLE.md` beside the new knobs. A guard shape: a capture
asserts its own block count against the window it asked for.
**Half-closed 2026-09-18 by item 369**: run107's stanza is the first in
`tools/gamelog/captures.txt` to carry that assertion (30 blocks,
9170..9199, no gap). Making it every capture's is still open.

(376) **The sibling borrow is measured harmless on every scored path, and
this is the record of it** (item 364, 2026-09-18, the negative the
commander asked for before pinning). Five Great Lakes captures — run33,
run53, run89, run97, run100 — take 14 `frame_seeds` and 14 `frame_guys`
from run12/run13 at frames 0–3 and 94–103. East Indies takes **nothing**:
setup word `793793043` never matches the donors' `1003723497`. run53 walked
all 24,000 frames both ways gives **word parts at 9362, sequence at 9182,
identical with the borrow and without**; run33's floor is unmoved between
the two columns. The installs are no-ops — our sim already produces the
original's word at each of those fourteen frames, all at frame ≤ 103 and
9,079 below the word. Structurally, `Built::tick` pushes the frame's
labels into `frame_sites` **before** installing anything, so an installed
frame's draw comparison is still the sim's own work; the install can only
re-anchor the next frame. **No pinned floor or word rests on installed
sibling frame data.**

(378) **Put the human back into `census_wars` and `census_strategy` too,
and the residue gets worse before it gets better.** 368 fixed
`census_territory`'s gate (the human is a leader: `plan_strategy@006b9620:159`
tests `leader_flags & 2`, `i != who`, `i >= 0` and nothing else, and on a
one-human-one-AI game the human is the only other leader there is);
`1/other_team_terr 0 theirs 266` is `0/my_team_terr 266`. The same defect
sits in the other two loops, but fixing them makes all five of
`weight_total`'s census facts agree and takes run19's leader residue from
**90 fields to 147** — the make list then parts on `t`, `cat`, `city` and
`val` across nearly every slot, because `active_wars != 0` reaches the
danger word and the region-strategy words as well.

The number is in the row on purpose: a successor that makes a residue
worse before better does not get taken unless its cost is visible.
Carried with it: the original runs a **full census for its human leader**
(`0/gatherers 5`, `0/peasants 5` in every dump) and this crate runs none,
so `census_strategy`'s `weaker` test reads the opponent's `attack` as
nought even once the gate is fixed. That is the part to build first.

(380) **A bare coordinate on the staged channel is a TILE**, `n × 0xc0 +
0x60` — measured by item 364, not read: `add hoplite who=0 4,40` put the
squad head at `(888, 7800)`. `docs/ORACLE.md`'s stored `arg × 768 + half a
footprint` is corrected for the staged channel, and 363's open "the
coordinate argument did not calibrate the way the runbook says" closes.

(383) **run107 opened two families never compared on Great Lakes.**
Player 1's ten `SITE` slots part as a **re-ordering of the same ten
sites** — our `SITE[3]` is their `SITE[1]` and so on, with `rank` in the
residue on every slot — and `tech_frame`/`tech_cat_frame[0..3]` sit at
nought against 8382/4976/8382/8182/6376. Both are inside
`run107_s_window_is_the_leader_record_at_the_word`'s 109 fields, so they
are floored and cannot regress silently; neither is on the word's frame,
which is why they park rather than queue. The re-ordering is the more
interesting: the same ten sites in a different order is a comparator or an
insertion order, not a missing mechanic.

(387) **Target selection should walk the world cell's object chain, and
ours is units-only.** Found by item 384 while closing 617's range half.
The crate already maintains the chain; switching the scan to it would
**drop buildings from target selection**, so it is a bigger item than it
looks and it would not have moved this frame. Parks on that second clause
— it names no score today — but it is the shape of a whole family, and
whoever takes it should expect the building half to come with it.

(388) **The golden record's `1/7` and `1/8` get their target from a group
path the sim does not have at all.** 384's other parting note: `near_o`
shows only a squad's captain searches (`1/7`, `1/8` and all of who=0 are
`-1` for 900 frames) and `Group::target_opportunity` spreads the captain's
find, which this crate does not model. Documented in `docs/GROUPS.md` §13.
Not on 617's frame — 386's tie-break is — so it parks until a word names it.

(393) **who=0's members take the captain's order a frame after it** —
`docs/GROUPS.md` §13's mirror of the thing item 391 closed on the other
side. 391 established that `Unit::target_opportunity@005fffc0`'s opening
`while` walks `o_up` to the victim's **captain**, and that the army rather
than the group moves a struck unit (`Armies::emergency` → `Army::process`
→ `Group::action_siege_attack_to`; `Group::target_opportunity`'s branch is
guarded by `type +0x2c8 & 0x10000 == 0` and never runs here). Dump-backed:
at the end of 616 the ATTACKORDER is on `0/6` alone, `0/7`/`0/8` a frame
later. Parks because 392's ring is what stands on the word's frame; this
is one frame below it and will likely fall out of the same model.

(400) **`0/8` plans its chase at 619 to the target's own point** where
the dump has (1224,8280) at 618 — item 395's second successor, dump
coordinates in `docs/COMBAT.md` §20.4. Below the golden word (624) rather
than on it, so it parks; likely to fall out of 399's `Army::process`
model, and worth re-measuring before it is taken.

(401) **Two reading-only residues from 395**, `docs/COMBAT.md` §20.3 and
§20.5: `find_melee_target@005ff9c0` carries its **own copy** of the
non-captain mirror, unimplemented here, whose only reachable caller is
`docs/GROUPS.md` §10's per-member call; and `think`'s shared epilogue at
`5f761a` clears the "could not reach" bit, which this crate never clears
at all. Neither is diff-backed — a run has reached neither — so they are
exactly the kind of claim the coverage section must list as reading-only,
and the kind a blind second reading is briefed with.

(403) **The launch table has one piece missing, and one capture closes
it.** Item 396 measured every release event `unit_graphics.xml` gives the
Longbowman — six rows reproducing all nine launch points to the unit
across three units and two facings — but run17's three Slinger families
are **measured and unusable**, because run17's `GUY` detail is 1 and
nothing there names the animation. A **`GUYS=4` re-run of any Slinger
fight** closes it, and the same shape closes every missile type in the
game. Cheap, and it generalises further than the item that found it.

(407) **A pool dump needs its INHERITED category set high enough, and
"DEATHS off" is the wrong rule.** Item 399 lost a `GROUPS` pool that came
back empty with `DEATHS` already off. `GuyData::log_data@005de6c0` sets
type `0x14` and then walks `set_detail(1,2,3,4)`, and **`set_detail` runs
whether or not the line is accepted**, so the pool inherits GUYS at detail
4 and `check_accept` drops it at `GUYS=2`. The rule is that the inherited
category must be set **at or above that dumper's highest `set_detail`**;
run31 and run92 satisfied it by accident with `GUYS=9`. In `docs/RUNS.md`
run110. Riding along: `tools/explore/golden_capture.sh` cds to
`tools/explore`, so `--cmd-file` must be absolute.

(409) **The ammo pool's order is not this crate's, and a two-landing frame
would spend the draws in the other order.** Found by item 402, 2026-09-19,
while widening the `AMMO` record. `Objects::add_ammo@00658b10` scans its
pool from slot 0 and takes the **lowest free slot**, so the original's
order is slot order with reuse — measured 0, 1, 2, 3, 4, 0, 1, 2, 3 over
run109's nine arrows. `Sim::projectiles` is a `Vec` that
`process_projectiles` `swap_remove`s from, so from the first landing it is
neither: at 9453 it holds a4, a3, a2 where the original's pool reads a2,
a3, a4. **Nothing in run109's window turns on it** — no two arrows land on
the same frame after 9451, and landing is what draws — which is why it
parks rather than queues. It becomes an item the moment a frame lands two.
A slot pool is the fix and it could move the word **in either direction**.
`docs/COMBAT.md` §24.4 names the rule and the consequence.

(410) **Three `AMMO` branches with no capture behind them.** Also 402. All
183 blocks print `traj 1`, so the parser's nested-`SplineData` skip — the
one written by indentation because the log writes no `END` — rests on a
hand-built fixture; **a catapult, or anything whose `ammo_path` is set**,
would make it evidence. Alongside it: `flags` bit 1, `rolling` and
`start_roll_angle` have no behaviour behind them, and nine allocations
from a cold pool cannot say whether `graph_index` wraps. Every row here is
one Longbowman shooting one farm on one trajectory, and each of the three
wants a different shooter rather than a different frame.

## Measured residues, none near a word

(454) **Five residue families never compared on Great Lakes, measured by
448's widening.** 2026-09-21, from 837 blocks and 1,975,563 record rows:
`form` **48 rows** (ours −1, theirs 9 on every unit outside a group — the
largest and the cheapest to read), `dest_angle` 22, `orders_x`/`orders_y`
36 (18–24 units off, one pair 984), `g.angle[0]` 10 (ours 1,431,655,765,
a third of a turn, against 0), `stance` 6 (item 190's, unchanged). All
are **older than the word**: of 298 keys parting over the window, 161
stood on the window's first block and all 49 that opened below the word
belong to a family already standing. So none of them names a score, and
a successor takes one only when a word's frame reaches it.

(455) **Two dumped records that nothing in `rondata::diff` compares at
all.** Item 448, 2026-09-21. `LEADERDATA`'s `score` and `leader_flags` —
four blocks a frame, on **every** capture ever taken — and the per-frame
`WORLD` census (`forest_size`, `mountain_size`, `rock_size`,
`total_metal`, `total_oil`, `goodies`, `land_resources`,
`sea_resources`), which the parser does not even read. The rule this
fails is `CLAUDE.md`'s own: when the original dumps a record, diff the
whole record. Nine tenths of a dumped record once went uncompared for a
month and the first widening failed on its first run; these are the next
two.

(451) **The original runs its war census for the HUMAN leader, and this
crate leaves it at zero forever.** Opened by run115's window (item 390,
2026-09-21) and not that item's: `0/wars`, `0/active_wars` and
`0/active_wars_with` agree at nought for 121 blocks and part on **8001**,
where the original writes the human `wars 1`, `active_wars 1`,
`active_wars_with 2` beside a single `production_step` tick that falls
back to 0 on 8002 — 56 blocks after first contact. **First direct
evidence on this disk** that the census runs for a human at all, which is
what `docs/AI.md` §45 argued from the decompile
(`plan_strategy@006b9620:1511,1557` gates on `leader_flags & 2`, `i !=
who` and the met bit, and on nothing about humans) and what §43's `human`
skip contradicts. Parks because it names no score *yet*: the AI reads its
own `active_wars`, not the human's, so what this would move is the human
leader's own production, and whether that lands on any word's frame is
unmeasured. The three fields are already pinned in `PARTS_ON_RUN115`, so
a successor has standing rows to move rather than a hypothesis.

(450) **The Merchant offer on Great Lakes 9380 is the original's and not
ours.** Item 442 matched both Scholar offers to the unit and the Citizen
exactly (`docs/AI.md` §53, `§52.2`'s table), leaving one row unaccounted:
the original offers `t61` Merchant at 869,565 in city 0 and this crate
offers nothing there. Parks rather than books because 442 moved the word
to **10161**, 781 frames past this frame — so the row names no score. It
becomes an item again only if the widening of the new word (448) reaches
back to a Merchant, or if a later word lands near 9380 again. The value
diff is on file and the city index is now confirmed by value rather than
assumed, so re-measuring it costs one run of 442's own test.

(371) **`market_speculation`'s two passes fire nowhere below the word.**
Read whole by item 362, both arms' predicates recorded in
`docs/journal/2026-09-18-item-362.md`. It was read because 362 was booked
as the market's and was not — `use_market` implements the decompile line
for line, and `MakeObject.num` was the cause. Parks because neither arm
fires below either map's word: it names no score, and the reading is
banked rather than lost.

(246) step 6's repath rests on run83's single event (239); (169)
`compute_site_stats`, 7,122 of run63's 27,000 site fields; (172) the
`bucket` pair on 5002/5061; (158) the human-leader sweep (AI §23.1); (159)
the `CITY` record's two seams; (146) `train_time`'s nine national arms;
(142) `World::tregion` (PATHFINDER §15–16); (122) 16 of 61 draws without a
`self.mark(`; (153) `TRIBE`; (20) the `Census` rows; (23) the formation
byte's sign, Echelon half (GROUPS §6.4), run52 the blocker; (103) a
woodcutter's clock, 445 v 480 (ORDERS §6.4); (105/45) Gaia's positions,
first bad 1658 (SYNC §4.2); (124) the loop flag is per animation file;
(116) the one `SITE` slot (AI §18); (166) `resource_cap` on five goods
(ECONOMY); (107) `epoch[0]`.

## Older backlog

(39) a 2D viewer over `Sim`; (41) `scenario.py`; a `find_target` block;
run7's orders; a mounted attacker; `calc_gather` non-flat;
`Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.
(306) **The `city` column is one high on every make row that carries one** —
ours 1/2 against theirs 0/1, on all 86 blocks of **both** windows, so an
off-by-one in whatever city index `make_me` is passed. Unrelated to any
value, which is what makes it separable from 305. AI §36.

(307) **`age_p`'s zero-age arm is unexercised** — no type in the shipped tree
reaches "a predecessor of age 0 leaves the walk looking" on either
window, so that half of 302's fix rests on the PE listing and not on a
diff. It is a reading-only claim in a document otherwise diff-backed,
which is exactly what the coverage sections exist to flag. AI §36.4,
§36.6. Falsified by a capture where a zero-age predecessor exists.

(459) **The building half of `ObjectData::visible` is read and not built** —
`Build::do_attack@006228f0:147` sets `visible |= 1 << target_who` after
`fire_ammo` and clears it on the same 32-frame slot a unit uses, and
`Build::do_missile_launch@00622670:70` writes `0xff`. So a tower that
shoots you becomes visible to you and a silo that fires becomes visible
to everyone, and `Wall::update_local_seen`'s third mask term is still a
stub here. VISION §9.1, §9.8. Names no score: no capture on disk has a
building shooting through fog. *Would settle it:* `BUILDS=7 UNITS=3`
over the frames a tower fires, and the `BUILDDATA` `visible` byte.

(460) closed 2026-09-21 by item 462: **falsified by its own
falsifier**, run exactly as written, and not one row moved — no
unit-target path in this crate reads `Profile::x_size` at all
(`docs/VISION.md` §9.8). The 622 destinations were
`find_nearby_target` walking the unit index where COMBAT §12.2
and §18.1 say it walks the cell's own `down` chain, plus the unit
half of `find_attack_pos` (COMBAT §32.2). The candidate was
cheap, precise and wrong, which is what a written falsifier is
for: it cost one run rather than an item.

