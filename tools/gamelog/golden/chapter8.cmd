# Golden record, chapter eight — the commanders, and a war that is declared.
#
# docs/GOLDEN.md §12. Chapter one's `604 war` is the BARE form: cases 0x2c-
# 0x2e print the diplomacy table and change nothing, and its squads engage
# because a Quick Battle already starts at war (docs/INPUT.md §11.6). This
# chapter is the one that actually moves the diplomacy state, three times,
# each with a fight running across the change: peace, then war, then
# alliance. And it carries the Command line — a General, whose aura is
# passive, and a Spy, whose abilities are the game's own spells.
#   run171 (item 660):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch8 \
#       --map 14 --end-frame 1200 --log-window 605 1200 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=5 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter8.cmd
#
# LEADERS=5 rather than 2: the diplomacy row is what this chapter reads.
# `LeaderData::log_data@006e5110` sets detail 3 just before `diplos[]`
# (docs/RUNS.md, run16), and run168's LEADERS=2 prints the row on its start
# block only, 32 lines, and on none of its 295 frames.
#
# The channel's rules this file obeys (docs/ORACLE.md, "The cheat channel"):
# a `!` line reaches the console-only half of `run_cmd`'s two switches and a
# bare line only the chat half; lines run in file order and a frame below its
# predecessor is clamped; and `no_mouse = 1`, so an `add` without an `x,y`
# silently reuses a stale cursor — every placement here carries one.
#
# Coordinates are TILES (docs/INPUT.md §11.3): the unowned arena of chapters
# one to three, world cell (1,10) and its neighbours.
#
# ---------------------------------------------------------------------------
# THE STAGING, checked before the run (docs/GOLDEN.md §3, point 5; item 660).
# The positions stand as designed; each check says why.
#
# 1. **The squads close and fight before 700.** Eight tiles is chapter two's
#    and chapter three's geometry (who=0 at 4,40, who=1's hoplites at
#    12,40), and in all three captures on disk (run112, run145, run146)
#    who=1's `1/6..1/8` take their ATTACKORDER on 635, after a who=0 squad's
#    own search on its 32-frame cadence (633..635) set the attacker's
#    `visible` bit (docs/COMBAT.md §31.5). Here the eyes are the commanders:
#    the General (LOS 8) sits on chapter one's who=1 seat, (1368, 7992),
#    seven tiles from who=1's captain at (2424, 7800), and the Spy (LOS 8)
#    at 12,41 lights who=1's fog over the General. This crate's is_seen is
#    diff-backed on chapter two's same geometry, and staged on run168's
#    start block (a scratch walk, not committed) it gives: both squads
#    ATTACK on 634; who=1 targets the General first, whose hits fall from
#    662 and who takes a FLEE_TO; the first blow on a who=0 hoplite on 692;
#    damage traded on both sides from 660 to 698. **Predicted: an
#    ATTACKORDER on both squads by block 636, and a `damage` change on both
#    sides before block 700.** A hoplite blow is 3 of 120 hits every ~30
#    frames in chapter one (run105), so a fight that starts is still running
#    at every later diplomacy change unless the diplomacy stops it.
# 2. **`!ai off` does not stop who=1's units** (docs/INPUT.md §11.10). Every
#    who=1 unit carries `unit_masks & 0x40000`, so `Unit::think@005f6e40:206`
#    is entered and left, and the tail runs as without the cheat. For the
#    hoplites that is `think_attack` above the block, so auto-engage, and the
#    AI's army join. For the **Spy** (attack 0, the 0x3a lineage) the tail's
#    last arm: on its `idle == 1` think and every 32 frames after, not in an
#    army, `think_scout(this, 0)`, whose first arm for a leader with bit 4
#    clear is `think_spellcaster`. What it casts is the capture's to say:
#    this crate gives the Spy no order in 1200 frames. The General is
#    who=0's, bit 4 set: the block's arms (`think_spellcaster` if special,
#    `think_scout` if not), then out, since who=0's units lack 0x40000. None
#    of this moves the diplomacy, which is what the chapter measures; it can
#    move the Spy, which is read, not pinned, until it moves.
# 3. **The diplomacy row.** `run_cmd` cases 0x2c..0x2e call
#    `Leader::set_diplo@006ec6a0` on `console->who` with the parsed target,
#    level ally 2, peace 1, war 0 (docs/INPUT.md §11.6). It is a no-op when
#    the level is unchanged; otherwise it writes **both** leaders' slots
#    (`diplos[1]` of who=0 and `diplos[0]` of who=1), calls
#    `Armies::diplo_change` on the declarer (who=0, human: nothing) and sets
#    `recalc_borders`. At `ally` it also counts live leaders allied to
#    neither side and, at zero, calls `Leader::victory`: gaia's leaders 8 and
#    9 are live (`leader_flags` 0x02000007) at war with both, so **no allied
#    victory at 900** and the game runs on. The row prints in LEADERDATA at
#    detail 3 as `diplos[scan]`, eight per leader. A line at N shows on
#    block N+1 (run168's 610 Fighter was born on block 611). Predicted:
#    `diplos[1]` of who=0 and `diplos[0]` of who=1 read 0 through block 700,
#    1 on 701..800, 0 on 801..900, 2 from 901. This crate's writer is
#    `Sim::set_diplo`, symmetric, and the harness compares `diplos[i]`
#    (crates/rondata/src/diff/leader.rs). run16 already saw `peace who=1`
#    and `war who=1` move the row through the chat box (docs/RUNS.md).
# 4. **The General's aura writes no field.** `UnitData::armor@00610160`
#    adds `general_rally_armor * (get_general_upgrade + 1)` when
#    `HeroesData::find_hero` finds a hero of the owner in range: rules.xml
#    GENERAL_RALLY_ARMOR 2, GENERAL_RADIUS 6 tiles. It is computed on read,
#    so no dumped field on the squad carries it (`hero` is the General's own
#    slot). **Its footprint is the blow**: a who=0 hoplite inside the radius
#    loses 2 fewer hits a strike than a who=1 hoplite. This crate models no
#    rally armor, so where the original's blow on who=0 is smaller, ours is
#    the unaided figure: 3 (the scratch walk's 692).
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS (docs/GOLDEN.md §12), and where each could first fire.
#
# 1. **`diplos[1]` unchanged in the block after 700, 800 or 900** — the
#    targeted form does no more than the bare one. First fires on LEADERDATA
#    0 and 1, blocks 701, 801 and 901. **Predicted NOT to fire** (check 3).
# 2. **Two squads still trading damage at frame 750, after the peace.**
#    Fires on the hoplites' UNITDATA `damage` moving on either side in
#    (701, 800]. Reachable only if check 1 holds: a fight by 700. This crate
#    drops the attack orders by 730 and trades no blow after 698, and
#    **predicts NOT**; the original's `Unit::fight` target test is the one
#    being measured.
# 3. **A General whose presence moves no field on the squad beside it.**
#    Restated by check 4: fires if the first blow on a who=0 hoplite inside
#    six tiles of the General is the same size as the blows on who=1's.
#    First reachable on the first who=0 `damage` step (this crate's 692).
#    **Predicted NOT to fire**: a smaller blow on who=0, 1 against 3.
#
# check: every staged line runs, twelve `INFO cmd` returning 1 (the ten here,
#        `37 !ffwd`, `1200 !quit`); `MAP_STYLE 14`, seed 12345, and 597
#        blocks: 1, 605..1199, 1201.
# check: `library who=N 2` prints `ages_get() 2` and `epochs_get() 8` on
#        both leaders, as chapter seven's who=0 did.
# check: four new who=0 UNITDATA (three hoplites on 611, the General on 613)
#        and four who=1 (three on 616, the Spy on 618); the Spy's
#        `unit_masks & 0x40000`.

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# The Medieval age: the General and the Spy both want Mathematics, and
# `library` carries the whole shelf rather than one tech.
600 library who=0 2
602 library who=1 2

# A squad each, so there is a fight for the diplomacy to interrupt, and a
# commander each: the General's bonus is an aura over the squad beside it,
# the Spy's is a spell it can be made to cast.
610 add hoplite who=0 4,40
612 add general who=0 5,40
615 add hoplite who=1 12,40
617 add spy who=1 12,41

# The three diplomacy levels, each with a target token, which is what makes
# them the non-bare form. `parse_who`'s default here is `console->who`, so a
# bare `1` is a player and not a level (docs/INPUT.md §11.2).
700 peace 1
800 war 1
900 ally 1
