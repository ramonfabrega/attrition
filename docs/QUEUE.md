# The queue

Where the work stands and what comes next — the file a fresh session reads
first, and the one a subagent never sees. `CLAUDE.md` is the rules; this is
the state; `docs/JOURNAL.md` is the story.

This file **deletes**. A finished item leaves it for the journal, whose
"Lifted from the queue" section and dated entries carry every number that
has ever been here — items keep their numbers for that reason, and new ones
continue the count. `crates/sim/src/docs_guard.rs` fails the build if this
file strikes an entry instead of deleting it, passes 180 lines, or lets the
handoff pass 32.

## Where things stand

*2026-08-30, after items 93, 94 and 37 (Opus).*

**The headline is the pair, lower map first.** East Indies: run39's
**word parts at 19**, window **62/55** of its first 64 frames. Great
Lakes: run10 ticks **572**, orders 776; run33's word at **780**, totals
951/838 of 1,850. None of the four moved. What did move is the angle
sub-score: run10's disagreements **9,156 of 33,992 → 8,969 of 35,868**,
with item 37's three rows gone. Tree green: 639 sim, 145 rondata.

**Items 93 and 94 are read, measured, and not landed.** Together they
take run39 to **64 of 64 draw for draw and the word to 69**; either alone
is worse than neither. `docs/SYNC.md` §3.11's last section has both, the
listing that settles each, and the numbers. What blocks them is the
*other* map — see item 36, which is now the headline's dependency and not
a residue.

**What landed is the third of the three**, and it was item 37:
`Guy::move` zeroes `last_speed` before the turn at the foot of the same
branch, so a standing foot or mounted body swallows its whole owed turn
in one frame. `docs/MOVEMENT.md`, "The body step" — whose pseudocode had
the order right since 2026-08-27 while the code read the stale value.
Item 72's fourth kind of bug, found by reading the document beside the
listing.

**Opener (Opus):** `take item 36 from @docs/QUEUE.md — the farmers'
angles are what blocks the headline now; docs/SYNC.md 3.11's last
section says what 36 unblocks and names guy_flags & 8 as the suspect.`

## The queue

In dependency order, headline-nearest first — the headline is the lower
map. Take the first unstarted one unless a different order is obviously
better, and say so. Numbers are stable; the journal is indexed by them.

36. **`Unit::set_angle`'s seventeen other callers — and it is the
    headline's dependency now.** Most of run10's angle rows where the
    positions agree; the farmers are all of it — `0/3`–`0/5`, `1/3`–`1/5`,
    `1/8` (GROUPS §4.1), 8,866 of 8,969. Item 93's arm reads
    `des_angle != angle` on **every** standing unit, so it cannot land
    until these are the original's. The other half of the same block is
    `Guy::do_turn@005d97a0:15`: `guy_flags & 8` — the guy's piece has a
    turn animation (`Guy::init_real@005db6b0:179`) — overrides the arm's
    walk with `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`, unmodelled here, and a
    guy on a turn animation spends no arrival draw where one on the walk
    does.

93. **The arrival pair: `Guy::move`'s turn arm, and `Unit::init`'s
    snap.** Read, measured, unlanded — **together** they take run39 to
    64/64 and the word 19 → 69; the arm alone gives 58/57, the snap alone
    20 and 60/53. Both are written and diffed in SYNC §3.11's last
    section, with the listing that settles each; the code is a dozen lines
    (`anim.rs`'s `guys_follow`, `farms.rs`'s `farm_add_animals`).
    **Blocked on 36**, which costs run33's word 780 → 584 and run10's
    orders 776 → 586 if this lands first.

69. **East Indies' order-list length at 168.** `1/4` holds two orders on
    the original's 168 where this holds one; `1/5` at 186 and `1/3` at
    202 the same. **Downstream of 93** — not booked until the word moves.

84. **The word at 780, four frames past the city.** Eleven draws against
    eight; ours opens `GameAccess::rnd+0x20 < Unit::do_job+0x67` where the
    original has `Unit::do_non_flat_gather+0x54b` — a citizen on a job.

87. **The widening ledger.** Items 74 and 83 were closed by comparing
    dumped fields nobody had compared. Make the backlog a number:
    `rondata::diff` prints, per dumped record, the fields it parses and
    does not compare, pinned as a floor that may only fall.

85. **`gather_slots`, and a slot `get_good` cannot produce.** 120 of the
    run40 diff's 480 good-frames. `Build::init` surveys a camp against its
    still-empty `gather_from`, so a harness camp activates with zero; and
    run40's *human* files one under good 2, where `get_good@0063bd50`'s
    table at `0063bd84` is Farm 0, Camp 1, Mine 4, University 3, Oil 5.

86. **The science discount's purchase side.** `Sim::tech_price` passes
    `Modifiers::default()`, so a tech is charged undiscounted, and
    `Build::refund_cost` inverts an expression the buy side never applies.
    Zero on every traced purchase. `calc_science_discount@006da630`.

82. **A hundred knowledge, oil and wealth nobody gave anybody.** run40:
    the original holds **0** in goods 3, 4 and 5 for both players and this
    crate **100** — inert while none is available, so a loader question.

35. **`mylos` as a cache.** VISION §7: player 1 carries no `0x4000000`
    at the end of frame 202 or 203, yet its Scout's `mylos` moves 4 → 6.
    Ours moves at 202 too, so what is owed is the *cache*.

89. **Two guards for the instrument itself.** (a) The handoff's headline
    numbers equal `rondata::diff`'s pinned floors — one sat two items
    stale. (b) Every `name@00xxxxxx` a document cites names that function
    in `INDEX.tsv`. Both made to fail first.

72. **A document and its code disagreeing is a diff waiting to be run.**
    Items 68, 79, 80 and 74 — four in a row. Every `name@00xxxxxx` and
    `+0xNN` a document pins, checked against the module implementing it,
    and every "halved"/"every frame"/"cleared" verb about a field here.

88. **The blind list is the ledger.** `report.py … blind docs/` lists the
    cited functions no traced run has entered (101 of 617); intersected
    with `crates/sim`, that is every reading-only claim here. Print it,
    pin its size as a floor, put it in each Coverage section.

48. **The object chain, whole.** `collide.rs` chains units only; the
    original threads buildings and goodies through the same list
    (COLLISION §3, §7), and the dump's `down`/`down_who` go uncompared.

73. **`UnitData::group`, compared on no frame.** No unit here holds the
    back-pointer, so `Group::normalize`'s cull (GROUPS §4.3) and its six
    writers of `−1` are unmodelled. The pool slot is the hard half: run33's
    scout goes 65 → 64 on 96 where `get_open_slot` (§3.1) predicts 65.

56. **The cell's `BUILDING` bit, which nothing here sets.** run13's
    frame-95 world carries `cell::BUILDING` on `(52, 22)` and this does
    not; `army`'s muster search reads it (ARMY §13). Find the writer —
    not `World::set_building_at`, which writes the *tile* mask.

90. **The capture queue.** A capture is fourteen unattended minutes — the
    screen for thirty seconds of lobby, then nothing a session needs — so
    it runs *beside* a session. A scenario file (`longtrace.sh`'s inputs)
    and a script running the lines back to back; 23, 45 and 57 first.

23. **The hand-back's inversion, and the formation byte's sign.**
    `kill_current_order` writes `order.facing XOR reversing(leader.angle −
    order.angle)`; no run has fired the XOR term. *Capture, still owed:*
    `UNITS=3` + `GROUPS=1`, a group turned right round while marching and
    re-ordered in Refused or an Echelon (GROUPS §13); with it COLLISION
    §9's pause capture, two units ordered head-on.

45. **Gaia's animals.** `Sim::reseat_animal` puts them back from every
    traced dump (SYNC §4.2), so a wrong animal is invisible until an
    untraced run. Widen to `ANIMALDATA`: 70,960 uncompared.

The road residue, all in ROADS §7.1 and SYNC §6: (57) the terraform,
`terraform_for_building@00875210` after `place_roads`, oracle run13's
`FRAME 100` against run32's 104; (58) the height loader's mean in `f32`,
three of run12's tiles; (60) the frame's second search, 1,460 nodes
against 1,870; (62) the standing swap at frame 99, the first draw
*sequence* difference, costing no word.

91. **The final scenario, not a final dump.** One 24,000-frame game per
    map: the **trace** whole (`cover=1`, no window — cheap; the word over
    the whole game is the long headline) and the dump in windows
    re-captured *on demand*, since the game is reproducible (`samegame.py`)
    and a new window is a five-minute re-run. A full dump is
    24,000 × 2.4 s. Due when a map's word matches its 1,850; before that
    the long word parts where the short one does.

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
