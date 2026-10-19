//! Independent sweep partition; test bodies keep their original paths.

#[path = "support.rs"]
mod support;
#[allow(unused_imports)]
use support::{emu_rows, row, tail_row};

// Registered standalone sweeps.
#[path = "game_access_const_find_data_z.rs"]
mod game_access_const_find_data_z;
#[path = "game_access_const_find_tcoord_z.rs"]
mod game_access_const_find_tcoord_z;
#[path = "game_access_const_get_scary_camera.rs"]
mod game_access_const_get_scary_camera;
#[path = "game_access_const_get_scary_cliffs.rs"]
mod game_access_const_get_scary_cliffs;
#[path = "game_access_const_get_scary_console.rs"]
mod game_access_const_get_scary_console;
#[path = "game_access_const_get_scary_console_leader.rs"]
mod game_access_const_get_scary_console_leader;
#[path = "game_access_const_get_scary_console_who.rs"]
mod game_access_const_get_scary_console_who;
#[path = "game_access_const_get_scary_doobers.rs"]
mod game_access_const_get_scary_doobers;
#[path = "game_access_const_get_scary_fog.rs"]
mod game_access_const_get_scary_fog;
#[path = "game_access_const_get_scary_graphic_pieces.rs"]
mod game_access_const_get_scary_graphic_pieces;
#[path = "game_access_const_get_scary_iface_current.rs"]
mod game_access_const_get_scary_iface_current;
#[path = "game_access_const_get_scary_iface_main.rs"]
mod game_access_const_get_scary_iface_main;
#[path = "game_access_const_get_scary_message_win.rs"]
mod game_access_const_get_scary_message_win;
#[path = "game_access_const_get_scary_my_player.rs"]
mod game_access_const_get_scary_my_player;
#[path = "game_access_const_get_scary_options.rs"]
mod game_access_const_get_scary_options;
#[path = "game_access_const_get_scary_scenario_editor.rs"]
mod game_access_const_get_scary_scenario_editor;
#[path = "game_access_const_get_scary_scene.rs"]
mod game_access_const_get_scary_scene;
#[path = "game_access_const_get_scary_select_group.rs"]
mod game_access_const_get_scary_select_group;
#[path = "game_access_const_get_scary_terrain.rs"]
mod game_access_const_get_scary_terrain;
#[path = "game_action_cheat_ai_speed_increase.rs"]
mod game_action_cheat_ai_speed_increase;
#[path = "game_action_cheat_ai_speed_normal.rs"]
mod game_action_cheat_ai_speed_normal;
#[path = "game_action_cheat_ai_toggle.rs"]
mod game_action_cheat_ai_toggle;
#[path = "game_add_game_fps_to_average.rs"]
mod game_add_game_fps_to_average;
#[path = "game_clock_init.rs"]
mod game_clock_init;
#[path = "game_current_age.rs"]
mod game_current_age;
#[path = "game_defeat_all.rs"]
mod game_defeat_all;
#[path = "game_get_armageddon.rs"]
mod game_get_armageddon;
#[path = "game_get_game_fps_average.rs"]
mod game_get_game_fps_average;
#[path = "game_get_patch_version.rs"]
mod game_get_patch_version;
#[path = "game_get_time_limit.rs"]
mod game_get_time_limit;
#[path = "game_info_default_setting.rs"]
mod game_info_default_setting;
#[path = "game_info_get_map_size_name.rs"]
mod game_info_get_map_size_name;
#[path = "game_info_load_from_playback.rs"]
mod game_info_load_from_playback;
#[path = "game_is_cooperative.rs"]
mod game_is_cooperative;
#[path = "game_is_deathmatch.rs"]
mod game_is_deathmatch;
#[path = "game_is_game_ready.rs"]
mod game_is_game_ready;
#[path = "game_is_infinite.rs"]
mod game_is_infinite;
#[path = "game_is_never_war.rs"]
mod game_is_never_war;
#[path = "game_is_no_rush.rs"]
mod game_is_no_rush;
#[path = "game_is_nomad.rs"]
mod game_is_nomad;
#[path = "game_is_observing.rs"]
mod game_is_observing;
#[path = "game_is_raining.rs"]
mod game_is_raining;
#[path = "game_is_recording.rs"]
mod game_is_recording;
#[path = "game_is_sandbox.rs"]
mod game_is_sandbox;
#[path = "game_is_scenario.rs"]
mod game_is_scenario;
#[path = "game_is_solo.rs"]
mod game_is_solo;
#[path = "game_is_team_game.rs"]
mod game_is_team_game;
#[path = "game_is_tournament.rs"]
mod game_is_tournament;
#[path = "game_is_turbo_resources.rs"]
mod game_is_turbo_resources;
#[path = "game_is_victory_timer.rs"]
mod game_is_victory_timer;
#[path = "game_on_find_leaderboard.rs"]
mod game_on_find_leaderboard;
#[path = "game_popwin_timer.rs"]
mod game_popwin_timer;
#[path = "game_run_unit_editor.rs"]
mod game_run_unit_editor;
#[path = "game_teams_locked.rs"]
mod game_teams_locked;
#[path = "game_unload_net_lib.rs"]
mod game_unload_net_lib;
#[path = "game_walk_data.rs"]
mod game_walk_data;
#[path = "game_walk_rules_data.rs"]
mod game_walk_rules_data;
#[path = "game_war_allowed.rs"]
mod game_war_allowed;
#[path = "game_wonder_timer.rs"]
mod game_wonder_timer;
#[path = "garrison_order_walk_data.rs"]
mod garrison_order_walk_data;
#[path = "garrison_order_walk_data_vtordisp.rs"]
mod garrison_order_walk_data_vtordisp;
#[path = "garrisonorder_get_garrison_order.rs"]
mod garrisonorder_get_garrison_order;
#[path = "gather_order_get_type.rs"]
mod gather_order_get_type;
#[path = "gather_order_get_type_vtordisp.rs"]
mod gather_order_get_type_vtordisp;
#[path = "gather_order_walk_data.rs"]
mod gather_order_walk_data;
#[path = "gather_order_walk_data_vtordisp.rs"]
mod gather_order_walk_data_vtordisp;
#[path = "gather_point_is_inside.rs"]
mod gather_point_is_inside;
#[path = "gather_point_list_get_new_data.rs"]
mod gather_point_list_get_new_data;
#[path = "gather_point_walk_data.rs"]
mod gather_point_walk_data;
#[path = "get_float_z.rs"]
mod get_float_z;
#[path = "get_type_count.rs"]
mod get_type_count;
#[path = "get_type_count2.rs"]
mod get_type_count2;
#[path = "get_unique_types.rs"]
mod get_unique_types;
#[path = "get_unique_types2.rs"]
mod get_unique_types2;
#[path = "good_data_get_gpiece.rs"]
mod good_data_get_gpiece;
#[path = "good_data_is_seen.rs"]
mod good_data_is_seen;
#[path = "good_inc_time.rs"]
mod good_inc_time;
#[path = "good_type_avail_drop.rs"]
mod good_type_avail_drop;
#[path = "good_type_compute_largest_gather.rs"]
mod good_type_compute_largest_gather;
#[path = "good_type_data_avail_ctw.rs"]
mod good_type_data_avail_ctw;
#[path = "good_type_data_best_gather_type.rs"]
mod good_type_data_best_gather_type;
#[path = "good_type_data_get_enhancer.rs"]
mod good_type_data_get_enhancer;
#[path = "good_type_data_is_flat.rs"]
mod good_type_data_is_flat;
#[path = "good_type_data_is_herd_type.rs"]
mod good_type_data_is_herd_type;
#[path = "good_type_data_snap_center.rs"]
mod good_type_data_snap_center;
#[path = "good_type_out_say_gather_level.rs"]
mod good_type_out_say_gather_level;
#[path = "good_type_walk_rules_data.rs"]
mod good_type_walk_rules_data;
#[path = "good_walk_data.rs"]
mod good_walk_data;
#[path = "graphicwinxml_get_graphicwinxml.rs"]
mod graphicwinxml_get_graphicwinxml;
#[path = "greatest_common_factor.rs"]
mod greatest_common_factor;
#[path = "group_action_begin.rs"]
mod group_action_begin;
#[path = "group_action_buildmask.rs"]
mod group_action_buildmask;
#[path = "group_action_hotkey.rs"]
mod group_action_hotkey;
#[path = "group_action_queue_up.rs"]
mod group_action_queue_up;
#[path = "group_attack_order_vector_deleting_destructor_vtordisp.rs"]
mod group_attack_order_vector_deleting_destructor_vtordisp;
#[path = "group_attack_order_walk_data.rs"]
mod group_attack_order_walk_data;
#[path = "group_attack_order_walk_data_vtordisp.rs"]
mod group_attack_order_walk_data_vtordisp;
#[path = "group_attack_to_order_get_group_attack_to_order.rs"]
mod group_attack_to_order_get_group_attack_to_order;
#[path = "group_attack_to_order_walk_data.rs"]
mod group_attack_to_order_walk_data;
#[path = "group_attack_to_order_walk_data_vtordisp.rs"]
mod group_attack_to_order_walk_data_vtordisp;
#[path = "group_can_activate.rs"]
mod group_can_activate;
#[path = "group_compute_speed.rs"]
mod group_compute_speed;
#[path = "group_data_can_transport.rs"]
mod group_data_can_transport;
#[path = "group_data_equals_group_const.rs"]
mod group_data_equals_group_const;
#[path = "group_data_find_leader.rs"]
mod group_data_find_leader;
#[path = "group_data_get_angle.rs"]
mod group_data_get_angle;
#[path = "group_data_get_auto_explore.rs"]
mod group_data_get_auto_explore;
#[path = "group_data_get_form.rs"]
mod group_data_get_form;
#[path = "group_data_get_form_mod_option.rs"]
mod group_data_get_form_mod_option;
#[path = "group_data_get_form_option.rs"]
mod group_data_get_form_option;
#[path = "group_data_get_infinite_queue.rs"]
mod group_data_get_infinite_queue;
#[path = "group_data_get_max_range.rs"]
mod group_data_get_max_range;
#[path = "group_data_get_num_cap_const.rs"]
mod group_data_get_num_cap_const;
#[path = "group_data_get_num_const.rs"]
mod group_data_get_num_const;
#[path = "group_data_get_o.rs"]
mod group_data_get_o;
#[path = "group_data_get_repeat_air.rs"]
mod group_data_get_repeat_air;
#[path = "group_data_get_unit.rs"]
mod group_data_get_unit;
#[path = "group_data_group_air.rs"]
mod group_data_group_air;
#[path = "group_data_has_air_units.rs"]
mod group_data_has_air_units;
#[path = "group_data_is_on_map.rs"]
mod group_data_is_on_map;
#[path = "group_data_member.rs"]
mod group_data_member;
#[path = "group_data_num_valid.rs"]
mod group_data_num_valid;
#[path = "group_data_one_member.rs"]
mod group_data_one_member;
#[path = "group_data_play_sound.rs"]
mod group_data_play_sound;
#[path = "group_data_valid_target.rs"]
mod group_data_valid_target;
#[path = "group_dbg_jump_to_action.rs"]
mod group_dbg_jump_to_action;
#[path = "group_dbg_jump_to_waypoint.rs"]
mod group_dbg_jump_to_waypoint;
#[path = "group_equals_group.rs"]
mod group_equals_group;
#[path = "group_find_role.rs"]
mod group_find_role;
#[path = "group_get_num.rs"]
mod group_get_num;
#[path = "group_get_num_cap.rs"]
mod group_get_num_cap;
#[path = "group_get_speed.rs"]
mod group_get_speed;
#[path = "group_is_attacking_near.rs"]
mod group_is_attacking_near;
#[path = "group_kill_group_order.rs"]
mod group_kill_group_order;
#[path = "group_kill_next_order.rs"]
mod group_kill_next_order;
#[path = "group_leader_report_speed.rs"]
mod group_leader_report_speed;
#[path = "group_move_order_get_group_move_order_vtordisp.rs"]
mod group_move_order_get_group_move_order_vtordisp;
#[path = "group_move_order_is_fleeing_vtordisp.rs"]
mod group_move_order_is_fleeing_vtordisp;
#[path = "group_move_order_is_pathed_vtordisp.rs"]
mod group_move_order_is_pathed_vtordisp;
#[path = "group_move_order_walk_data.rs"]
mod group_move_order_walk_data;
#[path = "group_move_order_walk_data_vtordisp.rs"]
mod group_move_order_walk_data_vtordisp;
#[path = "group_order_walk_data.rs"]
mod group_order_walk_data;
#[path = "group_out_issue_alarm.rs"]
mod group_out_issue_alarm;
#[path = "group_out_issue_attack.rs"]
mod group_out_issue_attack;
#[path = "group_out_issue_build.rs"]
mod group_out_issue_build;
#[path = "group_out_issue_buildmask.rs"]
mod group_out_issue_buildmask;
#[path = "group_out_issue_city_gather.rs"]
mod group_out_issue_city_gather;
#[path = "group_out_issue_disband.rs"]
mod group_out_issue_disband;
#[path = "group_out_issue_flight.rs"]
mod group_out_issue_flight;
#[path = "group_out_issue_form.rs"]
mod group_out_issue_form;
#[path = "group_out_issue_garrison.rs"]
mod group_out_issue_garrison;
#[path = "group_out_issue_halt.rs"]
mod group_out_issue_halt;
#[path = "group_out_issue_hotkey.rs"]
mod group_out_issue_hotkey;
#[path = "group_out_issue_launch_patrol.rs"]
mod group_out_issue_launch_patrol;
#[path = "group_out_issue_patrol.rs"]
mod group_out_issue_patrol;
#[path = "group_out_issue_recall.rs"]
mod group_out_issue_recall;
#[path = "group_out_issue_scramble.rs"]
mod group_out_issue_scramble;
#[path = "group_out_issue_set_transport.rs"]
mod group_out_issue_set_transport;
#[path = "group_out_issue_siege_attack.rs"]
mod group_out_issue_siege_attack;
#[path = "group_out_issue_spell.rs"]
mod group_out_issue_spell;
#[path = "group_out_issue_stance.rs"]
mod group_out_issue_stance;
#[path = "group_out_issue_stop_spell.rs"]
mod group_out_issue_stop_spell;
#[path = "group_out_issue_swarm_around.rs"]
mod group_out_issue_swarm_around;
#[path = "group_out_issue_trade.rs"]
mod group_out_issue_trade;
#[path = "group_out_issue_unitmask.rs"]
mod group_out_issue_unitmask;
#[path = "group_patrol_order_vector_deleting_destructor_vtordisp.rs"]
mod group_patrol_order_vector_deleting_destructor_vtordisp;
#[path = "group_patrol_order_walk_data.rs"]
mod group_patrol_order_walk_data;
#[path = "group_patrol_order_walk_data_vtordisp.rs"]
mod group_patrol_order_walk_data_vtordisp;
#[path = "group_report_speed.rs"]
mod group_report_speed;
#[path = "group_set_ordered.rs"]
mod group_set_ordered;
#[path = "group_set_up_insert.rs"]
mod group_set_up_insert;
#[path = "group_toggle.rs"]
mod group_toggle;
#[path = "group_walk_data.rs"]
mod group_walk_data;
#[path = "groupmoveorder_is_fleeing.rs"]
mod groupmoveorder_is_fleeing;
#[path = "groupmoveorder_is_pathed.rs"]
mod groupmoveorder_is_pathed;
#[path = "groups_copy_group.rs"]
mod groups_copy_group;
#[path = "groups_data_find_group_const.rs"]
mod groups_data_find_group_const;
#[path = "groups_walk_data.rs"]
mod groups_walk_data;
#[path = "guard_order_get_guard_order.rs"]
mod guard_order_get_guard_order;
#[path = "guard_order_get_guard_order_vtordisp.rs"]
mod guard_order_get_guard_order_vtordisp;
#[path = "guard_order_walk_data.rs"]
mod guard_order_walk_data;
#[path = "guard_order_walk_data_vtordisp.rs"]
mod guard_order_walk_data_vtordisp;
#[path = "guy_copy_data.rs"]
mod guy_copy_data;
#[path = "guy_data_get_render_angle.rs"]
mod guy_data_get_render_angle;
#[path = "guy_data_get_render_bank.rs"]
mod guy_data_get_render_bank;
#[path = "guy_data_get_render_pitch.rs"]
mod guy_data_get_render_pitch;
#[path = "guy_data_get_render_x.rs"]
mod guy_data_get_render_x;
#[path = "guy_data_get_render_y.rs"]
mod guy_data_get_render_y;
#[path = "guy_data_get_render_z.rs"]
mod guy_data_get_render_z;
#[path = "guy_data_is_corner.rs"]
mod guy_data_is_corner;
#[path = "guy_data_walk_data.rs"]
mod guy_data_walk_data;
#[path = "guy_set_bank.rs"]
mod guy_set_bank;
#[path = "guy_set_elev_angle.rs"]
mod guy_set_elev_angle;
#[path = "guy_set_new_z.rs"]
mod guy_set_new_z;
#[path = "guy_set_pitch.rs"]
mod guy_set_pitch;
#[path = "guy_set_pivot_angle.rs"]
mod guy_set_pivot_angle;
#[path = "guy_turn_angles.rs"]
mod guy_turn_angles;
#[path = "guy_turn_towards.rs"]
mod guy_turn_towards;
#[path = "guy_update_gpiece.rs"]
mod guy_update_gpiece;
#[path = "guy_update_z.rs"]
mod guy_update_z;
#[path = "herd_walk_data.rs"]
mod herd_walk_data;
#[path = "herds_find_herd.rs"]
mod herds_find_herd;
#[path = "herds_process.rs"]
mod herds_process;
#[path = "herds_start_herds.rs"]
mod herds_start_herds;
#[path = "herds_walk_data.rs"]
mod herds_walk_data;
#[path = "hero_walk_data.rs"]
mod hero_walk_data;
#[path = "heroes_data_find_hero.rs"]
mod heroes_data_find_hero;
#[path = "is_air.rs"]
mod is_air;
#[path = "is_move.rs"]
mod is_move;
#[path = "item_data_get_gpiece.rs"]
mod item_data_get_gpiece;
#[path = "item_data_is_seen.rs"]
mod item_data_is_seen;
#[path = "item_out_draw_overlays.rs"]
mod item_out_draw_overlays;
#[path = "item_type_data_snap_center.rs"]
mod item_type_data_snap_center;
#[path = "item_walk_data.rs"]
mod item_walk_data;
#[path = "land_data_get_amount.rs"]
mod land_data_get_amount;
#[path = "land_walk_data.rs"]
mod land_walk_data;
#[path = "lands_find_key.rs"]
mod lands_find_key;
#[path = "lands_walk_data.rs"]
mod lands_walk_data;
#[path = "link_vert_append.rs"]
mod link_vert_append;
#[path = "map_calc_bonus.rs"]
mod map_calc_bonus;
#[path = "map_calc_bonus2.rs"]
mod map_calc_bonus2;
#[path = "map_conquest_check_sea_map.rs"]
mod map_conquest_check_sea_map;
#[path = "map_fairness_apply_distances.rs"]
mod map_fairness_apply_distances;
#[path = "map_fairness_enable_team_mode.rs"]
mod map_fairness_enable_team_mode;
#[path = "map_fairness_point_lowers_highest.rs"]
mod map_fairness_point_lowers_highest;
#[path = "map_fairness_recompute.rs"]
mod map_fairness_recompute;
#[path = "map_fairness_unapply_distances.rs"]
mod map_fairness_unapply_distances;
#[path = "map_get_player_overage.rs"]
mod map_get_player_overage;
#[path = "map_get_player_underage.rs"]
mod map_get_player_underage;
#[path = "map_invert_land.rs"]
mod map_invert_land;
#[path = "map_is_near_ocean.rs"]
mod map_is_near_ocean;
#[path = "map_land_dist.rs"]
mod map_land_dist;
#[path = "map_scale_by_area.rs"]
mod map_scale_by_area;
#[path = "map_scale_by_axis.rs"]
mod map_scale_by_axis;
#[path = "map_script_run_map_script.rs"]
mod map_script_run_map_script;
#[path = "map_set_file.rs"]
mod map_set_file;
#[path = "map_stamp.rs"]
mod map_stamp;
#[path = "mining_list_clear.rs"]
mod mining_list_clear;
#[path = "misc_access_coord_vis_3.rs"]
mod misc_access_coord_vis_3;
#[path = "misc_access_coord_vis_4.rs"]
mod misc_access_coord_vis_4;
#[path = "misc_access_find_global_render_z.rs"]
mod misc_access_find_global_render_z;
#[path = "mountain_range_validate_tcoord.rs"]
mod mountain_range_validate_tcoord;
#[path = "mountains_data_mnt_loc_x.rs"]
mod mountains_data_mnt_loc_x;
#[path = "mountains_data_mnt_loc_y.rs"]
mod mountains_data_mnt_loc_y;
#[path = "mountains_data_total_gather_size.rs"]
mod mountains_data_total_gather_size;
#[path = "mountains_out_mnt_locs.rs"]
mod mountains_out_mnt_locs;
#[path = "mountains_out_mnt_ranges.rs"]
mod mountains_out_mnt_ranges;
#[path = "mountains_out_mnt_types.rs"]
mod mountains_out_mnt_types;
#[path = "mountains_tileset_texture_load.rs"]
mod mountains_tileset_texture_load;
#[path = "mountains_walk_data.rs"]
mod mountains_walk_data;
#[path = "move_order_get_move_order_vtordisp.rs"]
mod move_order_get_move_order_vtordisp;
#[path = "move_order_is_fleeing_vtordisp.rs"]
mod move_order_is_fleeing_vtordisp;
#[path = "move_order_is_pathed.rs"]
mod move_order_is_pathed;
#[path = "move_order_is_pathed_vtordisp.rs"]
mod move_order_is_pathed_vtordisp;
#[path = "move_order_walk_data_vtordisp.rs"]
mod move_order_walk_data_vtordisp;
#[path = "moveorder_is_fleeing.rs"]
mod moveorder_is_fleeing;
#[path = "nuke_been_damaged_before.rs"]
mod nuke_been_damaged_before;
#[path = "nuke_draw_ctw.rs"]
mod nuke_draw_ctw;
#[path = "nuke_get_name.rs"]
mod nuke_get_name;
#[path = "nuke_on_field_change.rs"]
mod nuke_on_field_change;
#[path = "nuke_out_num_current_nukes.rs"]
mod nuke_out_num_current_nukes;
#[path = "nuke_out_setup_terrain_terraforming.rs"]
mod nuke_out_setup_terrain_terraforming;
#[path = "nuke_out_update.rs"]
mod nuke_out_update;
#[path = "object_assault_benefit.rs"]
mod object_assault_benefit;
#[path = "object_data_armor.rs"]
mod object_data_armor;
#[path = "object_data_attack_dist.rs"]
mod object_data_attack_dist;
#[path = "object_data_attack_dist_at.rs"]
mod object_data_attack_dist_at;
#[path = "object_data_can_carry.rs"]
mod object_data_can_carry;
#[path = "object_data_can_carry_domain.rs"]
mod object_data_can_carry_domain;
#[path = "object_data_can_carry_pair.rs"]
mod object_data_can_carry_pair;
#[path = "object_data_can_carry_type.rs"]
mod object_data_can_carry_type;
#[path = "object_data_cargo_inside.rs"]
mod object_data_cargo_inside;
#[path = "object_data_get_inside.rs"]
mod object_data_get_inside;
#[path = "object_data_get_wonder_value.rs"]
mod object_data_get_wonder_value;
#[path = "object_data_has_general.rs"]
mod object_data_has_general;
#[path = "object_data_has_nuke.rs"]
mod object_data_has_nuke;
#[path = "object_data_has_objmask.rs"]
mod object_data_has_objmask;
#[path = "object_data_health_level.rs"]
mod object_data_health_level;
#[path = "object_data_hits_left.rs"]
mod object_data_hits_left;
#[path = "object_data_in_a_building.rs"]
mod object_data_in_a_building;
#[path = "object_data_in_a_ship.rs"]
mod object_data_in_a_ship;
#[path = "object_data_inside_bottom.rs"]
mod object_data_inside_bottom;
#[path = "object_data_is_active_build.rs"]
mod object_data_is_active_build;
#[path = "object_data_is_active_wallbuild.rs"]
mod object_data_is_active_wallbuild;
#[path = "object_data_is_dock.rs"]
mod object_data_is_dock;
#[path = "object_data_is_in_range.rs"]
mod object_data_is_in_range;
#[path = "object_data_is_in_range_at.rs"]
mod object_data_is_in_range_at;
#[path = "object_data_is_in_range_margin.rs"]
mod object_data_is_in_range_margin;
#[path = "object_data_is_in_range_this.rs"]
mod object_data_is_in_range_this;
#[path = "object_data_is_infiltrated.rs"]
mod object_data_is_infiltrated;
#[path = "object_data_is_jammed.rs"]
mod object_data_is_jammed;
#[path = "object_data_is_map_unit.rs"]
mod object_data_is_map_unit;
#[path = "object_data_is_siege.rs"]
mod object_data_is_siege;
#[path = "object_data_is_visible_wonder.rs"]
mod object_data_is_visible_wonder;
#[path = "object_data_isnt.rs"]
mod object_data_isnt;
#[path = "object_data_num_aircraft_here.rs"]
mod object_data_num_aircraft_here;
#[path = "object_data_num_aircraft_limit.rs"]
mod object_data_num_aircraft_limit;
#[path = "object_data_say_name.rs"]
mod object_data_say_name;
#[path = "object_data_support.rs"]
mod object_data_support;
#[path = "object_data_valid_target_const.rs"]
mod object_data_valid_target_const;
#[path = "object_execute_events.rs"]
mod object_execute_events;
#[path = "object_fire_ammo.rs"]
mod object_fire_ammo;
#[path = "object_get_army.rs"]
mod object_get_army;
#[path = "object_get_captain.rs"]
mod object_get_captain;
#[path = "object_hits.rs"]
mod object_hits;
#[path = "object_is_in_range.rs"]
mod object_is_in_range;
#[path = "object_kill_contents.rs"]
mod object_kill_contents;
#[path = "object_los.rs"]
mod object_los;
#[path = "object_must_walk.rs"]
mod object_must_walk;
#[path = "object_poor_target.rs"]
mod object_poor_target;
#[path = "object_repair_damage.rs"]
mod object_repair_damage;
#[path = "object_support_cost.rs"]
mod object_support_cost;
#[path = "object_type_backup.rs"]
mod object_type_backup;
#[path = "object_type_data_basic_type.rs"]
mod object_type_data_basic_type;
#[path = "object_type_data_blocked_site.rs"]
mod object_type_data_blocked_site;
#[path = "object_type_data_get_age.rs"]
mod object_type_data_get_age;
#[path = "object_type_data_get_age_slow.rs"]
mod object_type_data_get_age_slow;
#[path = "object_type_data_get_stance_type.rs"]
mod object_type_data_get_stance_type;
#[path = "object_type_data_get_wonder_value.rs"]
mod object_type_data_get_wonder_value;
#[path = "object_type_data_has_objmask.rs"]
mod object_type_data_has_objmask;
#[path = "object_type_data_is.rs"]
mod object_type_data_is;
#[path = "object_type_data_is_cargo.rs"]
mod object_type_data_is_cargo;
#[path = "object_type_data_is_defense.rs"]
mod object_type_data_is_defense;
#[path = "object_type_data_is_slow.rs"]
mod object_type_data_is_slow;
#[path = "object_type_data_is_upgrade.rs"]
mod object_type_data_is_upgrade;
#[path = "object_type_data_isnt.rs"]
mod object_type_data_isnt;
#[path = "object_type_out_draw.rs"]
mod object_type_out_draw;
#[path = "object_type_out_draw_template.rs"]
mod object_type_out_draw_template;
#[path = "object_type_restore.rs"]
mod object_type_restore;
#[path = "object_type_walk_rules_data.rs"]
mod object_type_walk_rules_data;
#[path = "object_update_los.rs"]
mod object_update_los;
#[path = "object_update_seen.rs"]
mod object_update_seen;
#[path = "object_update_seen_build.rs"]
mod object_update_seen_build;
#[path = "object_update_seen_small.rs"]
mod object_update_seen_small;
#[path = "object_update_seen_wide.rs"]
mod object_update_seen_wide;
#[path = "objects_array_get_data_type.rs"]
mod objects_array_get_data_type;
#[path = "objects_data_count_nukes_in_flight.rs"]
mod objects_data_count_nukes_in_flight;
#[path = "objects_data_find_any_building_at.rs"]
mod objects_data_find_any_building_at;
#[path = "objects_data_find_building_placed_at.rs"]
mod objects_data_find_building_placed_at;
#[path = "objects_data_find_city.rs"]
mod objects_data_find_city;
#[path = "objects_data_find_city_at.rs"]
mod objects_data_find_city_at;
#[path = "objects_data_find_dock.rs"]
mod objects_data_find_dock;
#[path = "objects_data_find_good_at.rs"]
mod objects_data_find_good_at;
#[path = "objects_data_find_good_at_tcoord.rs"]
mod objects_data_find_good_at_tcoord;
#[path = "objects_data_find_good_at_wcoord.rs"]
mod objects_data_find_good_at_wcoord;
#[path = "objects_data_find_goody_at.rs"]
mod objects_data_find_goody_at;
#[path = "objects_data_find_goody_at_tile.rs"]
mod objects_data_find_goody_at_tile;
#[path = "objects_data_find_town.rs"]
mod objects_data_find_town;
#[path = "objects_data_find_town_at.rs"]
mod objects_data_find_town_at;
#[path = "objects_data_find_unit_ordered_with_radius.rs"]
mod objects_data_find_unit_ordered_with_radius;
#[path = "objects_data_find_unit_with_radius.rs"]
mod objects_data_find_unit_with_radius;
#[path = "objects_data_find_village.rs"]
mod objects_data_find_village;
#[path = "objects_data_make_tree_mask.rs"]
mod objects_data_make_tree_mask;
#[path = "objects_out_sort_objects.rs"]
mod objects_out_sort_objects;
#[path = "objects_remove_item.rs"]
mod objects_remove_item;
#[path = "ocean_vars_get_name.rs"]
mod ocean_vars_get_name;
#[path = "oil_well_walk_data.rs"]
mod oil_well_walk_data;
#[path = "order_list_get_data_type.rs"]
mod order_list_get_data_type;
#[path = "order_list_get_new_data.rs"]
mod order_list_get_new_data;
