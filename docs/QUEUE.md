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

*2026-09-06, Opus commanding, the loop's first run at width two —
**Great Lakes 6862 → 6982**, four items landed, gate green at 217. East
Indies **7448 unmoved**, and it is now the map with the work.*

- **Both headline items are live.** 238 is Great Lakes' word: at 6982 this
  crate spends `Leader::produce_building+0x1805` where the original spends
  `Leader::make_stuff+0x221` — it queues a building the original does not.
  East Indies 7448 is `Guy::set_anim+0x97a < Guy::inc_time+0x271` here
  against `< Unit::move_step+0x823` there — the family 236 dissolved on
  the other map — and **no dump goes near it** (run77/78 are at 10150+).
- **Width two works.** Every gate is `zsh tools/memcap.sh 20 cargo test -p
  rondata --release -- --test-threads=2` (14.8 GB, ~29 GB for two); four
  threads stay off the table. A worker takes its base's tip with `ccc
  update <ref>`, never `git merge`; the commander removes a landed
  worker's session at merge time, not at session end.
- **Grep the detail, not only the range**: run79 covers 6982 and cannot
  answer it — 22 `MAKELIST` occurrences against run80's 902.
- **The docs-versus-code wave is not a ratification question** — its rows
  are two of this repo's texts disagreeing, the diff is their oracle, 6
  are assertions, the rest sit in `docs/audit/2026-09-05-<doc>-vs-code.md`
  and are read by any worker whose item touches that document.
- Lanes: `att-capture` holds the screen, corpus 232; cycle it after East
  Indies lands — `ORACLE.md` and `captures.txt` are its handoff.

Scoreboard: EastIndies 1851/1850 w1850 · GreatLakes 1772/1772 w1850
Long captures: EastIndies w7448 of 24,000 · GreatLakes w6982 of 24,000

**Opener: the main thread is the commander (DECISIONS 34), on Opus. 237
and 238 are in flight; East Indies 7448 is the next unstarted, and it is
the counter that has not moved.**

## The queue

In dependency order, headline-nearest first; **the headline is the long
captures' word**, lower map first — Great Lakes. Take the first unstarted
unless a better order is obvious — and say so. Numbers are stable; the
journal is indexed by them.

239. **A blocked step is a deflection, not a stop** — run83, Great Lakes
    6892: `1/29`, blocked by the standing citizen `1/17`, is pushed *back*
    on 6893 with `collide 1`, slides due west at exactly −26/frame for
    6894–6898 with **`y` pinned**, and resumes the diagonal on 6899;
    `idle` 0 throughout and `1/17` registers nothing. "Stand still and
    repath" is six frames and ~1.6 tiles wrong. run83 is the first dump
    over [6870, 6910), so this is behind the word and never compared.

226. **`cover=1` dies in the wow64 bop** (ORACLE), an x86 host the
    falsifier; `tools/trace/wow64bop.c` reproduces it. (220) TECH §13's
    twenty range blocks are unloaded. (209) **The once-per-game events a
    dump install swallows** (207's general half): a `set_*` whose **return
    value** drives an irreversible record; **takes 224**, AI §29.4's three
    other `mil_trainers` writers. (203) `mark_behind_tiles`' `0x4` is a
    building *finishing*, 32 at 4802 and 45 at 4803 (ROADS §6). (240) **The deployed
    merchant's own record, unasserted** — run82 caught the unpack on 6883
    and `gather_down`/`special` are still −1 at 6929, 46 frames on, where
    run76's settled merchant has 18 and 6; MERCHANT §7 is one field short.
    **Takes 184.** (216) **`create_buildings` offers a gather building the
    original does not**: 6582's slots 1 and 4 carry `t 418 cat 4 val
    41500` against empty; inert, so the *list* differs — **238 may be this
    list finally acted on**.

(242) **The crate's group id is not the original's** — run76's Archers carry
`GroupData +0x4 = 64` where `crates/sim`'s `group_id` stands in with the
army group's *slot*, 1; the other five group fields agree on all 630, and
237 reports the row without scoring it. (243) `UnitDump::group` is parsed
and **nothing compares it**. (244) **1,428 grouped order records no test
windows** — run79's 453 and run31's 945; windowing run79 is the cheapest
second capture for 237's newly-live rows. (245) **`focus.sh` matched a concurrent
worker's shell** on run84 — `ready (riseofnations.exe…)` where the traced
binary had launched; a reader would conclude otherwise.

AI residues, measured, none near a word: (169) `compute_site_stats`'
arithmetic — an extra site drags every `rank` (AI §2.13), 7,122 of run63's
27,000 site fields; (172) the `bucket` pair one apart on 5002/5061 (run60);
(158) the sweep runs for a human leader, skipped here (AI §23.1), gate
between steps 13 and 16; (159) the `CITY` record's two seams on run58;
(146) `train_time`'s nine national arms after the ramp (PRODUCTION); (142)
`World::tregion` is not `get_tregion`, eleven callers (PATHFINDER §15–16);
(122) 16 of 61 draws without a `self.mark(`; (153) the `TRIBE` record; the
thirteen `Census` fields written for consumers not built
(`rondata::writers`' no-reader ledger, 20 rows, may only fall).

178. **The danger map's unit pass — its falsifier may be a grep.** `role
    & 0x10000`, `(attack · 5) / 10` and the war gate rest on a reading
    (DANGER §8), but `danger[who][scan]` is on disk in **eighteen**
    archives, four of them late East Indies combat windows: run29's 15100
    carries 96 `TARGETORDER`s beside its danger map, run27's 46, past the
    15000 rebuild boundary. **Unverified**: that the army was on the map
    at the *rebuild* frame. Confirm §2's schedule; then no capture. (181) **CARAVAN §7.2–§7.3's
    unreached arithmetic**: the arrival box, `× 16 / 2`, `(epoch[1] + 1) · 10`.

**The widening ledger is built** (87, DATALAYER §4): **19** parsed fields
the harness names nowhere and **44** one capture names (237 moved both);
its blind spot is a field the parser never had — `avg_speed` (210). Still
uncounted: (88) the blind list, 101 of 617 cited functions never entered;
(72) every `+0xNN` a document pins vs its module, **with `att-audit`**;
(89) the instrument's last guard; (35) `mylos` as a cache, VISION §7.

175. **The uber chain past its birth**: `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3), nothing else reads the chain,
    `build_sim` leaves `o_up`/`o_down` unset. Takes (48) the object chain
    whole (COLLISION §3, §7); (73) `UnitData::group`'s missing
    back-pointer, so `Group::normalize`'s cull is unmodelled — **242 is
    its other half**; (56) the cell's `BUILDING` bit, ARMY §13.

117. **Territory's three remaining terms want the lobby.** The gem term
    landed (43 cells, diff-backed on run80); the temple and fort terms are
    **blocked at their prerequisite** in every game on disk; the handicap
    is 0 by the lobby — a click, and the only one a capture could reach. A
    scripted setup. (AI §2.1's `check_explore` answers the grid whole,
    900 v 36/19.)

Seven measured one-liners: (23) the formation byte's sign, Echelon half
(GROUPS §6.4), run52 the blocker; (103) a woodcutter's clock at 1,686, 445
against 480 (ORDERS §6.4); (105/45) Gaia's positions, East Indies, first
bad 1658 (SYNC §4.2); (124) the loop flag is per animation file (ANIM §3.3);
(116) the one `SITE` slot still wrong (AI §18); (166) `resource_cap` on five
goods, 1392 v 2000 (ECONOMY); (107) `epoch[0]` is Military, `army.rs`
reads `ages`.

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value`, §24.4's arm and `compute_largest_gather@0066e920` wait
    on it — **238 is the Great Lakes half and it is live**. (195)
    `find_repair_spot` is `build_done`'s last of three; no damaged
    building.

241. **East Indies' word, 7448 — the counter that has not moved**, and
    **run85 answers it** ([7400, 7479), `GUYS=4`). One stand: the lone
    `1/20`, group −1, blocked by `8/0` — an **animal**, stationary, one
    tile south of its line. On 7449 the original sets **both** its guys to
    `anim 0 / end_time 60 / stopped 1` and discards the 60-frame animation
    after one frame; it then freezes three frames, snaps due west, steps
    −23 with `y` pinned, and slides. **The two draws are two guys**, so
    this crate reaching the same function from `Guy::inc_time` is one
    missing blocked-step animation, not two. Why 236 missed it, now on
    disk: 239's Great Lakes stand is a squad Archer blocked by a citizen
    and slides at once — different obstacle class, different unit shape,
    different recovery. (167) run61's two: the landing search's *cell*,
    `Unit::do_strafe` (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
    0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
    doubling; the group cap, gated on `action_type == 0` (219) — it wants
    a grouped unit holding no action.


229. **A figure in melee does not step its clock** — `unit_masks2 & 0x10`
    freezes `Guy::inc_time` (ANIM §5), 26 of 26 off run17/run44; no field
    here, and the arm waits on a word that reaches a melee frame.

Older backlog: (235) the capture `String` each test reads, ~5 GB of the
suite's 10 — a rounded-up read buffer at ~100 call sites, no score; (39) a
2D viewer over `Sim`, dump overlaid; (41) `scenario.py`'s fate; a
`find_target` block; run7's orders; a mounted attacker; `calc_gather`
non-flat; `Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`. **The 200-line
  bound is a budget: a new item is paid for by compressing old ones.**
- **A floor that moves** moves three things together: `FLOORS` in
  `rondata::diff`, the assert that reads it, and the `Scoreboard:` line.
- **Start of session:** merge the lanes, then "Where things stand", the
  item, then its document. Lanes never write this file.
- **A brief names `docs/audit/2026-09-05-<doc>-vs-code.md`** where one
  exists: its rows are candidate items, taken on the way through.
- **Overlap a capture's neighbours on purpose.** A window is priced by its
  dumped blocks and the run-up is free, so the overlap costs seconds and
  turns "same seed, therefore same game" into a state check where `rngcmp`
  is only a word. Six blocks each end is the floor (run83); when a
  neighbour's window **contains** yours it is free and total (run84, 80 of
  80) with `--exclude <the category you raised>`. **The trap**:
  `samegame.py` exits 0 when nothing is in common, so assert the common
  count and range, not the verdict.
- **A check on a field that is never cleared must assert a CHANGE, not a
  value** — `collide_frame` is a permanent stamp, so "some unit has one in
  the band" is true of any window on the game, and run85's first teeth
  check passed on a band where nothing happened. Test both directions on
  real data before the capture runs.
- **Run the diff suite with `--release`**; a long wait is the capture, not
  a hang. **Before a blind fan-out**, `grep -n <mechanic> CLAUDE.md` and
  the memory index — a subagent inherits both, and neither they nor a
  brief nor an agent definition may quote this file or the journal.
