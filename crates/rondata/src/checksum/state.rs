//! State checksums from the uncorrected walk (CHECKSUM §8 and §24).
use crate::TribeDef;
use crate::artdata::BuildLook;
use sim::checksum::Line;

pub(crate) fn build_pieces_with(
    sim: &sim::Sim,
    look: &crate::artdata::BuildLook,
    tribes: &[crate::TribeDef],
) -> std::collections::BTreeMap<usize, i32> {
    let scene = sim.observe(None);
    let placed = crate::free::placed_buildings(&scene, look, tribes);
    let mut out = std::collections::BTreeMap::new();
    for (i, b) in sim.buildings.iter().enumerate() {
        if let Some(p) = placed
            .iter()
            .find(|p| p.who == b.owner && p.o == b.index && b.alive)
        {
            out.insert(i, p.piece);
        }
    }
    out
}

/// **What the three empty lines' walkers would find on our side**
/// (`docs/CHECKSUM.md` §8, item 4355): `[walls, shots in flight, death
/// objects]` at the moment the other lines are taken.
///
/// - Walls: `check_walls@00937360` walks each seated leader's objects
///   from `obj_base[2]` (the 3000 band) to its mark whose data
///   `flags & 1`; ours are the live [`sim::Building`]s at
///   `index >= WALL_BASE`.
/// - Ammo: `check_ammo@009374e0` walks the pool at `Objects +0x12c` whose
///   element's `flags & 3`; ours is [`sim::combat::CombatState`]'s
///   `projectiles`, whose every entry is in flight (a landing removes it).
/// - Deaths: `check_deaths@00936bb0` walks the 100 `DeathObj` slots at
///   `Objects +0x14c` whose first word is not 0; ours are the dead units'
///   [`sim::combat::Death`]s, each culled with its last live record — so
///   an empty list is no live record, and the count is units, not records.
pub(crate) fn held(sim: &sim::Sim) -> [usize; 3] {
    let walls = sim
        .buildings
        .iter()
        .filter(|b| b.alive && b.index >= sim::WALL_BASE)
        .count();
    [walls, sim.combat.projectiles.len(), sim.combat.deaths.len()]
}

/// Every available state line, in check_all order. Keep an unknown occupied
/// pool distinct from a known empty pool (whose Adler is one).
pub(crate) fn ranges(sim: &sim::Sim, look: &BuildLook, tribes: &[TribeDef]) -> [Option<Line>; 12] {
    let p = build_pieces_with(sim, look, tribes);
    let [walls, shots, _] = held(sim);
    [
        Some(sim.checksum_units()),
        Some(sim.checksum_builds(&|b| p.get(&b).copied().unwrap_or(-1))),
        (walls == 0).then(Line::default),
        (shots == 0).then(Line::default),
        sim.checksum_deaths(),
        Some(sim.checksum_groups()),
        Some(sim.checksum_guys()),
        Some(sim.checksum_leaders()),
        Some(sim.checksum_cities()),
        Some(sim.checksum_items()),
        Some(sim.checksum_goods()),
        Some(sim.checksum_world()),
    ]
}

pub(super) fn adlers(sim: &sim::Sim, look: &BuildLook, tribes: &[TribeDef]) -> [Option<u32>; 12] {
    ranges(sim, look, tribes).map(|line| line.map(|l| l.adler()))
}
