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

*2026-08-31, after item 104 (Opus).*

**East Indies' word and sequence moved 867 → 879**; Great Lakes holds at
780 with its totals unchanged at 986/884, its two boxes being at 898 and
1659, past its parting. run10 is untouched in every figure — 572/776, first
divergences 802 and 573, all fourteen by-unit partings, both coverage
totals. run39's game score holds at 167/167 with 217/168, its queue record
at 33,631 fields and its gather record at 16,152 to 897. Tree green: 661
sim, 154 rondata.

**Frame 867 was a goody box, and a box is a lottery.**
`Unit::explore_goody@005f9780` refuses knowledge by name and anything
`type_avail` does not call available, then spends **one draw per survivor**
— three in the Ancient age, which is what every capture that reaches a box
spends. The pile is `epoch[3] × GOODY_BOX_AGE + GOODY_BOX`, and `epoch[3]`
is the **Science** level, not the age. GOODY.md; VISION §7 is struck.

**One number fell**: run39's gaia agreement 190,417 → 189,843 of 192,504,
first parting frame 983 unmoved. Past the word's parting the stream is
nobody's, so moving the word re-rolls every later coin; the test pins 983.

**Opener (Opus):** `take item 106 from @docs/QUEUE.md — East Indies' word
at 879 is the scout going idle and re-thinking: two Unit::set_anim stands
then Unit::think_scout+0x436/+0x458 six times over and +0x64c twice.
docs/SCOUT.md §11 is the region fallback; diff it against run39.`

## The queue

In dependency order, headline-nearest first — the headline is the lower
map. Take the first unstarted one unless a different order is obviously
better, and say so. Numbers are stable; the journal is indexed by them.

106. **East Indies' word at 879, and the scout re-thinks.** Twenty-one
    draws against five. The scout that took the goody goes idle
    (`Guy::set_anim+0x97a < Unit::set_anim+0x56` and `+0xb6` under
    `Unit::do_idle+0x7d`), then `think_scout` spends `+0x436`/`+0x458`
    **six times over** and `+0x64c` twice. `docs/SCOUT.md` §11 is the
    region fallback; run33's re-targets at 361 and 482 are neighbours.

103. **The wood machine's record parts at 897, on a clock.** After 16,152
    agreeing fields the human's `1/2` holds a wait of 499 where the
    original's holds 509 — ten frames, tile and phase right. ORDERS §6.4.

69. **East Indies' order-list length at 168.** `1/4` and the original
    disagree on the order list at 168 and `1/5` at 186 — both **ahead** of
    the word, so they are seen while the two streams still agree.

84. **Great Lakes' word at 780, and a seventh farm.** Ours spends two
    `Unit::do_job+0x67` draws the original does not — a crop re-target,
    since run33's AI built no pasture — and from **781** on ours ticks
    **seven** `Farms::inc_time` chances where the original ticks six. The
    extra farm is the half to chase.

105. **Gaia's positions, and the 2,661 that are not.** 189,843 of run39's
    192,504 dumped animal-frames agree; the first that does not is **983**,
    `8/3` wandering off the point it was refused at. Great Lakes is
    uncompared.

87. **The widening ledger.** Items 74 and 83 were closed by comparing
    dumped fields nobody had. Make the backlog a number: `rondata::diff`
    prints, per dumped record, the fields it parses and does not compare.

85/82. **run40's 360 disagreeing good-frames, in two halves.** (85)
    `Build::init` surveys a camp against its still-empty `gather_from`, so
    a harness camp activates with zero `gather_slots`; and run40's *human*
    files one under good 2, where `get_good@0063bd50`'s table at `0063bd84`
    is Farm 0, Camp 1, Mine 4, University 3, Oil 5. (82) the original holds
    **0** in goods 3, 4 and 5 for both players and this crate **100** —
    inert while none is available, so a loader question.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000` at
    the end of 202 or 203, yet its Scout's `mylos` moves 4 → 6; ours moves
    at 202 too, so what is owed is the *cache*.

89. **Two guards for the instrument itself.** (a) The handoff's numbers
    equal `rondata::diff`'s pinned floors — one sat two items stale. (b)
    Every `name@00xxxxxx` a document cites names a function in `INDEX.tsv`.

72. **A document and its code disagreeing is a diff waiting to be run.**
    Items 68, 79, 80, 74, **36 twice over** and now 107. Every
    `name@00xxxxxx` and `+0xNN` a document pins, checked against the module
    implementing it, and every "halved"/"every frame"/"cleared" verb.

88. **The blind list is the ledger.** `report.py … blind docs/` lists the
    cited functions no traced run has entered (101 of 617) — every
    reading-only claim here. Print it, pin it as a floor, put it in each
    Coverage section.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through the same list (COLLISION
    §3, §7, GOODY §1), and the dump's `down`/`down_who` go uncompared.

73. **`UnitData::group`, compared on no frame.** No unit holds the
    back-pointer, so `Group::normalize`'s cull (GROUPS §4.3) is unmodelled.
    The pool slot is the hard half: run33's scout goes 65 → 64 on 96 where
    `get_open_slot` (§3.1) predicts 65.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries it on `(52, 22)` and this does not; `army`'s
    muster search reads it (ARMY §13). Find the writer — not
    `World::set_building_at`, which writes the *tile* mask.

90. **The capture queue.** A capture is fourteen unattended minutes — the
    screen for thirty seconds of lobby, then nothing — so it runs *beside*
    a session. A scenario file (`longtrace.sh`'s inputs) and a script
    running the lines back to back; 23, 96, 108 and 57 first.

96. **A turning vehicle or ship, which no capture has.** `guy_flags & 8`
    is set for 273 of the install's 1,359 unit pieces and none of the eight
    any traced guy carries, so `Guy::do_turn@005d97a0:15`'s turn override
    of the standing walk is unfalsifiable (ANIM §4.6). *Capture, owed:* a
    unit whose piece has one, turning in place, with `UNITS=3`.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group turned right round while marching and
    re-ordered in Refused or an Echelon (GROUPS §13); with it COLLISION §9,
    two units ordered head-on.

45. **Gaia's animals.** `Sim::reseat_animal` puts them back from every
    traced dump (SYNC §4.2), so `run_traced` reported **no** divergence for
    `8/2` on the ten frames item 95 had wrong. `ANIMALDATA`: 70,960
    uncompared.

The rest of the road residue, both in ROADS §7.1: (57) the terraform,
`terraform_for_building@00875210` after `place_roads`, run13's `FRAME 100`
against run32's 104; (58) the height loader's mean in `f32`, three tiles.

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    ARMY's glossary calls `epoch[0]` "the leader's current age";
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8`, so it is the **Military** level. `army.rs:769,1365` read
    `tech[w].ages`, so every `age < 2/3/5` gate in the muster caps and
    `find_target` fires on the wrong counter. Item 72's family, site named.

108. **The goody box's pile, and the capture that would pin it.**
    GOODY §6: `epoch[3] × 25 + 25` is listing-only, because the `LEADERS`
    detail that writes `bucket` and `epoch_get(scan)` is emitted once, at
    start. *Capture, owed:* the run39 lobby under `samegame.py`, `LEADERS`
    per frame, ~900 frames. `bucket[2]` steps by 50 on 867 and
    `epoch_get(scan)` reads `0 0 0 1`; `ages` would pay 25.

91. **The final scenario, not a final dump.** One 24,000-frame game per
    map: the **trace** whole (`cover=1`, no window — cheap) and the dump in
    windows re-captured *on demand*, since the game is reproducible
    (`samegame.py`). A full dump is 24,000 × 2.4 s. Due when a map's word
    matches its 1,850.

Older backlog: (39) a read-only 2D viewer over `Sim` state with the dump
overlaid; (41) `scenario.py`'s fate — run `zsh tools/fuzz/run.sh 424242
1000 1300` then `report.py … blind docs/` before deleting it; the `CITY`
widening; a `find_target` block; run7's order stream under the trace; a
mounted attacker; a caravan; `Leader::diplomacy`; `calc_gather` non-flat;
and (77) ANIM §3.2's `AGE3`/`AGE5` piece rows.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`.
- **Start of session:** "Where things stand", the item, then its document.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both.
- **Never** quote this file or the journal into `CLAUDE.md`, a subagent
  brief, an agent definition, or a memory hook.