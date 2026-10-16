//! Independent sweep partition; test bodies keep their original paths.

#[path = "support.rs"]
mod support;
#[allow(unused_imports)]
use support::{emu_rows, row, tail_row};

// Registered standalone sweeps.
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
#[path = "unit_air_alt_speed.rs"]
mod unit_air_alt_speed;
#[path = "unit_data_action_type.rs"]
mod unit_data_action_type;
#[path = "unit_data_activity_type.rs"]
mod unit_data_activity_type;
#[path = "unit_data_can_garrison.rs"]
mod unit_data_can_garrison;
#[path = "unit_data_can_gather.rs"]
mod unit_data_can_gather;
#[path = "unit_data_get_captain.rs"]
mod unit_data_get_captain;
#[path = "unit_data_get_order.rs"]
mod unit_data_get_order;
#[path = "unit_data_get_stance_type.rs"]
mod unit_data_get_stance_type;
#[path = "unit_data_has_damage.rs"]
mod unit_data_has_damage;
#[path = "unit_data_has_stance_type.rs"]
mod unit_data_has_stance_type;
#[path = "unit_data_hits.rs"]
mod unit_data_hits;
#[path = "unit_data_is_attacking.rs"]
mod unit_data_is_attacking;
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
#[path = "unit_data_is_unpacking.rs"]
mod unit_data_is_unpacking;
#[path = "unit_data_los.rs"]
mod unit_data_los;
#[path = "unit_data_next_action_type.rs"]
mod unit_data_next_action_type;
#[path = "unit_data_support_cost.rs"]
mod unit_data_support_cost;
#[path = "unit_data_transport_type.rs"]
mod unit_data_transport_type;
#[path = "unit_data_turn_speed.rs"]
mod unit_data_turn_speed;
#[path = "unit_data_will_be_corner.rs"]
mod unit_data_will_be_corner;
#[path = "unit_die.rs"]
mod unit_die;
#[path = "unit_do_board.rs"]
mod unit_do_board;
#[path = "unit_do_flee_to.rs"]
mod unit_do_flee_to;
#[path = "unit_do_form_change.rs"]
mod unit_do_form_change;
#[path = "unit_do_gather.rs"]
mod unit_do_gather;
#[path = "unit_do_spec_unit_anim.rs"]
mod unit_do_spec_unit_anim;
#[path = "unit_go_to_city.rs"]
mod unit_go_to_city;
#[path = "unit_inc_time.rs"]
mod unit_inc_time;
#[path = "unit_kill_current_action.rs"]
mod unit_kill_current_action;
#[path = "unit_must_walk.rs"]
mod unit_must_walk;
#[path = "unit_order_is_move_attack.rs"]
mod unit_order_is_move_attack;
#[path = "unit_order_walk_data.rs"]
mod unit_order_walk_data;
#[path = "unit_out_issue_come_out.rs"]
mod unit_out_issue_come_out;
#[path = "unit_out_issue_unqueue.rs"]
mod unit_out_issue_unqueue;
#[path = "unit_set_ai_control.rs"]
mod unit_set_ai_control;
#[path = "unit_set_mask.rs"]
mod unit_set_mask;
#[path = "unit_set_rally.rs"]
mod unit_set_rally;
#[path = "unit_set_type.rs"]
mod unit_set_type;
#[path = "unit_think_hero.rs"]
mod unit_think_hero;
#[path = "unit_think_supply.rs"]
mod unit_think_supply;
#[path = "unit_type_backup.rs"]
mod unit_type_backup;
#[path = "unit_type_data_can_entrench.rs"]
mod unit_type_data_can_entrench;
#[path = "unit_type_data_get_circle_radius.rs"]
mod unit_type_data_get_circle_radius;
#[path = "unit_type_data_get_kill_value.rs"]
mod unit_type_data_get_kill_value;
#[path = "unit_type_data_get_underlay_circle_radius.rs"]
mod unit_type_data_get_underlay_circle_radius;
#[path = "unit_type_data_has_stance_type.rs"]
mod unit_type_data_has_stance_type;
#[path = "unit_type_data_type_graft.rs"]
mod unit_type_data_type_graft;
#[path = "unit_type_restore.rs"]
mod unit_type_restore;
#[path = "unit_type_walk_rules_data.rs"]
mod unit_type_walk_rules_data;
#[path = "unit_update_order.rs"]
mod unit_update_order;
#[path = "units_make_valid.rs"]
mod units_make_valid;
#[path = "wall_data_can_capture.rs"]
mod wall_data_can_capture;
#[path = "wall_data_can_make.rs"]
mod wall_data_can_make;
#[path = "wall_data_get_garrison_limit.rs"]
mod wall_data_get_garrison_limit;
#[path = "wall_data_is_neutralized.rs"]
mod wall_data_is_neutralized;
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
#[path = "wall_die.rs"]
mod wall_die;
#[path = "wall_out_draw.rs"]
mod wall_out_draw;
#[path = "wall_out_get_gpiece.rs"]
mod wall_out_get_gpiece;
#[path = "wall_set_type.rs"]
mod wall_set_type;
#[path = "wall_set_type_dispatch.rs"]
mod wall_set_type_dispatch;
#[path = "wcoord_data_walk_data.rs"]
mod wcoord_data_walk_data;
#[path = "wcoord_get_tcorner.rs"]
mod wcoord_get_tcorner;
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
#[path = "world_data_is_pass_land.rs"]
mod world_data_is_pass_land;
#[path = "world_data_is_passable.rs"]
mod world_data_is_passable;
#[path = "world_data_is_tocean_slow.rs"]
mod world_data_is_tocean_slow;
#[path = "world_data_is_valid_coord.rs"]
mod world_data_is_valid_coord;
#[path = "world_data_is_valid_int.rs"]
mod world_data_is_valid_int;
#[path = "world_data_is_valid_tcoord.rs"]
mod world_data_is_valid_tcoord;
#[path = "world_data_restrict_coord.rs"]
mod world_data_restrict_coord;
#[path = "world_data_restrict_wcoord.rs"]
mod world_data_restrict_wcoord;
#[path = "world_data_start_city_wcoord.rs"]
mod world_data_start_city_wcoord;
#[path = "world_data_valid.rs"]
mod world_data_valid;
#[path = "world_data_was_seen.rs"]
mod world_data_was_seen;
#[path = "world_get_seen.rs"]
mod world_get_seen;
#[path = "world_get_was_seen.rs"]
mod world_get_was_seen;
#[path = "world_get_wdata.rs"]
mod world_get_wdata;
#[path = "world_is_edge_of_region.rs"]
mod world_is_edge_of_region;
#[path = "world_map_fog_get_fog_lookup.rs"]
mod world_map_fog_get_fog_lookup;
#[path = "world_scale_by_area.rs"]
mod world_scale_by_area;
#[path = "world_scale_by_axis.rs"]
mod world_scale_by_axis;
#[path = "world_set_behind.rs"]
mod world_set_behind;
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
#[path = "world_set_gather_edge.rs"]
mod world_set_gather_edge;
#[path = "world_set_orig_coast.rs"]
mod world_set_orig_coast;
#[path = "world_set_tocean.rs"]
mod world_set_tocean;
#[path = "world_set_was_seen.rs"]
mod world_set_was_seen;
#[path = "world_set_waterhalf.rs"]
mod world_set_waterhalf;
