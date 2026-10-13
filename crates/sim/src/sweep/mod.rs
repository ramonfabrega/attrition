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

mod attack_ground_order_get_type;
mod attack_order_get_type;
mod build_order_get_type;
mod build_type_corner_tile;
mod build_type_get_good;
mod city_get_empty_trade_routes;
mod cosx;
mod farms_grow;
mod farms_snip;
mod find_angle;
mod flanking;
mod follow_order_get_type;
mod gather_point_is_inside;
mod good_type_compute_largest_gather;
mod is_air;
mod is_move;
mod object_get_captain;
mod object_has_range;
mod object_is_in_range;
mod object_is_peasant;
mod object_is_scholar;
mod object_is_worker;
mod object_max_range;
mod object_min_range;
mod object_recharge;
mod repair_order_get_type;
mod reversing;
mod subobject_is_active;
mod tcoord_less;
mod think_order_get_type;
mod type_bonus;
mod type_build;
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
mod unit_type_get_stance_type;
mod unit_type_is_caravan;
mod unit_type_is_siege;
mod unit_type_is_spellcaster;
mod unit_type_is_tank;
mod unit_type_needs_packing;
mod unit_type_num_preq;
mod unit_uber_size;
mod wall_construct_time;
mod wall_has_repeat_air;
mod wall_is_active;
mod wall_is_started;
mod wall_is_under_attack;
mod wcoord_to_tcoord;
mod world_get_land;
mod world_get_tregion;
mod world_has_blocked_neighbors;
mod world_is_blocked_at;
mod world_is_built_at;
mod world_is_cliff_at;
mod world_is_gathered_from;
mod world_is_mountain_at;
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
