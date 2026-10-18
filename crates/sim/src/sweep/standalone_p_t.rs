//! Independent sweep partition; test bodies keep their original paths.

#[path = "support.rs"]
mod support;
#[allow(unused_imports)]
use support::{emu_rows, row, tail_row};

// Registered standalone sweeps.
#[path = "path_finder_add_to_openlist.rs"]
mod path_finder_add_to_openlist;
#[path = "path_finder_building_danger.rs"]
mod path_finder_building_danger;
#[path = "path_finder_calc_river_cost.rs"]
mod path_finder_calc_river_cost;
#[path = "path_finder_data_find_node_closed.rs"]
mod path_finder_data_find_node_closed;
#[path = "path_finder_data_find_node_open.rs"]
mod path_finder_data_find_node_open;
#[path = "path_finder_empty_lists.rs"]
mod path_finder_empty_lists;
#[path = "path_finder_find_road_restore.rs"]
mod path_finder_find_road_restore;
#[path = "path_finder_find_tpath.rs"]
mod path_finder_find_tpath;
#[path = "path_finder_find_upath.rs"]
mod path_finder_find_upath;
#[path = "path_finder_find_upath_restore.rs"]
mod path_finder_find_upath_restore;
#[path = "path_finder_find_wpath.rs"]
mod path_finder_find_wpath;
#[path = "path_finder_find_wpath_army.rs"]
mod path_finder_find_wpath_army;
#[path = "path_finder_kill_lists.rs"]
mod path_finder_kill_lists;
#[path = "path_finder_kill_tree_bt.rs"]
mod path_finder_kill_tree_bt;
#[path = "path_finder_kill_tree_te.rs"]
mod path_finder_kill_tree_te;
#[path = "path_finder_out_dbg_draw_failures.rs"]
mod path_finder_out_dbg_draw_failures;
#[path = "path_finder_out_get_name.rs"]
mod path_finder_out_get_name;
#[path = "path_finder_valid_tcoord.rs"]
mod path_finder_valid_tcoord;
#[path = "path_finder_valid_ucoord.rs"]
mod path_finder_valid_ucoord;
#[path = "path_finder_walk_data.rs"]
mod path_finder_walk_data;
#[path = "patrol_order_get_patrol_order.rs"]
mod patrol_order_get_patrol_order;
#[path = "patrol_order_walk_data.rs"]
mod patrol_order_walk_data;
#[path = "player_end_game.rs"]
mod player_end_game;
#[path = "player_is_active.rs"]
mod player_is_active;
#[path = "player_is_active_connection.rs"]
mod player_is_active_connection;
#[path = "player_is_connection.rs"]
mod player_is_connection;
#[path = "player_is_over.rs"]
mod player_is_over;
#[path = "player_lose.rs"]
mod player_lose;
#[path = "player_walk_data.rs"]
mod player_walk_data;
#[path = "player_win.rs"]
mod player_win;
#[path = "project_c.rs"]
mod project_c;
#[path = "project_d.rs"]
mod project_d;
#[path = "region_fix_borders.rs"]
mod region_fix_borders;
#[path = "region_is_coast.rs"]
mod region_is_coast;
#[path = "region_num_coasts.rs"]
mod region_num_coasts;
#[path = "region_release.rs"]
mod region_release;
#[path = "region_walk_data.rs"]
mod region_walk_data;
#[path = "regions_concat.rs"]
mod regions_concat;
#[path = "regions_fix_all_borders.rs"]
mod regions_fix_all_borders;
#[path = "regions_get_num_sea.rs"]
mod regions_get_num_sea;
#[path = "regions_walk_data.rs"]
mod regions_walk_data;
#[path = "repair_order_get_type.rs"]
mod repair_order_get_type;
#[path = "repair_order_get_type_vtordisp.rs"]
mod repair_order_get_type_vtordisp;
#[path = "resource_divvy_pool_add_early.rs"]
mod resource_divvy_pool_add_early;
#[path = "resource_divvy_pool_add_good.rs"]
mod resource_divvy_pool_add_good;
#[path = "resource_divvy_pool_add_late.rs"]
mod resource_divvy_pool_add_late;
#[path = "resource_divvy_pool_add_water.rs"]
mod resource_divvy_pool_add_water;
#[path = "resource_divvy_pool_done_adding_goods.rs"]
mod resource_divvy_pool_done_adding_goods;
#[path = "resource_divvy_pool_get_early.rs"]
mod resource_divvy_pool_get_early;
#[path = "resource_divvy_pool_get_late.rs"]
mod resource_divvy_pool_get_late;
#[path = "resource_divvy_pool_get_water.rs"]
mod resource_divvy_pool_get_water;
#[path = "resource_divvy_pool_refresh_early.rs"]
mod resource_divvy_pool_refresh_early;
#[path = "resource_divvy_pool_refresh_late.rs"]
mod resource_divvy_pool_refresh_late;
#[path = "resource_divvy_pool_refresh_water.rs"]
mod resource_divvy_pool_refresh_water;
#[path = "reversing.rs"]
mod reversing;
#[path = "river_doober_add_doob_for_render.rs"]
mod river_doober_add_doob_for_render;
#[path = "river_doober_add_for_render.rs"]
mod river_doober_add_for_render;
#[path = "river_doobers_add_doobers_for_render.rs"]
mod river_doobers_add_doobers_for_render;
#[path = "river_section_out_compute_vert_height.rs"]
mod river_section_out_compute_vert_height;
#[path = "rivers_out_get_name.rs"]
mod rivers_out_get_name;
#[path = "rivers_walk_data.rs"]
mod rivers_walk_data;
#[path = "roads_generate.rs"]
mod roads_generate;
#[path = "roads_out_fill_cache.rs"]
mod roads_out_fill_cache;
#[path = "roads_out_get_name.rs"]
mod roads_out_get_name;
#[path = "roads_out_set_neighbor.rs"]
mod roads_out_set_neighbor;
#[path = "roads_pieces_rot_byte.rs"]
mod roads_pieces_rot_byte;
#[path = "roads_road_added.rs"]
mod roads_road_added;
#[path = "roads_road_cleared.rs"]
mod roads_road_cleared;
#[path = "roads_scan_and_kill_straggled_tcoord.rs"]
mod roads_scan_and_kill_straggled_tcoord;
#[path = "search_valid_search.rs"]
mod search_valid_search;
#[path = "search_walk_data.rs"]
mod search_walk_data;
#[path = "setup_build_leader.rs"]
mod setup_build_leader;
#[path = "setup_get_starting_citizens.rs"]
mod setup_get_starting_citizens;
#[path = "setup_init_wild_life.rs"]
mod setup_init_wild_life;
#[path = "signposts_add.rs"]
mod signposts_add;
#[path = "signposts_cur_name.rs"]
mod signposts_cur_name;
#[path = "signposts_cur_x.rs"]
mod signposts_cur_x;
#[path = "signposts_cur_y.rs"]
mod signposts_cur_y;
#[path = "signposts_get_num.rs"]
mod signposts_get_num;
#[path = "signposts_head.rs"]
mod signposts_head;
#[path = "signposts_next.rs"]
mod signposts_next;
#[path = "sort_b.rs"]
mod sort_b;
#[path = "sort_c.rs"]
mod sort_c;
#[path = "special_anim_order_walk_data.rs"]
mod special_anim_order_walk_data;
#[path = "special_data_get_radius.rs"]
mod special_data_get_radius;
#[path = "special_walk_data.rs"]
mod special_walk_data;
#[path = "specials_data_find_special.rs"]
mod specials_data_find_special;
#[path = "spell_type_cast_blow_tread.rs"]
mod spell_type_cast_blow_tread;
#[path = "spell_type_cast_civilian.rs"]
mod spell_type_cast_civilian;
#[path = "spell_type_cast_depopulate.rs"]
mod spell_type_cast_depopulate;
#[path = "spell_type_cast_jam_radar.rs"]
mod spell_type_cast_jam_radar;
#[path = "spell_type_cast_lock_gate.rs"]
mod spell_type_cast_lock_gate;
#[path = "spell_type_cast_pack.rs"]
mod spell_type_cast_pack;
#[path = "spell_type_cast_rally.rs"]
mod spell_type_cast_rally;
#[path = "spell_type_cast_research_project.rs"]
mod spell_type_cast_research_project;
#[path = "spell_type_cast_supplies.rs"]
mod spell_type_cast_supplies;
#[path = "spell_type_cast_unlock_gate.rs"]
mod spell_type_cast_unlock_gate;
#[path = "spell_type_pay_cast_costs.rs"]
mod spell_type_pay_cast_costs;
#[path = "spell_type_unpay_cast_costs.rs"]
mod spell_type_unpay_cast_costs;
#[path = "spell_type_walk_rules_data.rs"]
mod spell_type_walk_rules_data;
#[path = "storm_get_name.rs"]
mod storm_get_name;
#[path = "storm_on_field_change.rs"]
mod storm_on_field_change;
#[path = "strafe_order_get_air_order.rs"]
mod strafe_order_get_air_order;
#[path = "strafe_order_get_attack_order.rs"]
mod strafe_order_get_attack_order;
#[path = "strafe_order_get_strafe_order_vtordisp.rs"]
mod strafe_order_get_strafe_order_vtordisp;
#[path = "strafe_order_walk_data.rs"]
mod strafe_order_walk_data;
#[path = "strafe_order_walk_data_vtordisp.rs"]
mod strafe_order_walk_data_vtordisp;
#[path = "strip_builder_check_last_connect.rs"]
mod strip_builder_check_last_connect;
#[path = "strip_builder_remove_end.rs"]
mod strip_builder_remove_end;
#[path = "sub_object_data_is_seen.rs"]
mod sub_object_data_is_seen;
#[path = "sub_object_data_is_spellcaster.rs"]
mod sub_object_data_is_spellcaster;
#[path = "sub_object_data_play_sound.rs"]
mod sub_object_data_play_sound;
#[path = "sub_object_is.rs"]
mod sub_object_is;
#[path = "sub_object_must_walk.rs"]
mod sub_object_must_walk;
#[path = "sub_object_out_is_on_screen.rs"]
mod sub_object_out_is_on_screen;
#[path = "sub_object_refresh_z.rs"]
mod sub_object_refresh_z;
#[path = "sub_object_set_new_location.rs"]
mod sub_object_set_new_location;
#[path = "sub_object_set_type.rs"]
mod sub_object_set_type;
#[path = "sub_object_swap_team.rs"]
mod sub_object_swap_team;
#[path = "sub_object_time.rs"]
mod sub_object_time;
#[path = "subobject_is_active.rs"]
mod subobject_is_active;
#[path = "subobject_swap_team.rs"]
mod subobject_swap_team;
#[path = "supplies_find_supply.rs"]
mod supplies_find_supply;
#[path = "supply_data_get_radius.rs"]
mod supply_data_get_radius;
#[path = "supply_deplete.rs"]
mod supply_deplete;
#[path = "supply_walk_data.rs"]
mod supply_walk_data;
#[path = "surf_get_name.rs"]
mod surf_get_name;
#[path = "surf_wave_section_load_old_bh3.rs"]
mod surf_wave_section_load_old_bh3;
#[path = "syllable_parse.rs"]
mod syllable_parse;
#[path = "sync_point_on_player_deleted.rs"]
mod sync_point_on_player_deleted;
#[path = "target_order_get_target.rs"]
mod target_order_get_target;
#[path = "target_order_get_target_order.rs"]
mod target_order_get_target_order;
#[path = "target_order_target_exists.rs"]
mod target_order_target_exists;
#[path = "target_order_walk_data.rs"]
mod target_order_walk_data;
#[path = "tcoord_data_walk_data.rs"]
mod tcoord_data_walk_data;
#[path = "tcoord_less.rs"]
mod tcoord_less;
#[path = "tech_type_add_preq_ai.rs"]
mod tech_type_add_preq_ai;
#[path = "tech_type_backup.rs"]
mod tech_type_backup;
#[path = "tech_type_restore.rs"]
mod tech_type_restore;
#[path = "tech_type_set_disabled.rs"]
mod tech_type_set_disabled;
#[path = "tech_type_walk_rules_data.rs"]
mod tech_type_walk_rules_data;
#[path = "terrain_build_beach_data.rs"]
mod terrain_build_beach_data;
#[path = "terrain_convert_errant_sandy.rs"]
mod terrain_convert_errant_sandy;
#[path = "terrain_has_south_forest.rs"]
mod terrain_has_south_forest;
#[path = "terrain_list_walk_data.rs"]
mod terrain_list_walk_data;
#[path = "terrain_map_cancel_button_enable.rs"]
mod terrain_map_cancel_button_enable;
#[path = "terrain_map_get_terrain_type.rs"]
mod terrain_map_get_terrain_type;
#[path = "terrain_map_map_change_river.rs"]
mod terrain_map_map_change_river;
#[path = "terrain_map_on_left_up.rs"]
mod terrain_map_on_left_up;
#[path = "terrain_map_on_mouse_leave.rs"]
mod terrain_map_on_mouse_leave;
#[path = "terrain_map_on_right_up.rs"]
mod terrain_map_on_right_up;
#[path = "terrain_map_remove_feature.rs"]
mod terrain_map_remove_feature;
#[path = "terrain_map_river_point.rs"]
mod terrain_map_river_point;
#[path = "terrain_map_start_river.rs"]
mod terrain_map_start_river;
#[path = "terrain_object_placed.rs"]
mod terrain_object_placed;
#[path = "terrain_oil_gain_tech.rs"]
mod terrain_oil_gain_tech;
#[path = "terrain_oil_lose_tech.rs"]
mod terrain_oil_lose_tech;
#[path = "terrain_oil_remove_rocky_patch.rs"]
mod terrain_oil_remove_rocky_patch;
#[path = "terrain_out_get_name.rs"]
mod terrain_out_get_name;
#[path = "terrain_refresh_good_z.rs"]
mod terrain_refresh_good_z;
#[path = "terrain_vis_get_verts_per_bucket.rs"]
mod terrain_vis_get_verts_per_bucket;
#[path = "terrain_vis_invalidate_fog_wcoord.rs"]
mod terrain_vis_invalidate_fog_wcoord;
#[path = "terrain_vis_invalidate_wcoord.rs"]
mod terrain_vis_invalidate_wcoord;
#[path = "terrain_vis_invalidate_wcoord_1.rs"]
mod terrain_vis_invalidate_wcoord_1;
#[path = "terrain_vis_set_base_source_pool.rs"]
mod terrain_vis_set_base_source_pool;
#[path = "terrain_vis_set_extra_source_pool.rs"]
mod terrain_vis_set_extra_source_pool;
#[path = "terrain_vis_wcoord_was_cached.rs"]
mod terrain_vis_wcoord_was_cached;
#[path = "terrain_walk_coord_data.rs"]
mod terrain_walk_coord_data;
#[path = "think_order_get_type.rs"]
mod think_order_get_type;
#[path = "time_sync_add_ping.rs"]
mod time_sync_add_ping;
#[path = "time_sync_analyze_response.rs"]
mod time_sync_analyze_response;
#[path = "time_sync_get_best_ping_time.rs"]
mod time_sync_get_best_ping_time;
#[path = "time_sync_on_player_deleted.rs"]
mod time_sync_on_player_deleted;
#[path = "time_sync_test_accept_ping.rs"]
mod time_sync_test_accept_ping;
#[path = "time_sync_update.rs"]
mod time_sync_update;
#[path = "trade_order_walk_data.rs"]
mod trade_order_walk_data;
#[path = "trade_order_walk_data_vtordisp.rs"]
mod trade_order_walk_data_vtordisp;
#[path = "tribe_get_name.rs"]
mod tribe_get_name;
#[path = "tribe_set_pseudonym.rs"]
mod tribe_set_pseudonym;
#[path = "tribe_walk_data.rs"]
mod tribe_walk_data;
#[path = "tribe_walk_rules_data.rs"]
mod tribe_walk_rules_data;
#[path = "tribes_find.rs"]
mod tribes_find;
#[path = "tribes_walk_data.rs"]
mod tribes_walk_data;
#[path = "turn_control_check_new_frame_playback.rs"]
mod turn_control_check_new_frame_playback;
#[path = "turn_control_check_new_frame_solo.rs"]
mod turn_control_check_new_frame_solo;
#[path = "turn_control_compute_next_turn_solo.rs"]
mod turn_control_compute_next_turn_solo;
#[path = "turn_control_end_cannon_time.rs"]
mod turn_control_end_cannon_time;
#[path = "turn_control_get_delta_percent.rs"]
mod turn_control_get_delta_percent;
#[path = "turn_control_get_frame_timing.rs"]
mod turn_control_get_frame_timing;
#[path = "turn_control_get_lag_time.rs"]
mod turn_control_get_lag_time;
#[path = "turn_control_is_cannon_time.rs"]
mod turn_control_is_cannon_time;
#[path = "turn_control_issue_speed_down.rs"]
mod turn_control_issue_speed_down;
#[path = "turn_control_issue_speed_set.rs"]
mod turn_control_issue_speed_set;
#[path = "turn_control_issue_speed_up.rs"]
mod turn_control_issue_speed_up;
#[path = "turn_control_issue_toggle_pause.rs"]
mod turn_control_issue_toggle_pause;
#[path = "turn_control_set_new_target_time.rs"]
mod turn_control_set_new_target_time;
#[path = "turn_control_set_pause.rs"]
mod turn_control_set_pause;
#[path = "turn_control_speed_down.rs"]
mod turn_control_speed_down;
#[path = "turn_control_speed_up.rs"]
mod turn_control_speed_up;
#[path = "turn_control_walk_data.rs"]
mod turn_control_walk_data;
#[path = "type_backup.rs"]
mod type_backup;
#[path = "type_bonus.rs"]
mod type_bonus;
#[path = "type_build.rs"]
mod type_build;
#[path = "type_data_avail.rs"]
mod type_data_avail;
#[path = "type_data_can_pay_cost.rs"]
mod type_data_can_pay_cost;
#[path = "type_data_count_discovered.rs"]
mod type_data_count_discovered;
#[path = "type_data_get_abbrev_parsed.rs"]
mod type_data_get_abbrev_parsed;
#[path = "type_data_get_kill_value.rs"]
mod type_data_get_kill_value;
#[path = "type_data_get_limiting_resource.rs"]
mod type_data_get_limiting_resource;
#[path = "type_data_get_score_value.rs"]
mod type_data_get_score_value;
#[path = "type_data_is.rs"]
mod type_data_is;
#[path = "type_data_is_age_type.rs"]
mod type_data_is_age_type;
#[path = "type_data_is_cancel_alliance.rs"]
mod type_data_is_cancel_alliance;
#[path = "type_data_is_declare_war.rs"]
mod type_data_is_declare_war;
#[path = "type_data_is_epoch_type.rs"]
mod type_data_is_epoch_type;
#[path = "type_data_is_merchant.rs"]
mod type_data_is_merchant;
#[path = "type_data_is_object_type.rs"]
mod type_data_is_object_type;
#[path = "type_data_is_pack.rs"]
mod type_data_is_pack;
#[path = "type_data_is_peasant.rs"]
mod type_data_is_peasant;
#[path = "type_data_is_plain_tech_type.rs"]
mod type_data_is_plain_tech_type;
#[path = "type_data_is_scholar.rs"]
mod type_data_is_scholar;
#[path = "type_data_is_unpack.rs"]
mod type_data_is_unpack;
#[path = "type_data_is_worker.rs"]
mod type_data_is_worker;
#[path = "type_data_research_time.rs"]
mod type_data_research_time;
#[path = "type_data_say_name.rs"]
mod type_data_say_name;
#[path = "type_data_time.rs"]
mod type_data_time;
#[path = "type_data_type_graft.rs"]
mod type_data_type_graft;
#[path = "type_final.rs"]
mod type_final;
#[path = "type_gaia.rs"]
mod type_gaia;
#[path = "type_good.rs"]
mod type_good;
#[path = "type_gov.rs"]
mod type_gov;
#[path = "type_gov_hero.rs"]
mod type_gov_hero;
#[path = "type_item.rs"]
mod type_item;
#[path = "type_pay_cost.rs"]
mod type_pay_cost;
#[path = "type_restore.rs"]
mod type_restore;
#[path = "type_set_graph_name.rs"]
mod type_set_graph_name;
#[path = "type_set_help_name.rs"]
mod type_set_help_name;
#[path = "type_set_name.rs"]
mod type_set_name;
#[path = "type_set_type_name.rs"]
mod type_set_type_name;
#[path = "type_specialty.rs"]
mod type_specialty;
#[path = "type_spell.rs"]
mod type_spell;
#[path = "type_sum_cost.rs"]
mod type_sum_cost;
#[path = "type_sum_rules_cost.rs"]
mod type_sum_rules_cost;
#[path = "type_tech.rs"]
mod type_tech;
#[path = "type_unit.rs"]
mod type_unit;
#[path = "type_unpay_cost.rs"]
mod type_unpay_cost;
#[path = "type_walk_rules_data.rs"]
mod type_walk_rules_data;
#[path = "type_wonder.rs"]
mod type_wonder;
#[path = "types_object_key.rs"]
mod types_object_key;
#[path = "types_walk_data.rs"]
mod types_walk_data;
#[path = "types_walk_rules_data.rs"]
mod types_walk_rules_data;
