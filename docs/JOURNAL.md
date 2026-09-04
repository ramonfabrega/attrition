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

## 2026-08-31 (later, Opus) — item 84: Great Lakes' word 780 → 986, and a byte that was not the guy's

Great Lakes **781/776 → 910/776** on ticks and orders, its word **780 →
986**, and its two window totals **953/877 → 1182/1114** — the largest
move either total has made and the first since item 81 that is not noise,
because the word itself carried it. Player 0's first divergence goes
**802 → 1154** and player 1's **782 → 911**. East Indies is untouched at
1374/1373: no farmer on that map is in the case.

```
run33: word parts at 986, sequence at 986; 1182 of 1850 frames spend the
       original's number of draws, 1114 of them draw for draw
run10: ticks 910, orders 776, first divergence [(0, Some(1154)), (1, Some(911))]
```

### The item's two halves were one byte

The queue booked two symptoms and asked which was the half to chase: two
`Unit::do_job+0x67` draws at 780 that the original does not spend, and a
**seventh** `Farms::inc_time` chance a frame from 781 where the original
ticks six. They are the same farmer.

`Farms::inc_time` skips a farm with fewer than five empty cells
(`docs/SYNC.md` §3.3), and `empty` only *falls* when something sows — the
sprout's own draw, or a farmer's `Farms::grow`. The original's b10 drops
to four empty on 780 and this crate's stays at five, so on 780 the
original's farmer sowed the cell it was standing on and this one drew for
a new tile instead. One decision, both rows.

### What decides it

`do_gather`'s farm switch, case 0 — an **empty** cell — reads
`*(char *)(**(int **)&this->field_0xf4 + 0x9c)`: `UnitData::guys[0]`,
then `GuyData +0x9c`, which the type record names `cur_anim`. Not `'$'`
→ sow. `'$'` → a new tile, two draws.

This crate kept a `Unit::farm_anim` flag instead, written only by that
switch and cleared only by a step that actually moved the body. The
difference is everything that plays an animation *without* moving: run33's
AI farmer `1/4` reaps farm `b10`'s cell 7 from frame 740; `inc_time`
decays the cell empty under it on 777; on 778 it is an empty cell under a
reaper, so both sides draw for a new tile; on **779 its walk is blocked**
and the stand `Unit::move_step+0x823` plays — the draw is in both traces,
frame 779, index 0 — replacing the reap animation while the body does not
move a unit. On 780 the original reads a byte that is no longer `'$'` and
sows. This crate's flag still said Reap, so it re-picked the same cell,
and then again every other frame for the rest of the capture, keeping the
farm at five empty for ever.

The fix is to delete the flag and read the guy. `docs/ORDERS.md` §6.5.

### What it closed on the way

Queue item 111 — run10's `1/6` taking a collision the original does not at
797, `collide 3` against 2 and `collide_frame` 796 against 795 — is
**gone**. It was item 69's residue and it was this: a farmer re-picking a
tile it should have sown is a unit walking where the original stands, and
the collision block is empty again over every comparable field-frame of
112,447.

### The new parting

986, and it is a **blocked stand a frame early**: this crate spends
`Guy::set_anim+0x97a < Unit::move_step+0x823` on 986 where the original
spends it on 987, with the six farms equal either side and the two
frames' remaining draws identical. Seven against six, then seven against
eight. That is the successor.

### The lesson

**A model that remembers is a model that can be wrong for ever.** The
flag was not a wrong reading of the switch — the switch's two arms were
right, and had been since the cell index was fixed. What was wrong is
that the byte the original reads is written by *every* animation in the
engine and the flag was written by one function. A cached copy of a field
somebody else owns needs the writers grepped, which is a rule this
project already has for frozen fields and had not applied to a field it
invented.

The cheap route to it was the one the working agreement names: the trace
said frame 780's first draw was ours and not theirs, and eight lines of
temporary `eprintln` over the farm list and the farm switch — which farm,
which cell, which unit, what state, what anim — said the rest in one run.

## 2026-08-31 (later still, Opus) — item 112: Great Lakes' word 986 → 1372, and a herd that does not walk

Great Lakes' word **986 → 1372** and its sequence with it, its two window
totals **1182/1114 → 1471/1439**, and player 0's first divergence **1154 →
1385**. East Indies is untouched at 1374/1373 and word 1373. **The headline
did not move**: ticks stay 910/776, and that is the fact this item hands
forward.

```
run33: word parts at 1372, sequence at 1372; 1471 of 1850 frames spend the
       original's number of draws, 1439 of them draw for draw
run10: ticks 910, orders 776, first divergence [(0, Some(1385)), (1, Some(911))]
```

### The blocked stand was five frames of walking, and the walk was aimed wrong

The queue booked one frame of a blocked unit's timing: ours spends
`Guy::set_anim+0x97a < Unit::move_step+0x823` on 986 where the original
spends it on 987, with the six farms and every other draw of both frames
equal. The unit is Gaia's sheep `8/3`.

It is not a timing question. The animal's wander coin comes up on 981 —
one draw, `Animal::do_idle+0x83`, in both traces — and the far branch then
picks a destination with **no draw at all**, so the word cannot see the
difference. Ours walks five steps of `(+15, −5)` to `(17688, 26616)`; the
original walks five of `(+12, −10)` to `(17688, 26328)`. Both are refused
by the same neighbour, and ours reaches the refusal a frame early.

### The centre the ring is drawn about

The far wander is `find_nearby_spot(herd_centre, 0xc0, −1, 0, 0x55555555)`
(`docs/ANIM.md` §7). Enumerating our own sweep — nine radii, thirty-one
bearings each — showed `(17688, 26328)` is not in it anywhere, so the
disagreement is the **centre**, not the sweep.

`herd_centre` is `((w + 2·c)·0x300 + 0x480)/3` per axis, which inverts:
our centre `(17536, 26496)` is `(wx + 2cx, wy + 2cy) = (67, 102)` and the
original's answer needs **101**. One cell of `wy`.

`Herd::process@00741760`:

```
iVar3 = *(int *)this             + -1 + rand % 3;   // +0x0  cx
iVar2 = *(int *)&this->field_0x4 + -1 + rand % 3;   // +0x4  cy
...
    *(int *)&this->field_0x8 = iVar3;               // wx
    *(int *)&this->field_0xc = iVar2;               // wy
```

It **reads the home cell and writes the wander centre**. `HerdData` names
`+0x0..+0xc` `cx, cy, wx, wy`. So the centre is jitter about home, bounded
to nine cells for ever; this crate read the destination as the source and
random-walked it.

The two readings agree until a herd's **second** walk, and the cadence is
`(frame >> 6) % 13` — thirteen herds on this map — so inside 1,850 frames
exactly one herd gets a second, herd 0 on frame 832. Its `wy` ended 34 here
against the original's 33, and 150 frames later that was five hundred
frames of the word.

### What the new parting is

Frame 1372, and it is a mechanic rather than a residue: the original spends
twenty-five draws in `Farms::add_animals` — `+0x92`, `+0x134`, `+0x182`,
five each — under `Build::activate+0x1c25`, and five more in
`Guy::init_real` under `Animal::init`. A farm activating stocks a pasture
with five animals. This crate spends three draws on that frame.

But the headline is no longer the word. Great Lakes' ticks are held at 910
by **one** unit, `1/1`, parting at 911 — four hundred and sixty frames
before the word does — while every other unit of the fourteen parts at
1375 or later. That unit is the item, and the pasture is behind it.

### The lesson

**A field's writer is not its reader, and the decompile prints both.** The
arithmetic here was right — `−1 + p % 3`, the bounds test, the feature
mask, the `< 8` — and it had been checked against a capture. What was
wrong is which field the `−1` was added to, and the capture that checked
it could not tell, because on a herd's first walk the two fields hold the
same number. The rule the project already has for frozen fields — grep the
writers — has a twin: **when a function reads one field and writes
another, say so out loud**, because a paraphrase that collapses them into
`w' = f(w)` is a different mechanic that agrees on its first step.

The cheap route to it was the sweep enumeration. Once the original's
answer was shown *not to be in our candidate list at any radius*, the
question stopped being "which candidate" and became "which centre", and
the centre inverts to two integers.

## 2026-08-31 (later still again, Opus) — item 113: Great Lakes' ticks 910 → 1375, orders 776 → 791, and a citizen sent to look for fog

Great Lakes' **headline moved for the first time in three items**: ticks
**910 → 1375** and orders **776 → 791**, player 1's first divergence
**911 → 1376**. The unit the queue named — the AI's `1/1` — is **gone from
the by-unit list entirely**: it never parts on position across the whole
1,772 frames, and every one of the other thirteen parts on the frame it did
before, to the frame. East Indies is untouched at 1374/1373 and word 1373.

```
run10: ticks 1375, orders 791, first divergence [(0, Some(1385)), (1, Some(1376))]
run10 by unit: [(0,3,1385), (0,4,1462), (0,5,1409), (1,3,1725), (1,4,1489),
                (1,5,1379), (1,6,1735), (1,8,1376), (1,10,1552)]
run33: word parts at 1372, sequence at 1372; 1467 of 1850 frames spend the
       original's number of draws, 1436 of them draw for draw
```

The two window totals fell 1471/1439 → **1467/1436**, and for once the
argument is exact rather than statistical: `first_count` is by construction
the first frame whose draw *count* differs, and it holds at 1372 with the
sequence, so every frame before 1372 is identical on both sides of the
change and all seven moved verdicts are past the parting.

### The oracle was already on disk, and nine tenths of it was uncompared

`LEADERS=9` prints `Leader::sites` **whole** — ten
`{wx, wy, val, reg, dist, rank}` a leader a frame — and
`LeaderData::territory` beside it. The two census windows captured for the
city *price* a day earlier (`run40` `[560, 600)`, `run41` `[770, 800)`)
therefore carried the site scorer's own per-frame output over the two frames
in the whole 1,850 on which `place_city_with_cost` runs, and nothing had
ever looked at it. Widening it took twenty minutes and printed the answer:
frames 560–575 agree slot for slot, **576 is the first disagreement**, and
it is one slot — ours `(49, 27) / 1159` against the original's
`(47, 28) / 6181`.

### The first defect: a border that never widened

Tracing our own 5×5 slide over the sampled cell `(48, 29)` showed
`(47, 28)` **UNSEEN**, and the whole fog plane around it zero on both sides
— so `WorldData::was_seen@006b53f0` was deciding it on its *first* arm,
which is territorial: a cell owned by an ally, and `is_ally` is reflexive.
The scorer sees exactly the AI's own borders here. The dump's own
`territory` field settled it in one line: **the AI holds 290 cells and this
crate held 261**, with the human's 266 exact on every frame of both windows.

`World::compute_reg_territory@006b0bb0` rebuilds its eight-row per-player
bonus table at the top of *every* pass and reads
`data_encrypted->epoch[1] ^ 0x63187` — the Civic library level, the same
field `get_city_limit` names — for `CIVIC_UPGRADE_TERR`. `crates/sim` built
that table once in `add_player` and never rewrote it, so the AI's City State
(researched around frame 200) never widened its border for the rest of the
game. `Sim::player_borders` reads it live now; `sync_territory` rebuilds all
eight rows and `apply_gained` re-syncs when a gain moves one.

A free correction fell out of the same loop body: the Russians'
`RUSSIAN_BORDERS_PER_AGE` multiplies the **Civic level**, not the age — the
decompile reads the identical `+0xec ^ 0x63187` expression twice, ten lines
apart.

### The second defect, which only the first could uncover

With the border right, the AI chose the original's site and handed `1/1` the
original's `BUILDORDER` to `(41448, 23928)` on frame 777 — and Great Lakes'
word **fell from 1372 to 786**. The path was the reason: eleven waypoints
against the original's eight, swinging ten cells west and back where the
original walks seven south-east.

The pathfinder's own entry log named it in one word: `scouting: true`, on a
citizen. `find_wpath@00688fc0`'s predicate is `type->role & 0x10` **and** the
order is `EXPLORE_TO`; this crate tested the order alone. `scouting` prices
seen ground at `0x400` against unseen `8` — the whole of "exploration seeks
the unexplored" — so any unit handed an `EXPLORETO` walked *toward the fog*.
`UnitTypeData::role` is `+0x2c8`, `UnitType::determine_roles@0061c320`
derives it, bit `0x10` is `is(SCOUT)` on land and `is(BARK)` at sea, and
`crate::ai_load::role::SCOUT` had been carrying exactly that number on every
type since the AI loader landed. One `&&`.

No capture had ever exercised it, and that is the point: until the borders
were right, no non-scout in either game was given an explore order over any
distance at all.

### What is left, and it is booked

The site record's residue is **one slot**: where the original's 5×5 leaves
the centre `(48, 29)` for `(47, 28)`, this crate keeps the centre — `q = 19`
against 17 — so the original's `blocked_town` refuses a cell `site_clear`
allows. `blocked_site@00636a50` counts footprint tiles whose fog half-cell
is unseen and refuses the site when more than half are dark, with only a
Dock exempt; `blocked_tcoord` here grants visibility everywhere
(`docs/CITIES.md` §11). Writing that rule out and running it moved **not one
number** — the AI's own territory answers "seen" through the same
territorial arm — so it was reverted and booked rather than landed blind.
And the order score's new holder is `1/1` again, one field: on frame 792 the
original's `coll_x/coll_y` is `(40539, 18258)` and this crate's
`(40632, 18044)`.

### The lesson

**Grep the dump before booking a reading — and then diff the whole record.**
The queue booked this item as "the AI's site choice, `compute_sites` /
`produce_building`", which would have been a reading of two long functions.
The site scorer's own output was already on disk, per frame, for exactly the
frames that mattered; comparing it cost twenty minutes and turned a scoring
question into a *territory* question, which the same record answers with a
single integer. Neither of the two defects was in either function the item
named.

**And a fix that lowers the score may still be the right fix.** The border
correction is verified against the original's own count — 290 against 290 —
and it took the word down four hundred frames on its own, because it opened
a code path nothing had walked. The instinct to revert it would have been
wrong; what it had exposed was a second, older defect sitting behind it.

## 2026-08-31 (later still again and again, Opus) — item 115: Great Lakes' orders 791 → 1374, and a pointer this crate does not have

Great Lakes' order score was 581 frames behind its tick score, and the whole
of the gap was one unit and one field. On run10's frame 792 the AI's `1/1`
carries `coll_x/coll_y (40539, 18258)` in the original and `(40632, 18044)`
here — the same walk, the same position on every frame, a different
collision point.

### Reading the dump first

`(40632, 18044)` is not a wrong answer; it is an **old** one. The original
carries exactly that pair on frames 788 through 791, and replaces it on 792.
So the question was never "what does the original probe" but "why did this
crate stop writing".

Instrumenting the probe settled it in one run: the crate probes the same
points on the same frames the original does — `(40539, 18258)` on internal
frame 791 and `(40539, 18250)` on 792, both reproducible from the sine
table with the **guy's** angle, not the unit's, because the citizen is
mid-turn. `Sim::detect_unit_collision` wrote the first of the two into the
order and something took it straight back out again.

### The defect

`detect_unit_collision` writes `coll_x/coll_y` **into the move order**. The
original then walks the rest of `move_step` on a `MoveOrder *` and keeps
writing its own fields through that same pointer, so the probe's answer is
simply there for everything downstream. This crate steps on a *copy* —
`unit_step(u, mut mo, speed)` — and every `store_move` below the probe put
the stale pair back.

`do_move`'s own waypoint test already knew this: it has a "take the copy
back" three lines after its `detect_unit_collision`, with a comment saying
why. `unit_step` did not.

The arm where it shows is narrow, which is why it survived: the copy has to
be stored *without* the probe running again. That is §5's **blocked stand
while a turn is still owed** — `move_step` marks the idle, stores the move
and returns without stepping. `1/1` stands mid-turn on 792, and its answer
was discarded on the way out.

One `if let` after the probe, taking `mo.coll` back off the live order.

### What it moved

Great Lakes' orders **791 → 1374**, one frame short of its ticks. Nothing
else moved at all, and that is the useful part of the number: ticks hold at
1375, both first divergences hold at 1385 and 1376, every one of the
fourteen units parts on the frame it did to the frame, run33's word and
sequence hold at 1372 and its totals at 1467/1436, and East Indies holds at
1374/1373. A field that is compared every frame and nothing else — no draw,
no position, no order kind — is what a pure bookkeeping fix looks like.

Both maps' both scores are now past their own word: Great Lakes 1375/1374
of 1,772 against a word at 1372, East Indies 1374/1373 of 1,850 against
1373. What holds Great Lakes now is `1/8` — its move order's destination
`x` is `40440` here against `40248` at 1375, and its position parts at
1376 — the same unit on both scores.

### The lesson

**A port that copies what the original points at owes a write-back at every
site, not at the one that hurt.** The pattern was already in the file, with
a comment explaining itself, three hundred lines above the site that needed
it — which is the shape of a fix that is applied where it was found rather
than where it belongs. The check that now stands is a section of
`docs/COLLISION.md` §4.3 saying the store is into the order and every
caller keeps it, plus `a_blocked_stand_keeps_the_point_the_probe_refused`,
written to fail first.

## 2026-08-31 (the capture lane, Opus; intake by Fable 5) — runs 42–50 come home: 108, 96 and 57's capture half retired, and 23's XOR term fired

The second lane of DECISIONS 27 ran beside the main loop for a day and
closed out everything it was booked for: nine runs, 42–50, all seed 12345.
The record is `docs/ORACLE.md`'s "The capture lane" section and the run
sections after it; the instrument it leaves behind is
`tools/gamelog/captures.txt` — the capture queue is a file now, one stanza
per run, and `runqueue.sh` walks it, skipping stanzas whose archive
already exists, so an interrupted queue resumes.

What the runs settled:

- **108 (run42).** run39's game re-captured with `LEADERS=2` per frame —
  `samegame.py --exclude LEADERDATA` says 900 frames in common, none
  differing — shows leader 1's `bucket[2]` step 50 → 100 on sim-frame
  867, with the trace putting `find_goody_at` and `explore_goody` on 867
  and on no neighbouring frame. GOODY §3's Science reading confirmed, the
  `ages` alternative refuted (it would pay 25). One correction: the
  predicted `epoch_get(scan)` vector was wrong — `0 1 0 1`, not
  `0 0 0 1` — and only `epoch[3]` enters the formula. GOODY §5/§6 amended.
- **57's capture half (run43).** run32's scenario with the `DUMP_ALL`
  window opened four frames earlier gives the terraform its own before
  and after: 128 corners move, in exactly two 8×8 boxes on the two placed
  tiles, nothing outside — ROADS §7.1's number, now one game's own
  difference rather than run13-vs-run32 — and the corner grid is one
  corner per tile (the 4 in `(4·xs+1)` is cells-to-tiles). What remains
  of 57 is harness arithmetic: the two node counts re-read on this grid.
- **96 (run44).** 452 guy-frames play a turn animation, both directions,
  both sides; every earlier capture has zero. No driver was needed:
  `Guy::move:109`'s standing arm passes the override too, so turning
  towards a target is enough and a fight supplies one. `guy_flags` 0x8 is
  exactly the three turner types, and 0x20 — which ANIM §9 carried as "no
  writer found" — is on 12,582 records, toggling within a type. Neither
  appears in any scored game (run13 and run38 carry 16 on every guy), so
  no score moves; ANIM §4.6 and §9 amended, the override now diff-backed.
- **23, half (runs 45/46/50).** run45's negative is the load-bearing
  part: in 900 frames the AI never lays a `GroupMoveOrder` — across nine
  archives the order is a human-click artifact — so the XOR term needed
  the mouse. run46's three driven right-clicks fired it, the first time
  in any capture, and both hand-backs come out as `kill_current_order`'s
  formula says: `0 XOR 1 = 1` lays the mirrored order, `1 XOR 1 = 0`
  hands it back. The Echelon half stayed shut behind four doors, each
  closed with evidence — no console verb, no command-card button, not
  gated on military research, and the formations ship unbound: zero
  `<INPUT>` elements in `data/playerprofile.xml`, and run50's F9 bind
  into the profile did not take. Re-booked as that half alone, with the
  open question named: does anything call `KeyMap::load`'s String
  overload at `007d39a0`?

The lane's traps are written in ORACLE where the next stanza will read
them, but two deserve the chronicle. zsh's `${(j:\n:)a}` joins on a
literal backslash-n, which folded two staged `add` lines into one and
produced a 550 MB archive holding half its scenario — caught by
`cmdsran.py` on its first real outing, which is why it had been written.
And `!ffwd` stops the renderer: a driven run under fast-forward clicks at
a picture minutes stale, proved by five byte-identical screenshots across
ninety sim frames. The lesson in both: an accepted console line is not a
line that did something, and a screenshot is only evidence if something
in it can change.

Intake (Fable): merged conflict-free — DECISIONS 27's second trial
question, "was the merge cost zero", measures zero, and the entry now
carries the verdict: kept, fenced as before. GOODY and ANIM amended as
above; 108, 96 and 90 deleted from the queue; 57 narrowed; 23 re-booked;
run42's per-frame `LEADERDATA` added to the widening ledger (87). The
headline did not move and was not booked to — the lane's items were owed
captures, and all four were delivered.

## 2026-08-31 (the capture lane, Opus — a coda) — the profile keymap is read, and run50's failure was the keyboard's

Runs 51 and 52 (`docs/ORACLE.md`), booked while the lane's session was
still warm. Rebinding `OPTION_AUTO_EXPLORE` — the one action whose
binding its own tooltip prints — moved that tooltip from CTRL + E to F9,
so the profile's `<KEYS>` is read and `KeyMap::load`'s String overload at
`007d39a0` runs, whatever the export shows about its callers: a
ten-minute behavioural check settled what the caller hunt could not. And
run52 separated *loading* from *arriving*, which run51 alone would have
conflated: pressing the bound key opened the chat box, so run50's Echelon
failure was a keystroke the game received as a different key, not a bind
that did not take. Item 23 re-booked around the real blocker, with the
letter-key ladder from ORACLE as its next step. The trap the probe
tripped — the game rewrites `Player.dat` on quit, so a restore while it
runs is silently undone — is in `bindkey.py`'s docstring now.

## 2026-08-31 (item 114, Opus) — a pasture nothing stocked, and Great Lakes runs out of frames to disagree on

The word parted at 1372 on twenty-five draws this crate did not spend, and
they were one call. `Build::activate`'s gather tail switches on the *good*,
not the type, and `LAB_00625a36` — where `do_bonus(0, FOOD_BONUS_FOR_FARM)`
falls through — is the same call as the `iVar18 == 0` arm every unpaid exit
lands on: a finished **farm** runs `Farms::add_animals` whatever the bonus
did. So the AI's fourth farm, a pasture by `Farms::add`'s own rule, stocks
five animals on the frame it completes — four draws each, coin, `y`, `x`,
`Guy::init_real` — and five more follow that frame as the newborns'
`end_time 0` clocks wrap.

Every one of those sites had been read: §3.6 in August, §3.11 with the
offsets and the species, §3.8 with the correction that names
`Build::activate` as the caller and the note "no capture contains one". The
capture did contain one. What was missing was not a reading but a **caller**
— `Sim::farm_add_animals` had exactly one, the harness's own setup — and the
existing function needed only a second entry point that draws its five seeds
instead of borrowing them.

The pin came free with it. run33's frame 1372 begins on `0xc91f99f2` and the
original's twenty-first draw of that frame begins on `0xafa38116`; twenty
draws of the same arithmetic land on it, all five coins even (five
chickens), the ten offsets `rnd % 0x180 − 0xc0`, each snapped by
`Unit::init`. The test asserts the word, the twenty labels in order and the
five points, and it was made to fail by swapping the `y` draw with the `x`.

**What it moved.** Great Lakes' word and sequence **1372 → 1802**, its two
totals 1467/1436 → **1832/1830** of 1,850, and the gap between them 31 → 2.
run10's headline **1375/1374 → 1772/1772 with neither player parting at
all**: no position disagreement, no scoring order disagreement, no gather
tile disagreement, anywhere in the capture. Items 118 and 119 — `1/8`'s move
order `x` and `1/1`'s cleared `last_x/last_y` — went with it; both had been
downstream of a stream wrong since 1372. Three coverage pins that had been
scores became sizes: the collision rows 128,672 → 139,514, the angle rows
49,088 → 53,402, the first disagreeing gather tile 1,598 → **never**. East
Indies is untouched at 1374/1373; run39's AI builds no farm inside its
capture.

**What it means for the finish line, said plainly.** run10 is 1,772 frames
of one map and it has run out of frames to disagree on. That is not the
finish line `CLAUDE.md` names — two maps at full length — and it is not
evidence about frame 1,800 to 24,000. The next number on this map has to
come from a longer capture (item 91), and the map still moving is East
Indies. What run33 says about the frames past 1772 is that its word survives
to 1802 and parts on `Unit::do_air_physics+0x639` — a gaia bird's flight
physics, the only time that site is reached in all 1,851 frames.

## 2026-08-31 (item 91, Opus) — the long captures, and the 20 GB they saved

Booked as "due when a map's word matches its 1,850", and the handoff that
booked it got its cost wrong twice in one paragraph. It called item 91 a
capture session; item 90's lane had already made it two stanzas in
`captures.txt` and an unattended wait. And it sized the trace at ~150 MB
against an actual 35 MB.

**The blocker, found before anything was spent.** `longtrace.sh` polls
`for i in {1..160}` at twenty seconds — 53 minutes — and its end is a
`pkill` and an archive, not a stop. Every capture to run52 fits inside it.
A 24,000-frame run does not, and what it would have produced is a truncated
archive indistinguishable from a finished one — a confident wrong answer,
which is the expensive kind. `POLL_MAX` is a hook now (default 160, so
nothing existing moves), `poll_max:` a stanza key, and the give-up path
announces its own truncation. `settle_min` needed the same care: a
`MISC`-only per-frame block never reaches the 10 MB the settle test wants.

**What the captures cost, which is the finding.** runs 53 and 54, one per
map, 24,000 frames each, trace whole: **eight minutes and 70 MB for both**.
run33 is 1,850 frames and took the better part of an hour. The sim was never
the bottleneck — unrendered under `!ffwd` it runs about 240 frames a second
— it is the per-frame dump, 155 KB a frame at run33's detail. Cut
`[End Frame]` to `MISC` and the floor is gone. `rngcmp.py` says both new
traces are 0-differing against their siblings over all 1,851 overlapping
frames, so they are the same games with twelve times more of them.

**And the answer inverted the plan.** The question the capture was taken to
settle was how big a full-detail dump to buy, with 20–30 GB authorised. On
thirteen times the frames Great Lakes' word **still parts at 1802** — the
same frame run33's 1,850 reported. So a full-detail 24,000-frame dump buys
about **thirty** frames of new measurable ground past run10's own 1,772, and
then twenty-two thousand frames of a stream that is nobody's. The budget
would have bought thirty frames. It is not spent, and the rule that falls
out is: **size the dump to the word, and take it after the word moves, not
before.**

`run53_s_24000_frames_put_the_ceiling_where_run33_did` pins the two frame
numbers and **prints rather than pins the two totals** — past 1802 they are
coincidence at about one frame in six, and item 89(c) is precisely the
warning against pinning a non-monotone score as monotone. Made to fail by
asking for 1803.

What the capture leaves behind is the successor made cheap: item 120, the
one draw at `Unit::do_air_physics+0x639`, is now scoreable over 24,000
frames without taking another capture at all.


## 2026-08-31 (item 110, Opus) — a list nobody dumped, in the grid's own order

East Indies' word at 1373, booked as "the AI scout takes a seventh ring".
It takes no seventh ring. The queue's own description of the frame was a
**mislabelled draw**, and finding that out was most of the item.

**The mark that was not there.** `Sim::scout_region_scan` had, since the
mechanic landed on 2026-08-26, taken §11's stride draw and nothing else —
`self.rng.roll()` with no `self.mark(...)` beside it. `mark_sites` builds
our side of the sequence from the marks and the words *between* them, so a
draw with no mark of its own is not omitted: it is attributed to whatever
label is still standing. Ours was `SITE_PHASE`, the ring walk's second
draw. The frame therefore read as `436 458 ×6` and then a lone `458` —
exactly what a seventh ring looks like — where what it actually was is six
ring pairs and then the fallback. **A draw without a mark is a lie, not a
gap**, and this one sent an item to the wrong mechanic for a day.

**What frame 1373 is.** The AI scout `1/0` walks its own city's six rings
and takes **not one** `+0x64c`: by 1373 every cell within twelve of that
city has been seen. So the city loop comes out with `best` still at
99,999,999, the tail's `199 < best` sends it to the region fallback, and
the original spends `+0x941` once and `+0xaba` five times there.

**The list nobody dumped.** §11 had been "read, not implemented" since the
day it was written, because the scan strides through `Region.coords` and
that array is built by the map generator's flood fill, which no dump
carries. The document said so, and `docs/SYNC.md` §6 carried it as an open
item, and the note under it read "*Capture:* none would help".

It is not the flood fill's order. `Regions::find_all@0067eff0` appends as
it floods, merges, sorts — and then, in its **last four statements**, frees
the list and calls `Regions::rebuild_coords@0067f800`, whose whole body is

```
for (y = 0; y < ys; y++) for (x = 0; x < xs; x++)
    coords[wdata[xs·y + x].region][count++] = (x, y);
```

a row-major sweep of the cell grid. The order is the grid's, the `WORLD`
dump's per-cell region map is enough to rebuild every list exactly, and the
five cells fall out draw for draw on the first run.

Total cost of the thing that had been unrecoverable: one
`grep -l rebuild_coords` over the export. The lesson is `CLAUDE.md`'s own,
in a shape it had not been said in — **grep the writers of every field you
call unrecoverable**, not only the ones you call frozen. A field can be
written twice, and it is the *last* writer that decides what a reader sees.

**What moved.** East Indies' word **1373 → 1570**, its ticks **1374 →
1477** and orders **1373 → 1476**, and its gaia positions **191,173 →
191,876** of 192,504 with the first parting 1381 → **1658**. Great Lakes is
untouched — run33's scout never leaves its city loop. The new parting at
1570 is an anim wrap (`Guy::set_anim+0x97a < Guy::inc_time+0x271`), a
different mechanic and the successor.

**What the frame does not settle.** Three of §11's constants — the `× 16`
distance scale, the `score *= 2` at `005f6bb9`, and the sense of
`local_74` — were each inverted in turn and the word held at 1570. The five
cells order the same way either way, and `best` is still 99,999,999 when
the scan starts, so nothing on disk compares a region score against a city
one. They are read from the listing and §13 item 1 says so, with the shape
of the capture that would separate them.

## 2026-08-31 (item 121, Opus) — one frame in the art loader, one in the order, and they had been hiding each other

Item 121 was booked as "East Indies' word at 1570, and an animation that
wraps": one draw at `Guy::set_anim+0x97a < Guy::inc_time+0x271 <
Unit::inc_time+0x3e`, ours three against theirs four, with the note that a
`GUYS=4` window over 1565–1572 would name the unit. No capture was needed.
The frame is a *citizen's* wood-dump animation running out, and it ran out
a frame late for two reasons that cancel almost everywhere.

**The first frame: a non-looping animation drops its last key.**
`AnimMgr::force_load@0053ade0` does not convert the root node's last key
time. It converts the last key of a **looping** animation and the *second
to last* of every other one:

```
ms = key_times[n − 1]
if loopings[i] == 0 and ms != 0: ms = key_times[n − 2]
frames[i] = round(ms · 3 / 200)
```

`loopings[i]` is the section of `anim_graphics.xml` the row sits in —
`GraphicPieces::init_anims_pool@008fca40` calls `AnimMgr::add(name, 1)`
under `<LOOPING>` and `add(name, 0)` under `<NONLOOPING>`, `<BUILDING>`
and `<PATH>` alike, and `AnimMgr::add@0053ac00` writes
`loopings[i] = param_2 != 0`. `lumberjack_dump.bha`'s keys end 2157, 2190,
so `CHAR_DUMP_WOOD` is 32 frames and not 33.

**And the corpus had said 32 all along.** Every `GUY` block in every dump
prints `end_time 32` for `cur_anim 27` — 756 of them over the four citizen
pieces `0`, `352`, `6336` and `6688`. The widening test that should have
caught it, `the_install_s_piece_lengths_match_the_dumps`, read five dumps,
and all five are starts or short windows: between them they print only the
idles, the walks, the chop, the sow and the reap. **The list was the
assertion, and it was five dumps too short.** It is eight now — run25 for
the attacks, both dumps and the ore half, run44 for the turns and
pack/unpack, run27 for `CHAR_WALK_WITH_ORE` — 187 rows over sixteen slots,
and it fails on the old arithmetic with
`gamelog-run44-islands-turners.txt: piece 352 slot 27 says 32`. That is
item 87's ledger paying for itself a fifth time.

**The second frame: an order that turns loses that frame's animation.**
With the length fixed the word fell to **572** — seven earlier wood dumps
now wrapped a frame early. run44's `DUMP_ALL` window says why, in three
rows of one citizen:

| frame | `wait` | guy |
|---|---|---|
| 541 | 32 | `26` 14/15 — the carrying walk |
| 542 | 32 → **31** | `26` **1/15** — at the camp, and the walk *restarted* |
| 543 | 30 | `27` 1/32 — the wood dump |

The order ran its at-camp branch on 542 — the `wait` proves it — and asked
for `CHAR_DUMP_WOOD`, and something overwrote it with the walk in the same
frame. That something is `Guy::move`'s turn arm, and the reason it fires is
the third argument of `Unit::set_angle`: a **snap flag**, which
`Guy::set_angle@005d9010` uses to decide whether the guy's own `angle`
follows its new `des_angle` or is left for the turn. **Every `set_angle` in
the ordinary order path passes zero** — the two in `do_non_flat_gather`,
`do_build`, `do_gather`, `fight`, `move_step`'s three, both in
`do_group_attack`, `do_attack_ground`, `do_form_change`, `check_idle`. Only
`do_move`'s re-face, `go_inside`, `do_spec_anim`, the debug jump and the
scenario loader set it.

So on the frame an order turns its unit, `Guy::move` finds the body at its
destination and owed a turn, and spends `set_anim(CHAR_WALK, 0, 1)` on top
of whatever the order asked for a moment earlier. This crate's
`Movement::set_facing` snapped facing, heading and `des_angle` together, so
the arm never fired anywhere. `Movement::set_heading` is the zero-flag call
and the six order sites use it now.

`docs/ANIM.md` §4.6 had *read* this arm on 2026-08-28 and left it
unmodelled, on the grounds that for a builder it cancels — the next frame's
`do_build` puts the animation back before anything reads the slot. It does
cancel there. It does not cancel for anything whose animation is a clock.

**What moved.** East Indies' word and its sequence **1570 → 1647**; ticks
1477 and orders 1476 unchanged; gaia 191,876 of 192,504, first parting
1658. Great Lakes is untouched — word 1802, ticks and orders 1772 of
1,772, gaia 74,040 of 74,040.

**The successor is not this family.** Frame 1647 is nineteen draws against
four: two `Guy::set_anim+0x97a < Unit::do_idle+0x7d` and then a whole AI
scout pass — `Unit::think_scout+0x436`/`+0x458` six times with one
`+0x64c` — that the original does not run on that frame at all. The
original spends nothing there but its four farm draws. A scout thinking on
a frame the original does not is a cadence question, not an animation one.

**Two things left open, both in `docs/ANIM.md` §9.** The install now gives
the loop flag per animation *file*, and by slot it is not a constant: 42
`<UNIT>` entries give `CHAR_DUMP_WOOD` a non-looping file and 38 a looping
one. `anim::non_looping` is still a rule by slot, and it agrees with every
piece a capture has reached. And a walking guy's clock: run44's citizen
counts its carrying walk 1…14 and restarts it on arrival, where this crate
re-issues the walk every frame and the clock sits at 1. Nothing observable
turns on it — a looping walk's wrap draws for nobody but a bird, and a bird
never reaches `guys_follow`.

**The lesson, and it is the working agreement's own.** A guard that reads a
corpus is only as good as the corpus it is pointed at. This one had teeth,
had failed twice before, and still let a wrong length stand for a month
because the eight slots that would have caught it were in dumps the list
did not name. *Widen the list before trusting the guard* — and the cost of
widening it was three strings and four seconds of test time.

## 2026-08-31 (item 123, Opus) — the scout's frame-1477 path, and the two columns of the record nobody compared

**The item, and what it turned out to be.** East Indies' word parts at
1647, where this crate spends nineteen draws — two `Guy::set_anim+0x97a <
Unit::do_idle+0x7d` and a whole AI scout pass — against the original's
four farm draws. It was booked as a cadence question. It is a **route**
question, and the whole of it happens 170 frames earlier.

Run39's AI scout `1/0` arrives at its explore target on frame 1476,
`think_scout` picks the next one on 1477, and `Group::action_move_near`
plans one world path for it. The original's stack is the goal and **six**
nodes — `(47,44) (46,43) (45,43) (44,43) (43,43) (42,42)` — north around
the mountain band at `(45..46, 44..45)`. This crate's is the goal and
**four** — `(46,46) (45,46) (44,45) (43,44)` — south around the same band.
Five steps against seven. So the scout arrives on **1647** where the
original arrives on **1653**, and spends its ring draws six frames early;
that is the word's parting, and it is also player 1's position parting at
**1478**, one frame after the order. One search, three numbers.

**Reproduced in one line.** Refusing the step `(47,45) → (46,46)` — or the
one after it — makes this crate's search return the original's seven
entries **exactly**, position for position. The whole difference is the
southern corridor's entrance and nothing downstream of it.

**What it is not, and this is the expensive half.** Each of these was
measured against the original's own record rather than argued about, and
each is now written into `docs/PATHFINDER.md` §12 so nobody pays for it
twice:

- the **fog** — the scout walked cells `(43,44)` through `(47,44)` itself
  between frames 1285 and 1476, so both sides have them lit; forcing
  `(43,43)` dark yields a *third* route, not the original's;
- the **danger map** — run38's `danger[8][900]` is zero everywhere but the
  two bases, blocks `(2..5, 2..5)` and `(24..27, 24..27)`;
- the **buildings** — this crate's fourteen at 1477 are the dump's,
  position for position, player 1's second city at tile `(180, 188)`
  included;
- `get_estimate@00688310`, the stop test at `00684086`, the node key
  `length + estimate`, and §5.1's corner-cutting, whose gate is the
  destination cell's `tcost` and which no cell on either route reaches.

What is odd, and worth carrying: under this crate's **own** cost model the
original's route is the cheaper of the two — 661 against 672 — and the
search still misses it, because the two unseen cells at its end make the
heuristic an over-estimate there (a dark cell costs a scout 1 where the
heuristic charges 60 a cell), so the northern `(43,43)` carries `value` 768
while this crate's arrival `(42,43)` carries 732 and pops first. Two sides
can agree on every step's price and still return different routes. The
check that settles it is a per-step cost dump from the original — an
`int 3` on `calc_cost@00684e50` in `tools/trace`, recording
`(from, to, dir, return)`.

**What landed instead, and it is the working agreement's own rule.**
`PATHDATA` prints four numbers per row and `rondata::diff` compared two of
them. `OrderMismatch::PathField` now scores `tolerance` and `flags` beside
`PathTo`'s point. No floor moved — of run39's 24 disagreeing rows the first
is frame **1518**, past that capture's score — but the census the guard
prints is an oracle in itself. Over run39's **1,724** multi-entry stacks:

| where | shape | count |
|---|---|---|
| bottom | `(0, FINAL)` — the order's own goal | 1,724, no exception |
| top | `(384, 0)` — §7's world reconstruction | 1,408 |
| top | `(0, 0)` — §4.4's collision rewrite, `big_radius × 3` | 243 |
| top | `(0, SIDESTEP)` — the unit grid | 73 |
| middle | `(384, 0)` / `(0, SIDESTEP)` | 2,325 / 54 |

Three shapes and no others, and all three are already modelled. The counts
are the dump's own, so `diff::tests::a_path_stack_s_rows_are_compared_whole`
pins them exactly rather than as a ceiling.

**A methodological cost worth recording.** The shape above was first read
off a hand-written Python pass over the dump, which said every stack top was
`(0, FINAL)` — and built an hour of theory on it about a second goal entry
and a search that started a cell further north. It was a parser bug: the
`MOVEORDER` block that follows a `STACK<TYPE>` carries its own `tolerance`
and `flags`, and the scratch parser was still writing into the last
`PATHDATA` when it reached them. The crate's own parser, which is asserted
against run20's nine-entry chain, said `(384, 0)`. **Use the harness's
parser, or assert the scratch one against a case the harness already
pins.**

**What moved.** Nothing. East Indies' word stays 1647, ticks 1477, orders
1476; Great Lakes 1802 and 1772/1772; gaia unchanged. The item bought a
diagnosis, a widening, and a list of six things the next session does not
have to rule out.

## 2026-08-31 (a Fable meta session) — items 89a and 89b become guards, and the queue stops golfing

Not a mechanic and not a steering pass — the fourth ran this morning and
the headline has moved every session since, so nothing was owed. This was
the other half of the working agreement's split: Opus runs the loop, this
session improved the loop. **No number moved**, and the tranche verified
green before anything was touched: 162 rondata tests in release with the
install and the bottle's dumps attached, zero `skipping` lines.

**Item 89a — the handoff's numbers are now parsed against the floors.**
`rondata::diff` gains `FLOORS`: the two maps' ticks, orders and word as
named constants, and the six scoring asserts (both pairs, both words,
run53's ceiling) now read them instead of their own literals. Beside it,
`the_handoff_s_scoreboard_is_the_floors` — a static test, no dump needed —
parses a fixed-format `Scoreboard:` line in the queue's handoff and
requires equality. Landed failing-first: the test ran red against the
line-less queue before the line existed. The error class it closes is the
one that cost the most this week: item 69's two numbers describing two
different simulations sharing a file for seven days, and the fourth pass
finding the handoff calling the wrong number the headline. A floor that
moves now moves three things together or the suite names the laggard.

**Item 89b — every `name@00xxxxxx` a spec cites is checked against the
export.** `docs_guard` gains `every_cited_address_names_its_function`: 474
distinct citations across the specs, each address looked up in
`INDEX.tsv`, the export's name (templates stripped) required to match the
cited one as a suffix in either direction. Addresses past the last
function (00ac4226) are data, which a function index cannot check — the
four `PATHDATA` tables live there. `JOURNAL.md` and `docs/audit/` are
deliberately unread: both tell stories *about* wrong citations. Skips
loudly without the export. Landed failing-first on two planted cites — a
typo'd address and a wrong name — and the calibration run taught the two
extraction truths worth keeping: a cite whose `@` follows an argument
list or a line wrap carries no adjacent name (the address half still
checks), and the export is sometimes *less* qualified than the document
(`get_new_order` is a bare global there). This is item 72's
`name@00xxxxxx` half made permanent; the `+0xNN` half stays booked.

**The queue's cap moved 180 → 200.** The file sat at exactly 180 and the
sessions were golfing lines to add an item. The board holds ~20 live
items whose booked shape is five or six lines each, plus a 32-line
handoff and the maintenance rules — that saturates 180 with no bloat
anywhere in it. The bound should bite on stories and changelogs, not on
the working agreement's own item size; the guard's comment carries the
reasoning. The queue was also trimmed where it duplicated cited doc
sections (items 23, 117, 125's diagnosis now lives only in PATHFINDER
§12 and ORACLE's run notes).

**The README's status paragraph caught up with the day**: one scored map
now runs a full human-versus-AI capture in lockstep for its whole 1,772
frames — that sentence was nowhere outward-facing before this session.
Entry 27's capture-lane ledger was checked and owes nothing; the only
open `FABLE:` row (orders R4) already travels with item 85.

## 2026-09-01 (item 125, Opus) — a function's own answer, and the field nobody grepped the writers of

**The scored line closed today.** East Indies went from 1477/1476 to
**1851/1850 of 1,851** with **neither player diverging anywhere in the
capture**, and its word ran to the end of run39. Great Lakes has stood at
1772/1772 of its own 1,772 since item 114. That is phase 3's stated finish
line — a fixed-seed, traced human-versus-AI capture on two maps, no
position or order disagreement, for its full length — and the headline
re-pins tomorrow to the long captures, where East Indies' word is **2176**
of 24,000.

### The instrument came first, and it is the reusable half

Item 125 had been booked with its own check written down: an `int 3` on
`calc_cost@00684e50` recording `(from, to, dir, return)`, because
`docs/PATHFINDER.md` §10 had established — correctly, and at the cost of a
full Opus survey — that the game prints no per-search number anywhere. The
`PATHFINDER` gamelog category emits one line at map generation. The two
`dbg_*` printers are gated on a flag nothing in the binary writes.
`PathFinderData::log_data` needs the `DUMP_ALL=1` that hangs the game.

What the question actually wanted was not a breakpoint but a **proxy**. A
draw hook logs and falls through, which can never give a return value; an
`int 3` says a function was entered and nothing more. So `tools/trace`
grew a third instrument: `rontrace.cfg`'s `callwin=LO-HI` replaces a listed
function with a stub of its own signature that logs the arguments, calls
the original through the displaced-prologue trampoline, and logs `eax`.
The arguments live in the proxy's own frame, so recursion and re-entrancy
cost nothing. Two sites are proxied — `PathFinder::astar_path`, whose
entry and return **delimit one search**, and `calc_cost` itself. Without a
`callwin` nothing is patched, so every earlier capture still reproduces.

Three things kept the cost of that down and are worth repeating. The
machine code was written out by hand and **disassembled with
`llvm-mc --disassemble` before the game was ever launched** — the encoding
was right first time. A **sixty-frame smoke run** proved the proxies did
not crash the game before the seventeen-minute capture was committed to.
And the callee-clean `ret <imm>` for each site came from the **PE bytes**,
not from the decompiler's argument list.

run55 is run39's game exactly — same lobby, seed and detail, 1,500 frames,
`cover=0` and `callwin=1460-1490` — and `rngcmp.py` says its `game_random`
word is run39's on all 1,501 overlapping frames with **zero** differing.
The proxies cost the simulation nothing, and that is asserted rather than
assumed.

### What it found, in one reading

Over the whole thirty-one-frame window the game ran **one** search: the AI
scout's, 110 `calc_cost` calls on sim-frame **1476**. (Every document
before today called it 1477, from the dump block it lands in; the trace
counts `Game::frame`. The first `calls` listing came back empty because of
it.)

Of the 110, **103 already agreed** with this crate's own answer for the
same argument list. All seven that did not were steps into the same four
cells, and they said one thing twice: `+176` on each, and one outright
refusal. `176` is `20 × 9 − 4` — the terrain term for a cell nine of whose
sixteen tiles are blocked, less the own-territory discount. Player 1's
second city stands on tile (180, 188), which is cell (45, 47), and its
footprint covers nine tiles of each of `(44,46) (45,46) (44,47) (45,47)`.

So **`WData.blocked` is a running count of the cell's blocked tiles**, and
`World::set_blocked_at@006b4900` is its only writer — the same function
that keeps `WData.solid` beside it, clears the tile's own `BAD_PATH` and
road, and spreads `BAD_PATH` onto all eight neighbours (with `WData.bad`
counting that per cell). Every caller in the executable goes through it:
`BuildType::mask_me` for a footprint, the mountains, the cliffs, a
`Good`'s own tiles, a packed siege engine. This crate set the tile bit by
hand and left the count at zero, so the pathfinder charged an empty field
where the original charges nine sixteenths of one — and the scout walked
**through** the city and arrived six frames early.

### The lesson, which is the audit README's own

§12 had ruled out six candidates against the original's record — the fog,
the danger map, the buildings' positions, the cell records, the estimate,
the stop test — and every one of those exclusions was correct. The
seventh, which nobody named, is that **a cell record can change**. The
terrain cost had been read, implemented, audited, and diffed, and each of
those passes treated `WData.blocked` as a property of the map, because the
frame-0 dump it is loaded from *is* a map. One `grep` for the field's
writers names `set_blocked_at` in a second.

§12 had also carried a reading that turned out to be wrong in its premise:
that under this crate's own costs the original's route was the *cheaper*
one (661 against 672), so two sides could agree on every step's price and
still return different routes. They did not agree on every step's price.
That entry is struck and replaced with what run55 says.

### What it cost elsewhere

Two floors moved forward rather than back, which is what a real convergence
looks like. run39's bird-landing list gained four entries (1736, 1776,
1784, 1800) because the frame the two streams still agree to moved, not
because a bird changed. And one synthetic road test was measuring a
*column* where the mechanic produces a *path*: the `BAD_PATH` halo a
blocked tile now spreads costs `weight::GROUND` and pushes a road a tile
off the straight line, so the assertion was rewritten to check that the
road runs rather than where.

`run54` — East Indies at 24,000 frames, taken with run53 and unread for a
day because while run39's word parted at 1647 no longer capture could say
anything — is read now, and it names the successor by its own frame: at
2176 the original spends **192 draws** at
`Build::find_gather_tiles+0x10a < Build::init+0x55b`, a gathering building
surveying its tiles, and this simulation spends none of them. That is item
85, booked off run40 a day earlier, now the leading map's first
divergence.

## 2026-09-01 (Fable steering) — the re-pin ratified, and one map earns the expensive capture

The steering pass the morning's Opus session called for, taken the same
day. Its questions and their answers, in order:

**Is the tranche real?** Yes, and it is the milestone: both scored
captures at their ceiling under the suite's own asserts, the sentence in
the README, the headline moved to `LONG_WORD_EAST_INDIES = 2176` with
run54's assert behind it. Nothing about the re-pin's shape needed
changing — `FLOORS` stays the scored captures' scoreboard (a fall there
is a regression, not a score) and the long word is a separate pin, which
is the right split.

**Entry 29's open half — which full-detail captures to size to the new
word — is decided.** East Indies takes one, 3,000 frames, as item 85's
first act: its word crossed the scored capture's 1,851, which is ORACLE's
own trigger, and item 85 (a camp's gather survey) changes exactly what
the new records would check, so diff-first says take the records before
reading. Great Lakes takes none yet — its word (1802) still sits inside
run33's 1,850 full-detail frames, and item 120's bird is what moves it.
The rule as ratified: a map earns its next expensive capture when its
word crosses the newest one it has, sized to the word with headroom.

**No ratification batch is due.** The accrued set is two rows: orders R4,
which travels with item 85 and is settled by its implementation; and R2
O1, on which nothing in `crates/sim` depends. The last pass was
2026-08-28 and the marker discipline has kept the set near zero since.

The queue was reordered to lead with 85/82 — the headline's first
divergence is the default item, and the list now says so — and the next
opener written for Opus: capture first, rngcmp against run54, then the
mechanic.

## 2026-09-01 (Opus) — the gather list is a mechanic, and item 85 splits in two

The morning's steering pass sized a capture and named an item; both are
done, and the item turned out to be two.

**run56.** East Indies, 3,000 frames, run39's recipe and detail unchanged —
the first full-detail capture on either map that reaches past its own
word. It cost thirty-five minutes and 789 MB. `rngcmp.py` calls it
run54's game on all 3,001 frames and `samegame.py` calls it run39's on all
1,850 they share, so it inherits both.

**The widening came first, and it was a whole record.** `Frame.builds` has
been parsed on every frame of every capture since the parser existed and
compared on *none*: `compare` walked units and only units. `gather_from`
— the tile list the slot count is surveyed out of — went past 594,618
times on run39 alone without anyone looking at it. It is compared now,
entry for entry, beside the `MiningList` header's `length` and
`BuildData::gather_down`, for every building of both players on every
frame. run39 and run33 both come back clean: the only disagreements are
four, on each run's own last frame, where the `!quit`'s end-of-game block
clears the human player's four `gather_down`s and leaves the AI's
untouched. That is a teardown, and the tests say so rather than pinning a
number they do not understand.

That both maps agree is the expected answer rather than a null one. Both
maps' camps are pre-placed and their lists come from the dump, so agreeing
costs the simulation nothing — which is exactly why the capture had to be
longer than either.

**`Build::find_gather_tiles@00623350` is implemented.** `Build::init`
branches the way the original's does: a gather type that is neither flat
nor the university fills its list, marks every tile `0x1000`, shuffles
it, and only then computes `gather_max`; everything else takes the plain
survey. Before this, a camp placed during a run surveyed its own still
empty list and activated with zero slots — `docs/ECONOMY.md` had it as an
open item and the queue as item 85.

Three things establish it, and none of them is a reading:

- **The tiles.** Clearing the `0x1000` marks off run39's two pre-placed
  camps and running the walk at each corner returns *exactly* the dump's
  73 tiles, both times, on a map the code had never seen. It also settles
  what the decompiler dropped. Ghidra prints the inner loop with only the
  `0x1000` test in it, which would make a qualifying cell worth all
  sixteen of its tiles; the record disagrees twice over — six cells
  holding 16, 12, 12, 12, 12 and 9 forest tiles list 16, 12, 12, 12, 12
  and 9 — so the loop takes the cell's **trees**, as `calc_gather`'s own
  survey does.
- **The shuffle's arithmetic.** `4 × length` rounds, one draw each.
  run54's trace spends **584** draws at `Build::find_gather_tiles+0x10a`
  during setup, where those two 73-tile camps are placed: 4 × (73 + 73),
  to the draw.
- **The order, seed-anchored.** run56's frame 2176 is the one camp the
  *game* built — player 1's `o 2009`, a Woodcutter's Camp at tile
  (198, 190), 48 tiles, 192 draws. Step to 2176, install the original's own
  word from the trace (`find_gather_tiles` is the first thing that frame
  draws, so the anchor is exact), place a camp where the original placed it,
  and the list comes back **entry for entry** in the original's shuffled
  order for exactly `4 × 48` draws. That is the walk's order, the marking,
  the round count and the modulus in one assertion, and it is the half the
  re-derivation cannot reach.

**And what is left at 2176 is not the list.** The frame's 227 draws are
this simulation's 35 plus exactly the 192 the shuffle costs, and the
simulation places no building there at all. What it does place is
uncanny: player 1's farm on frame 2, its second city on 977 and its
second farm on 1577, each at the original's own tile and object number,
frame for frame — the widening now compares 967,268 gather fields over
run56's 3,001 frames and the *only* rows are that one camp and the quit's
own four. Then the original places `o 2009` and this one does not. So item 85's second half is an
**AI build decision**, not a gather mechanic, and it is booked as its own
item with the three placements it already gets right as the evidence that
the path is nearly there.

Also fixed: `runqueue.sh` never reset `poll_max` between stanzas, so every
stanza after run53 silently inherited its 900. Harmless in effect — it can
only make a run wait longer — and exactly the kind of thing that is
invisible until a capture needs the default.

## 2026-09-01 (item 126, Opus) — a camp is scored by what it would gather, and the headline moves 2176 → 2665

Yesterday's session left East Indies' word at 2176 with the divergence
named precisely: the original places player 1's second Woodcutter's Camp
there — `o 2009`, tile (198, 190) — and this simulation places nothing,
though it puts that player's three earlier buildings up at the original's
own frames, tiles and object numbers. The queue's guess was `make_stuff`
slot 4's gather exception. It was not that at all.

**The script says so in one line.** Tracing every `ScenarioFuncSet` call at
frame 2176 shows the AI arriving at exactly the right place: script step 13,
`place_woodcutter` in `aibestbuildlibrary.bhs`, the `num_cities > 1` arm,
`place_building_with_cost(who, "Woodcutter's Camp", "City 2")` — and it
comes back **0**. Then the capital, also 0, then `can_pay_cost` says yes and
the step advances. So the AI wants the camp, can afford it, asks for it at
the right city, and `Leader::produce_building` refuses every site on the map.

**Every candidate scored zero, and the reason was a name.**
`produce_building`'s woodcutter branch is

```
score = score · n³;   if not (n > 2 or frame == 0): continue
```

and `n` is `local_34`, filled by the `blocked_site` call the spiral has
*already made* for that candidate. Follow it down: `blocked_site` passes it
to `blocked_location`, which writes it once, at the very end, from
`BuildTypeData::calc_gather`'s count out-parameter — **the same number
`max_gatherers` reads**, which this crate has had for a fortnight. This
crate had a one-tile ring of forest tiles there instead, `forest_around`,
plausible from the `count_trees_adjacent` sitting two branches above in the
same function and wrong: a camp stands on *clear ground next to* a forest
whose gather radius is eight tiles, so the ring of width one is empty at
every site a camp can actually occupy. Score zero, gate refuses, no camp.

Frame 0 is why nobody noticed. `frame == 0` skips the gate, and a
uniformly-zero score makes `score < best` false for every candidate, so the
spiral accepts its *last* candidate rather than its best — which is a
perfectly deterministic wrong answer that the setup path does not exercise,
because `Setup::small_city_buildings` is not modelled and the initial camps
are loaded from the dump. The AI has been unable to build a woodcutter's
camp for the whole life of the crate, and no capture was long enough to say
so until run56.

**What landed.** `blocked_location`'s tail whole, not just the number:
`Sim::gather_verdict` refuses a non-flat gather type with nothing under it
— `NoForest` for a camp, `NoMountain` for a mine, `NoResources` otherwise,
and the two `Taken` verdicts when the count comes back negative because
every candidate cell was already gathered from. `Sim::blocked_site_slots` is
the form that hands the count back, and `produce_building` reads it in the
two places the original does: the spiral's score, and the camp's compass-ring
jitter, which picks its sub-position on a strict improvement in the same
number. `Sim::site_gather_count` is the unclamped count; `gather_slots` and
`max_gatherers` keep the clamp, because `blocked_location` reads the sign
and `max_gatherers` does not.

**The guard fired on its own tests first.** Seven flat-world unit tests
failed on the first run — every one of them standing a woodcutter's camp on
a map with no trees, which is now correctly `Blocked::NoForest`. That is the
rule working. They plant a cell's centre tile of gatherable forest in the
eight cells around the camp now (`Sim::plant_camp_forest`, one tile a cell,
because the survey qualifies a cell on its centre tile and nothing else), and
`ai_make`'s fixture farm gained the `FLAT` flag the shipped data gives it —
without which a farm is a non-flat gather type and gets refused too.

**The result.** The AI reaches run56's frame 2176 and places `o 2009` at
tile (198, 190) — the original's frame, the original's tile, the original's
object number — and spends the original's own 192 shuffle draws. run56's
gather widening goes from 967,268 fields with fifty rows on the missing camp
to **1,048,118 fields with nothing but the quit's own four**, and East
Indies' long word moves **2176 → 2665**.

**And 2665 is a scout that will not stop thinking.** Both take a whole
`Unit::think_scout` on 2664 — ten `+0x436`, six `+0x458`, one `+0x941` — and
agree on the frame. On 2665 the original takes nine draws; this simulation
takes thirty, and twenty-one of them are a *second* whole `think_scout`. The
`+0x458` count is the fingerprint: SCOUT §6 skips the phase draw when
`ring / 4 + frame % 8 == 0`, so 2664 (`% 8 == 0`) spends six and 2665 spends
ten — which is exactly what the two lists hold. The walk is right; the unit
is still idle when the original's has left `do_idle` with its order, and from
2664 the original's scout thinks every 32 frames exactly. That is item 127.

## 2026-09-01 — `Unit::think` has two cadence gates, and the second one is the tail's (item 127)

**The premise was wrong and the frame was right.** Item 127 read 2665 as "the
unit is still idle when the original's has left `do_idle` with its order".
run56's own `UNITDATA` says otherwise on the first look: player 1's `o 0` is
`idle 0` through 2664, `idle 1` at 2665, `idle 2` from 2666, `3` from 2673 and
`4` from 2689 — it never leaves `do_idle` at all. The original's scout
*stays* idle and simply does not think, and the 2673/2689 pair (sixteen
apart, `o == 0`) is what says the dump row labelled `F` is the state at the
**start** of frame `F`, which is worth more than the item was.

**So the gate was the question, and `Unit::think` has two.** The one this
crate had is at the head — `(flags & 0x10) == 0 && idle > 2 && ((o + frame) &
15) != 0` returns — and a unit on its *second* idle frame passes it, because
`idle > 2` is false. The one it did not have sits after the human block's
`unit_masks & 0x40000` exit and in front of `think_fish`:

```
if (idle != 1 && ((o + frame) & 31) != 0) return;
```

Everything from there down is the tail — `think_fish`, `think_merchant`,
`think_carry`, `add_to_army`, `think_scout` — so a standing scout thinks on
its **first** idle frame and then once in thirty-two, phased by `o`. run56's
scout arrives on 2664 with `idle == 1`, thinks, and thinks next on 2688,
2720, 2752 … to the end of the capture, and `report.py … draws` counts
exactly those frames under `Unit::do_idle+0x94`. `think_peasant` sits above
the gate, at `LAB_005f7179`, which is why a citizen keeps its own cadence
and the scout does not.

**`docs/ORDERS.md` had the gate written down since the second reading** —
step 5's "everyone on `idle == 1`/32" — and the code had never carried it.
That is item 72's shape exactly: a document and its code disagreeing is a
diff waiting to be run, and this one was worth nineteen draws a frame.
`scout::tests::an_idle_scout_thinks_on_its_first_frame_and_then_once_in_
thirty_two` is the guard; made to fail first, it reports the old behaviour
verbatim — every frame of `idle <= 2`, then every sixteenth.

**The word did not move.** 2665 went from thirty draws against nine to
eleven against nine, and the same frame carries a second divergence
underneath: **the dog**. A scout is two guys — `Unit::set_anim`'s two loops,
`0..guy_mark` at `+0x56` and `squad_size..num_guys` at `+0xb6`, which is how
the trace tells them apart — and run56's per-frame `GUY` records carry `x`,
`y` and `angle` per guy. Guy 0 arrives on 2664; the dog is at (40365, 34367)
and walks to (40352, 34323), (40339, 34279), (40326, 34235) and (40322,
34217), arriving on 2668 and settling its angle on 2669. The original's
`+0xb6` draws land on exactly 2664, 2668 and 2669 and nowhere between,
because `Guy::set_anim`'s walking-guy early return tests **that guy's** `des`
against **that guy's** position. This crate has one body per unit and hands
it to every guy, so the dog re-rolls on 2665 and the frame runs long.

That is item 128, and it is `Guy::move`'s tracked branch — read whole in
MOVEMENT's "The body step" since 2026-08-27 and never built, because until
now the simulation only ever had guy 0. ANIM §9's "the every-frame re-roll of
a dog under a shorter idle" is closed onto it: the re-roll was never a clock.


## 2026-09-01 (item 128, Opus) — a guy is a body, and the headline moves 2665 → 3021

East Indies' long-capture word is **3021**. It was 2665 twice over, and the
second time it was the scout's dog.

**A unit is one or more figures, and a crew figure walks a body of its own.**
`Unit::process` runs `Guy::process` for the squad (`0..guy_mark`) and then for
the crew (`type->squad_size..num_guys`), and `Guy::move`'s third branch — the
one guy 0 never takes, gated on `guy_num != 0 && (track_dx || track_dy)` — is
a whole second integration: turn toward the point, give the frame up if the
turn is too large, step `floor(speed × 11 / 8)`, snap by Manhattan or clamp
each axis. It has been read since 2026-08-27 and never built, because until
now this simulation only ever had guy 0.

Three things had to be found before it could be.

**Where the offset comes from.** `track_dx`/`track_dy` are `+0x54`/`+0x58`,
and the queue had them at `+0x92`/`+0x94` — which are `off_x`/`off_y`, a
different field. The type record settled it, as the audit README says it
always does. The single writer is `Guy::update_gpiece@005d8530`, and it is
**art**: `trackoffsetx` and `trackoffsety` from `unit_graphics.xml`, times the
entry's `scale`, times `guy_scale` — a `float` in `.data` at `00c06244` that
`rise_z.map` attributes to `Guy.obj` and nothing but three console commands
ever writes. 4.8. Every scout's dog in the shipped data is
`trackoffsetx="-20" trackoffsety="10" scale="1"`, so the pair is `(-96, 48)`,
and the multiply happens once at load in `rondata::artdata::piece_tracks`.

The first thing that pair did was reproduce run56's frame-0 dump to the unit:
the human scout at (5784, 8088) facing `Unit::init`'s 120°, its dog at (5790,
7979). That was before a line of the mechanic existed.

**Where the point is written, and when.** Two functions, and the decompiler
prints both as `sin_table(unaff_ESI, unaff_EDI)` — so the listing again.
`Guy::set_new_location@005d86f0`'s crew loop rewrites it from guy 0's **new**
position and facing, and `Guy::set_angle@005d9010`'s from guy 0's current
position and whatever angle is being set. `Guy::do_turn` calls the second on
every turn, whether or not the angle moved; `Guy::move`'s settled arm
(`des_angle == angle`) jumps past the turn altogether and calls **neither**.
That last row is the whole of item 128: a standing, settled guy 0 stops
rewriting the point, and the crew walks on toward the last one it was given.

Four frames of it, in run56: guy 0 arrives on 2664 at (40416, 34272) and the
dog is told (40365, 34367); on 2665 the man turns where he stands, the point
rotates to (40322, 34217) and the dog sets off; it gets there on 2668 and
settles its angle on 2669. Every frame of that is a `Guy::set_anim` decision
the unit's body cannot make.

**And the turn rate that made it work.** The first attempt regressed the word
from 2665 to **413**, because the dog kept giving frames up to turning.
`GuyData::turn_speed@005de340` is not one formula but two: its whole first
half — the type's `TURN_SPEED`, the pack bonus, everything `docs/MOVEMENT.md`
"Turning" describes — is fenced behind `guy_num < squad_size`, and a crew guy
with a track offset returns a flat `0x40000000` **before** the
instant-from-a-stop test and before either mode. Ninety degrees a frame. The
give-up wants `|delta| > 3 × rate`, which at ninety degrees does not exist, so
a tracked crew guy always steps. run54's frame 412 is the proof: the dog turns
`1681129472 → -1540096000`, exactly a quarter, where the man manages the
scout's twenty-seven degrees.

**The check is the whole record.** run56's per-frame `GUY` blocks carry `x`,
`y` and `angle` for every figure and `Initial::frame_guys` was dropping them,
because it filters on the animation clock and this capture prints positions
without one. `Initial::frame_bodies` keeps them, and
`run56_s_figures_stand_where_the_original_s_do` compares all three fields for
every figure of every unit over 3,000 frames — **1,062,354 fields**. Nothing
in it is installed: guy 0's body is the unit's own and a crew guy's is derived
from the art and the two writers, so every row is a prediction.

Every player figure agrees on every frame. What is left is 301,810 rows of
gaia's spawn **bearing** — position right, angle wrong, because
`Sim::reseat_animal` puts an animal back and nothing derives its initial angle
— and one row on the capture's own last frame, which is half-written. Both
asserted at their numbers.

Made to fail first, by handing the crew no track: two figures part on **frame
0**, before a single tick, because a dog seated on its man is already 109
units from where the original's stands.

**A grep closed a second question on the way.** `off_x`/`off_y` — the pair
`Guy::set_anim`'s walking-guy early return subtracts, and the pair
`Guy::move`'s tracked branch subtracts before adding its step — are written
**once in the executable**, by `Guy::clear`, as one `undefined4` of zero. They
are not the formation's offsets. So `des_x != x − off_x` is `des != pos`, and
ANIM §9's two-line puzzle about `Guy::move:52` and `set_anim:163` disagreeing
was never a disagreement.

The new word, 3021, is four draws against five, and the missing one is an idle
request from `Unit::move_step` rather than from `Guy::inc_time`. It is past
run56's own 3,000 frames, so the next item on it needs a longer full-detail
capture or run54's trace alone.

## 2026-09-01 (item 129, Opus) — the collision was innocent, and the last building on the wrong cell is a dock

Item 129 asked for a capture: East Indies' word is 3021, the frame is four
draws against five, and the missing one is a blocked stand
(`Guy::set_anim+0x97a < Unit::move_step+0x823`) that only a full-detail dump
past 3,000 frames could explain. run56 stops at 3,000. So run57 was queued —
run56's recipe, only longer, 4,000 frames and a gigabyte — and while it ran,
the queue's own rule was applied to the capture already on disk: **diff the
whole record before booking a reading.** The answer was there.

**The collision block, first, because it is what the divergence is made of.**
`UnitData::log_data` writes `collide`, `collide_o`, `collide_who`,
`collide_guy` and `safe` at every detail level, and `run_traced` has compared
them on every capture since item 64 — but nothing on East Indies ever
*asserted* them. `run56_s_collision_block_agrees_past_the_scored_length` does:
**249,293 agreeing unit-frames, zero disagreements**, over three thousand
frames, where run10's own number is 139,514 over 1,772. So the collision model
is not what parts at 3021, and the seam is upstream.

**The seam is a building, and the field that names it had never been
compared.** `BuildDump::pos` has been parsed since the record existed;
`compare_frame` linked buildings by `(who, o)` and then looked only at
`gather_down` and the mining list. A building the AI sites sixteen tiles away
therefore links cleanly, agrees on every gather field, and reads as agreement.
`run56_s_buildings_stand_where_the_original_s_do` compares `x_internal` and
`y_internal` on every linked building of every frame — **92,626 fields** — and
the residue is one field of one building: player 1's **Dock `o 2010`**, laid
on frame 2977 at cell `x 57` in both and cell `y 52` here against the
original's **54**. Everything else stands where the original stands it: the
pre-placed buildings, both farms, the second city, and item 126's camp.

Frame 2977 is forty-four frames before 3021, and the chain is direct. The
citizen `1/11` — waiting out a gather order's `wait 282` — takes a
`BUILDORDER` for `o 2010` and an `EXPLORETOORDER` toward it. The original's
walks toward `(43704, 41688)` and steps `(24, 7)` a frame at `myspeed 25`;
this one walks toward `(43704, 38856)`. Twenty-one frames later the original's
step is refused and it plays the stand. There was never anything wrong with
the stand.

**One fix landed, and its oracle is a different map.** Reading
`produce_building` for the site scan turned up a timing error one layer under
§2.20's third defect: the spiral's index is stepped at the **bottom** of the
iteration by the stride the body has just set — `local_2c = local_2c + iVar13`
at `006e25bb` — and this crate stepped it at the top by the stride as it stood
*before*. That spends the old stride once more, so the first strided hop
starts one cell late and the whole tail of the walk is offset by one. §2.20
had said of its own fix that it "moved no measured number on any capture",
which was the tell: on a call that finds its site inside ring 2 the condition
is unobservable, and only the timing shows.

The check that catches it was already written and already failing at a number:
`the_fuzzed_map_s_frame_1_jitters_over_a_two_by_two_as_well` had asserted
`produce_building+0xc99` at **29 against the original's 30** for five days, as
its own stated residue. With the step at the bottom it is **30 against 30**
and the frame is 44 of 45. Nothing else in either suite moves.

**The word does not move**, and that is the honest score: the dock is still on
the wrong cell — `(57, 52)` rather than `(57, 54)` — so 3021 stands. `docs/AI.md`
§20 names the three candidates and which is unread. The strongest is the
dock's own **sub-position slide**: where `blocked_site` refuses a dock's exact
cell, `produce_building` runs a nested search over `local_88/2 × local_7c/2`
that this crate does not run at all, and two of the cells in this very walk
are refused here. No draw is spent anywhere on a dock's spiral or its slide —
the jitter is fenced behind `ident != Dock` — so the dumped position is the
*only* oracle any of it will ever have, and it is a test now.

**run57 was taken anyway**, and it is not wasted: East Indies at full detail
to 4,000 frames, sized to the word by the capture lane's standing rule. The
frames past 3,000 are what the successor reads once the dock lands.

Whether the lane's rule should have an *ordering* clause under it — widen
every dumped record the mechanic touches before booking a capture — is
marked `FABLE:` in `docs/ORACLE.md`'s run57 section and argued both ways
there. It is a working-agreement change, so it is the steering session's,
not this one's.

## 2026-09-01 (item 130, Opus) — the dock's slide, and the headline moves 3021 → 3435

Item 130 asked for one arm of `Leader::produce_building` — the sub-position
slide behind `is(0x1b0)` — and said the dumped position was the only oracle it
would ever have. Both halves were right. What was wrong was *why* it mattered.

**The slide does not place the dock.** run56 dumps `o 2010` at
`(44160, 41856)`, which is the plain centre of cell `(57, 54)` to the unit,
and the arm's whole reach is ±2 tiles — half a cell — so no slid position from
any neighbouring cell can land there. Half an hour of arithmetic on the dump
said the strongest candidate was ruled out.

**It is the accepted set that matters, not the position.** The spiral steps by
one until a candidate improves on a standing best past ring 3, and then by
three forever. Instrumenting the crate's own walk for frame 2976 printed the
whole thing: anchor `(51, 52)`, `start 0`, `end 145`, nothing accepted before
**129**, the stride engaging at **135**, and the walk `138 → 141 → 144` with
`141 = (57, 52)` the last accept. The original needs `143`, and `143 ≡ 2
(mod 3)`. Three of the walk's cells — 117 `(47, 57)`, 119 `(48, 58)`, 131
`(54, 58)` — are refused by `blocked_site`, and those are exactly the cells the
slide would rescue. With the arm built, 117 and 119 are accepted, the stride
engages at **119**, and the walk runs `122 … 140, 143`. 143 is the last index
of ring 6, ties replace, and the dock lands on the original's own cell at the
original's own point. `run56_s_buildings_stand_where_the_original_s_do`:
**92,626 fields, zero wrong**, where it had been one field of one building.

The arm's three readable details are in `docs/AI.md` §21: the base is the
*unpadded* centre, `dy` starts at `−(w / 2)` rather than `−(h / 2)` because
the compiler loaded it once, and the out-parameter is null. All three are
invisible on a 4×4 dock, and all three are written as the listing has them.
The same section notes that `produce_building`'s three dock tests are the
lineage `is(0x1b0)` and not the identity, which the crate now uses.

**Then the dock's new cell reached two code paths nothing had ever run.**

The first announced itself as an `i32` overflow in `find_angle`. All three
`find_*path` pull-back walks decompile to `if (angle < 0) step = -step;`
before two `sin_table` calls, and this crate had transcribed that as a
caller-level flip *on top of* the fold `movement::sin_component` already
performs. Doing it twice cancels it: for every western angle the goal walked
away from the start, the region test never matched, and the loop did not
terminate. The listing settles it in a minute — at `0x6894c3` the cosine's
distance is reloaded from the un-negated step and negated again only on the
sign of `angle + 0x40000000`, which is two independent folds and no caller
flip at all. The same two listings give `find_tpath`'s step as `0x60` and
`find_upath`'s as `0x18`, where the crate had `0x30`. `docs/PATHFINDER.md`
§13. **Every capture on disk broke out of that loop on its first region
test**, which is why a determinism soak and 175 diff tests had never touched
it: the body first executed when a dock moved onto a coastal cell whose tile
region is not its builder's.

The second was one line shorter and worse. `find_nearby_spot`'s
terrain-class-by-domain test was **not implemented** — the crate asked
`World::accepts`, a cell-bounds test the sweep had already made — so a land
unit could take a spot on an ocean tile. With the dock right, the builder's
swarm ring is drawn on a coast for the first time in any capture: `k = 0` is
refused by the footprint and `k = +1` lands on tile `(228, 215)`, mask
`0x420`, surface ocean. The original refuses it and takes `k = −1`. With the
test in place this crate takes `k = −1` too. `docs/ORDERS.md` §10.

**The score.** East Indies' word on the long capture goes **3021 → 3435**;
run56's collision block goes 249,293 → **249,413 field-frames, none wrong**,
and the only unit that ever leaves the original's point is player 0's Citizen
`o 5` on the quit's own half-written frame. Neither of the two path fixes
moves a number by itself, and both are kept because the listing says so.

**run57 became a test the same hour, and it names item 131.** The capture
taken for 3021 — 4,000 frames, the same game as run54 and run56 by
`rngcmp.py` and `samegame.py` — now carries both position records:
**130,326 building fields with two buildings wrong** (`1/2011` on frame 3177,
one cell east in `x`; `1/2012` on 3977, one tile) and **322,683 collision
fields with none wrong**, nothing leaving the original's point before 3177.
Its own earlier statement — seventeen units parting from 2978 — was the
dock's consequence, and it is gone. Both residues are asserted as they stand.

The word's frame, 3435, is three draws against two: a `Farms::inc_time+0x1ae`
this crate spends and the original does not. It is past 3177, so it is
plausibly item 131's consequence; that is item 132, and it says to re-measure
rather than to read.

## 2026-09-01 (item 131, Opus) — a farm's distance is in tiles, and the headline moves 3435 → 3579

Item 131 was run57's whole residue: the AI's farm `1/2011` going up on frame
3177 one cell east of the original's, with `1/2012` on 3977 one tile behind
it and flagged in the queue as "probably a second thing". It was one thing,
and the queue's method — instrument `produce_building`'s walk for the call —
found it in twenty minutes.

**The walk was not the problem.** Frame 3176, anchor cell `(45, 47)`,
`start 0`, `end 105`, and the stride never engages because nothing past ring
1 can reach the standing best. Every candidate that scores, scores in both;
every roll is spent in both. That is why the word held to 3435 with the farm
on the wrong cell — a draw diff *cannot* see this, and never could have.

**The problem was the score.** `produce_building` computes the candidate's
distance from the anchor twice, in two different units, and only the general
arm at `006e1f9a` is the one §2.20 described. The FARM/MINE arm at
`006e2004` rebuilds both sides in **tiles** — the candidate as `cell·4 + 2`,
the anchor as `div_3_table[(pos ^ 0x63637) >> 6]`, its *exact* position
rather than its cell's centre. Four times the resolution is the whole
finding: in cells all eight neighbours of the anchor are `4000 / 1` and the
random part is the entire score, so the last one drawn with the best roll
wins; in tiles, from an anchor that stands at `(34656, 36192)` and not at its
cell's centre, the far diagonal is `4000 / 9` against everyone else's
`4000 / 6`, which is 222 — a spread the roll's 500 does not close. Index 5
`(45, 48)` wins by 20 points where index 8 `(46, 48)` had won by 25.

Beside it, a constant that is not one. The `local_60 == 0` arm adds
`0xff − WData.val`, the map maker's own city-site value for the cell, so a
*better* site scores **lower** here. §2.20 had read it as "0 on this world",
which was true of the flat harness world and false of the islands map:
run38's dump gives the five cells this call parts on `val` 20, 31, 6, 21 and
27. It does not flip this call by itself — with the cell distance it still
lands on index 8 — and it is kept because the listing says so.

Both are in `docs/AI.md` §22, and both are settled by `llvm-objdump` over
`006e1f9a`–`006e2073`, where the decompiler prints `vector_dist(unaff_EDI,
unaff_ESI)` twice and says nothing about either.

**The score.** `run57_s_four_thousand_frames_stand_where_the_original_s_do`
is **130,326 building fields, zero wrong** over 4,000 frames, where it was
two buildings and 850 field-frames; its collision block grows to **330,643
field-frames, none wrong**, and the first unit to leave the original's point
now does so on 3582 rather than 3177. East Indies' word on the long capture
goes **3435 → 3579**. Nothing else in either suite moves: 175 rondata tests
and 680 sim tests green.

**Item 132 went with it.** The word's own frame at 3435 was three draws
against two, an extra `Farms::inc_time+0x1ae` this crate spent. It was this
farm's shadow — with player 1's farms on the original's own cells the extra
growth tick is gone — and the queue's instruction to re-measure rather than
read was the right one.

**What is next is a bird.** run54 now parts its *sequence* at 3579 and its
*count* at 3580, and both frames are one thing: the original's second draw
of 3579 is `Dock::init+0x125` where this crate spends a second
`Guy::init_real+0x52`. `Dock::init@00740a80` is four lines — it counts the
dock into the leader's per-region tally, spawns a **`GULLBIRD` of owner 9**
a tile north-west of the dock, rolls its facing as
`(r % 7) · 0xaaaaaaa − 0x40000000` on a turn of 7, and gives it a strafe
order around the dock. Nothing here spawns it. It fires when the dock
*finishes*, not when it is placed: run57's dock is laid on 2977 and this is
600 frames later. That is item 133.

## 2026-09-01 (item 133, Opus) — the dock's gull is a bird, and the headline moves 3579 → 3608

Item 133 was booked as "a finished Dock spawns a bird, and this crate does
not". The booking was half wrong in a way worth writing down: `dock_open`
had spawned the gull since 2026-08-25, and had spent both of `Dock::init`'s
draws in the right order. What it had not done was **say** which site the
second one was, or give the gull anything that made it a bird.

**Two defects, both this crate's rather than the reading's.**

The heading roll was `let _angle = (self.rng.get(0, 0xffff) % 7)…` with no
`self.mark(…)` above it, so `mark_sites` attributed it to the label still
standing — a second `Guy::init_real+0x52` — which is exactly queue item
122's shape, found here by the score rather than by the grep. One `mark`, one
row in `rondata::trace::SITES` at `0x0074_0ba5`, and frame 3579's sequence
agrees.

The gull was created with **no `type_index`**. That is the pasture's own bug
of 2026-08-30 one type over (`docs/SYNC.md` §3.11): `Guy::set_anim` names the
three gaia bird types by identity in its walk arm (`set_anim:620` —
`0x192`, `0x193`, `0x194`), so a gull carrying the default −1 could never
throw the wing beat's coin, and `Sim::do_idle`'s gaia arm — keyed on the
wild bird alone — sent it to `animal_idle` instead. So it never flew.

**A gull reaches `do_air_physics` by a different order, and that is the
whole of the modelling.** `Unit::do_job` dispatches `STRAFE` to
`Unit::do_strafe@005eab00` where a wild bird's `AIR_PATROL` goes to
`Unit::do_air_patrol@005ea620`; both call the `+0x180` virtual and then
`Unit::do_air_physics@005e86d0`. `think_bird`'s `0x194` arm returns after an
`order_type` call and draws nothing, and `do_air_patrol`'s counter tail is
`do_air_patrol`'s alone — so the gull's think is free, and what reaches the
stream is `do_air_physics`'s tail, `set_anim(CHAR_WALK, 0, 1)`. That is one
draw at birth (run54's frame 3580) and nothing after, because
`Guy::set_anim:141`'s gaia-walker early return catches every later frame;
then the gull is on `Guy::inc_time`'s wing beat like any other bird.

**Three readings became assertions instead of prose.** The chain
`Guy::set_anim+0x104b < Unit::set_anim+0x56 < Unit::do_air_physics+0x683`
occurs **12 times in run54's 24,000 frames** — ten wild-bird births, this
gull, and one at 15458 — which is what says the coin is a birth and not a
per-frame draw. `do_air_physics`'s own draw site `+0x639` fires **three
times** in the same 24,000 and all three are under `do_air_patrol`, never
under `do_strafe`, so the flight this crate does not model spends nothing.
And `Region::coast_here` leaves the blind list on frame 3580, under the
gull's first `do_strafe`.

**The widening.** `gull_o` was the one field of the `DOCK` record nothing
had ever compared, and run22 is this same game with a `DUMP_ALL` window on
`[3579, 3582)`: block 3580 reads `gull_o 15`. Owner 9 is absent from a dump
so the gull's own record is not there — but its object number is, and this
crate's gull comes out **15** at the end of 24,000 frames. Made to fail on 14
before it was landed.

**The score.** East Indies' word on the long capture goes **3579 → 3608**.
Great Lakes is unmoved at 1802 and the scored captures are unmoved at
1851/1850 and 1772/1772. 175 rondata tests and 680 sim tests green.

**And one assertion had to be rescoped, which is the lesson.**
`run57_s_four_thousand_frames_stand_where_the_original_s_do` asserted zero
wrong building fields over all 4,000 frames. That claim was **luck past the
parting**: a building the AI sites after the two streams have parted is
sited from draws that are nobody's, and `1/2012` on 3977 came back one tile
north the moment the word moved on a change that has nothing to do with it.
The building half is now asserted up to the word and printed past it — the
same rule run53/54's own tests carry and say out loud — and the collision
total, which counts *agreeing* unit-frames and so moves with quality, became
a floor rather than an equality. It rose 330,643 → 337,265 on the way, and
the units that ever leave the original's point fell from fourteen to eleven.

**It is marked `FABLE:` rather than settled** (`docs/ORACLE.md`, run57),
and the reason to re-read it is that the obvious account of the parting is
the wrong one. The harness does not free-run: `Built::tick` installs the
original's word at the end of every frame the dump checksums, and run57 is
a per-frame full dump, so both sides start every frame on the same word.
What is not reset is the position *within* a frame — from 3608 on this
simulation spends fewer draws before the AI's own rolls, so a placement 369
frames later is decided by a value that is nobody's. The rescope follows
from that, but so does a third option nobody costed: a **ratchet** on the
count rather than a cut to the range, which would fail on a new
past-the-word defect where a scope cut cannot. That trade applies to every
score this repo pins past a parting, which is why it is the steering
session's and not this one's.

**And the headline got a guard.** `the_handoff_s_scoreboard_is_the_floors`
parsed the queue's `Scoreboard:` line and stopped there — but East Indies'
scored capture is closed, so the number a session is judged by is on the
`Long captures:` line below it and nothing checked that one. It does now,
against `LONG_WORD_EAST_INDIES` and run53's own floor, and it was made to
fail on 3579 before it was landed.

**What is next is not a bird.** run54's frame 3608 is
`SpellType::cast_transport`'s first, and the three draws this crate does not
spend are two `Guy::set_anim+0x97a < Unit::set_anim < Unit::do_cast+0xc89`
and the cast unit's own `Guy::init_real+0x52 < Unit::init+0xb97 <
Objects::init_unit+0xbd`. It is the dock's shadow twice over: the transport
level granted at 3579 is what lets a unit board at all.
`docs/TRANSPORT.md` §6 has the mechanic and §12 has had the capture booked
since it was written — "a `UNITS=3` window over frames 3600–3640, where
`cast_transport` fires". That is item 134.

## 2026-09-01 — item 134: a scout sails, and the word does not move

**The item was booked as three missing draws and turned out to be a whole
half of the AI.** run54's frame 3608 is `SpellType::cast_transport`'s
first; this crate spent none of its three draws, and the reason was not
that boarding was unimplemented. It was that **nothing in this simulation
had ever wanted to cross water**. The unit that boards is the AI's
**scout `1/0`**, not the citizen §6 supposed, and what sends it is
`Unit::think_civilian_transport@005f40d0` — `docs/TRANSPORT.md` §7, read
last month, documented in full, and marked "not implemented: it needs the
unit AI's `think` and the danger grid's writers".

**The capture was already on disk, and reading it first saved a screen
hour.** §12's check 4 asked for a new `UNITS=3` window over 3600–3640.
run57 is the same game at run39's detail for 4,000 frames, so blocks
3585–3609 carry the whole mechanic outright: the scout idle at
`(40416, 34272)` with `idle 60` through 3583; on 3584 an eleven-waypoint
path and a `MOVE_TO` to `(35712, 25728)`; on 3608 an order list of
`[CASTORDER spell 650 paid 0, MOVEORDER]` and a path top carrying
`flags 4`; on 3609 the barge `1/14`, guy `type 320`, holding the scout's
path with that flag cleared, the scout's orders minus the cast, and
`inside_up 14` on the scout. That is the queue's own "grep the dump before
booking a reading" one level up — the answer cost a `sed` range.

**Five things had to land for one frame.**

- **§6, the boarding.** `set_new_location`'s shore arm (a land unit
  stepping onto ocean queues `0x28a` `QUEUE_FIRST` and does **not** move;
  a boat stepping off ejects and dies), a `CAST_SPELL` order,
  `do_cast`'s untargeted arm, and `cast_transport` — the boat born at the
  caster's own point, walked to the water, given the damage, the angle,
  the order list and the path stack, with the top waypoint's embark flag
  cleared when its region is the boat's.
- **§7, the island.** `think_scout`'s `005f6d74` tail — `Region.scouted`
  marked on the unit's own region — then the region search, the sea that
  coasts both, the strided cell walk with `coast_here` and
  `num_waterhalf`, and the group move to the cell's centre.
- **`Region.flags`, which no cell implies.** `go_here`'s first arm is
  `flags & 8`, the map generator's resource-region bit, and it had been a
  seam for that reason. `Regions::log_data` writes it: the harness now
  reads the `REGIONS` block out of an `InitialDump` and installs it
  through the same map the cells were numbered by. East Indies' eight
  middle islands carry `0xa8`, the two the players start in `0xa4`.
- **`invalid_loc`'s three domain arms**, where this crate had only the
  land one.
- **`find_wpath`'s pull-back gate**, and this is the one that would have
  been hard to find from a reading. `00689375` runs the goal walk only
  for a unit that **cannot** board. Without the gate the goal is dragged
  back until its region matches the unit's, which for an island target
  means back onto the unit's own island; the search then plans a route to
  the near shore and stops. That is exactly what happened: with §7 landed
  and the gate missing, East Indies' word fell **3608 → 3585** on a
  `Unit::do_move+0xe84` grid draw the original never spends. With the
  gate the eleven waypoints this crate plans are the dump's, entry for
  entry, and the destination cell `(46, 33)` is the original's own.

**The score did not move, and the reason is one draw.** The three cast
draws now match at their sites; the frame parts one draw later, on a
`Guy::set_anim+0x97a < Guy::inc_time+0x271` this crate spends for the
barge's brand-new guy and the original spends on no frame at all. Every
other new unit wraps on its birth frame — the dock's gull on 3579,
run33's trained citizen on 99, both reproduced — and nothing read so far
separates the barge from them: `squad_size` is a literal 1 for every type,
`Guy::init_real` writes `end_time` 0, and `Guy::set_anim`'s early returns
all want `cur_time < end_time`. It is booked with the capture that settles
it outright, a `GUYS=4` window over 3606–3612, and it is the only thing
between here and the sailing.

**Everything else held.** 175 rondata tests and 684 sim tests green;
Great Lakes unmoved at 1802, the scored captures at 1851/1850 and
1772/1772, run56's 249,413 collision field-frames and 92,626 building
fields still exact. run57's collision floor moved 337,265 → 334,258 and
the units ever off position eleven → fourteen, both past the word and
both printed rather than pinned, which is what the `FABLE:` marker of
2026-09-01 says they are.

## 2026-09-01 — Fable steering: the loop itself on the table

**The occasion was double**: the queue's own rule (134 held the word at
3608, one of the two sessions the rule counts) and a first-of-its-kind
brief from lore, the user's telemetry project, which metered ~70 sessions
of this loop and sent steering candidates by cross-session message. The
calibration mattered more than the candidates: none of them move the word.
They move **token cost per item at unchanged output**, and the user is
token-bound, not time-bound — a different axis than this queue has ever
scored, and the right one for a steering session to weigh.

**The marked rows first.** run57's two `FABLE:` markers are settled in
place (`docs/ORACLE.md`): the past-the-parting **rescope is ratified** and
the ratchet **declined** — item 134 itself is the evidence, a
fidelity-improving session under which the past-the-word collision total
*fell* 337,265 → 334,258, so a ratchet fails on progress and trains
number-editing. The standing rule: assert up to the word, print past it.
The **capture-ordering clause is adopted** into `CLAUDE.md` — widen the
records on disk before booking a capture — worded to order the booking and
not the screen, so the capture lane keeps its idle hours. The two old
ORDERS rows stay open as marked; nothing depends on them.

**Lore's candidates, weighed.** (1) The test-lane split: **landed**, as
`tools/guard.sh` — the paperwork guards plus the item's tests, 0.2 s warm
against the 15 s average and the 92 s suite — with the honest caveat sent
back that this moves wall-clock, not tokens; the `--release` suite remains
the pre-commit gate. (2) Promoting the recurring probe shapes into
`tools/`: **adopted as a convention, not a booked item** — a shape reached
for a third time graduates, one-off hypothesis probes stay scratch — so
the tool is built on next use rather than speculatively, per "earn every
dependency". This is the genuinely token-moving lever (~1.5 M output
tokens a window in scratch authoring). (3) The foreground sleep-and-grep
capture waits: **a convention now** — background the wait; every poll turn
re-bills the whole context. (4) The python-heredoc edit+test fusion lore
asked about: emergent, not designed — consistent with the worktree's
compound-command friction — and kept, as a named convention rather than a
wrapper script, since the wrapper would save ~80 tokens an invocation and
add an abstraction. Lore's own validations (bare "continue" openers,
orientation ramp, grep speed — all noise) confirm the handoff protocol as
it stands and were left untouched.

**What steering did not do**: touch item 135, the scores, or the floors.
The word is 3608, the opener is unchanged, and the next session is Opus.

*Addendum, same day.* Lore pinned the before-number so the conventions can
be scored: output tokens per item, grind sessions only, n=63 — median
189k, mean 216k, p90 318k, ~341 turns/item. *(Corrected 2026-09-03: hand-summed
over streaming snapshots, ~1.9× high; the deduped baseline is 113k median.)* The next steering pass asks
lore for the re-cut (sessions after 39afef6, same spec, compare medians,
read against the per-session distribution since item mix shifts). C1's
correction was conceded and ledgered on lore's side.

## 2026-09-01 — item 135: the barge's clock was the frame loop (Opus)

**The item was booked as an animation question and it was a loop
question.** East Indies' long word had stood at 3608, the frame
`SpellType::cast_transport` first runs, on one draw: this crate wrapped
the transport barge's brand-new guy's clock and the original wrapped no
barge's, on that frame or any later. `docs/TRANSPORT.md` §13 had a
`GUYS=4` window over 3606–3612 booked against it — `Guy::log_data` prints
every guy's `cur_time`/`end_time` at detail 4, and run57 carries `GUYS=2`,
so the field genuinely is not on disk.

**The disk answered it anyway, and not with the field.** Eleven frames in
run57's 4,000 create a guy through `Guy::init_real+0x52 < Unit::init+0xb97
< Objects::init_unit+0xbd`. Ten of them spend a
`Guy::set_anim+0x97a < Guy::inc_time+0x271` on the same frame — 274, 380,
494, 615, 1704, 1911, 3319, 3526, 3734 and the setup — and **3608 is the
only birth in the run with no wrap behind it**. That table is one pass
over the trace, it costs a minute, and it turns "why does this one not
wrap" into "what is different about this one birth". The answer is that
the other ten are *trained*, born in `Build::do_queue` in
`Objects::process_all`'s second loop after every unit has had its turn,
and the barge is cast from inside the **first** loop.

**Which matters because the loop's bound is re-read.**
`Objects::process_all@0065dce0`'s inner loop tests `o < unit_mark[who]` at
the bottom, out of `ObjectsData+0x15c` rather than out of a local. So a
unit created inside the loop, in a slot above the one being walked, takes
its own turn on the frame it is born. The barge is `1/14` and its caster
is the scout `1/0`; it inherits the scout's move order, steps,
`Unit::move_step` asks its guy for `CHAR_WALK`, and by the time
`Objects::inc_time` reaches it the clock reads `cur_time 1 < end_time`.
`Sim::tick` fixed its visit list before the loop and said so in a comment
— the eighth time the code carried an assumption the reading did not.

**What it moved.** East Indies' long word **3608 → 3687**, sequence with
it: seventy-nine more frames, every one draw for draw. Great Lakes holds
at 1802, the scored floors hold at 1851/1850 and 1772/1772, run57's
four-thousand-frame position test holds, and 175 rondata plus 684 sim
tests are green. A second change rode along, from the same reading: the
visit order inside an owner's band is the object order the loop walks
rather than `Sim::units`' storage order, which is what `docs/SYNC.md` §3.2
has said since it was written. Nothing moved on it.

**Made to fail on the way in.** Seven sim tests went red the first time,
all of them harnesses that seat a unit by pushing it onto `Sim::units`
without going through `find_free` — so `unit_mark` stayed 0 and the new
loop walked nobody. The bound is `max(unit_mark, highest slot filled)`
now, which is the same number in a real game and the honest one in a
hand-built scenario.

**What is at 3687**: ten draws against seven. Ours opens with a
`Guy::set_anim+0x97a < Unit::move_step+0x823` — a walker whose step was
refused asking for its idle — that the original does not spend, and then
throws seven gaia wing-beat coins where the original throws five. Item
137, and `docs/SYNC.md` §3.22 has both lists.

**The rule this is another instance of**, and it is the queue's own one
level up: grep the disk before booking a capture. The booked window would
have cost an hour of screen and would have printed a number that only
confirms the symptom; the trace on disk named the cause.
## 2026-09-01 — item 137: the citizen offers itself to the boat (Opus)

Item 137 was booked as two draws — a blocked walker's idle this crate
spends and the original does not, and seven gaia wing-beat coins against
five — and it was **one missing call**, ninety-five frames upstream of
the frame it was booked at.

**The blocked walker has a name, and the trace does not give it.** The
site says `Unit::move_step+0x823` and nothing more, so the first move was
to print the unit: the AI's citizen `1/11`, blocked at 3687 by `1/9` while
walking to a waypoint of its own. The second move was the run57 position
test, which already knew: `1/11 parts at 3582`. The two draws at 3687 were
one unit's the whole time — the extra coins are what a wrap re-throws when
one extra draw has shifted the stream, since a bird that picks the 23-frame
Flap while its clock reads 24–30 wraps again and throws again (§3.9).

**The original's own dump says what the citizen should have been doing.**
run57 is 4,000 frames of full detail and 3581 is inside it. `1/11`
finishes a build on 3580 and stands with no order; on 3581 it holds
`group 65`, a `MOVE_TO` whose `orig 38784, 24192` is the centre of cell
(50, 31) — `cell × 0x300 + 0x180`, which is
`Unit::think_civilian_transport`'s own arithmetic and nothing else's — and
a twenty-four-leg path. This crate gave it a gather order instead.

**And the call is one line of `think_peasant`.** `005f5760:53`, between
the idle gate and `find_build_spot`: an AI-driven unit
(`unit_masks & 0x40000`) whose **base type** is `0x32` or `0x33` calls
`think_civilian_transport(1)`, and a 1 back ends the think. The test is
the base type rather than the worker category, so a scholar never asks.
`docs/TRANSPORT.md` §7 had the function whole and had said for two days
that this caller was a seam; what it did not say is that the seam was
holding a hundred frames of the headline.

**How the two paths differed, which is the pleasing part.** With the wrong
destination the citizen still walked *the same tiles* — the pathfinder
found the same route — but every waypoint sat 144 units off, in both axes,
from the original's. That is `off_x/off_y`, the destination's offset inside
its world cell, which `find_tpath` and `go_around_building` both place
their points from (`docs/ORDERS.md` §4.1, §4.6.1). A destination two cells
wrong shows up as a detour waypoint at `tile*0xc0 + 132` against the
original's `tile*0xc0 + 60`, and reading that arithmetic backwards is what
said the destination was the thing to look at, not the pathfinder.

**What it moved.** East Indies' long word **3687 → 3978**, sequence with
it. run57's four thousand frames go from **eleven** units ever off the
original's point to **four**; the earliest parting from 3582 to 3647; the
comparable collision field-frames from 330,643 to 348,354, none wrong; the
buildings stay exact on all 130,326 fields. Great Lakes holds at 1802 and
both scored floors hold. 175 rondata and 684 sim tests green in
`--release`.

**A number the run57 test had been assuming, and now pins.** Its assertion
read "nothing parts before the word does", and with the word at 3978 and
the earliest parting at 3647 that is simply false — a unit can leave the
original's point without spending a draw for it. The assertion now pins
the parting frame and the count of ever-parted units as two facts of their
own, which is the honest shape: the word measures what the two streams
*spend*, and the position record measures where they *are*.

**What is at 3978**: four draws against five, and the one we do not spend
is the original's `61a1da` — inside `Unit::come_out@00617c10`'s tail
(`+0x25ca`), with no name in the trace. The AI's scout `1/0`, the unit
that cast the barge at 3608, leaves the original's point on 3979. Item
138.

**The rule this is another instance of.** The queue says grep the dump
before booking a capture; this one says something narrower and sharper —
**when a draw parts, ask the position record first**. The word tells you
which frame; `first_divergence_by_unit` tells you which unit and, more
usefully, *how long ago*. Ninety-five frames of head start were sitting in
a test that already ran on every commit.


## 2026-09-01 — item 138: the passenger comes ashore, and the word goes 3978 → 4020 (Opus)

The item was booked as one draw — `61a1da`, unnamed in the trace, inside
`Unit::come_out@00617c10`'s tail. It was three things, in the order the
original does them, and all three landed.

**The draw is an army coin, and only two lineages throw it.** The listing
(`llvm-objdump 0x61a0a0..0x61a230`) makes `come_out`'s tail
`Unit::add_to_army`'s fifth caller: an AI-driven, non-caravan,
non-merchant unit leaving whatever carried it joins an army unless a coin
says otherwise — and the coin is thrown only when the unit `is_special()`
or `is(BARK)`. `is_special` is `is(SCOUT)` by the loader, so the two arms
are the scout line (`% 2`, `+0x25ca`) and the naval-scout line (`% 3`,
`+0x25b0`); every other type takes a draw-free `is(SPY)` test and spends
nothing. That is why a site reached **eleven times** in run54's 24,000
frames had gone unseen for 3,977: the AI trains citizens and soldiers, and
neither asks. Both arms are on disk — nine `+0x25ca` under
`Object::eject_contents < Unit::set_new_location`, and two `+0x25b0` under
`Build::train < Build::finished`, which is the AI's Bark being trained on
10323 and 10465. The decompiler prints both `is` calls with their type
arguments dropped; only the listing names `0x143` and `0x3a`.

That moved the word by exactly one frame, to 3979, where the scout stood
in the wrong place and ran a whole `think_scout` the original does not.

**The spot is `come_out`'s host arm, and every term of it is the boat's.**
The function splits on the host's vslot `0x1c`. For a unit host the
bearing is `host->angle` (`+0x50`) and the ring runs from the **host's**
`block_radius` out to that plus `UNIT_DISEMBARK_DISTANCE` — and the radius
is read off a local that was reassigned to the host two lines above, which
is the easy thing to misread and is worth a whole ring. run57 block 3979
pins all three at once: the barge at `(35740, 26706)`, `angle -13303808`,
`BLOCK_RADIUS 3` → the ring `[144, 720]`, step `(720 − 144) / 8 = 72`,
whose first candidate at the first bearing snaps to `(35736, 26568)` —
the scout's own point, exactly.

**And the ring only reaches land because the barge marks nothing.** With
the ring right the sweep still refused its first three rings, because the
boat sat in the collision bitmask. `docs/COLLISION.md` §2 has said since
it was written that the marking gate compares the cell's `region` against
**`get_tregion`** of the marking figure's tile — and
`WorldData::get_tregion@006b52e0` answers a coastal cell's `region2` for
an *ocean* tile. A boat on the water half of a coastal cell therefore
marks no cells there at all. This crate asked the plain `region_of`, which
made the gate vacuous for exactly the case it exists for. Ninth instance
of a document and its code disagreeing, and the document was right.

**`Object::eject_contents` is `cast_transport` run backwards.** For a
passenger whose `uber_size` is 1 the boat's whole order list moves back
onto it and the boat's path stack is inverted and popped onto its own —
which restores the order it was in — with the top's embark flag cleared.
Block 3979's `MOVEORDER` and both `PATHDATA` entries are block 3978's
barge's, field for field, `flags 4 → 0`. Two smaller things rode with it:
`Movement::at` zeroes the speed, so a passenger put ashore stood there for
ever until the speed and turn rate were carried across (`come_out`'s
building arm has done that all along); and the crew has to be **seated**
on its track offset rather than left to walk there, because a walking guy
takes no idle roll — which is the second of the two `Unit::set_anim` draws
the original spends when the scout arrives at 4005.

**What it moved.** East Indies' long word **3978 → 4020**, sequence with
it. run57 goes from four units ever off the original's point to **three**
— `1/0` now stands where the original's does for the whole capture — and
its comparable collision field-frames 348,354 → **348,469**, none wrong,
buildings exact on all 130,326 fields. Great Lakes holds at 1802 and both
scored captures hold. 175 rondata and 684 sim tests green in `--release`.

**What is at 4020**: four draws against six, and the two we do not spend
are a `Guy::set_anim+0x97a < do_cast` pair — the scout reaching the *next*
shore and casting its second transport, man and dog. Its second leg is the
one `think_scout` gives it on 4005, the frame it lands and goes idle: both
sides spend the same eighteen draws there and then walk to different
places, and this crate reaches no water until 5115. Item 140.

**The rule this is another instance of.** Last session's was *when a draw
parts, ask the position record first*. This one is its sequel: **when the
position record still parts after the draw is landed, keep going in the
same function.** Three of the four findings here are in `come_out` and its
caller, and none of them would have been found by reading the tail alone —
each was forced by the next frame of the same diff.

## 2026-09-01 — item 140: the colonist cannot see the shore, and the word goes 4020 → 4275 (Opus)

**The item was booked on the wrong unit, and it was the trace that said
so — before the capture taken to answer it had finished running.**

Item 140 read frame 4020's unspent `Guy::set_anim+0x97a < do_cast` pair as
the AI scout `1/0` casting its second transport, and asked why the scout's
second leg — the one `think_scout` gives it on 4005 — went somewhere the
original's did not. run57's dump stops at **4001**, four frames short of
the decision, so the first move was to book run58: East Indies, run39's
recipe, 5,200 frames, sized past the word by the standing rule that owed
one anyway. It ran in the background for the whole session.

**What the reading found while it ran.** The scout's 4005 scan is right,
in every term. Its region is 6, `size` 151, stride `(151 + 99)/100 + 4005
% 8 = 7`, start `60673 % 7 = 4` — and of the twenty-two cells that walk
strides through, exactly **ten** pass the fog, the location test and the
surface test, which is exactly the ten `+0xaba` draws the original spends.
Sweeping the other seven phases gives 47, 32, 26, 19, 17, 11, 11: the
count is unique to the frame's own phase, so the fog, `invalid_loc` and
the coordinate list all agree with the original's. The winner is cell
**(48, 30)** at score 102, three cells away — and the scout arrives there
on **4110**, the very frame run54's trace throws its next `+0x941` and its
next ten `+0xaba`. Two scans agreeing frame for frame after a hundred
frames of walking is not luck.

So the caster at 4020 is not the scout. The tell was one draw: the scout's
own cast at 3608 spends **two** `Guy::set_anim` draws, `Unit::set_anim
+0x56` and `+0xb6` — a man and a dog — and 4020 spends `+0x56` alone. One
figure. It is the AI's citizen `1/15`, three hundred frames into a
colonise walk, and run57 has all of it.

**Then two reads, and both were a field written and never read.**

- **`Region::coast_here`'s neighbour probe is `WorldData::get_tregion`,
  and this crate asked the plain cell region.** `0068106a` steps a whole
  cell and re-reads at that neighbour's **centre tile** through
  `get_tregion`, which answers a coastal cell's `region2` — the *sea*
  region — when the tile it is given is ocean. A coastal cell is a land
  cell in `WData.region`, so with the plain read the function can only see
  a wholly-ocean neighbour, and a cell one in from the waterline coasts
  nothing at all. `think_civilian_transport` keeps only the sampled cells
  `coast_here` accepts, so the colonist's candidate set was the wrong one:
  on frame 3735 the original sends `1/15` to cell **(39, 34)** and this
  crate sent it to (37, 36) — which is *nearer* by the same scoring and
  simply was not one of the original's candidates. With the fix the first
  pass of region 8 accepts four cells scoring 23, 19, **15** and 18, and
  the minimum is the original's own. `docs/TRANSPORT.md` §9.3.
- **`Unit::move_step` reads the current waypoint's turn-in-place bit.**
  `005fb1a5` is `(manh < slow × 0xc0) || (path.flags & 4)`: a waypoint
  that crosses the waterline is turned to before it is walked to, however
  far away it is. `Sim::shore_flagged` has written that bit since the
  pathfinder landed and **nothing ever read it**, so `1/15` walked through
  the turn the original stands still for on 3988 and reached its embark
  point a frame early — which is why the first fix alone moved the word
  *backwards*, 4020 → 4019. `docs/MOVEMENT.md`, "The unit step".

**A third thing rode along, from the same afternoon's reading and moving
nothing.** `ObjectsData::find_unit_ordered@0065bc40` is three predicates
this crate did not have: the blocking unit must **have** an order, that
order must be in the move family `{1,2,3,4,0x12,0x13,0x15}`, and it must
stand in the target cell's own region. The name means what it says, and
`docs/SCOUT.md` §8 had stood on the name for a fortnight. It is vacuous on
every capture on disk — no unit ever shares a scout's type — so it is
landed with a unit guard rather than a diff. §8.1.

**What it moved.** East Indies' long word **4020 → 4275**, its count to
4288. run57's four thousand frames go from three units ever off the
original's point to **two** — `1/15` now stands where the original's does
for the whole capture, having parted at 3737 before the first fix and 3988
between the two — and its comparable collision field-frames 348,469 →
**349,794**, none wrong, buildings exact on all 130,326. Great Lakes holds
at 1802 and both scored captures hold. 175 rondata and 690 sim tests green
in `--release`.

**What is at 4275**: five draws against five, differing at the second —
ours a `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99` where the
original's is `< Guy::inc_time+0x271`. A woodcutter's clock, which is
§3.14's shape and item 103's; the count holds to 4288, where the extra
pair is a gaia herd's. Both are past run57's own length, so run58 is what
reads them. Item 141.

**The rules this is an instance of.** Three, and they are all the working
agreement's own:

- **Grep the disk before booking a capture — but an idle screen may still
  run the capture lane.** run58 was booked correctly (nothing on disk
  reaches 4005) and was still not what answered the item; the trace and
  the decompile were, while it ran. Neither the booking nor the reading
  was wasted, and doing them concurrently is why.
- **A draw site names a function, not a unit.** Attributing `do_cast` to
  the scout cost the item its first framing. What un-did it was counting
  the *figures*: two `set_anim` draws at 3608, one at 4020.
- **Grep the writers of every field you call frozen — and the readers of
  every field you write.** `path_flag::TURN_FIRST` was written by
  `shore_flagged` and read by nobody, and `World::tregion` is not
  `get_tregion` however much its name suggests it. That second one is now
  twice in one day (item 138's `collide.rs` gate was the first), which is
  what makes item 142 an item rather than a note.

## 2026-09-01 — item 141: the tile grid's tolerance, and the word goes 4275 → 4313 (Opus)

Booked as a woodcutter's clock. The clock was real and it was a symptom.

**What the trace said.** East Indies' long word parted at 4275 on five
draws against five, differing at the second: ours opened a
`Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99` — ORDERS §6.4's
`SITE_STAND_RETURN`, the tile-choosing branch — where the original spent a
`< Guy::inc_time+0x271`, an animation still running. The count parted at
4288 on a gaia herd. So: a gatherer thirteen frames ahead of its
counterpart, and 4288 the frame the original caught up.

**What the dump said, in one run.** run58 carries the gather order's whole
row on every frame (ORDERS §6.4's `OrderMismatch::Gather`), and its
earliest disagreement is **frame 3836**: the AI's citizen `1/13` holds
`wait 439` where the original holds **452**. Same roll — the streams still
agree — so not a different draw: thirteen frames of counting down that the
original had not started. `wait` only falls inside `0x140` of the tile
centre, so `1/13` had *arrived* thirteen frames early. And `1/13` is the
one unit in run57 or run58 that left the original's point before the word,
at 3647 — item 139, sitting three items down the queue as its own thing.
It was the same thing.

**Where the thirteen frames came from.** Three a leg, over the walk out.
At 3646 both sides stand on (40107, 38862) with the waypoint (40152,
38904) forty-five and forty-two away — and this crate called it reached,
popped the stack, and turned for the next one, while the original walked
the remaining two frames onto it exactly. The arrival test is
`manhattan(waypoint − step) <= UnitData::tolerance`, and the tolerance
comes off the waypoint the pathfinder wrote.

**And the tolerance is the unit's, not a constant.** `astar_path`'s
reconstruction at `00684bfc` (PATHFINDER §7): on the **tile** grid the
waypoint's tolerance is `0` when `anti_unit == 0` and either `unit_masks &
0x800000` without `unit_masks2 & 0x2000` — the auto-transport pair — or
the type carries `unit_flags & 0x10`; `0x60` otherwise. This crate wrote
`0x60` always, under a comment reading "SEAM: no transporters, so always
`0x60`" — true when it was written and false since the transport mechanic
landed. `1/13` is granted `0x800000` on frame **3580**, the frame after
its side's Dock finishes, and cut the corner off every leg from there.

**What moved.** East Indies' long word **4275 → 4313**. run57: two units
ever off the original's point → **one**, and that one is `0/5` on 4001,
the capture's last frame — nothing parts inside the capture at all now,
where the earliest parting had been 3647. Its comparable collision
field-frames 349,794 → **350,928**, none wrong. run58: nineteen units ever
off point → eighteen, and **none before the word**, which is what
`RUN58_PARTED` now pins at 0; its collision field-frames 435,399 →
**443,748**, none wrong, and buildings exact on all 178,326. Great Lakes
holds at 1802 and both scored captures hold. 176 rondata and 691 sim tests
green in `--release`.

**The widening that would have found it, and why it did not.** run39's
`a_path_stack_s_rows_are_compared_whole` has scored `tolerance` and
`flags` on every waypoint since 2026-08-31 and passed every day since —
because run39 is 1,850 frames and East Indies' Dock finishes on 3579, so
no unit in that capture ever satisfies the transport test and every tile
waypoint is `0x60` on both sides. The comparison was right and the capture
was short. Run the same filter over run58 and it opens **1,750 rows**, the
first at 3644, `tolerance ours 96 theirs 0`. That filter is now asserted
empty before the word in run58's own test, and it fails on purpose with
the fix backed out.

**What is at 4313**: the AI scout. The original opens the frame with 28
`Unit::think_scout` draws — one `+0x941` and 27 `+0xaba` — and this crate
spends none. The four re-thinks before it agree exactly (4005, 4110, 4229,
4282, same draw counts), and both sides turn the scout onto (41112, 20376)
on 4305 without drawing for it; then the original re-thinks eight frames
later and this crate waits until 4367. Item 143.

**The rules this is an instance of.**

- **A widening's census is only as wide as the frames it ran over.** A
  comparison that passes is evidence about the capture, not about the
  crate. A rule that switches on a mid-game grant needs a capture that
  reaches the grant — and when a longer one arrives, the standing
  widenings are re-run over it before anything new is written.
- **Grep the writers of every field you call frozen** — and re-read every
  `SEAM:` whose premise a later mechanic has retired. This one said "no
  transporters" and had been false for three items.
- **Two queue items can be one defect.** 139 was booked as `1/13`'s
  parting and 141 as a woodcutter's clock; the parting was the cause and
  the clock the symptom, 189 frames apart. What linked them was diffing
  the *whole* gather row rather than the field the item named.

## 2026-09-01 — `think_scout` has a second caller, and it is a citizen's (item 143)

Opus, in the main thread. East Indies' long word **4313 → 4461**.

**The item was booked wrong, and the trace had said so.** The queue read
"the scout re-thinks at 4313 and this crate does not", on the strength of
28 `Unit::think_scout` draws — one `+0x941`, 27 `+0xaba` — that this crate
does not spend. The first `report.py … sites 4313` prints the whole caller
chain, and it is not the scout's:

```
f4313  27  Unit::think_scout+0xaba < Unit::think_peasant+0x2ac < Unit::think+0x362
f4313   1  Unit::think_scout+0x941 < Unit::think_peasant+0x2ac < Unit::think+0x362
```

The four re-thinks the item cited as agreeing — 4005, 4110, 4229, 4282 —
are all `Unit::think_scout < Unit::think+0x7da < Unit::do_idle+0x94`, the
tail SCOUT §2 documents. 4313's is a different function entirely, and the
two draw sites name the branch: `+0x941` and `+0xaba` are §11's **region
scan**, which §3 routes a citizen to. Both facts were in the fold; the
item's name had been written from the frame number and the site addresses
alone.

**`Unit::think_peasant@005f5760`'s tail** (SCOUT §11.1, `LAB_005f5920`) is
what was missing. Below the job search, for a non-scholar of an AI leader:
the region under the unit's tile by `get_tregion`; if the leader's
`reg_cities[region]` is zero, a walk over the **ten `Sites`** for one with
`val != 0 && reg == region`; none, or `idle > 6` with one, and the worker
calls `think_scout(0)`. `docs/ORDERS.md` §5.9 had carried the arm as the
single line "AI: region/scout logic, then find_repair_spot()" since the
mechanic landed — a transcription stub nobody had gone back for.

Three readings the decompiler does not hand over. `LeaderData +0x6e34` is
a `Sites`, which the type record gives as `Array<Site>` whose `+0x10` is
the `Site *`; the listing walks from `*(+0x6e44) + 0xc` at stride six
ints, so the fields read are `Site.val` and `Site.reg`, and the walk stops
at ten with no length test. `div_3_table[(x ^ 0x63637) >> 6]` is `x /
0xc0`, the tile — `get_tregion@006b52e0` shifts its own arguments right by
two to index `wdata`, which settles the scale. And the scholar test wraps
the `unit_masks &= ~0x400` clear as well as the region arm, so a scholar
is the one worker that keeps "has been a builder" across a failed search.

**What the frame turns out to be.** The AI's citizen `1/15` finishes a
walk on 4312 at tile (158, 138) — cell (39, 34), region 8, a region where
leader 1 has no city and none of whose ten sites carries `reg 8`. It goes
idle on 4313, and with `frame % 8 == 1` over a hundred-cell region the
stride is 2: fifty cells visited, twenty-seven passing the fog and
location tests and scored, and the winner is tile (162, 138) — which
run58's block 4314 holds as the unit's new `EXPLORE_TO`, and its
`PATHDATA to_x 31200, to_y 26592` is that tile's centre exactly. With the
arm in, the whole scan reproduces seed for seed on the frame's own stream.

**What moved.** East Indies' long word **4313 → 4461**. run58's earliest
parting of any kind moved 4300 → **4479** and its count nineteen →
seventeen, so the arm was holding two other units off the original's point
as well; comparable collision field-frames 443,748 → **447,024**, none
wrong, buildings exact on all 178,326, and nothing parts before the word.
Great Lakes holds at 1802 and both scored captures hold. 176 rondata and
693 sim tests green in `--release`.

**What is at 4461**: the AI's Dock. The original opens the frame with
`Guy::init_real+0x52 < Unit::init+0xb97 < Objects::init_unit+0xbd` — the
birth of the ship `1/14`. `1/2010` queues the type-317 job on frame
**4376** and its `job_counter` climbs 100 a frame to **8481**, landing on
4461; this crate builds the same unit on **4489**, twenty-eight frames —
2,800 counter units — late. Both counters are in run58's per-frame
`BUILDQUEUE`, so whether it is a late decision or a slow counter is a
grep. Item 144.

**The rules this is an instance of.**

- **A draw site names a function; the ebp chain names the mechanic.** The
  item was booked off `Unit::think_scout+0x941` and cost a wrong frame of
  reasoning about a scout that was never involved. `report.py … sites`
  prints three frames of chain for exactly this reason, and reading the
  second one is free.
- **A transcription stub is a finding waiting to be found.** "AI:
  region/scout logic" was written when the mechanic landed and read as
  prose ever since; the arm behind it was two predicates and a ten-entry
  walk, and it was the headline.
- **A function's call sites are part of its specification.** SCOUT had
  read `think_scout` line by line and named one caller. There are three,
  and the one no capture reaches is the one the document called "the
  second".

## 2026-09-01 — item 144: the Dock was neither late nor slow, and the word goes 4461 → 4462 (Opus)

The item was booked as a question with two answers in it — "a late
*decision* or a slow *counter*" — and it was neither. The AI's Dock
`1/2010` queues its type-317 job on frame **4376** on both sides, and the
`job_counter` agrees on every hundredth of every frame from there. What
parts is the **target**: theirs caps at **8481** and this crate ran on to
**11,280**. And 11280 × 100 / 133 = 8481.2, which truncates to 8481.

**The 33 is `BRITISH_SHIP_SPEED`, and player 1 is British.** run38's
`PLAYER` block gives `who 1` `tribe 11`, and the roster's eleventh entry
is the British. `ObjectData::train_time@006508c0` runs a block of ten
national arms after the ramp; the British one is a single
`has_tribe_bonus(0xb)` around three tests — the type's domain being the
sea, `is(0xaa)` (Bowmen, the Archers root) and `is(0x119)` (the
Anti-Aircraft Gun) — each `t = t * 100 / (K + 100)`. `BRITISH_SHIP_SPEED`
and `BRITISH_AA_SPEED` ship as 33; `BRITISH_ARCHER_SPEED` ships as **0**,
so the middle test is live and inert at once. The unit being built is a
**Fisherman**, whose domain is the sea, and the AI had been paying a third
too much for it since the capture began.

The arm is checked twice over rather than once. The Dock's *second*
Fisherman has one of the type already owned, so the ramp puts our target at
12,030 where the original's is 9,045 — the same 100/133, on a different
number, four hundred frames later.

**Only the British arm is built, and that is the finding's shape rather
than a shortcut.** The other nine arms are other nations' powers, inert in
every capture on disk, so each would be a predicate no diff could falsify —
and the audit's standing lesson is that the predicates are where a reading
goes wrong. Everything the original applies *before* the British arm is
absent from this game too: the lobby handicap is `0` on both players, and
The President, the Mongol stable, the Japanese barracks and carrier and the
Chinese citizen are all gated by the same `has_tribe_bonus` this player
fails. So starting the tail here is exact, not approximate. What follows it
— `TROOPS_FASTER`, the speed-upgrade counts, the rares, the governments,
the unit wonders — is the queue's own item.

**It was found by widening, and the widening is the whole lesson.** The
queue record has been parsed since 2026-08-30 and compared in exactly one
test, against run39 — a capture 2,600 frames too short to reach the AI's
first ship. Moving that loop out of the test and into `diff::compare` put
it on every capture the harness reads, and run58's twenty-eight frames of
`queued ours 1 theirs 0` at `1/2010` printed the same minute. The record
had been on disk since the run was taken. This is `CLAUDE.md`'s "when the
original dumps a record, diff the whole record" with the emphasis moved one
word to the left: on **every capture**, not on the one whose test happened
to be written.

**What moved.** East Indies' long word **4461 → 4462**, one frame, and the
frame is the point: the ship is now born when the original births it, and
what stands behind it is the ship's first think. run58 now compares
**109,435 queue fields** (238 wrong, all of them past the word and
downstream of the same thing) and 449,279 collision field-frames, four
wrong, none before the word; all 178,326 building fields exact; no unit off
the original's point before the word. run39's own queue test keeps its
floor of 1851 and its ceiling of one, now reading the shared loop's
numbers. Great Lakes holds at 1802 and both scored captures hold. 176
rondata and 693 sim tests green in `--release`.

**What is at 4462**: `Unit::think_fish`. The original makes **55** draws on
that frame opening at `Unit::think_fish+0x27a` (`5f4eda`); this crate makes
2. The fishing AI has no counterpart here at all — `docs/ORDERS.md` §5 has
named it as the head of `think`'s tail since the mechanic landed and
nothing has ever been built below it. Item 145.

**The rules this is an instance of.**

- **A record parsed on one capture is a record not being diffed.** The
  widening's cost was forty lines moved between two functions; its yield
  was the headline, on a dump that had been sitting on disk for hours. The
  ledger item 87 asks for — per dumped record, the fields the parser has
  and nothing compares — should count *captures* as well as fields.
- **A booking's two candidate answers can both be wrong, and the diff says
  so faster than either.** "Late decision or slow counter" was a good
  question; the queue record answered a third thing — the target — before
  anyone had to choose between them.
- **A nation is a modifier, and a capture has one.** Twenty-four nation
  powers have been inert in this crate since `crates/sim/src/nations.rs`
  wired `has_tribe_bonus`, and it took a ship to notice that the AI has
  been playing a nation the whole time.

## 2026-09-01 — item 145: the fishing boat, and the word goes 4462 → 4871 (Opus)

`Unit::think_fish` exists now, and with it the whole of `think`'s tail from
step 5 down that nothing had ever built. East Indies' long word moves
**4462 → 4871**, count and sequence together, and the 409 frames in between
are one AI Fisherman's walk from its Dock to a shoal of fish.

**The mechanic**, in `docs/ORDERS.md` §6.8 and `crates/sim/src/fish.rs`: an
idle fishing boat scores the 289 cells of the 17 × 17 around its own, keeps
the ones that are water in its own `get_tregion` and not coastal
half-land, and takes the best of `1_000_000 / manhattan` — a million if the
cell holds a good it may gather, nothing if it does not — plus `rnd(60)`
**plus the ring index itself**. Standing on the winner, a packed boat
unpacks; otherwise it walks there.

**Three arms, three frames, and each one needed a different fact.**

- **4462 — nothing anywhere.** 54 accepted cells, 54 draws, every numerator
  zero, so the `+ i` term alone decides it and the last index wins. The
  last index is 288, and **`move_y[288]` is `−16` where a clean 17 × 17
  wants `−7`**: a single wrong nibble in the shipped `.rdata`, inside an
  `int[441]` the PDB types, that sends an idle fishing boat sixteen cells
  north out of the square it just searched. The boat's move order goes to
  cell (49, 39) and the dump agrees field for field.
- **4871 — a Fish five cells off, and an Oil five cells off too.** The Oil
  sits at a *higher* ring index, so with `+ i` it would have won on a tie;
  it does not, because `find_good_at` refuses `TypeIndex::OIL` and
  `Objects::init_good@00653f30` never wrote an oil patch into a cell's
  object chain in the first place. It returns before the terminator write,
  which is why every non-oil good on run38's map has `WData.down == −2` and
  every one of the twenty oils has `−1`.
- **4948 — standing on it.** `dist` floors at 1, a million beats every
  jitter, index 0 wins, and the boat casts. `add_cast_order@005e4a60`
  rewrites the generic unpack `0x28c` into the Fisherman lineage's own
  `0x292` on the way in — which is the `spell 658` run58's block 4949
  prints.

**Four things had to be built underneath it**, and each moved a number of
its own:

1. **A packing type is born packed** (`Unit::init:376`). Without that bit
   the 1,024-frame head gate refuses the search and 4462 spends nothing.
   Nothing unpacks yet; `GuyData::turn_speed` now reads the live bit rather
   than a copy taken from the type, which is what the pack turn bonus hangs
   off.
2. **The goods are a layer.** `rondata` parses the `FULL DUMP`'s `GOOD`
   records, `borrow_from_siblings` carries them the way it carries the
   cells, and `World` keeps the terminator `Objects::init_good` writes.
   Sixty-six goods on East Indies; forty-six of them linked.
3. **`come_out`'s sea arm** (`6183b6`): a boat's exit ring is
   `BOAT_TRAIN_*` plus the type's own `big_radius`, chosen on the *unit's*
   domain, not the building's. And `big_radius` — `ObjectType +0x244` — is
   `block_radius` for every type in the shipped build, because
   `UnitType::init` computes `(num_guys − 1) × guy_spacing / 2 +
   block_radius` thirty instructions after storing `num_guys = 1` and
   nothing else writes the field. With it the Fisherman is born on the
   original's point rather than 192 units short of it.
4. **The pathfinder was asking the wrong region function.** All four region
   reads in `astar_path` and the three pull-back walks call
   `WorldData::get_tregion`; this crate called `World::tregion`, which is
   the plain `region_of(cell_of_tile)`. A boat standing on a coastal
   half-land cell therefore answered its *land* region, the pull-back never
   matched, and the goal was dragged three quarters of the way home before
   the give-up exit fired. That is item 142's first payoff, and
   `docs/PATHFINDER.md` §15 has it.

**And a latent crash, 19,000 frames out.** `find_wpath` read
`self.nation[owner].human` and gaia is owner 8, which no `nation` row
covers. run58's own dump settles what the answer should be — leader 8
carries `leader_flags 33554439`, the same `0x2000007` the AI player has,
`& 4` set, against the human's `0x800113` — so an owner with no row is not
human and a wandering animal takes the AI's mode block.

**What is left, and it is now a pathfinder item.** Both AI Fishermen's
*sea* routes part from the original's on the frame they are planned, 4464
and 4870: the same nineteen-cell staircase from (57, 55) to the fish at
(49, 39), two cells north over the first half. run58's test pins both
frames and the boat's own parting at 4507 rather than excusing them, so the
residue cannot grow quietly and cannot be fixed without the test noticing.

**A reading became a check.** `crates/rondata/src/pe.rs` is 80 lines of PE
container — the section table and nothing else — and it reads `move_x` and
`move_y` out of the user's own executable and compares all 289 entries with
`sim::world::MOVE_289`, the typo included. Made to fail on purpose first.
That is the third probe of this shape (`docs/FORMATS.md`'s constants, the
`radius` table, this), so it graduated out of a scratch script.

**The rules this is an instance of.**

- **A table with a typo in it is a table you cannot generate.** Ring 2's
  corner reordering was already known; ring 8's last entry is a different
  kind of thing — not a quirk with a reason but a wrong byte — and it is
  the one the mechanic's very first frame reads.
- **The implementation is a pass of the audit.** Every one of the four
  supporting facts above was found by *building* `think_fish` and running
  the diff, not by reading it: three of them are in functions a reader
  would have had no reason to open.
- **Widen, then read.** 54, 177 and 161 draws on three frames, and the
  three arms they exercise, are what makes this mechanic diff-backed rather
  than plausible. Nothing about `think_fish` needed a capture that was not
  already on disk.

## 2026-09-01 — item 147: the world grid reads the raw region, and the word goes 4871 → 4945 (Opus)

The item was the two AI Fishermen's sea routes, pinned the day before:
`1/14` planned on frame 4464 and `1/16` on 4870, both the same staircase
from (57, 55) to the fish at (49, 39), both two cells north of the
original's over the first half. The queue booked it as
`avoid_land`/`avoid_sea` and `calc_cost`'s ocean terms, and that is exactly
where it was — one line earlier than expected.

**§15 was right about three sites and half right about the fourth.** The
day before, item 142's first payoff had found that all four of the
pathfinder's region reads call `WorldData::get_tregion` — the coastal
refinement that answers a `HALFLAND` cell's `region2` when its tile is
water — and switching the pull-backs to it fixed the boat's *goal*. The
fourth site, `astar_path`'s `avoid_land`/`avoid_sea` derivation, was read
at line 395 of the decompile. Line 395 is inside an `else`. The prologue
branches on the grid first, and the `param_2 == 0x300` arm calls nothing at
all: it reads two `short`s straight out of the `WData` array at
`world+0x134 + (width × cy + cx) × 0x1c + 4` — `types.txt` names the field
`region` — one for the start cell, one for the goal, and compares them.
Only the tile and unit grids get `get_tregion`.

Beside it, the water test the `same` branch then makes is
`WorldData::is_ocean@006b4830` — `flags & 0x100` clear *and* `land` 1 or 2
— where this crate had been asking whether the cell's region was a sea
region, in `astar_path` and in `calc_cost`'s ocean row both. The two
answers part on exactly the cells `HALFLAND` marks, which is precisely
where a boat sails.

**One bit, and the whole route.** The Fisherman stands on cell (57, 55):
`flags 0x104`, `region 11` (land), `region2 65` (sea). `get_tregion` says
65, the fish's cell says 65, so this crate said *same region*, then read
the start as not-ocean (`HALFLAND` never is) and set `avoid_land = 1`. The
original compares 11 against 65, says **different**, and leaves both avoids
at 0. Every coastal cell along East Indies' channel was then costing this
crate 200 the original charges nothing for; the search — which ends on its
3,200-probe budget, not on arrival, and reconstructs from whichever open
node is nearest the goal — ran out somewhere else, with a different node to
walk back from.

With the raw field both stacks agree with the original **row for row**,
point, tolerance and flag: sixteen entries each, `1/16`'s two rows stepping
around the boat already sitting on `1/14`'s cells included. The boats had a
pin of their own in run58's test and no longer need one; `RUN58_PARTED` is
**0** again — no unit leaves the original's point before the word — and the
word is **4871 → 4945**.

**What the new boundary is.** Frame 4945: the original makes three draws
and this crate two, and the extra one is `Guy::set_anim+0x97a` under a
chain the trace's site table has no entry for —
`Guy::do_turn+0x4a < Unit::move_step+0x3b6`. A **ninth caller** of the
animation coin, and a turning stand rather than any of the eight
`crates/sim/src/anim.rs` names. The frame after, the original's
`Animal::do_idle` spends one draw where this crate spends four: the same
guy, re-anchored.

**The rules this is an instance of.**

- **A line number is not a site.** §15 cited `astar_path@00683770:395`
  correctly and generalised it to a function that branches above it. The
  cheap guard is the one the audit README already names — read the
  *loaders*, and read what is above the line you are quoting.
- **Grep the writers of every field you call the same.** `tregion` and
  `get_tregion` are one letter apart and this project has now been bitten
  by the pair twice in two days, in opposite directions.


## 2026-09-01 — item 148: the turning stand, and the word goes 4945 → 4950 (Opus)

Frame 4945 was one draw the original made and this crate did not, under a
chain the site table had no name for: `Guy::set_anim+0x97a < Guy::do_turn+0x4a
< Unit::move_step+0x3b6`. `docs/ANIM.md` §4.7 had already read
`Guy::do_turn`'s override — with `guy_flags & 8` a turn replaces the walk
with `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` — and then argued it could not fire
in a scored game, because "`Guy::init_real@005db6b0:179` sets the bit only
for a guy whose piece names a turn animation" and none of the eight pieces a
`DUMP_ALL` run's guys carry does.

**The word "only" was the whole error.** `init_real` has *two* writers of the
bit and they are independent: `:179`, the packet's `CHAR_TURN_RIGHT` — slot
22, `action_ids[0x16]`, and only that slot — and `:215`, which is the `else`
arm of `(type+0x2b8 & 4) == 0` and therefore sets the bit for **every type
that packs**, whatever its art says. The first reading found one writer, saw
that it explained the flag on every guy it had looked at, and stopped.

So the bit is on the AI's Fisherman, which packs to fish and has no turn
animation at all. `Guy::do_turn` asks it for one; `Guy::set_anim:225–252`
tests `get_animobj` for that exact slot, finds nothing, and rewrites the
request to `CHAR_DEFAULT` — **past** the `param_1 == CHAR_DEFAULT` early
returns, which are the sibling arm and are not re-entered. The request lands
in the common tail as an idle one, and a guy on the **walk** category (which
a sailing boat's is, from `Unit::move_step:304`) has no same-category early
return to take. It rolls. One draw an episode, because the next frame of the
same turn finds the guy idle with its clock running.

**What made it a diff rather than a reading.** `guy_flags` is a dumped field,
and run26's `DUMP_ALL` window has a Fisherman in it. Folding the window's guy
blocks by `(type, gpiece, guy_flags)` states the rule outright: `FISHERMEN`
(317) and `MERCHANT` (61) pack, name no turn, and carry **8**; `CATAPULT`
(265) and `TREBUCHET` (266) do both and carry **8**; `PIKEMEN` (134) names
the turn and does not pack and carries **24**; `TRIREME`, `GALLEY` and
`DROMON` carry **0**. `MERCHANT` is the row that settles it — the packing
writer is the only thing that can explain its bit. Ten minutes of `grep` over
a dump that had been on disk for a week.

**And the arms were laid out backwards.** The first run of the new code put
the count right and the *label* wrong: this crate marked
`Unit::move_step+0x389` where the original's chain said `+0x3b6`. The
decompile lists the near arm first (`:148`) and the far arm second (`:170`),
and the compiler emitted them in the opposite order — `005fb24c` and
`005fb256` both jump **forward** past the far arm's body to the near one — so
`+0x3b6` is the near call and `+0x389` the far one. The listing settles in a
minute what the decompile's line order suggests wrongly, again.

Three call sites reach `do_turn` with the override enabled and all three fire
in a traced game: the two `move_step` arms and `Guy::turn_towards+0x69`,
which `Guy::move:109`'s standing arm calls while `guy_flags & 2` is clear —
52 / 17 / 8 on run54 and 75 / 23 / 6 on run53. All three are modelled and all
three have a site name. `Unit::detect_boat_collision:267` is the fourth and
no capture reaches it.

**What it moved.** East Indies' word `4945 → 4950`; every other capture is
unchanged, which is the expected shape — the only unit in these games that
asks for an animation it lacks is that one boat. The new boundary is the fish
search: both sides spend 161 `Unit::think_fish+0x27a` on frame 4948, and on
4950 this crate spends 165 more where the original spends none (item 149).

**And the new boundary was diagnosed while the context was warm**, which
belongs to item 149 rather than to this one: `transport.rs`'s `do_cast`
`kill_current_order`s every spell but the transport craft's `0x28a`, so the
Fisherman's `0x292` unpack dies the frame after `think_fish` queues it. The
boat is idle again on 4950 and — still packed, so `think_fish`'s
1,024-frame gate does not hold it — searches its 17 × 17 a second time. The
original keeps the order for `SpellTypeData::get_job_time@00675800`, and
the spell table is not in the data layer at all.

`docs/ANIM.md` §4 was over the guard's section ceiling by the time §4.8 was
written, so §§4.6–4.8 now sit under a heading of their own. They keep their
numbers — the code cites them — and the split is what the guard asks for: the
unit is what a session reads.

**The rules this is an instance of.**

- **Grep the writers of every field you call frozen** — the audit README's
  standing lesson, and this is the same failure one level down: a *flag* with
  two writers, read as if it had one. The tell was in the decompile the whole
  time, eleven lines below the writer that was found.
- **Grep the dump before booking a reading.** The flag is printed per guy in
  every `DUMP_ALL` capture. Nothing had ever compared it.
- **When the decompiler prints an order, the listing prints the addresses.**
  Source order and address order are not the same thing, and a site table is
  indexed by address.

## 2026-09-01 — item 149: the fishing boat deploys, and the word goes 4950 → 4988 (Opus)

The cause was named at the end of item 148 and it was right: `Unit::do_cast`
in `transport.rs` opened with

```rust
if order.spell != spell::TRANSPORT { self.kill_current_order(u); return; }
```

so the Fisherman's `0x292` died on the frame after `think_fish` queued it.
The boat came back idle on 4950, still packed — which means
`think_fish`'s 1,024-frame head gate does not hold it — and searched its
17 × 17 a second time: 165 draws where the original spends none.

**What was actually owed was the whole untargeted arm**, read once rather
than the one spell it had been written for. `docs/ORDERS.md` §6.9 is that
reading. In order: `pay_cast_costs` once per order; on the first frame the
animation, which is `CHAR_PACK` for a pack craft, `CHAR_UNPACK` for an
unpack one and `CHAR_DEFAULT` for everything else — with the state test
beside it, so a pack whose caster is already packed and an unpack whose
caster is not both die there; the transport's shore test, which belongs to
`0x28a` alone; `spell_time += 1` against `get_job_time`; `SpellType::cast`;
and **then `kill_current_order`, for every craft but the transport**. The
transport is the exception for a reason that reads as an accident and is
not: `cast_transport` has already moved the order list onto the new boat,
and the boat kills the cast at its own head.

**The craft table is in the data layer now.** `craftrules.xml`'s 55 `CRAFT`
records are `GameAccess::spelltypes`, at `TypeIndex 0x275 … 0x2ab` in file
order, and the loader already read the file — for `FROM`/`FROM2` and the
caster bit alone (`docs/DATALAYER.md` §2). Two more columns were all this
needed: `JOB_TIME`, and the `FLAGS` letters as bits, because `& 0xe` is
what makes a craft targeted. `Deploy (Fishermen)` is **40**, and
`SpellTypeData::get_job_time@00675800` adjusts nine of the fifty-five rows
without touching it. `TRANSPORT_JOB_TIME`, a literal `0` with a comment
apologising for the table not being loaded, is gone.

**The dump had the whole answer and nothing had ever asked it.** run58
prints `spell_time` on every unit record at every detail level: `1/14`
climbs 1 … 39 on frames 4950 … 4988 and drops to 0 on 4989, which is forty
`do_cast` steps and the `JOB_TIME` to the frame. Beside it `unit_masks`
goes `786440 → 262152` — `0x80000` out — and `mylos` goes `4 → 6`. Three
fields, one grep, no reading needed to confirm any of it.

**So two of them became assertions.** `rondata::diff` now compares
`unit_masks & 0x80000` against `Combat::packed` on every unit-frame of
every capture, beside the `mylos` comparison that was already there.
run58: **94,935 unit-frames, none wrong** — and asserted over the *whole*
capture rather than up to the word, deliberately, because the deploy is on
4989 and a check fenced to 4988 could not see the thing the item is about.
Stubbing `cast_unpack` out fails it at 4989 on both counters, which is how
it was tested.

**And the LOS clamp stopped being a seam.** `Unit::update_los@0060e4d0`
caps a packed unit at four tiles — `if (mylos > 3) mylos = 4` — gated on
the type's `unit_flags2 & 4` *and* either `unit_masks & 0x80000` or
`UnitData::is_packing` (the current order is a cast whose spell is a pack).
`docs/VISION.md` §7 had it as the type test alone, written that way because
this crate kept no packing state; it keeps one now. A caster that fails the
gate falls **through** to the merchants' fixed `epoch + 4` rather than
skipping both, which the old shape had wrong as well — inert, since no
capture has a merchant.

**What it moved.** East Indies' word `4950 → 4988`; Great Lakes unchanged
at 1802, and every scored capture unchanged. The new boundary is one draw
on 4988 and it is the deploy's own animation: the original's guy finishes
`CHAR_UNPACK` there and pays the wrap's idle roll, where this crate spends
a farm's. The boat's guy carries **gpiece −1** — `Art::pieces` is seeded
from the start dump's `GUY` blocks alone, and no Fisherman is in one — so
every length lookup misses, every `end_time` is `UNKNOWN`, and no animation
of its ever wraps. That is item 152, and it is bigger than one boat: every
type absent from the opening is in the same position.

**The rules this is an instance of.**

- **Read the arm, not the case.** `do_cast` had been written for the one
  spell that reached it, and the shape that made the transport work — "kill
  everything else" — was the bug. The whole untargeted half is forty lines.
- **Grep the dump before booking anything.** `spell_time`, `unit_masks` and
  `mylos` were on disk at every detail level and none of the three had ever
  been compared. The forty-frame clock was a `track.py` invocation.
- **A finding that can become an assertion must become one.** The packed
  bit is the item's whole state; it is now 94,935 field-frames of guard
  that fails on the exact frame.

## 2026-09-01 — item 152: a unit trained mid-game has a piece, and the word goes 4988 → 5106 (Opus)

**The boat's guy carried `gpiece −1`, and every animation of its was three
frames of nothing.** `Art::pieces` is seeded from the start dump's own
`GUY` blocks, so the table knows exactly the units the opening held. A
Fisherman the AI trains on frame 4,376 is not one of them: every length
lookup missed, every `end_time` was `UNKNOWN`, and the clock never wrapped.
The original's `1/14` finishes `CHAR_UNPACK` on 4988 and pays the wrap's
idle roll; this crate spent a farm's.

**What was owed was not the arithmetic — it was the walk.** §3.2 had
already inverted every `<UNIT name="…">` in `unit_graphics.xml` onto the
piece `init_piece_ranges`' four strides put it at, 1,359 of them, and that
was diff-backed against six pieces of two nations. What it never did was
*pick* one. `GraphicPieces::get_unit_gpiece@0090c030` picks it, and the
function is four nested walks rather than a sum: style, age bracket, gender
and crew give a starting piece, and then it steps the bracket down to zero
looking for a piece the art pool actually has — first with style and
gender, then with gender alone, then with style alone, then with neither,
and `first_unit_piece` if none of the four finds anything. `guy_num` is in
every one of them and is never dropped.

**The gender coordinate is where the mechanic lives.** `-PACKED` and
`-FEMALE` are the *same* slot — no shipped entry carries both — and
`Guy::update_gpiece@005d8530` passes `unit_masks & 0x80000` in the gender
argument's place. So a type that packs is born on its packed piece
(`Unit::init` sets the bit at `:376` and makes its guys at `:540`) and is
moved off it the moment `SpellType::cast_unpack` clears the bit and calls
`Unit::update_gpiece@005e2920`. The two entries are not cosmetic variants:
`FISHERMEN-DEFAULT-AGE0-PACKED` names a `CHAR_UNPACK` and no `CHAR_PACK`,
and the plain entry names a `CHAR_PACK` and no `CHAR_UNPACK`. The deploy's
animation lives on the piece the boat is standing on when it plays it.

**The nation's art style came out of the install and the dump agreed.**
`Tribe::unit_continent` (`+0x68`) is the tribe file's `<UNIT_CONTINENT>`,
written `0 European`, `1 Arab`, `2 American`, `3 Asian`, `4 Iroquois`,
`5 EIndian` — the six arms of `say_unit_art_style_name@006f02e0`.
`Install::tribe_defs` reads it beside the name the roster check already
used, and `cargo run -p rondata` re-derives the link by requiring each
style to name a citizen entry in `unit_graphics.xml`. `Tribe::log_data@
006f0d70` prints the field too, so the dumps state it independently — and
they print `graft[352]`, `barbarian`, `build_continent`, `people` and
`text_substitute` beside it, none of which this crate parses. That is item
153.

**The check is every guy any dump has ever named.**
`the_walk_gives_every_dumped_guy_its_own_piece` runs the walk against every
`GUY` block carrying a `gpiece` in ten dumps of two maps — 126 guys, 12
distinct pieces, four nations, both genders, both crews — and asserts the
number the original printed. Zeroing the style term fails it on the first
unit of the first dump (`left: Some(19)`, `right: Some(371)`), which is how
it was tested. What no row reaches is a second age bracket: no capture ages
a player up, so the `0x840` stride is still arithmetic.

**What it moved.** East Indies' word `4988 → 5106`; Great Lakes unchanged
at 1802 and every scored capture unchanged. 5106 is not a new frame — it is
the one item 151 already named, the single frame run58 reaches
`UnitData::calc_gather@00609180`, `think_fish`'s "can I still gather where
I stand". The queue predicted its own next boundary, which is what a
well-kept queue is for.

**The rules this is an instance of.**

- **The table was not the mechanism.** A month of piece work had produced a
  correct table and no function to consult it, and the gap was invisible
  because every unit the harness stood up came from a dump that already
  carried the answer. It only showed the day a unit was *trained*.
- **Prefer a diff to a reading.** The walk is a hundred lines of
  decompiled control flow with four near-identical loops; nothing about
  reading it was going to say whether it had been transcribed right. 126
  dumped guys said so in eleven seconds.
- **Grep the dump before booking a reading.** `unit_continent` was printed
  in every `DUMP_ALL` dump on disk, and so is the rest of the `TRIBE`
  record that TECH still calls unsourced.

## 2026-09-01 — item 151: a deployed boat stays on its fish, and the word goes 5106 → 5376 (Opus)

`UnitData::calc_gather@00609180` was the last thing between `think_fish` and
a fishing boat that behaves. The queue had it as "the head's *can I still
gather where I stand* test — unmodelled, so this crate answers no", and the
consequence was exact: on frame **5106** run54's `1/14` is sitting on its
fish, the original spends four draws on the whole frame, and this crate sent
the boat back through the 17 × 17 for 165.

**The scales are the mechanic.** The function has two searches and both walk
the octagonal spiral `circle_x`/`circle_y` in **tiles** — `div_3_table[v >>
6]`, not the `>> 8` the rest of `think_fish` uses — out to
`circle_radius[r]`, where `r` comes from `ObjectTypeData::upgrade_level@
00661090`: the `FROM` chain's length while each ancestor is still in the
starting type's lineage, `× 4 + 4` for a merchant by id or the `FISHERMEN`
lineage and `+ 2` for anything else. `<FROM>none</FROM>` on the shipped
Fishermen makes that **4** tiles, one cell.

A tile qualifies on its surface field — `(TData & 0x30) == 0x20`, ocean, for
everything that is not a merchant by id, and `!= 0x20` for one that is — and
on `TData & 0x200`. Then the good is looked up in that tile's **cell**. That
mismatch is the whole of why run58's dump prints `good_obj 1` and not 0: the
fish's object marks its own tiles, the boat is standing on a different tile
of the same cell, so ring 0 fails the `0x200` test and ring 1 carries it.
Reading the walk as cells would have produced a function that answered the
right question with the wrong index, and only the dumped field would ever
have said so.

**The return value is a crowd test, not a search result.** Having found a
good the function walks the object chains of the `circle_radius[2]` cells
around the unit — cells now, not tiles — and counts the owner's other live,
on-map, unpacked units of its own lineage inside `(their r + mine) × 192`.
The six gather rates are divided by `count + 1`, and with `param_7` set —
which is both gameplay callers — **any** competition returns 0. So a crowded
boat is told to move; only a boat alone on its fish stays. The distance test
came off the listing at `609761`..`609824`, where the decompiler had lost the
arguments to `vector_dist` entirely.

**Two fields nothing had ever compared.** `UnitData::rare` (`+0x54`, the
union with `air_alt` and `former_type`) and `UnitData::good_obj` (`+0x94`)
are in every `UNIT` record the log prints, and `grep -rn good_obj crates`
returned nothing. Reading them was what settled the ring index — 210 frames
of `rare 6, good_obj 1` from 4992 — and it also settled that they cannot yet
be *asserted*: the same trace shows `rare −1` arriving on 4872 from
`Unit::think@005f6e40:179`'s rare-collector arm and `rare 6` on 4992 from the
gather job, and this crate models neither. That is item 155, and the
widening is booked with it rather than added half-true.

**`unit_masks & 0x20` cannot be set by either gameplay caller.** `*param_2`
is zeroed at the top and written `1` only past a `param_7 != 0` test that
both callers fail. The grep confirms it independently: no `unit_masks` value
in run33, run39, run53, run54 or run58 carries the bit. A seam the queue had
carried as an open question turned out to have a one-line answer and a
dump-wide check.

**And then the second half, which the diff found for free.** With the head
landed the word moved to **5285**, where the AI's *second* Fisherman accepted
178 cells against the original's 177. `Object::add_to_world` pushes every
object onto the head of its cell's list, so a boat that has deployed onto its
fish **is** that cell's `WData.down` — and `think_fish`'s claim test was
reading the snapshot a start dump had loaded, which still held the good's own
terminator. Preferring the live unit chain, and keeping the snapshot only as
the fallback for the buildings and goodies this crate does not thread, moved
it to **5376**. One reader of item 48's chain, landed by a single frame's
draw count.

**What it moved.** East Indies' word `5106 → 5285 → 5376`; Great Lakes
unchanged at 1802 and every scored capture unchanged. 5376 is the first
divergence on this map in five sessions that is not a fishing boat's:
`Leader::produce_building+0x1805`, 37 draws to 33, the AI siting a building
this crate never scores a site for.

**The rules this is an instance of.**

- **The listing settles what the decompiler loses.** Ghidra printed
  `vector_dist(unaff_EDI, unaff_ESI)` — two registers it had no value for —
  and `llvm-objdump` gave the two operands, the `× 0xc0` and the `jge` in
  ninety seconds.
- **Grep the dump before booking a reading.** `good_obj` and `rare` were in
  every record on disk. One `grep` said the ring index was 1, which is the
  single fact that would have been easiest to get wrong and hardest to
  notice.
- **A finding that can become an assertion must become one — or say why it
  cannot.** The two fields are dump-comparable and are *not* compared, and
  the reason is written down with the item that unblocks it rather than left
  as a gap in the ledger.
- **The queue predicted its own next boundary twice.** 5106 was item 151's
  own frame, named a session before it was taken; 5285 was item 48's chain,
  named months ago. Neither cost a search.

## 2026-09-02 — item 154: three of the six resources were available from frame 0, and the AI's Coinage lands on 5177 (Opus)

Item 154 was the headline — `Leader::produce_building+0x1805` on East Indies
frame **5376**, the AI siting a building this crate never scores a site for.
It is two defects deep. The first is landed and it is a general one; the
second is measured and named, and the word has not moved.

**What the frame actually is.** The trace names the caller chain, and it is
the script: `Leader::produce_building+0x1805 <
ScenarioFuncSet::place_orphan_building_with_cost+0x136 <
ScenarioFuncSet::place_building_with_cost+0x6a`, four draws, the 2×2 jitter,
no `+0xc99` — so the type is not gather-scored. `economic.bhs` case 15 is
"Build Market #1", and case 23 ("Commerce II") jumps to it with `step = 15`
on a sea map when `research_tech_with_cost(who, "Coinage")` succeeds. The
harness's own host log said this crate was stuck in case 23, spending all
five of the script's loops on a Coinage that would not start.

**Why it would not start, and what that turned out to be.**
`research_tech_with_cost` refused on price: `charges = [0, 60, 0, 140, 0, 0]`
against a bucket of `[258, 94, 150, 100, 100, 100]` — a hundred and forty
knowledge against a hundred held. Coinage is `14k/6t` in `techrules.xml`, and
Knowledge's `PREQ0` in `resourcerules.xml` is the **Classical Age**. The AI
is in the Ancient age, so the original never asks it for knowledge at all: it
pays `UNDISC_COST_GOOD Food` at `UNDISC_COST_RATE 3/2`, `(140 × 384) >> 8` =
two hundred and ten food, and the sixty timber.

`economy::Holdings::available` and `discovered` were **all-true from frame 0
for as long as they had existed** — `holdings.rs` said so in a doc comment,
"the tech layer", and no tech layer ever wrote them. So the whole
undiscovered-redirect layer beneath them — read in the 2026-08-20 audit,
tabled in `cost::Redirects::RON`, unit-tested against hand-built arrays,
corrected once when it turned out to be reading the `*_SUPPORT_*` columns —
had never fired in a played game. `Sim::sync_goods_available` writes both
arrays now, from `type_avail`, out of `set_tech_tree` and `apply_gained`.
`TypeData::can_pay_cost@00667570` is the confirmation that it is the right
test: its loop over the six goods opens with `LeaderData::type_avail(good,
1)` before it asks `get_cost` anything, and `docs/TECH.md` had already
settled what that comes to for a good.

**What it moved.** run58's queue tail: twenty-four rows of `1/2005 queued
ours 0 theirs 1` — the AI's library holding a Coinage job from frame 5177
that this crate never started — are gone, and both sides now take it on
**5177 exactly**. That is the sub-score item 154 named, and it is the item's
first half. 178,326 building fields, 488,303 collision field-frames and
109,627 queue fields still compare clean; what is left of run58's tail is
frame **5201** alone, where the original prints `queued 0` for both buildings
that held a job on 5200 and prints the whole `BUILDDATA` list a second time,
truncated — the run was quitting. Both rows are asserted as they stand.

**What it did not move, and what the number is.** The word is still 5376. The
AI now reaches case 15 and calls `place_building_with_cost(who, "Market",
my_capital)` on that exact frame, as the original does — and `affordable`
answers **0**. A Market is `8t`, eighty timber; the AI holds thirty-four,
having paid the sixty for Coinage on 5177. Its timber income is `1280`
sixteenths and it accrues about three timber every twenty-five frames, so it
would place the Market around frame 5760. The original places it on 5376,
which means the original holds at least eighty there and this crate is
**about forty-six timber short by 5376**.

That is a resource level, and no capture on disk carries one past frame 800:
`LEADERS=1` at `[End Frame]` is five scalars and no goods, and the only
`LEADERS=9` windows are run40's `[560, 600)` and run41's `[770, 800)`, both
on run10's game rather than this one.
`run40_s_census_…` is the instrument that exists — it compares `bucket`,
`leftover`, `resources`, `income`, `resource_cap` and `gather_slots` per good
per player per frame — and it says food, timber and wealth are exact there,
so whatever is short arrives later. Two moves are booked with the item, in
order: **widen the `CITY` record**, which run58 dumps every frame and which
`rondata` parses four fields of, for the initial dump only — `gatherers`,
`busy`, `free`, `filled`, `space[scan]` and `ter[scan]` are the rate's own
inputs and nothing has ever compared them; then, only for what that cannot
answer, a `LEADERS=9` census window on East Indies at 5150–5400.

**And a second finding, inert, from the same reading.**
`Leader::init@006e3930` zeroes all six resources; `Leader::gain_tech@
006dcb60` walks the six on every gain and, for one whose bucket is zero and
whose `goodtypes[g] + 0x30` prerequisite is the tech just gained, calls
`bucket_add(g, game->starting[g])`. **`STARTING_GOODS` arrives with the age.**
That is why run40's dump holds `200 200 100 0 0 0` where this crate holds
`200 200 100 100 100 100` — the 240 rows that test has booked as inert since
it was written, now explained. It stays inert (an unavailable good is never
charged and never accrues) and it is booked as its own item, because where
the three that need no age are paid is not read: the loop in `gain_tech`
cannot pay them, and `Game::init_starting_resources@0058a500` only computes
the array.

**The rules this is an instance of.**

- **A field nothing writes is a layer nothing runs.** The redirect had a
  table, a recursion, a cycle guard, a `rondata` check and its own tests. All
  of it was correct and none of it executed, because one array upstream was
  a default. The tell was in `holdings.rs`'s own doc comment, which named
  `available` as somebody else's job for months.
- **Grep the dump before booking a capture** — and grep the *harness* before
  grepping the dump. `run40_s_census_…` had already measured the 100 in
  goods 3, 4 and 5 and written down that it was inert "because none of the
  three is available in the Ancient age". The sentence that explains the
  bug was sitting in the assertion that measured it.
- **The script is the AI.** Two of this item's three facts came out of
  `economic.bhs` and `aibestbuildlibrary.bhs` rather than the decompile: that
  case 23 jumps to case 15 on a sea map, and that case 15 is the Market. The
  shipped scripts are readable and they are the opening.

## 2026-09-02 — item 154(a): the `CITY` record, four fields to forty, and the reason the AI is poor (Opus)

The opener said to widen the `CITY` record before booking a capture, because
run58 dumps it every frame and `rondata` read four of its forty fields — at
frame 1, on one capture. That is what this session did, and the widening
answered the item it was booked under.

**The record.** `CityData::log_data@004895c0` writes one block per **live**
city — the whole body is behind `if ((city_flags & 1) == 0) return` — at
every detail level, on every frame a block exists. Forty fields, catalogued
in `docs/CITIES.md` §5.7. Two traps in it, both cheap once seen: the
`length`/`size`/`increment`/`flags` between `capture_strength` and `o` are
`Array<CaravanLink>::log_data@00489390`'s header and not the city's, and
`city` is the slot **within the owner's own array** — both starting cities
of a two-player game are `city 0`.

The comparison went into `compare`, so it now runs on every capture the
harness reads, and `run58_s_five_thousand_frames_stand_where_the_original_s_do`
pins it: **606,540 fields compared, 489,001 of them new agreement**, with
every open row pinned by `(who, o, field, first frame, count)` rather than
filtered out.

Two artifacts of the check itself had to go first, and both are worth
writing down. `reg` read as one wrong on every city of every capture, because
the simulation numbers its regions as it finds them and the dump numbers them
as the generator wrote them — `Built::region_map` is the translation, and the
census check had been going through it for a fortnight. And `city_flags` is
compared **bit by bit**, not as a word: four of its bits are the "an active
TEMPLE / GRANARY / LUMBERMILL / MARKET stands here" marks, which this crate
keeps on the economy's city record instead, so a whole-word compare would
have reported every frame of every game and hidden the bit that moved.

**What was left, and the one that matters.** `ter[6]` — the best per-good
gather amount over the city's occupied tiles — is zero on every city of
every frame, because `World::gather_at@006b07f0` is a declared seam that
answers `[0; 6]`. It is not inert while it does. `gather_value`, the gather
families' half of the make-list score, reads it per good and refuses the
good outright:

```
let ter = f.ter[g];
if !(ter != 0 || oil_ok) { continue; }
```

`oil_ok` needs `oil_patches.count`, which reads 0. So with `ter` zero the
loop admits **no good at all**: the AI's make list can never ask for a farm,
a camp, a mine or a university. Item 154 was "the AI is 46 timber short of
the Market it sites on 5376" and this is the shape of an answer to it —
booked as item 157, with the oracle already running, per good, per city, per
frame.

**And the original sweeps for the human leader.** `Leaders::strategy_all@
006ed430`'s gate is `leader_flags & 3 == 3` — in play, not defeated — with no
human test, and `Leader::plan_strategy@006b9620` has none either. The human is
filtered one level down, in `Leader::production_ai@006c1960`, whose first
statement sends a human without computer assist to the switch's `default` —
which **clears the step machine**, so the sweep re-arms and runs again on the
leader's next phase frame, forever. `Sim::strategy_all` skips the human
outright, so thirteen fields of the human's city are zero on all 5,201 frames.

The dump had said so twice over and nobody had looked: the human's city
carries a full site picture from frame 1 (`busy 5`, `land 84`, `filled 40`,
`space 50/50/44`, `ter 1/2/0/1/0/0`), and its `peasant_dist` moves on frame
**401** — leader 0's own phase, `frame % 200 == 0`, one frame late in the log
exactly as leader 1's 175-phase changes show at 376, 576, 776. Booked as 158,
not fixed, and the reason is in the item: this crate's sweep would then also
run step 16, which seeds an army, and no capture has a human one.

**The rules this is an instance of.**

- **Diff the whole record.** Nine tenths of this one had gone uncompared for
  a month; the first widening found a hard gate on the AI's whole economy.
  The unit of a widening is the record, not the field the mechanic wants.
- **A seam that answers zero is not neutral.** `gather_at` was written as a
  no-op "kept for its shape", and its zero silently disabled every gathering
  building the make list could ever want. A seam is only honest while
  something measures what it costs — which is what the record now does.
- **Check the checker before believing it.** The first run reported 131,324
  wrong fields; 13,785 of them were `reg` asking the wrong numbering. A
  widening's first output is a claim about the comparison, not about the sim.

---

## 2026-09-02 — `World::gather_at`, and a gate on a road nobody drives

*Opus. Item 157. The word did not move: East Indies 5376, Great Lakes 1802.*

The seam yesterday's widening found is closed, and it took a morning rather
than a session, because everything it needed was already on disk.

`World::gather_at@006b07f0` is thirty lines of the original and three pieces:
a table, a class, and a predicate.

**The table** is `rules.xml`'s `LANDS` — nine records, four `<MAKE num type>`
each, read by `Lands::init@0067e730` into a `0x138`-stride array with the good
indices at `+0x04` and the amounts at `+0x14`. Five of the nine make anything:
plain land a knowledge and a food, forest a timber, mountains and cliffs a
metal, an oil cell an oil, all of them **one**. `"none"` becomes `TYPE_NONE`,
−1, which the consumer's *unsigned* `< 6` test rejects — one of those places
where the sign of a comparison is the whole safety of an array index.

**The class** is `WorldData::get_land@006b4730` with its third argument 1:
five flag tests ahead of the stored `WData.land`, which is only four names
deep (`land_key[]`: `BASELAND`, `SANDY`, `OCEAN`, `NONE`). The same five tests
were already sitting inline in `army.rs`'s muster search, written from
`docs/ARMY.md` §13 without either reading knowing it was the other's — so the
implementation is one function now and the muster search calls it.

The class also settled a name. `WData.flags & 0x800` had been unnamed since
run20, because the map dump prints no word for it and run20 never sets it.
`WorldData::is_oil_at@00472af0` is that bit and nothing else. It is **`OIL`**.

**The predicate** is where the reading actually took work.
`GoodTypeData::is_flat@004780c0` sits at vtable slot `+0x94` of `GoodType`,
which the base `Type` leaves as the engine's return-zero stub — so the
decompiler prints, at both call sites, an *inlined* body calling a
devirtualised `ObjectTypeData::is` at slot `+0x60`, with the sense apparently
inverted between the two arms of the same `if`. Reading it as written gives a
contradiction. What resolves it is the map file: `??_7GoodType@@6BType@@@` is
at `00b44b70`, slot `+0x94` is `004780c0`, and that function is

```
!(is(TIMBER) || is(METAL) || is(OIL))
```

Food, wealth and knowledge are **flat**; the three the ground actually holds
are not. That split is the whole shape of `gather_at`: a flat good comes off
the cell you stand on at its face amount, a non-flat one is doubled — and, in
the neighbourhood arm, summed over nine cells first.

Sixty seconds with `xxd` over the PE beat an hour of arguing with a
decompiled `if`. That is the export README's own lesson and it keeps being
right.

**Step 13 passes `centre_only = 1`**, which is why every `ter` in every
capture is 0, 1 or 2 and never more, and why a tile some building already
gathers (`TData.mask & 0x1000`) is worth nothing at all to the non-flat half.
run58 frame 1: the AI's `ter` is `1/2/0/1/0/2` — a knowledge, a doubled
timber, a food, a doubled oil — exactly what the table gives.

The diff agreed on the first run. Nine pinned rows went — `1/2000`'s `ter[0]`,
`[1]`, `[3]`, `[5]` and `1/2007`'s `ter[0]`, `[1]`, `[3]`, `[4]`, `[5]` —
**39,309 field-frames** that had disagreed, first frame to last, per good.
`ter[2]` and `1/2000`'s `ter[4]` had never disagreed, because on that map
neither city's circle holds a mountain or anything that makes wealth.

### And then the score did not move

5376, unchanged. So the honest question is what item 157's premise was worth,
and a probe answers it in one run: `gather_value` is never called in run58.
Nor is `Sim::building_value`, on either pass, on any of the 5,201 frames. The
AI's camps and farms there all come off the script path (§19).

`ter` was a hard gate — on a road no capture drives down. The gate is open
now and diff-exact where the sweep runs, and what stands between the AI and a
make list is somewhere above it, in the step machine. Booked as item 160, the
new headline-nearest, and the two remaining zero-readers of the same block
(`GoodType::compute_largest_gather@0066e920`, `oil_patches.count`) go behind
it rather than in front.

**The rules this is an instance of.**

- **The item nearest the headline is a guess until the run says so.** Item
  157 was booked as "why the AI is poor" on a reading of `gather_value`'s
  first `continue`. It was a true reading of a function nothing calls. One
  `eprintln!` in a `--release` diff run would have said so before the work,
  and cost less than a minute — check that the path executes before booking
  the thing that gates it.
- **A session that moved no score says so in the handoff.** This one did not,
  and the queue's first line is the number that did not move.
- **The listing settles what the decompiler cannot.** Three call sites
  printed a predicate with two opposite senses; the vtable slot read straight
  out of the PE printed one function with one sense.
- **Two readings of one function are one implementation.** `army.rs` had
  `get_land` inline and `world.rs` needed it; nobody had noticed because
  neither document cited the other's address.


## 2026-09-02 — item 160: the make-list road is not driven for another 4,600 frames (Opus)

The opener said to find out what reaches `create_buildings` in run58, because
the make list never does. **Nothing does, and nothing is supposed to.** The
original does not reach it either — not in run58's 5,201 frames, not in the
whole of East Indies' word, not for another four and a half thousand frames
after it.

**The instrument was already on disk and this crate could not read it.** The
trace's HIT records are function-entry coverage, and outside a `window=` the
arming is one-shot from attach — so a whole-run capture carries one record
per function, on the frame the original first entered it. `report.py` has
printed them since the instrument existed; `rondata::trace` **dropped** them
at parse. Keeping them is nine lines, and it turns "which functions has the
original ever run, and when" into a `#[test]` fact.

What the two 24,000-frame traces then say, in one grep:

| step | function | East Indies (run54) | Great Lakes (run53) |
| --- | --- | --- | --- |
| 2 | `production_ai_setup` | 9977 | 6377 |
| 4 | `research_techs` | 9979 | 6379 |
| 5 | `upgrade_units` | 9980 | 6380 |
| 6 | `create_units` | 9981 | 6381 |
| 7 | `create_buildings` | **9982** | **6382** |

Five consecutive frames with one gap, and the gap is step 3 —
`found_cities`, entered at frame **576** already, where a one-shot arming
does not fire twice. That is `docs/AI.md` §2.4's step machine read straight
off the original: the script at step 1 answers `BLOCK_ON_THIS` on every
sweep while it is live, `BLOCK_ON_THIS` clears the machine, and so the
ladder cannot start until the script *ends*. The shipped opening runs for
two and a half hours of game time.

**And the 576 is the other half.** `found_cities` and `make_stuff` are
entered there on every East Indies and Great Lakes capture, thousands of
frames before steps 3 and 8 can run, and their caller is
`ScenarioFuncSet::place_city_with_cost` — the **script's** host function.
Every producer entry before the script ends is the script's. §3's "the
skirmish opening is the shipped script" was a reading of two `.bhs` files;
this is the same sentence as coverage.

**Two instruments, one game, no daylight.** run18b is run53's own game with a
`LEADERS=9` window over exactly those frames, and it had been read (§15.6) as
one sweep among many. Laid against the coverage it dates the same ladder to
the frame: `prod_script_run` falls **1 → 0** at the end of sim-frame 6376 —
`SCRIPT_DONE`, the only arm of the switch that writes it — and
`production_ai_setup` runs on 6377, which is run53's HIT. So the transition
is confirmed twice from two sides, and `docs/AI.md` §25 records both.

**What it means for the queue.** `create_buildings` is first driven 4,606
frames past East Indies' word and 4,580 past Great Lakes'. So
`building_value`, `gather_value`, the `ter` gate, `oil_patches.count` and
`GoodType::compute_largest_gather` — the whole block item 157 opened and
items 160 and 161 sat behind — cannot move either headline until the word
reaches those frames, and no dump on disk is long enough to compare them.
They go to the back.

**The rules this is an instance of.**

- **Grep the disk before booking a reading — and the disk includes the
  trace.** The question "does the original reach this function" had an
  answer in a file that had been sitting there since run53 was taken. It
  cost one `report.py functions | grep`, and it retired an item booked as
  the headline's nearest.
- **A finding that can become an assertion must become one.** The answer was
  a grep; the assertion is twelve frame numbers on two maps plus five rows
  of run18b's dump, and it fails on any of them.
- **The previous session's own rule, applied one level up.** Item 157's
  lesson was "check that the path executes before booking the thing that
  gates it". Item 160 was booked to find *why* it does not execute. The
  cheaper question was whether the original executes it at all.

## 2026-09-02 — item 154(b): fifty timber, measured at last, and it is a lump (Opus)

Item 154 left the headline as a number nobody had measured. East Indies'
word is 5376, the frame is `economic.bhs` case 15 asking for a Market at the
AI's capital, and a Market is eighty timber: the original places it and this
crate holds thirty-four. The item's own note booked two moves in order —
widen the `CITY` record, which the last session did, and then, only for what
that could not answer, a `LEADERS=9` census window on East Indies at
5150–5400. This is that window.

**It ran beside the work and cost none of it.** Launched in the first
minutes, read in the last, with item 160 done in between. Thirteen minutes,
446 MB, and `rngcmp.py` against run54 says 5,401 frames with **zero**
differing, so it is run58's game exactly.

**And it is cheap in a way the older census was not.** `censuswindow.sh` had
the shape with run10's game hardcoded and a poll that could not survive a
late window; `longtrace.sh` could only take the expensive `DUMP_ALL` kind,
which at 250 frames would be tens of gigabytes. A `FRAME_WINDOW` hook —
narrow the *cheap* per-frame dump instead — is nine lines, and it makes a
late census cost minutes: with no `[End Frame]` block written before 5,150
the game reaches the window in under a minute, where run58 spent fifty
getting there.

**What it measures.** 18,000 good-frames, two players, six goods, six
fields. 4,798 disagree, in nine shapes, and **every shape is wrong on all
250 frames** — a level, not an event.

- **The AI is fifty timber short, and `leftover` agrees on 234 of 250.** The
  fractional accumulator can only agree if both sides are paid the same
  amount every frame, so the fifty was banked before the window opened: a
  lump. On 5377 it flips to thirty *ahead*, because the original has spent
  eighty on the Market and this crate has not. That flip is the whole of
  East Indies' word in one field.
- **`gather_slots` is short by exactly the right amount.** The human, who
  builds nothing at all in this game, has **0** timber slots here against
  the original's 6 — its starting camp's, and this crate claims none of
  them. The AI has 4 against 10, which is the same six plus the four its own
  camps add. `TIMBER_BONUS_PER_WOOD_SLOT` is 5, paid per slot past the
  high-water mark, and ten slots is fifty timber.
- **The food slots are exact on both players**, which is what makes this one
  lineage rather than an array nobody writes.
- **Two rate seams beside it, and both are the AI's alone**: `income[food]`
  1440 against 1600, `income[wealth]` **0 against 160** with the AI's
  `bucket[wealth]` eighteen *ahead* anyway. Every one of the human's six
  incomes is exact on every frame.
- The wealth *slot* — 1 on both players, 0 here — is item 82, open since
  run40 and now measured on a second map.

**What it did not move.** The word is still 5376: this session measured the
cause and did not fix it. The successors are booked with what each is worth,
and the first of them is a lineage — the starting camp's slots — rather than
a search.

**The rules this is an instance of.**

- **The capture lane costs wall-clock, not attention.** run57 and run58 were
  each "second-best" because they were booked well and read late. This one
  was booked for the item the headline actually rests on, launched before
  the session's real work started, and answered it the same hour.
- **Widen the record, then read the residue.** The `CITY` widening was the
  move the item named first, and it is what cleared the field of everything
  that was *not* the cause. The census then had one thing left to say.
- **A shape that is wrong on every frame of a window is a level.** Nine
  shapes, nine standing states, and the one with an agreeing `leftover`
  beside it is the one that names a lump. That test — rate or lump — cost
  nothing and pointed straight at `Build::activate`'s tail.

## 2026-09-02 (Opus) — the fifty was a goody box, and a dock's thirty is why it picked wrong

Item 162 said the fifty timber was ten gather slots' bonus. It was not, and
the arithmetic said so before any capture did: the bonus is paid per slot
*past the high-water mark*, `Build::activate` pays nothing at frame 0, and
so the starting camp's six raise the mark unpaid on both sides — twenty
timber for the AI's own camp either way. **The human was the control**: six
slots short here for the whole of run59's window, and its timber bucket
exact on all 250 frames. An unpaid slot costs nothing.

**The slots were still a real defect, and closing them cost twenty minutes.**
`Build::init` fills a camp's tile list at placement and `Build::activate`
surveys `gather_max` out of it; a camp stood up from a dump could never
survey anything, because the dump's own tile masks go into the world
verbatim and the camp's tiles already carry `0x1000`, so the walk skips every
cell. The harness installed the dump's list *after* `activate` had already
counted the empty one. Moving two statements closes 500 of run59's 4,798
wrong good-frames — and moves the timber not at all, which is the
measurement that refuted the item.

**Then the disk answered the question the item could not.** "When was the
fifty banked" had been booked as needing a second capture. It did not:
**run42 had been on disk for two days and nothing had ever read its leader
block.** It is run39's game at `LEADERS=2` — the detail the encrypted block
is announced at — so it prints both leaders' six goods on all 900 of its
frames. 54,000 good-frames compared, and the only rows are item 156's three
unavailable goods. The AI's food and wealth *rates*, wrong on all 250 frames
of run59's window, are exact for the first 900.

**And then the capture, which was five minutes.** run59 narrowed an
expensive `[End Frame]` to 250 frames. run60 does the opposite and it is
strictly better: keep the window open for all 5,400 frames and make the
*block* cheap — `DETAIL_END="MISC,LEADERS=2"` and nothing else. 67 MB, under
five minutes, `rngcmp` zero differing. 324,000 good-frames, and the AI's
whole bucket curve parts on exactly **three** frames in 5,400.

- Frame 1: item 156.
- **Frame 3579**, wealth thirty behind. The AI's **Dock**.
- **Frame 4988**, timber fifty behind — and wealth fifty *ahead*.

**One event, one good, wrong.** 4988 is a second goody box. The trace cannot
show it: its coverage records fire on a function's *first* entry, and
`explore_goody` had already fired on 867. The pile is fifty either way and
the frame spends three draws either way — the lottery's *winner* is what
differs. Going in, the original held food 208, timber **99**, wealth 130;
this crate held food 208, timber **100**, wealth **100**. Thirty-one clear
of the field, timber wins outright against a jitter of at most 24; tied at
100, the jitter picks — and it picked wealth. `docs/GOODY.md` §6 had left
"whether a box's good is right" as answerable from disk and unanswered. It
was answerable, and the answer was no.

**And 3579 is why they tied.** `Build::activate` line 590, which this
document had called "a third `do_bonus` off a separate pair of counters at
`+0x8ac`/`+0x8dc`". They are not separate: `LeaderData +0x8a4` is
`gather_slots` and `+0x8d4` is `gather_slots_high`, so those two addresses
are the **third entry of each**. A dock, a market or a temple is a *wealth
gather slot*, and the first past the mark pays thirty. The wealth slot item
82 had been chasing since run40 — "a slot under good 2 that `get_good`'s
table cannot produce" — and the thirty nobody paid are the same line. The
`is_dock` half is a vtable call the decompiler leaves as
`(**(code **)(... + 0x108))()`, and it is settled without the PDB's vtable
records: `ObjectData::is_dock@004711e0` is that call and nothing else.

**So one unread block at 3579 cost fifty timber at 4988 and the Market at
5376.** Implemented, mirrored in `Build::close`, tested, and run59's census
falls 4,798 → **3,500** — the whole timber lineage, bucket and rate. Items
162 and 82 close together.

**The word did not move, and what blocks it has changed.** East Indies is
still 5376, but not for the old reason: the AI now pays its eighty and
places the Market, and the two sides' timber agrees across the purchase. The
frame parts on `Leader::produce_building+0x1805` — the placement jitter,
once per unblocked sub-position of the 2×2 — **two draws here against the
original's four**. That is a `blocked_site` disagreement at the AI's
capital, and it is the new opener.

**The rules this is an instance of.**

- **Grep the disk before booking a capture** — and then grep it again. run42
  answered half the question for the cost of a parser, after a document had
  written down that a capture was needed.
- **Build before ratifying.** The item's prose cited every address
  correctly and had the arithmetic wrong. Nothing but doing the work found
  it: the fix landed, the number did not move, and that was the finding.
- **An instrument's shape is part of its evidence.** The trace says "has
  this function ever run", never "did it run here", and a *second* goody box
  is exactly the event that shape hides. Two hours went into the frame's
  function coverage before the buckets were simply read off the dump.
- **The cheap capture beat the clever one.** A narrow window on an expensive
  block was thirteen minutes for 250 frames; a wide window on a cheap block
  was five for 5,400. Ask what the block costs before asking what the window
  should be.

## 2026-09-02 — a friend is a footprint, and the city is four cells' neighbour (item 164)

*Opus.* The opener asked which of the Market's four sub-positions
`blocked_site` refuses here and clears there. The answer was neither: both
sides refuse the same two, and the disagreement is a cell earlier.

**The elimination came off the disk, not off a reading.** The frame spends
two `produce_building+0x1805` draws against four, and four could be one call
with four clear sub-positions *or two calls of two* — the AI placing a second
building this crate never places. run60's timber curve settles it in one
line: `(1, "bucket", 1)` never appears in its shapes, so the AI's timber
agrees on every one of 5,400 frames including the eighty paid at 5376. One
building, one call, four draws.

**So the cell is wrong, and `find_friends` is why.** The spiral scores a
candidate by its neighbours, and this crate asked *which building's centre
is in that cell*. `ObjectsData::find_building_placed_at@00658c80` asks
*whose footprint covers the cell's centre tile* — it walks the nine cells
around `tile >> 2`, follows each object chain, and tests
`corner ≤ tile < corner + size`. Centre tiles are four apart, so a building
of four tiles or fewer covers exactly one and the two readings agree; five
to seven tiles can cover two on an axis. The exception is therefore the
**7×7 city centre**, which is the neighbour of four cells — and it is the
building every site is scored against.

With the Village counted, the Market's cell goes from `(49, 53)` at 4251 to
`(49, 52)` at 6252, four rows clear of the Library the old corner ran into,
and the jitter spends the original's four draws. **East Indies' word:
5376 → 5437.** Nothing else moved.

**And 5437 is Great Lakes' frame.** The site there is `5e8d09` —
`Unit::do_air_physics+0x639`, the bird that has bounded 1802 since run33.
One unmodelled draw now stands in front of both headlines, which is the
first time the two maps have wanted the same item.

**The rules this is an instance of.**

- **Diff before reading, and the diff was already on disk.** The whole
  "is it one call or two" branch closed on a `grep` of a test's own output.
- **A predicate is where the error is.** The arithmetic of `find_friends`
  — `+1` diagonal, `+2` cardinal, `(f + 2) × 1000` — was right from the
  first reading. *Which building is a neighbour* was not, and no test
  written from that reading could have caught it.
- **The size that breaks the rule is the one that matters.** Twenty-two
  items lived with this because every ordinary building is four tiles wide.

## 2026-09-02, Opus — the bird flies, in a module nobody calls

*Item 120. `docs/SYNC.md` §3.9 "The flight", `docs/DECISIONS.md` 30,
`crates/sim/src/air.rs`, `crates/sim/src/single.rs`. **The score did not
move**: EastIndies 5437, GreatLakes 1802, both still parted by
`Unit::do_air_physics+0x639`.*

One draw bounded both maps, so the item was the headline twice over, and
the first two thirds of it went exactly to plan. `do_air_physics` is a
wide function written for the game's aircraft, and a wild bird answers
almost every question in it the same way every frame: `is_animal` excuses
`check_fuel` and the landing approach; `AirOrder::returning` being zero for
a patrol excuses `land_plane`, the bank's doubling arm and both of the
places the speed is cut; owner **9** being over eight excuses the
ground-clearance test that would otherwise make altitude matter, which is
why `pitch_aircraft` — a monster — turns out to be irrelevant to where a
bird goes. What is left is a heading, a bank, a step and one coin.

**The coin is thrown when the step leaves the map, and that is all.**
`UnitData::invalid_loc` on an air type (`type +0x218 == 2`) returns valid
before every terrain test, and the caller passes zero for all five flags
that reach the rest, so the whole of `+0x639` is the world's rectangle.
`AirOrder::sharp_turn` — the type record's own name, at `+0x10`, beside
`cruising_alt` and `returning` — takes the coin's ±1 and stands until a
step lands inside again, which is why run53's 55 coins come in clusters a
hundred frames apart rather than one a frame.

**The bank is the state and the heading is downstream of it**, and that is
the finding worth keeping. A bird cannot turn until it has banked into the
turn: `bank_aircraft` moves a single-precision accumulator by at most ten a
frame toward ±55, and `air_turn_speed` returns `(TURN_SPEED/55)·|bank|`
floored at half a degree — reading the bank the *previous* frame left, so
the first frame of every turn is at the floor and a reversal costs a frame.
A bird therefore overshoots its patrol point, flies straight past it while
more than 45° is owed inside `0x300`, releases at `0x300` and comes round on
a radius near 400. That orbit is a limit cycle: perturbing a bird's initial
heading by one unit leaves its position 1,700 frames later unchanged to the
unit.

**`single.rs` is the second software float, and the first algebra.**
`combat::f32_sqrt` reproduces one expression; the banking is a dozen
operations with branches on their results, so this reproduces `addss`,
`subss`, `mulss`, `divss`, the two conversions and `cvttss2si` as exact
integer arithmetic with one round-to-nearest-even, checked against the
host's own float over thousands of random bit patterns. The constants came
out of the PE: 55, 0.33, 0.5, 2, 10, −55.

**And then it did not work, and the honest thing was not to land it.**
Wired into the unit loop the flight puts coins in the original's *epochs*
and not on its frames — 20 against 55 over run53's 24,000, the first at
2781 where the original's is 1802 — and on East Indies it throws one at
**5404**, thirty-three frames before the original's 5437, taking that map's
word down with it. Every arm was re-checked against the listing; the turn
rates the implementation produces are exactly `(rate/55)·|roll|` frame for
frame; a one-unit perturbation changes nothing. The residue is a *phase*
error in an orbit whose period is near a hundred frames, and it compounds
through the landing search, which reads the patrol point's region and so
picks a different cell once the bird is anywhere else.

So the module lands with its own tests and the call site does not. That is
`docs/DECISIONS.md` 30: a floor does not fall for a mechanic that is only
*nearer* than the one it replaces, because the score is the only thing
measuring us.

**What the session actually bought.** The reading, whole and cited, so the
next session does not redo it. The arithmetic, exact and tested. A
falsifiable statement of the residue. And the next move, which is not
another reading: the bird has one observable because nothing dumps owner 9,
and `tools/trace/`'s `CALLS` proxy already logs a chosen function's
arguments — `Unit::set_new_location@005f8d20` takes the new position, and a
window of it is a per-frame record of where a bird is. Build the field the
diff is missing, then the residue is arithmetic rather than search.

**Two smaller things fixed on the way.** A bird's patrol point is now
seeded at its hatch cell rather than read back off the bird's own position
— which was harmless while the bird stood still and is the first seven
frames of a flight otherwise — and a bird now carries the `MOVES` and
`TURN_SPEED` its type gives every other unit.

## 2026-09-02 — item 120, closed: the bird flies, and both words move (Opus)

The last session left the flight read, implemented and **not called**,
because calling it took East Indies' word from 5437 to 5404. It named the
next move exactly right and then stopped: build the oracle. This session
built it, and the whole thing — proxies, capture, diff, fix, wiring — took
under two hours.

**The capture.** Three entries added to `tracer.c`'s `CALLS`, each needing
three things off the listing and nothing else: `Unit::do_air_physics@
005e86d0` (`ret 0xc`, three args, prologue `55 8b ec 83 ec 28`),
`Unit::air_turn_speed@005ea390` (`ret 8`, two, `55 8b ec a1 f0 61 c0 00` —
the absolute `mov eax,[0xc061f0]` displaces safely), and
`Unit::set_new_location@005f8d20` (`ret 0x10`, four, `55 8b ec 83 ec 20`).
Then run60's own recipe with `callwin=0-5400`: five minutes, 23 MB, and
`rngcmp.py` against run60 says **5,401 frames, zero differing**. A proxy
costs the stream nothing, which is the fact that makes this instrument
usable at all.

**The idea worth keeping is the bracket.** `set_new_location` is taken by
every unit that moves, so its records alone cannot say which are a bird's.
Proxying the *dispatcher* as well makes the nesting the identity: a step is
a bird's exactly when a `do_air_physics` on the same `this` is still open.
No guess about a pointer, no correlation by position. Any mechanic whose
state is private to a class of unit can be read this way — proxy the
dispatcher and the mutator together — and it costs a five-minute capture
rather than a reading. `Trace::air_frames` and `Trace::air_births` are the
readers; `report.py … calls` prints them nested.

**What the record said.** 47,533 air frames, eleven flyers, born at 128,
160, 224, 480, 576, 576, 960, 1120, 1440, 2624 — every one a multiple of
32, which is `Objects::process_all`'s sampling cadence, so the hatching
already agreed. Fed the original's own goal and seeded from its own birth
state, `air.rs` reproduced **all ten wild birds exactly to frame 5,400** —
45,712 frames, every position and every zero-crossing of the bank. The
reading had been right in every arm.

**The error was twenty-four position units.** Every bird was put down at
its patrol point plus exactly `(24, 24)`, which is not a wander: it is
`Unit::init@00612100`'s first two lines snapping the requested position
onto the centre of its 48-unit tile, `div_3_table[p >> 4] · 0x30 + 0x18`.
`Object::init` is handed the snapped point and `add_air_patrol_order` keeps
the unsnapped one, so the two differ from birth. `Gaia::spawn_bird` had
handed the cell centre to both. One line.

The initial heading turned out to be already right, and pleasingly so:
`Unit::init` writes `0x55555555` into `UnitData::angle`, a fill pattern
that happens to be a third of a turn, and a bird is the one unit whose
first frames never overwrite it — so the fill pattern *is* its heading.
`movement::Angle::INITIAL` had carried it since the movement work. The
eleventh flyer is the dock's gull, born on a `StrafeOrder` at 3579 and
pointed due west before its first physics frame; it is the exception that
shows the rule, and it stays unmodelled.

**The score.** With the snap in and `do_air_physics` called from
`Sim::do_idle`, East Indies' long word goes **5437 → 5466** (the next cause
is `PathFinder::calc_road_cost+0x46`, item 57's) and Great Lakes' **1802 →
2419**. Great Lakes' *scored* capture, run33, now runs out its own 1,850
frames without parting, so the scored floor and the long word are two
numbers again and `LONG_WORD_GREAT_LAKES` exists to hold the second.

One label was missing on the way: at 5437 both sides threw the same coin
and the sequence still failed, because `rondata::trace::SITES` had no name
for `0x005e8d09`. A site the simulation marks and the table does not name
reads as a disagreement, which is worth remembering — the word moved to
5466 on the arithmetic and the *sequence* only followed once the name was
added.

**The lesson for the queue.** The previous session's verdict — "the residue
is a phase error, and no reading will settle it" — was correct, and its
successor was correctly booked as a capture rather than a reading. That is
the rule working: when a mechanic's product is arithmetic and the diff has
nothing to compare, build the field before booking anyone to read.

## 2026-09-02 — item 57, closed: a count is not a sequence (Opus)

**East Indies 5466 → 5592.** The opener named the cause and it was right
about the site and wrong about the size: 5466 is one road search — the AI
Market's own placement frame, 184 nodes against the original's 205, every
draw at `PathFinder::calc_road_cost+0x46` — and it is the *third* instance
of item 57's residue, after run32's Granary (1,046 v 1,043) and Smelter
(1,460 v 1,870). All three are on the frame a building is **placed**.

**The item had been stuck for a month because a count is not a sequence.**
Six searches matched the original's node count exactly; the two that did not
had survived three readings of §4–§5, a height-grid chase that cost a
session, and five separate measurements that each ruled a hypothesis out.
Nothing on the record could say *which node* — `calc_road_cost` computes a
number and hands it back, and the dumps print state.

**So the missing thing was an oracle, and run61's recipe built it in an
hour.** `tools/trace/tracer.c` gained three `CALLS` rows —
`astar_caravan_road` (the bracket), `valid_roadcoord` (the gate) and
`calc_road_cost` (the price) — and run62 is run32's own capture with
`RON_CALLWIN=99-101`. Thirteen minutes; `rngcmp.py` against run32 says zero
differing frames on all 111. **The pairing is the instrument**:
`calc_road_cost` takes a pooled `PathNode *`, so its record names an address
and no tile; the tile is the one the `valid_roadcoord` before it admitted.
Frame 100: 2 brackets, 3,028 gates, **2,913** prices — the road-draw count
exactly.

**The first run named the term.** The two sequences agreed in coordinate for
19 nodes; eleven of those nineteen prices differed, and **every difference
was a multiple of three**. `climb × 3` is the only multiple of three in
§5.2. So: the heights, and nothing else.

Two mechanics were behind it.

**§7.4 — a building flattens its ground before it plans its road.**
`Wall::start@0063e810` is five statements and the order is the whole
finding: `kill_competing_buildings`, `tile_corner`, **`Terrain::object_
placed`**, `mask_me` (whose tail is `place_roads`). §7.1 had concluded the
opposite from the one experiment it had — run32's frame-104 grid makes the
Granary cost 967 — and that experiment was right about its grid and wrong
about the mechanic, because frame 104 carries *both* footprints' terraforms
and the Granary must see only its own. Priced against the frame-104 grid the
sequence went from 19 nodes to **460**, which is what said it.
`crate::terrain` is the implementation: the mean of a box one corner wider
on the near side and two on the far, abandoned outright if any corner is at
or below zero; the interior takes the mean and the border `(h + mean) / 2`;
three refusals — a water cell, a mountain or cliff in the corner's own 3×3,
a good on the cell. `World` carries the **corner** grid now, because the
per-tile table the loader pinned once turns out to have a writer.

**§7.3 — `calc_road_cost` calls `was_seen`, not `was_really_seen`.** With
the heights right, six prices remained and each was *exactly twice* the
original's — the fog doubling. The two functions are one letter apart
(`006b53f0` and `006b54f0`) and this called the bare one; the one the cost
function calls has the ally-territory shortcut ahead of the fog read.
`crate::ai_sites` already had that shortcut from run20, and it still could
not fire: it reads `reg_cities`, the census fills `reg_cities`, and this
crate runs the census for AI leaders only — so the human, whose road this
is, had an empty array. `Sim::leader_reg_cities` answers from the census
where there is one and from step 8's own recount where there is not.

With both, run32's two searches are the original's **node for node**: 2,913
prices, every tile, direction and cost, and 1,043 and 1,870 exactly. The
test asserts the sequence now, not the count.

**One regression on the way, and it is worth writing down.** The first
run54 with the terraform on parted at frame **10** — worse than 5466. The
dump's height grid is *already* terraformed for every building the dump
lists, so standing the roster up through `Wall::start` flattened flat ground
a second time, and a second pass over a box whose border was blended is not
the identity. `start_of_game`'s tail puts the dump's grid back. The general
shape: **a loader that replays the original's own state must not re-run the
side effects that state already contains.**

**The score.** East Indies **5466 → 5592**; Great Lakes unchanged at 2419
(no regression); run32's counts exact for the first time. The next cause on
East Indies is `Unit::think_scout+0xaba` at 5592.

**The lesson.** *Diff the whole record* has a sibling: **when the record is a
number the mechanic computes, proxy it.** Item 57 was booked as a reading
three times and closed by a thirteen-minute capture, because the reading
never had anything to be wrong about — the arithmetic was right, and the
world it was fed was not.

## 2026-09-02 — the citizen that would not stand still (Opus)

**The item.** East Indies' word at **5592**, the opener's, and its name was
`Unit::think_scout+0xaba`: forty-six draws here against thirty-three there,
parting at index 31, where this crate ran a whole region scan the original
did not run at all.

**Which arm, from the caller offset.** The original's own `think_scout`
frames near it are 5455 and 5665, and the trace prints the caller: 5455 is
`Unit::think_peasant+0x2ac` and 5665 is `+0x2ca`. Those are the *two* call
sites of `docs/SCOUT.md` §11.1 — `005f5a07`, taken when the ten-site walk
runs off the end, and `005f5a25`, taken when a site does claim the region
and the worker has been idle seven frames. The listing settles which is
which in a minute. So before anything was captured, the record said the
original's **site list** had changed between 5455 and 5665, and that the
walk had not.

**The capture.** run63: run58's recipe with the `[End Frame]` narrowed to
`[5430, 5700)` and `LEADERS=9` in it, so `Leader::sites` and every unit's
position are both on the record either side of the parting. Sixteen minutes,
482 MB, `rngcmp.py` against run54 zero differing on 5,701 frames
(`docs/ORACLE.md`, run63).

**What it said.** The citizen `1/15` walks 137 frames to the target §11's
scan gave it on 5455 and arrives on 5592 — and this crate's walks it
*exactly*, position for position, arrival frame included. Then the original
stands there for seventy-three frames while its `idle` byte climbs one step
every sixteen, and re-targets on 5666 at `idle 7`. On 5577 its tenth site
becomes `(34, 33) val 9728 reg 7 dist 2`: the citizen's own cell, scored the
moment it stands there.

**The cause, and it is one flag.** This crate scored every cell of that
region zero. `compute_site_stats`' step 2 zeroes the base when
`blocked_site(TOWN, …)` refuses, and `blocked_location` refused every cell
of an unsettled region with `COLONIZE 0x1c`. The gate is
`has_preq(COLONIZE_BONUS 0x2af)` — and `COLONIZE_BONUS` is not a nation's
bonus at all. It is the **fourth of `rules.xml`'s 122 `TECHBONUSES`**, whose
one `PREQ` is `preq0="Coinage"` and whose `DESC` reads "Can colonize new
continents". `crates/sim` carried it as a `Nation` field nothing ever set,
so no AI in any capture could ever put a city on a second island. It is
`Roles::colonize_preq` now, loaded exactly as `TRANSPORT_BONUS`'s
prerequisite already was. `docs/CITIES.md` §2.6.1's open question 4 is
closed by the data.

**Why no earlier capture could have found it.** East Indies' AI takes its
Coinage job on frame **5177** — the same frame `RUN58_QUEUE_TAIL` names —
and run58, the longest full-detail dump on disk, ends at 5201. The bonus
lands inside run63's window and nowhere earlier.

**A widening that was free, and half the answer.** run59's census has been
printing `Leader::sites` whole since the day before and nothing had ever
compared it: ten slots, six fields, 250 frames. Fifteen minutes with it said
the site *values* were wrong — `(45, 52)` scoring 996 against 240, `(44, 52)`
1625 against 403, and one extra site taking an empty slot the original
leaves alone — before the capture was booked. That is now pinned in run63's
test rather than found twice.

**The score.** East Indies **5592 → 5669**; Great Lakes unchanged at 2419.
Two new position residues came out of the window, both on ground no capture
had ever reached: `1/18` runs five frames ahead of the original on the same
order and the same row from 5430, and `1/17` turns three frames late on the
same line at 5552. Both are pinned rather than filtered.

**The lesson.** *Grep the dump before booking a reading* has a sibling one
level down: **a draw's caller offset is a predicate's answer**. Two call
sites eighteen bytes apart told which branch of `think_peasant` the original
took, and therefore what its site list held, on a frame no dump covered —
which turned "why is the walk different" (it was not) into "which site
claims that region", and made the capture a confirmation rather than a
search.

## 2026-09-02 — the whale under the word (items 168, 165 and half of 170; Opus)

The opener booked item 168: East Indies' word at **5669**, where the original
spends **175 draws** at `Unit::think_fish+0x27a` off `Unit::do_idle`'s tail and
this crate spends none. run63's `[5430, 5700)` window already covered the frame,
so no capture was owed.

**It was not a `think_fish` bug, and the boat was not even late by its own
doing.** The 175 draws are `1/17`'s — the AI's third Fisherman, arriving at
`(38040, 30360)` on 5668, going idle on 5669 and searching. This crate's `1/17`
arrived on **5691**, twenty-three frames later. Item 170 had that as "turns
three frames late at 5552", read off the step deltas; the step deltas were the
answer and nobody had divided them. The original's boat steps `(-6, -44)` from
5552 and `(-28, -36)` after the turn — magnitude ~45. This crate's stepped
`(-5, -37)` and `(-24, -30)` — magnitude ~38. **The original's boat was faster
from 5552, and the turn came later here because a slower boat reaches its
waypoint later.**

`UNITDATA myspeed` says it in one line: on run63's block 5552 all three of
leader 1's Fishermen go **38 → 45** and its Transport Barge **25 → 30**, and
its citizens and its scout do not move. `38 × 120 / 100` is 45; `25 × 120 / 100`
is 30. `Unit::update_speed@006055c0` has exactly one arm that scales a naval
type by a percentage, and it is **Whales**: `WHALES_SHIPS_MOVE`, `20%` in
`rules.xml`, gated on rare bit 25 of `LeaderData::rare`.

**So the item was the economy's.** On block 5552 the AI's `1/16` — the second
Fisherman, deployed on 5543 and idle since 5544 — takes `rare 31, good_obj 1`.
Good 31 is `WHALES`. The writer is `Leader::calc_gather@006ceee0` **step 6**:
walk the player's units, hand every `is(FISHERMEN)` or `is_merchant` whose
`order_type` is `NONE` to `Unit::do_gather@005fce20`, add what each is standing
on and light its bit in `rare_owned`. `holdings.rs` has listed that step as
"not modelled" since the module existed.

**And item 165 was the same step from the other side.** run59's census had the
AI's `income[food]` at 1440 against 1600 and `income[wealth]` at 0 against 160,
on all 250 frames, since its `1/14` settled on a fish at 4992. `160` is
`10 × 16`, and `10` is both of Fish's `BONUS_NUM`s in `resourcerules.xml` —
which is where `LeaderData::calc_rare@006e08d0` reads a rare's payout from, not
from any constant in `rules.xml` as ECONOMY's open list had it. One mechanic,
two booked items, and the headline.

**What landed** (`crates/sim/src/rares.rs`, `docs/ECONOMY.md` step 6):

- `UnitData::calc_gather`'s **`param_7 = 0`** form, which is a different
  function from the one `docs/ORDERS.md` §6.10 specified: a packed fisherman or
  merchant is refused before anything else happens, the crowd count has no
  mid-unpack exemption, and the three outputs — rates, the `BitMask<44>`, the
  per-good tally — are live where `think_fish` passes null for all three.
- `calc_rare`: the good's two `(BONUS_TYPE, BONUS_NUM)` pairs times sixteen,
  then `MERCHANTS_BONUS[level]` **replacing** the 100 on a resource the unit
  stands on friendly ground for, or on the non-food half of Fish and Whales;
  then `FISHERMEN_BONUS[level]` **added**, to the food slot of those same two
  goods alone. Both ship inert at level 0, which is why the census's number was
  the raw `10 × 16` twice.
- The two upgrade ladders as `rules.xml` `TECHBONUSES` rows 19–21 and 99–102.
- `Leader::gather`'s tail — `rare = rare_owned | rare_conquest`, and
  `0x4000000` when it moves — and `Leader::calc_unit_stats@006cf970`, which
  `Leader::process` runs in the **same frame**, before any unit steps.
- `Unit::update_speed`'s whales arm, and nothing else of that function: no part
  of the rest of the pipeline is modelled anywhere in this crate, so recomputing
  a cached speed from `MOVES` loses nothing.

**The piece that was nearly missed.** With all of the above, the whale was
still claimed nowhere near 5551: `should_recompute` never fired in the window.
The dirty flag is what makes the 512-frame refresh an 8-frame one, and its
writer here is `Unit::check_idle@006032c0`'s **tail** — a per-unit latch
(`ObjectData +0x8 & 8`) taken the first frame a unit is idle, raising
`0x2000000` when the unit `is(0x13d)` or is one of the three merchant ids, with
`Unit::work@0060d180:268` clearing the latch and raising the same flag when the
unit has an order again. `holdings.rs` had listed that writer as "the same three
kinds" — peasant, scholar, merchant. It is fishermen too, and step 6's
membership is exactly what those two transitions change. With the latch the
recompute lands on 5551, seven frames after the boat went idle, which is where
the dump has it.

**The score.** East Indies **5669 → 5819**; Great Lakes unchanged at 2419.
run59's census went from 3,500 wrong good-frames of 18,000 to **1,500**, and
every one of the remainder is item 156's three unavailable goods — both
players' six rates and six incomes now agree on every frame of that window.
run63's two position residues are down to one: `1/17` is exact, and `1/18` is
what is left.

**The new frame is `1/18`'s.** At 5819 this crate spends one draw the original
does not — `Unit::come_out+0x25ca < Object::eject_contents+0x292 <
Unit::set_new_location+0x2b7`, a transport putting its passengers down — and
run54's trace has the original's on **5823**. `1/18` is the Transport Barge,
and run63 has it running ahead of the original on the same order and the same
row from the window's first frame. Whatever cost the original those frames is
in `(5202, 5430)`, which nothing on disk measures.

**The lesson.** *Diff the whole record* has a corollary about **which** record:
`myspeed` is in every `UNITDATA` block of every capture on disk and nothing had
ever compared it. One `track.py --changes` over eleven frames turned an item
booked as a turn-model residue into a rare-resource bonus, and took a second
booked item with it. The step deltas were on the screen for the whole of the
previous session's write-up; the ratio 38 : 45 was not computed.

---

## 2026-09-02 — item 171, closed: the barge was not late, it was born in the wrong water (Opus)

**The opener booked a capture, and the capture was already on disk.** Item 171
said `1/18` — the AI's Transport Barge — ran five frames ahead of the original
from the first frame of run63's window, that the cause lay in `(5202, 5430)`,
and that nothing on disk measured a frame in there. A `LEADERS=9` window over
the gap was the named next step.

**run59's census covers `[5150, 5400)`**, and it is a `UNITS=3` window like
run58's frames and run63's: every dumped unit's point is in it. The census
test read six goods a leader and compared nothing else in the file. One
`track.py UNITDATA … --where who=1,o=18` said the rest in a minute: `1/18`
does not exist before **5342** and it is born already loaded, already moving,
at `(33551, 21378)`. This crate bears it on the same frame at
`(33456, 21424)` — ninety-five units further down its own route. There were
never five lost frames; there was a birth cell two tiles wrong, and the gap
grew to five frames only because the two boats then walked slightly different
first legs onto the same row.

**Where the cell comes from.** A land unit that walks into the water does not
step onto it: `Unit::set_new_location` converts the step into a `0x28a` cast,
and `SpellType::cast_transport` builds a barge at the caster's snapped tile
and walks it to the water `UnitType::find_nearby_spot` found (TRANSPORT §6.1,
§6.2). Both boats' sweeps agree ring for ring — `r = 216`, bearings from
`0x55555555` in the order `0, +1, −1, +2, …` — and they part on one bearing:
the original takes `k = +1`, tile `(699, 445)`; this crate refused it and took
`k = +2`.

**Why it refused it.** `find_nearby_spot@0061de70` has two collision halves,
and `0061deb0` chooses between them: the pairwise pair —
`Objects::find_collision` and `find_ordered_collision`, Chebyshev in unit
cells against the sum of two `coll_size`s — is selected only when the filter
is `FILTER_NOT_ME`/`CAN_COLLIDE` **and `not_o` and `not_who` are both
non-negative**. `Unit::find_nearby_spot@00617010` fills those from `+0xa` and
`+0x9`, so every unit call site gets the pair and the crate was right
everywhere it had been checked. `Unit::do_cast` and `cast_transport` pass
`(-1, -1)`. They get `ObjectsData::find_unit_with_radius@00659890` instead:
a **distance**, `vector_dist(spot − it) <= its big_radius + r_coll`, over
players' live on-map units, with `r_coll` the asking type's own `+0x240` —
and its ordered sibling is skipped outright, because that one is guarded on
`not_who >= 0`.

The two disagree on exactly this bearing. The caster is the AI's scout: block
1, `big_radius 48`. The barge is block 3, `r_coll 144`. The radius test's
reach is **192** and the scout is `vector_dist(140, 172) = 228` from the
candidate — free. The Chebyshev test reads the same pair as offsets of 3 and 4
cells against `3 + 1`, and refuses. One bearing, one boat, three hundred
frames of the word.

**What landed.** `Sim::find_unit_with_radius` in `collide.rs` replaces
`find_collision_for`, and `find_nearby_spot_type` is its only caller —
`docs/COLLISION.md` §5.2.1 has the predicate, both arms of the original's
search and why the whole-array one answers the same, `docs/ORDERS.md` §10 and
`docs/TRANSPORT.md` §6.1 point at it.

**The score.** East Indies **5819 → 6164**; Great Lakes unchanged at 2419.
The sequence parts at 6164 and the count holds to 6165, so the floor is the
lower of the two, as it has always been.
run63's window went from one standing position residue to **none** — 6,775
unit-frames, nothing ever off point — and run59's 5,959 unit-frames are now
asserted alongside its goods, which is where the finding came from and where
it stays checked. run59's dump ends with the quit block, headed 5401 while it
holds 5400's state (the file goes 5399, then it); the test names that and
leaves it out rather than pretending the capture is a frame longer than it is.

**The lesson, and it is the same one twice in two days.** "Grep the disk
before booking a capture" is not a preference; it is the first step. run63's
own write-up said run59 had been printing `Leader::sites` for a day with
nothing comparing it. This session's item was booked on the sentence "nothing
on disk measures (5202, 5430)" — and the capture that measures it had been on
disk since 02:11 the same morning, with 5,959 unit positions in it that no
test read. The widening cost twenty minutes; the capture it replaced would
have cost sixteen and produced a file nobody needed.


## 2026-09-02 — item 173, closed: a unit is not one figure, and the Caravan is the word (Opus)

**The item.** East Indies' long word stood at 6164 on a draw the original
spends three times and this crate once: `Guy::init_real+0x52`, the idle
variant every figure of a new unit rolls. On 6165 the original spends two
`Guy::set_anim+0x97a < Unit::do_idle+0x7d` at the head of the frame that this
crate does not. The unit is the AI's first Caravan, and the opener asked two
questions — where the guy stack is sized, and which `unitrules.xml` column
feeds it.

**The answer, and it is two fields rather than one.**
`Unit::init@00612100:471`–`508` sizes the stack to
`crew_size (+0x30c) + squad_size (+0x304)`, pops a `Recycler<Guy>` into every
slot, and then walks `0..length` giving each figure a `Guy::init_real`.
`UnitType::init@0061ab50:723` writes the literal **1** into `+0x304` for
every type — unconditionally, on the line before it reads `UBER_SIZE` into
`+0x308` and `CREW_SIZE` into `+0x30c` — and nothing else in the image writes
it. So the count is `CREW_SIZE + 1`: 1 for a Citizen, 2 for a Scout (the
dog), 3 for a Caravan, 4 for a Trebuchet. `docs/ANIM.md` §3.5.

The field the queue had been calling the count, `guy_mark`, is `Unit::init`'s
**copy of `squad_size`** — which is why it reads 1 on all 128 units of run59's
window, and why it is not the length. It is what shrinks as figures die.

**What made this survive a month.** A unit stood up *from* a dump never had
the bug: `build_sim` hands it the `GUY` blocks the file prints, so twenty
scout dogs in the corpus walked their own bodies correctly the whole time,
and the crew machinery in `anim.rs` — the track offsets, `guys_follow`, the
mirror past `squad_size` — was written, tested and right. Only
`Sim::init_guys`, the path a unit **born in the sim** takes, had
`let count = 1usize`. The oracle and the defect never met until a capture ran
long enough for the AI to train a crewed unit.

**What landed.** `Profile::crew_size` carried from `unitrules.xml` through
`load.rs`; `sim::anim::SQUAD_SIZE`, the executable's literal, replacing two
hardcoded ones; and the widening that makes it a diff rather than a reading —
`every_dumped_unit_has_crew_size_plus_one_figures` asserts `CREW_SIZE + 1`
against the `GUY` blocks of every player unit of every `DUMP_ALL` capture on
disk. 120 units, 20 of them two-figure, and it was made to fail first.

**The score.** East Indies **6164 → 6166**; Great Lakes unchanged at 2419.
Two frames — the smallest move any item has booked — and the frame it leaves
is a different mechanic entirely: 3,207 `PathFinder::calc_road_cost+0x46`
draws on 6166 that this crate does not spend, two frames after the Caravan
that owns them was born.

**A seam the widening opened.** `build_sim` sets `Unit::squad_size` from
`u.guys.len()`, and `Unit::squad_size` is documented as the original's
`curr_uber_size` — which `0060a760` computes by walking the `o_up`/`o_down`
chain of `UnitData`, not by counting figures. The two are disjoint in the
data: all 109 `UBER_SIZE 3` types (the foot infantry) have `CREW_SIZE 0`, and
every crewed type has `UBER_SIZE 1`. So a scout is dealt attrition as a squad
of two and an infantry squad as a squad of one. It is booked, not fixed — no
capture on disk measures attrition on either.

**The lesson.** The widening rule has a corollary this item is the proof of:
*a field the harness reads from the dump is a field the simulation is not
being asked to derive.* Ten `DUMP_ALL` captures printed the guy count, the
harness copied it faithfully, and that faithful copy is exactly what hid a
hardcoded 1 from every test for a month. Where the harness can seed a value
from the original, the assertion that the install would have produced the
same value is the one worth writing.

## 2026-09-02 — item 174, closed: the road was the caravan's, and the estimate was wrong on purpose (Opus)

Item 174 was booked as one road: on East Indies' frame 6166 the original
spends 3,207 `PathFinder::calc_road_cost+0x46` draws and this crate spends
none, two frames after the Caravan item 173 gave its figures to was born.
The queue's opener guessed the gap was `find_road`'s *caller*. It was — and
the caller turned out to be a whole object nothing here modelled.

**`docs/CARAVAN.md` is the new document.** `docs/ROADS.md` opens by saying
its search's name was wrong, that `astar_caravan_road` runs for a
*building* and no caravan is ever in the game. On 6166 one is, and the arm
a building never takes is four things at once: a `CaravanData` per route,
twenty a leader, whose whole 0x44-byte layout the PDB's type record names;
`think_caravan`'s idle gate and its `FILTER_CAN_TRADE` city search;
`do_trade`'s destination loop; and `build_road`, which is the road search
with the caravan arm on.

**run64 is the capture, and it was designed as two halves.** run54's game
to 6,180 frames with a `DUMP_ALL` window on `[6164, 6172)` *and*
`callwin=6163-6172` for run62's three road proxies: the *sequence* the
proxies carry, and the *state the sequence reads*. Twenty minutes, 563 MB,
`rngcmp.py` against run54 zero differing over 6,181 frames. Four separate
findings came out of it inside an afternoon, and three of them were only
visible because both halves were taken together.

**One: the heuristic is wrong, and that is the mechanic.** Every non-root
node's estimate has two arms on `param_6 < 0`, and the decompiler prints
both as `vector_dist(dx, dy)`. The listing at `00685fd9` does not: the
caravan's arm loads `ecx = node.x − goal.x` and `edx = −goal.y` and calls
with those. **The goal's own `y` coordinate stands where the `y`
difference belongs.** The estimate is then all but constant — it varies
only as `dx²/(2·goal.y)` — so two nodes in the same column score
identically whatever their `y`, and the search floods breadth-first along
that axis. It is why a trade route costs 12,965 nodes where a building's
road costs three hundred, and why one plan takes five frames.

The measurement that caught it is the search's **second pop**. The root's
eight neighbours are priced 86 west and 135 north-west; an honest `× 2.5`
puts north-west first and the original pops west. West and north-west
share nothing but their `x`.

**Two: `can_transport` comes from the caravan unit.** `find_road` sets
`pathfinder+0x98` from `UnitData::can_transport`, and East Indies' caravan
has the bit — so ocean tiles stop being refused and `calc_road_cost`'s
ocean arm, dead for every building road ever measured, prices them. The
node that said so is `(36384, 43488)` on frame 6167, at 231.

**Three: the search stops at the budget and carries on.** A caravan's
`traversed >= 0xc80` puts the popped node back on the open list and moves
the three containers into the `CaravanData` with the wheel preference and
the goal. `traversed` goes with them and is **never read back** — the
restore arm reads `+0x34`, `+0x38`, `+0x3c` and stops — so every resumed
frame starts its budget again at 3,200. 3,204 + 3,204 + 3,204 + 3,202 +
151, and the last one arrives.

**Four, and it is the capture-design lesson: a farm never terraforms.**
The `DUMP_ALL` block carries `master_land_heights`, and comparing the
whole grid rather than the tile the road argued about said that **182
tiles** of it were this crate's own. `docs/ROADS.md` §7.4 had the
terraform in `Wall::start`; `Wall::start`'s call there is
`Terrain::object_placed`, which is the *renderer's*. The terraform is
`Wall::init@0063e9b0:70`, under `param_6 == 0` **and `type != FARM`** —
an identity on the type, not a lineage. run62 could not have told the two
call sites apart, because a cheat-placed enhancer inits and starts on one
frame and is not a farm.

**The score.** East Indies **6166 → 6169**; Great Lakes unchanged at 2419.
The three frames are the whole of the budgeted search: 6166, 6167 and 6168
now agree draw for draw, and 6169's 3,202 road nodes do too — the frame
parts on two `Guy::inc_time` wraps that are a different mechanic, booked as
item 176.

**A fifth thing landed on the way there and moved no score.** Chasing
those two wraps found that every walking guy in this crate was frozen at
`cur_time == 1`: `set_anim`'s walk arm is `if (cur_time < len) goto <past
the write>` and this had the *other* arm's `cur − min(cur, len)`, which is
zero for a clock still running. `Guy::move` asks for the walk again every
frame, so no guy walking longer than its cycle could ever wrap. Read off
the listing, landed, and the whole suite is unchanged by it — which is
worth saying plainly: it is a correction with no measurement behind it
yet, and the two wraps it was chased for are still missing.

**The lesson, and it is the widening rule again one level up.** The item
was booked against one frame's draw count. What closed it was comparing
*whole records*: every node the original priced, all 57,600 tile masks,
the whole height grid, all 3,600 cell owners. Three of the four findings
were in fields the road never read — the farms' heights are nowhere near
the route's own tiles — and none of them could have been reached by
reading harder about the one number the item named.

## 2026-09-02 — item 176, closed: sixty pieces that play nothing (Opus)

The item was two draws on one frame. It is 11,872 of them on 5,936 frames,
and it was booked as one because nobody had counted.

**The first thing done was the count.** The opener said to fold
`Unit::inc_time+0x6e` — the crew loop, `[squad_size, guy_count)` — over the
whole of run54's trace before touching anything. Every one of the 11,872
crew-loop draws in a 24,000-frame game is `Guy::set_anim+0x97a <
Guy::inc_time+0x271`, they start on **6169**, and from there they come two
at a time on one frame in three until the capture ends. So the item was
never a frame; it was a metronome that had been silent since the map was
first traced, and East Indies' word had parted on its first tick.

**And what it is is a `<UNIT>` entry with nothing in it.**

```xml
<UNIT name="CARAVAN-DEFAULT-AGE0-CREW1" model=".\art\artillery_crew_driver.bh3"
      texture=".\art\caravan0.tga" cache="1" scale="1" .../>
```

Sixty of `unit_graphics.xml`'s 1,435 entries are self-closing, and every
one of them is a `-CREW{k}`. The piece loads — it names a model — and its
`AnimationPacket` is empty. `AnimationPacket::get_game_frames` answers 3
for a slot no packet names, which is the `end_time 3` beside the driver's
27 in run64's dump; and `Guy::inc_time`'s loop test is
`slot < packet->count && ids[slot] >= 0 && loopings[id]`, whose first two
conjuncts fail before the flag is reached. So the walk of a crew figure is
non-looping, wraps to `CHAR_DEFAULT`, rolls, and `Guy::move` puts it back
on the walk the next frame: three frames, one draw, forever.

`rondata::artdata::unit_anims` had dropped an entry with no `<ANIM>` row
since the day it was written — `if !rows.is_empty()` — so
`Art::piece_lengths` did not hold the piece, `get_unit_gpiece`'s existence
walk fell through all four loops to `first_unit_piece`, and every crew
figure in the game had been playing **the citizen's** art. Looping, so it
never wrapped, so it never drew. One line, and 1,359 pieces became 1,440.

**Then the widening, and it had teeth on its first run.** run64 is a
`DUMP_ALL` window and `Initial::frame_guys` has been parsed since the
animation clock was first read — and nothing has ever *compared* it.
`Built::tick` **installs** it, which is the opposite of a check. So every
capture that carried the field said nothing about it, and this is item 87's
ledger again: the field the parser had. Eight frames, three figures, nine
fields apiece — `cur_anim`, `cur_time`, `end_time`, `last_time`, `gpiece`,
`stopped`, the position and the angle — 2,061 fields, and eight of them
wrong on the first run. Both were real:

- **The walk fallback is the asked guy's own packet.** `set_anim:596` falls
  a slot the packet lacks back to `CHAR_WALK`, and this crate read *guy
  0's* piece for it — invisible while every crew figure shared guy 0's
  piece. run64's 6168 is `g0 anim 7` beside `g1 anim 8`: the driver slogs
  and its crew, having no `CHAR_SLOG`, walks.
- **A carried crew figure is never owed a turn.** `Guy::do_turn` writes
  guy 0's new angle, hands it to the crew as `des_angle` through
  `Guy::set_angle`, and then restores **guy 0's own** `des_angle` from the
  local it saved — so only guy 0 keeps a heading, and the untracked crew it
  recurses into ends every turn settled. On the frame a caravan starts
  moving the driver takes `Guy::move`'s turning arm and the crew takes the
  standing one and is left for the mirror: run64's 6167, `last_time −1,
  stopped 1`, which this crate had stepped.

**The score.** East Indies **6169 → 6189**, and the count reaches 6197;
Great Lakes unchanged at 2419. And the widening bought a tightening beside
it: run64's frame 6170 was compared by tile and direction only, because the
two missing draws put every price two jitters out of phase. It is compared
whole now.

**What it leaves is one thing wearing three faces**, and all three are item
177. `do_trade`'s move to the near city is queued here with a **waypoint**
at `(37752, 41592)`, so from 6167 the caravan sets off south-west where the
original heads straight at `(39288, 40056)`. That is the window's thirty-
four remaining fields, pinned; it is 6189, where this crate's caravan
reaches its waypoint and its crew pays arrival draws; and it is 6198, where
the original's reaches a *city* and spends three `Unit::do_trade+0x40`
draws over the three figures.

**The lesson is the widening rule with the ledger's name on it.** The item
named two draws on one frame. Folding the trace first said it was 11,872 on
5,936, which is what said the mechanism had to be structural rather than a
frame's accident. And the check that found the two residual bugs was not a
reading: it was nine fields of a record the parser had held all along and
nobody had put a `!=` against.

## 2026-09-02 — items 177(a) and 136, closed: the caravan walked wrong because the map has no danger on it (Opus)

The opener said to start at `add_move_order`'s `QUEUE_FIRST` arm and ask why
this crate gives `do_trade`'s move a waypoint at all. The answer is that it
does not: `add_move_order` writes `has_waypoint: false`, and the waypoint at
`(37752, 41592)` was `find_wpath`'s. So the question became why the world
grid planned a route at all where the original plans none.

**The trace already had the answer, and it took twenty minutes.** run64's
`callwin` proxies `PathFinder::calc_cost`, so the original's own price for
every one of the forty-seven steps of that search is on disk. Eleven of them
were wrong here, and the deltas were 8, 10 and 16 — not a multiple of the
formula's own 20 or 4, so not a terrain or an owner term. They grouped by
**half-cell**: `(50, 51)` and `(51, 51)` both off by 10, `(50, 52)` and
`(50, 53)` both by 16. A half-cell grid with a `/8` on it is
`WorldData::danger`, which `docs/QUEUE.md` item 136 had been sitting on for
a fortnight as "no writer, and four readers index it wrongly".

**Then grep the dump.** `WorldData::log_data@006b6080:628` prints the whole
map — `danger[who][scan]`, eight rows of `reg_size` — and every `DUMP_ALL`
capture in the corpus has carried it since the first one. run64's leader 1
holds −65, −135, −80 and −65 at exactly those four half-cells, and
`−65/8 = −8`, `−135/8 = −16`, `−80/8 = −10`. Every delta, exactly, before a
line of `calc_danger` had been read.

**The map is a balance of force, and the sign is the mechanic.**
`GameDaemon::calc_danger` runs on `frame % 200 == 0` and rebuilds from
scratch: military units add for the leaders they are at war with, and
**every finished building writes for every viewer including its owner** —
so your own ground goes negative. A city is 100, a fort or tower is half its
hit points, a military trainer 50, anything else 10, spread `value / 2` over
the eight neighbouring half-cells and `value` at its own. The two offset
tables are the executable's own `move_x + 4` and `move_y + 4`, read out of
the PE at `0xadcaf4` and `0xadc404`. `docs/DANGER.md` is the specification.

**Two halvings, and a third mechanic between them.** `do_danger`'s enemy arm
halves for unseen and again for peace, and on run64 the original halves
exactly once. Two readings fit. The dump settles it — `ever_seen 1` and
`ever_seen 2`, neither leader has seen the other; `diplos` 0 both ways since
the start block — so it is *unseen, at war*. This crate had it the other way
round, because **nothing installed `diplos`**: `LeaderDump` carried
`leader_flags` and not the diplomacy table, so `build_sim` left `at_war` all
false and every capture ran with two leaders at peace who have been at war
since frame 0. Forty-six readers of `is_enemy`/`at_war_with` were answering
the wrong way. It is installed now, and the full suite is unchanged by it.

**What the missing term did to the search.** With the danger in, the seventh
node the caravan's A\* pops is `(50, 52)`, whose Manhattan distance to the
goal is 768 — the arrival tolerance — so the search stops there. Its parent
is the start, and the reconstruction drops both the arrival node and the
start, so it pushes **nothing**: `find_wpath` answers a stack of one, and
`do_move` takes the move's own destination as its waypoint and walks
straight at the city. That is what the original's dumped path stack says it
does, one `PATHDATA` at `(38775, 40515)` — the destination `find_path`'s
pull-back walked back out of the city's own tile.

**The score.** East Indies **6189 → 6198**, and 6198 is
`Unit::do_trade+0x40`: three `Guy::set_anim` draws over the caravan's three
figures as it *arrives*, which is item 177's second half and the first
mechanic past the road. Great Lakes unchanged at 2419. run64's window is
whole — all 2,061 fields, where thirty-four were pinned this morning — and
two new diffs stand behind it: the danger map itself, 7,200 values, and the
forty-seven prices of the search it broke.

**The lesson is the diff-first rule, twice in one item.** The item was
booked as a walk and was a cost function; the cost function was named by a
*trace* the capture already held, and the mechanic behind it by a *dump
field* every capture had printed for a fortnight and nothing had ever put a
`!=` against. Neither needed a reading to find — only to write down. And the
third bug, the diplomacy table, was found by refusing to accept a green
test: the map matched with `is_seen` forced true, and it matched for the
wrong reason.

## 2026-09-02 — item 177, closed: a caravan walks its road, and the road was never laid (Opus)

**The word's frame was one instruction.** East Indies parted at 6198 on
three `Guy::set_anim+0x97a` draws under `Unit::do_trade+0x40`, and `+0x40`
is the return address of the *first* call the function makes:
`set_anim(CHAR_DEFAULT, 0, 1)`, above the caravan-slot test and above every
one of its returns. It draws nothing while the unit is walking — a
walk-category figure whose body has not caught up leaves `Guy::set_anim`
without a roll — so the site is silent for the thirty frames the caravan
spends reaching its city and spends three draws on the frame it stands
still.

That is not a decoration: it is what says `do_trade`'s later frames are its
**arrivals**. Each leg queues its move with `QUEUE_FIRST`, so the move is
the current order and `do_trade` does not run under it; the trace agrees,
with the site firing at 6198, 6511, 6766, 7021 and every 255 frames after
— one arrival per leg, three draws or one depending on what the three
figures were playing.

**And then the road turned out never to have been laid.** `do_trade`'s tail
copies `CaravanData::road` onto the unit and walks it, so the leg needed
the road as the original keeps it — a `Stack<PathData>` of **world**
positions with a tolerance and a flag byte. This crate had been keeping a
`Vec<Pos>` of *tiles*: `caravan_build_road` handed `set_road_at` the
search's own world coordinates without `.tile()`, thirty thousand tiles off
a two-hundred-tile map, where `tile_index` bounds-checked them into
silence. Every trade road in the port, from the day the search landed, was
a no-op.

**The `CARAVAN` record has printed the whole stack all along.** Item 87's
ledger again, and this is the fourth row of it to pay: `Caravans::log_data`
writes six fields and the road under `BEGIN STACK<TYPE>`, and run64's
`FRAME 6171` block — the end of the sim-frame `build_road` answered 1 on,
and the only block in the capture with a laid road — carries twenty-six
`PATHDATA`. `(35232, 36384)` to `(38496, 39648)`, `tolerance 96`
throughout, `flags 33` at the ends and `32` between. Writing
`build_road@0073db10:82` out of the decompile and asserting against them
was right on the first run, which is what `flags 32` on every node buys:
`0x20` means *a road was laid here*, and it is set in the same arm that
calls `World::set_road_at`.

That loop is a mechanic and not bookkeeping (`docs/CARAVAN.md` §5.3). It
empties the search's stack top-first — the near-*start* end first — into a
scratch stack and pops the scratch back, which restores the orientation;
open water lays nothing and is **sampled**, the first of a run kept at
`tolerance 0x180` and the next four dropped, with both ends of a crossing
exempt; everything else lays tarmac. The reconstruction above it drops any
tile carrying `mask & 0x4000` outright, which had not mattered while the
answer was only ever laid and matters now that it is walked.

**The legs** (§7). The caravan shuttles. `TradeOrder +0x20 loaded` is which
way it is pointing, and the arrival test is made against exactly one city —
the far one while empty, the home one while carrying — inside a box of
`± (max(x_size, y_size) · 0x60 + 0x306)` on each axis independently.
Loading at the far city pays `City::new_caravan`'s one-off; unloading at
home pays it again, sets `caravan_flags & 4`, and runs `City::compute_trade`
on both. That last bit is the gate on the whole economy of the thing:
`trade_val` — `CityData +0x52`, and the *first* line of
`calc_city_resources`, ahead of everything a city gathers — sums
`trade_value(other) · 16 / 2` over the routes that carry `& 4`, so a trade
route is worth **nothing** until its caravan has come home once.

**Three things in the walk were settled off the listing.** The two
`vector_dist` calls at `5ee08e` and `5ee0cc` measure the unit against the
stack's bottom and its top and invert when the bottom is nearer, so the
invariant is *the top is the end I am standing on*; the pop throws that end
away and the move order's destination is the far end. The move goes in with
`add_move_order`'s fourth argument set, which is the order flag `1` —
"the path stack is already mine" — so `do_move` walks it instead of
planning; §4.1's arm passes `0` there, and since those same two arguments
are what `add_move_order` hands `find_angle`, each caller's arrival facing
is a literal: `find_angle(1, 0)` for the failure arm and `find_angle(1, 1)`
for the leg. And the smoothing is **perpendicular** — `to_x += dy/3`,
`to_y −= dx/3` — so the caravan walks a third of a tile beside its road
rather than down the middle of it. Ghidra prints the second half as an
unfolded multiply; `5ee18f` is the magic `0x55555555` with a `sub`/`sar`,
which is `x / −3`.

The first version of that loop compounded: it measured each step against
the *displaced* previous node, where `5ee14e` saves `to_x` and `to_y`
before either is written. The probe caught it in one line — the first
waypoint read `(38553, 39477)` where the hand calculation says
`(38560, 39456)`.

**The probes.** Neither of these existed this morning and neither residue
was findable without them. `RON_DEBUG_SITES=<lo>-<hi>` widens the word
test's print from the two parting frames to a window;
`RON_DEBUG_UNIT=<who>/<o>@<lo>-<hi>` prints one unit's position, angle,
path stack and figure clocks, frame by frame. The bug that is left is a
*cadence* — a crew figure wrapping every second frame instead of every
third — which is invisible at the frame it finally parts on and obvious
over a dozen either side.

**The score.** East Indies **6198 → 6207**; Great Lakes unchanged at 2419.
190 rondata tests and 755 sim tests green, and one new diff: the route's
stack, whole. What is left at 6207 is two frames of the caravan's turn out
of its own city — the original stands there from 6197 and starts walking on
6208 where this crate starts on 6206 — and the route past it is right, so
this is a residue and not a mechanic. **Nothing on disk covers frames
6198 to 6210**, which is the first time in a while that the answer is a
capture rather than a widening; item 179 books it.

## 2026-09-02 — item 179: the caravan was never the bug, the step was

Yesterday's entry ended by booking a capture, and the capture was the
right call for the wrong reason. Item 179 said the two frames were "the
turn, or the detour `do_move` plans around the footprint". They were
neither, and the window said so before it had been read twice.

**run65** is run54's game with a `DUMP_ALL` window on `[6196, 6214)` —
eighteen `FRAME` blocks — at `MISC` alone otherwise. 1.19 GB, thirty-seven
minutes, and `rngcmp.py rontrace-run54.log rontrace-run65.log` answers
**6,221 frames, zero differing**: the fifth capture in a row for which a
window, a coverage window and eight call proxies together cost the stream
nothing. The queue asked for `[6196, 6212)`; the window went two frames
wider because a `FRAME n` block is the end of sim-frame `n − 1` and the
two sides' *first walking frames* were the whole question — a question
you cannot answer from a window that stops on the later of them.

**What the window shows.** Both sides push the same detour node,
`(38508, 40620)`, on the same frame. Both turn through the same eight
bearings — 6°, 8°, 8°, 12°, 12°, 24°, 24°, 24° — because `avg_speed`
decays a quarter a frame on both and the turn rate is the base over
`avg / 4 + 1`. On sim-frame 6206 both are left owing 38.9°, inside
`move_step`'s 45° gate, and both compute the same full step to the same
point: `(38750, 40508)`. The original does not take it.

`Unit::move_step@005faf30` at `005fb7c1` compares the step's tile against
the tile the unit is standing on and, where they differ, asks
`UnitData::invalid_loc` with all five flags clear. `(38750, 40508)` is one
tile north of the caravan and inside its own city's footprint, so the
answer is a refusal and the step is dropped **whole**: no `set_anim`, so
the walk is not even requested; no `set_new_location`, so no move and no
reveal; the waypoint kept, and `move_step` returning 0 where every other
refusal returns 1. The unit turns 24° more and walks on 6207 through a
tile it may have. It is the last of the four things `docs/MOVEMENT.md`'s
`move_step` section listed as read and not modelled, and the fix is one
`if` in `Sim::unit_step`.

**Two hours went to a probe that read the wrong field.** A `UNITDATA`
block prints `angle` three times — the unit's, the `MOVEORDER`'s, and
every `GUY`'s — and a flat key/value sweep keeps the last one. The first
reading therefore had the original snapping its facing in one frame and
then standing eight frames for no reason at all, which is exactly the kind
of finding that gets written up. Nesting in these dumps is **indentation**;
the Rust parser has always known it and the scratch probe did not. The
second reading, with the indentation respected, showed the two sides'
bearings agreeing frame for frame, which is what made the step the only
place left to look.

**What is now an assertion.** `run65_s_window_is_the_original_s_unit_for_
unit` compares every dumped unit of every one of run65's twenty blocks —
position, `UnitData::angle`, `orders_x/y`, `tolerance`, the path stack's
length and every slot's point, tolerance and flag byte. **5,186 fields,
all the original's.** That is item 87's ledger paid again: the path stack
has been parsed for months and only the caravan's own top was ever
compared.

**By-catch, unfixed.** `do_trade@005ed270` calls `WorldData::get_tregion`
at all four of its region sites; `crates/sim/src/caravan.rs` asks the
plain `World::tregion` at both of its. It changes nothing on this cell —
`(50, 52)` has `flags 0x80`, so the coastal refinement never fires — and
it is two more of item 142's eleven unaudited callers. It was checked
because it was the *first* hypothesis for the missing `TURN_FIRST` bit,
and the run64 dump's own `WORLD` block killed that hypothesis in a
minute: grep the disk before believing a mechanism.

**The score.** East Indies **6207 → 6353**; Great Lakes unchanged at 2419.
191 rondata tests and 758 sim tests green. 6353 is two
`Guy::init_real+0x52` at the head of the original's frame — a unit it
trains and this crate does not — and nothing else in that frame parts, so
the next item is production and item 180 books it.

## 2026-09-02 — item 180: the AI had never trained a merchant, on any map

**The frame said "production" and the record said which one.** East Indies'
word had been parked on 6353, whose whole content was two
`Guy::init_real+0x52` under `Objects::init_unit` that this crate did not
spend. Two `Guy::init_real` calls is a **two-figure unit** — `Unit::init`
sizes the guy stack to `crew_size + squad_size`, and `unitrules.xml` gives
`CREW_SIZE 1` to twenty-nine types — and the caravan born on 6164 with
three figures was the Market's, so the Market was where to look.

**run65 already had it.** The capture taken for the caravan's turn carries
a `DUMP_ALL` window on `[6196, 6214)`, and in every one of its eighteen
blocks the AI Market `1/2013` (`orig_type 436`) reads `queued 1` with
`queue[scan].type 61` — `MERCHANT` — and a `job_counter` climbing exactly
a hundred a frame: 3100 at the block labelled 6196, 4800 at 6213. Walk it
back and the entry starts ticking on **6165**, the frame after the caravan
was handed over, and forward and it lands on **18,720** at 6353. `JOB_TIME`
156 × 100 × `UNIT_RATE_BASE` 120 / 100 is 18,720 on the nose, with no ramp
and no tail — the first Merchant, so `owned` is zero — and
`Sim::queue_target` already computed it. **The grep-the-disk rule again:**
the answer to "what unit" was a `BUILDDATA` block already on the disk, not
a capture and not a reading.

**The cause was one host function returning zero.** `game/ai/scripts/
economic.bhs` builds merchants inside a loop bounded by
`num_rare_resources_seen(who)`, and `crates/sim/src/ai_host.rs` answered a
flat `0` with the comment "Rares are not in the simulation: none seen" —
true when it was written and false since `rares.rs` landed. So no AI on any
map had ever trained a Merchant.

**What the function actually is.**
`ScenarioFuncSet::num_rare_resources_seen@009ea010` is the length of
`LeaderData +0x6e6c`, `SimpleArray<int> new_rares` — the goods-list indices
of the rares this leader has seen. `Leader::new_rare@006d9e70` is its
writer and `World::reveal_fog@006b3d30` its caller, on exactly the fog
cells `World::set_seen` answered *changed* for; the gate is the tile mask
`0x200` at `(2fx + 1, 2fy + 1)` and then `find_good_at` on the cell. The
recording skips a plain human on both sides of the call
(`leader_flags & 0xc == 4` for the caller, `& 4` for the recipient), spreads
to mutual allies, refuses a duplicate and a type the recipient cannot yet
build with — and **refuses `FISH` and `WHALES`**, which is the whole design:
those two pay a *fishing boat*, so a coastline buys no merchants.

Those last two tests are `SubObject::is`, and the decompiler prints them as
`(*(code *)ppuVar1[0x2e])(6, 0)` and `(0x1f, 0)`. `0x2e` is a pointer index,
so the byte offset is `0xb8`, and `vtables.txt` says `Good::vftable +0xb8` is
`SubObject::is`. That is the export's second half earning its place: the
decompiler will not name a slot, and `vtables.txt` will.

**The widening is what makes it an assertion.** run65's test compared units
only. It now compares the **build queues** as well — `queued`, and every
live slot's type and `job_counter` — so the Market's single merchant and
its 18,720 are checked on all eighteen blocks rather than inferred from a
frame number that happened to land. The record's own trap is in the slot
behind: `queued` is 1 and slot 1 also reads `type 61`, because `unqueue`
shifts the array down over the caravan and leaves the vacated tail
standing. Reading the tail would have said "two merchants" and sent the
session looking for a second one that is not there.

**The count is right, not merely non-zero.** East Indies' AI has seen
`CITRUS` (good 20) and `HORSES` (good 23) by frame 5000, so
`num_rare_resources_seen` answers 2; the script's loop tries twice and the
second `train_unit_with_cost` refuses, which is why one Merchant is queued
and not two. Had the count been wrong the window would have shown `queued`
2 against 1 on all eighteen blocks.

**What is left standing.** The start fog is **not replayed**: a rare under
a leader's fog at frame 0 was recorded by the original's `Setup` and is not
recorded here, because this simulation begins from an installed grid rather
than from the reveals that built it. On East Indies it costs nothing — the
one good under the AI's start fog is an oil patch, which `Objects::init_good`
never puts in a cell's chain — but a map where it is not oil would part.
`World::compute_reg_territory`'s call, the second way a rare is learned, is
not carried either.

**The score.** East Indies **6353 → 6356**; Great Lakes unchanged at 2419.
191 rondata tests and 761 sim tests green. Three frames is a thin move and
the handoff says so — but the zero it replaced was every AI on every map,
and 6356 is the same unit's *next* frame: `Guy::set_anim < Guy::do_turn <
Unit::move_step`, the Merchant's first step, three frames after its birth.
`Unit::think_merchant@005f4740` is what sends it, and item 182 books it.

## 2026-09-02 — item 182, closed: the merchant walks, and its crew was never seated (Opus)

**East Indies 6356 → 6570.** `docs/MERCHANT.md` is the new document, and
the item took two things rather than one.

**The first was the order.** `Unit::think_merchant@005f4740` is the whole
of an idle merchant's decision, and it has no draws in it at all — which is
why the trace could say *when* it ran but never *what it chose*. Its head
is `unit_masks & 0x80000`, packed, and the arm behind a clear bit is a
`return 1`: a merchant that has already deployed onto a rare never searches
again and never falls through to `Unit::think`'s tail either. Behind that,
`Unit::unpack_merchant@006038e0` asks whether here will do — and its own
head is `UnitData::calc_gather@00609180` **where the unit already stands**,
so a merchant crossing the map pays one gather search a think and walks no
candidates. Only then the score, over `LeaderData::new_rares`:

    base = 200 − 10·i, decremented at the loop tail whatever the slot did
    + 100 where the good's cell region equals my get_tregion
    − danger[who][cell >> 1], floored at 1

with three object searches that refuse a good outright — a sibling of my
**exact** type within `0x300` of the good's *cell centre*, a sibling of my
exact type ordered there (`find_unit_ordered`'s move-family test, which is
literal), an enemy that can shoot within `0xc00` — and the winner taken
**out of the list and appended**. That rotation is the whole of the "two
merchants do not go to the same rare" design, and it is cheaper than the
searches beside it: `num_rare_resources_seen` reads the same length
afterwards, and the next merchant to think scores the taken good last.

That landed, and the word did not move. It stayed on 6356.

**The second was the crew.** With the order in, the merchant turned on the
right frame — and spent **two** `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
Unit::move_step+0x389` draws a frame where the original spends one, for the
three frames of its first turn. `Guy::do_turn@005d97a0` recurses into the
crew figures whose `track_dx`/`track_dy` are **zero** and no others; a
merchant packs, so every one of its figures carries `guy_flags & 8`
(`docs/ANIM.md` §4.8) and asks for a turn animation the merchant's art has
not got, falls to the idle and rolls. The Merchant's `<UNIT>` entries all
carry `trackdist="10"`, so the original's crew figure has an offset and is
never recursed into.

This crate's had none — not because the offset was missing but because
nothing had ever *seated* it. `Unit::init@00612100:548`–`549` is
`update_gpiece` and then `set_new_location(x, y, 1, 1)`, and that
`param_3 = 1` reaches `Guy::set_new_location@005d86f0` on every crew figure
with its own `des`: the figure is **placed** on its track offset at birth.
`Sim::seat_guys` had two callers — the dump loader and the transport's
disembark — and every unit the simulation *trained* kept a trackless crew.
It had never shown because the merchant is the first trained unit with a
tracked crew figure that has ever turned in a capture. `Sim::init_guys` now
ends where `Unit::init` does, and the 191 rondata tests — the dumped-guy
piece walk and run53's Great Lakes ceiling included — are unchanged by it.

**What the run backs, and what it does not.** run54 now agrees draw for
draw through the merchant's think on 6354, its turn on 6356 and 214 frames
of its walk. It cannot check the arithmetic: East Indies' AI has two goods
in `new_rares`, one of them in its own region, so the pick is `300 > 190`
and each of the three terms could be wrong on its own. The `−10` step needs
a third good, the danger term needs a war, the floor needs both, and the
rotation needs the **second** merchant — which the original trains on 6571,
one frame past the new word. Twelve tests in `crates/sim/src/merchant.rs`
carry what the capture cannot, including an eleven-deep list where the step
and the bonus finally separate.

**And `docs/ORDERS.md` §6 step 5 had a gate backwards.** The
`do_gather` + `unpack_merchant(4)` arm was written as "(AI only)"; its gate
is `leader_flags & 4`, which is **human** — the same bit
`Leader::new_rare@006d9e70` tests to refuse a plain human's reveals. The
trace agrees with the flag rather than the prose: `Unit::unpack_merchant`
is first entered on frame 6354, the AI merchant's first idle frame, and
never in the four thousand frames the AI's fishing boats spend idle before
it. Amended in place.

**The score.** East Indies **6356 → 6570**, Great Lakes unchanged at 2419;
191 rondata tests and 772 sim tests green. 6570 is a **collision**: the
blocked stand `Unit::move_step+0x823`, twice, on the merchant two hundred
frames into its walk, with the AI's own citizen `1/2` standing still on a
gather order 180 units away and this crate's exemption ladder refusing to
let the two past. The original enters `PathFinder::find_wpath` for the
first time on 6602, so its merchant meets something of its own thirty
frames later; whether it is that citizen is what the next item asks.

---

## 2026-09-02 — the merchant walks past, and the probe was never the disc (item 183, Opus)

**East Indies 6570 → 6571**, Great Lakes unchanged at 2419. A one-frame
move for a finding that is larger than the number: the merchant's *whole*
walk — birth, turn, 214 frames, the collision that stops it, the centre
snap and the recovery — is now the original's position for position, and
the probe every unit in the game steps through was being asked the wrong
question.

**The word was a collision the original does not have.** On sim-frame 6570
the AI Merchant `1/19`, two hundred frames into its walk to a `CITRUS`,
stood blocked twice where the original walked on. It is the first
`BLOCK_RADIUS 2` unit any capture has ever collided, and the geometry was a
knife edge: the merchant passes the two standing citizens with 160 units of
clearance against a 144-unit block sum, so cell quantisation decides it and
a few units either way flips the answer. Reading was never going to settle
that, and nothing on disk covered the frame — run64's and run65's windows
both end at 6221.

**run66 is the capture, and its shape is the lesson.** 260 blocks of
run54's game over `[6340, 6600)` at run39's `[End Frame]` detail: **four
minutes and 89 MB**, against run65's eighteen `DUMP_ALL` frames for
thirty-seven minutes and 1.19 GB. `rngcmp.py` says 6,621 frames, zero
differing — the sixth in a row for which a window costs the stream nothing.
The rule that falls out is run60's one level up: **narrow the window when
the question is a whole record, cheapen the block when the question is a
field over time** — and a walk is a field over time. Positions, angles,
path stacks, order stacks and *both guys* of every unit are all in the
cheap block.

The capture killed three hypotheses in one reading. The citizens are where
this crate has them, on every frame. The merchant is where this crate has
it, on every frame — 197 of them, exact — so it was never a step gained or
a bearing lost. And the collision *does* happen: one frame later, at 6571,
against a **different citizen**, `collide_o 11` where this crate had named
`1/2`.

**That field is what named the mechanism.** `CollCheck::collide_here` has a
second sweep ahead of the disc, and `docs/COLLISION.md` §4.2 had it in one
sentence as a fast path. It is not an optimisation. With `nocoll` clear and
a proposal exactly one cell away on one axis it tests the **leading edge** —
the row or column the block is entering — and nothing else: a strict subset
of the parity-filtered disc, so it stops at a *different* first hit cell.
And §4.3's corner rule is decided **on the cell**, so the two probes
disagree about whether there is a collision at all, not merely about which
unit is named.

Sim-frame 6570: the merchant's cell is `(721, 786)`, its proposal
`(721, 785)`, so the sweep is the row `y = 783` from `x = 719`. The first
cell is `1/11`'s north-east corner, and the merchant's own north-west
corner meets it — `will_be_corner 1` against `is_corner 5`, a difference of
exactly 4 — so the two slip past and the step is taken. The whole disc's
first parity cell is `(721, 783)`, two to the right, inside `1/2`'s block
and no corner of the merchant's at all: a hard collision the original never
had. Sim-frame 6571: the proposal is one cell **west**, the sweep is the
column `x = 718`, `1/11` sits square on it rather than cornered, and the
original collides. The dump's `collide 1`, `collide_o 11`, `collide_who 1`,
`collide_guy 0`, `collide_frame 6571` all follow.

**`nocoll` is the argument that selects it**, and it is not uniform:
0 at every `detect_unit_collision` call site and at
`Objects::find_collision`'s, **1** at `PathFinder::valid_ucoord`'s and at
`resolve_unit_collision`'s own direct probe. So a *step* takes the edge and
a *path search* takes the disc, and threading that flag through this
crate's four call sites was the whole change.

**What is pinned.** `run66_s_window_is_the_original_s_unit_for_unit` —
12,094 position fields, zero disagreements: every unit on every block as
far as the word, and `1/19` alone for all 261 blocks, so its walk, its
collision, its snap and its recovery are asserted *past* the frame the
stream parts on. `docs/COLLISION.md` §9's `coll_size ≥ 2` row is struck.

**Two things the capture leaves.** The first is the new word: on 6571
`move_step`'s `set_anim(CHAR_DEFAULT, 0, 1)` rolls **once** in the original
and twice here, and the original's crew figure then *walks* — `1/19`'s `g1`
goes `(34442, 37613)` → `(34464, 37595)` on the frame the resolve snaps its
leader — where this crate's stands still and moves a frame late. Both
halves are one question about `Guy::set_anim`'s walking-guy early return and
who rewrites `Follow::des`, and run66 has both guys' positions on all 261
blocks to check it against. The second is the **second** Merchant, born
6571, which leaves its Market a frame early here (blocks 6578 onward).

**And a capture-design note worth the ink.** `SETTLE_MIN` was copied from
run59's 250 MB without thinking; run66's whole log is 89 MB, so the poll
could never call it settled and would have sat out its full 200 polls — 66
minutes — after a four-minute capture. It is a ceiling as well as a floor:
above the start dump, below the finished log.


## 2026-09-02 — the crew figure was never told where to stand (item 185)

**East Indies 6571 → 6574.** Three frames, and the item behind them turned
out to be one loop of eight lines that this crate had never run.

**The question run66 left.** On sim-frame 6571 the AI Merchant `1/19`
collides, and `Unit::move_step`'s blocked stand asks both of its figures to
idle (`set_anim(CHAR_DEFAULT, 0, 1)` at `+0x823`, before the three give-up
tests). The original spends **one** draw; this crate spent two. And the
original's crew figure then *walks* — `(34442, 37613)` → `(34464, 37595)`,
the leader's own displacement — where this crate's stood still.

**The reading ran out, and said so.** `Unit::set_anim` loops guy 0 and every
guy past `squad_size`, so both figures are asked on both sides.
`Guy::set_anim`'s early return for a walking guy is `des != pos`, and the
figure's position at the end of 6570 is `(34442, 37613)` on both sides —
which is exactly what `follower_des` gives for the leader's point and
facing, so it was standing on its destination and had no reason to return.
Every hypothesis that survived an hour — a suspended `openlist`, a
different blocked unit, a category that was not the walk — was refuted by
something already on disk. What the dump did not carry was the figure's own
`des`: `GUYS=2`, which every capture since run10 has used, prints nine
lines and none of them is it.

**The capture, and the shape it added.** `GuyData::log_data@005de6c0`
switches the log's detail four times, and the fourth block is the whole
record — `des_x`, `des_y`, `des_angle`, the clock, `stopped`, `guy_num`,
`gpiece`, `track_dx`, `track_dy`. So the question needed **`GUYS=4`**, not
`DUMP_ALL`: run67 is run54's game with the cheap per-frame window narrowed
to `[6545, 6605)` and that one category raised, and it cost **four minutes
and 43 MB** for sixty blocks — against run65's thirty-seven minutes and
1.19 GB for eighteen. `rngcmp.py` against run54: 6,621 frames, zero
differing, the seventh in a row.

That is a third arm on run60's rule, and it is the cheapest of the three:
narrow the *window* when the question is a whole record; cheapen the
*block* when it is a field over time; **raise one category's detail when it
is one record's own fields.** `grep -n '0x28))(' ` over a record's
`log_data` is how to find out whether the arm is available.

**What it settled: one loop, three findings.** `Guy::set_angle@005d9010`
and `Guy::set_new_location@005d86f0` end in the same crew loop, and its
`track != 0` test gates only the rotation — `des_angle` and the base `des`
are written for every figure past `squad_size`.

- **`Unit::set_angle` writes it, from the heading.** `move_step` opens with
  `set_angle(this, find_angle(…), …, 0)`, whose tail is
  `Guy::set_angle(guy 0, heading, 0)` — so on every frame the bearing moves
  at all, a figure that walked exactly onto its destination last frame is
  off it again *before* the collision block. One frame's turn here is
  720,896, which moves a `(-48, -192)` track one unit on each axis, and the
  early return then takes it. `docs/MOVEMENT.md` had this as the third row
  of its writer table with "Not modelled" beside it since the table was
  written; it is not a residue, it is the row a collision reads.
- **The cell-centre snap teleports the crew.**
  `Unit::set_new_location(·, ·, 1, 0)` — `resolve_unit_collision`'s — hands
  its `param_3` on as `Guy::set_new_location(guy 0, pos, 1)`, and the crew
  loop finishes each figure with `set_angle(crew, des_angle, 1)` and
  `set_new_location(crew, des, 1)`. The figure is *put* on its rotated
  offset with the leader's angle. Block 6572 is the record, to the digit.
- **The walk slot is the asked guy's own average speed.** `set_anim`'s walk
  arm divides `this->field_0x84` — not guy 0's — by `moves ·
  UNIT_MOVE_SPEED`, and `Guy::move`'s tracked branch pays a figure
  `(get_speed · 11) / 8` a frame to keep station. Eleven eighths is above
  the eleven tenths that jogs, so a tracked crew figure plays `CHAR_JOG`
  beside a walking leader: `cur_anim 9` against 8 on all sixty blocks. Not
  cosmetic — `Guy::move`'s arrival arm tests the **slot**, so reading guy
  0's cost a draw on every arrival a crew figure made.

**What is pinned.** `run67_s_window_is_every_figure_s_whole_record`:
**13,545 fields** over 61 blocks and 169 crew rows, zero disagreements —
position, angle, destination, destination angle, slot, clock, end time,
last time, piece, `stopped`, `guy_num` and both track components, for every
figure of every unit as far as the word and for `1/19` throughout. The
parser gained `des`, `des_angle` and `track` to carry it. Two fallbacks in
the harness are findings of their own: guy 0's `des_angle` is the
**heading** and a *trackless* crew figure's is guy 0's **facing**, because
`Unit::set_angle`'s write is overwritten by both of the other two.

**And a tool the dump had been hiding half of.** `tools/gamelog/track.py`
kept only the first of a repeated key, so a two-figure unit's crew was
invisible to it — `guy.x` was always guy 0's. Repeats now get `key#1`,
`key#2`, and the crew figure is readable at all.

**And the widening found a fourth finding on its first run**, which is the
rule working. Raising the floor to 6574 brought three more frames of every
unit into the comparison, and the *second* Merchant's crew figure was 696
units from where the original has it — a distance no rotation of a
`(-48, -192)` track can produce. `come_out@00617c10` places its squad with
`set_new_location(·, ·, 1, 1)`; this crate wrote `u.pos` and a fresh
`Movement` by hand and left `guys[].follow` alone, so a **trained** figure
kept the seat `Unit::init` gave it at the trainer's own centre and stood
there for life. Every unit a dump handed us was seated correctly, and every
unit trained mid-game was not — which is why nothing before a capture that
printed `des_x` could see it.

**What 6574 leaves.** Two draws in one frame, and one of them is already
booked. The extra `Guy::do_turn+0x4a < Unit::move_step+0x389` is item 186:
`1/20` stands idle through 6574 in the original — `cur_anim 0`,
`cur_time 3/60`, `stopped 1` — and takes its first step on 6575, where this
crate slogs. run67 excepts that one unit by name, so the exception is the
item. The other half is item 187: five gaia bird wing-beat coins against
the original's one, and two against three on 6575.

## 2026-09-02 — the second merchant was walking at the first one's rare (item 186, Opus)

**East Indies 6574 → 6715.** Item 186 was booked as a frame — "the second
Merchant leaves its Market a frame early" — and the frame was a symptom of
a destination.

**What the item said.** run67's window had `1/20` excepted by name: born on
6571, the original has it standing through block 6574 (`cur_anim 0`,
`cur_time 3/60`, `stopped 1`) and stepping on 6575, where this crate
slogged a frame sooner. That is all a `GUYS=4` block can show, because the
block prints a figure's clock and not its unit's order.

**What the dump said the moment it was asked the other question.** `1/20`'s
`UNITDATA` at block 6573 carries `orders_x/y 28728/24120`; this crate's
carries `31800/37176`, which is `1/19`'s — the *first* merchant's rare,
already taken and already being walked at. The block after that is the
whole of the frame: the original's `STACK<PathData>` is **22 entries**
long, this crate's seven, and `Unit::do_move`'s `if (path.length > 10)
return 1` is what holds the original's first step back to 6575. The
one-frame stand was never the mechanic. It was the tail of a 22-waypoint
path this crate never planned because it was walking somewhere nearer.

**The cause, and it is one operand.** `Unit::think_merchant` refuses a good
on three object searches, and the second is
`ObjectsData::find_unit_ordered@0065bc40` — "is one of my own kind already
on its way here". This crate asked it of the sibling's **body**. The
original asks it of `UnitData +0x70/+0x74`, `orders_x`/`orders_y`: where
the sibling is *going*. `1/19` had walked 2,800 units clear of its rare's
cell centre by 6572, so the body test passed and the good was free; its
`orders_x/y` had named that cell since 6355.

The decompiler prints the distance in both functions as
`vector_dist(unaff_EDI, unaff_ESI)` and names neither operand, so this is a
listing reading — but a cheap and safe one, because the two functions sit a
page apart and are otherwise the same twelve arguments and the same sweep:
`find_unit@0065ca80` at `0065cd0f` un-XORs `SubObject +0x10/+0x14`;
`find_unit_ordered` at `0065be35` loads `UnitData +0x70/+0x74`. One operand,
swapped. `docs/MERCHANT.md` §2.2.1 carries the pair.

That also settles what §2.3's rotation is *for*. The list rotation is the
cheap half of "two merchants do not go to the same rare"; the ordered search
is the half with teeth, and it works from anywhere on the map.

**What it moved.** The word went 6574 → **6715**, and the widening it paid
for is larger than the item. run67's window no longer excepts anybody: every
unit, every block, every `GuyData` field the dump prints — **28,890 fields**
against 14,910, zero differing. run66's stops excepting every non-merchant
past 6574 and now compares its whole window too.

**And it left one.** With the word past the whole of run66's capture, the
quit block came into the comparison for the first time and the human citizen
`0/5` is one step ahead in it: both sides re-think on sim-frame 6606 — the
trace's four `GameAccess::rnd+0x20 < Unit::do_job+0x67` draws — both walk the
same `(-18, +18)` step, and by the closing block this crate has taken
fourteen and the original thirteen. Twenty-seven units were checked at 6620,
6621 and 6622 to be sure the closing block is an ordinary `n`-tick block and
not an off-by-one: at 6621 twenty-six of them agree and only `0/5` does not.
No dump on disk covers `[6600, 6620]`, so it is item 188 and it is a
capture, not a reading.

## 2026-09-03 — the ledger, and a capture that answered three questions (items 87, 188, Opus)

Two things after item 186 closed, in the order they happened, because the
second is what the first predicted.

**The widening ledger** (item 87, `crates/rondata/src/ledger.rs`,
`docs/DATALAYER.md` §4). Item 186 was the twelfth queue item closed by a
field the parser had been filling and nothing had compared, so the count is
now a guard rather than a hope. It reads `gamelog.rs` and `diff.rs` at
compile time and prints two lists: **uncompared** — the field's identifier
appears nowhere in the differ — and **single-capture**, exactly one test
function names it. Both are pinned and may only fall.

The second list is the sharper one and it is item 87's "per capture" half.
On the day it was written it named `stance`, `idle`, `path_recursion`,
`safe`, `dest_angle` and the whole `UnitData` collision block as fields one
window was carrying alone.

**Then run68 landed and `stance` was wrong on every unit of every frame.**
2,700 rows, from the first block of the window. `Unit::init` switches five
ways on `get_stance_type` and reads the leader's own options
(`00612100:282–309`); `Unit::new` writes a flat 1, and a unit stood up
*from* a dump takes the dump's own value — which is exactly why every
earlier capture agreed and no reading had ever been asked. It is item 190.
The ledger predicted the shape of the finding a day before the capture
found it, which is the whole argument for counting.

---

**run68 itself** (`docs/ORACLE.md`, 2026-09-03): run54's game, the cheap
window over `[6595, 6730)`, `GUYS=4`. Five minutes, 83 MB, 136 blocks,
`rngcmp` 6,746 frames zero differing — the eighth window in a row that costs
the stream nothing.

**One capture for two items.** 188 wanted `[6600, 6620]` and 189 wanted
`[6710, 6720]`. A window is priced by its blocks and the frames before it
are free, so the span between two items a hundred frames apart is nearly
free to buy. Booking them separately would have cost two launches.

**Item 188 was never a divergence, and the quit block is why.** The human
citizen `0/5` walks identically on both sides — order on 6607, first step on
6608, `(4860, 5172)` on 6621. What said otherwise was run66's *closing*
block. Dump against dump, with no simulation in the loop: run66's and
run67's closing blocks each agree with run68's **ordinary** 6621 for 130 of
131 units, and hold the 6620 value for `0/5` alone — the same single unit,
in two independent captures, while five other units sit mid-step at 6621.
So a closing block is block `n` for almost everything and one tick behind
for at least one unit. It is not a frame state, the harness no longer scores
one, and `docs/ORACLE.md` carries the table. The cost of not knowing was a
day of item 188 and an exception written on a number the block had no
business supplying.

**Item 189 is the merchant's arrival.** The first field of the whole record
to part is `1/19`'s `orders_x/y` on **6714** — a frame *ahead* of the draw
stream's own 6715, which is the argument for diffing fields as well as
draws. `1/19` reaches its `CITRUS` and runs `find_merchant_spot`'s ring;
ours answers `(32076, 37188)` and the original `(32280, 36888)`.
`docs/MERCHANT.md` §6 had called that ring unreachable — "a merchant that
reaches its good is a merchant that has walked further than any capture
follows one" — and this capture follows one.

**And the whole-record rule paid on its first run again.** Comparing the
path *stack* and not only its length found `1/13` holding `path[2].y`
38712 against 38760 and `path[3].x` 40584 against 40536 from block 6686 —
two middle waypoints, one 48-grid step each, on a route whose length, ends
and flags all agree, and which the unit walks without parting for
thirty-two more frames. Item 191.

**118,948 fields over 119 blocks, zero differing**, with `stance` and that
one stack excepted by name.

---

## 2026-09-03 — the ring was right and the queue position was not (item 189, Opus)

**East Indies' long word 6715 → 6739.** Great Lakes unchanged at 2419.

run68's block 6714 was the first field of the whole record to part, a frame
ahead of the draw stream, and it was booked as `find_merchant_spot`'s ring
answering a different tile. It was not. The ring answers **(168, 192)** on
both sides — `MOVE_49`'s fourth entry, with entries 0, 1 and 2 refused —
and that is the first time any capture has entered the function at all.
What parted was the line after the ring.

**`Unit::unpack_merchant@006038e0`'s tail rotates.** It builds the
`MOVE_TO` inline, puts it on the list with `LinkListBase::add`, calls
`clear_partial_path`, and then runs **`head = head->next`** before
`update_action`. That statement is not decoration: it is character for
character what `Unit::add_cast_order`'s own `QUEUE_FIRST` arm runs, and
`docs/ORDERS.md` §1.5 has named its effect since the order document was
written — the new order becomes the current one. So the merchant's two
orders are `[MOVE_TO, CAST]`: it walks to the deploy spot and casts on
arrival. This crate appended the walk behind the cast, `update_action`
stopped on the cast (which is neither a plain move nor a `CHANGE_FORM`),
and `orders_x/y` stayed at the unit's own position.

**The dump prints an order list backwards, and three fields of one block
say so.** Block 6714's `STACK<TYPE>` reads `CASTORDER` then `MOVEORDER`,
while `orders_x/y` is 32280/36888, `dest_angle` is the `MOVEORDER`'s own
346619904, and the unit steps at that point four frames later. Both walks
are in the export: `OrderList::log_data@00730070` sets `node = head->prev`
and advances by `next`, so it prints `head` first;
`Unit::update_action@0060a870` sets the same node and advances by `prev`.
`UnitDump::orders_front_first` in `rondata::gamelog` has reversed the
printed list since it was written, so nothing in the harness had to change
— but until this block nothing had ever *pinned* which end was which.

**Two points, not one.** The destination is `t · 0xc0` snapped to the
48-grid — `(168, 192)` gives `(32280, 36888)` — and the angle is
`find_angle` of the delta to the **unsnapped** `t · 0xc0`, which is
346619904 where the snapped point would give 408616960. That split is
already `Unit::add_move_order`'s (`docs/ORDERS.md` §4.3), so the whole
correction is one argument: `QueuePos::Last` → `QueuePos::First`.

**What the capture cost, and what it bought.** Nothing — the run was
already on disk, booked the day before for two other items. The reading
that closed this one was a `grep` over the decompile export and one slice
of a dump; the diff that keeps it closed is the same run68 window, widened
from 119 blocks to **123** and from 118,948 fields to **122,752**. The
whole of `docs/MERCHANT.md` §3 is diff-backed now: the `calc_gather` gate,
`good_merchant_spot`'s two-by-two, the `MOVE_289` walk order, `radius[3]`,
the snap, the angle and the queue position. `detect_unit_collision`, the
ring's third test, still refused nothing — the winner was the fourth
candidate of forty-nine — so §7's seam stands.

**The frontier moved to a unit already on the queue.** With the merchant
right, run68's first parting is `1/13`'s own position on **6718**: item
191, whose two middle waypoints have been one 48-grid step off since block
6686 and which walks them for thirty-two frames before its body goes with
them. And a new item behind it: **no capture has ever seen a merchant
unpack.** run68 ends eleven frames short of the deploy spot, and its
closing block carries only the object base — enough to say the merchant
stands on (32280, 36888) with its `SubObjectData.flags` gone 9 → 1, and
nothing about the cast, the footprint or the `rare`/`good_obj` pair. Item
192, and a `[6730, 6800)` window buys the lot.

## 2026-09-03 — Fable steering: the lower map was standing still

**The occasion was the count**, not a stall: about forty-five items and
3,131 frames of East Indies since the 09-01 pass, the headline moving on
nearly every session. A steer by count over a tranche that is moving
mostly confirms the tranche, and this one did — with one exception the
count rule found and the stall rule could not have.

**Great Lakes had sat at 2419 for two days with no item on it.** Every
handoff since run61 said "Great Lakes unchanged at 2419" and the queue
said "East Indies leads". DECISIONS 25 says the headline is the pair,
lower map first, and the default item is the lower map's nearest
divergence; East Indies is 4,320 frames ahead. Nobody decided to chase
the higher map — the East Indies items kept landing, each one booked
against the score it moved, and the lower map slipped out of the
handoff's first line. The rule's own rationale is the reason it matters:
a residue that shows on one map only is what a single-map chase cannot
find, and Great Lakes has no islands, no barge and no merchant walk, so
whatever parts at 2419 is a mechanic East Indies reaches later or never.
`run53_s_24000_frames_put_the_ceiling_where_run33_did`, run in release
today: word and sequence both part at 2419, the draw-count kind, and the
labels are on disk. Item 193, at the front; 191 second.

**And the map had earned a capture nobody took.** Entry 29's rule — a
map earns its next full-detail capture when its word crosses the newest
one it has — fired on 2026-09-02 and did not run. Amended in place.

**On a second lane for captures**, which the user raised: no. The screen
is one resource — one window, one bottle, one `Logs\` — and the capture
needs no judgment; a background shell from the session's first five
minutes is the lane, as run58 showed (launched in the first five, read in
the last ten). A second agent would boot a full context to babysit a
wait, on a token-bound loop, and the failures that need attention need a
human either way. The case for an agent is a capture whose *recipe* is
unwritten; Great Lakes' is `longtrace.sh`'s default game.

**Lore's re-cut, and the number it corrects.** The conventions adopted
on 09-01 are invisible on the token axis: per grind session, output
median 113k → 120k, mean 122k both sides, p90 flat, out/req 516 → 533,
n=58 and 38. Per item closed, median 90k → 88k; the pooled drop (100k →
83k) is the batch-close mix — four sweeps closing 23 of 56 items inside
ordinary-cost sessions. What did move is throughput, 17.9 → 37.6 items a
calendar day, and session-hours per item 0.58 → 0.49: wall-clock, as C1
was conceded to buy. The 09-01 addendum's 189k median was hand-summed
over streaming snapshots and ~1.9× high; the baseline re-pins at 113k /
231 requests, and "one item ≈ one session" retires (0.72 → 0.69
sessions per item). Lore writes both onto its page. The instrument for
any future convention claim is out/req, not per-item anything.

**What steering did not do**: touch the scores, the floors or the code.
The cadence rule stays as written; note for the next count-triggered
pass that this one found a direction error, not a stall, which is the
case for keeping it.

## 2026-09-03 (Opus) — item 193: Great Lakes' 2419 is item 191's route, on the other map

**The steering pass booked the look and the capture in one session, and
that is how it went.** run69 — Great Lakes at run33's detail out to 3,000
frames — launched in the first five minutes as a background shell, and the
diagnosis was done off run53's trace while it ran. Twenty minutes, 468 MB,
`rngcmp` against run53 zero differing over 3,001 frames and `samegame`
against run33 zero differing over their 1,850 in common. The lane works;
it did not need an agent.

### One draw, and it is a frame and not a thing

Frames 2416–2418 and 2421–2422 agree label for label. 2419 is one draw
here and none there; 2420 is one there and none here; the draw is the same
site on both sides — `Guy::set_anim+0x97a < Unit::do_non_flat_gather+0xb99`,
the return-to-camp stand — and the unit is the AI's woodcutter `1/9`.

The clock behind it never disagreed. `1/9` chose its tree on frame 1959 and
the tile choice's own draw, `+0x54b`, returns **4** in the trace: `400 + 4 %
200` is 404 on both sides. The countdown does not start at the choice — the
walk goes in front of the gather order and the order is only reached again
when the walk ends — so 404 frames later is the walk's end plus 404, and
the parting says the walk ended a frame early.

### The walk, and the two nodes

run69's `MOVEORDER` rows are the first Great Lakes dump that carries it.
`1/9`'s waypoints are the original's on every frame of the game until
**1993**, and then:

```
slot 2   ours (40728, 17544)   theirs (40776, 17544)
slot 3   ours (40824, 17640)   theirs (40872, 17640)
```

Two middle slots, 48 short in x apiece. Same ends, same dog-leg, same
switch frame, same total length — ours 144 + diagonal + 144, theirs 192 +
diagonal + 96 — and this crate reaches the tree on **2015** where the
original reaches it on 2016.

That is item 191's signature exactly: run68's `1/13` on East Indies parts
on the same two middle slots of its own `PATHDATA` stack from block 6686.
**So the lower map's seam and the higher map's frontier are one defect**,
and the queue now has one item in front of both numbers instead of two.
Which is the argument for DECISIONS 25's own rule, made by the rule: a
residue that shows on one map only is what a single-map chase cannot find,
and Great Lakes — no islands, no barge, no merchant — reached the same
node difference through a woodcutter's walk to a tree.

### What the capture bought beside the answer

`run69_s_three_thousand_frames_stand_where_the_original_s_do`, in run57's
shape. Of fourteen units that ever leave the original's point, thirteen
part between 2467 and 2930 — all past the word, where both streams are on
draws that are nobody's. `1/9` parts 474 frames earlier than any of them,
and that gap is what makes "one unit before the word, and it is 191's" an
assertion rather than a coincidence. Collision: **228,821 field-frames,
none wrong**. Buildings: 95,476 fields, nothing wrong before the word and
one row after it (`1/2010`'s `y_internal` from 2577, an AI placement past
the parting). The waypoint rows are pinned **as they stand** — thirteen
`dest_x` from 1993 and two `dest_y` from 2000 — so the day the route is
right the test fails rather than passing quietly.

### What is not established

Which step of the search puts the node there. Both routes are legal,
equal-length and on the same grid, so this is a **tie-break rather than a
price**: `astar_path`'s direction wheel, its comparison on an equal `f`, or
the order the open list is walked. Nothing was read against the listing
today, and neither capture carries the search's own probe count — run55's
`callwin` over `astar_path` and `calc_cost` is the instrument that would,
and Great Lakes frame 1960 is where to point it.

**The score.** East Indies unchanged at 6739; Great Lakes unchanged at
2419. No number moved, and the session says so: what moved is that the two
frontiers became one item, and the map that had been standing still for two
days now has a capture that can see past its own word.

## 2026-09-03 (Opus) — item 191: the search was right, one bit of the grid was not

**Great Lakes 2419 → 2808, and both maps' frontier closed by one line.**

### The reading said the search was right

Item 191 came in as a tie-break question: two routes, legal, equal-length,
on the same grid, and the queue's opener said to read `astar_path`'s wheel
and its `<`/`<=` on an equal `f` before anything else. So that is what was
read — and every part of it came back confirmed. `get_estimate@00688310` is
`vector_dist × 10` on the unit grid and `vector_dist@0046cff0` is this
crate's function line for line. The unit grid's `calc_cost@00684e50` takes
the `param_6 == 0x30` arm and returns before every terrain, danger, owner
and fog term: a flat **32 cardinal, 40 diagonal**, which makes `h` about
fifteen times the true cost and the search a near-greedy beeline.
`Tree::ordered_insert@004796f0` descends right only on a strict `<`, so an
equal `value` goes left and the newest equal is expanded first;
`Tree::remove_current@00479770` is the textbook no-copy delete and keeps
the in-order, so the LIFO survives every removal; `find_node_open` reseats
`current_parent` from the node itself, so the decrease-key unlinks the node
it means to.

And then the arithmetic said the branch was not a tie at all. At the node
where the two routes part, this crate's diagonal child has `f = 2816` and
the original's cardinal one `f = 2958` — 142 apart. **No ordering rule
reaches a node 142 worse.** Either the original refused the cell, or
something no reading had found.

### The capture that answered it in three minutes

`calc_cost` is called **only for a neighbour that passed `valid_ucoord`**,
so a `callwin` over it does not merely price the steps — its argument list
*is* the validity filter's answer, one row a cell. run70: run53's game,
`cover=0`, `callwin=1955-1985`, `end: MISC`, 2,000 frames. Three minutes
and 10 MB, against run69's twenty minutes and 468 MB, and `rngcmp.py` says
2,001 frames zero differing.

Laid side by side, the original's twenty-one expansions and this crate's
twenty-two agree on **every cell either probed but one**:

```
ours valid, theirs invalid : [(849, 366)]
ours invalid, theirs valid : []
```

`(849, 366)` is `(40776, 17592)`, the node this crate turned south onto.
The original refuses it from all three neighbours that reach it and never
prices it at all. With it gone, the wheel and the heuristic put the route
exactly where the original's is — same nodes, same order, same waypoints.

### The bit, and the mechanism that puts it back

The refusal turns on `(848, 367)`, a diagonal of `(849, 366)` and a corner
of the **standing** gatherer `1/10`'s block. `1/9` had walked west through
`(849, 366)` on frame 1885 and left it diagonally for `(848, 365)` on 1889;
`CollCheck::move_unit`'s clear pass takes every cell of the old disc more
than `coll_size` from the new one, and `(848, 367)` is two rows from
`(848, 365)`. The index is **not refcounted** — `docs/COLLISION.md` §2 has
said so since the mechanic was written — so a walker punches holes in a
stander's block and nothing in `move_unit` or `add_to_world` ever fills
them.

`Guy::process@005e0230` fills them. After `Guy::move`, on the frames where
`(game->frame + o) % 64 == 0`, a guy whose `GuyData::avg_speed` is **zero**
re-marks its whole disc — `radius[coll_size]`, set-only, with §2's region
gate — allocating the `CollBlock` if the cell has none. `1/10` is object
10, so `(1910 + 10) % 64 == 0`: the original healed the hole on frame 1910
and its search saw a wall on 1970 where this crate saw a gap. The four
gates are the original's in its order — not air, `guy_num < squad_size`
(`UnitType +0x304`, not `UnitData::guy_mark`), `coll_size != 0`, the phase,
and the standing test. `docs/COLLISION.md` §2.2.

Eleven lines in `collide.rs` and one call in `process_movement`.

### What one line moved

- **Great Lakes' long word 2419 → 2808.**
- run69's collision record **228,821 → 247,543 field-frames, none wrong**;
  its buildings **650 rows wrong → none**, over the whole three thousand
  frames — the `1/2010` Dock placement at 2577 went with the route.
- Fourteen units ever off position → **eleven**, and the earliest **1993 →
  2804**. The word is 2808, so exactly one unit is now in front of it, by
  four frames, and it is pinned as it stands.
- On East Indies the same line closed item 191's other half: run68's window
  compares `1/13`'s stack whole with its exception **deleted**, and the
  window's clean run went 6718 → **6730**, the last block that capture
  carries. Every field of every unit, no exception but `stance`.
- East Indies' own word is unmoved at 6739 — the residue there is the
  merchant's unpack, which was already the next item.

### What is worth carrying

**"Prefer a diff to a reading" is not only about prose being wrong.** The
reading here was right in every particular and still could not answer the
question, because the question was about *state* and a decompile only
carries *code*. What settled it was a three-minute capture whose whole
product was a list of cells. The rule that found it is the one already
written down — grep the disk, then widen, then book the capture — and the
new part is which capture: a `callwin` over a **filtered** callee turns
that callee's caller into an oracle for the filter, which is a general
trick and had been used once.

**And the cheapest capture on the lane was the one nobody had repeated.**
run55 was seventeen minutes because it kept run39's full detail to stay a
sibling. Nothing about a call proxy needs the dump at all: `end: MISC` and
`cover=0` make a 2,000-frame Great Lakes run cost three minutes, and
`rngcmp` against run53 still proves it is the same game.


## 2026-09-03 (Opus) — item 194: the builder that finished a farm and stayed (Great Lakes 2808 → 2930)

Item 191 left Great Lakes with one unit in front of the word: `1/1`, the
AI's citizen, leaving the original's point on **2804** against a word of
2808. The item was booked to read which field parted first, because run69
is at run33's full detail and holds `1/1` whole either side of it.

### The dump answered it without a reading

`1/1` stands still on `(42984, 24600)` from 2653 to 2802 with one order —
a `BUILDORDER` on `2011`'s predecessor `2010` — and on **2803** three
things happen at once: `2010`'s flags go `3 → 7` (finished), the build
order's target goes `2010 → 2011` with its `flags 4 → 0`, and an
`EXPLORETOORDER` to `(42888, 23880)` is pushed on top. On 2804 it walks.

This crate's `1/1` was holding a `Gather` on the building it had just put
up, standing until 2809 and then walking somewhere else entirely. The line
is `Unit::build_done@00603bf0` (`docs/ORDERS.md` §5.5), whose AI arm is

```
find_build_spot() or find_repair_spot() or (starting_resources != 8 and find_gather_spot(range))
```

and of the three only the last was modelled. §5.2's step 6 had been right
since item 47 — an AI builder never adopts its own site — so the citizen
was reaching `build_done` correctly and `build_done` was sending it to
gather, which is the *third* thing the original tries.

### What `find_build_spot` turned out to be

`Unit::find_build_spot@00603e20` is short and its two searches are not.
Both `Objects::find_builds@0065a120` and `Objects::find_units@0065a620`
carry two implementations and choose on cost: `n = (range + 0x2ff) /
0x300`, the range in cells, and the circle path runs while
`circle_radius[n]` is no more than `game->num_def_builds` (a flat **200**
from `Game::init_data`, the per-player object-array size — which is why a
building's `o` is `2000 + i`) or `game->total_units` (the live count). The
build search is therefore always the circle at a citizen's ranges;
`circle_radius[3]` is 45 and `circle_radius[6]` is 145, against 59 live
units on run69's frame 2803, so the unit search is not.

`FILTER_CONSTRUCT` took the PE. `Search::valid_filter@0067dbb0` is one
indirect jump; the table is at `0067e57c`, the index is `filter −
FILTER_TYPE`, and `llvm-objdump` over six dwords names arm 5 at `0067dd54`
— `vtable+0xc`, then a **negated** `vtable+0x4c` — with arm 6 next door,
`FILTER_DAMAGED`, the same pair un-negated plus `+0x24 damage != 0`. That
neighbour is what settles the polarity: `FILTER_CONSTRUCT` is "exists and
is **not** active". The enum itself came off `llvm-pdbutil dump --types`
in one grep. `docs/ORDERS.md` §5.10 is the whole reading.

### What it moved

- **Great Lakes' long word 2808 → 2930.** East Indies unmoved at 6739.
- run69's collision record **247,543 → 253,874 field-frames, none wrong**;
  buildings 95,476 fields, none wrong, over the whole capture.
- Eleven units ever off position → **six**, and the earliest **2804 →
  2935**. Nothing parts before the word now, so the run69 assertion that
  was pinned to `[(1, 1, 2804)]` is the empty list — it failed on its
  first run after the change, which is what it was written for.
- run69 is 3,000 frames and the word is now 70 short of its end: the next
  full-detail Great Lakes capture is owed a longer one
  (`docs/DECISIONS.md` 29).

### The half that did not land, and why

§5.9 puts the same search in `think_peasant`, ahead of the gather one:
`not a scholar and (unit_masks & 0x400 or worker_stance ∈ {1,2}) and
find_build_spot()`. Written, measured, and taken back out. It is
`worker_stance` that decides who asks, and this crate writes a flat 1
where the original computes it per unit — item 190, which had been a
2,700-row curiosity on East Indies and is now load-bearing. On run69's
frame 110 the original's `1/6` carries `stance 0` and its four siblings
1; with the arm in, `1/6` is born on 100, walks to a build site on 101 and
leaves the original's point on **103**, 2,800 frames in front of the word.
Sixteen units parted instead of six.

That is the honest shape of it: a correctly-read predicate cannot land
while a field it reads is wrong, and the diff says which of the two to fix
first. Item 190 now has a dependent.

### What is worth carrying

**The dump is a reading of last resort's replacement, again.** The whole
of "which field parts first" was four `track.py` invocations over a
capture that already existed — the order stack whole, the two buildings'
flags, and a `--changes` fold that put 2803 on the screen with nothing
else on it. No emulator, no capture, no reader.

**And a jump table is six dwords.** `Search::valid_filter` has been "an
indirect jump the decompiler cannot recover" in three documents; naming
one arm of it cost one `llvm-objdump` window and one neighbour to check
the polarity against. `docs/CARAVAN.md` had already paid for the table's
address a week earlier — the note that recorded it is what made this
five minutes.

## 2026-09-03 (Opus) — item 196: the citizen was still walking (Great Lakes 2930 → 4241)

Great Lakes' word stood at run53's **2930**, and the queue had already
named the frame: the sequence there is the original's with one
`Animal::think_farm_animal+0x142` **prepended** — first in the frame,
every other label identical, 2929 and 2931 draw for draw. FARMS gave it
three gates that could drop the draw. The probe said which.

### The gate, and it was the one nobody suspected

A scratch fold over run53's trace printed every pasture animal beside its
phase and its gates:

```
animal o14 slot0 build17 gatherers[78] refTile(217,132) covers false   phase hits 2930
animal o9  slot0 build15 gatherers[8]  refTile(210,85)  covers true    phase hits 2935
```

The phase was right — 2930 is exactly where `(o · (slot + 1) + frame) %
128` lands for the pasture `2007`'s slot-0 animal. `build_covers_tile` was
right too. What was wrong is the **object it was handed**: the citizen
`78`, standing on tile `(217, 132)`, five tiles off a pasture whose own
animals sit around `(222, 127)`. A farm does not cover a tile five away,
so the draw was dropped.

Five tiles is not where a gatherer stands, and that was the tell.
`think_farm_animal@005d7700` picks its reference on

```
iVar12 = BuildData::num_gatherers(this_00, 1, 0);
sVar9  = iVar12 == 0 ? this->field_0x150          // the farm's own o
                     : *(short *)(iVar12 + 0x70); // gather_down, the head
```

and that first argument is `is_gathering_at`'s third — **`arrived`**,
which is `been_there`. `docs/ORDERS.md` §6.1 has had this since it was
written: a citizen joins the chain the moment `add_gather_order` issues
it, and sets `been_there` only when it gets there. So through the whole
walk out the count is **zero**, the measured object is the farm, the farm
covers its own tile, and the draw is spent. This crate read the chain's
*length* — `gatherers.first()` — and measured a citizen that was still
three hundred units from arriving.

One line, and both halves of it were already in the repo: `docs/ORDERS.md`
knew what `been_there` was for, and `Sim::num_gatherers(b, arrived,
skip_decoys)` had existed since the economy needed `(1, 1)`. What was
missing was the connection, and `docs/ORDERS.md` §6.1 had actively
asserted against it — "**every** UI/AI caller uses `num_gatherers(0, 0)`".
That sentence is now struck: there are three call shapes, not two.

### What it moved

- **Great Lakes' long word 2930 → 4241**, 1,311 frames on one draw. East
  Indies unmoved at 6739.
- run53's frames on the original's count 4,231 → **6,514**; draw for draw
  2,040 → **4,582**.
- run69's collision record 253,874 → **254,924 field-frames, none wrong**
  — and **no unit anywhere in its 3,000 frames ever stands where the
  original's does not**. Six parted between 2935 and 2996 yesterday; one
  draw took all six, because a pasture animal that does not walk is a
  pasture animal whose neighbours' collision arithmetic is a different
  game's. That is now asserted outright rather than only before the word,
  so run69 has nothing left to say about position.
- The word is past the whole of run69, so Great Lakes is owed a longer
  full-detail capture (`docs/DECISIONS.md` 29) — the successor item, and
  it is the same debt run69 itself was created to pay.

### What is worth carrying

**A gate with three inputs is three suspects, and the cheap probe names
one.** The queue booked this as "three gates can drop it" and listed all
three. Printing the three side by side for every animal in the capture
cost one scratch test and two minutes, and it eliminated two of them
before a single line of the decompile was read. The reading that followed
was one function and one call site.

**The predicate was wrong where the arithmetic was right, again.**
`docs/audit/README.md`'s most-recurrent lesson has now claimed the
pasture: the phase formula, the corner table, the `192·T + {24, 120, 168}`
destination and the snap were all exact, and had been for a week. What
was wrong was *which object* — the same shape as "which array a level
indexes" and "which step a multiplier belongs to".

**And a document that asserts a universal is a document that can be
falsified.** §6.1's "every UI/AI caller uses `(0, 0)`" was written from
reading the callers that mattered at the time; it is what made
`gatherers.first()` look safe. The correction is in place with its
successor named, per the amend-in-place rule — and the assertion that now
holds the rule was made to fail on the old code first.

## 2026-09-03 (Opus) — item 190: one byte, four meanings, and the trainer that overwrites it

`UnitData::stance` was 1 on every unit this crate created and 0 on 2,700 of
run68's rows. It is not one quantity: `GroupData::get_stance_option@0070bab0`
sizes the option array from `StanceTypes` — six for a combat unit, four for a
worker, two each for a caster and a packer — so the byte is an index into a
list whose length its *type* fixes, and `Unit::init@00612100:282–309` reads a
different place for each. `docs/ORDERS.md` §5.10 is the whole switch;
`crates/sim/src/stance.rs` is the code.

**The worker arm is inverted from what its field name says.** `leader_flags &
4` is `LeaderData::is_human@006ec170`, and it is the *human* that takes
`leader_options[who].peasants` while the **AI** takes the lobby's
`(starting_resources == 8) + 1`. That is what makes the human's starting
citizens 0 and the AI's 1 — which is exactly what two captures print, on two
maps, at their first block. run69's leaders say which is which without a
reading: leader 0's flags are `176160775` (`…111`, bit 2 set) and leader 1's
`176160787` (`…10011`, clear).

**And then the trainer overwrites it.** The first fix got the AI's *starting*
citizens right and left `1/6..1/15` — the ones it trained — wrong in the
other direction. `Build::train@0062f9b0:86–101` calls `Unit::set_stance` with
the **building's** own byte whenever the two stance kinds match, and a
building's byte comes from `Build::init@00629740:92–113`, which is the same
switch with two arms rewritten: its worker arm is
`(!human && starting_resources == 8) ? 2 : peasants`, so a city is 0 for
everybody. A citizen out of a city takes the city's 0; an AI's five starting
citizens, which never went through `train`, keep the 1 they were born with.
One capture prints both populations in one block.

**What it bought.** run68's window is now **138,769 fields over 135 blocks
with none differing**, against 122,752 over 123 with `stance` excepted by
name — the last exception in that test is gone. Neither word moved:
East Indies 6739, Great Lakes 4241. The widening ledger's single-capture
count fell 43 → 42; it had named `stance` as a one-capture field the day
before run68 found it wrong, and this is the same field leaving by the other
door.

**194's other half landed with it.** §5.9's build arm — `not a scholar and
(unit_masks & 0x400 or worker_stance ∈ {1,2}) and find_build_spot()` — had
been written and taken back out twice, because with a flat 1 every citizen
asked and run69's `1/6` left the original's point on 103. With the stance
right it asks who the original asks, and both words held.

### What it cost, and what the method says

**Four grep-sized reads beat a capture, and the dump had the answer twice
over.** The whole derivation is `Unit::init`, `Build::init`, two
`get_stance_type`s and `LeaderOptions::init`; the *check* is two start blocks
already on disk. The queue had booked this as blocked on a reading of
`LeaderOptions +0x4/+0xc/+0x1c`, and the names came out of the executable's
own UTF-16 literals at `0xae1628`, `0xae1630`, `0xae156c` and `0xae1668`,
which `LeaderOptions::log_data@006f1480` passes to the logger — the PE-global
trick, one level down from a struct.

**The first fix was right and the diff still failed, which is the argument
for diffing a whole window rather than a block.** `init_stance` alone
reproduces both maps' frame 0 exactly and would have been called done on that
evidence. It was run68's 6595 that showed the trained population going the
other way, and `Build::train` is not reachable from `Unit::init` by reading —
only by asking who else calls `set_stance`.

**A seed and a rule are different claims, so they are now different tests.**
`build_sim` takes the dump's `stance` where the dump prints one, which is the
stronger seed; `init_stance_is_the_original_s_on_both_maps_first_blocks`
asserts the *derivation* against the same blocks, so a wrong rule cannot hide
behind a right seed. It was made to fail first, by inverting the human
branch: `0/1 (type_index 50) stance left: 1 right: 0`.

**Not established**, and in §5.10's coverage: what `buildings +0xc` is named
for, given `Unit::init` reads it for a combat unit; the option bits beyond 3
and 4; both `starting_resources == 8` arms, which no lobby on disk reaches;
and the two writers — a player's click and `set_auto_peasant_level` — neither
of which is modelled, which is why `Sim::build_stance` computes the
building's byte instead of storing it.

## 2026-09-03 (Opus) — item 197: the capture that took two sessions to launch, and the 64 frames it found underneath the word

**The item was one capture and it was blocked on a checkbox.** Great Lakes'
word is 4241 and its longest full-detail dump was run69's 3,000, so every
frame of the parting fell past the end of the only file that could show it —
`docs/DECISIONS.md` 29's standing rule, one map later than run56 and run69
answered it for East Indies. The capture had been launched and parked twice.
Both times the diagnosis was "a Claude Code update reset macOS
Accessibility; toggle `ClaudeCode.app` off and on". Both times that was done,
and both times it did nothing, because **both halves of that sentence are
wrong**.

`tccd` says so in one line, and reading it took a minute where guessing had
taken two sessions:

```
AUTHREQ_SUBJECT: subject=/Users/…/.local/share/claude/versions/2.1.259
```

The responsible process is the **bare versioned binary**, not the bundle. It
has no `Info.plist`, so TCC has nothing to key on but the absolute path — and
that path carries the version number. So every update writes a new path and
revokes Accessibility, Screen Recording and Automation together; and adding
`ClaudeCode.app` in System Settings does nothing whatever, because macOS
never evaluates that path. The row appears, stays ticked, and is never
consulted. `TCC.db` needs Full Disk Access and its mtime is not evidence — a
*denial* updates it too — but `/usr/bin/log show --predicate 'subsystem ==
"com.apple.TCC"'` is open to anyone, and it names the exact path being
judged. That is now the first thing ORACLE tells the next session to do.

**`~/bin/RonDriver.app` ends the tax** (`tools/gamelog/rondriver/`,
`viadriver.sh`). `open -a` launches it through LaunchServices, so the bundle
is the responsible process and every child inherits that; under it the
subject is `com.ramonfabrega.rondriver`, an *identifier* rather than a path,
which is the stable thing the native install never had. It spawns and waits
rather than exec'ing — an exec would replace its image with `/bin/zsh` and
hand the attribution straight back to the interpreter, which is the bug it
exists to escape. The attribution was read back out of `tccd`'s log before
any of this was believed.

**The probe that let it through had already been rewritten once, and the
rewrite lied the same way.** `cliclick p` reads the cursor without needing
Accessibility, so it passed while every synthetic event was being dropped;
its replacement asked System Events for `every window of process "Finder"`,
on the reasoning that Finder always exists. Finder is usually running with
**no windows open**, and an empty list needs no accessibility call to
produce, so the question returns the empty string and rc=0 either way. It
printed `probe ok (cursor 1649,0)` on the very run it was written to catch,
and that run then clicked Solo, Quick and Start into a dead menu and sat in
its poll loop reporting `gamelog=0 still=8`. The same call against the game,
which does have a window, refuses with -1728; the probe simply never asked
anything that had to be answered.

Two probes, both defeated the same way: each asked a question whose answer
looks identical granted and refused. `perm_probe` (`tools/gamelog/probe.sh`,
now shared by all four capture scripts) asks one that cannot be — post a
synthetic move, read back where the cursor actually went, twice so a cursor
already on the target cannot pass by luck. Made to fail first against a stub
`cliclick` that behaves exactly like a denied one, which is also the trap
underneath all of this: **denied, `cliclick` exits 0 and prints a plausible
position, warning only on stderr.** A script's own log line is not evidence
that a click landed. `screencapture` is.

**run71 is clean and it checks itself.** 832 MB, 5,001 frame blocks,
`MAP_STYLE 14`, seed 12345, run33's recipe with only the length changed;
`rngcmp` against run53 is 5,001 identical frames and 0 differing, and
`samegame` against run69 differs on none of their 3,000 common frames.

**And what it found is 64 frames below the word.** The word is 4241,
measured off run53's *draw* stream, and nothing had ever compared a position
past run69's 3,000. Positions part at **4177**, and two units go at the same
instant, differently:

- `1/11` **stops**. Ours holds (40824,19032) frame after frame while the
  original walks (40831,19056), (40838,19080), (40845,19104) — a clean
  +7,+24 a frame toward a `to_x`/`to_y` of (41736,22584) that ours never
  resumes for.
- `1/19` **turns wrong**. Ours steps +14,-20 a frame against the original's
  +25,0. Both carry `myspeed 25`, and ours moves sqrt(14² + 20²) ≈ 24.4 of
  it — so the speed is right and the *heading* is not, with `angle`
  1073741824 against a `dest_angle` of 1353318400, mid-turn.

Whether one cause or two is not established; that they fire on the same
frame is the only reason to suspect one. Both are pinned in
`run71_s_five_thousand_frames_reach_past_the_word` as the list they are, so
the day either is fixed the test fails rather than passing quietly.

**The collision block is perfect over the whole capture** — 475,556
field-frames compared, none wrong — and the buildings have exactly one
residue: 425 fields, all `1/2015`'s `y_internal`, 15936 here against 15744,
four cells of 48, and none of them before 4577. That is four hundred frames
downstream of the parting, so nothing is claimed about its cause; it is
pinned so it cannot spread unnoticed.

**The cost, stated plainly, because it is the lesson.** Roughly three hours
across two sessions went to a permission dialog, of which the diagnostic that
actually settled it was one `log show`. The rule it earns: when a permission
is refused, read the system's own verdict before touching a checkbox — and
when a run reports progress, confirm it against something the run does not
write.

## 2026-09-03 (Opus) — items 198/199/200: one defect wearing three faces (Great Lakes 4241 → 4803)

The queue booked three items off run71 and asked, as the first move on any
of them, whether 198 and 199 were one defect. They were — and 200 was the
same one, four hundred frames downstream.

**What frame 4177 actually is.** `1/11` did not stop and `1/19` did not
mis-steer. On the game frame the dump calls 4177 the AI **places a farm**:
`BUILDDATA 1/2014`, `orig_type 417`, at (41856, 22848), with a `BUILDORDER`
and a walk inserted ahead of it — the frame-1 shape of `docs/AI.md` §2.20's
own table, eight ages later. The record count says so before any field
does: 4176's block is 10,933 lines and 4177's is 11,056, and the inventory
diff is one `BUILDDATA`, one `BUILDORDER`, one `SUBOBJECT`, one `MOVEORDER`
and one `GATHERORDER` fewer.

Both sides place that farm, on that frame, on that tile — run71's buildings
never disagreed about *where*. Both sides pull a citizen off gathering to
build it. **They pull a different citizen.** The original's `1/11` takes
`[ExploreTo (41736,22584), Build ox 2014]`; this crate handed the identical
pair to `1/19`. So the unit that "stopped" was the one that was never given
the job, and the unit that "turned wrong" was turning correctly toward a
job that was not its. One event, two symptoms, and the third — `1/2015`'s
`y_internal` four cells south from 4577 — was the AI siting a later
building around a citizen standing in the wrong place.

**The rule the original uses.** `produce_building`'s builder loop scores
every citizen of the leader by tile distance to the site plus a penalty for
what it is doing (`docs/AI.md` §2.20). The decompile prints the distance as
`vector_dist(unaff_EDI, unaff_ESI)` — Ghidra lost both arguments, because
both are set outside the loop — so the listing had to settle it, and it is
not what was implemented. `006e28b2`–`006e28ec` reads the unit's own `x`
and `y`, converts **each** through `div_3_table` (a floor, one coordinate
at a time), and subtracts the results from the candidate's **corner tile**
(`local_5c`/`local_70`, built at `006e2656` — the same value the jitter
starts from). This crate took the difference in world units and divided
once. Truncating a difference and differencing two floors are the same
function only when the two floors do not straddle a tile boundary, which is
most of the time and was not this time.

The arithmetic, on the frame: corner (216, 116); `1/11` at (40824, 19032),
tile (212, 99), so |4|,|17| → `vector_dist` 17; `1/19` at (42017, 25656),
tile (218, 133), so |2|,|17| → 17. Both gather at a woodcutter, so both
take the same +10 timber penalty against a city radius of 20. **27 each,
and `jge` at `006e2a57` keeps the earlier unit.** Difference-then-divide
read 18+10 = 28 for `1/11` and 15+10 = 25 for `1/19`, and sent the wrong
one. The whole defect is a tie the wrong arithmetic could not produce.

**A second reading the listing paid for, and cost nothing.** The penalty
ladder calls `BuildTypeData::get_good` twice, and Ghidra prints the second
call's `this` as an uninitialised `this_03` — which reads exactly like the
ladder testing the *placed* type after testing the *gathered* one, a
predicate bug of precisely the kind the audit README says to expect. It is
not one: `get_good@0063bd50` is `[this+4] − 417` into a six-way jump table
and writes only `eax`, so `ecx` survives and the second call is the same
object as the first. Thirty seconds of `llvm-objdump` against a plausible
wrong answer.

**What it moved.** Great Lakes' word **4241 → 4803**; its position parting
**4177 → 4827**; units ever off the original's point in run71's 5,000
frames **19 → 11**; the collision block from 475,556 field-frames to
**513,465**, still none wrong; and the buildings from 425 wrong fields to
**zero**, untouched. `LONG_WORD_GREAT_LAKES`, the run71 test's floor and
its unit list all move together, and the queue's `Long captures:` line with
them.

**What is now first.** 4827 is `1/15`, alone, and it is a farm's work
spot rather than a route: the citizen has been at farm `2013` (centre
(42624, 22656)) with `been_there 1` since long before, and on 4826 the
original gives it a move to **(42936, 22776)** where this crate gives it a
move to its own point, then on 4828 one to **(42360, 22968)**. Same farm,
same cadence, different tile. That is item 201.

**The paperwork tax, paid as designed.** `## 2. The production AI — read`
sits over the 16 KB section ceiling and is pinned at its own size, so the
new subsection was paid for by compressing §2.20's older prose — the
run20 draw-count history, the stride's consequence paragraph, and
`buildings_allowed`'s closing story, all of which are here now. The pin
came down 71,929 → 71,928 with it.

## 2026-09-03 — item 201 was a symptom, and the flattening was in the wrong frame (Opus)

**The item as booked was not a defect.** `1/15` re-picks a cell of its own
farm on Great Lakes' 4827, and the queue had it as the farm work-spot pick.
The pick is right. Sim-frame 4825 spends the two `GameAccess::rnd+0x20 <
Unit::do_job+0x67` draws `docs/ORDERS.md` §6.5 names, and the trace says
what they read: 26899 and 16738, whose `% 4` is (3, 2) — the tile the
original walks to, `corner + (3, 2)` snapped, (42936, 22776). This crate
reads different numbers because it is drawing from a stream that parted
**twenty-four frames earlier**, at the word.

**The word is a road search.** Frame 4803 is 277 draws at
`PathFinder::calc_road_cost+0x46 < PathFinder::astar_caravan_road+0x52b <
PathFinder::find_road+0x3a8` against this crate's 266 — one search, and the
building is player 1's Market `o 2015`, which finishes on that frame and
plans its road from its far corner tile (228, 83) to London's (220, 84).
Everything from 4809 on, the position parting included, is downstream of
eleven nodes.

**run72 is run62's instrument on Great Lakes' own word** — the `DUMP_ALL`
window on `[4800, 4806)` and the three road proxies over `[4799, 4807]`,
fourteen minutes and 430 MB, `rngcmp` against run53 zero differing across
4,811 frames (`docs/ORACLE.md`). Booked because nothing on disk could
answer it: run71 is the map's only long dump and carries no tile masks, no
heights and no call records.

**What it found before the sequence did.** run64's lesson is that a
`DUMP_ALL` block carries `master_land_heights` whole, and comparing the
whole grid said in one run that **62 tiles** of the Market's own ground
were still the map generator's here where the original had already
flattened them — flat 175 across the 4 × 4, tapering to (223, 78) and
(230, 85). `docs/ROADS.md` §7.4 had read the call correctly a day earlier:
the terraform is `Wall::init@0063e9b0:70`'s, under `param_6 == 0` and
`TVar10 != FARM`, and `Wall::start`'s own statement is
`Terrain::object_placed`, which moves no height. The **implementation** had
it in `Wall::start`, and nothing could tell: for a building placed and
started at once — run32's two cheat-channel enhancers, the only ones any
capture had — the two land on the same frame. A building the AI *builds*
is placed on 4577 and started on 4803, **226 frames apart**. With the call
moved to `Sim::init_build`, the height grid is the original's on all
**921,600** tiles.

**And what is left is a mechanic this crate does not have.** The node
records agree for 80 nodes and part on the 81st: the same tile (223, 79),
the same direction, priced **387** here against **27** — plain ground
against road. The ring is not the difference; this crate lays the Market's
sixteen ring tiles exactly, the border of `[224, 228] × [79, 83]`. The
original lays a **seventeenth**. `World::set_road_at@006b43b0` ends, for a
tile that was not already a road, in `Roads::road_added@008954d0` →
`Roads::add_roads@0088f4b0` → `Roads::set_diags@0088e9d0`, which lays road
of its own to close the corner between the ring's new (224, 79) and the
road already standing at (223, 80). 33 tile masks carrying a `0x4` bit
nothing here writes are the same absence seen from the other end.
`docs/ROADS.md` §9 is the booking, and run72 is its oracle, already on
disk.

**This session moved no score.** Great Lakes' word is still 4803 and its
position parting still 4827; the whole suite is green and 921,600 tiles of
height that were wrong are right. That is the honest shape of it: the
capture converted a number into a named mechanic and a pinned node, and
the mechanic is the next item.

**A lane note, because it cost forty minutes.** The first two launches of
run72 hung with the game at 0 % CPU and **the wineserver blocked inside
`open()`** — every other thread waiting on it, the trace stopped at 1,552
records in the CRT's static initialisers. That is the shape
`docs/QUEUE.md`'s memory calls a consent dialog, and it was not one: no
TCC request for the bottle appears in `tccd`'s log at all, and three
controls — run71's recipe bare, then plus the proxies, then run72's exact
configuration with the window moved to frame 50 — all reached the lobby,
after which run72 itself did too, unchanged. A cold bottle can hang on its
first launch. `sample <wineserver-pid>` is the one-minute check that says
so, and the run71-recipe control is the discriminator: if it reaches the
lobby, relaunch rather than diagnose.


## 2026-09-03 — the seventeenth tile (item 202)

**Great Lakes' word 4803 → 5502, by draw and by sequence, and run71's whole
five thousand frames now have no unit anywhere off the original's point.**
East Indies is unmoved at 6739. One mechanic, `docs/ROADS.md` §9, and it is
`crate::mesh`.

**What the previous session left.** run72 had priced the Market `1/2015`'s
road node for node against the original's and found them agreeing for
eighty nodes and parting on the eighty-first: the same tile `(223, 79)`,
the same direction, **387** here against **27** — plain ground against
road. The ring was not the difference. `place_roads` lays sixteen tiles,
the border of `[224, 228] × [79, 83]`, and this crate lays all sixteen
exactly. The original lays a **seventeenth**, and no search puts it there.

**What lays it.** `World::set_road_at@006b43b0`, for a tile that was not
already a road, calls `Roads::road_added@008954d0`, which queues a
`RoadModification` and runs `Roads::add_roads@0088f4b0` over the queue —
**once per tile**, not once per frame. Its second pass calls
`Roads::set_diags@0088e9d0`, whose first line is the whole shape of the
thing: *a tile with two or more orthogonal connections does none of this*.
What the pass is for is a road that ends or turns beside another one. Over
the four corners, when the corner is itself a road with **two or more**
connections of its own and neither tile between them is road or footprint,
it lays road at `(x + corner_x[i], y)` — the corner's horizontal
neighbour. The ring's brand-new `(224, 79)`, the elbow standing at
`(223, 80)`: `(223, 79)`.

**The two things a reading of it will get wrong.** Both are in
`RoadsOut::mark_and_trim_directions@008935c0`, which is where the four
cardinal connection bits come from, and both were only settled by reading
`fill_cache` first rather than the consumer.

- `neighbor_cache[d]` is *a building footprint that is **not** a road*. So
  its gate is open whenever the neighbour is a road, and the whole
  `flags |= bit; if not a road: flags &= ~bit` dance reduces to
  `bit = road_cache[d]` in every ordinary case. The gate exists only to
  *withhold a recomputation* beside a building.
- The two halves are gated on **each other's axis**. A footprint east or
  west is what lets the north–south pair be recomputed; one north or south
  lets east–west be; with no footprint adjacent, both run. Read the other
  way round it is exactly backwards, and it is one `goto` in the
  decompilation.

`set_diags`' seven five-entry tables are one contiguous stack array
indexed `[i]`, `[i + 5]`, `[i + 10]` … from a single base, so the
decompiler prints them as `local_a4`, `local_8c` and `local_78` with
overlapping indices. Laying the three declarations end to end recovers all
seven, and with them the eight connection bits: `N 0x40000000`,
`E 0x10000000`, `S 0x04000000`, `W 0x01000000` and the four diagonals in
the same byte. `RoadsOut::get_orthog_connects@00893560` counts the four
cardinals and answers **zero for a tile with no element** — a gate, not an
accident.

**What it cost and what it bought.** The module is 620 lines with its
tests: the element store, the modification queue, `fill_cache`,
`mark_and_trim_directions`, `set_diags`, `clear_roads`, `clear_support` and
the redo ping-pong that re-derives every tile either pass touched, at most
eleven rounds. The visual passes — ten of them — are not there.

- run72's 277 nodes are **node for node** the original's, tile, direction
  and cost.
- run71's whole capture goes from one unit parting at 4827 to **none**,
  and the 4827 unit was `1/15` re-picking a farm cell off a seed that was
  nobody's, twenty-four frames downstream of the road.
- run53's word runs from 4803 to **5502**.
- The world at 4802 does not move: 921,600 heights, every cell owner and
  every tile mask, with the mesh running for all 4,802 frames.

That last one is worth being precise about, because it is the strongest
and the weakest evidence at once. The mesh lays its **first** tile on
4803. So "the mesh does not over-lay" is diff-backed over 4,802 frames of
a real game, and "the mesh lays" is diff-backed at exactly one tile.

**And a claim in §9 that was wrong.** The previous session had the 33
residual tile masks — all of them bit `0x4` — as "the same absence seen
from the other end". They are not. `grep`ping the writers took two
minutes: bit `0x4` is `World::set_behind@006b4230`'s low arm, and its only
callers are `Wall::mark_behind_tiles@0063d230` and
`Mountains::add_mountain@0089c2e0`. **Nothing under `Roads` writes it at
all.** It is the strip of tiles a building stands in front of. The
correction is item 203, and three of the 33 carry a second difference —
`(217–219, 124)` are road here and are not road there — that no reading in
§9 explains. The rule that would have caught it a day earlier is the one
already written down: *grep the writers of every field you call frozen*.
It applies to a field you call somebody's, too.

**What is left.** 5502 is a **move**: five draws here against four there,
`GameDaemon::calc_market` ×3 agreeing and then `Unit::do_move+0xe84` where
the original goes straight to `Farms::inc_time`. 5503 carries the same
unit's `Guy::set_anim < Unit::move_step` on top. One unit stepping a frame
the original does not, and run53's site fold now prints the parting frame
the way run54's has since East Indies became the second map.

## 2026-09-03 — the store the recovery does not make (item 204, Opus)

Great Lakes' long word **5502 → 5571**. One store, in
`Unit::resolve_unit_collision`'s last four lines.

**Reading the frame first.** The queue's opener said the sites and the
unit watch were on run53; they were not — `debug_watch` is called from
`run_traced` and run53's test has its own tick loop. Wiring it in was two
lines, and the probe that mattered came with it: `mark_sites` folds the
marks into site labels and **drops the enclosing `unit who/o`**, which is
the one thing a one-draw divergence always wants to know. `attributed_sites`
keeps it. The label itself was also lying — `process_unit` marked
`unit {owner}/{slot}`, the vector index, where every other `who/o` in the
project is the game's `o`. That cost one probe run and is now the unit's
own `index`.

The frame named **`1/7`**, a citizen of the AI's walking back to farm `8`
at tile `(209, 96)`. `1/22` blocks it on 5501; both sides play the blocked
stand, `Guy::set_anim < Unit::move_step+0x823`, and agree draw for draw.
On 5502 the original spends nothing for it and this crate spent
`Unit::do_move+0xe84` — the `% 5` grid roll.

**Two frames of it, for ever.** The unit watch showed what the site fold
could not: from 5501 `1/7` oscillated between `(40344, 18264)` and
`(40332, 18287)` and never moved again — 18,000 frames of a two-frame
livelock. The snap put it on its cell centre, `find_upath` laid a plan, the
next `do_move` popped the plan, the tile grid walked it back into `1/22`,
and round again.

**Two arms, both from `005f7b30` and `005f9d30`.**

- `do_move`'s grid branch tests `field_0x88` — `collide`. A unit that has
  not been colliding drops its loose near waypoints and plans on the
  **tile** grid; one that has keeps them and plans on the **48** grid. This
  crate always took the first, and the code said so: *"the `collide != 0`
  arm … is still unmodelled"*. `docs/ORDERS.md` §4.4 has had the branch in
  its pseudocode since the second reading. The length a positive return is
  compared against is read *after* the pops, too, and per arm.
- `resolve_unit_collision`'s tail is the one that moved the word.
  `005f9d30`'s two closing blocks are the **same** store through
  `update_order()->get_move_order()` — the order's `+0x10`, `dest = 0` —
  and the only thing the successful one adds is the pause roll. This crate
  took the top of its own fresh `find_upath` plan as the waypoint instead.
  That top is the cell the snap has just put the unit on, so `1/7` stood
  **on** its waypoint with `dest` set: `do_move`'s arrival test runs only
  on the frame a waypoint is *taken*, so it never ran, and the frame fell
  through to the roll. `docs/COLLISION.md` §6 step 6 already said "clear the
  order's `+0x10`". The document was right and the code was not.

With the store as the original writes it, 5502 and 5503 agree, `1/7` walks
its detour and reaches its farm on 5508, and the word runs to **5571**.

**What each change is worth, separately.** The tail alone moves the word
to 5571; with the grid arm disabled it is still 5571. So the grid arm is a
confirmed divergence that no capture yet scores — which is exactly the kind
of change that rots into prose, so it is pinned by
`collide_sends_the_re_plan_to_the_unit_grid_not_the_tile_grid`, whose
scenario is run53's own frame 5502 hand-built: a unit standing on its
waypoint with a 48-grid plan under it. Both new tests were made to fail on
purpose before they were kept.

**What is left.** 5571 is 3,200 `PathFinder::calc_road_cost` draws that
agree exactly, and then **two `Guy::set_anim+0x97a < Guy::inc_time+0x271`**
this crate spends and the original does not — two figures wrapping an
animation that should not have wrapped. They fall in the `guys_inc_time`
phase, so the unit loop's attribution does not reach them; `docs/ANIM.md`
§3.3 and queue item 124 (the loop flag is per animation file) are the first
place to look.

## 2026-09-03 — item 205: the walk asked for twice, and Great Lakes' first window

*Opus.* Great Lakes 5571 → **5573**; East Indies 6739, unmoved.

The word's two draws were `Guy::set_anim+0x97a < Guy::inc_time+0x271` and
they fall in `guys_inc_time`, past the unit loop, so nothing named the unit
that spent them. The first thing done was to name it: `guys_inc_time` now
marks each unit it visits, exactly as `Sim::tick`'s unit loop does. The
mark is taken before any draw of that unit's, so it carries none of its own
and the site sequence is unchanged — and the fold said `1/23` at once, a
**caravan trained on 5564**, its two crew figures. run54's clock probe was
folded into `RON_DEBUG_UNIT` at the same time, the third capture to want it.

**The cadence was the whole diagnosis, and it was done off disk.**
`docs/ANIM.md` §3.6's crew figure is a metronome: an empty animation packet,
every slot three frames, none of them looping. A *standing* caravan's crew
mirrors guy 0 and wraps on alternate frames; a *walking* one wraps every
third. Counting the site per frame on both sides:

| | crew wraps |
|---|---|
| ours | 5569, **5571**, 5574, 5578, 5580, 5583, … |
| the original | 5569, **5572**, 5575, 5579, 5581, 5584, … |

Identical from 5572 on, shifted by one. Ours ran the standing cadence
through 5571 and the original ran the walking one from 5570 — so the
original's caravan was moving where this crate was still turning in place,
and the crew's walk request is `F + 1` on both sides.

**Two readings of `move_step` were then tested and both were wrong**, which
is what made the capture worth booking rather than another day of reading.
The near/far test measures the Manhattan distance to the **waypoint**
(`MoveOrder+0x2c`) and not to the order's destination: putting the
destination there collapsed the word from 5571 to **307**. And the
`unit_flags & 0x20` arm at `005fb0a4`, which skips the turn-in-place block
outright, is the **helicopter** bit. run72's own block, already on disk,
ruled out the third: `(227, 85)` is `0x6103` — a building — on both sides
and the caravan's own `(228, 85)` is `0x2113`, so both marches are refused
at the same tile and both detour; `go_around_building`'s push arithmetic and
the `off_x/off_y` behind it check out term for term against the decompile.

**run73** is Great Lakes' first `DUMP_ALL` window, `[5564, 5580)`, with the
road proxies over `[5563, 5581]`. 1.05 GB, thirty-three minutes, and
`rngcmp` says 5,591 frames with zero differing. It settled the item in one
field. Guy 0's clock is this crate's on every frame — the driver never
differed. The crew's parts once, on 5571: `cur_anim 8, cur_time 2,
last_time 1` against `cur_anim 0, cur_time 0, last_time −1`. A `last_time`
of 1 says the clock stood at **1** before the step, and the figure came off
the mirror at **4**. Three taken off — the length.

`Guy::set_anim`'s walk arm only subtracts for the slot **already playing**;
a slot *change* rescales, and the rescale passes the old slot to both
`get_anim_time` calls, so it is `cur_time · t / t` and keeps the clock. One
call can therefore never produce a 1. **There are two.**
`Unit::move_step` asks every guy for `CHAR_WALK` immediately before
`set_new_location` — `:304` on the partial step, `:355` on the snap, both
past the turn-in-place arms that `return 1` — and `Guy::move` asks again in
the body follow. The first is the change, `SLOG → WALK`, clock kept at 4;
the second is the same slot and takes 3 off.

**`docs/ANIM.md` §4.8 had cited that call and the implementation never made
it.** It is quoted there, by address, in a sentence about the fishing
boat's turn: *"which a sailing boat's is, from `Unit::move_step:304`'s
`set_anim(CHAR_WALK, 0, 1)`"*. The reading was right; nothing had ever run
it. That is the third time this shape has cost a session, and it is the
argument for the widening rule rather than for more reading: the field was
in a record nothing compared.

**What it is worth beyond the item.** `run73_s_window_clocks_are_the_
original_s` puts all **4,869** `GUY` fields of the window against this
crate's — `cur_anim`, `cur_time`, `end_time`, `last_time`, `gpiece`,
`stopped`, position and angle — with no unit of the dump unmatched, and
none of them parts. Made to fail on purpose first: without the call it
fails on the window's **first** frame and on the **human's** units, not
only the caravan. Every guy that walks had been carrying the wrong clock;
the word only saw it where a clock wrapped, because the extra request costs
no draw.

**What is left.** 5573 is a **road**: the caravan's own
`Caravan::build_road`, 1,761 `PathFinder::calc_road_cost` draws here
against 1,535. The oracle for it was taken by the same run — the `callwin`
over `[5563, 5581]` carries `astar_caravan_road`, `valid_roadcoord` and
`calc_road_cost` node for node, which is run62/run64/run72's instrument —
so the successor's evidence was on disk before the successor existed.

## 2026-09-03 — item 206: the road a farm takes away (Great Lakes 5573 → 5786, Opus)

**Booked as a search and settled as a footprint.** The item said run53's
frame 5573 spends 1,754 `PathFinder::calc_road_cost` draws where the
original spends 1,528 — the caravan `1/23`'s `Caravan::build_road`, the
last frame of its budget search — and named the instrument: run73's
`callwin` over `[5563, 5581]` already had `astar_caravan_road`,
`valid_roadcoord` and `calc_road_cost` on disk, so the search was readable
node for node the way run62's, run64's and run72's were. Nothing new was
captured. The whole session ran off a file taken the day before for a
different question.

**The search is eight frames, not one.** `astar_caravan_road` answers −1 on
5566 through 5572 — seven budgets of `0xc80`, each re-entered by
`find_road_restore` next frame — and 1 on 5573. So the first thing the
oracle said is that the frame the word named was the *seventh* resumption,
and asking only about 5573 would have been asking about the wrong frame:
5566 through 5571 are node for node identical, **5572 parts at node
2,170**, and by 5573 the two searches are in different parts of the map
altogether. A count is not a sequence, again.

Node 2,170 of 5572 is the diagonal from tile `(216, 123)` to `(217, 124)`.
This crate priced it; the original never asked. `valid_roadcoord`'s
occupied arm is the reason (`docs/ROADS.md` §5.1): a tile carrying a
building's `mask & 3 == 3` is refused unless it is **road**, or unless it
belongs to a city endpoint. `(217, 124)` was road here and plain ground
there — so one stray tile of tarmac opened a door the original keeps shut,
and every node after it was somebody else's.

**And the tile was already on the record.** It is one of the three
`docs/ROADS.md` §9.4 named a month's worth of sessions ago and could not
explain: run72's world at 4802 had 33 tile-mask residues, thirty of them
bit `0x4` alone, and `(217, 124)`, `(218, 124)` and `(219, 124)` road here
and not there. The residue had been pinned as a count that only falls, with
no theory. It was 770 frames upstream of the word the whole time.

**Where it came from.** A watch on the three tiles across 4,810 frames:
road on **1104**, `PLACED` on **3376**, `OBJECT_BUILDING` on **3465** with
the road bit still set. The building is player 1's **Farm** `o 2012`,
4 × 4, corner `(217, 121)`. So the road was laid by somebody else's plan
long before, the farm went up on top of it, and the original's world has no
road under the farm.

**`BuildType::mask_me@006312a0` writes roads, and nothing here knew it.**
Its footprint loop, with `param_4` set:

```
mask &= ~0x40; mask &= ~0x80; mask |= 3
if not is_city and (is_gather_type or FLAGS e) and not is(UNIVERSITY):
    set_road_at(x, y, 0, 0, 0)
if template[x_size · v + u] == 1: set_blocked_at(x, y, param_4)
else:
    set_blocked_at(x, y, 0)
    if is_city or connects_to_roads: set_road_at(x, y, 1, 0, 0)
```

`connects_to_roads` **is** `(is_gather_type or FLAGS e) ? is(UNIVERSITY) :
true`, so the two gates are one predicate read from opposite sides: a
footprint tile either loses its road or gains one, and which is a property
of the type. `docs/CITIES.md` §3.6 had the *laying* arm written down since
August and the implementation never made it; the *removal* arm was in
neither.

Both calls pass `param_5 = 0`, so both go through the mesh door — the
removal through `Roads::road_cleared@008955d0`, which queues the tile with
`added = 0` and runs `add_roads`. That half mattered: with the mask bit
cleared and no `road_cleared`, run72's own invariant caught it at once —
**three tiles that are not roads and still hold a
`RoadElementCandidate`** — which is exactly the leak that would let
`get_orthog_connects` answer for a road that is not there. The mesh is 126
elements at 4802 no longer; it is 123, and the guard says so.

**What it is worth.** All eight frames of run73's search are the
original's node for node — tile, direction and price, 22,145 of them, the
arrival frame at 1,528 apiece. The world at 5565 has **no surface
difference anywhere on the map**: 921,600 heights, every cell owner, and
every tile mask but 45 that differ by `Wall::mark_behind_tiles`' `0x4`
alone — which is now a homogeneous residue rather than one with three
unexplained members in it, and item 203 is the poorer for it in the good
way. Great Lakes' word runs **5573 → 5786**, by draw and by sequence. East
Indies is unmoved at 6739.

**The lesson is the one the working agreement already states, and this is
its third witness.** The finding was not in the mechanic the item named. It
was in a **pinned residue with no theory**, sitting in a test that had been
passing for a day, 770 frames upstream. Both of §9.5's arms were reachable
from a document this project wrote in August; neither had been built. Prose
that cites an address correctly is not an implementation, and a residue
nobody can explain is a queue item wearing a passing test.

**What 5786 is.** `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
Unit::move_step+0x389` — a third draw the original spends and this crate
does not, on a guy turning inside a move step. Two draws here against
three. It is `Guy::do_turn`, and it is the same family as item 205's, one
call site over.

## 2026-09-04 — item 207: the rare the harness never offered (Great Lakes 5786 → 6080, Opus)

**Booked as a turn, settled as a destination.** The word was 5786 and the
draw was one `Guy::set_anim+0x97a < Guy::do_turn+0x4a <
Unit::move_step+0x389` — `move_step`'s **far** turn-in-place arm, whose
idle roll is only ever paid by a guy that carries `guy_flags & 8` and has
no `CHAR_TURN_RIGHT` in its packet. It is the **only turn draw either side
spends in the whole 5,800 frames**, and that rarity is what named the unit
before anything was captured: of the six units moving on 6080's predecessor
frame, exactly one type **packs**, and `docs/ANIM.md` §4.8's own table has
the row — `MERCHANT` packs, names no turn, carries the bit anyway. The AI's
Merchant `1/24`, born on 5753.

**What the reading got right, and where it stopped.** Off the disassembly:
`005fb24c` and `005fb256` both jump forward to `5fb2d2`, so `+0x3b6` is the
near arm and `+0x389` the far one — the assignment §4.8 already had, now
read off the listing rather than the decompiler's line order. The far arm
wants 45° of residual turn and a Manhattan distance past two tiles (the
Merchant is land and not a `VEHICLE`, so the 80° branch is not its). This
crate's Merchant pops its world-grid waypoint `(44088, 17208)` on 5781 —
`manh` 377 against that node's `tolerance` 384 — swings 49.25° and is left
owing **43.93°**, one degree and change under the gate. Two readings fitted
that and no capture on disk separated them, so a capture was booked.

**run74 is the cheap window, and it is the run this project should have
been taking all along.** The question was a position, a facing and a path
stack, all of which `UNITS=3` writes, so `frame_window` narrowed run33's
ordinary detail to `[5700, 5800)` rather than turning `DUMP_ALL` on: three
minutes and 31 MB, against run73's thirty-three and 1.05 GB for a window
six times shorter. 5,801 frames, zero differing against run53's trace.

**And the answer was not the turn at all.** The two merchants are walking
to **different rares** — `MOVE_TO (40344, 14232)` on seven nodes there,
`(31800, 21816)` on sixteen here. `Unit::think_merchant` scores
`LeaderData::new_rares` at `200 − 10 · position`, so with a flat region
bonus the list's *order* is the whole pick; the original's list held three
goods and this crate's two, and the missing one was at the front.

**The cause is in the harness, and the document had already written the
sentence nobody acted on.** `Sim::reveal_fog` has exactly one caller —
`World::set_seen` answering that `seen2` **changed** — so a good is offered
to a leader once for the life of a game. `rondata::diff::build_sim`
*installs* the dump's `seen2` grid instead of walking the sweeps that
produced it, so every offer the original had made before the block was
written was skipped, and skipped for good: those cells now read as seen and
`set_seen` can never answer true for them again.
`docs/ECONOMY.md`'s "The rares a leader has seen" has said "a rare under
the **start** fog is recorded during `Setup`, before frame 0" since
2026-09-02. `Sim::seed_new_rares_from_fog` replays those reveals once, over
the installed grid, after the goods and the leaders' `human` bits are in —
the three things `Leader::new_rare` reads.

**What it is worth.** Great Lakes' word **5786 → 6080**; East Indies
unmoved at 6739. run74's hundred blocks carry **no order, path or angle
disagreement at all**, and one unit off position on the closing block
alone. It also gives the merchant's score its **third rare** — East Indies'
AI has only ever had two, so `docs/MERCHANT.md` §6 had the `−10` step down
as reading-only; the ordering it encodes now has an oracle, though the
step's size still does not.

**The lesson, and it is a new one.** The last three items were the working
agreement's "the finding is not in the mechanic the item named". This one
is a level up: **the finding was not in the simulation.** A harness that
starts from a snapshot owes the simulation every irreversible event the
snapshot is downstream of, and a once-per-game offer gated on a *change* is
exactly that shape — install the state and the event is gone, silently, for
the rest of the run. Two things follow. The general one: grep for the other
once-per-game gates a dump install can swallow. The narrow one, which is
now `docs/MERCHANT.md` §7: the replay walks the fog grid row-major where
the original walked its sweeps, so two rares first seen in the *same* start
reveal would be ranked differently — a snapshot cannot carry that back, and
no capture on disk has a leader with two.

**What 6080 is.** The AI's scout `1/0` spends **41 draws** the original
does not — twelve `Unit::think_scout+0x436`, six `+0x458`, twenty-one
`+0x64c` and two `Unit::do_idle+0x7d` — while the original's frame is birds
and farms alone. It re-thinks an explore where the original's scout does
not, and `docs/SCOUT.md` §3 is the whole of it.

## 2026-09-04 — item 208: the river the scout wades (Great Lakes 6080 → 6151, Opus)

**Booked as a scout, settled as a speed.** The word was 6080 and the frame
was 41 draws the original spends none of: the AI scout `1/0` arrives at its
explore target, goes idle, and runs the whole of `Unit::think_scout` — 12
`+0x436`, 6 `+0x458`, 21 `+0x64c`, two `do_idle+0x7d` — where the
original's frame is birds and farms alone. `docs/SCOUT.md` §3 was the
obvious place to look and it was the wrong one.

**The trace ruled the mechanic out before anything was captured.** The
original's `think_scout` frames run …, 4125, 5851, **6151**, and on 5851
both sides spend the same 11 `+0x436`, 11 `+0x458` and 5 `+0x64c`. The
gate is `idle == 1` or `(o + frame) & 31 == 0`, and neither 5851 nor 6151
is a multiple of 32 with `o = 0`, so both are **arrival** frames. Same
order, same frame, arrival seventy-one frames apart: the divergence is the
walk, not the decision.

**And a walk is invisible.** Walking straight spends no draws; the scout's
type does not pack, so even its turns are free (`docs/ANIM.md` §4.8); and
run53's dump is checksums with all 179 `UNITDATA` records in the start
block. Nothing on disk could place either scout on any frame between 5852
and 6150. So run74's cheap recipe, with the window moved six hundred frames
on and widened to `[5845, 6160)`: five minutes, 75 MB, 316 blocks, 6,161
frames zero-differing against run53's trace.

**Both scouts take the same eight-node path to the same cell.** The capture
prints it node for node identical on 5852, both push the same detour node
`(36780, 29868)` on the same frame, and the positions agree to the frame
until **5949**. There the original's step goes from 34 to **17** and stays
there for 141 frames.

**The field beside it is `z_internal`, and it is the whole answer.** 14 on
5948, **0** on every frame from 5949 to 6089, 17 on 6090 — and the step is
34 outside that span and 17 inside it, on a `myspeed` of 34 throughout.
That is `UnitData::get_speed@00608720`'s land arm: a unit standing on a
**tile** carrying `0x800` walks at half speed while its own `z` is not
above zero. `docs/MOVEMENT.md` has had the sentence since the mechanic was
first read; `crates/sim` had **none of that function's third layer** —
`Sim::get_speed` answered the cached aura speed for everything that is not
an animal, and `movement::group_capped` had sat written and uncalled since
it was landed.

**The trap in it was the decompiler, and the listing cost a minute.** The
decompilation of the test is `(z ^ 0x63637) > 0` skipping the halving,
which reads as a predicate and would mean `z < 0` — and the capture halves
at `z == 0`. The same constant is on `x_internal` and `y_internal` in the
`+0x2f` wrapper `UnitData::get_speed@006086f0`, which only forwards them,
and on all three in `SubObjectData::log_data`, which only prints them.
`00608705` is `xorl $0x63637, %eax` **before the push**: the three
coordinates are stored **obfuscated**, every reader decodes them, and the
predicate is `z > 0`. `docs/audit/README.md`'s standing lesson is a
constant inside an array index; this is its sibling one level out — a
constant inside a *comparison* is a predicate only if the field is stored
the way it reads.

**And `0x800` finally has a name from the symbols.** `docs/MOVEMENT.md`
carried "what world tile flag `0x800` is — forest, swamp or shallow water
are the obvious candidates" as an open question for a fortnight.
`WorldData::is_river@0046d390` is `tdata[tile_xs·y + x].mask & 0x800` and
nothing else. It is a river, and `crate::world::tile::RIVER` had guessed
right without evidence.

**What it is worth.** Great Lakes' word **6080 → 6151**; East Indies
unmoved at 6739. run75's 316 blocks — 10,014 unit fields, 9,981 order and
path fields, 19,962 angles — carry **no unit off the original's point
before the word**, the scout included on every one of the 315 frames of its
walk. The frame past it is one figure draw: `Guy::set_anim+0x97a <
Guy::move+0x19f` here against `< Guy::inc_time+0x271` there, ours 39 to
their 38.

**Two things stay open in the mechanic, and both are inputs rather than
readings.** `unit_masks & 0x10` — `target_opportunity`'s "moving in contact
with a target" — is set and cleared inside one frame, so no dump can ever
print it and this crate's `target_opportunity` is the retaliation alone.
`has_general(0, 0x162)`'s siege doubling has no general in any capture. The
group cap has its arithmetic and no group speed to feed it. The order scale
(`ATTACK` ×9/8, `GUARD` ×10/8 for a computer) is now in and moves nothing:
no capture on disk has a unit under either action while it walks.

**The lesson.** 207's was "the finding was not in the simulation"; this
one's is nearer home. **A function this crate answers with one cached field
is a function nobody has read the third layer of** — `Sim::get_speed`
looked implemented, had a doc comment about all three layers, and returned
`movement.speed`. The tell was available for free: `movement::group_capped`
existed, was tested, and had no caller. A public helper with no live caller
is a layer somebody meant to write.


## 2026-09-04 — item 210: the crew the cast teleports (Great Lakes 6151 → 6463, Opus)

Booked as a body animation and it was one, but not the one the queue
named. Great Lakes' word parted at 6151 on a single draw: the AI Merchant
`1/24` spent a `Guy::set_anim+0x97a < Guy::move+0x19f` — the arrival stand
— that the original spends none of, 39 draws to their 38, with every other
draw of the frame agreeing including `1/0`'s whole second `think_scout`.

**The queue said no capture was needed and it was half right.** run75's
window covers 6151, but at `GUYS=1` its `GUY` blocks carry position and
angle only — no clock. What settled it was the position, and it settled it
in one line. The two sides walk the crew figure together from 6141 to 6144;
on the dump's block 6145 the original's figure is at `(40706, 14716)` with
`angle 1605566464`, its driver's to the digit, and this crate's is at
`(40631, 14634)` still jogging. It **teleported**, four frames before it
could have walked there.

**`Unit::do_cast` re-seats a rare collector on the first frame of an
unpack.** `005eca9c`–`005ecb0d`, between the packed test and the animation:
`good_merchant_spot` on the caster's own tile or the order dies, then
`Unit::set_new_location(unit-cell centre, 1, 1)`, then `Unit::set_angle(guy
0's angle)`. It has been a stated seam in `docs/ORDERS.md` §6.9 since the
craft table was read, and the reason it looked harmless is that the point
is almost always the one the merchant already holds — it walks to a tile
corner and stops on a 48-grid centre. What the call is *for* is its third
argument: `Guy::set_new_location`'s crew loop **puts** every tracked figure
on its offset instead of leaving it to chase.

**And this crate's `set_new_location` opened with `if from == to { return
true }`.** `005f8d20` has no such return. An unchanged point makes both of
its cell tests false and falls straight through to `LAB_005f9033`, which
writes the coordinates back and then runs the guy half anyway. So the
"no-op" was the whole mechanic, and the early return had been quietly
eating `resolve_unit_collision`'s crew snap too.

**What the extra draw actually cost, and how narrow it was.** A tracked
crew figure jogs, so §4's arrival test — `cur_anim == 8`, the *slot* —
fails for it. It stops failing when the leader stops: the figure's body
reaches `des` with its angle short of `des_angle`, `Guy::move`'s standing
arm puts it back on `CHAR_WALK`, and `set_anim` re-resolves the walk from a
decaying `avg_speed`. The steady figure averages **28** against a base of
23 — `280 > 253`, a jog. Its last step onto `des` is partial, so
`last_speed` is `vector_dist(12, 13) = 18` and not the full 31, and
`(28·3 + 18) / 4` is **25**: `250 < 253`, a walk, by three parts in a
thousand. One unit of `avg_speed` either way and the frame would have cost
nothing.

**The widening came first and paid for itself before the item did.**
`GuyData`'s `last_speed` and `avg_speed` were not parsed at all, and
`des_x`, `des_y` and `des_angle` were parsed and compared on one capture.
Widening the three `DUMP_ALL`/`GUYS=4` clock comparisons — run64, run67,
run73 — from nine and fifteen fields to fifteen and seventeen **failed on
its first run**, on `des_angle`, and the defect was the new comparison's
own: a trackless crew figure carries guy 0's **facing**, not its heading,
which is what `Sim::guys_follow`'s `settled || g >= SQUAD_SIZE` already
models. run67 now compares **32,742** fields where it compared 28,890, and
the tracked merchant crew's speed pair — the arithmetic the paragraph above
turns on — is checked against the original's own numbers over sixty blocks.

**What it is worth.** Great Lakes' word **6151 → 6463**, by draw and by
sequence; East Indies unmoved at 6739. The frame past it is one
`Guy::set_anim+0x97a < Guy::do_turn+0x4a < Guy::turn_towards+0x69` — the
turning stand of §4.8 — spent by a gaia unit late in the loop, where this
crate spends none.

**Two smaller things came out of it.** run75's position guard had been
excluding the file's truncated quit block by accident, because the word was
ten frames below it; it excludes it on purpose now. And `fish.rs`'s fixture
had a cell grid of ocean over a **tile** grid of zeroes, which reads as dry
land — invisible until `do_cast` started asking `calc_gather` about it.

**The lesson.** 208's was that a function answered with one cached field
is a function whose third layer nobody read. This one is the same shape one
level out: **an early return that skips a function's tail is a claim about
the tail**, and `if from == to { return }` claimed that moving a unit to
where it already stands does nothing. The original's own control flow is
the only thing that can settle that, and it took one `grep` of the
decompile to see there was no such return. Second: the seam list is not
decoration. "`is_rare_collector`'s merchant re-seat on the first frame of
an unpack" was written down, in the right file, a session before it cost
312 frames.

## 2026-09-04 — item 212: the driver that turns for free (Great Lakes 6463 → 6582, East Indies 6739 → 7448, Opus)

Great Lakes' word was 6463 and the frame was one draw: the original spends a
`Guy::set_anim+0x97a < Guy::do_turn+0x4a < Guy::turn_towards+0x69` — §4.8's
turning stand — that this crate spends none of, 14 draws against 15, every
other draw of the frame agreeing and in order.

**The queue booked it as gaia and the rotation says otherwise.** The extra
draw sits at index 1, between the chicken `9/13`'s `think_farm_animal` and
the first `Unit::inc_time` wrap, and the handoff read that as "a gaia unit
late in the loop". But `Objects::process_all` rotates the owners by
`(frame + i) % 10`, and 6463 % 10 is 3 — so the visit order is 3…9, then 0,
1, 2, and *player 1 comes after gaia*. That reopened the whole player list,
and the trace closed it again in one line: the site fires **six times in
24,000 frames**, at 6123, 6463, 6574, 7147, 12215 and 13373, and this crate
already spends 6123 and 6574 — the Merchants `1/24` and `1/25` arriving at
their destinations, where `do_move`'s final arm sets the heading and
`Guy::move` turns to it. A site that rare is a `guy_flags & 8` site, and in
these two games the only type carrying the bit is the one that **packs**.

**No capture was needed, and the reason is worth keeping.** run53's dump is
checksums, so the frame had nothing in it to read — but
`rngcmp.py rontrace-run53.log rontrace-run18b.log` is **6,601 frames, zero
differing**, and run18b is a `[6374, 6590)` window at `UNITS=3 GUYS=1`,
captured on 2026-08-25 for the make list and never asked this question. Its
blocks carry every guy's `x`, `y` and `angle`, which is exactly the record.

**What they say.** The Merchant `1/26`'s *second* figure sits at
`(39471, 18769)` in the block before the frame and in the block after — it
does not move — and its angle goes `-1153564672 → -1605566464`, its
leader's, in that one frame. Guy 0 is settled and standing; the crew figure
has just caught up and swallows the whole turn it was owed. `crates/sim` had
that figure's position, its destination and both angles right to the unit,
frame for frame, and simply never asked for the animation.

**The mechanic.** `Guy::process → Guy::move` runs for **every** guy, so a
*tracked* crew figure standing on its offset with `des_angle != angle`
reaches `Guy::move:109`'s own `turn_towards → do_turn(…, 1)` in its own
right, and `do_turn`'s override answers `guy_flags & 8` — which
`Guy::init_real@005db6b0:215` sets for every guy of a packing type, whatever
its art says. §4.8 had read the override as guy 0's, on the true observation
that `do_turn@005d97a0:37` recurses only into the crew with *no* track; the
recursion was never the point. `Sim::process_follower` marks the site and
calls the new per-guy `Sim::guy_do_turn_anim`, which is the loop body
`do_turn_anim` already had.

**What it moved.** Great Lakes **6463 → 6582**, by draw and by sequence, and
East Indies **6739 → 7448** with it — the first item in a while to move both
maps, because the mechanic is a merchant's and both games have merchants.
8,597 → 8,783 frames on the original's count and 6,800 → 6,975 draw for
draw. The frames past them: Great Lakes 6582 is a third `Leader::make_stuff`
draw, `+0x63d`, the bought slot's own roll — run18b's window covers it and
`run18b_and_run19_s_make_list_windows_replay_slot_for_slot` already names
that frame's expiry. East Indies 7448 is two `Unit::move_step+0x823`, the
blocked stand: two units the original holds still and this crate steps.

**The lesson.** Twice now the handoff's own reading of a frame has been the
thing that cost the time — 210's seam list was right and read late, and this
one's "gaia, index above 13" was wrong because the owner rotation was not
re-derived for *that* frame number. A frame's position in the draw stream
names a *visit order*, not an owner. And the second half: **grep the disk
before booking a capture** paid its largest dividend yet — a window captured
ten days earlier for an unrelated mechanic answered a question that looked
like it needed a new run, because `rngcmp` makes "is this the same game" a
one-second question.


## 2026-09-04 — item 213: the number every producer multiplies (Great Lakes 6582 → 6612, Opus)

**The frame.** 6582 is the AI's `make_stuff`, and the original spends three
draws there where this crate spent two. The two are `+0x221`, the head's
expiry over the Temple at slots 0 and 8, and they agreed to the roll. The
third is `+0x63d` — the expiry over the slot `make_stuff` has just bought,
which is the citizen at slot 5. This crate never bought it, because step 6
skips a slot whose `val` is zero and this crate's citizen was offered at
**0** where the original's was **714**.

**The field.** `create_units@006c40a0:289` is one line —
`base = (pop × 1000) / max(1, city_num)` — and `research_techs@006c6ba0`
is the same line with `× 200`. `pop` is `LeaderData +0x95c`, and **nothing
in this crate ever wrote it**. So the base was zero, and with it every
value either producer could compute, for the whole game. `docs/AI.md` §2.3
called it "kept by the unit lifecycle"; it is the *city* lifecycle, and it
was being kept by nobody.

`CityData::get_pop_value@00738450` is the value of one city — 1 for a Small
City, 3 for a Large, 5 for a Major or the Forbidden City, which is
`2 · city_level − 1` — and every one of the five writers of `pop`
(`City::init`, `close`, `capture`, `check_upgrade`, and `Build::finished`'s
upgrade delta) adds or subtracts exactly that for exactly one city. So the
incremental sum the original keeps is a recount over the leader's live
cities, and all five are already call sites of `Sim::sync_pop_cities`.
`reg_pop` (`+0xe62`) is the same sum per region.

**Where the hour went, and it is the lesson.** With `pop = 2` and
`city_num = 2` the base is 1000, and the multiplier chain the citizen
branch runs over run18b's own dump — `infra_mod 256`, Norwich at
`free 0 busy 11 q 0` against 12 slots, `reg_gatherers 19 < reg_gather_slots
22` — gives **1500**. The dump says 714. An hour went into hunting a
missing multiplier that was not there: the answer is `create_units`' *tail*,
the last thing it does before `make_me`, which divides by
`want + units + queued` — `20 × 1500 / 42 = 714`. This crate had the tail
right all along. **Reading a value's arithmetic backwards from the number
means reading the whole function, not the branch**; the branch was where
the eye went because the branch was where the interesting predicates were.

**What it moved.** Great Lakes **6582 → 6612**, by draw and by sequence.
East Indies did not move (7448), which is what a Great-Lakes-shaped
citizen buy should do. The census widening now carries `pop` and
`reg_pop[home]`, made to fail on purpose first.

**The frame past it.** 6612 is three `Guy::init_real+0x52` and three
`set_anim < Unit::do_idle` — a three-guy unit arriving that this crate has
not queued. It is *not* the citizen bought thirty frames earlier: a citizen
is one guy and its clock is 180 frames here. Its shape is 5564's, `b22`'s
`ty9`, which both sides made. So the question is which queue the original
filled between 6153 and 6612 and this crate did not.


## 2026-09-04 — item 215: the archer a Barracks pays for (Great Lakes 6612 → 6650, Opus)

The item was booked as a queue: "which queue did the original fill between
6153 and 6612?" It was **no queue**, and the disk said so in three greps.

**The first grep was the queues themselves.** run18b is run53's own game
over `[6374, 6589]` with `BUILDQUEUE` on every frame, and over those 216
frames the composition of every queue in the game changes **twice** — the
Library swapping its research entry at 6377, and the citizen the AI buys at
6582. Nothing else. So whatever arrives at 6612 was not queued in the two
hundred frames before it, and a unit's clock is longer than the twenty-two
frames the window leaves. That ruled the whole hypothesis out before a
single line was written.

**The second grep was a clock.** The AI's Barracks `1/2016` is under
construction through the window at a flat hundred a frame:
`job_counter` 38700 at label 6581, 39600 at 6589, against `constr_time`
42000. Twenty-four frames left, and 6588 + 24 is **6612** exactly.

**The third was the trace's own coverage.** `tools/trace/` arms each
function once, so `report.py coverage 6612` lists the functions the original
entered for the *first time in 24,000 frames* on that frame: fifteen of
them, and they are `Army::add_unit`, `Unit::add_to_army`,
`Unit::think_attack`, `Unit::find_melee_target`, `Armies::find_local_army`.
The AI's **first military unit**, on the frame its first Barracks finishes.

**What pays it is `Build::activate`'s high-water block**, six hundred lines
into that function and a single row of `docs/CITIES.md` §4 until today:
`n = num_buildings[type] + get_buildings(to)`, and when that beats
`high_buildings[get_base_type(type)]` a counted, uncaptured building pays
the owner's nation its free units. run53's AI is `tribe 11` — the British —
and the British arm is `BRITISH_AGE_FOR_1/2/3_ARCHER` over
`min(epoch[0], ages)`, with `BRITISH_AGE_FOR_1_ARCHER = 0`: one Bowmen at
Ancient. The row called this "first-of-its-kind"; it is a high-water mark,
so **every** Barracks pays and only a *replacement* for one that died does
not. §4.3 now carries all sixteen arms, their counts and their three
constants apiece, and `cargo run -p rondata` re-derives all twenty-five
from `rules.xml`.

**And then one archer was three draws.** The first implementation trained a
Bowmen and spent one `Guy::init_real` where the original spends three. The
guy count is `crew_size + squad_size` (`Unit::init@00612100:508`) and
`squad_size` is **written 1 by `UnitType::init@0061ab50:723` and never
written again** — so no unit in the game has three figures that way. What
has three is `Objects::init_unit@0065e0c0:34`, which reads `uber_size` and
**loops**: a Bowmen is `UBER_SIZE 3, CREW_SIZE 0`, and the squad is three
one-figure *objects* threaded `o_up`/`o_down` as a list. run17's frame 1301
prints it: `o 6` with `o_up −1, o_down 7`, `o 7` with `6, 8`, `o 8` with
`7, −1`. Only the head is counted — `init_unit` hands every unit with an
`o_up` straight back to `track_unit_type(·, −1, ·)` — so a squad stays one
unit and one population everywhere else. That is half of item 175, which
had been sitting in the backlog as a note about `curr_uber_size`.

**What it moved.** Great Lakes **6612 → 6650**, by draw and by sequence.
East Indies unmoved at 7448, and the other 203 checks unmoved with it.
`a_british_barracks_pays_one_bowmen_as_three_chained_units` pins the chain,
the single count, the second Barracks paying again and the rebuild paying
nothing; the two ladder helpers are pinned against the shipped constants.

**The frame past it** is the squad's first order: one
`Unit::do_move+0xe84 < Unit::do_group_move+0x148 <
Unit::do_group_attack_to+0x11`. The original puts the newborn archer in a
group and marches it; this crate leaves it standing. That is item 217.

**The lesson, and it is the working agreement's own.** Grep the dump before
booking a reading, and grep the disk before booking a capture — but the
third source here was neither: the *trace's first-entry list* named the
mechanic in one line where a reading of `Build::activate`'s two thousand
lines would have taken an hour to reach the same row. A coverage list is an
oracle for "what kind of thing happened", and it had never been used that
way.

## 2026-09-04 — item 217: the order a squad marches under (Great Lakes 6650 → 6736, Opus)

The item asked who gives the newborn archer squad its group attack-to on
frame 6650. Both halves of the answer turned out to be mechanics, and the
second was the bigger one.

**Who gives it.** `Army::do_forming` — but the army had to have the squad
first, and `Unit::add_to_army` has six callers. The trace named the one
that matters from the other side: `Unit::think_attack@005f5a80` is entered
for the *first time in 24,000 frames* on 6612 of run53 and on 10187 of
run54, and `Unit::add_to_army` on 6612 and 5823. Its **head** — ahead of
its own target search — is the site, and no capture had ever reached it.

The head is five gates, and the listing settles two things the decompiler
does not. The weak-region flag the decompile prints as a sixth gate is
**dead**: every path through the block leaves `find_melee_target`'s
argument at −1, so the whole `range` variable exists to gate the
`add_to_army` call and nothing else. And the `find_city` whose answer looks
like it feeds the own-territory flag is a **dead call**: `local_10` is
written 1 at `5f5ce8`, before the call at `5f5cef`, and `eax` is never
read. What the head actually turns away, once a unit is AI-driven and
military, is a **damaged foot or mounted unit standing inside one of its
own cities' radius** — it stays to heal.

**Which units enter `think_attack` at all** cost a run of its own:
`type.attack != 0` **and** `role & 0x10000`. Taking only the first put
run53's woodcutters in an army on frame **307** and dropped the word by six
thousand frames — the same mistake, in a different function, that item 68
had already made once with the think tail.

With the join, the squad lands in leader 1's **army 1** — every army being
empty, `find_local_army`'s `90,000,000` and its `<=` hand the last valid
slot the win — whose tick is `frame ≡ 250 (mod 256)`, which is 6650.

**And then three archers spent three draws where the original spends one.**
`Group::action_siege_attack_to` orders all three out; the original runs
`Unit::do_move` for the group's **leader alone** and steers the followers
off the leader's own position with `move_step`. `docs/ORDERS.md` §8.4's
"N independent moves plus an offset" verdict had stood for two weeks; the
score is what overturned it, and only for an **army's** group. A player's
selection has no persistent group here — and that is not a gap in the port,
it is `do_group_move`'s own first line: `if (this->group == −1)
ungroup_move_order(...)`, which turns the order straight back into a plain
move. So the seam and the mechanic now sit on either side of a line the
original itself draws.

`GroupMoveOrder` is a real order now (§15), and the listing corrected the
outline §8.3 was written from in four places: the follower's give-up test
is `d_goal − d_slot ≤ 0x60 || d_goal ≤ 0x180` against the **goal**, not
"the slot reached"; the refresh trigger measures `0x5ff` from the order's
**own destination**, not from the leader's cell; the walk-straight window
is a **third** of a turn against the bearing to the **goal**, not 60° off
the leader's heading; and `Group::update_positions` rotates the slot table
by the bearing to the leader's **waypoint** and not by its heading
(`713844` loads the heading as the default and `7138e1` replaces it).

**Two more readings moved the number after the mechanic landed.** The
rotation angle above was worth nothing to the word but 164 frames of
draw-for-draw agreement. What was worth the word was `docs/COLLISION.md`
§4.3's **group** arm — two members of one group walking a
`GROUP_MOVE`/`GROUP_ATTACK_TO` are a *nudge*, not a collision — which had
been a stated seam for want of a `UnitData::group` to ask about. This crate
has one now: an army's members are its group. Without it the squad stood
blocked on its own leader at 6716, and with it the word ran to 6736.

**What it moved.** Great Lakes **6650 → 6736**, by draw and by sequence,
and 6939 → 7148 of 24,000 frames draw for draw. East Indies unmoved at
7448, and the other 203 checks unmoved with them. Four tests, each made to
fail first: the group-order gate, the follower that tracks the leader
rather than the destination, the ungroup that walks a whole squad, the
third-of-a-turn window, and the collision arm.

**The lesson.** A seam is a *claim about the port*, and this one had two
loads on it: "we do not model group moves" was really "we have no group
pool", which was true of a human's selection and false of an army's. The
seam had been written when the second half did not exist yet, and nothing
re-read it when it did. The same sentence appeared in three documents; two
of them are now split down the line the original draws.

## 2026-09-04 — item 218: the upgrade a nation is given (Great Lakes 6736 → 6779, Opus)

The queue booked 6736 as "another three-object unit arriving" — 6612's
shape, three `Guy::init_real` and three more draws behind them. It was not
a unit arriving. It was three units **converting**, and the trace's own
call stack said so in one line:

```
Guy::init_real+0x52 < Unit::set_type+0x40c < Leader::gain_tech+0x1071
```

`set_type`'s first guy loop clamps `guy_mark` to the new type's
`squad_size`, and `squad_size` is written 1 for every type in the game, so
that loop runs **at most once per call**. Three draws at `+0x40c` is
therefore three objects, not one unit with three figures — the same
arithmetic that made item 215's squad three objects, read the other way
round.

**Which upgrade.** Frame 6736 is the frame the AI reaches the Classical
Age, and its leader is British. `gain_tech`'s step 13 has a
`BRITISH_ARCHER_UPGRADES` block that hands a British player every Barracks
unit of the Bowmen line whose prerequisites the gain has just completed —
**Archers** — and step 7's object half then converts the three standing
Bowmen in place.

**Both halves were half-built, and in the same way.** `free_rules` had a
`Shape`, a `Gate`, a cascade and a unit test, and an **empty table**:
nothing in the loader ever built one. `Gained::UnitUpgrade` had a
producer, a doc comment naming exactly what it was for, and no consumer.
Each was a mechanism waiting for the other, and neither had ever fired.

**What the listing settled that the decompiler could not.** The candidate
predicates are pushes the decompiler drops: `0xaa` Bowmen, `0x45` Scout,
`0x109` Catapult, `0x99`/`0x84` the heavy-infantry lineages, `0xd1` Light
Horse — and `0x1ab`, which is the Barracks and is compared against the
`where` **column**, not asked as a lineage. Five blocks of §13's
twenty-five are a predicate over unit types like these; the rest name a
run of tech indices whose endpoints are each their own reading, and no
capture reaches one, so they are item 220 rather than a guess.

**The test that corrected the document.** `docs/TECH.md` §13's row for the
German light cavalry said its candidates "do not exist"; the first
implementation asserted the opposite, because `0xd1` is Light Horse and
Light Horse plainly exists. The test failed, and the row was right for a
reason it did not give: the block asks for **Barracks** units, and a Light
Horse is trained at the Stable. The block is empty because of the `where`,
not the lineage — and the assertion now says which.

**And a detail of `set_type` that shows on the same frame.** It does not
rebuild the guy stack: it keeps every guy the new type still has room for,
body and place intact, and gives each one a fresh `Guy::init_real`. That
leaves `end_time` at zero, so every re-typed guy wraps on its very next
`Unit::inc_time` and rolls again. 6736's six draws are three of each, and
the second three fall out of the first for free.

**What it moved.** Great Lakes **6736 → 6779**, by draw and by sequence.
East Indies unmoved at 7448, and the other 203 checks unmoved with them.
What stands at 6779 is a different subsystem: two draws inside
`strategy_all` on a frame that is nobody's phase — the AI owns Archers now
and asks its production for something else.

**The lesson, and it is a sibling of the last item's.** Item 217 found a
seam with two loads on it; this one found two *mechanisms* with nothing
between them. Both were written by sessions that did the half in front of
them and left the join for later, and in both cases the join was cheap —
one loop and one table — and had been sitting unwritten for weeks because
nothing failed without it. A producer with no consumer is not a seam and
does not read like one: `docs/TECH.md` listed `Unit::set_type` under "what
is not established" and said nothing at all about `free_rules` being
empty. **Where a document names a mechanism, it should say whether
anything calls it.**

## 2026-09-04 — item 221: the nation a leader starts as (Great Lakes 6779 → 6782, Opus)

The word was two draws inside `strategy_all` on a frame that is nobody's
phase, and the queue's guess was that the AI, having just gained Archers,
was asking its production for something else. It was asking for something
else. What it was asking *from* was wrong four thousand frames earlier.

**The step, then the types, then the seeding.** A print of the step
machine over 6770–6790 put 6779 on **step 5**, `upgrade_units`, whose
only draw is one matchup roll per eligible type; a print of the loop named
the two — **Slingers** and **Javelineers**. Both had become eligible when
the AI's Barracks finished at 6612, and both were eligible only because
`Slingers` read `RESEARCHABLE` rather than `AVAILABLE`: a type the leader
could research, with a predecessor chain that owns nothing and offers
nothing, is exactly what the original's `owned != 0 || !avail || siege`
gate lets through.

**A British leader that started the game owning Atl-Atls.** The tech
tree's own dump said why. `Slingers`' `TRIBE_MASK` is `0xfffff4` — bits 0
and 1 cleared, because the Aztecs and the Maya have their own variants —
and `Atl-Atls`, mask `0x1`, was marked researched **and** started, for a
leader whose tribe is 11. `Leader::init` sets the nation first and its
unit arm is `has_preq && tribe_can_type`, so the opening tech set is a
function of the nation; `rondata::diff::build_sim` called `Loaded::sim`,
which calls `Sim::start_techs` for every player, and only afterwards read
the dump's `LEADER` records to call `Sim::set_tribe`. **Every capture ever
built in this harness opened with its leaders' unit bits laid down for
`tribe = 0`.**

The fix is four lines and no new rule: lay the starting position again
once the nations are known. `scene_at` had the same order and takes the
same; it also puts the lobby's `starting_age` in front of the seeding,
which `Loaded::sim` cannot know either. Nothing draws in `start_techs` and
the players are still empty when it runs, so it is a re-lay and not a
replay.

**Three sites named, because the instrument was the second half of the
number.** With the ordering fixed the *count* moved to 6782 and the
*sequence* stopped at 6780 — on a frame where both sides spend the same
eight draws. Every AI draw reached the comparison under the coarse
`strategy_all` mark, so the first producer draw the original also spends
parts the sequence whatever happens. Four addresses, all return addresses
of a `call Random::get` in the listing, now carry the mark that spends
them: `create_units+0x642`, `upgrade_units+0x5a4` and
`create_buildings`' wonder pair `+0xffb`/`+0x1017` (`docs/AI.md` §28).
Naming three of the four lifted the sequence 6780 → 6781 → 6782, where it
meets the count.

**What it moved.** Great Lakes **6779 → 6782**, by draw and by sequence.
East Indies unmoved at 7448, and the other 203 checks unmoved with them —
which is itself the finding's size, since re-seeding every leader's
opening tech set could have moved anything. What stands at 6782 is a
building this crate buys and the original does not: both spend
`make_stuff`'s two `+0x221` expiries over the head's type, and only this
crate goes on to `produce_building` and to the two `+0x63d` expiries over
the slot it bought. The head of the make list agrees; the buy does not
(item 222).

**The lesson.** The wrong thing was not in the mechanic the word pointed
at, and it was not subtle once looked at: a British leader owned an Aztec
unit from frame 0, in a field a `#[test]` could have read at any time in
the last month. What hid it is that the seeding is *silent* — no
assertion, no dump comparison, and every visible consequence four
thousand frames downstream. `rondata::diff` compares what the original
prints; the original never prints a leader's opening tech set, so nothing
here was ever going to notice. **Where the harness computes a starting
state the dump does not carry, the order it computes it in is a fact
nobody is checking** — and this is the second such ordering (the first
was `diplos`, installed 2026-09-02 after forty-six readers had run
against an all-peace matrix).

## 2026-09-04 — item 222: the list a Barracks joins (Great Lakes 6782 → 6848, Opus)

**One frame, three functions, no reading and no capture.** The word stood
at 6782 on a building this crate bought and the original did not: both
sides spent `make_stuff`'s two `+0x221` expiry draws over the head's type,
and only this crate went on to `produce_building`'s two `+0x1805` jitter
draws and to the two `+0x63d` expiries over the slot it had just bought.
The queue booked it as "an affordability or a `make_this` gate".

**The draws said where it was not.** `make_stuff`'s slot loop runs the
expiry *after* the buy and regardless of whether the producer succeeded,
so a slot the original had reached at all would have cost it at least one
`+0x63d`. Zero of them means the original bought **nothing** from slots
1..10 — which rules out the producer and puts the whole question on the
gate. Printing this crate's list at 6782 gave slot 1 as a **University**,
60 timber and 30 metal, against a bucket of 113 and 127 and a head cost of
31 and 51: `113 ≥ 31 + 60` and `127 ≥ 51 + 30`, so of course it bought. The
gate is right. The **bucket** is not.

**The head is what pays.** Slot 0 is two **Longbowmen** — the British
unique archer, and the AI is British — bought by `make_this(0)` two lines
earlier, which charges. The bucket had not moved. `Leader::produce_unit`'s
military-trainer arm walks **`mil_trainers`**, `LeaderData+0x6e50`, and a
one-line probe said it was empty. `grep` said it had always been empty:
`Leader::new` sets `Vec::new()` and the only pushes in the crate are in
two unit tests. **The AI in this harness had never queued a military unit
in 24,000 frames**, on either map, since the AI landed a fortnight ago.

**Two writers, and they were not where the reading looked.** Neither
`plan_strategy`'s census nor any producer writes the list.
`Wall::increment_stats@00643270` and `Wall::decrement_stats@00642da0` do —
the same pair that keeps `reg_buildings`, which §2.3 step 8 already cited
for that and not for this. Each splits on `is_active`; the active arm
files or removes the trainer, and the removal is
`SimpleArray<int>::remove@00462e70`, which finds the first slot holding
the value and **shifts the tail down**, so the list is in activation order
and stays in it. Four call sites, all in the building lifecycle
(`Wall::activate`, `Build::close`, `add_to_city`/`remove_from_city`,
`Wall::set_type`); this crate implements the first two, beside the dock
registry's own calls, which are the same shape one field over.

**What it moved.** Great Lakes **6782 → 6848**, count and sequence
together. East Indies unmoved at 7448. The original queues both
Longbowmen and pays 62 timber and 102 metal for them, and 51 timber is
not 91, so the University never comes up.

**The lesson, and it is the same one as 221's with a different face.**
The wrong thing was not in the mechanic the word pointed at. 221 was a
starting state nothing checks; this is a **field with no writer** — the
crate had the reader, the test fixture and the doc paragraph, and the
production path that fills it did not exist. `docs/QUEUE.md` has carried
a standing section for exactly this ("Three fields nothing here writes",
items 48, 73, 56) and `mil_trainers` was not in it, because nobody had
grepped for its writers. **A field a producer reads and no lifecycle
writes is silent by construction**: it produces no divergence of its own,
only a resource level four hundred frames later. The grep that finds them
is cheap and is now worth running as a sweep rather than one field at a
time.

**What stands at 6848** is one extra `Guy::set_anim+0x97a <
Unit::move_step+0x823` — the blocked walker's stand, `sim::anim::
SITE_BLOCKED` — a step this crate refuses and the original takes. East
Indies' 7448 is two of the same site with the sign the other way round,
so for the first time both maps' words are the same predicate (item 223).

## 2026-09-04 — item 223: the word at 6848 is a squad's position, not a predicate (Opus)

**Booked as the collision predicate and it is not one.** Both long words
now part on `Guy::set_anim+0x97a < Unit::move_step+0x823` with opposite
signs — Great Lakes 6848 one blocked stand this crate spends and the
original does not, East Indies 7448 two the original spends and this
crate does not — and the queue read the two signs as one predicate to be
explained. Great Lakes' half is now named whole and it is arithmetic
nobody has to explain: **the two blocks overlap.**

The blocked unit is the AI's Archer `1/28`, a follower of the marching
squad `1/27`/`1/28`/`1/29` (`docs/ORDERS.md` §15). On 6848
`Group::update_positions` puts its slot at `(42773, 24617)`, one unit
cell north of where it stands; `dy = −1` takes §4.2's leading edge, the
row `y = 511` is swept from `x = 890`, and `(890, 511)` is a cell of the
standing citizen `1/13`'s block. `will_be_corner` answers NW and
`is_corner` answers 0, so it is hard. Every step of that is forced.

**And the blocker is not in doubt.** `1/13` stands at `(42744, 24504)` —
`ucell_centre(890, 510)` exactly — for the whole neighbourhood of frames,
and run18b's dump has the original's `1/13` on the same point with the
same `angle`. So the question is where the *archers* are, and the trace
answers it from the other side: the original's own first blocked stand in
that neighbourhood is **6860**, and the next 6892. Twelve frames at 26
units a frame is about 1.7 tiles.

**No record on disk can place them.** The three archers are born on 6612;
run18b's `DUMP_ALL` window closes at 6590 and run75, this map's longest,
stops at 6160. A walk spends no draws, so the trace cannot help either.
Stanza **run 76** is booked in `tools/gamelog/captures.txt` — run75's
recipe with `frame_window 6640 6870`, opened before `Army::do_forming`
issues the group order on 6650 so the whole march is on disk.

**The lane was written off twice on the wrong evidence, and it was
neither.** First `cliclick p` from a Claude Code shell answers *"WARNING:
Accessibility privileges not enabled"*, and this session read that as a
permission-shaped failure and wrote it into three documents. It is not the
lane's answer — the capture runs through `~/bin/RonDriver.app`, a fixed
path holding the three grants, and its own probe reported *"synthetic move
landed"* on the first try; the rule already existed in memory (**probe the
lane by launching it**) and was reached for one step late.

Then both launches stalled in `waitwin.sh` anyway: game alive, every wine
thread at 0 % CPU, `wineserver` idle in its select loop, no `gamelog.txt`,
and `tccd`'s log clean. That reads exactly like run74's unanswered
Documents prompt, and it is not that either. One line answered it —
`System Events`, from inside RonDriver's domain, asked for the windows of
the visible wine app:

    windows of wineloader: [Expired Bottle: ron]

**CrossOver's bottle licence has expired.** The dialog blocks the game
before it creates a window, so the whole capture lane is down until a
human renews it, and run 76 could not be taken. `docs/ORACLE.md`, "The
window nobody can see", carries the probe.

The lesson under both is one lesson: *a stall is not a diagnosis, and
neither is a warning printed by a different process.* `sample`, the TCC
log and `cliclick` all had something plausible to say here and all three
were wrong; the window title was right and cost a line. Ask the thing
that is stuck what it is showing, first.

**One reading landed on the way, and it is inert.**
`UnitData::is_corner@0060a040` is not the unit's own block:
it walks the unit's figures `0 .. guy_mark` and returns the first
non-zero `GuyData::is_corner@005de270`, which measures the hit cell
against that figure's `GuyData::x/y`. `UnitData::is_here`, four lines
earlier in `detect_unit_collision`, reads the unit's own `x_internal` —
so the original mixes two frames of reference inside one predicate.
Because the first non-zero wins and guy 0 usually stands on its unit, the
effect is one-directional: a crew figure can only turn a hard collision
**soft**. `Sim::guy_corner` and
`the_corner_rule_reads_the_blocker_s_figures_and_not_the_blocker`, which
was made to fail against the old form before it was kept. Neither long
word moved — no capture yet has a blocker whose figures disagree with it
— so it is a reading-only claim and `docs/COLLISION.md` §8 says so.

**The lesson.** "Both signs are one predicate" was a plausible reading of
two numbers and it cost the session's direction: the diff that settles
which of *predicate* or *position* is wrong is the one that prints the
proposal and the blocker, and it took twenty minutes to write and one run
to answer. Print the two operands before theorising about the operator.
The residue is that the queue's own framing of an item is a hypothesis
like any other, and the item's first job is to falsify it.

## 2026-09-04 — the lane, and what CrossOver was actually for (Opus)

**The trial ran out mid-session, so the question stopped being item 223's
and became the project's.** An oracle behind a renewing licence is a
dependency on somebody else's business decisions, on a project whose stated
prior art is OpenTTD. So: what does CrossOver give us, and is any of it
ours to lose?

**Less than it looked.** Two things, and neither is CodeWeavers'. The
wow64 packaging that lets the 32-bit `riseofnations.exe` run in a 64-bit
process is upstream Wine's architecture — `lib/wine/{i386-windows,
x86_64-unix, x86_64-windows}`, and **WineHQ Stable 11.0 already on this
machine has the identical three directories**. D3DMetal is Apple's, and
also ships in the free Game Porting Toolkit. What is left of CrossOver is
the bottle manager and the installer scripts, which `longtrace.sh` does not
touch: it calls `wine --bottle ron --workdir … riseofnations_trace.exe`
and nothing else.

**How far free Wine got in twenty minutes.** A fresh prefix boots clean on
Wine 11.0; MoltenVK comes up on the M4 Max; the **stock** game runs and
reaches its own `CrossplayNetLibSys INIT`, writing through a symlink that
keeps `setlog.py`, `longtrace.sh` and `rondata::diff`'s `dump()` pointed at
the files they already know. It is not a compatibility failure. What did
not happen is a **window** — and that result is not clean, because the
screenshot at the end shows a macOS consent dialog, *"RonDriver would like
to access files in your Desktop folder"*, modal and holding focus through
both attempts. Run74's signature a second time in one session, and the
second time it was not diagnosed either: it was found by looking at the
picture.

**And the traced copy is its own question**: `riseofnations_trace.exe`
page-faults at `7BF21139` where the stock copy does not, which is
`tools/trace`'s int3 patching meeting a different Wine. The capture lane
needs both halves.

**The clock neither option controls.** Rosetta 2 ends with macOS 28 in
autumn 2027, and both paths ride on it to translate the x86-64 Wine host.
CodeWeavers shipped a Mac ARM64 preview in July 2026 — Wine 10's ARM64EC
plus their own macOS port of FEX — so the paid path has a route past it and
the free one has no announced equivalent. Which means the durable answer is
not CrossOver-versus-Wine at all: it is **getting the oracle off macOS**,
where any x86 machine runs the original natively, with no translation
layer, no licence and captures faster than three frames a second. That is
item 225's real horizon, and `docs/ORACLE.md`, "Off CrossOver", carries the
evidence.

**The session's own lesson, stated once because it happened twice.** A
warning printed by the wrong process (`cliclick`), a stall with a clean log
(`waitwin.sh`), and a test taken in front of an unanswered dialog are three
faces of one mistake: reasoning about a screen without looking at it. The
window title cost one line and settled the first; the screenshot cost one
line and settled the third. **Look at the screen before theorising about
it** now sits in `docs/ORACLE.md` twice over.

## 2026-09-04 (Opus) — item 225: the lane is off CrossOver, and the blocker was one call

**The capture lane runs on free WineHQ Stable 11.0.** The main menu draws,
the traced executable runs, and no licence is involved. The score did not
move — no capture was driven — but the thing that blocks every future
capture did.

**The opener was wrong, and the disk said so in four minutes.** It named
Apple's Game Porting Toolkit as "probably the answer", because D3DMetal was
believed to be what CrossOver contributed. CrossOver's own bundle disproves
it: `lib64/apple_gptk/wine/` holds `x86_64-unix` and `x86_64-windows` and
**no `i386-windows`**. `riseofnations.exe` is PE32. D3DMetal was never
serving this game, and GPTK could not have.

**`gfx.sh` ran and both variants failed** — with `GraphicsDLL=` cleared the
exe page-faults inside itself; with `d3dgl.dll` it reaches its own box. The
lever the file was written to test was the wrong lever, and the script is
deleted rather than kept, because its question is answered.

**The blocker is one call, and reading beat guessing.** The message belongs
to `d3dgl.dll`, whose import table names `d3d11.dll` — the name is a lie,
it is a D3D11 renderer. `llvm-objdump` around the address the backtrace
gave (`d3dgl+0x24183`, the instruction after the error call) shows the whole
decision in eleven pushes:
`D3D11CreateDevice(NULL, HARDWARE, NULL, 0, {0xa000}, 1, 7, …)` — one
feature level, `0xa000` = `D3D_FEATURE_LEVEL_10_0` — and a box on any
negative HRESULT. **A requirement that specific is a shopping list**, and
three candidates were then measured against it rather than argued about:
wined3d cannot get a 3.2+ GL context out of `winemac.drv` at all; stock
DXVK 2.7 skips the M4 Max for want of `geometryShader`; DXVK-macOS 1.10.3
answers with feature level 10_0 exactly. `docs/ORACLE.md`, "Off CrossOver",
carries each verbatim.

**Two instruments earned their keep and one did not.** `sample` is useless
here — Rosetta-translated stacks unwind into a thousand frames of the same
ntdll address — and `winedbg`'s `bt all` gave the answer instead, in one
line naming `d3dgl`. A `+relay` trace then showed what the game called
*around* the failure but never the failure itself, because **Wine's relay
only instruments builtin DLLs** and the interesting call went into a native
one. That is worth remembering: with a native override in place, relay goes
blind exactly where the question is.

**The empty log that was not empty.** DXVK's file log stayed zero bytes
through four runs and the natural reading — "never called" — was wrong
twice over; `DXVK_LOG_PATH=none` sends the same messages to stderr, where
nothing buffers them, and they said `Skipping: Device does not support
required feature 'geometryShader'` immediately. **A silent log is not
evidence of silence.**

**The `7BF21139` fault is real, and `cover=0` is the workaround.**
`docs/ORACLE.md` had written it up as possibly downstream of the DirectX
failure, to be re-taken once the renderer worked. It was re-taken: the
traced copy still faults at the same address with the renderer fixed, and
survives with `cover=0`. So it is VEH dispatch, as the diagnostic was
designed to decide — captures work, at the cost of function coverage, which
is the queue of blind readings. That is now item 226.

**Ported, not yet driven.** Eight capture scripts had the same three-line
CrossOver launch and thirteen had the same `osascript` focus line naming a
process — `riseofnations.exe` — that only CrossOver ever used; free Wine
calls every GUI process `wine`. Both are now one file each
(`tools/gamelog/winelaunch.sh`, `tools/gamelog/focus.sh`), with
`tools/gamelog/dxvk.sh` installing the DLLs into the prefix. **None of the
driven part is proven**: the lobby clicks, the fast-forward, `!quit` and
`gamelog.txt` under free Wine are all still to run, and the user was at the
machine, so nothing that moves their mouse was attempted.
