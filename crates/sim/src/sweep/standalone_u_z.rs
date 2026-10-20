//! Independent sweep partition; test bodies keep their original paths.

#[path = "support.rs"]
mod support;
#[allow(unused_imports)]
use support::{emu_rows, row, tail_row};

// Registered standalone sweeps.
#[path = "do_cast_emu.rs"]
mod do_cast_emu;
#[path = "unbuilt_cities_add_unbuilt_city.rs"]
mod unbuilt_cities_add_unbuilt_city;
#[path = "unbuilt_cities_remove_unbuilt_city.rs"]
mod unbuilt_cities_remove_unbuilt_city;
#[path = "unbuilt_city_get_x.rs"]
mod unbuilt_city_get_x;
#[path = "unbuilt_city_get_y.rs"]
mod unbuilt_city_get_y;
#[path = "unbuilt_city_is_started.rs"]
mod unbuilt_city_is_started;
#[path = "unbuilt_city_walk_data.rs"]
mod unbuilt_city_walk_data;
#[path = "unbuilt_fort_get_x.rs"]
mod unbuilt_fort_get_x;
#[path = "unbuilt_fort_get_y.rs"]
mod unbuilt_fort_get_y;
#[path = "unbuilt_fort_is_started.rs"]
mod unbuilt_fort_is_started;
#[path = "unbuilt_fort_walk_data.rs"]
mod unbuilt_fort_walk_data;
#[path = "unbuilt_forts_add_unbuilt_fort.rs"]
mod unbuilt_forts_add_unbuilt_fort;
#[path = "unbuilt_forts_remove_unbuilt_fort.rs"]
mod unbuilt_forts_remove_unbuilt_fort;
#[path = "unbuilt_wonder_is_started.rs"]
mod unbuilt_wonder_is_started;
#[path = "unbuilt_wonder_walk_data.rs"]
mod unbuilt_wonder_walk_data;
#[path = "unbuilt_wonders_add_unbuilt_wonder.rs"]
mod unbuilt_wonders_add_unbuilt_wonder;
#[path = "unbuilt_wonders_remove_unbuilt_wonder.rs"]
mod unbuilt_wonders_remove_unbuilt_wonder;
#[path = "unit_add_attack_ground_order.rs"]
mod unit_add_attack_ground_order;
#[path = "unit_add_await_board_order.rs"]
mod unit_add_await_board_order;
#[path = "unit_add_board_order.rs"]
mod unit_add_board_order;
#[path = "unit_add_build_order.rs"]
mod unit_add_build_order;
#[path = "unit_add_cast_order.rs"]
mod unit_add_cast_order;
#[path = "unit_add_garrison_order.rs"]
mod unit_add_garrison_order;
#[path = "unit_add_gather_order.rs"]
mod unit_add_gather_order;
#[path = "unit_add_patrol_order.rs"]
mod unit_add_patrol_order;
#[path = "unit_add_repair_order.rs"]
mod unit_add_repair_order;
#[path = "unit_add_spec_anim_order.rs"]
mod unit_add_spec_anim_order;
#[path = "unit_add_trade_order.rs"]
mod unit_add_trade_order;
#[path = "unit_air_alt_speed.rs"]
mod unit_air_alt_speed;
#[path = "unit_bank_aircraft.rs"]
mod unit_bank_aircraft;
#[path = "unit_build_done.rs"]
mod unit_build_done;
#[path = "unit_cavarch_fight.rs"]
mod unit_cavarch_fight;
#[path = "unit_cavarch_fight_written.rs"]
mod unit_cavarch_fight_written;
#[path = "unit_change_target.rs"]
mod unit_change_target;
#[path = "unit_check_idle.rs"]
mod unit_check_idle;
#[path = "unit_check_target_path.rs"]
mod unit_check_target_path;
#[path = "unit_come_out.rs"]
mod unit_come_out;
#[path = "unit_come_out_edx.rs"]
mod unit_come_out_edx;
#[path = "unit_data_action_type.rs"]
mod unit_data_action_type;
#[path = "unit_data_activity_type.rs"]
mod unit_data_activity_type;
#[path = "unit_data_can_ever_transport.rs"]
mod unit_data_can_ever_transport;
#[path = "unit_data_can_garrison.rs"]
mod unit_data_can_garrison;
#[path = "unit_data_can_gather.rs"]
mod unit_data_can_gather;
#[path = "unit_data_find_def_pos.rs"]
mod unit_data_find_def_pos;
#[path = "unit_data_get_attrition.rs"]
mod unit_data_get_attrition;
#[path = "unit_data_get_captain.rs"]
mod unit_data_get_captain;
#[path = "unit_data_get_capture_value.rs"]
mod unit_data_get_capture_value;
#[path = "unit_data_get_caster_stance.rs"]
mod unit_data_get_caster_stance;
#[path = "unit_data_get_combat_stance.rs"]
mod unit_data_get_combat_stance;
#[path = "unit_data_get_gpiece_2.rs"]
mod unit_data_get_gpiece_2;
#[path = "unit_data_get_gpiece_vt.rs"]
mod unit_data_get_gpiece_vt;
#[path = "unit_data_get_order.rs"]
mod unit_data_get_order;
#[path = "unit_data_get_packer_stance.rs"]
mod unit_data_get_packer_stance;
#[path = "unit_data_get_stance_type.rs"]
mod unit_data_get_stance_type;
#[path = "unit_data_get_worker_stance.rs"]
mod unit_data_get_worker_stance;
#[path = "unit_data_good_merchant_spot.rs"]
mod unit_data_good_merchant_spot;
#[path = "unit_data_has_damage.rs"]
mod unit_data_has_damage;
#[path = "unit_data_has_stance_type.rs"]
mod unit_data_has_stance_type;
#[path = "unit_data_hits.rs"]
mod unit_data_hits;
#[path = "unit_data_home_base.rs"]
mod unit_data_home_base;
#[path = "unit_data_in_supply.rs"]
mod unit_data_in_supply;
#[path = "unit_data_is_attacking.rs"]
mod unit_data_is_attacking;
#[path = "unit_data_is_busy.rs"]
mod unit_data_is_busy;
#[path = "unit_data_is_caravan.rs"]
mod unit_data_is_caravan;
#[path = "unit_data_is_casting.rs"]
mod unit_data_is_casting;
#[path = "unit_data_is_cloaked.rs"]
mod unit_data_is_cloaked;
#[path = "unit_data_is_corner.rs"]
mod unit_data_is_corner;
#[path = "unit_data_is_detected.rs"]
mod unit_data_is_detected;
#[path = "unit_data_is_entering_or_exiting.rs"]
mod unit_data_is_entering_or_exiting;
#[path = "unit_data_is_gathering.rs"]
mod unit_data_is_gathering;
#[path = "unit_data_is_here.rs"]
mod unit_data_is_here;
#[path = "unit_data_is_modern_infantry.rs"]
mod unit_data_is_modern_infantry;
#[path = "unit_data_is_moving.rs"]
mod unit_data_is_moving;
#[path = "unit_data_is_packing.rs"]
mod unit_data_is_packing;
#[path = "unit_data_is_packing_or_unpacking.rs"]
mod unit_data_is_packing_or_unpacking;
#[path = "unit_data_is_rare_collector.rs"]
mod unit_data_is_rare_collector;
#[path = "unit_data_is_seen.rs"]
mod unit_data_is_seen;
#[path = "unit_data_is_unpacking.rs"]
mod unit_data_is_unpacking;
#[path = "unit_data_los.rs"]
mod unit_data_los;
#[path = "unit_data_needs_transport.rs"]
mod unit_data_needs_transport;
#[path = "unit_data_next_action_type.rs"]
mod unit_data_next_action_type;
#[path = "unit_data_order_type.rs"]
mod unit_data_order_type;
#[path = "unit_data_recharge.rs"]
mod unit_data_recharge;
#[path = "unit_data_support_cost.rs"]
mod unit_data_support_cost;
#[path = "unit_data_transport_type.rs"]
mod unit_data_transport_type;
#[path = "unit_data_turn_speed.rs"]
mod unit_data_turn_speed;
#[path = "unit_data_will_be_corner.rs"]
mod unit_data_will_be_corner;
#[path = "unit_detect_boat_collision.rs"]
mod unit_detect_boat_collision;
#[path = "unit_die.rs"]
mod unit_die;
#[path = "unit_do_air_attack_ground.rs"]
mod unit_do_air_attack_ground;
#[path = "unit_do_air_physics.rs"]
mod unit_do_air_physics;
#[path = "unit_do_attack.rs"]
mod unit_do_attack;
#[path = "unit_do_attack_ground.rs"]
mod unit_do_attack_ground;
#[path = "unit_do_attack_neg.rs"]
mod unit_do_attack_neg;
#[path = "unit_do_board.rs"]
mod unit_do_board;
#[path = "unit_do_build.rs"]
mod unit_do_build;
#[path = "unit_do_cast_anim.rs"]
mod unit_do_cast_anim;
#[path = "unit_do_cast_head.rs"]
mod unit_do_cast_head;
#[path = "unit_do_cast_tail.rs"]
mod unit_do_cast_tail;
#[path = "unit_do_explore_to.rs"]
mod unit_do_explore_to;
#[path = "unit_do_flee_to.rs"]
mod unit_do_flee_to;
#[path = "unit_do_form_change.rs"]
mod unit_do_form_change;
#[path = "unit_do_gather.rs"]
mod unit_do_gather;
#[path = "unit_do_gather_005ef2a0.rs"]
mod unit_do_gather_005ef2a0;
#[path = "unit_do_group_attack_to.rs"]
mod unit_do_group_attack_to;
#[path = "unit_do_group_move_leader.rs"]
mod unit_do_group_move_leader;
#[path = "unit_do_idle.rs"]
mod unit_do_idle;
#[path = "unit_do_patrol.rs"]
mod unit_do_patrol;
#[path = "unit_do_repair.rs"]
mod unit_do_repair;
#[path = "unit_do_spec_unit_anim.rs"]
mod unit_do_spec_unit_anim;
#[path = "unit_end_trade_route.rs"]
mod unit_end_trade_route;
#[path = "unit_execute_events.rs"]
mod unit_execute_events;
#[path = "unit_find_garrison_build.rs"]
mod unit_find_garrison_build;
#[path = "unit_find_gather_spot.rs"]
mod unit_find_gather_spot;
#[path = "unit_find_goody_box.rs"]
mod unit_find_goody_box;
#[path = "unit_find_melee_pos.rs"]
mod unit_find_melee_pos;
#[path = "unit_find_nearby_spot.rs"]
mod unit_find_nearby_spot;
#[path = "unit_find_new_bomber_target.rs"]
mod unit_find_new_bomber_target;
#[path = "unit_find_repair_spot.rs"]
mod unit_find_repair_spot;
#[path = "unit_get_speed.rs"]
mod unit_get_speed;
#[path = "unit_go_around_building.rs"]
mod unit_go_around_building;
#[path = "unit_go_inside.rs"]
mod unit_go_inside;
#[path = "unit_go_to_city.rs"]
mod unit_go_to_city;
#[path = "unit_has_repeat_air.rs"]
mod unit_has_repeat_air;
#[path = "unit_inc_time.rs"]
mod unit_inc_time;
#[path = "unit_is_blown.rs"]
mod unit_is_blown;
#[path = "unit_is_entrenched.rs"]
mod unit_is_entrenched;
#[path = "unit_is_gov_hero.rs"]
mod unit_is_gov_hero;
#[path = "unit_is_hero.rs"]
mod unit_is_hero;
#[path = "unit_is_supply.rs"]
mod unit_is_supply;
#[path = "unit_kill_current_action.rs"]
mod unit_kill_current_action;
#[path = "unit_kill_current_path.rs"]
mod unit_kill_current_path;
#[path = "unit_kill_group_order.rs"]
mod unit_kill_group_order;
#[path = "unit_modify_group_order.rs"]
mod unit_modify_group_order;
#[path = "unit_must_walk.rs"]
mod unit_must_walk;
#[path = "unit_on_duty.rs"]
mod unit_on_duty;
#[path = "unit_order_is_move_attack.rs"]
mod unit_order_is_move_attack;
#[path = "unit_order_walk_data.rs"]
mod unit_order_walk_data;
#[path = "unit_out_issue_come_out.rs"]
mod unit_out_issue_come_out;
#[path = "unit_out_issue_unqueue.rs"]
mod unit_out_issue_unqueue;
#[path = "unit_plunder.rs"]
mod unit_plunder;
#[path = "unit_replace_form_id.rs"]
mod unit_replace_form_id;
#[path = "unit_resolve_block.rs"]
mod unit_resolve_block;
#[path = "unit_same_damage.rs"]
mod unit_same_damage;
#[path = "unit_set_ai_control.rs"]
mod unit_set_ai_control;
#[path = "unit_set_angle.rs"]
mod unit_set_angle;
#[path = "unit_set_mask.rs"]
mod unit_set_mask;
#[path = "unit_set_new_location.rs"]
mod unit_set_new_location;
#[path = "unit_set_rally.rs"]
mod unit_set_rally;
#[path = "unit_set_type.rs"]
mod unit_set_type;
#[path = "unit_supplies_left.rs"]
mod unit_supplies_left;
#[path = "unit_target_opportunity.rs"]
mod unit_target_opportunity;
#[path = "unit_think.rs"]
mod unit_think;
#[path = "unit_think_caravan.rs"]
mod unit_think_caravan;
#[path = "unit_think_carry.rs"]
mod unit_think_carry;
#[path = "unit_think_carry_civilian.rs"]
mod unit_think_carry_civilian;
#[path = "unit_think_hero.rs"]
mod unit_think_hero;
#[path = "unit_think_peasant.rs"]
mod unit_think_peasant;
#[path = "unit_think_spellcaster.rs"]
mod unit_think_spellcaster;
#[path = "unit_think_supply.rs"]
mod unit_think_supply;
#[path = "unit_type_backup.rs"]
mod unit_type_backup;
#[path = "unit_type_blocks_while_dead.rs"]
mod unit_type_blocks_while_dead;
#[path = "unit_type_data_can_entrench.rs"]
mod unit_type_data_can_entrench;
#[path = "unit_type_data_can_garrison.rs"]
mod unit_type_data_can_garrison;
#[path = "unit_type_data_get_circle_radius.rs"]
mod unit_type_data_get_circle_radius;
#[path = "unit_type_data_get_kill_value.rs"]
mod unit_type_data_get_kill_value;
#[path = "unit_type_data_get_military_level.rs"]
mod unit_type_data_get_military_level;
#[path = "unit_type_data_get_relative_value.rs"]
mod unit_type_data_get_relative_value;
#[path = "unit_type_data_get_underlay_circle_radius.rs"]
mod unit_type_data_get_underlay_circle_radius;
#[path = "unit_type_data_has_stance_type.rs"]
mod unit_type_data_has_stance_type;
#[path = "unit_type_data_is_missile.rs"]
mod unit_type_data_is_missile;
#[path = "unit_type_data_research_time.rs"]
mod unit_type_data_research_time;
#[path = "unit_type_data_type_graft.rs"]
mod unit_type_data_type_graft;
#[path = "unit_type_determine_roles.rs"]
mod unit_type_determine_roles;
#[path = "unit_type_find_nearby_spot.rs"]
mod unit_type_find_nearby_spot;
#[path = "unit_type_gather_radius.rs"]
mod unit_type_gather_radius;
#[path = "unit_type_get_stance_type.rs"]
mod unit_type_get_stance_type;
#[path = "unit_type_is_caravan.rs"]
mod unit_type_is_caravan;
#[path = "unit_type_is_siege.rs"]
mod unit_type_is_siege;
#[path = "unit_type_is_spellcaster.rs"]
mod unit_type_is_spellcaster;
#[path = "unit_type_is_tank.rs"]
mod unit_type_is_tank;
#[path = "unit_type_needs_packing.rs"]
mod unit_type_needs_packing;
#[path = "unit_type_num_preq.rs"]
mod unit_type_num_preq;
#[path = "unit_type_restore.rs"]
mod unit_type_restore;
#[path = "unit_type_walk_rules_data.rs"]
mod unit_type_walk_rules_data;
#[path = "unit_ungroup_move_order.rs"]
mod unit_ungroup_move_order;
#[path = "unit_update_armor.rs"]
mod unit_update_armor;
#[path = "unit_update_ceo_position.rs"]
mod unit_update_ceo_position;
#[path = "unit_update_gpiece.rs"]
mod unit_update_gpiece;
#[path = "unit_update_local_seen.rs"]
mod unit_update_local_seen;
#[path = "unit_update_order.rs"]
mod unit_update_order;
#[path = "unit_update_patrol_order.rs"]
mod unit_update_patrol_order;
#[path = "unit_update_z.rs"]
mod unit_update_z;
#[path = "unit_work.rs"]
mod unit_work;
#[path = "unit_work_tail.rs"]
mod unit_work_tail;
#[path = "units_make_valid.rs"]
mod units_make_valid;
#[path = "valid_filter_case0.rs"]
mod valid_filter_case0;
#[path = "valid_filter_case1.rs"]
mod valid_filter_case1;
#[path = "valid_filter_case10.rs"]
mod valid_filter_case10;
#[path = "valid_filter_case11.rs"]
mod valid_filter_case11;
#[path = "valid_filter_case12.rs"]
mod valid_filter_case12;
#[path = "valid_filter_case13.rs"]
mod valid_filter_case13;
#[path = "valid_filter_case14.rs"]
mod valid_filter_case14;
#[path = "valid_filter_case15.rs"]
mod valid_filter_case15;
#[path = "valid_filter_case16.rs"]
mod valid_filter_case16;
#[path = "valid_filter_case17.rs"]
mod valid_filter_case17;
#[path = "valid_filter_case18.rs"]
mod valid_filter_case18;
#[path = "valid_filter_case19.rs"]
mod valid_filter_case19;
#[path = "valid_filter_case2.rs"]
mod valid_filter_case2;
#[path = "valid_filter_case20.rs"]
mod valid_filter_case20;
#[path = "valid_filter_case21.rs"]
mod valid_filter_case21;
#[path = "valid_filter_case22.rs"]
mod valid_filter_case22;
#[path = "valid_filter_case23.rs"]
mod valid_filter_case23;
#[path = "valid_filter_case24.rs"]
mod valid_filter_case24;
#[path = "valid_filter_case25.rs"]
mod valid_filter_case25;
#[path = "valid_filter_case26.rs"]
mod valid_filter_case26;
#[path = "valid_filter_case3.rs"]
mod valid_filter_case3;
#[path = "valid_filter_case4.rs"]
mod valid_filter_case4;
#[path = "valid_filter_case5.rs"]
mod valid_filter_case5;
#[path = "valid_filter_case6.rs"]
mod valid_filter_case6;
#[path = "valid_filter_case7.rs"]
mod valid_filter_case7;
#[path = "valid_filter_case8.rs"]
mod valid_filter_case8;
#[path = "valid_filter_case9.rs"]
mod valid_filter_case9;
#[path = "valid_target_const_air.rs"]
mod valid_target_const_air;
#[path = "valid_target_const_neg_i.rs"]
mod valid_target_const_neg_i;
#[path = "valid_target_const_p_high.rs"]
mod valid_target_const_p_high;
#[path = "valid_target_const_p_low.rs"]
mod valid_target_const_p_low;
#[path = "vector_dist_coords.rs"]
mod vector_dist_coords;
#[path = "vector_dist_ints.rs"]
mod vector_dist_ints;
#[path = "wall_check_ever_seen.rs"]
mod wall_check_ever_seen;
#[path = "wall_data_armor.rs"]
mod wall_data_armor;
#[path = "wall_data_can_capture.rs"]
mod wall_data_can_capture;
#[path = "wall_data_can_make.rs"]
mod wall_data_can_make;
#[path = "wall_data_get_garrison_limit.rs"]
mod wall_data_get_garrison_limit;
#[path = "wall_data_hits.rs"]
mod wall_data_hits;
#[path = "wall_data_is_defensive.rs"]
mod wall_data_is_defensive;
#[path = "wall_data_is_neutralized.rs"]
mod wall_data_is_neutralized;
#[path = "wall_data_is_now_seen.rs"]
mod wall_data_is_now_seen;
#[path = "wall_data_is_sea_trade.rs"]
mod wall_data_is_sea_trade;
#[path = "wall_data_is_trade.rs"]
mod wall_data_is_trade;
#[path = "wall_data_los.rs"]
mod wall_data_los;
#[path = "wall_data_support_cost.rs"]
mod wall_data_support_cost;
#[path = "wall_data_valid_buildmask.rs"]
mod wall_data_valid_buildmask;
#[path = "wall_data_walk_data_vt.rs"]
mod wall_data_walk_data_vt;
#[path = "wall_die.rs"]
mod wall_die;
#[path = "wall_is_active.rs"]
mod wall_is_active;
#[path = "wall_is_started.rs"]
mod wall_is_started;
#[path = "wall_kill_competing_buildings.rs"]
mod wall_kill_competing_buildings;
#[path = "wall_mask_city.rs"]
mod wall_mask_city;
#[path = "wall_mask_me.rs"]
mod wall_mask_me;
#[path = "wall_out_draw.rs"]
mod wall_out_draw;
#[path = "wall_out_get_gpiece.rs"]
mod wall_out_get_gpiece;
#[path = "wall_out_make_options.rs"]
mod wall_out_make_options;
#[path = "wall_process.rs"]
mod wall_process;
#[path = "wall_set_type.rs"]
mod wall_set_type;
#[path = "wall_set_type_dispatch.rs"]
mod wall_set_type_dispatch;
#[path = "wall_swap_team.rs"]
mod wall_swap_team;
#[path = "wall_update_local_seen.rs"]
mod wall_update_local_seen;
#[path = "wcoord_data_walk_data.rs"]
mod wcoord_data_walk_data;
#[path = "wcoord_get_tcorner.rs"]
mod wcoord_get_tcorner;
#[path = "wcoord_to_tcoord.rs"]
mod wcoord_to_tcoord;
#[path = "wcoord_traverse_x.rs"]
mod wcoord_traverse_x;
#[path = "wcoord_traverse_y.rs"]
mod wcoord_traverse_y;
#[path = "wcoord_wcoord.rs"]
mod wcoord_wcoord;
#[path = "window_on_app_focus.rs"]
mod window_on_app_focus;
#[path = "window_on_focus.rs"]
mod window_on_focus;
#[path = "wonder_data_get_value.rs"]
mod wonder_data_get_value;
#[path = "wonder_get_timer.rs"]
mod wonder_get_timer;
#[path = "wonder_walk_data.rs"]
mod wonder_walk_data;
#[path = "world_adjust_fog.rs"]
mod world_adjust_fog;
#[path = "world_compute_all_territory.rs"]
mod world_compute_all_territory;
#[path = "world_compute_all_vals.rs"]
mod world_compute_all_vals;
#[path = "world_data_buildings_allowed.rs"]
mod world_data_buildings_allowed;
#[path = "world_data_get_bottom.rs"]
mod world_data_get_bottom;
#[path = "world_data_get_coll_block.rs"]
mod world_data_get_coll_block;
#[path = "world_data_get_down.rs"]
mod world_data_get_down;
#[path = "world_data_get_goods.rs"]
mod world_data_get_goods;
#[path = "world_data_get_land.rs"]
mod world_data_get_land;
#[path = "world_data_get_land_wcoord.rs"]
mod world_data_get_land_wcoord;
#[path = "world_data_get_whose.rs"]
mod world_data_get_whose;
#[path = "world_data_has_gather_access.rs"]
mod world_data_has_gather_access;
#[path = "world_data_has_mountain_tcoords.rs"]
mod world_data_has_mountain_tcoords;
#[path = "world_data_is_building_wcoord.rs"]
mod world_data_is_building_wcoord;
#[path = "world_data_is_city_at.rs"]
mod world_data_is_city_at;
#[path = "world_data_is_detected.rs"]
mod world_data_is_detected;
#[path = "world_data_is_detected_by_enemy.rs"]
mod world_data_is_detected_by_enemy;
#[path = "world_data_is_edge.rs"]
mod world_data_is_edge;
#[path = "world_data_is_enemy_territory.rs"]
mod world_data_is_enemy_territory;
#[path = "world_data_is_flat.rs"]
mod world_data_is_flat;
#[path = "world_data_is_pass_land.rs"]
mod world_data_is_pass_land;
#[path = "world_data_is_passable.rs"]
mod world_data_is_passable;
#[path = "world_data_is_seen.rs"]
mod world_data_is_seen;
#[path = "world_data_is_tocean_slow.rs"]
mod world_data_is_tocean_slow;
#[path = "world_data_is_valid_coord.rs"]
mod world_data_is_valid_coord;
#[path = "world_data_is_valid_int.rs"]
mod world_data_is_valid_int;
#[path = "world_data_is_valid_tcoord.rs"]
mod world_data_is_valid_tcoord;
#[path = "world_data_num_waterhalf.rs"]
mod world_data_num_waterhalf;
#[path = "world_data_restrict_coord.rs"]
mod world_data_restrict_coord;
#[path = "world_data_restrict_wcoord.rs"]
mod world_data_restrict_wcoord;
#[path = "world_data_start_city_rad_wcoord.rs"]
mod world_data_start_city_rad_wcoord;
#[path = "world_data_start_city_wcoord.rs"]
mod world_data_start_city_wcoord;
#[path = "world_data_valid.rs"]
mod world_data_valid;
#[path = "world_data_was_really_seen.rs"]
mod world_data_was_really_seen;
#[path = "world_data_was_seen.rs"]
mod world_data_was_seen;
#[path = "world_data_was_seen_wcoord.rs"]
mod world_data_was_seen_wcoord;
#[path = "world_get_land.rs"]
mod world_get_land;
#[path = "world_get_region.rs"]
mod world_get_region;
#[path = "world_get_region2.rs"]
mod world_get_region2;
#[path = "world_get_seen.rs"]
mod world_get_seen;
#[path = "world_get_tregion.rs"]
mod world_get_tregion;
#[path = "world_get_was_seen.rs"]
mod world_get_was_seen;
#[path = "world_get_wdata.rs"]
mod world_get_wdata;
#[path = "world_get_who.rs"]
mod world_get_who;
#[path = "world_get_who2.rs"]
mod world_get_who2;
#[path = "world_has_blocked_neighbors.rs"]
mod world_has_blocked_neighbors;
#[path = "world_is_blocked_at.rs"]
mod world_is_blocked_at;
#[path = "world_is_built_at.rs"]
mod world_is_built_at;
#[path = "world_is_cliff_at.rs"]
mod world_is_cliff_at;
#[path = "world_is_coast.rs"]
mod world_is_coast;
#[path = "world_is_dead_build.rs"]
mod world_is_dead_build;
#[path = "world_is_edge_of_region.rs"]
mod world_is_edge_of_region;
#[path = "world_is_forest.rs"]
mod world_is_forest;
#[path = "world_is_gathered_from.rs"]
mod world_is_gathered_from;
#[path = "world_is_impassable.rs"]
mod world_is_impassable;
#[path = "world_is_mountain_at.rs"]
mod world_is_mountain_at;
#[path = "world_is_mountains.rs"]
mod world_is_mountains;
#[path = "world_is_ocean.rs"]
mod world_is_ocean;
#[path = "world_is_oil_at.rs"]
mod world_is_oil_at;
#[path = "world_is_river.rs"]
mod world_is_river;
#[path = "world_is_rocks.rs"]
mod world_is_rocks;
#[path = "world_is_tocean.rs"]
mod world_is_tocean;
#[path = "world_is_tree_at.rs"]
mod world_is_tree_at;
#[path = "world_map_fog_add_update.rs"]
mod world_map_fog_add_update;
#[path = "world_map_fog_fill_colors.rs"]
mod world_map_fog_fill_colors;
#[path = "world_map_fog_get_fog_lookup.rs"]
mod world_map_fog_get_fog_lookup;
#[path = "world_map_fog_set_fog_lookup.rs"]
mod world_map_fog_set_fog_lookup;
#[path = "world_map_on_right_down.rs"]
mod world_map_on_right_down;
#[path = "world_map_reinit_all.rs"]
mod world_map_reinit_all;
#[path = "world_new_coll_block.rs"]
mod world_new_coll_block;
#[path = "world_reveal_fog.rs"]
mod world_reveal_fog;
#[path = "world_scale_by_area.rs"]
mod world_scale_by_area;
#[path = "world_scale_by_axis.rs"]
mod world_scale_by_axis;
#[path = "world_set_behind.rs"]
mod world_set_behind;
#[path = "world_set_blocked_at.rs"]
mod world_set_blocked_at;
#[path = "world_set_building_at.rs"]
mod world_set_building_at;
#[path = "world_set_city_at.rs"]
mod world_set_city_at;
#[path = "world_set_cliff_at.rs"]
mod world_set_cliff_at;
#[path = "world_set_coastal.rs"]
mod world_set_coastal;
#[path = "world_set_down.rs"]
mod world_set_down;
#[path = "world_set_edge_flags.rs"]
mod world_set_edge_flags;
#[path = "world_set_gather_edge.rs"]
mod world_set_gather_edge;
#[path = "world_set_gathered_at.rs"]
mod world_set_gathered_at;
#[path = "world_set_land.rs"]
mod world_set_land;
#[path = "world_set_mountain_at.rs"]
mod world_set_mountain_at;
#[path = "world_set_oil_at.rs"]
mod world_set_oil_at;
#[path = "world_set_orig_coast.rs"]
mod world_set_orig_coast;
#[path = "world_set_resource_at.rs"]
mod world_set_resource_at;
#[path = "world_set_river_at.rs"]
mod world_set_river_at;
#[path = "world_set_seen2.rs"]
mod world_set_seen2;
#[path = "world_set_started2_at.rs"]
mod world_set_started2_at;
#[path = "world_set_started_at.rs"]
mod world_set_started_at;
#[path = "world_set_tocean.rs"]
mod world_set_tocean;
#[path = "world_set_tree_at.rs"]
mod world_set_tree_at;
#[path = "world_set_was_seen.rs"]
mod world_set_was_seen;
#[path = "world_set_waterhalf.rs"]
mod world_set_waterhalf;
#[path = "world_to_screen_a.rs"]
mod world_to_screen_a;
#[path = "world_to_screen_b.rs"]
mod world_to_screen_b;
