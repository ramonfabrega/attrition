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

*2026-09-07, Opus commanding (DECISIONS 34) — the loop is running: 261 and
258 in flight, and the reap found three items the crash had eaten.*

- **Great Lakes 7176** (leads; item **261** in flight): 97 draws differing
  at 93 — the original in `Unit::do_idle+0x7d`, this crate in
  `Guy::inc_time+0x271`; a unit goes idle there and not here, and run79's
  `[6910,7250]` covers it. **East Indies 7529**: a `Guy::set_anim` under
  `Unit::do_non_flat_gather+0xb99` the original spends and this does not.
- **258 runs beside it** at width two, harness only; its tests open a new
  module rather than grow `harness.rs` — 260's split, first cut.
- **The nine worktrees are reaped and one was not empty.** `loop-234` held
  406 uncommitted lines no branch carried, and `8b37e5f` had deleted
  230–234 in one go: two had landed, **232, 233 and 234 never did**. On
  `rescue-234`, re-booked below, audited in `docs/audit/queue-ledger.md`.
- **`tools/queueledger.py` is the guard that would have caught it**: a
  number leaves this file only if the journal names it or the ledger says
  where it went, and a `not-landed` line is owed a queue entry. 165 of 191
  already complied. **A branch check cannot see a dirty tree** — ccc holds
  the other half as their item 32.
- **The gate is pinned to two threads** — `.cargo/config.toml`, guarded in
  `floors.rs`; 15 GB at two against 24–33 at sixteen. **No pipe between
  `memcap.sh` and the exit code.** The stopping rule stands at two, with
  DECISIONS 34's exception.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7529 of 24,000 · GreatLakes w7176 of 24,000

**Opener: commander on Opus (DECISIONS 34). Merge 261 and 258, gate, file;
then the headline is whatever Great Lakes' word reads after 261. 232, 233
and 234 are re-booked and cheap — 234's two static assertions first.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious, and say so. Numbers are stable.

258. **In flight: the finish line's own frame is on disk, free** (252) —
    whole-map states at **24001 on both scored maps** (run53; run21/23/54)
    and East Indies at 15105/15401/16007/16489, diffed whole and pinned as
    a third scoreboard line that may only fall; (259) run29's 15105 reads
    **51 off, 22 unlinked**.

260. **The parser goes lazy and the gate sub-GB** — `Log::parse` builds
    all 76.6 M fields of an 800 MB dump (235) where a test reads hundreds.
    Frame offsets in one pass, blocks on demand over a memory-mapped file,
    whole-file scans (252) stream; the 230 tests are the oracle.

226. **`cover=1` dies in the wow64 bop** (ORACLE), an x86 host the
    falsifier. (220) TECH §13's twenty range blocks. (209) **the
    once-per-game events a dump install swallows** — a `set_*` whose
    **return value** drives an irreversible record; **takes 224**. (203)
    `mark_behind_tiles`' `0x4` is a building *finishing*. (240) the
    merchant's `gather_down`/`special` −1 at 6929; **takes 184**. (216)
    `create_buildings` offers a gather building the original does not.

(242) **The crate's group id is not the original's** — `GroupData +0x4 =
64` against `group_id`'s stand-in, the army group's *slot* 1; the other
five agree on all 630 and 237 reports the row without scoring it. (243)
`UnitDump::group` is parsed and **nothing compares it**. (244) **1,428
grouped order records no test windows** — run79's 453, run31's 945.
(245) `focus.sh` matched a concurrent worker's shell on run84.

255. **`go_to_unit`'s `0x480` and `go_to`'s `MOVE_TO` arm are read and
    never run** — every joiner on disk is farther than `0x480` from its
    army's `get_unit(0)`; the falsifier is one born beside its army.
    (256) `come_out`'s three `action_move_to` sites stay unreached.

247. **An upgrade is an in-place guy-type change on the standing unit** —
    run76's **6737**, three Archers going guy **170 → 177** keeping
    `(who, o)` and `group 64`, not a modifier on a type; East Indies'
    `1/32` does 340 → 341, danger row moving by `(110 − 100) / 2`. (181)
    CARAVAN §7.2–§7.3.

234. **Four rules of the turn/idle animation neither side has** (ANIM §9),
    written and never landed — branch `rescue-234`. **The two static
    assertions are the cheap half**; the limbs move no score (§4.7's
    turning types are in neither scored game). Takes the ledger's nine
    `unverified` rows on the way.

233. **A building's periodic phase is its object number**, not its slot:
    `Build::process@0061edf0:728` keys the 32-frame phase on `o`;
    `process_building` uses `frame + b`, right until a slot is reused
    (CITIES §5). (232) `Levels::for_player` ignores its player, answering
    a constant whose `taxation` is 0 (ECONOMY, audit R8).

**The widening ledger** (87, DATALAYER §4, §4.1): **19** fields the
harness names nowhere, **44** one capture names; blind spot `avg_speed`
(210) — it counts *fields* and cannot see a record the parser never
visits, which is what hid 252's dumps. Uncounted: (88) the blind list, 101
of 617; (72) every `+0xNN` a document pins vs its module; (89) the guard;
(35) VISION §7.

257. **Nineteen kept tests compare a torn block** (252): a closing dump is
    frame n but for the one unit the quit caught mid-update, and every
    nested archive's last `FRAME n` body *is* its closing dump. No live
    case found, but unaudited; the fix is `compare_shutdown`'s n−1
    allowance given to those last blocks.

175. **The uber chain past its birth**: `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3); nothing else reads it. Takes (48)
    COLLISION §3/§7, (73) `UnitData::group`'s back-pointer, (56) ARMY §13.

248. **ATTRITION's Territory section is 15,625 bytes of 16,000** and wants
    a retelling pass before anything is added. Its remaining terms are
    closed — handicap, temple and fort are unreachable in any game that
    runs here; read it before re-booking any. (AI §2.1's `check_explore`.)

Measured residues, none near a word: (246) step 6's repath rests on
run83's single event (239); (169) `compute_site_stats`, 7,122 of
run63's 27,000 site fields; (172) the `bucket` pair on 5002/5061; (158)
the human-leader sweep (AI §23.1); (159) the `CITY` record's two seams;
(146) `train_time`'s nine national arms; (142) `World::tregion`
(PATHFINDER §15–16); (122) 16 of 61 draws without a `self.mark(`; (153)
`TRIBE`; (20) the `Census` rows; (23) the formation byte's sign, Echelon
half (GROUPS §6.4), run52 the blocker; (103) a woodcutter's clock, 445 v
480 (ORDERS §6.4); (105/45) Gaia's positions, first bad 1658 (SYNC §4.2);
(124) the loop flag is per animation file; (116) the one `SITE` slot (AI
§18); (166) `resource_cap` on five goods (ECONOMY); (107) `epoch[0]`.

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value` and §24.4's arm wait on it. (195) `find_repair_spot`.

253. **`1/13` parts at 6938**, a citizen at `myspeed 25` — run86's only
    other divergence, and run82's 6946 dump shows it off by (11,7). A
    diff, not a capture. (254) `come_out`'s ring has one sample (7284)
    and wants a second disembark. (167) run61's two (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
    0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
    doubling; the group cap, gated on `action_type == 0` (219) — it wants
    a grouped unit holding no action. (229) a figure in melee does not
    step its clock — `unit_masks2 & 0x10` freezes `Guy::inc_time` (ANIM
    §5), and that arm waits on a melee frame.

251. **`memcap.sh` has no fixture and a real hole** (lore): its `ps | awk`
    reads `0 0` when `ps` answers nothing, so a refused sample reads as no
    memory used and the ceiling never fires — Friday's crash by another
    door, under the guard width two rests on. Wants a fixture: past the
    cap, dead in N seconds, exit 137.

Older backlog: (39) a 2D viewer over `Sim`; (41) `scenario.py`; a
`find_target` block; run7's orders; a mounted attacker; `calc_gather`
non-flat; `Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch, headline
  first and whether it moved; delete finished items to the journal. **The
  200-line bound is a budget** — a new item is paid for by compressing old
  ones, and you count the lines before writing, not after.
- **A floor that moves** moves `FLOORS`, the assert reading it, and the
  `Scoreboard:` line together.
- **Start of session:** merge the lanes, then "Where things stand", the
  item, its document. Lanes never write this file.
- **A brief names `docs/audit/2026-09-05-<doc>-vs-code.md`** where one
  exists — candidate items, taken on the way through.
- **Overlap a capture's neighbours on purpose.** The run-up is free, so an
  overlap costs seconds and turns "same seed, therefore same game" into a
  state check. Six blocks each end is the floor (run83); a neighbour that
  **contains** yours is free and total (run84, 80 of 80) with `--exclude
  <the category you raised>`. **The trap**: `samegame.py` exits 0 when
  nothing is in common, so assert the count, not the verdict.
- **Grep for the derived quantity, the mechanic's SHAPE and the GATE — not
  only for coverage of a frame.** `BUILDQUEUE` prints
  `queue[scan].cost[0..2]`, so a price the original *paid* is on disk
  wherever `BUILDS` is on (238); 178's two passes write different
  footprints, so a half-cell with no building in its 3×3 isolates the unit
  pass in archives already held. And a term zero in every dump is either
  zero-valued or switched off: 117 and 178 were booked on the first and
  were the second.
- **A check on a field never cleared must assert a CHANGE, not a value** —
  `collide_frame` is a permanent stamp, so "some unit has one in the band"
  is true of any window, and run85's first teeth check passed on a band
  where nothing happened. Test both directions on real data first.
- **A commander's clear is free when**: every landed branch merged, gated
  and pushed; nothing in flight owing its result elsewhere; this handoff
  current with the headline *measured*; nothing unfiled. `ccc clear <ref>
  --then "continue"` arms it — 0.1.29 fixed the gate, and the next cut
  resolves a session **name** everywhere rather than only a daemon id.
- **Run the diff suite with `--release`** under `memcap.sh`, redirected to
  a file — a pipe launders its 137 — and keep the `peak` line; a long wait
  is the capture, not a hang. **Before a blind fan-out**, `grep -n
  <mechanic> CLAUDE.md` and the memory index — a subagent inherits both,
  and neither they nor a brief may quote this file or the journal.
