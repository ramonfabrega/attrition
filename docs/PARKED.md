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

## Loop — the steering pass's, never a worker's

Tooling, guards and the queue's own rules. The Fable pass takes these
(`CLAUDE.md`, "Fan-out rules"); a commander never spawns one. Item 279's
two ledger regexes closed in the third pass, 2026-09-17; the fourth pass,
2026-09-18, ruled six (`docs/audit/2026-09-18-fable-pass-4.md`).

(251) **`memcap.sh` has two doors left and half a fixture**: the refused
sample that read as zero is closed (exit 125, lab L25) and the mode-644
door closed 09-07. Left: it takes a **GiB cap as its first argument**; and
(281) it read `ps rss`, which over-counted the mapping 280 removed — ~1.3 GB
of 260's 8,816 was run58's clean text — and a 2 s poll under-reports a
sawtooth. Still wants the fixture with teeth: past the cap, dead in N
seconds, exit 137.

(335) **The commit trailer names the model the commander expected, not
the one that ran.** This repo reads the `Co-Authored-By` trailer to answer
"who wrote this tranche", while `CLAUDE.md`'s fan-out rules say to verify
the model **from the transcript, never from the spawn parameter**. Both of
the trailer's feeders are the forbidden source:

- **The commander dictates it in the brief** — eight of twelve transcript
  hits are worker prompts reading `Commit on loop-NNN with the trailer
  "…Claude Opus 5…"` (loops 219, 227, 228, 234, 235, 236, 7448,
  att-capture), written before the worker has made one request.
- **The harness's attribution reminder is not the session's model** —
  measured 2026-09-17, it said `Claude Fable 5.1` on a `claude-opus-5`
  session, and the day's first commit went out wrong.

**No realised wrong trailer is known here**: loop-329 ran `claude-opus-5`
over 201 requests and its `4fb593b` says Opus. Latent, and it bites the
first time a commander is wrong about what it spawned. Two smaller shapes
ride along: **one model, two strings** (356 commits `Claude Opus 5 (1M
context)` against 123 `Claude Opus 5`) and **149 of 772 commits untrailered**,
46 of them merges. A guard can compare a commit's trailer against what
served the session that wrote it — the first-request source `lore spawns`
already trusts. Guard, hook, or "a brief never dictates it": the pass's.

(341) **The click-free lane cannot take a capture worth having, so every
scored capture must drive the cursor.** Ramon's flag, 2026-09-18: "i
thought we had figured out occluded/non main focus captures". We had, for
one job and not this one. This rests on the tool's source, not on any run.

`unattended_capture.py` needs no TCC grant and no human at the menu, but
the window and detail live in `live_session.stage()` and are **hardwired**:
`rise2.ini` gets `LogStartFrame 18` / `LogEndFrame 36`, and every
`gamelog.ini` category is forced to `0` except `[End Frame] UNITS=3` and
`[Misc Logging] COMMANDMANAGER=1`. Right for startup evidence and autostart
receipts; useless for the frontier, which needs a **late window** (run96
`[23960, 24000)`, run97 `[8030, 9350)`) and **rich categories** (run80 and
run96's `BUILDS=7, CITIES=5, GUYS=2, LEADERS=9, DEATHS=1`).

So every scored capture goes down the **clicked** lane: three macOS
permissions, a driven cursor, correctness contingent on being frontmost.
That is a coupling — the lane can only run on an unattended box — not a bug.
**The keys already exist**: `stage()` writes all of them, so the work is
exposing them rather than pinning them, plus a caller-supplied category set.

**A separate lesson for the runbook**: losing focus mid-run fails in a shape
that **mimics a revoked permission** — a hang with the game alive at 68 %
CPU and zero FRAME records, then a `perm_probe` miss two pixels wide while
TCC showed no denial. **Check what is frontmost before blaming TCC.**
Contention with the machine's owner is the normal condition, not an anomaly.

(343) **The game plays sound at whoever is near the machine**, and nothing
turns it off. Ramon's flag the same day. The settings are three tags in
`PlayerProfile/Player.dat` — `<MUSIC_VOL value="128"/>`, `<SOUND_VOL
value="192"/>`, `<TAUNT_VOL value="192"/>` — the same file and the same
regex-substitution mechanism `unattended_capture.py`'s `set_map()` already
uses for `<MAP_STYLE>`, already covered by `live_session`'s settings backup
and restore. A few lines in a place the tool already edits, with the
restore already written, and it makes an unattended run a good neighbour.

(356) **Nothing checks that a documented claim reached the code.** Item
352, 2026-09-18: `BuildType::mask_me@006312a0`'s first statement —
`cells[…].flags |= 0x4000` on the mark, `&= 0xbfff` on the unmark — had
been in `docs/CITIES.md` §3.6 **in prose since that mechanic's first
reading**, and `crates/sim` never wrote the bit. Without it the muster
ring's score cannot separate two open cells, so the earliest tie won and
Great Lakes parted 221 frames early. run34's start block confirms the
original from the other side: thirteen cells carry the bit and each is a
starting building's.

This is the method's own seam. A document is written from the original and
the code is implemented from the document, and `docs_guard` checks that a
cited **address** exists and that a **section** does not grow — never that
a claim was built. So a correct reading can sit in a specification for
weeks doing nothing, and the diff only catches it when some frame happens
to depend on it.

A tractable shape exists and 352 used it by hand: **a hex constant or flag
named in `docs/` that appears nowhere under `crates/sim/`** is greppable,
and would have found this one. It will be noisy — a constant can be named
for context, or built under another spelling — so guard, report or
author's checklist is the pass's call. The gap is real and cost frames.

(321) **A document that cites a dead-listed function by address cites
code the game does not run.** `LeaderData::is_human@006ec170` is on
EMULATOR.md §4's dead list — no call, no jmp, no embedded address — so
audit V14's `leaders.list[who] & 4` verdict was read from a standalone
body the executable never executes; the verdict itself survives on
evidence (run58's dump has the AI carrying `& 4` and the human not), but
the citation does not. Found by item 320 on 2026-09-17 while in
EMULATOR.md for another reason. `docs_guard` already checks every
function address a specification cites against the decompile export's
index — extending that check to the **dead list** is the guard shape, and
it would name every other document in this position at once. The steering
pass's: a guard, and then whatever it finds.

(313) **The landing chain wants one verb.** Merge, gate, push and reap are
one chain by rule since 09-17; ccc has `merge`, `update` and `clear`, and
the reap is a separate command a commander typed after the chain twice and
forgot twice. Filed with ccc as `land <ref>` or `merge --reap`; until it
exists, the chain is one shell line in the commander's brief.

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
what hid 252's dumps. Uncounted: (88) the blind list, 101 of 617; (72)
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

Neither is a mechanic and neither moves a word, which is why both park
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
306. **The `city` column is one high on every make row that carries one** —
    ours 1/2 against theirs 0/1, on all 86 blocks of **both** windows, so an
    off-by-one in whatever city index `make_me` is passed. Unrelated to any
    value, which is what makes it separable from 305. AI §36.

307. **`age_p`'s zero-age arm is unexercised** — no type in the shipped tree
    reaches "a predecessor of age 0 leaves the walk looking" on either
    window, so that half of 302's fix rests on the PE listing and not on a
    diff. It is a reading-only claim in a document otherwise diff-backed,
    which is exactly what the coverage sections exist to flag. AI §36.4,
    §36.6. Falsified by a capture where a zero-age predecessor exists.

