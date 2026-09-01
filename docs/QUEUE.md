# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/JOURNAL.md` is the story.

This file **deletes**. A finished item leaves it for the journal, which
carries every number that has ever been here — items keep their numbers for
that reason, and new ones continue the count. `docs_guard.rs` fails the
build if this file strikes an entry instead of deleting it, passes 200
lines, or lets the handoff pass 32.

## Where things stand

*2026-09-01, Opus — the gull flew, and East Indies' word went **3579 →
3608**.* Item 133 closed on two defects that were this crate's rather than
the reading's, and the next seam is a different mechanic.

- **The roll had no `mark` and the gull had no `type_index`.**
  `dock_open` had spawned the gull and spent both of `Dock::init`'s draws
  in the right order since 2026-08-25; the heading roll carried no site, so
  the sequence read a second `Guy::init_real+0x52` (item 122's shape, found
  by the score), and the default −1 `type_index` meant `Guy::set_anim`
  never recognised one of its three gaia bird types. TRANSPORT §5.2.1.
- **A gull reaches `do_air_physics` by `do_strafe`, not `do_air_patrol`.**
  `think_bird`'s `0x194` arm draws nothing, so all it spends is the tail's
  `set_anim(CHAR_WALK, 0, 1)` — once, at birth, on 3580 — then the wing
  beat. Three counts over run54 back it: 12 birth coins, 3
  `do_air_physics+0x639` draws all a wild bird's, `Region::coast_here` off
  the blind list. And `gull_o`, the `DOCK` record's unasserted field, is
  run22's **15** here too.
- **One assertion was luck and is rescoped.** run57's "zero wrong building
  fields over 4,000 frames" held past the parting by chance; `1/2012` on
  3977 moved one tile the instant the word did. The building half is now
  asserted up to the word and printed past it, and the collision total is a
  floor — it rose 330,643 → 337,265, units that ever part 14 → 11.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1802
Long captures: EastIndies w3608 of 24,000 · GreatLakes w1802 of 24,000

**Opener (Opus):** `The headline is East Indies' word at 3608 and item 134
is what holds it: SpellType::cast_transport's first fire, three draws this
crate does not spend. docs/TRANSPORT.md §6 has the mechanic and §12 has
had the capture booked since it was written.`

## The queue

In dependency order, headline-nearest first; **the headline is now the long
captures' word**, and East Indies leads it. Take the first unstarted unless a
better order is obvious — and say so. Numbers are stable; the journal is
indexed by them.

134. **A unit casts its own transport at 3608, and this crate does not.**
    run54's frame 3608 is `SpellType::cast_transport@00670db0`'s first,
    and the three draws are two `Guy::set_anim+0x97a < Unit::set_anim <
    Unit::do_cast+0xc89` and the cast unit's `Guy::init_real+0x52 <
    Unit::init+0xb97 < Objects::init_unit+0xbd`. It is the dock's shadow:
    the level granted at 3579 is what lets a unit board.
    `docs/TRANSPORT.md` §6 is the mechanic, §12's check 4 is the capture
    it wants — a `UNITS=3` window over 3600–3640 of this same game — and
    `Unit::do_cast@005ebfe0`, `SpellType::cast@00676ce0`,
    `pay_cast_costs@00676c40` and `SpellTypeData::get_job_time@00675800`
    all first execute on that frame, so none of them is blind any more.

82. **run40's human files one slot under good 2.** `get_good@0063bd50`'s
    table at `0063bd84` cannot produce it;
    `Leader::plan_strategy@006b9620` line 1137 assigns the whole array from
    `City::count_gather_slots` and raises the high-water to match, and that
    second writer is unread. Beside it: the original holds **0** in goods
    3–5 where this crate holds **100** — inert while none is available, so
    a loader question. The run40 diff asserts both as they stand.

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
    191,876 of 192,504, first bad **1658**; run33 is exact at 74,040.
    `Sim::reseat_animal` still corrects them where a dump carries clocks
    (SYNC §4.2).

122. **A draw with no mark of its own.** `mark_sites` attributes an
    unmarked draw to the label still standing, so the sequence reads wrong
    rather than short (item 110's day went to one, item 133's frame 3579 to
    another). **16 of the 61** `rng.roll()`/`rng.get()` calls in
    `crates/sim` have no `self.mark(` in the six lines above them — some
    are marked by their caller, and the point is that nothing says which.
    Mark or explain each, then make the list a guard.

124. **The loop flag is per animation file** (ANIM §3.3): by slot it is
    not a constant — 42 `<UNIT>` entries give `CHAR_DUMP_WOOD` a
    non-looping file and 38 a looping one. Carry it beside
    `Art::piece_lengths`.

87. **The widening ledger.** Items 74, 83, 69, 113 and 123 were closed or
    sharpened by fields the parser had and nothing compared. Make the
    backlog a number: per dumped record, the fields `rondata::diff` parses
    and never compares. run42 adds the per-frame `LEADERDATA` to that pool.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000` at
    the end of 202 or 203, yet its Scout's `mylos` moves 4 → 6; ours moves
    at 202 too, so what is owed is the *cache*.

89. **The instrument's last guard.** (a) and (b) are built (2026-08-31);
    what is left is (c): run33's floor pins totals mostly past its
    parting — fallen three times (109, 69, 113).

72. **A document and its code disagreeing is a diff waiting to be run.**
    The `name@00xxxxxx` half is a guard now (`docs_guard`, 2026-08-31).
    Left: every `+0xNN` a document pins, checked against its module, and
    every "halved"/"every frame"/"cleared" verb.

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

23. **The formation byte's sign — the Echelon half.** `reverse`'s
    *displacement* is read only on GROUPS §6.4's Echelon rows and every
    captured group is a Line. run52 names the blocker — the keystroke
    arrived as the chat key; ORACLE runs 51/52 name the next steps (bind a
    plain letter, `osascript keystroke`, confirm by tooltip, then
    `FORM_E_RIGHT` and `form 3` in the dump). With it COLLISION §9.

116. **The one `SITE` slot still wrong, and the rule that is not it.** AI
    §18: a 5×5 slide keeps its centre where the original leaves it, so
    `blocked_town` refuses a cell `site_clear` allows. `blocked_site`'s
    fog-majority `0x24` (CITIES §11) was run once and moves **no** number,
    so the slide wants a second reading, not that rule.

117. **Two seams the census windows now measure.** (a) ATTRITION,
    "Territory": the temple and fort border levels, the Colosseum, the
    Eiffel Tower, a gem rare, the handicap, and `was_seen`'s `reg_forts`
    arm — all inert until a capture has a Temple or a Fort; the bonus-preq
    detail is in that section. (b) AI §2.1's `check_explore` answers the
    whole region grid — `explored` is 900 here against the dump's 36 and
    19 on every frame of both windows.

The road residue, both in ROADS §7.1: (57) the node counts — 1,046 v
1,043 and 1,460 v 1,870 — re-read on run43's own before-grid (ORACLE
run43); (58) the height loader's mean in `f32`, three tiles.

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
- **A floor that moves** moves three things together: `FLOORS` in
  `rondata::diff`, the assert that reads it, and the `Scoreboard:` line.
- **Start of session:** "Where things stand", the item, then its document.
- **Run the diff suite with `--release`.** run53's 24,000 frames tripled
  `cargo test -p rondata` in debug — 114 s to 285 s — where `--release` is
  92 s including the build. The five-minute wait is the capture, not a hang.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both. **Never** quote this file or the journal
  into `CLAUDE.md`, a subagent brief, an agent definition, or a memory hook.
