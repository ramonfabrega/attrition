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
    ), // **The Trireme, piece 290: run127's three rounds** (`docs/COMBAT.md`
    // §50.2, item 542). Its `<RELEASEEVENT>`s are `400`, `666` and `1200`
    // ms on node 0, frames 5, 9 and 17, and node 0 walks down the keel as
    // the swing plays. The first round leaves 45 units ahead of the hull's
    // centre, dead on its facing. The second and third leave 13 and 65
    // behind it. Every round both ships fire in run127 leaves from one of
    // these three points, 16 volley rounds over 622..720 alone, each to
    // the unit, `sz` 88, 88 and 87 over the water's 0.
    //
    // **One facing only.** Both hulls hold `671481856` broadside for the
    // whole capture (§49), so the dump fixes each offset in the world but
    // cannot show it turn. The rotation is §22.2's, read, not measured
    // for this piece. At that facing the first node is the keel exactly;
    // for the other two no stern bearing reproduces both integers, and
    // the row is the solution nearest the keel, 1.3° off it.
    //
    // `CHAR_ATTACK2` plays the same file, `Trireme Attack1`, with the same
    // three events, so it takes the same rows. run127 prints no
    // `cur_anim` at `GUYS=2`, and which slot the original swings is not
    // on this disk.
    (
        290,
        crate::anim::ATTACK1,
        5,
        Node {
            bearing: 0,
            radius: 45,
            dz: 88,
        },
    ),
    (
        290,
        crate::anim::ATTACK1,
        9,
        Node {
            bearing: -2_131_755_008,
            radius: 13,
            dz: 88,
        },
    ),
    (
        290,
        crate::anim::ATTACK1,
        17,
        Node {
            bearing: -2_131_755_008,
            radius: 65,
            dz: 87,
        },
    ),
    (
        290,
        crate::anim::ATTACK2,
        5,
        Node {
            bearing: 0,
            radius: 45,
            dz: 88,
        },
    ),
    (
        290,
        crate::anim::ATTACK2,
        9,
        Node {
            bearing: -2_131_755_008,
            radius: 13,
            dz: 88,
        },
    ),
    (
        290,
        crate::anim::ATTACK2,
        17,
        Node {
            bearing: -2_131_755_008,
            radius: 65,
            dz: 87,
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

    /// The table is the install's own release events and no others — the
    /// key is `rondata::artdata::release_frame`'s frame, so a row this
    /// crate cannot reach is a row that will never fire.
    ///
    /// Piece **127** is the Longbowman's six (run109, §22.1); **472** and
    /// **384** are chapter two's bowmen and slingers, three keys and two,
    /// from run112's own `AMMO` records (item 485, §41.3); **290** is the
    /// Trireme's three, in both attack slots, from run127's (item 542,
    /// §50.2). Its keys are 5, 9 and 17 because the frame is `starttime /
    /// 67` (§50.1): under the old `× 3 / 200` the first and third would be
    /// 6 and 18, keys the event walk never reaches.
    #[test]
    fn the_table_is_every_release_event_a_capture_has_measured() {
        let mut keys: Vec<(i32, i8, u32)> =
            MEASURED.iter().map(|(p, a, t, _)| (*p, *a, *t)).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                (127, crate::anim::ATTACK1, 6),
                (127, crate::anim::ATTACK1, 22),
                (127, crate::anim::ATTACK2, 4),
                (127, crate::anim::ATTACK2, 22),
                (127, crate::anim::ATTACK3, 10),
                (127, crate::anim::ATTACK3, 24),
                (290, crate::anim::ATTACK1, 5),
                (290, crate::anim::ATTACK1, 9),
                (290, crate::anim::ATTACK1, 17),
                (290, crate::anim::ATTACK2, 5),
                (290, crate::anim::ATTACK2, 9),
                (290, crate::anim::ATTACK2, 17),
                (384, crate::anim::ATTACK1, 21),
                (384, crate::anim::ATTACK2, 22),
                (472, crate::anim::ATTACK1, 12),
                (472, crate::anim::ATTACK2, 9),
                (472, crate::anim::ATTACK3, 15),
            ]
        );
    }

    /// **Every launch point run112 recorded**, and the error in each
    /// (`docs/COMBAT.md` §41.3).
    ///
    /// Chapter two's capture asks for `AMMO=5`, so `AmmoData::log_data`
    /// prints each arrow's `sx, sy, sz` on the block it is created; the
    /// nine below are every arrow of the golden window. The columns are
    /// the dump's own — the shooter's guy position and angle from the
    /// `UNITDATA`/`GUY` blocks of the same block, `sx, sy` from the
    /// `AMMO` record beside them.
    ///
    /// **Unlike run109's nine, three of these are a unit or two out.**
    /// A single `(bearing, radius)` turned by the yaw cannot hit all
    /// three of `(472, ATTACK1, 12)`'s rows at once — the original
    /// rotates a float vector with a height in it — so the error is
    /// pinned per row rather than asserted away. It may only shrink: a
    /// table that drifts, or a model that improves, fails here. Nothing
    /// integral depends on the residue, because the flight time is a
    /// truncated `dist / proj_speed` and a unit of distance moves it
    /// only across a boundary; the consequence the fix is measured on is
    /// the impact *frame*, which `rondata::diff`'s chapter-two widening
    /// puts on the dump's own for every arrow whose node is here.
    #[test]
    fn run112_launch_points_are_reproduced_to_within_a_unit() {
        /// `(gpiece, anim, starttime, guy x, guy y, guy angle, sx, sy,
        /// Manhattan error)` — the dump's own columns.
        type Row = (i32, i8, u32, i32, i32, i32, i32, i32, i32);
        const M: &[Row] = &[
            (
                472,
                crate::anim::ATTACK2,
                9,
                888,
                7800,
                1_131_216_896,
                975,
                7806,
                2,
            ),
            (
                472,
                crate::anim::ATTACK2,
                9,
                888,
                7800,
                1_466_499_072,
                961,
                7846,
                0,
            ),
            (
                472,
                crate::anim::ATTACK1,
                12,
                1032,
                7800,
                1_137_115_136,
                1120,
                7804,
                0,
            ),
            (
                472,
                crate::anim::ATTACK1,
                12,
                1032,
                7800,
                1_530_593_280,
                1103,
                7853,
                2,
            ),
            (
                472,
                crate::anim::ATTACK1,
                12,
                936,
                7944,
                1_390_804_992,
                1017,
                7979,
                1,
            ),
            (
                472,
                crate::anim::ATTACK3,
                15,
                936,
                7944,
                1_073_741_824,
                1002,
                7953,
                0,
            ),
            (
                384,
                crate::anim::ATTACK2,
                22,
                1224,
                8184,
                948_568_064,
                1347,
                8201,
                0,
            ),
            (
                384,
                crate::anim::ATTACK2,
                22,
                936,
                8088,
                983_498_752,
                1058,
                8112,
                0,
            ),
            (
                384,
                crate::anim::ATTACK1,
                21,
                1320,
                8328,
                862_191_616,
                1406,
                8373,
                0,
            ),
        ];
        let mut exact = 0;
        for &(gp, anim, t, gx, gy, ga, sx, sy, err) in M {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), gp, anim, t);
            assert_eq!(
                (got.x - sx).abs() + (got.y - sy).abs(),
                err,
                "piece {gp} anim {anim} t {t} from ({gx},{gy}) facing {ga}:                  want ({sx},{sy}), got ({},{})",
                got.x,
                got.y
            );
            exact += i32::from(err == 0);
        }
        // Six of the nine are exact, and none of the rest is more than
        // two units of Manhattan distance out. A model that fits all
        // nine exactly may re-pin every `err` to zero; nothing may
        // raise one.
        assert_eq!(exact, 6, "the number of exactly reproduced rows fell");
    }

    /// **Every trireme launch point run127 recorded on its first two
    /// volleys**, both ships (`docs/COMBAT.md` §50.2). The columns are the
    /// dump's own: the hull's position and angle from its `UNITDATA`, and
    /// `sx, sy, sz` from the `AMMO` record on the block each round first
    /// prints. Both ships hold the same broadside facing, so this pins the
    /// three nodes at one facing, exactly.
    #[test]
    fn run127_trireme_rounds_leave_from_the_keel() {
        /// `(block, hull x, hull y, starttime, sx, sy, sz)`.
        const M: &[(i32, i32, i32, u32, i32, i32, i32)] = &[
            (622, 12408, 35832, 5, 12445, 35807, 88),
            (626, 12408, 35832, 9, 12396, 35839, 88),
            (634, 12408, 35832, 17, 12352, 35867, 87),
            (640, 11640, 34680, 5, 11677, 34655, 88),
            (644, 11640, 34680, 9, 11628, 34687, 88),
            (652, 11640, 34680, 17, 11584, 34715, 87),
        ];
        for &(block, x, y, t, sx, sy, sz) in M {
            for anim in [crate::anim::ATTACK1, crate::anim::ATTACK2] {
                let got = launch_point(Pos::new(x, y), Angle(671_481_856), 290, anim, t);
                assert_eq!((got.x, got.y), (sx, sy), "block {block} t {t}");
                assert_eq!(node(290, anim, t).map(|n| n.dz), Some(sz), "block {block}");
            }
        }
    }
}
