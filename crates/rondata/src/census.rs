//! The census by layer, and the board's number: the simulation layer's
//! backed share (item 1467; DECISIONS 63 (iv), the sweep lane's score).
//!
//! `tools/census.py --layers` files every function the Ghidra export lists
//! under a layer — simulation, AI, engine, interface or unknown — by its
//! PDB source path, and marks it **backed** when a document's coverage
//! section cites it in a diff-backed span or a sweep `#[test]` in
//! `crates/sim` (one that runs the original under the emulator) names it.
//! The rule and what it cannot count are the tool's docstring.
//!
//! This module pins the number, as [`crate::blind`] pins the blind list:
//! a document's new coverage citation, or a new sweep, moves it, and the
//! landing that moves it re-pins it here and says so; the handoff's
//! `Census:` line is read against the same two constants
//! (`diff::floors`). The export, the PDB and `llvm-pdbutil` live outside
//! the repo; a machine without them skips and says so.

/// Simulation-layer functions a coverage section's diff or a sweep backs (item 2117: 566 -> 571,
/// `docs/GOLDEN.md` §77.1 — `think_attack`, `add_to_army`, `get_unit`, `go_to_unit`, `go_to`; item
/// 2129: 571 -> 581, the sweep's fifty-first batch; item 2149: 581 -> 591, the fifty-second; item
/// 2131: 591 -> 602, `docs/GOLDEN.md` §78.1 — `get_final_loc`, `get_loc_to`, `action_move_near`,
/// `action_move_to`, `process_move_to`, `compute_form`, `add_group_move_order`, and chapter twelve's
/// follow line, `process_follow`, `action_follow`, `add_follow_order`, `do_follow`; item 2160: 602 -> 612, the sweep's
/// fifty-third batch; item 2170: 612 -> 622, the fifty-fourth; item 2181: 622 -> 632, the
/// fifty-fifth; item 2165: 632 -> 639, `docs/GOLDEN.md` §79.1 — `Unit::init`'s Bark arm,
/// `set_stance`, `fight`'s re-search, `issue_stance`, `process_stance`, `action_stance`,
/// `get_stance_option`, `get_stance_type`; item 2187: 639 -> 649, the sweep's fifty-sixth batch; item
/// 2200: 649 -> 659, a second sweep lane's ten with a port consumer; item 2211: 659 -> 669, the
/// sweep's fifty-seventh batch; item 2205: 669 -> 670, `compare_target`, by golden chapter sixty-four's
/// sea arm, `docs/GOLDEN.md` §80.1; item 2223: 670 -> 680, the sweep's fifty-eighth batch; item 2217:
/// 680 -> 690, the second sweep lane's ten with a port consumer; item 2241: 690 -> 700, the sweep's
/// fifty-ninth batch; item 2253: 700 -> 710, the sweep's sixtieth batch; item 2247: 710 -> 720, the second
/// sweep lane's next ten, the territory family through `obj --sparse`; item 2235: 720 -> 721,
/// `UnitData::get_action`, by the city alarm's worker arm, `docs/GOLDEN.md` §81.1; item 2271: 721 ->
/// 731, the sweep's sixty-first batch, the backups and restores through `obj --ptrarg-null`; item
/// 2276: 731 -> 741, the second sweep lane's ten, the squad and city chains through `obj --link`; item
/// 2281: 741 -> 751, the sweep's sixty-second batch, seven of its ten through `obj --record`; item
/// 2298: 751 -> 761, the sweep's sixty-third batch, the unbuilt lists, the speed and the spells;
/// item 2287: 761 -> 771, the second sweep lane's ten with a port consumer, five through `obj --record`;
/// item 2309: 771 -> 782, the sweep's sixty-fourth batch, the leaders-table walks through `obj --record IDX@S`;
/// item 2325: 782 -> 793, the sweep's sixty-fifth batch, eleven, `get_inside`'s out-pointer through
/// `obj --outarg` and `memcpy` through `--stub-cdecl`;
/// item 2314: 793 -> 803, the second sweep lane's ten with a port consumer, four through `obj --rows N`;
/// item 2336: 803 -> 813, the sweep's sixty-sixth batch, a `bool`'s low byte through `obj --ret al` and
/// `timeGetTime` through `--stub iat`;
/// item 2331: 813 -> 815, chapter sixty-seven's coverage, `train_time` and `get_cost` diff-backed by run845;
/// item 2346: 815 -> 825, the sweep's sixty-seventh batch, `edx` beside `ecx` through `obj --edx`, a table
/// index through `--tame`, a container emptied through `--stub-write`;
/// item 2341: 825 -> 835, the second sweep lane's next ten with a port consumer, the order adders and the
/// gaia walks, five through `obj --record-order`;
/// item 2362: 835 -> 845, the sweep's sixty-eighth batch, eight `vtordisp` thunks through `fwd`'s
/// `vtordisp` token, `set_shooter_vis` and `issue_unqueue`;
/// item 2374: 845 -> 873, the sweep's sixty-ninth batch, the twenty-eight `vtordisp` thunks left, one `fwd`
/// line each, the band empty on `census.py --unbacked`;
/// item 2357: 873 -> 874, chapter sixty-eight's coverage, `get_military_level_slow` diff-backed by run852;
/// item 2379: 874 -> 884, the sweep's seventieth batch, ten `CommandManager::issue_*` whose command
/// bytes `obj --deref` reads at the call;
/// item 2368: 884 -> 894, the second sweep lane's ten with a port consumer, `Caravan::trade_value`'s eight
/// windows among them, on `--regen`'s kept port block and `--seq`'s terminator;
/// item 2395: 894 -> 904, the sweep's seventy-first batch, ten `issue_*` issuers left after 2379
/// (`CommandManager` six, `GroupOut` four), re-pinned by the commander at booking;
/// item 2400: 904 -> 916, the port-consumer sweep's eleven on `--record-edx`, `--ring` and `--ecx-answer`,
/// `compute_trade` and `kill_group_move` regenerated without hand edits;
/// item 2421: 916 -> 926, the sweep's seventy-second batch, ten `GroupOut::issue_*`, seven of them
/// forwarders written as hand `--port`s;
/// item 2443: 926 -> 936, the sweep's seventy-third batch, eight `GroupOut` and two `CommandManager`
/// issuers, `--fwd-gate` built and most of the list `issue_halt`'s shape after all;
/// item 2454: 936 -> 946, the sweep's seventy-fourth batch, ten issuers on the new `--cmd-gate`,
/// `issue_camera`'s truncation ported as integers;
/// item 2464: 946 -> 956, the sweep's seventy-fifth batch, the `this+OFF` token and ten `CommandManager`
/// issuers, six on `--cmd-gate` and four hand `--port`s reading an object argument;
/// item 2469: 956 -> 963, the sweep's seventy-sixth batch, `--cmd-obj` built (the three object-argument
/// issuers regenerated from it) and seven group-first issuers on `--cmd-gate`;
/// item 2481: 963 -> 965, the sweep's seventy-seventh batch, three of ten, `--cmd-byte` built and three
/// group-first issuers on `--cmd-gate`, the rest of the list read and not the shape;
/// item 2486: 965 -> 974, the sweep's seventy-eighth batch, `--cmd-seh` built and the treaty family's seven
/// on it, `_chat_set` and `_player_speed` by hand on existing flags;
/// item 2495: 974 -> 984, the sweep's seventy-ninth batch, `--chain` built and `PathFinder`'s four
/// forwarders on it, `issue_cannon_time`'s by-value `String` arm and five more, ten of ten;
/// item 2507: 984 -> 994, the sweep's eightieth batch, `mix`'s `stale` token and the three `walk_data` on it,
/// `TurnControl`'s three issuers, two `PathFinder` forwarders and `MapFairness`'s two, ten of ten;
/// item 2513: 994 -> 997, the sweep's eighty-first batch, `--deref`'s call-time cell read and the three `find_*`
/// regenerated on it, `Border::build_border`, `find_global_render_z` and `air_alt_speed`, three of ten;
/// item 2537: 997 -> 1003, the sweep's eighty-second batch, `obj --ret xmm0` and `GuyData`'s five render getters
/// and `get_game_fps_average` on it, six of ten;
/// item 2547: 1003 -> 1013, the sweep's eighty-third batch, `mix --gate-this` and `--vbase` and ten `walk_data`s on
/// them — `Army`, six orders, `Land`, `Groups`, `DeathObjData` — ten of ten;
/// item 2562: 1013 -> 1023, the sweep's eighty-fourth batch, `--gate-this OFF:MASK` and `--len-at` and seven order
/// `walk_data`s, `Player`, `CommandPackage` and `Caravan` on them, ten of ten;
/// item 2567: 1023 -> 1033, the sweep's eighty-fifth batch, `stale` on any step and three `walk_data`s on it, two
/// `walk_rules_data` and five `get_name`, ten of ten;
/// item 2583: 1033 -> 1036, the sweep's eighty-sixth batch, `mix --gate-ne`, `--ret-arg` and `--gate-bare`, and
/// `Rivers::walk_data` and two `get_name`s on them, three new and five regenerated;
/// item 2588: 1036 -> 1046, the sweep's eighty-seventh batch, `mix` `rec` and `--store`, `Constants` and `Dock`
/// `walk_data` on them, five empty-body voids and three `obj`s, ten of ten;
/// item 2593: 1046 -> 1052, the sweep's eighty-eighth batch, `mix --elems` and the `g` tokens, `Types` and
/// `TerrainList` `walk_data` on them and four more, six new;
/// item 2604: 1052 -> 1059, the sweep's eighty-ninth batch, `obj --ret arg:N`, five inlined-`String` `get_name`s
/// and `Doober::find_x_pos`/`find_y_pos` on it, seven new;
/// item 2615: 1059 -> 1060, the sweep's ninetieth batch, `obj --jump` and `Setup::get_starting_citizens` on it;
/// item 2621: 1060 -> 1063, the sweep's ninety-first batch, `obj --table`, `WorldData::get_bottom`,
/// `UnitData::is_corner` and `SubObjectData::play_sound`;
/// item 2633: 1063 -> 1068, the sweep's ninety-second batch, `--answer-win` a call, `is_gathering`, `is_seen`,
/// `Guy::update_z`, `BorderNodes::find` and `is_here`;
/// item 2644: 1068 -> 1074, the sweep's ninety-third batch, `--deref` with a written-byte mask, `issue_come_out`,
/// `action_hotkey`, `default_setting`, `add_to_openlist`, `sort_objects` and `Map::set_file`;
/// item 2655: 1074 -> 1083, the sweep's ninety-fourth batch, nine harvested on the built shapes, no generator change;
/// item 2672: 1083 -> 1087, the sweep's ninety-fifth batch, `fwd --record-order` and `walk --count-only`, three
/// `walk_rules_data`s and `Tribe::walk_data`;
/// item 2688: 1087 -> 1094, the sweep's ninety-seventh batch, `mix --stale-col`, `--vbase2` and `--bit-at`, the six
/// two-displacement order `walk_data`s and `City::walk_data`;
/// item 2698: 1094 -> 1095, the sweep's ninety-eighth batch, `mix --elems count:`, `BuildQueue::walk_data`;
/// item 2683: 1095 -> 1104, the sweep's ninety-sixth batch, ten harvested on the built shapes, no generator change;
/// item 2709: 1104 -> 1107, the sweep's ninety-ninth batch, `--gtab`, `inside_bottom`, `has_damage` and `equals_group`;
/// item 2714: 1107 -> 1110, the sweep's hundredth batch, three harvested on the built shapes, no generator change;
/// item 2749: 1110 -> 1115, the sweep's hundred-and-fourth batch, slot ED's four and `verify_gather_tiles`,
/// `--stub-write … dec`;
/// item 2743: 1115 -> 1125, the sweep's hundred-and-third batch, ten harvested on the built shapes, no generator change;
/// item 2768: 1125 -> 1129, the sweep's hundred-and-fifth batch, four `GroupData` list sweeps;
/// item 2774: 1129 -> 1140, the sweep's hundred-and-sixth batch, ten harvested on the built shapes, no generator
/// change).
/// the thirty-eighth pass: 1233 -> 1224, nine functions that were backed only by a chunk whose script stubbed
/// them (`// Map::point (stub 0) answers …` in a port's comment); `census.py`'s `sweep_backed` now subtracts a
/// script's hooked callees, so a function a sweep answers for is never counted as one it ran.
/// item 2863: 1224 -> 1229, the sweep's hundred-and-nineteenth batch, `--answer-by` built (a stub that answers by
/// `ecx`) for `is_disbanding` and `get_building`, and three more.
/// item 2877: 1229 -> 1238, the sweep's hundred-and-twentieth batch, nine harvested on the built shapes (its
/// `Group::kill_next_order` dropped for 2863's at the merge).
/// item 2893: 1238 -> 1247, the sweep's hundred-and-twenty-second batch, nine harvested on the built shapes.
/// item 2900: 1247 -> 1252 on the merged tree, the sweep's hundred-and-twenty-third batch, `--vtat-keep`,
/// `--link-hi` and `--place-null K:WIN+OFF=V` built, four swept on them.
/// item 2910: 1252 -> 1260 on the merged tree, the sweep's hundred-and-twenty-fourth batch, eight harvested
/// (its two copies of 2900's functions dropped at the merge).
/// item 2959: 1319 -> 1329 on the merged tree, the sweep's hundred-and-thirtieth batch, ten `CommandPackage::process_*`
/// bodies that are not the three shapes (a table field before the action, a virtual slot, the objects table by owner).
/// item 2966: 1329 -> 1339 on the merged tree, the sweep's hundred-and-thirty-first batch, ten harvested
/// (`Unit::execute_events`, `do_idle`, `Supplies::find_supply`, `WorldData::was_seen_wcoord` and six more).
/// item 2990: 1339 -> 1349 on the merged tree, the sweep's hundred-and-thirty-second batch, ten more
/// `CommandPackage::process_*` bodies (`board_ship`, `siege_attack`, `attack`, `build`, `move_near` and five more).
/// item 3004: 1349 -> 1359 on the merged tree, the sweep's hundred-and-thirty-third batch, ten harvested
/// (`Balance::lookup_absolute_name`, `CityData::count_farms`, `BuildData::get_queue` and seven more).
/// item 3014: 1359 -> 1369 on the merged tree, the sweep's hundred-and-thirty-fourth batch, ten more
/// `CommandPackage::process_*` bodies (`ping`, `sell`, `trade`, `quit`, `speed_set` and five more).
/// item 3008: 1369 -> 1379 on the merged tree, the sweep's hundred-and-thirty-fifth batch, ten world and table bodies
/// (`Borders::check_border_blocked`, `Map::is_near_ocean`, `Map::land_dist` and seven more).
/// item 3024: 1379 -> 1389 on the merged tree, the sweep's hundred-and-thirty-sixth batch, ten type and group bodies
/// (`Region::walk_data`, `TypeData::count_discovered`, `ObjectData::get_inside` and seven more).
/// item 3038: 1389 -> 1399 on the merged tree, the sweep's hundred-and-thirty-seventh batch, ten world and map bodies
/// (`Doober::remove_hold_doobers`, `World::set_seen2`, `Fractal::get_height` and seven more).
/// item 3030: 1399 -> 1407 on the merged tree, the sweep's hundred-and-thirty-eighth batch, eight shapes
/// (`CommandPackage::process_rename_city`, `Tribes::find`, `Object::poor_target`, `Wall::swap_team` and four more).
/// item 3044: 1407 -> 1418 on the merged tree, the sweep's hundred-and-thirty-ninth batch, eleven unit and border bodies
/// (`Armies::process_all`, `Borders::process_all`, `Unit::update_z` and eight more).
/// item 3064: 1418 -> 1429 on the merged tree, the sweep's hundred-and-fortieth batch, eleven order-adders, clocks and records
/// (`Unit::add_board_order`, `TurnControl::check_new_frame_solo`, `World::set_edge_flags` and eight more).
/// item 3058: 1429 -> 1434 on the merged tree, the sweep's hundred-and-forty-first batch, five shapes
/// (`Unit::update_armor`, `Build::swap_team`, `SubObject::swap_team`, `AmmoData::walk_data`, `Game::walk_data`;
/// `TurnControl::end_cannon_time` and `Tribes::find`'s import arm beside them).
/// item 3049: 1434 -> 1444 on the merged tree, the sweep's hundred-and-forty-second batch, ten small bodies
/// (`Border::rebuild_visible_list`, `Roads::generate`, `Doober::remove_doobers` and seven more).
/// item 3120: 1450 -> 1454 on the merged tree, the shape lane's four parked shapes
/// (`ObjectData::num_aircraft_here`, `ObjectTypeData::get_wonder_value`, `ObjectsData::find_dock`, `Caravans::new_danger`).
/// item 3134: 1454 -> 1459 on the merged tree, the shape lane's seventh batch
/// (`ObjectsData::find_town_at`, `ObjectsData::find_good_at`, `Build::process_ejection`, `Unit::cavarch_fight`, `Army::send_here`).
/// item 3182: 1469 -> 1471 on the merged tree, the shape lane's tenth batch
/// (`Unit::check_idle`, `ObjectData::is_in_range@006486b0`).
/// item 3200: 1471 -> 1477 on the merged tree, the shape lane's eleventh batch (`Ammo::check_hit`,
/// `Ammo::hit_target`, `Group::action_buildmask`, `Unit::add_build_order`, `World::compute_all_territory`,
/// `Army::release_mustering`).
/// item 3219: 1477 -> 1483 on the merged tree, the shape lane's twelfth batch (`Army::use_scouts`,
/// `Army::use_spies`, `City::count_gather_slots`, `Unit::add_repair_order`, `Unit::find_repair_spot`,
/// `Unit::build_done`).
/// item 3234: 1483 -> 1489 on the merged tree, the shape lane's thirteenth batch (`Armies::find_local_army`,
/// `Wall::check_ever_seen`, `Unit::find_new_bomber_target`, `Unit::do_patrol`, `Unit::add_trade_order`,
/// `Unit::same_damage`).
/// item 3229: 1489 -> 1494 on the merged tree, the nations seat's shape batch (`Caster::process_spells`,
/// `Guy::update_gpiece`, `Unit::find_garrison_build`, `Group::is_attacking_near`,
/// `ObjectsData::find_building_placed_at`).
/// item 3238: 1494 -> 1500 on the merged tree, the shape lane's fourteenth batch
/// (`PathFinderData::valid_roadcoord`, `ObjectsData::find_unit_ordered_with_radius`, `Build::find_gather_tiles`,
/// `Army::do_marching`, `Unit::end_trade_route`, `Unit::add_cast_order`).
/// item 3239: 1500 -> 1506 on the merged tree, the nations seat's third shape batch
/// (`Roads::scan_and_kill_stray_roads`, `Army::engagement`, `Unit::go_inside`, `BuildTypeData::snap_center`,
/// `ObjectsData::find_any_building_at`, `Unit::find_goody_box`).
/// item 3250: 1506 -> 1512 on the merged tree, the shape lane's fifteenth batch (`UnitTypeData::can_garrison`,
/// `Group::target_opportunity`, `Unit::find_new_air_target`, `Build::add_to_city`, `Army::do_forming`,
/// `Unit::think_peasant`).
/// item 3259: 1512 -> 1518 on the merged tree, the nations seat's fourth shape batch (`Animal::think_farm_animal`,
/// `ObjectData::has_nuke`, `Army::do_transporting`, `BuildTypeData::blocked_site`, `Animal::think_bird`,
/// `Army::march_to_target`).
/// item 3268: 1518 -> 1525 on the merged tree, the nations seat's fifth shape batch (`Build::remove_from_city`,
/// `ObjectsData::find_unit_with_radius`, `Object::update_seen`, `Unit::ungroup_move_order`, `Unit::add_gather_order`,
/// `Roads::redo_changed_roads`), and one layout-backed function besides.
/// item 3272: 1525 -> 1531 on the merged tree, the nations seat's sixth shape batch (`City::check_upgrade`,
/// `World::gather_at`, `Build::add_gather_point`, `Unit::bank_aircraft`, `TechType::is_ultimate_preq`,
/// `ObjectData::valid_target_const`).
/// item 3261: 1531 -> 1537 on the merged tree, the shape lane's sixteenth batch (`TypeData::get_preq`,
/// `Group::action_eject_all`, `Guy::set_all_pivots`, `BuildData::construct_time`, `Objects::find_ordered_collision`,
/// `BuildTypeData::is_dock_tile`).
/// item 3281: 1537 -> 1540 on the merged tree, the nations seat's seventh shape batch, three of six
/// (`Unit::think_carry`, `Farms::add`, `Army::find_besieged_city`).
/// item 3289: 1540 -> 1546 on the merged tree, the nations seat's eighth shape batch (`Unit::find_melee_pos`,
/// `Wall::do_construct`, `Unit::do_attack_ground`, `Unit::set_new_location`, `CollCheck::collide_here`,
/// `Unit::do_air_patrol`).
/// item 3282: 1546 -> 1549 on the merged tree, the shape lane's seventeenth batch, three of six (`Build::finished`,
/// `Object::fire_ammo`, `Unit::think_fish`).
/// item 3295: 1549 -> 1555 on the merged tree, the nations seat's ninth shape batch (`ObjectsData::find_building_at`,
/// `Build::start`, `City::assimilate`, `UnitType::find_nearby_spot`, `Unit::explore_goody`, `Unit::do_build`).
/// item 3304: 1555 -> 1561 on the merged tree, the nations seat's tenth shape batch (`Wall::update_local_seen`,
/// `Build::new_library`, `Unit::do_attack`, `Unit::detect_boat_collision`, `Unit::check_target_path`,
/// `City::find_buildings`).
/// item 3320: 1561 -> 1566 on the merged tree, the nations seat's eleventh shape batch, five of six
/// (`SpellType::cast_ambush`, `Unit::find_gather_spot`, `Objects::add_flock`, `Unit::do_air_physics`,
/// `SpellType::cast_create_decoy`).
/// item 3330: 1566 -> 1572 on the merged tree, the nations seat's twelfth shape batch, all six
/// (`Group::action_queue_up`, `UnitData::calc_gather`, `Unit::think_spellcaster`, `TechType::compute_ai_values`,
/// `SpellType::cast_transport`, `Unit::go_around_building`).
/// item 3337: 1572 -> 1576 on the merged tree, the nations seat's thirteenth shape batch, four of six
/// (`Object::replace_hotunit`, `ObjectsData::find`, `World::reveal_fog`, `PathFinder::find_tpath`).
/// item 3339: 1576 -> 1580 on the merged tree, the nations seat's fourteenth shape batch, four of six
/// (`BuildTypeData::find_friends`, `ObjectsData::find_unit`, `PathFinder::find_wpath`, `PathFinder::find_upath`).
/// item 3346: 1580 -> 1586 on the merged tree, the nations seat's fifteenth shape batch, all six
/// (`Wall::process`, `Unit::think`, `Unit::do_repair`, `Unit::think_carry_civilian`, `Object::disband`,
/// `BuildTypeData::blocked_tcoord`).
/// item 3348: 1586 -> 1589 on the merged tree, the second sweep lane's first shape batch, three of six
/// (`Unit::work`, `Unit::target_opportunity`, `Army::find_muster_spot`).
/// item 3358: 1589 -> 1590 on the merged tree, the second sweep lane's second batch, one of six
/// (`Group::action_gather_point`).
/// item 3356: 1590 -> 1596 on the merged tree, the nations seat's sixteenth shape batch, all six
/// (`Unit::plunder`, `Build::check_capture`, `ObjectData::get_damage`, `Object::eject_contents`, `Ammo::do_damage`,
/// `Unit::do_gather`).
/// item 3362: 1596 -> 1601 on the merged tree, the second sweep lane's third batch, five of six
/// (`Group::action_attack`, `Balance::type_damage`, `Cities::capture_city`, `Army::find_target`, `Unit::come_out`).
/// item 3364: 1601 -> 1607 on the merged tree, the nations seat's seventeenth shape batch, all six
/// (`Object::die`, `ObjectsData::find_building`, `SpellTypeData::is_castable`, `BuildTypeData::blocked_location`,
/// `Group::action_alarm`, `Build::activate`).
/// item 3372: 1607 -> 1613 on the merged tree, the second sweep lane's fourth batch, all six
/// (`Unit::suffer_attrition`, `Object::remove_from_inside`, `Unit::check_build_order`, `Balance::compute_modifier`,
/// `Unit::find_build_spot`, `Group::finish_insert`).
/// item 3376: 1613 -> 1619 on the merged tree, the nations seat's eighteenth shape batch, six of six reached
/// (`Unit::think_civilian_transport`, `Group::action_garrison`, `Unit::do_cast`, `Object::take_damage`,
/// `Unit::do_group_move`, `Roads::add_roads`); item 3381: 1619 -> 1624, the second sweep's fifth batch, five of six
/// (`Unit::do_non_flat_gather`, `Group::action_trade`, `Group::distribute_attack`, `Group::action_attack_ground`,
/// `PathFinder::astar_path`; `Unit::find_build_spot` already answered by 3372); item 3379: 1624 -> 1630, the nations
/// seat's nineteenth batch, six of six (`Group::update_positions`, `Group::redo_patrol_order`, `Group::action_halt`,
/// `Group::action_form`, `Object::add_to_world`, `Objects::find_builds`).
/// **The commander re-pins this constant in the booking commit** (Loop 2367,
/// the thirty-fourth pass): it is the two sweep lanes' shared pin, seven of
/// thirteen merges backed out on it, and the lane gate forgives its red
/// (`release_gate.py`'s `COMMANDERS_LINES`); a lane reports the count
/// `tools/census.py --layers` measures on its tree.
pub const SIMULATION_BACKED: usize = 1630;
/// Of those, the **port-backed**: a document's coverage section diffs the
/// function, or `crates/sim` spells its own `Class::method` outside
/// `sweep/` — a consumer this crate runs. The rest are **layout-backed**: a
/// sweep test asserts the raw word and nothing here reads it (Loop 3093,
/// 1725: Ramon's answer at the fortieth steer; the forty-first pass
/// measured 293 of 1459 on this tree). `tools/census.py --layers` prints
/// both, the `Census:` line carries both, and the commander re-pins both at
/// booking. Item 3182: 303 -> 305 (both of its bodies have a port consumer); item
/// 3200: 305 -> 311 (all six); item 3219: 311 -> 317 (all six); item 3234: 317 -> 323 (all six); item 3229: 323 -> 328 (five of six); item 3238: 328 -> 334 (all six); item 3239: 334 -> 340 (all six); item 3250: 340 -> 347 (all six, and one backed function the crate now spells); item 3259: 347 -> 353 (all six); item 3268: 353 -> 359 (all six); item 3272: 359 -> 365 (all six); item 3261: 365 -> 371 (all six); item 3281: 371 -> 376 (three of six, and two backed functions the crate now spells); item 3289: 376 -> 382 (all six); item 3282: 382 -> 385 (three of six); item 3295: 385 -> 393 (all six, and two backed functions the crate now spells); item 3304: 393 -> 402 (all six, and three backed functions the crate now spells); item 3320: 402 -> 411 (five of six, and four backed functions the crate now spells); item 3330: 411 -> 423 (all six, and six backed functions the crate now spells); item 3336: 423 -> 424 (a backed function the Egyptian word's fixes spell); item 3337: 424 -> 430 (four of six, and two backed functions the crate now spells); item 3339: 430 -> 436 (four of six, and two backed functions the crate now spells); item 3346: 436 -> 443 (all six, and one backed function the crate now spells); item 3348: 443 -> 446 (three of six); item 3358: 446 -> 447 (one of six); item 3356: 447 -> 459 (all six, and six backed functions the crate now spells); item 3362: 459 -> 471 (five of six, and seven backed functions the crate now spells); item 3369: 471 -> 473 (two backed functions the Dutch word's fixes spell); item 3364: 473 -> 486 (all six, and seven backed functions the crate now spells); item 3372: 486 -> 492 (all six); item 3376: 492 -> 501 (all six, and three backed functions the crate now spells); item 3381: 501 -> 507 (five of six, and one backed function the crate now spells); item 3379: 507 -> 516 (all six, and three backed functions the crate now spells).
pub const SIMULATION_PORT_BACKED: usize = 516;
/// Simulation-layer functions: those whose PDB line record starts in a
/// `game\` file the layer rule files under simulation.
pub const SIMULATION_FUNCTIONS: usize = 3611;

#[cfg(test)]
mod tests {
    use super::*;

    /// The number the census measures from `json`'s key `"<key>": N`.
    fn field(json: &str, key: &str) -> Option<usize> {
        let at = json.find(&format!("\"{key}\":"))? + key.len() + 3;
        json[at..]
            .trim_start()
            .split(|c: char| !c.is_ascii_digit())
            .next()?
            .parse()
            .ok()
    }

    /// **The simulation layer's backed share is the pinned one**, measured
    /// by `tools/census.py --layers --json` on this tree. Made to fail first
    /// with the pin moved by one.
    #[test]
    fn the_census_s_simulation_share_is_pinned() {
        let home = std::env::var("HOME").unwrap_or_default();
        let index = format!("{home}/ghidra-projects/decomp/INDEX.tsv");
        if !std::path::Path::new(&index).is_file() {
            eprintln!("skipping: no {index} (the Ghidra export is not on this machine)");
            return;
        }
        let Some(install) = crate::testenv::install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL) for sbl/rise.pdb");
            return;
        };
        let pdb = format!("{install}/sbl/rise.pdb");
        if !std::path::Path::new(&pdb).is_file() {
            eprintln!("skipping: no {pdb}");
            return;
        }
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        let out = std::process::Command::new("python3")
            .arg(format!("{root}/tools/census.py"))
            .args(["--layers", "--json", "--index", &index, "--pdb", &pdb])
            .output()
            .expect("python3 runs");
        let text = String::from_utf8_lossy(&out.stdout);
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            if err.contains("no llvm-pdbutil") {
                eprintln!("skipping: {err}");
                return;
            }
            panic!("census.py --layers failed: {err}");
        }
        let measured = (
            field(&text, "simulation_backed").expect("simulation_backed"),
            field(&text, "simulation_port_backed").expect("simulation_port_backed"),
            field(&text, "simulation_functions").expect("simulation_functions"),
        );
        assert_eq!(
            measured,
            (
                SIMULATION_BACKED,
                SIMULATION_PORT_BACKED,
                SIMULATION_FUNCTIONS
            ),
            "the census moved: re-pin `census::SIMULATION_BACKED`, \
             `SIMULATION_PORT_BACKED` and `SIMULATION_FUNCTIONS` to (backed, port, \
             functions) as measured, and the handoff's `Census:` line with them"
        );
    }

    #[test]
    fn a_measured_field_is_read_whole() {
        let json = r#"{"layers": {}, "simulation_backed": 45, "simulation_port_backed": 9, "simulation_functions": 3611}"#;
        assert_eq!(field(json, "simulation_backed"), Some(45));
        assert_eq!(field(json, "simulation_port_backed"), Some(9));
        assert_eq!(field(json, "simulation_functions"), Some(3611));
        assert_eq!(field(json, "simulation"), None);
    }
}
