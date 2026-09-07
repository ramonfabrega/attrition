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
    6892, `1/29` blocked by the standing citizen `1/17`. It is pushed
    *back* to (42408, 23880) on 6893 with `collide 1`, then slides due
    west at exactly −26/frame for 6894–6898 with **`y` pinned**, and the
    diagonal resumes on 6899; `idle` is 0 throughout, and `1/17`
    registers nothing — the whole interaction is written on the walker.
    Anything modelling a block as "stand still and repath" is six frames
    and ~1.6 tiles wrong here. Behind the word (6982) and so never yet
    compared: run83 is the first dump over [6870, 6910).

226. **`cover=1` dies in the wow64 bop at the exe's entry**, reproduced by
    `tools/trace/wow64bop.c` (ORACLE); an x86 host is the falsifier. (220)
    **TECH §13's twenty range blocks** are unloaded, no capture reaches
    one. (209) **The once-per-game events a dump install swallows** (207's
    general half): a `set_*` whose **return value** drives an irreversible
    record. **Takes 224**, AI §29.4's three other `mil_trainers` writers.
    (203) `Wall::mark_behind_tiles`' `0x4` is a building *finishing*, 32 at
    4802 and **45** at 4803, pinned by count and kind (ROADS §6). (240) **The deployed
    merchant's own record, unasserted** — run82 caught the unpack on 6883
    and `gather_down`/`special` are still −1 at 6929, 46 frames on, where
    run76's settled merchant has 18 and 6; MERCHANT §7 is one field short.
    **Takes 184.** (216) **`create_buildings` offers a gather building the
    original does not**: 6582's slots 1 and 4 carry `t 418 cat 4 val
    41500` against empty; inert, so the *list* differs — **238 may be this
    list finally acted on**.

(242) **The crate's group id is not the original's** — run76's Archers carry
`GroupData +0x4 = 64` (the unit block's own `group`) where `crates/sim`'s
`group_id` stands in with the army group's *slot*, 1; the other five group
fields agree on all 630 rows, and 237 marked the `id` row non-scoring with
the seam named. (243) `UnitDump::group` is parsed and **nothing compares
it** — the widening-ledger row 242 was found in.

AI residues, measured, none near a word: (169) `compute_site_stats`'
arithmetic — an extra site drags every `rank` (AI §2.13), 7,122 of run63's
27,000 site fields, (172) the `bucket` pair one apart on 5002/5061 (run60);
(158) the sweep runs for a human leader, here it is skipped (AI §23.1), the
gate is between steps 13 and 16; (159) the `CITY` record's two seams on
run58, `1/2007`'s `land`/`filled` from 1819 and the gatherers from 2576;
(146) `train_time`'s nine national arms after the ramp (PRODUCTION), (142)
`World::tregion` is not `get_tregion`, eleven callers (PATHFINDER §15–16);
(122) 16 of 61 draws without a `self.mark(`; (153) the `TRIBE` record, whole;
the thirteen `Census` fields the sweep writes for consumers not built
(`rondata::writers`' no-reader ledger, 20 rows, may only fall).

178. **The danger map's unit pass — and its falsifier may be a grep.**
    `role & 0x10000`, `(attack · 5) / 10` and the war gate rest on a
    reading (DANGER §8). But `danger[who][scan]` is already on disk in
    **eighteen** archives, four of them late East Indies combat windows —
    run29's frame 15100 carries 96 `TARGETORDER`s beside its danger map,
    run27's 46, just past the 15000 rebuild boundary. **Unverified**: that
    the army was on the map at the *rebuild* frame, not merely at 15100;
    confirm §2's schedule first. If it holds, no capture is owed. (181) **CARAVAN §7.2–§7.3's
    unreached arithmetic**: the arrival box, `× 16 / 2`, `(epoch[1] + 1) · 10`.

**The widening ledger is built** (87, DATALAYER §4): **22** parsed fields
the harness names nowhere and **42** one capture names; its blind spot is a
field the parser never had — `avg_speed` (210). Still uncounted: (88) the
blind list, 101 of 617 cited functions never entered — pin it as a floor;
(72) every `+0xNN` a document pins vs its module, **with `att-audit`**; (89)
the instrument's last guard, (c) alone; (35) `mylos` as a cache, VISION §7.

175. **The uber chain past its birth**: `Objects::init_unit` threads
    `uber_size` objects (CITIES §4.3), nothing else reads the chain,
    `build_sim` leaves `o_up`/`o_down` unset. Takes (48) the object chain
    whole (COLLISION §3, §7), `down`/`down_who` uncompared; (73)
    `UnitData::group`'s missing back-pointer, so `Group::normalize`'s cull
    is unmodelled (run33's scout 65 → 64 on 96) — **242 is its other
    half**; (56) the cell's `BUILDING` bit on `(52, 22)`, ARMY §13.

117. **Territory's three remaining terms all want the lobby.** The gem
    term landed (43 cells, diff-backed on run80). The temple and fort
    terms are **blocked at their prerequisite** in every game on disk, and
    the handicap is 0 by the lobby — a click, not a longer wait, and the
    only one of the three a capture alone could still reach. A scripted
    setup, a session's work. (Also AI §2.1's `check_explore` answers the
    grid whole, 900 v 36/19.)

Seven measured one-liners: (23) the formation byte's sign, Echelon half
(GROUPS §6.4), run52 the blocker; (103) a woodcutter's clock at 1,686, 445
against 480 (ORDERS §6.4); (105/45) Gaia's positions, East Indies, first
bad 1658 (SYNC §4.2); (124) the loop flag is per animation file (ANIM
§3.3); (116) the one `SITE` slot still wrong (AI §18); (166) `resource_cap`
on five goods, 2958–2959, 1392 v 2000 (ECONOMY); (107) `epoch[0]` is
Military and `army.rs` reads `ages`, `army.rs:769,1365`.

161. **The make-list block is 2,500 frames behind East Indies' word**:
    `create_buildings` first runs on 9982 (AI §25), so `building_value`,
    `gather_value`, §24.4's arm, `oil_patches.count` and
    `compute_largest_gather@0066e920` wait on it — **238 is the Great
    Lakes half, and it is live**. (195) `find_repair_spot` is
    `build_done`'s last of three (ORDERS §5.5–§5.9); no damaged building.

241. **East Indies' word, 7448 — the counter that has not moved.** At
    draw 30 this crate spends `Guy::set_anim+0x97a < Guy::inc_time+0x271`
    where the original spends `< Unit::move_step+0x823`: it steps an
    animation clock where the original stands blocked. Two of the same
    site, the family 236 dissolved on Great Lakes by making a follower
    read `move_step`'s zero — **but 236 did not move this**, so it is a
    second cause, not the same one. **No dump goes near it**: run77/78
    open at 10150 and run54 carries only the start dump's detail, so the
    window is owed before the reading. (167) run61's two: the landing
    search's *cell*, `Unit::do_strafe` (SYNC §3.9).

211. **`get_speed`'s three remaining arms** (MOVEMENT): `unit_masks &
    0x10`, set and cleared inside a frame; `has_general(0, 0x162)`'s siege
    doubling; the group cap, gated on `action_type == 0` (219), which a
    marching unit never answers — it wants a grouped unit holding no
    action.


229. **A figure in melee does not step its clock** — `unit_masks2 & 0x10`
    freezes `Guy::inc_time` (ANIM §5), 26 of 26 off run17/run44.
    `unit_masks2` has no field here, and the arm waits on a word that
    reaches a melee frame.

Older backlog: **the capture `String` each test reads**, ~5 GB of the
suite's remaining 10 — a rounded-up read buffer at ~100 call sites, and it
moves no score (235). (39) a read-only 2D viewer over `Sim`, dump overlaid;
(41) `scenario.py`'s fate — fuzz `424242 1000 1300`, blind report, delete;
a `find_target` block; run7's order stream; a mounted attacker;
`calc_gather` non-flat; `Leader::diplomacy`; (77) ANIM §3.2; (58) ROADS §7.4.

## How to maintain this file

- **End of session:** rewrite "Where things stand" from scratch — the
  headline first, and whether it moved. Delete finished items; their story
  goes to the journal. Run `cargo test -p sim docs_guard`. **The 200-line
  bound is a budget: a new item is paid for by compressing old ones.**
- **A floor that moves** moves three things together: `FLOORS` in
  `rondata::diff`, the assert that reads it, and the `Scoreboard:` line.
- **Start of session:** merge `lane-capture` and `lane-audit`, then "Where
  things stand", the item, then its document. Lanes never write this file.
- **A brief on an item touching `docs/<DOC>.md` names
  `docs/audit/2026-09-05-<doc>-vs-code.md`** where one exists: its rows
  are candidate items, the diff settles them, taken on the way through.
- **Overlap a capture's neighbours on purpose** — six blocks at each end,
  checked with `samegame.py` both ways (run83: 6 in common, 0 differing,
  twice). A window is priced by its dumped blocks and the run-up is free,
  so an overlap costs seconds and turns "same seed, therefore same game"
  into a state check where `rngcmp` is only a word. **The trap**:
  `samegame.py` exits 0 when nothing differs *including* when nothing is
  in common, so assert the common count and range, not the verdict.
- **Run the diff suite with `--release`** — 92 s against debug's 285 s
  since run53's 24,000 frames. A long wait is the capture, not a hang.
- **Before a blind fan-out:** `grep -n <mechanic> CLAUDE.md`, and the memory
  index — a subagent inherits both. **Never** quote this file or the journal
  into `CLAUDE.md`, a brief, an agent definition, or a memory hook.
