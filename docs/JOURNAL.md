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
