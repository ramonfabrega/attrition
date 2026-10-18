//! **The sweep** — the original's functions run under the emulator against
//! this crate's on chosen inputs (item 1575; DECISIONS 63 (iv), the middle
//! track). One module a function: its `#[test]` names the function as
//! `Name@00xxxxxx`, which is what `tools/census.py --layers` counts as
//! backed, and runs `tools/emu/sweep_<module>.py` — a script on
//! `tools/emu/callfn.py`'s machine that prints one row per input,
//! `<args…> -> <result>` — asserting every row against the port. An input
//! where the two part is a finding: the test names it and the item parks
//! it with its frame.
//!
//! Each test skips, and says so, on a machine without the install or `uv`,
//! as `path::tests::the_emulated_original_agrees_on_every_row` does.

mod support;
pub(crate) use support::{emu_rows, row, tail_row};

mod build_type_get_good;
mod city_get_empty_trade_routes;
mod farms_add_animals;
mod object_has_range;
mod object_is_peasant;
mod object_is_scholar;
mod object_is_worker;
mod object_max_range;
mod object_min_range;
mod object_recharge;
mod objects_add_ammo;
mod objects_find_collision;
mod objects_find_free;
mod path_finder_calc_road_cost;
mod spell_type_cast_unpack;
mod unit_can_transport;
mod unit_control_cost;
mod unit_data_invalid_loc;
mod unit_find_new_air_target;
mod unit_find_open_slots;
mod unit_is_captain;
mod unit_is_merchant;
mod unit_is_on_map;
mod unit_is_plane;
mod unit_is_special;
mod unit_kill_garrison_order;
mod unit_mana_left;
mod unit_min_range;
mod unit_uber_size;
mod wall_construct_time;
mod wall_data_covers_tile;
mod wall_has_repeat_air;
mod wall_is_under_attack;
mod wall_start;
mod wall_update_hits;
mod world_space_at_corner;

/// A `Sim` holding one unit of type `ty`, for the predicates that read a
/// unit's type record: `(sim, unit)`.
pub(crate) fn sim_with_unit(ty: crate::UnitType) -> (crate::Sim, usize) {
    use crate::world::Pos;
    let mut s = crate::Sim::new(crate::Tuning::RON, crate::World::new(8, 8), 2);
    let t = s.add_unit_type(ty);
    let mut u = crate::Unit::new(1, 0, Pos::new(0x1000, 0x1000), 1);
    u.ty = Some(t);
    let u = s.add_unit(u);
    (s, u)
}

#[test]
fn a_row_is_its_arguments_and_its_result() {
    assert_eq!(row("3 -4 0 -> -7"), (vec![3, -4, 0], -7));
    assert_eq!(tail_row("1 -> 2 |  | nnn"), (vec![1], 2, vec!["", "nnn"]));
}
mod animal_data_get_speed;
mod animal_process;
mod armies_find_aggressive_army;
mod army_do_marching;
mod army_do_marching_found;
mod army_set_stance;
mod army_use_generals;
mod army_use_scouts;
mod army_use_spies;
mod build_all_gathering;
mod build_check_gatherers;
mod build_data_calc_gather;
mod build_data_count_queue;
mod build_data_num_gatherers;
mod build_find_city;
mod build_type_data_check_land_adjacent;
mod build_type_data_count_water;
mod build_type_data_gathers;
mod build_type_data_get_enhancing_good;
mod build_type_data_get_town;
mod build_type_data_in_enemy_territory;
mod build_type_data_max_gatherers;
mod build_type_data_non_friendly_territory;
mod build_type_data_tile_corner;
mod build_type_data_total_gather_access;
mod caravan_distance_wcoord;
mod caravan_trade_value;
mod city_compute_trade;
mod city_data_can_upgrade;
mod city_data_count_buildings;
mod city_data_enough_kinds;
mod city_data_num_buildings;
mod city_data_num_kinds;
mod city_data_num_wonders;
mod city_fix_world_vals;
mod coll_block_set;
mod game_wonder_winning;
mod group_kill_group_move;
mod groups_push_group;
mod guy_data_get_speed;
mod herd_process;
mod object_adjacent_to;
mod object_data_is_cargo;
mod object_data_num_inside;
mod object_type_data_upgrade_level;
mod object_update_hits;
mod object_valid_target;
mod path_finder_valid_wcoord;
mod region_coast_here;
mod region_go_here;
mod spell_type_cast_march;
mod unit_add_air_attack_ground_order;
mod unit_add_air_patrol_order;
mod unit_add_move_facing_order;
mod unit_add_strafe_order;
mod unit_add_think_order;
mod unit_air_turn_speed;
mod unit_data_curr_uber_size;
mod unit_data_decoy_time;
mod unit_data_is_fleeing;
mod unit_data_is_flying_high;
mod unit_data_is_flying_low;
mod unit_data_is_gathering_at;
mod unit_data_is_idle;
mod unit_do_attack_to_pause;
mod unit_do_think_order;
mod unit_get_goody_box;
mod unit_kill_group_move;
mod unit_repath;
mod unit_set_anim;
mod unit_set_attack;
mod unit_set_attacking;
mod unit_set_group;
mod unit_set_in_danger;
mod unit_type_data_can_attack_ground;
mod unit_update_action;
mod wall_data_in_unfriendly_territory;
mod wall_data_tile_corner;
mod world_data_check_building_wcoord;
mod world_set_bad_path;
mod world_set_road_at;
mod world_set_seen;
