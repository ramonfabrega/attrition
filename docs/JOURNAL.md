# Journal

The chronicle: what each session did, dated, append-only. It is never the
opener — `docs/QUEUE.md` is what a session reads first — and it is never
inherited by a subagent, because it names the findings a blind reader is
meant to re-derive. What it is for is the yield story, which the mechanic
documents deliberately prune and git history carries badly: what a reading
found, what a run confirmed, what a diff overturned, and what each cost. It
is also the one document here that could one day be public, as the account
of how this was built.

Rules:

- One entry per session, dated, newest last, pointing at commits and
  document sections rather than restating them.
- Label a claim's status where the claim is — "observed in run16", "read,
  not yet run", "inferred" — never in a preamble. A chronicle reads as
  narrative, and a hedge stated once at the top is settled fact by the next
  reader.
- A struck queue entry points here; this file never points back at the
  queue.
- Never quoted into `CLAUDE.md`, a subagent brief, or a memory hook.

## Lifted from the queue, 2026-08-25 — by item, not by date

What follows is the working agreement's queue as it stood in `CLAUDE.md` on
the morning of 2026-08-25, lifted verbatim into `docs/QUEUE.md`
(`docs/DECISIONS.md` entry 21) and moved here the same day when the queue's
entries were cut to one line each. It is organised by queue item, and each
item's narrative runs session by session, oldest first. From the next
dated heading on, the journal is by date.

0. ~~**Corrections from the second reading**~~ — **done, 2026-08-20.** All six
   mechanics' audits under `docs/audit/` are landed; each document now ends
   with a "Second reading — landed" section, and each place a claim changed
   says so inline. The ones that changed observable behaviour: attrition's
   sixteenths-damage and bleeding wagons, production's library-only fan-out
   and research-that-trains-nothing, costs' redirect table, movement's
   unit-versus-body step. Done the way the next one should be: one worker per
   mechanic, each re-verifying every claim against the decompile before
   changing it, document first, then implementation and tests.
1. ~~**The tech tree**~~ — **done, 2026-08-20**, `docs/TECH.md` and
   `crates/sim/src/tech.rs`: `has_tech`, `get_preq`, `has_preq`,
   `type_eligible`, `type_avail`, `queue_here`, `gain_tech` with its cascades,
   `lose_tech`, `set_age`, the starting position, the lobby's start/end ages.
   The `TypeIndex` enum is now dumped whole by
   `tools/ghidra/scripts/DumpEnumAll.java`. Second reading done and landed
   (`docs/audit/2026-08-20-tech.md`): it found the loaders' derived fields —
   every combat unit implicitly needs its age's Military epoch — which a
   reading of the predicates alone cannot see. Read the loader too.
2. ~~**Combat**~~ — **done, 2026-08-20**, `docs/COMBAT.md`, `crates/sim/src/
   combat.rs` (the arithmetic), `fight.rs` (the attack step, targeting,
   projectiles, buildings) and `balance.rs` (the combat table's generation).
   The damage formula end to end, sixteenths delivery, the recharge cadence,
   accuracy/scatter/flight time/hit test/splash, the game's RNG, buildings'
   arrows, the target search and ranking. Second reading done and adjudicated
   (`docs/audit/2026-08-20-combat.md`). What it left as inputs: the nation,
   wonder and patriot modifiers, terrain flags, the `type_damage` lineage
   predicates that need a tree loader; what it left open: the flank direction
   convention (a behavioural check) and the `balance.xml` flag-row separator
   (the `RULES=1` log settles it).
3. ~~**Cities and buildings**~~ — **done, 2026-08-20**, `docs/CITIES.md`,
   `crates/sim/src/build.rs` (the type, the construction clock, the site's
   hit points, repair, refunds), `place.rs` (the tile layer and the
   `blocked_site` predicates with the original's `BlockIndex` verdicts),
   `city.rs` (the record, membership, the automatic level-up, capture,
   assimilation, the city heal, plunder, default-mode elimination) and
   `garrison.rs` (the FIFO chain, `can_garrison`, the `do_garrison` gates,
   one-squad-a-frame ejection, the garrison heal). Read by five readers in
   parallel, one per sub-area, and re-read blind the same way
   (`docs/audit/2026-08-20-cities.md`). Two of the first reading's claims were
   settled in the disassembly rather than the decompile — the argument
   `do_build` passes and the value `capture_strength` takes — which is the
   cheap move whenever the decompiler prints a local that cannot be right.
   What it left as inputs: the nation, wonder and tech layers (`city::Nation`,
   `build::BuildMods`/`ClockMods`/`HitsMods`), the per-wonder one-offs of
   `activate`, the AI branches, visibility, the capital-countdown elimination
   mode.
4. **The behavioural-check batch** — **run 2026-08-20, two sessions; the
   method is proven and the list is mostly closed.** What it took was finding
   the loggers' real switchboard (the ini values are per-category *detail
   thresholds*, `UNITS=3 BUILDS=6 CITIES=5` is the setting, `DUMP_ALL=1` is a
   different switch that hangs the game but is the only way to the type
   tables and `COMBATTABLE`), the chat-cheat vocabulary, the `~` console and
   its tile-coordinate mode, `-config`/`-automation` for the lobby, and the
   fact that `cheat select <o>` plus a right-click issues a normal order —
   which made every remaining check scriptable. All of it is one recipe in
   `docs/ORACLE.md`, "Running a check: the recipe in one place", with the
   scripts in `tools/gamelog/`.

   **Landed:** the construction clock, both halves — one builder is +100 a
   frame, two are +150, the clock re-bakes on a tech change for unstarted
   sites only (`docs/CITIES.md` §3.2–§3.3); the lobby start age grants no
   epochs, techs or pop cap (`docs/TECH.md`); a city is refused at exactly 24
   tiles and starts at 25, on two bearings, with the tower-site-first control
   that makes every other `blocked_site` verdict testable (`docs/CITIES.md`
   §2.6.2); the city heal, period 4 and one point per level (`docs/CITIES.md`);
   `UnitData.angle` is the facing, so the flank convention is settled at its
   premise — level 1 is the rear (`docs/COMBAT.md` §14.1, `docs/MOVEMENT.md`);
   the `balance.xml` separator, settled against the dumped table (see 6).

   **Still open, none blocking, each a 20–60 minute drive when the game is up
   for another reason:** the city mask's remaining three rungs (recipe in
   `docs/CITIES.md` §3.6); the garrison heal — no script path into a building
   found yet, the remaining route is `Build::train` + `cheat finish`
   (`docs/CITIES.md` §15); ~~the Nubian attrition step, which needs attrition to
   actually run (`docs/ATTRITION.md`)~~ — **done, run16, 2026-08-24: the
   clause is dead, by reading and by observation** (`docs/ATTRITION.md`,
   last section; the whole attrition/supply mechanic was observed in that
   run, every prediction holding); ~~the flank damage ratio~~ — **done,
   run17, 2026-08-24: 48/85/117 by damage, level 1 the rear
   (`docs/COMBAT.md` §16)** — for which
   `tools/gamelog/hits.py` is the instrument and one damage source on the
   target is the setup (`docs/COMBAT.md` §14.1); and, new from the
   `BUILDS=7` run, **`FARM_GROWS`** — the original re-targets a farmer at
   frame 102 against the documented ~200, which the harness's order diff
   measures for free on any `UNITS=3` dump (`docs/ORDERS.md` §6.5).
5. ~~**The data layer into the sim, and the diff**~~ — **done, 2026-08-20**,
   `docs/DATALAYER.md`, `crates/rondata/src/{gamelog,dump,load,diff}.rs`:
   the gamelog reader, the loader (364 units, 129 buildings, 628-entry tree,
   the combat table — every key resolved, by `TYPENAME`), and the harness
   (`rondata <install> --gamelog <dump> --diff`). The dump's `CONSTANTS`
   block checked 231 of 232 `Tuning::RON` slots against the program's own
   loaded values and classified the representation of all 716. Taken out of
   order, before 4, because the game window was on the user's screen; the
   harness's score is the expected ceiling (1 tick — no AI, no orders) and
   its next inputs are `DUMP_ALL=1` and an order stream. Two corrections
   landed on the way: XML *fields* are read by tag name from
   `internal_strings.xml` (records stay positional — `docs/FORMATS.md`), and
   `RULES=1` does not dump the type tables (`docs/ORACLE.md`).
6. ~~**The combat table's hardcoded half**~~ — **done, 2026-08-20**,
   `docs/COMBAT.md` §15.2, `crates/rondata/src/typesdump.rs` and
   `rondata --types <dump>`. The start-of-game dump's `COMBATTABLE` is the
   engine's own composed table, and its `UNITTYPE` blocks are the inputs —
   so the check compares both, the 364 `Kind`s field by field and then the
   364 × 364 block. It found three predicate errors, not thirty: the age one
   too low for 306 units (`get_age_slow` adds 1 to an age tech's `AGE`), four
   lineage roots keyed by a display name that was the wrong unit (`ECOMPANION`
   is *Royal* Companion), and `0x14000` misread as M|O where it is O|Q. With
   them landed **0 of 132,496 cells differ**, and the check is the
   regression guard from here. The `Flag_` rows are dead data (§14.9), landed
   with it. The building half followed the same day (§15.3): `combat::Table`
   is two-family, `rondata` builds a `Kind` per building, the fight path
   applies the table to every pair, and **all four quadrants of the
   493 × 493 are equal** — the one finding on the way was a building's
   domain, `BuildType::set_domain` from `BUILD_FLAGS b`.

7. ~~**Orders**~~ — **done, 2026-08-21**, `docs/ORDERS.md`,
   `crates/sim/src/orders.rs`: the order list and its three enqueue modes,
   one order stepped per frame, the move order end to end with the
   pathfinder as a named seam, build/repair/garrison through the swarm ring,
   the gather registration and the wood/ore machine, the attack order's
   reload gate and chase, the idle cadence, the start of a game, the spot
   search, the log format. Taken before AI because AI issues orders and the
   harness's score could not move without them. The blind second reading and
   all seven adjudications are landed (`docs/audit/2026-08-21-orders.md`),
   and so is the harness work: the start of a game in `build_sim` and the
   `UNITS=3` order blocks read back and diffed **every frame**
   (`docs/DATALAYER.md` §3.1). The intent diff earned itself immediately —
   six farm citizens holding `THINK` where the original holds `GATHER`, from
   frame 1, with both simulations agreeing on every position. The
   `BUILDS=7` dump that was blocking it **has been captured**
   (`gamelog-run6`, `docs/ORACLE.md`), `gather_from` and `orig_type` are
   read into the harness, and **a woodcutter's citizen now matches the
   original's position and order list for all 432 logged frames**. What
   still cannot move: the AI's units (they need the order stream) and the
   farmers, which part at frame 102 — the run's own finding, that
   `FARM_GROWS = 200` is wrong by about a factor of two (`docs/ORDERS.md`
   §6.5).
8. ~~**The pathfinder**~~ — **done, 2026-08-23**, `docs/PATHFINDER.md` and
   `crates/sim/src/path.rs`: `astar_path` on all three grids (greedy
   heuristic, LIFO tie-break, metric-keyed dedup, budgets, suspend,
   reconstruction), `calc_cost` with every term modelled or a named seam,
   the three wrappers with their pre-walks and post-processing,
   `invalid_loc` with the original's five flags, `do_move`'s planner wiring,
   and §4.6's march fixed point **settled** — the original recomputes the
   angle every step and clamps each axis, so the 2026-08-22 guard and its
   sync divergence are retired. `PathLength`/`PathTo` **score** now; on
   `gamelog-run6` the first woodcutter matches all 432 frames and the
   second's path stack matches for 427 (both pinned as tests). Every
   remaining path disagreement is player 1's units crossing **forest the
   flat harness world does not carry** — so the next path-diff win is
   world data (the map's tile layer into the harness), not search. Found
   on the way: flat gatherers' footprints were wrongly `BLOCKED`
   (`mask_me`'s template only blocks marked tiles), and the cost function
   clamps its additive term at zero, so the own-territory −4 only ever
   offsets danger. The blind second reading ran the same day and is landed
   (`docs/audit/2026-08-23-pathfinder.md`): 32 verdicts, the structure
   doubly confirmed, five behavioural corrections applied — among them
   `leaders.flags & 4` = `is_human` gating `army`/`worker` (which exposed
   the gamelog parser handing every leader its successor's flags — fixed),
   the pull-back walk's give-up exit, and the corner-cutting probe table,
   settled in the listing. Still open: `go_around_building`, collision
   recovery (`find_upath`'s caller), suspend/restore stashing, and the
   forest tile layer for the harness world (the remaining run6 path gap).
9. ~~**The recorded-game container**~~ — **done, 2026-08-24**,
   `docs/RECGAME.md` and `crates/rondata/src/recgame.rs`
   (`rondata --recgame <file.rcx>`). Taken out of order because the missing
   piece arrived from outside: the container was derived in ORACLE.md Part 1
   but deliberately left unimplemented until a file existed, and
   heavengames' downloads section (user link, ~245 recorded games) supplied
   the one explicitly EE-era sample. The full header walk (`Game::walk_data`
   → `GameInfo::walk_data`, three wire primitives), the random-game blob
   (playback reads it and frees it unused), the embedded rules, and the
   package stream to EOF. Found on the way: **a recording embeds the loaded
   rules tables**, and the sample's composed 493×493 combat table equals
   ours **cell for cell across a seven-year build gap** — so any recording
   is now the combat-table ground truth, replacing the `DUMP_ALL` run that
   hangs the game. The blind second reading ran the same day and is landed
   (`docs/audit/2026-08-24-recgame.md`): the 943-byte header map identical
   in both readings, the package-stream start proven unique by a backward
   DP at the landmark's exact byte, and three corrections adopted — the
   file is written raw and gzipped only in `finalize` (sniff `1f 8b`), the
   marker is the insensitive hash's low byte, and the Types stretch is
   readable in principle but provably needs the loader's `types.list`
   order. What it leaves open: ~~the command payload encoding~~ (done, see
   10), the 806 per-type rules walkers (skipped by a package-validated
   landmark), and ~~recording a fixed-seed game on *this* install so order
   stream and gamelog describe the same run~~ — **done, see 12**.
10. ~~**The command payload encoding**~~ — **done, 2026-08-24**,
    `docs/COMMANDS.md` and `crates/rondata/src/commands.rs`. The full
    dispatch table (`CommandPackage::process`, 0x00–0x51, 82 commands),
    every wire layout from the PDB's `*Command` structs (packed from +0x1,
    doubly derived — decode-side `process_*` returns and encode-side
    `issue_*` sizes agree everywhere), the four variable-length encodings,
    the selection model (a `group` command sets the package's selection,
    `num = 0` meaning "same as last"), and the MP-only obfuscation
    (seed-keyed u16 XOR in `CommandPackage::send` plus seeded
    `Random::get(0,2)` padding gaps) — single-player recordings are plain,
    and the turn pump shows recordings store packages *pre-decode*.
    Verified in one shot: all 21,884 payloads of the heavengames sample
    decode with zero errors to exact size (25,779 commands — a camera
    command per frame, the rest a 23-minute game's real input), pinned as
    an install-gated test and a histogram in `rondata --recgame`. The
    blind second reading ran the same day and is landed
    (`docs/audit/2026-08-24-commands.md`): the table re-derived
    identically, four corrections adopted — `console_cmd` physically
    cannot be sent (0x209 > the 512-byte cap), the save-game walker
    serialises `group` where the recording head does not, five commands
    are dispatched but provably never emitted, and the `valid` flag is a
    free SP/MP fingerprint. What it leaves open: an MP sample to exercise
    `decode_mp`, and `marwan`'s semantics.
11. ~~**The recorded order stream into the harness**~~ — **done,
    2026-08-24**, `docs/INPUT.md`, `crates/rondata/src/input.rs`, and
    `rondata --recgame <file> --gamelog <dump> --diff`. Taken out of order
    because it was the one item on the queue that needed the game running
    and a person at the keyboard. **The paired run exists**:
    `gamelog-run7-ancient-nubian-orders.txt` and `Playback - 2026.08.24
    10'15'53 (Mon).rcx` — 1,732 frames and 1,732 packages of the same game,
    same lobby as run6 so the only variable is that this one has input in
    it. Feeding it turns `Length { ours: 0 }` and `Kind { ours: 7 }` into
    path-level disagreements on the three units the stream names, i.e. the
    orders now exist and match in kind. Found on the way: **recording was
    already switched on** (`prefs & 0x1000000`, the Options → Game
    checkbox), so fourteen `.rcx` files of earlier sessions were already on
    disk; **gzip is the clean-quit fingerprint** (`finalize` is the only
    repack, and the two gzipped files are exactly the two runs quit through
    the menu, thirteen raw ones exactly the killed runs — the second
    reading's correction confirmed against fifteen files at once); and
    **the path stack's goal is the un-snapped click, not the snapped
    destination** (§7, evidence in the dump, deliberately not implemented —
    it is a pathfinder change and belongs with the pathfinder's pins).
12. **AI** — last, because it is the largest and the least oracled.
    ~~and better oracled once 4, 5 and the pathfinder stand~~ — still true,
    but **the reason it is last has changed and it is now blocking**. No AI
    class issues a command (`docs/INPUT.md` §1: every `issue_*` caller is a
    UI class or the turn pump, and `issue_cheat_ai_toggle` being a
    replicated command proves the AI is re-derived per client). So **a
    recording replays the AI by re-simulating it**, and the replay diff's
    score cannot pass player 0 on any match with an AI in it, no matter how
    good the order stream is. The queue used to imply otherwise.

    **In progress, 2026-08-24** — `docs/AI.md` is the handoff. The first
    reading is complete (the driver, the census, the step machine, the
    make list, the sites, the city AI, research, the market, the scripts;
    four readers' reports for the language, the 55 host functions and the
    two `create_*` producers, ratified in §11), `crates/sim/src/ai.rs`
    holds the driver's pure half and `crates/sim/src/bhs.rs` the script
    interpreter (`docs/DECISIONS.md` entry 20). The finding that reframes
    this item: **the skirmish AI's opening is the shipped `economic.bhs` /
    `defensive.bhs`**, run through the scripting VM at production step 1 —
    "scripted in the open" was righter than the queue knew, and "escape
    the 2002 VM" is answered by an interpreter of our own reading the
    scripts from the install. The oracle is unusually good: run7's first
    115 seconds are the script's decisions, frame by frame (§5).

    **The opening runs end to end, 2026-08-24 (second session).** Object
    numbers (`find_free`'s bands), a technology in the production queue,
    the `Lobby`, the 55 host functions (`ai_host.rs`), `produce_building`
    (`ai_place.rs`, `circle_init` rebuilt in integers), and the driver in
    `Sim::tick` (`ai_drive.rs`). **Pinned on run7**: at frame 2 the AI's
    city holds three citizens at 25/26/27 food, its library Written Word
    then City State, and one farm site numbered 2006 with a citizen
    ordered onto it — `defensive` steps 6, 7, 9, 10 in one call. Run6's
    unlinked unit-frames went 673 → **0**; run7's 6,543 → 1,970. A
    `LEADERS=9` run (run8) settled the personality and the script
    (`docs/ORACLE.md` corrected: `LEADERS` obeys the threshold). ~~Next:
    the census and the C++ producers (§12.1), the map's tile layer for
    the farm's exact tile, then the blind second reading.~~

    **The census and the C++ producers landed, 2026-08-24 (third
    session)** — seven modules by seven parallel Opus workers over a
    shared surface (`ai_census`, `ai_sites`, `ai_research`, `ai_units`,
    `ai_build`, `ai_make`, `ai_types`), every formula re-read against the
    decompile as it was written, **thirty-odd corrections to §2 recorded
    in `docs/AI.md` §14** for the second reading. Two oracles found on
    the way, each one ini line: **`WORLD=6` under `[Start Game]` dumps
    the map** (every cell's `val`/`goods`/`region`/owner and every
    tile's mask — run9; `rondata` loads it, and the AI's farm lands on
    the original's tile), and **`LEADERS=9` is the census** (run8/run9's
    frame 1 agrees with the sweep field for field, one ceiling: the
    woodcutter camp's `calc_gather` slots). ~~Next: run10 (this lobby,
    the map, run7's length) into the harness; the frame-0 sync stream
    (the map maker's draws — `compute_sites`' stride is on the wrong
    stream);~~

    **The sync stream is read out of a dump, 2026-08-24 (fourth
    session)** — `GameLog::say_checksum` is the setup path's own trace
    and prints `game_random seed` at `check_all_level=14` (`rise.ini`)
    with `[Misc Logging] CHECKSUM=2`; run11 is the capture
    (`docs/ORACLE.md`, "The setup path's checksum trace is the RNG
    state"). Every draw from the lobby seed to frame 0 is counted
    (`docs/ORDERS.md` §9.2), the AI's personality is exactly the twenty
    draws between `Leader::init`'s checkpoints and reproduces the
    original's block from its own state, and `build_sim` installs the
    frame-0 word. With it — and the terrain heights, which only a
    `DUMP_ALL` dump prints (run3; `World::tile_z`, the `find_tcoord_z`
    seam retired) — **run9's best city site is the original's
    `(52,14)/370`**, pinned. The same stream shows the next gap: the
    original draws **~120 times a frame** (units, herds, farms, ammo —
    run12's per-frame states), the sim's sweep twice, so the script's
    `rand_int`s at frame 1 land elsewhere and run10 scores 744 on the
    true stream (the 268 was a lucky branch on the sim's own). ~~Next: the
    per-frame draws (`docs/AI.md` §12.1 item 2′);~~

    **The per-frame draws are read, 2026-08-24 (fifth session)** —
    `docs/SYNC.md`: `Game::do_frame`'s order, every `game_random` site by
    phase, and run12's four frames attributed draw by draw by replaying
    the LCG against the dump's own outcomes — the market's flux values
    place its 18 draws at 2–19 and nowhere else, the 40 animals' idle-anim
    variants match at 48–87 with zero mismatches, the herd's step and the
    farms' sprout pin the tail. The steady six a frame is
    **`Farms::inc_time`**, one draw per complete farm — and it is the
    missing half of the farm clock, which retires `FARM_GROWS`: a farmed
    cell gets two `0.005f` adds a frame and single precision reaches
    `1.0f` on the 201st, so the farmer re-targets on the log's frame 102
    exactly as run6 measured. Landed: `market.rs` (the price cycle),
    `farms.rs` (the crop cells, integer adds with the two pinned
    crossings), `gaia.rs` (the birds' sampling, the herd walk, herds from
    a `DUMP_ALL` sibling), and the harness's `Built::tick`, which reads
    the per-frame words, installs them, and prints the count on both
    sides: **frame 0 ours 48 / theirs 120, frame 1 54 / 54, 6 / 6, 7 / 6**.
    Run7's first script call and run6's AI now take the original's branch
    (pinned); run10 holds 268 with `1/9` linking. Next: the 72 of frame 0
    — the idle-anim draw per unit with the art's animation lengths (a
    table from the dumps' `GUY` blocks), animals as gaia units with the
    wander, the scouts' `think_scout` scan (needs the seen map), the
    4-draw tail, frame 3's one extra `do_move` draw (`docs/SYNC.md` §6);
    ~~a `Checksum Dump`/`Break` frame-window capture~~ — **done the same
    day (run13, an Opus drive)**: the window is `LogStartFrame`/
    `LogEndFrame` in `rise2.ini`, and `gamelog-run13-window-95-105.txt`
    has sim-frames 95–103 at 23, 28, 7, 6, 8, 18, 21, 6, 6 draws; ~~read
    its `FRAME 102` block first — the 100/101 spikes fit the twelve
    animals whose anim ends at 101 and leave no room for the farmers'
    re-target draws where §6.5 puts them (`docs/SYNC.md` §6)~~ — **read,
    2026-08-24 (sixth session), and every one of the window's draws on
    99–103 is placed by outcome** (`docs/SYNC.md` §4.1): the twelve at 100
    are **animation wraps**, one `% 100` per guy whose idle animation ran
    out, and they live in `Objects::inc_time` → `Guy::inc_time` →
    `set_anim(CHAR_DEFAULT, 0, 1)` — phase 7, after every `process` and
    before the farms — not in `do_idle`; the twelve type-411 animals are
    `HERDFISH` in twelve schools, never herd members, so they never reach
    the wander roll; the farmers' re-target *is* on sim-frame 101 as §6.5
    said, and its modulus is **4**, not 3 (all six farmers' twelve draws
    match `% 4` and nothing else; `FARM_SPAN` landed with run13's pins in
    `farms.rs` and `diff.rs`); the unit loop's owner rotation
    (`(frame + i) % 10`) is now in `Sim::tick`, and the AI's three farmers'
    re-target goals match the dump. The harness takes run13 as a
    `--sibling` and prints the window: 98, 102, 103 match at 6/6; the gaps
    are the scout's scan (95–97), the trained citizen's two creation draws
    (99), the fish wraps (100) and the sheep's arrival plus the scout's
    wrap (101) — ~~so the next mechanic is **the animation clock** (every
    guy's `cur_time`/`end_time`, the wrap's draw, the lengths read from a
    dump's `GUY` blocks as an input), which is worth 14 of the window's 16
    missing draws~~ — **the animation clock landed, 2026-08-24 (seventh
    session)**, `docs/ANIM.md` and `crates/sim/src/anim.rs`: `UnitAnimCat`
    read out of the executable's `.rdata` (the idles are one category,
    the seven walks one, the attacks one — the category decides every
    branch), `Guy::set_anim`'s idle roll with its thresholds and its
    arrival test, `init_real`'s roll, `Guy::inc_time`'s step, wrap and
    mirror (a scout's dog copies guy 0 and never draws itself), the
    animals as units of owner 8 with `Animal::do_idle`'s wander, and the
    lengths and pieces as an `Art` input read out of the dumps (gaia's
    piece is `(seed + o) % 3`, a citizen's takes the gender bit `o & 1`;
    the loop flags in `anim_graphics.xml` turn out not to matter for an
    idle, since both restarts roll). The harness installs the clocks
    beside the words and skips a unit walking on one side only. **Window:
    98, 99, 100, 102, 103 exact** (99 the trained citizen's two, 100 the
    twelve fish), 101 20/21 (the sheep's walk), 95–97 the AI scout; with
    run12, **frame 0 is 96 of 120** (the forty animals, the scouts' four,
    the woodcutters' four — the 24 left are the scan, the explore path and
    the tail). `tools/gamelog/anims.py` reads the clocks. Open in its §9:
    the woodcutters' un-stepped frame 0 (neither `inside_up` nor
    `unit_masks2` — a `GUYS=4` capture), whether the dog rolls on the
    unit's request, the unobserved lengths (a `DUMP_ALL` window at
    108–125 shows the chop, sow and walk wraps; a BHA reader settles all),
    `think_farm_animal`. Next: **`think_scout`** (the scan and the
    re-target are now the whole of 95–97 and of frame 0's gap but the
    tail — it needs the seen map), the sheep's walk through the
    pathfinder, then the draw-site trace below; frame 0's 4-draw tail is
    *not* a wrap (`docs/SYNC.md` §6);
    `tools/gamelog/{passes,framediff,farms,draws,anims}.py` are the
    window's instruments; ~~**the draw-site trace** — a log-and-continue breakpoint on `Random::get` recording
    the caller's EIP under winedbg in the bottle (Wine's dbghelp reads
    `rise.pdb`, so the callers come out named), one run for frames 0–3,
    which is the only way to place the draws that leave no outcome in a
    dump (the scouts' 23, the tail's 4) and de-risks the whole of §6 —
    proposed by the `lore` session, 2026-08-24~~ — **done, 2026-08-24
    (eighth session), and not with winedbg: `tools/trace/`** is an
    in-process DLL loaded by a patched copy of the exe (no debugger, no
    install), which trampolines the four LCG-stepping sites (the fourth is
    `MathUtilFuncSet::rand_real`, the script VM's, inlined on
    `game_random`) and plants an `int 3` on all 48,233 function entries for
    **function coverage** — cumulative over a run and per frame in a
    window. Run14 reproduced run12/13's counts exactly and **placed every
    draw of frames 0–3 and 95–103 by site** (`docs/ORACLE.md`, "The
    draw-site trace and function coverage"): the scouts are 24 at three
    `think_scout` offsets, the tail's 4 are phase-7 wraps at 110–113 with
    the farms last, frame 3 is farms only. The coverage side answers the
    question the anim audit raised — which claims rest on the reading
    alone: **of the 423 functions `docs/` cites by address, 156 have never
    run in any traced game** (`report.py … blind docs/`) — attrition and
    supply, all of combat, the AI's C++ producers, every order but
    move/gather/build, the trade economy. That list is the queue of
    behavioural runs, each one a trace: a border crossing, a fight, a long
    game past the script, run7's order stream replayed under the trace.
    **The border crossing is run (run16, 2026-08-24, an Opus drive from a
    written brief):** the whole of attrition and supply observed, twelve
    predictions written before the log was read and all twelve seen — the
    periods 8/48/24/0, the sixteenths, **489 of 489 ticks on the
    `(frame + o) % period` grid**, the wagon's shelter with its `0x40000`
    flag in `unit_masks2`, the Nubian clause dead (`docs/ATTRITION.md` and
    `docs/SUPPLY.md`, last sections; `tools/gamelog/attr.py`). The blind
    list is 99. What the run cost is the loop — an hour of typing into a
    chat box that drops four lines in ten — so ~~the next build is the
    scripted cheat channel in `rontrace.dll`~~ **built the same night:
    `rontrace.cmd`** (`docs/ORACLE.md`, "The cheat channel"; `tools/trace/
    README.md`) — run16 replayed from twelve lines in twelve minutes,
    unattended, every line on its frame, `!quit` closing the game by the
    menu's path, and the sheltered-from-the-start case run16 never reached
    seen (run16b). A scenario is now a file; the speed floor is the dump
    (~3 frames a second at `UNITS=3`), so window it. ~~Next: run17,
    combat~~ — **run17 done the same night through the channel**
    (`docs/COMBAT.md` §16, `docs/ORACLE.md` "The combat run"): every
    unit-on-unit hit in 2,600 frames is a size the formula predicted
    before the log was read — 122 on a wagon from any bearing (CIVILIAN,
    no flank), 48/85/117 hoplite on hoplite, 32/58/85 slinger on hoplite,
    53 on a scout — the flank sectors settled by damage, the projectile
    draw sites named. The blind list is 95: the order commands other than
    move/gather/build, air, patrol/follow/guard, the group actions, and
    `is_fleeing` (a wagon's flight is something else). ~~**Next blind runs,
    each a file:** run7's order stream replayed under the trace (the
    command processors), a long game past the script (the AI's C++
    producers at `LEADERS=9` around a sweep)~~ — **the long game is run
    (run18, 2026-08-25)**, and it is the first dump that has ever shown
    this mechanic run: `docs/AI.md` §15, `docs/ORACLE.md` "The producers'
    run", `tools/gamelog/steps.py`. Nine predictions written first and all
    nine held — the script exits at sim-frame 6376 from `defensive`'s
    `case 29` (Classical Age *bought*, `ages_get()` still 0); the two step
    ladders are `1,2,3,4,5,6,7,8,0` on the sweep the script dies on and
    `1,3,4,5,6,7,8,0` on every later one, frame for frame; the make list
    fills slots 0, 5 and 8 and nowhere else, which is `make_me`'s ranked
    four over `list[cat]` exactly; and **five expiry draws decide five
    slots by `% 3 == 0`** against the trace's own seeds, settling an arm
    §2.6's prose had backwards (`ai_make.rs` had it right — the pin fails
    if the prose is restored). The lever that made it cheap: **`ffwd` is a
    presentation switch the sim never reads**, so with the per-frame dump
    gated off (`LogStartFrame=0 LogEndFrame=0`) the sim runs at **~500
    frames a second, 35× real time** — 24,000 frames in 45 seconds. A run's
    whole cost is `dump blocks × ~2.4 s` (~1.7 MB each at `LEADERS=9`), so
    budget the window and fast-forward the rest. Blind list **95 → 89**.
    **run19 followed the same day**, 19 blocks and 90 seconds
    (`docs/AI.md` §15.6) — the window around frame
    8182, where the AI is in the Classical Age with a full make list: the
    **second pass** (steps 9–11, never seen before; `make_stuff` is
    `production_ai+0x1fa` at step 8 and `+0x236` at step 11), two purchases
    in one call, `research_techs` reaching the 9,999,999 overflow guard and
    drawing nothing, the runners-up shifting through slots 1–3, and a
    scholar **kept at a non-head slot and cleared at the head on the same
    `% 3` residue** — which isolates `expire_all`'s head clause from the
    probabilistic one on a single type. Nine expiry observations, nine
    agreeing. Still to run, each a file: run7's order stream replayed under
    the trace (the command processors), a mounted attacker for the cavalry
    flank reduction, a caravan, and a window at frame 576 for
    `found_cities`' purchases (which needs the script's caller told apart
    from the producer's by the dump's step). Then the AI queue below.
    Next for the AI: ~~fold §14 into §2~~ — **done, 2026-08-25**: the
    thirty-odd corrections the seven implementation workers found are now
    *in* §2 (the census's radius base and its two aliased locals, the
    make list's danger map — the same misreading in four places — the
    site insertion rule, the distance loop's `return`s, research's
    stockpile gates and inverted government polarity), and
    `create_units`/`create_buildings`/`produce_building` were **promoted
    to §2.18–§2.20** rather than folded, because §2 had never covered them
    at all; §14 is now a map of where each went. §2.9 lists what is still
    unread at document level (`upgrade_units`, `produce_unit`,
    `produce_upgrade`, `produce_spell`, `check_income`, `unit_prod_value`,
    `check_transport`).

    **The loader's half of the producers' seams landed, 2026-08-25** —
    `docs/DATALAYER.md`, "The derived words no column carries", and
    `crates/sim/src/ai_load.rs`. What made it cheap: **the type dump carries
    the derived words too.** `UnitType::log_data` prints `role`,
    `unit_flags` and `unit_flags2`; `BuildType::log_data` prints
    `build_flags`; `TechType::log_data` prints the **eleven `ai[scan]`
    weights**. So run3's `DUMP_ALL` dump is a field-by-field oracle for the
    whole item and `rondata --types <dump>` now reports **0 differences on
    all four** — 364 roles, 364 `unit_flags2`, 129 `build_flags`, 85 × 11
    weights, with the 493 × 493 combat table still 0 beside them. Landed:
    `UnitType::cols` (`determine_roles`, `init_final_flags`, and the caster
    bit `craftrules.xml` seeds and `init_spellcasters` walks down the graft
    chains), the six derived `build_flags` bits, `TypeDef::ai[11]`
    (`compute_ai_values` + `add_preq_ai`), and the seams in `ai_units.rs`
    and `ai_research.rs` closed behind them. Three corrections came with it,
    each one a claim the reading alone had wrong: **`build_flags &
    0x8000000` is live** — it is `TechType::set_research`'s mark, not a
    `BUILD_FLAGS` digit, so `create_buildings`' civic block is not dead code
    and `is_military_trainer` is not dead either (`docs/AI.md` §2.19); the
    build queue's capacity is **three-way, 20/10/2** (`docs/CITIES.md`); and
    the unit table has a **name group** — `Types::init` loads it in five
    passes and, from the third, gives a record whose `NAME` repeats the
    previous one's the *first* record of that run, so sixty-four records
    take another row's columns and four shipped rows (a General's `FLAGS`,
    Riflemen's `ARMOR`, two `LOS`, the Howitzer's `SPLASH_PERCENT`) are read
    past (`docs/FORMATS.md`).

    ~~Then: the blind second reading — which can run from any
    session, since the readers are subagents and only the *brief* must
    avoid leaking claims~~ — **done, 2026-08-25**, `docs/audit/
    2026-08-25-ai.md` and `docs/AI.md` §16. Seven blind readers on Opus 5 in
    two waves, adjudicated in the main thread; 590 claims, 42 doubly
    confirmed, 31 corrections, five of §13's open questions closed, **three
    corrections to `crates/sim`**: the coastal ring is centred on the
    original cell and not the slid one (which **neither** reading had — the
    document and the implementation were both wrong), difficulty 2 reads the
    *previous* age's stamp, and `Site::dist` carries the last enemy capital's
    quotient because the original clobbers the slot. The last of those was
    found by **widening the harness's `SITES` check from the best site to all
    ten records** — it failed on run9 the first time it ran, which is the
    cheapest correction the audit produced and the model for the rest.

    **Two things are owed and are not done:**

    1. **A guard for the ring fix (audit B4-k).** No capture on disk can
       meet it: the branch needs `world.sea_map > 2` and run9's map reports
       `sea_map 1`, so it is dead in every dump we have. It needs **a run on
       a many-islands map** — worth doing for its own sake, since the whole
       sea half of the AI (docks, transports, `check_transport`,
       `reg_naval` and B5-b's `% 63` aliasing) is on the trace's
       never-executed list.
    2. **Widen the make-list diff the way `SITES` was widened.** Audit B7-f:
       `LeaderData::log_data` dumps all eleven slots at `LEADERS=9` under
       `t val escrow city up o num cat wx wy`, so every §2.6/§2.11 claim is
       falsifiable from a capture we already have.

    **And the leak, which is the next session's first task**
    (`docs/DECISIONS.md` entry 21): a subagent inherits this file, and this
    file names the findings a blind reader is meant to re-derive — so the AI
    audit's protocol leaked, and every future one will unless the queue moves
    out of `CLAUDE.md` and into `docs/QUEUE.md`.

    `docs/audit/README.md` also owes a paragraph each to the
    pathfinder, commands and recgame audits it never got. And
    `gather_max` for non-flat buildings is **not** a seam after all:
    `BuildTypeData::calc_gather` is a thousand-line terrain scan with mining
    lists and cliff tests, so it is a mechanic of its own and belongs with
    `docs/ECONOMY.md`'s gathering.

## 2026-08-25 — the meta-docs, the AI audit's third pass, the method

Session "attrition" on Fable 5, after the AI audit's Opus session. No
simulation change; everything below is observed or is a decision.

- **The Opus 5 tranche stamped** (`9e123c0..abd365c`, eleven commits,
  ~4,100 lines). The three verdicts that changed `crates/sim` re-verified
  from their own citations: the coastal ring's centre from
  `compute_site_stats` lines 233–234 and 253–254, `Site::dist` from
  `local_34`'s three writers, the difficulty-2 stamp from the listing at
  `0x6c6e94`/`0x6c6eac` and `TypeData::is_age_type` at vtable `+0x34`.
  Nothing retracted. `cargo test` 492 + 82 + 3 + 3 with the install
  attached, 0 ignored; clippy and fmt clean; `rondata --types` against
  run3 39 checks, 0 differences. Recorded as `docs/audit/2026-08-25-ai.md`,
  "Third pass".
- **The queue left `CLAUDE.md`.** `57546c0` the verbatim move (847 → 304
  lines); `caf6bf4` the rules stripped of the findings they cited as
  examples (288), with one rule added — this file must never name what a
  blind reader is meant to re-derive. Decision 21 executed.
- **The verification split agreed with the user** and written as a rule
  (`746bb26`): diff first wherever a dump exists; a blind reading scoped to
  what no run reaches, its readers briefed to output assertions; the soak a
  determinism guard, never fidelity. Item 13 — differential fuzzing against
  the original, the soak's generator through the cheat channel into the
  harness's diff — added with a brief that says what must become true first.
- **Lore's review** of the layout (the meta/wiki session, answering from its
  corpus of other repositories' continuity files): one queue file is right
  *on the condition* that a struck entry is one line and a pointer, which is
  how `CLAUDE.md` reached 847; the chronicle belongs in a dated append-only
  journal, this file; the model split belongs in git, not in a memory file;
  a file map with an inherited-by-subagents column; a "last verified"
  banner and a literal opener on the handoff. All adopted, in `docs/
  QUEUE.md`, `CLAUDE.md`, decision 22 and `docs/audit/README.md`.
- **Checked, not assumed:** whether a subagent inherits the memory
  directory as well as `CLAUDE.md`. A probe agent on Haiku, told to read
  nothing, quoted the memory index's bullet lines verbatim — so **the index
  is inherited, hooks and all**, and is a second leak surface the
  `CLAUDE.md` cut had not closed. Its hooks are now finding-free, and the
  two memory files that duplicated `docs/ORACLE.md` are deleted.

## 2026-08-25 — the make list diffed whole (owed item 1)

Session on Fable 5, the queue's opener taken as written. One sim change,
one guard, two document corrections.

- **The widening.** Not the best slot but the window: every consecutive
  pair of `LEADERS=9` blocks in run18b (216) and run19 (18), all eleven
  `MAKEOBJECT`s × ten fields, each step of the production ladder replayed
  with the sim's own `clear` / `make_me` / `expire` on the previous record
  and compared to the next; the frames each class covers asserted so the
  test cannot pass by calling everything unchanged. Run9's frame 1 pinned
  to the init record beside the census. `docs/AI.md` §15.7.
- **What it found.** `MakeList::clear@006c9db0` rewrites the whole record
  (`val −1`, `num 1`, the rest cleared) and `Array<MakeObject>::init`
  fills the same; the sim cleared `t` alone with `val 0, num 0` behind it.
  Behavioural, not cosmetic: the first sweep's citizen is offered at
  **`val 0`** — §15.2 had copied the second sweep's 714 onto it — and
  lands only because `−1 < 0`. Corrected in `ai.rs`; the guard was run
  against the old `clear` first and failed at dump-frame 6577 naming the
  three stale `val`s. Also read off the replay: `research_techs` must
  offer its cat-8 line first (two of 24 orders reproduce 8179); step 9's
  `create_units` offered a merchant the end state hides, overwritten at
  the head by a scholar at `num 5`; the expiry writes `t` alone and its
  ghost records travel intact through `make_me`'s shift.
- **The method note.** Second time running that a whole-record diff found
  in twenty minutes what a reading had passed twice (`SITES` was the
  first). Both were *predicates on the empty case* — what a cleared slot
  holds, what an unscored site's `dist` holds — which a fixture built from
  the same reading cannot see because the fixture's empty is the
  reading's empty.
- `cargo test --workspace` with the install 492 + 83 + 13 + 3, 0 ignored;
  clippy and fmt clean; `rondata` survey exits 0. One `ai_make`
  fixture had leaned on `num 0` making `can_pay` vacuous and was given a
  purse.

## 2026-08-25 — the `--types` check as a test (owed item 1, the second)

Session on Fable 5, the queue's opener taken as written. No simulation
change; one guard moved from a flag to a test.

- **What was wrong.** `rondata --types <dump>` compared twenty things —
  the loader's 364 unit and 129 building `Kind`s field by field, the four
  derived words, the 85 techs' eleven weights, the whole 493 × 493 table —
  and only ran when someone passed the flag. The definition of done's
  `cargo run -p rondata -- <install>` never reached it, so the guard for
  `docs/COMBAT.md` §5 and `docs/DATALAYER.md`'s derived words could lapse
  by omission and had no way of saying so.
- **The move.** The comparison left the binary for the library
  (`rondata::typesdump::compare`, returning a report of notes and checks
  rather than printing); the CLI prints that report, unchanged to the
  character (39 `[ok]`, the same 20 under `--types`). The test
  `typesdump::tests::run3_s_type_dump_is_the_program_s_on_every_check`
  reads run3's 152 MB dump with the streaming reader in ~5 s, asserts the
  dump's shape first (364, 129, 85, the table present — so a dump with
  nothing to check cannot pass), then that all twenty checks hold. The
  install and dump helpers the `SITES` pin and the `load.rs` tests each
  carried are now one `testenv` module in `lib.rs`.
- **Made to fail once.** The loader's siege predicate flipped from
  `0x20000` to `0x10000`: the test named the 16 siege units and 3,664
  table cells, then was restored. The first run also failed on its own
  pin — the check count was written as 24 and is 20 — which is the pin
  doing its job on the author.
- `cargo test --workspace` with the install 492 + 84 + 13 + 3, 0 ignored;
  clippy and fmt clean; `rondata` survey exits 0; `--types` against run3
  39 checks, 0 differences.

## 2026-08-25 — the audit README's three missing paragraphs (owed item 1, the third)

Session on Fable 5, the queue's opener taken as written. Documents only.

- **What was owed.** `docs/audit/README.md`'s record narrated every second
  reading from the first pass through the AI, except three it named in a
  placeholder at the end: the pathfinder (2026-08-23), the recorded-game
  container and the command payload encoding (both 2026-08-24). Each had
  its audit file; none had its paragraph in the method's record.
- **Written**, from the audit files, the journal's struck entries 8–10 and
  the commit trailers (all three adjudications on Fable in the main
  thread; the blind readers Opus 5). In date order between the orders
  ratification and the animation clock. What each paragraph keeps: the
  pathfinder's lesson that a flag left uninterpreted by both readings was
  already settled in `docs/ORDERS.md` §8; the container's package-stream
  start reached by two algorithms with no shared code, and its correction
  in the writers (raw, gzipped only in `finalize`); the commands audit's
  three corrections of one kind — what can actually be emitted — settled
  by sweeping every `add_command` caller. And one thing the record had not
  said: the recgame and commands blind reports were lost with the job's
  tmp, against the orders audit's own third lesson; the pathfinder's are
  at `~/ghidra-projects/reports/pathfinder/`, one directory over from
  `reading/`.
- The placeholder removed; the queue's owed list down to two.

## 2026-08-25 — the islands map: B4-k's guard, and four things it was hiding (owed item 1, the last)

On Fable, with the user at the machine (the lobby's three clicks and a
combo were driven by `cliclick`; nothing else needed a hand). The owed
item said "a many-islands run through the cheat channel, then the B4-k
assertion", and that is what happened, but the assertion took four
corrections to `crates/sim` to pass, and only one of them was about the
ring.

- **Run20** — East Indies under `DUMP_ALL` for frames 0–3, its own setup
  trace, heights, herds and fog; seven minutes. **Run21** — the same lobby,
  dump off, `!ffwd` to 24,000; two minutes. `docs/ORACLE.md`, "run20 and
  run21". The map style could not be set from `check.ini` (never could —
  every earlier run was Great Lakes, not the Sahara the file named) and
  Save to Profile did not outlive a killed process; the combo was clicked
  both times.
- **The guard**: `run20_s_islands_sites_walk_the_coastal_ring_from_the_
  original_cell`, the ten-record `SITES` diff plus the per-region census
  for *every* region. It passes; with the ring re-centred on the slid cell
  it moves the record by a slot. `docs/AI.md` §15.8, the audit's fourth
  pass.
- **What it was hiding**, each found by the diff failing and settled at
  its own citation: `world+0x34` is the map style's `SEA_MAP` class (0–4),
  not a landmass count — every reader in `crates/sim` renamed, and the
  document's five "landmasses" glosses corrected; `land_key[]` is the
  static `BASELAND, SANDY, OCEAN, NONE`, and the harness's first-appearance
  numbering had been right on run9 by luck; `is_ocean` is by cell kind;
  and `was_seen` is not a seam — it has a territory arm (an ally's cell in
  a region with cities or forts is seen unfogged) that the dump's fog grid
  alone cannot reproduce, and without which the sim scored no site at all.
  The fog grid is now loaded from the WORLD dump and the arm is in
  `ai_sites.rs`.
- **The blind list, 92 → 87** over ten traces — and the discovery that
  the sea half run21 lit up (`check_transport`, the docks,
  `Army::do_transporting`) was invisible to it, because §9 cites those
  functions without an address. Named by address now.
- **The one loose end, pulled the same evening** at the user's prompt
  ("file it or fix it?"): the world's `sea_map 4` on a style whose file
  says 3. The first writer search had grepped for one pointer name;
  `World::analyze_map` stores through another, and its rule — starts on
  separate landmasses and the free islands holding twice the players'
  land promote a 3 to 4 — reproduces run20's 4 from the dump's own region
  sizes (1097 ≥ 1058). Twenty minutes, and the reason to do it before
  `/clear`: the decompile paths, the string-table decoder and run21's
  setup trace were all still in hand.
- **Open, honestly**: `reg_forts` and `was_seen`'s leader-flag exits
  unmodelled; the sea half read by no one yet.
- The queue's owed list is empty. Item 12 is done.

## 2026-08-25 — the sea half: transports and docks, in one session

On Fable, unattended: the opener offered the reading or the capture and
the session took both, in the order that turned out to matter — read
first, stage the capture from the trace's own frame numbers, spawn the
blind readers while it runs, adjudicate against both. Under two hours
from `/clear` to commit.

- **The reading** (`docs/TRANSPORT.md`): a leader's transport permission
  is three bits of `leader_flags` that `check_transport` sets as a block
  once the leader holds the bonus's prerequisite (Written Word —
  `rules.xml`'s third `TECHBONUS`, found in the data) *and* a dock; the
  unit's `0x800000` bit follows at birth and on every change; the docks
  registry is twenty slots a leader with a `dock_mark`, and a finished dock
  spawns a gull with **two draws** — run21's trace had both, in order, at
  frame 3579. `think_civilian_transport` is the citizen's island choice,
  `do_mustering`'s expand arm hands an army to `do_transporting`, which is
  a one-shot region retarget; `init_navy`/`send_navy` are the navy's two
  hooks.
- **The implementation** (`crates/sim/src/transport.rs`): the level, the
  predicates, the registry with its draws, `needs_transport`,
  `is_dock_tile`, and the coast masks from the cells. Two census seams
  closed, `carry` and fishermen wired. The unit AI's island choice and the
  army's step are documented and deliberately not built — they need the
  `think` and the `Army` family.
- **The widening that paid** — run20's `CITY` record, whole: every step-13
  field matches the harness, `dock_tile` included, except `space[0..1]`
  (48 against 58), which is `space_at_corner`'s fifth argument on ten
  cells — a placement finding, pinned as a known divergence in the test
  and owed in the queue.
- **Run22**: `tools/gamelog/window.py stage 3579 3582`, five `cliclick`s,
  eight minutes: the `DOCK` record, the 14-unit `0x800000` flip from block
  3579 to 3580, the human's six without. And the day's largest fact came
  from the record, not the readings: the dock's `reg` is **65, the sea** —
  a dock's centre is on water, so the `reg_docks` counter both the first
  reading and the blind reader described is never incremented for it.
  Owner 9's gull is not in a `DUMP_ALL` block at all.
- **The second reading** (`docs/audit/2026-08-25-transport.md`): two Opus
  readers, 114 claims, six verdicts against the document — an air unit
  skips a gate rather than failing it, `+0x14` is `num_captains`,
  `Dock::close` leaves `gull_o`, `num_coasts` on a sea region is 1 (so the
  census's "more than one coast" clause is dead), the apron's eraser, the
  tile rule's only caller. Two changed Rust. A.41's own sentence inverted
  its own formula; the formula won.
- **Blind list**: 471 cited, 96 never — up from 87 because 33 addresses
  were added; the seven never-run ones are named in the queue with the
  scenario that would light each.
- Item 14 is done; the queue owes the `space_at_corner` finding.

## 2026-08-25 — `space_at_corner`'s walk order (the owed finding)

On Fable, one sitting, from `/clear` to commit. The queue offered the owed
finding or `docs/ARMY.md`; the owed one had a guard that would flip, so it
went first.

- **The diagnosis was wrong before the fix started.** The queue said
  `check_building_wcoord`'s fifth argument; `space_at_corner@006b27f0`'s
  body never reads it. The function walks its sixteen tiles through
  `grid_index_x/y`, and the early-out is on "the first four" of *that*
  order. The PE settled it in the two-minute recipe (`S_LDATA32`, the
  section table, `xxd`): centre 2×2 first. Every 3×3 of a 4×4 contains the
  centre four, which is why `grid_threes` is four rows of five and why the
  early-out is safe. And the ≥ 8-blocked branch returns 2, not 0.
- **The guard flipped before the pin came out** — `space[0]/[1]` ours 58
  (was 48), theirs 58 — on the first run after the walk order landed; then
  the pin was retired and the `CITY` record is compared whole. Three unit
  tests, the walk-order one failing on the old code at a blocked corner
  tile; `check_building_wcoord`'s occupied/`blocked == 0x10` gates added
  from the same listing. `docs/AI.md` §15.9.
- **The method note.** A known divergence pinned in a test with its
  suspected cause named is the right shape even when the cause is wrong:
  the pin flipping is what proved the real one. And the lesson the audit
  README already carries — read the body before the caller — was the one
  the queue's diagnosis skipped.
- The owed list is empty. Next is `docs/ARMY.md`.

## 2026-08-25 — armies: the state machine, four runs, and the family's audit

On Fable, one sitting, `/clear` to commit. The queue's opener: open
`docs/ARMY.md` from the transport audit's army rows and run21's frames
past 14586.

- **Read whole in the main thread** — 5,500 lines of decompile, the
  listing for every dropped register argument (`find_angle(dx, dy)` is
  `ecx, edx`; the engaged radii are 0xc00 from the army's point and from
  the centre of gravity), the type records for every name. The shape that
  came out: sixteen slots a leader, a 256-frame tick phased by slot and
  owner, one implicit group, counts from the units, a muster point that
  halves its distance to the target each tick, and `find_target` as a
  scored search over every city with a `Random::get` per candidate.
- **The capture came before the reading finished, and it was cheap in the
  end and expensive in the middle.** Run23 declared war at frame 6000 and
  changed **nothing** — every per-frame RNG word identical to run21's,
  because a Quick Battle starts at war. Ten seconds with `rngcmp.py`
  would have said so; it is in `tools/gamelog/` now. Run24 dropped seven
  human hoplites beside the AI's capital instead: the capital fell at
  13125, the AI was defeated at 16488, and nine of the family's
  never-executed functions ran. Three `DUMP_ALL` windows on run24's
  frames followed, chained by one script, thirty minutes unattended.
- **What the records said.** The AI's third army is a **navy** that, on its
  first tick after the raid, targets the AI's **own attacked capital** —
  §12's `L == me && attacked → ×10`, `capital → ×10`, visible in the
  record as `target_o 2000, target_who 1`. And `emergency` at 12129 closed
  army 1 through the one path the sim's seam cannot take: the muster-spot
  ring search failed at a city where it had succeeded for 9,000 frames,
  `do_mustering` set marching, `do_marching` found no attacker, `close`.
  The ring search is the seam the first capture reaches; `docs/ARMY.md`
  §18 says so and what would close it.
- **Landed.** `docs/ARMY.md` (19 sections), `crates/sim/src/army.rs` (the
  record and the decisions; the group orders, the ring search and the
  fort pass as named seams), wired into the census's step 16,
  `create_units`, the unit think, city close and capture, defeat, damage
  and war. `rondata`: run20's frame-1..3 `ARMY` record equals the
  harness's step 16 **field for field**; run22's two records are `init`'s
  with the ring search's muster cells. `SYNC.md` §3.5 has the new draw
  sites; `AI.md` and `TRANSPORT.md` point here. Commit `2240e6d`.
- **The second reading**: two blind readers on Opus 5, one wave, briefed
  with the captures. 142 claims; eleven verdicts changed the document and
  six changed `army.rs`, and the largest went against a claim the first
  reading had marked **settled** — the two averages in `find_target` are
  over `sea_combat`, because the listing's loop register sits at the
  leader plus 8 and its `+0x944` is `+0x94c`. Both readers went to
  `rise.pdb` for the enums the export does not ship, and that was worth
  five verdicts; a "settled in the listing" is a claim like any other.
  `docs/audit/2026-08-25-army.md`.

## 2026-08-25 — the muster-spot ring search, and why two armies died

The seam the army's captures had reached (`docs/ARMY.md` §18): the ring
search of `find_muster_spot`, which the harness had stood in for with
the original's own fallback. One session, on Fable.

- **The premise was stale.** The queue said the search waited on "the
  cell classes the sim does not yet load"; `World` had carried every
  cell's `flags`, `land`, region and owner since run20. What was missing
  was the score table and the search itself, and the score table turned
  out to be a constant: `lands[class].+0x100` is `LandData::move_rate`,
  which `Lands::init` writes as `0x100` to all nine lands and nothing
  else in the export writes. So a candidate scores 256 per admissible
  neighbour and the walk order decides — which made the search a
  question about the map, and the map is in every `DUMP_ALL` block.
- **Three things the readings shared and the decompile does not.** A
  land army is out on a coastal neighbour, not merely unscored; the
  chase falls through into the ring search rather than returning; and
  the return value is the last candidate's verdict, not "found" — the
  listing's `mov eax, [ebp-8]` at `6f6824`, settled in a minute. The
  first two would have kept every own-city search out of the ring; the
  third is a quirk no block shows, pinned by a unit test from the
  listing. The flag bits behind the classes were solved from run20's
  3,600 cells — each level-4 word appears exactly when its bit is set —
  and the 7 × 7 walk table was read out of the executable at the PDB's
  address (`world::MOVE_49`).
- **The blocks explained themselves.** The window dumps carry the world
  per frame, so a loader (`army_tests::scene_at`) that builds one
  block's map, cities and armies was enough to run the search where the
  original ran it. Run22's two muster cells come out in the order the
  game found them; at 12129 Norwich has exactly one admissible cell in
  its two rings and army 0's chase muster sits three cells from it — the
  same-owner spacing rule is what closed army 1, and the `CITY` records
  show the `0x2000` mark going on between the blocks; at 15100 the same
  cell is the human's territory. The navy's (46, 58) comes out of both
  blocks, the angle differing by `find_target`'s turn-about. Six
  searches, cells and angles equal.
- **Landed.** `army.rs`: `find_muster_spot(o, who, flag)` whole, the
  `no_muster` mark on `City` and its division in `find_target`;
  `world.rs`: `cell` (the flag legend), `MOVE_49`, `World::is_ocean`;
  `rondata`: `world_from` factored out of `build_sim` so a frame block's
  `WORLD` can build a world, and the three replays beside the unit
  tests. `docs/ARMY.md` §13 rewritten, §16.5, §17, §18; the audit's
  B.56 closed with a note on the three amendments; `ORACLE.md` carries
  the flag legend.
- **A method note.** The scene loader is the first time a mid-game
  block has fed the harness at all, and it cost an afternoon, not the
  "harness that can stage a frame-12000 state" item 13 imagines — for
  a *single function*, a block's records are the state. `find_target`
  at run26's 12024 is the obvious next use.

## 2026-08-25 — `find_target` replayed on run26's block, and the gate it failed

The obvious next use, taken: the navy's first live target choice
(`docs/ARMY.md` §16.5), one function of one block, on Fable.

- **The trace named the draws before anything was built.** `report.py
  … draws 12024` on run26's trace: sim-frame 12024 opens with three
  `game_random` draws, all `Army::find_target+0x7df` — the per-candidate
  `% 200 + 900` — from `0x63ffe763`, the block's own `game_random seed`,
  and no coin. So the candidate count (three, on a three-city map), the
  order, and the stream's word after the function were known before the
  loader was extended, and the test was written to them.
- **The loader grew by what §12 reads.** The block's frame and sync word
  (`sim.frame`, `sim.rng`), the diplomacy table from `diplos[scan]`
  (0 war, 1 peace, 2 allied, **2 on the diagonal**), `defense_mod`, the
  two census counts, `frame_attacked`/`attacked_by`, the personality's
  `raid` and `early_army`, and each city building's `damage` from its
  `BUILDDATA` — read from the block itself, since `LeaderDump` carries
  the level-0 fields only. An afternoon's extension, as predicted.
- **It failed on the draw count, and the failure was a predicate.** Two
  draws, not three: the sim's diff-≤-1 gate skipped an enemy's city
  unless the AI had founded it — the inverse of the listing, which
  admits an enemy's city unconditionally and applies the founder test to
  mine and my allies'. B.25 had named the filter without reading its
  sense. The same re-reading, with `types.txt` in hand, retired the
  first reading's "my ally-slot": `LeaderData +0x8` is `who`, the
  decompiler writes `L == me` twice, and the `diplos` diagonal makes `me`
  pass "allied both ways" — so my own untroubled city takes the size
  factor, which the sim had excluded. And the `diff == 1` clause sits
  behind a `goto` the decompiler's `else` hides: it is reachable only at
  difficulty 1, where the sim had it dead. Three predicates, no
  arithmetic.
- **The gate's other half was `do_mustering`'s tail.** The record's point
  is one cell south of London, the AI's own cell, where the navy would
  not be aggressive and the gate would draw a coin; the original drew
  none because `do_mustering` had already moved the point to the muster
  cell's centre — ocean nobody owns — before `do_marching` ran. The
  12025 record's `angle 292028416` is that tail's copy of the old
  `muster_angle`. The test applies the tail by hand and says why.
- **Landed.** `army.rs`: the three predicates; `rondata::diff`: the
  loader and the replay, which asserts the target, `rally_dist`, the
  point, the muster cell and angle, the untouched stamps, and the word
  three draws on. `docs/ARMY.md` §12 amended in place, §16.5, §17, §18,
  §19. All checks green; the guard was made to fail first.
- **What it does not establish.** The score itself: London wins by two
  hundred to one, so every multiplier but its three could be off by a
  factor and the block would not say. §18 names the capture that would.

## 2026-08-25 — the group orders: the army's other half, and the run that made `engagement` fire

The queue's next item, taken whole: `docs/GROUPS.md`,
`crates/sim/src/group.rs`, and run28. On Opus 5 in the main thread —
Fable is being saved for the next architecture, so this session's
reading, implementation and adjudication are all one model's, and the
document says so.

- **The layer, not the mechanic.** Nothing in the original gives an order
  to a unit: a click, an army tick and `Unit::go_to` all build a `Group`
  and call a `Group::action_*`, which walks the members and calls
  `Unit::add_*_order`. `docs/ORDERS.md` §8 had read the entry from the
  command stream; this reads the layer — the record (the PDB names every
  field of `GroupData`), the pool of 64 groups a leader, membership, and
  the five actions `docs/ARMY.md` cites. 3,500 lines of decompile.
- **The find is §6.5, and it is a predicate.** `action_move_near` has an
  **AI branch** neither `docs/ORDERS.md` §8.2 nor the army's own reading
  had: `!human && group.army >= 0` turns it on, and then `army.hurry`
  splits it. Hurrying, with a friendly city within `0x200` of the
  destination, a **supply wagon, siege unit or hero is sent into that
  city** — `add_garrison_order`, or a move to a spot around it — while
  the line marches. Not hurrying, a **siege unit whose order is already
  `ATTACK` is left entirely alone**: orders not cleared, no move, no
  path. Two predicates, three times over in the function (the clear, the
  order, the path), and both now tested — each made to fail first by
  `&&`-ing `false` into it.
- **`action_siege_attack_to` is a nicer mechanic than its name.** The
  siege units become a sub-group; with none, the first supply wagon
  joins it, and with none of those the first hero. Then the sub-group's
  member with the smallest total Manhattan distance to the whole group is
  the **anchor**, the sub-group attack-moves, and everyone else
  `action_guard`s the anchor. With no siege, no wagon and no hero — every
  traced army — it degrades to a whole-group attack-move, which is what
  the trace shows at 10232.
- **`Form::compute`'s slot table is a declared seam.** 1,200 lines across
  four functions, floats in the middle, and no capture pins its output;
  it decides *where within the formation* each member stands and nothing
  the army's behaviour turns on. Every member is given the group's own
  destination, `docs/GROUPS.md` §6.4 and §12 say so, and §13 names the
  `UNITS=3` capture that would settle it — `Form::compute_dests` runs on
  **frame 0**, so it is a seam of arithmetic, not of reachability.
- **The run was predicted, then staged, then fired.** The path to
  `Army::engagement` is narrower than `docs/ARMY.md` §6 reads: the
  dispatch is `do_forming` **or** `engagement`, and `march_to_target`'s
  engaged arm does not clear `0x10`, so an army that arrives and fights
  never reaches it. The one path is `do_mustering`'s release, whose tail
  overwrites `status` whole. So: run24's game with six hoplites dropped
  on **army 0's own point** — run27's block gives it as tile (180, 192),
  seven units, mustering — at 15020–15030. `Army::engagement@006f5160`
  entered at **15100**, army 0's own tick frame, and the frame's coverage
  names the chain down to `Group::action_attack` →
  `Unit::find_melee_target` → `Unit::add_attack_order`. Three minutes,
  trace only, no dump.
- **A correction the same check produced.** `docs/ARMY.md` §16.4 and §18
  said `engagement` had never executed in any traced game. It had —
  **run16, frame 6652, since 2026-08-24**. The claim was true of the four
  islands runs and had never been asked of the corpus, which
  `report.py … blind docs <every log>` answers in ten seconds. The blind
  list is only honest when every log is passed to it; the one after run28
  leaves `docs/ARMY.md` three functions and `docs/GROUPS.md` **none**.
- **And three more `docs/ARMY.md` predicates, corrected while wiring the
  callers.** Writing `march_to_target` against the decompile rather than
  against §9 caught: the move-or-siege choice is **not** "`hurry` alone"
  but §8.3–8.4's whole test, `is_ally` clause included; the 90 %-damage
  test's sense was **inverted** (`damage >= hits × 9/10` — a building
  nearly dead, not nearly whole); and it applies only to a **city
  centre**, not any building. §8.4's own friendly test is `is_ally`, so a
  leader merely at peace does not qualify — the first reading had written
  "mine or an ally's" and the sim had implemented `!is_enemy`. The audit
  README's recurring lesson, for the fourth mechanic running: the
  arithmetic survives, the predicates do not, and it is *writing the
  caller* that finds them.
- **run29 turned the coverage into an assertion.** The same scenario
  under a `DUMP_ALL` window at [15100, 15103). Army 0's two blocks are the
  tick, field for field: `status 1 → 32`, `city 1 → −1`, and the point
  from the city's `(34656, 36960)` to the muster cell's centre
  `(34944, 37248)`, `muster` untouched — `do_mustering`'s common tail
  whole. The sim's own `do_mustering` now reproduces all four, and asserts
  what the whole run was for: **no `FORMING` bit survives the release**.
  Made to fail by ORing `0x10` into the tail. `scenes()` parses a 250 MB
  window once for both blocks.
- **Landed.** `docs/GROUPS.md` (13 sections), `crates/sim/src/group.rs`
  (9 tests), `army.rs`'s `do_forming` / `march_to_target` / `engagement` /
  `send_here` / `charge` / `set_stance` / `close`'s halt (7 tests, one
  end-to-end: the order the group issues is stepped and the unit is nearer
  the muster spot 64 frames on), `docs/ARMY.md` §16.6, §17, §18. 532 sim
  tests and 93 diff tests green, clippy and fmt clean, `rondata` exits 0.
- **What it does not establish.** The slot table; `FormData::type_cat`,
  which picks a mixed group's leader; and the *positions* of a traced
  army's units, because the harness cannot yet stage a frame-15100 unit
  list — run29's `UNITS=3` half is on disk for whoever extends
  `scene_at` to read it.
- **The second reading was launched at the end, not alongside — and that
  was a plain miss.** The previous session's own handoff had recorded the
  shape to repeat: *"read, stage the captures from the trace's frame
  numbers, spawn the readers while they run, adjudicate against both."*
  The readers need only the Ghidra export and a function list, both of
  which existed twenty minutes in, so nothing prevented it. Two blind
  readers on Opus 5 went out once the commit landed — A over
  `action_move_near`/`compute_form`/`Form::*`/`action_halt`/
  `action_stance`, B over the pool, membership, `action_siege_attack_to`
  and `action_attack` — briefed with the run28 coverage frames and told
  to name the capture that would falsify each claim.
  **The adjudication went to a third subagent, not to this session**: the
  first reader adjudicating their own document is exactly the conflict
  the three-role split exists to prevent, and a subagent has no
  authorship stake where a cleared session still reads the document as
  the project's own.
- **The audit landed the same night** — `docs/audit/2026-08-25-groups.md`,
  124 verdict rows, five `FABLE:` markers, eight named assertions. It is
  the harshest of the four so far, and the two worst findings are both
  about *the first reading's method rather than its arithmetic*.
  `docs/GROUPS.md` §6.4 declared the slot table a seam on two grounds and
  **both were false**: there is no float barrier (fourteen instructions in
  the whole family, all inside `compute_dests`, all integer-exact as
  `((2·rows − 1)·depth·k)/2`, the `0.5f` read out of the PE), and "no
  capture pins its output" was wrong because
  `GroupData::log_data@0045e1d0` had been dumping `off_x`, `off_y`,
  `curr_x`, `curr_y` and `angles` per member all along — run29 already
  held a four-member group in formation 0. §13 asked for a capture that
  was on disk. Three of §13's five guessed vtable slots are wrong for the
  same reason: `rise_z.map`, which `CLAUDE.md`'s own thesis paragraph
  names, was never opened. Two live bugs in `group.rs` fell out of it.
  What survived: **§9's Manhattan anchor**, which the blind reader missed
  entirely, and all three `docs/ARMY.md` predicates the session had
  corrected without an adjudicator — ratified against the listing, no Rust
  changed. The pattern across four audits now: the arithmetic holds, the
  predicates wobble, and the *method shortcuts* — the file not opened, the
  grep not run, the capture not looked for — are what actually cost.

## 2026-08-26 — applying the group orders' audit, and four markers that cost minutes

One session, no new reading commissioned, and the largest yield of the four
audit-application sessions so far. Three commits: `a7c043e` the nine
Rust-changing verdicts, `bdf8bc3` the twenty-five document corrections,
`f74a743` the whole `GROUPDATA` record in `rondata::diff`. 648 tests green,
clippy and fmt clean, `rondata` exits 0.

- **The nine verdicts, each landed against a test written to fail first —
  and seven of them did.** The two live bugs were the ones the audit named:
  `group_action_halt` wrote each *unit's* `form` where `0070d0c0:29` writes
  the *group's*, once, before the member loop (and `group_get_form` reads
  the unit bytes, so a halt was costing the group the formation its members
  still carried); and `group_action_attack`'s "already attacking" skip was
  unconditional where the original's has two sub-arms. Reaching the second
  arm turned out to **require** a change the audit had not spelled out: the
  outer test has to be on `get_action`, the intent under the transit legs,
  not on the current order's own type, because otherwise the "a current
  *move* that carries me into range" arm is unreachable by construction.
  That needed a new `is_in_range_at` — the eight-argument overload, asking
  the question from a point rather than from the attacker.
- **The rest:** the stance cycle steps from `get_stance_option@0070bab0`,
  the modal option over the members with ties to the lowest index, not from
  the leader; a plane never receives a stance or a group order;
  `siege_anchor`'s sum gates on `is_on_map`; `FORM_NONE = 9` was misnamed
  and is now `FORM_MOB`, because 9 is a real formation and −1 is the
  sentinel; and §6.5's clear/order asymmetry — a hurrying AI army that
  finds no friendly city clears its shooting siege unit's orders at
  `70524f` and then issues it nothing at `7054c7`. That last one is
  reproduced deliberately rather than smoothed over.
- **Four of the five `FABLE:` markers were settled before the corrections
  landed, each by the check the audit itself had named, and each in
  minutes.** This is the session's lesson and it belongs at the top: *a
  marker is a question with a costed answer, and the cost is usually
  smaller than the estimate written beside it.*
  - `unit_flags` bit `f` is `unitrules.xml`'s own legend line — "Unit flies
    like a helicopter" — and exactly three of the 364 records carry it,
    `Helicopter` and the two `Attack Helicopter`s, all `<DOMAIN>Air`. The
    audit had guessed the item might be **vacuous**; it is the opposite. A
    helicopter is not a plane, so it is halted, stanced and group-ordered
    like a ground unit while a fighter is skipped everywhere.
  - `game->semaphore.ptr[1] & 8` is bit 11 of `GameData +0x814`'s
    `BitMask<256>`, and `ConsoleWin::run_cmd@007d6a70` **sets** it right
    after `ScenarioEditor::init` and **resets** it right after
    `ScenarioEditor::close`. `Options::do_formation` calls
    `Group::dbg_jump_to_action` under it. It means *the scenario editor is
    open*. The document's "the network semaphore bit, so a networked game
    never mirrors" was wrong about the flag and therefore about the
    conclusion.
  - A.23's asymmetric `facing` restore is confirmed by twelve instructions
    of `llvm-objdump`, and its consequence is narrower than A thought: the
    toggle at `707eba` and the restore at `707f01` are guarded by the
    **same** compares against `%esi`, computed at `707ea8` and never
    rewritten (`Form::compute` pushes `ebx`/`esi`/`edi`). So `facing` is
    *invariant* across `compute_form` unless that editor bit is set. The
    same listing re-confirms `reverse = |Δ| ≥ 90°`, both bounds inclusive.
  - `role & 0x10` is `is(SCOUT)` on land and `is(BARK)` at sea, from
    `determine_roles`' two writers and the PDB's own `TypeIndex`. Vslot
    `+0x60` is `ObjectTypeData::is`, confirmed independently by
    `init_final_flags` reproducing five `uflags2` names this project had
    derived from the data layer. `docs/ORDERS.md` §8.2's "workers,
    caravans" gloss is struck.
- **Settling the third marker turned up a writer no reading had.** run29
  prints `facing 1` on three live groups, which `compute_form` cannot
  produce with the editor bit clear. The third writer of
  `GroupData::facing` is **`Unit::kill_current_order@005e2cb0`**: when the
  order being killed is a move-family kind and this unit is its group's
  leader, the dying order's own reverse flag is written onto the group. It
  is outside the `Group` family entirely, which is why neither blind
  brief could reach it — and why a `set_stance` from a *human* can rewrite
  a group's mirror flag.
- **The `GROUPDATA` widening, and it failed on its first run.** Nothing in
  `crates/rondata` had ever opened one; the record has twenty scalars and
  six parallel per-member arrays and the original had been dumping all of
  it every frame. Four tests over run29's frames 15100–15102, three made to
  fail on purpose. The fourth failed by itself: the audit predicted
  `priority 1` on every `HOTKEYGROUPDATA`, and hotkey slot 28 carries **0**
  with a `stamp` of 13125 — it held a group and lost its last member, and
  `Group::kill`'s `num == 0 → clear(−1)` runs `Group::clear`, which writes
  over the bit. So the bit means "this slot is a live control group", and
  an emptied hotkey slot is indistinguishable from a pool slot by
  `priority` alone. `last_group[8]` was wrong in the audit for a related
  reason: it is `{p × 0x40}` only in the *initial* dump, and by 15100
  player 1's has moved to slot **70** — a better assertion, because it is
  live.
- **And one of the audit's own assertions was walked back.** `off_x =
  [0, −14, 13, −28]` cannot recover A.28's `X0 = 0, −w, +w, −2w` without
  knowing the rounding: `trunc` makes −14 and +13 contradictory, and a
  floor gives `w ∈ (648, 672)`, not the audit's `(656, 672]`.
  `div_3_table[v >> 4]` is an arithmetic shift, which argues for the
  floor — left to whoever writes `compute_dests`. The rotation check is
  landed but pins less than claimed: every `off_y` in the window is zero,
  so the y-flip's `−cos·off_y` term never fires and a formation with depth
  is what would pin it.
- **What the whole day says about method.** The first reading's failures
  here were not arithmetic; they were *files not opened*. `rise_z.map`,
  named in `CLAUDE.md`'s own thesis paragraph, would have settled five
  vtable slots. `GroupData::log_data`, one grep away, would have killed the
  seam before it was declared. `unitrules.xml`'s comment header names bit
  `f` in English. Four audits in, the arithmetic keeps holding and the
  shortcuts keep costing — and the cheapest correction available is always
  the one where the evidence is already on disk.

## 2026-08-26 — the group orders' third pass: the session as the ratifier

The Fable ratification pass, run the way the day before had agreed: the
session itself, on Fable 5 after a `/clear`, briefed by the charter and not
by the list. The record is `docs/audit/2026-08-25-groups.md`, "Third pass —
verdicts"; this is the story.

- **The floor held eight of nine, and the ninth was wrong in the way a
  checklist cannot catch.** Every one of the nine Rust-changing verdicts
  was re-read from its own citation. Item 34's two *gates* were exactly as
  the second reading had them — the `QUEUE_NEW` clear tests `hurry` alone,
  the order loop tests `hurry && a city was found`. Its *consequence* —
  "clears the shooting siege unit's orders and then issues it nothing" —
  was wrong, because the order loop re-reads `UnitData::order_type` after
  `clear_orders` has emptied the list, `order_type` answers `NONE` on an
  empty list, and the unit falls through into the move like everyone else.
  The sim had faithfully reproduced the wrong conclusion, with a test that
  proved it. The test was rewritten to the right one, ran red (`left: 0,
  right: 2`), and the fix is three lines. A verdict's consequence is a
  separate claim from its gate, and it needs its own reading.
- **The last marker closed from the PDB's method records.** `rise_z.map`
  could never name vslot `+0x1c` for `BuildData` because that slot is the
  COMDAT-folded `return 1` at `0041e0e0` that twenty symbols share, and the
  linker keeps one name per address. `llvm-pdbutil dump --types` names it
  in ten seconds: `SubObjectData` introduces `is_unit` at vftable offset
  24, **`is_wallbuild`** at 28, `is_build` at 32, `is_seen` at 72 — and the
  same records confirm every other slot the audit had inferred from uses
  (`is_on_map` 188, `is_plane` 192, `has_stance_type` 260,
  `get_stance_type` 264, `hits` 284). The type stream is the strongest
  evidence in the project and it had not been asked.
- **What four passes missed, in order of consequence.** The predicate
  behind every stance decision, `UnitTypeData::get_stance_type` — military
  first (packer or combat), then the citizen ids, then a caster that does
  not pack, else none — where the sim tested caster, packer, "has an
  attack", citizen; three of seven cases differed, and the fix ran red
  first. The slot table's rounding: `div_3_table` is built as a floor on
  both sides of zero and `>> 4` is arithmetic, so the `/48` is a floor,
  which is what run29's `[0, −14, 13, −28]` had been saying. `update
  positions` read whole from the listing and reproduced **bit for bit**
  from the leader's logged `UNITDATA` `angle` through the sim's own sine
  table — the first exact pin on the formation machinery. A fourth writer
  of `facing`, `Unit::set_angle`, which toggles it whenever the leader
  turns by 90° or more — the ordinary reason a live group reads `facing 1`.
  And the loaders: `Groups::clear` writes `stamp = 0` and `priority = 0`
  *over* `Group::clear`'s values, and `Group::clear` zeroes `who`, which is
  why the widening's `who == id/64` can only hold for a live slot.
- **Two mechanical scans were delegated and everything else was not.**
  Two Opus `lean` subagents swept the whole export — every reader and
  writer of a `GroupData` field outside the family (561 candidate files,
  13 writers), every caller of the family's 58 functions (441 call sites)
  — and wrote to `~/ghidra-projects/reading/` from the first hit. Every
  claim built on a hit was re-read here. The scans cost a quarter of an
  hour and found the `set_angle` writer, `Army::stop`'s `form = −1`, the
  `0x8ca` list base in the save-game loader (real: the listing says
  `[ecx + 2*eax + 0x8ca]`), and that `UnitData::get_speed` caps a grouped
  unit at the group's speed — which the sim already had. One correction
  to a scanner's brief mid-flight, when `action_halt`'s `this->field_0x49`
  showed a `Group *` carries `GroupData`'s offsets unshifted.
- **The widening was made to fail and then taught something.** The
  `o_angle` control — "the group's move angle does *not* reproduce
  `curr`" — did not fail: in this window the leader's heading is the
  move's bearing, so the record cannot separate the two, and the listing
  (`mov ebx, [ecx + 0x50]`) is what says `update_positions` reads the
  heading. The control was dropped and the reason recorded; the
  quarter-turn control stays. The y-flip is still unpinned, for the
  reason the day before gave: every `off_y` in the window is zero.

Where it leaves things: the group orders are done, third pass included;
item 17 (`Form::compute`) starts from settled rounding and two exact
fixtures rather than a question; and the older Fable debt from the AI,
transport and army audits is still booked, with item 13's captures the
cheapest way to clear most of it.

## 2026-08-26 — item 17: the slot table, and three accepted verdicts overturned

**Landed.** `Form::compute`'s slot table, whole, in
`crates/sim/src/form.rs` — `type_cat`, `categorize`,
`compute_rows_and_columns`, `compute_dests`, `get_form_mod_option` and
`update_positions` — with `crates/sim/src/group.rs` giving each member
**its own slot destination** instead of the group's, and `GroupState`
carrying `form_num`, `off`, `curr` and `angles` so the record has
something to compare. Two type columns joined the loader
(`x_spacing`/`y_spacing`, the `X_SPACING`/`Y_SPACING` columns times
`UNIT_FORMATION_SPACING`) and one byte joined the unit (`+0xab`, the
formation width twin of `+0xaa`). Eight tests in `form.rs`, one in
`rondata::diff`, and `docs/GROUPS.md` §6.4 rewritten from a four-line
sketch into the table. This was the last of the group orders' seams that
cost work rather than a grep.

**The diff.** The one that matters runs
`unitrules.xml` → `x_spacing 660` → `FORM_CAT_ARTILLERY` →
`form_mod 50` → `cols 4` → the slot arithmetic → the floor divide by 48,
and lands on `[0, −14, 13, −28]` — run29 `GROUPDATA` `id 66`'s own
`off_x`, with `off_y`, `angles`, `form_num` and all four `curr` pairs
matching across the window's three frames. Nothing in it is a fixture
written from the answer: the numbers come out of the install's own columns
and the simulation's arithmetic, and the test also asserts that fifteen of
the install's types carry `x_spacing 660` and that every one lands in the
same category, so the loader and `type_cat` break it rather than passing
quietly.

**Eleven deliberate breakages, ten of them red.** A truncating quantiser,
a missing even-column shift, a missing anchor slide, either half of the
rank stack removed, a placing Square, a `form_mod` off by one, the `human`
gate ignored, a Column two to a rank, the left-right alternation dropped —
each turned a named test red on the first try. The eleventh, **flipping
the sign of `update_positions`' `cos θ · y` term, stayed green**, because
every `off_y` in run29's window is zero and no capture on disk can tell a
rotation from a rotation-with-a-flip. That is the same limit the day
before recorded; the test now says so in place and pins the flip from the
listing instead, which is the honest label rather than a silent pass.

**Three of the audit's accepted verdicts were wrong**
(`docs/audit/2026-08-25-groups.md`, "Fourth pass"), all three under the
one row adjudicated as "additions … as cited in A":

- the block's anchor is the **lowest**-indexed non-empty category, not the
  last — the machine code writes the loop's `prev` only while it is
  negative (`72cfe2`, and the `prev >= 0` arm reloads a slot it never
  wrote at `72d837`), so Ghidra's decompilation was right and reader A's
  rewrite of it was not;
- `compute_dests`' final loop **drops the anchor's x from the
  destinations** — `%edx` is explicitly zeroed at `72d737` before the
  `cosx` call — so an even column count, the commonest case there is,
  sends a group to points displaced from where its own offsets put it.
  `GROUPDATA` cannot see it; a `UNITS=3` order list can;
- **formation 6, Square, is dead code in the shipped executable.** A.28
  said "Square not established at all"; it cannot be established.
  `compute_rows_and_columns`' `== 6` arm fills three fields
  (`space[18][4]`, `across`, `per[7]`) that **no function in the export
  reads**, `compute_dests` has no Square branch, and the same arm sets
  `wedge = −1` so the wedge branch cannot cover for it.

**And two nobody could have reached by reading.** `Form::compute` declares
`int rows[18]`, never initialises it, and the wedge arm reads
`rows[wedge]` before writing it (`72dc90`) — so a wedge with a second
category is not reproducible by anyone, us included. And
`get_form_mod_option`'s value, which §13 had booked as a reading, was
printed in the dump all along: `form_mod 50`, on every member of the navy.
The audit README's own lesson, again — diff first, then read what no run
reaches.

**What this says about the method.** An adjudicator's "as cited in A" is
only as strong as the arithmetic nobody re-derived, and where a reading's
product is a *formula* the citations can all be real while the conclusion
is wrong. Three rows here were exactly that, and all three fell out of
writing the code rather than reading it again. **Where a mechanic's
reading produces a formula, the implementation is the third pass** — and
running it before the ratification would have been cheaper than after,
because the Fable pass spent its budget confirming a rank-stack loop that
a compiler had already contradicted.

Where it leaves things: the group orders are done, slot table included,
and the queue's next item is run29's `UNITS=3` half — which is now owed
twice over, since it is both `scene_at`'s missing order lists and the only
capture that can see §6.4's `to`/`off` asymmetry.

**The same day, the working agreement changed.** Decided with the user
after the above, and recorded rather than inferred: `Opus drives` — it
carries implementation, diffs, widenings, adjudication and the ordinary
reading, and Fable is chosen for a *first* decompile reading, for
overarching or genuinely new design, and for ratification. Ratification
**batches over what is marked**, on a ledger in `docs/audit/README.md`,
instead of gating each mechanic on a pass of its own; the batch size and
cadence are deliberately left open — "I don't want to proscribe yet" — and
that is written down as the reason, not left as a gap. And the lesson this
session paid for becomes a default: **where a reading's product is a
formula, the implementation is a pass of the audit**, so build before
ratifying or in parallel. `CLAUDE.md`, `docs/DECISIONS.md` entry 22
amended and 23 new, `docs/audit/README.md` steps 6–8.

Building the ledger corrected the queue's own bookkeeping. "The older
Fable debt from the AI, transport and army audits" was wrong in both
directions: the AI audit **has** a Fable third pass, and the real list is
fifteen audits — the nine of 2026-08-20, the pathfinder, anim, commands
and recgame, plus transport and army. Two `FABLE:` markers are still open
and both deliberately so — the orders audit's R2 O1 and
`find_gather_tcoords`, each kept as a pointer to a check nothing yet needs
— which I first wrote down as "none", and checked. At the top of the
ledger sits the newest row and the awkward one: the
group orders' **fourth pass**, which retracts three verdicts an earlier
Fable ratification had confirmed, so a ratifier taking it must be told
that a previous pass agreed with the rows now being pulled.

## 2026-08-26 — item 18: run29's `UNITS=3` half, and the frame nobody could reach

The queue's opener asked for one thing — teach `scene_at` to load a
block's `UNITDATA` order lists — and named four open items it would pin.
It pinned two, killed one outright, and found two bugs and a missing
frame on the way. All five are worth writing down, because three of them
are the same lesson in different clothes: **the record was already there
and nobody had opened it**.

**The loader.** `OrderDump` carried three fields of the `MOVEORDER` row
and `UnitDump` carried none of the order layer's `UNITDATA` fields; both
now carry the whole record — the twenty `MOVEORDER` fields
`docs/ORDERS.md` §4.1 read back from the PE, `ATTACKORDER`'s seven, and
`group`, `form`, `form_mod`, `stance`, `orders_x/y`, `dest_angle`,
`myspeed`, `o_up`, `inside_up`, `myhits`, `damage`. `scene_at` stands
every unit of a block up as a `sim::Unit` — typed from its first guy's
`TypeIndex`, facing the record's own `angle`, carrying its formation
bytes, its order list front-first and its path stack — and gives each
army the membership of its `GROUPDATA` slot **in the record's own `list`
order**, which turns out to be the load-bearing detail.

**`engagement`'s choice of unit, settled** (`docs/ARMY.md` §11, and §18's
open item struck). At 15100 army 0's seven members hold attack orders
pointed at a *scatter* — who 0's objects 15, 16 and 17. At 15101 six of
the seven hold **15**, every one has gained the action bit (`flags 0x10 →
0x14`) and the group's `order_num` has gone `0 → 1`. That is
`Group::action_attack` firing on the tick, and the seed it adopted is the
target of `o 54` — the **first entry of the group's `list`**, which is
what `ArmyData::get_unit` walks. The harness picks the same unit and the
same object. The seventh member ends on 26 because `action_attack` gives
each member `find_melee_target`'s own nearest and the army's target is
only the fallback (`docs/GROUPS.md` §10) — so what the record shares is
the *seed*, not the outcome.

**Two bugs, both found by running the capture rather than reading.**
`is_engaged` and `engagement` tested the unit's **front** order for
`ATTACK`, where `6f51fa` calls `UnitData::get_action`. Every member here
is walking a pathed transit leg in front of its attack, which is the
ordinary shape — so on the one frame in five runs that reaches
`Army::engagement`, the simulation found nobody engaged and did nothing.
And the listing gave a second correction the decompiler cannot show:
`%edi`/`%ebx` hold `ox`/`whom`, **every** qualifying unit overwrites them
(`6f531a`, `6f533f`), and `is_map_unit` at `6f5362` gates only the
*break*. So the rule is "the first qualifying unit whose target is a map
unit; failing that, the **last** qualifying unit's target, whatever it
is". That arm is unobserved — run29 breaks on the first — so it goes on
the ratification ledger with the capture named.

**`find_leader`'s key, implemented** (`docs/GROUPS.md` §4.4). The
simulation had taken the first on-map captain since the module was
written; the listing's test is `local_8 < 0 || cat < best`, so the first
qualifying member leads and only a **strictly** lower `type_cat`
displaces it. `sim::form::type_cat` existed for `Form::compute` and this
is its second reader. The capture the queue had named for it — an army
with a wagon and a hoplite — would not have worked: the member at slot
`(0, 0)` is the anchor of the lowest-indexed non-empty category, which is
the same quantity, so it cannot disagree. The capture that *would* is a
two-category group whose members' headings differ, in a formation with
non-zero `off`, where `curr` names the heading `update_positions` used.

**And one item killed.** §6.4's `to`/`off` asymmetry — the offsets slid
by the whole anchor, the destinations by its `y` alone — was booked
against this very capture, and run29 cannot see it. The one group with
non-zero offsets, the navy, holds **no orders at all**; the one group
whose members hold move orders has `form −1` and every offset zero. The
window has no group that both stands in a formation and walks to one, and
the AI in these lobbies never issues a formation move. It wants a human's
right-click on four units of one type, two frames of `DUMP_ALL`, and
that is now written into both §6.4 and §13 in place of "run29's `UNITS=3`
half".

**§7's halt, seen for the first time.** Army 0's group carries `form −1`
while all seven members carry `form 0` — `action_halt` clears the
group's byte and touches no member's, so `get_form` gives back what the
group's own field lost. The same record separates `get_form_mod_option`
from `get_form` outright: two of the seven carry `form_mod −1` and the
option is still **50**, which is a *mean over the members that have one*
and not the all-agree-or-−1 twin §4.4 called it. `form.rs` had it right
from the function; the document had it wrong from the family.

**The frame nobody could reach.** Counting move orders disagreed with the
`grep` by a factor of two, which turned out to be `full_dump` running at
both `begin_frame` and `end_frame`: two dumps a frame, and for run29's
frame 15100 all 2,583,636 lines of the pair match bar the checksum index,
`turn_control`, the two command stamps and the timing counters. Harmless
— except at the end of a run, where `!quit` leaves a `FRAME` block with
nothing under it and the final dump lands **at `FRAME`'s own indent**, a
sibling with no following frame to duplicate it. run29's free 15105
state had been sitting in the file, unreadable, since the day it was
captured. `Log::dumps` finds it, and the window is four states rather
than three.

**The whole `MOVEORDER` row, diffed.** `docs/ORDERS.md` §4.1's table was
read off the PE and compared with nothing; it is now walked over all 79
move orders of the four states — the cell-centre snap, `off = x mod
0x300` (the offset *inside the world cell*, not a formation slot),
`tolerance`/`pause`/`retry`/`attempts`/`timer` at zero, `dest_x/dest_y`
as the path stack's top whenever `dest` is 1, the goal flag on the
bottom of the stack, the pathed bit as "has a stack", and `orders_x/y` as
the current move's own point. `ATTACKORDER`'s seven fields, which the
table said "no dump has one yet" of, are in this one 181 times and match
the PE read exactly.

Both new behavioural claims were made to fail on purpose before landing:
testing the front order instead of `get_action` loses the seed entirely,
and walking the member list backwards seeds from `o 25` and object 26.

## 2026-08-26 — item 20: the human group move, and the cheap group pool

The capture three of `docs/GROUPS.md`'s open items were waiting on, and
the first that needed a **person** in the middle of a run: a player's
right-click on a multi-unit selection, which the AI's armies in these
lobbies never produce and the cheat table has no verb for. Two runs, four
new diffs, and a per-frame record nobody knew was free.

**The run got ten times cheaper before it started.** The queue had item 20
budgeted as a `DUMP_ALL` window of two frames — ten minutes, 250 MB, and
the click had to land inside a two-frame window a human cannot aim at.
`GameLog::full_dump@00930380` says otherwise: past the `do_dump_all`
branch it is a list of per-category gates, `GameLog::end_frame` calls it
every frame, and `details[mode][0x12]` is `GROUPS`. So the 512-slot group
pool is an ordinary `[End Frame]` record, ~130 KB a frame at full speed.
The window becomes hundreds of frames wide and the timing problem
disappears.

It does not come out that way on the first try, and the reason is the
better half of the finding. `GroupData::log_data` never sets its own type
or detail, so its lines are accepted against whatever the previous dumper
left — and `dump_deaths` ends by calling `WorldData::log_data` **twice**,
leaving `current_type` at `WORLD`. With `WORLD=0` the pool is dropped
silently: the key is on, the dumper runs, and not one line survives
`check_accept`. That is what run30 is: 378 MB, 215 frames, and no
`GROUPDATA` at all. `DEATHS=0` fixes it. `docs/ORACLE.md`, "The group pool
is a per-frame record", has the working settings; `setlog.py` takes
`CAT=N` now, because a bare name meant 1 and `UNITS=3` was unreachable
through it.

**The selection turned out to be scriptable.** `run_cmd`'s `select` case
takes `[type] [who] [+]`, walks every object of that player and adds each
match; the trailing `+` suppresses the clear. Two channel lines select
twelve units, and the only thing left for a person is one right-click with
`cliclick`. The earlier "`+` is not reliable" was the chat box dropping
lines, not the command. `tools/gamelog/live.sh` is `runwin.sh` stopped at
the point a driver takes over, `archive.sh` is the other end, and
`groups.py` reads the pool.

**run31**: three right-clicks at three bearings, eight hoplites and four
slingers, selected slingers-first on purpose so the group's `list[0]` is
not its lowest category. Forty frames carry both a live group and a
`GroupMoveOrder`. What it settled:

- **`find_leader`'s key, from a record — and by a better field than the
  one asked for.** `GroupOrder::oxx` names the leader's object outright on
  every member's order, and the slot table's anchor is that object's slot.
  No need for `curr` to infer it. The record's leader is a hoplite where
  `list[0]` is a slinger, which the first-on-map-captain rule the sim used
  until 2026-08-26 cannot produce.
- **§6.4's `to`/`off` asymmetry, observed.** The one member whose slid
  offset is exactly `(0, 0)` holds an order whose destination is *not* the
  click its own `orig_x`/`orig_y` records — on all three moves, every
  frame. Slid by the whole anchor, it would be the click exactly.
- **`update_positions`' y-flip, pinned.** run29 reproduced `curr` too, but
  every `off_y` in its window was zero, so the flipped column multiplied
  nothing and the eleventh deliberate breakage of the slot-table session —
  flipping the sign of `cos θ · y` — **did not turn a test red**. run31
  stands in three ranks; the flip is now a check with a control.
- **`GroupMoveOrder`, the record.** The order §6.6 step 6 adds, which no
  dump had held. Two bases and a field: the member's slot destination and
  the click on `MOVEORDER`, the leader and the member's slot index on
  `GROUPORDER`, `in_group` of its own.

**Two bugs the capture found, one in the parser and one in the reading.**
The parser's order walk matched `ends_with("ORDER")`, and
`GroupMoveOrder` is the one block in the family the binary keeps in mixed
case — so it was dropped, *and* every later `type` slid onto the wrong
body, because the pairing is positional. And `Form::compute_dests`'
follower arm had never executed: `Group::add` keeps a non-captain only
with `keep_captain`, which nothing an army does sets, but a **player's**
selection group keeps every figure. Thirty-six members, not twelve.
`Sim::form_follower_slot` implements it from the listing — one
`guy_spacing` to the side of the last captain, alternating, each step
added to the previous member — and the record's `(c, c+3, c−3)` per squad
is what it was written against.

**One thing chased and dropped, worth writing down.** `curr` came out one
or two units off on 31 of the 40 frames, and the first suspect was
`sin_component`'s second-quadrant fold: the sim mirrors the angle where
`sin_table@00a46a00` appears to keep the index and compute
`0xffff − base + delta`. Implementing the decompiler's rendering made the
error *hundreds* rather than one, so the mirror is right and that branch's
decompilation is not. The real answer was smaller and more useful: a
heading within a twentieth of a degree of the dumped one reproduces all
seventy-two numbers **exactly**, on every frame. `curr` is a **mid-frame**
quantity — `do_group_move` computes it and the unit turns afterwards — so
the end-frame `angle` is the heading a hair past the one that was used.
Nine frames of forty, where the leader was not turning, are exact. Any
later `curr` diff needs to know that.

**What it did not settle**, and it is now the sharpest open question the
document has: run31's leader is object 9 on one frame and object 6 on the
next, both hoplite captains of the same category, with 6 first in `list`.
A first-wins tie names 6 both times. Reproducing the whole 36-member
table is what would answer it, and that is also what would turn §6.4's
*observed* asymmetry into a *measured* one. The record is on disk for
both.

**The thing this session earned.** Item 18's lesson was "the record was
already there and nobody had opened it". This one is a level up again:
**the record was never written, because a key nobody had questioned was
zero**. `GROUPS` had sat at 0 in every `[End Frame]` since the loggers
were first configured, and the reason it stayed 0 was that turning it on
did nothing — which read as "this category is not per-frame" rather than
"this category inherits its acceptance from the last one that ran". Worth
asking of the other keys before booking a window: not only *what does a
dump on disk already carry*, but *which of the thirty-seven categories has
never been turned on, and what happened the one time it was*.

## 2026-08-26 — item 21: the 36-member table, and the writer nobody had looked for

The whole of run31's group reproduced from the install's own columns —
36 members, both coordinates, all forty frames, and 900 slot destinations
with it. It closed the two questions item 20 left open, and it found a
function writing the record behind the reading's back.

**The chain has no free parameter.** `unitrules.xml` gives Hoplites and
Slingers `X_SPACING`/`Y_SPACING`/`GUY_SPACING 12` and `UBER_SIZE 3`;
`UnitType::init` multiplies by 12; `Form::categorize` widens a multi-figure
rank by `min(uber_size, 3)`, so the category is 432 wide and 144 deep;
`type_cat` splits eight hoplite captains into `FORM_CAT_FOOT` and four
slingers into `FORM_CAT_FOOT_RANGED`; `span` and `form_mod 50` give both
four columns; `Group::add`'s subordinate recursion turns twelve captains
into 36 members in exactly the record's own `list` order; and
`compute_dests` plus the floor divide by 48 lands on `off`. Five
deliberate breakages, all red on the first try.

**The finding: `compute_dests` is not the last thing that writes `off`.**
Thirty-nine of the forty frames matched at once and one did not — frame
204, whose block sits one column over, with every *relative* number in it
right. The listing (`72d17c`–`72d198`) says the anchor is `cat_id 0` of
the lowest non-empty category and nothing else, so the table was right and
the record had been moved. `Group::refresh_group_order@00713a50` is what
moves it: `Unit::do_group_move` checks that the unit the order's
`GroupOrder::oxx` names is still usable as the block's origin — alive, on
the map, in this group, holding a matching group order — and when it is
not, the first member to notice **re-origins the whole table onto itself**
and rewrites everyone's order. Frame 205, one frame on, is that same table
re-origined again onto a third member. Nothing about the layout ever
changed.

That is the entire answer to the "leader is object 9 here and object 6
there" question the last session called its sharpest. `find_leader` names
object 6 both times. `oxx` is not `find_leader`'s output at all; it is the
block's *current* origin.

**And the displacement was in a field nobody had read.** §6.4's `to`/`off`
asymmetry — the anchor marching to a point its own offset says is the
click — is exactly the rotated `anchor_x`, and `Form::compute`'s tail
measures precisely that into `group.o_dist` before `action_move_near`
overwrites `o_angle` and leaves it alone. `x_spacing/2 = 216`; the record
prints 215 on one move and 217 on the other two, which is `vector_dist`'s
octagonal approximation of the same 216 at three bearings. Observed became
measured for the cost of noticing which of two adjacent writes lands.

**One question closed two and opened one.** The mirror `Form::compute` is
handed is `facing XOR (leader ≥ 90° off the bearing)`, and `GROUPDATA`
prints only `facing` — so run31's three moves need mirrors of 0, 0, 1
against dumped `facing` of 1, 0, 1. The toggle has to fire on the first
move and not the third, and by its own logged heading the first move's
leader is 76° off, which predicts no toggle. Either the end-of-frame
heading is not the one the call used — `curr` is already known to be a
mid-frame quantity — or `facing` moved between clicks, which for a ground
group only this toggle and `Group::clear` can do. The capture is named in
`docs/GROUPS.md` §13 and it is cheap: one window, two clicks, a formation
that leans so the angle byte is not zero either.

**The method note.** This is the third session running where the
implementation was the audit. Prose had the anchor rule right, the
quantiser right and the rank stack right, and still could not have told
you that a second function slides the result — because prose compares a
formula with a formula and only the code compares it with the record.
`git checkout` on a file mid-breakage cost twenty minutes of retyping,
too: a deliberate breakage wants a scripted apply/revert, never the
working tree's own undo.

## 2026-08-26 — item 22: the mirror's predicate, and the run that was not needed

The queue's opener for this session named a behavioural run: one
`GROUPS=1` + `UNITS=3` window, a leaning formation, two right-clicks, to
settle which mirror `Form::compute` is handed. **No run was needed.** The
answer was in run31's dump, which has been on disk since the small hours,
and the two functions that produce it were already written down in two
different documents.

**The reading error was two errors.** The first: the last session read the
leader's heading off **frame 204**, the click frame itself — where the
group is one frame old and its leader has already snapped 118° into the
march — instead of frame 203. The second, and the one that mattered:
`GroupData::facing` is a *running* flag with four writers, and the reading
had only counted two. `Unit::set_angle@00605400` toggles it whenever the
group's **leader** is turned by 90° or more, which is what every marching
leader does the moment it takes a bearing. And
`Unit::kill_current_order@005e2cb0`, on a dying move, **assigns** the
order's own `MoveOrder::facing` back onto the group, inverted if the leader
has since turned around.

**The ordering is the finding, and it is one grep of the listing.**
`action_move_near`'s `QUEUE_NEW` clear loop runs at `70524f`; its
`Unit::clear_orders` is at `70538d`; `compute_form` is at `7053ec`. The
clear comes **first**. So a group's second right-click hands the flag back
before it lays anything out, and `Form::compute` never sees the march's
flag at all — it sees the mirror the *last* layout used. That is why
run31's frame 328 lays out square while its record prints `facing 1` the
frame before.

Both writers were already documented — `docs/GROUPS.md` §4.1's field table
has had them since the group orders' third pass, and `docs/ORDERS.md` §3.2
describes the hand-back in full, calling it a "carry-over". Nobody had put
the two documents next to the listing's line numbers. **A fact written in
two places and joined in none is not established**, and the queue had
booked a run to rediscover it.

**Six predictions, five breakages, one honest green.** The check
(`run31_s_three_mirrors_come_out_of_facing_s_three_writers`) drives the
whole machine over run31's three clicks and predicts both observables each
time: the mirror, taken from the `facing` byte the orders themselves
carry, and the `GROUPDATA::facing` the click frame prints. All six land,
and the same test carries the old model as a control — it gets one of the
three wrong. Four deliberate breakages went red on the first try. The
fifth, dropping the hand-back's `reversing` inversion, stayed **green**,
because both of run31's kills catch the leader 10.6° and 6.3° off the
dying order's angle. That term is carried by the listing alone and
`docs/GROUPS.md` §13 now names the capture that would reach it — which is
a much narrower run than the one this session was told to make.

**And the seam the angle sat behind is gone.** `Sim::add_move_facing_order`
now takes the caller's angle and the formation's mirror, so a group move's
orders carry `angle + (angles[i] << 24)` and `MoveOrder::facing` — the
`+0x28` field that had a reader in the original and none here. The sim
runs the state machine live: the `QUEUE_NEW` clear hoisted into its own
pass ahead of the layout, `Sim::unit_set_angle` on `Unit::move_step`'s own
call, and `Sim::hand_back_facing` inside `kill_current_order`.

**The method note.** Two sessions ago the lesson was that the
implementation is the audit. This one is smaller and cheaper: **before
booking a run, grep the dump you already have, and grep the listing for
every writer of the field you are about to call frozen.** The queue
entry said "nothing in the file writes it for a ground group but this
toggle and `Group::clear`", and cited a grep — a grep that had found
`action_air_patrol` and `action_flight` and stopped, because the two real
writers reach the field through a group pointer rather than by name. The
same trap `MoveOrder::facing` was already recorded as having: it is
fetched through an order vtable slot, so grepping the field name finds
nothing.

## 2026-08-26 — the fuzzer, and three assumptions it cost to keep

Item 13 was the entry with "the largest leverage per hour by a distance",
and the argument was that its three pieces already existed and only needed
wiring. They did. The wiring took an afternoon. What took the day was that
three of the entry's assumptions were wrong, and every one of them was
settled by a run or a measurement after a reading had already been written
down.

**The vocabulary is readable from the install, and half of it is out of
reach.** `ConsoleWin::init_cmds` copies every command's name and help out of
`int_str_array`, which is `Data/internal_strings.xml` positionally — so
`tools/gamelog/console.py` re-derives all 102 commands from the user's own
files. `run_cmd` then opens by jumping past its first switch when
`from_chat` is set, and the two switches are **disjoint**: 56 console-only,
45 reachable from a `cheat ` line. The `!` prefix in a `.cmd` file is not a
convenience; it picks which half the line can reach.

And the half that matters is not there. **`move` is a teleport** —
`Unit::set_new_location`, which is `remove_from_world` then `add_to_world`
— and no console command issues an order at all. The queue entry had said a
scenario could contain "an order (move, gather, build, attack, garrison)".
It cannot contain one. Every `add_*_order`, `do_*`, `action_*` and
`process_*` on the blind list needs the UI, which is why run31's three
right-clicks are still the only thing that has ever entered
`CommandPackage::process_move_to`.

**`restart` wedges the game, and the gate is what found it.** The plan was
many scenarios per launch on the strength of `restart <seed>` being a
console command. It is one, and it does what the decompile says. But the
channel fires at `Game::do_frame` entry, so `Game::close`/`Game::init` tear
down the game whose tick they are in. The trace is unambiguous, because the
`INFO cmd` record is written *after* `parse_cmd` returns and there is no
record for that line: the window went black, the process stayed alive, and
no further `FRAME` was ever written. `rise.ini`'s `Seed (0 for random)`
costs one launch per seed instead, needs no re-entrancy, and still
regenerates the map — which is all `restart` was wanted for.

**The first seed found a panic.** Seed 424242 staged nine spawns and a few
pokes, ran to its window, quit cleanly at frame 3004 — and the diff panicked
in our own code: `Sim::is_enemy` indexing two-player diplomacy tables with
the nature player, `index out of bounds: the len is 2 but the index is 8`.
run29's dump carries `who 8` as well and does not panic, so this is a *path*
31 hand-built runs and the soak never walked, not an input they never saw.
One data point, and the best argument the fuzzer has made for itself.

**Then the measurement that retired an afternoon's plan.** The dump cost
~25–30 MB and about a minute a frame, which made a fuzzed seed score over
three frames instead of hundreds, which made the ledger nearly meaningless.
The obvious answer was to stop asking the game's logger for text and write
an in-process binary dumper in `rontrace.dll` — we have the DLL, we have
5,880 struct layouts. Twenty minutes of measuring the existing dump killed
that: **the logger is expensive for a reason we control.** `DUMP_ALL=1`
overrides the per-category levels, and `window.py stage` sets it for exactly
one reason — `scene_at` asserts on a `WORLD` block at the stand-up frame.

**And then the correction, which is the part worth keeping.** The first pass
at that measurement read the section names and called 85% of a block static.
Hashing the sections instead said **0% identical** — until the diff showed
the only difference between two copies of `COMBATTABLE` was *one space of
indentation*, across all 251,738 lines. Normalised, the honest figure is
**51%**: `COMBATTABLE` 36%, `UNITTYPE` 14%, the small type tables the rest —
the rulebook, which we already hold from the XML. `WORLD` is the other 31%
and **genuinely changes**, so the tidy idea of taking it from the start
block is wrong, and was claimed in the queue before it was checked.

**The method note.** Two of them, and they are the same note from opposite
ends. **Measure the oracle before building a better one** — a day's design
went away for twenty minutes of counting. And **"static" is a claim about
what changes between frames, so check it between frames**: naming a section
`COMBATTABLE` is not evidence that it is constant, hashing it across six
blocks is, and the first attempt at that got the answer wrong twice — once
by trusting names, once by trusting bytes that differed only in whitespace.
The rule the project already had — grep the writers of every field you call
frozen — turns out to apply to whole sections of a dump too.


## 2026-08-26 — the who-8 panic, and the window that was cheap all along

Two jobs from the opener, and each one turned out to be smaller than it
looked and to be hiding something larger.

### The panic: the original never asks

`Sim::is_enemy(who, 8)` indexing a two-player diplomacy table was the
symptom. The fix could have been one `.get`, and that would have been
wrong, because the interesting question is not *what does 8 mean* but
*why does the original never ask*.

It never asks because it cannot. `LeaderData::diplos` is `int[8]` with
`treaties` at the next offset, so `is_enemy(8)` would read `treaties[0]`.
And it does not need to, because two independent bounds stop every search
at leader eight, and `Leaders::list` is `Leader[10]` — eight players, then
gaia's animals and gaia's birds:

- `ObjectsData::find_unit@0065ca80` walks the per-leader object lists with
  a stride of `0x6eec` (one `Leader`) while the cursor is `< 0x37760`,
  which is exactly eight of them; and its by-cell branch, which reads the
  leader out of a cell's object chain, guards `(int)leader < 8` outright.
- `ObjectData::valid_target_const@006472c0` returns 0 on `7 < who` in its
  **first line**, before `LeaderData::is_enemy` is reached at all.

So: **in Rise of Nations nothing can attack an animal**, no search returns
one, and an arrow that comes down on a sheep passes through it
(`Ammo::check_hit` is a `find_unit`). The second bound is the load-bearing
one for the port, because `Object::find_nearby_target@00648da0` walks the
cell chains with *no* leader bound of its own — `valid_target` is the only
thing keeping gaia out of the ring search.

There is an asymmetry worth keeping: the bound is on the **target**, and
`valid_target_const` dispatches through the *attacker's* vtable, so
`AnimalData::valid_target_const@005d8120` — two lines, `flags & 1`, no
diplomacy — means an animal may target anything while nothing may target
it. It does not arise; `Sim::animal_idle` only wanders.

`world::PLAYER_SLOTS` is the bound now, and `docs/ANIM.md` §6.1 is where
it lives, because §6 already owned gaia's units. Five tests, five
deliberate breakages, all red — and two of them reproduce the original
panic verbatim, which is the point of writing them.

**What is honestly not settled**, and it is in `docs/CITIES.md` §7.2:
`Build::check_capture`'s tally walks the cell chains with no leader bound
and writes `local_8c[leader]`, a **local `int` array**. An animal reaching
it would overrun the stack. Either its filter — index 8 in
`Search::valid_filter`'s jump table, the block at `0067de47`, which turns
on an unnamed type field `+0x1e8` — excludes animals for another reason,
or the overrun is real. The sim skips gaia there on the leader bound,
which cannot change an outcome but is not derived.

**The method note**: the reading that mattered took ten minutes and was
one line of one function. The temptation was to make `is_enemy` total and
move on; the totality is in, but as a *guard*, labelled as one, and the
model is the bound. Two very different things had to both be written down.

### The window: the queue asked the wrong question, and the answer was on disk

The opener named an experiment — `DUMP_ALL=0` with `[End Frame] WORLD=6`,
and what does a 300-frame window cost. **Both halves of its premise were
wrong.**

`window.py stage` sets `DUMP_ALL=1` because `scene_at` wants a `WORLD`
block with its 3600 cells. But `[Start Game] WORLD=6` writes those cells
on its own with `DUMP_ALL=0` — run31's start dump has all 3600 of them and
has since the small hours of the same day — because
`WorldData::log_data@006b6080` calls `set_detail(2)` before its per-cell
loop. And no frame *inside* the window needs a `WORLD` block at all:
`run_traced` stands the sim up from the **start** dump and ticks forward.
So `[End Frame] WORLD=6` was never needed, and the thing to measure was
not its cost.

Same seed, back to back: the `DUMP_ALL` window bought **5** frames for 249
MB; the cheap one bought **301** for 207 MB. 72× cheaper a frame, 60× more
frames, less wall clock. The one thing it gives up is `master_land_heights`,
which is genuinely `DUMP_ALL`-only — and for a fuzzed seed there is no
sibling to borrow it from, because the whole point is a map nothing else
has captured.

### The control run, and then the audit that took two thirds of it back

The third run was the control: `scenario.py --no-stage`, which issues no
cheat at all — not even `ai off`, whose whole effect is to stop a leader
the sim would keep playing — and a window at [1, 301). 195 MB, ten minutes,
301 frames, and **20** unlinked units instead of 3,610.

**It scores `survived = 1`.** Player 1's `o 0` is 24 position units off on
both axes at sim-frame 2; player 0's `o 1` by 10 at frame 4; and at frame 1
two of player 1's units already hold an order the sim never issued. Then
the heights sibling — six more minutes, `window.py stage 1 3` on the same
seed — came back **identical**, so that is the port and not the flat map.

The first draft of this entry called that the first honest fidelity number
on a map we had not tuned against, and credited the fuzzer with three
findings. **Then the session was asked, plainly, whether the fuzzing was
real or whether the panic was a nonsense input of our own making. Two of
the three claims did not survive the question**, and the whole audit cost
three `--diff` runs against dumps already on disk, about forty seconds:

- **The who-8 panic is not a fuzzer finding.** Reverting the sim to the
  pre-fix commit and re-diffing **run31** — the tuned lobby, no cheats,
  captured the previous day — panics **identically**, same line, same
  message, same `find_muster_spot` → `nearest_enemy_attacker` stack. What
  reached it was *stepping more frames*, not a new map and not a cheat.
- **The frame-0 draw gap is not a fuzzer finding either.** run20 reads
  `ours 160 draws, the original's 175` — the same **15** short — and
  scores the same `ticks before divergence: 1`. That makes it a *better*
  lead, because one missing block reproducible on two maps is easier to
  chase than a map accident. It just was not found by fuzzing.
- **§9.3's sixth citizen is.** `check_start_orders` is `[ok]` on run20 and
  fails on the fuzzed map. A rule built and confirmed on one map that does
  not generalise: exactly what map variation is for, and so far the only
  thing it has produced.

And **run20 had already compared sim-frame 1** — its `DUMP_ALL` window was
[0, 4) — so the "nobody has ever" claim was wrong before it was written.
What the cheap window actually buys is *width*: 300 early frames for 195
MB where run20 bought 4 for 278.

**Zero of the three came from the cheat staging.** All three surface at
frames 0–2, before `scenario.py`'s first `add` at frame 200 could fire.
What has earned its place is `seedini.py` — a new map per seed — and the
early wide window. The random `add`/`resource`/`military` generator, the
part that most looks like fuzzing, has produced nothing yet.

### Was the fix defensive, then?

Worth splitting, because the honest answer is "partly", and the parts are
not the same size:

- **Three of the five guards are defensive.** `is_enemy`/`is_ally` going
  total, and the raw `at_war[a][b]` indexes going through `at_war_with`,
  turn a crash into `false`. They change no outcome the original produces,
  because the original never asks. Robustness, not fidelity — and worth
  having, since a panic costs a ten-minute capture's whole diff.
- **One is a live behavioural fix.** `check_hit` is a `find_unit`, and
  `find_unit` cannot return gaia. Without the bound, a stray arrow that
  lands within two tiles of a sheep **damages the sheep** — animals have
  hit points, so that is a different `DeathsSync`, different health,
  different draws. RoN cannot do it; we could. Reachable whenever a ranged
  unit misses near an animal, and run28's map carries 47 herds.
- **One is right-for-the-right-reason.** `valid_target`'s `7 < who`
  changes nothing today, because gaia is also excluded incidentally by not
  being at war with anybody. It becomes load-bearing the moment the
  diplomacy table is anything other than the lobby's width.

So the underlying defect was real and ours: we assumed a unit's owner is
always a lobby player, and in Rise of Nations it never only is. But
"the fuzzer found a crash" and "we were getting a rule wrong" are two
different claims, and only the second is worth the ink.

### What the day cost, and the note

Four runs, half an hour of wall clock, about 590 MB, plus three re-diffs
of old dumps. Item 13's verdict is still **keep** — for map variation and
the early wide window, not for the cheat generator, and on one finding
rather than three.

**Two notes, and the second is the one that will keep earning.**
Yesterday's was *measure the oracle before building a better one*; today's
first is the same rule one step earlier — **check whether the expensive
setting is doing anything before pricing it**, since a 300-frame window
had been budgeted against 25 MB a frame and costs 0.69, and the evidence
was a `grep -c who2` on a file eleven hours old.

The second: **before crediting a new capture with a finding, run the same
diff against a dump already on disk.** Three claims were made here on the
strength of a new run; two of them were wrong; the control cost thirteen
seconds each and nobody had been in the habit of running it. A novel
capture is the most persuasive kind of evidence and the least controlled,
which is precisely the combination that needs a control.

## 2026-08-26 — item 24: the fifteen draws that were four things

The queue had carried this for four days, in one line: *the sim draws 15
fewer than the original at frame 0, on two maps — run20 reads 160/175, the
fuzzed map 180/195 — so it is one fixed missing block. Needs no new
capture. Find it.*

Both halves of that were right except the middle. No capture was needed;
the block was four blocks; and two of them cancel, which is exactly why
the shortfall came out the same on two unrelated maps and read as one
thing.

### The instrument, which took half an hour and did all the work

`tools/trace/report.py … sites 0` folds the original's frame-0 draws by
return address — that has existed since run14. The sim had no such fold:
`rondata --diff` printed one number a frame, "ours 160 draws, the
original's 175", and there was no way to ask *which* 160.

So: a mark at every phase boundary of `Sim::tick`, and one before every
unit the loop visits, recording the stream's word. The draws between two
marks belong to the earlier one, and a frame becomes

```
rng: frame 0: ours by phase — strategy_all 2, markets 18, unit 0/0 2,
  unit 0/1..0/2 ×2 1, unit 1/6 2, unit 1/7..8/115 ×106 1, gaia 22, farms 6
```

Set that beside the trace's fold and the answer falls out in one reading.
It is gated on `Sim::trace_phases`, which the harness sets and the soak
does not, so it costs nothing where it is not wanted. Half an hour to
build; it turned a four-day-old mystery into a table.

### The four blocks

| block | run20 | the fuzzed map |
|---|---|---|
| `Unit::think_scout` — the sim has no seen map | −10 | −10 |
| the pasture's five animals and its `think_farm_animal` | −6 | −6 |
| `Farms::inc_time` on a farm that grows nothing | +1 | +1 |
| the citizens' stand, spent in the unit loop | +4 | +5 |
| the same guys' wrap, not spent in phase 7 | −4 | −5 |
| | **−15** | **−15** |

The last two are one defect seen twice, and they sum to zero on every map,
because the guys the sim gives an early stand to are exactly the guys the
original wraps late. A total can never see it. That is the argument for
the fold in one line.

### The pasture

`Farms::inc_time` skips a farm whose `+0xbd` byte is 1 — `docs/SYNC.md`
§3.3 has said so since the day it was written, with no idea what the byte
was. `rise.pdb`'s type record says: `FarmStruct+0xbd` is **`farm_type`**,
and the dump prints it, six times, right after `valid`. Run20's six farms
read `1, 0, 0, 0, 0, 4`.

`Farms::add_animals@008d8f30` is guarded by the same byte, and it is the
find: a `farm_type == 1` farm is a **pasture**, and it gets five animals —
`do { … } while (i < 5)` — of **owner 9**, chicken or pig on a coin. Each
is stamped with the farm and its own place in the five, and that byte
phases its `Animal::think_farm_animal` tick: `(o · (slot + 1) + frame) %
128 == 0`. At frame 0 the first animal, `o = 0, slot = 0`, fires and the
other four do not.

So a pasture costs six draws a frame at frame 0 and saves one, and the
sim was spending neither. The trace had been saying so for a day and a
half in plain sight: run20's frame 0 has **109** `Animal::do_idle` idle
rolls and the dump has **104** animals, with the five extra sitting at the
end of the run and the single `think_farm_animal` draw between the first
of them and the rest.

**Owner 9 is why nobody had noticed.** No dump prints a leader-9 object —
run20's first `FULL DUMP` has 104 `ANIMALDATA` records and not one `who 9`
field anywhere in it. The five animals exist in a capture *only* as draws.
It is the cleanest example yet of the thing the trace is for: a mechanic
whose whole footprint in every logger the game ships is a count that does
not add up.

### What landed

`Farm::farm_type`, `Sim::farm_add_animals`, `Sim::think_farm_animal`,
`Sim::build_covers_tile`, `gamelog::Initial::farms` (the list read off the
dump's flat fields — `Farms::log_data` gives it no block of its own), and
the fold. Frame 0 goes 160 → **165** on run20 and 180 → **185** on the
fuzzed map, and **frame 2, which is the crop farms and nothing else on
both sides, goes from 6/5 to 5/5** — the first frame of run20 the harness
matches outright.

Five deliberate breakages, all caught: four animals instead of five, the
think phase off by one, the slot dropped from the phase, the `covers_tile`
gate removed, and the animals never reaching `think_farm_animal`.

### What is left, and it is now one block

**Frame 0's whole remaining gap is `Unit::think_scout` — ten draws on
run20 and ten on the fuzzed map**, at the same three sites in the same
proportions (`+0x436` ×4, `+0x458` ×2, `+0x64c` ×4). That two unrelated
maps give the identical count is itself a lead: a scan drawing "once per
unseen candidate cell" would not. It needs no capture either.

And the ±4/−4 pair is a real defect wearing a zero. Run20's own
end-of-frame-0 dump half-explains it: the AI's two woodcutters end the
frame wrapped (`cur_anim 1, cur_time 0`), the human's two end at
`cur_anim 0, cur_time 1, end_time 33` — a `set_anim` that took no draw at
all. Two of the four wraps have owners; two do not.

### The note

**A number that reproduces is not a cause that reproduces.** Fifteen on
two maps was read as one block precisely *because* it was stable, and
stability was the wrong inference: it was three map-independent blocks
plus a pair that cancels by construction. The fix was not more reading —
it was making the sim's own draws as legible as the trace already made the
original's, which is the same move as every instrument this project has
built, one level in. When two totals disagree, build the fold before
building the theory.

## 2026-08-26 — the scout's ten draws, and the zero-sum defect that stopped being zero-sum (item 24's remainder)

The opener asked for one thing: `Unit::think_scout` draws ten at frame 0 on
two maps, at `+0x436` ×4, `+0x458` ×2, `+0x64c` ×4, and the sim draws none;
the same count on both maps says at least one site is fixed-count, so read
it, no capture needed. What came out is `docs/SCOUT.md` — the whole
mechanic, implemented, and checked against **three** traces seed for seed —
plus a finding the reading was not looking for.

### The reading was steered by the trace from the first minute

The three offsets were the entire brief, and disassembling them first was
what made the rest cheap. `+0x436` and `+0x458` are 0x22 apart and both are
the return of a `Random::get` followed by an `idiv`: two draws at the head
of a loop, on the tables at `00cbe32c`/`00cbe330` — which turn out to be
`circle_radius` and `circle_radius − 1`, so **ring `r` is the index range
`[radius[r−1], radius[r])`** and the whole loop is a ring walk. `+0x64c` is
a `vector_dist` followed by a draw folded to `& 7`: a per-candidate jitter.
Three sites, three roles, before a line of the decompile was read closely.

The counts then constrained the shape. Two draws per ring, the second
guarded by `ring / 4 + frame % 8 >= 1`, means at frame 0 the phase draw
only fires from ring 4 up — so `4 / 2` is **four rings of which the last
two are ≥ 4**, which is 1, 3, 5, 7 and *not* 1, 2, 3, 4. That forced a step
of 2, which forced `unit_masks & 0x40000`, which `Unit::init@00612100:586`
sets exactly when `(leader_flags & 0xc) != 4` — **every unit of a
non-human leader**. `crate::path` had it down as the amphibious bit; it is
the AI-unit bit.

The same discipline placed the caller. `Unit::think`'s tail calls
`think_scout` only after the human block returns at
`if (!(unit_masks & 0x40000)) return`, so a **human's** scout never gets
here at all — and run20's interleaved `HIT` records show exactly that: the
first idle unit reaches `Unit::think_spellcaster` and stops, the second
reaches `Object::get_army` and then `Unit::think_scout`. Two scouts, one
thinks. (`Unit::think_scout`'s `HIT` arriving *late* in the frame is the
proof: an `int 3` is one-shot per arming, so a first entry at record 798
says the unit at 758 never called it.)

### The check that made it a mechanic rather than a story

A count is a weak check and this one had a strong alternative sitting in
the log: **every draw record carries the seed it was taken on.** Install
the seed of the trace's first `think_scout` draw in the harness's own
scout, run one call, and compare the seed left standing. On all three
captures it is exact:

| capture | first draw | ours | sites |
| --- | --- | --- | --- |
| run20 | `0x9c59_1b2b` | **10** | 4 / 2 / 4, ending on the trace's own draw 32 |
| fuzz 424242 | `0x242c_b7ed` | **10** | 4 / 2 / 4, likewise |
| run10/run14 | `0x15fe_bc41` | **24** | **6 / 2 / 16**, split 4/2/1 and 2/0/15 across two cities |

The third row is the one that earns the reading. The Great Lakes map is the
only capture that exercises the **foreign**-city arm — `max_ring = 3`,
`step = 1`, rings 1 and 2 with no phase draw at either — and the harness
reproduces its five hits and its ten without being told anything about
them. Nothing about that arm was inferable from run20.

### And then the finding

Run on the frame's own stream instead of the trace's, the harness arrives
at `think_scout` **two draws early**. That is the stand/wrap swap the last
session found and filed as zero-sum: the sim spends a unit-loop stand for
each gathering citizen where the original spends none and wraps the same
figures in phase 7 instead, +4/−4 on run20 and +5/−5 on the fuzzed map. The
queue's own words were "zero-sum, so no count will ever catch it".

**A count catches it now.** `think_scout`'s draw count depends on the
stream it runs on, because the rotation decides which cells of a ring are
visited and the fog decides how many of those are taken. Run20 lands on 175
either way — its ring 5 gives four cells on both streams — but the fuzzed
map gives five on ours and four on the original's, so frame 0 is **196
against 195**. One cell, and it is the whole of the gap.

That is the second time in two sessions that the same instinct paid: a
defect that hides behind a total stops hiding the moment something
downstream reads the *order*. The `GUYS=4` capture the queue has been
holding for the swap is now the check for a real number, not for a
principle.

### What landed

`docs/SCOUT.md` (thirteen sections, the circle loader re-derived, the
score's four multipliers, the region fallback read and explicitly not
implemented), `crates/sim/src/scout.rs`, the wiring in `Sim::think`'s tail,
five tests in `sim::scout` and
`run20_s_ai_scout_draws_ten_at_frame_0_in_four_rings` in `rondata::diff`.
Frame 0 on run20 is **175/175** — the first frame-0 match the harness has
had — and frame 2 stays 5/5.

One seam closed on the way past: `WorldData::is_cliff_at@0046f8c0` is one
line, `(TData.mask & 3) == 1`, so the two-bit terrain-object field's third
value is named (`tile::OBJECT_CLIFF`) and `crate::path`'s `invalid_loc`
refuses a cliff as the original does (`docs/PATHFINDER.md` §11). It did not
change any count — none of the candidate cells on either map is a cliff —
which is worth saying, because the hypothesis that it *would* is what sent
me to read it.

Five deliberate breakages, four in `sim::scout` and one in the diff check:
`step` 1 instead of 2, the early exit removed, the phase guard relaxed to
`>= 0`, the fog gate short-circuited, and the pinned seed advanced by one
draw. All caught.

### The note

**Read the sites before the function.** The offsets in the queue entry were
not a hint about where to start — they were most of the answer. Three
return addresses, disassembled, gave the loop's shape, its two guards and
its per-candidate cost in about ten minutes; the decompile after that was
confirmation and naming. The general form: when a trace has already told
you *where* the draws are, the arithmetic around each site is a much
cheaper question than "what does this function do", and it constrains the
answer to the second question hard enough that the reading almost writes
itself.

And **the seed is a better assertion than the count.** Every draw record in
`rontrace` carries its seed; a mechanic that reproduces a *sequence* from a
pinned seed is checked in a way a total can never be, and it is checkable
even while the stream that reaches it is still wrong. That is what let this
land with the upstream defect still open — and what turned the upstream
defect into a number.

### The primitive, built the same day

Doing the seed-anchored check by hand twice — once with `eprintln!`, once
as a hand-written assertion — was the whole argument for building it
properly, so it was built before clearing.

`crates/rondata/src/trace.rs` parses `rontrace.log` from Rust. The format
is thirty-two-byte records of eight `u32`, documented in
`tools/trace/README.md` and authoritative in `tracer.c`'s `emit`; the two
fields that matter are the **`Random *`**, which separates the sync stream
from the renderer's four other generators, and the **seed before the
step**, which is the word a replay seeds with. Sites are normalised back to
`0x400000` so a relocated run compares against the export's addresses.

Three things fall out, and all three are reusable:

- `rondata --trace <rontrace.log>` prints the original's per-frame fold
  **by site**, in the same shape `--diff`'s `by phase` note prints ours.
  Run20's frame 0 reads
  `… 5dac7a ×4, 5f6446 ×3, 5f6468, 5f665c ×4, 5f6446, 5f6468, 5dac7a ×105, …`
  — the scout's ten sitting between the four stands and the animals'
  hundred and five, on one line, with no Python in the loop. Frame 1's
  `8d8c7f, 8d8c9b` are the two `Farms::add` draws the queue's next-but-one
  item is about, visible without looking for them.
- `Trace::run_in(frame, lo, hi)` isolates one function's own draws from
  the ones its callees took — `[think_scout, think)` for this mechanic.
- A mechanic marks its own draw sites under the original's offsets
  (`Sim::mark`, already gated on `trace_phases` so the soak pays nothing),
  and `diff::mark_sites` expands the marks into one label per draw.

So the scout check is now a **sequence** comparison: read the trace,
filter to `think_scout`'s own draws, seed the sim with the first of them,
run one call, compare site for site. Made to fail by transposing two of
the three marks — which leaves the count at ten and the order wrong, and
is exactly the class of error a total cannot see:

```
left:  [+0x458, +0x458, +0x458, +0x458, +0x64c ×4, +0x458, +0x458]
right: [+0x436, +0x436, +0x436, +0x458, +0x64c ×4, +0x436, +0x458]
```

Four tests on the parser itself, including a relocated run folding back
and a non-trace being refused.

**Why this one and not another tool.** It is the only kind of check that
works while the *upstream* stream is still wrong, which is the position
every mechanic lands in for a while: the frame that reaches it is off by a
few draws, so nothing about the frame's totals can be trusted, but the
mechanic itself is exactly assertable from a pinned seed. That is a
general shape, not a scout-specific one — item 27 is the list of mechanics
that should get the same treatment, and the trace fold is already printing
their targets.

## 2026-08-26 — items 27 and 26: the frame as one sequence, and the line that was two bugs

Item 27 was "mark the other mechanics' draw sites", filed as cheap tooling
with no run needed, and the queue's opener said to take it before item 26
because it would be the instrument for it. It settled item 26 on its first
run, and the settlement was one line of our own code.

### The table, and why it needs a caller

The scout's check (the previous entry) compared one function's draw
*sequence* against the trace's. Widening that to a whole frame needed the
trace's raw addresses and the harness's marks to be the same strings.

`trace::SITES` is that table: `(address, an optional caller, the label)`,
where the label is a `pub const` in the mechanic's own module —
`sim::market::SITE_A`, `sim::gaia::SITE_HERD_X`, `sim::anim::SITE_WRAP`.
The name lives beside the code that spends the draw; only the address
lives in `rondata`. It is deliberately not a symbol table: naming a trace
in general is still `report.py`'s job with the Ghidra export, and nothing
from that export enters the repo.

The optional caller is the part that had to be there, because **one
address is several sites**. `Guy::set_anim+0x97a` is the idle roll for an
animal, for an idle unit, for a gathering one's stand, and for the phase-7
wrap; the four are told apart only by the record's `ebp` chain. So a row
matches on the site plus, optionally, a frame anywhere in that chain, and
`Guy::set_anim+0x97a < Guy::inc_time+0x271` is a name rather than a
footnote.

Marked this session: `Leader::compute_sites`' two, `calc_market`'s three,
the four `Guy::set_anim` callers, `Guy::init_real`, `think_farm_animal`,
the birds' two, `Herd::process`' two, `Farms::inc_time`'s chance and
sprout, `do_non_flat_gather`'s three stands and two waits, and
`do_move+0xe84`. With those, run20's frame 0 has no unattributed draw
left, and the whole frame is one `Vec<String>` on each side.

### What it found, at draw 22

```
  ≠   22  ours Guy::set_anim+0x97a < Unit::do_non_flat_gather+0x10f
          theirs Guy::set_anim+0x97a < Unit::do_idle+0x7d
```

The sim was spending a `set_anim` roll for each gathering citizen at a
call the original does not make. `do_non_flat_gather`'s camp-arrival
branch is a two-way `CHAR_DUMP_WOOD` / `CHAR_DUMP_ORE`: the decompile
decrements `wait`, sets `been_there`, returns if `wait < 0`, then faces
and dumps, and the listing at `5f0b5e`–`5f0b89` shows those two `set_anim`
calls and no third. `been_there` is written there and never read. Our
`else { set_default_anim(u) }` for a first arrival was invented — in 2026,
by us, to explain four draws on run12's frame 0 that turned out to be
something else entirely.

**Removing it moved four draws, not two.** The stand had been resetting
the citizens' clocks, so the four phase-7 wraps the original spends
between the herd walk and the farms never fell due here. Both halves of
the stand/wrap swap were the same line, and both traced maps' frame 0 now
matches the original **draw for draw**: run20 175/175, the Great Lakes
120/120 (from 128), the fuzzed map 195/195 (from 196). `ticks before
divergence` has not moved — the position divergence at frame 2 is
untouched — but the stream underneath it is now exact for a whole frame on
two maps.

The `GUYS=4` capture the queue had been holding for four days was never
booked.

### The evidence that had made it look half explained

`docs/SYNC.md` §6 recorded that run20's end-of-frame-0 dump only accounts
for two of the four wraps: `1/1` and `1/2` end at `cur_anim 1, cur_time 0`
— a wrap — while `0/1` and `0/2` end at `cur_anim 0, cur_time 1`, read as
"a `set_anim` that took no draw". That reading was the trap. A wrap whose
roll returns **the slot already running** takes `set_anim`'s
same-animation apply, which leaves `cur_time` stepping rather than
resetting it, so it prints as an ordinary step. Two wraps were invisible
in the dump and perfectly visible in the stream. `docs/ANIM.md` §5's "the
four woodcutters' draws are unit-phase stands" and §9's "which gate
skipped their `Guy::inc_time`" are both struck: no gate skipped anything.

### And a second drift, from the same document

`docs/ORDERS.md` §6.4's pseudocode had the camp arrival **right** — it
reads `wait--; been_there; wait < 0 → return; face; CHAR_DUMP_*` — and it
also carries a `set_anim(CHAR_DEFAULT)` before the tile approach's
`find_nearby_spot` that the implementation had simply never had (the
original's site is `+0xfd4`, reached at run21's frame 23,299). Two lines
of one block, both correct in the prose and wrong in the code, for four
days, invisible to every test written from that same prose.

That is `CLAUDE.md`'s own default arriving with a bill attached: *prose
can cite every address correctly and still leave the arithmetic wrong, and
an adjudicator cannot tell without doing the work.* Here the prose was
right and the code was wrong, which is the mirror image and just as
undetectable by reading. The five draw sites of that function are marked
now, so the next drift is an `assert_eq!`.

### What is left, and it is sharper than it was

Run20's frame 1 is 52 against 53, and the fold now names every part of it:
one `Leader::produce_building` draw short, two `Farms::add` draws
unmodelled (item 25), and **two spurious `Unit::do_move+0xe84`** — the
same gate as §6's frame-3 item, now visible a frame earlier and on another
map. Run21's 23,000 frames reach the original's `+0xe84` five times in
total and never under a gather's transit, which is a real narrowing of
that item.

**The thing this session earned.** *A count is not a check, and a
per-block table is barely one.* Frame 0 had read 175/175 for a day with
two errors cancelling inside it; the block table in §4.2 could see the
blocks disagree but not which call made a draw. Naming the **caller** —
four different meanings of one address — is what turned it into a
diagnosis, and the diagnosis was a line of our own code rather than
anything unread in the original. The corollary for the queue: a capture
booked to settle a question is worth re-examining once the instrument
improves, because the instrument may already be able to answer it.


## 2026-08-26 — items 25 and 28: the farm's emitter, and the goal that walks backwards

Two of run20's frame-1 draws were `Farms::add`'s, two were `do_move`'s that
the original never spends, and both were settled from the export and the
dumps already on disk. No capture was booked. Frame 1 went 52/53 → 53/53 on
run20; the Great Lakes and the fuzzed map did not move by a single draw or a
single position, which is how the second change was checked.

### `Farms::add`'s two draws are an ambience emitter, one to a city

`docs/SYNC.md` §3.8 is the new section. The offsets the trace named —
`+0x23f` and `+0x25b` under `Build::init+0x4ea` — are `rand % 3` for `x` and
then for `y`, a third of a tile each, feeding
`GraphicEvents::add_ambience(1, …, 135.0f, who, o)`. The emitter is art. What
is **not** art is the `4` it ors into `farm_type`, because the walk that
precedes the pair looks for exactly that bit on any farm of the same city and
gives up if it finds one: **one emitter a city, ever**, and it is why run20's
sixth farm reads `4` in the dump.

The rest of `Farms::add` is the `farm_type` decision, and it is a six-row
table (§3.8) rather than the coin §3.6 had found. The coin at `+0x128` is the
*last* row, and it fires on none of the three captures: run20's new farm joins
the city that already holds the map's pasture, so `others != crops` settles it
with no draw. Two rows are worth naming for how odd they are — a farm with no
city takes its type from **the parity of its own object number**, and a city
whose farms are all crops turns the fourth or fifth into a pasture outright.

**The off-by-one that decides everything.** `Build::init` writes the
building's `BuildData+0x78` — the record's slot — *after* `Farms::add`
returns, so the farm being placed is invisible to the `count_farms` its own
`Farms::add` runs. The first implementation missed that, counted the new farm
among the city's crops, and took the `others != crops` row for a city's very
first farm instead of the coin. `Farm::valid` is that field now, and the test
that caught it is the one written for the table.

Two attributions were wrong in the documents and are corrected in place.
`Farms::add` runs from `Build::init`, so a farm's record and its `Farms` slot
are created **with the site**, not at activation — the simulation now joins
the list there, which is the order `Farms::inc_time` walks. And
`Farms::add_animals` is called from `Build::activate` line 1209, not from
`Farms::add`: §3.6's "spent where the farm is built" survives only because a
starting pasture is placed and activated in the same breath.

### Item 28 was a line `docs/ORDERS.md` had carried since the first reading

`find_path`'s pull-back: *while the goal's own tile refuses `invalid_loc`,
walk the goal back toward the unit one step at a time, rewriting `mo->waypoint`
and the path stack's top wherever they are the goal.* §4.6 had it. The
simulation had never implemented it.

What it does is put a **woodcutter at the edge of the forest**. Forest refuses
`invalid_loc`, so a citizen ordered at a forest tile is silently re-aimed at
the last open point short of it, and the march that follows never enters a bad
tile. Without the pull-back the sim marched into the forest, found the tile
invalid, had no `go_around_building` to ask, and paid `do_move`'s grid draw for
a detour the original never needed. That is the whole of one of run20's two
spurious `+0xe84` draws, and run20's path-stack disagreements over four frames
fell 25 → 21 with it.

The **other** one is the AI scout's, and it is not the same bug: unit `1/0`'s
straight line clips a *building* at tile `(201, 207)` three tiles short of a
waypoint whose own tile is clear, so the pull-back has nothing to pull.
`go_around_building@005fc350` is what answers that, and it is a mechanic of its
own rather than a fix.

### A stale attribution, caught by reverting rather than by reading

The Great Lakes' frame-3 extra draw has been written down as this same
`do_move` gate since 2026-08-24. It is not: it is a
`Unit::do_non_flat_gather+0x54b`, a citizen picking a tile a frame early.
That was established by building the *same* commit with the pull-back
reverted and diffing the fold — which showed byte-identical output on that
map — so it had already stopped being `do_move` before this session began.
`docs/SYNC.md` §5's table and §6's frame-3 entry both say so now.

The general point is small and worth keeping: **when a change is meant to
move one number, run the captures it is *not* meant to move, on both sides of
the change.** It cost two builds and it converted "probably no regression"
into "identical on two maps, and by the way the third's item was stale".

### What is left of frame 1

Run20 reads 53 against 53 and **is still wrong**: one `Leader::produce_building`
draw short (39 + 4 against our 42), one `do_move+0xe84` over. The two cancel.
A count would call the frame done; §5.1's sequence is what does not, and
`diff::tests::run20_s_frame_1_spends_the_farm_s_ambience_pair` pins each of
the two residues as a row that has to be edited when it goes. The fuzzed map
is blunter about it — 51 against 45, with `produce_building` six over, one
gather draw short and a farm sprout the original does not spend.
## 2026-08-26 — item 25's last half: the three defects behind one draw

The queue asked for one thing: mark `Leader::produce_building`'s two draw
sites the way item 27 marked the others, so that run20's "one draw short"
and the fuzzed map's "six over" become rows instead of subtractions. The
marks took twenty minutes. What they exposed took the rest of the session
and was worth several times the asking price.

### The offsets, settled by the listing

`+0xc99` and `+0x1805` off `Leader::produce_building@006e1400` are
`0x006e2099` and `0x006e2c05`. `llvm-objdump` puts a `cltd; mov ecx,
0x1f4; idiv` at the first — the spiral candidate's `% 500` — and a `cltd;
mov ecx, 0x64; idiv` at the second, the jitter's `% 100`, which settles
which is which without reading the decompiler's line numbering. Ghidra had
in fact printed the first as a stray local (`TVar31.value = 0x6e2099`, a
spilled return address); the listing is still what confirms it, and it
cost a minute.

### One number, three defects, and they cancelled

Run20's frame 1 had read **53 against 53** for two days and the queue said,
correctly, that it was still wrong. With the marks the frame splits into
`+0xc99` 39 against our **41** and `+0x1805` 4 against our **1** — and
those two rows are three separate bugs:

1. **The jitter walks a 2×2.** Both loops at `006e2a78` are inclusive
   (`while ((int)uVar11 <= (int)uVar19)`), so an ordinary building — whose
   `ex == ey == 1` — tries four sub-positions and draws for each that
   `blocked_site` clears. The implementation had `0..ex`, which is one.
   Run20 spends four; the fuzzed map spends **three**, one sub-position
   blocked — the second capture is what makes the inclusive reading a rule
   rather than a coincidence.
2. **The stride-by-three tested the wrong index.** `local_10 = 3` when a
   candidate improves on a standing best past `circle_radius[3]`; the code
   compared the loop's *start* index, which is 0, 1 or exactly
   `circle_radius[3]`, so it never fired. Fixing it moved **no** measured
   number on any capture — the frame-1 call accepts inside ring 2 and
   nothing later improves on it — and it is in anyway, because the next
   call that does find something early walks 105 cells instead of 40.
3. **`WorldData::buildings_allowed` was not modelled at all**, and it is
   the one that mattered. The name reads like the `0x78` field; the
   function (`006b2340`) is a *predicate* — `return (flags & 0x78) == 0` —
   so rock, mountain, forest and the unnamed `0x40` take no building. The
   world dump has carried those flags since the map became a dump. The
   simulation was scoring forest cells and drawing for them.

With all three, run20's frame 1 is `+0xc99` **39/39**, `+0x1805` **4/4**,
and — the check that matters — the farm the call places lands at
`(41856, 39552)`, the `who 1, o 2006` record of the run's own `BUILDDATA`,
to the unit. It had been one sub-position away, which is what a jitter fed
a stream two draws late does.

### The residue that vanished for the wrong reason

On the intermediate build — jitter fixed, `buildings_allowed` not — run20's
lone `Unit::do_move+0xe84` disappeared, and the frame read 55 against 53
with `go_around_building`'s row at zero. It was not closed: the farm had
simply moved out of the scout's line. The next fix put the farm on the
original's tile and the draw came back. **A residue that vanishes when an
unrelated object moves has not been closed**, and the only reason this was
caught is that the fold names the site rather than counting draws.

### What the captures say now

| capture | frame 1, before | after |
|---|---|---|
| run20 (islands) | 53/53, wrong in three places | **54/53**, and the only residue is `go_around_building`'s one grid draw |
| fuzzed 424242 | 48/45 | **43/45** — one spiral candidate short, one `do_non_flat_gather+0x54b` short, jitter 3/3 |
| Great Lakes | 120/54/6/7 vs 120/54/6/6 | unchanged, byte for byte |

Run6's long pin was re-based, and the honest shape of that is worth
recording: its **totals fell**, 1,679/1,199 order and path disagreements to
1,552/1,160, while the split's non-farmer half rose 1,246/879 → 1,267/892,
because the farmers' share fell further than everyone else's. The ratchet
excludes the farmers by object number, so a change that helps them
disproportionately reads as a regression in the half that is asserted.

### The thing this session earned

*Mark the phase before believing the total — and mark it even when the
total already agrees.* Frame 1 agreed exactly while carrying three
independent errors, and one of them was a map layer the harness had been
loading and never reading. Item 27's instrument has now paid twice
(§5.1's stand/wrap swap was the first); the queue item that says "one draw
short" is the shape of question it answers, and the answer is rarely one
draw.

## 2026-08-26 — item 29: `go_around_building`, and the residue that belonged to another unit

The last `Unit::do_move+0xe84` on any capture is gone, and run20's frame 1
is **53 against 53 draw for draw over the whole sequence** — frames 0, 1
and 2 all match on that map now. What closed it was a mechanic rather than
a fix: `Unit::go_around_building@005fc350`, the tile-edge walk that answers
a straight line clipping a *building*, plus the half of `find_path` that
decides whether the detour it pushed counts.

### The shape

The pull-back (item 28) cannot help here. It walks the *goal* back until
its own tile is clear, and this goal's tile is clear already; the obstacle
is in the middle of the march. So the original does something else
entirely. The step that failed crossed a tile edge, which gives the unit a
**lane** — the row or column it is still standing in — and a **blocked
lane**, the one it tried to enter. The walk runs perpendicular to the
crossing, one tile at a time in *both* directions, asking two tiles at each
offset: the unit's own lane, and the blocked one. It stops when the blocked
lane opens (success) or when the unit's own lane closes (success only if
the blocked lane happens to be open there). Off the map is that direction's
failure; both failing gives up.

Of the two winners, the closer to `mo->dest_x/dest_y` — forward on a tie —
and then one or three `PathData`s go on the stack. Three when the unit's
own lane is blocked at the found offset: a turn-in point in the blocked
lane, a `flags 8` midpoint between the lanes, and the target one tile back.
One otherwise. Every one of them is placed at `tile*0xc0 + 0x30 + (off %
0xc0)/2` rather than the tile centre, from the order's own `off_x/off_y`,
so a formation's units do not all aim at the same point.

Then `find_path` lifts them all off again, drops any that lands on the
unit, refuses six degenerate cases, and **recurses on what is left**. A
verified line puts them back with each entry's `flags & 4` recomputed
against the one above it — set when exactly one of the two tiles is ocean,
so a leg crossing the shore turns before it walks. A refusal discards them
and pays the grid draw, exactly as before. `docs/ORDERS.md` §4.6.1.

### The check is the original's own stack

Not the count — the count is what misled this document twice. Run20's unit
`1/1` ends frame 1 with

```
[{(41640, 39384), tol 0, flags 1}, {(40644, 39036), tol 0, flags 0}]
```

and the log's frame-2 record for `1/1` is those same two entries,
coordinates, tolerances and flags. The second is the edge walk's output,
and the `36` separating `40644` from the tile centre `40608` is the `off %
0xc0 / 2` skew — pinning the skew at `0x30` moves it to `(40608, 39072)`
and the assertion fails, which is how it was made to fail on purpose.

### The residue was never the unit `docs/SYNC.md` named

§6 had it as the AI scout, `1/0`, at `(38040, 40344)` on a waypoint of
`(38784, 39552)`, clipping the building at tile `(201, 207)`. When the
mechanic landed and the blocked branch was printed, the only unit that
reaches it on run20's frame 1 is **`1/1`**; `1/0` never enters it, because
its `find_wpath` chain has moved — most likely when `produce_building`'s
three fixes moved the AI's farm the session before. The count was one both
times.

*A residue's owner is a measurement too, and naming it costs one print.*
This is the sibling of last session's rule about phases: the fold names the
site, and the site is not the same thing as the unit paying for it.

### What the same print found on the way

Printing `1/0`'s stack beside the original's paid for itself twice over,
because the two are the same length and made of different numbers:

| | ours | theirs |
|---|---|---|
| the goal, at the bottom | `(41976, 36600)` | `(41952, 36576)` |
| the world chain | `cell*0x300 + 0x180` (the cell centre) | `cell*0x300 + 504` |

The second row is **`toff`**, and `docs/PATHFINDER.md` §7 had it written
down all along: a reconstructed world node is pushed at
`node + toff − 0x180`, and `crates/sim/src/path.rs` carries `toff = 0` as a
stated seam because "move orders here have point goals". The arithmetic
closes — `504 − 0x180 = 120`, and the order's own `off_x` is `504` — so the
seam is wrong for a *plain* move too: `toff` is the move order's
`+0x4c/+0x4e`, which is `off_x/off_y`. That single number is most of
run20's remaining path-to disagreements and all of `1/0`'s position drift.
The first row then follows: the order is at `41976` on both sides, so the
`0x18` is `find_wpath`'s pre-walk moving the goal before the dump sees it.
Both are on disk, both cost a grep, and neither needed a run.

### What the captures say now

| capture | before | after |
|---|---|---|
| run20 (islands) | frame 1 **54/53**, one `+0xe84` | **53/53, the whole sequence** |
| run20 path stacks, five frames | 21 (path-length 5, path-to 16) | **17** (3, 14) |
| fuzzed 424242 | 195/195, 43/45 | unchanged |
| Great Lakes | 120/54/6/7 vs 120/54/6/6 | unchanged |
| run6, 432 frames | 1,552 / 1,160 | unchanged |

The three unmoved rows are the point: no capture on disk except run20
reaches the blocked branch at all, so this is one mechanic landing without
disturbing anything else.

### What it did not establish

No capture reaches the diagonal entry, the `spd − 1` recursion, the
turn-in pair, the `0x300` cut-off, or the shore's `flags & 4`. Those are
read, not diffed, and `docs/ORDERS.md` §4.6.1 names the window that would
settle them. Two tests stand in for now without the install — a citizen
whose line clips a barracks detours and never re-plans, and a mountain
ridge with no way round gives up and pays the grid draw — and both were
made to fail on purpose before landing.


## 2026-08-26 — items 30 and 31: `toff` is the order's, and the goal is the group's

Two rows of last session's side-by-side print, taken as written. One of
them was right about the number and wrong about the reason; the other was
right about the number and wrong about *which document* the answer was
already in.

### `toff` — the two folded vtable slots

`docs/PATHFINDER.md` §4.1 said `astar_path`'s prologue reads the offset
"if the current order's target is a transport-relevant unit (vfunc
`+0x14`)", and took `+0x4c/+0x4e` off the target. Both virtuals belong to
the base `UnitOrder`, and the PDB's own method list names them —
`+0x14 is_move`, `+0x40 update_move_order` — which `docs/ORDERS.md` §4.1
had printed correctly a month ago. Ghidra's `vtables.txt` shows neither,
because both slots hold **COMDAT-folded** stubs and the linker keeps one
name for the fold: slot `+0x14` prints as `StrafeOrder::is_air` on
`MoveOrder` and as `Window::get_button` on the base, and slot `+0x40`
prints as `MoveOrder::get_move_order`, which is a *different* slot
(`+0xb8`) with the same body.

The bodies settle it in three lines of `llvm-objdump`:

| address | bytes | meaning |
|---|---|---|
| `0041e0e0` | `b8 01 00 00 00  c3` | `is_move` on the move family: **1** |
| `0041bff0` | `33 c0  c3` | `is_move` on the base `UnitOrder`: **0** |
| `00482f60` | `8d 41 ac  c3` | `update_move_order`: `this − 0x54`, the order's own `MoveOrder` |

So `toff` is **the current move order's `off_x/off_y`**, which
`Unit::add_move_facing_order@005e55c0` writes as `x mod 0x300` — the
destination's offset inside its own world cell — for every move, targeted
or not. Nothing about a target is involved anywhere. The same misreading
sat in §3's `find_tpath` line, where `docs/ORDERS.md` §4.6's table had
been right all along; both are corrected in place.

*A folded vtable slot is named by the type record, never by the listing —*
and this is the second time that rule has paid (`docs/audit/README.md`).
The memory note about `LF_ONEMETHOD vftable offset` was written for
exactly this and had not been used on a *predicate* before.

### What it bought, against the original's own chain

Run20's unit `1/0` walks a `find_wpath` chain the original logs at
`cell*0x300 + 504` on both axes. The simulation emitted the cell centre:

| | before | after |
|---|---|---|
| `1/0`'s world nodes | `(41856, 38016) (41856, 38784) (41088, 39552) …` | `(41976, 38136) (41976, 38904) (41208, 39672) …` |
| on the original's stack | **none** | **three of five**, entry for entry |

`diff::tests::run20_s_world_chain_sits_on_the_move_orders_own_offset` is
the assertion, and it was made to fail twice on purpose — once with `toff`
pinned to `(0, 0)` (the chain lands on the cell *corner*), once with the
offset removed altogether (the cell centre, the old seam). Two unit tests
in `path.rs` carry the same claim without the install.

The path-stack **count** did not move — still 17 over five frames — and
that is the honest reading: the two stacks are now the same numbers
*offset by one slot*, because the original's is nine entries and ours is
seven. A slot-wise diff cannot see a chain that agrees but is shifted, so
the count is the wrong instrument here and the entry-for-entry test is the
right one.

### The other row: the goal, which was never the pre-walk

`(41952, 36576)` against the order's `(41976, 36600)`, `0x18` short on both
axes. Last session's guess was `find_wpath`'s pre-walk. It cannot be:

- the AI walk steps through `sin_table` by `0x180`/`0x30` and cannot
  produce `−0x18` on both axes at once; the human variant only pops;
- `Unit::do_move` pushes `{mo->x, mo->y}` verbatim, and
  `add_move_facing_order` writes `x = u*0x30 + 0x18` — so **every** `mo->x`
  is `≡ 0x18 (mod 0x30)` and `41952` is `≡ 0`. It cannot be an `mo->x` at
  all.

What pushes it is `Group::action_move_near@00704990`, and
`docs/GROUPS.md` §6.7 had it in plain words: the group plans **one** path
on the global `grouppath` whose `FINAL` entry is the **leader's raw slot
destination**, read out of the form table at `form+0x514`/`form+0x714`,
un-snapped, and hands the result to every member. The dump agrees on every
number: `1/0` carries `group 65`, `form 0`, `form_mod 50`, an
`EXPLORETOORDER` whose `orig_x/orig_y` are `(41952, 36576)` — which only
`action_move_near` and `ungroup_move_order` ever write, and the latter
writes `orig = x` — and whose `x/y` are that same point snapped.
`Group::finish_insert@0070e620` is the caller: a unit that joins a group
holding an `EXPLORE_TO` has it re-issued through `action_move_near` at the
old order's `orig`.

So item 31 is not a pathfinder item. It is **§6.7, which the simulation
does not implement at all** — and `docs/GROUPS.md` §12 claimed it did,
which is corrected. The gap costs run20 three separate disagreements at
once, all on the dump already on disk: the goal, the *timing* (the
original's `1/0` is `is_pathed` with nine entries on the frame ours has an
empty stack and `flags 0`, because the group planned at order time and the
simulation waits for `do_move`), and the *route* (the original's chain is
the leader's, translated; ours is each member's own from its own
position).

*The answer to an open question is sometimes already written down in
another mechanic's document.* The queue named the pre-walk and the
pre-walk was innocent; one `grep` of `docs/GROUPS.md` for the word "slot"
would have cost a minute and saved the reading. **Before booking a reading
for a residue, grep the documents of every mechanic the value passes
through, not only the one the residue was measured in.**

### What the captures say now

| capture | before | after |
|---|---|---|
| run20 (islands) frames 0/1/2 | 175/175, 53/53, 5/5 | unchanged |
| run20 path stacks, five frames | 17 (3, 14) | 17 (3, 14) — *the numbers changed, the slots did not align* |
| run20 `1/0`'s world nodes on the original's stack | 0 | **3** |
| fuzzed 424242 | 195/195, 43/45 | unchanged |
| Great Lakes | 120/54/6/7 | unchanged |
| run6, 432 frames | 1,552 / 1,160 | unchanged |

### What it did not establish

The middle of `1/0`'s chain still parts — ours walks `(52,51) (51,51)`
where the original walks `(52,50) (51,50) (50,50)`, and the original
carries one node between the goal and the first shared one that ours does
not. Both sit inside §6.7's leader-path-plus-offset, so they are worth
re-measuring only once that lands, not before. Nothing here was checked
against a run: every claim is either a byte in the PE or a field in a dump
already on disk.

## 2026-08-26 — item 31: the group plans, and the decompiler's one bad stride

Last session found the owner of run20's `0x18` residue and named it:
`Group::action_move_near` plans one path of its own, at order time, off
the leader's **raw** slot, and hands every member a translated copy.
`docs/GROUPS.md` §6.7 had the mechanic in plain words and the simulation
had none of it. This session built it — `crates/sim/src/grouppath.rs`,
about 250 lines — and the interesting part is not the code.

### Two of three, and the third changed owner

| | before | after | the original |
|---|---|---|---|
| `1/0`'s order flags at frame 1 | `0` | `1` (`PATHED`) | `1` |
| its path stack at frame 1 | empty | 7 entries | 9 |
| the stack's bottom | `(41976, 36600)` | `(41952, 36576, 0, 1)` | `(41952, 36576, 0, 1)` |
| whole entries shared | 0 | **4** | — |
| `rondata --diff`, order disagreements | 1 | **0** | — |

The **goal** and the **timing** are closed. The **route** is not, and it
is no longer this mechanic's: both sides now plan on the same frame, from
the same position — `(38040, 40344)`, which the new test pins — to the same
goal with the same `toff`, and `astar_path` still walks cell row 51 where
the original walks row 50 and drops a node at each end. That went to
`docs/PATHFINDER.md` §12 with the cheapest thread named: the **first
step**, orthogonal in the original and diagonal here, two cells from the
start on open ground.

The third half of item 31 — the *translation*, `slot[i] − slot[leader]` —
run20 cannot measure at all, because its group has one member. That is
now the only thing `docs/GROUPS.md` §13 owes this section, and it wants
the `UNITS=3` window item 23 already books.

### The count went up, and the count was already known to be wrong

`rondata --diff` scores run20 at **21** path-stack disagreements where it
scored 17. Nothing regressed: at frame 1 there is now a seven-entry stack
to disagree with instead of an empty one, and the differ compares slot for
slot from the bottom while the two chains agree one slot apart. Last
session's queue said exactly this about the frame-2 comparison and moved
to an entry-for-entry test; this session did the same for frame 1. *A
metric that rewards having no answer over having most of one is not a
metric.* Worth remembering when the path-stack differ is next touched: the
fix is an alignment, not a threshold.

### The decompiler's one bad stride

The area guard pulls a member's waypoint back into the leader's cell when
the slot offset has pushed it across a coastline. Ghidra renders it as

```
iVar22 = (wy / 0x300 - fy / 0x300) * 0x300 + fy;
piVar24 = fx + (wx / 0x300 - fx / 0x300) * 0xc0;      // <- 0xc0
```

— a tile stride on `x` against a cell stride on `y`. That is *exactly* the
shape of an original bug this project reproduces on purpose, and a
faithful port would have shipped it. It is not one. `70672d`–`706764` is
`leal (%eax,%eax,2)` then `shll $0x8` — × 3 × 256 — on both axes, with
`imull $0x2aaaaaab` / `sarl $0x7` either side, which is a signed divide by
`0x300` and not by `0xc0`. Two minutes of `llvm-objdump` against a defect
no test on disk could have caught, because no capture crosses a coastline.

Three more the listing settled while the code was being written, all in
§6.7 and §16 now: the `0x900` is measured from the search's own start to
the leader's slot (not from the group to the order's point); `grouppath`
and `cols` are **function-local statics** of `action_move_near`, and
`cols` — which Column indexes its non-final waypoints by — is written by
no instruction in the export, so that arm reads uninitialised heap and is
a seam by necessity; and `pathfinder +0x70` is not a private hint but the
*same* `army` mode `find_wpath` derives for an AI's own units, forced on
from outside, which is the only way a **human**'s army ever gets AI army
costing. The engine names the pair itself:
`PathFinder::find_wpath_army@00683730` is four instructions and has zero
callers, because it is inlined here — which `docs/PATHFINDER.md` §2's
`army` row had already noticed.

### The guard that could not fail

Five checks landed, each made to fail on purpose first, and one of them
did not fail on the first try. `a_short_group_move_plans_no_route_at_all`
put its goal two cells east; removing the `0x900` gate left it green,
because `find_wpath`'s **own** near test exits under a cell-Manhattan of 3
and produces the same one-entry stack. The distance moved to
`(1500, 1500)` — `vector_dist 2250`, cell-Manhattan 4 — where only the
`0x900` gate can produce it, and then the breakage was red. *A test whose
subject is one of two short-circuits has to be placed against the other
one*, and the way to find that out is the same way as always: break the
thing on purpose and watch.

### What this session earned

*The decompiler is a reading tool, and the place it is least trustworthy
is a fold of `lea` and `shl` into a multiply.* Both of this session's near
misses were that shape — the `0xc0` stride, and `cols._padding_` standing
in for every field of a static. Neither is a subtle semantic question; both
are one `llvm-objdump` away. The rule the audit README already carries —
*when the decompiler prints a local that cannot be right, the listing
settles it in a minute* — extends: when it prints a constant that **could**
be right and would be a bug, check it anyway. A plausible bug is more
dangerous than an implausible one.

## 2026-08-26 — item 32: the first step was never the wheel's

Item 31 had stripped run20's `1/0` down to one variable. Both sides
planned on the same frame, from `(38040, 40344)`, to `(41952, 36576)`,
with the same `toff (504, 504)`; four of the sim's seven waypoints were
the original's whole, and the middle parted. The queue booked it as
"`astar_path`'s direction wheel or `calc_cost` on open ground, two cells
from the start", and the opener said as much.

**The ground was not open.** The first thing this session did was print
the map around the route, and cell `(50,51)` — the diagonal step the sim
took where the original stepped orthogonally to `(49,51)` — carries
`WData.blocked = 9`. So does `(51,51)`, the next one the sim walked.
§5 prices that at `+20 × 9 = 180` a cell, and `calc_cost` was charging
zero for it, because the terrain term was a seam:

> SEAM: terrain movement cost — the cell byte the original reads (`+0x11`,
> `+0x13` for iroquois) has no layer here; grass is 0

That was true when it was written. It had not been true since the `WORLD`
dump's `WData` records started loading. The same paragraph three lines up
said the same thing about fog — *no fog model, every cell is seen* — and
`World::seen2` had been installed from the dump for as long.

So this was not a reading. It was two stale seams and a `grep` for the
word.

### They had to land together, and that is the finding

The tempting move is to land the terrain cost, watch the number fall, and
stop. Landing it alone makes run20 **worse**: 21 path-stack disagreements
become 24. Landing the fog branch alone gets 12. Landing both gets **0** —
nine waypoints, position, tolerance and flag, entry for entry, and the
whole order stream matched over the five frames the window steps.

The reason is in §5's own numbers and it generalises past this capture. A
scout's base cost is `8` on ground it cannot see and `0x400` on ground it
can: a factor of **128**. Whether a cell is known and what it costs once
known are not two independent terms that happen to sit in the same
function — they multiply. Pricing terrain while calling every cell seen
tells a scout the whole map is expensive and the rough parts more so;
pricing fog while calling every cell free tells it the dark is cheap and
nothing else. Only both together say what the original says, which is
*the dark is nearly free and the light is dear, and inside the light the
rough is dearer still*.

The generalisable form: **a seam is not a per-term stub when the terms
share a multiplier.** Closing half of a product is not half a fix.

### What came with them

Once `tcost` could be non-zero, §5.1's corner-cutting probes became
reachable for the first time — they are gated on `tcost != 0 || iroquois`,
so a flat world never ran them and the table sat in the document with
nothing to check it. They went in from the audit's byte-verified table,
with the direction recovered from the step's **world-cell delta** rather
than from the wheel index: a big unit's two-cell stride matches no
`move_x` entry, and the block is skipped for it. That is the original's
behaviour, and taking `dir` instead would have silently invented a
different one the day big-unit strides go live.

The halfland `base ×= 3` came too, and with it the one thing the
transport tail had been getting wrong: `00685773`–`006858b9` gates the
multiplier on whichever `needs_transport` ran **last**, and at `depth == 1`
the second probe overwrites the first. The sim was testing the first
either way, and testing `crossing != 1` where the original tests the
reassigned value.

### Twelve breakages, and one the captures cannot see

Every new term got a unit test, and every unit test was made to fail:
terrain off, the `+100000` off, the army's `+10000` off, fog off, the
scout's base off, halfland off, the corner probes off, `any` swapped for
`all` on the diagonals, `all` for `any` on the cardinals, the depth gate
widened, the forest-walker exemption removed. Twelve, all caught.

The twelfth is the interesting one. `calc_cost` reads the fog at
`div_3_table[to >> 7]` — the **half-cell** containing the step, `to /
0x180`. `Unit::think_scout` reads `2c + 1`. They are different functions
with one letter between their names, and swapping one for the other in
`calc_cost` moves **not a single number** in run20 or in the 301-frame
fuzz map. Every world-grid node in every capture on disk sits at a
sub-cell offset past `0x180` — run20's chain is at `+504`, its start at
`+408` — so the two readings agree everywhere a run has ever looked. The
decompile is unambiguous; the unit test is the only thing holding it; and
the capture that would separate them is any world path whose unit stands
under `0x180` inside its cell. That is now in §12 by name.

### The row that got worse, and why it is the right kind of worse

Run10's frame 102 went from 6 draws to 24, and the run13 window test
caught it. The cause is not the fog branch: it is that the fog the sim
reads is the **frame-0 snapshot** and nothing ever updates it, because
line of sight is not modelled. `World::set_fog` has one caller. At frame
1 that is the truth; at frame 101 the AI scout is planning against a map
it walked off the edge of a hundred frames ago, its unit-grid search
fails, `find_upath` kills the `EXPLORE_TO`, and `think_scout` spends
eighteen ring draws re-targeting.

Pinning the fog read back to "everything is seen" makes that row `(6, 6)`
again and makes run20's chain part. So the 24 is asserted as it stands,
with the cause named in the test, and the fix — reveal cells as units move
— is the queue's next pathfinder item rather than a quiet revert. A stale
input surfacing as a visible failure is the system working; the failure
was there before, invisible, in every path the sim planned past frame 1.

### What this session earned

*Grep the seams before booking the reading.* The queue named a direction
wheel and a cost tie and pointed at two document sections. What the item
actually needed was to notice that two `SEAM:` comments were describing a
world the sim had stopped living in. Both had been written truthfully and
neither had been re-read since the layer under it landed. The rule the
project already has — *grep the writers of every field you call frozen* —
has a sibling: **grep the readers of every layer you land.** A seam is a
claim about the world, and a claim about the world goes stale.

## 2026-08-27 — item 33: the fog moves, and the field the dump was already printing

`docs/VISION.md`, `crates/sim/src/vision.rs`, `seen`/`seen2` on
`crate::world::World`, `mylos` in `rondata::diff`.

Item 32 left the pathfinder reading a fog grid nothing wrote. This closed
that: `Object::update_seen@00651b80` in full — the radius, the centre, the
two index ranges, the write — plus the two callers that decide when it
runs. The reading was ordinary and the decompile was generous. Three
things it settled that were not obvious:

**`LOS` is in tiles and the radius is half of it.** `unitrules.xml` says
so in a comment nobody had read (`LOS = Line of sight, in TCoords`), and
`update_seen` computes `(los * 0xc0) / 0x180` — a tile over a half-cell.
A Citizen's `2` is a radius of **one** fog cell; a Scout's `4` is two.
The listing at `651c0d` is `lea (%eax,%eax,2); shl $6` and the
divide-by-384 magic sequence, which is how the `0xc0` was pinned rather
than assumed.

**A small land unit sees from in front of its nose.** Below radius four —
which in the Ancient age is every unit a game starts with — the disc is
centred not on the unit but on `project(x, y, dist = 0x180, angle)`, one
half-cell along its facing. Ghidra dropped both of `project`'s register
arguments; `mov 0x50(%ecx), %ecx` at `651cf1` (the unit's `angle`) and
`mov $0x180, %edx` at `651d00` put them back. A citizen's vision is
mostly this.

**`ring_init` is a thickened circle, and its offsets are in `.rdata`.**
The decompiler mis-symbolised three of its globals — `circle_radius − 1`
printed as `KeyMap::await_keymap`, `ring_radius − 1` as
`ConquestBonusCardTypes::loads` — so the listing settled the loop and the
PE's own bytes at `00add254`/`00add214` settled the four orthogonal
offsets. It matters only from radius four up, which no capture on disk
reaches, so it is held by unit tests alone and the document says so.

### The `grep` that was worth more than the reading

`docs/VISION.md` was written from the decompile, blind, the way a first
reading is. Then one `grep` for `mylos` in a log already on disk turned
its central section into a differential check: **every object record, in
every capture, at every detail level, prints `ObjectData::mylos`** —
`update_los`' whole output. `rondata::diff` compares it now, `--diff`
tallies it, and run10 asserts **26,433 unit-frames with exactly one
disagreement**.

That one is a cache. `Leader::calc_unit_stats` refreshes `mylos`, and
`Leader::process` calls it only when `leader_flags & 0x4000000` is set —
the twin of the `0x8000000` this simulation already models as
`wall_stats_dirty`. So the original's value is one frame behind its
inputs, and the sim's pure function is one frame ahead: on the frame
player 1's first science level lands, the sim says the Scout sees `4 + 1 ×
2 = 6` and the original still says 4. **That pair is what proves
`epoch[3]` is the Science line** — the type record named the field
(`LeaderDataEncrypt +0xe8 int[4] epoch`, so the `^ 0x87` byte read at
`+0xf4` is its fourth entry), and the run confirmed the name.

### What the check found, which was not about vision at all

On its first run, 5,170 of those 26,433 unit-frames disagreed with `ours
0`. A unit the simulation *trained* had never been given its type:
`advance_job` set `kind` and left `Unit::ty` unset, so it had no `LOS`, no
combat profile, no speed of its own and no worker role. `Unit::init` sets
all of them and the harness's own dump loader always had; the production
path did not, and a comment said so — *"the unit's `ty` stays unset here,
as it always has"* — which is what an unmeasured gap looks like from the
inside. 5,170 → 1.

It cost something. The AI's trained citizens **started gathering**, so
run6's non-farmer path ceiling rose from 892 to 1,602: a unit that idles
disagrees once a frame and a unit that works disagrees in detail on every
frame it lives. Every traced check held to the number — run20 175/175,
53/53, 5/5 and its world chain entry for entry; the fuzzed map 195/195,
43/45 and 1,377/1,221 unmoved — which is the same bar the earlier
re-bases used, and it is written into the assertion.

### The row this item was booked to close, and did not

`docs/PATHFINDER.md` §12 blamed run10's frame-102 draw gap on stale fog:
24 against the original's 6, the AI scout re-targeting against a
hundred-frame-old map. With the fog live it went to **22** and stopped,
which is what sent someone to the trace instead of the theory.

Both sides give the scout the same `EXPLORE_TO`, to `(45048, 19704)`, and
the destination agrees on every frame from 57 to 101. On **frame 62** the
original stands still for exactly one frame and turns, stepping a constant
`(−19, +29)` from 63 to the end. The simulation takes one more step
north-east, then **stands still for seven frames**, and eases into the new
heading over eight more, reaching `(−19, +29)` only at 77. Ten frames
behind, it arrives at 101 where the original arrived at 95, and spends its
eighteen `think_scout` ring draws on 102 where the original spent them on
96.

So the row is `docs/MOVEMENT.md`'s stopped-unit instant turn, not the fog.
It is re-pinned at `(22, 6)` with the frames written into the test, and it
is the queue's item 34.

### Thirteen deliberate breakages, and two guards that had to be fixed first

Ten against the unit tests — the radius divisor, the science term, the
projection and its gate, the ring table's reach, `ring_init`'s patch
condition and its strict inequality, the half-cell trigger, the resync's
frame, `set_seen`'s return — and three against the diff: a trained unit
losing its type again, the nomad term firing on every map, the merchants'
fixed radius applied to every type.

Two of the ten came back **green**, and the lesson is in what was wrong
with the guards rather than with the code. A test that counts fog cells
cannot tell a *skipped* sweep from a *repeated* one, because both reveal
nothing new — so `moved_to` now answers `None` when the half-cell test
declined, and the test asserts on that. And asserting that a sweep is
flagged `ring` does not assert that it walks the ring *table* — so the
indices are compared against `ring_radius` directly, and against the
circle's for inequality. Neither hole would have been found by writing
more tests; both were found by breaking the code on purpose and watching
the tests not care.

### What this session earned

*Grep the dump for the field before booking anything.* The project already
has "grep the dump before booking a reading" for open questions. This is
the stronger form: the field you want may be in a capture you already
have, at a detail level you are already paying for, and it may turn a
whole document's central section into a differential check. `mylos` cost
one `grep`, produced 26,433 rows of evidence, confirmed a type-record name
by behaviour, and found a bug in a *different* mechanic that no amount of
reading about vision would have surfaced. `docs/ORACLE.md` now says which
fields the `OBJECT` level gives away for free.

## 2026-08-27 — item 34: the body was never chasing, and the angle was never the facing

The queue booked this as "the stopped unit's instant turn", with run10's AI
scout `1/0` as the pinned case and frames 62 to 78 as the evidence: the
original stands one frame and steps a constant `(−19, +29)` thereafter, the
simulation stands seven and eases in over eight more. Ten frames of lag that
never close, and one `think_scout` re-target six frames late because of it.

It was three things, and none of them was the turn rule `docs/MOVEMENT.md`
already had.

### Read the dump first

The whole diagnosis came out of one `track.py` run over a log already on
disk. `UNITDATA` at run10's frames 50–85 gives the scout's position, its
`angle`, its `tolerance`, its path stack and its order — and at `GUYS=2` its
figures' positions and angles too. Two facts fell straight out.

**The original's frame 62 turned 78° in one frame.** From `0x5fb30000` to
`0x973c0000`, at a standstill, with the unit not moving. That is
`turn_towards` snapping, which needs the instant rate, which needs
`last_speed == 0`.

**And guy 0's angle that frame was neither.** It read `0x61d5221c` — exactly
`0x222221c` past the old one. A rate-limited turn, and not by anything a
multiple of `UNIT_TURN_SPEED`. Solving `0x222221c × n` for a plausible base
gives `n = 9` against a 27° type: `avg_speed / 4 + 1 = 9` means `avg_speed`
between 32 and 35, where an eleven-eighths chase of a 34-speed unit would
have held 43.

So two different fields, and an average that was too high.

### `Guy::move` for guy 0 does not chase

The first test after the walk animation is
`guy_num == 0 || (track_dx == 0 && track_dy == 0)`, and it goes straight to
`set_new_location(des_x, des_y)` with `last_speed = vector_dist(dx, dy)`. No
turn, no trig, no clamps, **no 11/8**. The body is written onto the unit every
frame; the walking branch underneath belongs to a guy with a track offset,
which guy 0 never is.

The consequence is the whole of the mechanic. `vector_dist(24, 24)` is 36, and
`(3a + 36) / 4` truncates to a fixpoint of **33** — so the turn rate divides
by nine, not eleven, and the original's `0x222221c` comes out to the unit. And
a unit that spent its frame turning in place has `last_speed` zero *that same
frame*, which is what arms the next frame's instant turn. Seven frames of
standing become one.

The 11/8 has now been wrong twice in this document, one field apart: the first
reading put it on the unit, the audit moved it to the body, and it belongs to
neither. Both readers walked past the `guy_num == 0` test on their way to the
arithmetic under it — which is `docs/audit/README.md`'s oldest lesson, that
the predicates are where the errors are.

### Nothing had ever set the instant-turn flag

`crates/sim` had `instant_from_stop` on `movement::Turning`, had the rule, had
a unit test for it — and nothing outside the hand-built test scenes ever set
it. Every unit built from the shipped data came out with it clear, so no unit
in the simulation had ever turned instantly. `sim::turning_of` derives the
whole `Turning` from the type now.

The predicate is `Guy::init_real`'s, and reading it out has a detail worth
keeping: objmask `0x20` and `0x1000` are `FOOT` and `MOUNTED`, so it is the
data file's own note — but `unit_flags & 2` vetoes it, and that flag's legend
is *"Unit is a horse-drawn cart type thing"*. A cart does not pivot.

### `UnitData::angle` is the heading

`Unit::move_step` calls `find_angle` and then `set_angle(want, …, 0)` at once,
before any turning. So `+0x50` is the bearing to the destination, quantised to
2¹⁶ of a circle, and the direction the step is actually taken along is
`GuyData::angle` at `+0x18`, which only `Guy::do_turn` moves and only by the
rate. `Unit::set_angle` writes the heading into guy 0's `des_angle` as well —
which is what an idle body turns toward — and the third argument is a snap
flag the step passes zero for at all three of its call sites.

That corrects this document, `docs/ORDERS.md` §4.5 (which had raised the flag
and was right to), and `docs/GROUPS.md` §6.6, whose "mid-frame quantity" now
has a name and a bound. It also un-conflates `UnitData::dest_angle` at `+0x58`,
a third field that is the *order's* angle and which `crates/sim` had been
storing the heading in.

One more field fell out for free: `Unit::init` writes **`0x55555555`** into
both, so every unit faces 120° until its first order. The simulation was
starting them at north.

### The check, and it is 13,542 rows

Both angles are compared against the dump now, on every unit-frame **where
the two sides agree on the position** — the gate matters, because a unit that
walked somewhere else points somewhere else as a consequence, and counting
that would measure the position gap twice. run20's opening is 72 comparisons
and **zero** disagreements. run10's 1,772 frames are 13,542 comparisons and
5,435 disagreements, and none of the residue is the step's: it is
`Unit::set_angle`'s other seventeen callers, which this simulation does not
make. Unit `0/2` alone is 2,680 of it, and its story is legible in one screen
of trace — it walks to `(4440, 28680)` on frame 432 with both sides agreeing
on position, path and both angles, and on 433 the original turns it to face
what it is about to gather.

The scout that opened the item is **two rows in 1,772 frames**, both the frame
after an arrival, and frames 57 to 91 are exact on position and on both
angles.

run13's frame 102 went `22 / 6` to **`6 / 6`**, closing the row the item was
booked on. run6's totals fell from 1,793/2,043 to 1,516/1,911, and the split
moved: the farmers' share went 612/441 to 304/317, because they now walk the
original's frames.

### Five deliberate breakages, all red

The instant-turn flag forced off (run13's 102 → `27 / 8`); `last_speed` set
back to the 11/8 step (run10's comparison *count* falls, and the unit test on
the fixpoint fails outright); the initial angle put back to north (run20's
zero becomes 64); the heading compared against the facing; and the step's
`set_angle` given the facing instead of the heading.

### What this session earned

*A field that two blind readings agreed on is not thereby settled.* The 11/8
survived a first reading, an adversarial second reading and an adjudication —
and it survived them because all three were reading the same arithmetic and
none re-read the `if` above it. What found it was the first frame of a
differential check on a field the dump had been printing all along, and that
is now twice in two sessions: `mylos` last time, `angle` this time. The
capture is cheaper than the reader, and it disagrees.

## 2026-08-27 — the first steering session: the number nobody had written down

Run on Fable, in the main thread, at the user's request after two days of
`/clear` and "go": *are we still aligned, or did we go down a rabbit hole?*
The answer took the morning, and it was both.

### What checked out

Thirty commits and twenty journal entries over 2026-08-26 and -27, all Opus
by their trailers, `+14,645 / −719` over 63 files. The tree was green with
the install wired in (13 + 121 + 589 + 3 tests, clippy and fmt clean), the
dump-backed checks were really reading the captures (run10's takes 28 s),
nothing from the install was tracked, and the numbers the queue quoted were
live assertions. The last commit's central claim — `Guy::move` writes guy 0
straight onto the unit, `guy_num == 0 || (track_dx == 0 && track_dy == 0)`,
the 11/8 in the `else` — was re-derived here from the decompile and the
type record (`+0xa2 guy_num`, `+0x54/+0x58 track_dx/dy`) and held; so did
`Unit::init`'s `0x55555555` at `+0x50` and `+0x58`.

### What had drifted

The loop had changed shape without a decision. 08-20 to 08-25 was one
mechanic per session — a document, a module, a blind audit. 08-26 and -27
were one diff residue per session, items 17 to 34, each closed item opening
one or two more at the front of the queue. Every sub-score improved. The
score phase 3 names had not moved and was stated nowhere:

```
run10, 1,772 frames, RNG seeded from run11's trace:
  ticks before divergence 3, orders 2
  player 0 first diverges at frame 103, player 1 at frame 4
  order lists: 17,751 of 26,433 unit-frames disagree
```

`ticks_before_divergence()` had a definition and no caller in any test.
The queue was 409 lines against its own "about twenty"; its struck entries
had grown paragraphs; three documents were over 150 KB; `SCOUT.md` and
`VISION.md` had no second reading and `MOVEMENT.md`'s was from 08-20; the
ratification ledger owed fifteen audits and nothing scheduled the pass.
The fuzzer, by its own same-day audit, had one real finding (map variation)
and none from its cheat generator, at a cost of two sessions.

None of that is a rabbit hole in quality. It is a long middle with no
stopping rule and no number, which is what a long middle becomes.

### What landed

- **The headline is pinned.** `run10_s_opening_…` asserts ticks 3, orders
  2, player 0 @ 103, player 1 @ 4 as a floor, with a dated history line.
  Made to fail first at `9999`, which is how the real numbers were read.
- **The queue deletes.** `docs/QUEUE.md` went 409 → 134 lines: the headline
  first, open items only (25, 35–37, 23 kept; 38–42 new), no struck lines.
  `crates/sim/src/docs_guard.rs` fails the build on a strike, on 180 lines,
  on a 32-line handoff, on a finding in `CLAUDE.md`, and on any of the eight
  documents over 60 KB growing — it failed four of five on the old file
  before the rewrite, as a guard should.
- **The rules.** `CLAUDE.md`: phase 3's finish line; an item is booked with
  the score it moves and spawns to the back; specification and story are
  different documents; blind readings are for reading-only claims; Fable
  never reads and steers instead. `docs/DECISIONS.md` 24.
- **The `sin_table` marker, closed from the listing.** The decompiler was
  right about `sin_table@00a46a00`; the mirror is `sinx`/`cosx`'s and the
  compiler inlined it at 63 of 65 call sites — which Ghidra hid behind
  `sin_table(unaff_EDI, unaff_ESI)`. The two unfolded sites are one loop in
  `MapGrass::make_continents`, so the odd branch is map generation's alone.
  No code change; `docs/MOVEMENT.md`, "The mirror is the original's".

### Index of the items this session deleted from the queue

Items 0–16 are told in "Lifted from the queue" above. The rest, by number,
under the dated entries of 2026-08-25 to -27: 17 the slot table · 18 run29's
`UNITS=3` half · 19 the formation angle's adder (with 22) · 20 the human
group move · 21 the 36-member table · 22 the mirror's predicate · 24 the
frame-0 draw gap (two entries) · 26 the stand/wrap swap (with 27) · 27 the
draw sites · 28 the `do_move` grid draw (with 25) · 29 `go_around_building`
· 30 `toff` (with 31) · 31 the group's own path (two entries) · 32 the
pathfinder's first step · 33 the fog moves · 34 the body never chased.

### What this session earned

*The number has to be written down, or the loop optimises what is.* Twenty
sessions improved every score they could see and none they could not; the
capture that would have shown the whole standing still was on disk the
whole time. And the cheapest steering instrument is a floor assertion: it
costs one line, it cannot be forgotten, and it turns "are we converging"
from a conversation into a test.

## 2026-08-27 — item 25: the tree in the middle of the forest (Opus 5)

**The headline moved for the first time since it was pinned: ticks before
divergence 3 → 99, orders 2 → 102, player 1's first divergence frame 4 →
100.** The floor in `run10_s_opening_…` carries the new numbers and its
second history line.

### The row

The opener named it exactly: player 1's unit `1/2` holds a second order at
the original's frame 3 that the simulation does not
(`Length { ours: 1, theirs: 2 }`, then `Kind { ours: 7, theirs: 1 }` — a
`MOVEORDER` pushed in front of the `GATHERORDER`). Two frames later the
simulation pushes one too, so the shape was a lag, and lags are usually a
clock. This one was not.

Reading the two sides frame by frame is what said so. The original's
`GATHERORDER` at frame 2 carries `tx 213 ty 92`; the simulation's carries
`(214, 93)`. Both are in the woodcutter's mining list, both are three tiles
from the camp, and the scoring rule in `docs/ORDERS.md` §6.4 —
`max(3, vector_dist) × dist_mod + (i >> 2)` — genuinely prefers the
simulation's: `(214, 93)` sits at index 5 and scores 13, `(213, 92)` at index
10 and scores 14. The arithmetic was right. **The predicate in front of it
was missing.**

`005f0575` scores a candidate only if it still carries `mask & 0x4000` *and*
`WorldData::has_gather_access` holds for it; the implementation scored every
tile in the list. `(214, 93)`'s four orthogonal neighbours — `(213, 93)`,
`(215, 93)`, `(214, 92)`, `(214, 94)` — are all in the same mining list, so
it is a tree in the middle of a forest with nowhere to stand, and the
original never considers it. `(213, 92)` has `(213, 91)` clear, so it does.
With the filter the two sides pick the same tree and `1/2` tracks the
original's position from frame 4 to frame 567.

That is the recurring lesson again, in the form `docs/audit/README.md`
already states it: *the arithmetic is doubly confirmed almost everywhere and
the predicates are where the errors are.* The pseudocode in §6.4 has carried
this filter since the first reading. Nothing but a diff was ever going to
notice that the code did not.

### The other half: run6 was being diffed against the wrong map

Landing the filter broke `the_original_s_own_run_is_still_matched_frame_for_
frame`, and the reason was worth more than the fix. run6's `WORLD` block is
`BUILDS=7`'s — seventeen scalars, no cells, no tile masks — so the harness
had been standing up a flat, region-less, **treeless** world and measuring
run6 against it for three weeks. On such a world `has_gather_access` is false
everywhere, and the new filter correctly refuses every tile.

The tell had been on disk the whole time and nobody looked: **run6 and run10
are the same game**, and their diffs disagreed. run6 said player 1 first
diverged at frame 2; run10 said frame 4. run6 said its farmers held to 213;
run10 said 103.

`borrow_from_siblings` now takes the whole `WORLD` block from a sibling whose
seventeen scalars match ours field for field — the map seed, the extent, the
generator's eight totals, the territory limits. Two captures of one game now
report the same headline and the same per-unit breakdown, which is the check
on the borrow. Some of run6's ceilings went *up* as a result (the farmers'
304/317 → 626/420); that is a re-base, not a regression, and run10 had always
read them the higher way.

### The widening

`compare_orders` compared an order's kind, action bit, flags and target and
nothing else, so a `GATHERORDER` whose `tx`/`ty` disagreed was invisible until
it produced a different *walk* two frames later. The gather order's whole row
— `tx`, `ty`, `wait`, `goto_build`, `been_there`, `dist_mod` — is now diffed
every frame (`OrderMismatch::Gather`); `tx`, `ty`, `dist_mod` and
`non_flat_gather` were not even parsed. It costs 1,221 rows on run6 and 6,330
on run10, none of them before the score, and all of run6's are player 1's
`1/1` — the citizen the original turns into a builder and this simulation
keeps at the woodcutter.

### Numbers

| | before | after |
|---|---|---|
| headline (run10) | ticks 3, orders 2, p0 @ 103, p1 @ 4 | **ticks 99, orders 102, p0 @ 103, p1 @ 100** |
| run6 headline | ticks 1, orders 0, p0 @ 213, p1 @ 2 | ticks 99, orders 102, p0 @ 103, p1 @ 100 |
| run10 angle rows compared | 13,542 | 15,336 |
| run10 `mylos` | 26,433 rows, 1 disagreement | unmoved |
| run6 order/path rows | 1,516 / 1,911 | 2,583 / 1,674 |

`1/2`'s position now agrees to frame 567; `0/1` and `0/2` agree for the whole
of run6.

### Paperwork

`docs/ORDERS.md` §6.4 states the filter as a rule and its 2026-08-21 status
block moved here (below); `docs/DATALAYER.md` has the `WORLD` borrow and a
corrected run6 score; `docs_guard::OVER` lowers `ORDERS.md` to 191,335.

### Lifted from `docs/ORDERS.md`, the 2026-08-21 status block

**Status (2026-08-21).** First reading, implementation, blind second reading
and **all seven adjudications** are landed —
`docs/audit/2026-08-21-orders.md`. What is left of the queue item is the
harness work §13's last two bullets name (the start-of-game in `build_sim`
and reading the `UNITS=3` order blocks back), not the reading.

- **The blind side, done.** Seven readers on Opus 5, split the same way as
  the first reading, each given only the entry points and the traps: 355
  numbered claims across R1 the order system (38), R2 the move order (42),
  R3 build/repair/garrison (57), R4 gather (48), R5 the start of a game
  (56), R6 the combat and group orders (64), R7 the spatial queries and the
  log (50). The reports are **not in the repo** (entry 7) — they are at
  `~/ghidra-projects/reading/orders-2026-08-21/`, with a README there
  giving the state and how to finish.
- **The adjudication, all seven**, one per sub-area, each taking every
  disagreement back to the decompiled function, the listing or the PE:
  R1 (A 11 · B 11 · both 15 · neither 2), R2 (A 4 · B 8 · both 3 ·
  neither 1 · open 2), R3 (**A 0 · B 21** · both 39 · neither 4 · open 2),
  R4 (A 5 · B 13 · both 29 · neither 1), R5 (A 0 · B 9 · both 26 ·
  neither 2), R6 (A 6 · B 14 · both 24 · neither 8 · open 1),
  R7 (A 6 · B 13 · both 29 · neither 0 · open 2). Ten corrections landed in
  `crates/sim`, every one re-verified in the listing before it was applied;
  the document corrections are marked inline throughout, each naming the
  sub-area that found it.
- **The three first-reader disagreements are all closed, none needing a
  behavioural check.** `UnitOrder::flags & 4` is the action bit, settled by
  `Group::set_up_insert@0070e520:25` — a reader neither reading had cited
  (R1). The `OrderIndex` values are in the PDB (R1). `do_gather`'s approach
  radius is `min(x_size, y_size) × 0x60 + 0x30`, settled in the listing at
  `0x5ef756` **twice over**, by R4 and R7 independently.
- **What the second reading cost the first.** Two claims the blind side
  raised against this document held: `add_repair_order` really has no
  `QUEUE_FIRST` branch (R3), so §3.1's "one shape, 21 functions" has
  exceptions. One did not: `do_attack` owning no reload logic is **not** a
  disagreement — §7.2 already put the gate in `fight`, and the
  implementation was right where the prose was wrong (R6).

## 2026-08-27 — item 43: where a trained unit comes out (Opus 5)

**The headline moved twice in one session: ticks before divergence 99 → 102,
and player 1's first divergence 100 → 103.** Both players now first part on
the same frame, 103, and that frame is one mechanic — the farm re-target —
rather than five units going wrong in five ways.

### The row

Item 25 left player 1 first diverging at frame 100, on unit `1/6`. The dump
made it easy: `1/6` is created on frame 100 at `(42360, 17208)`, and the
simulation created it on the same frame at `(42336, 16224)` — London's centre
tile, a thousand units north. Not a movement problem, a **birth** problem.

`Build::train@0062f9b0` creates a trained unit at the building's own position,
calls `Unit::go_inside`, and then `Unit::come_out`. The simulation did the
first of those three and stopped: `Handover::Trained` put the unit at
`bd.pos` and left it there. `come_out` existed, was correct about the FIFO
and the cadence, and was an admitted stub about the spot — "the exit ring
toward the building's facing at the minimum distance", which is the `+x` axis.

The ring itself was already right, and the listing at `618411` confirms it:
`(x_size + y_size) × 0x30 + UNIT_TRAIN_DISTANCE` out to
`… + UNIT_TRAIN_MAX_DISTANCE`, with the inner radius handed to the search
only while the building is alive (`618437`, `flags & 1`). What was missing is
that the original then calls `find_nearby_spot` on it — the same sweep the
gather order uses — and takes the first free quarter-tile centre.

### The bearing is diff-backed, and says so

The angle argument is where the decompiler gives out. It aliases the stack
slot: `local_a18` is written from the container's `angle` on the arm where
the container is a *unit*, and on the building arm it is never written at all,
so what reaches `find_nearby_spot` cannot be read off the C. The listing
(`0x6184cc` pushes `[esp+0x2c]`) says the same thing — that slot is written
only on the sibling arm.

So it was settled by the dump instead. **Due south at the inner radius
predicts `(42360, 17208)` exactly**, and every citizen run10's AI trains —
frames 100, 206, 320, 1297, 1505 — appears there, as do the fuzzed map's two.
`docs/CITIES.md` §6.5's earlier reading says a *set gather point* turns the
exit toward it, so this is the no-rally-point default; the simulation does not
model gather points, and the document says which capture would separate the
two.

### The bug underneath

Wiring the handover through `go_inside` crashed on the first tick, indexing
`nation[8]`. `garrison.rs` was reading `combat.captain` — which is an **object
number**, the thing `damage_o` is compared against — as a simulation index,
and `squad_of` matched on it without the owner. So a lone unit's "squad" was
every other player's `o`-th unit, and the first trained citizen took a herd of
Gaia's animals into London with it. The two helpers are owner-aware now. The
bug was latent in `do_gather`'s `go_inside` the whole time; only a unit whose
`o` collided across owners could see it.

### Numbers

| | before | after |
|---|---|---|
| headline (run10) | ticks 99, orders 102, p0 @ 103, p1 @ 100 | **ticks 102, orders 102, both @ 103** |
| `1/6` first divergence | 100 | 103 |
| `1/5` / `1/7` / `1/8` | 219 / 206 / 320 | 224 / 209 / 323 |
| run6 order/path rows | 2,583 / 1,674 | 2,591 / 1,613 |
| run10 angle rows compared | 15,336 | 15,010 |

Every unit's **first** divergence held or improved. The angle count fell
because the AI's citizens are now alive and walking through the untraced
stretch instead of standing on their city, so their later frames are their
own — the same trade item 25's own note describes, and the first-divergence
list is what says which way it went.

### Paperwork

`docs/CITIES.md` §11 states what is modelled and what is not, and its
2026-08-20 second-reading section moved here (below) to pay for the room;
`docs_guard::OVER` lowers `CITIES.md` to 106,858.

### Lifted from `docs/CITIES.md`, the 2026-08-20 second reading

Five blind readers re-derived the five sub-areas from the same export without
this document, the implementation or the first reports; the adjudication is
`docs/audit/2026-08-20-cities.md`. **Doubly confirmed**, branch by branch: the
`BlockIndex` verdicts and the site-over-tile rule, the foothold, the spacing
lists and their `≤`, the must-belong-to-a-city and one-per-city rules, the
city limit; the harmonic builders, the `do_construct` argument (both readers
went to the disassembly), the site's `>> 5` hit-point growth and the wonder's
half, the refund's float and its `job_counter_2` numerator, the repair period
and its price; the member chain, membership by nearest covering city with the
`+100` push, the automatic level-up on `CITY_BUILDINGS + 1` exact type ids
with the city counting itself, the level's consumers; the garrison chain and
its FIFO, `num_inside`'s two modes, the limit's two techs, the full
`can_garrison` table, every `do_garrison` gate in order, the exit ring, one
squad a frame, the heal's rate and eligibility; capture eligibility at zero,
the radius count with buildings at `7 + garrison`, `capture_strength = mine`
(both readers in the listing), the hand-over at ten hit points, the plunder
formulas and the 4501-frame protection, the assimilation stamp and its three
modifiers, the city heal, the elimination modes. **Overturned and landed
above:** the construction clock is re-baked on `calc_wall_stats`, not frozen
(§3.2); the building's own attrition runs every 32 frames, 16 only under rush
rules before war (§9.5); the city radius mask is the even circle of a rounded
`sqrtf`, not `vector_dist` (§3.6); `find_buildings` stops at a failed
conversion (§5.4); built forts are spaced without a region test (§2.6.3); the
Chinese assimilation `set_type` lays no mask (§8.1); `CityData::pop` is not
the pop value (§1.1). **Settled for the first reading:** type-vtable slot
`+0xfc` is `is_fort` (read out of the PE: `0x472ba0 BuildTypeData::is_fort`),
so the Senate HP exemption is forts, towers and lookouts, not wonders;
`town_hits` is read by nothing but the type's backup/restore. The
implementation was corrected to match (`Sim::wall_stats_dirty`,
`calc_wall_stats`, `ENEMY_TERRITORY_PERIOD = 32`, `city_mask_tiles`), and a
test added for each.

## 2026-08-27 — item 44: what frame 103 actually was (headline 102 → 122)

The opener booked this as the farm re-target: both players first diverged
on run10's frame 103, three of player 0's farmers and player 1's `1/6` at
once, and one mechanic looked like the answer. It was not the farm
re-target — that arithmetic was already right, and the AI's own three
farmers matched to the unit. Frame 103 was **two different things wearing
the same date**, one per player, and neither is a farm.

### Player 1: a building's blocked tiles are art, not rules

`1/6`, the citizen trained at frame 99, walks back to its Woodcutter's Camp
and `do_gather` sends it to `find_nearby_spot(camp, 240, …)`. The original
takes bearing `k = −1` and stands it on `(40680, 17688)`; the sim rejected
that candidate and took `k = −2`, 48 units short on both axes — one step of
the quarter-tile lattice, which is why it read as a rounding bug and was
not one.

The rejected tile is the **camp's own footprint**, and the sim had it
carrying `0x4000`. The original's map does not: the dump's own
`tdata[scan].mask` words show all four of the camp's tiles with the object
field set and the blocked bit clear. `find_nearby_spot` refuses a `0x4000`
tile (`docs/ORDERS.md` §10) and is otherwise happy to put a unit on a
building — so a gatherer stands on its camp, and the whole disagreement was
one bit of loaded data.

Where that bit comes from is the finding, and `docs/CITIES.md` §3.6 had
already said it and been ignored: `mask_me` consults a **per-tile
template**, one byte per footprint tile, and clears the blocked bit
wherever the byte is not 1. The template is not in `buildingrules.xml` at
all. It is `masks.txt` at the install root — 36 named grids — selected by
the `mask=` attribute of the building's graphic in
`Data/building_graphics.xml`. A Woodcutter's Camp is `2x2 gather` and
blocks **nothing**; the Mine beside it is `2x2 solid` and blocks all four;
every `extra space` mask leaves the last row and column free, so a city's
7×7 footprint blocks 6×6. This crate had been blocking every non-flat
footprint whole. `docs/DATALAYER.md` has the two loaders and the file
formats; the "read the loaders, not only the consumers" lesson scores
again, and this time the consumer's own document was right.

The check with teeth: a `WORLD ≥ 6` start dump prints all 57,600 tile
masks, so the map the harness loads is the oracle for the map it then
stamps its buildings onto.
`every_footprint_takes_the_blocked_bits_the_original_s_map_shows` compares
the object and blocked bits of every tile on run10, run20 and run9. Made to
fail first: with the old rule, run10 alone disagrees on 59 tiles, the first
being the last column of London's 7×7.

### Player 0: a sheep, and where a draw falls in a frame

Frame 101's stream is, in order: the AI's three farmers (two `% 4` each),
**a sheep arriving**, the human's three farmers, the human scout's wrap, the
seven farm draws — 21 draws (`docs/SYNC.md` §4.1). The sim spent 20. The
missing one is the sheep's, and because it sits *between* the two players'
farmers, its absence did not lose a draw so much as shift six: player 0's
three farmers spent the values the sheep should have had and walked one
tile wrong on each axis.

The sheep wandered at sim-frame 89, on a `% 10` the harness cannot reach —
run10 has no traced word before 94, and frame 94 alone costs 472 draws the
sim does not model. So there is no version of the animal model that puts
that arrival on frame 101. The fix is a harness correction and is labelled
one: `Sim::reseat_animal` puts gaia's animals back on every traced frame's
dumped position and goal, beside the clocks and the word that are already
installed there, and **reports the drift** rather than hiding it (run10's
is under a tile, on two animals). Frame 101 went 20/21 → 21/21.

A second, smaller correction fell out on the way: a walking clock on a unit
the sim has standing used to be skipped wholesale, and one of those cases
is an *arrival* — the dump's own order list is empty and the figure still
holds the walk it was playing, which is exactly the state `Guy::set_anim`
reads as an arrival and draws for. Those are installed now; run10's frame
94 has one.

### The number

| | before | after |
|---|---|---|
| **ticks before divergence** | **102** | **122** |
| orders before divergence | 102 | 122 |
| player 0 first divergence | 103 | 182 |
| player 1 first divergence | 103 | 123 |
| run10 angle rows compared | 15,010 | 15,318 |
| run13 frame 101 draws | 20 / 21 | **21 / 21** |

What is left at 123 is `1/6` again, and it is the pathfinder this time:
`PathLength { ours: 2, theirs: 7 }`. Player 0 holds to 182.

### Lifted from `docs/SYNC.md` §4.2, to pay for the room

**And the unit paying run20's last `+0xe84` was not the one that document
named.** §6 had it as the AI scout, `1/0`, on a waypoint of `(38784,
39552)`; when `go_around_building` landed, the only unit whose march reaches
the blocked branch on run20's frame 1 is `1/1`, and `1/0` never enters it —
its `find_wpath` chain had moved, most likely when `produce_building`'s
three fixes moved the AI's farm. The count was right twice and the owner was
wrong. *A residue's owner is a measurement too, and naming it costs one
print.*

**Run20's frame 1 read 53 against 53 for two days and was wrong in three
places at once** — the jitter drew once where the original draws four times,
the spiral's stride-by-three never engaged, and
`WorldData::buildings_allowed` was not modelled, so a forest cell scored and
drew. The three residues cancelled to one. `docs/AI.md` §2.20 has each; the
general lesson is `docs/SYNC.md` §5.1's, and this is its second scalp.

## 2026-08-27 — item 46: unit collision (headline 122 → 170)

The opener said the frame-123 divergence was the pathfinder, because `1/6`'s
path stack was 2 entries deep where the original's was 7. It was not the
pathfinder. The five extra entries all carried `flags 2`, and `flags 2` is
what `find_upath` stamps on a reconstructed 48-grid waypoint — so the
question was not "why is the plan wrong" but "what asked for a plan at all".
The answer was in the same record, three lines up: `collide 1`,
`collide_frame 122`, `collide_o 3`, `collide_who 1`, `collide_guy 0`.
`1/6` had walked into `1/3`.

`docs/MOVEMENT.md` had this as an open question — "Collision and pushing …
Unread" — and it is now `docs/COLLISION.md` and `crates/sim/src/collide.rs`.

### Two indices, and why neither can be a scan

The mechanic is not a search over units. It is two indices the world keeps.

`CollBlock` is a bitmask of **48-unit cells**, one per world cell, and every
unit sets the Chebyshev disc of radius `BLOCK_RADIUS` around its figure's
cell. `CollCheck::collide_here` reads it. The tempting simplification —
recompute occupancy from the unit list on demand — is wrong for a reason
worth writing down: `CollCheck::move_unit` **clears** the cells a unit
leaves without asking whether anybody else is standing on them. Two
overlapping blocks share bits, and one of them moving punches holes in the
other. For `BLOCK_RADIUS 1` — 220 of the 364 shipped types, and every unit
in every capture — the blocks overlap most of the time, so the two answers
differ constantly. The bug is the behaviour.

The second index is the per-world-cell object chain, `WData::down` threaded
through `ObjectData::down`/`up`, head-inserted by `Object::add_to_world`.
That is how the bitmask's "something is at cell (871, 354)" becomes "that is
`1/3`". This crate chains units only; the original chains buildings and
goodies too, but the walk skips them and dropping them from a linked list
does not reorder the rest. What that costs is a diff — the dump prints
`down`/`down_who` and they cannot be compared until buildings join — and it
is booked.

### The corner rule, which the dump adjudicated

With a hit cell in hand, the exemption ladder decides whether this is a
collision or a nudge, and the last rung is geometric:
`will_be_corner(me, hit)` and `is_corner(other, hit)` each return `1, 3, 5,
7` for NW, NE, SE, SW when the hit is exactly a diagonal corner of the
block, and the pair passes **only if they differ by four** — opposite
diagonals, two units touching at one corner from opposite sides.

For run10's pair: `1/6` proposes cell `(872, 355)`, the hit is `(871, 354)`,
`will_be_corner` is 1 and `1/3`'s `is_corner` is 0 because the hit is on its
edge and not its corner. `|1 − 0| ≠ 4`, so it is hard. The dump's five
`collide*` fields and its `coll_x 41880, coll_y 17065` are what say the
reading is right, and `coll_x/coll_y` corrected `docs/ORDERS.md` on the way
past: they are the **refused step point**, not the blocker's position.

### `move_guys`, and the frame that was nearly lost

`resolve_unit_collision` snaps the blocked unit onto its own 48-cell centre
and re-plans. Implemented that far, frame 123 matched exactly — the
position, all seven path entries, every field — and frame 124 was one step
behind for the rest of the walk. The unit had turned 71° to face its new
waypoint and spent the frame doing it; the original turned and walked.

The difference is `Unit::set_new_location`'s third argument. `move_step`
passes 0 and leaves the body to chase the unit; `resolve_unit_collision`
passes **1**, which teleports the body onto the new point. The follow phase
then reads `last_speed 0`, and a foot type standing still turns instantly
(`docs/MOVEMENT.md`, "The body step"). One boolean, threaded through the new
`Sim::set_new_location`, and `1/6` went from parting at 124 to parting at
208.

That is the third time in three sessions that the residue was in a
*predicate or a flag* rather than in arithmetic, which is what the audit
README has been saying.

### The check with teeth

`UnitData::log_data` writes `collide`, `collide_frame`, `collide_o`,
`collide_who`, `collide_guy` and `safe` at **every detail level**, so the
whole block is comparable on every capture the harness reads. Widened and
pinned: **40,600 field-frames on run10, 285 disagreements, none before frame
201** — and 277 of those are one sticky byte, because `collide_guy` is
written to 0 by a hard collision and never cleared, so a single extra
collision on `1/3` reads 0 against −1 for the rest of the run. The other
eight are two units and two collisions, both past the score.
`coll_x`/`coll_y` are compared too, as a scoring order mismatch, and adding
them did not move the score.

`crates/sim/src/collide.rs` carries four tests written to fail first,
including the one that pins the un-refcounted clear.

### The number

| | before | after |
|---|---|---|
| **ticks before divergence** | **122** | **170** |
| orders before divergence | 122 | 166 |
| player 0 first divergence | 182 | 182 |
| player 1 first divergence | 123 | **171** |
| `1/6` first divergence | 123 | 208 |
| run10 angle rows compared | 15,318 | 16,206 |
| collision-block field-frames | — | 40,600 (285 bad) |

What pins player 1 now is `1/1` at 167, and it is not collision: it is the
gather order's `dist_mod`, on the citizen the original turns into a builder
and this simulation keeps at the woodcutter. Player 0 is unmoved at 182.

run6 re-based with it: the farmers' share went 626/420 to 675/346 — six
citizens clustered round one farm collide constantly, so their walks now go
round each other — while everyone else's fell on both halves.

## 2026-08-27 — item 47: the builder that does not keep what it built (headline 170 → 181, Opus 5)

`run10` frame 167. The AI's citizen `1/1` finishes a farm; the original's
gather order carries `dist_mod 4` and this simulation's carries 0. Everything
else on the row agrees — kind, flags, target — which is what made it look
like an arithmetic slip in one field.

It was not a field. `dist_mod` is written once, by `add_gather_order`, and it
is 4 for a **non-flat** gather building and 0 for a flat one. The two sides
were gathering at different buildings and the diff could not say so: the
target comparison is skipped when either side names a building the harness
cannot map to a logged `o`, and both of these were built during the game. The
original's `1/1` was on the Woodcutter's Camp `2001`; this one was on the
farm it had just raised.

### The predicate, and why the run that "confirmed" it did not

`Unit::do_build`'s step 6 hands the finished site to its builder only when

> `is(OILPLATFORM)` **or** (`unit_masks & 0x40000` clear **and**
> `get_worker_stance() ∈ {0, 1}`)

and step 2 — the arm for a site that finished before the builder arrived —
is the same test negated. `Unit::do_repair` carries it a third time. This
crate had the stance half of all three and none of the AI half, so every AI
builder adopted its own site.

The reason nothing caught it for a month is in `docs/ORDERS.md` §5.2's own
worked example, which has now been rewritten. The logged run it was written
from *did* leave the AI builder gathering at the farm it had built — but by
`build_done` → `find_gather_spot`, which takes the nearest gather building
with room, and that was it. A predicate and its negation produce the same
observation whenever the fresh search lands back on the site, which is most
of the time. Frame 167 is one of the times it does not.

`build_done`'s AI arm is now its own: `find_build_spot` → `find_repair_spot`
→ `find_gather_spot`, with **no stance gate**, gated on the lobby's
`starting_resources != 8`, and **without the tail** that hands a human
builder its site. The two object searches stay seams.

With the term in, `1/1` walks to the original's camp, takes the original's
tile `(212, 93)`, and carries `dist_mod 4`. The first frame on which any
gather tile disagrees went from 169 to **430**.

### The measure that was being paid off

Pinned beside the headline was "1/9 trains on the original's frame, and only
the last citizen is missing — 268 unlinked unit-frames". It was not true, and
the measure could not see it: `unlinked` counts units the *original* has that
this simulation does not, so a unit that arrives **too early** is free. The
AI's ninth citizen had been standing here from frame **897** against the
original's 1297, and from 1297 the link existed and the earlier four hundred
frames cost nothing.

`FrameResult::extra_units` is the mirror, added here and made to fail on
purpose first: on the pre-fix code it reports those 400 at once. With the fix
the AI has one farmer fewer and never reaches its ninth citizen inside 1,772
frames, so the gap changed sign rather than closing — 268 + 400 hidden
becomes 744 + 0 seen. **The AI's long-run economy is what this measure now
names**, and it is booked as its own item rather than smuggled into this one.

### Numbers

| | before | after |
|---|---|---|
| **ticks before divergence** | **170** | **181** |
| orders before divergence | 166 | **168** |
| player 0 first divergence | 182 | 182 |
| player 1 first divergence | 171 | **203** |
| first gather-tile disagreement | 169 | **430** |
| roster unit-frames (missing + extra) | 268 + 400 | 744 + 0 |
| run10 angle rows compared | 16,206 | 17,018 (6,926 bad) |
| collision-block field-frames | 40,600 (285 bad) | 42,630 (400 bad) |
| `mylos` unit-frames | 26,433 | 25,957 |

The two totals that fell are the same citizen: `1/9` is 476 of the `mylos`
rows and its absence is the whole of the roster's move. The two that rose are
`1/3` and `1/6` holding the original's positions for longer — 812 more angle
rows, of which 752 agree, and 115 more frames of `1/3` reading its one stale
`collide_guy`, which is 392 of the 400.

### Paperwork

`docs/ORDERS.md` §5.2 gains the predicate and the trap, §13 the AI arm; §6.5
gave up the superseded `FARM_GROWS` narrative to pay for the room (the story
is in this file's 2026-08-24 entry) and the pin came down 191,335 → 191,190.
`crates/sim/src/cities_tests.rs` carries the split as a test, written to fail
first.

## 2026-08-28 — item 50: the bird, and the ledger that found it (headline 181 → 185, Opus 5)

The opener said the order score's first divergence was one frame to read:
at frame 169 `1/1`'s camp-return draw is ours 581 against theirs 460, the
same `400 + rnd % 200` on a different word, so read frame 169. It is not
one frame. It is sixty-five.

### The measurement that turned it into a ledger

`1/1` picks its tile in the *same* frame on both sides — the dump has
`tx 212, ty 93, wait 460` at `FRAME 169`, and the simulation writes its own
`wait` on the tick that produces that block. So the tile choice is right and
the **word** is wrong, and the word is wrong because the last one installed
from a dump is frame 103's: everything from 104 to 168 runs on the
simulation's own stream. Walking the LCG forward from the frame-104 word
says how far: our draw is the 460th, and the nearest word yielding 60 is
**33 further on**.

Thirty-three is not a bug in a frame. It is a bill.

Paying it needed the original's per-frame draws for a stretch no `DUMP_ALL`
window covers — and **run14 has them**. It is run10's own lobby and seed
with `tools/trace` attached, 284 frames, every draw named by its site. The
harness already had both halves of the comparison (`trace::SITES`,
`diff::mark_sites`) and used them on frame 0 alone; `Built::frame_sites`
now records the simulation's own sequence on **every** frame, before the
word is installed, and `run14_s_frames_match_the_trace_draw_for_draw`
compares all 284. The first run: **179 of 284**, and a list naming what is
missing on each of the other 105.

The thirty-three, itemised: 27 `Animal::think_bird`, 6 phase-7 wraps of a
guy of owner 9, −4 from a sampling loop that ran two pairs too many twice,
+3 from a market run displaced by the rest, +1 an arrival stand. Every one
of them, except the last, is **the bird**.

### The bird

`Objects::process_all`'s tail samples ten cells every 32 frames and hatches
a bird on a `flags & 0x20` one. The simulation had been drawing the twenty
sampling draws and *recording* the hit — `gaia.bird_spawns` — and creating
nothing. So it missed `Guy::init_real`'s hatching roll, missed
`Animal::think_bird`'s three draws every eighth frame from then on, and
kept sampling ten pairs where the original, one bird up, samples nine.

Run14 settles all three without a capture:

- hatchings at **96, 192, 256** (`Guy::init_real` < `Unit::init` <
  `Animal::init`);
- the sampling at **10, 10, 10, 10, 9, 9, 9, 8, 8** on frames 0…256;
- `think_bird` at **3, 3, … 6, 6, … 9** — three a live bird, every eighth
  frame, from 104.

`think_bird`'s counter is `UnitData::spell_time`, which a bird reuses: it
steps every frame the function runs and again on each think, and the number
it has reached is the **modulus of the landing roll**. That is why the
landing search — thirty rounds, two draws each — cannot fire for the first
ninety frames of flight and fires on no traced frame: `rnd % counter` can
only equal 100 once the counter has passed it.

`docs/SYNC.md` §3.9 has the whole of it. The two things that made it cheap
were `is_air` — the domain is loaded, so a bird need not paint the
occupancy grid, and a documented seam closed itself — and the fact that
nothing reads a bird's patrol point, so the flight can be loose while the
stream is exact.

### What it did not close, and why the order score did not move

`Guy::set_anim`'s bird branch draws once and takes the second walk
animation when `rnd % 100 > 0x31`. Run14 spends it 28 times, **25 of them
as phase-7 wraps**, and a wrap falls when the animation ends — so it needs
the bird's two animation *lengths*. Those are art data, `Art` is read out
of a dump's `GUY` blocks, and **no dump prints owner 9 at all**. The
simulation's bird carries `piece = −1`, never wraps, and is six draws short
between frame 103 and frame 168. `1/1`'s `wait` went 581 → **476** against
460: nearer, and still on the wrong word.

That is the honest shape of it. The tick score moved because positions stop
depending on the stream sooner than orders do; the order score is pinned by
a bird's animation, and the item that unpins it is named with the frames
that check it.

### Numbers

| | before | after |
|---|---|---|
| **ticks before divergence** | **181** | **185** |
| orders before divergence | 168 | 168 |
| player 0 first divergence | 182 | **186** |
| player 1 first divergence | 203 | 203 |
| **frames matching the trace, draw for draw** | **179 / 284** | **184 / 284** |
| `1/1`'s frame-169 `wait` (theirs 460) | 581 | 476 |

### Paperwork

`docs/SYNC.md` gains §3.9 and strikes "Birds after creation"; it paid for
the room by giving up two closed narratives — the frame-0 tail and frame
3's extra draw — whose stories are in this file's 2026-08-24 and
2026-08-26 entries, and the pin came down 70,026 → 67,763.
`docs/COLLISION.md` records the air seam as closed.

## 2026-08-28 — item 52: the bird's wing beat, and the art file that was simulation state (orders 168 → 180, Opus 5)

Item 50 left the bird flying and mute. Its animation wanted two numbers —
how many frames *Bird Soar* and *Bird Flap* run for — and the note said
they were art data no dump carries, which was true and was the wrong place
to stop. The install carries them, in the open, in files anyone can read.

### The chain, and it is three files long

`Data/unit_graphics.xml` has `<UNIT name="WILDBIRD-TYPE0">` with two
children and only two: `CHAR_WALK` is *Bird Soar*, `CHAR_JOG` is *Bird
Flap*. `Data/anim_graphics.xml` turns those names into `art/bird_soar.bha`
and `art/bird_flap.bha`. And the `.bha` is a chunk stream whose root node
carries a key count and a 36-byte key each, the first `f32` of which is
that key's duration in seconds — which `AnimObj::load_hier` accumulates
into a `u16` of milliseconds and `AnimMgr::force_load` converts with

```
frames = round(times · 3 / 200)          # fifteen frames a second
```

**31 and 23.** And a slot the packet does not name — every idle, for a
bird — is `AnimationPacket::get_game_frames`'s own `return 3`.

The reader is `crates/rondata/src/artdata.rs` and it took an afternoon
because the *decompile* said exactly where to look: `force_load` is forty
lines of chunk walking and one line of arithmetic, and `load_hier` names
the key stride. Guessing at the header would have cost a day and been
wrong; three wrong offsets were discarded in five minutes each because the
node's chunk has to end exactly where its keys do, and that is now the
assertion `cargo run -p rondata -- <install>` makes.

### The check that made it safe

A new reader of a binary format is a new way to be confidently wrong, so
it is checked against the oracle that already existed. A `DUMP_ALL` dump
prints `end_time` on every `GUY`, and run12 shows thirteen gaia rows: the
three sheep pieces' `CHAR_DEFAULT` at 90, 109 and 250, and the three fish
pieces' idles at 170, 101 and 116. The file agrees with every one.

That agreement does more than validate the arithmetic. It pins the one
link neither file states — that `-TYPE0`, `-TYPE1`, `-TYPE2` in
`unit_graphics.xml` **are** the variant `(seed + o) % 3` picks — because
`Sheep Idle1` / `Idle3` / `Idle5` land on pieces 60063 / 60064 / 60065 in
that order and no other.

### Four rules, and the bird's whole stream

With the lengths in hand the wing beat fell out of `Guy::set_anim` and
`Guy::inc_time` in one sitting, and three of the four rules were things
this crate had wrong for reasons that had nothing to do with birds:

- **`set_anim` dispatches on the category, and the walk arm opens with the
  category as its answer.** So `set_anim(CHAR_JOG)` is not a request for
  `CHAR_JOG`: it re-resolves from `CHAR_WALK` and throws the coin again.
  That is what lets a bird alternate its two beats — and what makes one
  wrap spend several draws, because a flip keeps `cur_time` (the rescale
  is the identity the second reading found) and 31 still overruns 23. Run14
  spends two coins at frame 127 and **nine** at 243, and the loop bound of
  four in `guy_inc_time` would have swallowed five of them.
- **`Guy::init_real` leaves `end_time` at zero**, so a guy's first
  `inc_time` always wraps. For a unit trained inside a building nobody
  notices; a bird is created on open ground in the middle of
  `Objects::process_all`, so the same frame's clock reaches it. The
  "created this frame" skip here only ever meant *created inside a
  building*, and frame 96's twenty-second draw is the proof.
- **`do_air_physics` asks for `CHAR_WALK` every frame** and draws exactly
  once per bird, at birth, because from the frame after, the gaia early
  return takes it.
- And the bird's `type_index` was **−1**. `spawn_bird` set `kind` and `ty`
  and not the index `set_anim` names by identity, twice. Every rule above
  was inert until that line landed, and the symptom was a bird drawing an
  unlabelled idle roll every frame — which the ledger showed as `unit 9/52`
  where the trace said `Guy::set_anim+0x104b`, at frame 97, seven frames
  after the hatch. A count would have said "seven against seven" and moved
  on.

### The two scores met

The order score went **168 → 180** and the ledger **184 → 192**. The tick
score went 185 → 181, and that is worth stating plainly rather than
burying: it is the same residue, and it moved *forward*.

`0/3`'s order list goes wrong when its gather tile is picked off a stream
that is still short, and that pick moved **169 → 181**. At 169 the wrong
order did not move the unit for another seventeen frames, so `ticks` read
185 while the orders had already parted — a position score above the order
score is an accident, not a gain. Now the position follows the order by one
frame, which is what the two numbers mean when both are honest. Two other
sub-scores moved with the coverage: the angle rows compared went 16,456 →
17,302 and the collision fields 41,225 → 43,340, both because more
unit-frames hold their positions long enough to be seen.

One sub-score fell for the same reason item 50's did: `1/6`'s second tree
is chosen at frame 407 now rather than 413, on a stream that has drifted by
then for reasons of its own. A closer stream at 180 is not a closer stream
at 407.

### The collision tally was cut the wrong way

`1/4` now holds its position 97 frames longer, and every one of those
frames carries the same stale `collide_guy` — the byte a hard collision
writes and no path clears. The assertion that said "at most ten rows that
are not `1/3`'s" went to 105 without anything changing about the mechanic,
because the cut was by *unit*. It is by **field** now: `collide_guy` is the
sticky one, everything else is a real disagreement, and there are eight.

### Numbers

| | before | after |
|---|---|---|
| **orders before divergence** | **168** | **180** |
| ticks before divergence | 185 | 181 |
| player 0 first divergence | 186 | 182 |
| player 1 first divergence | 203 | 203 |
| **frames matching the trace, draw for draw** | **184 / 284** | **192 / 284** |
| angle rows compared | 16,456 | 17,302 |
| collision fields compared | 41,225 | 43,340 |
| first gather-tile disagreement | 413 | 407 |

### Paperwork

`docs/FORMATS.md` gains "The animation file (`.BHa`)" under BH3/BHA — the
container, the arithmetic and both ways it is checked. `docs/ANIM.md` gains
§3.1 and corrects §4's walk arm. `docs/SYNC.md` §3.9 is rewritten around
the wing beat and paid for the room by compressing §4.2's and §5.1's
narrative, whose stories are in this file's 2026-08-26 and 2026-08-27
entries; the pin came down 67,763 → 67,737.

## 2026-08-28 — lifted from `docs/SYNC.md` §6, to pay for §3.10

`docs/SYNC.md` is pinned in `docs_guard::OVER` and may only shrink, so
adding §3.10 meant taking the same weight out. What follows is the original
text of the world-grid item, kept verbatim; the item's *answer* stays in
§6, and only the account of finding it moves here.

> (2026-08-26, found while closing the item above and unread.)
> Run20's unit `1/0` walks a `find_wpath` chain the original logs at
> `(42744, 37368)`, `(41976, 38136)`, `(41976, 38904)`, `(41208, 39672)`,
> … — every one of them `cell*0x300 + 504` on both axes, where the
> simulation's `astar_path` emits the cell **centre**, `cell*0x300 + 0x180`.
> It is `toff`, and `docs/PATHFINDER.md` §7 already had it: a reconstructed
> world node is pushed at `node + toff − 0x180`, and the simulation carried
> that as a **stated seam** (`path.rs`: "the target-is-a-unit offsets
> (`toff`) are zero — move orders here have point goals"). The arithmetic
> closes: `504 − 0x180 = 120`, and the order's own `off_x` is `504` —
> `41976 mod 0x300`, from a destination the simulation computes the same
> way the original does. So the seam was wrong for a *plain* move, not only
> for a unit target: `toff` is the order's `+0x4c/+0x4e`, which on a
> `MoveOrder` is `off_x/off_y` (`docs/ORDERS.md` §4.1). That one number was
> most of run20's remaining path-to disagreements and all of `1/0`'s
> position drift, and it needed no capture: the dump on disk had both
> sides.

The same trade also condensed §4.2's post-mortem prose and two of §6's
closed entries. Nothing that a diff or a test cites was removed; what went
was the account of how each was found, which is this file's job.

## 2026-08-28 — item 53: the residue's names, and the seam that measured worse

Booked on the ledger, which moved **192 → 198 of 284**. The headline did not
move and was not touched.

The queue named two families and there were five. Getting them took one
Python reader over `rontrace-run14.log` — the format is nine lines of
`struct.unpack` (`crates/rondata/src/trace.rs` documents it) — folding the
sync draws of frames 2–284 by `(site, up[0], up[1])` and resolving each
address through the Ghidra export's `INDEX.tsv`. That histogram is the
whole method: **thirty-five distinct chains, and every one either already
had a `SITES` row or was a hole**. It cost minutes and it is repeatable;
before it, the residue was read out of a failure message one frame at a
time.

The five, in the order they were worth:

- **`MathUtilFuncSet::rand_int+0x18`**, eight draws, all frame 1 — the AI's
  opening `rand_int(1, 10)` × 8 inside the script VM. Frame 1 had been
  reading *54 against 54* and failing, because ours said `strategy_all`
  where theirs said `9e18a8`. One row, one frame.
- **`Animal::do_idle+0x83`** and its three step draws at `+0x1a4`,
  `+0x1d4`, `+0x212` — the herd animal's wander. Already implemented in
  `Sim::animal_idle`, already correct, and entirely invisible because the
  four draws were being attributed to `SITE_IDLE_ANIMAL`.
- **`GameAccess::rnd+0x20`**, thirty draws over eight frames — the farmer's
  cell re-pick, also already implemented. This one is the finding worth
  keeping: **`GameAccess::rnd` is frameless**. It is
  `Random::get(0, 0xffff) % ecx` with no `push ebp`, so its `Random::get`
  call returns to `+0x20` for every caller in the executable *and* the
  tracer's `ebp` walk skips both it and `Unit::do_gather`, landing on
  `Unit::do_job+0x67` — which is `do_gather`'s own return address, the one
  that appears a level up in `do_non_flat_gather+0x54b < do_gather+0xea0 <
  do_job+0x67`. The chain that looks wrong is the chain that proves it.
  Reading `do_gather` also turned up a second, unmodelled `rnd` pair on the
  **pasture** branch, behind a `(o·7 + frame + who) % 256 == 0` phase gate;
  no capture reaches it.
- **`Guy::set_anim+0x97a < Guy::move+0x19f`** — the arrival stand, and the
  one caller that reaches `Guy::set_anim` directly rather than through
  `Unit::set_anim+0x56`, so its disambiguator sits at `up[0]`.

The fifth is the one that did not land. **`Unit::move_step+0x823`** is the
stand a blocked unit plays, and `docs/COLLISION.md` §7 had carried it as a
seam with a reason: "this crate does not set the walk animation either, so
setting the idle one would be a lone half of a pair". That reason was
false — `Sim::guys_follow` has set the walk for some time. So the seam was
closed, and it measured **worse**: 43340 → 42755 agreeing unit-frames on
run10 and 198 → 196 traced frames on run14. Reverted, and §7 now carries
the measurement instead of the wrong reason. The row stays in `SITES`, so
run14's frames 122, 184 and 256 read as `SITE_BLOCKED` from the original's
side rather than as a bare `5dac7a` — a name for a thing we deliberately do
not do is worth more than no name, because it is what says the frame is not
a naming fault.

Why it measured worse is the same fact that governs the rest of the
residue. **`PathFinder::calc_road_cost+0x46` is the only site on run14 with
no name**, 657 draws on frames 10, 11 and 171 where the sim draws 6, 6 and
7 — the game planning a caravan road two frames after the start with no
caravan in it. From frame 10 the two streams are 657 draws apart, so every
value-driven label after it can differ while every mechanic is right: the
`Farms::inc_time+0x1ae`/`+0x1de` orderings the queue called "the cheapest
family, same count wrong order" are exactly this, and they are not cheap at
all — they are the road. Our collisions fall on different frames for the
same reason, which is why paying the blocked stand's draws costs rather
than earns. That is item 54, and the note there says to read *who calls
`find_road`* before reproducing an A* expansion draw for draw.

Paperwork: `docs/SYNC.md` gained §3.10 and, being pinned in
`docs_guard::OVER`, paid for it — §4.2's post-mortem prose and two of §6's
closed entries condensed to their standing facts, one "Original text:"
block lifted here verbatim, and the pin lowered 67,737 → 67,581. Nothing a
test or a diff cites was removed; what went was the account of how each was
found, which is this file's job. `docs/ANIM.md` §4's caller table gained the
two `set_anim` rows it was missing.

## 2026-08-28 — the road nobody asked for (item 54)

The queue's guess was that the road at frames 10, 11 and 171 was planned
"for a reason the sim never has", and that skipping it would be cheaper
than reproducing it. Both halves were wrong.

**The caller is `Build::process`.** A building carries
`WallData::build_masks & 0x100`, "my roads want replanning", and fires it on
the frame where `(frame + o) % 16 == 0` — which is why run14's road frames
are 10, 11 and 171 and not any others. The flag is set by
`City::regen_roads`, which walks a city's member chain and marks every
building in it, and whose callers are `Build::activate` and
`Build::remove_from_city`. So the road is a *building's* road to its city
centre, `astar_caravan_road` is just the one road search the engine has,
and the caravan argument is `−1` — which is the sign that turns its wheel
from eight directions to four.

Every step of that was settled by grepping the dump rather than by reading.
`build_masks` is printed for every building at every detail level, and its
lifetime on run14 is the rule exactly: `2006` (a Market) fires at 10,
`2005` (both Libraries) at 11, the four farms and the woodcutter at 12 to
15 and draw nothing, and at 167 the AI's first farm *activates* and flags
its whole city again, so `1/2005` searches at 171. One `grep` and a
sixteen-line script; the reading only had to explain what the grep had
already shown. The queue's own rule — grep the dump before booking a
reading — paid twice over here, because it also named the three buildings,
which is what made the rest falsifiable.

**Everything but the count is established.** The nine `road_*` weights came
out of the PE at the two `movaps` sources `PathFinder::init` names;
`calc_road_cost` was read off the listing term for term after the
decompiler's rendering of it proved trustworthy but unhelpful about ±1s;
`valid_roadcoord`'s guard, its city-only footprint exemption, the wheel's
parity gate, the two different heuristics (the root's divides by `0xc0`,
every other node's by `0x180` — a real inconsistency, and harmless), the
budget and the endpoints all likewise. `docs/ROADS.md` is the write-up.

**The ring is diff-backed, and by an oracle that was sitting on disk.**
run13's `DUMP_ALL` window starts at sim-frame 95, so its tile masks are the
world *after* frames 0, 10 and 11 laid their roads; run10's `WORLD=6` block
is the world before. They are identical — all 57,600 masks, not one tile
moved in ninety-five frames, because everything a ring or a road reaches is
already a road or is blocked. That is now
`run14_s_road_rings_change_no_tile_the_original_does_not`, and it was made
to fail on purpose first: widening the city's ring by the one column the
*third* `place_roads` arm uses lays twenty-six roads the original does not.
The same dump settled the ring's shape directly — a picture of the tiles
round p0's city centre shows a full eight-by-eight border, which is only
possible because a city stands on seven tiles each way and blocks the inner
five.

**The count is six per cent short, and that is the whole of what is left.**
208, 222 and 178 nodes costed against 220, 248 and 189. The world was
checked (the sim's masks at frame 10 differ from the dump's in sixteen
tiles, all of them the `PLACED` bit on a farm site two hundred tiles from
the search), the stream was checked (frames 0 to 9 match draw for draw, so
the word at the head of frame 10 is the original's), the cost function and
the validity predicate were checked against the listing, and the wheel, the
preference, the endpoints and the search's direction were each varied and
each made it worse. `docs/ROADS.md` §7 lists all of it, and the two inputs
that have still never been diffed against anything: the loader's per-tile
heights, and `vector_dist`'s rounding at the magnitudes the heuristic uses.

So `Sim::plan_roads` is **false**. This is the first time a finished
mechanic has landed switched off, and the reason is arithmetic rather than
taste: the draws land in the middle of a frame, so a count that is close
puts every later draw in that frame on the wrong word, and run14's ledger
falls 198 → 153. Off — with the rings still laid and the schedule still
kept — it is unchanged at 198. A mechanic that is *nearly* right on a
shared stream is worth less than none, and the switch is how that is said
in code rather than in prose.

Paperwork: `docs/SYNC.md`'s §3.10 and §6 entries struck through and pointed
at `docs/ROADS.md`; the pin came down 67,581 → 67,140, paid for by
condensing §3.10's account of the `GameAccess::rnd` chain to the two rules
a reader needs.

## 2026-08-28 — what the headline actually is (item 55, Opus 5)

The session opened on item 38, the long traced capture, and did not run
it. `screencapture` answered *could not create image from display* and an
`osascript` to System Events timed out after a minute, and that was read as
"this machine has no display". **It was wrong.** Both are what revoked
screen-recording and automation permissions look like, a Claude Code update
had reset them, and the moment the user re-approved both, `screencapture`
wrote a 983 KB PNG and System Events answered at once.

The cost was the whole session's booked work, and the lesson is a process
one rather than a technical one: a permission-shaped failure — a capture
that refuses, an Apple Event that times out, a tool that hangs — **ends the
turn with a question**. It does not get diagnosed into a fact about the
hardware and worked around, because the user cannot unblock what they are
not told about, and an hour of good side work is not worth an hour of the
work that was asked for. The user's words: *stop the turn so I can know you
are stuck.*

What follows is what the session did instead. It is worth keeping, and it
is also unadjudicated: the same judgement that concluded "no display" wrote
it.

What was left was the tables, and they had more in them than expected.

**The headline was chased to its cause, and the cause is item 55.** Phase
3's score is `ticks before divergence` and it has read 181 since item 52,
pinned by player 0's citizen `0/3` at frame 182. The dump says what that
citizen is doing: it walks onto its farm on frame 110, sows one cell for a
hundred frames, re-picks a tile on 212 and walks off on 213. Here it does
the same thing thirty-one frames early — re-picks on **181**, moves on 182
— and `do_farm`'s "a new tile" branch is two draws, so the order list
parts and the position follows.

The farm's clock is not wrong. `Farms::inc_time` adds `0.005f` a frame and
the farmer's `grow` adds another, and 201 adds reach `1.0f` — that is
pinned in `farms.rs` and it is exactly the original's hundred frames. What
is wrong is *which cell was already growing when the farmer arrived*: the
original's is empty and takes the full 101 frames; this simulation's was
sprouted on frame 49 and is 61 adds old on 110, so it ripens on 180.

The sprout is a coin, `rand % 1000 < chance`, one a frame a farm — and by
frame 94 this simulation's stream is **466 draws behind** the original's.
`--diff` had been printing that all along, in a line nobody had read as a
number: *frame 94: ours 6 draws, the original's 472*. The 466 are the road
searches item 54 landed switched off. So the chain is: no road draws → the
farms sprout off the wrong words → a cell is already half grown when the
farmer reaches it → the citizen leaves thirty-one frames early → the
headline. **Item 55 is not a residue chase; it is the headline item**, and
the queue now says so.

**The deficit is a constant, not a percentage.** `docs/ROADS.md` §7 had it
as "about six per cent", which is what 208 against 220 looks like on one
frame. Frame 11 is *two* searches, 52 + 170 against 248: six per cent of
222 is 13, and the actual gap is 26. Twelve, thirteen and thirteen, eleven
— **each search is short by about a dozen nodes whatever its size**, which
is three expansions, not a bias that grows with the path. That changes what
to look for: something that happens once per search.

**A widening, and it ruled the map out.** The road search reads the world
and nothing else, so "is our map the original's" is the first question the
count has to answer — and it had only ever been asked of the start dump the
map was *loaded from*, which is circular. Run13's `DUMP_ALL` writes a whole
`WORLD` block at sim-frame 95, ninety-five frames of the same game later.
`run13_s_world_at_frame_95_is_the_original_s_cell_for_cell` compares it:
3,600 cell owners, 3,600 cell flag words, 57,600 tile masks. Everything
agrees but one cell's `BUILDING` bit, which nothing in this simulation
sets — a real gap, found by the widening, and item 56. Made to fail twice
before landing: once by comparing the world before the ninety-five frames
(the tile masks catch it) and once by moving a single cell's owner.

Four more things were ruled out by measurement rather than by reading. The
territory term: forcing every cell friendly changes no count, so every cell
the three searches touch is already the searcher's; forcing them unowned
takes frame 10 from 208 to 405, so the term is live and the map's answer is
the one in use. The fog: forcing `was_seen` true changes nothing. The
heights' orientation and scale: every object record carries `z_internal`,
and on flat ground it equals `World::tile_z` at the object's tile exactly,
while the transpose is nonsense at every one of them — and flattening the
heights takes frame 10 to 103, so the climb term is what shapes this search
and it is reading the right numbers. And the containers, read this time
from the listing: the open list is a plain BST keyed by `value` whose equal
keys go left and whose leftmost is popped, which is LIFO among equals and
is what this crate does; the refs and closed lists are red-black *maps*
keyed by metric that overwrite on an equal key. Nothing to model there.

The decompiler was wrong about one thing worth recording: it prints the
goal's `x` and `y` crossed in `astar_caravan_road`'s locals, so the
heuristic reads as `vector_dist(node.x − goal.y, node.y − goal.x)`. The
listing settles it at the sane pairing — `[ebp-0x20]` is the goal's `x`,
written straight from the popped `PathData`. A reading that had believed
the decompiler here would have found a bug that is not there.

**And a correction to `docs/VISION.md` §7.** Its list of the sites that
refresh `mylos` was assembled from a text search and includes the AI's
`check_explore` and `plan_strategy`, neither of which calls `update_los`:
both read `World +0x160`, a *field* at the same offset as the vtable slot.
The real callers are `Unit::init`, `Unit::set_type`, `calc_unit_stats`,
`calc_wall_stats`, `Build::activate`, `Cities::capture_city`,
`Wall::swap_team` and three spells; the dirty bit is raised by
`Build::activate`, `Build::close`, and `gain_tech` only for a gained type
that `is_unit_type` and `is(MILITIA)`. Which leaves the item's own
evidence unexplained: run10's single `mylos` disagreement is player 1's
Scout going 4 → 6 across frames 202 and 203, and the per-frame
`LEADERDATA` shows **no** `0x4000000` on player 1 at the end of either
frame — where player 0 carries it at 202 and is clear at 203, the bit
behaving exactly as read. The check that would settle it is a `rontrace`
run over frames 195–210, so item 35 is behind item 38 too.

One more measurement, banked for item 45: gaia's animals track the
original's positions **exactly** to frame 90 and first part at 91. The
`ANIMALDATA` record is 70,960 animal-frames on run10 and still compares
nothing; the drift it would measure is the stream's, which is item 55's.

## 2026-08-28 — the second steering pass (Fable 5): the tranche holds, and the road's oracle is the problem

Two days and thirteen commits of Opus sessions since the first steer
(2489b8b), all on `worktree-replan-pdb` and none yet on `main`. The
question was whether to intervene. **No** — the loop is doing what it was
set up to do, and the numbers say so: ticks before divergence **3 → 181**,
orders **2 → 180**, the run14 draw-for-draw ledger **179 → 198 of 284**,
every step booked with its number, the one that went backwards (item 52)
stated as such. `cargo test --workspace` with `RON_INSTALL` set: 132
rondata and 610 sim tests, no `skipping` lines — the dump-backed tests
found the captures and ran. The wasted session (permissions read as "no
display") is recorded honestly with its lesson, which is now a rule in
`CLAUDE.md` rather than a line in a handoff that gets rewritten.

### The road, read a third time

`docs/ROADS.md` §4–§5 were re-derived from the export and the listing —
`astar_caravan_road`, `find_road`, `place_roads`, `calc_road_cost`,
`valid_roadcoord`, `first_open_node`, `find_node_open`, both `BRTree`
inserts and `seek` — and **every line agrees with `roads.rs`**. Two
additions from the listing: `find_tcoord_z@008544a0` has a fourth,
stack-passed argument the decompiler cannot name, and both road call
sites push `0` for it, so the root's unclamped `z_val` rests on the
listing now rather than on inference.

Then the one input §7 admitted was never diffed, the heights, and a chase
that ended in a dead end with useful debris. `calc_road_cost` reads
`TerrainOut+0x4a4`, which by `TerrainData`'s record is inside
`master_mount`'s index buffers — and `terraform_for_building` writes it,
in float, at every non-farm building's `Wall::init`. For an hour that
was a second, working height grid the dump never prints, and the search
spends its expansions exactly where it would differ. It is not:
`TerrainOut` (0x6ac0) carries a **0x40-byte prefix** over `TerrainData`
(0x6a80) — `tesselation_level` at `+0x4b3c` is the code's `+0x4b7c` —
so `+0x4a4` is `master_land_heights.list`, the array the dump prints.
And run12's frame-0 heights are already terraformed: p0's city stands on
a plateau of 536.016 with the `(h + mean) × 0.5` blend on its border, the
Library's later terraform overwriting the city's western columns. The
loader's table is the original's grid at frame 0. What the chase left
behind: `gamelog.rs`'s "multiples of ⅛, so the text is exact" is false
(39,746 of 58,081 vertices are off the grid, because of the terraforms);
the loader's exact-millionths mean truncates differently from the
original's `f32` mean on three tiles, none near the searches; and a
building placed *during* a game re-terraforms the grid, which the
simulation does not model. Two queue items, at the back. The trap — a
derived class's `field_0xNNN` may be a base field at `NNN − prefix` — is
in `tools/ghidra/README.md`.

So three readings agree and the count is still twelve short a search.
**The oracle is the problem, not the reading**, and §7 now says what the
next session does instead of a fourth: a building placed on fresh,
un-roaded, sloped ground, a frame window round its road frame, the laid
tiles and the count. Folded with item 38's capture.

### The ledger, two rows

`docs/ARMY.md` §18's `FABLE:` marker — `engagement`'s last-qualifier
fallback — ratified from the listing at `6f52f6`–`6f5383`: the reading
holds, with one refinement. `6f5345`/`6f5349` leave the iteration on a
negative `ox`/`whom` **without** restoring the spills, so a qualifying
unit whose target has gone erases an earlier building target and the
tail gives nothing. `Sim::army_engagement_seed` now does the same; the
new test was made to fail against the old line first. The old test,
`engagement_ignores_a_building_target`, asserted the opposite of §11's
own rule and passed because an unarmed building is not `active`, so
`group_action_attack` ordered nobody — a test written from the reading,
confirming nothing. Rewritten around an armed wall.

The groups "fourth pass" — three rows an earlier Fable pass confirmed
and the implementation overturned — is off the ledger **by diff**, not by
reading: the slot table and `compute_dests` are backed by run29's and
run31's `GROUPDATA` (`docs/GROUPS.md` §12), and Square being dead code
models nothing. The nine audits of 2026-08-20 and the five that followed
stay owed, and the stance stays: a capture retires them faster than a
pass.

### Numbers

Unmoved, and this session did not book a score: ticks 181, orders 180,
ledger 198 / 284.

### Paperwork

`CLAUDE.md` gains the permission rule. `docs/ARMY.md` §11 and §18 amended
and, being pinned, paid for the room by lifting §16.2 and §16.3 here:

> ### 16.2 run21 — the family without an enemy
>
> The islands game to 24000 (`docs/ORACLE.md`): `do_mustering`,
> `do_forming`, `release_mustering`, `find_muster_spot`, `is_moving`,
> `is_engaged`, `set_stance`, `count` from **252** (army 0's first tick);
> `add_group`/`add_unit`/`member` 10187, `get_unit` 10232,
> `center_of_gravity` 10488, `num_armies` 14074, `do_transporting`,
> `find_target`, `find_aggressive_army` **14586**, `do_marching` and
> `Army::close` **14838**, `use_generals` 15898, `leader_defeated` at the
> quit. Never: `engagement`, `march_to_target`, `do_defending`,
> `find_besieged_city`, `remove_group`, `stop`, `send_here`, `charge`,
> `use_scouts`, `use_spies`, `find_waiting_unit`, `find_useful_army`,
> `find_city`, `update_city`, `emergency`, `diplo_change`, `send_navy`.
>
> ### 16.3 run23 — the null result
>
> The same lobby with `6000 war who=1` in `rontrace.cmd`: the line ran
> (`INFO cmd`, `Leader::set_diplo` entered at 6000) and **every one of the
> 24,001 `game_random` words is identical to run21's** — a Quick Battle
> already starts at war (run16 had to declare *peace* first,
> `docs/ORACLE.md`), so the command changed nothing, and run21 was already
> a war in which the AI never marched on an idle human. Worth a line
> because it is the cheapest possible check of "did the scenario take": the
> per-frame RNG word in the trace's `FRAME` records
> (`tools/gamelog/rngcmp.py`).

`docs/ROADS.md` §7 gains the heights' identity and the oracle paragraph;
`docs/audit/README.md`'s ledger strikes two rows; `gamelog.rs`'s
comment tells the truth about the ⅛ grid. `main` fast-forwarded to this
branch at the end of the session.

## 2026-08-28 — item 55: the capture is staged and the mouse is not (Opus 5)

The steering pass said the road's oracle, not the road's reading, is the
problem, and named the capture: a building on fresh, un-roaded ground far
enough from its city to lay a road the map has never had, under a
`DUMP_ALL` frame window and the trace. This session picked the site,
staged the run — and could not drive the lobby, because macOS
**Accessibility** is off for this terminal. The turn ends with the ask.

### The capture, chosen rather than guessed

Three decisions, each made against the simulation rather than by eye.

**The channel places the building; no human hand is needed.** `run_cmd`'s
`add` case, for a type index past the units, calls
`Objects::init_build(who, type, x, y, 0, -1)` and then — when the `NEW`
token is absent — the object's vtable slot `+0x1a8`, which `vtables.txt`
names **`Build::activate`**. `Build::init` calls `find_city` on the way in
and `Build::activate` ends at `City::regen_roads`, so a cheat-channel
`add` is a finished building that flags its city exactly as a human's
would. `ConsoleWin::parse_type` matches the type table's own name with no
availability filter, so a prerequisite the human has not researched is not
in the way.

**The site has to bind to the city, and the simulation knows which do.**
`Build::find_city` → `get_town` → `find_city_at` wants every footprint
tile inside the `CITY_RADIUS` mask and `vector_dist ≤ CityData::get_radius`
(20 tiles for a Small City), which `crate::place` already models. Scanning
every tile 10–20 tiles from p0's centre `(16, 160)`, for every type that
`connects_to_roads` and that `place_building` will accept — the city has a
Library and a Market already, so those two are refused as one-per-city and
the **Granary**, the **Lumber Mill** and the **Smelter** are not — gives
the two the run uses:

| type | tile | dist | the road our search lays |
| --- | --- | --- | --- |
| Granary | `(6, 171)` | 15 | `(16, 164…168)` then west `(8…16, 169)` — an L, 14 tiles |
| Smelter | `(33, 161)` | 17 | `(20…30, 157)` — straight, 11 tiles |

Both on ground with real relief (the climb term is what shapes this
search: flattening it took run14's frame 10 from 208 nodes to 103), both
laying road where the map has none, and on opposite sides of the city so
neither can disturb the other.

**The frames come out of the schedule.** p0's objects run 2000–2006, so
the two new ones are 2007 and 2008, and `(frame + o) % 16 == 0` puts their
replans on sim-frames **105** and **104**. A block `FRAME n` is the end of
sim-frame n−1, so the window `[104, 109)` brackets both roads, the state
before them, and the Market's and the old Library's replans behind them —
five `DUMP_ALL` blocks, about six minutes and 300 MB.

### What the run was to produce, and why it is a fifty-number oracle

The dump gives the laid tiles in the `WORLD` masks and the
post-terraform `master_land_heights` (which is why the terraform this
simulation does not model — queue item 57 — cannot spoil the comparison:
the heights come from the dump, after `Wall::init` has flattened the
footprint). The trace gives, per draw, **the seed before the step**, so
the harness can set `sim.rng.seed` to the word at the search's first
`calc_road_cost` draw and reproduce the original's jitters exactly, then
diff the road tile by tile instead of comparing three totals.

### The block

`waitwin.sh` sat ten minutes on `osascript … get name of every window of
process`, which answers **`osascript is not allowed assistive access
(-1728)`**, and `cliclick p` prints `0,0` with an Accessibility warning.
Screen Recording and Automation both passed the pre-drive probe — they
were re-approved this morning — so the lesson is that there are **three**
permissions and they come back separately; the probe now includes
`cliclick p`, and the memory says so. The game was killed at the main
menu, where nothing is logged.

The staging is left in place — `rise.ini` `Seed=12345 InitialDump=1`,
`gamelog.ini` `DUMP_ALL=1`, `rise2.ini` `LogStartFrame=104
LogEndFrame=109`, `check.ini` and `Player.dat` moved back to map style 14
(Great Lakes, run10–14's map), `rontrace.cfg`/`rontrace.cmd` written — so
that the run is one command once the checkbox is ticked. Anything else
that launches the game before then gets a `DUMP_ALL`; `python3
tools/gamelog/window.py restore` undoes it.

### Numbers

Unmoved, and this session booked no score: ticks 181, orders 180, ledger
198 / 284.

## 2026-08-28 — lifted from `docs/ORACLE.md`, to pay for run32

Two sections written while the original was being made to run at all, both superseded by "Running a check: the recipe in one place" and by the memory entry that carries the launch line. They are the record of how the oracle was got, kept here so the document can stay the *runs'* index (`docs/QUEUE.md` item 40).

> ## Running it: how far Wine gets, and what stops it (2026-08-20)
>
> The first attempt at phase 2, recorded because the failure is specific and the
> partial success is reusable.
>
> **What works.** Homebrew's `wine-stable` cask (Wine 11.0, x86-64 under Rosetta)
> installs without admin rights if `--skip-cask-deps` skips the `gstreamer-runtime`
> `.pkg`, which needs a password and which Wine only wants for media playback. The
> cask fails Gatekeeper, so `xattr -dr com.apple.quarantine` on the app bundle is
> required or the binary is `SIGKILL`ed on launch. A prefix built with `wineboot`
> comes up `win64` with a populated `syswow64`, and **32-bit PE execution works** —
> `syswow64\cmd.exe /c ver` returns `Microsoft Windows 10.0.19045`.
>
> `riseofnations.exe` then launches, loads 70 modules, and runs far enough to
> write its own configuration.
>
> **Which incidentally confirmed this document's INI derivation.** The game
> created `rise.ini` and `rise2.ini` at
> `%APPDATA%\Microsoft Games\Rise of Nations\`, the exact path derived above from
> `Prefs::get_primary_app_directory`, alongside the `synclogger.ini` placed there
> in advance. `rise.ini` also confirms the `[Section] key=value` shape, and turns
> up two settings worth knowing:
>
> ```ini
> [RISE OF NATIONS]
> GraphicsDLL=d3dgl.dll
> AllowLogs=0
> Dialog Error Level (0 - 3)=2
> ```
>
> `GraphicsDLL` means the renderer is a swappable module — but `d3dgl.dll` is the
> only one the install ships, so there is no D3D9 fallback to switch to.
> `rise2.ini`'s `Fullscreen=3` accepts `0` for windowed, which works.
>
> **What stops it, and it is not a configuration problem.** Despite its name,
> `d3dgl.dll` implements a **Direct3D 11** context — the strings around its error
> are `d3d11context.cpp`, `IDXGIDevice`, `IDXGIFactory`, `IDXGIAdapter` — and it
> requests exactly one feature level, `D3D_FEATURE_LEVEL_10_0`, with no fallback.
> Three ways of providing that were tried and all three fail:
>
> | path | failure |
> | --- | --- |
> | wined3d over OpenGL (default) | `wined3d_select_feature_level`: none of the requested levels supported with the current shader backend — macOS OpenGL caps at 4.1 |
> | DXVK 3.0.2 | `Skipping: Device does not support required feature 'geometryShader'` → no adapters |
> | wined3d over Vulkan (`renderer=vulkan`) | creates a `VkDevice` on the M4 Max, then `dxgi_device_init` fails `0x80004005` |
>
> The DXVK line is the informative one. **Metal has never had geometry shaders**,
> so MoltenVK cannot advertise the feature, and DXVK requires it. That is an
> architectural gap rather than a missing package, and no amount of prefix
> configuration closes it.
>
> The remaining candidate was **D3DMetal**, Apple's Game Porting Toolkit
> translation of D3D11 straight to Metal, which handles the gaps MoltenVK
> exposes because it targets Metal directly rather than going through Vulkan.
> CrossOver bundles the same technology. The next section is what happened when
> it was tried.
>
> It is worth being clear about what GPTK is *not* for here: it translates a
> Windows binary's D3D calls, which is useful for running the original as an
> oracle and has nothing to do with this project's own renderer. Phase 4 is a
> Rust client and will not go near it.
>
> ---
>
> ## Running it, second attempt: the original runs, and it writes (2026-08-20)
>
> **CrossOver with D3DMetal renders the game fully.** The whole path, so it can
> be repeated without rediscovery:
>
> ```
> brew install --cask crossover
> cxbottle --bottle ron --create --template win10_64 \
>          --param 'EnvironmentVariables:CX_GRAPHICS_BACKEND=d3dmetal'
> wine --bottle ron --workdir <install> --wait-children <install>/riseofnations.exe
> ```
>
> (`cxbottle` and `wine` are under
> `/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/`; the
> `--cx-app` form wants a bottle-internal path and fails on a native one.) The
> game's own configuration lands at
> `~/Library/Application Support/CrossOver/Bottles/ron/drive_c/users/crossover/AppData/Roaming/Microsoft Games/Rise of Nations/`,
> the same `%APPDATA%` path as before. The first launch after creating the bottle
> page-faulted once in a system DLL; the second and every later one ran: player
> profile, main menu, Quick Battle setup, map generation, the in-game view with
> the economy ticking at its normal rate. Keyboard input reaches it from
> `osascript`; synthetic clicks from System Events do not, and `cliclick` (brew)
> does. The window is borderless at screen size, so screenshot coordinates are
> click coordinates. The in-game menu is the icon at the top-right corner of the
> screen; Escape does not open it.
>
> **What it wrote back settles most of the open questions below, and one of the
> answers is a second oracle nobody had derived.**
>
> ### The SyncLogger's real configuration
>
> The game rewrites `synclogger.ini` on first read with its full key set. There
> is no mask. Every one of the 37 categories is its own key — `WorldSync=0`,
> `UnitsSync=0`, … — defaulting to **0**, so a file that sets only
> `DesyncTrackingEnabled=1` and `DesyncCategoryMask=-1` enables nothing. It also
> adds `SkipCountdown`, `NoteOnlySync`, `FinalSync` and, after a run, a
> `LogFile=` line naming the `Logs\` directory. The categories-to-track header it
> later writes lists `NoteOnlySync` and `FinalSync` as "cannot be turned off".
>
> With every category on and a Quick Battle played for two minutes and quit
> through the menu, four files appear in `Logs\`: `SyncLog Standard .txt`,
> `SyncLog TurnLog .txt`, `SyncLog SendLog .txt`, `SyncLog ReceiveLog .txt` —
> the `%s %s` of the pattern above are the session mode and an empty lobby
> string. Each holds the settings header and then the line
> `<snipped data frames>` under `Game completed without desync`. **In a solo game
> that does not desync, the frame data is dropped at write time.** That matches
> the code: `Game::run_solo` calls `setupWithConfigSettings`,
> `beginNewLogSession(SessionModeStandard)`, runs the whole game, and only then
> `writeToFileAndReset(null)`; the other writer is `CommandPackage::end_process`,
> on an actual desync, which also sets `mDesyncOnTurn`. Whether a flag makes the
> no-desync write keep its frames is the remaining question, named below. Killing
> the process writes nothing, which is why the first two runs produced no file.
>
> ### The older logger, which is the one that works
>
> `rise.ini` carries `AllowLogs=0`. `Log::init` reads exactly that key through
> `Prefs` and returns before opening anything when it is 0; with `AllowLogs=1`
> the game writes `gamelog.ini` with its own full key set and then
> `Logs\gamelog.txt`, and adds `AllowLogs_ToConsole=1` to `rise.ini`.
>
> `gamelog.ini` is the 2003 engine's logging switchboard:
>
> ```ini
> [Logging Options]
> Checksum Dump=-1
> Checksum Break=-1
> DUMP_ALL=0
> LogFile=...\Logs\gamelog.txt
> DumpFileName=Logs\dumplog.txt
> [Misc Logging]   [Start Game]   [End Game]   [Start Frame]   [End Frame]
> WORLD=0  CITIES=0  BUILDS=0  UNITS=0  ANIMALS=0  WALLS=0  AMMO=0  DEATHS=0
> GROUPS=0  LEADERS=0  GUYS=0  GOODS=0  ITEMS=0  MAPMAKE=0  TERRAIN=0
> PATHFINDER=0  CHECKSUM=0  RULES=0  SCRIPT=0  ...            (37 per section)
> ```
>
> — the same 37 categories as `sSyncDefines`, under five phases. Each category
> under `[Start Game]` dumps that subsystem once when the game starts; under
> `[End Frame]`, **every frame**. The output is a nested text dump produced by the
> objects' own `log_data` virtuals — `UnitData::log_data` is slot 0 of
> `Unit::vftable` — in the shape:
>
> ```
> BEGIN FRAME 100
>   BEGIN UNITDATA
>    BEGIN OBJECT
>     BEGIN SUBOBJECT
>      flags 73
>      o 0
>      who 0
>      x_internal 4248
>      y_internal 32664
>      z_internal 528
>    BEGIN GUY
>   BEGIN LEADERDATA
>    who 0
>    tribe 11
>    ...
> ```
>
> The initial dump with everything enabled (`InitialDump=1` in `rise.ini` plus
> `[Start Game]` all on) is 337k lines and contains `BEGIN CONSTANTS` — **every
> field of the loaded `Constants` struct, by name, in its in-memory
> representation** (`unit_move_speed 1`, `river_modifier 512`,
> `fort_upgrade_terr[scan] 2 4 6 9`, `peasant_rate …`) — followed by `BEGIN
> WORLD` (`seed`, `xs ys`, `player_territory_limit 44`, …), every leader, every
> city, building and unit with positions. That is a direct check on every
> `Slot::Ratio256`/`Ratio100` claim `rondata` makes, and on `Tuning::RON` as a
> whole, read from the program rather than from our reading of its loader.
>
> Two practical facts about cost. With every category on under both `[Start
> Frame]` and `[End Frame]`, the simulation crawled to about one frame per five
> seconds and the file grew at ~25 MB a minute — unusable. With `[End Frame]`
> `UNITS`, `LEADERS`, `DEATHS`, `CHECKSUM` only, the game ran at full speed and
> logged 1,730 frames (1:55 of game time) in 20 MB: one `BEGIN FRAME n` per
> simulation frame, every unit's `flags o who x y z` per frame. ~~Per-unit detail
> beyond the object base (`UnitData::log_data` goes on to `collide_frame`,
> `damage_frame`, `angle`, and some fifty more fields) is emitted with a
> detail-level argument of 1, which is presumably what `DUMP_ALL=1` unlocks;
> untried.~~ **Both halves of that were wrong** — the detail argument is not 1,
> and `DUMP_ALL=1` is not how you ask for it. See "The detail level is the
> knob" below.
>
> **`Seed (0 for random)` in `rise.ini` fixes the game.** Two runs with
> `Seed=12345` produced the same nation, the same map and the same opening; two
> runs with `0` did not. So a logged run is reproducible from a config file,
> which is the property the SyncLogger section above wanted and now has, from the
> older system.
>
> ### What this makes possible
>
> A replay diff no longer needs a recorded game at all. Fix the seed, enable
> `[End Frame]` for the categories a mechanic emits, play or script a short game,
> and `gamelog.txt` is a per-frame ground truth for exactly those subsystems —
> positions for movement, leader fields for economy and tech, deaths for
> attrition — against which `crates/sim` can be run from the same initial dump.
> The `BEGIN CONSTANTS` block is the cheapest win and should be wired into
> `rondata` first.
>
> Two more things the running game offers, both read from the binary before it
> ran: the **unit balance tool** (`game/balancerules.txt`, `UnitBalance` in
> `unitbalance.cpp`) runs scripted unit-versus-unit combats and writes results —
> its switch is `game.semaphore.ptr[1] & 2`, set somewhere unread — and the
> `[Start Game]` dump with `RULES=1` ~~should print the loaded type tables~~
> — ~~**it does not**: the 114 MB run had `RULES=1` under `[Start Game]` and
> wrote no type table and no `COMBATTABLE`; the only `RULES` in it are the
> `GAME_RULES` and `RUSH_RULES` lobby settings. `Game::log_rules_data` is
> reached some other way, or under a flag not yet found.~~ **Found: the flag is
> `DUMP_ALL=1`.** `RULES=1` was never the switch — `Game::log_rules_data` is
> reached from `dump_all`, which `full_dump` calls only when it is passed a
> non-zero argument, and the only thing that passes one is `do_dump_all`. With
> it the type tables are all there: 1,820 `UNITTYPE` blocks, 387 `BUILDTYPE`,
> 255 `TECHTYPE`, and a `COMBATTABLE` of 493×493 shorts. See below.
>
> **And the type blocks carry the loader's derived words, which is what makes
> them an oracle for more than the combat table** (2026-08-25). Each type's
> `log_data` prints the fields the rules files never say: `UnitType::log_data`
> writes `unit_flags`, `unit_flags2` and **`role`** per unit type,
> `BuildType::log_data` writes `build_flags`, and `TechType::log_data` writes
> **eleven `ai[scan]` shorts** — `TechType::ai[11]`, the production AI's
> per-technology weights. So one `DUMP_ALL` start dump settles every derivation
> in `docs/DATALAYER.md`'s "The derived words no column carries", and
> `rondata --types <dump>` checks all of them: 364 roles, 364 `unit_flags2`,
> 129 `build_flags`, 85 × 11 weights, plus `armor` and `splash_percent` for the
> name-group rule. Run3 is the dump.
>
> ### Read back (2026-08-20, later)
>
> The dump is now read by `crates/rondata/src/gamelog.rs`, and what it holds
> is written up in `docs/DATALAYER.md`. In short: at detail level 0, per unit
> per frame the object base (`flags o who x_internal y_internal z_internal`)
> and nothing else — the `GUY` blocks carry `type` (a `TypeIndex`), position
> and `angle` only in the start-of-game dump; per leader `who tribe
> defeated_by gov score leader_flags leader_flags2`, no goods; buildings the
> same base with no type; no terrain under any category the two runs enabled.
> The `CONSTANTS` block's keys are the `Constants` struct's field names — the
> lowercased `rules.xml` tags, one renamed — and its values the loaded
> representation, which `rondata --gamelog` classifies for all 716 matched
> constants and checks against every `Tuning::RON` slot (231 of 232 equal;
> `LIBERTY_FREE_UPGRADES` is loaded and not logged). And the seed reproduces
> the game to the position unit: two runs with `Seed=12345` move the same
> units to the same coordinates on the same frames.

## 2026-08-28 — item 55: the road was never wrong; the heights were another game's (ticks 181 → 202, Opus 5)

The capture staged this morning ran, and it answered a question nobody had
asked. Item 55's whole premise — that `astar_caravan_road` expands six per
cent fewer nodes than the original's — was an artefact of the harness. The
search has been exact all along.

### The run

`tools/gamelog/roadcapture.sh`, nine minutes: run10–14's game (seed 12345,
Great Lakes), a Granary dropped at tile `(6, 171)` and a Smelter at
`(33, 161)` from `rontrace.cmd` at sim-frame 100, a `DUMP_ALL` window over
`[104, 109)` and the draw-site trace throughout. The frame-0 word is
`0x3bd39ae9` and frames 0–3 draw 120, 54, 6, 6, so it is run14's game to
the word; `INFO cmd` says both `add` lines parsed and ran.

**The lobby had to be driven by hand**: the desktop is 1920×1080 with the
main monitor off, not the 3440×1440 every stored coordinate assumed, so the
five blind clicks landed on nothing and the game sat on the Main Menu for
ten minutes. Solo Game is at (960, 565), Quick Battle at (960, 495) and
Start at (292, 994), one press. Both desktops are a table now —
`tools/gamelog/lobby.sh`, sourced by every drive script, which measures the
screen before it clicks and refuses a width it does not know; the whole
path was smoke-tested against the game afterwards.

### What the capture found first, which was not the road

Frame 100's first 2,913 draws are `calc_road_cost`'s — *before* phase 1,
out of the `add`. `Wall::start@0063e810` passes **`REGEN_FORCE`** to
`mask_me`, whose tail is `place_roads`, so a building lays its ring and
plans its road **the moment it starts**; `City::regen_roads`' flag is what
makes it happen again sixteen frames later. No traced game had shown this,
because none had ever placed a non-farm building. `Sim::start_building`
models it now, with its own test.

And the two searches are separable without inference: `Wall::activate`
plays a sound off a *different* generator, so the one non-sync record
inside frame 100 splits the road draws into 1,043 and 1,870. (Looking our
own word up in the trace does not split anything — every draw's seed is the
LCG advanced that far, so the answer is always our own count. That cost
twenty minutes.)

### The heights of another game

The Granary's road came out **tile for tile the original's** and its count
did not, which is only possible if the terrain differs. It does.
`borrow_from_siblings` fills a dump's missing fields from a sibling, and
`Initial::heights` was being taken from **run3** — the same seed, style and
size as run10–14 but `GAME_RULES 0` rather than 1, so its starting
buildings stand elsewhere and `terraform_for_building` flattened different
ground before frame 0. The two grids differ on **237 corners from (7, 83)
to (230, 163)**, over the human's own city as well as the AI's. run12's and
run13's agree with each other exactly, and they are the right ones.

With the map's own heights, run14's frames 10 and 11 cost **220** and
**248** — the original's numbers, exactly, where they had read 208 and 222.
`same_start` is the guard: a sibling's heights are borrowed only when every
starting building matches by owner, object number and position. The comment
two fields above already said run3 was "a different start"; the frame seeds
were guarded on it and the heights were not.

### What is diff-backed now

Six searches, node for node, each seeded with the word the trace records
before its first draw: run14's frames 10 and 11, run32's 104–107 (332, 231,
232, 60). And **62 road tiles**, both rings and both roads, on ground the
map has never had a road on. Two tests carry it, and both were made to fail
first — feeding the fresh-road check the post-terraform grid moves the
Smelter's road a row and it says so.

The same capture settles the terraform's timing, which item 57 will want:
the search reads the **pre**-terraform grid. Under run32's frame-104
heights the first search costs 967 nodes and lays a different second road;
under run13's frame-100 heights it costs 1,046 and both roads are exact. So
`terraform_for_building` runs after `place_roads`, and the 128 corners it
moved — the two footprints' boxes and nothing else — are that capture's own
before-and-after.

### The scores, including the two that fell

`Sim::plan_roads` is **on**.

| | before | after |
|---|---|---|
| ticks before divergence | 181 | **202** |
| player 0's first parting | 182 | **213** |
| the first frame whose draws differ from the trace | 10 | **18** |
| ticks before an order diverges | 180 | 168 |
| the ledger, frames matching the trace | 198 / 284 | 173 / 284 |

The first three are the gain and the last two are the same coin. Frames 10
and 11 now spend the 468 draws they always should have, so the stream is
the original's through frame 17 rather than through 9 — and every value
after the *new* divergence at 18 is a different wrong value. `0/3`, which
pinned the old 180, now holds to 213; what pins 168 is `1/1`'s gather wait
at 169, off by ten, which the old stream happened to land on. Past the
first divergence, which of two parted streams labels a frame the same way
is luck, and 198 was that luck with the divergence at 10.

So the ledger test now asserts **the first frame that differs** as well,
which the tail cannot flatter, and every pin that moved carries a line
saying why: the collision block's coverage 43,340 → 40,750, the angle
block's 17,302 → 16,266, the first gather-tile disagreement 407 → 430, and
the bird's hatch frames, which are drift and are now stated as drift — the
wing beat is pinned at the hatch's own offsets instead of at frame 96's.

### What is left of item 55

Two counts, both on the frame a building is *placed*, both with the right
road: 1,046 against 1,043 for the first search of the frame and 1,460
against 1,870 for the second. Ruled out by measurement: the cost terms and
the endpoints (six other searches are exact), the territory arm (no cell
either search touches is unowned), the fog (forcing it moves the road, so
the dump's is the one being used), the route (a coin flip in this model,
and ours takes the original's), and any other split of the 2,913 (scanned
one by one). What is left is `PathFinder`'s own state between two searches
of a frame, and a three-node rounding difference that is item 58's.

## 2026-08-28 — item 59: the builder's own animation (the stream parts at 99, Opus 5)

Item 59 was one frame, one draw, one unit, and it was a whole caller this
crate had never made.

### The lead

With the road on, run14's frames 0–17 matched the trace draw for draw and
frame 18 did not: ours 7 draws, theirs 6, and the extra sat at **index 0** —
`Guy::set_anim+0x97a < Guy::move+0x19f`, the arrival stand. Everything
before it was exact, so there was nowhere for the cause to hide.

The unit named itself in a minute. Its position at the end of frame 17 —
`(40440, 17544)` — is in run10's dump once: player 1's `uid 7`, a citizen
with a `BUILDORDER` on `ox 2006`, standing adjacent to its site. The dump
tracks it walking in from frame 2, arriving on sim-frame 16, and being
turned on 17 (`angle -136249344 → -292028416`, both frames printed). This
simulation had it on the same tile with the same two angles on the same
frames. The only thing it had differently was the animation.

### The rule

`Guy::move`'s arrival test is `field_0x9c == 8 && field_0x9d != 0` — the
guy's **slot** is `CHAR_WALK` and it was already standing on its
destination last frame. `Unit::do_build@005eebf0`'s step 4 is
`set_anim(is(FARM) ? CHAR_SOW : CHAR_BUILD, 0, 1)` and then the facing;
`Unit::do_repair@005ee420` opens with `set_anim(CHAR_REPAIR, 0, 1)` ahead
of every gate. Neither costs a draw — a work animation is its own category
— but between them they mean **a worker is never on the walk slot when it
arrives**, so no capture has a builder's arrival stand in it.

`docs/ORDERS.md` §5.2 has had step 4 written down correctly since the
mechanic was read. The implementation did not have it. That is the whole
bug: two `set_anim` calls, six lines, and a document that needed no
correction. `docs/ANIM.md` §4.6 now states the consequence, which is the
part neither document had — that the work animation is what keeps a worker
off the arrival stand — and §4's caller table has both rows.

### The scores

| | before | after |
|---|---|---|
| the first frame whose draws differ from the trace | 18 | **99** |
| the first frame whose draw **count** differs | – | **122** |
| the ledger, frames matching the trace | 173 / 284 | **219 / 284** |
| ticks before an order diverges | 168 | **185** |
| ticks before divergence | 202 | 190 |
| player 0's first parting | 213 | 191 |

**Frame 99 is not a divergence.** Eight draws either side, and the one that
differs is the same address under a different caller: ours
`Unit::do_idle+0x7d`, the original's `Guy::inc_time+0x271` — the standing
swap `docs/SYNC.md` §6 already names. The word is still the original's.
What parts it at **122** is the blocked stand, `Unit::move_step+0x823`,
which this crate does not take at all (item 49). So the ledger test now
pins that number too, and it was made to fail first.

**`orders` rose and `ticks` fell, and the two are the same 81 frames.**
`1/1`'s gather `wait` at frame 169 — what pinned `orders` at 168 — is the
original's now, because the draws between 18 and 99 are. What pins 185 is
`0/4`, a human farmer whose re-target moved 220 → 186: the same shape of
disagreement (a `MOVE_TO` in front of a gather the original never
re-issues), the same already-wrong `wait`, a different frame. Every other
unit held or improved; `1/1` went 577 → 647. run6, the same game read
through a second capture, fell from 2,591/1,613 disagreements to
**1,588/1,415**.

### The bird came back to its own frame

The row worth more than the headline. This crate's first bird used to hatch
at sim-frame 32 and the original's at 96, so the hatch frames were pinned as
drift and the wing beat was pinned at offsets from the hatch rather than at
frames. With the arrival stand gone the sampling reads the original's cells
off the original's stream and **the first bird hatches at 96, the frame the
original hatches it**. So the ledger test now asserts against the trace
directly:

- `Animal::think_bird`, three draws a bird every eighth frame, **row for row
  from 104 to 192** — twelve rows, ours equal to the trace's;
- the wing-beat coins at **97, 127, 142, 150** on both sides — the birth coin
  the frame after the hatch, then *Bird Soar*'s 31 and *Bird Flap*'s 23.

The second bird is still drift (ours 224, the original's 192) because the
word has parted at 122 by then, and that is now the stated reason rather
than a hedge.

### The pins that moved, and why

Four coverage counters, all in the same direction and all for the same
reason — more unit-frames hold their positions, so more of them are
compared: the collision block 40,750 → 42,615 rows and its split 10 → 18
(the same two fields on `1/4` over six more frames, 316–321, guarded now by
unit as well as by field), the angle block 16,266 → 17,012, the farmers'
share of run6 719/404 → 739/390, and the first gather-tile disagreement
430 → 407, which is `1/6`'s second tree drawn hundreds of frames past any
traced word and has been luck in both directions since item 47.

## 2026-08-28 — a steering aside: the guard-must-fail tool, rejected as a port target (no score, Fable 5)

The `lore` session routed an outside team's CI check — `elenchus.py`, "a
fix is guarded when its changed test records an assertion failure against
the parent tree" — on the grounds that this project breaks its guards on
purpose every session by hand. Read at the source rather than on report,
and rejected on three structural facts, each worse than the last:

- Its test matcher wants `test_`, `_test.`, `.spec.` or a `tests/` parent.
  `find crates -path '*/tests/*.rs'` is empty here; all 58 test-bearing
  files are inline `#[cfg(test)]`. Every commit verdicts `unguarded`.
- Widen the matcher and it lies: copying a changed `.rs` onto the parent
  copies the fix with the test, so the test passes and the verdict is
  `passed` — "not a guard at all". A false accusation against a real guard.
- The one no matcher fixes: its three report formats are Python, Node and
  Solidity, where a call to a not-yet-existing function fails at runtime as
  an assertion. In Rust it fails to compile, which its `classify()` scores
  as `errors > 0 → inconclusive` *before* it can reach `guarded`. For the
  shape this project does most — new mechanic, new function — `guarded` is
  unreachable.

The routing premise was also off: the three standing guards were each
broken once, at creation, not per commit, and the discipline has no
observed defect rate. lore verified all three points against the script,
retracted the premise, and retracted a second claim — that the same repo's
enumerated review register contradicts `docs/DECISIONS.md` entry 22's
"charter, not checklist". It does not: their register is a floor with a
third field, *leads not pursued*, for findings outside it; ours names the
verdicts as the floor and the misses as the mandate. Two vocabularies for
one rule, read as a contradiction at the level of slogans.

**The one keeper** is that third field. `docs/audit/` has "covered" and
"could not settle" (`FABLE:`) and no slot for "noticed and did not chase",
so those evaporate. Whether a record gains the heading is the ratification
pass's call, now on item 42. Set aside knowingly: a perf-discipline skill
(no perf phase exists), a rule refusing epsilons chosen to make a test pass
(`no_float.rs` leaves nothing to choose), and a SHOULD-HOLD/EXPLORATORY tag
declared before a campaign — the live one of the three, since the fuzzer's
own same-day audit kept one finding of three, but it moves no score today.

Verified from the transcript, not the spawn label: this session's system
prompt said Opus and its last twelve API rows say `claude-fable-5`.

## 2026-08-28 — item 49: the blocked stand, and the sheep that had to stand still first (ticks 190 → 192, the word 122 → 185, Opus 5)

Item 49 was one call this crate had never made — and it could not be made
until an unrelated gate on gaia's animals was.

### The lead, which was already written down

`Unit::move_step:281` asks for `CHAR_DEFAULT` the instant a step is refused,
**before** all three give-up tests (`docs/COLLISION.md` §5). The call is at
`005fb74e`, so the draw site is `Unit::move_step+0x823`, and run14's trace
spends it on frames **122, 184 and 256**. Frame 122 was the first frame whose
draw *count* differed from the original's, and the seam had been named in the
code since the mechanic landed.

It had also been *measured* and refused. `docs/COLLISION.md` §7 said adding
the call cost both scores — 43340 → 42755 agreeing unit-frames, 198 → 196
traced frames — "because this simulation's collisions do not yet fall on the
original's frames". That reading turned out to be half right and, as a
verdict, wrong: with the debug print in, this crate enters the collision
block on run14's frames **112, 122, 184** and then a storm from 206. Two of
those three are the original's own, to the frame. The one that is not is
**112, unit `8/1`** — a sheep.

### The sheep

The original's `8/1` does not move at all in run14: `x_internal 17448`,
`y_internal 26424` on every frame from 0 to 119, `collide_frame −1`. This
crate had it at `(17472, 26445)` walking to `(17640, 26568)` and bumping
into `8/0` on the way. The dump was unambiguous and the trace agreed: our
sheep wandered where the original's never does.

`Animal::do_idle@005d7460` says why. After `WorldData::is_valid` it calls

    Unit::detect_unit_collision(this, x, y, 1, 1, 0, 0, 0)

and **only then** `add_move_order`. A wander destination with anything
standing on it is not ordered at all. This lobby's four `HERDSHEEP` are one
herd standing shoulder to shoulder, so for them the gate is not an edge case
— it is what keeps them still. The seam was in the code as a comment
(`// the collision test is a seam`) and cost nothing to close: the four
wander draws are already spent by the time it runs, so it changes an order
and never the stream.

Note the shape of it. Two of the three blocked stands were *already* on the
original's frames; what made the earlier measurement read as a loss was one
spurious collision on an animal, fourteen frames ahead of the first real one.
A measurement that says "the mechanic costs" and a measurement that says
"one unit is in the wrong place" look identical from the score.

### The scores

| | before | after |
|---|---|---|
| run10 `ticks` | 190 | **192** |
| run10 `orders` | 185 | 185 |
| player 0's first divergence | 191 | **193** |
| player 1's first divergence | 203 | 203 |
| run14, first frame whose draw **count** differs | 122 | **185** |
| run14, frames matching draw for draw | 219 / 284 | **235 / 284** |
| run6 orders / paths | 1,588 / 1,415 | **1,351 / 1,340** |
| run6, the farmers' share | 739 / 390 | **503 / 290** |
| collision block, rows compared | 42,615 | 43,575 |
| angle block, rows compared | 17,012 | 17,396 |

The wander gate on its own moves nothing (190/185, and 122/219 on the trace);
the blocked stand on its own *loses* the trace score, parting the word at 112.
The two together are the item.

run6's fall is the largest in the tranche and it is the same mechanism: six
farmers clustered round one farm are what collides most in that capture, so
the idle a refused step re-rolls is theirs more often than anyone's, and
1,003 order disagreements and 198 path ones go with it.

### What it cost, and what it named

The second bird. Ours used to hatch at 96 and 224 against the original's 96,
192 and 256; it now hatches at 96 alone. That is not a regression in the
bird — the sampling reads cells off the stream, the original's second hatch
is on frame **192**, and the word now parts at **185**, seven frames short.
The pin says so rather than hiding it.

And what parts 185 is named: two `orders::SITE_FARM_CELL` draws — the human
farmer's cell re-pick, **item 61** — falling on 185, 187, 189 and 191 where
the original spends them on 199. The queue's next item is the one the trace
points at.

### The checks

Both new tests were made to fail first: `collide.rs`'s
`a_refused_step_re_rolls_the_idle_before_the_give_up_tests` (the mark is
absent without the call) and `anim.rs`'s
`a_wander_onto_an_occupied_cell_is_not_ordered`, a differential pair — the
same seed with and without a unit standing on the destination the open run
picks, asserting the refusal and that it costs no draw.

The trace test's `SITE_BLOCKED` assertion changed sides. It used to say the
original names three and this crate takes none; it now asserts the original's
three frames *and* that ours are `[122, 184]` up to the divergence.

## 2026-08-28 — item 61: the farmer's cell, transposed (ticks 192 → 200, the word 185 → 201, Opus 5)

The trace said the human's farmer re-picked its cell on 185 where the
original re-picks on 199, and the item was booked as "the farmer's
re-target". It is one index.

`FarmStruct` is `uchar[4][4] status` at `+0xac` and `float[4][4] percent`
at `+0x8`. `Unit::do_gather`'s farm branch computes the farmer's `(dx, dy)`
inside the footprint and reads **`status[dx][dy]`** — `005eff54` addresses
`(dx + farm·0x30)·4 + 0xac + dy`, and `Farms::grow(farm, dy, dx)` writes
the same byte from the other argument order. This crate read
`status[dy][dx]`. `docs/ORDERS.md` §6.5 had it transposed too, so the code
was faithful to the document and the document was wrong.

**It hides for a hundred frames.** Every starting farmer stands on
`(2, 2)`, which is its own transpose, and every farm's cell 10 is therefore
the right cell on both readings. The six farmers re-pick on frame 101 — the
first time any of them stands anywhere else — and from that frame all six
sow the wrong cell. `0/4`'s new cell was 61 adds old under the transpose
and empty under the correct index, so it ripened fourteen frames early and
the farmer walked off fourteen frames early, two draws on the sync stream
and a `MOVE_TO` in front of its gather.

### The oracle was in the dump the whole time

`Farms::log_data` writes every farm as flat fields: `who`, `o`, **sixteen
`percent[scan][scan2]` / `status[scan][scan2]` pairs**, twenty-five corner
heights, `valid`, `farm_type`. The harness parsed four of those fields and
threw the thirty-two cells away. Two `DUMP_ALL` captures of this game print
them — run12's frames 1–3 and run13's 95–104 — which is ninety-five frames
of the farm clock, cell for cell, that nobody had ever compared.

`run12_and_run13_s_farm_records_are_the_original_s_cell_for_cell` compares
the whole record now, and it was made to fail twice: dropping
`inc_time`'s `0.005f` parts frame 1, and transposing the sprout's search
parts frame 2 on the AI's `1/2003`. What it does **not** catch is the
defect that prompted it — its captures stop at 104 and no farmer reaches
its new cell until 109 — so the trace test pins the re-target's own frames
instead. Both are worth having; only one of them was the diff that would
have found this.

The reading that settled it came from the same decompile the document was
written from. What was new was knowing which line to read, and the trace
is what said which line.

### The scores

| | before | after |
|---|---|---|
| run10 `ticks` | 192 | **200** |
| run10 `orders` | 185 | **200** |
| player 0's first divergence | 193 | **213** |
| player 1's first divergence | 203 | 201 |
| run14, first frame whose draw **count** differs | 185 | **201** |
| run14, frames matching draw for draw | 235 / 284 | **251 / 284** |
| run6 orders / paths | 1,351 / 1,340 | **1,210 / 1,360** |
| run6, the farmers' share | 503 / 290 | **378 / 312** |
| collision block, rows compared | 43,575 | 39,950 |
| angle block, rows compared | 17,396 | 15,946 |

The trace's re-target row is the item's own: the original spends its two
`orders::SITE_FARM_CELL` draws on 101 (all six farmers), 199, 201, 211,
217, 218, 220 and 241. Ours now agrees on 101, 199, 211 and 217 — every one
up to the word's divergence, and two past it — where before it read 101,
185, 187, 189, 191.

The second bird comes back with it. Ours hatched at 96 and 224 when the
word parted at 122, at 96 alone when it parted at 185, and now at **96 and
192** — both the original's own frames — with the tail past 201 its own.

**Player 1's own number fell, 203 → 201, and it is the newly-correct 199
that exposes it.** The AI's `1/4` re-picks the cell it is *standing on*:
both sides draw `(3, 2)`, both queue a move to `(41400, 17400)`, which is
where the unit already is. The original's move is refused by a collision on
the next frame — run10's frame-201 record carries `collide_o 2`,
`collide_who 1` and the order gone with the unit unmoved — and it re-picks
again on 201. Ours finds a path and walks. That is the successor item and
the first divergence now.

### What it cost the paperwork

`docs/ORDERS.md` and `docs/SYNC.md` are both over the size ceiling, so item
40's rule applied: the new specification went in and an equal weight of
narrative came out. SYNC's two superseded per-frame count tables
(2026-08-24, the ones the draw-for-draw comparison replaced) are the
journal's now; ORDERS lost the audit stories around §6.1, §6.4 and §6.6 and
two settled entries in "what is not established" — the farm re-target
modulus and the 200-vs-201 grow count, both of which this item's diff now
holds. Both pins came down.
## 2026-08-28 — item 63: the waypoint's own collision test (ticks 200 → 207, the word 201 → 232, Opus 5)

`Unit::do_move`'s waypoint take — the block that runs on the frame a move
order's `dest` goes 0 → 1, once per leg — ends with a call to
`detect_unit_collision` at the waypoint. It is the third and last of that
function's call sites, and the only one that runs *before* a step rather
than on one. On a hit, a **final** waypoint under a `GATHER`, `ATTACK`,
`BUILD_AT` or `TRADE_ROUTE` action kills the whole move where the unit
stands; otherwise a **parked** collider — one whose current order is not a
move — widens the tolerance to three of its `big_radius` and the walker
gives up short of it.

`docs/ORDERS.md` §4.4 has had the block written out since the second
reading of 2026-08-21, guard corrected and all. Nothing implemented it.
Twenty lines in `orders.rs` and the traced word went from 201 to 232.

### The case, and a correction to yesterday's reading of it

The item was booked as "the AI's farmer `1/4` re-picks the cell it is
standing on", off `orders_x/orders_y` reading `41400, 17400` — the unit's
own position — in run10's frame-201 record. That is not what happened, and
`orders_x/y` is why: `update_action` writes it as the end of the leading
run of transit moves, so a unit with **no** move carries its own position
there. The record whose move order is intact is frame **200**, and it says
`MOVEORDER x 40824 y 17592`, `off_x 120 off_y 696` — farm cell `(0, 3)`,
not the `(3, 2)` the farmer is standing on.

What is on that cell is `1/2`, and it is not a farmer at all: it is the
AI's woodcutter, `GATHER`ing tile `(213, 92)` from camp `2001` and parked
at `(40872, 17640)`, which happens to be inside farm `2003`'s footprint,
48 units diagonally off cell `(0, 3)`'s centre. Two `coll_size 1` discs
that close overlap, so the cell is blocked.

So: on frame 199 `do_farm` re-picks `(0, 3)` and queues the walk. On 200
`do_move` takes the waypoint, finds `1/2` under it, writes
`collide_o 2 / collide_who 1 / collide_guy 0`, and — the waypoint being
final and the action a `GATHER` — kills the move without a step. On 201
`do_farm` runs again and picks `(2, 1)`, and *that* walk is the one the
farmer takes. This simulation had planned a path on 200 and walked.

The lesson is the one the audit README already carries in another form:
**a dumped field is only as good as its writer**, and `orders_x/y`'s writer
is `update_action`, not the order. The order block was in the same record
all along.

### The scores

| | before | after |
|---|---|---|
| run10 `ticks` | 200 | **207** |
| run10 `orders` | 200 | **206** |
| player 0's first divergence | 213 | **326** |
| player 1's first divergence | 201 | **208** |
| run14, first frame whose draw **count** differs | 201 | **232** |
| run14, frames matching draw for draw | 251 / 284 | **260 / 284** |
| run6 orders / paths | 1,210 / 1,360 | **1,050 / 1,205** |
| run6, the farmers' share | 378 / 312 | **197 / 181** |
| collision block, rows compared | 39,950 | **42,840** |
| angle block, rows compared | 15,946 | **17,102** |
| run10, first gather-tile disagreement | 415 | 407 |

Every one of run6's ten units held or improved its own first divergence —
`0/3` and `0/5` 213 → 326, `0/4` 220 → 356, `1/3` 219 → 345, `1/4`
201 → 316, `1/5` 219 → 243, and `1/0`, `1/6`, `1/7`, `1/8` unmoved. The
non-farmer ceiling rose by 21 order-frames and fell by 24 path-frames, all
of it `1/6`, `1/7` and `1/8` hundreds of frames past their own partings;
the gather-tile row bounced back to 407 for the sixth time, on a draw that
stays luck until the stream reaches frame 407 in step.

**The farm re-target schedule is now the original's, entire.** The trace
test used to compare a prefix — 101 and 199 — with 211 and 217 checked
separately and the tail past the word unusable. It now asserts the whole
list, 101, 199, 201, 211, 217, 218, 220, 241, draw for draw over all 284
frames. The 201 row is this item's: it is `1/4` re-picking a second time
after the refusal.

The bird follows the word again: ours hatched at 96, 192, **224**, 256 and
256 when the word parted at 201, and the spurious 224 is gone now that it
parts at 232 — `[96, 192, 256, 256]` against the original's 96, 192, 256.

### What parts them now

Player 1 at 208 is `1/6`, the woodcutter: the original collides on 206 and
repaths onto a seven-entry stack this simulation does not build. Player 0
at 326 is `0/3` and `0/5`, a farm re-target the original makes a frame
before this simulation does, onto a different cell.

### What it cost the paperwork

`docs/ORDERS.md` is over the ceiling, so item 40's rule applied again: the
new specification went in and more than its weight of narrative came out.
§4.7 ("Collision, in one paragraph") is now a pointer at `docs/COLLISION.md`,
which has owned the mechanic since item 46; §6.5's account of the
transpose's hundred invisible frames is this journal's. The pin came down
to 191,141. `docs/COLLISION.md` gained §5.1 and is well under its ceiling.

## 2026-08-28 — item 64: `is_flat`, the fence on step 2 (ticks 207 → 209, the collision block to zero, Opus 5)

`Unit::resolve_unit_collision`'s step 2 abandons a walk outright when the
unit is standing inside the footprint of the building its `GATHER` action
targets. `docs/COLLISION.md` §6 had the shape of it and one gap: the step
also asks the target's **type** a virtual, `+0x94`, and this crate read it
as `true` and said so in §9 as a seam.

`+0x94` is `BuildTypeData::is_flat` — `build_flags & 0x10000000` — and
`docs/CITIES.md` §1.5 has named it that since the type table was read. So the
answer was in the repository the whole time, one document over from the one
that needed it.

`FLAT` is not a letter in any shipped `BUILD_FLAGS` string: the loader
derives it for the Farm, the Oil Well and the Oil Platform lineages and for
nothing else (`crate::build::init_final_flags`). So step 2 is **the
farmer's step**. A citizen bumped while standing on the field it works
gives up the walk where it stands and lets `do_farm` pick again next frame.
A citizen bumped while standing on a woodcutter's camp — a footprint it is
very likely to be standing on, because a camp is placed among its trees —
is *not* meant to take that branch at all.

### What it cost

run10's `1/6` is the AI's woodcutter. It stands at `(40680, 17688)`, inside
camp `2001`'s footprint, and on frame 206 `do_gather`'s clock runs out and
queues a `MOVE_TO` to `(40680, 18168)`. The first step south, to
`(40680, 17713)`, walks into `1/1` standing three cells below. The original
falls through to step 6: `collide 1`, `collide_frame 206`, `collide_o 1`,
`collide_who 1`, `collide_guy 0`, `coll_x/coll_y` the refused point, the
centre snap a no-op because the unit is already on its cell centre, and
`find_upath` puts a **seven-entry stack** on it — the goal plus six `flags
2` waypoints that go west, south and back east around the blocker.

This simulation killed the order instead. Then `do_gather` queued it again
on 208, killed it again, queued it on 210 — every other frame to the end of
the capture, a unit standing still re-making the same order for 1,500
frames.

### The measurement that says it is right

The dumped collision block — `collide`, `collide_frame`, `collide_o`,
`collide_who`, `collide_guy`, `safe` on every unit-frame whose position
agrees — went from **245 disagreements in 42,840 field-frames to 0 in
48,790**. 243 of the 245 were one sticky byte: `collide_guy` is written by
a hard collision and never cleared, so a single collision this simulation
had and the original did not left a unit reading 0 against −1 for every
remaining frame of the run. That byte was the residue's shape for a week,
and it was never `collide_guy`'s fault — it was the tail of one wrong
predicate. The pin is emptiness now, not a ceiling: one field on one
unit-frame fails it.

### The scores

| | before | after |
|---|---|---|
| run10 `ticks` | 207 | **209** |
| run10 `orders` | 206 | **208** |
| player 0's first divergence | 326 | 326 |
| player 1's first divergence | 208 | **210** |
| `1/6`'s own first divergence | 208 | **253** |
| collision block, disagreements | 245 / 42,840 | **0 / 48,790** |
| angle block, rows compared | 17,102 | **19,464** |
| run6 orders / paths, non-farmer | 853 / 1,024 | **697 / 1,044** |
| run14, first frame whose draw **count** differs | 232 | 232 |
| run14, frames matching draw for draw | 260 / 284 | 260 / 284 |

run6 is run10's game, and every one of its units held or improved: `0/4`
left the divergence list altogether — it now tracks the original's position
for the whole run — `0/3` 326 → 331, `1/3` 345 → 411, `1/4` 316 → 318,
`1/6` 208 → 253, and `0/5`, `1/0`, `1/5`, `1/7`, `1/8` unmoved. The twenty
extra path-frames are `1/6`'s own: it walks for another forty-five frames
instead of standing still, and a unit carrying a stack is a unit whose
stack can disagree.

The traced word did not move, which is the honest half of the entry: `1/6`
takes no draw on 206 that it was not taking before, so run14's sites are
unchanged to the frame. What did change past the word's divergence is one
farm re-target of our own at frame **243**, which the original does not
make inside the traced 284. That is downstream of a word that has been ours
since 232, so the trace test's farm-schedule assertion was split rather
than relaxed: everything before `first_count` must be the original's
exactly, and the one extra row is named and pinned where it is.

### What parts them now

Player 1 at 210 is `1/7`, the citizen trained on frame 206 and sent to the
same camp. Both sides send it with a `MOVE_TO`; the destinations differ —
ours `(40680, 17688)`, which is **the cell `1/6` is standing on**, against
the original's `(40680, 18024)`, seven cells further south. That is
`find_nearby_spot`'s unmodelled half (`docs/ORDERS.md` §10): the search
filters candidates by other units' positions *and their ordered
positions*, and this crate takes the first candidate that is on the map
and walkable. Item 66.

Player 0 at 326 is unmoved: `0/3` and `0/5`, a farm re-target the original
makes a frame before this simulation makes its own, onto a different cell
(item 65).

### The lesson, which is the audit README's

The identification cost one `grep`. `docs/CITIES.md` had `+0x94 is_flat` in
its vtable table, `crates/sim/src/build.rs` had the derived bit with the
listing quoted in its doc comment, and `Sim::add_gather_order` was already
reading `is_flat` for the *same* virtual at the *same* call shape — one
function away in the same decompiled file. What was missing was anybody
asking the seam in `collide.rs` §9 whether it was still a seam. **A stated
assumption is a debt, and the ledger of them is the document's "what is not
established" section**; this one had been sitting on it since item 46 with
the answer already written down elsewhere. Before booking a reading, grep
the documents for the slot.

## 2026-08-28 — item 66: `find_nearby_spot`'s collision half (ticks 209 → 252, run14's word to the end of the capture, Opus 5)

Every walk an order makes ends at a point `UnitType::find_nearby_spot`
returns. The sweep is rings of a fixed radius sequence, 31 bearings a ring
in a fixed order, each candidate snapped to its quarter-tile centre, and a
ladder of tests: on the map, in the region, off a `0x4000` tile, off the
target building's footprint, of the unit's terrain class — and then, last,
**free of other units**. This crate had the first five and not the sixth,
declared as a seam in `docs/ORDERS.md` §10 since the reading: *"on open
ground with nothing in the way the first candidate is free, which is the
stated assumption."*

The ground stopped being open around frame 200 of run10.

### The two queries

`docs/COLLISION.md` §5.2 now carries them. Which pair a call site gets is
its `FilterIndex`, and every build, repair, gather, garrison, idle-wander
and stable site passes `FILTER_NOT_ME` with the unit's own `(o, who)` — so
they all take the **pairwise** pair, and `nocoll` is what skips it:

- **`Objects::find_collision(x, y, o, who, 0)`** — is anything *standing*
  here. A land caller is `CollCheck::collide_here` and nothing else: the
  same probe a step takes, with the caller's own block exempt and the
  parity filter applied. Sea and air walk the 3×3 world cells' object
  chains and compare current positions, Chebyshev, in unit cells.
- **`Objects::find_ordered_collision(x, y, o, who)`** — is anything
  *walking* here. The chain walk for every domain, against each other
  unit's `orders_x`/`orders_y`.

Both skip gaia (`who < 8`) and both fall out at once for a type with no
block. The reading (R7 §1.5, 2026-08-21) had all of it; what was missing
was somebody writing it down in Rust.

### What it cost

run10's AI trains its citizen `1/7` on frame 206, and on 208
`do_non_flat_gather` sends it to camp `2001`. The sweep starts on the
unit's own bearing, and its first *passable* candidate is
`(40680, 17688)` — the exact quarter-tile `1/6` is standing on. The original refuses it, refuses the six bearings behind it,
and issues `(40680, 18024)`, seven quarter-tiles further south: the first
candidate clear of both `1/6`'s block **and** the `(40680, 18168)` that
`1/6` is itself walking to, which is the ordered half earning its place on
the same frame as the standing half.

With the pair, `1/6` and `1/7` both hold to 253.

### The scores

| | before | after |
|---|---|---|
| run10 `ticks` | 209 | **252** |
| run10 `orders` | 208 | **252** |
| player 0's first divergence | 326 | 326 |
| player 1's first divergence | 210 | **253** |
| `1/7`'s own first divergence | 210 | **253** |
| run14, first frame whose draw **count** differs | 232 | **284 (none)** |
| run14, frames matching draw for draw | 260 / 284 | **282 / 284** |
| run14, gaia's bird hatches | 96, 192, 256, **256** | **96, 192, 256** |
| collision block, field-frames compared | 48,790 | 46,941 |
| angle block, rows compared | 19,464 | 18,724 |

### run14's trace is spent, and that is the finding

The word — the per-frame draw *count* — used to part at 232. It now runs
to the end of all 284 frames, and 282 of them match the original draw for
draw. The earliest of the two that do not is 99, the attribution swap the
test has always excused: eight draws either side, the same address under a
different caller.

Gaia's bird makes the same point from the other end. The hatch frames were
`[96, 192, 256, 256]` against the original's 96, 192 and 256 — one hatch of
our own, off a stream that had parted. They are now `[96, 192, 256]`, the
original's three and nothing else, and the assertion is equality rather
than a prefix plus an excuse.

**So nothing on disk can now say where this simulation next parts from the
original by site.** run14 traces 284 of run10's 1,772 frames; the headline
is 252. Item 38 — one long traced capture, `UNITS=3` plus `rontrace`, ≥ 1,800
frames — stops being a nice-to-have and becomes the instrument the next
several items need. It moves to the front of the queue.

### The two things that went with it

**`come_out`'s arms were branched on the wrong field.** `docs/CITIES.md`
§11 said the refusing arm belonged to a type with a non-zero `big_radius`
and that nothing modelled here had one. The branch is on `block_radius`
(`+0x240`), and a Citizen's `BLOCK_RADIUS` is 1 — so every unit any capture
trains takes the refusing arm: `FILTER_NOT_ME` over the exit ring, then the
*same* ring with `nocoll`, then a refusal that keeps the unit inside. The
doubling-and-fall-back-to-the-building arm belongs to the ten
`BLOCK_RADIUS 0` types, and its filter is `FILTER_ALL`, whose general test
is still a seam. §6.5 had this right all along; §11's summary of it did
not, which is the second time in two items that a document's *own* other
section held the answer.

**The collision block's emptiness is now scoped.** Item 64 took it to zero
over the whole capture. Item 66 moved `1/3` from parting at 103 to parting
at 345, and 158 frames past that it takes a collision the original does not
— one sticky `collide_guy 0` from frame 503 to 1600. A unit whose position
has been wrong for a hundred frames is not evidence about collision, so the
assertion is emptiness **before each unit's own first divergence** rather
than over the whole run. That is a stronger check, not a weaker one: it no
longer flatters itself with frames that were never comparable, and it does
not have to be relaxed again the next time coverage moves.

### The lesson

Two items running, the answer was already in `docs/` — `is_flat` in
`CITIES.md` §1.5, and this one in the audit report `R7-spot.md` §1.5, whose
prose describes both functions completely. The pattern is not that the
readings are wrong; it is that **a reading's product is not landed until
somebody writes the code**. `docs/ORDERS.md` §10 has carried the collision
half's full specification since 2026-08-21 and carried "not modelled"
beside it for a week, and the seam cost the score twice — once at 210 and,
before that, in every walk that happened to be lucky. The queue's rule
already says an item is booked with the score it moves; the corollary is
that a *seam* should be booked the same way, and the seam list in a
document's §13 is a queue nobody reads.

## 2026-08-29 — item 38: the long trace, and a spent instrument replaced (the word 284 → 307, Opus 5)

The item the last session put first, and it was first for a reason: run14's
trace reached 284 of run10's 1,772 frames, and once item 66 made the
simulation's word match all 284, **nothing on disk could say where the two
next parted by site**. The residue chase had run out of instrument before it
had run out of residue.

### The capture

`tools/gamelog/longtrace.sh` is the whole run, unattended: the three
permission probes, `tools/gamelog/mapstyle.py` writing style 14 into
`check.ini` *and* the profile's `<MULTI>` block, seed 12345, run10's exact
detail thresholds under both `[Start Game]` and `[End Frame]`, `rise2.ini`'s
frame window `[0, 1900)`, `rontrace.cfg` `cover=1` with no window, and a
two-line `rontrace.cmd` — `5 !ffwd 30`, `1850 !quit`. Fourteen minutes,
284 MB of dump, 9.5 MB of trace, 1,851 frame blocks, `MAP_STYLE 14`,
`(int)seed 12345`, and the frame-0 word `0x3bd39ae9` that every capture of
this game has carried since run11.

### The check that made it worth having

A capture is only a *longer sibling* of run10 if it played run10's game, and
the frame-0 word is a weak way to say so. `tools/gamelog/samegame.py`
reduces each `BEGIN FRAME` block to a digest of its indented lines — the
dump proper, with the ambient `[Misc Logging]` chatter and its wall-clock
stamps left out — and compares two dumps frame by frame. Calibrated on run10
against run14's own gamelog it says 284 identical blocks and one difference,
run14's truncated last. Run10 against run33: **1,771 blocks in common and
not one that differs.**

So the traced executable, an `int 3` on all 48,233 function entries, the
cheat channel and `!ffwd 30` are between them invisible to the simulation
over 1,771 frames. run18a had checked that over four. The instrument is
free, and every future long capture can be taken with it on.

### What it says

The word — the per-frame draw count — is the original's through 306 and
parts at **307**. The original spends eleven draws there and this simulation
nine, and the two it does not spend are one unit's non-flat gather: a stand
issued from inside `Unit::do_non_flat_gather+0x10f`, then `+0x54b`, the
gather's own roll. The site fires nineteen times over the run (frames 1,
168, 204, **307**, 465, 508, 565, 687, 780, 983, 985, 1030, 1091, 1393,
1498, 1544, 1590, 1643, 1781); this simulation makes the first three and
misses the fourth, so it is not a missing mechanism but a clock. That is
item 68.

Over the whole 1,850 frames, 668 spend the original's number of draws and
556 are its draws in its order, both pinned in
`run33_s_long_trace_says_where_the_word_parts` beside the 307. The draw
*sequence* still parts at 99, on the standing-swap attribution item 62
names — the same address under a different caller, with the count equal
either side.

### The thing it did not buy, which is also a finding

The blind list did not move. 617 functions cited by address under `docs/`,
516 entered, **101 never** — with run33 added to the other twenty traces
exactly as without it. run33 entered 6,702 functions against run14's 6,585,
and not one of the extra 117 is cited anywhere. **A long run of the same
no-input game lights nothing new.** The list is shrunk by scenarios, not by
frames, so a run booked to shrink it has to do something no earlier run did
— which is what items 23 and 35 are, and what the ones after them should be
written as.

### The paperwork it forced

ORACLE.md is one of the documents pinned in `docs_guard::OVER`, so adding
run33 to it meant taking something out first — item 40's rule working as
intended. The run23–27, run28/29 and run30/31 sections were chronicle; they
are below, and what they leave behind in ORACLE.md is an inventory table and
the four facts every later run uses (`select` is scriptable, `rontrace.cmd`
clamps a frame to the previous line's, `!quit` at `HI + 1` leaves the
process running, and the East Indies starts). The pin came down from 139,186
to 138,767.

## Lifted from ORACLE.md — the army's and the group's captures (2026-08-29)

The three run-by-run sections below stood in `docs/ORACLE.md` from
2026-08-25/26 until item 38's session, which needed the room. Their
inventory and their traps stayed there under "runs 23–31"; this is the
story.

### run23–run27 — the army's captures (2026-08-25)

Five runs of the run21 lobby for `docs/ARMY.md`, all driven unattended by
one script (`tools/gamelog/runwin.sh N LO HI TAG`: stage with `window.py`,
re-add the scenario lines, launch, the five clicks, poll until the dump is
quiet, archive, restore), chained three at a time.

- **run23** (`gamelog-run23-islands-war.txt`, `rontrace-run23.log`; dump
  off, trace on, `6000 war who=1`): **a null result worth keeping.** The
  line ran (`INFO cmd`, `Leader::set_diplo` entered at 6000) and every one
  of the 24,001 per-frame `game_random` words is identical to run21's — a
  Quick Battle **already starts at war**, so the command changed nothing.
  `tools/gamelog/rngcmp.py A B` is the ten-second check that a scenario
  took; run it before reading anything else.
- **run24** (`gamelog-run24-islands-raid.txt`, `rontrace-run24.log`; seven
  `add hoplite who=0` beside the AI's capital at 12000–12006 and
  16000–16006): the words diverge at 12001, the capital falls at 13125
  (`Cities::capture_city`), the AI is defeated at 16488 and the game ends
  — the first traced game with the army's combat half in it
  (`docs/ARMY.md` §16.4). Two minutes.
- **run25–27**: `DUMP_ALL` windows of run24's game at `[12129, 12132)`
  (`Armies::emergency`), `[12024, 12027)` (`find_target` with a live
  enemy) and `[15100, 15103)` (`do_defending`). About ten minutes each: at
  frame 12000 a block is ~130 MB and takes two to three minutes. Each has
  **five** blocks, not three: the `!quit` at `HI + 1` returns the game to
  the Game Over screen and the simulation runs on at fast-forward until
  the process is killed; the last block (16007 in run25 and run26) is the
  dump written then — a free capture of a later frame, labelled by the
  frame it was written at.

### run28 and run29 — the army engaged while mustering (2026-08-25)

Staged from a reading rather than a guess (`docs/ARMY.md` §16.6): the one
path in `Army::process` that reaches `Army::engagement` is
`do_mustering`'s release, so the army has to be **engaged at its own
mustering tick**. run24's game plus six `add hoplite who=0` on army 0's
own point — run27's block gives it as tile (180, 192) — at 15020–15030,
and quit at 15400.

- **run28** (`gamelog-run28-islands-engagement.txt`,
  `rontrace-run28.log`; dump off, `cover=1`, `window=15095-15105`):
  **three minutes**, and `Army::engagement@006f5160` is entered at
  **15100** with the frame's coverage naming the chain down to
  `Group::action_attack` → `Unit::find_melee_target` →
  `Unit::add_attack_order`. A `cover=1` trace with no dump is the cheapest
  behavioural check there is, and it answers "did this function ever
  execute" outright.
- **run29** (`gamelog-run29-islands-engagement-window.txt`): the same
  scenario under a `DUMP_ALL` window at [15100, 15103), for the records
  on either side of that frame. Ten minutes, ~250 MB, and its `ARMY`
  half is a test the same day (`docs/ARMY.md` §17 item 6) — `status 1 →
  32`, `city 1 → −1`, the point to the muster cell's centre. Its
  **`UNITS=3` half** was opened 2026-08-26: 465 order blocks, 79 move
  orders, and `Army::engagement`'s choice of unit (`docs/ARMY.md` §11).
  It carries **four** states, not three — 15100, 15101, 15102 and the
  free 15105 — and the fourth needed `Log::dumps` to reach ("A `DUMP_ALL`
  frame writes its dump twice", above).

A trap the same session found: `rontrace.cmd` clamps a frame lower than
the previous line's **to it**, so a `!quit` written after a later-frame
`add` runs at the later frame. run27's `15104 !quit` sat after lines at
16000–16006 and so quit at 16006, which is where its free 16007 block
came from. Write the file in ascending frame order.

### run30 and run31 — the human group move (2026-08-26)

`docs/QUEUE.md` item 20: the capture three of `docs/GROUPS.md`'s open
items were waiting on, and the first that needed a **human-shaped**
action in the middle of the run. The cheat table has no order verb, so the
right-click has to come from `cliclick` on the live game — which is why
`tools/gamelog/live.sh` exists: it stages nothing, launches, drives the
five lobby clicks and **returns with the game running**, where
`runwin.sh` would have waited for a `!quit`. `archive.sh N TAG` is the
other end.

**The selection is scriptable after all**, and that was the finding that
made the run cheap. `ConsoleWin::run_cmd`'s `select` case takes
`[[ob#|type] [who] [+]]`: with a type it walks every object of that player
and adds each match to the player's `SelectGroup`, and the trailing `+`
suppresses the clear that otherwise opens the case. So

```
160 select slinger who=0
170 select hoplite who=0 +
```

selects twelve units from the channel, and the only thing left for a
person is one right-click. The earlier note that `+` "is not reliable"
was the **chat box's** dropped lines, not the command: through
`rontrace.cmd` it appended cleanly on both runs, twelve portraits in the
tray.

- **run30** (`gamelog-run30-humangroup-nogroups.txt`, 378 MB, 215
  frames): the same scenario with `[End Frame] GROUPS=1` and `DEATHS=1`,
  so **no `GROUPDATA` came out** — the trap above. Kept anyway, because it
  is the first dump on disk that holds `GroupMoveOrder` blocks, and
  because it proved the select-then-cliclick chain end to end before the
  ini was spent on it.
- **run31** (`gamelog-run31-humangroup.txt`, 160 MB, 219 frames;
  `rontrace-run31.log`, `cover=1`, no trace window): three right-clicks at
  three bearings on the human's own island, eight hoplites and four
  slingers added at frames 60–96 and selected at 160/170. Forty frames
  carry a `GroupMoveOrder` and a live group. `docs/GROUPS.md` §11 and
  §12.1 are what it holds.

**Driving notes.** The window sat at `(760, 152)` again and the five
lobby clicks of `runwin.sh` were unchanged. `cheat camera 29,31` with
`Console Coord Mode=2` centres tile (29, 31), which is enough aiming: the
right-click's own world point is recorded in the order's
`orig_x`/`orig_y`, so the click does not have to be *precise*, only on
land. Reading the screenshot before each click is what keeps it on land.
The human's start on this lobby (East Indies, seed 12345, `MAP_STYLE 18`)
is tiles 26–30 × 25–42; the AI's is 198–214 × 201–214.

At ~370 KB a frame the game runs at about one frame every two seconds,
which is *convenient*: it leaves a driver plenty of wall clock between
frames, and it means a run can be killed the moment the capture is in
hand. `Log` flushes per line, so the kill costs at most the frame in
progress — run31's last block is half-written, which any reader of it has
to tolerate.

### The combat run (run17, 2026-08-24) — the channel's first real run

Fourteen lines, no driver, 2,600 frames in fourteen minutes; the arena is
unowned mid-map land (cells x 27–44, y 27–35 of run9's owner map — tiles
110–170, 110–150), `!ai off` at frame 100 so the AI's wagons stand where
they are placed. A cheat-placed unit faces `0x55555555` = 120° (clockwise
from north, y south) until ordered, so the bearings are precomputed:
`dx = 4 sin b`, `dy = −4 cos b`. No `die` lines — an object number cannot
be predicted from a file, so each trial has its own spot thirty tiles
from the last.

```
100  !ai off
200  add supply who=1 110,110     230  add hoplite who=0 107,108    # rear, b 300°
600  add supply who=1 140,110     630  add hoplite who=0 138,113    # side, b 210°
1000 add supply who=1 110,140     1030 add hoplite who=0 113,142    # front, b 120°
1400 add hoplite who=1 140,140    1430 add hoplite who=0 144,140
1800 add supply who=1 170,120     1830 add slinger who=0 176,120
2200 add tower who=0 170,150      2230 add supply who=1 170,154
2600 !quit
```

What it found is `docs/COMBAT.md` §16: every unit-on-unit hit in the run
is one of the sizes the formula predicts (122; 48/85/117; 32/58/85; 53),
the flank sectors confirmed by damage, the projectile draw sites. What it
taught about staging: **a Supply Wagon flees on sight**, so a wagon is a
one-hit target unless the attacker spawns within striking distance;
`add`'s `find_nearby_spot` moved one hoplite eight tiles from the asked
tile; the AI's own units kept training after `!ai off` (o 8, 9 citizens
at its city), so **`!ai off` stops the leader's strategy, not the
buildings' queues**; and the `[End Frame]` dump grew to 155 KB a frame
with twenty extra units — 2,600 frames in 403 MB.

~~**Open:** `move 6 190,60` from the channel ran (`parse_cmd` returned 1)
and did not move the unit, where the typed `move 10,0 190,60` in run16
moved one to tile (0, 190). Whether `move`'s coordinate arm reads the
mouse tile the channel does not supply, or the unit's engagement at the
time refused it, is a reading of `run_cmd`'s `move` case.~~ **Read
2026-08-26**, and the question was the wrong shape: `move` is a
**teleport** (`Unit::set_new_location`), not an order, and `no_mouse`
widens rather than narrows what the case will do — "The channel's
vocabulary, and what it cannot do", above. **The speed
floor is now the dump, not the input**: run16b ran at ~3.3 sim-frames a
second with `UNITS=3` (137 KB a frame), so a 2,400-frame scenario is
twelve minutes whatever drives it; ~~`ffwd` cannot help while every frame
is logged~~ — **and that is exactly why it helps: gate the dump off and
`ffwd` runs 24,000 frames in a quarter of an hour** (run18a, below) — a
`LogStartFrame`/`LogEndFrame` window around the frames that matter is the
lever.

## 2026-08-29 — item 38, second half: the other map, and its first number (East Indies 167/167, Opus 5)

Item 38 asked for the long trace "on both maps". The first half is above.
This is the second, and it cost three captures of the wrong map before it
cost anything else.

### `-config` pins the map style and no file can move it

`tools/gamelog/mapstyle.py` was written to put a style in `check.ini` and
in the profile with the game closed — faster than clicking a combo and
unable to mis-click. It came back on Great Lakes. So did the next version,
which had found that `Player.dat` carries a `<SOLO>` block *and* a
`<MULTI>` block with a `<MAP_STYLE value>` each and wrote both. So did the
third, which also wrote the `<SETTINGS><MAP_STYLE>N</MAP_STYLE>` near the
file's head. Three fifteen-minute captures, each announced by one line of
its own log.

What settled it was launching **without** `-config check.ini`: the lobby
came up East Indies, from the profile, on the first try. So the chain is
that `-config` builds the lobby from a default `GameInfo` with the file's
rules half applied on top, `mapstyles=` is one of the four combos that
have never taken (in either spelling — `East Indies` or
`#ICON102East Indies`), and the style therefore stays the default 14
whatever any file says. `docs/ORACLE.md` had recorded the four-combo gap
since 2026-08-20 and had never connected it to the map style being
*unsettable*; the note said "read the combo from a screenshot every
launch", which is true and was the wrong lesson.

Two facts fell out of the same hour. **A traced run can never write the
profile**: quitting `riseofnations_trace.exe` through the menu dies in a
Wine `Program Error` box before the write, which is the real reason a
combo pick has never survived a launch — not the `pkill` that was blamed
for it. And the rules half of the profile lobby is run10's anyway
(`MAP_SIZE 2`, `GAME_RULES 1`, `REVEAL_MAP 1`, seed 12345), so dropping
`-config` moves the map and nothing else.

### run38 and run39

`startcapture.sh` takes the `DUMP_ALL` start a game on a new map needs —
two frames, 151 MB — and `longtrace.sh` takes the 1,850-frame
dump-plus-trace, 482 MB. Their traces agree on every frame they share.

**run38 is the whole sibling list.** Its own `Initial` carries the
heights, the checksum trace, the herds and the frame seeds, so `build_sim`
stands the simulation up on a map it has never seen with nothing borrowed
— and frame 0 spends **175 draws against 175** on the first try. Nothing
about that was arranged: it is what the year of Great Lakes work bought.

**The score, first time of asking: ticks 167, orders 167**, player 0
parting at 219 and player 1 at 168. Great Lakes stands at 252 the same
day. The gap matters less than the closeness: a residue chased on one map
for a month could have been chased into that map's shape, and 167 against
252 says it was not.

What parts it is an order-list **length** — `1/4` holds two orders on the
original's frame 168 where this simulation holds one, and its position
parts on the same frame; `1/5` at 186 and `1/3` at 202 are the same
disagreement. That is item 69, and it is the first residue in this project
that was found somewhere other than Great Lakes.

## 2026-08-29 (later, Opus) — item 68: the think tail was conscripting the woodcutters

**ticks 252 → 322, orders 252 → 320; player 0 @ 326 → 356, player 1 @ 253 →
323. run33's word parts at 345 rather than 307.** East Indies is unmoved at
167/167, which is what says the fix is not this map's shape.

### What item 68 turned out to be

The item was booked as a clock. run33's trace says the original spends two
draws on frame 307 that this simulation does not, both from
`Unit::do_non_flat_gather` — the `+0x10f` stand and the `+0x54b` roll of a
citizen choosing a wood tile — and the site fires on 1, 168, 204, **307**,
465, … with the first three made here. A site that fires three times and
then misses looks like a timer that has drifted.

It was not a timer. `tools/gamelog/track.py` on run33's frames 290–320 names
the unit: `1/7`, the AI's eighth citizen, whose `GATHERORDER` sits at
`wait 0, goto_build 1, been_there 0` for seventeen frames, ticks to `wait
−1, been_there 1` on 306 as it reaches its camp, and takes a tile
(`tx 209, ty 96, wait 586`) on 307. This simulation's `1/7` was nowhere
near that camp: it had been given an `ATTACK_TO` to tile (201, 69) on frame
**252** and was walking north-west across the map, and so had `1/6`.

Which is item 67, the other open item — the same two units, the same frame.
The backtrace says `Armies::process_all` → `army_tick` →
`group_action_siege_attack_to`, and §5's cadence explains the frame exactly:
leader 1's army 0 ticks at `frame ≡ 252 (mod 256)`.

### How they got into an army, and how the original says they never do

`Sim::think_join_army` joined "an attacker that is not a scout or a
caravan". A citizen has an attack, so `1/6` joined army 0 on frame 99 and
`1/7` on 205, at the end of an idle think.

Two independent oracles say the original does no such thing.

**The dump.** Every `UNITDATA` record carries `group`, and over run33's
frames 90–219 the only player-1 unit with a non-negative one is the scout
`1/0` — 65 at frame 90, then 64 from 96. `1/6` appears on frame 100 with
`group −1` and `1/7` on 206 with `group −1`, and neither ever changes. That
alone does not settle it, because `do_non_flat_gather` writes `group = −1`
every frame it runs, so a citizen that joined and resigned each frame would
dump the same.

**The coverage.** `tools/trace/report.py rontrace-run33.log functions` lists
every function the traced executable entered and the frame it was first
entered on: 6,703 of them over 1,850 frames. `Unit::add_to_army@005f7740`
is not among them. Neither is `Army::add_unit@006f9f40`, `Army::add_group`,
`find_local_army` or `Army::member`. The AI of a no-contact opening never
puts a single unit in an army; `Armies::init_army` runs at frame 0 from the
census and the slot stays empty, which is why `Army::do_mustering` and
`do_forming` can first run at 252 and move nothing.

### The reading that was wrong, and the one that was right

`Unit::think`'s tail at `005f7615`:

```
if (!is_supply(this) && !is_hero(this)) {
    if ((type->role & 0x10) == 0 && !is(SPY, 0)) return;
    if (get_army() < 0) think_scout(this, 0);
    return;
}
add_to_army(this);
```

`docs/SCOUT.md` §2 transcribed exactly that, in June's scout work, and the
simulation's `scout_thinks` implements its middle arm. `docs/ARMY.md` §4
described the **complement** — "for any type that is not a supply wagon, a
hero, or a spy without the special flag" — and that is the sentence
`think_join_army` was written from. The `attacker` half of the predicate is
`think_attack@005f5a80:155`'s condition, a different call site behind that
function's own city search, grafted onto the wrong one.

So this is not a decompiler trap or a subtle predicate. It is two documents
disagreeing about one listing for four days with the implementation
following the wrong one, and the thing that found it was neither a reading
nor a re-reading: it was following one missing draw back through a
`track.py` on the dump and a `functions` on the trace. The lesson the audit
README already carries — *the predicates are where the errors are* — with a
new corollary: **when two documents cite the same address, they are a diff
waiting to be run.** Nothing checks that today.

A second line came out of the same listing. `005f7195` is
`if (is_worker && think_peasant(0)) goto LAB_005f761a` — the function's own
exit — and `Sim::think` was calling `think_peasant` and dropping its return
value, so a citizen that had just been given a gather job walked on into the
tail on the same frame. It costs nothing on any capture now that the tail
joins nobody, and it is the listing.

### What moved

- `1/6` 253 → **735**, `1/7` 253 → **937**, and player 1's first divergence
  is now the AI's ninth citizen `1/8` at **321**.
- **Player 0 moved with them, 326 → 356**, on the stream the two players
  share: item 65's human-farmer re-target at 326 is gone, `0/3` and `0/5`
  hold to 450 and 455, and `0/4`'s farm walk parts at 351 (its path goal one
  tile north-west of the original's) with the position following at 356.
- The collision block's coverage 46,941 → **62,307** field-frames with zero
  disagreements before each unit's own parting frame, and the angle rows
  18,724 → **24,120** compared, 4,755 of the 5,396 new ones agreeing — the
  best ratio a widening has had.
- run33's word 307 → **345**.

And one number fell. run33's totals over all 1,850 frames go 668 → **618**
frames on the original's draw count and 556 → **460** draw for draw. Every
frame before 345 matches on both counts, so the whole of the fall is past
the divergence: after `1/8` takes the wrong job this simulation is on a
*different* wrong stream from the one it was on after `1/7` was marched off,
and it coincides with the original's less often. The floors are re-pinned at
618 and 460 with that written down, because a total past the divergence is
noise and `first_count` is not.

### The successor

`1/8`'s gather order on frame 321 names the Woodcutter's Camp `2001` with
`dist_mod 4`; the original names `2006` with `dist_mod 0` — a **farm**. So
`Unit::find_gather_spot`'s choice is what parts player 1 now, and it is the
same mechanism queue item 51 wants for the AI's long-run economy.

### Paperwork moved

`docs/ARMY.md` §4 is rewritten to the listing and the section's first-reading
provenance trimmed to keep the file under its pin (lowered to 84,493).
`docs/ORDERS.md` §4.5 steps 4 and 5 are corrected, and its **"How this was
established"** function inventory moved here to pay for it:

*Lifted from `docs/ORDERS.md` 2026-08-29.* Symbol names, struct layouts and field offsets
from `game/sbl/rise.pdb`; behaviour from the decompile export under
`~/ghidra-projects/decomp` (`tools/ghidra/`). Seven readers took one sub-area
each — the list and the frame; the move order and the path seam; build/
repair/garrison and `come_out`; gather and `work`; the start of a game; the
combat and group orders and the command stream; the spatial queries — and
each read its functions in full: `Unit::process`, `work`, `do_job`, `do_idle`,
`check_idle`, `think`, `update_order`, `update_action`, `kill_current_order`,
`close_orders`, `clear_orders`, `add_think_order`, `do_think_order`,
`OrderList`/`LinkListBase` and `OrdersMemManager::*`; `Unit::add_move_order`,
`add_move_facing_order`, `do_move`, `find_path`, `repath`, `move_step`'s
arrival half, `go_around_building`, `resolve_unit_collision` (summarised), the
three `PathFinder::find_*path` wrappers up to their `astar_path` call;
`Unit::add_build_order`, `add_repair_order`, `add_garrison_order`, `do_build`,
`check_build_order`, `build_done`, `do_repair`, `do_garrison`,
`kill_garrison_order`, `go_inside`, `come_out`, `Group::action_swarm_around`,
`Unit::find_build_spot`, `find_repair_spot`, `think_peasant`, `Wall::process`'s
recruiter; `Unit::add_gather_order`, `do_gather`, `do_non_flat_gather`,
`find_gather_spot`, `Build::add_gatherer`/`remove_gatherer`/`check_gatherers`,
`BuildData::num_gatherers`/`is_gathered_by`/`calc_gather`'s count,
`UnitData::is_gathering_at`, `Build::find_gather_tiles`, `WorldData::
has_gather_access`, `GatherPoint*`; `Setup::build_game`/`build_empire`/
`build_cities`/`build_units`/`place_unit`/`small_city_buildings`/
`large_city_buildings`/`build_civ_specific`/`get_starting_citizens`, `Game::run`,
`init_rules_and_teams`, `init_starting_resources`, `do_frame`, `Objects::clear`/
`find_free`/`init_unit`, `Leader::produce_building` (in shape), `GameLog::
check_accept`; `Unit::add_attack_order`, `do_attack`, `fight`'s
order-management half, `do_attack_to`, `do_attack_ground`, `do_guard`,
`do_follow`, `do_patrol`, `add_*` of each, `do_group_move`,
`ungroup_move_order`, `kill_group_move`, `modify_group_order`, `Group::
refresh_group_order`, `distribute_attack`, `action_attack`, `action_move_near`
(the per-member choice in full, the leader-path copy in outline),
`CommandPackage::process_*`; `UnitType::find_nearby_spot`, `UnitData::
invalid_loc`, `Object::adjacent_to`, `WallData::covers_tile`,
`Unit::detect_unit_collision`'s interface. Every `*Order` constructor and
`clear`. Four claims were settled in the PE bytes or the listing
(`llvm-objdump`): `get_starting_citizens`' jump table, `add_move_order`'s
register-passed `find_angle` arguments, the `QueuePos` values in
`add_move_facing_order` (the decompile prints the `QUEUE_NEW` clear as
unconditional; it is not), and `MoveOrder::log_data`'s twenty keys (which also
settle the `this[-1]` offset shift, §1.1). Two real gamelogs at `UNITS=3`
confirmed the list orientation, the log format, the citizen's first moving
frame and a 327-frame build trace. Nothing is transcribed; see
`docs/DECISIONS.md` entry 7.

## 2026-08-29 (later still, Opus) — item 70: the gather score is cap headroom, not distance

**ticks 322 → 355, orders 320 → 350; player 1 @ 323 → 363. Player 0 is
unmoved at 356 and is now the whole headline.** run33's totals go 618/460 →
635/488, East Indies is unmoved at 167/167, and run6's order-field diff
loses the carve-out it has carried since 2026-08-24.

### The item as booked, and what it was

Item 70 was booked on one row: on run10's frame 321 the AI's ninth citizen
`1/8` takes a `GATHERORDER` naming the Woodcutter's Camp `2001` with
`dist_mod 4`, where the original names `2006` with `dist_mod 0` — a farm.
`dist_mod` is not a field to chase; `add_gather_order` writes it from the
building's type, so the whole disagreement is *which building*.

`docs/ORDERS.md` §6.6 had the predicates right and the arithmetic wrong. It
read the numerator as "`Σ_goods rate_g`, `rate_g` the leader's per-good rate
for the building's `best_gather_type`", and `crates/sim` took that term as an
input worth 1 — which makes the score `500 / (dist / 0xc0 + 2)` and the
search a nearest-building search.

It is not a rate. The loop at `find_gather_spot@005f5170` walks `iVar7` from
`0x30` to `0x44` — six goods — over `LeaderData::data_encrypted`, and the
three fields it touches are `resource_cap` (`+0x30`, XOR key `0x1281`),
`over_cap` (`+0x4c`, key `0x8932`) and `income` (`+0x94`, key `0x90236`).
The body is `value += resource_cap[g] − income[g]`, gated on
`type_avail(g, 1)`, on the building being `is(best_gather_type(g))`, and on
`over_cap[g] == 0`. So the numerator is **the unused part of the commerce
cap** for the good this building gathers, and the citizen goes to whichever
good the player is furthest from maxing.

Three readings settle the field names, and none of them is the surrounding
code. `LeaderData::resource_cap_get@0046ee80` is a one-line getter, `return
data_encrypted->resource_cap[i] ^ 0x1281`, which fixes `0x1281`.
`Leader::do_gather@006ce450` writes all three: `income[g] = resources[g] −
support[g] + …`, clamped at `resource_cap[g]`, with `over_cap[g]` set to `1`
or `2` when the clamp bites and to `0` when it does not — and `0x8932` is
`over_cap`'s key, so the search's raw `== 0x8932` is `over_cap[g] == 0`. And
`0x640` is `dutch_interest_cap × 16`: `do_gather` lets the Dutch interest
bonus carry income that far above the cap, and `find_gather_spot` hard-codes
the same headroom where `do_gather` reads the constant.

### Why the distance term could not decide it

Both terms are integers and `dist / 0xc0` buckets by the tile, so ties are
the common case rather than the exception. Frame 321's two candidates are
1,958 (the camp) and 2,041 (the farm) from `1/8`; `1958 / 192` and
`2041 / 192` are both 10, so both denominators are 12 and the distance term
cancels **exactly**. With four woodcutters at the camp against three farmers,
timber income is the higher and food has the larger headroom — so the farm
wins by the numerator alone. A distance-only score cannot produce that row on
any tie-break, and this one had gone to the camp because it is scanned first.

The rest of the function came with the arithmetic: the `tregion` gate,
the city-crossing rule at `:108` (`CityData +0x5a free` plus `+0x5c
gatherers`, the AI census counters, which a human leader never fills — so a
human never crosses), `is_gathering_at` rather than the gatherer chain as the
exemption from `num_gatherers < gather_max`, and the strict `local_20 <
score` from a starting zero, which makes a zero-scoring building unpickable
rather than a last resort.

### What moved

`1/8` parts at 506 rather than 323, and player 1's first divergence is now
the **scout** `1/0` at 363. Player 0 is untouched at 356 — `0/4`'s farm walk,
item 71 — and it is the headline.

The sub-scores, in both directions and said plainly:

- **The gather tile**, the first frame on which any `tx`/`ty` of a gather
  order disagrees: **407 → 1,298**. This sub-score had bounced between 407
  and 430 six times on a draw nobody had fixed; this is not that bounce.
  Every gather tile of the capture now agrees until the frame after the
  original trains `1/9`.
- **run6's order-field diff**: the carve-out is gone. It had excepted a
  farmer after frame 150 for a re-target whose two draws come off a drifted
  stream; there is nothing left in it, and the assertion is now the plain
  "no modelled order field disagrees anywhere in the 432 frames".
- **run33**: `first_count` is **unmoved at 345**, and item 68's note guessed
  wrong about what parts it there. It is not `1/8`'s camp — item 70 fixed
  exactly that and moved the number not at all. The row is ours
  `Guy::set_anim < Guy::inc_time` against the original's
  `Farms::inc_time+0x1ae`, eight draws against seven, byte for byte the same
  before and after. What moved is the totals: 618/460 → **635/488**.
- **The roster**: 744 + 0 → **268 + 400**, back to where it stood before
  item 47. The AI reaches its ninth citizen again — more food, sooner — but
  trains it on frame 897 against the original's 1,297. The over-production
  is unexplained and is its own item; the pair total is what says the whole
  is not worse for it. `mylos` comes back with it, 25,957 → 26,433, its one
  disagreement unmoved.
- **Coverage against the headline**, a sixth time: the collision block
  62,307 → 60,247 and the angle tally 24,120 → 23,296. Two units moved and
  opposite ways — `1/8` +183 frames, `1/4` −113 — and both counts are scoped
  to *agreement* rather than to first divergence, so a re-tasked farmer costs
  more than its own parting.

### The tests

`a_citizen_gathers_where_the_cap_has_the_most_room_left` makes the geometry a
dead heat on purpose — a farm six tiles east, a camp six tiles west, both
`1152 / 0xc0 + 2 == 8` — so only the ledger can decide. The choice flips with
the incomes, an over-cap good takes its building out of the search entirely
rather than scoring it low, and with every good at its cap the search finds
nothing at all rather than falling back on the nearest. It was made to fail
first: with the numerator forced to a constant 1 the tie goes to the
first-placed building and the very first case answers Farm where it wants
Woodcutter.

`find_gather_spot@005f5170` is entered by the traces, so §6.6 is coverage- and
diff-backed rather than reading-only; the two gates no run reaches —
`is_neutralized`, which `crates/sim` does not model at all, and the Dutch
`0x640` — are booked in §14.

### Paperwork

`docs/ORDERS.md` §6.6 is rewritten, §5.2's "which takes the nearest gather
building" corrected, §13's coverage bullet updated and its struck-through
history trimmed to pay for the addition. The document is 190,215 bytes
against a 190,800 pin: **item 40 is now due for this file**, and the next
edit to it that is not a deletion will fail the guard.

## 2026-08-29 (later still, Opus) — item 71: the idle variant nothing had a length for

**ticks 355 → 362, orders 350 → 361; player 0 356 → 464; run33's word
345 → 361, and its totals 635/488 → 696/523.** The item was "`0/4`'s farm
walk, one tile north-west", and the farm had nothing to do with it.

### What the item actually was

Player 0's farmer `0/4` re-targets its cell on run10's frame 349 with two
`GameAccess::rnd(4)` draws, and the goal it walked to was `(1848, 31800)`
against the original's `(2040, 31992)` — one tile north-west in both axes,
which is `r = (1, 0)` where the original drew `(2, 1)`. Both draws wrong by
one is not a farm defect; it is a **stream offset**, and the trace says
where it came from. By the start of frame 349 this simulation had spent two
draws fewer than the original, all of it accumulated over frames 345–348:

| frame | ours | theirs | what |
|---|---|---|---|
| 345 | 8 | 7 | ours spends a wrap the original does not |
| 346–350 | one fewer each | | theirs spends `Guy::set_anim+0x97a < Unit::do_idle+0x7d`, five frames running |

So item 71 and item 75 (run33's word at 345) were the same defect, and the
queue had them as two.

### The defect: a length nothing knew

The human scout `0/0` stands idle from frame 101 on and its guy 0 wraps
every 61 frames — 162, 223, 284 — and on **284** the roll took `CHAR_IDLE1`.
`Art::lengths` is built from `DUMP_ALL` dumps' `GUY` blocks, no dump has ever
shown that slot for the scout's piece, and `Guy::set_anim`'s "a variant the
packet lacks falls back to `CHAR_DEFAULT`" was implemented as "a variant
*nothing knows a length for*". So the scout played its default idle, 61
frames, and wrapped on 345; the original's `CHAR_IDLE1` is **76** frames and
runs to 360.

The five frames the original spends and this simulation did not are the
**dog's**. A crew member past the squad's size mirrors guy 0's `cur_time`
every frame in `Guy::inc_time` and keeps its own `end_time` — 61, its own
default — so from the frame guy 0's clock passes 61 the dog's mirrored time
is past its own end, and `Unit::do_idle`'s `set_anim(CHAR_DEFAULT, 0, 1)`
re-rolls it once a frame until a long enough variant comes up. `docs/ANIM.md`
§5 had predicted exactly this and called it unobserved. It is observed now,
on run33's 346–350.

### The fix: read the install, not the dumps

`docs/ANIM.md` §3.1 (2026-08-28) read the gaia types' lengths out of
`unit_graphics.xml` + `anim_graphics.xml` + the `.bha` files and said the
player units needed `get_unit_gpiece`'s tribe/age/gender walk first. They
need only its **arithmetic**, because a dump already hands over the piece
*number*:

```
piece = (TypeIndex − 0x32) + 0x160·style + 0x840·age + 0x18c0·gender + 0x3180·crew
```

— four literals out of `GraphicPieces::init_piece_ranges@008f70e0`, whose
strides nest exactly (six styles to an age, three ages to a gender, two
genders to a crew) and whose `total_num_unit_pieces` is `0xc606`, four crews
plus the six "over time" pieces `get_unit_gpiece` reaches by `total − 6 …
total − 1`. `unit_graphics.xml`'s `<UNIT name="SCOUT-ARAB-AGE0-CREW1">` is
that sum spelled out, so **inverting the name** gives the piece, and no
tribe, age or gender has to be modelled at all. The six styles are
`say_unit_art_style_name@006f02e0`'s six arms — six consecutive
`internal_strings.xml` entries, `Europe`/`Arab`/`American`/`Asian`/`NA`/
`India`, which are the values a nation file's `<UNIT_CONTINENT>` carries.
1,359 pieces, `rondata::artdata::piece_lengths`.

The four data points the dumps had were the check and they all fell out:
player 0 is Nubian (`1 Arab`) and player 1 British (`0 European`), so the
scouts are `69 − 0x32 + 0x160 = 371` and `19`, the dogs `+0x3180`, the
female citizen `+0x18c0`. And `first_bird_piece = 0xea8d` reproduces run12's
sheep at 60063 through the *other* formula in the same function.

### Two more, both found by the first

**`man_walk.bha` carries thirty-one keys and says thirty.** `key_times` had
required `28 + 36n == 16 + size` — "the chunk ends exactly where its keys
do", which was true of every file the gaia reader had touched.
`AnimObj::load_hier@0054b700` requires no such thing: it reads
`header[1].size` keys from `header + 12` and never compares the two. Nine of
the first run's fourteen failing rows were that one test throwing away every
citizen's `CHAR_WALK` and `CHAR_JOG`.

**The carrying walk is `unit_masks & 0x78000000`, not the gather order.**
With the carrying slots' lengths finally known, a stand-in that had been
invisible for months stopped being invisible: `Sim::gather_walk` derived
`WALK_TO_WOOD`/`WALK_WITH_WOOD` from the order's `goto_build`, and every
walk of a gather became a carrying one — including the citizen's **first**
walk to its camp, which `find_gather_spot` issues and `do_non_flat_gather`
has never run for. The original plays that one as the plain `CHAR_WALK` and
takes the arrival stand `Guy::move+0x19f` at the end of it; run33's `1/7`
carries `unit_masks 262146` on the frame it arrives, and with the carrying
slot in place the word fell from 345 to **232**. `Unit::carry` is the nibble
now, written at `do_non_flat_gather`'s four sites (`:96` clears, `:150`,
`:154`, `:376`, `:694`, `:700` set), `Unit::think:82` for a citizen and
`kill_current_order:107` for anybody.

**And no unit packet names a `CHAR_GROUP_IDLE2`** — none of the 1,337
`<UNIT>` entries — so `set_anim`'s captain gate, the one frame in sixteen
that skips the idle roll, can never fire. `Art::group_idle` was an empty set
waiting for a dump; it is a fact now, and the gate reads the packet.

### The numbers

- **The headline**: ticks 355 → **362**, orders 350 → **361**. Player 0 went
  356 → **464** — `0/4` takes the original's cell, and the three farmers
  behind it hold — and what pins the headline now is player 1's **scout**
  `1/0` at 363, unmoved.
- **run33's word**: 345 → **361**, and the totals 635/488 → **696/523**, the
  largest move either has made. The sequence still parts at 99, and it is
  still the same attribution swap (a new unit's first idle roll: ours in
  `do_idle`, the original's in the phase-7 wrap) at 99, 205 and 319, three
  frames whose *counts* agree.
- **Coverage with the headline**, for once: the collision block 60,247 →
  **62,932** and the angle tally 23,296 → **24,370**.
- **run6 costs one unit.** Its stream is the simulation's own past frame 3,
  and with the clocks moved the AI farmer `1/5`'s second re-target lands one
  frame apart: it parts in position on frame 421 and its order list follows.
  The order-field assertion is scoped to each unit's own first divergence
  now — the same scoping the collision block already had — rather than
  carrying a named exception.

### The tests

`the_install_s_piece_lengths_match_the_dumps` is the two-oracle check, and
where §3.1's asserted the `.bha` arithmetic this one asserts the
**addressing**: the 88 `(gpiece, cur_anim) → end_time` rows five dumps print
over six pieces of two nations, every one against the install. It had teeth
on its first run — fourteen failures, both families real.

**Five of them still fail, and that is the check's other half.** They are all
on the two dogs, and a mirrored guy's dumped pair is not a length row at all:
the slot is guy 0's and the length is its own. Each of the five is a length
the *same piece* carries at another slot, which is what says they are the
mirror rather than a mis-addressed piece — and it means **a dump is not a
source of lengths for a mirrored guy**, which nothing had noticed.

Beside it: `a_unit_graphic_s_name_gives_its_piece` (the grammar and the four
strides, from the names alone), `a_known_piece_s_missing_slot_is_three_
frames_and_not_a_variant` (`slot_length` and `packet_has` are two questions,
and running them together is what swallowed the scout's `IDLE1`),
`the_scout_s_idle1_runs_seventy_six_frames_not_the_default_s_sixty_one` (the
same draw, two lengths, two wrap frames — 345 against 360), and
`the_carrying_walk_comes_off_the_mask_and_not_off_goto_build`.

### Paperwork

`docs/ANIM.md` gains §3.2 and a coverage paragraph, and §9 loses three open
items to it — the unshown lengths, the group-idle gate and the build/repair
clocks — while the walk's speed ratio becomes *observable* for the first
time, because the scout's `CHAR_JOG` is 12 frames against `CHAR_WALK`'s 15.
`docs/ORDERS.md` §6.4 gains one rule (the mask, not `goto_build`) and stands
at 190,546 of its 190,800 pin.

## 2026-08-30 (Opus) — item 76: the scout's surface probe read the wrong tile

**ticks 362 → 436, orders 361 → 427; player 1's scout `1/0` 363 → 484;
run33's word 361 → 432 and its totals 696/523 → 724/606.** Player 0 fell
464 → 450, and every unit that fell parts after the word does. East Indies
unmoved at 167/167.

### One line, four days old, and a `+ 4`

The item was "player 1's scout `1/0` at 363, the whole headline", and the
queue had already named the shape: run33's word parts at **361**, thirty-five
draws against thirty-seven, at index 15 ours `Unit::think_scout+0x436` where
the original has `+0x64c`.

Reading the frame's draws by ring says what that is. Both sides walk seven
rings around London and then two around Napata; every ring's rotation and
phase agree; and in ring 7 the original takes **six** cells where this
simulation takes five. The cell it does not take is `(48, 23)` — which is
the cell the original's scout *goes to*: run33's frame 363 prints `1/0`'s
new order at `dest 37344, 18144`, the centre of tile `(194, 94)`, which is
`4·48 + 2, 4·23 + 2`.

`crates/sim` refused it on §7's surface test, and §7's surface test was
reading the wrong tile:

```
surf = tdata[(4*wy + 2)*tile_xs + 4*wx] & 0x30;    // note: 4*wx, not 4*wx + 2
```

Cell `(48, 23)` is a shoreline cell. Tile `(192, 94)` carries `0x420` —
ocean — and tile `(194, 94)` carries `0x0`.

**The `+ 4` was a fold, not a field.** Ghidra prints the probe as
`*(byte *)(world->tdata + 4 + (...) * 2)`, and two lines above it, over a
*different* array, the identical `+ 4` really is a field offset: `wdata` has
stride `0x1c` and `WData::region` at `+0x4`. But `TData` is `size 0x2` with
`mask` at `+0x0`, so four bytes is **two elements**. The listing says it in
five instructions:

```
005f652e  leal 0x2(,%edi,4), %eax      ; 4·wy + 2
005f6535  imull 0x18(%ebx), %eax       ; × tile_xs
005f6539  leal (%eax,%esi,4), %ecx     ; + 4·wx
005f653c  movl 0x138(%ebx), %eax       ; tdata
005f6542  movb 0x4(%eax,%ecx,2), %al   ; tdata[ecx + 2].mask
```

and the array's canonical accessors say it a second way: `is_cliff_at` and
`is_tocean_slow` are both `tdata[(tile_xs·ty + tx) * 2]`, with no constant at
all. So the surface tile is the cell **centre**, the same tile `invalid_loc`
is handed four lines later, and the asymmetry §7 was written to explain never
existed.

### Why nothing had caught it

Every capture that exercised this mechanic was a **frame-0** one — run20,
the fuzzer's control map, run14's Great Lakes — and the three of them pin the
draw sequence exactly. None of their candidate cells straddles a shoreline,
so the two tiles agree on all of them. A mechanic checked only at the opening
frame is checked on its easiest input, and run33 is the first capture long
enough to reach a second call.

### The numbers

- **The headline**: ticks 362 → **436**, orders 361 → **427**. Player 1's
  scout `1/0` goes 363 → **484**, and eight of the twelve compared units
  improve with it: `1/5` 421 → 663, `1/3` 470 → 583, `0/4` 464 → 577.
- **run33's word**: 361 → **432**, and the totals 696/523 → **724/606** —
  eighty-three more frames are the original's draws in the original's order.
- **Player 0 falls 464 → 450**, and it is downstream rather than a
  regression: the word now parts at 432, and every unit that fell — `0/5` at
  450, `0/3` at 455, `1/4` at 437 — parts after that frame, on a stream that
  is nobody's. `0/4`, which used to pin player 0 at 464, went to 577.
- **Coverage**: the collision block 62,932 → **62,957** and the angle tally
  24,370 → **24,380**, the two directions nearly cancelling.
- **East Indies unmoved at 167/167** — its first divergence is at 168, three
  hundred frames before any scout re-targets.

### What parts the word now, at 432

A gather stand: this simulation spends twenty-three draws where the original
spends twenty-two, and at index 15 ours is
`Guy::set_anim < Unit::do_non_flat_gather+0xb99` where the original spends a
farm's `Farms::inc_time+0x1ae`. That is the successor, at the front of the
queue.

### The test

`run33_s_scout_re_targets_at_361_on_the_original_s_cell` is the frame as
**two** oracles: the trace's own site sequence for frame 361 (twenty-seven
draws over seven rings, filtered to `scout::CODE` on both sides) and the
destination the dump prints for the order the call issues. Unlike
`run20_s_ai_scout_draws_ten_at_frame_0_in_four_rings`, the stream is
*reached* rather than installed — the simulation walks its own way to frame
361. Made to fail by putting the probe back on `4*wx`, which is the defect it
was written for.

### Paperwork

`docs/SCOUT.md` §7 gains a subsection on the misreading and what settled it,
its confidence paragraph says which line was wrong and for how long, §12
lists the new check, and §13 gains one open question the frame turned up: the
order's destination is the tile centre **plus 24 on both axes** on the frame
after it is issued and the centre exactly on the frame after that, and this
crate reproduces the first but not the second. `docs/audit/README.md` gains
the lesson — *a constant byte offset inside an array index is a field offset
only if the element is wide enough to hold one* — and the corollary about
frame-0-only coverage.

## 2026-08-30 (later, Opus) — item 78: the gather order's write-back went to the front of the list

The word parted at 432 on a gather stand: twenty-three draws against
twenty-two, ours `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99` at
index 15 where the original spends a farm's `Farms::inc_time+0x1ae`. The
frame's own draws name the unit — `unit 0/2`, the human's woodcutter — and
its order tells the rest.

### What it was

`Sim::store_gather` wrote the gather order's fields back to
`orders.front_mut()`. Every walk `do_gather` and `do_non_flat_gather` issue
is a `QUEUE_FIRST` move *in front of* the gather order, so on the four
branches that walk, the front is a `Move` by the time the store runs and the
`if let Body::Gather` fell through — a silent no-op. The return to camp is
where it shows, because that branch writes `goto_build = 1` and `wait = 32`
**after** `add_move_order`:

```
find_nearby_spot(b, d, …) ok → set_anim(CHAR_DEFAULT); move(spot);
                               goto_build = 1; wait = 32; unit_masks |= …
```

The original has no such hazard: `do_non_flat_gather(go)` is handed a
pointer to the order object and `add_move_order` only relinks the list head,
so `go->goto_build` lands on the same object all through the function. This
is the same rule `docs/ORDERS.md` §6 already had for the *read* side —
`is_gathering_at@00608880` matches on `get_action()`, not on the front,
"and it is load-bearing" — and the write side was never made to match it.

### What it cost, in the game

`0/2` chopped for its 400-odd frames, finished on 426, set off for its camp
on 427 and arrived on 431. With `goto_build` still 0 and `wait` still −1 it
re-entered the same branch on 432, spent the stand again, and re-issued a
walk to a spot it was already standing on. It did that every other frame for
the remaining 1,400 frames of the capture: it never unloaded, never took
another tile, and never faced its camp.

### The numbers

- **The headline**: ticks 436 → **505**, orders 427 → **482**. Player 0
  450 → **687**, player 1 437 → **506**. **Every one of the twelve compared
  units improves** — `0/5` 450 → 687, `0/3` 455 → 702, `0/4` 577 → 703,
  `1/8` 506 (unmoved, and now what pins player 1), `1/0` 484 → 637.
- **run33's word**: 432 → **482**, totals 724/606 → **752/622**.
- **Coverage, and the first widening whose disagreements fell with it**: the
  collision block 62,957 → **74,429** (11,472 more field-frames comparable,
  the largest move that tally has made) and the angle tally 24,380 →
  **28,916** *against* 9,378 → **7,366** bad.
- **East Indies unmoved at 167/167** — item 69's order-list length at 168 is
  three hundred frames before any of this.

### A booked item that was not what it said

The angle block's own note, written on 2026-08-27 when item 34 landed, read:
"Unit `0/2` alone is 2,680 of it: it walks to `(4440, 28680)` on frame 432
with both sides agreeing on the position, the path and both angles, and on
433 the original turns it to face what it is about to gather while the
simulation leaves it pointing the way it walked." Those 2,680 rows were
booked to item 36, `Unit::set_angle`'s seventeen other callers. They were
not: `0/2` never reached the camp-arrival branch that faces the camp, and
with the write-back landing its share is 140. Item 36 is 2,012 rows smaller
than it was, and what is left of it is the farmers — `0/3`–`0/5` and
`1/3`–`1/5` are 6,658 of the 7,366.

The lesson is the note's own frame number: **432**, the exact frame the word
parted on. A residue attributed to one item and a divergence sitting on the
same frame are worth reading together before either is booked.

### What parts the word now, at 482

The scout again, and the cell filter again. `1/0` re-targets on 482; the
two sides agree draw for draw through index 12 — four rings' `+0x436` /
`+0x458` pairs and two accepted cells — and then the original accepts a
**third** cell in that ring (`+0x64c`) where this simulation moves on to the
next ring. Because `best_ring` is lowered on every hit and the walk runs one
ring past it (`docs/SCOUT.md` §6), the original then stops a ring earlier:
thirty draws against thirty-one, six accepted cells against seven. The same
frame is the order score, as `1/0`'s path stack at 483.

### Paperwork

`docs/ORDERS.md` §6.4's "three lines the implementation has drifted from"
is four, the new one stated as a rule about where a walk goes relative to
the order that issued it; the Status paragraph was compressed to pay for it
and the file is at **190,724 of its 190,800** — item 40 is now blocking for
this document. `store_gather`'s doc comment carries the same rule at the
site. `the_return_walk_goes_in_front_of_the_gather_order_it_updates`
(`cities_tests.rs`) is the focused guard — a woodcutter whose shift has
ended, one `work` call, and the two fields read off the order rather than
off the list head; made to fail by putting `front_mut()` back.

## 2026-08-30 (Opus) — a note for the next steering session: what the byte pins are actually doing

Written the moment it was felt, so it is not reconstructed later. Item 78
needed four lines of rule in `docs/ORDERS.md` §6.4 and the file was 254
bytes under its pin, so the session paid for them by compressing the
Status paragraph. It landed at **190,724 of 190,800**. Three observations
the next `docs_guard` rewrite should have in front of it:

- **The tax lands on whoever adds a finding, and the cheapest thing to cut
  is not reliably the story.** The guard's comment says "the way to add a
  correction to one is to move its story out to the journal first"; what
  actually happens under a 254-byte margin is that a session trims whatever
  it personally needs least. This one considered cutting §6.4's worked
  walk-through of the dump's woodcutter — evidence, not story — and did not
  only because the Status paragraph happened to be softer. The new bullet
  also lost its cross-reference to §6's read-side rule, which is the one
  sentence that would tell a reader the rule generalises.

- **Bytes are the wrong unit; the cost we care about is what a session must
  read to take an item.** Nobody reads `ORDERS.md` whole. A session takes
  §6.4 (about 2 KB) and §4. A 190 KB file with a good section index is
  cheaper to start from than a 55 KB one without. If the metric were "the
  largest contiguous chunk an item requires", the answer would be to split
  `docs/orders/` into per-section files with an index and pin each — and
  then the ceiling would mean something and adding to §6.4 would not tax
  §12. That is item 40 with a design rather than an instruction.

- **The 60 KB ceiling has no split path.** `OVER` is eight grandfathered
  files; everything else is capped flat. `SYNC.md` is 65,400 and pinned,
  `AI.md` 157,530, `ORACLE.md` 138,489 with item 40 owed twice. The next
  document to cross 60 KB — `PATHFINDER.md` at 48,169 is nearest — has
  nowhere to go but into `OVER`, which is how `OVER` got to eight.

None of this is an argument for growth. It is an argument that the ratchet
should measure the thing it is for. `docs/QUEUE.md` items 40 and 42 are
both Fable's, both owed, and the pin design belongs with them.

## 2026-08-30 (Opus) — item 79: the vision disc was thrown along the wrong angle (505 → 571)

**The headline: ticks 505 → 571, orders 482 → 571.** run33's word parts at
571 (from 482), its totals go 752/622 → **791/662**, and every one of
player 1's units improves or holds — `1/8` 506 → 778, `1/5` 550 → 663,
`1/4` 550 → 673, `1/0` 637 → 722.

**What the item was booked as.** The queue said: `1/0` re-targets on
run33's frame 482, the two sides agree draw for draw through index 12, and
then the original takes a **third** cell in that ring where this walks on
— thirty draws against thirty-one — with `best_ring` and the one-ring-past
exit as the suspects (`docs/SCOUT.md` §6). That framing was wrong in an
instructive way: the ring *structure* agrees exactly, seven rings on both
sides with the same two draws each. What differs is one cell inside city
one's ring 7, and everything after it is the shifted stream.

**Ruling out the filter, one predicate at a time.** The ring's ten
candidates are identical on both sides (same `rot`, same `idx`, same
table), and this simulation refuses five of them: `(50, 26)`, `(56, 28)`,
`(58, 28)`, `(48, 20)`, `(48, 24)`, every one at `was_really_seen` with
the cell owned by the scout's own leader. The escape from that arm is
`WData +0` read as a signed short — bit `0x8000`, `GOODY` — and the world
carries exactly 22 goody cells, matching the dump's own `goodies 22`, none
of them in the ring. So the escape was not it, and the question became the
fog.

**§8's arithmetic named the cell where the trace could not.** Every
accepted cell draws at the same site, so the trace says *how many*, never
*which*. The score does: `vector_dist × 8` from the scout's cell `(48,
20)` puts `(50, 26)` at 48, `(48, 20)` at 0 and `(48, 24)` at 32 — all
below the 76 that wins the frame — and the dump prints the winner as cell
`(52, 28)`. So three of the five are impossible, and the original's third
cell is `(56, 28)` or `(58, 28)`.

**The fog was wrong because the projection was.** Both of those were
revealed here by the scout itself, at frames 168 and 140, each at exactly
the disc's edge — offset `(−1, +2)` and `(0, +2)` from a radius-2 centre.
`Object::update_seen` throws a small land unit's disc half a cell forward
of its nose, and this crate projected along `Movement::facing`, the guy's
eased angle. The listing does not: `project` at `00651d05` is handed
`UnitData +0x50` by `movl 0x50(%ecx), %ecx` two instructions earlier, with
`ecx` the `units.list[who][o]` the branch above it read. That is the
unit's *heading* — what `Unit::set_angle` writes toward the next waypoint,
and what the dump prints as `UNITDATA angle`, against each `GUYS`
sub-record's own `angle`.

At frame 140 the two agreed and `(58, 28)` is revealed either way. At
frame 168 the scout was mid-turn: heading −51.6°, facing −83.0°,
thirty-one degrees apart, and the two projections land in different fog
cells. Along the heading, `(56, 28)`'s sample point is three fog cells out
and the radius is two. One line, and the word moved eighty-nine frames.

**`docs/VISION.md` §3 had it right all along.** The section says
`angle = unit->angle` and cites `mov 0x50(%ecx), %ecx` by address; the
implementation reached for the field with the friendlier name. It is the
mirror of item 68's lesson — there the document was wrong and the listing
right; here the document was right and the code did not follow it.

**The widening: the fog grid, whole.** `WData::log_data`'s `WORLD` block
prints all 14,400 bytes of `seen2` and nothing had ever compared them; the
grid was installed from a frame-0 dump and grown by `crate::vision`
against no oracle at all. run13 dumps it ten times, frames 95–104, and
`run13_s_fog_grid_is_the_original_s_on_every_cell_of_ten_frames` now walks
the simulation forward with nothing installed and compares all 144,000
cells. It is exact, and a radius one fog cell too large fails it on the
first frame.

It did **not** catch this bug, and that is the part worth keeping: the
projection is wrong only while a unit turns, and in that ten-frame window
nothing turned far enough to move a fog cell. A monotone grid hides its
own errors until something downstream reads a cell — and the reading
arrives three hundred frames later wearing the downstream mechanic's name.
Item 79 was booked as a scout's ring walk.

**Player 0 fell, and it is the same downstream effect item 76 had.** The
three human farmers went 687/702/703 → 579/574/577. Their old numbers all
sat past the word's own parting at 482, on a stream that was nobody's; the
new ones sit three to eight frames past the new parting at 571. The whole
capture now diverges within eight frames of the word, which is what
convergence looks like rather than a regression.

**What parts the word at 571** is a collision: the original spends a draw
at `5fa882`, inside `Unit::resolve_unit_collision@005f9d30`, that this
simulation does not — eight draws against seven. That is the successor.

## 2026-08-30 (Opus) — item 80: the repath throttle was a lifetime count (571 → 576)

**The headline: ticks 571 → 572, orders 571 → 576.** run33's word parts at
**576** (from 571), its totals go 791/662 → **802/688**, and player 0
recovers everything item 79 cost it and passes it: `0/3`, `0/4`, `0/5` go
577/574/579 → **700/703/687**. East Indies is unmoved at 167/167.

**What the item was booked as, and it was exactly that.** The queue said:
run33's frame 571 is the original's eight draws against this simulation's
seven, and the first to differ is theirs at `5fa882` — inside
`Unit::resolve_unit_collision@005f9d30`, `+0xb52` — against ours
`Farms::inc_time+0x1ae`. That draw is the last line of §6 step 6: the
repath succeeded, the unit it collided with is colliding with *it*, and it
rolls `Random::get(0, 0xffff) % 9 + 1` into the order's `pause`. The
stagger for a head-on pair, so the two do not both step off on the frame
their searches land and collide again.

`docs/COLLISION.md` §9 listed it as unmodelled and asked for a capture:
"two units of the same player ordered into each other head-on". The
capture was already on disk and had been for a day.

**The draw was the symptom; the throttle was the defect.** With the roll
alone nothing would have moved, because this simulation never reached the
repath. The trace of our own side says `1/2` resolves its collision on
571 and 572 and repaths only on 572, and the reason is one line of
`GameDaemon::process_all` that nothing here ran:

```
repaths[p] = repaths[p] / 2;  if (repaths[p] < 3) repaths[p] = 0;
```

`Game::do_frame` calls `GameDaemon::process_all` **every frame**, between
`Leaders::strategy_all` and `Objects::process_all` — so the throttle
`resolve_unit_collision` reads is a *rate*: how hard this player has been
colliding in the last frame or two. Nothing halved it here, so it was a
lifetime tally. Player 1 reached 5 somewhere in the first five hundred
frames and stayed there for the rest of the game, which puts the throttle
permanently into its `repaths ≥ 4` arm — `if ((o + collide) & 3) return`,
three collisions in four thrown away. `1/2` is object 2 with `collide 1`
on frame 571: `3 & 3 ≠ 0`, and the frame was thrown away.

**The document had it, again.** `docs/PATHFINDER.md` §8 has said "halved
every frame and snapped to zero below 3 (`GameDaemon::process_all`)" since
the writers survey, and `crates/sim/src/lib.rs` had `repaths` as a
monotone `Vec<i32>` with one writer and no decay. That is item 79's shape
for the third time in a week — §3 of `VISION.md` pinned the field and the
code read another; here §8 pinned the decay and the code had none — and it
is exactly the pair item 72 is booked to find mechanically.

**The dump named the frame the trace could not.** The trace says how many
draws and at which site, never for which unit. `UNITDATA` carries
`collide`, `collide_o` and `collide_who` on every record of every frame,
and `track.py … --frames 569-573` prints two lines that are not zero:
`1/4` naming `1/2` from 570 to 573 with its own `collide` never moving,
and `1/2` naming `1/4` with `collide 1` in the block after 571. A mutual
collision, which is precisely the tail's guard. Twenty seconds of `grep`
against a reading that would have taken an hour.

**What it cost to check the guards.** Both tests were made to fail on
purpose before landing: with the decay commented out the throttle test
fails on the first tick, and with `collide_pause` commented out the
stagger test reports an empty mark list. The first attempt at the stagger
test asserted on the RNG *seed* rather than the mark, and passed for the
wrong reason — the recovery's own `set_new_location` teleports the body
and can roll an idle variant of its own, so the seed moves either way.
The second attempt failed for a better reason still: the "one-sided" case
was not one-sided, because the two units really had collided during the
setup tick and `1/4`'s `collide_o` already named `1/2`. Both had to be
written out explicitly.

**The numbers.**

- **The headline**: ticks 571 → **572**, orders 571 → **576**; player 0
  574 → **687**, player 1 572 → **573**.
- **run33's word**: 571 → **576**, totals 791/662 → **802/688**.
- **The collision block**: 77,211 → **80,161** comparable field-frames,
  still zero disagreements — the largest single move that count has made
  since item 78, and this is the block's own item.
- **The angles**: 29,878 → **31,022** compared, 7,242 → 7,870 bad. The
  residue is item 36's and nothing else's: `0/3`–`0/5`, `1/3`–`1/5` and
  `1/8`, every one a farmer, are 7,818 of the 7,870 and no other unit
  contributes more than twenty-eight rows.
- Two units fell — `1/8` 778 → 621 and `1/2` to 573 — and both sit past
  the word's own parting.

**What parts the word at 576** is a **city founding**, and it is a large
one: five `ScenarioFuncSet::place_city_with_cost` calls, each spending two
`Leader::compute_sites` draws and then **three** in
`Leader::make_stuff+0x221` under `Leader::found_cities+0x696`. This
simulation spends the compute_sites pair five times and the make_stuff
triple never — forty draws against sixty. That is the successor, and
`make_stuff` was already in the queue's older backlog.

## 2026-08-30 (Opus) — item 81: a building's ramp has no ceiling (576 → 776)

**The item was booked as a city founding, and it was a price.** run33's
frame 576 spends sixty draws where this simulation spent forty, in five
repeats of one block: `ScenarioFuncSet::place_city_with_cost+0x68` calls
`Leader::compute_sites` twice, then `+0x7f` calls `found_cities+0x696` →
`make_stuff+0x221` three times. The last session read that as "the
compute_sites pair five times and the make_stuff triple never". It was
not: unmarked draws inherit the previous mark's label, so this
simulation's five draws were **one whole block** — the pair *and* the
triple, spent correctly once — and the four blocks it did not spend were
not spent at all.

`ScenarioFuncSet::place_city_with_cost@009f5860` says why. Its first act
is a guard:

```
if (get_city_limit() <= get_total_cities()) return -1;
```

and `get_total_cities` counts queued cities. So the five calls the
`defensive.bhs` step-11 loop makes (`num_loops = 5`, through
`aibestbuildlibrary.bhs`'s `city_placement` and its `city_build` trigger)
are five *drawing* calls only while nothing has been bought. The original
made five; this simulation bought on its first and returned −1, without a
draw, four times. **The frame's word is a yes/no answer to "did the AI buy
its second city".**

**Which made it a question about resources, and run33 could not answer
it.** run33 carries `LEADERS=1` at `[End Frame]`: five scalars, no
resources at all. A `LEADERS=9` record is the census oracle and is ~10k
lines a leader, so it is a window setting rather than a whole-run one —
which is what `tools/gamelog/censuswindow.sh` now is: `longtrace.sh`'s
recipe with `LEADERS=9` under `[End Frame]` and `window.py frames LO HI`.
**run40** is `[560, 600)` and **run41** `[770, 800)`, the only two frames
in the whole 1,850 that reach `place_city_with_cost`. Six minutes each,
unattended; both traces' words are run33's draw for draw where they
overlap, which is how we know they are the same game.

**What they measure.** On frame 576 the AI holds **69 food and 59
timber** and does not buy. On 776 it holds 83 and 73, buys, and holds 23
and 14 after. Sixty of each, twice over — and this crate priced the
Small City's second copy at **22**.

**Two ramps, not one.** A Small City is `COST 1t/1f` with `SUPPORT food
50 / timber 50`. `TypeData::get_cost@00664090` is one function with two
ramp arms, and the decompile merges them badly enough that reading it is
not enough; the listing separates them in a line:

```
$ llvm-objdump -d --start-address=0x664090 ... | grep -E '0x35[48]|0x37c|0x39[48c]|0x3a0'
  6651ad: movl 0x354(%eax), %eax    # UNIT_COST_FACTOR
  6653ff: movl 0x39c(%eax), %ecx    # UNIT_OTHER_CIVILIAN_RAMP_MAX
  66568f: movl 0x3a0(%eax), %ecx    # UNIT_MILITARY_RAMP_MAX
  66569f: movl 0x398(%eax), %ecx    # UNIT_WORKER_RAMP_MAX
  6656af: movl 0x394(%eax), %ecx    # UNIT_SCHOLAR_RAMP_MAX
  665787: movl 0x358(%eax), %eax    # BUILD_COST_FACTOR
  665ad1: imull 0x37c(%eax), %esi   # BUILD_SUPPORT_FACTOR
```

Every ceiling is above `665787` and none below it. The building arm reads
its own two factors and **no `RAMP_MAX` at all**; its count is
`num_units[t] + num_queued[t]` off two `u16` arrays with no `PROGRESSION`
shaping. This crate gave every building `RampClass::default()` — the
military 125% — which clamped the fifty-timber ramp to twelve. Ten plus
twelve is twenty-two; ten plus fifty is sixty; the AI held fifty-nine.

**And both readings of 2026-08-20 called the four ceilings "doubly
confirmed".** They were, on one arm. `docs/COSTS.md`'s ceiling table now
says whose they are, and the document has a **What is diff-backed**
section for the first time — the audit's own lesson that the predicates,
not the arithmetic, are where the errors live, in its purest form: nothing
about the formula was wrong, only *which types it applies to*.

**Two smaller listing-backed corrections landed with it**, both from the
same eight lines of `place_city_with_cost`: `compute_sites` is called with
**`force = 0`** (`push $0x0` at `009f58bd`), not 1 — inert for an AI
leader, whose gate is `force || !human`, and not inert for a human one —
and `MakeList::clear()` runs **on both sides** of `found_cities`, where
`found_cities`' own tail clears only when it bought. Neither moved a
number; both are what `docs/AI.md` §2.12 now says.

**The numbers.**

- **The headline**: ticks 572 (unmoved — `1/2` still parts at 573),
  orders 576 → **776**; player 0 687 → **802**. East Indies unmoved at
  167/167.
- **run33's word**: 576 → **776**, totals 802/688 → **944/828** — the
  largest move either has made.
- **The roster**: 268 missing + 400 extra → **468 + 0**. `1/9` used to
  stand here from frame 897 against the original's 1297, four hundred
  frames early, on the hundred and twenty food and timber the AI had not
  spent on a city; it now arrives on 1497, two hundred late. The pair is
  one-sided again for the first time since item 47.
- **The collision block**: 80,161 → **87,548** comparable field-frames,
  still zero disagreements before each unit's own divergence.
- **The angles**: 31,022 → 33,992 compared, 7,870 → **9,156** bad; 1,684
  of the 2,970 new rows agree and the residue is still item 36's farmers
  (8,866 of 9,156), with `0/1` and `0/2` newly on the list at 140 and 136.
- **`mylos`**: 26,433 → 26,233 compared, the one cache disagreement
  unmoved.
- **The first gather-tile disagreement**: 1298 → **1373**.

**What parts the word at 776 is the same block, the other way up**:
theirs five draws, ours twenty-five, because now it is the original that
buys on its first call and this simulation that cannot pay. run40 says
why in one line — the AI's food is **36 against 68** on every frame of
its window, where player 0's food and both players' timber and metal are
exact on all forty. That is item 74, and it is the successor.

**Also measured, and booked as item 82**: the original holds **0**
knowledge, oil and wealth for both players on every frame of the window
and this crate holds **100**. Inert while none of the three is available,
because an unavailable good is never charged — and it stops being inert
the moment the AI ages up.

**What it cost to check the guard.** The new pins can all fail and one
did, on its first run: `run40_s_census_prices_the_ai_s_second_city_at_sixty`
was written with a guessed floor of 144 disagreeing good-frames and
reported 280, which is what produced the breakdown above. Its price
assertion was then made to fail on purpose — with `load.rs` put back to
`RampClass::default()` it reads `[22, 22]` against `[60, 60]` — so the
one line that carries the whole item is guarded rather than assumed. The
`cost.rs` unit test keeps the old number beside the new one for the same
reason: `RampClass::Military` on the same price still gives 22.

**Owed:** `docs/ORACLE.md` is at its pin (138,489) and now owes run40 and
run41 rows; that is item 40's, and it is named in the queue's handoff.
`docs/AI.md` shrank to 156,559 to fit §2.12's new paragraph — its §4, a
closed deliberation, was folded into `docs/DECISIONS.md` entry 20, which
already carried the choice, the two rejected options and the two
conditions.

## 2026-08-30 (Opus) — item 74: the AI's thirty-two food, which was two lumps (776 → 780)

**The headline: run33's word 776 → 780, its totals 944/828 → 951/838.**
run10 is unmoved at ticks 572, orders 776 — but its **roster is**, and by
the largest step it has taken: 468 missing + 0 extra → **268 + 0**, the
lowest the pair has ever been. `1/9`, the AI's ninth citizen, arrives on
the original's own frame instead of two hundred late; the collision block
goes 87,548 → **93,341** compared field-frames and the angles 33,992 →
**35,868**, both of them `1/9`'s rows returning with eight of the other
twelve units holding longer.

**What the item was booked as, and what settled it in the first ten
minutes.** run40's census said the AI's food was 36 against 68 on every
frame of `[560, 600)` — a flat thirty-two — with the human's food and both
players' timber and metal exact. The first move was to widen the same
record rather than read anything: `leftover`, `resources`, `income`,
`resource_cap` and `gather_slots` beside the `bucket` that was already
compared. **`leftover` agreed on all forty frames**, and that is the whole
diagnosis. The fractional accumulator can only agree if the two sides are
paid the same amount every frame, so the thirty-two was never a rate — it
was a lump, or two, somewhere behind the window.

**run13 halved the interval for nothing.** `gamelog-run13-window-95-105.txt`
is the same game (`MAP_STYLE 14`, seed 12345) and its `[Start]` detail
includes `LEADERS=9`, so it carries the census at frames 95–105 as well as
the start. Every field of both leaders agrees there — bucket, leftover,
rate, income — which put the two lumps in `(105, 560)` without a new
capture. A dump already on disk answered what looked like a run.

**And the trace named them.** `rontrace-run40.log` is `cover=1` over the
whole game, so every function carries the frame it was **first** entered
on. Crossing that list against every function in the decompile that writes
the leader's `bucket` — a `grep -l 0x8221` over `funcs/`, twenty-one hits
in the coverage — leaves exactly two inside the interval:
`Build::do_bonus@00627140` at frame 166 and `Build::refund_cost@00620490`
at 201. Twenty and twelve. It took longer to write the cross-reference than
to read the answer off it.

- **Twenty, on 166**: `Build::activate`'s tail. A finished gather building
  adds its `gather_max` to the leader's `gather_slots[good]`, and whatever
  part of that is past `gather_slots_high` is paid for — `FOOD_BONUS_FOR_FARM`
  is 20, flat, and the AI's fourth farm is the slot. The high-water never
  falls, so a rebuilt farm pays nothing. `docs/ECONOMY.md` writes the block
  out, including the two constants that pay **per new slot** rather than
  flat and the wealth case that falls through.
- **Twelve, on 201**: `Leader::gain_tech`'s refund pass. Gaining a Science
  epoch tech re-prices every technology still queued in the player's first
  library, and hands the difference back where it sits. The AI bought City
  State for 120 food on frame 2 and Written Word landed on 201 with the
  City State behind it: `120 → 108`. `docs/COSTS.md` §Paying had been
  calling this a cancel refund since it was first written, while its own
  open-questions section had it right — item 72's shape exactly, and the
  third time in a row that a document disagreeing with itself was a real
  finding.

**The thing worth keeping about `refund_cost`** is that it does not
remember a price. It divides the amount paid by the discount that was in
force when it was paid, recovers the base, and re-strikes it one Science
level further along. That is why a chain of levels does not compound its
truncations — and why the inversion is exact only for an age or an epoch
tech: the purchase side adds one to `TechType +0x1c8` for a plain tech and
the refund reads it raw.

**Three more things the widening turned up, all inert, all booked.**

- The AI's `resource_cap` is **1392** on every frame against the human's
  1120. `70 × 125 / 100` is 87.5 and 87 × 16 is 1392, so the British
  commerce bonus is real, truncates per percentage, and truncates before
  the `<< 4`. `sim::economy::commerce_cap` now computes it — and nothing
  sets `Nation::british`, because **nothing in this harness reads the
  dump's own `tribe` at all**. Every traced game so far has been played
  with no nation power on either side; the dump prints `tribe 11` for the
  AI and `tribe 4` for the human and `rondata`'s own nation table maps
  those to the British and the Nubians. That is a new item and it is a
  bigger one than the cap.
- `gather_slots` agrees on every farm and on neither camp: `Build::init`
  surveys a camp against its own still-empty `gather_from`, so a camp the
  harness stands up from a dump activates with zero slots.
- run40's **human** files one slot under good 2, which
  `BuildTypeData::get_good@0063bd50` cannot produce — its jump table at
  `0063bd84` is Farm 0, Camp 1, Mine 4, University 3, Oil 5 and nothing
  else, checked in the listing rather than the decompile. There is a second
  writer: `Leader::plan_strategy@006b9620` line 1137 assigns the **whole**
  array from `City::count_gather_slots` and raises the high-water to match.
  Unread.

**What parts the word at 780** is four frames past the city the AI can now
afford: eleven draws against eight, and the first of ours is
`GameAccess::rnd+0x20 < Unit::do_job+0x67` where the original's is
`Unit::do_non_flat_gather+0x54b`. A citizen is on a job where the original
has it gathering. That is the successor.

**The method note.** The rule that earned its keep was "diff first, then
read what no run reaches", and it earned it twice over in one item: the
widening said *lump not rate*, the sibling dump said *between 105 and 560*,
and the trace's first-entry list said *these two functions* — three
mechanical narrowings, no reading at all, before a single decompiled
function was opened. What was read afterwards was two short functions whose
identity was already known.

**Owed:** `docs/ORACLE.md` is still at its pin and still owes run40/run41
rows (item 40).

## 2026-08-30 (later, Opus) — item 83: the nation nobody had (run40's cap, 200 → 0)

The item was booked from item 74's widening with a sentence that turned out
to be half wrong, and the half that was wrong is the interesting part.

**What was already true.** `rondata::diff` has read the dump's
`LeaderData::tribe` since the harness existed — `sim.tech[who].tribe` and
`sim.tech[who].power` were both set from it, so `TechTree::has_tribe_bonus`
has been answering correctly all along, and every rule that asks the tree
directly (the free-tech blocks, `has_preq`'s waivers, `tribe_can_type`) has
had its nation. What had no wire was `city::Nation`, the seventeen-boolean
input struct the older mechanics take instead of the tree — `british`,
`nubians`, `egyptians`, `french`, `inca` and the rest, every one of them
false on every traced game since the struct was written. `commerce_cap`
reads `Holdings::british`, `Holdings` copies `Nation::british`, and nothing
ever set it. So the claim "no nation power on either side" was true of the
half of the codebase that the cap happens to live in, and false of the
other half.

**The wire**, `crates/sim/src/nations.rs`: `ROSTER`, the twenty-four
nations in `rules.xml`'s own `TRIBES` order; `Sim::set_tribe`, which takes
the dump's number; and `Sim::refresh_nation_powers`, which recomputes the
booleans. They are computed **through `has_tribe_bonus`**, one call per
roster index, so they are a cache of that answer rather than a second
source and the "No Nation Powers" and no-city gates apply to them for free.
`docs/TECH.md` gains the roster table and the wiring paragraph.

**Two things the old two lines had wrong** besides the missing flags.
`l.tribe.max(0)` turned the `−1` a gaia leader carries into roster index 0,
which is the Aztecs — inert only because the harness's player loop stops
before the gaia slots. And `Setup::no_nation_powers`, which is the gate
`has_tribe_bonus` actually reads, was never set from `info.flags & 4`;
`Lobby::no_nation_powers` was, and the AI's host function reads that one.
Same bit, two layers, and only one of them wired. Both are fixed.

**The score.** run40's census: 560 of 2,880 good-frames disagreed, now
**360**. `resource_cap` was 200 of those and is **0** — both players' whole
cap, on all forty frames, including the AI's British 1392. The two shapes
left are the hundred knowledge/oil/wealth nobody granted (item 82) and the
camps' `gather_slots` (item 85). The headline did not move and was not
expected to: run33 and run10 are the same two nations, and in an Ancient-age
window the British power is a commerce cap that never binds (the largest
rate in the window is 800) and an air-defence build speed with no air
defence, while the Nubian power is Market hit points.

**The guard.** Every nation power in this crate is a hardcoded index, so a
roster that shifted by one would hand a power to its neighbour with nothing
to say so. `cargo run -p rondata -- <install>` now re-derives all
twenty-four names from `rules.xml` and `tribes/` and fails if one moved; it
was made to fail on purpose by swapping the Greeks and the Romans, which it
named exactly, before being restored.

**What is worth remembering** is the shape of the miss rather than the fix.
The cap had a correct implementation, a correct constant, a correct
truncation order and a *diff-backed* comment saying so — and it computed the
wrong number for a month, because one boolean between the tree and the
mechanic had no writer. The widening found it in twenty minutes. A reading
of `calc_resource_caps` would not have: every line of that function was
already right.

## 2026-08-30 — the third steering pass (Fable 5): the loop is faster than its rules, and the rules catch up

Twelve items and sixteen Opus sessions since the 08-28 steer, every one of
them single-threaded — `lore spawns` shows no subagent since 08-25 — and
every one booked with its number. The question was the same as the last
two times: intervene, or `lgtm, back to Opus`. The answer is the second,
with four paperwork rules amended to match what the work already does, one
stale instrument fixed, and a plainer statement of where the end is.

### The tranche

| score | 08-28 | 08-30 | of |
|---|---|---|---|
| run10 ticks before divergence | 200 | **572** | 1,772 |
| run10 orders | 200 | **776** | 1,772 |
| run33's word parts at | 284 | **780** | 1,850 |
| run33 totals, word / draw-for-draw | — | 951 / 838 | 1,850 |
| East Indies (run39) ticks / orders | — | **167 / 167** | 1,850 |
| run10 roster, missing + extra | 468 + 0 | **268 + 0** | `1/10` only |

`cargo test --workspace` with `RON_INSTALL`: 637 sim and 143 rondata
tests, no `skipping` line. One finding spot-checked against the decompile
rather than the journal: `Build::activate` calls
`do_bonus(this, 0, constants->food_bonus_for_farm)` flat for a farm and
`timber_bonus_per_wood_slot × slots` for a camp, which is item 74's
"flat versus per new slot" verbatim.

**The loop changed shape again, and again for the better.** Decision 24
said "diff first, then read". What the tranche actually does is *widen*
first: items 74 and 83 — the two largest roster and census moves — were
closed by comparing dumped fields nobody had compared, with no decompiler
open. Blind readings and adjudications stopped on their own; the listing is
read when one decompiled line is suspect, and that is all the reading there
is. Nothing in the rules forbade any of this. What the rules had wrong was
around it.

### Four amendments (DECISIONS 25)

**The headline is the pair, lower map first.** Entry 24 pinned "the
longest traced capture"; both maps' captures are 1,850 frames, the finish
line names two maps, and one stood at 32–42 % while the other stood at 9 %.
The default item is now the nearest divergence on the lower map, so the
opener goes to East Indies (item 69). A residue that shows on one map only
is exactly the kind a single-map chase never meets.

**The size pin's unit is the section.** Opus's note of this morning ("what
the byte pins are actually doing") was right: a file ceiling taxes whoever
adds a finding to any section of a large file, and under a 254-byte margin
what gets cut is whatever the session needs least. Measured: the largest
`## ` section is 38 KB in ORDERS, 72 KB in AI, 122 KB in ORACLE (its whole
run log), 45 KB in GROUPS; **eleven sections** in all documents exceed
16 KB. `docs_guard.rs` now holds every section under 16 KB, pins those
eleven by file and heading, and has no file ceiling. Made to fail three
ways first — a pin lowered, a heading misspelled, and the unpinned section
that exposed — and it named all three. ORACLE's owed run40/run41 rows now
cost a heading promotion (`### run` → `## run`), not a compression.

**The ratification ledger is the blind list.** The fifteen audits "owed"
a Fable pass carry **zero** `FABLE:` markers between them — the nine of
08-20 predate the marker discipline. "Marked rows only" over unmarked
files was a batch of nothing, owed indefinitely, and the README already
said a capture retires more than a pass. The surface is now claim-level
and mechanical: a cited function no traced run has entered (101 of 617)
that `crates/sim` implements. Item 88 builds the intersection; a run that
enters the function retires the claim; a steer takes what is marked on it.
Items 40 and 42 leave the queue here.

**The steer runs every twenty items, or when the headline stalls two
sessions.** Three passes in four days each found the loop sound.

### The instrument

The run10 floor in `rondata::diff` still asserted `orders >= 576` and
`player 0 @ 687` while the queue had carried 776 and 802 for two items —
the pin is the thing that is supposed to notice, and it was two steps
behind the prose. Raised to what the run prints; item 89 makes the
handoff-equals-pins check a guard so it cannot happen quietly again, and
pairs it with a citation check against the export's `INDEX.tsv`, item 72's
shape made mechanical.

### Where the end is

The engine runs at 15 frames a second, so both 1,850-frame captures are
**two minutes of game**: the Ancient-age opening, citizens and scouts. At
the tranche's pace — roughly 400 frames of headline in two days — the
finish line as written is about a week away on Great Lakes and further on
East Indies. It is the right finish line for the skeleton and it is not
the end of phase 3: what stands beyond it is the next length (item 91;
run18a's 24,000 frames is the precedent), and that is where every mechanic
with a reading and no diff — combat, armies, ages — gets its oracle. The
blind list is shrunk by scenarios, not frames (run33 proved that), so item
90 books the capture queue: a scenario file and a script that runs
captures back to back while the machine is idle, instead of each one
waiting for the session that needs it.

### What was not changed, and the case against changing it

Opus driving, one item per session, `Continue` as the whole prompt, thirty
to sixty minutes an item, no subagents. Considered and not taken: two
sessions in parallel on the two maps — both write `diff.rs`, the queue and
the journal, and the merge tax would eat the gain unless the second lane is
fenced to widenings and captures, which is worth one measured trial and no
more; Sonnet for the widenings — the judgment is in what a dumped field
means, not in the comparison; Fable in the loop — the 11/8 taught that the
oracle is the limit, not the reader. The subagent machinery stays for the
mechanic no run reaches.

### Housekeeping

Six merged branches from the 08-20 readings deleted with their worktrees;
`worktree-agent-a60f39ef1fda297a6` kept, its worktree removed, because ten
of its commits are not ancestors of this branch and a steer does not delete
what it has not read. `main` is fast-forwarded by hand from
`worktree-replan-pdb`, by the user, when they choose.

### Postscript — the giant capture, and why it is a scenario

Asked whether the original could be run faster or headless for a "final"
capture that covers everything. ORACLE.md had already measured the answer:
the sim runs at ~500 frames a second under `!ffwd` and a dump block costs
2.4 s, so a fully dumped 24,000-frame game is sixteen hours of dump and
forty gigabytes, and no renderer work touches that. The giant capture is
one giant *scenario*: the trace whole (cheap; the word over the whole game
is the long headline) and the dump in windows re-captured on demand,
because the game is reproducible and a new window is a five-minute re-run.
Item 91 restated to say so. And a capture takes the screen for thirty
seconds and then nothing a session uses, so item 90's "while idle" was
wrong: it runs beside a session.

## 2026-08-30 (later, Opus) — item 69 was a consequence: East Indies' word parts at 19, on the pasture

The opener said item 69, the second map's order-list length at frame 168.
The first thing done instead was the cheapest thing nobody had done: read
`rontrace-run39.log`. It shipped with run39 on 08-29, 11 MB at `cover=1`,
and no test had ever opened it — East Indies was scored on its dump alone.

**The word parts at 19.** One hundred and forty-eight frames before 168.
So run39's ticks 167 and orders 167 are figures on a stream that is
nobody's, item 69 is a consequence of something much earlier, and the
number to steer the second map by is now the word, exactly as run33's is
for Great Lakes.

### What parts it

The AI's **pasture** — East Indies has one, Great Lakes has none, which is
why run33's word never saw any of this. `docs/SYNC.md` §3.6 had listed
three things it left open in August, and all three are here:

1. **No `type_index`.** A pasture animal is stood up by
   `Sim::farm_add_animals` with `ty` set and the index left at −1, so
   `Sim::slot_length` cannot reach the install's gaia table and every one
   carries `anim::UNKNOWN`. A clock that never wraps costs no draw — and
   that is **three** of the original's six `Guy::inc_time` wraps on frame
   29.
2. **The table had no row anyway.** `rondata::artdata`'s `GAIA_UNITS` left
   `FARMPIG` and `FARMCHICKEN` out of the twelve gaia types, on the ground
   that their missing `-TYPE2` would be a guess. It is not: the `-TYPE0`
   fall-back is the piece pool's own, and both of their `-TYPE` entries
   name the *same three animation files*, so the variant cannot change a
   length. Added.
3. **The wrong species, and the coin is settled.** `Farms::add_animals`
   throws `(rnd & 1) == 0 ? FARMCHICKEN : FARMPIG` per animal — and a
   pasture is therefore **always one species**. The four draws an animal
   are a fixed stride, so every coin lands on the same parity of the
   stream, and `Random::get(0, 0xffff)` returns `((seed & 0xffff) ·
   0xffff) >> 16`, whose low bit is the complement of the seed's, which
   the LCG flips every step. Five even coins or five odd, never a mix.
   Which of the two is a setup draw — but **the trace carries the seed
   before each step**, so a capture's own coins are readable, and run39's
   are even. Chickens: `CHAR_DEFAULT` 30 frames where a pig's is 90.

### The two draws of frames 19 and 20

The animal whose `think_farm_animal` phase hits frame 0 is handed a
`MOVE_TO` this crate reads and does not issue. It walks — silently, since
a walk's re-resolve draws for no gaia type but the bird — and **its
arrival spends two `Animal::do_idle` set_anim draws on consecutive
frames**, 19 and 20. Both rolls come out `≤ 69` (24 and 14, computed from
the trace's own seeds), so both take `CHAR_DEFAULT`; and the animal's
clock is then nineteen frames behind the other four and wraps at 49 rather
than 29, which the trace shows. The signature repeats all game: every one
of the fifteen `think_farm_animal` draws in the first 400 frames is
followed nine to twenty-five frames later by such a pair.

The five animals' `o` values are confirmed as 0–4 with slots 0–4: the
trace's own phase set `{0, 108, 116, 122, 126}` is `(o·(slot+1)) % 128`
for exactly that assignment and no other.

### What landed, and what deliberately did not

Landed: the word test
(`run39_s_long_trace_says_where_the_second_map_s_word_parts`), pinned at
**19**, and beside it the **early window** rather than a total — of the
first 64 frames, 49 spend the original's number of draws and 47 draw for
draw. The window is the right unit here because a total past the parting
is noise, and this session measured that directly: a strictly more
faithful pasture scores *worse* over 1,850 frames (494/329 → 456/310).
Landed too: `GAIA_UNITS`' two missing types, inert until (1) is fixed, and
the comment saying so.

**Not landed: (1) and (3).** Together they take the early window 49 → 60
and 47 → 53, and frames 29 and 32 come right — but they cost run39's ticks
score **167 → 102**, because the stream after 19 is nobody's either way
and 167 was luck on it. Trading a pinned headline for a stream that is
locally better and still wrong is the wrong shape of commit: they land
with the walk, not before it. The measured numbers are here so the next
session can redo the change in minutes.

### The wall, and the shape of the fix

Issuing that `MOVE_TO` needs the animals' **positions**, and
`add_animals` places each of the five at the farm ± `% 0x180 − 0xc0` on
each axis — up to a whole tile — from two draws inside
`Setup::build_empire`, whose stream the harness does not replay. The
farm's own centre puts every arrival frame somewhere else. But the trace
carries those draws as well: run39's five are `(−143, 40)`, `(−187,
−148)`, `(−39, −144)`, `(−83, −76)`, `(−63, 56)` as `(dy, dx)`. So the
pasture's five are **borrowable the way the heights, the herds and the
frame seeds are** — a new kind of borrowing, from a trace rather than a
sibling dump, and that is the item.

`docs/SYNC.md` §3.11 is the specification; §3.6's "what this leaves open"
is struck through and pointed at it, and `docs/ORACLE.md`'s run38/run39
row too.

### Postscript — two inputs item 92 will not have to re-derive

The user asked whether the warm context mapped onto the deferred fix. The
half-fix does not — it is the wrong shape whoever writes it — but two of
item 92's *inputs* were a few minutes each with the decompile open, and
both are now durable rather than in a session's head.

**`corner_x` and `corner_y`.** `rise_z.map` names them at `00adc3e0` and
`00adc3c0` in `compass.obj`, and read out of the PE they are a centre and
four corners — `x = [0, −1, 1, 1, −1]`, `y = [0, −1, −1, 1, 1]`, one per
animal, which is what `Animal+0x154` is for. With them
`think_farm_animal`'s destination collapses to `192·T + {24, 120, 168}`
for a corner of `−1`, `0`, `+1`, and §3.11 carries the derivation. Item 92
no longer owes a PE read.

**`report.py <log> draws` now prints the value each draw returned.** The
whole coin argument of this session rested on a throwaway script that
stepped the LCG from the seed a record carries; the record's seed is the
word *before* the step, so **every draw's outcome is recoverable from a
trace without the game**, including the ones that leave nothing in any
dump — a setup coin, a direction, an idle roll. That belongs in the
instrument, not in a session, so it is fifteen lines in the reader. It
reproduces this session's numbers exactly (`add_animals+0x92` 27358,
`+0x134` 15025, `+0x182` 55912, ...) and names the chain while it is at
it, which confirms §3.8's `Build::activate` correction from the ebp chain
rather than from a reading.


## 2026-08-30 (Opus) — item 92: the pasture's five, and the walk they take

The second map's word parts at **19** and the whole of it is five animals
of owner 9 that no dump prints. Yesterday's session priced the three
things wrong with them and stopped at the wall: the positions are two
draws inside `Setup::build_empire`. This session went through the wall the
way the queue said to — **borrow them from the run's own trace** — and
landed all three parts in one commit.

### The borrow is a new kind

`Initial` has carried the heights, the herds, the farm list and the frame
seeds from *sibling dumps* since the harness existed. `Initial::pasture`
is the first field a **trace** fills. `Trace::add_animals` walks the setup
path's records at `Farms::add_animals+0x92`, `+0x134` and `+0x182`, three
to an animal, and recovers each outcome from the seed the record carries —
`Draw::value`, the Rust half of yesterday's fifteen lines in `report.py`.
run39's five come back `(−143, 40)`, `(−187, −148)`, `(−39, −144)`,
`(−83, −76)`, `(−63, 56)`, all five coins even, and the test asserts the
list rather than a count. run20's trace and the fuzzed map's carry their
own fifteen, so the reader is confirmed on more than one capture; run20's
test now borrows too.

### What landed, and the one line that was missing

The species, the `type_index` that follows from it and reaches the gaia
table, the borrowed positions, and `think_farm_animal`'s `MOVE_TO` at
`add_move_facing_order`'s snap of the unsnapped point — with the angle
taken to the *unsnapped* point, which is why it cannot go through
`add_move_order` (the listing computes `find_angle` at `5d7889` before the
`sarl $4` that snaps).

The line that was missing for an hour: **`movement.speed`**.
`farm_add_animals` had never set it, because until now the animals never
had an order. An animal carrying a `MOVE_TO` it cannot step is worse than
one standing still — it was the *only* reason the first run's player-0
score fell 219 → 217, and with the speed and turn rate in it the pin is
back at 219 untouched. The feared cost of parts (1) and (2) — "the ticks
score 167 → 102" — never arrived either: the walk was what they were
missing, not a second perturbation.

**The score.** run39's early window goes **49/47 → 62/55** of its first 64
frames. Frames 20, 29 and 32 come right. Ticks and orders hold at 167.

### The word held at 19, and the reason is not the pasture's

One draw, on the arrival frame, and two experiments separated the two
residues behind it — which is the whole yield of the session's second half:

- **The animal arrives one frame late.** Give the chicken one more unit of
  speed and frame 19 matches the original draw for draw. 455 units at 25 a
  frame is nineteen steps here and eighteen there; the arrival test is
  `dist ≤ tolerance` and this crate's straight-line goal carries
  `tolerance 0`.
- **An arrival costs two `Animal::do_idle` draws, always on consecutive
  frames** — 19/20, 121/122, 134/135, 245/246, all game, one pair per
  `think_farm_animal` draw nine to twenty-five frames earlier. This crate
  spends one. `Guy::set_anim`'s early return is by **category**, and the
  decompile names the two stashes it returns through — `Guy+0x9e` when the
  body is not at des, `+0xa0` when the guy is already idle — so a second
  draw means a second *non-idle* category on the following frame. The unit
  is still easing onto the order's angle across both, and `Guy::move`'s
  turn arm is unmodelled here. That is items 36 and 37's ground, not this
  mechanic's, and it is now booked with the capture that falsifies it.

Both are in `docs/SYNC.md` §3.11's last section with the evidence, and the
coverage bullet in §6 says which of §3.11's claims a diff backs and which
two rest on the listing alone.

## 2026-08-30 (later, Opus) — items 93, 94 and 37: the arrival pair is read whole, and one third of it lands

Item 93 was booked as "an arrival costs two `Animal::do_idle` draws and this
crate spends one". It is three things, not one, and by the end of the session
all three were read out of the original, all three were measured against
run39, and **one of them landed**. The other two are worth 19 → 69 on the
lower map's word and a perfect 64-of-64 window, and they are blocked by a
different item.

### 1. The turn arm, exactly as booked

`Guy::move@005d9240`'s at-des branch splits on `des_angle == angle`. The
settled half is the arrival stand this crate already had. **The other half was
never modelled**: a body standing on its unit that has not yet come round to
the order's angle is put *back on `CHAR_WALK`* and marked unstopped, every
frame it is still turning — so the next frame's `Animal::do_idle` sees the
walk category and rolls again. Two draws, on consecutive frames. The guard the
arm carries is a sea unit (`type+0x218 == 1`) or a `SPECIAL_ANIM` order,
neither of which is a chicken.

The `Guy+0x9e`/`+0xa0` stashes the last session named here are `hold_attack`
and `queued_attack` and belong to `set_anim`'s **attack** category. They had
nothing to do with it. The type record said so; the surrounding code did not.

### 2. The arrival is a frame early because of where the animal is born

The animal walked 455 units of `y` at 25 a frame — nineteen steps here,
eighteen there — and the last session's experiment was one extra unit of
speed. That is not where the unit comes from: `Unit::update_speed@006055c0`
gives `MOVES × unit_move_speed` = 25 and its four multipliers are all
`is(ARQUEBUSIERS|RIFLEMAN|INFANTRY|MECHINFANTRY)`, so a chicken qualifies for
none. The tolerance really is zero; every writer of `UnitData::tolerance` was
grepped.

**`Unit::init@00612100:69` snaps every unit's starting position** —
`div_3_table[v >> 4] · 0x30 + 0x18`, the same 48-unit snap a move order's
destination takes — before it hands the pair to `Object::init` and to
`set_new_location`. `Farms::add_animals` computes `building ± (rnd % 0x180 −
0xc0)` and `Objects::init_unit` passes it straight through, so run39's first
animal is born at `(40536, 40392)` and not at the `(40552, 40369)` its two
draws name. **432 units, not 455. Eighteen steps.** A pasture animal is the
only object this crate places from a raw, unrounded point — every other one
comes from a dump, which prints where an object *is* rather than where it was
born — so the snap belongs at the pasture and not in `Sim::add_unit`.

### 3. The standing body turns instantly, and that was item 37

With both of the above the pair still came out three draws, not two: the
residual angle is 8.8° and a chicken turns 5° a frame. The answer is an
ordering the document already had and the code did not. `Guy::move` writes
`last_speed = 0` at the **head** of the at-des branch, ahead of the
`turn_towards` at its foot, and `GuyData::turn_speed@005de340:29` answers a
zero `last_speed` on a foot or mounted guy with `0x80000000` — larger than any
turn that can be owed. **A standing body swallows its whole owed turn in one
frame, this frame, however slowly its type turns.** `crates/sim` read
`last_speed` as it stood before the frame, so a slow turner came round at its
rate.

That is queue item 37 whole — run10's AI scout at 96, 362 and 721, "where the
original's body has already snapped onto the order's angle". `docs/MOVEMENT.md`
had listed the suspect as `guy_flags & 2` and called it "worth one grep"; it
was neither. The pseudocode in the same document had `last_speed = 0` above
the turn since 2026-08-27, correctly, and nobody had read the two lines
against the implementation. Item 72's fourth kind of bug in a row.

**Landed alone.** run10's angle disagreements go **9,156 of 33,992 → 8,969 of
35,868** — the three scout rows gone and 1,876 more rows kept in view — and
every other floor holds untouched. The assertion is now `scout.is_empty()`,
and `a_standing_body_takes_its_whole_turn_in_one_frame` in `harness_tests.rs`
was made to fail first (a five-degree turner is the whole of the test; at a
Citizen's forty-five it lands in one frame either way).

### What the pair is worth, and what stops it

| | run39 word | window | run33 word | run10 orders |
|---|---|---|---|---|
| before | 19 | 62/55 | 780 | 776 |
| the snap alone | 20 | 60/53 | — | — |
| the arm alone | 19 | 58/57 | 205 | 212 (ticks) |
| both, with (3) | **69** | **64/64** | 584 | 586 |
| (3) alone — landed | 19 | 62/55 | 780 | 776 |

Either of (1) and (2) alone is worse than neither, and together they cost the
*other* map more than they win on this one. The arm reads `des_angle != angle`
on **every** standing unit, and Great Lakes' facings are not the original's
yet: that is queue item 36, the farmers, 8,866 of the 8,969 rows. The other
half of the same block is named now — `Guy::do_turn@005d97a0:15` overrides the
arm's walk with `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` whenever the guy's piece has
a turn animation (`guy_flags & 8`, `Guy::init_real@005db6b0:179`), a category
this crate does not model, and a guy on a turn animation spends **no** arrival
draw where one on the walk does.

So item 36 stops being a residue and becomes the headline's dependency, and
item 93 becomes a dozen lines waiting on it. Both halves are written into
`docs/SYNC.md` §3.11's last section with the listing that settles each.

### The listing settled two things the decompiler could not

`Animal::think_farm_animal`'s `find_angle(unaff_EDI, unaff_ESI)` — the order's
own facing — is taken to the **unsnapped** point, and `5d7887`–`5d78a5` shows
it: `edi`/`esi` hold `(3(T + m) + C)·0x40 + 0x60` and the `sar $4` that snaps
them comes ten instructions *after* the call. An experiment that took the
angle to the snapped destination also produced 64/64, and the listing is what
says it was a second compensating error rather than the answer. And
`Unit::init`'s snap is two lines of the same function's prologue, which no
amount of reading its callers would have found.

## 2026-08-30 (later, Opus) — items 36 and 93: East Indies' word 19 → 69, and the farmers were never seventeen callers

**The headline moved: East Indies' word parts at 19 → 69, and its first
sixty-four frames are 64 of 64 on the count and 64 draw for draw.** Great
Lakes holds at 780 and rises to 954/843 of 1,850; run10 holds at ticks 572,
orders 776. run10's angle sub-score, item 36's own, goes **8,969 of 33,992 →
1,227 of 35,868** with item 36 and settles at **1,910 of 35,984** once item 93
lands on top of it. Tree green: 642 sim, 145 rondata.

### Item 36 as booked, and what it was

"`Unit::set_angle`'s seventeen other callers." It had carried that name since
2026-08-27, on the strength of a residue that looked like it: farmers standing
exactly where the original stands them and pointing somewhere else, 8,866 of
the 8,969 rows, and a list of seventeen call sites the simulation does not
make.

It was two predicates in code this crate already had. Neither is a caller.

**One.** `Unit::add_move_order@00616ed0` takes the angle to the point it was
handed, not to the snap of it. The listing is unambiguous — `ecx = x −
(this->field_0x10 ^ 0x63637)`, `edx = y − (field_0x14 ^ 0x63637)`, then
`call find_angle`, and only after that the two `sar $4`s that index
`div_3_table` for the coordinates it pushes. This crate had `let dest =
snapped(to)` one line above the `find_angle` it fed. It matters because
`Unit::do_gather`'s wheat branch picks `(corner + rnd % 4) · 0xc0 + 0x60` —
the centre of a **192**-unit tile, which is never the centre of a 48-unit
cell — so every farm walk in the game ends facing a bearing 24 units an axis
off the one this crate computed.

run10's `0/3` walks from `(2712, 32136)` to the cell whose snapped centre is
`(2808, 31992)`. The dump's `MOVEORDER angle` is `0x10889...` — 23.25° — and
`find_angle` over the raw `(2784, 31968)` gives it exactly, where the snapped
pair gives 33.72°. `0/4`'s `−124.74°` is `find_angle(−312, 216)` on the same
rule. Two exact matches with no free parameters.

**Two.** `move_step@005faf30` ends a final leg two ways and the gather clause
belongs to only one of them. The Manhattan snap (`manh <= step`) faces the
order's angle when the move was the only order **or** the action beneath is a
`GATHER` (`005fb562`); the partial step — the unit walked its whole step and
happened to land on the destination — faces it only when the move was the
only order (`005fb4a8`). `docs/ORDERS.md` §4.5 has had both since it was
written. The code applied the gather clause to both arms.

The two arms are six frames apart in one capture. `0/3` arrives on 110 with
`manh 7` at speed 25 — the snap, and its heading becomes the order's 23.25°.
`1/3` arrives on 116 from `(41789, 17040)` with `manh 29`: a full 25-unit
step whose sine and cosine are −5 and 24 lands it exactly, so it is the
partial arm, and its heading stays `find_angle(−5, −24) = −11.42°` while its
order's angle is −18.58°. A farmer's last leg is routinely the second kind,
which is why every farmer was in the residue and nothing else was.

### What found it

Not a reading of the mechanic. One row of the residue — a farmer standing
still and facing wrong on frame 110 — and then the original's own `UNITDATA`
record for the twenty frames around it, printed as one line a frame. The
`MOVEORDER angle` sitting in that record next to a position the dump also
prints is the whole derivation; the listing only confirmed which of the two
candidate points it was taken to.

`docs/SYNC.md` §3.11 had already found this rule for the pasture two hours
earlier and implemented it as a special case, with a note saying the walk
"cannot go through `Sim::add_move_order`". It can now. The special case was
the bug report, and nobody read it as one.

### Item 93, which item 36 existed to unblock

`docs/SYNC.md` §3.11's pair, unchanged from how it was written and measured
yesterday: `Unit::init@00612100:69`'s snap on a pasture animal's birth point
(`farms.rs`), and `Guy::move`'s turn arm putting a standing body still owed a
turn back on `CHAR_WALK` and unstopping it (`anim.rs`'s `guys_follow`). A
dozen lines, and the numbers came out exactly as the table in the previous
entry predicted for "both, with (3)" — except for the column that was the
whole problem:

| | run39 word | window | run33 word | run33 totals |
|---|---|---|---|---|
| before item 36 | 19 | 62/55 | 780 | 951/838 |
| item 36 alone | 19 | 62/55 | 780 | 951/838 |
| + item 93 | **69** | **64/64** | **780** | **954/843** |

Predicted for item 93 without item 36: run33's word 780 → 584 and run10's
orders 776 → 586. With the farmers' facings right it costs nothing at all,
which is what a dependency looks like when it is real.

### The named suspect was a dead end, and that is now an assertion

Item 36's second half was `Guy::do_turn@005d97a0:15` — with `guy_flags & 8`
the turn overrides the arm's walk with `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT`, and
a guy on a turn animation spends no arrival draw. `Guy::init_real@005db6b0:179`
sets that bit only for a guy whose piece names a turn animation. **273 of the
install's 1,359 unit pieces do, and none of the eight a `DUMP_ALL` run's guys
carry** — 0, 19, 352, 371, 6336, 6688, 12691, 13043, the two scouts, their
dogs and the six citizens of a Nubian and a British start; gaia's
60063–60074 are not `<UNIT>` entries at all. So the override cannot fire on
any capture there is. It is asserted in
`the_install_s_piece_lengths_match_the_dumps`, beside the `GROUP_IDLE2`
finding it rhymes with, and left unmodelled with the check that would make it
matter named: a capture with a vehicle or a ship turning in place.

That is the second time a whole booked mechanic has been closed by reading
the install's own art tables rather than the executable.

### The checks

- `only_the_snap_arm_s_arrival_faces_the_order_s_angle_under_a_gather`
  (`cities_tests.rs`): the same two geometries as run10's frames 110 and 116,
  plus the lone-move case that pins the clause they sit beside. Red when the
  gather clause is given to both arms.
- `a_standing_body_still_owed_a_turn_walks_in_place_and_draws_again`
  (`anim.rs`): the arm, the settled-facing arrival beside it, and the sea
  guard. Red when the guard is forced on.
- `an_animal_is_born_on_the_snap_of_its_two_draws` (`farms.rs`): red when the
  animal is placed on the raw point.
- `the_three_queue_modes_append_rotate_and_replace` now asserts the order's
  angle **is** the bearing to the caller's point and **is not** the bearing to
  the snapped destination — the two differ by 1.7° in that fixture, and the
  old assertion was the wrong one of the pair.
- `move_step` reports which arm it arrived on (`Step::snapped`), pinned both
  ways in `the_clamps_land_the_unit_exactly_and_that_counts_as_arrival`.

### Paperwork

`docs/SYNC.md` §3.12 is new and carries both predicates with the evidence;
§3.11's last section is rewritten from "not landed" to landed, and its
coverage bullet's reading-only `find_angle` claim is struck — the argument is
diff-backed now, on the farmers rather than on the animal. `docs/ORDERS.md`
§4.3 says what `add_move_order` computes the angle *from*, which it never did;
§4.5 needed no correction and is marked diff-backed, and paid for the new
bytes by losing a struck question it had already answered. `docs/ANIM.md` §4.6
carries the arm and the turn-animation finding. `docs/MOVEMENT.md`'s checks
section strikes "it is `Unit::set_angle`'s other seventeen callers".

### The lesson, and it is item 72's a fifth time

`docs/ORDERS.md` §4.5 stated the two arrival arms correctly and the code
merged them. §4.3 said `add_move_order` "is a thin wrapper that snaps and
computes the angle" and never said what it computes it *from* — a sentence
that is not wrong, and that a reader checking the code against it would pass.
Neither was found by reading the mechanic again. Both were found by printing
the original's own record around one bad row.

And a booked item's *name* is a hypothesis. "Seventeen callers" survived three
sessions and two widenings because every new measurement was reported against
it — "the same seventeen callers again" appears four times in
`rondata::diff`'s own history comments. The residue was never once opened.

## 2026-08-30 (later, Opus) — item 95: East Indies' word 69 → 91, and an animal in a hurry

One item, one session, and the headline moved on the map that is the
headline. The queue booked item 95 as "East Indies' blocked stand, a frame
late": on run39's frame 69 the original spends
`Guy::set_anim+0x97a < Unit::move_step+0x823` — `sim::anim::SITE_BLOCKED`,
`docs/COLLISION.md` §5 — and this crate spent it on 70. The word now parts at
**91**, and the early window still holds at 64 of 64 on the count and 64 draw
for draw.

### What it was

Not the collision block. The collision block is right, and reading it again
would have found nothing. The unit is gaia's `8/2`, blocked by its herd-mate
`8/1`; both sides start the walk on the same frame from `(28776, 24360)` with
the same goal, `(28968, 23976)`, and both stop at exactly `(28856, 24197)`,
because the stop is positional — the next step's point is where `8/1` stands.
The original covers that ground in **nine** steps. This crate took **ten**.

Solving each dumped step against `find_angle` and the sine table gives one
integer per frame with no slack in it: the original walks at **28, 28, 19,
18…** where this crate walks at 19 throughout. The two long steps gain
exactly one step of ground, which is the frame.

`19 × 3 / 2 = 28`. **`AnimalData::get_speed@005d8380`** is where it comes
from: it occupies slot `+0x17c` on `Animal`'s and `AnimalData`'s vtables —
the virtual `Unit::do_move` and `Unit::find_path` both take the step length
from — and it **does not call `UnitData::get_speed` at all**. It takes
`UnitData::speed`, and then, on a land or sea animal whose current order
`is_move`, measures `vector_dist` from the animal to that order's **goal**
and multiplies by `3/2` when it is more than `0x180`. Then a floor of 3. An
air animal returns before both.

So none of `docs/MOVEMENT.md`'s "speed pipeline" layer 3 reaches an animal:
not the order scale, not the `unit_masks & 0x10` halving, not the `0x800`
tile, not the group cap. That whole section had been read and written and
was simply about a different function.

`vector_dist` to the goal on the three frames that matter is **432, 404,
376**. The last is the first under `0x181`, and it is the frame the step
drops to 19.

### How it was found, and the false trail on the way

By taking one row of the residue and printing the original's own record for
the twenty frames around it — the method §3.12 wrote down two items ago,
used a second time.

The false trail is worth recording because it was convincing. `z_internal`
goes 39 → 35 on exactly the frame the speed changes, and the animal crosses
a tile boundary on that frame; `UnitData::get_speed` has a **tile flag
`0x800` that halves the speed**, gated on `z_internal <= 0`. Three facts
lining up, and all three coincidence: the halving is a halving, and 28 → 19
is not one. What settled it was arithmetic rather than another reading —
28/19 is 3/2, and 3/2 is a ratio a function has to be looking for.

The dump's own `myspeed` closed it: run39's `8/2` prints **19** on every one
of the nine frames, including the two whose step is 28 long. So the `3/2` is
applied strictly downstream of the cached speed, and the animal's base is not
some other number.

### The vtable slots, and the PDB trick that named them

`AnimalData::get_speed` calls two virtuals on the order, `+0x14` and `+0xb8`,
and the export names them `StrafeOrder::is_air` and `Window::get_button` —
COMDAT-folded stubs, both wrong. The PDB's `LF_ONEMETHOD` records carry each
method's real `vftable offset`, and on `UnitOrder` **20 is `is_move`** and
**184 is `get_move_order`**. That is the trick the memory index has been
carrying since the global-table work, used here for the first time on a
predicate rather than a data table, and it is what makes the `MoveOrder
+0x4/+0x8` reading — the order's *goal*, not `dest_x/dest_y`, the
pathfinder's current leg — a fact rather than a guess.

### The check, made to fail three ways

`an_animal_more_than_0x180_from_its_order_hurries_by_three_halves`
(`rondata::diff`) does not check nine frames. It re-derives `move_step`'s
whole step from the `myspeed`, `orders_x/orders_y` and positions the dump
prints, for **every gaia unit-frame on which an animal moved** in run39 *and*
run33: **264 steps, 39 of them beyond `0x180`, and 264 predicted exactly**.
Great Lakes' herd carries `myspeed 11` where East Indies' carries 19, so the
ratio is exercised against two bases.

Then it was made to fail. Drop the `3/2` and exactly the 39 far steps break;
move the threshold to `0x200` and 27 break; make the ratio `4/3` and all 39
break. The test asserts the triple `(264, 39, 39)`, so the absence of the
rule is pinned as well as its presence.

`an_animal_beyond_0x180_of_its_order_walks_at_three_halves` (`gaia.rs`) is
the unit-level twin: the base, the hurry, the exact `0x180` that is *not*
more than `0x180`, a player's unit taking none of it, and the air arm
returning before the floor as well as before the hurry — which is the only
way to see those two apart.

### What it cost

East Indies' word **69 → 91**. Great Lakes untouched where it can be seen:
`first_count` holds at 780, `first_part` at 99, run10's ticks and orders at
572/776, and its collision rows *rise* 93,357 → **93,398**.

Two floors fell and both were re-pinned with the reason. run33's weak totals
954/843 → **938/841** — and it can be said exactly where, which is the part
worth keeping: with the hurry in and out, that capture's per-frame draw
counts are **identical up to frame 1108**, 328 frames past its own parting at
780 and past every one of the twelve units' first divergence. run39's player
0 goes 219 → **217**: by 217 the two sides have been on different mid-frame
draw orders for a hundred and twenty frames, and which of three citizens
parts first there is not a fact about this simulation. run10's angle rows
35,984 → 35,942, the coverage effect against the headline for the seventh
time.

### What the diff cannot see, and the item it belongs to

The harness **reseats gaia's animals from the dump on every traced frame**
(`Sim::reseat_animal`, §4.2), so `run_traced` reported **no** divergence for
`8/2` on any of the ten frames its position was wrong. A whole class of
error is invisible to the score and visible only in the draw stream, which
is queue item 45 and is now worth more than it looked: the animal that was
wrong here was wrong for ten frames and cost the word twenty-two.

### Where the word goes next

Frame **91**, and it is the same site again: gaia `8/0` takes a wander order
to `(28968, 23976)`, steps once, and is blocked. This crate sends it to
`(28776, 24120)` — a point `Animal::do_idle`'s **near** branch can reach and
the original's cannot, since the near offsets cap at `4 × 0x30` an axis. So
the two took different branches on frame 89, and ours never meets `8/1`.

### Paperwork

`docs/MOVEMENT.md` has a new section, "The animal's own `get_speed`",
immediately after the three-layer pipeline it corrects the scope of.
`docs/SYNC.md` §3.13 carries the story and the numbers, and its coverage
section names what is diff-backed (the threshold, the ratio, the base) and
what is reading-only (that `+0x14` is `is_move`, and the air arm's missing
floor).

## 2026-08-30 (later, Opus) — item 95's successor: East Indies' word 91 → 201, and a blocked animal gives up

The queue booked this as "gaia `8/0`'s wander branch": on run39's frame 89
the original sends `8/0` to `(28968, 23976)` and this crate sends it to
`(28776, 24120)`, which is exactly `4 × 0x30` on each axis and therefore
looks like `Animal::do_idle`'s **near** arm. It is not. The near arm was
never taken. Herd 0's centre on that frame is `(28800, 23936)` —
`((wx + 2cx) · 0x300 + 0x480) / 3` with `wy` already walked from 31 to 30 on
frame 0 — and `8/0` stands `vector_dist 413` from it, over the `0x180` the
near arm needs. Both sides take the far arm; the near-arm arithmetic was a
coincidence of two offsets.

### What the record actually said

The way in was to stop looking at `8/0` and print every unit within 700 of
the contested point for the twenty frames around it. `8/2` — §3.13's animal,
blocked by `8/1` on frame 69 — is at `(28856, 24197)` in the original on
every frame from 70 to the end of the capture, and in this crate it had
**walked on**: a side-step to `(28872, 24216)`, a snap onto its own cell
centre, a fresh path through `(28824, 24168)` and `(28824, 24024)`, and by
frame 88 it is sitting at `(28872, 23976)` — ninety-six units from the spot
`8/0` was about to be given. `find_nearby_spot` refused that candidate for
us and took the next one.

So the divergence at 91 was not a branch and not a wander. It was one
animal's collision, twenty frames earlier, and this crate was giving it the
whole of `docs/COLLISION.md` §6.

### The branch nobody had read, and why

`Unit::resolve_unit_collision@005f9d30`'s **first statement**, before the
attack arm, before the flat-gather fence, before everything the document
had:

```
if ((**(code **)(*(int *)this + 0x30))() != 0) {
    this->unit_masks &= ~0x4000000;
    this->path.length = 0;
    close_orders(this, 0);  clear_partial_path(this);  update_action(this);
    return 1;
}
```

— the same five lines `add_move_facing_order@005e55c0:58` runs for a
`QUEUE_NEW` order. The whole order list and the path go, and none of the six
steps below runs.

Slot `+0x30` is where the reading had stopped every previous time, because
the export cannot name it: both overrides are trivial and COMDAT-folded, so
`vtables.txt` prints `Buffer::is_pending_load` for `Animal` and
`Window::get_button` for `Unit`. Those two stubs are `return 1` and
`return 0` — the predicate is right there in the fold — but nothing says so.
The PDB's `LF_ONEMETHOD` list does: `SubObjectData::is_animal`, **vftable
offset 48**, between `is_wonder` and `get_gpiece`. That is the third slot
this session's family of items has had to take from the type stream rather
than the map, after `is_move` at 20 and `get_move_order` at 184.

`crates/sim/src/collide.rs` now opens `resolve_unit_collision` with
`if self.units[u].is_gaia() { self.clear_orders(u); return; }`. The
predicate is the owner rather than the class, which is a stated seam: every
`ANIMALDATA` record in run12 (360) and run20 (936) carries `who 8`, and the
pasture's carry `who 9`.

### What it cost

East Indies' word **91 → 201**, the early window holding at 64 of 64 on the
count and 64 draw for draw. `8/2` now stands at `(28856, 24197)` from frame
70 to the end, frame for frame with the original, and `8/0` gets
`(28968, 23976)` on 89.

Great Lakes holds where it can be seen: run33's `first_count` at 780 and
`first_part` at 99, run10's ticks and orders at 572/776, both players' first
divergence at 802 and 573, and **twelve of run10's thirteen units unchanged
to the frame**. The thirteenth, `1/9`, parts at 1320 rather than 1377, and
that one unit is the whole of the fall in the two coverage counts: collision
rows 93,398 → **91,210**, angle rows 35,942 → **35,188**. run33's weak
totals go 938/841 → **943/827**, the count up and the order down, and the
same argument as §3.13's holds with the same shape — with the rule in and
out, that capture's per-frame draws are **identical, count and sequence
both, up to frame 1128**, 348 frames past its own parting.

### The checks

Two, and both were made to fail first.

`rondata::diff::a_blocked_animal_drops_its_walk_where_it_stands` is the
oracle half and it needed no new capture: over run39 and run33, every animal
walk that **ends short of its goal** — the order stops naming a point and
starts naming the animal, while the animal is not standing on the goal —
must leave the position *unchanged* across that frame. Sixteen of those,
twelve on East Indies and four on Great Lakes, against seventeen that
arrive; the pair `(17, 16)` is asserted so a parse that stopped seeing
orders fails rather than passes with nothing to check. §6 step 6's
cell-centre snap is exactly what would break it, and this crate broke it on
all sixteen.

`sim::collide::an_animal_drops_its_walk_where_it_stands_and_takes_no_step`
is the twin of the existing `the_recovery_snaps_to_the_cell_centre_and_
paths_around`: the same walk, the same blocker, the same frame, and gaia's
walker stops dead with an empty order list and an empty path while the
player's snaps and paths around. With the branch commented out it fails on
the snap, which is how it was landed.

### Where the word goes next

Frame **201**, and it is the other map's item 84 arriving here: this crate
spends 38 draws where the original spends 36, and the first difference is
ours opening `GameAccess::rnd+0x20 < Unit::do_job+0x67` — a citizen on a job
— where the original opens `Guy::set_anim+0x97a < Guy::inc_time+0x271`, the
standing-turn residue run33 already shows on 99, 205 and 319. Two known
items, both now on both maps.

### Paperwork

`docs/COLLISION.md` §6 has a new **step 0** at the top, §7 says the
predicate is read off the owner and why, §8 lists the new diff, and §9 gains
the `unit_masks & 0x4000000` the branch clears — whose two readers,
`Unit::work` and `Unit::think`, an animal never runs. `docs/SYNC.md` §3.14
carries the story; §3.13's closing paragraph, which named the near branch,
is struck through in place and points at it.

---

## 2026-08-30 (later, Opus) — item 97: East Indies' word 201 → 219, and the branch the document had all along

The residue at frame 201 was two draws, and the whole of it was a branch
`docs/ORDERS.md` §6.5 had written down weeks ago and `orders.rs::do_farm`
had never had.

### What the frame said

The trace's frame 201 on run39 is 36 game draws: 31 `Guy::inc_time` wraps
and five `Farms::inc_time` chances. This crate spent 38 — the same 36, with
two `GameAccess::rnd+0x20 < Unit::do_job+0x67` in front of them. So the
queue's framing ("ours opens on a citizen's job draw where the original
opens on the standing-turn residue") was an artefact of the insertion:
nothing was reordered, two draws were *added*.

The pair is a farmer re-picking its cell, and printing every `do_farm`
re-target the crate makes named the unit at once: the AI's `1/3`, at
building `1/2002`, whose `FarmStruct::farm_type` is **1** — the pasture. It
had been sowing its centre cell since frame 1 and the cell ripened under it
on 201.

### Why 201 and not 101

Every other farmer re-picked on **101**, which is the clock §6.5 already
describes: `Farms::inc_time` adds `0.005f` to a growing cell and the
farmer's own `Farms::grow` adds a second, so a sown cell crosses `1.0f` on
the 201st add, a hundred frames in. A pasture is the one farm `inc_time`
**skips**, so the herder's own add was the only one and the crossing came
on the two hundredth frame. The wrong branch and the missing clock together
made a residue that looked like a citizen wandering and was neither.

### The arm, from the listing

`Unit::do_gather@005ef2a0` asks `FarmsData::get_farm_type` — which is
`farm_type & 1`, so an ambience-carrying pasture still answers 1 — and
branches at `005efd77` before the cell arithmetic:

```
005efd80  set_anim(CHAR_SOW, 0, 1)
005efd8c  edx = o*7 + game->frame + who;  edx &= 0x800000ff (signed % 256)
005efdbb  jne  → return
005efdd8  rnd(x_size / 2)          ; ecx = 0x234 >> 1
005efde5  rnd(y_size / 2)          ; ecx = 0x238 >> 1
005efe04  move((cx + 1 + rx)*0xc0 + 0x60, (cy + 1 + ry)*0xc0 + 0x60)
```

`GameAccess::rnd@0043cca0(n)` takes its modulus in `ecx`, which is why the
decompiler lost it in all four places it appears here. The crop's two loads
at `005efff9`/`005f0004` are the type's `x_size` and `y_size` whole — 4 and
4 for a farm, which is the number run13 had already *measured* off the
dump's goals in August — and the pasture's halve each. So the herder goes
to one of the inner four tiles and the farmer to any of the sixteen, and
the `+ 1` is what centres the smaller square.

### What it cost

East Indies' word **201 → 219**, and its *sequence* with it: every draw of
every frame before 219 is now the original's, in its order, which is the
first time this map has had a whole-run draw-for-draw stretch rather than a
64-frame window. The score holds at 167/167 with player 0 at 217 and player
1 at 168, and the herder `1/3`'s own position now tracks the original's to
frame **492**.

Great Lakes is untouched to the byte — run33's AI built seven farms and no
pasture: `first_count` 780, `first_part` 99, run10's ticks and orders
572/776, all thirteen units' divergences unchanged, collision rows 91,210
and angle rows 35,188 equal.

### The check

`rondata::diff::a_pasture_herder_walks_only_on_its_own_256_frame_phase`,
made to fail three ways. Its oracle half needed no new capture: run39's
dump moves the herder on **42** frames of 1,850, in **four** runs, and each
run's first frame is the one after a phase frame — the order is issued on
the phase, the step lands next. Three of the seven phase frames move it
nowhere at all, because the inner square holds four tiles and the roll may
name the one it stands on. The trace's half is that all **seven** phase
frames — 234, 490, 746, 1002, 1258, 1514, 1770 — spend the pair. The port's
half is that this crate walks the original's first run frame for frame,
sim-frames 235 to 242, and starts no walk off a phase over the whole
capture. Removing the arm makes it walk from 202; halving the modulus makes
it walk from 107; giving it the crop's own span makes its first walk six
frames too long.

### Where the word goes next

Frame **219**, and it is not a farm. The original spends that frame's
citizen re-target *before* the frame's road search and this crate after it,
and the two searches cost 129 nodes against 152 — the road residue of
`docs/ROADS.md` §7.1 and queue item 60, arriving as the next thing in the
way. Great Lakes' own 780 was looked at while this was open and is a
different animal: no pasture there, but a **seventh farm ticking where the
original ticks six** from frame 781 on, four frames after a city is
founded. Item 84 keeps its number and its own story.

### Paperwork

`docs/SYNC.md` gains §3.15 and a coverage row. Two sections were split
rather than grown, which is what `docs_guard` asks for: `ORDERS.md`'s §6
was 21 KB and its subsections 6.5, 6.6 and 6.7 are now `## ` headings of
their own — the numbers the code cites are untouched, and its `OVER` row is
gone because all four halves are under the ceiling. `SYNC.md`'s §6 had been
carrying the coverage list inside "what is not established"; that list is
now `## 7`, which is also where CLAUDE.md says a coverage section belongs.
`FARM_SPAN` is gone from `orders.rs`: the modulus is read from the type,
where the original reads it.

## 2026-08-30 (later, Opus) — item 60: East Indies' word 219 → 274, and the frame was two loops

Item 60 was booked as a road search: the original spends frame 219's citizen
re-target *before* the frame's road costs and this crate after it, and the two
searches cost **129 nodes against 152**. The queue called the ordering the
cheaper half. It was the whole of it, and it was not about roads.

### What it was

`Objects::process_all@0065dce0` is **two loops**. The first walks the ten owner
slots rotated — `(frame + i) % 10`, each owner's objects `0..unit_mark` — and
the second walks the ten leaders **unrotated**, each one's `2000..build_mark`
(the buildings) and then `3000..wall_mark` (the walls). `Sim::tick` ran the
buildings first. `docs/SYNC.md` §3.2 has said the opposite since the day it was
written, in a sentence that ended "which is a known divergence this document
does not close" — so nothing has read it against the code in five weeks.

That is the **seventh** time a document and its code have disagreed and the
second running where the document was the one that was right. Item 72 keeps
earning its place.

With the loops in their own order frame 219 is draw for draw, and the search
that had looked 152 nodes costs **129** — the original's exactly. The count was
never a residue: it was the same search reading a world the frame's units had
not yet touched. `docs/ROADS.md` §7.1's own two counts, run32's 1,046 and
1,460, are untouched by this and stay open.

### The two compensations it uncovered

Swapping the loops alone made things worse in a way worth recording, because
both errors it exposed had been *cancelling* the loop order exactly.

**Great Lakes' word fell 780 → 99 and run10's ticks 572 → 103.** Frame 99 is
where run33's AI trains its citizen: `Guy::init_real+0x52`, then one more
draw. This crate skipped a newborn in `guys_inc_time` (`born == frame`) and
spent the second draw at `Unit::do_idle+0x7d` in the same frame's unit loop;
the original spends it at `Guy::set_anim+0x97a < Guy::inc_time+0x271`. That
disagreement is the "standing swap" the queue has carried as item 62 since the
first trace, and it was never an attribution question. A trained unit is
created in the *second* loop, so the original never reaches it in that frame's
unit loop at all, and `Objects::inc_time` is what finds its
`cur_time 0, end_time 0` clock and wraps it. run13's `1/6` ends sim-frame 99 at
`0/232, last −1` — the state the wrap's `set_anim` leaves, not the one
`Guy::init_real` does. The dump had been saying so for weeks beside a document
sentence that read it the other way.

With the skip gone the word ran to **122**, where the citizen's walk hit a
collision a frame late. `Unit::think_peasant@005f5760:16` reads the owner's
idle-citizen option only when `unit_masks & 0x40000` is *clear* and takes
**`T = 1`** when it is set: an AI-driven worker finds a job on the first frame
it is idle, a human's on the second. `docs/ORDERS.md` §5.9's pseudocode has
carried `T = AI ? 1 : …` from its first writing — the eighth instance of the
same lesson in one session's work — and the crate used the option for both. The
extra unit-loop visit the wrong loop order gave a newborn was worth exactly the
frame that hid it: the citizen came out on 99, was first visited on 100, took
its gather order on 101 rather than 100, and the collision that ends its walk
landed on 123 where the original has 122. The same line's other half — an AI
worker searches with no range limit — landed with it and moves no traced frame.

### What it moved

- East Indies' word **219 → 274**, its sequence with it.
- Great Lakes' word holds at **780** and its **sequence runs 99 → 576**: the
  first 576 frames of the long capture are the original's draws in the
  original's order. Weak totals 943/827 → **943/830**.
- run14's whole traced window is **284 of 284, draw for draw**, where it had
  stood at 282 since item 66. Both of the two frames that were not — 99 and
  100 — were this.
- run10's ticks and orders hold at 572/776 with both players' first divergence
  at 802 and 573; East Indies' game score holds at 167/167 with player 0 at 217
  and player 1 at 168.

### The check

Every floor moved is pinned: run39's word and sequence at 274, run33's sequence
at 576 and its totals at 943/830, run14's `first_part` and `matched` at 284.
Made to fail three ways, and all three were seen on the way in rather than
staged: the loops swapped with the newborn still skipped (780 → 99, ticks
572 → 103), the wrap restored with the AI's threshold still 2 (→ 122), and the
threshold alone, which is what run14's frames 99 and 100 had always been.

`Unit::born` is gone from the sim: nothing read it once the skip went.

### Where the word goes next

Frame **274**, and it is a unit this crate does not make. The original spends
`Guy::init_real+0x52` and the newborn's wrap there — two draws for a guy — and
this crate creates none. It is the training clock, not an order or an animal:
the next item is which building finishes what, and when.


## 2026-08-30 (later, Opus) — item 98: East Indies' word 274 → 413, and the AI stopped training citizens on its second call

Item 98 was one guy. The original's frame 274 opens with `Guy::init_real+0x52`
and the newborn's wrap, two draws for a unit this crate never created, and the
brief was `docs/PRODUCTION.md`'s clock against run39's own `BUILDS`. The clock
was fine. Nothing was ever queued.

### Following the queue back

Run39's `BUILDQUEUE` says exactly where to look: the AI's city hall `1/2000`
starts a citizen on frame **176** and its `job_counter` climbs 100 a frame to
`train_time` 9,750, capping there on 273 and handing the guy over on 274. This
crate's only live queue in that game was the library's research. So the
question was not the counter but the order to start one.

Frame 176 is the AI's script call. `economic.bhs`'s every-call line is
`train_unit_with_need(who, needed_citizens, "Citizen")`, and a host trace
showed it being called with **`needed_citizens = 0`** — where step 8, on game
frame 1, sets it to 9. The static was being written back correctly (a trace of
the store showed `economic#9 0 → 9` at the end of frame 1) and read back as
zero on the next call.

### What it was

`VirtualMachine::get_value@004d1010` and `set_value@009e07b0` decide where a
variable lives from two bits of the operand: `0x20000000` is the constant pool,
`0x40000000` is `Script::static_vars`, and neither is the call's own frame. A
`static` is **one `ScriptType *` on the `Script` object** — it never touches a
frame, and it is live from a call's first instruction.

This crate gave it a frame slot and mirrored the slot back into the store after
every expression statement — over *all* of the function's statics, including
the ones whose declaration the call had not yet reached. `economic.bhs` has
three expression statements above its `static` block. So on the **second** call
and every one after, never the first, the AI's whole opening state was zeroed:
`needed_citizens`, `prev_step`, `timer_started`, `fishermen_total`,
`wood_camp`, `max_woodcutters`, `build_merchant` — and `needed_techs`, which is
initialised from a host call and stayed zero, because "once ever" is keyed on
the store holding nothing and the store held a zero.

The AI trained no citizen from frame 176 to the end of the game, on either map.

`docs/AI.md` §17. The second reading left the BHS language deliberately unread
(§16's last paragraph) because "run7's opening reproduces frame for frame". It
does. The second call does not, and nothing had ever looked at a second call.

### What it moved

- **East Indies' word and sequence 274 → 413.** What parts there is something
  else: the original spends two `Unit::do_idle+0x7d` idle anims and a nine-draw
  `Unit::think_scout` scan that this crate does not.
- **Great Lakes' weak totals 943/830 → 943/851**; its word holds at 780 and its
  sequence at 576.
- **run10's roster is the original's both ways for the first time.** `1/10` —
  the tenth citizen, trained at 1772, that no session had reached — arrives, so
  268 missing + 0 extra becomes **0 + 0**. Its coverage follows: `mylos`
  26,433 → 26,701 unit-frames, the collision block 91,210 → 92,766, the angles
  35,188 → 35,742. The headline 572/776 and both first divergences (802, 573)
  are unmoved to the frame.
- run39's own game score (167/167, 217/168) and run14's whole 284-frame window
  are unmoved.

The guard is `bhs.rs`'s
`a_statement_above_the_declarations_does_not_wipe_the_statics`, written against
the old seeding first: the second call returns 1 where it must return 2, and
only the second.

### The widening, and what it found

`BUILDQUEUE` and `BuildData::queued` were parsed by nothing, and the whole of
`docs/PRODUCTION.md` — a document read end to end — rested on **one**
hand-transcribed frame, run7's `[25, 26, 27]` citizen ramp. The dump has
written `queue_size` and every capacity slot's `type`, `job_counter`,
`cost[0..2]` and `good[0..2]` on every frame all along.

Parsed and diffed whole:
`run39_s_build_queues_are_the_original_s_clock` compares both players' every
building over run39's 1,851 frames — **33,631 fields, of which 21 disagree**.
That is four times further than the sync word reaches on that map, and it makes
the accelerator, the compare-before-add, the cap at the target, the charge, the
per-entry ramp and the handover diff-backed rather than read.

It also cost a wrong turn worth recording. Read with a python parser that kept
only the first value of each repeated key, the record looked like *one* queued
citizen where this crate had three, and half an hour went into "the AI has more
food than the original's". The dump writes all twenty slots one after another
under the same key; `queued 3` was three lines further up the same block. The
lesson is the export's own: read the record's shape before believing a summary
of it.

The 21 that disagree are one residue, and it sits **upstream of the word's
413**: `TECH_SCIENCE_SPEEDUP` is loaded into `Tuning` at 10 and read by
nothing, so the AI's library takes 20,000 hundredths where the original takes
18,000 and its research lands on 402 rather than 382. That is
`calc_science_discount@006da630`'s time side; its purchase side is queue item
86, and the two are one session's work.


## 2026-08-30 (later, Opus) — item 86: the science discount, both sides, and run39's queue record 21 → 1

`TECH_SCIENCE_SPEEDUP` shipped in `Tuning` at 10 and was read by nothing. Item
86 was booked off the widening of the previous session: of the 33,631 queue
fields `run39_s_build_queues_are_the_original_s_clock` compares, **21
disagreed**, and twenty of them were twenty consecutive frames of
`1/2005: queued ours 1 theirs 0` — the AI's library holding an entry the
original had already finished.

### What the two sides actually are

`calc_science_discount@006da630` was booked as serving both call sites. It does
not. The **price** side is that function; the **time** side is an inline block
inside `ObjectData::train_time@006508c0`, and reading it as the same function
would have got three things wrong:

- the constant is `TECH_SCIENCE_SPEEDUP`, a *second* `Tuning` entry that merely
  ships at the same ten;
- the level is the tech's `AGE` column **raw** — none of the price side's
  plus-one for a tech that is neither an age nor an epoch;
- it is gated on `level < epoch[3]`, so falling behind costs nothing in time,
  where the price side turns the same distance into a surcharge out of the same
  expression.

And the level for a **non-tech** research job — a unit or building whose
availability bit is clear, which reaches the same research block — is not the
type's own, since only a tech record has an `AGE`, but its first
prerequisite's: `TypeData +0x30` is `preq[0]`, `TechTypeData +0x1c8` is the
column. That arm is implemented and unexercised; no capture queues one.

The arithmetic is a subtraction of a truncated term, not a `(100 - pct)` scale.
The listing spells it as the `-0x51eb851f` magic multiply **added** to the
time, which `production::neg_hundredth` had already named a session ago with a
comment pointing at exactly this step. `Adjust::Off` is that shape; `Off(10)`
and `Scale(90)` differ on 7 hundredths, and the test says so.

### What the record says

The AI queues Written Word and City State at frame 2 and both are
`JOB_TIME 200`, so both start at 20,000 hundredths. Written Word *is* the
Science epoch, so while it is being researched `epoch[3]` is zero against its
own level of zero, the strict gate is false, and it takes the full 20,000 and
lands on 201. From 202 the player is a level ahead of City State's own zero, so
City State takes 18,000 and lands on **382**. This crate charged it 20,000 too
and emptied the queue on 403.

That is why the twenty frames are evidence rather than a coincidence of
magnitudes: they are exactly ten per cent of one `JOB_TIME 200` entry, arriving
on the frame the level does, in a record that pins the counter to the
hundredth. `run39_s_build_queues_are_the_original_s_clock` now reports
**33,631 fields, 1 disagreeing, first at 1851** — the capture's last frame, at
`1/2000`, unrelated. Both floors are pinned: `first >= 1851` and
`wrong.len() <= 1`.

### What it moved

- **run39's queue record 21 → 1**, and its first divergence **383 → 1851**.
  That is the sub-score the queue named for this item.
- **The headline did not move.** East Indies' word and sequence hold at 413,
  Great Lakes' word at 780 and its sequence at 576 with 943/851, run10's 572
  ticks / 776 orders and 802/573 to the frame, run39's own game score 167/167
  with 217/168. The library's twenty frames were upstream of the word in the
  queue record and are not what parts the trace at 413: item 99 — the AI's
  scout, 20 draws against 7 — is.
- The end-to-end test walks the whole thing:
  `the_science_epoch_shortens_the_entry_behind_it_and_refunds_its_price` queues
  the two epochs at a library with the shipped numbers and asserts 201, the
  re-struck 108, the 18,000 target and 382.

### The price side is implemented and still unmeasured

`Sim::tech_price` passed `Modifiers::default()`, so a technology was charged
undiscounted. It now carries `Modifiers::science_ahead`, applied in `cost_of`
where the original calls it: after `TECH_COST_FACTOR` and the nation tail,
before the age-behind discount and the final-tech ramp, and **inside** the
per-resource computation, so the redirect at the end of `get_cost` carries the
discounted number rather than discounting a redirected one.

No diff touches it, and grepping the dumps is what says so rather than
assuming: `queue[scan].type` across run33's and run39's whole captures takes
four values — `-1`, `0`, `50` (a citizen) and the two epochs `551` (`0x227`,
Written Word) and `565` (`0x235`, City State). An epoch's level is its `AGE`
with no plus-one, so `ahead` is zero on every purchase ever traced, which is
what `run40_s_census`'s 120 food for City State already showed. The plus-one,
and the 110% an Ancient *plain* tech therefore costs a player at Science 0, are
the reading's alone; `docs/COSTS.md` §"What is diff-backed" names the capture
that would settle them.

The two sides are inverses of each other only for an age or an epoch:
`reprice` reads `AGE` raw where `science_discount` adds one, and
`the_refund_inverts_the_discount_exactly_for_an_epoch` pins that asymmetry as
the original's rather than a bug in one of them.

## 2026-08-31 — a building's own line of sight (item 99, Opus)

The item was booked as the AI's scout: East Indies parts at frame 413, where
the original spends two `Unit::do_idle+0x7d` idle anims and a nine-draw
`Unit::think_scout` scan and this crate spends none of them. It was the
scout's *arrival* that was late, not its scan. The original's `1/0` reaches
its explore target on frame 412 and idles on 413; this one was still twelve
frames short, because the two had taken different routes since **285**.

### What the two routes were

Both scouts are ordered `EXPLORE_TO (43512, 41976)` on frame 238 by the same
`think_scout` call, draw for draw. The dump prints the whole path stack at
`UNITS=3`, and the two differ in exactly three of seven entries: the original
walks cells `(55, 50)`, `(55, 51)`, `(55, 52)`; this crate walked
`(56, 50)`, `(56, 51)`, `(56, 52)`, one column east.

`PathFinder::calc_cost` is what chooses, and for a scout the fog is the whole
of it: an unseen world cell costs base **8** and a seen one **0x400**
(`docs/PATHFINDER.md` §5), a factor of 128 against a diagonal's surcharge of
8. Column 55 is inside the AI's frame-0 reveal and column 56 is not — so with
this crate's fog, column 56 was 128× cheaper and the search took it. The
arithmetic was not close: 151 against 512.

### The false trails, and what closed it

Three readings were checked and all three held. `find_wpath@00688fc0`'s
`scouting` predicate — the type's scout bit, `EXPLORE_TO`, the order's
`flags & 4`, `get_type() == 3` — is byte-verified in the listing at
`0x68957c`, including the `cmovel` that makes the write conditional and the
`movl $0x0, 0xe85ed4` that resets it at the head; nothing about it can differ
between two calls of the same unit. `calc_cost`'s own branch, the fog read at
`div_3_table[to >> 7]`, and `div_3_table` itself (`n / 3`, filled by
`init_coord_lookup_array@00681db0`) are all as `docs/PATHFINDER.md` had them.
And the frame-1 path — nine entries, reproduced exactly — *needs* the scout's
fog pricing: turning it off costs the word 187 frames.

So the model was right and its **input** was wrong. Forcing the three cells
seen from frame 234 moved the word 413 → 576 in one run, which said the fog
and nothing else. The reveal is `Build::activate@00623e20`'s last statement,
vtable `+0x174`, `update_seen(0)` — the whole disc. run39's AI finishes its
sixth farm on frame **219** at cell `(54, 51)`, and nothing in this crate
threw that disc: the grid grew only where units walked.

### `Wall::update_los`, and the tail nobody would guess

A building's line of sight is not `Unit::update_los`. `Build` and `Wall`
share slot `+0x160`, and the function there ends
`mylos += type->x_size / 2` — a term the unit's has no counterpart to. The
dump is what settles it, and it is unambiguous: run39's AI opens with a Small
City at `mylos 15` (`LOS 12`, `X_SIZE 7`), a Woodcutter's Camp at 7 (`6`,
`2`) and four Farms at 8 (`6`, `4`), and the city goes to **17** on frame 202
— the frame its Science epoch reaches 1, `SCIENCE_LOS` being 2. An unfinished
building sees `1 + x_size / 2`; an unstarted one sees nothing. All of it is
`docs/VISION.md` §2.1.

At `mylos 8` the farm's disc has radius 4, which reaches the three cells of
column 56 beside it and stops short of `(56, 54)` — the cell `think_scout`
still needs dark to pick as its next target, and does. That coincidence is
the check: force `(56, 54)` seen and frame **0** breaks, because the
original's own ring walk accepts it there.

### Why the fog diff did not catch it

`run13_s_fog_grid_is_the_original_s_on_every_cell_of_ten_frames` compares all
144,000 cells over ten frames and is exact. It covers frames 95–104 of a game
in which no building finishes. And `vision.rs`'s own comment had justified
skipping buildings in the hundred-frame resync "because `seen2` is monotone —
the pass cannot *remove* a bit from it", which is true and answers a question
nobody asked: monotonicity says a second sweep cannot unset anything, and
says nothing about a sweep that is never thrown. The lesson is the twin of
item 79's, five days earlier, and it is the same grid.

### What landed

`Sim::build_los`, `Sim::build_sweep` and `Sim::update_seen_build` in
`vision.rs`; the call at the end of `Sim::activate`; buildings walked by
`update_all_seen` beside units; `BuildType::los`/`science_los` through the
loader. Three unit tests and one diff —
`run39_s_sixth_farm_lights_the_cells_its_scout_then_paths_around`, which
asserts the three cells dark on 218 and lit on 220, `(56, 54)` still dark,
and the scout's stack on 238 entry for entry against the original's. All four
made to fail first; dropping the `activate` call leaves the *word* at 576,
because the hundred-frame pass happens to cover this one case on frame 233,
and that is exactly why the frame-220 half of the diff exists.

**East Indies' word and sequence: 413 → 576.** Great Lakes holds at 780 and
576 with 943/851; run10 at 572 ticks, 776 orders, 802/573; run39's own game
score at 167/167 with 217/168 and its queue record at 33,631 fields with one
disagreement. The new parting frame is the AI's **second city**:
`place_city_with_cost` five times over, and the two
`Leader::make_stuff+0x221` draws each of them spends that this crate does
not — 118 draws against 56.

## 2026-08-31 (later, Opus) — item 100: East Indies' word 576 → 645, and the frame was not what it was booked as

Item 100 was booked as the AI's second city. Frame 576 of run39 is
`ScenarioFuncSet::place_city_with_cost` five times over, and the queue read
the trace as saying this crate spent the two `Leader::make_stuff+0x221`
draws each call takes "not at all" — 118 draws against 56, parting at the
third.

It spends every one of them. What it does not do is **name** them.

### An unnamed draw is a wrong answer, not a neutral one

`Built::frame_sites` turns the simulation's `Sim::mark` calls into one label
per draw, and the label of a draw with no mark in front of it is whatever
site marked *last*. `make_stuff`'s expiry walk never carried a mark — its
arithmetic was settled against run18's own seeds on 2026-08-25, five of five
(`docs/AI.md` §15.3), and the marks were simply never added — so its ten
draws at 576 read as `Leader::compute_sites+0x50a`. The comparison therefore
parted on a draw that was correct, at index 2, and the frame's real hole sat
sixty draws further down where nobody looked.

Adding the two marks (`+0x221` the head's walk, `+0x63d` the bought slot's)
changed nothing about the simulation and took **Great Lakes' draw sequence
from 576 to 780** — its word's own frame, so that map's two numbers are now
one. That number had been pinned at 576 since item 60. `docs/SYNC.md` §3.17.

### The mechanic: a bird that lands

Frame 576's actual gap is `Animal::think_bird@005d79e0`'s tail. A bird's
third draw is `rnd % spell_time`, and `== 100` or `> 799` opens a landing
search: thirty rounds over the cell list of the region the patrol point sits
in, two draws a round — `% size` for a cell (`+0x2aa`, skipped for a region
of one) and `% 0x32 + 1` for a score (`+0x2d3`). Sixty draws.

`docs/SYNC.md` §3.9 had read the branch on 2026-08-28 and left its draws
unmodelled with a reason that was true at the time: the modulus of the roll
that opens it *is* the counter, so it cannot fire before a bird has flown a
hundred think-cycles — ~90 frames — and no capture then in hand was long
enough. A `Gaia::bird_landings` list was kept so that a capture which did
reach it would read as a note rather than as silent drift. run39 is 1,850
frames long, and the note was already there: `(576, 8)`, on the nose, the
first entry of fourteen.

**The score decides nothing.** The loop's "is this one better" test is
`-1 < score`, and a `% 0x32 + 1` product of 1..300 can never fail it, so the
patrol point lands on the *thirtieth* cell sampled and the two terrain
multipliers — `×3` forest, `×2` mountain — are dead arithmetic. It is
transliterated anyway, so the next reader does not have to re-derive that it
is inert. Nothing dumps owner 9, so the sixty draws are the whole oracle.

### What landed

`Sim::bird_landing_search` in `gaia.rs`; `SITE_BIRD_SEARCH_CELL`/`_SCORE`
and `SITE_EXPIRE_HEAD`/`_SLOT` in the trace's naming table; the marks in
`Sim::expire`. The diff is
`run39_s_bird_lands_on_576_and_spends_the_search_s_sixty` — the landing at
`(576, 8)`, thirty pairs in the original's alternation, and frame 576 whole
at 118 draws against the trace, label for label. Made to fail first by
dropping the call (back to 56 against 118) and, for the naming half, by
dropping the mark (Great Lakes' sequence back to 576).

**East Indies' word and sequence: 576 → 645.** Great Lakes' word holds at
780 and its **sequence rises 576 → 780**, with 964/864 (from 943/830).
run10's ticks and orders hold at 572/776 with first divergences 802 and 573,
and its two coverage totals rise — collision field-frames 92,766 → 97,333
and angle rows 35,742 → 37,376, the largest rise either has taken from a
mechanic with no unit in it. run39's own game score holds at 167/167 with
217/168 and its queue record at 33,631 fields, one disagreeing.

The new parting frame is **645**, where this crate spends a
`Guy::set_anim+0x97a` under `Unit::do_non_flat_gather+0xb99` — the return
stand — that the original does not, eight draws against seven.

## 2026-08-31 (later, Opus) — item 101: East Indies' word 645 → 742, and two draw sites at one branch

Frame 645 of run39 is seven draws and this crate spent eight. The extra one
was a `Guy::set_anim+0x97a` under `Unit::do_non_flat_gather+0xb99` — the
return stand — so a woodcutter of ours had decided to walk home on a frame
the original's did not. The queue booked it as "the frame or the arm".

It was neither. It was a **wait**, and the number that settled it was in
the dump.

### Grep the dump before booking a reading

`GATHERORDER` is printed every frame of run39's 1,850, and the whole row
with it. Our `0/1` reached `wait 1` on 643 and `−1` on 644; the original's
`0/1` on 645 holds **250**. Walking its history back: the original rerolled
once, on frame 539, from `1` to **355** — and the trace's only draw on 539
is `Unit::do_non_flat_gather+0xcc3`, returning 30755. `30755 % 100 + 300`
is 355. `30755 % 50 + 100` is 105, which is what this crate had.

So the site this crate calls `SITE_WORK_WAIT` does not roll the formula
this crate rolls at it.

### The branch above the tile

`Unit::do_non_flat_gather` reads guy 0's `cur_anim` **before** it reads the
tile (`005f0d0f`), and each arm carries its own wait:

- `CHAR_MINE_ORE` returns immediately — a miner mid-swing does nothing at
  all, no decrement, no facing, no animation.
- `CHAR_CHOP_WOOD` decrements, and on zero rerolls `% 100 + 300` at
  `+0xcc3`. This is where a woodcutter spends the rest of its life: the
  first frame at the tile is what *sets* `CHAR_CHOP_WOOD`, and every frame
  after it takes this arm.
- Only that first frame reaches the distance test, whose reroll is
  `% 50 + 100` at `+0xdad`, and whose `all_gathering` arm returns at
  `LAB_005f0ef1` before the facing and the animation.

`docs/ORDERS.md` §6.4 has had all three since August. The implementation
had one merged branch — the arrival's, marked with the chop site's name —
so a woodcutter ran its cycle at roughly a third of the original's length
and walked home two hundred frames early, every time.

### Why six hundred frames of a matched word said nothing

Both sites draw **exactly once**. The count matches, the sequence matches,
and even a label-for-label comparison of the frame matches, because the
label was the right one — `+0xcc3` really is the site the original spends.
What was wrong was the arithmetic behind it, and the only thing that can
see that is the field the roll lands in.

Which the dump prints. So the first half of the new diff uses **no
simulation at all**: every rise in a dumped `wait` over run39's 1,850
frames is matched against the value the trace's own draw returned on that
frame, under that site's formula. Twenty-four rerolls, nineteen at
`+0x54b` and five at `+0xcc3`, **none at `+0xdad`** — the branch the
implementation spent every reroll on is the one the original reaches never.
Two records of the original, checked against each other, naming the sites
without this crate in the room.

### What landed

The anim branch in `orders.rs`, the arrival's decrement moved in front of
its facing and animation, `SITE_ARRIVE_WAIT` in `sim::orders` and in
`rondata::trace`'s table. The diff is
`run39_s_woodcutters_reroll_on_the_chop_branch_not_the_arrival_s`: the
24 rerolls by site, then the whole non-flat `GATHERORDER` row against ours
— tile, wait, phase, `been_there`, `dist_mod` — **16,152 fields to frame
897**. Made to fail two ways, both met: the labels swapped in the naming
table (half one panics on run39's frame 529) and the arrival's formula put
back at the chop site (the record's floor 897 → 530).

**East Indies' word and sequence: 645 → 742.** Great Lakes holds at 780 on
both and its totals rise 964/864 → **977/866**. run10's ticks and orders
hold at 572/776 with first divergences 802 and 573, and its two coverage
totals **fall** — collision field-frames 97,333 → 97,118 and angle rows
37,376 → 37,174, the third time either has fallen while a score rose. One
of the fourteen units moved: the AI's `1/10` parts at 1522 rather than
1579, and the other thirteen are unchanged to the frame. A woodcutter that
now stays at its tile three times as long is a different unit on the map
from frame 500 on. run39's own game score holds at 167/167 with 217/168
and its queue record at 33,631 fields, one disagreeing.

The new parting frame is **742**, where the original spends a
`Guy::set_anim+0x97a` under `Unit::move_step+0x823` — the blocked stand —
that this crate does not: seven draws against six, and the other six (a
bird's wing-beat coin and five `Farms::inc_time`) agree. So a unit of the
original's has its step refused on 742 and ours walks on.

## 2026-08-31 (later, Opus) — item 102: East Indies' word 742 → 867, and the blocked stand was already right

Frame 742 of run39 is seven draws and this crate spent six. The one missing
sat at the **head** of the frame: `Guy::set_anim+0x97a < Unit::set_anim+0x56
< Unit::move_step+0x823`, the stand a unit plays when its proposed step is
refused. The queue booked it as a collision — "find which unit is blocked
and what blocks it", with item 48's uncompared `down`/`down_who` named as
the near neighbour.

The collision model was already right. Nothing in `collide.rs` moved.

### Which unit, from the dump

Sixteen player units and every one of them prints `collide_frame −1` around
742; the two that are walking (the AI's scout and its `1/9`) walk on
undisturbed. So the blocked unit is gaia's, and the dump names it: `8/3`
walks from frame 737 and, in the state the dump writes as `FRAME 743` —
which is the end of the frame the trace calls 742 — it is standing at
`(28793, 24288)` with `collide_o 2`, `collide_who 8` and its
`orders_x`/`orders_y` collapsed onto itself. That last is `§3.14`'s
`QUEUE_NEW` clear: a blocked animal drops its walk where it stands. Its
blocker is its herd-mate `8/2`, which has not moved in two hundred frames.

Our `8/3` was three hundred units west of all of that, walking due north.

### The wander, six frames earlier

`Animal::do_idle`'s coin comes up on frame 736 — `+0x83`, `39952 % 10 = 2`
— and **no direction draws follow it**. The animal is 477 from its herd
centre, past the `0x181` gate, so it takes the far branch, which spends no
draws:

```
UnitType::find_nearby_spot(type, cx, cy, &out_x, &out_y,
                           0xc0, -1, 0, 0x55555555, FILTER_NOT_ME, o, who, …)
```

The ninth argument is the bearing the sweep's thirty-one directions fan out
from. `do_gather` passes the angle to its camp there; `do_build` the angle
to its site; `action_swarm_around` `find_angle(me − target)`. **This one
passes a literal**: `0x55555555`, which is `Unit::init`'s untouched-angle
constant, 120°, with nothing behind it. Every far wander any herd makes
starts from the same direction, whichever way the animal is looking.

This crate passed `Movement::facing`. run39's `8/3` was facing south
(`UNITDATA angle -2147483648`) — 180° out — and the two answers are two
different walks. Around the herd centre `(28800, 23936)` at `r = 0xc0`, the
original refuses 120° and 142.5° and takes **97.5°**, `k = −1` of the
sweep, snapping to `(28968, 23976)`; ours took a bearing near due north and
walked to `(28728, 24120)`.

With the constant in, `8/3`'s walk is the dump's step for step — `(28741,
24384)`, `(28754, 24360)`, `(28767, 24336)`, `(28780, 24312)`, `(28793,
24288)` — and then the refusal, the dropped walk and the stand all fall on
the original's own frames.

### The widening it came with

run39 prints no `GUY` clocks, so `Built::tick` re-seats nothing in it and
gaia's animals free-run for the whole 1,850 frames with nothing installed.
Nobody had ever compared them. They are now the widest population this
capture speaks to: **190,417 of 192,504 dumped animal-frames stand on the
original's own point**, and the first that does not is frame 983 — 116
frames past the word's own parting, and `8/3` again, wandering off the
point it was refused at.

`a_far_wander_sweeps_from_the_literal_bearing` holds all of it: the whole
walk frame for frame, the drop rather than a path round, and that floor.
Made to fail by putting `Movement::facing` back, which parts the walk on
its second frame and never reaches the stand.

### What landed

One argument in `sim::anim::animal_idle`, and the paperwork: `docs/SYNC.md`
§3.19, `docs/ANIM.md` §7, `docs/ORDERS.md` §10 and its call-site table,
`docs/MOVEMENT.md`'s far-branch note.

**East Indies' word and sequence: 742 → 867.** Great Lakes holds at 780 on
both and its totals rise 977/866 → **986/884** — the same branch, a
different herd. run10's ticks and orders hold at 572/776 with first
divergences 802 and 573 and **every one of its fourteen by-unit partings
unchanged to the frame**; its two coverage totals fall by two unit-frames
each, 97,118 → 97,108 and 37,174 → 37,170, which is a herd wandering
elsewhere deep past the parting. run39's game score holds at 167/167 with
217/168, its queue record at 33,631 fields and its gather record at
16,152 to frame 897.

The new parting frame is **867**, and it is three draws of
`Unit::explore_goody+0x27c < Unit::set_new_location+0x3cc <
Unit::move_step+0x8f4` — a unit walking onto a goody, which this crate has
no model of at all.

### The lesson

A constant in an argument list is a claim about the mechanic. Where the
decompiler prints a literal where a variable would read naturally, the
natural reading is a guess, and one grep of the other call sites is what
tells you whether it is the right one. At three of `find_nearby_spot`'s six
call sites the natural reading is wrong; here it cost 125 frames of the
word, and the item that chased it spent its budget on the wrong mechanic
until the dump was asked which unit was blocked.

## 2026-08-31 (later, Opus) — item 104: East Indies' word 867 → 879, and the box is a lottery

Frame 867 of run39 is nine draws and this crate spent five. The first three
are `Unit::explore_goody+0x27c < Unit::set_new_location+0x3cc <
Unit::move_step+0x8f4`, and nothing here modelled the function at all. The
ninth fell out with them: it was the frame's sixth `Farms::inc_time` draw,
`+0x1de`, the sprout that only happens when the chance draw before it comes
up — and the chance draw before it was reading the wrong word.

### Which unit, and which cell

The `WORLD` record's cells answer both. East Indies has seven cells whose
first `short` is negative — `goodies 7`, and `flags` prints them as 32768
and 33280 —

```
(52, 5)  (22, 22)  (53, 26)  (8, 28)  (36, 31)  (38, 34)  (45, 49)
```

and the per-frame dump puts exactly one unit near one of them: player 1's
scout `1/0`, walking south at three units a frame, `(34984, 38437)` at the
end of 866 and `(34977, 38371)` at the end of 867 — across `y = 38400`,
which is the boundary between cells 50 and 49. `(45, 49)` is the seventh.

### One draw per good you can gather

`Unit::explore_goody@005f9780` is short and the loop is the whole mechanic:

```
best = 99,999,999;  pick = -1
for good in 0..6:
    if type_avail(who, good, 1) == 0: continue
    if good == 3: continue                            // KNOWLEDGE, always
    score = Random::get(game_random, 0, 0xffff) % 25 + bucket[good]
    if score < best: best = score; pick = good
if pick < 0: pick = 2                                 // WEALTH
```

So **the draw count is the candidate count**, and three is what an Ancient
game gives: food, timber and wealth available, metal and oil not, knowledge
refused by name. Every capture on disk that reaches a box spends three —
run39 on 867, run33 on 898 and 1659 — which makes the frame a test of
`LeaderData::type_avail` over the six good types rather than of the
lottery.

The pick itself is neither "the poorest good" nor a coin: `draw % 25 +
bucket`, lowest wins, ties to the earlier good. A good more than 24 behind
the field wins outright; within 24 the jitter decides.

### The constant whose name lies

```
5f9a37  movl 0xf4(%eax), %ebx         ; LeaderDataEncrypt +0xf4
5f9a5f  imull 0xc28(%eax), %ebx       ; constants->goody_box_age = 25
5f9a66  addl  0xc24(%eax), %ebx       ; constants->goody_box     = 25
```

`+0xf4` is `epoch[3]`, not `ages` — `ages` is `+0xdc` and masked `0x62766`
where this site masks `0x63187` — and `LeaderData::get_epoch_base(3)`
returns `BASE_EPOCHTYPES`, which the `TypeIndex` enum gives the same value
as `BASE_SCIENCETYPES`. **`GOODY_BOX_AGE` is per Science library level.**
run39's player 1 is on Science 1 at 867, so its box pays 50 where the age
reading would pay 25. The Spanish pair replaces *both* halves, not the base
alone.

Two smaller things the listing settled that the decompiler could not. The
guard in front of the call is vtable slot `+0x30`, which the map folds onto
`Window::get_button`/`Buffer::is_pending_load` and the PDB's `LF_ONEMETHOD`
list names `SubObjectData::is_animal` — the same slot step 0 of
`docs/COLLISION.md` §6 reads, so **gaia takes nothing**. And the cell's
object-chain repair has a genuine dead store in it: where the cell's own
`down` is `-3` the original writes `-3` straight back (`movl
$0xfffffffd, %edx; movw %dx, 0x8(...)` at `5f98ff`). It is the `flags &=
0x7fff` two instructions later that actually spends the box.

### What landed

`crates/sim/src/goody.rs`, four constants in `Tuning` (all checked against
`rules.xml` by `cargo run -p rondata`, made to fail once), a
`goody_box_resources` counter on `Ledger` for `LeaderData +0x86c`, the four
guards in `Sim::set_new_location`, one row in `rondata::trace::SITES`, and
`docs/GOODY.md`. `docs/VISION.md` §7's open question is struck and pointed
at it — and it was wrong twice over: not one draw but three, and it shares
`set_new_location`'s *caller*, not its trigger, since the reveal hangs off
the tile test and the goody off the cell test.

`a_goody_box_draws_once_for_each_good_its_finder_can_gather` is the rule
against the record: the seven cells, the bit set through 866 and clear
after 867 with the other six untouched, the frame's sequence equal to the
trace's, and fifty wealth in player 1's bucket. Made to fail twice — by
dropping the availability guard, which spends five draws and parts the word
back at 867, and by reading `ages`, which pays 25.

**East Indies' word and sequence: 867 → 879.** Great Lakes holds at 780 on
both with its totals unchanged at 986/884 — its two boxes are at 898 and
1659, past its parting. run10 is untouched in every figure, headline,
by-unit and coverage; it has no trace and world6's units reach no goody.
run39's game score holds at 167/167 with 217/168, its queue record at
33,631 fields and its gather record at 16,152 to 897. Tree: 661 sim, 154
rondata.

**One number fell**, and it is worth being plain about: run39's gaia
agreement over the whole capture, 190,417 → **189,843** of 192,504, with
its first parting frame **983 unmoved**. Every frame past the word's own
parting draws from a stream that is nobody's, so moving the word forward
re-rolls all of them; a herd's coin at frame 1,400 is not evidence about
anything. The test now pins 983 — the number with meaning — alongside the
weakened total, and says so.

### What is not settled

The pile. No dump on disk prints a leader's `bucket` after a box opens:
the `LEADERS` detail that writes `bucket`, `ages_get()` and
`epoch_get(scan)` is emitted once, in the start block, where every value is
still its opening one. So `epoch[3] × 25 + 25` rests on the listing alone.
The capture that would settle it is cheap and is now in the queue: the
run39 lobby under `samegame.py` with `LEADERS` per frame, nine hundred
frames, and `epoch_get(scan)` reading `0 0 0 1` on 867 is the whole of the
difference between the Science reading and an `ages` one.

### The lesson

A constant's *name* is not evidence about what it multiplies. `GOODY_BOX_AGE`
multiplies a library level; `SPANISH_RUINS` replaces the per-level half and
`SPANISH_RUINS_BASE` the flat one, which the pairing of names does not tell
you either. Item 102's lesson was that a literal argument is a claim; this
is the same lesson one level down — the tuning file's vocabulary is the
designers', and only the consumer says what a constant means. `CLAUDE.md`
has said "read the consumer before believing the digits" about
*representation* since the 8.8 traps; it holds for meaning too.

## 2026-08-31 (later, Opus) — item 106: East Indies' word 879 → 1256, and a scout that finished a walk nobody gave it

Item 106 was booked as frame 879: the scout going idle and re-thinking, two
`Unit::set_anim` stands and `think_scout`'s six ring pairs with two cell
draws — sixteen draws this crate did not spend. It was that, and the item
was not about `think_scout` at all. **The question was why the scout was
idle on 879**, twelve frames after it took the goody box on 867, when
`think_scout` had sent it to a cell it had not reached.

### The reading

`Unit::do_explore_to@005f24a0` is not `do_move`. It is `do_move` and then:

```c
if ((o + frame) % 15 == 0 && orderlist.head == this_order && is_captain())
    find_goody_box(this);
```

`Unit::find_goody_box@005f2540` is a 49-cell sweep — `move_x`/`move_y` to
`0xc4 / 4`, the 7 × 7 the muster search already uses — for a cell of the
sweeper's own region carrying `WData.flags & 0x8000`, and the first one it
accepts is handed to `Unit::get_goody_box@005f7690`, which pushes a
one-member group and orders it `EXPLORE_TO` the box's **cell centre**. None
of it draws. `think_scout`'s own first line calls the same function, which
is why `docs/SCOUT.md` §13 had carried it as item 2, unread, since August.

So run39's scout re-aims on frame **825** — the fifteenth frame after the
box comes into its own line of sight — walks to `(45, 49)`, takes the box on
867 as item 104 already had it, arrives on 879 with an empty order list, and
re-thinks. `docs/GOODY.md` §7.

### The gate that is not the obvious one

The sweep has two fog tests and the first one is a decoy. The cell's is
`WorldData::was_seen`, which carries the **ally-territory shortcut**: a cell
owned by an ally of the asker, in a region where that ally has a city, is
seen whether or not anyone has looked at it. `(45, 49)` sits inside player
1's own borders, so that test answers yes from frame 0 — and a sweep gated
on it alone fires on **796**, the frame `think_scout` runs, and parts the
word there. The implementation did exactly that on its first run: 879 →
796, a regression, which is how the second gate got found.

The second is the **item's**: `find_goody_at` walks the cell's object chain
and asks what it finds `ItemData::is_seen` (vtable `+0x48`), which reads
`ever_seen & ally_mask` — a per-item byte `check_ever_seen` accumulates out
of the current line-of-sight grid, with no shortcut at all. That is what
holds the scout until its own eyes reach the box, some time between 811 and
825. This crate chains no items, so it models `is_seen` as `was_really_seen`
over the same four half-cells; the two are the same accumulation under
different names.

**Two functions one letter apart decided a four-hundred-frame move.** The
crate has had `was_seen` and `was_really_seen` side by side since
`docs/SCOUT.md` §12 wrote them down as "different functions with one word
between their names". This is the first time the difference has been worth
anything, and it was worth 377 frames.

### And a document that was right while the code was not

`Group::action_move_near`'s `QUEUE_FIRST` arm halts the group and re-runs
itself as `QUEUE_NEW`, restoring only the leader's **action-flagged** orders
afterwards. `docs/GROUPS.md` §6.2 has said so since the first reading — and
this crate had been passing the queue position straight down to
`add_move_facing_order`, which stacks and keeps. The difference is one field
of run39's `FRAME 826`: **one** order in the scout's list, not two. That is
the queue's item 72 shape again — a document and its code disagreeing is a
diff waiting to be run — and the fix went into `group_action_move_to` for
every caller, `army_charge` included.

The correction could not go in §6.2, which the size guard pins; it is
`docs/GROUPS.md` §17, in the shape §15 and §16 already had.

### What landed

`Sim::find_goody_box`, `Sim::get_goody_box` and the two fog gates in
`crates/sim/src/goody.rs`; `Sim::was_seen_fog`, split out of
`ai_sites::site_was_seen` so the sweep can sample all four half-cells;
`do_explore_to`'s tail in `orders.rs`; `think_scout`'s head in `scout.rs`;
the group `QUEUE_FIRST` arm in `group.rs`; `docs/GOODY.md` §7,
`docs/GROUPS.md` §17, `docs/ORDERS.md` §13's entry, and `docs/SCOUT.md` §13
item 2 struck and pointed at its answer.

`a_scout_re_aims_its_walk_at_a_goody_box_it_has_seen` is the rule against
the record — four frames of run39's own `UNITDATA`, `orders_x/y`, the whole
path stack and the order count on each: 797's three legs, 826's one, 879's
empty list, 880's four. Made to fail by dropping the item gate, which fires
on 796 and parts the word there.

**East Indies' word and sequence: 879 → 1256.** Three hundred and
seventy-seven frames, with four `think_scout` frames — 879, 1021, 1143,
1231 — inside them. Great Lakes holds at 780 with 986
frames on the count and its draw-for-draw total **884 → 892**. run10's
headline is unmoved at 572/776 with both first divergences at 802 and 573,
and three of its fourteen units hold longer — `1/0` 872 → 959, `1/8` 904 →
906, `1/10` 1522 → 1552 — which takes its collision rows 97,108 → **98,019**,
its angle rows 37,170 → **37,450** and its first disagreeing gather tile
1,373 → **1,384**. run39's gaia agreement rises 189,843 → **190,690** of
192,504 and its first parting frame 983 → **1261**. Its queue record holds
at 33,631 fields with one disagreement, and **its gather record went 16,152
fields to frame 897 → 24,738 to frame 1,573**, which closes item 103 without
touching it: the ten-frame wait that parted at 897 was downstream of the
scout. Tree: 668 sim, 155 rondata.

### The lesson

**A frame the word parts on is not always the frame the mechanic is in.**
879 was a real hole and its sixteen draws were exactly what the trace said;
the thing that had to change to fill it happened fifty-four frames earlier
and spent nothing. Item 102 learned that a booked frame can turn out to be
a different mechanic; this is the sharper version — the booked frame was the
right *symptom* and its cause left no trace at all, because
`find_goody_box` draws nothing. The draw stream says where two runs stop
agreeing, never where they stopped doing the same thing. When the frame's
own content is already understood and still will not come out, the question
to ask is what the unit was doing before it.

## 2026-08-31 (later, Opus) — item 109: East Indies' word 1256 → 1373, and one step in the caller

Frame 1256 was a bird. Ours spent 87 draws where the original spends 27:
eight birds thinking three draws each, and our second one falling into
`Animal::think_bird+0x2aa`'s sixty-draw landing search on a frame no
original bird lands.

The draws going in were identical — the same stream, the same seed, the
same value. What decides a landing is `rnd % spell_time`, so the only
thing that could differ was the **counter**, and the counter is the one
piece of a bird's state nothing can see: no dump prints owner 9 at all.

### The step was never in `think_bird`

`think_bird` was read whole in August and its arithmetic is right. The
step it was missing belongs to its caller. `Unit::do_air_patrol@005ea620`
runs the `+0x180` virtual, then `do_air_physics`, and then:

```text
if (do_air_physics(...) != 0) {
    if (vtable+0x30 () == 0)   … the military plane's target search …
    else if (spell_time == 0)  spell_time = 1
}
```

`vtable+0x30` is `SubObjectData::is_animal` — the slot `docs/SYNC.md`
§3.14 had already named against the map's COMDAT folding, which puts
`Buffer::is_pending_load`'s `return 1` there — so a bird always takes the
second arm, and `do_air_physics` returns 1 on every path a bird carrying
its single air-patrol order takes through it. `think_bird` steps the
counter on **every** frame it runs, so `spell_time` is 0 at that point
only on a frame the landing search has just zeroed it. **One extra step
per landing, and nothing else.**

Which is the whole of frame 1256. run39's second bird landed on 944; 311
frames and 38 think-frames later its counter reads 349, `think_bird`'s own
two steps make the modulus 351, and `27127 % 351` is exactly **100** — the
single value in the counter's range that the `== 100` arm tests for. With
the caller's step the modulus is 352, the remainder 23, and the bird flies
on. The original's own landings at 1368 and 1376 then fall where they
fall.

### The assertion a bird can carry

Nothing dumps owner 9, so the counter is unobservable — but the frame a
search fires on is not, thirty `+0x2aa` draws at a time, and run39's frame
944 spends sixty because two birds land on it.
`a_bird_s_landing_frames_are_the_trace_s_own` reads the landing frames
straight out of the trace and holds this simulation to them up to the
word: **576, 944, 944, 1016, 1144, 1368**, six for six. Made to fail by
dropping the branch, which adds a seventh on 1256.

### What landed

The `spell_time == 0` tail in `orders.rs`'s bird arm of `do_idle`;
`docs/SYNC.md` §3.9's new subsection and its first Coverage row; the new
diff test.

**East Indies' word and sequence: 1256 → 1373.** A hundred and seventeen
frames with two of the original's own bird landings inside them. run10's
headline is unmoved at 572/776 with both first divergences at 802 and 573,
and one of its fourteen units holds longer — `1/10` 1552 → **1579** —
which takes its collision rows 98,019 → **99,607**, its angle rows 37,450
→ **38,104** and its first disagreeing gather tile 1,384 → **1,394**.
run39's gaia agreement rises 190,690 → **191,173** of 192,504 with its
first parting 1261 → **1381**, and its gather record went 24,738 fields to
1,573 → **26,094 to 1,686** — which moves item 103 without touching it,
from a wait a hundred long on frame 1,573 to one thirty-five short on
1,686. Tree: 668 sim, 156 rondata.

### The floor that fell

Great Lakes' word and sequence hold at 780, and for the first time its two
window totals **fell**: 986/892 → **959/876**. Every frame whose verdict
changed is 1209 or later, 429 past the parting — 31 gained, 47 lost — and
the cause is exactly the mechanic: a counter one higher after each landing
moves this map's landings from 904, 936, 1080, 1136, 1152, 1288, 1416,
1520, 1728, 1784 to 904, 936, 1080, 1136, 1152, **1184, 1232, 1456,
1640**, 1784. Against the original's thirteen it now meets two rather than
one.

That is the noise `run33_s_long_trace_says_where_the_word_parts`'s own
comment has described since it was written — a total past the parting is
on a stream that is nobody's — but it had never actually moved backwards
before, and a guard whose number only ever rises has not been tested
either. The floors are lowered with the reason in the comment beside them.

### The lesson

**A function read whole can still be missing a line, and the line is in
its caller.** §3.9 transcribed `think_bird` correctly, tested it against
run14's frame counts, and pinned its landing search against run39's frame
576 — and none of that could see a `+= 1` that happens two calls up the
stack, because the counter it moves is unobservable and the landing it
shifts was three hundred frames away. The rule the audits keep
rediscovering is *read the loaders, not only the consumers*; this is its
twin. **Read the caller, not only the callee** — especially where the
callee's whole product is a piece of hidden state.

## 2026-08-31 — the fourth steering pass (Fable 5): the word outran the finish line

Sixteen item commits in a day since the third pass, every trailer Opus,
no subagent. The verification came first and it all held: the floors in
`rondata::diff` equal the handoff's numbers, the tree is green with the
install attached — 668 sim, 156 rondata, no `skipping` line — and the
tranche's newest claim, re-read against the decompile, came back
verbatim: item 109's `do_air_patrol` tail is exactly `if
(do_air_physics(…)) { if (vtable+0x30() == 0) … the military search …
else if (field_0x98 == 0) field_0x98 = 1; }` — the one-step bump on the
animal arm, `spell_time` by the PDB. The tranche is real. It was also,
after item 102, booked on the wrong number, and the queue's own handoff
half-saw it: "the headline is Great Lakes at 780 and it has not moved in
six items" — with three items booked on the higher map and none saying
so.

### The two scores, and which one is the finish line

Phase 3's score is **ticks before divergence** (CLAUDE.md, DECISIONS 24),
and this pass measured the pair rather than reading it off the prose:

```
run39: ticks 167, orders 167, first divergence [(0, Some(217)), (1, Some(168))]
run10: ticks 572, orders 776
```

East Indies 167/167 of 1,850; Great Lakes 572/776 of 1,772. **Neither
number moved in the whole tranche** — both stand where they stood on
08-30, and run39's has stood since 08-29, before the tranche began. What
moved was East Indies' *word*, 19 → 1373, and the handoff was calling
Great Lakes' word 780 "the headline" — the lower map read off the words
(780 < 1373) when the score reads the other way (167 < 572).

When run39's word parted at 19, `diff.rs`'s own comment was right: ticks
167 was a figure on a stream that is nobody's, and the word was the
number to move. The comment did not age. The word passed 168 between
items 95 and 97, and from that frame the inference runs the other way:
every draw before 168 now agrees, draw for draw, so what parts at 168 —
`1/4`'s order list, then `1/5`'s at 186 — is arithmetic no stream
touches, and no amount of word motion past it can close it. Item 69
became the most valuable item on the board around item 97 and sat third
in the queue for eleven items while the chase ran to 1373.

### What the chase bought anyway

Not nothing, and it matters for the verdict: the goody box and the bird
counter are cross-map mechanics — Great Lakes' bird landings went from
meeting one of the original's thirteen to two, run10's collision rows
rose 91,210 → 99,607 and its angle rows 35,188 → 38,104, run39's gaia
agreement reached 191,173 of 192,504, and its gather record 26,094
fields to 1,686 — which retired item 103's 897 and 1,573 versions
without anyone booking them. A word chase on one map keeps paying the
other. The failure is not the chase; it is that nothing bounded it.

### The amendment, and the batch

DECISIONS 26: **the headline is the tick pair, lower first; each map's
word is the instrument beside it; and a word chase is bounded by its
map's tick divergence** — the frame the word passes it, the tick item is
the default booking. The reporting rule stays ("booked off the default
says so"), with the judgment call that made silence easy removed.

The batch was two parked rows in the orders audit. **R4 —
`find_gather_tcoords@0063bdc0`, parked as "matters only when the harness
builds a camp itself" — is live**: item 85's finding is a harness-built
camp activating with zero `gather_slots`. The marker travels with item
85; the audit file is left as it is, per the section pins. R2 O1
(`do_move`'s attack-retarget block) stays parked; nothing in
`crates/sim` reads it yet.

Cadence: by entry 25's rule the steer was due after item 106 — the
headline had not moved for two sessions — and it ran after 109. One
session late; the rule is unchanged, and what actually fired it was the
handoff writer noticing, which is the system working.

### Where the next session starts

Item 69, on Opus: East Indies' order list at 168, the pair's own nearest
divergence, now draw-free — run39's dump names the fields, and the score
it moves is the headline itself.

## 2026-08-31 (later, Opus) — item 69: the score run was not the run the word measures

East Indies **167/167 → 1374/1373**. Great Lakes **572/776 → 781/776**.
Both maps' tick scores now sit one frame past their own word's parting.

```
run39: ticks 1374, orders 1373, first divergence [(0, Some(1411)), (1, Some(1375))]
run10: ticks  781, orders  776, first divergence [(0, Some(802)),  (1, Some(782))]
```

The opener asked the right question and split it the wrong way round. A
parting no draw feels for 1,200 frames is a field no draw reads, **or the
diff's own misreading** — and it was the second, in a form nobody had
looked for: the run that scores and the run that matches the word were
not the same simulation.

### The widening first, because the item's own claim was wrong

`1/4` was booked as an order-list *length* at 168. It is not. At 167 both
sides hold two orders, the same kinds, the same target, the same flags —
and different destinations: ours `(40440, 39480)`, the original's
`(40248, 39288)`. Nothing compared them. The dump has written the whole
`MOVEORDER` row since the parser was widened for it, and `compare_orders`
read `coll_x/coll_y` and stopped.

`OrderMismatch::Move` now walks the row — `x`, `y`, `angle`, `dest`,
`dest_x/dest_y`, `last_x/last_y`, `pause`, `timer`, `facing`,
`off_x/off_y` — on every dumped move of both maps. Three fields are
reported and do not score, each for a reason on the variant: `dest` and
`last_x/last_y` are the pathfinder seam's own timing, `facing` is item
23's open formation mirror. `dest_x/dest_y` is compared on the frames
`dest` says it is live.

It paid immediately and twice. `1/1`'s swarm move disagreed on `angle`
at **frame 2**: `Group::action_swarm_around` passes
`find_angle(site − spot)` — the bearing from the spot back to the site,
so a builder arrives facing what it will build — taken from the ring's
answer *before* the `BUILD_AT` nudge. This crate derived
`find_angle(spot − here)` instead. The decompiler prints both of that
function's `find_angle` calls with the same two locals because the pair
travels in `ecx`/`edx`; the listing separates them in a minute
(`7103f3`–`710406` reads both out-parameters of `find_nearby_spot`
straight into the subtraction, and `710415` nudges the same register
afterwards). `docs/ORDERS.md` §5.4.

### The pasture was in the trace and not in the score

With the angle fixed, `1/4` still parted at 167, and the trace said
frames 164–169 were draw for draw. Two `GameAccess::rnd(4)` at the same
stream position cannot return `(1, 0)` on one side and `(2, 1)` on the
other — so the streams were not the same, and the word check said they
were. They are two different runs.

A pasture's five animals are owner 9 and **no dump prints them**
(`docs/SYNC.md` §3.6). The only source is the trace, through
`diff::borrow_pasture` — and that call lived in
`run39_s_long_trace_says_where_the_second_map_s_word_parts` and nowhere
else. `run_traced`, which every tick-and-order score is measured on,
borrowed the siblings and no pasture. East Indies' AI builds one. Five
idle rolls a frame went missing from the scoring run, its stream parted
within a few frames of the pasture finishing, and every figure this
capture has ever produced — `1/4` at 167, `1/5` at 186, player 0's
citizens at 217, and the fourth steer's whole reading of the tranche —
was a different game's arithmetic.

`run_traced` now takes the trace as a source beside the siblings, and the
score asserts the pasture is there rather than scoring low in silence.
That alone took East Indies to **1374/536**.

### And then the pause, which was Great Lakes'

The widened row put `1/2`'s `pause` at frame 573 at 2 against the
original's 3, on the frame Great Lakes' ticks had parted for weeks.

`do_move`'s straight-line check ends `if (masks & 8) goto STEP`, and
`STEP` is **past** the pause check; only the re-plan's `TAKE` comes back
through `STEP_IF_MOVING`. So a unit that re-verifies its line this frame
steps this frame and its collision pause does not tick. The dump carries
the whole example: 572 writes `pause 3`, `coll_x/coll_y` and `dest 0`;
573 steps with `pause` still 3; 574–576 are the three still frames; 577
walks. This crate ticked on 573 and was a frame ahead for the rest of the
walk.

Great Lakes' ticks went 572 → **781** and its player 1 573 → 782. East
Indies' orders went 536 → **1373**. `docs/ORDERS.md` §4.9.

### What landed

- `OrderMismatch::Move`, the whole `MOVEORDER` row, with three fields
  reported and not scoring and the reason on each.
- `run_traced` takes `Option<&Trace>`; `rondata --diff --trace` passes it.
  The run39 score test asserts the pasture is in the run.
- `Sim::swarm_spot` returns the move's angle;
  `Sim::do_move` models `goto STEP`.
- `docs/ORDERS.md` §4.9 (new) and §5.4; `docs/SYNC.md` §3.6.
- Floors raised: run39 1374/1373/1411/1375, run10 781/776/802/782.

### The floors that fell, and one residue

run33's window totals 959/876 → **953/877** and run10's angle rows
38,104 → **37,838**: both are whole-run totals that count coincidences
past a parting, and both moved while the parting itself held at 780. The
number to read is the headline, and it rose on both maps.

run10's collision block is no longer empty inside the comparable window.
`1/6`'s frame 797 reads `collide 3` against 2 and `collide_frame 796`
against 795 — one collision more, entered a frame later, on a unit whose
position parts at 798. It only became comparable because the window grew
by two hundred frames. It is pinned as two rows, any third fails, and it
is the queue's successor to this item.

### The lesson

**A harness has state, and state that only one check installs is state
the other checks do not have.** The pasture was found, read, implemented
and asserted five days ago; what was never asked is *which runs get it*.
The word check did, the score did not, and for a week the two numbers
described different games while sitting in the same file. A tranche of
sixteen items chased one of them.

The cheap guard is the one now in the test: the score asserts its own
inputs. Not "the pasture mechanic works" — that was already true — but
"this run has one".

And the second lesson is the older one, which cost the first three hours
here before the first fixed anything: **when the original dumps a record,
diff the whole record.** The move order's row had been parsed and
uncompared for as long as the parser has had it. It found the swarm
angle at frame 2, the farm tile at 167 and the collision pause at 573 in
the same afternoon.
