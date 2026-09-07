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

Neither is a mechanic and neither moves a word, which is why both park
rather than take a slot under a cap that stood at 18 of 18 the day they
were booked. Both are for a **Fable** pass — the model that steers, per
CLAUDE.md's fan-out rules — and 293 wants **lore** in the room with it.

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

(293) **Is the guard pipeline paying for itself, or taxing the loop?** —
Ramon's observation, 2026-09-07: the guards work, and they are still spammy
and reach further into a session than their value seems to want; the
suspected cause is the long-running suites. Two mechanical candidates worth
measuring before any redesign — `tools/guard.sh` is **four separate cargo
invocations**, docs_guard, no_float, the_handoff and one per filter, each
re-checking freshness and printing its own summary where one filtered run
might do; and `docs_guard` itself reads the entire paperwork surface, the
queue, every specification's sections and the decompile index, which is
likely what "reaches in" is naming. But the shape of the script is not the
question. The question is whether the guard-and-gate pipeline is improving
the loop or taxing it, and **lore is the oracle that can answer it across
sessions rather than by anecdote**: `lore tools` counts how often
`guard.sh` is actually invoked against the release gate, `lore trace` and
`lore usage` price the turns each costs, and `lore polls` already prices
the waiting shape that grows around long runs. On the table and none of it
prejudged: optimise the pipeline before deepening it, or fall back to a
single-runner mode, which Ramon doubts. **A cross-item constraint if it is
ever taken** — every worker runs `guard.sh`, so a change to it belongs in
briefs the day it starts, not the day it lands.

**Measured on the 289/290 pair, 2026-09-07, and it is not what this item
guessed.** Neither worker was in a red-gate loop. 289's release gate went
green on its **third** run — 245 tests, peak 11,218 MiB — and stayed green
for an hour; its two red runs were expected re-pins of numbers its own
change had moved, not a fix-and-retry cycle. The hour went to **one
unoptimised debug `cargo test`, still running past 45 minutes**, against
that release gate's five. What the debug run buys over the release gate is
**eleven `debug_assert!`s** — four in `rondata/src/load.rs`'s tree loaders,
seven across `sim` — and **no `#[cfg(debug_assertions)]` path at all**,
because `[profile.release]` already sets `overflow-checks = true`. So
`[profile.dev] opt-level = 2` would keep all eleven and cost the
incremental compile speed `tools/guard.sh`'s whole reflex is built on:
a real trade, and a steer's call rather than a worker's. **And the second
half is not the compile at all** — two lanes running six threads of diff
harness on one box is most of why that hour bought what five minutes buys
alone, so serialising the lanes' gates may be worth more than any profile
flag, and it bears directly on entry 34's width-two discretion. Reported by
loop-289 while waiting on the very run it was measuring.

**Corrected within the hour by loop-290, and the correction matters more
than the finding.** The contention was **three** unoptimised copies of the
heaviest suite, not two: loop-290 had launched the debug `cargo test`
**twice by mistake** and killed its duplicate once it ran `ps`. So the
sentence above must not be read as evidence against entry 34's width two —
a third of that load was one worker's own error, and an operator mistake
and a structural cost look identical from the outside. What survives the
correction is narrower and still worth the steer: even two honest copies of
a 45-minute debug run on one box is a poor way to spend a lane, and neither
worker could see the other's load without being asked. **What is not
established is whether width two alone, run cleanly, costs anything at
all** — no one has measured that, and this pair cannot answer it.

**And the sharpest form of it is loop-289's, because it names what the
contention actually broke.** Two lanes on one box do not merely halve each
other's throughput — they inflate every lane's **wall-clock**, and wall
clock is exactly what a commander reads to decide whether a lane is stuck.
289's hour looked like a red-gate loop from outside for precisely that
reason, and the commander went and asked both workers on that reading. So
the cost is not only throughput; it is that the loop's own health signal
degrades as width rises, which is a defect in entry 34's width discretion
rather than in any lane. Set against it: the opt-level trade is **bounded**
— it costs `guard.sh`'s incremental compiles and nothing else — while this
one is unbounded in the wrong direction.

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

(297) **Codex parity is a copy today and would rot** — a second agent's
setup was stood up in the main checkout on 2026-09-07 and deliberately NOT
committed: `AGENTS.md`, 26,352 bytes against `CLAUDE.md`'s 26,359 with
**three lines differing** — the rules table's own filename row, the memory
path, and "a Claude Code update resets the permissions". A hand-maintained
near-duplicate of the one file this project calls "the rules" is the drift
this repo exists to avoid, and no guard reads the copy, so it would go
stale in silence the first time `CLAUDE.md` changed — which is weekly.
Ramon's call, and the shape he named: **a symlink or a pointer file rather
than a copy, and the hook generated rather than transcribed.** Two defects
in the copy are the evidence that transcription had already gone wrong on
day one: the memory path read `~/.Codex/…` with a capital C, a
find-and-replace artifact rather than a real path; and `.codex/hooks.json`
invoked the poll guard through **`$CLAUDE_PROJECT_DIR`**, a variable the
other harness does not set, behind an `[ -f ] || exit 0` that would have
made the hook a silent no-op rather than an error. Parked, not booked: it
moves no word, and `docs_guard` would need to learn the pointer before the
symlink is safe.

## Measured residues, none near a word

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
