//! Where a shot starts — the release node's world position
//! (`docs/COMBAT.md` §22).
//!
//! `GraphicEvents::execute_game_events@008e48e0+0x40d` adds
//! `GraphicPieces::get_position`'s vector to the **guy's** `x/y/z`
//! immediately before `Objects::add_ammo`, and that vector is per
//! `(piece, node, anim, starttime)`: the bow hand is in a different place
//! four frames into `CHAR_ATTACK2` than ten frames into `CHAR_ATTACK3`.
//! The original reads it out of `GraphicPieces::positions`, an array
//! `init_position@00901820` fills at load by running the model's own
//! animation — so it is already `docs/DECISIONS.md` entry 16's second
//! shape, a table built once before the first frame, and this module is
//! the same table with the building step replaced by a measurement.
//!
//! **Measured, not derived.** run109 raised `AMMO=5` over `[9420, 9480)`
//! on the Great Lakes seed, and `AmmoData::log_data` prints the arrow's own
//! `sx, sy, sz`; subtracting the shooter's guy position gives the offset
//! directly. Nine shots, three Longbowmen, two facings, six release
//! events — every one the type has. `docs/RUNS.md`, run109.
//!
//! The entry is stored the way the rest of this crate stores a direction:
//! a bearing **relative to the guy's facing** and a radius, so the world
//! offset is `movement`'s own integer sine and nothing else. That is what
//! lets the port match `get_position`'s float rotate without one — the
//! six entries below reproduce all nine measured integer launch points
//! exactly, including the two that were taken at different facings.

use crate::movement::{Angle, cos_component, sin_component};
use crate::world::Pos;

/// One release node in the shooter's own frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    /// Bearing from the guy's facing, in the engine's 32-bit angle.
    pub bearing: i32,
    /// Distance from the guy's position, in position units.
    pub radius: i32,
    /// Height above the guy. Recorded because the record prints it; the
    /// flight time is a plan distance, so nothing reads it yet.
    pub dz: i32,
}

/// `(piece, animation slot, starttime)` → node.
///
/// The piece key is [`crate::anim::Guy::gpiece`] — the same number the
/// original's own `GUY` record prints, and the key `PieceReleases` is
/// built on (`PieceName::piece`). **127 is `LONGBOWMEN`**, whose unit
/// type is 177: the piece is the graph index and the type is
/// `0x32 + it`, and the two are one apart by a constant only because
/// this graph happens to be its own type's. The starttime key is the one
/// `rondata::artdata::release_frame` produces from the install's
/// `<RELEASEEVENT starttime>`, and the six rows below are exactly the six
/// that file gives the Longbowman.
///
/// A piece with no row here launches from the unit's own position, which
/// is what every shot in this crate did before run109.
const MEASURED: &[(i32, i8, u32, Node)] = &[
    (
        127,
        crate::anim::ATTACK1,
        6,
        Node {
            bearing: -26_692_241,
            radius: 81,
            dz: 169,
        },
    ),
    (
        127,
        crate::anim::ATTACK1,
        22,
        Node {
            bearing: -26_692_241,
            radius: 81,
            dz: 169,
        },
    ),
    (
        127,
        crate::anim::ATTACK2,
        4,
        Node {
            bearing: -136_574_224,
            radius: 82,
            dz: 137,
        },
    ),
    (
        127,
        crate::anim::ATTACK2,
        22,
        Node {
            bearing: -36_624_995,
            radius: 80,
            dz: 169,
        },
    ),
    (
        127,
        crate::anim::ATTACK3,
        10,
        Node {
            bearing: 106_731_108,
            radius: 70,
            dz: 185,
        },
    ),
    (
        127,
        crate::anim::ATTACK3,
        24,
        Node {
            bearing: 59_520_002,
            radius: 64,
            dz: 184,
        },
    ),
    // **run112's own nine arrows** (`docs/COMBAT.md` §41.3, item 485).
    // Chapter two's `--detail end:` carries `AMMO=5`, so the golden
    // capture has printed `AmmoData::log_data`'s `sx, sy, sz` on every
    // block since it was taken and nothing had read them: the two pieces
    // the chapter stages had no row here, so every shot in the
    // engagement left the shooter's own square while the original's left
    // the bow hand eighty-odd units ahead of it. That shortens the flight
    // by one frame on four of the nine and put the whole wound ladder one
    // arrival behind the dump from block 656 (§40.4).
    //
    // Piece **472** is the bowmen's and **384** the slingers'. Each row
    // is solved from every arrow the capture carries for its key — three
    // for `(472, ATTACK1, 12)`, two each for `(472, ATTACK2, 9)` and
    // `(384, ATTACK2, 22)`, one each for the other two — at two or three
    // different facings, which is what makes it a rotation rather than a
    // stored world vector.
    //
    // **Two of the nine are reproduced to a unit rather than exactly**,
    // unlike run109's (§22.1): the planar `(bearing, radius)` model
    // cannot hit all three of `(472, ATTACK1, 12)`'s rows at once, and
    // `(472, ATTACK2, 9)` is two units out on one of its two. §41.3 says
    // what that leaves open. Nothing integral depends on it — the flight
    // time is `dist / proj_speed` truncated, and a unit of distance moves
    // it only across a boundary — and the consequence the fix is measured
    // on is the impact *frame*, which is now the dump's on eight of the
    // nine.
    (
        472,
        crate::anim::ATTACK2,
        9,
        Node {
            bearing: -17_683_648,
            radius: 86,
            dz: 164,
        },
    ),
    (
        472,
        crate::anim::ATTACK1,
        12,
        Node {
            bearing: -37_883_648,
            radius: 89,
            dz: 163,
        },
    ),
    (
        472,
        crate::anim::ATTACK3,
        15,
        Node {
            bearing: 81_516_352,
            radius: 67,
            dz: 164,
        },
    ),
    (
        384,
        crate::anim::ATTACK2,
        22,
        Node {
            bearing: 216_316_352,
            radius: 125,
            dz: 151,
        },
    ),
    (
        384,
        crate::anim::ATTACK1,
        21,
        Node {
            bearing: 532_316_352,
            radius: 97,
            dz: 178,
        },
    ),
];

/// The node a piece releases from, or `None` when nothing has measured it.
pub fn node(gpiece: i32, anim: i8, starttime: u32) -> Option<Node> {
    MEASURED
        .iter()
        .find(|(p, a, t, _)| *p == gpiece && *a == anim && *t == starttime)
        .map(|(_, _, _, n)| *n)
}

/// The world point a shot leaves from: the guy's position plus the node,
/// turned by the guy's facing.
pub fn point(pos: Pos, facing: Angle, n: Node) -> Pos {
    let a = Angle(facing.0.wrapping_add(n.bearing));
    Pos::new(
        pos.x + sin_component(a, n.radius),
        pos.y - cos_component(a, n.radius),
    )
}

/// [`point`] for a piece that has a row, and `pos` for one that does not.
pub fn launch_point(pos: Pos, facing: Angle, gpiece: i32, anim: i8, starttime: u32) -> Pos {
    match node(gpiece, anim, starttime) {
        Some(n) => point(pos, facing, n),
        None => pos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every launch point run109 measured, reproduced to the unit.
    ///
    /// `(guy x, guy y, guy angle, anim, starttime, sx, sy)` — the dump's
    /// own columns, `docs/COMBAT.md` §22.1. Three units and two facings,
    /// and 1/27's two rows are the ones that make this a rotation rather
    /// than a stored world vector: same node, different facing, different
    /// answer, and the same two numbers.
    #[test]
    fn run109_launch_points_are_reproduced_to_the_unit() {
        const M: &[(i32, i32, i32, i8, u32, i32, i32)] = &[
            (
                4680,
                29928,
                -1_408_303_104,
                crate::anim::ATTACK3,
                10,
                4613,
                29951,
            ),
            (
                4776,
                30168,
                -1_348_206_592,
                crate::anim::ATTACK2,
                4,
                4708,
                30215,
            ),
            (
                4680,
                29928,
                -1_408_303_104,
                crate::anim::ATTACK3,
                24,
                4620,
                29954,
            ),
            (
                4776,
                30168,
                -1_348_206_592,
                crate::anim::ATTACK2,
                22,
                4703,
                30204,
            ),
            (
                4680,
                29928,
                -1_408_303_104,
                crate::anim::ATTACK1,
                6,
                4609,
                29969,
            ),
            (
                4584,
                29784,
                -1_449_000_960,
                crate::anim::ATTACK2,
                4,
                4523,
                29840,
            ),
            (
                4680,
                29928,
                -1_408_303_104,
                crate::anim::ATTACK1,
                22,
                4609,
                29969,
            ),
        ];
        for &(gx, gy, ga, anim, t, sx, sy) in M {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), 127, anim, t);
            assert_eq!(
                (got.x, got.y),
                (sx, sy),
                "anim {anim} t {t} from ({gx},{gy}) facing {ga}"
            );
        }
    }

    /// A piece nothing has measured keeps the old behaviour exactly.
    #[test]
    fn an_unmeasured_piece_launches_from_the_unit() {
        let p = Pos::new(1234, 5678);
        assert_eq!(
            launch_point(p, Angle(0x1234_5678), 32, crate::anim::ATTACK1, 6),
            p
        );
        assert_eq!(launch_point(p, Angle(0), 127, crate::anim::ATTACK1, 7), p);
    }

    /// The table is the install's own six release events and no others —
    /// the key is `rondata::artdata::release_frame`'s frame, so a row this
    /// crate cannot reach is a row that will never fire.
    #[test]
    fn the_table_is_the_longbowman_s_six_release_events() {
        let mut keys: Vec<(i8, u32)> = MEASURED.iter().map(|(_, a, t, _)| (*a, *t)).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                (crate::anim::ATTACK1, 6),
                (crate::anim::ATTACK1, 22),
                (crate::anim::ATTACK2, 4),
                (crate::anim::ATTACK2, 22),
                (crate::anim::ATTACK3, 10),
                (crate::anim::ATTACK3, 24),
            ]
        );
        assert!(MEASURED.iter().all(|(p, _, _, _)| *p == 127));
    }
}
