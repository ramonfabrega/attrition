//! The draw-site trace — reading `rontrace.log` from Rust.
//!
//! `tools/trace/` is the instrument (`docs/ORACLE.md`, "The draw-site trace
//! and function coverage"); `tools/trace/report.py` has been the only
//! reader of it since it existed. This is the second, and it exists for
//! one reason: **a count is a weak assertion and the trace carries a
//! sequence**. Every draw record names the *site* that took it — the
//! return address into the calling function — so a mechanic can be checked
//! draw for draw rather than by a total, and checked that way even while
//! the frame's stream has not yet been made to line up
//! (`docs/SCOUT.md` §10 is the first such check).
//!
//! Nothing here reads the user's install: a trace is a file the user's own
//! run wrote, taken by path exactly as a `gamelog.txt` is, and nothing
//! from it is committed.
//!
//! ## The format
//!
//! A 32-byte header then 32-byte records, each eight little-endian `u32`:
//! `[kind, a, b, c, d, e, f, frame]` (`tools/trace/tracer.c`'s `emit`). The
//! header is a record whose `kind` is the ASCII `RONT`, carrying the
//! version, the image base, `.text`'s RVA and size, the function count and
//! the window's low frame.
//!
//! | kind | a | b | c | d | e | f |
//! | --- | --- | --- | --- | --- | --- | --- |
//! | 0 `HIT` | function VA | thread | | | | |
//! | 1 `get()`, 3 `get(a,b)`, 4 `rand_real`, 6 `reseed` | the **caller's return address** | the `Random *` | the seed **before** the step | the caller's caller | and its caller | `arg0` |
//! | 2 `FRAME` | frame | `game_random`'s word | functions re-armed | `do_frame`'s caller | | |
//! | 5 `INFO` | code | … | | | | |
//!
//! Two things about that table decide everything below. The `Random *` in
//! `b` is what separates the **sync stream** from the renderer's four
//! other generators — only a step of `game_random` is simulation state.
//! And the seed in `c` is the word *before* the step, so a run of draws
//! reads as the LCG's own sequence and the first of them is the word to
//! seed a replay with.

use std::path::Path;

/// The image base every address here is normalised to. A run that
/// relocated reports its own base in the header and is folded back to this
/// one, so a site is comparable between runs and against the Ghidra
/// export.
pub const IMAGE_BASE: u32 = 0x0040_0000;

/// `game_random`'s RVA — the one generator whose steps are simulation
/// state. `tools/trace/report.py` carries the same constant.
pub const RVA_GAME_RANDOM: u32 = 0x00a3_7a8c;

/// The sites the simulation models, and the names it marks them with.
///
/// This is **not** a symbol table: naming the trace in general needs the
/// Ghidra export's `INDEX.tsv`, which never enters this repo, and
/// `tools/trace/report.py` stays where a name comes from. This is the far
/// smaller thing a differential check needs — the handful of addresses a
/// mechanic in `crate::sim` has claimed, each paired with the string that
/// mechanic's own `Sim::mark` writes. The label lives in `sim`, beside the
/// code that spends the draw; only the address lives here.
///
/// `via` is the disambiguator, and it is why the table is not a flat map.
/// One address can be several sites: `Guy::set_anim+0x97a` is the idle
/// roll for an animal, for an idle unit, for a gathering one's stand and
/// for the phase-7 wrap, and the trace tells them apart only by the `ebp`
/// chain ([`Draw::up`]). An entry with `via` matches when that address is
/// somewhere in the chain; the first matching entry wins, so a
/// chain-qualified entry must precede a bare one for the same site.
///
/// Every offset here is a return address into the named function, taken
/// from a run's own trace and checked against the Ghidra export
/// (`docs/SYNC.md` §3, §5).
pub const SITES: &[(u32, Option<u32>, &str)] = &[
    // `Leader::compute_sites@006cc950` — the AI's region sweep.
    (0x006c_cdfc, None, sim::ai_sites::SITE_STRIDE),
    (0x006c_ce5a, None, sim::ai_sites::SITE_MARK),
    // `Leader::make_stuff@006c8af0` — the two expiry walks, step 4's over
    // the head's type and step 6's over a bought slot's (`docs/AI.md` §2.6).
    (0x006c_8d11, None, sim::ai_make::SITE_EXPIRE_HEAD),
    (0x006c_912d, None, sim::ai_make::SITE_EXPIRE_SLOT),
    // `Leader::use_market@006c91c0` — the sell rotation's offset, the one
    // draw in the whole market. `make_stuff` calls `use_market` first, so
    // this precedes both expiry walks in a frame that takes it.
    (0x006c_93ad, None, sim::ai_make::SITE_MARKET_SELL),
    // `Leader::create_units@006c40a0` and `Leader::upgrade_units@006c6430`
    // — the matchup bias over `unit_prod_value`, one draw per candidate,
    // and none on difficulty 2 (`docs/AI.md` §11).
    (0x006c_46e2, None, sim::ai_units::SITE_UNIT_BIAS),
    (0x006c_69d4, None, sim::ai_units::SITE_UPGRADE_BIAS),
    // `Leader::research_techs@006c6ba0` — the Senate arm's coin and the
    // survivor's scale; East Indies spends the second on 15378 (item 643).
    (0x006c_77af, None, sim::ai_research::SITE_GOV_COIN),
    (0x006c_7812, None, sim::ai_research::SITE_GOV_ROLL),
    // `Leader::create_buildings@006c1be0` — the wonder arm's pair.
    (0x006c_2bdb, None, sim::ai_build::SITE_WONDER_MOD),
    (0x006c_2bf7, None, sim::ai_build::SITE_WONDER_SCALE),
    // `Leader::produce_building@006e1400` — the spiral's score and the
    // 2×2 jitter's.
    (0x006e_2099, None, sim::ai_place::SITE_SPIRAL),
    (0x006e_2c05, None, sim::ai_place::SITE_JITTER),
    // `GameDaemon::calc_market@00732270` — three a good.
    (0x0073_22c4, None, sim::market::SITE_A),
    (0x0073_22ee, None, sim::market::SITE_B),
    (0x0073_232e, None, sim::market::SITE_LENGTH),
    // `Guy::set_anim@005da300+0x97a` — one address, four callers.
    (
        0x005d_ac7a,
        Some(0x005d_7479), // `Animal::do_idle+0x19`
        sim::anim::SITE_IDLE_ANIMAL,
    ),
    (
        0x005d_ac7a,
        Some(0x0060_dd4d), // `Unit::do_idle+0x7d`
        sim::anim::SITE_IDLE_UNIT,
    ),
    (
        0x005d_ac7a,
        Some(0x005d_a081), // `Guy::inc_time+0x271`
        sim::anim::SITE_WRAP,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_027f), // `Unit::do_non_flat_gather+0x10f`
        sim::anim::SITE_STAND_GATHER,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_1144), // `Unit::do_non_flat_gather+0xfd4`
        sim::anim::SITE_STAND_TILE,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_0d09), // `Unit::do_non_flat_gather+0xb99`
        sim::anim::SITE_STAND_RETURN,
    ),
    (
        0x005d_ac7a,
        Some(0x005d_93df), // `Guy::move+0x19f`, the arrival stand
        sim::anim::SITE_ARRIVE,
    ),
    // …and the idle an **attack** falls to when its packet does not loop
    // it, which is `Guy::inc_time`'s *other* call site (`docs/ANIM.md`
    // §6.2). Reading it as `+0x271` would have read an attack's end as an
    // ordinary wrap.
    (
        0x005d_ac7a,
        Some(0x005d_9ffd), // `Guy::inc_time+0x1ed`
        sim::anim::SITE_WRAP_ATTACK_END,
    ),
    // `Guy::set_anim@005da300+0xf2f` — **the attack roll**, a different
    // block of the same function and so a different address. Three
    // callers, none of them the swing: `Unit::fight` defers the request
    // into `GuyData +0x9e` and these are where it is paid
    // (`docs/ANIM.md` §6.2).
    (
        0x005d_b22f,
        Some(0x005d_93a6), // `Guy::move+0x166`, the settled arm
        sim::anim::SITE_ATTACK_STAND,
    ),
    (
        0x005d_b22f,
        Some(0x005d_9323), // `Guy::move+0xe3`, the still-turning arm
        sim::anim::SITE_ATTACK_TURN,
    ),
    (
        0x005d_b22f,
        Some(0x005d_a081), // `Guy::inc_time+0x271`, the queued attack
        sim::anim::SITE_ATTACK_WRAP,
    ),
    (
        0x005d_b22f,
        Some(0x005d_a167), // `Guy::inc_time+0x357`, past the wrap loop
        sim::anim::SITE_ATTACK_INC,
    ),
    // And the swing paid where it is asked, under `Unit::set_anim@00616f40`
    // from `Unit::fight+0x19f6`, when the unit's pivot bears and nothing
    // turns (`docs/COMBAT.md` §52): one loop for the squad's figures, one
    // for the crew.
    (
        0x005d_b22f,
        Some(0x0061_6f96), // `Unit::set_anim+0x56`, figures `0 .. guy_mark`
        sim::anim::SITE_ATTACK_FIGHT,
    ),
    (
        0x005d_b22f,
        Some(0x0061_6ff6), // `Unit::set_anim+0xb6`, the crew
        sim::anim::SITE_ATTACK_FIGHT_CREW,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_b753), // `Unit::move_step+0x823`, the blocked stand
        sim::anim::SITE_BLOCKED,
    ),
    // …and the **snap** arm's own, which is a different block at a
    // different address: `Unit::move_step+0x4e2`, the collision the
    // Manhattan arrival runs into (`docs/COLLISION.md` §5.4). Until item
    // 360 this crate spent `+0x823` for both and the trace printed a bare
    // `5dac7a` for the original's, which is a comparison that cannot fail.
    (
        0x005d_ac7a,
        Some(0x005f_b412), // `Unit::move_step+0x4e2`, the snap's stand
        sim::anim::SITE_SNAP_BLOCKED,
    ),
    // …and `Unit::fight`'s reloading stand, the recharging arm's
    // `set_anim(CHAR_DEFAULT, 1, 1)`. The trace printed it bare as
    // `5dac7a` on golden chapter one's 774 until item 530.
    (
        0x005d_ac7a,
        Some(0x005f_d639), // `Unit::fight+0x169`
        sim::anim::SITE_RELOAD_IDLE,
    ),
    // …and the **turning** stand, three chains of `Guy::do_turn+0x4a`
    // (`005d97ea`). The `via` is the frame above `do_turn`, which is what
    // tells the three callers of the override apart: `Unit::move_step`'s
    // near arm, its far arm, and `Guy::turn_towards`, which `Guy::move`'s
    // standing arm calls (`docs/ANIM.md` §4.8).
    (
        0x005d_ac7a,
        Some(0x005f_b2e6), // `Unit::move_step+0x3b6`, the near arm
        sim::anim::SITE_TURN_NEAR,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_b2b9), // `Unit::move_step+0x389`, the far arm
        sim::anim::SITE_TURN_FAR,
    ),
    (
        0x005d_ac7a,
        Some(0x005d_9789), // `Guy::turn_towards+0x69`
        sim::anim::SITE_TURN_STAND,
    ),
    // …and the crew's, `do_turn` recursing into its trackless crew from
    // `+0xe5`. The trace printed it bare as `5dac7a` on golden chapter
    // seven-b's 672 until item 628.
    (
        0x005d_ac7a,
        Some(0x005d_9885), // `Guy::do_turn+0xe5`
        sim::anim::SITE_TURN_CREW,
    ),
    // `Unit::resolve_unit_collision@005f9d30+0xb52` — the head-on pair's
    // stagger, the collision mechanic's only draw.
    // `Unit::think_fish@005f4c60` — the jitter each accepted cell spends
    // (`docs/ORDERS.md` §6.8).
    (0x005f_4eda, None, sim::fish::SITE_JITTER),
    // `PathFinder::astar_path@00683770`'s two unit-grid failure tails —
    // the retry roll each buys a blocked mover (`docs/PATHFINDER.md` §21).
    // Both are `Random::get`'s own return address, so neither needs a
    // chain; they are separate entries because their gates differ.
    (0x0068_4e07, None, sim::path::SITE_UPATH_RETRY),
    (0x0068_48c9, None, sim::path::SITE_UPATH_RETRY_BUDGET),
    (0x005f_a882, None, sim::collide::SITE_PAUSE),
    // `Unit::explore_goody@005f9780+0x27c` — the goody box's lottery, one
    // draw a candidate good (`docs/GOODY.md` §3).
    (0x005f_99fc, None, sim::goody::SITE_PICK),
    // `Guy::set_anim@005da300+0x104b` — the gaia bird's wing-beat coin.
    // Its own address, so no chain is needed to tell it from the other
    // four (`docs/SYNC.md` §3.9).
    (0x005d_b34b, None, sim::anim::SITE_BIRD_COIN),
    (
        0x005d_ac7a,
        Some(0x005e_cc69), // `Unit::do_cast+0xc89`, through `Unit::set_anim`
        sim::anim::SITE_CAST,
    ),
    // `Unit::do_trade@005ed270+0x40` — the trade step's own opening
    // `set_anim(CHAR_DEFAULT, 0, 1)`. The frame between it and
    // `Guy::set_anim` is `Unit::set_anim`, at `+0x56` for the guys below
    // `field_0xb5` and `+0xb6` for the crew above `type+0x304`, so the
    // *caller* is the disambiguator and not the intermediate frame: run54's
    // 6198 is one draw through the first and two through the second.
    (
        0x005d_ac7a,
        Some(0x005e_d2b0), // `Unit::do_trade+0x40`, through `Unit::set_anim`
        sim::anim::SITE_TRADE,
    ),
    // `Unit::go_inside@0061a2e0+0x280` — the scholar's seating, through
    // `Unit::set_anim`. Great Lakes 8272 is the first birth in 24,000
    // frames that reaches it, and every earlier one is a non-scholar
    // (`docs/CITIES.md` §6.5.2).
    (
        0x005d_ac7a,
        Some(0x0061_a560), // `Unit::go_inside+0x280`, through `Unit::set_anim`
        sim::anim::SITE_GO_INSIDE,
    ),
    // `Unit::do_move@005f7b30+0x11cf` — an unarmed attack-move standing
    // out its pause, through `Unit::set_anim`; the trace printed it bare as
    // `5dac7a` on golden chapter four's 1416 until item 569
    // (`docs/ORDERS.md` §24.9).
    (
        0x005d_ac7a,
        Some(0x005f_8cff), // `Unit::do_move+0x11cf`, through `Unit::set_anim`
        sim::anim::SITE_PAUSE_STAND,
    ),
    // `Unit::do_guard@005e5c70`'s three stands (item 569, `docs/ORDERS.md`
    // §24.4): the idle on the post, the stand before the `retry` roll, and
    // a dead target's.
    (
        0x005d_ac7a,
        Some(0x005e_6464), // `Unit::do_guard+0x7f4`
        sim::orders::SITE_GUARD_IDLE,
    ),
    (
        0x005d_ac7a,
        Some(0x005e_6550), // `Unit::do_guard+0x8e0`
        sim::orders::SITE_GUARD_STAND,
    ),
    (
        0x005d_ac7a,
        Some(0x005e_6596), // `Unit::do_guard+0x926`
        sim::orders::SITE_GUARD_DEAD,
    ),
    // `Unit::do_follow@005e65d0`'s stand within the standoff (item 714,
    // `docs/ORDERS.md` §28).
    (
        0x005d_ac7a,
        Some(0x005e_68fa), // `Unit::do_follow+0x32a`
        sim::orders::SITE_FOLLOW_STAND,
    ),
    // `Guy::init_real@005db6b0` — the creation roll.
    (0x005d_b702, None, sim::anim::SITE_INIT_REAL),
    // `Dock::init@00740a80+0x125` — a finished dock's gull, the second of
    // its two draws (`docs/TRANSPORT.md` §5.2).
    (0x0074_0ba5, None, sim::transport::SITE_GULL_ANGLE),
    // `Unit::come_out@00617c10`'s tail — the army coin, two arms and two
    // addresses of their own: `+0x25b0` is `is(BARK)`'s `% 3` and
    // `+0x25ca` the scout's `% 2` (`docs/ARMY.md` §4). No chain is needed;
    // the *caller* is what tells a trained unit from a disembarking one,
    // and that is a distinction neither arm makes.
    (0x0061_a1c0, None, sim::army::SITE_COME_OUT_BARK),
    (0x0061_a1da, None, sim::army::SITE_COME_OUT),
    // `Army::find_target@006f69b0` — its own two, and until item 317 the
    // trace spelled both as bare addresses while the sim spelled neither.
    // `+0x410` is the per-leader coin at the call `6f6dbb`, whose answer
    // is read `and $0x80000001` / `jne`; `+0x7df` the per-candidate score
    // at `6f718a`, read `cltd` / `idiv $0xc8` / `lea 0x384(%edx)` — the
    // `% 200 + 900` of `docs/ARMY.md` §12. Both are the army's own
    // address, so no chain is needed.
    (0x006f_6dc0, None, sim::army::SITE_FIND_TARGET_COIN),
    (0x006f_718f, None, sim::army::SITE_FIND_TARGET_SCORE),
    // `Unit::do_non_flat_gather@005f0170` — the wood machine's own three.
    // The last two are one apparent branch and two real ones: `+0xcc3` is
    // the chopping guy's `% 100 + 300` and `+0xdad` the arrival frame's
    // `% 50 + 100` (`docs/ORDERS.md` §6.4).
    (0x005f_06bb, None, sim::orders::SITE_TILE_WAIT),
    (0x005f_0e33, None, sim::orders::SITE_WORK_WAIT),
    (0x005f_0f1d, None, sim::orders::SITE_ARRIVE_WAIT),
    // `GameAccess::rnd@0043cca0+0x20` — the frameless helper. Its address
    // says nothing on its own; the chain does, and `Unit::do_job+0x67` is
    // `do_gather`'s own return address (both it and `GameAccess::rnd` are
    // skipped by the `ebp` walk).
    (
        0x0043_ccc0,
        Some(0x0061_7a77), // `Unit::do_job+0x67` — `Unit::do_gather`
        sim::orders::SITE_FARM_CELL,
    ),
    // `Unit::do_move@005f7b30` — the grid draw.
    (0x005f_89b4, None, sim::orders::SITE_MOVE_GRID),
    // `Unit::think_scout@005f6010` — the ring walk (`docs/SCOUT.md` §10).
    (0x005f_6446, None, sim::scout::SITE_ROTATION),
    (0x005f_6468, None, sim::scout::SITE_PHASE),
    (0x005f_665c, None, sim::scout::SITE_CELL),
    // …and the region fallback's two (`docs/SCOUT.md` §11).
    (0x005f_6951, None, sim::scout::SITE_REGION_STRIDE),
    (0x005f_6aca, None, sim::scout::SITE_REGION_CELL),
    // `Animal::do_idle@005d7460` — a herd animal's wander: the coin, then
    // the direction and the two step counts. Four addresses of its own.
    (0x005d_74e3, None, sim::gaia::SITE_WANDER_ROLL),
    (0x005d_7604, None, sim::gaia::SITE_WANDER_DIR),
    (0x005d_7634, None, sim::gaia::SITE_WANDER_X),
    (0x005d_7672, None, sim::gaia::SITE_WANDER_Y),
    // `Animal::think_farm_animal@005d7700` — a pasture animal's step.
    (0x005d_7842, None, sim::farms::SITE_ANIMAL_DIR),
    // `Objects::process_all@0065dce0` — the birds' sampling.
    (0x0065_dfbf, None, sim::gaia::SITE_BIRD_X),
    (0x0065_dfeb, None, sim::gaia::SITE_BIRD_Y),
    // `Animal::think_bird@005d79e0` — a live bird's three, every eighth
    // frame, under `Unit::do_air_patrol+0x28` < `Unit::do_job+0xd7`.
    (0x005d_7a62, None, sim::gaia::SITE_BIRD_WANDER_X),
    (0x005d_7a86, None, sim::gaia::SITE_BIRD_WANDER_Y),
    (0x005d_7bd8, None, sim::gaia::SITE_BIRD_LAND),
    // …and the landing search it opens, thirty rounds of two.
    (0x005d_7c8a, None, sim::gaia::SITE_BIRD_SEARCH_CELL),
    (0x005d_7cb3, None, sim::gaia::SITE_BIRD_SEARCH_SCORE),
    // `Unit::do_air_physics@005e86d0` — the edge coin, thrown on the frame
    // a bird's step first leaves the world and not again until one lands
    // inside (`docs/SYNC.md` §3.9). run54's is at 5437.
    (0x005e_8d09, None, sim::air::SITE_AIR_TURN),
    // …and a non-bomber plane's `cruising_alt` redraw, every eighth frame
    // of its flight (`docs/ORDERS.md` §33.1): chapter seventeen's 642.
    (0x005e_878a, None, sim::air::SITE_AIR_ALT),
    // `Herd::process@00741760` — one herd's walk.
    (0x0074_1777, None, sim::gaia::SITE_HERD_X),
    (0x0074_1796, None, sim::gaia::SITE_HERD_Y),
    // `MathUtilFuncSet::rand_int@009e1890` — the script VM's one draw,
    // under the interpreter's call-out.
    (
        0x009e_18a8,
        Some(0x009d_5901), // `ScriptFuncSet::call_func+0x401`
        sim::ai_host::SITE_RAND_INT,
    ),
    // `PathFinder::calc_road_cost@00686300` — the road jitter, one draw a
    // node costed, under `astar_caravan_road+0x52b < find_road+0x3a8`
    // (`docs/ROADS.md` §5). Its own address, so no chain is needed.
    (0x0068_6346, None, sim::roads::SITE_COST),
    // `Farms::inc_time@008d8600` — the crop clock.
    (0x008d_87ae, None, sim::farms::SITE_CHANCE),
    (0x008d_87de, None, sim::farms::SITE_SPROUT),
    // `Farms::add@008d8a40` — the pasture coin and the ambience emitter.
    (0x008d_8b68, None, sim::farms::SITE_TYPE_COIN),
    (0x008d_8c7f, None, sim::farms::SITE_AMBIENCE_X),
    (0x008d_8c9b, None, sim::farms::SITE_AMBIENCE_Y),
    // `Farms::add_animals@008d8f30` — the five animals a finished pasture
    // is stocked with, three draws each. The same three addresses
    // [`ADD_ANIMALS_COIN`] and its pair name for the *setup* borrow; here
    // they are the labels the simulation's own drawing path writes.
    (ADD_ANIMALS_COIN, None, sim::farms::SITE_ANIMAL_COIN),
    (ADD_ANIMALS_Y, None, sim::farms::SITE_ANIMAL_Y),
    (ADD_ANIMALS_X, None, sim::farms::SITE_ANIMAL_X),
    // `Build::find_gather_tiles@00623350+0x10a` — the mining list's
    // shuffle, one draw a round over `4 × length` of them, under
    // `Build::init+0x55b`. Its own address, so no chain is needed.
    (0x0062_345a, None, sim::gather::SITE_SHUFFLE),
    // `Unit::find_attack_pos@00601280+0xea9` — the ring walk's one draw,
    // and **two chains**, because one address is two mechanics: the
    // group order's single call on its leader, and the chase each member
    // then runs for itself (`docs/COMBAT.md` §17). The simulation spends
    // neither yet; the entries are here so the frame reads as itself
    // rather than as a bare `602129`, which is a comparison that cannot
    // fail. `+0x2d` of the *seven*-argument overload `@00602e60` is the
    // frame between `fight` and the draw, so the `via` names the caller
    // above it rather than the thunk.
    (
        0x0060_2129,
        Some(0x005f_e184), // `Unit::fight+0xcb4`
        sim::fight::SITE_ATTACK_POS_FIGHT,
    ),
    (
        0x0060_2129,
        Some(0x0071_28aa), // `Group::action_attack+0x41a`
        sim::fight::SITE_ATTACK_POS_GROUP,
    ),
    // `Unit::fight@005fd4d0+0x9b0` — the one-in-five re-search's roll,
    // spent before either suppression is read (`docs/COMBAT.md` §8.2
    // step 0). One caller, so no chain is needed.
    (0x005f_de80, None, sim::fight::SITE_FIGHT_RESEARCH),
    // `Ammo::init@0067bbf0` — the landing scatter's two draws, the whole
    // cost of a shot that hits open ground or a building
    // (`docs/COMBAT.md` §9.1). Both reach here through
    // `Objects::add_ammo+0x119`, from either launch route — the
    // animation's release event or `Object::fire_ammo` — so no chain
    // separates them at this depth and none is given.
    (0x0067_c8c9, None, sim::fight::SITE_AMMO_SCATTER_X),
    (0x0067_c8fb, None, sim::fight::SITE_AMMO_SCATTER_Y),
    // The attack-ground arm's own pair (`init:461`–`484`), ahead of
    // `find_data_z`: run146's catapult spends them on 797, where they
    // read as a bare `67c6d8`/`67c715` until item 617 named them
    // (`docs/COMBAT.md` §59.5).
    (0x0067_c6d8, None, sim::fight::SITE_AMMO_GROUND_SCATTER_X),
    (0x0067_c715, None, sim::fight::SITE_AMMO_GROUND_SCATTER_Y),
    // `Object::take_damage@00652020` — a building's **first wound**: one
    // roll at `+0xe1` whenever combat damage reaches an object whose
    // `damage` is still zero and whose vtable `+0x1c` answers 1, and a
    // second at `+0x18b` for the flock a siege hit on a fort, temple or
    // town puts up (`docs/COMBAT.md` §7.2 step 3). Every caller reaches
    // both through `Object::do_damage`, so no chain separates them.
    (0x0065_2101, None, sim::fight::SITE_FIRST_WOUND),
    // `+0x18b` is `0x0065_21ab`, and this row read `0x0065_218b` until
    // item 483 — two digits transposed, on an address that is not even an
    // instruction boundary (`65218a` is a three-byte `mov`), so the row
    // could never match a draw and the flock's would have printed as a
    // bare `6521ab`. `every_site_s_address_is_the_function_its_label_
    // names` found it on its first run; no capture on disk takes the draw,
    // so no comparison moved (`docs/COMBAT.md` §39.3).
    (0x0065_21ab, None, sim::fight::SITE_FIRST_WOUND_FLOCK),
    // `Ammo::do_damage@00678060` — the puncture point of a shot that hit
    // nothing, `+0xc59` for x and `+0xc7e` for y (`docs/COMBAT.md` §39).
    // The function has exactly two `Random::get` calls and these are
    // both, so no chain is needed. Great Lakes spends them on 10237,
    // 10242 and 10249 and nowhere else in 24,000 frames, and this crate
    // spends its own on the same three; until they were named the
    // original's read as a bare `678cb9`/`678cde` against this crate's
    // `projectiles` phase mark, and the sequence word could not pass
    // 10237 however the simulation behaved.
    (0x0067_8cb9, None, sim::fight::SITE_PUNCTURE_X),
    (0x0067_8cde, None, sim::fight::SITE_PUNCTURE_Y),
    // `Unit::close@0060ee50` — the death animation's own draw, and the
    // two more a `dtype == 4` death takes (`docs/COMBAT.md` §42.1). The
    // three calls are the only `Random::get`s in the function and they
    // are consecutive, so no chain is needed; the listing is
    // `60fb01`/`60fb31`/`60fb55` and a trace site is the **return**
    // address. Chapter two's word stood at 683 on this one draw: the
    // original's `1/8` dies there and takes it, and this crate's died
    // there from item 485 and took nothing, so the streams parted at
    // draw 0 with the original's site reading as a bare `60fb06`.
    (0x0060_fb06, None, sim::fight::SITE_DEATH_ANIM),
    (0x0060_fb36, None, sim::fight::SITE_DEATH_ANIM_ALT),
    (0x0060_fb5a, None, sim::fight::SITE_DEATH_ANIM_FACING),
];

/// The header's `kind`: `RONT`, little-endian.
const MAGIC: u32 = 0x544e_4f52;

/// The record kinds that step a generator.
const DRAW_KINDS: [u32; 4] = [1, 3, 4, 6];

/// The proxied call sites, by the id their records carry (`tracer.c`'s
/// `CALLS`). A proxy logs a function's **arguments and its answer**, which
/// is the one thing neither a draw hook nor the gamelog can give.
pub mod call_site {
    /// `PathFinder::astar_path@00683770(stack, step, anti)`.
    pub const ASTAR_PATH: u32 = 0;
    /// `PathFinder::calc_cost@00684e50(from.x, from.y, to.x, to.y, dir,
    /// step, depth, uchar *transport)` — `docs/PATHFINDER.md` §5.
    pub const CALC_COST: u32 = 1;
    /// `Unit::do_air_physics@005e86d0(UnitOrder *, goal.x, goal.y)` —
    /// `docs/SYNC.md` §3.9. Its entry and return **bracket** a bird's
    /// frame, so a [`CALC_COST`]-style flat filter is not how these are
    /// read: [`Trace::air_frames`] folds a bracket and everything nested
    /// inside it into one record, which is the only way owner 9 is told
    /// apart from every other unit that moved.
    pub const DO_AIR_PHYSICS: u32 = 2;
    /// `Unit::air_turn_speed@005ea390(sign, 0)` — the frame's turn rate,
    /// and thereby the bank angle no dump prints.
    pub const AIR_TURN_SPEED: u32 = 3;
    /// `Unit::set_new_location@005f8d20(x, y, 0, 1)` — where the step
    /// landed. Called by every moving unit, so only the nested ones are a
    /// bird's.
    pub const SET_NEW_LOCATION: u32 = 4;
    /// **`RON_LEADER_PROBE` builds only** (DECISIONS 41 §6, built by the
    /// seventh pass): ids 8–10 are the AI's own decisions, and the same
    /// ids are the target probe's in a `RON_TARGET_PROBE` build — a log
    /// says which by its `PROXIED` records ([`Trace::proxied`]).
    /// `Leader::create_units@006c40a0(void)`, the bracket: a `make_me`
    /// nested in it is a unit offer.
    pub const LEADER_CREATE_UNITS: u32 = 8;
    /// `MakeList::make_me@006c9be0(t, val, escrow, cat, city, up, p7,
    /// num, wx, wy)` — the offer itself; the first eight ride the record.
    pub const LEADER_MAKE_ME: u32 = 9;
    /// `Leader::make_this@006c94f0(slot)` — the purchase; the answer is
    /// whether it bought.
    pub const LEADER_MAKE_THIS: u32 = 10;
    /// **`RON_COLLIDE_PROBE` builds only** (`docs/COLLISION.md` §9, item
    /// 456): ids 8–12 are the collision sweep read from inside, and they
    /// share 8–10 with the leader and target probes — a log says which
    /// build wrote it by its `PROXIED` records ([`Trace::site_va`]).
    ///
    /// `Unit::detect_unit_collision@00617060(x, y, quick, boats, p5,
    /// nocoll, top_only)` — the **bracket**: everything nested inside it
    /// belongs to one unit's probe of one point, and its answer is
    /// whether the step was refused. `this` is the asking unit, named in
    /// `(who, o)` by [`Trace::unit_of`].
    pub const DETECT_UNIT_COLLISION: u32 = 8;
    /// `CollCheck::collide_here@00682540(o, who, ucx, ucy, coll_size,
    /// &hit_x, &hit_y, nocoll)` — §4.2's probe. The first two arguments
    /// are the **asker's** own pair; the hit cell rides
    /// [`WILL_BE_CORNER`], because here it is behind out pointers.
    pub const COLLIDE_HERE: u32 = 9;
    /// `UnitData::will_be_corner@00609fa0(hit_x, hit_y, ucx, ucy)` —
    /// called once, immediately after a successful probe, so its presence
    /// **is** `collide_here != 0` and `args[0..2]` is the hit cell as
    /// values. The answer is the asking unit's half of §4.3's corner rule.
    pub const WILL_BE_CORNER: u32 = 10;
    /// `UnitData::is_here@0060a0c0(&hit_x, &hit_y)` — one call per
    /// candidate the 3×3 world-cell walk reaches, `this` the candidate:
    /// the census of who was looked at, and the answer is who covers the
    /// hit cell. Both arguments are pointers, so [`Trace::unit_of`] is
    /// what makes the record readable.
    pub const IS_HERE: u32 = 11;
    /// `UnitData::is_corner@0060a040(hit_x, hit_y, self)` — the blocker's
    /// half of the corner rule, reached only when [`WILL_BE_CORNER`] was
    /// non-zero **and** every soft arm declined. Its absence is therefore
    /// as informative as its presence.
    pub const IS_CORNER: u32 = 12;
    /// `PathFinder::astar_caravan_road@00685990(stack, whoA, whoB, p4, p5,
    /// caravan, p7)` — one road plan, bracketed by its entry and return
    /// (`docs/ROADS.md` §5).
    pub const ASTAR_ROAD: u32 = 5;
    /// `PathFinderData::valid_roadcoord@00688740(x, y, from.x, from.y, …)`
    /// — the gate, and the record that carries a candidate's **world
    /// coordinate**; [`CALC_ROAD_COST`] is handed a pooled `PathNode *`,
    /// so the tile a price belongs to is the one admitted just before it.
    pub const VALID_ROADCOORD: u32 = 6;
    /// `PathFinder::calc_road_cost@00686300(node, whoA, whoB, dir, this)` —
    /// `docs/ROADS.md` §5.2's per-node price, answer and all.
    pub const CALC_ROAD_COST: u32 = 7;
}

/// One draw, as the trace records it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    /// The **return address** into the function that drew, normalised to
    /// [`IMAGE_BASE`] — what `report.py` prints as `Class::method+0xNN`.
    pub site: u32,
    /// The `Random *` this stepped, normalised. [`Draw::sync`] is the test
    /// that matters.
    pub rng: u32,
    /// The generator's word **before** the step.
    pub seed: u32,
    /// The caller's caller, and its caller — two more frames of the `ebp`
    /// chain, normalised. Zero where the walk ran out.
    pub up: [u32; 2],
    /// The sim-frame, as `Game::do_frame` counts it; −1 for the setup path.
    pub frame: i64,
    /// The record kind — 1 `get()`, 3 `get(a, b)`, 4 `rand_real`,
    /// 6 `reseed`.
    pub kind: u32,
}

impl Draw {
    /// Whether this stepped `game_random`, which is the whole of the
    /// simulation's stream. A draw that did not is the renderer's and
    /// belongs to no frame's count.
    pub fn sync(&self) -> bool {
        self.rng == IMAGE_BASE + RVA_GAME_RANDOM
    }

    /// **What the draw returned.**
    ///
    /// The record carries the word *before* the step, so the outcome is
    /// recoverable without the game: step the LCG once, then apply
    /// `Random::get(lo, hi)`'s own scaling. Every site the documents cite
    /// is `(0, 0xffff)`, which is [`sim::combat::Rng::roll`].
    ///
    /// This is the only reader of a draw whose outcome **no dump holds** —
    /// a coin inside `Setup::build_empire`, a direction, an idle roll —
    /// and it is what makes a setup draw borrowable (`docs/SYNC.md`
    /// §3.11). `rand_real` and `reseed` return `None` rather than a number
    /// that would be a guess.
    pub fn value(&self) -> Option<i32> {
        if self.kind != 1 && self.kind != 3 {
            return None;
        }
        let mut rng = sim::combat::Rng::new(self.seed);
        Some(rng.roll())
    }
}

/// `Farms::add_animals@008d8f30`'s three draws per animal, at the offsets
/// run39's own trace names: the species coin, the `y` offset and the `x`
/// (`docs/SYNC.md` §3.11). The fourth of the four is `Guy::init_real`'s,
/// inside `Objects::init_unit`, and leaves nothing to borrow.
pub const ADD_ANIMALS_COIN: u32 = 0x008d_8fc2;
pub const ADD_ANIMALS_Y: u32 = 0x008d_9064;
pub const ADD_ANIMALS_X: u32 = 0x008d_90b2;

/// One proxied call and the answer it came back with — a `CALL` record
/// paired with its `RET`.
///
/// This is the only oracle in the project that reports a **function's
/// return value**. The gamelog prints state; a draw hook prints a seed; an
/// `int 3` prints that something ran. A cost function's answer appears in
/// none of them, which is why `docs/PATHFINDER.md` §10 recorded that there
/// is no numeric per-search oracle to switch on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Call {
    /// Which proxy — see [`call_site`].
    pub site: u32,
    /// The `this` pointer, unnormalised. Two searches on one finder share
    /// it; it is the finder's identity, not the caller's.
    pub this: u32,
    /// The eight stack arguments, zero-padded for a site with fewer.
    pub args: [i32; 8],
    /// What the function returned in `eax`.
    pub ret: i32,
    /// The byte behind an out-argument, where the site names one.
    pub out: Option<u8>,
    /// How many proxied calls were open when this one was entered — 0 for
    /// a call nobody proxied the caller of, 1 for a `calc_cost` inside an
    /// `astar_path`.
    pub depth: usize,
    /// The sim-frame.
    pub frame: i64,
}

impl Call {
    /// This call's [`sim::path::CostKey`] — the seven arguments §5 prices
    /// a step by — for a [`call_site::CALC_COST`] record, so the
    /// original's answer can be looked up beside the simulation's.
    pub fn cost_key(&self) -> Option<sim::path::CostKey> {
        if self.site != call_site::CALC_COST {
            return None;
        }
        let a = self.args;
        Some((a[0], a[1], a[2], a[3], a[4], a[5], a[6]))
    }
}

/// One unit's whole air frame, folded out of the three proxies
/// `docs/SYNC.md` §3.9 needs — **the oracle owner 9 has never had.**
///
/// A wild bird is dumped by nothing: it is not a leader's unit, so no
/// `UNITS` record carries it, and its only observable was the coin at
/// `Unit::do_air_physics+0x639`. This is the record that replaces the
/// coin: the goal the frame steered at, the turn rate the bank produced,
/// and where the step landed — per frame, per bird.
///
/// [`Trace::air_frames`] builds it. The three proxies are matched by
/// their **`this`**, which is the same `Unit *` in all three
/// (`bank_aircraft` and `set_new_location` are both called on the unit
/// `do_air_physics` was), so no reliance on record order is needed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AirFrame {
    /// The sim-frame `do_air_physics` was entered on.
    pub frame: i64,
    /// The `Unit *`, unnormalised — a bird's identity across the window.
    pub unit: u32,
    /// `do_air_physics`' two coords: the patrol point, already clamped by
    /// `WorldData::restrict` at the caller.
    pub goal: (i32, i32),
    /// What `do_air_physics` answered.
    pub ret: i32,
    /// Each `air_turn_speed(sign, 0)` of the frame as `(sign, answer)`,
    /// in the order they returned. Empty when the bank was settled and
    /// `bank_aircraft` took neither turning arm.
    pub turn_speed: Vec<(i32, i32)>,
    /// `set_new_location`'s `(x, y)` — where the step landed, after
    /// `WorldData::restrict` clamped a refused one. `None` only if the
    /// window closed inside the frame.
    pub to: Option<(i32, i32)>,
}

/// A parsed `rontrace.log`.
#[derive(Clone, Debug)]
pub struct Trace {
    /// The image base the run reported.
    pub base: u32,
    /// Every draw, in file order.
    pub draws: Vec<Draw>,
    /// Each `FRAME` record: the sim-frame and `game_random`'s word at its
    /// `do_frame` entry — the same pairing `gamelog::Log::frame_seeds`
    /// gives from a `DUMP_ALL` dump, from the other side.
    pub frames: Vec<(i64, u32)>,
    /// Every proxied call, in the order each **returned** — so a callee
    /// precedes the caller it was nested in.
    pub calls: Vec<Call>,
    /// Every `HIT`: a function's virtual address, folded to the Ghidra
    /// export's numbering, and the sim-frame it was entered on. **A
    /// function appears at most once per arming** — outside
    /// `rontrace.cfg`'s `window=` the arming is one-shot from attach, so
    /// on a whole-run capture this is one record per function, carrying
    /// the frame it was *first* reached on; inside a window every listed
    /// function is re-armed at the head of each frame and hits again.
    /// `-1` is the setup path, before the first `do_frame`.
    ///
    /// This is the coverage half of the instrument
    /// (`tools/trace/README.md`), and what makes "no run has ever entered
    /// this function" an assertion rather than a reading.
    pub hits: Vec<(u32, i64)>,
    /// Every `PROXIED` record: the site id and the **virtual address** it
    /// patched, folded to the export's numbering. Since the seventh pass
    /// the tracer writes the id; a log older than that carries `0` for
    /// every site, and [`Trace::site_va`] then answers nothing.
    pub proxied: Vec<(u32, u32)>,
    /// `RON_COLLIDE_PROBE`'s identity records (INFO 15): `(UnitData *,
    /// o, who)`, one per proxied call whose `this` is a unit.
    ///
    /// The collision proxies are `__thiscall` on a pointer and the dump
    /// beside them is keyed on `(who, o)`, so without this a record of
    /// which unit refused a step is a heap address and nothing more. The
    /// pair is read off `+0xa` (a short) and `+0x9` (a byte), which is
    /// what `UnitData::will_be_corner@00609fa0` itself indexes
    /// `units[who][o]` with.
    pub unit_ids: Vec<(u32, i32, i32)>,
}

impl Trace {
    /// The `(who, o)` behind a `this` pointer, from the collide probe's
    /// own identity records. `None` for a log no such build wrote, and
    /// for a `this` that is not a unit — `CollCheck::collide_here`'s is a
    /// stack slot.
    pub fn unit_of(&self, this: u32) -> Option<(i32, i32)> {
        self.unit_ids
            .iter()
            .find(|(p, _, _)| *p == this)
            .map(|&(_, o, who)| (who, o))
    }

    /// The address a site id patched in this run, when the log says —
    /// how a test tells a `RON_LEADER_PROBE` log's site 9 from a
    /// `RON_TARGET_PROBE` log's.
    pub fn site_va(&self, site: u32) -> Option<u32> {
        let named = self.proxied.len() == 1 || self.proxied.iter().any(|(i, _)| *i != 0);
        named
            .then(|| {
                self.proxied
                    .iter()
                    .find(|(i, _)| *i == site)
                    .map(|(_, va)| *va)
            })
            .flatten()
    }

    /// Permissive diagnostic parser; use `parse_finalized` or `read` for evidence.
    /// `None` if the header is not a trace.
    pub fn parse(bytes: &[u8]) -> Option<Trace> {
        if bytes.len() < 32 {
            return None;
        }
        let word = |off: usize| -> u32 {
            u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
        };
        if word(0) != MAGIC {
            return None;
        }
        let base = word(8);
        // A run that loaded elsewhere is folded back, so a site is the
        // same number as the Ghidra export's.
        let norm = |va: u32| -> u32 {
            if va == 0 {
                0
            } else {
                va.wrapping_sub(base).wrapping_add(IMAGE_BASE)
            }
        };
        let mut t = Trace {
            base,
            draws: Vec::new(),
            frames: Vec::new(),
            calls: Vec::new(),
            hits: Vec::new(),
            proxied: Vec::new(),
            unit_ids: Vec::new(),
        };
        // CALL and RET nest, so one stack pairs them: a RET belongs to the
        // innermost open CALL of the same site. A window that opens mid
        // search leaves a RET with no CALL and one that closes mid search
        // leaves a CALL with no RET; both are dropped rather than half
        // reported.
        let mut open: Vec<(u32, u32, [i32; 4], i64)> = Vec::new();
        let mut off = 32;
        // `emit` writes the frame counter into slot 7, and it is `-1`
        // before the first `do_frame`.
        while off + 32 <= bytes.len() {
            let r: Vec<u32> = (0..8).map(|i| word(off + i * 4)).collect();
            let frame = i64::from(r[7] as i32);
            if r[0] == 0 {
                t.hits.push((norm(r[1]), frame));
            } else if r[0] == 5 && r[1] == 12 {
                // `a` is an RVA, not a VA: fold it to the export's base.
                t.proxied.push((r[5], r[2].wrapping_add(IMAGE_BASE)));
            } else if r[0] == 5 && r[1] == 15 {
                // INFO 15: `o` is a signed short in a u32 slot.
                let o = i32::from(r[4] as u16 as i16);
                if !t.unit_ids.iter().any(|&(p, _, _)| p == r[3]) {
                    t.unit_ids.push((r[3], o, r[5] as i32));
                }
            } else if r[0] == 2 {
                t.frames.push((i64::from(r[1] as i32), r[2]));
            } else if r[0] == 7 {
                open.push((
                    r[1],
                    r[2],
                    [r[3] as i32, r[4] as i32, r[5] as i32, r[6] as i32],
                    frame,
                ));
            } else if r[0] == 8 {
                if let Some(i) = open.iter().rposition(|c| c.0 == r[1]) {
                    let (site, this, lo, cframe) = open.remove(i);
                    t.calls.push(Call {
                        site,
                        this,
                        args: [
                            lo[0],
                            lo[1],
                            lo[2],
                            lo[3],
                            r[3] as i32,
                            r[4] as i32,
                            r[5] as i32,
                            0,
                        ],
                        ret: r[2] as i32,
                        out: (r[6] != u32::MAX).then_some(r[6] as u8),
                        depth: i,
                        frame: cframe,
                    });
                }
            } else if DRAW_KINDS.contains(&r[0]) {
                t.draws.push(Draw {
                    site: norm(r[1]),
                    rng: norm(r[2]),
                    seed: r[3],
                    up: [norm(r[4]), norm(r[5])],
                    frame,
                    kind: r[0],
                });
            }
            off += 32;
        }
        Some(t)
    }

    /// Validate a finalized trace before it can serve as fidelity evidence.
    /// `parse` remains permissive for inspecting incomplete diagnostic streams.
    pub fn parse_finalized(bytes: &[u8]) -> std::io::Result<Trace> {
        let invalid = |message| std::io::Error::new(std::io::ErrorKind::InvalidData, message);
        if bytes.len() < 32 || !bytes.len().is_multiple_of(32) {
            return Err(invalid("incomplete trace header or record"));
        }
        let word = |record: &[u8], slot: usize| {
            u32::from_le_bytes(record[slot * 4..slot * 4 + 4].try_into().unwrap())
        };
        if word(bytes, 0) != MAGIC {
            return Err(invalid("not a trace header"));
        }
        if !matches!(word(bytes, 1), 1 | 2) {
            return Err(invalid("unsupported trace version"));
        }
        let mut previous_frame = None;
        for record in bytes[32..].chunks_exact(32) {
            // Writer protocol: INFO (5), I_DROPPED (14), count in slot 2.
            if word(record, 0) == 5 && word(record, 1) == 14 && word(record, 2) != 0 {
                return Err(invalid("trace reports dropped records"));
            }
            if word(record, 0) == 2 {
                // tracer.c emits Game::frame in both slots at do_frame entry.
                // Finalized evidence represents one consecutive run; diagnostics
                // may still inspect discontinuous streams through `parse`.
                let frame = i64::from(word(record, 1) as i32);
                if word(record, 1) != word(record, 7) {
                    return Err(invalid("trace FRAME fields disagree"));
                }
                if previous_frame.is_some_and(|previous| frame != previous + 1) {
                    return Err(invalid("trace FRAME sequence is not consecutive"));
                }
                previous_frame = Some(frame);
            }
        }
        Self::parse(bytes).ok_or_else(|| invalid("not a trace"))
    }

    /// Read finalized evidence. Unrecognized files remain `Ok(None)`;
    /// recognized but malformed or lossy traces return `InvalidData`.
    pub fn read(path: &Path) -> std::io::Result<Option<Trace>> {
        let bytes = std::fs::read(path)?;
        if !bytes.starts_with(&MAGIC.to_le_bytes()) {
            return Ok(None);
        }
        Self::parse_finalized(&bytes).map(Some)
    }

    /// **The pasture's five, read back out of the setup path.**
    ///
    /// `Farms::add_animals` runs inside `Setup::build_empire`, whose stream
    /// the harness does not replay, so its three marks — the species coin
    /// and the two offsets — exist nowhere else: no dump prints an owner-9
    /// object at all (`docs/SYNC.md` §3.6). [`Draw::value`] recovers them
    /// from the seeds the records carry, which is what makes the five
    /// **borrowable** the way a sibling dump's heights and herds are.
    ///
    /// Returns one `Vec` a pasture, in the order the setup created them,
    /// each of [`sim::farms::FARM_ANIMALS`] seeds. A trace that opened
    /// after the setup — every windowed capture — returns nothing, and the
    /// simulation then keeps its stand-in.
    pub fn add_animals(&self) -> Vec<Vec<sim::farms::AnimalSeed>> {
        let sites = [ADD_ANIMALS_COIN, ADD_ANIMALS_Y, ADD_ANIMALS_X];
        let marks: Vec<&Draw> = self
            .draws
            .iter()
            .filter(|d| d.sync() && d.frame < 0 && sites.contains(&d.site))
            .collect();
        let mut seeds: Vec<sim::farms::AnimalSeed> = Vec::new();
        // The three are consecutive and in this order; anything else is a
        // trace whose window clipped the run, and a partial animal is
        // dropped rather than half-borrowed.
        for t in marks.chunks(3) {
            let [coin, y, x] = t else { break };
            if (coin.site, y.site, x.site) != (sites[0], sites[1], sites[2]) {
                break;
            }
            let (Some(coin), Some(y), Some(x)) = (coin.value(), y.value(), x.value()) else {
                break;
            };
            let fold =
                |v: i32| v.rem_euclid(sim::farms::ANIMAL_SPREAD) - sim::farms::ANIMAL_SPREAD / 2;
            seeds.push(sim::farms::AnimalSeed {
                chicken: coin & 1 == 0,
                dy: fold(y),
                dx: fold(x),
            });
        }
        seeds
            .chunks(sim::farms::FARM_ANIMALS as usize)
            .filter(|c| c.len() == sim::farms::FARM_ANIMALS as usize)
            .map(<[sim::farms::AnimalSeed]>::to_vec)
            .collect()
    }

    /// The sim-frame a function was **first entered** on, or `None` if no
    /// frame of this capture entered it at all. `-1` is the setup path.
    ///
    /// The address is the Ghidra export's — [`Trace::parse`] folds a run
    /// that loaded elsewhere back onto [`IMAGE_BASE`].
    pub fn first_entry(&self, va: u32) -> Option<i64> {
        self.hits
            .iter()
            .filter(|(a, _)| *a == va)
            .map(|(_, f)| *f)
            .min()
    }

    /// One site's proxied calls on one sim-frame, in the order they
    /// returned.
    pub fn calls_in(&self, frame: i64, site: u32) -> Vec<Call> {
        self.calls
            .iter()
            .filter(|c| c.frame == frame && c.site == site)
            .copied()
            .collect()
    }

    /// The road search's priced nodes on one sim-frame, in the order the
    /// original priced them (`docs/ROADS.md` §7.2).
    ///
    /// `calc_road_cost` is handed a pooled `PathNode *`, so its own record
    /// carries no coordinate at all; the tile it prices is the one the
    /// `valid_roadcoord` that returned just before it admitted. That is
    /// why both are proxied — the gate is where a candidate's world
    /// coordinate is, and the price is where the answer is.
    pub fn road_nodes(&self, frame: i64) -> Vec<sim::roads::RoadCostMark> {
        let mut out = Vec::new();
        let mut gate: Option<Call> = None;
        for c in self.calls.iter().filter(|c| c.frame == frame) {
            match c.site {
                call_site::VALID_ROADCOORD => gate = Some(*c),
                call_site::CALC_ROAD_COST => {
                    if let Some(g) = gate.take() {
                        out.push(sim::roads::RoadCostMark {
                            to: (g.args[0], g.args[1]),
                            from: (g.args[2], g.args[3]),
                            dir: c.args[3],
                            cost: c.ret,
                        });
                    }
                }
                _ => {}
            }
        }
        out
    }

    /// Every air frame in the trace, ordered by `(frame, unit)` — the
    /// per-frame flight record of every unit that flew inside the
    /// `callwin`.
    ///
    /// `do_air_physics` brackets the frame and the other two proxies are
    /// matched to it by `this`, so a `set_new_location` taken by a
    /// walking villager on the same frame is left out: the fold keeps
    /// only the calls whose unit flew.
    pub fn air_frames(&self) -> Vec<AirFrame> {
        let mut out: Vec<AirFrame> = self
            .calls
            .iter()
            .filter(|c| c.site == call_site::DO_AIR_PHYSICS)
            .map(|c| AirFrame {
                frame: c.frame,
                unit: c.this,
                goal: (c.args[1], c.args[2]),
                ret: c.ret,
                turn_speed: Vec::new(),
                to: None,
            })
            .collect();
        out.sort_by_key(|a| (a.frame, a.unit));
        for c in &self.calls {
            let key = (c.frame, c.this);
            let Ok(i) = out.binary_search_by_key(&key, |a| (a.frame, a.unit)) else {
                continue;
            };
            match c.site {
                call_site::AIR_TURN_SPEED => out[i].turn_speed.push((c.args[0], c.ret)),
                call_site::SET_NEW_LOCATION => out[i].to = Some((c.args[0], c.args[1])),
                _ => {}
            }
        }
        out
    }

    /// Where each flying unit was **put down**, and on which frame: the
    /// last `set_new_location(x, y, 1, 1)` it took before its first air
    /// frame, which no `do_air_physics` brackets.
    ///
    /// That call is `Unit::init@00612100`'s own, and the position it
    /// carries is already snapped — `div_3_table[p >> 4] · 0x30 + 0x18`,
    /// the centre of the unit's 48-unit tile. For a wild bird, whose
    /// patrol point is the cell centre it hatched on, that is the cell
    /// centre plus twenty-four on each axis; `docs/SYNC.md` §3.9.
    pub fn air_births(&self) -> std::collections::BTreeMap<u32, (i64, (i32, i32))> {
        let mut first: std::collections::BTreeMap<u32, i64> = Default::default();
        for a in self.air_frames() {
            first.entry(a.unit).or_insert(a.frame);
        }
        let mut out = std::collections::BTreeMap::new();
        for c in &self.calls {
            if c.site != call_site::SET_NEW_LOCATION || (c.args[2], c.args[3]) != (1, 1) {
                continue;
            }
            let Some(&f0) = first.get(&c.this) else {
                continue;
            };
            if c.frame > f0 {
                continue;
            }
            let e = out
                .entry(c.this)
                .or_insert((c.frame, (c.args[0], c.args[1])));
            if c.frame >= e.0 {
                *e = (c.frame, (c.args[0], c.args[1]));
            }
        }
        out
    }

    /// The sync-stream draws of one sim-frame, in the order they were
    /// taken. `-1` is the setup path.
    pub fn frame_draws(&self, frame: i64) -> Vec<Draw> {
        self.draws
            .iter()
            .filter(|d| d.frame == frame && d.sync())
            .copied()
            .collect()
    }

    /// The sync-stream draws of one sim-frame whose **site** lies inside
    /// `[lo, hi)` — one function's own draws, with everything its callees
    /// took left out. `hi` is the next function's address in the export.
    pub fn run_in(&self, frame: i64, lo: u32, hi: u32) -> Vec<Draw> {
        self.frame_draws(frame)
            .into_iter()
            .filter(|d| (lo..hi).contains(&d.site))
            .collect()
    }

    /// One draw's name, from [`SITES`]: the string the simulation's own
    /// `Sim::mark` writes at the same site, or the bare address when
    /// nothing models it. An unmodelled draw therefore reads as a hex
    /// number in the comparison, which is what makes a hole in the
    /// simulation legible rather than silent.
    pub fn label(&self, d: &Draw) -> String {
        SITES
            .iter()
            .find(|(site, via, _)| *site == d.site && via.is_none_or(|v| d.up.contains(&v)))
            .map_or_else(
                || format!("{:x}", d.site),
                |(_, _, name)| (*name).to_string(),
            )
    }

    /// A frame's sync draws as a **sequence of names** — the original's
    /// side of the comparison [`crate::diff::mark_sites`] builds for ours.
    ///
    /// This is the whole point of the naming table. `--diff` prints a
    /// per-phase count and `--trace` a per-site one, and lining the two up
    /// has been an eye exercise; two `Vec<String>`s of the same vocabulary
    /// are an `assert_eq!`, and where they part is where the simulation's
    /// frame parts from the original's.
    pub fn labels(&self, frame: i64) -> Vec<String> {
        self.frame_draws(frame)
            .iter()
            .map(|d| self.label(d))
            .collect()
    }

    /// A frame's sync draws folded into consecutive runs of one site —
    /// `5f6446 ×3, 5f6468, 5f665c ×4, …` — which is the harness's answer
    /// to `report.py … sites` for a *sequence* rather than a total, and
    /// the shape `Built::phase_fold` prints for our own side.
    ///
    /// Sites are bare addresses: naming them needs the Ghidra export's
    /// `INDEX.tsv`, which never enters this repo. `report.py` is where a
    /// name comes from.
    pub fn site_fold(&self, frame: i64) -> String {
        let draws = self.frame_draws(frame);
        let mut out: Vec<String> = Vec::new();
        let mut i = 0;
        while i < draws.len() {
            let mut j = i;
            while j + 1 < draws.len() && draws[j + 1].site == draws[i].site {
                j += 1;
            }
            let n = j - i + 1;
            out.push(if n > 1 {
                format!("{:x} ×{n}", draws[i].site)
            } else {
                format!("{:x}", draws[i].site)
            });
            i = j + 1;
        }
        out.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One header and three records, hand-built: a `FRAME`, a
    /// `game_random` draw and one of the renderer's.
    fn bytes(recs: &[[u32; 8]]) -> Vec<u8> {
        let mut v = Vec::new();
        for r in recs {
            for w in r {
                v.extend_from_slice(&w.to_le_bytes());
            }
        }
        v
    }

    #[test]
    fn finalized_traces_reject_partial_unknown_and_lossy_records() {
        for version in [1, 2] {
            let header = [MAGIC, version, IMAGE_BASE, 0, 0, 0, 0, 0];
            let valid = bytes(&[header, [2, 7, 123, 0, 0, 0, 0, 7]]);
            assert_eq!(
                Trace::parse_finalized(&valid).unwrap().frames,
                vec![(7, 123)]
            );
            for cut in 1..32 {
                assert!(Trace::parse_finalized(&valid[..valid.len() - cut]).is_err());
            }
            let lost = bytes(&[header, [5, 14, 3, 0, 0, 0, 0, 7]]);
            assert!(
                Trace::parse(&lost).is_some(),
                "diagnostics remain inspectable"
            );
            assert!(
                Trace::parse_finalized(&lost)
                    .unwrap_err()
                    .to_string()
                    .contains("dropped")
            );
            assert!(Trace::parse_finalized(&bytes(&[header, [5, 14, 0, 0, 0, 0, 0, 7]])).is_ok());
        }
        let unknown = bytes(&[[MAGIC, 99, IMAGE_BASE, 0, 0, 0, 0, 0]]);
        assert!(
            Trace::parse_finalized(&unknown)
                .unwrap_err()
                .to_string()
                .contains("version")
        );
        assert!(Trace::parse_finalized(b"RONT").is_err());
    }

    #[test]
    fn finalized_frames_require_a_consecutive_single_run() {
        for version in [1, 2] {
            let header = [MAGIC, version, IMAGE_BASE, 0, 0, 0, 0, 0];
            let frame = |n: u32| [2, n, 123, 0, 0, 0, 0, n];
            // Arbitrary starts and non-frame records between frames are valid.
            let valid = bytes(&[header, frame(71), [5, 14, 0, 0, 0, 0, 0, 71], frame(72)]);
            assert_eq!(Trace::parse_finalized(&valid).unwrap().frames.len(), 2);
            for next in [71, 73, 0] {
                let invalid = bytes(&[header, frame(71), frame(next)]);
                assert!(Trace::parse(&invalid).is_some());
                assert_eq!(
                    Trace::parse_finalized(&invalid).unwrap_err().kind(),
                    std::io::ErrorKind::InvalidData,
                    "duplicate, gap, or reset ending at {next}"
                );
            }
            let mut mismatched = frame(71);
            mismatched[7] = 72;
            assert!(Trace::parse_finalized(&bytes(&[header, mismatched])).is_err());
            // Interpret the writer's frame as signed, without integer overflow.
            assert!(Trace::parse_finalized(&bytes(&[header, frame(u32::MAX), frame(0)])).is_ok());
            assert!(
                Trace::parse_finalized(&bytes(&[
                    header,
                    frame(i32::MAX as u32),
                    frame(i32::MIN as u32)
                ]))
                .is_err()
            );
        }
    }

    #[test]
    fn finalized_file_reader_propagates_validation_errors() {
        let path = std::env::temp_dir().join(format!(
            "attrition-trace-boundary-{}.log",
            std::process::id()
        ));
        let valid = bytes(&[[MAGIC, 2, IMAGE_BASE, 0, 0, 0, 0, 0]]);
        std::fs::write(&path, &valid).unwrap();
        assert!(Trace::read(&path).unwrap().is_some());
        let mut partial = valid;
        partial.push(1);
        std::fs::write(&path, partial).unwrap();
        assert_eq!(
            Trace::read(&path).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        std::fs::write(&path, b"unrecognized file").unwrap();
        assert!(Trace::read(&path).unwrap().is_none());
        std::fs::remove_file(path).unwrap();
    }

    const GAME_RANDOM: u32 = IMAGE_BASE + RVA_GAME_RANDOM;

    #[test]
    fn a_trace_parses_and_keeps_only_the_sync_stream() {
        let log = bytes(&[
            [MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 48_233, 0, 0xffff_ffff],
            [2, 0, 0x1111_1111, 3, 0, 0, 0, 0],
            [3, 0x005f_6446, GAME_RANDOM, 0x9c59_1b2b, 0, 0, 0, 0],
            // The renderer's — a different `Random *`, so not the stream.
            [3, 0x0022_3908, 0x0022_3908, 0xdead_beef, 0, 0, 0, 0],
            [3, 0x005f_6468, GAME_RANDOM, 0x70fc_74f0, 0, 0, 0, 0],
            [0, 0x005f_6010, 7, 0, 0, 0, 0, 0],
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(t.base, IMAGE_BASE);
        assert_eq!(t.frames, vec![(0, 0x1111_1111)]);
        assert_eq!(t.draws.len(), 3, "the HIT and the FRAME are not draws");
        let f0 = t.frame_draws(0);
        assert_eq!(f0.len(), 2, "the renderer's is not the sync stream");
        assert_eq!(f0[0].seed, 0x9c59_1b2b);
        assert_eq!(f0[1].site, 0x005f_6468);
        // …but the HIT is kept as coverage, on the frame it was entered.
        assert_eq!(t.hits, vec![(0x005f_6010, 0)]);
        assert_eq!(t.first_entry(0x005f_6010), Some(0));
        assert_eq!(t.first_entry(0x005f_6446), None, "a draw site is not a hit");
    }

    /// **Every row of [`SITES`] says its own address**, checked against
    /// the decompile export's `INDEX.tsv` — the whole table, in a
    /// millisecond, every commit.
    ///
    /// The table's rows are the one place in this crate where a *number*
    /// carries a *name*, and a wrong pairing is the one error the
    /// differential check cannot report: a label put on the wrong address
    /// makes a frame read as agreeing, or as parting somewhere else, and
    /// nothing else in the suite looks at the pairing at all. Item 483
    /// added two rows and moved Great Lakes' sequence word seven frames
    /// **without changing one line of simulation**, which is exactly the
    /// shape a wrong row would also have. So the row is checked rather
    /// than trusted: the address is resolved to its containing function in
    /// the export, and the label's own `Name+0xoff` must be that function
    /// and that offset.
    ///
    /// A chain-qualified row is checked at both ends — its `via` against
    /// the label's last link — which is what catches a `via` copied from
    /// the neighbouring entry. Two labels name a link without an offset
    /// (`< do_cast`, `< do_trade`, where the intermediate frame is the
    /// disambiguator and not the caller); for those the name is checked
    /// and the offset is not.
    ///
    /// Made to fail four ways before it landed: one digit off each of the
    /// two new sites, `SITE_PUNCTURE_X`'s address swapped onto
    /// `SITE_FIRST_WOUND`'s row, and `SITE_TURN_NEAR`'s `via` replaced by
    /// its neighbour's. **And it failed for real on its first run**, on
    /// `SITE_FIRST_WOUND_FLOCK` — §39.3.
    ///
    /// Skips loudly without the export, the way
    /// `sim`'s `every_cited_address_names_its_function` does — the export
    /// never enters this repo.
    #[test]
    fn every_site_s_address_is_the_function_its_label_names() {
        let home = std::env::var("HOME").unwrap_or_default();
        let index_path = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
        let Ok(index) = std::fs::read_to_string(&index_path) else {
            eprintln!("skipping: no {index_path} (the Ghidra export is not on this machine)");
            return;
        };
        // `addr -> name`, ordered, so the containing function of a site is
        // the last entry at or below it.
        let funcs: std::collections::BTreeMap<u32, String> = index
            .lines()
            .filter_map(|l| {
                let mut it = l.split('\t');
                let addr = u32::from_str_radix(it.next()?, 16).ok()?;
                Some((addr, it.next()?.to_string()))
            })
            .collect();
        assert!(funcs.len() > 40_000, "the export's index is 48k functions");

        // One link of a label: `Name+0xoff`, or a bare name.
        let check = |what: &str, addr: u32, link: &str| -> Option<String> {
            let (name, off) = match link.split_once("+0x") {
                Some((n, o)) => (n, u32::from_str_radix(o, 16).ok()),
                None => (link, None),
            };
            let (&at, real) = funcs.range(..=addr).next_back()?;
            // The export strips template arguments the way `docs_guard`
            // does; a label never carries them.
            let real = real.split('<').next().unwrap_or(real);
            if !real.ends_with(name) {
                return Some(format!(
                    "{what} {addr:#010x} is inside `{real}@{at:08x}`, and the label says `{name}`"
                ));
            }
            match off {
                Some(o) if addr - at != o => Some(format!(
                    "{what} {addr:#010x} is `{real}+{:#x}` and the label says `+{o:#x}`",
                    addr - at
                )),
                _ => None,
            }
        };

        let mut failures = Vec::new();
        for (site, via, label) in SITES {
            let links: Vec<&str> = label.split(" < ").collect();
            if let Some(f) = check(label, *site, links[0]) {
                failures.push(f);
            }
            // The `via` is the frame the chain walk matches on, and the
            // label's last link is how a reader spells it.
            if let (Some(v), Some(last)) = (via, links.last().filter(|_| links.len() > 1))
                && let Some(f) = check(label, *v, last)
            {
                failures.push(format!("{f} (the chain's `via`)"));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// A run that loaded somewhere other than `0x400000` folds back, so a
    /// site is the same number as the export's.
    #[test]
    fn a_relocated_run_normalises_to_the_export_s_addresses() {
        let base = 0x0100_0000;
        let log = bytes(&[
            [MAGIC, 1, base, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            [3, base + 0x001f_6446, base + RVA_GAME_RANDOM, 7, 0, 0, 0, 0],
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(t.draws[0].site, 0x005f_6446);
        assert!(t.draws[0].sync(), "and it is still the sync stream");
    }

    /// A hit's address folds back the same way a draw's does, and
    /// `first_entry` answers the **earliest** arming — a windowed capture
    /// re-arms every function at the head of every frame in the window.
    #[test]
    fn first_entry_is_the_earliest_arming_of_a_relocated_hit() {
        let base = 0x0100_0000;
        let log = bytes(&[
            [MAGIC, 1, base, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            [0, base + 0x002c_1be0, 7, 0, 0, 0, 0, 12],
            [0, base + 0x002c_1be0, 7, 0, 0, 0, 0, 13],
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(t.hits.len(), 2);
        assert_eq!(t.first_entry(0x006c_1be0), Some(12));
    }

    /// `run_in` isolates one function's own draws, and `site_fold` reads
    /// them as runs.
    #[test]
    fn run_in_isolates_a_function_and_site_fold_reads_it_as_runs() {
        let d = |site: u32| [3, site, GAME_RANDOM, 0, 0, 0, 0, 0];
        let log = bytes(&[
            [MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            d(0x005f_6446),
            d(0x005f_6446),
            d(0x005f_6468),
            // A callee's, outside `think_scout`'s range.
            d(0x005f_2800),
            d(0x005f_665c),
        ]);
        let t = Trace::parse(&log).expect("a trace");
        let mine = t.run_in(0, 0x005f_6010, 0x005f_6e40);
        assert_eq!(
            mine.iter().map(|d| d.site).collect::<Vec<_>>(),
            vec![0x005f_6446, 0x005f_6446, 0x005f_6468, 0x005f_665c],
            "the callee's draw is not this function's"
        );
        assert_eq!(t.site_fold(0), "5f6446 ×2, 5f6468, 5f2800, 5f665c");
    }

    /// **A setup draw's outcome, recovered from the seed it carries.**
    ///
    /// The three records are run39's own first animal, verbatim
    /// (`report.py <log> draws setup`): seeds `a236f580`, `588a6adf`,
    /// `51d23ab2`, which return 27358, 15025 and 55912 — an even coin, so
    /// a chicken, and `% 0x180 − 0xc0` on each of the other two, so
    /// `(dy −143, dx 40)`. Nothing else in the capture holds any of it:
    /// no dump prints an owner-9 object at all (`docs/SYNC.md` §3.11).
    #[test]
    fn a_setup_draw_s_outcome_is_recovered_from_the_seed_it_carries() {
        let setup = |site: u32, seed: u32| [3u32, site, GAME_RANDOM, seed, 0, 0, 0, 0xffff_ffff];
        let log = bytes(&[
            [MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            setup(ADD_ANIMALS_COIN, 0xa236_f580),
            setup(ADD_ANIMALS_Y, 0x588a_6adf),
            setup(ADD_ANIMALS_X, 0x51d2_3ab2),
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(
            t.draws.iter().map(Draw::value).collect::<Vec<_>>(),
            vec![Some(27358), Some(15025), Some(55912)],
            "the values report.py prints beside these three records"
        );
        // Four animals short of a pasture, so nothing is borrowable yet.
        assert_eq!(t.add_animals(), Vec::<Vec<sim::farms::AnimalSeed>>::new());

        let mut recs = vec![[MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 1, 0, 0xffff_ffff]];
        for _ in 0..sim::farms::FARM_ANIMALS {
            recs.push(setup(ADD_ANIMALS_COIN, 0xa236_f580));
            recs.push(setup(ADD_ANIMALS_Y, 0x588a_6adf));
            recs.push(setup(ADD_ANIMALS_X, 0x51d2_3ab2));
        }
        let five = Trace::parse(&bytes(&recs)).expect("a trace");
        let got = five.add_animals();
        assert_eq!(got.len(), 1, "one pasture");
        assert_eq!(
            got[0],
            vec![
                sim::farms::AnimalSeed {
                    chicken: true,
                    dy: -143,
                    dx: 40,
                };
                sim::farms::FARM_ANIMALS as usize
            ]
        );
    }

    #[test]
    fn something_that_is_not_a_trace_is_refused() {
        assert!(Trace::parse(b"not a trace at all, no header here").is_none());
        assert!(Trace::parse(b"short").is_none());
    }
}
