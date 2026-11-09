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
    // **122 is `CROSSBOWMEN`** (unit type 172; item 3347, run982 on the
    // French Great Sahara seed, `AMMO=5` over `[13416, 14093]`): the three
    // first rounds of who=1's archers on 13822–13839, each the guy's pose on
    // the block before its first print against the round's `sx, sy, sz` —
    // `1/41`'s `CHAR_ATTACK2` frame 3, guy (10008, 30984) at −1120927744,
    // (9943, 30974, 707) over a guy `z` of 573; `1/42`'s `CHAR_ATTACK1`
    // frame 5, (9624, 31176) at −1057882112, (9540, 31165, 769) over 603;
    // and `1/43`'s `CHAR_ATTACK3` frame 6, (9672, 30984) at −1130299392,
    // (9596, 30977, 758) over 597. One shot each: the radius is the only
    // one that fits and the bearing the middle of its exact fits. The
    // Cavalry Archer's three rows, piece 138, left for `pivot::RELEASES`
    // (item 3633): its `<RESTRICTION>` makes it a pivot piece, and its
    // rounds turn with the rider (`docs/AI.md` §330).
    (
        122,
        crate::anim::ATTACK1,
        5,
        Node {
            bearing: 77_676_544,
            radius: 84,
            dz: 166,
        },
    ),
    (
        122,
        crate::anim::ATTACK2,
        3,
        Node {
            bearing: 157_679_616,
            radius: 65,
            dz: 134,
        },
    ),
    (
        122,
        crate::anim::ATTACK3,
        6,
        Node {
            bearing: 123_879_424,
            radius: 76,
            dz: 161,
        },
    ),
    // Piece 842, the Horse Archer (item 3338), left for `pivot::SAME_AS`
    // (item 3658): its `AttachPos` rows are piece 138's, a pivot piece's, so
    // its rounds turn with the rider's turret.
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

/// The whole degree [`Bay::point`] turns by: `fast_angle_to_degrees` of
/// the figure's angle less a half turn, as `execute_game_events` hands it
/// to `get_position` ([`crate::pivot::release_rotation`]), turned back by
/// the half turn — `Bay`'s axes are the figure's, not the flipped ones.
///
/// **A half-way byte rounds up** (item 2304): `byte ≡ 16 mod 32`, 22.5°
/// and its kin, is read by `angle_to_degrees`, which rounds a half degree
/// early (`docs/COMBAT.md` §52). This was taken to the even degree while no
/// measured heading sat on one; run835's Bark `0/7` faces byte 80 (112.5°)
/// on blocks 638–653, and only 113 puts its rounds at the dump's points.
pub fn whole_degrees(facing: Angle) -> i64 {
    i64::from((crate::pivot::release_rotation(facing) + 180) % 360)
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
        let (x, y) = self.offset(whole_degrees(facing));
        Pos::new(pos.x + x as i32, pos.y + y as i32)
    }

    /// The vector turned by the whole degree `d`, each axis truncated.
    fn offset(self, d: i64) -> (i64, i64) {
        let (c, s) = cos_sin(d);
        let den = 1000i64 << 30;
        (
            (self.right * c + self.fwd * s) / den,
            (self.right * s - self.fwd * c) / den,
        )
    }
}

/// `(piece, animation slot, starttime)` → bay. **Piece 254, the Bomber**:
/// `CHAR_ATTACK2`'s ten `BomberBomb`s alternate `node 0` (frames 1, 5, 10,
/// 16, 21) and `node 1` (2, 8, 13, 18, 24), and neither moves with the
/// event's frame.
const BAYS: &[(i32, i8, u32, Bay)] = &[
    // **Piece 125, the Royal Kushite Archers** (unit type 175; item 3849,
    // `docs/GOLDEN.md` §153.1): run1484's packet holds the piece's three
    // attack `AttachPos` rows, node 0 at frame 15 of `CHAR_ATTACK1`,
    // `CHAR_ATTACK2` and `CHAR_ATTACK3`, `RData +0x88` 1.0, and `fit_bay.py`
    // fits each sweep (386, 495 and 3512 lattice points, 361 of 361
    // headings; the centroid is kept). run1485's arrows of 801 leave
    // (12, 66) and 101 up (`0/15`, `0/17`, `CHAR_ATTACK2`) and (−71, 85)
    // and 116 up (`0/16`, `CHAR_ATTACK1`) — where ours left the archers'
    // squares, and `0/17`'s flew 21 frames against 20 and punctured the
    // ground on 821 against the original's 820.
    (
        125,
        crate::anim::ATTACK1,
        15,
        Bay {
            right: 102_456,
            fwd: 44_072,
            dz: 116,
        },
    ),
    (
        125,
        crate::anim::ATTACK2,
        15,
        Bay {
            right: 23_036,
            fwd: 63_394,
            dz: 101,
        },
    ),
    (
        125,
        crate::anim::ATTACK3,
        15,
        Bay {
            right: 38_786,
            fwd: 60_881,
            dz: 153,
        },
    ),
    // **Piece 282** (unit type 332; item 3849, `docs/GOLDEN.md` §153.1):
    // run1484's packet holds the piece's two `AttachPos` rows, `RData
    // +0x88` 0.65 — node 0 at frame 7 of `CHAR_ATTACK1` and of
    // `CHAR_ATTACK2` — and `fit_bay.py` fits each sweep (253 and 266
    // lattice points, 361 of 361 headings; the centroid is kept). Without
    // them run1484's `1/20` and `2/20` rounds of 660 left from the hull's
    // square at height 0, where the original's left at 51 and 34.
    (
        282,
        crate::anim::ATTACK1,
        7,
        Bay {
            right: -99_683,
            fwd: 18_131,
            dz: 51,
        },
    ),
    (
        282,
        crate::anim::ATTACK2,
        7,
        Bay {
            right: -101_773,
            fwd: 196_991,
            dz: 34,
        },
    ),
    // **Piece 91, the TOW** (`TOW`, unit type 141; item 3760, `docs/AI.md`
    // §372.4): run1437's packet on the American Information Great Lakes
    // seed loads its three attack `AttachPos` rows (node 0, frame 17 of
    // `CHAR_ATTACK1`, 7 of `CHAR_ATTACK2`, 13 of `CHAR_ATTACK3`), `RData
    // +0x88` 1.0, and `fit_bay.py` fits each sweep (720, 191 and 361 lattice
    // points, 361 of 361 headings; the centroid is kept). Without them
    // who=1's `1/17` loosed its missile of 4291 from its own square (5832,
    // 31320, 384) against the original's (5742, 31270, 531), and `1/18`'s of
    // 4302 and `1/26`'s of 4303 the same.
    (
        91,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: 32_149,
            fwd: 121_967,
            dz: 124,
        },
    ),
    (
        91,
        crate::anim::ATTACK2,
        7,
        Bay {
            right: 34_159,
            fwd: 98_259,
            dz: 147,
        },
    ),
    (
        91,
        crate::anim::ATTACK3,
        13,
        Bay {
            right: 47_814,
            fwd: 110_240,
            dz: 142,
        },
    ),
    // **Piece 94, the Recoilless Gun** (`RECOILGUN`, unit type 144; item
    // 3834, `docs/GOLDEN.md` §152.13): run1472's packet holds the piece's
    // three attack `AttachPos` rows (node 0, frame 12 of `CHAR_ATTACK1`, 22
    // of `CHAR_ATTACK2`, 15 of `CHAR_ATTACK3`), `RData +0x88` 1.0, and
    // `fit_bay.py` fits each `get_position.py … sweep 94 0 <anim> <time>`
    // (176, 278 and 75 lattice points, 361 of 361 headings, one `z` a row;
    // the centroid is kept): the muzzle some 80 to 92 out and 73 to 77 over
    // the figure. Without them who=0's `0/19` loosed its round of 892 on
    // the Main Battle Tank `1/21` from its square (18168, 15672, 506) with
    // `total_time` 5, where run1479 prints (18182, 15594, 583) with 4, and
    // the round, rolling on past its dead target, came down a frame later.
    (
        94,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 31_393,
            fwd: 86_406,
            dz: 74,
        },
    ),
    (
        94,
        crate::anim::ATTACK2,
        22,
        Bay {
            right: 25_178,
            fwd: 75_595,
            dz: 77,
        },
    ),
    (
        94,
        crate::anim::ATTACK3,
        15,
        Bay {
            right: 31_038,
            fwd: 86_590,
            dz: 73,
        },
    ),
    // **Piece 80, the MG42** (`MG42`, unit type 130; item 3827,
    // `docs/GOLDEN.md` §152.11): run1472's `RON_STATE_FRAME=707` packet on
    // the third melee's lobby holds the piece's 28 attack `AttachPos` rows
    // (node 0; eleven on `CHAR_ATTACK1`, seven on `CHAR_ATTACK2`, ten on
    // `CHAR_ATTACK3`, whose frame 12 is entered twice with the same bits),
    // `RData +0x88` 1.0, and `fit_bay.py` fits each `get_position.py …
    // sweep 80 0 <anim> <time>` (40 to 3,260 lattice points, 361 of 361
    // headings, one `z` a row; the centroid is kept): the muzzle some 145
    // ahead of the figure and 84 to 103 over it. Without them who=0's
    // `0/18` loosed its round of 819 on the Anti-Tank Missile `1/22` from
    // its own square (17400, 15672, 477), and the round, rolling on past
    // its dead target, stayed over the ground a frame longer than theirs.
    (
        80,
        crate::anim::ATTACK1,
        3,
        Bay {
            right: -7_425,
            fwd: 147_384,
            dz: 93,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 88,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: -16_551,
            fwd: 144_695,
            dz: 91,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: -9_573,
            fwd: 146_738,
            dz: 90,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        20,
        Bay {
            right: 752,
            fwd: 146_479,
            dz: 84,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        24,
        Bay {
            right: 13_866,
            fwd: 144_797,
            dz: 89,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        29,
        Bay {
            right: 18_457,
            fwd: 146_563,
            dz: 86,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        37,
        Bay {
            right: 4_674,
            fwd: 148_076,
            dz: 90,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_935,
            dz: 91,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_376,
            dz: 88,
        },
    ),
    (
        80,
        crate::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 88,
        },
    ),
    (
        80,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: -402,
            fwd: 146_341,
            dz: 88,
        },
    ),
    (
        80,
        crate::anim::ATTACK2,
        3,
        Bay {
            right: -2_732,
            fwd: 145_714,
            dz: 93,
        },
    ),
    (
        80,
        crate::anim::ATTACK2,
        5,
        Bay {
            right: -5_603,
            fwd: 144_577,
            dz: 98,
        },
    ),
    (
        80,
        crate::anim::ATTACK2,
        8,
        Bay {
            right: -7_510,
            fwd: 145_038,
            dz: 103,
        },
    ),
    (
        80,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: -5_569,
            fwd: 143_666,
            dz: 103,
        },
    ),
    (
        80,
        crate::anim::ATTACK2,
        10,
        Bay {
            right: -3_051,
            fwd: 145_130,
            dz: 103,
        },
    ),
    (
        80,
        crate::anim::ATTACK2,
        12,
        Bay {
            right: 2_021,
            fwd: 145_012,
            dz: 103,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        3,
        Bay {
            right: 1_011,
            fwd: 144_798,
            dz: 91,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        4,
        Bay {
            right: 1_830,
            fwd: 143_956,
            dz: 92,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        12,
        Bay {
            right: 11_993,
            fwd: 147_443,
            dz: 85,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        19,
        Bay {
            right: 18_422,
            fwd: 144_413,
            dz: 88,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        20,
        Bay {
            right: 18_428,
            fwd: 146_106,
            dz: 85,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        29,
        Bay {
            right: 1_830,
            fwd: 146_325,
            dz: 90,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 146_847,
            dz: 90,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_400,
            dz: 88,
        },
    ),
    (
        80,
        crate::anim::ATTACK3,
        46,
        Bay {
            right: 0,
            fwd: 146_272,
            dz: 88,
        },
    ),
    // **Piece 77, the Advanced Machine Gun** (`ADVANCEDMACHINEGUN`, unit
    // type 127; item 3813, `docs/GOLDEN.md` §152.7): run1472's
    // `RON_STATE_FRAME=707` packet on the third melee's lobby holds the
    // piece's 32 attack `AttachPos` rows (node 0; twelve on `CHAR_ATTACK1`,
    // eight on `CHAR_ATTACK2`, twelve on `CHAR_ATTACK3`, whose frame 12 is
    // entered twice with the same bits), `RData +0x88` 1.0, and `fit_bay.py`
    // fits each `get_position.py … sweep 77 0 <anim> <time>` (2 to 3,260
    // lattice points, 361 of 361 headings, one `z` a row; the centroid is
    // kept): the muzzle some 145 ahead of the figure and 90 to 110 over it.
    // Without them who=2's `2/25` loosed its pair of 712 and 713 on the
    // Elite Special Forces `0/22` from its own square (20520, 18360, 456)
    // against run1473's (20432, 18244, 554) and (20434, 18245, 555) — frames
    // 3 and 4 of `CHAR_ATTACK3` — flew 7 frames against 6, and `0/22` was
    // struck and fled a tick late.
    (
        77,
        crate::anim::ATTACK1,
        3,
        Bay {
            right: -7_394,
            fwd: 147_049,
            dz: 100,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 95,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: -16_534,
            fwd: 144_463,
            dz: 98,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: -9_569,
            fwd: 146_592,
            dz: 97,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        20,
        Bay {
            right: 737,
            fwd: 146_742,
            dz: 91,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        24,
        Bay {
            right: 13_870,
            fwd: 144_718,
            dz: 96,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        29,
        Bay {
            right: 18_498,
            fwd: 146_683,
            dz: 93,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        32,
        Bay {
            right: 18_363,
            fwd: 144_626,
            dz: 90,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        37,
        Bay {
            right: 4_655,
            fwd: 147_954,
            dz: 96,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_756,
            dz: 97,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_320,
            dz: 95,
        },
    ),
    (
        77,
        crate::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_206,
            dz: 95,
        },
    ),
    (
        77,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: -404,
            fwd: 146_277,
            dz: 95,
        },
    ),
    (
        77,
        crate::anim::ATTACK2,
        2,
        Bay {
            right: -1_409,
            fwd: 147_815,
            dz: 97,
        },
    ),
    (
        77,
        crate::anim::ATTACK2,
        3,
        Bay {
            right: -2_741,
            fwd: 145_367,
            dz: 100,
        },
    ),
    (
        77,
        crate::anim::ATTACK2,
        5,
        Bay {
            right: -5_593,
            fwd: 143_864,
            dz: 105,
        },
    ),
    (
        77,
        crate::anim::ATTACK2,
        8,
        Bay {
            right: -7_499,
            fwd: 144_016,
            dz: 109,
        },
    ),
    (
        77,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: -5_575,
            fwd: 142_638,
            dz: 109,
        },
    ),
    (
        77,
        crate::anim::ATTACK2,
        10,
        Bay {
            right: -3_046,
            fwd: 144_117,
            dz: 110,
        },
    ),
    (
        77,
        crate::anim::ATTACK2,
        12,
        Bay {
            right: 1_993,
            fwd: 143_949,
            dz: 110,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        3,
        Bay {
            right: 2_084,
            fwd: 145_768,
            dz: 98,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        4,
        Bay {
            right: 2_759,
            fwd: 144_837,
            dz: 99,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        12,
        Bay {
            right: 13_626,
            fwd: 147_563,
            dz: 92,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        19,
        Bay {
            right: 18_407,
            fwd: 144_206,
            dz: 95,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        20,
        Bay {
            right: 18_469,
            fwd: 146_452,
            dz: 91,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        27,
        Bay {
            right: 7_510,
            fwd: 145_934,
            dz: 96,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        29,
        Bay {
            right: 1_202,
            fwd: 145_993,
            dz: 97,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 147_095,
            dz: 96,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_144,
            dz: 95,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        42,
        Bay {
            right: 0,
            fwd: 145_525,
            dz: 95,
        },
    ),
    (
        77,
        crate::anim::ATTACK3,
        46,
        Bay {
            right: 0,
            fwd: 146_477,
            dz: 95,
        },
    ),
    // **Piece 126, `APEDEMAKARCHERS`** (unit type 176, the Apedemak
    // Archers; item 3759, `docs/GOLDEN.md` §149.2): run1438's
    // `RON_STATE_FRAME=684` packet on the first melee's lobby holds its
    // three attack `AttachPos` rows (node 0, anims 11, 12 and 13, all at
    // frame 15), `RData +0x88` 1.0, and `fit_bay.py` fits each sweep (386,
    // 495 and 3512 lattice points; the centroid is kept, 361 of 361
    // headings each). Without them the human's `0/14` loosed its round of
    // 674 at the British Bombard `1/23` from its own square (16632, 15672,
    // 539) against the original's (16634, 15605, 640), flew 11 frames
    // against 10, and `1/23` was struck on 684 a frame late.
    (
        126,
        11,
        15,
        Bay {
            right: 102_456,
            fwd: 44_072,
            dz: 116,
        },
    ),
    (
        126,
        12,
        15,
        Bay {
            right: 23_036,
            fwd: 63_394,
            dz: 101,
        },
    ),
    (
        126,
        13,
        15,
        Bay {
            right: 38_786,
            fwd: 60_881,
            dz: 153,
        },
    ),
    // **Piece 124, `KUSHITEARCHERS`** (unit type 174, the Nubians' Kushite
    // Archers; item 3785, `docs/GOLDEN.md` §150.1): run1457's
    // `RON_STATE_FRAME=700` packet on the second melee's lobby holds its
    // three attack `AttachPos` rows (node 0, anims 11, 12 and 13, all at
    // frame 15), `RData +0x88` 1.0, and `fit_bay.py` fits each sweep to
    // piece 126's three rows exactly (386, 495 and 3512 lattice points, the
    // same centroids): the two archers share the model. Without them the
    // human's `0/14` loosed its round of 673 at the Greek Phalanx `1/19`
    // from its own square (15096, 15672, 574) against the original's
    // (15167, 15587, 690), flew 15 frames against 14, and `1/19` was struck
    // on 688 a frame late.
    (
        124,
        11,
        15,
        Bay {
            right: 102_456,
            fwd: 44_072,
            dz: 116,
        },
    ),
    (
        124,
        12,
        15,
        Bay {
            right: 23_036,
            fwd: 63_394,
            dz: 101,
        },
    ),
    (
        124,
        13,
        15,
        Bay {
            right: 38_786,
            fwd: 60_881,
            dz: 153,
        },
    ),
    // **Piece 752, `ARQUEBUSIERS`** (`TY` 48, the Arquebusiers, `TI`
    // 98; item 3781, `docs/GOLDEN.md` §149.8): run1451's
    // `RON_STATE_FRAME=945` packet on the first melee's lobby holds its
    // three attack `AttachPos` rows (node 0; frame 6 of `CHAR_ATTACK1`, 25
    // of `CHAR_ATTACK2`, 23 of `CHAR_ATTACK3`), `RData +0x88` 1.0, and
    // `fit_bay.py` fits each sweep (492, 207 and 63 lattice points; the
    // centroid is kept, 361 of 361 headings each). Without them who=2's
    // `2/16` loosed its round of 953 at the British Elite Knight `1/22`
    // from its own square (15576, 16776, 628) against the original's
    // (15694, 16697, 779), flew 3 frames against 2, and `1/22` died on 955
    // against 954.
    (
        752,
        crate::anim::ATTACK1,
        6,
        Bay {
            right: 34_662,
            fwd: 150_565,
            dz: 181,
        },
    ),
    (
        752,
        crate::anim::ATTACK2,
        25,
        Bay {
            right: 30_200,
            fwd: 139_796,
            dz: 151,
        },
    ),
    (
        752,
        crate::anim::ATTACK3,
        23,
        Bay {
            right: 14_565,
            fwd: 132_355,
            dz: 164,
        },
    ),
    // **Piece 50, `RIFLEMAN`** (unit type 100, the Riflemen; item 3705,
    // `docs/AI.md` §357.2): run1382's `RON_STATE_FRAME=3228` packet on the
    // Spanish Industrial Great Lakes seed holds its three attack
    // `AttachPos` rows (node 0, frames 10, 15 and 16), `RData +0x88` 1.0,
    // and `fit_bay.py` fits each sweep (629, 15 and 412 lattice points; the
    // centroid is kept, 361 of 361 headings each): the same three keys and
    // regions as piece 65's, the Black Watch's, below. No other packet on
    // disk loads the piece. Without them who=1's `1/53` loosed its round of 3229
    // at the human's building `0/2003` from its own square (4392, 33912,
    // 466) against the original's (4328, 33795, 588), flew 3 frames
    // against 2, and the building was struck a frame late.
    (
        50,
        11,
        10,
        Bay {
            right: 15_269,
            fwd: 121_470,
            dz: 168,
        },
    ),
    (
        50,
        12,
        15,
        Bay {
            right: 5_281,
            fwd: 133_941,
            dz: 122,
        },
    ),
    (
        50,
        13,
        16,
        Bay {
            right: 13_151,
            fwd: 120_822,
            dz: 167,
        },
    ),
    // **The Caravel's piece 275** (unit type 325; item 3633, `docs/AI.md`
    // §330): run1293's `RON_STATE_FRAME=3606` packet on the Roman Medieval
    // East Indies seed holds its nine attack `AttachPos` rows, node 0 at
    // anims 10, 11 and 12, frames 5, 8 and 13 (the three
    // `ArcherArrowWOtip` releases at 366, 570 and 900 ms). The three anims
    // share their bits, and frames 5 and 8 share theirs; each sweep has one
    // feasible region (101 and 1394 lattice points) reproducing all 361
    // headings, x and y, and the rows are its middles. The gun is the
    // broadside, 150 units off the hull's square: who=1's `1/130` loosed
    // at the Citizen `0/10` on 5544 from (3476, 2151) where this crate
    // loosed from the hull's (3576, 2040), 7 frames against 8, and `0/10`
    // was struck on 5550 a frame late.
    (
        275,
        10,
        5,
        Bay {
            right: 66031,
            fwd: -135_008,
            dz: 28,
        },
    ),
    (
        275,
        10,
        8,
        Bay {
            right: 66031,
            fwd: -135_008,
            dz: 28,
        },
    ),
    (
        275,
        10,
        13,
        Bay {
            right: 65135,
            fwd: -135_024,
            dz: 30,
        },
    ),
    (
        275,
        11,
        5,
        Bay {
            right: 66031,
            fwd: -135_008,
            dz: 28,
        },
    ),
    (
        275,
        11,
        8,
        Bay {
            right: 66031,
            fwd: -135_008,
            dz: 28,
        },
    ),
    (
        275,
        11,
        13,
        Bay {
            right: 65135,
            fwd: -135_024,
            dz: 30,
        },
    ),
    (
        275,
        12,
        5,
        Bay {
            right: 66031,
            fwd: -135_008,
            dz: 28,
        },
    ),
    (
        275,
        12,
        8,
        Bay {
            right: 66031,
            fwd: -135_008,
            dz: 28,
        },
    ),
    (
        275,
        12,
        13,
        Bay {
            right: 65135,
            fwd: -135_024,
            dz: 30,
        },
    ),
    // **The Frigate's piece 293** (unit type 343, `FRIGATE-DEFAULT-AGE0`;
    // item 3643, `docs/AI.md` §335.3): run1304's `RON_STATE_FRAME=3865`
    // packet on the Spanish Gunpowder East Indies seed holds its two attack
    // `AttachPos` rows, node 0 at frame 7 (the `InvisibleAmmo` release at
    // 500 ms) of `CHAR_ATTACK1` (`Frigate Left Attack`) and `CHAR_ATTACK2`
    // (`Right Attack`). Each `get_position.py` sweep has one feasible region
    // (1097 and 151 lattice points) reproducing all 361 headings, x and y;
    // the rows are its middles and `dz` the sweep's `z`. who=1's `1/29`
    // fired at the Farm `0/2004` on 3866 from (1809, 5233), `sz` 20, where
    // this crate fired from the hull's (1896, 5352): 11 frames against 12,
    // and the Farm struck on 3876, a frame early.
    (
        293,
        crate::anim::ATTACK1,
        7,
        Bay {
            right: -117_433,
            fwd: -11_024,
            dz: 47,
        },
    ),
    (
        293,
        crate::anim::ATTACK2,
        7,
        Bay {
            right: -103_432,
            fwd: 106_239,
            dz: 20,
        },
    ),
    // **The Arquebusiers' piece 1104** (unit type 98 as the Japanese field
    // it; item 3590, `docs/AI.md` §316): run1237's `RON_STATE_FRAME=2674`
    // packet on the Japanese Gunpowder Great Sahara seed holds its three
    // attack `AttachPos` rows, node 0 at anims 11, 12 and 13, times 6, 25
    // and 23, and `tools/recomp/get_position.py`'s sweep of each has one
    // feasible region (240, 757 and 194 lattice points) reproducing all 361
    // headings, x and y; the rows are its middles. who=1's `1/19` fired at
    // the Citizen `0/12` on 2693 from (9940, 30945) in run1208d where this
    // crate fired from the figure's own (10056, 30984): 6 frames against 7,
    // and the Citizen struck a frame late.
    (
        1104,
        11,
        6,
        Bay {
            right: 16431,
            fwd: 145_696,
            dz: 175,
        },
    ),
    (
        1104,
        12,
        25,
        Bay {
            right: 20291,
            fwd: 131_870,
            dz: 170,
        },
    ),
    (
        1104,
        13,
        23,
        Bay {
            right: 3610,
            fwd: 123_230,
            dz: 175,
        },
    ),
    // **Piece 48, the Egyptians' Arquebusiers** (unit type 98 as the
    // Egyptians field it; item 3603, `docs/AI.md` §324): not piece 1104's
    // art — run1254c's `RON_STATE_FRAME=3115` packet on the Egyptian
    // Gunpowder Great Sahara seed holds three attack `AttachPos` rows on
    // 1104's keys (anims 11, 12 and 13 at 6, 25 and 23) with their own
    // points, and `tools/recomp/fit_bay.py` fits each 361-heading sweep to
    // one region (14, 1293 and 238 lattice points); the rows are its
    // centroids. Without them the Arquebusiers `1/32`'s round at the
    // Citizen `0/9`, fired on 3114, left from the figure's own (7560,
    // 30840) and flew 6 frames where the original's left from (7459,
    // 30915, 792) and flew 5, and `0/9` died a frame late.
    (
        48,
        11,
        6,
        Bay {
            right: 26_383,
            fwd: 140_546,
            dz: 168,
        },
    ),
    (
        48,
        12,
        25,
        Bay {
            right: 25_298,
            fwd: 124_214,
            dz: 159,
        },
    ),
    (
        48,
        13,
        23,
        Bay {
            right: 13_387,
            fwd: 117_132,
            dz: 166,
        },
    ),
    // **Piece 1456, the Lakota's Arquebusiers** (unit type 98 as the
    // Lakota field it; item 3645, `docs/AI.md` §336): run1305b's
    // `RON_STATE_FRAME=4181` packet on the Lakota Gunpowder Great Sahara
    // seed holds three attack `AttachPos` rows on 1104's keys, and
    // `tools/recomp/fit_bay.py` fits each sweep to the very regions piece
    // 48's packet gave (14, 1293 and 238 lattice points): the same model,
    // so the same rows. Without them group 64's Arquebusiers left from the
    // figure's own point — run1289b's `1/39` on 4217 from (8520, 27864)
    // against the original's (8395, 27931, 659), and `1/37`'s round on 4236
    // flew 5 frames against 4.
    (
        1456,
        11,
        6,
        Bay {
            right: 26_383,
            fwd: 140_546,
            dz: 168,
        },
    ),
    (
        1456,
        12,
        25,
        Bay {
            right: 25_298,
            fwd: 124_214,
            dz: 159,
        },
    ),
    (
        1456,
        13,
        23,
        Bay {
            right: 13_387,
            fwd: 117_132,
            dz: 166,
        },
    ),
    // **Piece 49, `MUSKETEERS`** (unit type 99 as the French and the
    // Egyptians field it; item 3691, `docs/AI.md` §351): every round of
    // piece 49 the disk holds — run1344b's 92 on the French Enlightenment
    // East Indies seed (4495..4756) and run971's two (item 3336, which kept
    // them as two `Node` rows) — each against its figure's `GUY` pose on the
    // block before. The dump does not print the animation, and the rounds
    // fall into three heights, 218, 178 and 176 over the figure: run971's
    // `1/54` (`CHAR_ATTACK1`, frame 1) and `1/136` (`CHAR_ATTACK3`, frame
    // 4) name the first two, and this crate's `1/37` on 4499 played
    // `CHAR_ATTACK2` at frame 7 for the third. Each height's rounds (23, 30
    // and 41, at 18, 24 and 29 facings) have one feasible region under the
    // whole-degree turn (729, 697 and 28 lattice points at a step of 5);
    // the rows are its centre-most point. The `Node` rows missed every one
    // by 1 to 3 units, and `CHAR_ATTACK2` had none: `1/37` left from the
    // figure's own (6072, 9480) on 4499 where the original's left from
    // (6152, 9334), and its round struck a frame late, on 4505.
    (
        49,
        crate::anim::ATTACK1,
        1,
        Bay {
            right: 33_072,
            fwd: 139_813,
            dz: 218,
        },
    ),
    (
        49,
        crate::anim::ATTACK2,
        7,
        Bay {
            right: 11_452,
            fwd: 166_595,
            dz: 176,
        },
    ),
    (
        49,
        crate::anim::ATTACK3,
        4,
        Bay {
            right: 12_256,
            fwd: 162_078,
            dz: 178,
        },
    ),
    // **Piece 1457, `MUSKETEERS-NA`** (unit type 99 as the Iroquois field
    // it, `musketeer_na.bh3`; item 3655, `docs/AI.md` §337): run1320b's
    // `RON_STATE_FRAME=5066` packet on the Iroquois Enlightenment East
    // Indies seed holds its three attack `AttachPos` rows (node 0, 0, 500
    // and 333 ms: frames 1, 7 and 4), `RData +0x88` 1.1, and
    // `tools/recomp/fit_bay.py` fits each sweep (348, 73 and 27 lattice
    // points; the centroid is kept). Without them group 78's Musketeers
    // left from the figure's own point — run1320's `1/75` on 5067 from
    // (7512, 8664, 298) against the original's (7614, 8539, 476).
    (
        1457,
        11,
        1,
        Bay {
            right: 32_553,
            fwd: 139_478,
            dz: 218,
        },
    ),
    (
        1457,
        12,
        7,
        Bay {
            right: 11_054,
            fwd: 166_181,
            dz: 176,
        },
    ),
    (
        1457,
        13,
        4,
        Bay {
            right: 11_950,
            fwd: 161_692,
            dz: 178,
        },
    ),
    // **Piece 753, `MUSKETEERS-AMERICAN`** (unit type 99 as the Inca
    // field it, `Musketeer_amer.bh3`; item 3659, `docs/AI.md` §339):
    // run1323b's `RON_STATE_FRAME=3052` packet on the Inca Enlightenment
    // Great Sahara seed holds its three attack `AttachPos` rows (node 0,
    // frames 1, 7 and 4), `RData +0x88` 1.0, and `tools/recomp/fit_bay.py`
    // fits each sweep (203, 1428 and 762 lattice points; the centroid is
    // kept, 361 of 361 headings each). Without them group 64's Musketeers
    // left from the figure's own point — run1323's `1/25` on 3053 from
    // (7752, 30840, 640) against the original's (7795, 30717, 838).
    (
        753,
        11,
        1,
        Bay {
            right: 30_043,
            fwd: 127_094,
            dz: 198,
        },
    ),
    (
        753,
        12,
        7,
        Bay {
            right: 10_401,
            fwd: 151_438,
            dz: 160,
        },
    ),
    (
        753,
        13,
        4,
        Bay {
            right: 11_212,
            fwd: 147_366,
            dz: 162,
        },
    ),
    // **Piece 790, `FUSILIER-AMERICAN`** (unit type 136 as the Inca field
    // it; item 3659, `docs/AI.md` §339): run1323b's packet holds its three
    // attack `AttachPos` rows (node 0, frames 6, 6 and 8), `RData +0x88`
    // 1.0, and `fit_bay.py` fits each sweep (48, 207 and 1679 lattice
    // points; the centroid is kept, 361 of 361 headings each). Without
    // them the Fusiliers left from the figure's own point — run1323c's
    // `1/44` on 3815 from (8136, 31080, 662) against the original's
    // (8048, 31009, 772), at the human's capital `0/2000`.
    (
        790,
        11,
        6,
        Bay {
            right: 21_445,
            fwd: 111_997,
            dz: 110,
        },
    ),
    (
        790,
        12,
        6,
        Bay {
            right: 4_874,
            fwd: 96_147,
            dz: 116,
        },
    ),
    (
        790,
        13,
        8,
        Bay {
            right: 16_206,
            fwd: 100_152,
            dz: 115,
        },
    ),
    // **Piece 1105, `MUSKETEERS-ASIAN`** (unit type 99, `musketeer_asian.bh3`,
    // the Mongols' Musketeer; item 3692, `docs/AI.md` §352.1): run1361's
    // `RON_STATE_FRAME=4840` packet on the Mongol Enlightenment Great Lakes
    // seed holds its three attack `AttachPos` rows (node 0, frames 1, 7 and
    // 4), `RData +0x88` 1.1, and `fit_bay.py` fits each sweep (411, 196 and
    // 113 lattice points; the centroid is kept, 361 of 361 headings each).
    // Its own model, so not piece 49's rows, which miss its sweep at 4 and
    // 32 headings of `CHAR_ATTACK1` and `CHAR_ATTACK3`. Without them who=1's
    // `1/21` loosed its `CHAR_ATTACK1` round of 4854 from its own square
    // (6552, 28728, 423) against the original's (6415, 28769, 641), flew 3
    // frames against 2, and the Citizen `0/6` outlived the word.
    (
        1105,
        11,
        1,
        Bay {
            right: 33_049,
            fwd: 139_808,
            dz: 218,
        },
    ),
    (
        1105,
        12,
        7,
        Bay {
            right: 11_451,
            fwd: 166_599,
            dz: 176,
        },
    ),
    (
        1105,
        13,
        4,
        Bay {
            right: 12_330,
            fwd: 162_102,
            dz: 178,
        },
    ),
    // **Piece 66, `MANCHUMUSKETEERS`** (unit type 116, the Chinese Manchu
    // Musketeers; item 3704, `docs/AI.md` §354.2): run1381's
    // `RON_STATE_FRAME=3262` packet on the Chinese Enlightenment Great
    // Lakes seed holds its three attack `AttachPos` rows (node 0, frames 1,
    // 7 and 4), `RData +0x88` 1.03, and `fit_bay.py` fits each sweep (285,
    // 14 and 132 lattice points; the centroid is kept, 361 of 361 headings
    // each). No other packet on disk fills it. Without them who=1's `1/42`
    // loosed its round of 3261 at the human's city centre `0/2000` from its
    // own square (5208, 31512, 464) against the original's (5094, 31444,
    // 678), flew 5 frames against 4, and the city was struck a frame late.
    (
        66,
        11,
        1,
        Bay {
            right: 29_267,
            fwd: 130_145,
            dz: 214,
        },
    ),
    (
        66,
        12,
        7,
        Bay {
            right: 9_324,
            fwd: 155_739,
            dz: 182,
        },
    ),
    (
        66,
        13,
        4,
        Bay {
            right: 9_222,
            fwd: 152_372,
            dz: 178,
        },
    ),
    // **Piece 1142, `FUSILIER-ASIAN`** (unit type 136, the Fusiliers as the
    // Chinese and the Mongols field them; item 3704, `docs/AI.md` §354.2):
    // run1381's packet holds its three attack `AttachPos` rows (node 0,
    // frames 6, 6 and 8), `RData +0x88` 1.0 — run1361's packet holds the
    // same floats — and `fit_bay.py` fits each sweep (475, 12 and 206
    // lattice points; the centroid is kept, 361 of 361 headings each).
    // Without them who=1's `1/52` loosed its round of 3195 from its own
    // square (6216, 29448, 492) against the original's (6117, 29386, 609).
    (
        1142,
        11,
        6,
        Bay {
            right: 52_732,
            fwd: 124_892,
            dz: 112,
        },
    ),
    (
        1142,
        12,
        6,
        Bay {
            right: 22_738,
            fwd: 114_918,
            dz: 117,
        },
    ),
    (
        1142,
        13,
        8,
        Bay {
            right: 47_713,
            fwd: 110_072,
            dz: 111,
        },
    ),
    // **Piece 69, `MARINES`** (unit type 119, the Americans' Continental
    // Marines; item 3661, `docs/AI.md` §341): run1325c's
    // `RON_STATE_FRAME=5578` packet on the American Enlightenment East
    // Indies seed holds its three attack `AttachPos` rows (node 0, frames 1,
    // 7 and 4), `RData +0x88` 1.1, and `fit_bay.py` fits each sweep (65,
    // 2136 and 595 lattice points; the centroid is kept, 361 of 361
    // headings each). Without them the Marines left from the figure's own
    // point — run1325b's `1/122` on 5578 from (7464, 9624, 190) against the
    // original's (7451, 9479, 418), and its round on the human's capital
    // `0/2000` landed a frame late: the word 5583.
    (
        69,
        11,
        1,
        Bay {
            right: 49_240,
            fwd: 137_168,
            dz: 228,
        },
    ),
    (
        69,
        12,
        7,
        Bay {
            right: 27_027,
            fwd: 168_687,
            dz: 188,
        },
    ),
    (
        69,
        13,
        4,
        Bay {
            right: 28_173,
            fwd: 164_233,
            dz: 191,
        },
    ),
    // **Piece 294, `MANOWAR-DEFAULT`** (unit type 344, `manowar.bh3`;
    // item 3660, `docs/AI.md` §340): run1185's packet (logger frame 4582
    // of the British Enlightenment lobby) holds its two attack
    // `AttachPos` rows — node 0 at frame 7 (500 ms) of `CHAR_ATTACK1` and
    // `CHAR_ATTACK2`, the port and starboard broadsides — `RData +0x88`
    // 0.7, and no `<RESTRICTION>`, so it is a bay rather than a pivot.
    // `tools/recomp/fit_bay.py` fits each sweep (141 and 558 lattice
    // points; the centroid is kept, 361 of 361 headings each). Without
    // them the Man o' War fired from the hull's own square at the water's
    // `z` 0: run1324's `1/97` loosed at the Citizen `0/13` on 4082 from
    // (3000, 8472, 0) and flew 11 frames, where the original's left
    // (2866, 8510, −33) and flew 12, so ours struck on 4091 and theirs on
    // 4092 (the Indian Enlightenment word 4157's flee, a frame early).
    (
        294,
        11,
        7,
        Bay {
            right: -137_698,
            fwd: -75_529,
            dz: 22,
        },
    ),
    (
        294,
        12,
        7,
        Bay {
            right: -135_399,
            fwd: 36_280,
            dz: -33,
        },
    ),
    // **Piece 1809, the Indians' Musketeer** (unit type 99 as the Indians
    // field it; item 3660, `docs/AI.md` §340): run1324c's
    // `RON_STATE_FRAME=4904` packet on the Indian Enlightenment East
    // Indies seed holds its three attack `AttachPos` rows (node 0, frames
    // 1, 7 and 4), `RData +0x88` 1.1, and `fit_bay.py` fits each sweep
    // (132, 247 and 624 lattice points; the centroid is kept, 361 of 361
    // headings each). Without them the Musketeers left from the figure's
    // own point — run1324b's `1/80` on 4905 from (11256, 7608, 315)
    // against the original's (11095, 7683, 489), flying 5 frames against
    // 4, at the Citizen `0/10`.
    (
        1809,
        11,
        1,
        Bay {
            right: 30_988,
            fwd: 151_411,
            dz: 219,
        },
    ),
    (
        1809,
        12,
        7,
        Bay {
            right: 8_796,
            fwd: 177_696,
            dz: 174,
        },
    ),
    (
        1809,
        13,
        4,
        Bay {
            right: 9_395,
            fwd: 173_220,
            dz: 176,
        },
    ),
    // **Piece 291, the Galley** (unit type 341; item 3662, `docs/AI.md`
    // §342): run1217's packet (logger frame 7728 of the Greek Classical
    // lobby) holds its six attack `AttachPos` rows — node 0 at frames 9,
    // 12 and 17 of `CHAR_ATTACK1` and `CHAR_ATTACK2` — `RData +0x88` 0.8,
    // and no `<RESTRICTION>`, so it is a bay. `tools/recomp/fit_bay.py`
    // fits each sweep (9, 298, 269, 9, 298 and 439 lattice points; the
    // centroid is kept, 361 of 361 headings each). Without them the Galley
    // fired from the hull's own square at the water's `z` 0: run1316c's
    // `1/29` loosed at the human's `0/2004` on 4298 from (2472, 4200, 0)
    // where the original's left (2690, 3917, 52) (the British Classical
    // word 4567's Galleys).
    (
        291,
        11,
        9,
        Bay {
            right: 2648,
            fwd: -357_619,
            dz: 52,
        },
    ),
    (
        291,
        11,
        12,
        Bay {
            right: 2541,
            fwd: -357_616,
            dz: 52,
        },
    ),
    (
        291,
        11,
        17,
        Bay {
            right: 963,
            fwd: -357_625,
            dz: 53,
        },
    ),
    (
        291,
        12,
        9,
        Bay {
            right: 2648,
            fwd: -357_619,
            dz: 52,
        },
    ),
    (
        291,
        12,
        12,
        Bay {
            right: 2541,
            fwd: -357_616,
            dz: 48,
        },
    ),
    (
        291,
        12,
        17,
        Bay {
            right: 1164,
            fwd: -357_625,
            dz: 57,
        },
    ),
    // **Piece 1625, the Lakota's Bombard** (unit type 267; item 3645,
    // `docs/AI.md` §336): run1305b's packet holds its one attack
    // `AttachPos` row, node 0 at `CHAR_ATTACK2` frame 9 (666 ms), and
    // `tools/recomp/fit_bay.py` fits the sweep to one region of 924 lattice
    // points; the row is its centroid, `dz` 104 at every heading. Without it the Bombards `1/58`
    // and `1/66` fired at the city `0/2000` from the hull's own point —
    // run1289b's `1/58` on 4303 from (9000, 27144) against the original's
    // (8898, 27250, 585).
    (
        1625,
        12,
        9,
        Bay {
            right: -710,
            fwd: 147_904,
            dz: 104,
        },
    ),
    // **Piece 218, the Cannon** (unit type 268; item 3661, `docs/AI.md`
    // §341): run1325c's packet holds its three attack `AttachPos` rows,
    // node 0 at frame 9 of `CHAR_ATTACK1`..`3`, one point, `RData +0x88`
    // 1.0, and `fit_bay.py` fits each sweep to the same region of 99
    // lattice points; the rows are its centroid, `dz` 100 at every
    // heading. Without them who=1's Cannons fired from the carriage's own
    // point — run1325d's `1/96` on 6475 from (9480, 9864, 318) against the
    // original's (9369, 9768, 418), in the closing window.
    (
        218,
        11,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    (
        218,
        12,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    (
        218,
        13,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    // **The Javelineers' piece 34** (item 2997, `docs/AI.md` §244): run912e's
    // `RON_STATE_FRAME=12753` packet holds its three `AttachPos` rows, and
    // `tools/recomp/get_position.py`'s sweep of each is reproduced by these
    // at all 361 headings, x, y and z. who=1's Elite Javelineers `1/42`
    // threw at the Scout `0/0` on 12754 from (6466, 8872) in run912d where
    // this crate threw from the figure's own (6504, 8904): 13 frames
    // against 14, and the lead at (5184, 8682) against (5159, 8707).
    (
        34,
        11,
        12,
        Bay {
            right: 39258,
            fwd: 59130,
            dz: 166,
        },
    ),
    (
        34,
        12,
        6,
        Bay {
            right: 1121,
            fwd: 50544,
            dz: 173,
        },
    ),
    (
        34,
        13,
        27,
        Bay {
            right: 21620,
            fwd: 35314,
            dz: 159,
        },
    ),
    // **The Atl-Atls' pieces 35 and 36** (`ATLATL` and `ELITEATLATL`, unit
    // types 85 and 86; item 3613, `docs/AI.md` §325.1): run1266c's
    // `RON_STATE_FRAME=8700` packet on the Aztec East Indies seed holds each
    // piece's three attack `AttachPos` rows, all at time 11, and
    // `get_position.py`'s sweep of each is reproduced at all 361 headings by
    // these, each the centroid of its exact region (piece 35: 252, 869 and
    // 473 lattice points; piece 36: 28, 35 and 135). Without them the Royal
    // Atl-Atls threw from the figure's own square: `1/163`'s javelin at the
    // Citizen `0/4` flew 15 frames where run1266 shows 14 from (6327, 9920),
    // and the wound landed on 8694 against 8693.
    (
        35,
        11,
        11,
        Bay {
            right: 27_451,
            fwd: 148_341,
            dz: 166,
        },
    ),
    (
        35,
        12,
        11,
        Bay {
            right: 40_884,
            fwd: 135_492,
            dz: 188,
        },
    ),
    (
        35,
        13,
        11,
        Bay {
            right: 27_982,
            fwd: 110_381,
            dz: 178,
        },
    ),
    (
        36,
        11,
        11,
        Bay {
            right: 34_560,
            fwd: 123_009,
            dz: 168,
        },
    ),
    (
        36,
        12,
        11,
        Bay {
            right: 49_097,
            fwd: 105_964,
            dz: 186,
        },
    ),
    (
        36,
        13,
        11,
        Bay {
            right: 32_102,
            fwd: 84_123,
            dz: 182,
        },
    ),
    // **`HEAVYARCHERS-NA`'s piece 1531** (unit type 173, the Native American
    // Heavy Archer; item 3375, `docs/AI.md` §288): run1001d's
    // `RON_STATE_FRAME=13760` packet on the Iroquois Great Sahara seed holds
    // its three attack `AttachPos` rows, and `tools/recomp/get_position.py`'s
    // sweep of each is reproduced by these at every heading
    // [`whole_degrees`] reaches (257 of 361), x and y, the fits' own middle.
    // who=1's `1/35` loosed at the fleeing Scout `0/0` on 13749 from (8976,
    // 31176) in run1001b where this crate loosed from the figure's own
    // (9048, 31128): 8 frames against 9, and the Scout struck a frame late.
    (
        1531,
        11,
        12,
        Bay {
            right: -1826,
            fwd: 88743,
            dz: 163,
        },
    ),
    (
        1531,
        12,
        9,
        Bay {
            right: 334,
            fwd: 87412,
            dz: 164,
        },
    ),
    (
        1531,
        13,
        15,
        Bay {
            right: 9709,
            fwd: 66502,
            dz: 164,
        },
    ),
    // **`HEAVYARCHERS`' piece 123** (unit type 173, the Heavy Archer as the
    // Inca field it; item 3385, `docs/AI.md` §293): run1011c's
    // `RON_STATE_FRAME=13225` packet on the Inca's Great Sahara seed holds
    // its three attack `AttachPos` rows, and `tools/recomp/get_position.py`'s
    // sweep of each is reproduced at all 361 headings, x and y, by piece
    // 1531's three bays unchanged — the two pieces' rows are the same
    // model's. who=1's `1/50` loosed at the Nubians' `0/2002` on 13206 from
    // (5242, 31906) in run1011 where this crate loosed from the figure's own
    // (5256, 31992): 12 frames against 11, and the building struck a frame
    // late.
    (
        123,
        11,
        12,
        Bay {
            right: -1826,
            fwd: 88743,
            dz: 163,
        },
    ),
    (
        123,
        12,
        9,
        Bay {
            right: 334,
            fwd: 87412,
            dz: 164,
        },
    ),
    (
        123,
        13,
        15,
        Bay {
            right: 9709,
            fwd: 66502,
            dz: 164,
        },
    ),
    // **The Archers' piece 121** (unit type 171, `ARCHERS`; item 3574,
    // `docs/AI.md` §310): the Heavy Archer's three bays unchanged — the
    // same model, as piece 123 is piece 1531's. run1217b's `AMMO=5` window
    // on the Greek East Indies seed prints twenty-one rounds of who=1's
    // Archers over 7692..7867, fifteen poses, six figures at nine headings, every key
    // the piece releases on, and these put every one at the dump's `sx,
    // sy` to the unit, and at its `sz` over the figure's `z`. `1/53` loosed
    // at the fleeing Citizen `0/5` on 7727 from (8201, 8775) where this
    // crate loosed from the figure's own (8232, 8856): 18 frames against
    // 19, and the Citizen struck a frame late (run1217's packet).
    (
        121,
        11,
        12,
        Bay {
            right: -1826,
            fwd: 88743,
            dz: 163,
        },
    ),
    (
        121,
        12,
        9,
        Bay {
            right: 334,
            fwd: 87412,
            dz: 164,
        },
    ),
    (
        121,
        13,
        15,
        Bay {
            right: 9709,
            fwd: 66502,
            dz: 164,
        },
    ),
    // **The Mongol Archers' piece 1177** (unit type 171, `ARCHERS` as the
    // Mongols field it; item 3579, `docs/AI.md` §311): piece 121's bays
    // unchanged — the same model again. run1222's `AMMO=5` window on the
    // Mongol Great Sahara seed prints `1/74`'s `CHAR_ATTACK3` round from
    // (7087, 27292, 560) over a figure at (7128, 27240, 396), angle
    // −1790246912, and `1/73`'s `CHAR_ATTACK1` from (6795, 27316, 577) over
    // (6840, 27240, 414) at −1755316224: both exact, x, y and height. Ours
    // loosed from the figure's own square, 16 frames against 15, and the
    // fleeing Citizen `0/8` was struck a frame late. `CHAR_ATTACK2`'s row
    // is piece 121's, unmeasured here.
    (
        1177,
        11,
        12,
        Bay {
            right: -1826,
            fwd: 88743,
            dz: 163,
        },
    ),
    (
        1177,
        12,
        9,
        Bay {
            right: 334,
            fwd: 87412,
            dz: 164,
        },
    ),
    (
        1177,
        13,
        15,
        Bay {
            right: 9709,
            fwd: 66502,
            dz: 164,
        },
    ),
    // **The Mongol Javelineers' piece 1089** (unit type 83; item 3579): the
    // Javelineers' piece 34's bays in plan. run1222's `1/49` threw its
    // `CHAR_ATTACK2` javelin at `0/8` from (7227, 27933, 612) over a figure
    // at (7272, 27912, 442), angle −1382547456 — piece 34's `(right, fwd)`
    // exactly, and a height of 170 where piece 34's is 173, so the height is
    // the dump's. Ours threw from the figure, 18 frames against 17. The two
    // other rows are piece 34's, unmeasured here.
    (
        1089,
        11,
        12,
        Bay {
            right: 39258,
            fwd: 59130,
            dz: 166,
        },
    ),
    (
        1089,
        12,
        6,
        Bay {
            right: 1121,
            fwd: 50544,
            dz: 170,
        },
    ),
    (
        1089,
        13,
        27,
        Bay {
            right: 21620,
            fwd: 35314,
            dz: 159,
        },
    ),
    // **The Fire Lances' piece 46** (`FIRELANCES`, unit type 96; item 3577,
    // `docs/AI.md` §313): run1220b's `RON_STATE_FRAME=2976` packet on the
    // Chinese Great Sahara seed holds its three attack `AttachPos` rows,
    // node 0 at the keys the art names, and `tools/recomp/get_position.py`'s
    // sweep of each is reproduced by these at all 361 headings, x and y,
    // the fits' own middle, and `z` the sweep's. run1220's `AMMO=5` window
    // prints `1/21`, `1/22` and `1/23`'s rounds at the Citizen `0/10` at
    // these points; `1/22` loosed on 2968 from (9038, 31212) where this
    // crate loosed from the figure's own (9096, 31224): 10 frames against
    // 11, and the Citizen struck a frame late.
    (
        46,
        11,
        24,
        Bay {
            right: 12754,
            fwd: 65740,
            dz: 95,
        },
    ),
    (
        46,
        12,
        27,
        Bay {
            right: 1898,
            fwd: 79146,
            dz: 78,
        },
    ),
    (
        46,
        13,
        19,
        Bay {
            right: 8390,
            fwd: 58926,
            dz: 130,
        },
    ),
    // **The Royal Hwarang's piece 132** (`KORYOHWARANG`, unit type 182; item
    // 3365, `docs/AI.md` §283): the Longbowman's six release events on a
    // smaller figure (`koryo.bh3`), so piece 127's nodes do not serve it.
    // run991d's `RON_STATE_FRAME=15256` packet on the Korean Great Sahara
    // seed holds its six `AttachPos` rows, and `tools/recomp/
    // get_position.py`'s sweep of each is reproduced by these at all 361
    // headings, x and y (the fits' own middle, the sweep's degree a half
    // turn from [`whole_degrees`]). who=1's `1/74` loosed at
    // the Scout `0/0` on 15256 from (9264, 30827) in run991c where this
    // crate loosed from the figure's own (9336, 30840): 20 frames against
    // 19, and the Scout's first wound a frame late.
    (
        132,
        11,
        6,
        Bay {
            right: -3071,
            fwd: 73432,
            dz: 167,
        },
    ),
    (
        132,
        11,
        22,
        Bay {
            right: -3071,
            fwd: 73432,
            dz: 167,
        },
    ),
    (
        132,
        12,
        4,
        Bay {
            right: -13756,
            fwd: 73977,
            dz: 138,
        },
    ),
    (
        132,
        12,
        22,
        Bay {
            right: -3071,
            fwd: 73432,
            dz: 167,
        },
    ),
    (
        132,
        13,
        10,
        Bay {
            right: 9698,
            fwd: 61840,
            dz: 182,
        },
    ),
    (
        132,
        13,
        24,
        Bay {
            right: 4247,
            fwd: 57115,
            dz: 181,
        },
    ),
    // **The Native American Archers' piece 1529** (item 3255, `docs/AI.md`
    // §257): run943c's `RON_STATE_FRAME=10674` packet holds its four
    // `AttachPos` rows (`ARCHERS-NA-AGE0`'s three attack releases and a
    // group idle's), and `tools/recomp/get_position.py`'s sweep of each
    // attack row is reproduced by these at all 361 headings, x and y.
    // who=1's Archers `1/32` loosed at the Scout `0/0` on 10674 from
    // (5316, 31436) in run943c where this crate loosed from the figure's
    // own (5400, 31416): 18 frames against 19.
    (
        1529,
        11,
        12,
        Bay {
            right: -1826,
            fwd: 88748,
            dz: 163,
        },
    ),
    (
        1529,
        12,
        9,
        Bay {
            right: 335,
            fwd: 87407,
            dz: 164,
        },
    ),
    (
        1529,
        13,
        15,
        Bay {
            right: 9711,
            fwd: 66507,
            dz: 164,
        },
    ),
    // **The Native American Javelineers' piece 1441** (item 3255,
    // `docs/AI.md` §257): the same packet's three attack `AttachPos` rows,
    // swept and reproduced at all 361 headings, x and y. who=1's
    // Javelineers `1/27` threw on 10726 from (4820, 31215) in run943b
    // where this crate threw from the figure's own (4872, 31272).
    (
        1441,
        11,
        12,
        Bay {
            right: 42732,
            fwd: 64492,
            dz: 167,
        },
    ),
    (
        1441,
        12,
        6,
        Bay {
            right: -241,
            fwd: 56838,
            dz: 174,
        },
    ),
    (
        1441,
        13,
        27,
        Bay {
            right: 20479,
            fwd: 35674,
            dz: 162,
        },
    ),
    // **The Native American Peltasts' piece 1442** (item 3287, `docs/AI.md`
    // §260): run943c's `RON_STATE_FRAME=10674` packet already holds its
    // three attack `AttachPos` rows, though no Peltast is fielded until
    // 10852, and each row's sweep is reproduced by these at all 361
    // headings, x and y (the centre of the region, as 1441's are). who=1's
    // Javelineers became Peltasts on 10852, and `1/48`'s next javelin left
    // the figure's own (3336, 31512) in this crate: `0/2000` took it on
    // frame 10870 where run943 shows it on 10869 (block 10870).
    (
        1442,
        11,
        12,
        Bay {
            right: 43050,
            fwd: 64644,
            dz: 167,
        },
    ),
    (
        1442,
        12,
        6,
        Bay {
            right: -344,
            fwd: 57100,
            dz: 173,
        },
    ),
    (
        1442,
        13,
        27,
        Bay {
            right: 20539,
            fwd: 36144,
            dz: 161,
        },
    ),
    // King's Yeomanry: run586 get_position sweep, checked against
    // run587's actual arrows (COMBAT §87, item 1436).
    (
        129,
        11,
        6,
        Bay {
            right: -3361,
            fwd: 82393,
            dz: 169,
        },
    ),
    (
        129,
        11,
        22,
        Bay {
            right: -3361,
            fwd: 82393,
            dz: 169,
        },
    ),
    (
        129,
        12,
        4,
        Bay {
            right: -15899,
            fwd: 81974,
            dz: 137,
        },
    ),
    (
        129,
        12,
        22,
        Bay {
            right: -3361,
            fwd: 82393,
            dz: 169,
        },
    ),
    (
        129,
        13,
        10,
        Bay {
            right: 10521,
            fwd: 70321,
            dz: 185,
        },
    ),
    (
        129,
        13,
        24,
        Bay {
            right: 4876,
            fwd: 65759,
            dz: 184,
        },
    ),
    // **The Trireme, piece 290** (`docs/COMBAT.md` §50.2, items 542 and
    // 1200). Its `<RELEASEEVENT>`s are `400`, `666` and `1200` ms on node
    // 0, frames 5, 9 and 17, and node 0 walks down the keel as the swing
    // plays: 45 ahead of the hull's centre, then 14 and 66 behind it,
    // `sz` 88, 88 and 87 over the water's 0. run127 fixed each point at
    // one whole degree, 56 (both hulls broadside at `671481856`, seven
    // rounds a key), and item 542 wrote them as planar `Node`s. run437's
    // trireme `0/6` fires the same three at 176 (`2100559872`, 29 rounds
    // a key), where those rows put the third node's round a unit or two
    // off and one flight on 1448 a frame long — 1400.7 against 1399.6
    // at `PROJ_SPEED` 100. Each row is the centroid of the region that
    // reproduces both facings to the unit under the whole-degree
    // rotation. `CHAR_ATTACK2` plays the same file with the same events.
    // **Since item 2304 each row is the packet's** (run839, chapter
    // sixty-four's game, which fields the computer's Trireme `1/6`): the
    // model point times `guy_scale × RData +0x88` (4.8 × 0.8), `right =
    // −s·x` and `fwd = −s·y`, inside the region that reproduces
    // `get_position`'s truncated answer at all 256 headings. The centroids
    // of two facings had the third key's round a unit out at a third:
    // run835's `1/6` at 63° (`756678656`), `sx` 12348 against 12349.
    (
        290,
        crate::anim::ATTACK1,
        5,
        Bay {
            right: -293,
            fwd: 45_430,
            dz: 88,
        },
    ),
    (
        290,
        crate::anim::ATTACK1,
        9,
        Bay {
            right: -1_294,
            fwd: -14_718,
            dz: 88,
        },
    ),
    (
        290,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: -2_106,
            fwd: -66_171,
            dz: 87,
        },
    ),
    (
        290,
        crate::anim::ATTACK2,
        5,
        Bay {
            right: -293,
            fwd: 45_430,
            dz: 88,
        },
    ),
    (
        290,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: -1_294,
            fwd: -14_718,
            dz: 88,
        },
    ),
    (
        290,
        crate::anim::ATTACK2,
        17,
        Bay {
            right: -2_106,
            fwd: -66_171,
            dz: 87,
        },
    ),
    // **Piece 120, a British player's Bowmen: run256's twelve arrows**
    // (`docs/GOLDEN.md` §29, item 813). Its release events are 472's —
    // frame 12 on `CHAR_ATTACK1`, 9 on `CHAR_ATTACK2`, 15 on
    // `CHAR_ATTACK3` — and 472's nodes put every arrow within one to
    // four units of the dump, a frame of flight on the first, so the
    // piece is measured on its own. Three bowmen at three facings
    // (−999686144, −987168768, −885391360) give seven distinct launch
    // points; each row is the centroid of the region of integer `(right,
    // fwd)` that reproduces every point of its key exactly under the
    // whole-degree rotation, and `dz` is `sz` less the figure's own `z`,
    // the same on every arrow of the key.
    (
        120,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: -1_830,
            fwd: 88_800,
            dz: 163,
        },
    ),
    (
        120,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: 118,
            fwd: 87_244,
            dz: 164,
        },
    ),
    (
        120,
        crate::anim::ATTACK3,
        15,
        Bay {
            right: 9_536,
            fwd: 66_509,
            dz: 164,
        },
    ),
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
    // **Piece 32, the Slingers (type 82): run17's 27 stones**
    // (`docs/COMBAT.md` §70, item 1040). run17 prints `AMMO=5` over a
    // Slinger fight but its `GUY` detail is 1, so no record names the
    // animation. The unit's own `recharging` does: the stone leaves on
    // the frame the release event is crossed, `starttime − 1` frames after
    // the block `recharging` jumps, and the piece's three events (21 on
    // `CHAR_ATTACK1`, 22 on `CHAR_ATTACK2`, 16 on `CHAR_ATTACK3`) are
    // distinct, so each stone names its key. Three Slingers at twenty
    // whole-degree headings; each row is the centroid of the region of
    // integer `(right, fwd)` that reproduces every stone of its key
    // exactly, and `dz` is `sz` less the figure's `z`, the same on every
    // stone of the key. Without them every Slinger stone left the unit's
    // own square and flew a frame long across a boundary: Great Lakes'
    // second word, 4781.
    (
        32,
        crate::anim::ATTACK1,
        21,
        Bay {
            right: 69_776,
            fwd: 67_449,
            dz: 178,
        },
    ),
    (
        32,
        crate::anim::ATTACK2,
        22,
        Bay {
            right: 41_209,
            fwd: 118_012,
            dz: 151,
        },
    ),
    (
        32,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: -42_148,
            fwd: 113_222,
            dz: 177,
        },
    ),
    // **Piece 38, the Maya's Balamob Slingers** (type 88, `BAMOBSLINGERS`;
    // item 3599, `docs/AI.md` §320.2): a nation's own piece on its own
    // model (`Balamob.bh3`), so no continent offset and none of piece 32's
    // keys — `CHAR_ATTACK1` at 18, `2` at 19, `3` at 21. run1247's
    // `RON_STATE_FRAME=4306` packet on the Maya East Indies game holds the
    // three `AttachPos` rows, and `tools/recomp/get_position.py`'s sweep of
    // each is reproduced at all 361 headings by these, the centroids of
    // their exact regions (650, 666 and 97 lattice points). Without them
    // every Balamob stone left the unit's own square: the Maya word's
    // 4324, whose state had parted on 4310 — `1/32`'s stone at the Citizen
    // `0/10` from (6072, 8616) in 7 frames where the original's left from
    // (5950, 8535, 362) in 6.
    (
        38,
        crate::anim::ATTACK1,
        18,
        Bay {
            right: 18_702,
            fwd: 146_963,
            dz: 177,
        },
    ),
    (
        38,
        crate::anim::ATTACK2,
        19,
        Bay {
            right: 22_458,
            fwd: 145_250,
            dz: 168,
        },
    ),
    (
        38,
        crate::anim::ATTACK3,
        21,
        Bay {
            right: 36_186,
            fwd: 144_839,
            dz: 173,
        },
    ),
    // **Piece 39, the Royal Balamob Slingers** (type 89, the Maya's
    // Classical slinger; item 3785, `docs/GOLDEN.md` §150.1): piece 38's
    // keys — `CHAR_ATTACK1` at 18, `2` at 19, `3` at 21 — on its own model.
    // run1457's `RON_STATE_FRAME=700` packet on the second melee's lobby,
    // where the idle human fields three, holds the three `AttachPos` rows,
    // `RData +0x88` 1.0, and `fit_bay.py` fits each sweep at all 361
    // headings (46, 223 and 428 lattice points; the centroid is kept).
    // Without them `0/19`'s stone of 713 at the Greek Catapult `1/26` left
    // the figure's square, and the Catapult's strike of 718 did not come:
    // the second melee's word stood at 718 rather than 784.
    (
        39,
        crate::anim::ATTACK1,
        18,
        Bay {
            right: 37_765,
            fwd: 138_192,
            dz: 172,
        },
    ),
    (
        39,
        crate::anim::ATTACK2,
        19,
        Bay {
            right: 37_229,
            fwd: 135_427,
            dz: 161,
        },
    ),
    (
        39,
        crate::anim::ATTACK3,
        21,
        Bay {
            right: 32_848,
            fwd: 144_969,
            dz: 148,
        },
    ),
    // **Piece 1440, the Slingers in the `NA` art style** (`32 + 4 ×
    // 0x160`; item 3345, `docs/AI.md` §277): the Lakota's and the
    // Iroquois' Slinger, whose `UNIT_CONTINENT` is 4. run954b's
    // `RON_STATE_FRAME` packet on the Iroquois Great Lakes game holds its
    // three `AttachPos` rows (`CHAR_ATTACK1` at 21, `2` at 22, `3` at 16,
    // piece 32's keys), and `tools/recomp/get_position.py`'s sweep of each
    // is reproduced by these at all 361 headings, x, y and z; each is the
    // centroid of its exact region (534, 319 and 28 lattice points).
    // Without them every Lakota stone left the unit's own square: the
    // Lakota word's 6071, whose state had parted on 6052 — `1/18`'s stone
    // at `0/2` landing a frame late (8 frames against 7) and `1/20`'s at
    // the city centre `0/2000` (10 against 9).
    (
        1440,
        crate::anim::ATTACK1,
        21,
        Bay {
            right: 71_298,
            fwd: 69_592,
            dz: 174,
        },
    ),
    (
        1440,
        crate::anim::ATTACK2,
        22,
        Bay {
            right: 32_704,
            fwd: 120_229,
            dz: 144,
        },
    ),
    (
        1440,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: -47_065,
            fwd: 114_687,
            dz: 168,
        },
    ),
    // **Piece 1792, the Slingers in the Persians' art style** (`32 + 5 ×
    // 0x160`; item 3355, `docs/AI.md` §279): the Persians'
    // `UNIT_CONTINENT` is 5. run984c's `RON_STATE_FRAME=6251` packet on
    // the Persian Great Lakes game holds its three `AttachPos` rows, piece
    // 1440's keys, and `get_position.py`'s sweep of each is reproduced at
    // all 361 headings by piece 1440's own three bays — the same centroids
    // of the same regions (534, 319 and 28 lattice points). Without them
    // the Persian stones left the unit's own square: the Persian word's
    // 6323, whose state had parted on 6319 — `1/22`'s stone at the citizen
    // `0/3` landing a frame late (6 frames against 5).
    (
        1792,
        crate::anim::ATTACK1,
        21,
        Bay {
            right: 71_298,
            fwd: 69_592,
            dz: 174,
        },
    ),
    (
        1792,
        crate::anim::ATTACK2,
        22,
        Bay {
            right: 32_704,
            fwd: 120_229,
            dz: 144,
        },
    ),
    (
        1792,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: -47_065,
            fwd: 114_687,
            dz: 168,
        },
    ),
    // **Piece 1088, the Slingers in the Asian art style** (`32 + 3 ×
    // 0x160`; item 3578, `docs/AI.md` §312): the Koreans' Slinger, whose
    // `UNIT_CONTINENT` is 3. run1221's `RON_STATE_FRAME=4287` packet on
    // the Korean East Indies game holds its three `AttachPos` rows, piece
    // 32's keys, at points of their own; `tools/recomp/get_position.py`'s
    // sweep of each is reproduced at all 361 headings by these, the
    // centroids of their exact regions (64, 96 and 64 lattice points).
    // Without them every Korean stone left the unit's own square: the
    // Korean word's 4350, whose state had parted on 4294 — `1/29`'s stone
    // at the Citizen `0/13` from (6600, 9768) in 10 frames where the
    // original's left from (6541, 9663) in 9.
    (
        1088,
        crate::anim::ATTACK1,
        21,
        Bay {
            right: 74_224,
            fwd: 62_577,
            dz: 176,
        },
    ),
    (
        1088,
        crate::anim::ATTACK2,
        22,
        Bay {
            right: 43_592,
            fwd: 116_950,
            dz: 150,
        },
    ),
    (
        1088,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: -38_569,
            fwd: 114_599,
            dz: 176,
        },
    ),
    // **Piece 130, the Koreans' Hwarang** (type 180; item 3578, `docs/AI.md`
    // §312): a nation's own piece, so no continent offset. run1221's
    // packet holds its six attack `AttachPos` rows, two releases on each
    // slot (`CHAR_ATTACK1` at 6 and 22, `2` at 4 and 22, `3` at 10 and
    // 24); three share one point. `get_position.py`'s sweep of each is
    // reproduced at all 361 headings by these (30, 422, 17 and 483 lattice
    // points). Without them the Hwarang `1/44`'s arrow at `0/13`, loosed
    // on 4279, left from the figure's own (6072, 10728) and flew 23 frames
    // where the original's left from (6085, 10656, 193) and flew 22.
    (
        130,
        crate::anim::ATTACK1,
        6,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        130,
        crate::anim::ATTACK1,
        22,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        130,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: -13_756,
            fwd: 73_975,
            dz: 138,
        },
    ),
    (
        130,
        crate::anim::ATTACK2,
        22,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        130,
        crate::anim::ATTACK3,
        10,
        Bay {
            right: 9_698,
            fwd: 61_839,
            dz: 182,
        },
    ),
    (
        130,
        crate::anim::ATTACK3,
        24,
        Bay {
            right: 4_247,
            fwd: 57_120,
            dz: 181,
        },
    ),
    // **Piece 131, the Koreans' Elite Hwarang** (type 181; item 3609,
    // `docs/AI.md` §322): run1263's `RON_STATE_FRAME=8362` packet on the
    // Korean Classical Great Sahara seed holds its six attack `AttachPos`
    // rows, bit for bit piece 130's, and `get_position.py`'s sweep of each
    // is reproduced at all 361 headings by piece 130's bays (the same 30,
    // 422, 17 and 483 lattice points). Without them the Elite Hwarang
    // `1/135`'s arrow at `0/4`, loosed on 8361, left from the figure's own
    // (9432, 28584) and flew 15 frames where the original's left from
    // (9389, 28645, 606) and flew 14.
    (
        131,
        crate::anim::ATTACK1,
        6,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        131,
        crate::anim::ATTACK1,
        22,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        131,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: -13_756,
            fwd: 73_975,
            dz: 138,
        },
    ),
    (
        131,
        crate::anim::ATTACK2,
        22,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        131,
        crate::anim::ATTACK3,
        10,
        Bay {
            right: 9_698,
            fwd: 61_839,
            dz: 182,
        },
    ),
    (
        131,
        crate::anim::ATTACK3,
        24,
        Bay {
            right: 4_247,
            fwd: 57_120,
            dz: 181,
        },
    ),
    // **Piece 112, the Persians' Immortals** (type 162; item 3355,
    // `docs/AI.md` §279): a nation's own piece, so no continent offset.
    // run984b's `RON_STATE_FRAME=6011` packet on the Persian Great Lakes
    // game holds its two `AttachPos` rows — `CHAR_ATTACK2` at 14 and `3` at
    // 6, there is no `CHAR_ATTACK1` row — and `tools/recomp/get_position.
    // py`'s sweep of each is reproduced by these at all 361 headings, x, y
    // and z; each is the centroid of its exact region (105 and 115 lattice
    // points). Without them every Immortal's arrow left the unit's own
    // square: the Persian word's 6046, whose state had parted on 6021 —
    // `1/16`'s arrow at the Scout `0/0` thrown on 6012 landing a frame late
    // (11 frames against 10), so `0/0` took no wound and stood.
    (
        112,
        crate::anim::ATTACK2,
        14,
        Bay {
            right: 25_067,
            fwd: 104_603,
            dz: 178,
        },
    ),
    (
        112,
        crate::anim::ATTACK3,
        6,
        Bay {
            right: 27_678,
            fwd: 36_793,
            dz: 171,
        },
    ),
    // **Piece 1143, the Anti-tank Rifle in the Asian art style** (type
    // 137, `87 + 3 × 0x160`; item 3538, `docs/AI.md` §305): the
    // Chinese's `UNIT_CONTINENT` is 3. run1155c's `RON_STATE_FRAME=3003`
    // packet on the Chinese Great Sahara game holds its three `AttachPos`
    // rows — node 0 at 18 of `CHAR_ATTACK1`, 4 of `2` and 16 of `3` — and
    // `tools/recomp/get_position.py`'s sweep of each is reproduced by these
    // at all 361 headings, x and y; each is the centroid of its exact
    // region (62, 160 and 57 lattice points). The muzzle stands some 180
    // units ahead of the figure. Without them every rifle round left the
    // unit's own square and landed a frame late: the Chinese word's 3271,
    // whose state had parted on 3243 — `1/41`'s round at the Citizen
    // `0/4`, thrown on 3239, striking on 3243 against theirs' 3242 (5
    // frames over 1,823 units at `PROJ_SPEED` 350, against 4), so `0/4`
    // fled a frame late and `1/42`'s chase review of 3270 found it in
    // reach of the move's goal where theirs' did not.
    (
        1143,
        crate::anim::ATTACK1,
        18,
        Bay {
            right: 17_058,
            fwd: 180_195,
            dz: 60,
        },
    ),
    (
        1143,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: 4_863,
            fwd: 179_398,
            dz: 63,
        },
    ),
    (
        1143,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: -2_310,
            fwd: 161_815,
            dz: 83,
        },
    ),
    // **Piece 88, the German Anti-tank Rifle** (type 138,
    // `ANTITANKRIFLEGERMAN`; item 3677, `docs/AI.md` §346): run1351's
    // `RON_STATE_FRAME=4303` packet on the German Industrial Great Lakes
    // game holds its three `AttachPos` rows, bit for bit piece 1143's (node
    // 0 at 18 of `CHAR_ATTACK1`, 4 of `2` and 16 of `3`), and each sweep's
    // exact region is 1143's (62, 160 and 57 lattice points), so the rows
    // are its. who=1's `1/50` and `1/51` fired their rounds of 4305 from the
    // figures' own squares here, where the original's left (−180, −7) and
    // (−175, +38) off them, 60 and 63 up.
    (
        88,
        crate::anim::ATTACK1,
        18,
        Bay {
            right: 17_058,
            fwd: 180_195,
            dz: 60,
        },
    ),
    (
        88,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: 4_863,
            fwd: 179_398,
            dz: 63,
        },
    ),
    (
        88,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: -2_310,
            fwd: 161_815,
            dz: 83,
        },
    ),
    // **Piece 90, the German Bazooka** (type 140, `BAZOOKAGERMAN`,
    // `bazooka_german.bh3`; item 3737, `docs/AI.md` §365.2): run1416's
    // `RON_STATE_FRAME=4016` packet on the Spanish Modern East Indies game
    // holds its three `AttachPos` rows (node 0 at 17 of `CHAR_ATTACK1`, 7 of
    // `2` and 13 of `3`), `RData +0x88` 1.0, and `fit_bay.py` fits each sweep
    // (114, 396 and 254 lattice points; the centroid is kept, 361 of 361
    // headings each). No other packet on disk loads the piece. who=1's `1/33`
    // and `1/32` loosed their rounds at the human's Citizen `0/13` from the
    // figures' own squares here, 146 and 120 below the original's, and flew
    // a frame longer.
    (
        90,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: 37_009,
            fwd: 121_444,
            dz: 120,
        },
    ),
    (
        90,
        crate::anim::ATTACK2,
        7,
        Bay {
            right: 37_800,
            fwd: 98_330,
            dz: 146,
        },
    ),
    (
        90,
        crate::anim::ATTACK3,
        13,
        Bay {
            right: 52_635,
            fwd: 109_819,
            dz: 139,
        },
    ),
    // **Piece 203, the Main Battle Tank** (type 253, `HEAVYTANKCTWW`,
    // `challenger.bh3`; item 3743, `docs/AI.md` §367.2): run1419b's packet
    // (`RON_STATE_FRAME=3212`) holds its seven `AttachPos` rows — node 4 and
    // node 0 at 11 of `CHAR_ATTACK1`, 23 of `2` and 27 of `3`, `RData +0x88`
    // 0.8. **No `<RESTRICTION>` names `HEAVYTANKCTWW`** (the foot's
    // `HEAVYTANK` is type 251's graph), so the event's `node & 3` is not
    // below a count of 0 and `get_position` takes its non-pivot arm: the
    // muzzle alone, about 350 out and 38 up, not the turret's 143. `fit_bay.py`
    // fits each sweep (638, 268 and 178 lattice points; the centroid is
    // kept, 361 of 361 headings each). Without them who=1's `1/55` loosed
    // its round of 3514 at the human's Citizen `0/10` from its own square
    // (5352, 30792, 515) against the original's (5166, 30496, 553), flew 7
    // frames against 6, and the citizen died a frame late.
    (
        203,
        crate::anim::ATTACK1,
        11,
        Bay {
            right: -541,
            fwd: 347_780,
            dz: 38,
        },
    ),
    (
        203,
        crate::anim::ATTACK2,
        23,
        Bay {
            right: -534,
            fwd: 350_174,
            dz: 38,
        },
    ),
    (
        203,
        crate::anim::ATTACK3,
        27,
        Bay {
            right: -541,
            fwd: 308_736,
            dz: 35,
        },
    ),
    // **Piece 54, the Assault Infantry** (type 104, `MECHINFANTRY`,
    // `ASSAULTINFANTRY-DEFAULT-AGE0`, `assault_infantry.bh3`; item 3740,
    // `docs/AI.md` §367.1): six packets on disk load it — run785, run789,
    // run795, run810, run823 and run825, the same floats in each — with its
    // eight `AttachPos` rows (node 0 at 12 and 13 of `CHAR_ATTACK1`, 4, 14
    // twice and 24 of `2`, 12 and 17 of `3`: the art's eight
    // `RELEASEEVENT`s, tracers among them), `RData +0x88` 1.0, and
    // `fit_bay.py` fits each sweep on run785's packet (147, 103, 57, 167,
    // 124, 97 and 378 lattice points; the centroid is kept, 361 of 361
    // headings each). Without them who=1's `1/34` loosed its round of 3204
    // at the human's Citizen `0/13` from its own square (5640, 31800, 364)
    // against the original's (5530, 31784, 533), flew 3 frames against 2,
    // and the citizen was struck a frame late.
    (
        54,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 24_607,
            fwd: 108_706,
            dz: 169,
        },
    ),
    (
        54,
        crate::anim::ATTACK1,
        13,
        Bay {
            right: 20_358,
            fwd: 103_245,
            dz: 173,
        },
    ),
    (
        54,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: 8_133,
            fwd: 85_578,
            dz: 121,
        },
    ),
    (
        54,
        crate::anim::ATTACK2,
        14,
        Bay {
            right: 31_037,
            fwd: 80_976,
            dz: 118,
        },
    ),
    (
        54,
        crate::anim::ATTACK2,
        24,
        Bay {
            right: 22_929,
            fwd: 83_745,
            dz: 122,
        },
    ),
    (
        54,
        crate::anim::ATTACK3,
        12,
        Bay {
            right: 27_684,
            fwd: 159_512,
            dz: 106,
        },
    ),
    (
        54,
        crate::anim::ATTACK3,
        17,
        Bay {
            right: 38_064,
            fwd: 146_570,
            dz: 117,
        },
    ),
    // **Piece 1108, the Infantry in the Asian art style** (type 102,
    // `INFANTRY-ASIAN-AGE0`, `inf_asian.bh3`: base 52 at style 3; item
    // 3745, `docs/AI.md` §369.1): no packet on disk loaded it, and run1424's
    // `RON_STATE_FRAME=3156` packet on the Japanese Modern Great Lakes seed
    // holds its six `AttachPos` rows (node 0 at 12 and 13 of
    // `CHAR_ATTACK1`, 4 and 14 twice of `2`, 12 of `3`), `RData +0x88` 1.0.
    // The art has no `<RESTRICTION>`, so the rounds take the non-pivot arm,
    // and `fit_bay.py` fits each sweep (49, 98, 915, 819 and 322 lattice
    // points; the centroid is kept, 361 of 361 headings each). Without them
    // who=1's `1/18` loosed its rounds of 3157 at the human's Citizen
    // `0/4` from its own square (4200, 34056, 489) against the original's
    // (4203, 33991, 607), flew 3 frames against 2, and the citizen was
    // struck a frame late.
    (
        1108,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        1108,
        crate::anim::ATTACK1,
        13,
        Bay {
            right: 20_717,
            fwd: 82_052,
            dz: 170,
        },
    ),
    (
        1108,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: 9667,
            fwd: 64_207,
            dz: 120,
        },
    ),
    (
        1108,
        crate::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        1108,
        crate::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
        },
    ),
    // **Piece 60, the Jaguar Infantry in the default art style** (type 110,
    // `JAGINFANTRY-DEFAULT-AGE0`, `jaguar_inf.bh3`; item 3766, `docs/AI.md`
    // §374.1): run1441's packet (`RON_STATE_FRAME=3294` on the Aztec Modern
    // Great Lakes seed) loads its six `AttachPos` rows (node 0; frame 12 of
    // `CHAR_ATTACK1`, 14 twice and 24 of `CHAR_ATTACK2`, 12 and 17 of
    // `CHAR_ATTACK3`), `RData +0x88` 1.0, and `fit_bay.py` fits each sweep
    // (49, 819, 348, 94 and 540 lattice points, 361 of 361 headings; the
    // centroid is kept). The three `Inf Attack1`/`Inf Attack2` keys fit
    // piece 52's rows to the unit; the two `JaguarInf Attack3` keys are its
    // own. Without them who=1's `1/24` loosed its round of 3309 from its own
    // square (5640, 30840, 474) against the original's (5625, 30777, 592),
    // flew 6 frames against 5 and exploded a frame late on 3313.
    (
        60,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        60,
        crate::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        60,
        crate::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        60,
        crate::anim::ATTACK3,
        12,
        Bay {
            right: 17_840,
            fwd: 218_651,
            dz: 181,
        },
    ),
    (
        60,
        crate::anim::ATTACK3,
        17,
        Bay {
            right: 22_846,
            fwd: 225_519,
            dz: 170,
        },
    ),
    // **Piece 52, the Infantry in the default art style** (type 102,
    // `INFANTRY-DEFAULT-AGE0`, `inf.bh3`: base 52 at style 0, the
    // Egyptians' `UNIT_CONTINENT`; item 3747, `docs/AI.md` §371.1): no
    // packet on disk loaded it, and run1426b's `RON_STATE_FRAME=3722`
    // packet on the Egyptian Modern Great Lakes seed holds its six
    // `AttachPos` rows (node 0 at 12 and 13 of `CHAR_ATTACK1`, 4, 14 and
    // 24 of `2`, 12 of `3`), `RData +0x88` 1.0. The art has no
    // `<RESTRICTION>`, so the rounds take the non-pivot arm, and
    // `fit_bay.py` fits each sweep (49, 98, 915, 819, 348 and 322 lattice
    // points; the centroid is kept). Five keys fit piece 1108's rows to the
    // unit; `CHAR_ATTACK2` at 24 is this piece's own. Without them who=1's
    // `1/19` loosed its round of 3723 at the human's Citizen `0/12` from
    // its own square (6456, 31752, 221) against the original's (6417,
    // 31701, 341), and the citizen died on 3748 in theirs alone.
    (
        52,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        52,
        crate::anim::ATTACK1,
        13,
        Bay {
            right: 20_717,
            fwd: 82_052,
            dz: 170,
        },
    ),
    (
        52,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: 9667,
            fwd: 64_207,
            dz: 120,
        },
    ),
    (
        52,
        crate::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        52,
        crate::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        52,
        crate::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
        },
    ),
    // **Piece 1145, the Bazooka in the Asian art style** (type 139,
    // `BAZOOKA-ASIAN-AGE0`, `bazooka_asian.bh3`: base 89 at style 3; item
    // 3745, `docs/AI.md` §369.1): run1424's packet holds its three
    // `AttachPos` rows (node 0 at 17 of `CHAR_ATTACK1`, 7 of `2`, 13 of
    // `3`), `RData +0x88` 1.0; the art has no `<RESTRICTION>`, and
    // `fit_bay.py` fits each sweep (85, 1278 and 187 lattice points; the
    // centroid is kept, 361 of 361 headings each). Without them who=1's
    // `1/52` loosed its round of 3842 from its own square (5400, 30744,
    // 514) against the original's (5299, 30765, 664), and the human's
    // `0/2002` and `0/2000` were struck late (run1424b, 3850 and 3860).
    (
        1145,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: 32_578,
            fwd: 121_906,
            dz: 127,
        },
    ),
    (
        1145,
        crate::anim::ATTACK2,
        7,
        Bay {
            right: 33_663,
            fwd: 97_657,
            dz: 150,
        },
    ),
    (
        1145,
        crate::anim::ATTACK3,
        13,
        Bay {
            right: 48_137,
            fwd: 109_678,
            dz: 146,
        },
    ),
    // **Piece 89, the Bazooka in the default art style** (type 139,
    // `BAZOOKA-DEFAULT-AGE0`, `bazooka.bh3`: base 89 at style 0; item 3747,
    // `docs/AI.md` §371.1): run1426b's packet holds its three `AttachPos`
    // rows (node 0 at 17 of `CHAR_ATTACK1`, 7 of `2`, 13 of `3`), `RData
    // +0x88` 1.0, no `<RESTRICTION>`; `fit_bay.py` fits each sweep (85, 1278
    // and 187 lattice points) to piece 1145's rows to the unit. Without them
    // who=1's `1/54` loosed its round of 3840 from its own square (5880,
    // 30648, 462) against the original's (5859, 30547, 612).
    (
        89,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: 32_578,
            fwd: 121_906,
            dz: 127,
        },
    ),
    (
        89,
        crate::anim::ATTACK2,
        7,
        Bay {
            right: 33_663,
            fwd: 97_657,
            dz: 150,
        },
    ),
    (
        89,
        crate::anim::ATTACK3,
        13,
        Bay {
            right: 48_137,
            fwd: 109_678,
            dz: 146,
        },
    ),
    // **Piece 441, the Bazooka in the Arab art style** (type 139,
    // `BAZOOKA-ARAB-AGE0`, `bazooka_arab.bh3`: base 89 at style 1, the
    // Bantu's `UNIT_CONTINENT`; item 3748, `docs/AI.md` §370.2): no packet
    // on disk loads it, and its three attack keys are taken as piece 1145's
    // above — the same `Bazooka Attack1..3` animations and release keys.
    // run1425's `1/86` round on the human's `0/13` (block 4295, the figure
    // at (6648, 31224) and −1041694720, `z` 267) left (6527, 31190, 394),
    // which the `CHAR_ATTACK1` row reproduces exactly; without it the
    // round left the figure's own square, flew 10 frames against 9 and
    // exploded a frame late on 4304. The `CHAR_ATTACK2` and `CHAR_ATTACK3`
    // rows rest on the shape alone.
    (
        441,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: 32_578,
            fwd: 121_906,
            dz: 127,
        },
    ),
    (
        441,
        crate::anim::ATTACK2,
        7,
        Bay {
            right: 33_663,
            fwd: 97_657,
            dz: 150,
        },
    ),
    (
        441,
        crate::anim::ATTACK3,
        13,
        Bay {
            right: 48_137,
            fwd: 109_678,
            dz: 146,
        },
    ),
    // **Piece 87, the Anti-tank Rifle in the default art style** (type
    // 137, `ANTITANKRIFLE`; item 3686, `docs/AI.md` §348.1): the piece the
    // Americans' rifles play here (`RON_DEBUG_GAP`'s `p87`). Taken as
    // piece 1143's three rows, as 3677 found piece 88's: run1338b's `1/18`
    // round of 3105 (`CHAR_ATTACK3` frame 16, the figure at (9528, 31080)
    // and −798359552, `z` 612) left (9379, 31019) at 695, which the
    // `CHAR_ATTACK3` row reproduces exactly, x, y and height. The other
    // two rows are 1143's unmeasured on this piece: no packet was taken.
    // Without them the round left the figure's square, and run1338b's
    // `1/44` round of 3142 flew 3 frames against 2.
    (
        87,
        crate::anim::ATTACK1,
        18,
        Bay {
            right: 17_058,
            fwd: 180_195,
            dz: 60,
        },
    ),
    (
        87,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: 4_863,
            fwd: 179_398,
            dz: 63,
        },
    ),
    (
        87,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: -2_310,
            fwd: 161_815,
            dz: 83,
        },
    ),
    // **Piece 167, the Greyhound** (type 217, `GREYHOUND`,
    // `GREYHOUND-DEFAULT-AGE0`, `armoredscoutcar_na.bh3`; item 3690,
    // `docs/AI.md` §348.2): no `<RESTRICTION>` names it and its
    // `turret_angles` are 0, so its rounds take `get_position`'s non-pivot
    // arm. run1359's `RON_STATE_FRAME=5619` packet on the American
    // Industrial Great Sahara game holds its 37 `AttachPos` rows (`pos_start`
    // 136, `RData +0x88` 1.1): node 0 at the nine `<RELEASEEVENT>`s of
    // `CHAR_ATTACKWALK` and of `CHAR_ATTACK2` (frames 3, 9, 12, 17, 20, 22,
    // 24 and 29 twice, the two 29s the same bits). Each row is the centroid
    // of the exact region `tools/recomp/fit_bay.py` finds for its
    // `get_position.py` sweep, all 361 headings, x and y. The muzzle stands
    // some 177 units ahead of the figure: without the rows run1356's `1/104`
    // round of 5620 left the figure's square (7926, 30423) at 655, where
    // theirs left (7758, 30368) at 672, and flew 3 frames against 2.
    (
        167,
        crate::anim::ATTACKWALK,
        3,
        Bay {
            right: 496,
            fwd: 177_238,
            dz: 17,
        },
    ),
    (
        167,
        crate::anim::ATTACKWALK,
        9,
        Bay {
            right: 691,
            fwd: 177_606,
            dz: 13,
        },
    ),
    (
        167,
        crate::anim::ATTACKWALK,
        12,
        Bay {
            right: 591,
            fwd: 177_528,
            dz: 14,
        },
    ),
    (
        167,
        crate::anim::ATTACKWALK,
        17,
        Bay {
            right: 0,
            fwd: 177_722,
            dz: 12,
        },
    ),
    (
        167,
        crate::anim::ATTACKWALK,
        20,
        Bay {
            right: -703,
            fwd: 177_226,
            dz: 17,
        },
    ),
    (
        167,
        crate::anim::ATTACKWALK,
        22,
        Bay {
            right: -717,
            fwd: 177_502,
            dz: 15,
        },
    ),
    (
        167,
        crate::anim::ATTACKWALK,
        24,
        Bay {
            right: -834,
            fwd: 177_267,
            dz: 17,
        },
    ),
    (
        167,
        crate::anim::ATTACKWALK,
        29,
        Bay {
            right: -87,
            fwd: 177_560,
            dz: 14,
        },
    ),
    (
        167,
        crate::anim::ATTACK2,
        3,
        Bay {
            right: 0,
            fwd: 176_708,
            dz: 17,
        },
    ),
    (
        167,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: 0,
            fwd: 177_096,
            dz: 13,
        },
    ),
    (
        167,
        crate::anim::ATTACK2,
        12,
        Bay {
            right: 0,
            fwd: 176_983,
            dz: 14,
        },
    ),
    (
        167,
        crate::anim::ATTACK2,
        17,
        Bay {
            right: 0,
            fwd: 177_197,
            dz: 12,
        },
    ),
    (
        167,
        crate::anim::ATTACK2,
        20,
        Bay {
            right: 0,
            fwd: 176_708,
            dz: 17,
        },
    ),
    (
        167,
        crate::anim::ATTACK2,
        22,
        Bay {
            right: 0,
            fwd: 176_983,
            dz: 15,
        },
    ),
    (
        167,
        crate::anim::ATTACK2,
        24,
        Bay {
            right: 0,
            fwd: 176_747,
            dz: 17,
        },
    ),
    (
        167,
        crate::anim::ATTACK2,
        29,
        Bay {
            right: 0,
            fwd: 177_029,
            dz: 14,
        },
    ),
    // **Piece 65, the Black Watch** (type 115, `EHIGHLANDERS`,
    // `HIGHLANDERELITE-DEFAULT-AGE0`; item 3683, `docs/AI.md` §347):
    // run1355's `RON_STATE_FRAME=2933` packet on the British Industrial
    // Great Sahara game holds its three `AttachPos` rows (node 0 at 10 of
    // `CHAR_ATTACK1`, 15 of `2` and 16 of `3`, `RData +0x88` 1.0), and
    // `fit_bay.py` fits each sweep at all 361 headings with its region's
    // centroid (629, 15 and 412 lattice points). The musket stands some 122
    // to 134 units ahead of the figure. who=1's `1/15`, `1/16` and `1/17`
    // threw from the figures' own squares here; run1355b's `1/16` round of
    // 2911 left (8435, 31093) at 763, 133 ahead of the figure's (8568,
    // 31080) and 122 up, and ours' struck the Citizen `0/8` on 2901 where
    // theirs' did not.
    (
        65,
        crate::anim::ATTACK1,
        10,
        Bay {
            right: 15_269,
            fwd: 121_470,
            dz: 168,
        },
    ),
    (
        65,
        crate::anim::ATTACK2,
        15,
        Bay {
            right: 5_281,
            fwd: 133_941,
            dz: 122,
        },
    ),
    (
        65,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: 13_151,
            fwd: 120_822,
            dz: 167,
        },
    ),
    // **Piece 64, the Highlanders** (type 114, `HIGHLANDER-DEFAULT-AGE0`;
    // item 3542, `docs/AI.md` §306): node 0 at frame 6 of `CHAR_ATTACK1`, 10
    // of `2` and 12 of `3`, `RData +0x88` 1.05, off the same run1185 packet
    // as the Sloop's below; each row reproduces its sweep at all 361
    // headings, x, y and z, the centroid of its exact region (655, 481 and
    // 174 lattice points). The musket stands some 145 units ahead of the
    // figure. Without them `1/91`'s round at the Citizen `0/10`, thrown on
    // 4732 from the figure's own (5928, 10488), flew 4 frames and struck on
    // 4735 against theirs' 4734.
    (
        64,
        crate::anim::ATTACK1,
        6,
        Bay {
            right: 13_374,
            fwd: 144_082,
            dz: 168,
        },
    ),
    (
        64,
        crate::anim::ATTACK2,
        10,
        Bay {
            right: 13_848,
            fwd: 145_503,
            dz: 169,
        },
    ),
    (
        64,
        crate::anim::ATTACK3,
        12,
        Bay {
            right: 13_535,
            fwd: 145_683,
            dz: 168,
        },
    ),
    // **Piece 277, the Sloop** (type 327, `SLOOP-DEFAULT-AGE0`; item 3542,
    // `docs/AI.md` §306): its `InvisibleAmmo` leaves node 0 at frame 4 of
    // `CHAR_ATTACK1` and node 1 at frame 4 of `CHAR_ATTACK2` and
    // `CHAR_ATTACKWALK`, `RData +0x88` 0.7. run1185's `RON_STATE_FRAME=4582`
    // packet on the British East Indies game holds the three `AttachPos`
    // rows, and `tools/recomp/get_position.py`'s sweep of each is
    // reproduced by these at all 361 headings, x, y and z; each is the
    // centroid of its exact region (969, 844 and 844 lattice points).
    // Without them the Sloop `1/61`'s round at the Citizen `0/13`, thrown
    // on 4580, left the hull's own square (3336, 6840) and flew 8 frames
    // where theirs left (3315, 6861) and flew 9: `0/13` died on 4587
    // against theirs' 4588, the British word.
    (
        277,
        crate::anim::ATTACK1,
        4,
        Bay {
            right: 29_986,
            fwd: 193,
            dz: -2,
        },
    ),
    (
        277,
        crate::anim::ATTACK2,
        4,
        Bay {
            right: -98_342,
            fwd: -4_178,
            dz: 50,
        },
    ),
    (
        277,
        crate::anim::ATTACKWALK,
        4,
        Bay {
            right: -98_342,
            fwd: -4_178,
            dz: 50,
        },
    ),
    // **Piece 33, the Javelineers (type 83): the original's own
    // `get_position`** (item 1194). Its `Spear` leaves node 0 on frame 12
    // of `CHAR_ATTACK1`, 6 of `CHAR_ATTACK2` and 27 of `CHAR_ATTACK3`.
    // Without these rows every javelin left the unit's own square, some
    // 80 units behind the hand, and flew one frame long: Great Sahara's
    // long word, 16681, where a Farm the original had lost a tick
    // earlier drew one more Longbow volley here. run448 prints two
    // rounds under `AMMO=5`, and its packet answers
    // `GraphicPieces::get_position@0090b750(33, 0, anim, frame, deg)`
    // under unicorn at every whole degree: each row reproduces all 360
    // truncated points exactly, and `dz` is the same at every degree.
    (
        33,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 38_941,
            fwd: 99_188,
            dz: 163,
        },
    ),
    (
        33,
        crate::anim::ATTACK2,
        6,
        Bay {
            right: 1_649,
            fwd: 89_737,
            dz: 170,
        },
    ),
    (
        33,
        crate::anim::ATTACK3,
        27,
        Bay {
            right: 21_769,
            fwd: 74_413,
            dz: 156,
        },
    ),
    // **Piece 273, the Bark (type 323), and piece 567, the Catapult (type
    // 265): the original's own `get_position`** (item 2304, `docs/GOLDEN.md`
    // §83). Neither pivots (`turret_angles` and `node_flags` read 0 on every
    // round), so the round takes the non-pivot arm, and without a row it left
    // the figure's own square: the Bark's 52 under its release and up to
    // 110 units off, the Catapult's 245 under and 33 off (parked 2206,
    // 2266). run839's packet (chapter sixty-four's game) holds the Bark's
    // nine `AttachPos` entries — node 0 at frames 2, 10 and 17 of
    // `CHAR_ATTACKWALK`, `CHAR_ATTACK1` and `CHAR_ATTACK2`, one point a
    // frame across the three — and run842's (chapter sixty-five's) the
    // Catapult's one, node 0 at frame 12 of `CHAR_ATTACK2`. Each row is the
    // model point times the piece's scale (`guy_scale` × `RData +0x88`:
    // 4.8 × 0.78 for the Bark, 4.8 for the Catapult), `right = −s·x` and
    // `fwd = −s·y` in thousandths, and sits inside the region that
    // reproduces `get_position`'s truncated answer at all 256 headings
    // under unicorn.
    (
        273,
        crate::anim::ATTACKWALK,
        2,
        Bay {
            right: -150,
            fwd: 67_991,
            dz: 52,
        },
    ),
    (
        273,
        crate::anim::ATTACKWALK,
        10,
        Bay {
            right: 1_110,
            fwd: -110_298,
            dz: 52,
        },
    ),
    (
        273,
        crate::anim::ATTACKWALK,
        17,
        Bay {
            right: -1_724,
            fwd: -26_208,
            dz: 52,
        },
    ),
    (
        273,
        crate::anim::ATTACK1,
        2,
        Bay {
            right: -150,
            fwd: 67_991,
            dz: 52,
        },
    ),
    (
        273,
        crate::anim::ATTACK1,
        10,
        Bay {
            right: 1_110,
            fwd: -110_298,
            dz: 52,
        },
    ),
    (
        273,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: -1_724,
            fwd: -26_208,
            dz: 52,
        },
    ),
    (
        273,
        crate::anim::ATTACK2,
        2,
        Bay {
            right: -150,
            fwd: 67_991,
            dz: 52,
        },
    ),
    (
        273,
        crate::anim::ATTACK2,
        10,
        Bay {
            right: 1_110,
            fwd: -110_298,
            dz: 52,
        },
    ),
    (
        273,
        crate::anim::ATTACK2,
        17,
        Bay {
            right: -1_724,
            fwd: -26_208,
            dz: 52,
        },
    ),
    // **Piece 274, the Dromon (type 324)** (item 3616, `docs/AI.md` §327):
    // the Bark's art file, `Dromon Attack`, on another model. Its nine
    // `AttachPos` rows (run1217's packet; run959's and run912e's hold the
    // same) are node 0 at frames 2, 10 and 17 of anims 10, 11 and 12, the
    // Bark's points with `y` 31.88 model units lower, 119 units further
    // forward on the water — (0.04,
    // −50.04, 14.03), (−0.296, −2.42, 14.03), (0.460, −24.88, 14.02) — at
    // the Bark's scale, 4.8 × 0.78. Each row reproduces
    // `get_position`'s truncated answer at all 361 whole degrees under
    // unicorn; frame 2's region is three lattice points, and this is its
    // middle. Without them the Dromon `1/24`'s first round at the Farm
    // `0/2004` left the hull's own (2808, 6568) and flew 13 frames where
    // theirs left (2808, 6381) and flew 12: the Inca Classical word, 4501.
    (
        274,
        crate::anim::ATTACKWALK,
        2,
        Bay {
            right: -148,
            fwd: 187_348,
            dz: 52,
        },
    ),
    (
        274,
        crate::anim::ATTACKWALK,
        10,
        Bay {
            right: 1_110,
            fwd: 9_061,
            dz: 52,
        },
    ),
    (
        274,
        crate::anim::ATTACKWALK,
        17,
        Bay {
            right: -1_724,
            fwd: 93_151,
            dz: 52,
        },
    ),
    (
        274,
        crate::anim::ATTACK1,
        2,
        Bay {
            right: -148,
            fwd: 187_348,
            dz: 52,
        },
    ),
    (
        274,
        crate::anim::ATTACK1,
        10,
        Bay {
            right: 1_110,
            fwd: 9_061,
            dz: 52,
        },
    ),
    (
        274,
        crate::anim::ATTACK1,
        17,
        Bay {
            right: -1_724,
            fwd: 93_151,
            dz: 52,
        },
    ),
    (
        274,
        crate::anim::ATTACK2,
        2,
        Bay {
            right: -148,
            fwd: 187_348,
            dz: 52,
        },
    ),
    (
        274,
        crate::anim::ATTACK2,
        10,
        Bay {
            right: 1_110,
            fwd: 9_061,
            dz: 52,
        },
    ),
    (
        274,
        crate::anim::ATTACK2,
        17,
        Bay {
            right: -1_724,
            fwd: 93_151,
            dz: 52,
        },
    ),
    (
        567,
        crate::anim::ATTACK2,
        12,
        Bay {
            right: 0,
            fwd: -33_219,
            dz: 245,
        },
    ),
    // **Piece 215, the Catapult's default style** (item 3662, `docs/AI.md`
    // §342): run1326's packet (the British Classical lobby, logger frame
    // 4567) holds its one attack `AttachPos` row, piece 567's own — node 0
    // at frame 12 of `CHAR_ATTACK2`, `RData +0x88` 1.0 — and
    // `tools/recomp/fit_bay.py` fits its sweep in 435 lattice points that
    // hold 567's row, 361 of 361 headings. Without it the British
    // Catapults' rounds in run1326b left their square 245 under.
    (
        215,
        crate::anim::ATTACK2,
        12,
        Bay {
            right: 0,
            fwd: -33_219,
            dz: 245,
        },
    ),
    // **Piece 1794, the Peltasts in the Persians' art style** (`34 + 5 ×
    // 0x160`, type 84; item 3555, `docs/AI.md` §308): run1007b's
    // `RON_STATE_FRAME=15915` packet on the Indian Great Sahara game holds
    // its three attack `AttachPos` rows — piece 34's keys, node 0 — and
    // `get_position.py`'s sweep of each is reproduced at all 361 headings by
    // these, each the centroid of its exact region (229, 895 and 1047
    // lattice points). Without them the Persian javelins left the figure's
    // own square: `1/22`'s at the Nubian city centre `0/2000` flew 14
    // frames against 13 and struck on 4048 where run1189b shows 4047.
    (
        1794,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 38_946,
            fwd: 60_202,
            dz: 163,
        },
    ),
    (
        1794,
        crate::anim::ATTACK2,
        6,
        Bay {
            right: 1_685,
            fwd: 50_747,
            dz: 170,
        },
    ),
    (
        1794,
        crate::anim::ATTACK3,
        27,
        Bay {
            right: 21_762,
            fwd: 35_444,
            dz: 156,
        },
    ),
    // **Piece 1793, the Javelineers in the Persians' art style** (`33 + 5 ×
    // 0x160`, type 83; item 3663, `docs/AI.md` §343): run1327's
    // `RON_STATE_FRAME=10930` packet on the Persian Great Lakes game holds
    // its three attack `AttachPos` rows — piece 33's keys, node 0, `y` 8.12
    // further forward — and `get_position.py`'s sweep of each is reproduced
    // at all 361 headings by these, the centroids of the same exact regions
    // as piece 1794's (229, 895 and 1047 lattice points). Without them
    // `1/312`'s javelin at the Citizen `0/12` left the figure's own square
    // (5064, 31368) on 10931 where run1318b's leaves (5054, 31417), 170 up.
    (
        1793,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 38_946,
            fwd: 60_202,
            dz: 163,
        },
    ),
    (
        1793,
        crate::anim::ATTACK2,
        6,
        Bay {
            right: 1_685,
            fwd: 50_747,
            dz: 170,
        },
    ),
    (
        1793,
        crate::anim::ATTACK3,
        27,
        Bay {
            right: 21_762,
            fwd: 35_444,
            dz: 156,
        },
    ),
    // **Piece 1090, the Peltasts in the Asian art style** (`34 + 3 ×
    // 0x160`, type 84, the Koreans' Elite Javelineers; item 3666,
    // `docs/AI.md` §345): run991d's `RON_STATE_FRAME=15256` packet on the
    // Korean Great Sahara game holds its three attack `AttachPos` rows —
    // piece 34's keys, node 0 — and `get_position.py`'s sweep of each is
    // reproduced at all 361 headings by these, the centroids of the same
    // exact regions as piece 1794's (229, 895 and 1047 lattice points).
    // Without them `1/47`'s javelin at the Citizen `0/13` left the figure's
    // own square (6984, 30312) on 3354 where run1331b's leaves (6934,
    // 30317), 170 up.
    (
        1090,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 38_946,
            fwd: 60_202,
            dz: 163,
        },
    ),
    (
        1090,
        crate::anim::ATTACK2,
        6,
        Bay {
            right: 1_685,
            fwd: 50_747,
            dz: 170,
        },
    ),
    (
        1090,
        crate::anim::ATTACK3,
        27,
        Bay {
            right: 21_762,
            fwd: 35_444,
            dz: 156,
        },
    ),
    // **Piece 1881, the Archers in the Persians' art style** (`121 + 5 ×
    // 0x160`, type 171; item 3663, `docs/AI.md` §343): run1327's packet
    // holds its three attack `AttachPos` rows on piece 121's keys, node 0,
    // and `get_position.py`'s sweep of each is reproduced at all 361
    // headings by these, each the centroid of its exact region (221, 277
    // and 124 lattice points). Without them the Persian arrows left the
    // figure's own square: `1/299`'s at the Citizen `0/11` from (5592,
    // 30984) on 10980 where run1318b's leaves (5528, 30923), 16 frames
    // against 15, and `0/11` went unstruck on 10993.
    (
        1881,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        1881,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        1881,
        crate::anim::ATTACK3,
        15,
        Bay {
            right: 9_711,
            fwd: 66_508,
            dz: 164,
        },
    ),
    // **Piece 113, the Anusiya** (type 163, the Persians' own archers; item
    // 3663, `docs/AI.md` §343): run1327's packet holds its two attack
    // `AttachPos` rows, node 0, and `get_position.py`'s sweep of each is
    // reproduced at all 361 headings by these, the centroids of their
    // exact regions (105 and 115 lattice points). Without them the
    // Anusiya loosed from the figure's own square: `1/333`'s arrow at the
    // human's city centre `0/2000` on 11381 left (3912, 31080), 178 under
    // run1327b's (3821, 31024), in 3 frames against 2.
    (
        113,
        crate::anim::ATTACK2,
        14,
        Bay {
            right: 25_067,
            fwd: 104_603,
            dz: 178,
        },
    ),
    (
        113,
        crate::anim::ATTACK3,
        6,
        Bay {
            right: 27_678,
            fwd: 36_793,
            dz: 171,
        },
    ),
    // **Piece 1975, the Catapult in the Persians' art style** (`215 + 5 ×
    // 0x160`, type 265; item 3663): run1327's packet holds its one
    // `AttachPos` row, node 0 at frame 12 of `CHAR_ATTACK2`, and its sweep
    // admits piece 567's row (435 lattice points about it). Without it
    // `1/119`'s stone at `0/2000` on 11511 left the carriage's own point,
    // (6264, 32904) at 208, where run1327b's leaves (6291, 32922) at 453.
    (
        1975,
        crate::anim::ATTACK2,
        12,
        Bay {
            right: 0,
            fwd: -33_219,
            dz: 245,
        },
    ),
    // **Piece 212, the Mahout** (type 262, `mjvyk`; item 3555): the same
    // packet's five node-1 rows, the howdah's archer, one per attack slot
    // at frame 16 (`CHAR_ATTACKWALK` the fire on the move's), each the
    // centroid of its exact region over all 361 headings (645, 280, 694,
    // 849 and 742 lattice points). The piece's node-4 rows are another
    // node's; every Mahout release in run1189b's window names node 1.
    (
        212,
        crate::anim::ATTACKWALK,
        16,
        Bay {
            right: -29_938,
            fwd: 87_586,
            dz: 448,
        },
    ),
    (
        212,
        crate::anim::ATTACK1,
        16,
        Bay {
            right: -291,
            fwd: 71_525,
            dz: 451,
        },
    ),
    (
        212,
        crate::anim::ATTACK2,
        16,
        Bay {
            right: -17_740,
            fwd: -7_348,
            dz: 465,
        },
    ),
    (
        212,
        crate::anim::ATTACK3,
        16,
        Bay {
            right: -18_234,
            fwd: 60_948,
            dz: 463,
        },
    ),
    (
        212,
        crate::anim::ATTACKSPECIAL,
        16,
        Bay {
            right: -18_369,
            fwd: 79_421,
            dz: 452,
        },
    ),
    // **Piece 213, the Gun Mahout** (type 263, `mahoutmusket.bh3`; item
    // 3713, `docs/AI.md` §358.4): run1385p's packet on the Persian
    // Gunpowder lobby holds its five node-1 rows (run1007b's carries the
    // same floats), the musket on the howdah, one per attack slot at the
    // file's release frame (833 → 12, 999 → 14), each the centroid of its
    // exact region over all 361 headings (186, 68, 589, 94 and 1639
    // lattice points). Piece 212's rows are the Mahout's archer, another
    // model. Without them `1/99`'s round at the human's `0/11` on 4907 left
    // the elephant's own square, (6921, 8712) at 273, where run1387's
    // leaves (6929, 8608) at 699, and landed a frame late.
    (
        213,
        crate::anim::ATTACKWALK,
        14,
        Bay {
            right: -18_396,
            fwd: 103_327,
            dz: 426,
        },
    ),
    (
        213,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 9_808,
            fwd: 94_776,
            dz: 440,
        },
    ),
    (
        213,
        crate::anim::ATTACK2,
        12,
        Bay {
            right: 48,
            fwd: 63_526,
            dz: 422,
        },
    ),
    (
        213,
        crate::anim::ATTACK3,
        12,
        Bay {
            right: -1_026,
            fwd: 87_367,
            dz: 449,
        },
    ),
    (
        213,
        crate::anim::ATTACKSPECIAL,
        14,
        Bay {
            right: -16_035,
            fwd: 89_982,
            dz: 506,
        },
    ),
    // **Piece 43, the Bantu Umpakati** (type 93; item 3591, `docs/AI.md`
    // §315.2): run1238c's `RON_STATE_FRAME=5021` packet on the Bantu East
    // Indies game holds its three attack `AttachPos` rows — the
    // Javelineer's animations and keys, node 0, on the piece's own model —
    // and `get_position.py`'s sweep of each is reproduced at all 361
    // headings by these, each the centroid of its exact region (42, 229 and
    // 95 lattice points). Without them the Umpakati's javelins left the
    // figure's own square: `1/86`'s at the Citizen `0/11` flew 16 frames
    // from (6792, 8520) where run1238b's round leaves (6799, 8469) and flies
    // 15, and struck on 5037 where theirs struck on 5035.
    (
        43,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 33_637,
            fwd: 65_607,
            dz: 185,
        },
    ),
    (
        43,
        crate::anim::ATTACK2,
        6,
        Bay {
            right: -13_157,
            fwd: 50_004,
            dz: 187,
        },
    ),
    (
        43,
        crate::anim::ATTACK3,
        27,
        Bay {
            right: 14_497,
            fwd: 40_695,
            dz: 173,
        },
    ), // **Piece 44, the Yangombi Umpakati** (type 94): the same packet's
    // three rows carry piece 43's floats bit for bit at the same scale, and
    // the sweeps are identical at all 361 headings. Without them who=1's
    // Yangombi `1/90` threw at the Citizen `0/10` on 5083 from its own
    // square (5592, 8520), 17 frames, where theirs struck on 5098.
    (
        44,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: 33_637,
            fwd: 65_607,
            dz: 185,
        },
    ),
    (
        44,
        crate::anim::ATTACK2,
        6,
        Bay {
            right: -13_157,
            fwd: 50_004,
            dz: 187,
        },
    ),
    (
        44,
        crate::anim::ATTACK3,
        27,
        Bay {
            right: 14_497,
            fwd: 40_695,
            dz: 173,
        },
    ),
    // **Piece 473, the Archers** (type 171; item 3592, `docs/AI.md`
    // §315.3): run1238c's packet holds its three attack `AttachPos` rows,
    // node 0 at 472's keys and with 472's floats bit for bit, and
    // `tools/recomp/fit_bay.py` on `get_position.py`'s sweep of each finds
    // one region (221, 277 and 124 lattice points); the rows are their
    // centroids. 472's planar `Node`s are not exact on the sweep and
    // piece 120's `Bay`s miss 28 to 124 headings, so the piece is its
    // own. Without them who=1's Archers loosed from the figure's square:
    // `1/41` at the farmer `0/3` on 5306 from (5688, 8280), 19 frames.
    (
        473,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        473,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        473,
        crate::anim::ATTACK3,
        15,
        Bay {
            right: 9_711,
            fwd: 66_508,
            dz: 164,
        },
    ),
    // **Piece 824, the Maya's Bowmen** (`BOWMEN-AMERICAN`, type 170, `120 +
    // 2 × 0x160`; item 3600, `docs/AI.md` §320.3): run1247's
    // `RON_STATE_FRAME=4306` packet on the Maya East Indies game holds its
    // four `AttachPos` rows — node 0 at anims 11, 12 and 13, times 12, 9
    // and 15, piece 120's keys, and a `CHAR_DIE` row (anim 4) no round
    // reads — and `tools/recomp/fit_bay.py` on `get_position.py`'s sweep of
    // each finds piece 473's regions to the point (221, 277 and 124
    // lattice points), so the rows are 473's centroids. Without them the
    // Bowmen `1/46`'s arrow at the fleeing Citizen `0/10`, loosed on 4318,
    // left the unit's own square (6552, 9240) and flew 21 frames.
    (
        824,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        824,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        824,
        crate::anim::ATTACK3,
        15,
        Bay {
            right: 9_711,
            fwd: 66_508,
            dz: 164,
        },
    ),
    // **Piece 825, the Inca Archers** (`ARCHERS`, type 171; item 3617,
    // `docs/AI.md` §327.2): run1266c's packet holds its four `AttachPos`
    // rows at scale 1.0 — node 0 at anims 11, 12 and 13, times 12, 9 and
    // 15, and a `CHAR_DIE` row (anim 4) no round reads — and
    // `tools/recomp/fit_bay.py` on `get_position.py`'s sweep of each finds
    // piece 473's regions to the point (221, 277 and 124 lattice points),
    // so the rows are 473's centroids. Without them who=1's Archers loosed
    // from the figure's own square, a frame long: `1/58` at the Scout `0/0`
    // on 5327 from (7224, 8952), 18 frames where theirs left (7278, 8882)
    // and flew 17, and `0/0` was struck a frame late.
    (
        825,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        825,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        825,
        crate::anim::ATTACK3,
        15,
        Bay {
            right: 9_711,
            fwd: 66_508,
            dz: 164,
        },
    ),
    // **Piece 1176, the Japanese Bowmen** (`BOWMEN`, type 170, `120 + 3 ×
    // 0x160`; item 3610, `docs/AI.md` §323): run1264b's
    // `RON_STATE_FRAME=4919` packet on the Japanese Ancient East Indies
    // game holds its four `AttachPos` rows at scale 0.98 — node 0 at
    // piece 120's keys, and a `CHAR_DIE` row no round reads — and
    // `tools/recomp/fit_bay.py` on each key's sweep finds one region (321,
    // 274 and 332 lattice points); the rows are their centroids, and the
    // heights 160, 160 and 161 are the sweep's and the dump's `sz − z`.
    // Without them the Bowmen `1/50`'s arrow at the Citizen `0/6` left the
    // unit's own square (7608, 5784) and flew 17 frames where theirs left
    // (7529, 5749) and flew 16, and `0/6` was struck a frame late.
    (
        1176,
        crate::anim::ATTACK1,
        12,
        Bay {
            right: -1_790,
            fwd: 86_974,
            dz: 160,
        },
    ),
    (
        1176,
        crate::anim::ATTACK2,
        9,
        Bay {
            right: 331,
            fwd: 85_656,
            dz: 160,
        },
    ),
    (
        1176,
        crate::anim::ATTACK3,
        15,
        Bay {
            right: 9_531,
            fwd: 65_185,
            dz: 161,
        },
    ),
    // **Piece 1178, the Crossbowmen in the Asian art style** (`CROSSBOWMEN`,
    // type 172, `122 + 3 × 0x160`; item 3712, `docs/AI.md` §362.2): the
    // Mongols' Crossbowman. run1296's `RON_STATE_FRAME` packet (and run1294's,
    // run1237's and run1220b's, every float the same) holds its three
    // `AttachPos` rows at scale 1.04, node 0 at piece 122's keys, and
    // `tools/recomp/fit_bay.py` on each key's sweep finds one region (1177,
    // 491 and 223 lattice points); the rows are their centroids, and the
    // heights 173, 140 and 168 are the sweep's and run1386's `sz − z`.
    // Piece 122's planar [`MEASURED`] rows are the default style's and never
    // reached it. Without these who=1's `1/43` loosed its bolt of 3981 at the
    // human's `0/2004` from its own square (6120, 11208, 19) against the
    // original's (6103, 11142, 159), flew 11 frames against 10, and the
    // building was struck a frame late.
    (
        1178,
        crate::anim::ATTACK1,
        5,
        Bay {
            right: 11_856,
            fwd: 87_864,
            dz: 173,
        },
    ),
    (
        1178,
        crate::anim::ATTACK2,
        3,
        Bay {
            right: 15_464,
            fwd: 67_070,
            dz: 140,
        },
    ),
    (
        1178,
        crate::anim::ATTACK3,
        6,
        Bay {
            right: 15_879,
            fwd: 78_459,
            dz: 168,
        },
    ),
];

/// `(piece, animation slot, node)` → bay, for a piece whose every event
/// on a node leaves from the same point — **piece 239, the Fighter's two
/// guns** (item 853, `docs/ORDERS.md` §39.3). `FIGHTER`'s `CHAR_ATTACK2`
/// fires its eight frames on node 0 and on node 1, and run265 dumps all
/// 36 of its rounds on the Barracks (first printed on blocks 924–1076):
/// each row is the centre of the region that reproduces every round of
/// its node to the unit, from the `GUY` record on the round's own first
/// block, at eleven whole-degree headings between 94 and 255. Node 0 is
/// the left gun and node 1 the right; both leave one unit over the
/// figure.
const GUNS: &[(i32, i8, i8, Bay)] = &[
    // **Piece 289, the Attack Submarine's two tubes** (`ATTACKSUB`, unit
    // type 339; item 3849, `docs/GOLDEN.md` §153.1): run1484's packet
    // holds the piece's two `AttachPos` entries, `RData +0x88` 0.9 — node
    // 1 at frame 1 and node 2 at frame 4 of `CHAR_ATTACK2`, (∓5.33,
    // −80.29, −43.63) — and `fit_bay.py` fits each sweep (478 lattice
    // points, 361 of 361 headings; the centroid is kept). The torpedo
    // leaves 347 ahead and 23 to a side, **188 under the water**:
    // run1484's `2/15` rounds of 669 and 672 print `sz` −188, where ours
    // left from the hull's square at 0.
    (
        289,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: 23_033,
            fwd: 346_844,
            dz: -188,
        },
    ),
    (
        289,
        crate::anim::ATTACK2,
        2,
        Bay {
            right: -23_033,
            fwd: 346_844,
            dz: -188,
        },
    ),
    (
        239,
        crate::anim::ATTACK2,
        0,
        Bay {
            right: -74_266,
            fwd: 39_505,
            dz: 1,
        },
    ),
    (
        239,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: 73_493,
            fwd: 39_462,
            dz: 1,
        },
    ),
    // **Piece 237, the Biplane's two guns** (item 1200, `docs/GOLDEN.md`
    // §50): run437 dumps 26 of its rounds, thirteen a node, at eleven
    // whole-degree headings between 101 and 291 (blocks 965–1614). Each
    // row is the centre of the region that reproduces every round of its
    // node to the unit — `right` −13,120..−12,900 and `fwd`
    // 101,150..101,390 for node 0, `right` 11,260..11,420 and `fwd`
    // 101,180..101,280 for node 1 — and both leave 39 over the figure.
    // Nose guns a tile's half ahead: from the figure's own point the
    // second pass's pair on 966 flew six frames where the original's flew
    // five.
    (
        237,
        crate::anim::ATTACK2,
        0,
        Bay {
            right: -13_010,
            fwd: 101_270,
            dz: 39,
        },
    ),
    (
        237,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: 11_340,
            fwd: 101_230,
            dz: 39,
        },
    ),
    // **Piece 245, the Jet Fighter's two `AARocket`s** (`JETFIGHTER`, unit
    // type 295; item 3760, `docs/AI.md` §372.4): run1437's
    // `RON_STATE_FRAME=3894` packet on the American Information Great
    // Lakes seed loads its two `AttachPos` rows (`CHAR_ATTACK2`, node 0 at
    // frame 9 and node 1 at frame 4), `RData +0x88` 0.654, and `fit_bay.py`
    // fits each sweep (152 and 236 lattice points, 361 of 361 headings;
    // the centroid is kept): one under the wings and behind them, 36 below
    // the figure. Without them who=1's strafing `1/57` loosed its rounds of
    // 4225 and 4230 from its own square, (4703, 30220, 2127) against the
    // original's (4889, 30343, 2091), and the second's splash on 4242 took
    // building `0/2006`'s `damage_frac` to 8 against 11.
    (
        245,
        crate::anim::ATTACK2,
        0,
        Bay {
            right: 184_080,
            fwd: -126_691,
            dz: -36,
        },
    ),
    (
        245,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: -184_152,
            fwd: -126_707,
            dz: -36,
        },
    ),
    // **Piece 258, the Fighter Bomber's two wing guns** (`FIGHTERBOMBER`,
    // unit type 308; item 3829, `docs/GOLDEN.md` §152.12): run1472's
    // `RON_STATE_FRAME=707` packet loads sixteen `AttachPos` rows, `RData
    // +0x88` 0.765, all in `CHAR_ATTACK2` at frames 1, 3, 7, 11, 14, 18, 22
    // and 26 — node 0 at (29.13, −9.34, −7.27) and node 1 at (−29.23,
    // −9.66, −7.27) on every one — and `fit_bay.py` fits each node's sweep
    // at 361 of 361 headings (202 and 513 lattice points; the centroid is
    // kept), 26 below the figure. Without them who=0's Fighter Bomber `0/15`
    // loosed both rounds of each pair from its square at the figure's
    // height, (15096, 15672, 0), where run1477's block 822 prints (15179,
    // 15597, −26) and (15074, 15782, −26), `total_time` 7 and 8 against
    // ours' 8 and 8.
    (
        258,
        crate::anim::ATTACK2,
        0,
        Bay {
            right: -106_967,
            fwd: 34_296,
            dz: -26,
        },
    ),
    (
        258,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: 107_331,
            fwd: 35_471,
            dz: -26,
        },
    ),
    // **Piece 259, the Jet Fighter Bomber's two `AARocket`s**
    // (`JETFIGHTERBOMBER`, unit type 309; item 3829, `docs/GOLDEN.md`
    // §152.12, parked 3816a): the same packet loads its two rows (`CHAR_
    // ATTACK2`, node 0 at frame 9, (−50.29, 36.08, −19.34), and node 1 at
    // frame 4, (50.54, 36.08, −19.31)), `RData +0x88` 0.765, and
    // `fit_bay.py` fits each sweep at 361 of 361 headings (168 and 367
    // lattice points; the centroid is kept), 71 and 70 below the figure.
    // Without them who=2's `2/18` loosed its rounds from its square at the
    // figure's height, (15864, 16632, 0), where run1468's block 801 prints
    // (15843, 16406, −70) and (15657, 16725, −71).
    (
        259,
        crate::anim::ATTACK2,
        0,
        Bay {
            right: 184_664,
            fwd: -132_488,
            dz: -71,
        },
    ),
    (
        259,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: -185_591,
            fwd: -132_491,
            dz: -70,
        },
    ),
    // **Piece 255, the Strategic Bomber's two bays** (`STRATEGICBOMBER`,
    // unit type 305; item 3760, `docs/AI.md` §372.4): run1437's packet loads
    // twelve `AttachPos` rows, `RData +0x88` 0.9, all in `CHAR_ATTACK2` — six
    // on node 0 (frames 1, 2, 4, 7, 9, 11) and six on node 1 (1, 3, 5, 8,
    // 10, 13), each node one point at all of its frames (the sweeps at the
    // first and last frame agree on every heading) — and `fit_bay.py` fits
    // each (149 and 884 lattice points, 361 of 361 headings; the centroid
    // is kept), level with the figure. Without them who=1's `1/45` dropped
    // its bombs of 4270 from its own square, (3717, 30596) against the
    // original's (3805, 30661) and (3732, 30489).
    (
        255,
        crate::anim::ATTACK2,
        0,
        Bay {
            right: -93_878,
            fwd: -57_148,
            dz: 0,
        },
    ),
    (
        255,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: 93_885,
            fwd: -54_260,
            dz: 0,
        },
    ),
    // **Piece 589, the same two guns on the human Biplane's graph** (item
    // 2693, `docs/GOLDEN.md` §99): run875's rounds leave from the plane's
    // own `GUY` point plus the 237 rows' vector, node for node — the
    // figure at (37463, 16578), angle −590,544,896, its node 0 at
    // (37377, 16525) and node 1 at (37392, 16506), 39 over the figure.
    (
        589,
        crate::anim::ATTACK2,
        0,
        Bay {
            right: -13_010,
            fwd: 101_270,
            dz: 39,
        },
    ),
    (
        589,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: 11_340,
            fwd: 101_230,
            dz: 39,
        },
    ),
    // **Piece 261, the Attack Helicopter's two rocket pods** (item 3804,
    // `docs/GOLDEN.md` §152.3): `ATTACKHELICOPTER`'s `CHAR_ATTACK2` fires
    // a `SmallRocket` on node 0 at 0 ms and on node 1 at 500. run1469's
    // `2/15` (`TI` 311) at (13560, 16056), angle 0, `z` 330, releases its
    // node 0 round from (13488, 16015, 371) on 627 and its node 1 round
    // from (13638, 16029, 371) on 633. **One heading only**: each row is
    // the centre of the unit square that heading truncates to, so the
    // vector's turn at any other heading is not measured.
    (
        261,
        crate::anim::ATTACK2,
        0,
        Bay {
            right: -72_500,
            fwd: 41_500,
            dz: 41,
        },
    ),
    (
        261,
        crate::anim::ATTACK2,
        1,
        Bay {
            right: 78_500,
            fwd: 27_500,
            dz: 41,
        },
    ),
];

/// The bay a piece releases from, or `None` when none is measured: a
/// [`BAYS`] row by the event's frame, or a [`GUNS`] row by its node.
pub fn bay(gpiece: i32, anim: i8, starttime: u32, node: i8) -> Option<Bay> {
    BAYS.iter()
        .find(|(p, a, t, _)| *p == gpiece && *a == anim && *t == starttime)
        .map(|(_, _, _, b)| *b)
        .or_else(|| {
            GUNS.iter()
                .find(|(p, a, n, _)| *p == gpiece && *a == anim && *n == node)
                .map(|(_, _, _, b)| *b)
        })
}

/// The release height over the figure, from whichever table measures it.
pub fn release_dz(gpiece: i32, anim: i8, starttime: u32, node: i8) -> Option<i32> {
    bay(gpiece, anim, starttime, node)
        .map(|b| b.dz)
        .or_else(|| self::node(gpiece, anim, starttime).map(|n| n.dz))
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
pub fn launch_point(
    pos: Pos,
    facing: Angle,
    gpiece: i32,
    anim: i8,
    starttime: u32,
    node: i8,
) -> Pos {
    if let Some(b) = bay(gpiece, anim, starttime, node) {
        return b.point(pos, facing);
    }
    match self::node(gpiece, anim, starttime) {
        Some(n) => point(pos, facing, n),
        None => pos,
    }
}

#[cfg(test)]
mod tests;
