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

mod air_order_get_air_order;
mod airattackgroundorder_get_air_order;
mod airpatrolorder_get_air_patrol_order;
mod animal_update_los;
mod attack_ground_order_get_type;
mod attack_order_get_type;
mod attacktoorder_get_attack_to_order;
mod build_order_get_type;
mod build_queue_get_size;
mod build_type_corner_tile;
mod build_type_data_is_city;
mod build_type_data_is_dock;
mod build_type_data_is_fort;
mod build_type_data_is_mine;
mod build_type_data_is_tower;
mod build_type_data_num_preq;
mod build_type_get_good;
mod city_get_empty_trade_routes;
mod city_get_trade_routes;
mod cosx;
mod farms_grow;
mod farms_snip;
mod find_angle;
mod flanking;
mod follow_order_get_type;
mod game_is_raining;
mod garrisonorder_get_garrison_order;
mod gather_order_get_type;
mod gather_point_is_inside;
mod get_type_count2;
mod get_unique_types;
mod get_unique_types2;
mod good_data_get_gpiece;
mod good_inc_time;
mod good_type_avail_drop;
mod good_type_compute_largest_gather;
mod good_type_data_get_enhancer;
mod graphicwinxml_get_graphicwinxml;
mod group_action_begin;
mod group_data_one_member;
mod group_get_speed;
mod groupmoveorder_is_pathed;
mod is_air;
mod is_move;
mod mining_list_clear;
mod moveorder_is_fleeing;
mod object_assault_benefit;
mod object_data_is_active_wallbuild;
mod object_data_is_map_unit;
mod object_data_support;
mod object_get_captain;
mod object_has_range;
mod object_hits;
mod object_is_in_range;
mod object_is_peasant;
mod object_is_scholar;
mod object_is_worker;
mod object_los;
mod object_max_range;
mod object_min_range;
mod object_recharge;
mod object_support_cost;
mod player_is_over;
mod repair_order_get_type;
mod reversing;
mod storm_on_field_change;
mod sub_object_out_is_on_screen;
mod subobject_is_active;
mod subobject_swap_team;
mod supply_deplete;
mod target_order_get_target_order;
mod tcoord_less;
mod think_order_get_type;
mod type_bonus;
mod type_build;
mod type_data_is;
mod type_final;
mod type_gaia;
mod type_good;
mod type_gov;
mod type_gov_hero;
mod type_item;
mod type_specialty;
mod type_spell;
mod type_tech;
mod type_unit;
mod type_wonder;
mod unit_can_transport;
mod unit_control_cost;
mod unit_data_hits;
mod unit_data_support_cost;
mod unit_get_speed;
mod unit_has_repeat_air;
mod unit_is_blown;
mod unit_is_captain;
mod unit_is_entrenched;
mod unit_is_gov_hero;
mod unit_is_hero;
mod unit_is_merchant;
mod unit_is_on_map;
mod unit_is_plane;
mod unit_is_special;
mod unit_is_supply;
mod unit_mana_left;
mod unit_min_range;
mod unit_supplies_left;
mod unit_type_blocks_while_dead;
mod unit_type_data_get_circle_radius;
mod unit_type_data_get_relative_value;
mod unit_type_data_has_stance_type;
mod unit_type_get_stance_type;
mod unit_type_is_caravan;
mod unit_type_is_siege;
mod unit_type_is_spellcaster;
mod unit_type_is_tank;
mod unit_type_needs_packing;
mod unit_type_num_preq;
mod unit_uber_size;
mod wall_construct_time;
mod wall_data_can_make;
mod wall_data_hits;
mod wall_data_is_neutralized;
mod wall_data_is_sea_trade;
mod wall_data_is_trade;
mod wall_data_los;
mod wall_data_support_cost;
mod wall_has_repeat_air;
mod wall_is_active;
mod wall_is_started;
mod wall_is_under_attack;
mod wall_out_get_gpiece;
mod wcoord_to_tcoord;
mod world_data_get_whose;
mod world_data_is_edge;
mod world_get_land;
mod world_get_region;
mod world_get_region2;
mod world_get_seen;
mod world_get_tregion;
mod world_get_was_seen;
mod world_get_who;
mod world_get_who2;
mod world_has_blocked_neighbors;
mod world_is_blocked_at;
mod world_is_built_at;
mod world_is_cliff_at;
mod world_is_coast;
mod world_is_dead_build;
mod world_is_forest;
mod world_is_gathered_from;
mod world_is_impassable;
mod world_is_mountain_at;
mod world_is_mountains;
mod world_is_ocean;
mod world_is_oil_at;
mod world_is_river;
mod world_is_rocks;
mod world_is_tocean;
mod world_is_tree_at;
mod world_space_at_corner;

/// The rows `tools/emu/<script>` prints for the install's executable, or
/// `None` (said on stderr) where the install or `uv` is absent.
pub(crate) fn emu_rows(script: &str) -> Option<String> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let Some(install) = crate::testenv::install_root() else {
        eprintln!("skipping: set RON_INSTALL (the sweep runs the install's executable)");
        return None;
    };
    let out = std::process::Command::new("uv")
        .args(["run", &format!("{root}/tools/emu/{script}")])
        .arg(format!("{install}/riseofnations.exe"))
        .output();
    match out {
        Ok(o) if o.status.success() => Some(String::from_utf8(o.stdout).expect("utf-8 rows")),
        Ok(o) => panic!("{script} failed: {}", String::from_utf8_lossy(&o.stderr)),
        Err(e) => {
            eprintln!("skipping: uv not runnable ({e})");
            None
        }
    }
}

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

/// One row, `<args…> -> <result>`, split into its integers.
pub(crate) fn row(line: &str) -> (Vec<i64>, i64) {
    let (lhs, rhs) = line
        .split_once(" -> ")
        .expect("a row is `<args…> -> <result>`");
    let args = lhs
        .split_whitespace()
        .map(|s| s.parse().expect("an integer argument"))
        .collect();
    (args, rhs.trim().parse().expect("an integer result"))
}

#[test]
fn a_row_is_its_arguments_and_its_result() {
    assert_eq!(row("3 -4 0 -> -7"), (vec![3, -4, 0], -7));
}
mod air_attack_ground_order_get_air_attack_ground_order;
mod ammo_process;
mod angle_diff;
mod border_spline_on_field_change;
mod build_data_is_infinite;
mod build_data_max_gatherers;
mod build_data_most_shots;
mod build_data_num_gather;
mod build_queue_data_get_job_counter;
mod build_queue_data_get_queue;
mod build_queue_set_job_counter;
mod build_type_data_area;
mod build_type_data_gathers;
mod build_type_data_is_civilian;
mod build_type_data_is_embassy;
mod build_type_data_is_flat;
mod build_type_data_is_gather_type;
mod build_type_data_max_flat_gatherers;
mod build_type_data_max_knowledge_gatherers;
mod build_type_walk_rules_data;
mod caravan_link_walk_data;
mod caster_walk_data;
mod circle_render_on_field_change;
mod city_data_pop_growth;
mod cliff_average_neighbor_norms;
mod cliff_render;
mod cliff_walk_data;
mod farms_data_get_farm_type;
mod farms_data_get_wheat_status;
mod follow_order_get_follow_order;
mod form_order_get_form_order;
mod fort_walk_data;
mod game_access_const_get_scary_camera;
mod game_access_const_get_scary_cliffs;
mod game_access_const_get_scary_console;
mod game_access_const_get_scary_console_leader;
mod game_access_const_get_scary_console_who;
mod game_access_const_get_scary_doobers;
mod game_access_const_get_scary_fog;
mod game_access_const_get_scary_graphic_pieces;
mod game_access_const_get_scary_iface_current;
mod game_access_const_get_scary_iface_main;
mod game_access_const_get_scary_message_win;
mod game_access_const_get_scary_my_player;
mod game_access_const_get_scary_options;
mod game_access_const_get_scary_scenario_editor;
mod game_access_const_get_scary_scene;
mod game_access_const_get_scary_select_group;
mod game_access_const_get_scary_terrain;
mod game_is_deathmatch;
mod game_is_infinite;
mod game_is_never_war;
mod game_is_no_rush;
mod game_is_nomad;
mod game_is_recording;
mod game_is_sandbox;
mod game_is_scenario;
mod game_is_solo;
mod game_is_team_game;
mod game_is_tournament;
mod game_is_turbo_resources;
mod gather_point_walk_data;
mod get_type_count;
mod good_type_data_avail_ctw;
mod good_type_walk_rules_data;
mod good_walk_data;
mod group_attack_to_order_get_group_attack_to_order;
mod group_attack_to_order_walk_data;
mod group_data_get_unit;
mod group_leader_report_speed;
mod group_report_speed;
mod groupmoveorder_is_fleeing;
mod guard_order_get_guard_order;
mod guy_data_walk_data;
mod guy_set_bank;
mod guy_set_elev_angle;
mod guy_set_pitch;
mod herd_walk_data;
mod hero_walk_data;
mod item_data_get_gpiece;
mod item_out_draw_overlays;
mod item_type_data_snap_center;
mod item_walk_data;
mod map_fairness_point_lowers_highest;
mod mountains_out_mnt_locs;
mod mountains_out_mnt_ranges;
mod mountains_out_mnt_types;
mod mountains_tileset_texture_load;
mod move_order_is_pathed;
mod nuke_out_update;
mod object_data_is_active_build;
mod object_data_is_dock;
mod object_data_is_siege;
mod object_data_is_visible_wonder;
mod object_execute_events;
mod object_type_data_blocked_site;
mod object_type_data_get_stance_type;
mod object_type_data_has_objmask;
mod object_type_data_is_upgrade;
mod object_type_data_isnt;
mod object_type_out_draw;
mod objects_remove_item;
mod oil_well_walk_data;
mod order_list_get_data_type;
mod path_finder_walk_data;
mod patrol_order_get_patrol_order;
mod player_is_active;
mod search_walk_data;
mod setup_init_wild_life;
mod signposts_cur_x;
mod signposts_cur_y;
mod signposts_get_num;
mod special_anim_order_walk_data;
mod special_data_get_radius;
mod special_walk_data;
mod spell_type_cast_jam_radar;
mod spell_type_cast_rally;
mod strafe_order_get_air_order;
mod strafe_order_get_attack_order;
mod supply_walk_data;
mod sync_point_on_player_deleted;
mod target_order_get_target;
mod tcoord_data_walk_data;
mod tech_type_backup;
mod tech_type_restore;
mod terrain_convert_errant_sandy;
mod terrain_map_get_terrain_type;
mod terrain_map_on_left_up;
mod terrain_map_on_right_up;
mod terrain_vis_wcoord_was_cached;
mod time_sync_test_accept_ping;
mod turn_control_is_cannon_time;
mod type_data_avail;
mod type_data_is_age_type;
mod type_data_is_cancel_alliance;
mod type_data_is_declare_war;
mod type_data_is_epoch_type;
mod type_data_is_merchant;
mod type_data_is_pack;
mod type_data_is_peasant;
mod type_data_is_scholar;
mod type_data_is_unpack;
mod type_data_is_worker;
mod type_data_say_name;
mod type_data_type_graft;
mod type_set_graph_name;
mod type_set_help_name;
mod type_walk_rules_data;
mod types_walk_rules_data;
mod unbuilt_city_walk_data;
mod unbuilt_fort_walk_data;
mod unbuilt_wonder_walk_data;
mod unit_data_can_gather;
mod unit_data_is_caravan;
mod unit_data_is_rare_collector;
mod unit_die;
mod unit_do_flee_to;
mod unit_do_spec_unit_anim;
mod unit_kill_current_action;
mod unit_order_is_move_attack;
mod unit_order_walk_data;
mod unit_think_hero;
mod unit_think_supply;
mod unit_type_data_get_underlay_circle_radius;
mod wall_die;
mod wall_out_draw;
mod wcoord_data_walk_data;
mod wcoord_get_tcorner;
mod wcoord_traverse_y;
mod wcoord_wcoord;
mod window_on_app_focus;
mod window_on_focus;
mod wonder_walk_data;
mod world_adjust_fog;
mod world_data_get_goods;
mod world_data_is_valid_coord;
mod world_data_is_valid_int;
mod world_data_is_valid_tcoord;
mod world_data_valid;
mod world_get_wdata;
