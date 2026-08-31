# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/JOURNAL.md` is the story.

This file **deletes**. A finished item leaves it for the journal, which
carries every number that has ever been here — items keep their numbers for
that reason, and new ones continue the count. `docs_guard.rs` fails the
build if this file strikes an entry instead of deleting it, passes 180
lines, or lets the handoff pass 32.

## Where things stand

*2026-08-31, after item 113.*

**Great Lakes' headline moved for the first time in three items.** Phase 3's
score is ticks before divergence: **Great Lakes 1375/791** of 1,772 (run10)
and **East Indies 1374/1373** of 1,850 (run39, unchanged). The words are
**1372** and 1373; Great Lakes' totals went 1471/1439 → 1467/1436, every
moved frame being at 1372 or later — past the word's own parting.

Item 113 was the AI's `1/1`, and it was two defects.
`compute_reg_territory` rebuilds its per-player border table every pass and
reads the **Civic** level from it; this crate cached the table at
`add_player`, so the AI held 261 cells against 290 — and `was_seen`'s first
arm is territorial, so those cells were dark to the site scorer and its
second city went up in the wrong place. Fixing that took the word *down* to
786, because it opened a path nothing had walked: `find_wpath`'s `scouting`
needs the **type** to be a scout (`role & 0x10`) as well as the order to be
`EXPLORE_TO`. With both, `1/1` never parts at all.

**Great Lakes' ticks reached its word; its orders did not.** Orders part at
792 on one field of one unit — `1/1`'s `coll_x/coll_y` — where ticks part at
1376 and the word at 1372. That is item 115, and it is the default.

**Opener (Opus):** `Continue — item 115: run10's frame 792, where `1/1`'s
`coll_x/coll_y` is `(40539, 18258)` in the original and `(40632, 18044)`
here. Same walk, a different collision point; it holds Great Lakes' orders
at 791 against ticks 1375.`

## The queue

In dependency order, headline-nearest first; the headline is the tick pair.
Both maps' **ticks** now sit on their own word; Great Lakes' orders are 581
frames short of it. Take the first unstarted unless a better order is
obvious — and say so. Numbers are stable; the journal indexed by them.

115. **run10's frame 792: one collision point, and it is the order score.**
    `1/1` walks the original's own path to its city site and on 792 the
    original writes `coll_x/coll_y` `(40539, 18258)` where this crate writes
    `(40632, 18044)` — the only thing between Great Lakes and its own word
    on both scores. COLLISION §6, ORDERS §4.1.

114. **Great Lakes' word at 1372, and a pasture being stocked.** The
    original spends twenty-five draws in `Farms::add_animals` — `+0x92`,
    `+0x134`, `+0x182`, five each — under `Build::activate+0x1c25`, plus
    five `Guy::init_real` under `Animal::init`: the five animals a farm gets
    on activation. This crate spends three, and nothing creates a pasture
    mid-run.

110. **East Indies' word at 1373, and a seventh ring.** The AI scout walks
    six rings on both sides; the original then leaves the loop for
    `Unit::think_scout+0x941` once and `+0xaba` five times — sites SCOUT
    §6/§7 have never named — where ours takes a seventh ring.

103. **The wood machine's record parts at 1,686, on a clock.** After 26,094
    agreeing fields the human's `0/2` holds a wait of 445 where the
    original's holds 480, tile and phase right. ORDERS §6.4.

105/45. **Gaia's positions, and the residue on both maps.** run39: 191,173
    of 192,504 agree, first bad **1381**; run33: 73,608 of 74,040, first bad
    **1420**. Both pinned; `Sim::reseat_animal` still corrects them where a
    dump carries clocks (SYNC §4.2).

87. **The widening ledger.** Items 74, 83, 69 and **113** were closed by
    fields the parser had and nothing compared. Make the backlog a number:
    per dumped record, the fields `rondata::diff` parses and never compares.

85/82. **run40's 360 disagreeing good-frames, in two halves.** (85)
    `Build::init` surveys a camp against its still-empty `gather_from`, so
    a harness camp activates with zero `gather_slots`; run40's *human* files
    one under good 2 (`get_good@0063bd50`'s table at `0063bd84`). (82) the
    original holds **0** in goods 3–5 and this crate **100** — inert while
    none is available, so a loader question. The orders audit's parked
    `FABLE:` R4 row travels here.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000` at
    the end of 202 or 203, yet its Scout's `mylos` moves 4 → 6; ours moves
    at 202 too, so what is owed is the *cache*.

89. **Three guards for the instrument itself.** (a) The handoff's numbers
    equal `rondata::diff`'s pinned floors. (b) Every `name@00xxxxxx` a
    document cites names a function in `INDEX.tsv`. (c) run33's floor pins
    totals mostly past its parting — a non-monotone score pinned as
    monotone, fallen three times (109, 69, 113).

72. **A document and its code disagreeing is a diff waiting to be run.**
    Items 68, 79, 80, 74, **36 twice over**, 107, 112's `wx'` and 113's
    `+0x2c8`. Every `name@00xxxxxx` and `+0xNN` a document pins, checked
    against its module, and every "halved"/"every frame"/"cleared" verb.

88. **The blind list is the ledger.** `report.py … blind docs/` lists the
    cited functions no traced run has entered (101 of 617). Print it, pin
    it as a floor, put it in each Coverage section.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through it too (COLLISION §3,
    §7, GOODY §1), and `down`/`down_who` go uncompared.

73. **`UnitData::group`, compared on no frame.** No unit holds the
    back-pointer, so `Group::normalize`'s cull (GROUPS §4.3) is unmodelled;
    run33's scout goes 65 → 64 on 96 where `get_open_slot` says 65.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries it on `(52, 22)` and this does not; `army`'s
    muster search reads it (ARMY §13). Find the *tile* mask's writer.

90. **The capture queue** is live — an independent session, DECISIONS 27's
    shape, running the owed 23, 96, 108 and 57 beside the loop.

96. **A turning vehicle or ship, which no capture has.** `guy_flags & 8` is
    set for 273 of 1,359 unit pieces and no traced guy's, so
    `Guy::do_turn@005d97a0:15`'s override is unfalsifiable (ANIM §4.6).
    *Capture, owed:* one turning in place.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group turned right round while marching and
    re-ordered in Refused or an Echelon (GROUPS §13); with it COLLISION §9.

116. **The one `SITE` slot still wrong, and the rule that is not it.** AI
    §18: a 5×5 slide keeps its centre where the original leaves it, so
    `blocked_town` refuses a cell `site_clear` allows. `blocked_site`'s
    fog-majority `0x24` (CITIES §11) was run once and moves **no** number,
    so the slide wants a second reading, not that rule.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels
    (`has_preq(TEMPLEBORDERS2..4)`, `FORTBORDERS2..4` — bonus types the
    loader reads as `bonus_preqs` and does not expose), the Colosseum, the
    Eiffel Tower, a gem rare, the handicap, and `was_seen`'s `reg_forts`
    arm; all inert until a capture has a Temple or a Fort. (b) AI §2.1's
    `check_explore` answers the whole region grid — `explored` is 900 here
    against the dump's 36 and 19 on every frame of both windows.

The rest of the road residue, both in ROADS §7.1: (57) the terraform after
`place_roads`; (58) the height loader's mean in `f32`, three tiles.

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8`, so it is the **Military** level, not ARMY's glossary's "current
    age". `army.rs:769,1365` read `tech[w].ages`, so every `age < 2/3/5`
    gate in the muster caps and `find_target` fires on the wrong counter.

108. **The goody box's pile, and the capture that would pin it.** GOODY §6:
    `epoch[3] × 25 + 25` is listing-only. *Capture, owed:* the run39 lobby
    under `samegame.py`, `LEADERS` per frame, ~900 frames; `bucket[2]`
    steps by 50 on 867.

91. **The final scenario, not a final dump.** One 24,000-frame game per
    map: the **trace** whole (`cover=1`, no window) and the dump in windows
    re-captured on demand. Due when a map's word matches its 1,850.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — run `zsh tools/fuzz/run.sh 424242 1000
1300` then `report.py … blind docs/` before deleting it; the `CITY` widening;
a `find_target` block; run7's order stream; a mounted attacker; a caravan;
`Leader::diplomacy`; `calc_gather` non-flat; (77) ANIM §3.2's piece rows.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`.
- **Start of session:** "Where things stand", the item, then its document.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both. **Never** quote this file or the journal
  into `CLAUDE.md`, a subagent brief, an agent definition, or a memory hook.
