//! Independent sweep partition; test bodies keep their original paths.

#[path = "support.rs"]
mod support;
#[allow(unused_imports)]
use support::{emu_obj, emu_rows, row, tail_row};

// Registered standalone sweeps.
#[path = "activate_view.rs"]
mod activate_view;
#[path = "add_type_bucket.rs"]
mod add_type_bucket;
#[path = "add_type_bucket2.rs"]
mod add_type_bucket2;
#[path = "air_attack_ground_order_vector_deleting_destructor_vtordisp.rs"]
mod air_attack_ground_order_vector_deleting_destructor_vtordisp;
#[path = "air_patrol_order_vector_deleting_destructor_vtordisp.rs"]
mod air_patrol_order_vector_deleting_destructor_vtordisp;
#[path = "ammo_check_hit.rs"]
mod ammo_check_hit;
#[path = "ammo_data_walk_data.rs"]
mod ammo_data_walk_data;
#[path = "ammo_hit_target.rs"]
mod ammo_hit_target;
#[path = "ammo_out_set_shooter_vis.rs"]
mod ammo_out_set_shooter_vis;
#[path = "ammo_process.rs"]
mod ammo_process;
#[path = "angle_diff.rs"]
mod angle_diff;
#[path = "animal_data_is_seen.rs"]
mod animal_data_is_seen;
#[path = "animal_data_valid_target_const.rs"]
mod animal_data_valid_target_const;
#[path = "animal_data_walk_data_vt.rs"]
mod animal_data_walk_data_vt;
#[path = "animal_think_bird.rs"]
mod animal_think_bird;
#[path = "animal_think_farm_animal.rs"]
mod animal_think_farm_animal;
#[path = "animal_update_los.rs"]
mod animal_update_los;
#[path = "animal_work.rs"]
mod animal_work;
#[path = "armies_diplo_change.rs"]
mod armies_diplo_change;
#[path = "armies_emergency.rs"]
mod armies_emergency;
#[path = "armies_find_army.rs"]
mod armies_find_army;
#[path = "armies_find_city.rs"]
mod armies_find_city;
#[path = "armies_find_local_army.rs"]
mod armies_find_local_army;
#[path = "armies_find_useful_army.rs"]
mod armies_find_useful_army;
#[path = "armies_leader_defeated.rs"]
mod armies_leader_defeated;
#[path = "armies_num_armies.rs"]
mod armies_num_armies;
#[path = "armies_process_all.rs"]
mod armies_process_all;
#[path = "armies_send_navy.rs"]
mod armies_send_navy;
#[path = "armies_update_city.rs"]
mod armies_update_city;
#[path = "army_charge.rs"]
mod army_charge;
#[path = "army_count.rs"]
mod army_count;
#[path = "army_data_select.rs"]
mod army_data_select;
#[path = "army_do_defending.rs"]
mod army_do_defending;
#[path = "army_do_mustering.rs"]
mod army_do_mustering;
#[path = "army_engagement.rs"]
mod army_engagement;
#[path = "army_is_engaged.rs"]
mod army_is_engaged;
#[path = "army_is_moving.rs"]
mod army_is_moving;
#[path = "army_member.rs"]
mod army_member;
#[path = "army_member_by_unit.rs"]
mod army_member_by_unit;
#[path = "army_normalize.rs"]
mod army_normalize;
#[path = "army_normalize_one.rs"]
mod army_normalize_one;
#[path = "army_process.rs"]
mod army_process;
#[path = "army_release_mustering.rs"]
mod army_release_mustering;
#[path = "army_remove_group.rs"]
mod army_remove_group;
#[path = "army_send_here.rs"]
mod army_send_here;
#[path = "army_walk_data.rs"]
mod army_walk_data;
#[path = "balance_compute_modifier.rs"]
mod balance_compute_modifier;
#[path = "balance_lookup_absolute_name.rs"]
mod balance_lookup_absolute_name;
#[path = "balance_return_modifier.rs"]
mod balance_return_modifier;
#[path = "balance_type_damage.rs"]
mod balance_type_damage;
#[path = "balance_walk_rules_data.rs"]
mod balance_walk_rules_data;
#[path = "blocked_location_emu.rs"]
mod blocked_location_emu;
#[path = "border_build_border.rs"]
mod border_build_border;
#[path = "border_generate_morph_side.rs"]
mod border_generate_morph_side;
#[path = "border_nodes_find.rs"]
mod border_nodes_find;
#[path = "border_process.rs"]
mod border_process;
#[path = "border_process_morph.rs"]
mod border_process_morph;
#[path = "border_rebuild_visible_list.rs"]
mod border_rebuild_visible_list;
#[path = "border_spline_get_name.rs"]
mod border_spline_get_name;
#[path = "border_spline_on_field_change.rs"]
mod border_spline_on_field_change;
#[path = "border_spline_render_is_visible.rs"]
mod border_spline_render_is_visible;
#[path = "border_spline_render_make_bbox.rs"]
mod border_spline_render_make_bbox;
#[path = "borders_check_border_blocked.rs"]
mod borders_check_border_blocked;
#[path = "borders_get_who.rs"]
mod borders_get_who;
#[path = "borders_is_border_hidden.rs"]
mod borders_is_border_hidden;
#[path = "borders_out_no_border.rs"]
mod borders_out_no_border;
#[path = "borders_process_all.rs"]
mod borders_process_all;
#[path = "borders_remove_border_region.rs"]
mod borders_remove_border_region;
#[path = "borders_save_last_who.rs"]
mod borders_save_last_who;
#[path = "build_activate_arms.rs"]
mod build_activate_arms;
#[path = "build_activate_head.rs"]
mod build_activate_head;
#[path = "build_activate_senate.rs"]
mod build_activate_senate;
#[path = "build_add_attack_order.rs"]
mod build_add_attack_order;
#[path = "build_add_gather_point.rs"]
mod build_add_gather_point;
#[path = "build_add_gatherer.rs"]
mod build_add_gatherer;
#[path = "build_add_to_city.rs"]
mod build_add_to_city;
#[path = "build_check_capture.rs"]
mod build_check_capture;
#[path = "build_check_capture_garrison.rs"]
mod build_check_capture_garrison;
#[path = "build_data_attack.rs"]
mod build_data_attack;
#[path = "build_data_can_make.rs"]
mod build_data_can_make;
#[path = "build_data_can_queue.rs"]
mod build_data_can_queue;
#[path = "build_data_can_queue_4d2030.rs"]
mod build_data_can_queue_4d2030;
#[path = "build_data_check_capture_eligible.rs"]
mod build_data_check_capture_eligible;
#[path = "build_data_check_capture_health.rs"]
mod build_data_check_capture_health;
#[path = "build_data_construct_time.rs"]
mod build_data_construct_time;
#[path = "build_data_could_queue.rs"]
mod build_data_could_queue;
#[path = "build_data_find_in_queue.rs"]
mod build_data_find_in_queue;
#[path = "build_data_get_base_garrison_arrows.rs"]
mod build_data_get_base_garrison_arrows;
#[path = "build_data_get_caster_stance.rs"]
mod build_data_get_caster_stance;
#[path = "build_data_get_combat_stance.rs"]
mod build_data_get_combat_stance;
#[path = "build_data_get_first_gather.rs"]
mod build_data_get_first_gather;
#[path = "build_data_get_garrison_arrows.rs"]
mod build_data_get_garrison_arrows;
#[path = "build_data_get_most_shots.rs"]
mod build_data_get_most_shots;
#[path = "build_data_get_packer_stance.rs"]
mod build_data_get_packer_stance;
#[path = "build_data_get_queue.rs"]
mod build_data_get_queue;
#[path = "build_data_get_shot.rs"]
mod build_data_get_shot;
#[path = "build_data_get_worker_stance.rs"]
mod build_data_get_worker_stance;
#[path = "build_data_has_objmask.rs"]
mod build_data_has_objmask;
#[path = "build_data_hits.rs"]
mod build_data_hits;
#[path = "build_data_is_gathered_by.rs"]
mod build_data_is_gathered_by;
#[path = "build_data_is_infinite.rs"]
mod build_data_is_infinite;
#[path = "build_data_is_now_seen.rs"]
mod build_data_is_now_seen;
#[path = "build_data_is_unassimilated.rs"]
mod build_data_is_unassimilated;
#[path = "build_data_is_wonder.rs"]
mod build_data_is_wonder;
#[path = "build_data_max_gatherers.rs"]
mod build_data_max_gatherers;
#[path = "build_data_most_shots.rs"]
mod build_data_most_shots;
#[path = "build_data_num_gather.rs"]
mod build_data_num_gather;
#[path = "build_data_num_scholars.rs"]
mod build_data_num_scholars;
#[path = "build_data_show_later_tech.rs"]
mod build_data_show_later_tech;
#[path = "build_do_attack.rs"]
mod build_do_attack;
#[path = "build_find_gather_tiles.rs"]
mod build_find_gather_tiles;
#[path = "build_find_target.rs"]
mod build_find_target;
#[path = "build_finished.rs"]
mod build_finished;
#[path = "build_finished_628490.rs"]
mod build_finished_628490;
#[path = "build_new_library.rs"]
mod build_new_library;
#[path = "build_plunder.rs"]
mod build_plunder;
#[path = "build_process_city.rs"]
mod build_process_city;
#[path = "build_process_ejection.rs"]
mod build_process_ejection;
#[path = "build_process_head.rs"]
mod build_process_head;
#[path = "build_process_messages.rs"]
mod build_process_messages;
#[path = "build_process_supply.rs"]
mod build_process_supply;
#[path = "build_process_wonders.rs"]
mod build_process_wonders;
#[path = "build_queue_data_get_job_counter.rs"]
mod build_queue_data_get_job_counter;
#[path = "build_queue_data_get_next_helicopter.rs"]
mod build_queue_data_get_next_helicopter;
#[path = "build_queue_data_get_next_non_caravan.rs"]
mod build_queue_data_get_next_non_caravan;
#[path = "build_queue_data_get_next_non_unit.rs"]
mod build_queue_data_get_next_non_unit;
#[path = "build_queue_data_get_queue.rs"]
mod build_queue_data_get_queue;
#[path = "build_queue_get_size.rs"]
mod build_queue_get_size;
#[path = "build_queue_set_job_counter.rs"]
mod build_queue_set_job_counter;
#[path = "build_queue_set_queue.rs"]
mod build_queue_set_queue;
#[path = "build_queue_un_queue.rs"]
mod build_queue_un_queue;
#[path = "build_queue_walk_data.rs"]
mod build_queue_walk_data;
#[path = "build_remove_from_city.rs"]
mod build_remove_from_city;
#[path = "build_remove_gatherer.rs"]
mod build_remove_gatherer;
#[path = "build_repair_damage.rs"]
mod build_repair_damage;
#[path = "build_swap_team.rs"]
mod build_swap_team;
#[path = "build_type_backup.rs"]
mod build_type_backup;
#[path = "build_type_corner_tile.rs"]
mod build_type_corner_tile;
#[path = "build_type_data_already_built.rs"]
mod build_type_data_already_built;
#[path = "build_type_data_area.rs"]
mod build_type_data_area;
#[path = "build_type_data_basic_type.rs"]
mod build_type_data_basic_type;
#[path = "build_type_data_blocked_location.rs"]
mod build_type_data_blocked_location;
#[path = "build_type_data_blocked_location_tail.rs"]
mod build_type_data_blocked_location_tail;
#[path = "build_type_data_blocked_location_town.rs"]
mod build_type_data_blocked_location_town;
#[path = "build_type_data_blocked_site.rs"]
mod build_type_data_blocked_site;
#[path = "build_type_data_blocked_tcoord.rs"]
mod build_type_data_blocked_tcoord;
#[path = "build_type_data_blocked_town.rs"]
mod build_type_data_blocked_town;
#[path = "build_type_data_calc_gather_a.rs"]
mod build_type_data_calc_gather_a;
#[path = "build_type_data_calc_gather_b.rs"]
mod build_type_data_calc_gather_b;
#[path = "build_type_data_calc_gather_c.rs"]
mod build_type_data_calc_gather_c;
#[path = "build_type_data_calc_gather_d.rs"]
mod build_type_data_calc_gather_d;
#[path = "build_type_data_calc_gather_emu.rs"]
mod build_type_data_calc_gather_emu;
#[path = "build_type_data_can_capture.rs"]
mod build_type_data_can_capture;
#[path = "build_type_data_check_enemy_adjacent.rs"]
mod build_type_data_check_enemy_adjacent;
#[path = "build_type_data_connects_to_roads.rs"]
mod build_type_data_connects_to_roads;
#[path = "build_type_data_count_mountains_adjacent.rs"]
mod build_type_data_count_mountains_adjacent;
#[path = "build_type_data_count_trees_adjacent.rs"]
mod build_type_data_count_trees_adjacent;
#[path = "build_type_data_find_friends.rs"]
mod build_type_data_find_friends;
#[path = "build_type_data_fundamental_type.rs"]
mod build_type_data_fundamental_type;
#[path = "build_type_data_gather_radius.rs"]
mod build_type_data_gather_radius;
#[path = "build_type_data_get_city_level.rs"]
mod build_type_data_get_city_level;
#[path = "build_type_data_get_garrison_limit.rs"]
mod build_type_data_get_garrison_limit;
#[path = "build_type_data_get_kill_value.rs"]
mod build_type_data_get_kill_value;
#[path = "build_type_data_get_stance_type.rs"]
mod build_type_data_get_stance_type;
#[path = "build_type_data_has_stance_type.rs"]
mod build_type_data_has_stance_type;
#[path = "build_type_data_is_city.rs"]
mod build_type_data_is_city;
#[path = "build_type_data_is_civilian.rs"]
mod build_type_data_is_civilian;
#[path = "build_type_data_is_defensive.rs"]
mod build_type_data_is_defensive;
#[path = "build_type_data_is_dock.rs"]
mod build_type_data_is_dock;
#[path = "build_type_data_is_embassy.rs"]
mod build_type_data_is_embassy;
#[path = "build_type_data_is_flat.rs"]
mod build_type_data_is_flat;
#[path = "build_type_data_is_fort.rs"]
mod build_type_data_is_fort;
#[path = "build_type_data_is_gather_enhancer.rs"]
mod build_type_data_is_gather_enhancer;
#[path = "build_type_data_is_gather_type.rs"]
mod build_type_data_is_gather_type;
#[path = "build_type_data_is_military_trainer.rs"]
mod build_type_data_is_military_trainer;
#[path = "build_type_data_is_mine.rs"]
mod build_type_data_is_mine;
#[path = "build_type_data_is_spellcaster.rs"]
mod build_type_data_is_spellcaster;
#[path = "build_type_data_is_tower.rs"]
mod build_type_data_is_tower;
#[path = "build_type_data_is_training_building.rs"]
mod build_type_data_is_training_building;
#[path = "build_type_data_makes_roads.rs"]
mod build_type_data_makes_roads;
#[path = "build_type_data_max_flat_gatherers.rs"]
mod build_type_data_max_flat_gatherers;
#[path = "build_type_data_max_knowledge_gatherers.rs"]
mod build_type_data_max_knowledge_gatherers;
#[path = "build_type_data_num_preq.rs"]
mod build_type_data_num_preq;
#[path = "build_type_data_queue_here.rs"]
mod build_type_data_queue_here;
#[path = "build_type_data_snap_center.rs"]
mod build_type_data_snap_center;
#[path = "build_type_mask_me.rs"]
mod build_type_mask_me;
#[path = "build_type_place_roads.rs"]
mod build_type_place_roads;
#[path = "build_type_restore.rs"]
mod build_type_restore;
#[path = "build_type_set_domain.rs"]
mod build_type_set_domain;
#[path = "build_type_walk_rules_data.rs"]
mod build_type_walk_rules_data;
#[path = "build_unpay_cost.rs"]
mod build_unpay_cost;
#[path = "build_unqueue.rs"]
mod build_unqueue;
#[path = "build_unqueue_short.rs"]
mod build_unqueue_short;
#[path = "build_update_max_gatherers.rs"]
mod build_update_max_gatherers;
#[path = "build_verify_gather_tiles.rs"]
mod build_verify_gather_tiles;
#[path = "caravan_data_has_city.rs"]
mod caravan_data_has_city;
#[path = "caravan_distance.rs"]
mod caravan_distance;
#[path = "caravan_link_walk_data.rs"]
mod caravan_link_walk_data;
#[path = "caravan_process.rs"]
mod caravan_process;
#[path = "caravan_restart_trade_route.rs"]
mod caravan_restart_trade_route;
#[path = "caravan_verify_road.rs"]
mod caravan_verify_road;
#[path = "caravan_walk_data.rs"]
mod caravan_walk_data;
#[path = "caravans_new_danger.rs"]
mod caravans_new_danger;
#[path = "caster_process_spells.rs"]
mod caster_process_spells;
#[path = "caster_walk_data.rs"]
mod caster_walk_data;
#[path = "categories_find_data.rs"]
mod categories_find_data;
#[path = "categories_find_key.rs"]
mod categories_find_key;
#[path = "circle_render_get_name.rs"]
mod circle_render_get_name;
#[path = "circle_render_on_field_change.rs"]
mod circle_render_on_field_change;
#[path = "cities_capture_city.rs"]
mod cities_capture_city;
#[path = "city_assimilate.rs"]
mod city_assimilate;
#[path = "city_calc_gather.rs"]
mod city_calc_gather;
#[path = "city_check_upgrade.rs"]
mod city_check_upgrade;
#[path = "city_could_queue.rs"]
mod city_could_queue;
#[path = "city_count_gather_slots.rs"]
mod city_count_gather_slots;
#[path = "city_data_count_farms.rs"]
mod city_data_count_farms;
#[path = "city_data_depop_can_queue.rs"]
mod city_data_depop_can_queue;
#[path = "city_data_enhancer_amount.rs"]
mod city_data_enhancer_amount;
#[path = "city_data_enhancer_level.rs"]
mod city_data_enhancer_level;
#[path = "city_data_get_building.rs"]
mod city_data_get_building;
#[path = "city_data_get_farm_limit.rs"]
mod city_data_get_farm_limit;
#[path = "city_data_get_level.rs"]
mod city_data_get_level;
#[path = "city_data_get_literacy.rs"]
mod city_data_get_literacy;
#[path = "city_data_get_pop_value.rs"]
mod city_data_get_pop_value;
#[path = "city_data_get_radius.rs"]
mod city_data_get_radius;
#[path = "city_data_get_taxes.rs"]
mod city_data_get_taxes;
#[path = "city_data_get_trade_value.rs"]
mod city_data_get_trade_value;
#[path = "city_data_granary_level.rs"]
mod city_data_granary_level;
#[path = "city_data_is_disbanding.rs"]
mod city_data_is_disbanding;
#[path = "city_data_is_seen.rs"]
mod city_data_is_seen;
#[path = "city_data_is_unassimilated.rs"]
mod city_data_is_unassimilated;
#[path = "city_data_lumber_level.rs"]
mod city_data_lumber_level;
#[path = "city_data_num_city_buildings.rs"]
mod city_data_num_city_buildings;
#[path = "city_data_num_kinds_needed.rs"]
mod city_data_num_kinds_needed;
#[path = "city_data_pop_cap.rs"]
mod city_data_pop_cap;
#[path = "city_data_pop_growth.rs"]
mod city_data_pop_growth;
#[path = "city_data_ready_to_upgrade.rs"]
mod city_data_ready_to_upgrade;
#[path = "city_data_smelter_level.rs"]
mod city_data_smelter_level;
#[path = "city_data_upgrades_to.rs"]
mod city_data_upgrades_to;
#[path = "city_find_buildings.rs"]
mod city_find_buildings;
#[path = "city_get_trade_routes.rs"]
mod city_get_trade_routes;
#[path = "city_set_name.rs"]
mod city_set_name;
#[path = "city_walk_data.rs"]
mod city_walk_data;
#[path = "cliff_average_neighbor_norms.rs"]
mod cliff_average_neighbor_norms;
#[path = "cliff_get_vert_offsets.rs"]
mod cliff_get_vert_offsets;
#[path = "cliff_mining_data_walk_data.rs"]
mod cliff_mining_data_walk_data;
#[path = "cliff_render.rs"]
mod cliff_render;
#[path = "cliff_walk_data.rs"]
mod cliff_walk_data;
#[path = "cliffs_out_get_mining_data.rs"]
mod cliffs_out_get_mining_data;
#[path = "cliffs_out_get_mining_size.rs"]
mod cliffs_out_get_mining_size;
#[path = "cliffs_set_edge_flags.rs"]
mod cliffs_set_edge_flags;
#[path = "coll_block_get.rs"]
mod coll_block_get;
#[path = "coll_check_collide_here.rs"]
mod coll_check_collide_here;
#[path = "constants_walk_data.rs"]
mod constants_walk_data;
#[path = "cosx.rs"]
mod cosx;
#[path = "death_obj_data_blocking.rs"]
mod death_obj_data_blocking;
#[path = "death_obj_data_walk_data.rs"]
mod death_obj_data_walk_data;
#[path = "dock_walk_data.rs"]
mod dock_walk_data;
#[path = "doober_find_x_pos.rs"]
mod doober_find_x_pos;
#[path = "doober_find_y_pos.rs"]
mod doober_find_y_pos;
#[path = "doober_get_num.rs"]
mod doober_get_num;
#[path = "doober_has_doobers.rs"]
mod doober_has_doobers;
#[path = "doober_inc_time.rs"]
mod doober_inc_time;
#[path = "doober_remove_cover_doober.rs"]
mod doober_remove_cover_doober;
#[path = "doober_remove_doobers.rs"]
mod doober_remove_doobers;
#[path = "doober_remove_hold_doobers.rs"]
mod doober_remove_hold_doobers;
#[path = "draw_thick_diamond.rs"]
mod draw_thick_diamond;
#[path = "farms_add_animals_all.rs"]
mod farms_add_animals_all;
#[path = "farms_data_get_farm_type.rs"]
mod farms_data_get_farm_type;
#[path = "farms_data_get_nearest_farm_type.rs"]
mod farms_data_get_nearest_farm_type;
#[path = "farms_data_get_wheat_status.rs"]
mod farms_data_get_wheat_status;
#[path = "farms_grow.rs"]
mod farms_grow;
#[path = "farms_out_get_name.rs"]
mod farms_out_get_name;
#[path = "farms_remove.rs"]
mod farms_remove;
#[path = "farms_snip.rs"]
mod farms_snip;
#[path = "farms_walk_data.rs"]
mod farms_walk_data;
#[path = "find_angle.rs"]
mod find_angle;
#[path = "flanking.rs"]
mod flanking;
#[path = "fog_master_get_name.rs"]
mod fog_master_get_name;
#[path = "fort_walk_data.rs"]
mod fort_walk_data;
#[path = "forts_find_fort.rs"]
mod forts_find_fort;
#[path = "fractal_get_height.rs"]
mod fractal_get_height;
