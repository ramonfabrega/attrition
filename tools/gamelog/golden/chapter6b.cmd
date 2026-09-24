# Golden record, chapter six-b — the air line from a base.
#
# docs/GOLDEN.md §10's restage (item 651). Chapter six's run168 fired its
# second falsifier: an aircraft `add`ed outside a base never moves. The
# Fighter and the Bomber stood on their seats from birth to 899 at `air_alt`
# 0 with empty order stacks, burning fuel. This chapter is chapter six's
# lines with ONE lever added — an Airbase for each side, staged before the
# aircraft — and a window long enough to run both aircraft out of fuel.
#
#   run175 (item 651):
#   zsh tools/gamelog/viadriver.sh tools/explore/golden_capture.sh ~/ron-golden/ch6b \
#       --map 14 --end-frame 1250 --log-window 605 1250 --timeout 3600 \
#       --detail end:UNITS=3,GUYS=2,AMMO=5,DEATHS=1,LEADERS=2,BUILDS=7 \
#       --detail start:MISC,WORLD=6,TERRAIN=2,GOODS=3,UNITS=3,BUILDS=7,CITIES=5,GUYS=2,LEADERS=9,DEATHS=1 \
#       --detail misc:COMMANDMANAGER=1 \
#       --cmd-file tools/gamelog/golden/chapter6b.cmd
#
# Chapter six's header (`chapter6.cmd`) has the channel's rules, the reading
# of `bird`, and why every placement carries a coordinate. `BUILDS=7` is
# added to chapter six's `end:` because the lever is a building.
#
# ---------------------------------------------------------------------------
# THE PREMISE AND ITS KILLER, read before the run (docs/GOLDEN.md §3, point 5).
#
# The premise, as booked: **a based aircraft moves where an unbased one did
# not.** The reading kills the base-link half of it by construction, and says
# so here so that the run tests the reading rather than the hope.
#
# 1. **What moves an aircraft is an air order and nothing else.** Its
#    position is written by `Unit::do_air_physics@005e86d0`, whose callers
#    are `do_air_patrol`, `do_air_attack_ground` and `do_strafe` — the air
#    orders — and by an attack order's own walk. run168's aircraft had an
#    empty stack (`UnitData +0xd8`) and `inside_up` (`+0x82`) −1, so each
#    frame was `Unit::do_idle@0060dcd0 → think`.
# 2. **The base's writers of an air order act only on aircraft INSIDE it.**
#    There are three, and each walks the base's `inside_down` /
#    `inside_down_who` chain (`ObjectData +0x28/+0x3e`, types.txt):
#    - `Object::do_launch@0064f3b0`, from `Build::process@0061edf0:78` on
#      `build_masks & 8`, which `Build::init@00629740:280` ORs in (`| 0x88`).
#      **Its whole body is under `-1 < inside_down && -1 < inside_down_who`**,
#      its first test, so an empty base spends nothing — not even its
#      `launch_frames` (+0x41) count. Its computer arm (leader_flags & 4
#      clear, who=1) patrols the aircraft inside over the best enemy city,
#      fort or wonder; its leader loops are bounded `0..7` (`local_34 <
#      0xe0`, eight 0x1c strides; `local_20 < 0xe71ef8`, eight Leaders of
#      0x6eec past `leaders.list[0].field_0x408`).
#    - `Object::attempt_launch@00643a10`, from `Object::do_damage@0064a480:196`
#      on the DAMAGED object: a fighter-lineage plane (`is(BIPLANE 0x11f)`)
#      inside a base an enemy unit hits is sent to patrol the attacker. The
#      same chain, the same empty-head exit.
#    - `Build::train@0062f9b0`'s gather arm: `add_air_patrol_order` on the
#      fresh trainee at each gather point, bounded by `num_gather`.
# 3. **Where `add` and a trained aircraft differ.** `Build::train` calls
#    `Objects::init_unit(who, type, base.x, base.y, −1, −1, −1)` and then
#    `Unit::go_inside(base)` (train:102): a trained aircraft is born INSIDE
#    its base, `inside_up` = the base, and sits there (refuelling at
#    `air_unit_mana_recharge` in `Unit::process@00610bc0:88`) unless a
#    gather point orders it out. `add` (run_cmd 0x4d/0x4e) is
#    `find_nearby_spot` + `init_unit(…, −1, −1, −1)` and nothing after it
#    (docs/INPUT.md §11.5): the aircraft is born outside, `inside_up` −1.
# 4. **The only writer of `inside_up` is `Unit::go_inside@0061a2e0`** (and
#    `come_out`, which clears it). Its callers in the export: `Build::train`,
#    `Unit::do_garrison`/`do_board`/`do_gather`/`do_cast`/`do_spec_anim`,
#    `Group::action_garrison`/`action_gather`/`action_transport`,
#    `Unit::process:409` (a carrier's own trainee), `Unit::swap_team`,
#    `Object::do_damage:1031` (a ruin's spawn, which comes out at once),
#    `SpellType::cast_transport`, `ObjectData::cargo_inside`/`can_carry`,
#    `Setup::build_units`, and the editor's and the scenario funcs'
#    `add_unit`. None is reached by a channel line; the orders among them
#    need an issuer or the Leader AI's group actions, which `!ai off` holds
#    for run168's 295 frames (the Bomber sat in group 64 with no order).
#    `Unit::land_plane@005e9950`, the return to base, is reached only from
#    `check_fuel` under an air order.
#
# So the reading predicts **the base-link half of the premise dies**: both
# aircraft stand exactly as run168's did, base or no base.
#
# 5. **But the Airbase is also a target, and that half is open.** who=1's
#    Bomber carries `0x40000` (docs/INPUT.md §11.10), so its idle search in
#    `Unit::find_melee_target@005ff9c0` reaches
#    `max(…, UNIT_RESPOND_RANGE 12 × 0x180)` = 4608 units, 24 tiles
#    (crates/sim/src/fight.rs, `find_melee_target`, diff-backed on chapters
#    one to three). who=0's Airbase, centred on tile (4, 33), is ~10.8 tiles
#    from the Bomber's seat. Whether a `FLY_HIGH 0` plane on the ground takes
#    a building is NOT read here (`docs/COMBAT.md` §61 covers a plane target
#    only). The human's Fighter searches 12 tiles, 2304 units, and who=1's
#    Airbase is ~10.6 tiles from it; the Fighter's `OBJ_MASK 63TG` is
#    `ANTI_AIR`, and whether it may take a building is not read either.
#    **The order stack separates the two halves**: an order whose target is
#    the enemy Airbase is the target arm; `inside_up ≥ 0`, or an air order
#    with no enemy target, is the base arm.
# 6. **Fuel runs out inside the window.** run168 printed `mana_burn` as
#    block − 610 for the Fighter and block − 615 for the Bomber (289 and 284
#    on 899). `MANA` is 400 and 600 (unitrules.xml), so the counter reaches
#    `mana()` on block **1010** for the Fighter and **1215** for the Bomber.
#    `Unit::process:76` caps it there and does nothing else; the one reader
#    that acts on an empty tank, `Unit::check_fuel@005e9be0` ("out of fuel",
#    the death vslot 0x158), is called only from `do_air_physics`, under an
#    air order. Predicted: the counter stops and nothing else changes.
# 7. **The seats are run168's.** `find_nearby_spot@0061de70:143` skips the
#    terrain test for domain 2, and neither Airbase's footprint reaches row
#    40: Airbase is X_SIZE 5, Y_SIZE 9 (buildingrules.xml), centred on tile
#    row 33, so tiles 29–37. Predicted: Fighter (888, 7800), Bomber
#    (2424, 7800), exactly run168's.
# 8. **The ground is free.** run168's start `WORLD` block: cells (0..6,
#    7..9), tiles 0–27 × 28–39, are BASELAND, region 1, blocked 0; the
#    blocked cells nearest are (1..2, 11..13), south of the arena.
# 9. **The computer's Airbase is not disbanded as an orphan.**
#    `Leader::check_orphaned_buildings@006c9f20` runs for who=1 under
#    `!ai off` (plan_strategy's sweep, docs/INPUT.md §11.4) and names
#    Airbase (`is(0x1bf)`) in its disband arm, but that arm is entered only
#    on `build_masks & 1`, whose one writer is `Unit::resolve_block@005fccc0:49`
#    — a unit blocked by the building. Its second arm is for a building not
#    `is_active`; `add`'s building arm calls `Build::activate`.
#
# ---------------------------------------------------------------------------
# THE FALSIFIERS, and where each could first fire.
#
# 1. **The premise: an aircraft moves, or takes an order, or goes inside.**
#    First fires on the Fighter's UNITDATA on block 611 (who=0) and the
#    Bomber's on 616 (who=1), any block to 1249: `x_internal/y_internal` off
#    the seat, `air_alt` above 0, a non-empty order stack, or `inside_up`
#    ≥ 0. PREDICTED TO FIRE ONLY ON THE TARGET ARM, if at all (points 1–5):
#    a move whose order targets the enemy Airbase kills nothing in the
#    reading; a move with `inside_up ≥ 0` or a targetless air order kills
#    point 2 or 4.
# 2. **The Airbases displace the seats.** Fires on block 611 or 616: a seat
#    other than (888, 7800) / (2424, 7800). Predicted not to (point 7).
# 3. **An empty tank does something.** Fires on block 1010 or later for the
#    Fighter, 1215 or later for the Bomber: a DEATHS record for either, a
#    position or `air_alt` change, or an order, on or after the frame the
#    counter reaches `mana()`. Predicted not to (point 6).
# 4. **An empty base spends something.** Fires on the trace from 607/609:
#    any draw under `Object::do_launch` or `Object::attempt_launch`, or an
#    Airbase's `launch_frames` above 0, or who=1's Airbase leaving the dump
#    (a disband). Predicted not to (points 2 and 9).
#
# RUN 2026-09-23 as run175 (item 651; docs/RUNS.md): 315 s, 112 MB. The
# FIRST falsifier fired, on the target arm: each aircraft took an
# ATTACKORDER on the ENEMY Airbase on its birth block and walked to it on
# the ground at air_alt 0 (the Fighter to (600, 7944) by 632, then idle;
# the Bomber to (1992, 7704) by 700, then stuck). inside_up -1, air_alt 0,
# launch_frames 0 on every block: the base link is dead as read. The
# second, third and fourth did not fire.
#
# check: every staged line runs. `cmdsran.py` shows ten `INFO cmd` returning
#        1 (the eight here, `37 !ffwd`, `1250 !quit`).
# check: `MAP_STYLE 14`, seed 12345, blocks 1 and 605..1249 at least.
# check: the aircraft and the Airbases are identified by type and birth
#        block, never by slot.
# ---------------------------------------------------------------------------

# `ai` is console-only (table index 12). Every chapter's premise.
0 !ai off

# Chapter six's Modern age, unchanged.
600 library who=0 6
602 library who=1 6

# THE LEVER. One Airbase a side, finished and activated (`add`'s building
# arm, docs/INPUT.md §11.5), each centred seven tiles north of its own
# aircraft's seat, on the frames before the aircraft so that each is born
# with its base already standing. `airbase` matches no unit name, so
# `parse_type` resolves it to the building `Airbase`.
606 add airbase who=0 4,33
608 add airbase who=1 12,33

# Chapter six's aircraft, on chapter six's frames and seats.
610 add fighter who=0 4,40
615 add bomber who=1 12,40

# Chapter six's bird, unchanged, so the only difference from run168 before
# the fuel runs out is the two Airbases.
700 bird
