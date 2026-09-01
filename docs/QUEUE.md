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

*2026-08-31, after item 121 (Opus).*

**East Indies' word is 1647**, up from 1570; its ticks and orders are
unchanged at 1477/1476 and its gaia positions 191,876 of 192,504 with the
first parting at 1658. Great Lakes is untouched — word **1802**, ticks and
orders **1772/1772 of 1,772**, gaia 74,040 of 74,040.

Item 121 was two frames of one citizen's wood dump, hiding each other.
`AnimMgr::force_load` drops a **non-looping** animation's last key, so
`CHAR_DUMP_WOOD` is 32 and not 33 — which every `GUY` block prints and no
dump the lengths guard read had been asked. And `Unit::set_angle`'s third
argument is a **snap flag** every order passes as zero, so `Guy::move`'s
turn arm overwrites the animation an order sets on the frame it turns.
ANIM §3.3, §4.7.

**Each map's next number is still one mechanic.** Great Lakes: 120. East
Indies: 123, and it leads — its headline is the only one still moving.

**Opener (Opus):** `Continue — item 123: East Indies' word at 1647. Ours
nineteen draws against theirs four: two `Guy::set_anim+0x97a <
Unit::do_idle+0x7d` and a whole AI scout pass the original does not run on
that frame.`

## The queue

In dependency order, headline-nearest first; the headline is the tick pair.
East Indies is the only map whose headline can still move, so its word item
leads. Take the first unstarted unless a better order is obvious — and say
so. Numbers are stable; the journal indexed by them.

123. **East Indies' word at 1647, and a scout that thinks a frame early.**
    Ours nineteen draws against theirs four: the original spends nothing
    on 1647 but its four `Farms::inc_time+0x1ae`, this crate two
    `Guy::set_anim+0x97a < Unit::do_idle+0x7d` and a whole scout pass —
    `+0x436`/`+0x458` six times, one `+0x64c`. 1644–1646 and 1648–1650
    agree draw for draw, so it is the *cadence*: SCOUT §11, AI §2.

120. **Great Lakes' word at 1802, and a bird's flight physics.** One draw
    at `Unit::do_air_physics+0x639 < Unit::do_air_patrol+0xf3 <
    Unit::do_job+0xd7`, reached **once** in run33's 1,851 frames and still
    the boundary over run53's 24,000. It is the last thing between this map
    and a word past its own capture. `docs/SYNC.md` §3.9 has the bird;
    `do_air_physics` is named there as unread and is the one arm of it no
    run has forced.

103. **The wood machine's record parts at 1,686, on a clock.** After 26,094
    agreeing fields the human's `0/2` holds a wait of 445 where the
    original's holds 480, tile and phase right. ORDERS §6.4.

105/45. **Gaia's positions — East Indies' half is what is left.** run39:
    191,173 of 192,504 agree, first bad **1381**. run33 is **74,040 of
    74,040 with no first bad frame** since 114 and is pinned exactly.
    `Sim::reseat_animal` still corrects them where a dump carries clocks
    (SYNC §4.2).

122. **A draw with no mark of its own.** `mark_sites` attributes an
    unmarked draw to the label still standing, so the sequence reads wrong
    rather than short (item 110's day went to one). Grep `rng.roll()` in
    `crates/sim` for calls with no `mark` above them; mark or explain each.

124. **The loop flag is per animation file; `anim::non_looping` is a
    rule.** `rondata::artdata` already reads it (ANIM §3.3), and by slot
    it is not a constant — 42 `<UNIT>` entries give `CHAR_DUMP_WOOD` a
    non-looping file and 38 a looping one. Carry it beside
    `Art::piece_lengths`. Moves no number until a capture disagrees.

87. **The widening ledger.** Items 74, 83, 69 and **113** were closed by
    fields the parser had and nothing compared. Make the backlog a number:
    per dumped record, the fields `rondata::diff` parses and never compares.
    run42 adds the per-frame `LEADERDATA` — buckets, epochs — to that pool.

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

23. **The formation byte's sign — the Echelon half.** runs 45/46 fired the
    XOR term and `kill_current_order`'s formula holds (ORACLE); `reverse`'s
    *displacement* is read only on GROUPS §6.4's Echelon rows — Refused has
    no reverse term — and every captured group is a Line. The profile route
    works — run51's tooltip says F9, so `KeyMap::load@007d39a0` runs and
    `bindkey.py`'s element is right — and run52 names the real blocker: the
    keystroke arrived as the chat key, which is all run50's "failure" was.
    Next, in order (ORACLE runs 51/52): bind a plain letter, send it with
    `osascript keystroke`; the tooltip names it; it toggles Auto Explore;
    only then `FORM_E_RIGHT`, and `form 3` in the dump. With it
    COLLISION §9.

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

The rest of the road residue, both in ROADS §7.1: (57) the two node counts
— 1,046 v 1,043 and 1,460 v 1,870 — re-read on run43's own before-grid,
its capture half landed (ORACLE run43); (58) the height loader's mean in
`f32`, three tiles.

107. **`epoch[0]` is the Military level, and `army.rs` reads `ages`.**
    `get_epoch_base(0)` is `BASE_MILITARYTYPES` and the army family reads
    `+0xe8`, so it is the **Military** level, not ARMY's glossary's "current
    age". `army.rs:769,1365` read `tech[w].ages`, so every `age < 2/3/5`
    gate in the muster caps and `find_target` fires on the wrong counter.

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
- **Run the diff suite with `--release`.** run53's 24,000 frames tripled
  `cargo test -p rondata` in debug — 114 s to 285 s — where `--release` is
  92 s including the build. The five-minute wait is the capture, not a hang.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both. **Never** quote this file or the journal
  into `CLAUDE.md`, a subagent brief, an agent definition, or a memory hook.
