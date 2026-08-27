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
