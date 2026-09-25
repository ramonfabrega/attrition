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

/// **A release node the original turns by a whole degree** (item 770,
/// `docs/ORDERS.md` §35.2): the model's own vector, in thousandths of a
/// position unit — `right` across the heading, `fwd` along it — and the
/// height under or over the figure.
///
/// `GraphicPieces::get_position@0090b750`'s non-pivot arm builds its
/// rotation from `(int)param_5`, and `param_5` is
/// `fast_angle_to_degrees@00a28f70` of the figure's angle, whose table
/// is filled at `angle << 24` steps: **a whole degree, the nearest to the
/// angle's top byte**. The vector is turned by it in singles and each
/// axis truncated toward zero when `execute_game_events` adds it to the
/// figure. [`Bay::point`] does the same in integers with [`COS_DEG`].
///
/// Measured, not read: run235 dumps all 49 of the Bomber pair's bombs,
/// at thirteen whole-degree headings between 80 and 257, and each row's
/// vector is the centre of the region that reproduces every one of its
/// node's points to the unit. The planar `(bearing, radius)` [`Node`]
/// with the engine's own sine misses 17 of 25.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bay {
    /// Across the heading, to its right, in thousandths.
    pub right: i64,
    /// Along the heading, in thousandths.
    pub fwd: i64,
    /// Height above the figure.
    pub dz: i32,
}

/// `cos(d°) · 2^30`, nearest, for `d` in `0..=90` — a pinned table, built
/// once with doubles and checked against the host's in
/// `the_whole_degree_table_is_the_cosine`.
const COS_DEG: [i64; 91] = [
    1_073_741_824,
    1_073_578_288,
    1_073_087_729,
    1_072_270_298,
    1_071_126_243,
    1_069_655_912,
    1_067_859_754,
    1_065_738_315,
    1_063_292_242,
    1_060_522_280,
    1_057_429_273,
    1_054_014_162,
    1_050_277_989,
    1_046_221_891,
    1_041_847_103,
    1_037_154_959,
    1_032_146_887,
    1_026_824_413,
    1_021_189_159,
    1_015_242_840,
    1_008_987_269,
    1_002_424_350,
    995_556_083,
    988_384_560,
    980_911_966,
    973_140_576,
    965_072_759,
    956_710_970,
    948_057_759,
    939_115_760,
    929_887_697,
    920_376_381,
    910_584_710,
    900_515_665,
    890_172_315,
    879_557_810,
    868_675_383,
    857_528_349,
    846_120_104,
    834_454_122,
    822_533_958,
    810_363_241,
    797_945_680,
    785_285_058,
    772_385_229,
    759_250_125,
    745_883_746,
    732_290_163,
    718_473_518,
    704_438_018,
    690_187_940,
    675_727_625,
    661_061_475,
    646_193_961,
    631_129_609,
    615_873_009,
    600_428_808,
    584_801_711,
    568_996_477,
    553_017_922,
    536_870_912,
    520_560_366,
    504_091_252,
    487_468_587,
    470_697_435,
    453_782_903,
    436_730_145,
    419_544_355,
    402_230_767,
    384_794_656,
    367_241_333,
    349_576_144,
    331_804_471,
    313_931_728,
    295_963_357,
    277_904_834,
    259_761_657,
    241_539_355,
    223_243_478,
    204_879_599,
    186_453_311,
    167_970_228,
    149_435_979,
    130_856_211,
    112_236_583,
    93_582_766,
    74_900_443,
    56_195_305,
    37_473_049,
    18_739_379,
    0,
];

/// `fast_angle_to_degrees`' whole degree for an angle: its top byte in
/// 256ths of a turn, to the nearest degree. SEAM: the half-way bytes
/// (`byte ≡ 16 mod 32`, 22.5° and its kin) are taken to the even degree;
/// no measured heading sits on one.
pub fn whole_degrees(facing: Angle) -> i64 {
    let byte = i64::from((facing.0 as u32) >> 24);
    let (q, r) = ((byte * 45) / 32, (byte * 45) % 32);
    if r > 16 || (r == 16 && q % 2 == 1) {
        q + 1
    } else {
        q
    }
}

/// `(cos, sin)` of a whole degree, scaled by 2^30.
fn cos_sin(d: i64) -> (i64, i64) {
    let c = |d: i64| match d.rem_euclid(360) {
        d @ 0..=90 => COS_DEG[d as usize],
        d @ 91..=180 => -COS_DEG[(180 - d) as usize],
        d @ 181..=270 => -COS_DEG[(d - 180) as usize],
        d => COS_DEG[(360 - d) as usize],
    };
    (c(d), c(d + 270))
}

impl Bay {
    /// The world point the round leaves from: the figure's point plus the
    /// vector turned by the whole degree, each axis truncated toward zero.
    pub fn point(self, pos: Pos, facing: Angle) -> Pos {
        let (c, s) = cos_sin(whole_degrees(facing));
        let den = 1000i64 << 30;
        let x = (self.right * c + self.fwd * s) / den;
        let y = (self.right * s - self.fwd * c) / den;
        Pos::new(pos.x + x as i32, pos.y + y as i32)
    }
}

/// `(piece, animation slot, starttime)` → bay. **Piece 254, the Bomber**:
/// `CHAR_ATTACK2`'s ten `BomberBomb`s alternate `node 0` (frames 1, 5, 10,
/// 16, 21) and `node 1` (2, 8, 13, 18, 24), and neither moves with the
/// event's frame.
const BAYS: &[(i32, i8, u32, Bay)] = &[
    (
        254,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        2,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        5,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        8,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        10,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        13,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        16,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        18,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        21,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::anim::ATTACK2,
        24,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
];

/// The bay a piece releases from, or `None` when none is measured.
pub fn bay(gpiece: i32, anim: i8, starttime: u32) -> Option<Bay> {
    BAYS.iter()
        .find(|(p, a, t, _)| *p == gpiece && *a == anim && *t == starttime)
        .map(|(_, _, _, b)| *b)
}

/// The release height over the figure, from whichever table measures it.
pub fn release_dz(gpiece: i32, anim: i8, starttime: u32) -> Option<i32> {
    bay(gpiece, anim, starttime)
        .map(|b| b.dz)
        .or_else(|| node(gpiece, anim, starttime).map(|n| n.dz))
}

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

/// [`Bay::point`] or [`point`] for a piece that has a row, and `pos` for
/// one that does not.
pub fn launch_point(pos: Pos, facing: Angle, gpiece: i32, anim: i8, starttime: u32) -> Pos {
    if let Some(b) = bay(gpiece, anim, starttime) {
        return b.point(pos, facing);
    }
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
    /// The pinned table is the cosine, to the nearest 2^-30, against the
    /// host's doubles — the test oracle `no_float.rs` allows.
    #[test]
    fn the_whole_degree_table_is_the_cosine() {
        for (d, &c) in COS_DEG.iter().enumerate() {
            let want = ((d as f64).to_radians().cos() * f64::from(1u32 << 30)).round() as i64;
            assert_eq!(c, want, "cos {d}");
        }
        assert_eq!(cos_sin(90), (0, 1 << 30));
        assert_eq!(cos_sin(180), (-(1 << 30), 0));
        assert_eq!(cos_sin(270), (0, -(1 << 30)));
    }

    /// Every one of run235's 49 Bomber bombs: `(node's frame, figure x, y,
    /// angle, sx, sy)`, the dump's own columns. **The degree is the
    /// angle's top byte to the nearest degree**: 1003487232 is 84.1° but
    /// 59 in the top byte, 83°.
    #[test]
    fn run235_s_bomb_bays_are_reproduced_to_the_unit() {
        const M: &[(u32, i32, i32, i32, i32, i32)] = &[
            (1, 20573, 16573, 1003487232, 20631, 16495),
            (2, 20632, 16567, 1002831872, 20624, 16636),
            (5, 20809, 16549, 999751680, 20867, 16471),
            (8, 20986, 16530, 990904320, 20978, 16599),
            (10, 21104, 16515, 981793360, 21160, 16436),
            (13, 21281, 16489, 968019104, 21277, 16558),
            (16, 21458, 16462, 968019104, 21512, 16381),
            (18, 21576, 16444, 968019104, 21572, 16513),
            (21, 21753, 16417, 968019104, 21807, 16336),
            (24, 21930, 16390, 968019104, 21926, 16459),
            (1, 20552, 16527, 1057947648, 20618, 16455),
            (2, 20611, 16526, 1057423360, 20596, 16593),
            (5, 20788, 16523, 1054736384, 20851, 16449),
            (8, 20965, 16518, 1050542080, 20953, 16586),
            (10, 21083, 16514, 1047855104, 21146, 16440),
            (13, 21260, 16508, 1039990784, 21249, 16576),
            (16, 21437, 16502, 1039990784, 21499, 16427),
            (18, 21555, 16498, 1039990784, 21544, 16566),
            (21, 21732, 16492, 1039990784, 21794, 16417),
            (24, 21909, 16486, 1039990784, 21898, 16554),
            (1, 21022, 16037, 2014183424, 21104, 16089),
            (2, 21033, 16096, 2012610560, 20964, 16096),
            (5, 21068, 16273, 2006777856, 21152, 16322),
            (8, 21105, 16450, 1995177984, 21036, 16450),
            (10, 21132, 16568, 1981807184, 21217, 16616),
            (13, 21174, 16745, 1981807184, 21105, 16745),
            (16, 21216, 16922, 1981807184, 21301, 16970),
            (18, 21244, 17040, 1981807184, 21175, 17040),
            (21, 21286, 17217, 1981807184, 21371, 17265),
            (24, 21328, 17394, 1981807184, 21259, 17394),
            (1, 21059, 15985, 2073605984, 21137, 16043),
            (2, 21065, 16045, 2073755648, 20996, 16037),
            (5, 21083, 16225, 2068381696, 21161, 16283),
            (8, 21104, 16405, 2059206656, 21035, 16399),
            (10, 21120, 16525, 2045377104, 21201, 16579),
            (13, 21144, 16705, 2045377104, 21075, 16701),
            (16, 21168, 16885, 2045377104, 21249, 16939),
            (18, 21184, 17005, 2045377104, 21115, 17001),
            (21, 21208, 17185, 2045377104, 21289, 17239),
            (24, 21232, 17365, 2045377104, 21163, 17361),
            (1, 21681, 16384, -1228677792, 21633, 16469),
            (2, 21622, 16398, -1223819264, 21622, 16329),
            (5, 21445, 16439, -1222705152, 21396, 16523),
            (8, 21268, 16479, -1223032832, 21268, 16410),
            (10, 21150, 16505, -1221328896, 21101, 16589),
            (13, 20973, 16547, -1227423744, 20973, 16478),
            (16, 20796, 16589, -1227423744, 20748, 16674),
            (18, 20678, 16617, -1227423744, 20678, 16548),
            (21, 20501, 16659, -1227423744, 20453, 16744),
        ];
        for &(t, x, y, a, sx, sy) in M {
            let p = launch_point(Pos::new(x, y), Angle(a), 254, crate::anim::ATTACK2, t);
            assert_eq!((p.x, p.y), (sx, sy), "frame {t} at {a}");
        }
        assert_eq!(M.len(), 49);
        assert_eq!(whole_degrees(Angle(1_003_487_232)), 83);
        assert_eq!(release_dz(254, crate::anim::ATTACK2, 21), Some(-19));
    }

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
