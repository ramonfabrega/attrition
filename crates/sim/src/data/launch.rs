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
//! The initial Longbowman rows came from nine arrows in run109. Item 4365
//! measures each animation socket across all whole-degree headings and
//! uses the Cartesian rotation in [`Bay::point`]. [`MEASURED`] holds the
//! remaining polar estimates from captures without a full socket sweep.

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
/// The piece key is [`crate::units::anim::Guy::gpiece`] — the same number the
/// original's own `GUY` record prints, and the key `PieceReleases` is
/// built on (`PieceName::piece`). **127 is `LONGBOWMEN`**, whose unit
/// type is 177: the piece is the graph index and the type is
/// `0x32 + it`, and the two are one apart by a constant only because
/// this graph happens to be its own type's. The starttime key is the one
/// `rondata::artdata::release_frame` produces from the install's
/// `<RELEASEEVENT starttime>`, and its six Cartesian rows in [`BAYS`] are the six
/// that file gives the Longbowman.
///
/// A piece with no row here launches from the unit's own position, which
/// is what every shot in this crate did before run109.
const MEASURED: &[(i32, i8, u32, Node)] = &[
    // Piece 122, the Crossbowmen (item 3347's three rows off run982, one bolt
    // each), is [`BAYS`] since item 4517: the original's sweep at every
    // heading, where the planar rows stood a unit off at row 75's 4004.
    // Piece 842, the Horse Archer (item 3338), left for `pivot::SAME_AS`
    // (item 3658): its `AttachPos` rows are piece 138's, a pivot piece's, so
    // its rounds turn with the rider's turret.
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
        crate::units::anim::ATTACK2,
        9,
        Node {
            bearing: -17_683_648,
            radius: 86,
            dz: 164,
        },
    ),
    (
        472,
        crate::units::anim::ATTACK1,
        12,
        Node {
            bearing: -37_883_648,
            radius: 89,
            dz: 163,
        },
    ),
    (
        472,
        crate::units::anim::ATTACK3,
        15,
        Node {
            bearing: 81_516_352,
            radius: 67,
            dz: 164,
        },
    ),
    // Piece 384's three rows, `(384, ATTACK2, 22)` and its `ATTACK1` and
    // `ATTACK3` kin (the last item 4256's, §15.1 of `docs/CLIENT_INPUT.md`),
    // are [`BAYS`] since item 4277: the planar one-shot fits stood two
    // units off the original at the buildmasked game's facings.
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
/// to `get_position` ([`crate::data::pivot::release_rotation`]), turned back by
/// the half turn — `Bay`'s axes are the figure's, not the flipped ones.
///
/// **A half-way byte rounds up** (item 2304): `byte ≡ 16 mod 32`, 22.5°
/// and its kin, is read by `angle_to_degrees`, which rounds a half degree
/// early (`docs/COMBAT.md` §52). This was taken to the even degree while no
/// measured heading sat on one; run835's Bark `0/7` faces byte 80 (112.5°)
/// on blocks 638–653, and only 113 puts its rounds at the dump's points.
pub fn whole_degrees(facing: Angle) -> i64 {
    i64::from((crate::data::pivot::release_rotation(facing) + 180) % 360)
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
    // Longbowman, item 4365. All six node-0 animation events of the
    // Great Lakes state-9746 snapshot, each fitted over 361 headings.
    // The two ATTACK1 events and ATTACK2 at 22 share the same socket.
    (
        127,
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: -3_362,
            fwd: 82_391,
            dz: 169,
        },
    ),
    (
        127,
        crate::units::anim::ATTACK1,
        22,
        Bay {
            right: -3_362,
            fwd: 82_391,
            dz: 169,
        },
    ),
    (
        127,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: -15_902,
            fwd: 81_966,
            dz: 137,
        },
    ),
    (
        127,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: -3_362,
            fwd: 82_391,
            dz: 169,
        },
    ),
    (
        127,
        crate::units::anim::ATTACK3,
        10,
        Bay {
            right: 10_522,
            fwd: 70_329,
            dz: 185,
        },
    ),
    (
        127,
        crate::units::anim::ATTACK3,
        24,
        Bay {
            right: 4_877,
            fwd: 65_764,
            dz: 184,
        },
    ),
    // **Piece 384, the Slingers** (item 4277, `docs/CLIENT_INPUT.md` §25):
    // run1615b's packet (the buildmasked file at logger frame 6869) holds
    // the piece's three `AttachPos` rows, node 0 at frame 21 of
    // `CHAR_ATTACK1`, 22 of `CHAR_ATTACK2` and 16 of `CHAR_ATTACK3`, `RData
    // +0x88` 1.0; `get_position.py … sweep 384 0 <anim> <time>` and
    // `fit_bay.py` fit each (1696, 273 and 592 lattice points, 361 of 361
    // headings; the centroid is kept). Until then they were planar rows in
    // [`MEASURED`], solved from chapter two's stones and run1603's one, which
    // put `0/9`'s stone of 6868 at (3391, 32213) where the packet's leaves
    // (3389, 32212): 750.3 from its mark rather than 749.5, 5 frames at
    // speed 150 rather than 4.
    (
        384,
        crate::units::anim::ATTACK1,
        21,
        Bay {
            right: 69_764,
            fwd: 67_823,
            dz: 178,
        },
    ),
    (
        384,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: 41_238,
            fwd: 117_982,
            dz: 151,
        },
    ),
    (
        384,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: -42_142,
            fwd: 113_256,
            dz: 177,
        },
    ),
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
        crate::units::anim::ATTACK1,
        15,
        Bay {
            right: 102_456,
            fwd: 44_072,
            dz: 116,
        },
    ),
    (
        125,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: 23_036,
            fwd: 63_394,
            dz: 101,
        },
    ),
    (
        125,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        7,
        Bay {
            right: -99_683,
            fwd: 18_131,
            dz: 51,
        },
    ),
    (
        282,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 32_149,
            fwd: 121_967,
            dz: 124,
        },
    ),
    (
        91,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 34_159,
            fwd: 98_259,
            dz: 147,
        },
    ),
    (
        91,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 31_393,
            fwd: 86_406,
            dz: 74,
        },
    ),
    (
        94,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: 25_178,
            fwd: 75_595,
            dz: 77,
        },
    ),
    (
        94,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        3,
        Bay {
            right: -7_425,
            fwd: 147_384,
            dz: 93,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 88,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -16_551,
            fwd: 144_695,
            dz: 91,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -9_573,
            fwd: 146_738,
            dz: 90,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        20,
        Bay {
            right: 752,
            fwd: 146_479,
            dz: 84,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        24,
        Bay {
            right: 13_866,
            fwd: 144_797,
            dz: 89,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        29,
        Bay {
            right: 18_457,
            fwd: 146_563,
            dz: 86,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        37,
        Bay {
            right: 4_674,
            fwd: 148_076,
            dz: 90,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_935,
            dz: 91,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_376,
            dz: 88,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 88,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -402,
            fwd: 146_341,
            dz: 88,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: -2_732,
            fwd: 145_714,
            dz: 93,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -5_603,
            fwd: 144_577,
            dz: 98,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -7_510,
            fwd: 145_038,
            dz: 103,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -5_569,
            fwd: 143_666,
            dz: 103,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: -3_051,
            fwd: 145_130,
            dz: 103,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 2_021,
            fwd: 145_012,
            dz: 103,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
        3,
        Bay {
            right: 1_011,
            fwd: 144_798,
            dz: 91,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 1_830,
            fwd: 143_956,
            dz: 92,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 11_993,
            fwd: 147_443,
            dz: 85,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
        19,
        Bay {
            right: 18_422,
            fwd: 144_413,
            dz: 88,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
        20,
        Bay {
            right: 18_428,
            fwd: 146_106,
            dz: 85,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
        29,
        Bay {
            right: 1_830,
            fwd: 146_325,
            dz: 90,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 146_847,
            dz: 90,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_400,
            dz: 88,
        },
    ),
    (
        80,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        3,
        Bay {
            right: -7_394,
            fwd: 147_049,
            dz: 100,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 95,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -16_534,
            fwd: 144_463,
            dz: 98,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -9_569,
            fwd: 146_592,
            dz: 97,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        20,
        Bay {
            right: 737,
            fwd: 146_742,
            dz: 91,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        24,
        Bay {
            right: 13_870,
            fwd: 144_718,
            dz: 96,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        29,
        Bay {
            right: 18_498,
            fwd: 146_683,
            dz: 93,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        32,
        Bay {
            right: 18_363,
            fwd: 144_626,
            dz: 90,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        37,
        Bay {
            right: 4_655,
            fwd: 147_954,
            dz: 96,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_756,
            dz: 97,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_320,
            dz: 95,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_206,
            dz: 95,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -404,
            fwd: 146_277,
            dz: 95,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -1_409,
            fwd: 147_815,
            dz: 97,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: -2_741,
            fwd: 145_367,
            dz: 100,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -5_593,
            fwd: 143_864,
            dz: 105,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -7_499,
            fwd: 144_016,
            dz: 109,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -5_575,
            fwd: 142_638,
            dz: 109,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: -3_046,
            fwd: 144_117,
            dz: 110,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 1_993,
            fwd: 143_949,
            dz: 110,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        3,
        Bay {
            right: 2_084,
            fwd: 145_768,
            dz: 98,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 2_759,
            fwd: 144_837,
            dz: 99,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 13_626,
            fwd: 147_563,
            dz: 92,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        19,
        Bay {
            right: 18_407,
            fwd: 144_206,
            dz: 95,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        20,
        Bay {
            right: 18_469,
            fwd: 146_452,
            dz: 91,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: 7_510,
            fwd: 145_934,
            dz: 96,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        29,
        Bay {
            right: 1_202,
            fwd: 145_993,
            dz: 97,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 147_095,
            dz: 96,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_144,
            dz: 95,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
        42,
        Bay {
            right: 0,
            fwd: 145_525,
            dz: 95,
        },
    ),
    (
        77,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: 34_662,
            fwd: 150_565,
            dz: 181,
        },
    ),
    (
        752,
        crate::units::anim::ATTACK2,
        25,
        Bay {
            right: 30_200,
            fwd: 139_796,
            dz: 151,
        },
    ),
    (
        752,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        7,
        Bay {
            right: -117_433,
            fwd: -11_024,
            dz: 47,
        },
    ),
    (
        293,
        crate::units::anim::ATTACK2,
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
    // **The rows are the original's own sweep since item 4591**
    // (`docs/RECGAME.md` §13): `get_position` after `init_position` on
    // run1579's packet, at every whole degree, admits one region a key
    // (411, 196 and 113 lattice points, 361 of 361 headings each), and the
    // rows are their centroids. The rounds' fit held 357, 361 and 329 of
    // 361: row 33's `2/25` on 3385 (`CHAR_ATTACK3` at 4, facing
    // 244,973,568) left from x 6906 where theirs left from 6907.
    (
        49,
        crate::units::anim::ATTACK1,
        1,
        Bay {
            right: 33_049,
            fwd: 139_808,
            dz: 218,
        },
    ),
    (
        49,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 11_451,
            fwd: 166_599,
            dz: 176,
        },
    ),
    (
        49,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 12_330,
            fwd: 162_102,
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
    // **Piece 86, the Fusiliers in the default art style** (unit type 136,
    // `FUSILIER`; item 4159's third residue): run1185's packet holds its
    // three attack `AttachPos` rows (node 0, frames 6, 6 and 8), `RData
    // +0x88` 1.0, and `fit_bay.py` fits each sweep to piece 790's rows
    // exactly (48, 207 and 1679 lattice points, the same centroids). Without
    // them the British Fusiliers left from the figure's own point and flew
    // a frame long: run1185b's blow on the capital `0/2000` of 5275 landed
    // on 5276 in ours (`damage` 604 against 610).
    (
        86,
        11,
        6,
        Bay {
            right: 21_445,
            fwd: 111_997,
            dz: 110,
        },
    ),
    (
        86,
        12,
        6,
        Bay {
            right: 4_874,
            fwd: 96_147,
            dz: 116,
        },
    ),
    (
        86,
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
        crate::units::anim::ATTACK1,
        5,
        Bay {
            right: -293,
            fwd: 45_430,
            dz: 88,
        },
    ),
    (
        290,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -1_294,
            fwd: -14_718,
            dz: 88,
        },
    ),
    (
        290,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -2_106,
            fwd: -66_171,
            dz: 87,
        },
    ),
    (
        290,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -293,
            fwd: 45_430,
            dz: 88,
        },
    ),
    (
        290,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -1_294,
            fwd: -14_718,
            dz: 88,
        },
    ),
    (
        290,
        crate::units::anim::ATTACK2,
        17,
        Bay {
            right: -2_106,
            fwd: -66_171,
            dz: 87,
        },
    ),
    // Piece 120, the Bowmen. Run256 measured twelve arrows at three
    // facings. The window A state-9746 snapshot supplies all 361 headings
    // for each event, measured before the seed-123 window B comparison.
    (
        120,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        120,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        120,
        crate::units::anim::ATTACK3,
        15,
        Bay {
            right: 9_711,
            fwd: 66_508,
            dz: 164,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        13,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        16,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        18,
        Bay {
            right: 67_610,
            fwd: -16_464,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
        21,
        Bay {
            right: -71_064,
            fwd: 67_455,
            dz: -19,
        },
    ),
    (
        254,
        crate::units::anim::ATTACK2,
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
    //
    // **Re-fitted on the original's own sweep** (item 4507, `docs/RECGAME.md`
    // §11.1): twenty headings left each region wide, and run17's centroids
    // held 205, 324 and 345 of the 361 whole degrees. Row 5 of the third
    // held-out battery parted its ammo line on 4805 on one of the misses:
    // who=1's Slinger `1/22` at (5064, 30744), facing −1,097,334,784,
    // `CHAR_ATTACK2` frame 22, left from (4944, 30710) where theirs left
    // from (4945, 30709). `get_position.py … --init 0,<anim>,<time>,3 sweep
    // 32 0 <anim> <time>` on run1579's packet (`init_position` then
    // `get_position`) prints every heading; each row is now `fit_bay.py`'s
    // centroid of 1,696, 273 and 592 lattice points, 361 of 361 each.
    (
        32,
        crate::units::anim::ATTACK1,
        21,
        Bay {
            right: 69_764,
            fwd: 67_823,
            dz: 178,
        },
    ),
    (
        32,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: 41_238,
            fwd: 117_982,
            dz: 151,
        },
    ),
    (
        32,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: -42_142,
            fwd: 113_256,
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
        crate::units::anim::ATTACK1,
        18,
        Bay {
            right: 18_702,
            fwd: 146_963,
            dz: 177,
        },
    ),
    (
        38,
        crate::units::anim::ATTACK2,
        19,
        Bay {
            right: 22_458,
            fwd: 145_250,
            dz: 168,
        },
    ),
    (
        38,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        18,
        Bay {
            right: 37_765,
            fwd: 138_192,
            dz: 172,
        },
    ),
    (
        39,
        crate::units::anim::ATTACK2,
        19,
        Bay {
            right: 37_229,
            fwd: 135_427,
            dz: 161,
        },
    ),
    (
        39,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        21,
        Bay {
            right: 71_298,
            fwd: 69_592,
            dz: 174,
        },
    ),
    (
        1440,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: 32_704,
            fwd: 120_229,
            dz: 144,
        },
    ),
    (
        1440,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        21,
        Bay {
            right: 71_298,
            fwd: 69_592,
            dz: 174,
        },
    ),
    (
        1792,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: 32_704,
            fwd: 120_229,
            dz: 144,
        },
    ),
    (
        1792,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: -47_065,
            fwd: 114_687,
            dz: 168,
        },
    ),
    // **Piece 736, the Slingers in art style 2** (`32 + 2 × 0x160`; item
    // 4547, `docs/RECGAME.md` §12.3): who=2's Slinger on row 43 of the
    // fifth held-out battery, piece 32's keys. `get_position.py … --init
    // 0,<anim>,<time>,3 sweep 736 0 <anim> <time>` on run1579's packet
    // prints every heading, and `fit_bay.py` finds piece 1440's regions to
    // the point (534, 319 and 28 lattice points), so the rows are 1440's
    // centroids, 361 of 361 headings each. Without them row 43's stones
    // left the figure's own square: `2/30`'s on 3191 at `1/14` from (8472,
    // 23832, 516) in 6 frames where theirs left from (8571, 23834, 690) in
    // 5, the ammo line's first parting on 3192.
    (
        736,
        crate::units::anim::ATTACK1,
        21,
        Bay {
            right: 71_298,
            fwd: 69_592,
            dz: 174,
        },
    ),
    (
        736,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: 32_704,
            fwd: 120_229,
            dz: 144,
        },
    ),
    (
        736,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        21,
        Bay {
            right: 74_224,
            fwd: 62_577,
            dz: 176,
        },
    ),
    (
        1088,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: 43_592,
            fwd: 116_950,
            dz: 150,
        },
    ),
    (
        1088,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        130,
        crate::units::anim::ATTACK1,
        22,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        130,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: -13_756,
            fwd: 73_975,
            dz: 138,
        },
    ),
    (
        130,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        130,
        crate::units::anim::ATTACK3,
        10,
        Bay {
            right: 9_698,
            fwd: 61_839,
            dz: 182,
        },
    ),
    (
        130,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        131,
        crate::units::anim::ATTACK1,
        22,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        131,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: -13_756,
            fwd: 73_975,
            dz: 138,
        },
    ),
    (
        131,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: -3_071,
            fwd: 73_435,
            dz: 167,
        },
    ),
    (
        131,
        crate::units::anim::ATTACK3,
        10,
        Bay {
            right: 9_698,
            fwd: 61_839,
            dz: 182,
        },
    ),
    (
        131,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_067,
            fwd: 104_603,
            dz: 178,
        },
    ),
    (
        112,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        18,
        Bay {
            right: 17_058,
            fwd: 180_195,
            dz: 60,
        },
    ),
    (
        1143,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 4_863,
            fwd: 179_398,
            dz: 63,
        },
    ),
    (
        1143,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        18,
        Bay {
            right: 17_058,
            fwd: 180_195,
            dz: 60,
        },
    ),
    (
        88,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 4_863,
            fwd: 179_398,
            dz: 63,
        },
    ),
    (
        88,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 37_009,
            fwd: 121_444,
            dz: 120,
        },
    ),
    (
        90,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 37_800,
            fwd: 98_330,
            dz: 146,
        },
    ),
    (
        90,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        11,
        Bay {
            right: -541,
            fwd: 347_780,
            dz: 38,
        },
    ),
    (
        203,
        crate::units::anim::ATTACK2,
        23,
        Bay {
            right: -534,
            fwd: 350_174,
            dz: 38,
        },
    ),
    (
        203,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 24_607,
            fwd: 108_706,
            dz: 169,
        },
    ),
    (
        54,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 20_358,
            fwd: 103_245,
            dz: 173,
        },
    ),
    (
        54,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 8_133,
            fwd: 85_578,
            dz: 121,
        },
    ),
    (
        54,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 31_037,
            fwd: 80_976,
            dz: 118,
        },
    ),
    (
        54,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 22_929,
            fwd: 83_745,
            dz: 122,
        },
    ),
    (
        54,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 27_684,
            fwd: 159_512,
            dz: 106,
        },
    ),
    (
        54,
        crate::units::anim::ATTACK3,
        17,
        Bay {
            right: 38_064,
            fwd: 146_570,
            dz: 117,
        },
    ),
    // **Piece 1814, the Assault Infantry in the India art style** (type
    // 104, `ASSAULTINFANTRY-INDIA-AGE0`, `assault_infantry_india.bh3`:
    // base 54 at style 5, the Persians' and the Indians'; item 4159's
    // fourth residue): no packet on disk had built it, so its `AttachPos`
    // rows were never filled. `GraphicPieces::init_position@00901820` was
    // run for the art's eight `RELEASEEVENT`s (node 0 at 12 and 13 of
    // `CHAR_ATTACK1`, 4, 14 twice and 24 of `2`, 12 and 17 of `3`, axis 3)
    // under unicorn on run1579's packet, which holds every unit piece's
    // `RData`; the same call reproduces piece 54's (run1487), 1462's
    // (run854) and 1809's (run1324c) stored rows bit for bit. The eight
    // rows are piece 1462's (`ASSAULTINFANTRY-NA-AGE0`) bit for bit,
    // `RData +0x88` 1.0, and `fit_bay.py` fits each `get_position` sweep
    // (49, 98, 915, 819, 348, 322 and 225 lattice points; the centroid is
    // kept, 361 of 361 headings each). Without them run1075's Persian
    // `1/70` loosed its round of 2120 at `0/17` from its own square, flew
    // 5 frames against 4, and struck on 2124 against the original's 2123.
    (
        1814,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        1814,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 20_717,
            fwd: 82_052,
            dz: 170,
        },
    ),
    (
        1814,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 9_667,
            fwd: 64_207,
            dz: 120,
        },
    ),
    (
        1814,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        1814,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        1814,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
        },
    ),
    (
        1814,
        crate::units::anim::ATTACK3,
        17,
        Bay {
            right: 40_184,
            fwd: 125_286,
            dz: 115,
        },
    ),
    // **Piece 1837, the Advanced Machine Gun in the India art style**
    // (type 127, `ADVMACHINEGUN-INDIA-AGE0`: base 77 at style 5; item
    // 4159's fourth residue), **piece 1851, the Anti-Tank Missile in it**
    // (type 141, `TOW-INDIA-AGE0`: base 91 at style 5), and **piece 81,
    // the Flamethrower in the default style** (type 131,
    // `FLAMETHROWER-DEFAULT-AGE0`): the three other pieces run1075's walk
    // launched from the figure's own square once piece 1814 had its rows.
    // No packet holds 1837's or 1851's `AttachPos` rows; 33 hold
    // 81's (run785's among them), and no row was ever entered for it. All
    // three by piece 1814's way: `init_position@00901820` for each
    // `RELEASEEVENT` (node 0, axis 3) on run1579's packet — the same call
    // gives piece 77's 32 stored rows (run1472), 91's three (run1487) and
    // 81's three (run785, run1165) bit for bit — then `get_position`
    // swept, and `fit_bay.py` fits each sweep (1837: 23 to 3_260 lattice
    // points; 1851: 103, 137, 156; 81: 332, 471, 1_564; the centroid is kept, 361
    // of 361 headings each). 1837's `CHAR_ATTACK3` frame 12 is entered
    // twice with the same bits, as piece 77's. The India rows are not the
    // default's: 1837 and 1851 differ from 77 and 91 in every row.
    (
        1837,
        crate::units::anim::ATTACK1,
        3,
        Bay {
            right: -7_425,
            fwd: 147_384,
            dz: 93,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 88,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -16_551,
            fwd: 144_695,
            dz: 91,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -9_573,
            fwd: 146_738,
            dz: 90,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        20,
        Bay {
            right: 752,
            fwd: 146_479,
            dz: 84,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        24,
        Bay {
            right: 13_866,
            fwd: 144_797,
            dz: 89,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        29,
        Bay {
            right: 18_457,
            fwd: 146_563,
            dz: 86,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        32,
        Bay {
            right: 18_306,
            fwd: 144_313,
            dz: 83,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        37,
        Bay {
            right: 4_674,
            fwd: 148_076,
            dz: 90,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_935,
            dz: 91,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_376,
            dz: 88,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 88,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -402,
            fwd: 146_341,
            dz: 88,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -1_420,
            fwd: 148_009,
            dz: 90,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: -2_732,
            fwd: 145_714,
            dz: 93,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -5_603,
            fwd: 144_577,
            dz: 98,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -7_510,
            fwd: 145_038,
            dz: 103,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -5_569,
            fwd: 143_666,
            dz: 103,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: -3_051,
            fwd: 145_130,
            dz: 103,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 2_021,
            fwd: 145_012,
            dz: 103,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        3,
        Bay {
            right: 2_079,
            fwd: 146_008,
            dz: 91,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 2_774,
            fwd: 145_170,
            dz: 93,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 13_610,
            fwd: 147_369,
            dz: 85,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        19,
        Bay {
            right: 18_423,
            fwd: 144_252,
            dz: 89,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        20,
        Bay {
            right: 18_424,
            fwd: 146_258,
            dz: 85,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: 7_504,
            fwd: 146_012,
            dz: 89,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        29,
        Bay {
            right: 1_218,
            fwd: 146_171,
            dz: 90,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 90,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_187,
            dz: 88,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        42,
        Bay {
            right: 0,
            fwd: 145_525,
            dz: 88,
        },
    ),
    (
        1837,
        crate::units::anim::ATTACK3,
        46,
        Bay {
            right: 0,
            fwd: 146_477,
            dz: 88,
        },
    ),
    (
        1851,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 28_130,
            fwd: 97_466,
            dz: 125,
        },
    ),
    (
        1851,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 29_987,
            fwd: 76_566,
            dz: 143,
        },
    ),
    (
        1851,
        crate::units::anim::ATTACK3,
        13,
        Bay {
            right: 40_693,
            fwd: 87_027,
            dz: 140,
        },
    ),
    (
        81,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: 14_704,
            fwd: 82_911,
            dz: 126,
        },
    ),
    (
        81,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 19_632,
            fwd: 79_726,
            dz: 129,
        },
    ),
    (
        81,
        crate::units::anim::ATTACK2,
        19,
        Bay {
            right: 16_955,
            fwd: 80_963,
            dz: 126,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        1108,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 20_717,
            fwd: 82_052,
            dz: 170,
        },
    ),
    (
        1108,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 9667,
            fwd: 64_207,
            dz: 120,
        },
    ),
    (
        1108,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        1108,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        60,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        60,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        60,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 17_840,
            fwd: 218_651,
            dz: 181,
        },
    ),
    (
        60,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        52,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 20_717,
            fwd: 82_052,
            dz: 170,
        },
    ),
    (
        52,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 9667,
            fwd: 64_207,
            dz: 120,
        },
    ),
    (
        52,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        52,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        52,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 32_578,
            fwd: 121_906,
            dz: 127,
        },
    ),
    (
        1145,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 33_663,
            fwd: 97_657,
            dz: 150,
        },
    ),
    (
        1145,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 32_578,
            fwd: 121_906,
            dz: 127,
        },
    ),
    (
        89,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 33_663,
            fwd: 97_657,
            dz: 150,
        },
    ),
    (
        89,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 32_578,
            fwd: 121_906,
            dz: 127,
        },
    ),
    (
        441,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 33_663,
            fwd: 97_657,
            dz: 150,
        },
    ),
    (
        441,
        crate::units::anim::ATTACK3,
        13,
        Bay {
            right: 48_137,
            fwd: 109_678,
            dz: 146,
        },
    ),
    // **Piece 1849, the Bazooka in the India art style** (type 139,
    // `BAZOOKA-INDIA-AGE0`: base 89 at style 5; item 4626, `docs/RECGAME.md`
    // §14.2): run1816's packet (row 28's game at 3955) holds its three
    // `AttachPos` rows (node 0 at 17 of `CHAR_ATTACK1`, 7 of `2`, 13 of `3`),
    // `RData +0x88` 1.0, no `<RESTRICTION>`; `fit_bay.py` fits each sweep (77,
    // 128 and 121 lattice points; the centroid is kept, 361 of 361 headings
    // each). Piece 89's rows reproduce 224, 229 and 221 of them, and its
    // `CHAR_ATTACK2` height is 150 against this one's 151. Without them
    // who=2's `2/34` loosed its round of 4026 at `0/2000` from its own
    // square (12264, 12552, 145) against the original's (12208, 12465, 296).
    (
        1849,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 32_862,
            fwd: 122_217,
            dz: 127,
        },
    ),
    (
        1849,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 33_947,
            fwd: 97_903,
            dz: 151,
        },
    ),
    (
        1849,
        crate::units::anim::ATTACK3,
        13,
        Bay {
            right: 48_436,
            fwd: 109_919,
            dz: 146,
        },
    ),
    // **Piece 1812, the Infantry in the India art style** (type 102,
    // `INFANTRY-INDIA-AGE0`: base 52 at style 5; item 4630, `docs/RECGAME.md`
    // §14.2): run1816's packet (row 28's game at 3955) holds its six
    // `AttachPos` rows at piece 52's keys, `RData +0x88` 1.0; `fit_bay.py`
    // fits each sweep to piece 52's regions exactly (49, 98, 915, 819, 348
    // and 322 lattice points, the same centroids and heights). Without them
    // who=2's `2/50` loosed its round of 4166 from its own square (13032,
    // 9000, 24) against the original's (12973, 9025, 144).
    (
        1812,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        1812,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 20_717,
            fwd: 82_052,
            dz: 170,
        },
    ),
    (
        1812,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 9667,
            fwd: 64_207,
            dz: 120,
        },
    ),
    (
        1812,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        1812,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        1812,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
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
        crate::units::anim::ATTACK1,
        18,
        Bay {
            right: 17_058,
            fwd: 180_195,
            dz: 60,
        },
    ),
    (
        87,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 4_863,
            fwd: 179_398,
            dz: 63,
        },
    ),
    (
        87,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACKWALK,
        3,
        Bay {
            right: 496,
            fwd: 177_238,
            dz: 17,
        },
    ),
    (
        167,
        crate::units::anim::ATTACKWALK,
        9,
        Bay {
            right: 691,
            fwd: 177_606,
            dz: 13,
        },
    ),
    (
        167,
        crate::units::anim::ATTACKWALK,
        12,
        Bay {
            right: 591,
            fwd: 177_528,
            dz: 14,
        },
    ),
    (
        167,
        crate::units::anim::ATTACKWALK,
        17,
        Bay {
            right: 0,
            fwd: 177_722,
            dz: 12,
        },
    ),
    (
        167,
        crate::units::anim::ATTACKWALK,
        20,
        Bay {
            right: -703,
            fwd: 177_226,
            dz: 17,
        },
    ),
    (
        167,
        crate::units::anim::ATTACKWALK,
        22,
        Bay {
            right: -717,
            fwd: 177_502,
            dz: 15,
        },
    ),
    (
        167,
        crate::units::anim::ATTACKWALK,
        24,
        Bay {
            right: -834,
            fwd: 177_267,
            dz: 17,
        },
    ),
    (
        167,
        crate::units::anim::ATTACKWALK,
        29,
        Bay {
            right: -87,
            fwd: 177_560,
            dz: 14,
        },
    ),
    (
        167,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: 0,
            fwd: 176_708,
            dz: 17,
        },
    ),
    (
        167,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 0,
            fwd: 177_096,
            dz: 13,
        },
    ),
    (
        167,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 0,
            fwd: 176_983,
            dz: 14,
        },
    ),
    (
        167,
        crate::units::anim::ATTACK2,
        17,
        Bay {
            right: 0,
            fwd: 177_197,
            dz: 12,
        },
    ),
    (
        167,
        crate::units::anim::ATTACK2,
        20,
        Bay {
            right: 0,
            fwd: 176_708,
            dz: 17,
        },
    ),
    (
        167,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: 0,
            fwd: 176_983,
            dz: 15,
        },
    ),
    (
        167,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 0,
            fwd: 176_747,
            dz: 17,
        },
    ),
    (
        167,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 15_269,
            fwd: 121_470,
            dz: 168,
        },
    ),
    (
        65,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: 5_281,
            fwd: 133_941,
            dz: 122,
        },
    ),
    (
        65,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: 13_374,
            fwd: 144_082,
            dz: 168,
        },
    ),
    (
        64,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: 13_848,
            fwd: 145_503,
            dz: 169,
        },
    ),
    (
        64,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        4,
        Bay {
            right: 29_986,
            fwd: 193,
            dz: -2,
        },
    ),
    (
        277,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: -98_342,
            fwd: -4_178,
            dz: 50,
        },
    ),
    (
        277,
        crate::units::anim::ATTACKWALK,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 38_941,
            fwd: 99_188,
            dz: 163,
        },
    ),
    (
        33,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: 1_649,
            fwd: 89_737,
            dz: 170,
        },
    ),
    (
        33,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACKWALK,
        2,
        Bay {
            right: -150,
            fwd: 67_991,
            dz: 52,
        },
    ),
    (
        273,
        crate::units::anim::ATTACKWALK,
        10,
        Bay {
            right: 1_110,
            fwd: -110_298,
            dz: 52,
        },
    ),
    (
        273,
        crate::units::anim::ATTACKWALK,
        17,
        Bay {
            right: -1_724,
            fwd: -26_208,
            dz: 52,
        },
    ),
    (
        273,
        crate::units::anim::ATTACK1,
        2,
        Bay {
            right: -150,
            fwd: 67_991,
            dz: 52,
        },
    ),
    (
        273,
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 1_110,
            fwd: -110_298,
            dz: 52,
        },
    ),
    (
        273,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -1_724,
            fwd: -26_208,
            dz: 52,
        },
    ),
    (
        273,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -150,
            fwd: 67_991,
            dz: 52,
        },
    ),
    (
        273,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: 1_110,
            fwd: -110_298,
            dz: 52,
        },
    ),
    (
        273,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACKWALK,
        2,
        Bay {
            right: -148,
            fwd: 187_348,
            dz: 52,
        },
    ),
    (
        274,
        crate::units::anim::ATTACKWALK,
        10,
        Bay {
            right: 1_110,
            fwd: 9_061,
            dz: 52,
        },
    ),
    (
        274,
        crate::units::anim::ATTACKWALK,
        17,
        Bay {
            right: -1_724,
            fwd: 93_151,
            dz: 52,
        },
    ),
    (
        274,
        crate::units::anim::ATTACK1,
        2,
        Bay {
            right: -148,
            fwd: 187_348,
            dz: 52,
        },
    ),
    (
        274,
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 1_110,
            fwd: 9_061,
            dz: 52,
        },
    ),
    (
        274,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -1_724,
            fwd: 93_151,
            dz: 52,
        },
    ),
    (
        274,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -148,
            fwd: 187_348,
            dz: 52,
        },
    ),
    (
        274,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: 1_110,
            fwd: 9_061,
            dz: 52,
        },
    ),
    (
        274,
        crate::units::anim::ATTACK2,
        17,
        Bay {
            right: -1_724,
            fwd: 93_151,
            dz: 52,
        },
    ),
    (
        567,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 38_946,
            fwd: 60_202,
            dz: 163,
        },
    ),
    (
        1794,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: 1_685,
            fwd: 50_747,
            dz: 170,
        },
    ),
    (
        1794,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 38_946,
            fwd: 60_202,
            dz: 163,
        },
    ),
    (
        1793,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: 1_685,
            fwd: 50_747,
            dz: 170,
        },
    ),
    (
        1793,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 38_946,
            fwd: 60_202,
            dz: 163,
        },
    ),
    (
        1090,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: 1_685,
            fwd: 50_747,
            dz: 170,
        },
    ),
    (
        1090,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        1881,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        1881,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_067,
            fwd: 104_603,
            dz: 178,
        },
    ),
    (
        113,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACKWALK,
        16,
        Bay {
            right: -29_938,
            fwd: 87_586,
            dz: 448,
        },
    ),
    (
        212,
        crate::units::anim::ATTACK1,
        16,
        Bay {
            right: -291,
            fwd: 71_525,
            dz: 451,
        },
    ),
    (
        212,
        crate::units::anim::ATTACK2,
        16,
        Bay {
            right: -17_740,
            fwd: -7_348,
            dz: 465,
        },
    ),
    (
        212,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: -18_234,
            fwd: 60_948,
            dz: 463,
        },
    ),
    (
        212,
        crate::units::anim::ATTACKSPECIAL,
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
        crate::units::anim::ATTACKWALK,
        14,
        Bay {
            right: -18_396,
            fwd: 103_327,
            dz: 426,
        },
    ),
    (
        213,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 9_808,
            fwd: 94_776,
            dz: 440,
        },
    ),
    (
        213,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 48,
            fwd: 63_526,
            dz: 422,
        },
    ),
    (
        213,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: -1_026,
            fwd: 87_367,
            dz: 449,
        },
    ),
    (
        213,
        crate::units::anim::ATTACKSPECIAL,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 33_637,
            fwd: 65_607,
            dz: 185,
        },
    ),
    (
        43,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: -13_157,
            fwd: 50_004,
            dz: 187,
        },
    ),
    (
        43,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 33_637,
            fwd: 65_607,
            dz: 185,
        },
    ),
    (
        44,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: -13_157,
            fwd: 50_004,
            dz: 187,
        },
    ),
    (
        44,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        473,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        473,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        824,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        824,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -1_826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        825,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        825,
        crate::units::anim::ATTACK3,
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
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -1_790,
            fwd: 86_974,
            dz: 160,
        },
    ),
    (
        1176,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 331,
            fwd: 85_656,
            dz: 160,
        },
    ),
    (
        1176,
        crate::units::anim::ATTACK3,
        15,
        Bay {
            right: 9_531,
            fwd: 65_185,
            dz: 161,
        },
    ),
    // **Piece 122, the Crossbowmen** (`CROSSBOWMEN`, type 172; item 4517,
    // `docs/RECGAME.md` §12): node 0 at frame 5 of `CHAR_ATTACK1`, 3 of
    // `CHAR_ATTACK2` and 6 of `CHAR_ATTACK3`. `get_position.py … --init
    // 0,<anim>,<time>,3 sweep 122 0 <anim> <time>` on run1579's packet gives
    // the original's offset at all 361 whole degrees, and `fit_bay.py` admits
    // one region a key (133, 688 and 417 lattice points, 361 of 361
    // headings); the rows are the centroids, the heights the sweep's. They
    // are piece 1178's rows below over its scale 1.04, to the unit: one
    // socket under two styles. Until then they were item 3347's planar
    // [`Node`]s, one bolt each off run982, and row 75 of the fourth held-out
    // battery parted its ammo line on 4004 at a heading they missed by a
    // unit: who=1's `1/163`'s `CHAR_ATTACK3` bolt left (3922, 34374) where
    // run1768's leaves (3923, 34374).
    (
        122,
        crate::units::anim::ATTACK1,
        5,
        Bay {
            right: 11_399,
            fwd: 84_486,
            dz: 166,
        },
    ),
    (
        122,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: 14_863,
            fwd: 64_484,
            dz: 134,
        },
    ),
    (
        122,
        crate::units::anim::ATTACK3,
        6,
        Bay {
            right: 15_287,
            fwd: 75_439,
            dz: 161,
        },
    ),
    // **Item 4559's plain pieces** (`docs/RECGAME.md` §12.3; DECISIONS 152
    // §3), keys the held-out games and the 2017 sample fired from the
    // figure's own square. Run1579's packet holds no `AttachPos` row for any
    // of them (`pos_num` 0, scale 1.0), so `get_position.py … --init
    // 0,<anim>,<time>,3 sweep <piece> 0 <anim> <time>` fills each key first,
    // and `fit_bay.py` admits one region a key, 361 of 361 headings, its
    // centroid inside it; the heights are the sweep's. Run1355's packet
    // stores the Machine Gun's 32 rows, run1001d's the Bowmen's four and
    // run1438's the Bombard's one, and a sweep of each stored row agrees
    // with run1579's at every heading (35 of 35 keys).
    //
    // Piece 921, the American Bombard (`BOMBARD-AMERICAN-AGE0`, type 267 in
    // art style 2): `bombardcannon.bh3` and `BOMBARD-DEFAULT-AGE0`'s anims,
    // and its sweep admits piece 217's region (924 points) and centroid.
    (
        921,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -710,
            fwd: 147_904,
            dz: 104,
        },
    ),
    // Piece 1528, the Native American Bowmen (`BOWMEN-NA-AGE0`, type 170
    // in art style 4, `Bowmen_NA.bh3`): node 0 at frame 12 of
    // `CHAR_ATTACK1`, 9 of `2` and 15 of `3` (221, 277 and 124 points).
    // Row 43 of the fifth held-out battery fired 788 of its rounds from the
    // figure's square from 3360.
    (
        1528,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -1826,
            fwd: 88_749,
            dz: 163,
        },
    ),
    (
        1528,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 336,
            fwd: 87_408,
            dz: 164,
        },
    ),
    (
        1528,
        crate::units::anim::ATTACK3,
        15,
        Bay {
            right: 9711,
            fwd: 66_508,
            dz: 164,
        },
    ),
    // Piece 73, the Machine Gun (`MACHINEGUN-DEFAULT-AGE0`, type 123,
    // `maxim_gunner.bh3`): node 0 on every one of its 32 `<RELEASEEVENT>`s,
    // 31 keys (two events share frame 12 of `CHAR_ATTACK3`), each its own
    // region; row 83 fired 79 rounds from the figure's square from 4252.
    (
        73,
        crate::units::anim::ATTACK1,
        3,
        Bay {
            right: -7425,
            fwd: 147_384,
            dz: 93,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 88,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -16_551,
            fwd: 144_695,
            dz: 91,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -9573,
            fwd: 146_738,
            dz: 90,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        20,
        Bay {
            right: 752,
            fwd: 146_479,
            dz: 84,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        24,
        Bay {
            right: 13_866,
            fwd: 144_797,
            dz: 89,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        29,
        Bay {
            right: 18_457,
            fwd: 146_563,
            dz: 86,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        32,
        Bay {
            right: 18_306,
            fwd: 144_313,
            dz: 83,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        37,
        Bay {
            right: 4674,
            fwd: 148_076,
            dz: 90,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_935,
            dz: 91,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_376,
            dz: 88,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 88,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -402,
            fwd: 146_341,
            dz: 88,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -1420,
            fwd: 148_009,
            dz: 90,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: -2732,
            fwd: 145_714,
            dz: 93,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -5603,
            fwd: 144_577,
            dz: 98,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -7510,
            fwd: 145_038,
            dz: 103,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -5569,
            fwd: 143_666,
            dz: 103,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: -3051,
            fwd: 145_130,
            dz: 103,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 2021,
            fwd: 145_012,
            dz: 103,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        3,
        Bay {
            right: 1011,
            fwd: 144_798,
            dz: 91,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 1830,
            fwd: 143_956,
            dz: 92,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 11_993,
            fwd: 147_443,
            dz: 85,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        19,
        Bay {
            right: 18_422,
            fwd: 144_413,
            dz: 88,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        20,
        Bay {
            right: 18_428,
            fwd: 146_106,
            dz: 85,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: 8410,
            fwd: 146_147,
            dz: 88,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        29,
        Bay {
            right: 1830,
            fwd: 146_325,
            dz: 90,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 146_847,
            dz: 90,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_400,
            dz: 88,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        42,
        Bay {
            right: 0,
            fwd: 145_708,
            dz: 88,
        },
    ),
    (
        73,
        crate::units::anim::ATTACK3,
        46,
        Bay {
            right: 0,
            fwd: 146_272,
            dz: 88,
        },
    ),
    // **Piece 1494, the Fusiliers** (type 136; item 4558, `docs/RECGAME.md`
    // §12.3): node 0 at frame 6 of `CHAR_ATTACK1` and `CHAR_ATTACK2`, 8 of
    // `CHAR_ATTACK3`. Run1579's packet holds no `AttachPos` row for the
    // piece (`pos_num` 0, scale 1.0), so `get_position.py … --init
    // 0,<anim>,<time>,3 sweep 1494 0 <anim> <time>` fills them first, and
    // `fit_bay.py` admits one region a key (177, 5 and 574 lattice points,
    // 361 of 361 headings). The first and third rows are the centroids; the
    // second's centroid falls outside its five points and misses four
    // headings, so the row is the region's middle point. The heights are the
    // sweep's. Without them row 63 of the fifth held-out battery parted its
    // ammo line on 3683: who=1's `1/45` left from its own square at (8568,
    // 12312, 468) where run1782's leaves from (8633, 12404, 578).
    (
        1494,
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: 20_902,
            fwd: 111_780,
            dz: 110,
        },
    ),
    (
        1494,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: 4_640,
            fwd: 95_886,
            dz: 116,
        },
    ),
    (
        1494,
        crate::units::anim::ATTACK3,
        8,
        Bay {
            right: 15_834,
            fwd: 99_875,
            dz: 115,
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
    // Piece 122's rows above are the default style's and never reached it.
    // Without these who=1's `1/43` loosed its bolt of 3981 at the
    // human's `0/2004` from its own square (6120, 11208, 19) against the
    // original's (6103, 11142, 159), flew 11 frames against 10, and the
    // building was struck a frame late.
    (
        1178,
        crate::units::anim::ATTACK1,
        5,
        Bay {
            right: 11_856,
            fwd: 87_864,
            dz: 173,
        },
    ),
    (
        1178,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: 15_464,
            fwd: 67_070,
            dz: 140,
        },
    ),
    (
        1178,
        crate::units::anim::ATTACK3,
        6,
        Bay {
            right: 15_879,
            fwd: 78_459,
            dz: 168,
        },
    ),
    // **Forty plain pieces the widened suite fired from the figure's square**
    // (item 4159's fifth residue, 4159r4a): a probe on `launch_point`'s
    // fallback over the widened suite (`widened_whole walks_back
    // walks_the_gap dates_the_gap old_word_keeps attrition_stamp2`, 673
    // tests) listed 1,159 rounds from 53 pieces with no row. Each plain
    // piece's rows were built by piece 1814's way on run1579's packet:
    // `GraphicPieces::init_position@00901820` for each of the art's
    // `RELEASEEVENT`s (the packet already held pieces 15's and 371's), then
    // `get_position` swept at every whole degree, and `fit_bay.py` fits each
    // sweep; the centroid is kept, 361 of 361 headings each. Piece 1462's
    // seven are piece 1814's bit for bit, as its stored rows on run854 are.
    // The three whose every event on a node leaves one point (251, 257,
    // 1311) are [`GUNS`] rows. The Armed Supply Wagon (15) is left out: its
    // four rows are zero, the figure's own square. The pivot pieces (148,
    // 158, 173, 207, 232, 234, 520, 844, 1899) need `pivot::RELEASES` rows.
    // Piece 45, the Impi, `IMPI-DEFAULT-AGE0`: lattice 42, 229, 95 (6 launches).
    (
        45,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 33_637,
            fwd: 65_607,
            dz: 185,
        },
    ),
    (
        45,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: -13_157,
            fwd: 50_004,
            dz: 187,
        },
    ),
    (
        45,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: 14_497,
            fwd: 40_695,
            dz: 173,
        },
    ),
    // Piece 57, the Red Guard, `REDGUARD-DEFAULT-AGE0`: lattice 49, 819, 348, 322, 225 (22 launches).
    (
        57,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        57,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        57,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        57,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
        },
    ),
    (
        57,
        crate::units::anim::ATTACK3,
        17,
        Bay {
            right: 40_184,
            fwd: 125_286,
            dz: 115,
        },
    ),
    // Piece 59, the German Infantry, `INFANTRYGERMAN-DEFAULT-AGE0`: lattice 49, 819, 348, 322, 225 (2 launches).
    (
        59,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        59,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        59,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        59,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
        },
    ),
    (
        59,
        crate::units::anim::ATTACK3,
        17,
        Bay {
            right: 40_184,
            fwd: 125_286,
            dz: 115,
        },
    ),
    // Piece 63, the Royal Janissary, `JANISSARYROYAL-DEFAULT-AGE0`: lattice 277, 508, 5 (6 launches).
    (
        63,
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: -909,
            fwd: 147_973,
            dz: 164,
        },
    ),
    (
        63,
        crate::units::anim::ATTACK2,
        25,
        Bay {
            right: 26_573,
            fwd: 133_083,
            dz: 159,
        },
    ),
    (
        63,
        crate::units::anim::ATTACK3,
        23,
        Bay {
            right: 13_153,
            fwd: 125_321,
            dz: 164,
        },
    ),
    // Piece 68, the Manchu Infantry, `MANCHUINFANTRY-DEFAULT-AGE0`: lattice 49, 819, 348, 322, 225 (14 launches).
    (
        68,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        68,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        68,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        68,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
        },
    ),
    (
        68,
        crate::units::anim::ATTACK3,
        17,
        Bay {
            right: 40_184,
            fwd: 125_286,
            dz: 115,
        },
    ),
    // Piece 71, the Marine Infantry, `MARINEINFANTRY-DEFAULT-AGE0`: lattice 49, 98, 915, 819, 348, 322 (3 launches).
    (
        71,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        71,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 20_717,
            fwd: 82_052,
            dz: 170,
        },
    ),
    (
        71,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 9_667,
            fwd: 64_207,
            dz: 120,
        },
    ),
    (
        71,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        71,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        71,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
        },
    ),
    // Piece 111, the Gun Samurai, `GUNSAMURAI-DEFAULT-AGE0`: lattice 188, 668, 117 (15 launches).
    (
        111,
        crate::units::anim::ATTACK1,
        25,
        Bay {
            right: 33_663,
            fwd: 104_507,
            dz: 161,
        },
    ),
    (
        111,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 54_561,
            fwd: 133_846,
            dz: 158,
        },
    ),
    (
        111,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 25_189,
            fwd: 133_072,
            dz: 130,
        },
    ),
    // Piece 114, the Athanatoi, `ATHANATOI-DEFAULT-AGE0`: lattice 105, 115 (18 launches).
    (
        114,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_067,
            fwd: 104_603,
            dz: 178,
        },
    ),
    (
        114,
        crate::units::anim::ATTACK3,
        6,
        Bay {
            right: 27_678,
            fwd: 36_793,
            dz: 171,
        },
    ),
    // Piece 210, the Russian Heavy Tank, `RUSSHEAVYTANK-DEFAULT-AGE0`: lattice 688, 478, 1118, 1174, 478, 609, 74 (3 launches).
    (
        210,
        crate::units::anim::ATTACK1,
        11,
        Bay {
            right: -498,
            fwd: 326_463,
            dz: 33,
        },
    ),
    (
        210,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 15_112,
            fwd: 148_429,
            dz: 130,
        },
    ),
    (
        210,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 15_129,
            fwd: 149_061,
            dz: 126,
        },
    ),
    (
        210,
        crate::units::anim::ATTACK2,
        23,
        Bay {
            right: -515,
            fwd: 328_700,
            dz: 33,
        },
    ),
    (
        210,
        crate::units::anim::ATTACK3,
        1,
        Bay {
            right: 15_112,
            fwd: 148_429,
            dz: 130,
        },
    ),
    (
        210,
        crate::units::anim::ATTACK3,
        7,
        Bay {
            right: 23_807,
            fwd: 147_942,
            dz: 126,
        },
    ),
    (
        210,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: -500,
            fwd: 289_756,
            dz: 32,
        },
    ),
    // Piece 216, the Trebuchet, `TREBUCHET-DEFAULT-AGE0`: lattice 266 (108 launches).
    (
        216,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -199,
            fwd: -301_788,
            dz: 693,
        },
    ),
    // Piece 217, the Bombard, `BOMBARD-DEFAULT-AGE0`: lattice 924 (29 launches).
    (
        217,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -710,
            fwd: 147_904,
            dz: 104,
        },
    ),
    // Piece 219, the Artillery, `ARTILLERY-DEFAULT-AGE0`: lattice 492 (22 launches).
    (
        219,
        crate::units::anim::ATTACK2,
        70,
        Bay {
            right: 0,
            fwd: 53_219,
            dz: 136,
        },
    ),
    // Piece 220, the German Artillery, `ARTILLERYGERMAN-DEFAULT-AGE0`: lattice 492 (1 launches).
    (
        220,
        crate::units::anim::ATTACK2,
        70,
        Bay {
            right: 0,
            fwd: 53_219,
            dz: 136,
        },
    ),
    // Piece 221, the Howitzer, `HOWITZER-DEFAULT-AGE0`: lattice 1456, 38, 261 (9 launches).
    (
        221,
        crate::units::anim::ATTACK1,
        1,
        Bay {
            right: 0,
            fwd: 154_785,
            dz: 250,
        },
    ),
    (
        221,
        crate::units::anim::ATTACK2,
        26,
        Bay {
            right: 0,
            fwd: 176_390,
            dz: 206,
        },
    ),
    (
        221,
        crate::units::anim::ATTACK3,
        8,
        Bay {
            right: 0,
            fwd: 163_564,
            dz: 262,
        },
    ),
    // Piece 225, the Siege Mortar, `SIEGEMORTAR-DEFAULT-AGE0`: lattice 449, 449 (7 launches).
    (
        225,
        crate::units::anim::ATTACK1,
        48,
        Bay {
            right: 108,
            fwd: 3_619,
            dz: 161,
        },
    ),
    (
        225,
        crate::units::anim::ATTACK2,
        48,
        Bay {
            right: 108,
            fwd: 3_619,
            dz: 161,
        },
    ),
    // Piece 226, the Basilica Bombard, `BASILICABOMBARD-DEFAULT-AGE0`: lattice 238 (1 launches).
    (
        226,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -720,
            fwd: 147_722,
            dz: 104,
        },
    ),
    // Piece 227, the Basilica Cannon, `BASILICACANNON-DEFAULT-AGE0`: lattice 99 (21 launches).
    (
        227,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    // Piece 229, the Flaming Arrow, `FLAMINGARROW-DEFAULT-AGE0`: lattice 265 (19 launches).
    (
        229,
        crate::units::anim::ATTACK2,
        53,
        Bay {
            right: 0,
            fwd: 154_204,
            dz: 135,
        },
    ),
    // Piece 230, the Heavy Flaming Arrow, `HVYFLAMINGARROW-DEFAULT-AGE0`: lattice 265 (6 launches).
    (
        230,
        crate::units::anim::ATTACK2,
        53,
        Bay {
            right: 0,
            fwd: 154_204,
            dz: 135,
        },
    ),
    // Piece 249, the MiG, `MIG-DEFAULT-AGE0`: lattice 395, 139 (11 launches).
    (
        249,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: -142_352,
            fwd: -126_701,
            dz: -55,
        },
    ),
    (
        249,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 140_602,
            fwd: -126_701,
            dz: -55,
        },
    ),
    // Piece 250, the desert MiG, `MIGDES-DEFAULT-AGE0`: lattice 395, 139 (3 launches).
    (
        250,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: -142_352,
            fwd: -126_701,
            dz: -55,
        },
    ),
    (
        250,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 140_602,
            fwd: -126_701,
            dz: -55,
        },
    ),
    // Piece 253, the Eagle fighter-bomber, `EAGLEFB-DEFAULT-AGE0`: lattice 5, 147 (4 launches).
    (
        253,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 156_855,
            fwd: -41_553,
            dz: -51,
        },
    ),
    (
        253,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -156_740,
            fwd: -41_874,
            dz: -51,
        },
    ),
    // Piece 256, the Stealth Bomber, `STEALTHBOMBER-DEFAULT-AGE0`: lattice 359, 90, 73, 440 (12 launches).
    (
        256,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 81_583,
            fwd: -65_371,
            dz: 12,
        },
    ),
    (
        256,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: -81_802,
            fwd: -65_359,
            dz: 17,
        },
    ),
    (
        256,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 81_684,
            fwd: -65_361,
            dz: 0,
        },
    ),
    (
        256,
        crate::units::anim::ATTACK2,
        11,
        Bay {
            right: -82_048,
            fwd: -65_357,
            dz: 12,
        },
    ),
    // Piece 260, the Helicopter, `HELICOPTER-DEFAULT-AGE0`: lattice 223, 664 (24 launches).
    (
        260,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -53_724,
            fwd: 73_841,
            dz: 38,
        },
    ),
    (
        260,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 58_849,
            fwd: 58_457,
            dz: 38,
        },
    ),
    // Piece 371, the Arab Scout, `SCOUT-ARAB-AGE0`: lattice 133 (1 launches).
    (
        371,
        crate::units::anim::ATTACK2,
        23,
        Bay {
            right: 10_650,
            fwd: 80_505,
            dz: 160,
        },
    ),
    // Piece 568, the Arab Trebuchet, `TREBUCHET-ARAB-AGE0`: lattice 266 (15 launches).
    (
        568,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -199,
            fwd: -301_788,
            dz: 693,
        },
    ),
    // Piece 573, the Arab Howitzer, `HOWITZER-ARAB-AGE0`: lattice 1456, 38, 261 (2 launches).
    (
        573,
        crate::units::anim::ATTACK1,
        1,
        Bay {
            right: 0,
            fwd: 154_785,
            dz: 250,
        },
    ),
    (
        573,
        crate::units::anim::ATTACK2,
        26,
        Bay {
            right: 0,
            fwd: 176_390,
            dz: 206,
        },
    ),
    (
        573,
        crate::units::anim::ATTACK3,
        8,
        Bay {
            right: 0,
            fwd: 163_564,
            dz: 262,
        },
    ),
    // Piece 919, the American Catapult, `CATAPULT-AMERICAN-AGE0`: lattice 435 (70 launches).
    (
        919,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 0,
            fwd: -33_217,
            dz: 245,
        },
    ),
    // Piece 920, the American Trebuchet, `TREBUCHET-AMERICAN-AGE0`: lattice 266 (17 launches).
    (
        920,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -199,
            fwd: -301_788,
            dz: 693,
        },
    ),
    // Piece 1271, the Asian Catapult, `CATAPULT-ASIAN-AGE0`: lattice 435 (50 launches).
    (
        1271,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 0,
            fwd: -33_217,
            dz: 245,
        },
    ),
    // Piece 1272, the Asian Trebuchet, `TREBUCHET-ASIAN-AGE0`: lattice 266 (26 launches).
    (
        1272,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -199,
            fwd: -301_788,
            dz: 693,
        },
    ),
    // Piece 1273, the Asian Bombard, `BOMBARD-ASIAN-AGE0`: lattice 924 (30 launches).
    (
        1273,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -710,
            fwd: 147_904,
            dz: 104,
        },
    ),
    // Piece 1274, the Asian Cannon, `CANNON-ASIAN-AGE0`: lattice 99, 99, 99 (15 launches).
    (
        1274,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    (
        1274,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    (
        1274,
        crate::units::anim::ATTACK3,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    // Piece 1275, the Asian Artillery, `ARTILLERY-ASIAN-AGE0`: lattice 492 (8 launches).
    (
        1275,
        crate::units::anim::ATTACK2,
        70,
        Bay {
            right: 0,
            fwd: 53_219,
            dz: 136,
        },
    ),
    // Piece 1462, the North American Assault Infantry, `ASSAULTINFANTRY-NA-AGE0`: lattice 49, 98, 915, 819, 348, 322, 225 (3 launches).
    (
        1462,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 23_820,
            fwd: 87_417,
            dz: 167,
        },
    ),
    (
        1462,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 20_717,
            fwd: 82_052,
            dz: 170,
        },
    ),
    (
        1462,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 9_667,
            fwd: 64_207,
            dz: 120,
        },
    ),
    (
        1462,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 25_919,
            fwd: 60_398,
            dz: 118,
        },
    ),
    (
        1462,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 21_173,
            fwd: 62_648,
            dz: 121,
        },
    ),
    (
        1462,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 31_494,
            fwd: 138_342,
            dz: 107,
        },
    ),
    (
        1462,
        crate::units::anim::ATTACK3,
        17,
        Bay {
            right: 40_184,
            fwd: 125_286,
            dz: 115,
        },
    ),
    // Piece 1623, the North American Catapult, `CATAPULT-NA-AGE0`: lattice 435 (22 launches).
    (
        1623,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 0,
            fwd: -33_217,
            dz: 245,
        },
    ),
    // Piece 1626, the North American Cannon, `CANNON-NA-AGE0`: lattice 99, 99, 99 (9 launches).
    (
        1626,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    (
        1626,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    (
        1626,
        crate::units::anim::ATTACK3,
        9,
        Bay {
            right: -720,
            fwd: 147_549,
            dz: 100,
        },
    ),
    // Piece 1882, the India Crossbowmen, `CROSSBOWMEN-INDIA-AGE0`: lattice 1177, 491, 223 (115 launches).
    (
        1882,
        crate::units::anim::ATTACK1,
        5,
        Bay {
            right: 11_856,
            fwd: 87_864,
            dz: 173,
        },
    ),
    (
        1882,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: 15_464,
            fwd: 67_070,
            dz: 140,
        },
    ),
    (
        1882,
        crate::units::anim::ATTACK3,
        6,
        Bay {
            right: 15_879,
            fwd: 78_459,
            dz: 168,
        },
    ),
    // Piece 1977, the India Bombard, `BOMBARD-INDIA-AGE0`: lattice 924 (13 launches).
    (
        1977,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -710,
            fwd: 147_904,
            dz: 104,
        },
    ),
    // Piece 6706, the packed Arab Partisans, `PARTISANS-ARAB-AGE0-PACKED`: lattice 751, 1022, 751 (5 launches).
    (
        6706,
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 11_953,
            fwd: 105_627,
            dz: 170,
        },
    ),
    (
        6706,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: 5_193,
            fwd: 118_936,
            dz: 124,
        },
    ),
    (
        6706,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: 9_938,
            fwd: 105_144,
            dz: 170,
        },
    ),
    // **The pivot pieces' events past their restriction count** (item
    // 4159r5): a node no restriction governs takes `get_position`'s
    // non-pivot arm, so its point is a `BAYS` row like any other. Piece
    // 207, the T80DES (four node-1 events, run1579's packet, `RData +0x88`
    // 0.75), 232, the Flak Gun (four, read off run1164's packet, which holds
    // its seventeen rows; run1579 lacks its `CHAR_ATTACKWALK` and stops in
    // `ModManager::prepend_content_dir`), and 234, the Avenger (one,
    // run1579). Each `fit_bay.py` sweep 361 of 361; the centroid is kept.
    (
        207,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 15_112,
            fwd: 148_429,
            dz: 130,
        },
    ),
    (
        207,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 15_129,
            fwd: 149_061,
            dz: 126,
        },
    ),
    (
        207,
        crate::units::anim::ATTACK3,
        1,
        Bay {
            right: 15_112,
            fwd: 148_429,
            dz: 130,
        },
    ),
    (
        207,
        crate::units::anim::ATTACK3,
        7,
        Bay {
            right: 23_807,
            fwd: 147_942,
            dz: 126,
        },
    ),
    (
        232,
        crate::units::anim::ATTACKWALK,
        14,
        Bay {
            right: -26_034,
            fwd: -116_044,
            dz: 180,
        },
    ),
    (
        232,
        crate::units::anim::ATTACK1,
        14,
        Bay {
            right: -26_034,
            fwd: -116_044,
            dz: 180,
        },
    ),
    (
        232,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: -21_206,
            fwd: -111_768,
            dz: 172,
        },
    ),
    (
        232,
        crate::units::anim::ATTACK3,
        17,
        Bay {
            right: -4_043,
            fwd: -111_887,
            dz: 177,
        },
    ),
    (
        234,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 65_333,
            fwd: -25_391,
            dz: 173,
        },
    ),
    // **Item 4574's pivot pieces' node-1 events** (`docs/COMBAT.md` §55): a
    // node past the restriction count takes `get_position`'s non-pivot arm,
    // as 4159r5's. Piece 206, the Main Battle Tank (`T80`, two events,
    // run785's packet, `RData +0x88` 0.75): its sweeps are 207's rows' to
    // the heading (478 and 1,118 points; 207's centroids, `--check` 361 of
    // 361). Piece 233, the Anti-Aircraft Battery (two, run1416's packet,
    // 0.8): 62 and 3,594 points, the centroid kept; row 93.
    (
        206,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 15_112,
            fwd: 148_429,
            dz: 130,
        },
    ),
    (
        206,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 15_129,
            fwd: 149_061,
            dz: 126,
        },
    ),
    (
        233,
        crate::units::anim::ATTACKWALK,
        14,
        Bay {
            right: 0,
            fwd: -83_842,
            dz: 202,
        },
    ),
    (
        233,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 0,
            fwd: -81_331,
            dz: 199,
        },
    ),
    // **Item 4574's plain pieces** (`docs/COMBAT.md` §55; DECISIONS 153 §5),
    // keys the sixth held-out battery fired from the figure's square, each
    // after its row's first parting. `get_position.py … --init
    // 0,<anim>,<time>,3 sweep <piece> 0 <anim> <time>` on run1579's packet
    // fills each key, and `fit_bay.py` admits one region a key, 361 of 361
    // headings, its centroid inside; the heights are the sweep's. The packets
    // that store a piece agree with run1579's at every heading: run1155c
    // (67), run1416 (222), run785 (1133) and run854 (1485). No packet of the
    // 145 on disk stores 1131.
    //
    // Piece 67, the Manchu Riflemen (`MANCHURIFLEMEN-DEFAULT-AGE0`, type
    // 117, `manchu_rifleman.bh3`): node 0 at frame 10 of `CHAR_ATTACK1`, 15
    // of `2` and 16 of `3` (34, 53 and 1,181 lattice points); row 73.
    (
        67,
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 14_871,
            fwd: 132_717,
            dz: 168,
        },
    ),
    (
        67,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: 4698,
            fwd: 145_145,
            dz: 122,
        },
    ),
    (
        67,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: 12_672,
            fwd: 131_919,
            dz: 168,
        },
    ),
    // Piece 222, the Howitzer (`HUMMEL-DEFAULT-AGE0`, type 272,
    // `hummel.bh3`, `RData +0x88` 0.8): frames 1, 26 and 8 (111, 806 and
    // 1,120 points), on the barrel's line; row 93.
    (
        222,
        crate::units::anim::ATTACK1,
        1,
        Bay {
            right: 0,
            fwd: 84_150,
            dz: 310,
        },
    ),
    (
        222,
        crate::units::anim::ATTACK2,
        26,
        Bay {
            right: 0,
            fwd: 128_733,
            dz: 253,
        },
    ),
    (
        222,
        crate::units::anim::ATTACK3,
        8,
        Bay {
            right: 0,
            fwd: 92_200,
            dz: 322,
        },
    ),
    // Piece 1131, the Asian Heavy Machine Gun (`HVYMACHINEGUN-ASIAN-AGE0`,
    // type 125, `machinegun_gunner_asian.bh3`): node 0 on all of its 29 keys
    // (two events share frame 12 of `CHAR_ATTACK3`), 23 to 3,260 points a
    // region; row 93. Its `CHAR_ATTACK1` frame 3 is piece 73's row.
    (
        1131,
        crate::units::anim::ATTACK1,
        3,
        Bay {
            right: -7425,
            fwd: 147_384,
            dz: 93,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 88,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -16_551,
            fwd: 144_695,
            dz: 91,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        20,
        Bay {
            right: 752,
            fwd: 146_479,
            dz: 84,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        24,
        Bay {
            right: 13_866,
            fwd: 144_797,
            dz: 89,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        29,
        Bay {
            right: 18_457,
            fwd: 146_563,
            dz: 86,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        32,
        Bay {
            right: 18_306,
            fwd: 144_313,
            dz: 83,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        37,
        Bay {
            right: 4674,
            fwd: 148_076,
            dz: 90,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_935,
            dz: 91,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_376,
            dz: 88,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 88,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -402,
            fwd: 146_341,
            dz: 88,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -1420,
            fwd: 148_009,
            dz: 90,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: -2732,
            fwd: 145_714,
            dz: 93,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -5603,
            fwd: 144_577,
            dz: 98,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -7510,
            fwd: 145_038,
            dz: 103,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -5569,
            fwd: 143_666,
            dz: 103,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: -3051,
            fwd: 145_130,
            dz: 103,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 2021,
            fwd: 145_012,
            dz: 103,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        3,
        Bay {
            right: 1011,
            fwd: 144_798,
            dz: 91,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 1830,
            fwd: 143_956,
            dz: 92,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 11_993,
            fwd: 147_443,
            dz: 85,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        19,
        Bay {
            right: 18_422,
            fwd: 144_413,
            dz: 88,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        20,
        Bay {
            right: 18_428,
            fwd: 146_106,
            dz: 85,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        29,
        Bay {
            right: 1830,
            fwd: 146_325,
            dz: 90,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 146_847,
            dz: 90,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_400,
            dz: 88,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        42,
        Bay {
            right: 0,
            fwd: 145_708,
            dz: 88,
        },
    ),
    (
        1131,
        crate::units::anim::ATTACK3,
        46,
        Bay {
            right: 0,
            fwd: 146_272,
            dz: 88,
        },
    ),
    // Piece 1133, the Asian Advanced Machine Gun (`ADVMACHINEGUN-ASIAN-AGE0`,
    // type 127, `minigun_gunner_norm.bh3`): the one key row 133 fired,
    // `CHAR_ATTACK1` frame 3 (244 points), 1131's to the point.
    (
        1133,
        crate::units::anim::ATTACK1,
        3,
        Bay {
            right: -7425,
            fwd: 147_384,
            dz: 93,
        },
    ),
    // Piece 1485, the Native American Advanced Machine Gun
    // (`ADVMACHINEGUN-NA-AGE0`, type 127, `minigun_gunner_na.bh3`): its 19
    // keys of `CHAR_ATTACK2` and `3` (56 to 3,081 points); row 133. Its
    // `CHAR_ATTACK2` regions are 1131's to the point.
    (
        1485,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -402,
            fwd: 146_341,
            dz: 88,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -1420,
            fwd: 148_009,
            dz: 90,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: -2732,
            fwd: 145_714,
            dz: 93,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -5603,
            fwd: 144_577,
            dz: 98,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -7510,
            fwd: 145_038,
            dz: 103,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -5569,
            fwd: 143_666,
            dz: 103,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: -3051,
            fwd: 145_130,
            dz: 103,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 2021,
            fwd: 145_012,
            dz: 103,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        3,
        Bay {
            right: 2079,
            fwd: 146_008,
            dz: 91,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 2774,
            fwd: 145_170,
            dz: 93,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 13_610,
            fwd: 147_369,
            dz: 85,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        19,
        Bay {
            right: 18_423,
            fwd: 144_252,
            dz: 89,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        20,
        Bay {
            right: 18_424,
            fwd: 146_258,
            dz: 85,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: 7504,
            fwd: 146_012,
            dz: 89,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        29,
        Bay {
            right: 1218,
            fwd: 146_171,
            dz: 90,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 90,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_187,
            dz: 88,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        42,
        Bay {
            right: 0,
            fwd: 145_525,
            dz: 88,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK3,
        46,
        Bay {
            right: 0,
            fwd: 146_477,
            dz: 88,
        },
    ),
    // **Item 4625: piece 292, the Carrack** (`GALLEON-DEFAULT-AGE0`,
    // `Carrack.bh3`, unit type 342, sea; `docs/RECGAME.md` §14.4). Its
    // `<UNIT>` gives nine `RELEASEEVENT`s, one a node and attack anim, all
    // on the starboard side. Row 88 of the seventh held-out battery fired
    // who=1's `1/82` from the figure's square on 2790, where the original's
    // round left (42, 89) to its right at `sz` 35 (run1818's ranges, ammo
    // record 0). `get_position.py … --init <node>,<anim>,<time>,3 sweep 292
    // <node> <anim> <time>` on run1579's packet fills each key, and
    // `fit_bay.py` admits one region a key, 361 of 361 headings, its
    // centroid kept (109 to 4,321 lattice points), but for node 2's
    // `CHAR_ATTACK2` frame 24: **no point reproduces all 361 headings**,
    // and the row is the point that misses one, heading 283 (sweep
    // `(-127, 70)`, ours `(-127, 71)`). The heights are the sweep's.
    (
        292,
        crate::units::anim::ATTACK1,
        29,
        Bay {
            right: 99_202,
            fwd: 88_805,
            dz: 35,
        },
    ),
    (
        292,
        crate::units::anim::ATTACK1,
        34,
        Bay {
            right: 100_129,
            fwd: -27_681,
            dz: 30,
        },
    ),
    (
        292,
        crate::units::anim::ATTACK1,
        19,
        Bay {
            right: 97_830,
            fwd: -68_353,
            dz: 37,
        },
    ),
    (
        292,
        crate::units::anim::ATTACK2,
        36,
        Bay {
            right: 100_816,
            fwd: 52_335,
            dz: 30,
        },
    ),
    (
        292,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 98_551,
            fwd: 12_974,
            dz: 35,
        },
    ),
    (
        292,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 97_789,
            fwd: -107_946,
            dz: 36,
        },
    ),
    (
        292,
        crate::units::anim::ATTACK3,
        34,
        Bay {
            right: 100_386,
            fwd: 86_474,
            dz: 32,
        },
    ),
    (
        292,
        crate::units::anim::ATTACK3,
        32,
        Bay {
            right: 99_614,
            fwd: 12_048,
            dz: 32,
        },
    ),
    (
        292,
        crate::units::anim::ATTACK3,
        29,
        Bay {
            right: 98_847,
            fwd: -108_246,
            dz: 32,
        },
    ),
    // **Item 4618, the pieces with no bay** (`docs/COMBAT.md` §55): every
    // key of `bay-probe-{fifth,sixth,seventh}.out` that takes this arm, each
    // fitted by `fit_bay.py` over 361 of 361 headings, the centroid kept
    // (sweeps under `~/ron-data/lab-experiments/2026-11-24-item-4618/`).
    // Piece 128, the Elite Longbowman (`LONGBOWMENELITE-DEFAULT-AGE0`, type 178,
    // `longbow_kings.bh3`): its six node-0 keys, row 88's 1,392 rounds;
    // run1579's `--init`, 52 to 609 points a region, piece 127's rows to the
    // thousandth.
    (
        128,
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: -3_362,
            fwd: 82_391,
            dz: 169,
        },
    ),
    (
        128,
        crate::units::anim::ATTACK1,
        22,
        Bay {
            right: -3_362,
            fwd: 82_391,
            dz: 169,
        },
    ),
    (
        128,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: -15_902,
            fwd: 81_966,
            dz: 137,
        },
    ),
    (
        128,
        crate::units::anim::ATTACK2,
        22,
        Bay {
            right: -3_362,
            fwd: 82_391,
            dz: 169,
        },
    ),
    (
        128,
        crate::units::anim::ATTACK3,
        10,
        Bay {
            right: 10_522,
            fwd: 70_329,
            dz: 185,
        },
    ),
    (
        128,
        crate::units::anim::ATTACK3,
        24,
        Bay {
            right: 4_877,
            fwd: 65_764,
            dz: 184,
        },
    ),
    // Piece 37, the Xopilli Atlatl (`XOPILLIATLATL-DEFAULT-AGE0`, type 87,
    // `Xopilli.bh3`): its three keys, row 148's 143 rounds; run1579's `--init`, 23 to
    // 442 points a region.
    (
        37,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 4_728,
            fwd: 152_809,
            dz: 72,
        },
    ),
    (
        37,
        crate::units::anim::ATTACK2,
        11,
        Bay {
            right: 49_097,
            fwd: 105_964,
            dz: 186,
        },
    ),
    (
        37,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 13_515,
            fwd: 141_259,
            dz: 132,
        },
    ),
    // Piece 1499, the Native American TOW (`TOW-NA-AGE0`, type 141,
    // `antitankmissile_na.bh3`): its three keys, row 133; run1819's packet
    // after `--init`, 114 to 220 points a region.
    (
        1499,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 32_413,
            fwd: 122_285,
            dz: 124,
        },
    ),
    (
        1499,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 34_422,
            fwd: 98_506,
            dz: 148,
        },
    ),
    (
        1499,
        crate::units::anim::ATTACK3,
        13,
        Bay {
            right: 48_103,
            fwd: 110_494,
            dz: 143,
        },
    ),
    // Piece 1133's other 30 keys, row 133 (run1819's packet after `--init`):
    // every sweep is 1485's stored rows' to the heading; 23 to 3,260 points
    // a region.
    (
        1133,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 88,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -16_551,
            fwd: 144_695,
            dz: 91,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -9_573,
            fwd: 146_738,
            dz: 90,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        20,
        Bay {
            right: 752,
            fwd: 146_479,
            dz: 84,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        24,
        Bay {
            right: 13_866,
            fwd: 144_797,
            dz: 89,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        29,
        Bay {
            right: 18_457,
            fwd: 146_563,
            dz: 86,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        32,
        Bay {
            right: 18_306,
            fwd: 144_313,
            dz: 83,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        37,
        Bay {
            right: 4_674,
            fwd: 148_076,
            dz: 90,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_935,
            dz: 91,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_376,
            dz: 88,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 88,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -402,
            fwd: 146_341,
            dz: 88,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -1_420,
            fwd: 148_009,
            dz: 90,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK2,
        3,
        Bay {
            right: -2_732,
            fwd: 145_714,
            dz: 93,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: -5_603,
            fwd: 144_577,
            dz: 98,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -7_510,
            fwd: 145_038,
            dz: 103,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: -5_569,
            fwd: 143_666,
            dz: 103,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: -3_051,
            fwd: 145_130,
            dz: 103,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK2,
        12,
        Bay {
            right: 2_021,
            fwd: 145_012,
            dz: 103,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        3,
        Bay {
            right: 2_079,
            fwd: 146_008,
            dz: 91,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 2_774,
            fwd: 145_170,
            dz: 93,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 13_610,
            fwd: 147_369,
            dz: 85,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        19,
        Bay {
            right: 18_423,
            fwd: 144_252,
            dz: 89,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        20,
        Bay {
            right: 18_424,
            fwd: 146_258,
            dz: 85,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: 7_504,
            fwd: 146_012,
            dz: 89,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        29,
        Bay {
            right: 1_218,
            fwd: 146_171,
            dz: 90,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        34,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 90,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        37,
        Bay {
            right: 0,
            fwd: 145_187,
            dz: 88,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        42,
        Bay {
            right: 0,
            fwd: 145_525,
            dz: 88,
        },
    ),
    (
        1133,
        crate::units::anim::ATTACK3,
        46,
        Bay {
            right: 0,
            fwd: 146_477,
            dz: 88,
        },
    ),
    // Piece 1485's twelve `CHAR_ATTACK1` keys, row 133: stored on run1819's
    // packet (the take's own game at 2375).
    (
        1485,
        crate::units::anim::ATTACK1,
        3,
        Bay {
            right: -7_425,
            fwd: 147_384,
            dz: 93,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        9,
        Bay {
            right: -16_600,
            fwd: 145_697,
            dz: 88,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: -16_551,
            fwd: 144_695,
            dz: 91,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -9_573,
            fwd: 146_738,
            dz: 90,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        20,
        Bay {
            right: 752,
            fwd: 146_479,
            dz: 84,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        24,
        Bay {
            right: 13_866,
            fwd: 144_797,
            dz: 89,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        29,
        Bay {
            right: 18_457,
            fwd: 146_563,
            dz: 86,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        32,
        Bay {
            right: 18_306,
            fwd: 144_313,
            dz: 83,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        37,
        Bay {
            right: 4_674,
            fwd: 148_076,
            dz: 90,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        39,
        Bay {
            right: 0,
            fwd: 147_935,
            dz: 91,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        46,
        Bay {
            right: 0,
            fwd: 146_376,
            dz: 88,
        },
    ),
    (
        1485,
        crate::units::anim::ATTACK1,
        47,
        Bay {
            right: 0,
            fwd: 147_224,
            dz: 88,
        },
    ),
    // Piece 206's two node-1 `CHAR_ATTACK3` keys, row 133 (run1579's `--init`):
    // the node past the T80's one restriction takes the non-pivot arm, and
    // each sweep is 207's row's to the heading (478 and 1,118 points).
    (
        206,
        crate::units::anim::ATTACK3,
        1,
        Bay {
            right: 15_112,
            fwd: 148_429,
            dz: 130,
        },
    ),
    (
        206,
        crate::units::anim::ATTACK3,
        7,
        Bay {
            right: 23_807,
            fwd: 147_942,
            dz: 126,
        },
    ),
    // **The bays' lister, item 4651** (`docs/COMBAT.md` §55): sixty-six rows
    // for nineteen pieces of a held piece's family (its type or its
    // `GRAPH`) whose every swept key no held row reproduces, each the
    // centroid of the region `fit_bay.py` admits over 361 of 361 headings of
    // `init_position` and `get_position` on run1579's packet.
    // type 68 PARTISANS DEFAULT: fitted, no held row reproduces it.
    (
        18,
        crate::units::anim::ATTACK1,
        14,
        Bay {
            right: 20_612,
            fwd: 90_178,
            dz: 163,
        },
    ),
    (
        18,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: 17_519,
            fwd: 75_785,
            dz: 133,
        },
    ),
    (
        18,
        crate::units::anim::ATTACK3,
        4,
        Bay {
            right: 14_855,
            fwd: 74_427,
            dz: 133,
        },
    ),
    // type 83 JAVELINEERS ARAB: fitted, no held row reproduces it.
    (
        385,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 35_119,
            fwd: 60_718,
            dz: 170,
        },
    ),
    (
        385,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: -1_547,
            fwd: 49_504,
            dz: 176,
        },
    ),
    (
        385,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: 19_209,
            fwd: 31_386,
            dz: 158,
        },
    ),
    // type 141 TOW ARAB: fitted, no held row reproduces it.
    (
        443,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: 27_881,
            fwd: 97_234,
            dz: 125,
        },
    ),
    (
        443,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 29_732,
            fwd: 76_384,
            dz: 143,
        },
    ),
    (
        443,
        crate::units::anim::ATTACK3,
        13,
        Bay {
            right: 40_449,
            fwd: 86_848,
            dz: 139,
        },
    ),
    // type 311 ATTACKHELICOPTER ARAB: fitted, no held row reproduces it.
    (
        613,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -72_559,
            fwd: 41_859,
            dz: 41,
        },
    ),
    (
        613,
        crate::units::anim::ATTACK2,
        7,
        Bay {
            right: 78_169,
            fwd: 27_468,
            dz: 41,
        },
    ),
    // type 69 SCOUT AMERICAN: fitted, no held row reproduces it.
    (
        723,
        crate::units::anim::ATTACK2,
        23,
        Bay {
            right: 6_506,
            fwd: 65_950,
            dz: 135,
        },
    ),
    // type 83 JAVELINEERS AMERICAN: fitted, no held row reproduces it.
    (
        737,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 32_282,
            fwd: 64_350,
            dz: 169,
        },
    ),
    (
        737,
        crate::units::anim::ATTACK2,
        6,
        Bay {
            right: -7_922,
            fwd: 46_634,
            dz: 173,
        },
    ),
    (
        737,
        crate::units::anim::ATTACK3,
        27,
        Bay {
            right: 15_707,
            fwd: 31_771,
            dz: 157,
        },
    ),
    // type 304 BOMBER AMERICAN: fitted, no held row reproduces it.
    (
        958,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -70_896,
            fwd: 67_418,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        5,
        Bay {
            right: 67_650,
            fwd: -16_399,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        8,
        Bay {
            right: -70_896,
            fwd: 67_418,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        13,
        Bay {
            right: 67_650,
            fwd: -16_399,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: -70_896,
            fwd: 67_418,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        19,
        Bay {
            right: 67_650,
            fwd: -16_399,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        21,
        Bay {
            right: -70_896,
            fwd: 67_418,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 67_650,
            fwd: -16_399,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        28,
        Bay {
            right: -70_896,
            fwd: 67_418,
            dz: -19,
        },
    ),
    (
        958,
        crate::units::anim::ATTACK2,
        31,
        Bay {
            right: 67_650,
            fwd: -16_399,
            dz: -19,
        },
    ),
    // type 68 PARTISANS ASIAN: fitted, no held row reproduces it.
    (
        1074,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 70_739,
            fwd: 56_268,
            dz: 116,
        },
    ),
    (
        1074,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 69_642,
            fwd: 60_181,
            dz: 108,
        },
    ),
    (
        1074,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 13_804,
            fwd: 89_764,
            dz: 164,
        },
    ),
    (
        1074,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 26_720,
            fwd: 20_701,
            dz: 218,
        },
    ),
    (
        1074,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 15_958,
            fwd: 92_251,
            dz: 122,
        },
    ),
    (
        1074,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 32_060,
            fwd: 51_679,
            dz: 126,
        },
    ),
    // type 69 SCOUT ASIAN: fitted, no held row reproduces it.
    (
        1075,
        crate::units::anim::ATTACK2,
        23,
        Bay {
            right: 10_865,
            fwd: 67_249,
            dz: 156,
        },
    ),
    // type 324 DROMON ASIAN: fitted, no held row reproduces it.
    (
        1330,
        crate::units::anim::ATTACKWALK,
        2,
        Bay {
            right: -149,
            fwd: 170_585,
            dz: 52,
        },
    ),
    (
        1330,
        crate::units::anim::ATTACKWALK,
        10,
        Bay {
            right: 1_115,
            fwd: -7_707,
            dz: 52,
        },
    ),
    (
        1330,
        crate::units::anim::ATTACKWALK,
        17,
        Bay {
            right: -1_718,
            fwd: 76_381,
            dz: 52,
        },
    ),
    (
        1330,
        crate::units::anim::ATTACK1,
        2,
        Bay {
            right: -149,
            fwd: 170_585,
            dz: 52,
        },
    ),
    (
        1330,
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 1_115,
            fwd: -7_707,
            dz: 52,
        },
    ),
    (
        1330,
        crate::units::anim::ATTACK1,
        17,
        Bay {
            right: -1_718,
            fwd: 76_381,
            dz: 52,
        },
    ),
    (
        1330,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -149,
            fwd: 170_585,
            dz: 52,
        },
    ),
    (
        1330,
        crate::units::anim::ATTACK2,
        10,
        Bay {
            right: 1_115,
            fwd: -7_707,
            dz: 52,
        },
    ),
    (
        1330,
        crate::units::anim::ATTACK2,
        17,
        Bay {
            right: -1_718,
            fwd: 76_381,
            dz: 52,
        },
    ),
    // type 69 SCOUT NA: fitted, no held row reproduces it.
    (
        1427,
        crate::units::anim::ATTACK2,
        23,
        Bay {
            right: 3_596,
            fwd: 76_500,
            dz: 168,
        },
    ),
    // type 100 RIFLEMEN NA: fitted, no held row reproduces it.
    (
        1458,
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 16_289,
            fwd: 121_420,
            dz: 170,
        },
    ),
    (
        1458,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: 6_158,
            fwd: 134_019,
            dz: 124,
        },
    ),
    (
        1458,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: 14_124,
            fwd: 120_779,
            dz: 170,
        },
    ),
    // type 137 ANTITANKRIFLE NA: fitted, no held row reproduces it.
    (
        1495,
        crate::units::anim::ATTACK1,
        18,
        Bay {
            right: 17_436,
            fwd: 180_450,
            dz: 60,
        },
    ),
    (
        1495,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 5_266,
            fwd: 179_652,
            dz: 63,
        },
    ),
    (
        1495,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: -2_040,
            fwd: 162_082,
            dz: 83,
        },
    ),
    // type 309 JETFIGHTERBOMBER NA: fitted, no held row reproduces it.
    (
        1667,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: -169_361,
            fwd: -93_784,
            dz: -70,
        },
    ),
    (
        1667,
        crate::units::anim::ATTACK2,
        9,
        Bay {
            right: 169_610,
            fwd: -93_782,
            dz: -71,
        },
    ),
    // type 69 SCOUT INDIA: fitted, no held row reproduces it.
    (
        1779,
        crate::units::anim::ATTACK2,
        23,
        Bay {
            right: 6_133,
            fwd: 77_710,
            dz: 171,
        },
    ),
    // type 98 ARQUEBUSIERS INDIA: fitted, no held row reproduces it.
    (
        1808,
        crate::units::anim::ATTACK1,
        6,
        Bay {
            right: 17_748,
            fwd: 141_904,
            dz: 189,
        },
    ),
    (
        1808,
        crate::units::anim::ATTACK2,
        25,
        Bay {
            right: 42_168,
            fwd: 116_515,
            dz: 168,
        },
    ),
    (
        1808,
        crate::units::anim::ATTACK3,
        23,
        Bay {
            right: 27_266,
            fwd: 115_577,
            dz: 175,
        },
    ),
    // type 68 PARTISANS DEFAULT gender: fitted, no held row reproduces it.
    (
        6354,
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 10_733,
            fwd: 107_147,
            dz: 158,
        },
    ),
    (
        6354,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: 2_908,
            fwd: 120_340,
            dz: 113,
        },
    ),
    (
        6354,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: 9_012,
            fwd: 106_658,
            dz: 158,
        },
    ),
    // type 68 PARTISANS ASIAN gender: fitted, no held row reproduces it.
    (
        7410,
        crate::units::anim::ATTACK1,
        12,
        Bay {
            right: 68_178,
            fwd: 62_598,
            dz: 119,
        },
    ),
    (
        7410,
        crate::units::anim::ATTACK1,
        13,
        Bay {
            right: 67_227,
            fwd: 66_320,
            dz: 112,
        },
    ),
    (
        7410,
        crate::units::anim::ATTACK2,
        4,
        Bay {
            right: 7_557,
            fwd: 92_745,
            dz: 165,
        },
    ),
    (
        7410,
        crate::units::anim::ATTACK2,
        14,
        Bay {
            right: 21_561,
            fwd: 24_774,
            dz: 219,
        },
    ),
    (
        7410,
        crate::units::anim::ATTACK2,
        24,
        Bay {
            right: 10_083,
            fwd: 94_311,
            dz: 123,
        },
    ),
    (
        7410,
        crate::units::anim::ATTACK3,
        12,
        Bay {
            right: 30_445,
            fwd: 58_511,
            dz: 130,
        },
    ),
    // type 68 PARTISANS NA gender: fitted, no held row reproduces it.
    (
        7762,
        crate::units::anim::ATTACK1,
        10,
        Bay {
            right: 10_953,
            fwd: 117_837,
            dz: 161,
        },
    ),
    (
        7762,
        crate::units::anim::ATTACK2,
        15,
        Bay {
            right: 3_064,
            fwd: 131_119,
            dz: 115,
        },
    ),
    (
        7762,
        crate::units::anim::ATTACK3,
        16,
        Bay {
            right: 9_084,
            fwd: 117_244,
            dz: 161,
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
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 23_033,
            fwd: 346_844,
            dz: -188,
        },
    ),
    (
        289,
        crate::units::anim::ATTACK2,
        2,
        Bay {
            right: -23_033,
            fwd: 346_844,
            dz: -188,
        },
    ),
    (
        239,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -74_266,
            fwd: 39_505,
            dz: 1,
        },
    ),
    (
        239,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -13_010,
            fwd: 101_270,
            dz: 39,
        },
    ),
    (
        237,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: 184_080,
            fwd: -126_691,
            dz: -36,
        },
    ),
    (
        245,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -106_967,
            fwd: 34_296,
            dz: -26,
        },
    ),
    (
        258,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: 184_664,
            fwd: -132_488,
            dz: -71,
        },
    ),
    (
        259,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -93_878,
            fwd: -57_148,
            dz: 0,
        },
    ),
    (
        255,
        crate::units::anim::ATTACK2,
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
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -13_010,
            fwd: 101_270,
            dz: 39,
        },
    ),
    (
        589,
        crate::units::anim::ATTACK2,
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
    // from (13638, 16029, 371) on 633.
    //
    // **Both rows are fitted at every heading** (item 4663, `docs/COMBAT.md`
    // §55.6): item 3804's rows were the centres of the unit squares that
    // one heading truncates to, (−72,500, 41,500) and (78,500, 27,500),
    // and reproduced 201 and 197 of the original's 361. run1797's
    // `1/66` (row 133) faces 319° on 2546 and fired node 0's round from
    // x 17415 where theirs left from 17414. `tools/recomp/sweep_keys.py`
    // on run1579's packet (`init_position`, then `get_position` at every
    // whole degree) and `fit_bay.py`'s centroids: regions of 360 and 914
    // points, each row 361 of 361.
    (
        261,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -72_559,
            fwd: 41_859,
            dz: 41,
        },
    ),
    (
        261,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 78_169,
            fwd: 27_468,
            dz: 41,
        },
    ),
    // **The planes of item 4159r5** whose every event on a node leaves one
    // point (`BAYS`' comment above says how they were measured): the
    // Advanced Fighter fires both guns on frame 1, so `BAYS`' frame key
    // cannot hold it.
    // Piece 251, the Advanced Fighter, `ADVFIGHTER-DEFAULT-AGE0`: 2 events, one point a node (18 launches).
    (
        251,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: 23_349,
            fwd: 12_939,
            dz: -49,
        },
    ),
    (
        251,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: -23_311,
            fwd: 11_954,
            dz: -49,
        },
    ),
    // Piece 257, the Lancaster, `LANCASTERBOMBER-DEFAULT-AGE0`: 10 events, one point a node (11 launches).
    (
        257,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -145_105,
            fwd: 57_416,
            dz: -30,
        },
    ),
    (
        257,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 118_560,
            fwd: 56_488,
            dz: -30,
        },
    ),
    // Piece 1311, the Asian Strategic Bomber, `STRATEGICBOMBER-ASIAN-AGE0`: 12 events, one point a node (206 launches).
    (
        1311,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -93_863,
            fwd: -159_157,
            dz: 0,
        },
    ),
    (
        1311,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 93_870,
            fwd: -156_288,
            dz: 0,
        },
    ),
    // **Pieces 242 and 243, the German and Asian Fighters' two guns**
    // (`ME109-DEFAULT-AGE0` and `FIGHTERZERO-DEFAULT-AGE0`, types 292 and
    // 293, `RData +0x88` 0.765; item 4574, `docs/COMBAT.md` §55): row 93.
    // Each fires `CHAR_ATTACK2`'s eight frames (1, 3, 7, 11, 14, 18, 22 and
    // 26) on node 0 and on node 1, and run1579's `--init` writes one point a
    // node at all eight — 242's (4.27, −14.23, 10.03) and (−4.43, −14.23,
    // 10.03), 243's (25.24, −9.40, −1.91) and (−24.69, −9.40, −1.91) —
    // which run1417's and run1424's stored rows are bit for bit.
    // `fit_bay.py` fits each node's sixteen sweeps alike (242: 225 and 462
    // points; 243: 396 and 194; the centroid is kept), 36 over the figure
    // and 7 under it.
    (
        242,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -15_671,
            fwd: 52_259,
            dz: 36,
        },
    ),
    (
        242,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 16_250,
            fwd: 52_253,
            dz: 36,
        },
    ),
    (
        243,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -92_688,
            fwd: 34_516,
            dz: -7,
        },
    ),
    (
        243,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 90_656,
            fwd: 34_518,
            dz: -7,
        },
    ),
    // **Piece 238, the Albatross's two guns** (`ALBATROSS-DEFAULT-AGE0`, type
    // 288, `biplane_alb.bh3`; item 4618): row 83 (the fifth). Each node's
    // eight `CHAR_ATTACK2` events (frames 1 to 26) sweep to one point, 361
    // of 361 headings on run1579's `--init`, so `BAYS`' frame key cannot
    // hold the pair.
    (
        238,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -18_701,
            fwd: 132_451,
            dz: 27,
        },
    ),
    (
        238,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 17_643,
            fwd: 132_452,
            dz: 27,
        },
    ),
    // **The Fighter's Native American and Indian sets** (pieces 1647 and 1999,
    // item 4651): each node's eight `CHAR_ATTACK2` events sweep to one
    // point on run1579's `init_position`, as 239's do, at another point.
    (
        1647,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -85_887,
            fwd: 5_684,
            dz: 1,
        },
    ),
    (
        1647,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 85_370,
            fwd: 5_701,
            dz: 1,
        },
    ),
    (
        1999,
        crate::units::anim::ATTACK2,
        0,
        Bay {
            right: -74_245,
            fwd: 39_501,
            dz: 1,
        },
    ),
    (
        1999,
        crate::units::anim::ATTACK2,
        1,
        Bay {
            right: 73_441,
            fwd: 39_521,
            dz: 1,
        },
    ),
];

/// **A piece that releases from another's rows** (item 4651): `(piece,
/// source)`, where every `<RELEASEEVENT>` key of `piece` was swept with
/// `init_position` and `get_position` on run1579's packet (584's walk key
/// on run1816's) and `source`'s [`BAYS`] or [`GUNS`] row at the same key
/// reproduces it at 361 of 361 headings. The bays' lister
/// (`rondata::artdata`'s
/// `every_release_of_a_bay_piece_s_family_has_its_row`) named the keys:
/// another art set, age, gender or type of a piece the crate already held,
/// which no capture had fired. A source of another type (an elite, or a
/// `GRAPH` shared) agrees there to the unit; a piece here holds no row of
/// its own (`tests::a_piece_that_reads_another_s_bays_holds_none`).
const SAME_AS: &[(i32, i32)] = &[
    (19, 371),    // type 69 SCOUT DEFAULT as type 69 SCOUT ARAB
    (75, 73),     // type 125 HVYMACHINEGUN DEFAULT as type 123 MACHINEGUN DEFAULT
    (327, 294),   // type 377 MANOWAR DEFAULT as type 344 MANOWAR DEFAULT
    (330, 52),    // type 380 INFANTRY DEFAULT as type 102 INFANTRY DEFAULT
    (331, 52),    // type 381 INFANTRY DEFAULT as type 102 INFANTRY DEFAULT
    (332, 52),    // type 382 INFANTRY DEFAULT as type 102 INFANTRY DEFAULT
    (333, 52),    // type 383 INFANTRY DEFAULT as type 102 INFANTRY DEFAULT
    (336, 49),    // type 386 MUSKETEERS DEFAULT as type 99 MUSKETEERS DEFAULT
    (338, 251),   // type 388 ADVFIGHTER DEFAULT as type 301 ADVFIGHTER DEFAULT
    (346, 293),   // type 396 FRIGATE DEFAULT as type 343 FRIGATE DEFAULT
    (370, 6706),  // type 68 PARTISANS ARAB as type 68 PARTISANS ARAB gender
    (386, 1090),  // type 84 PELTASTS ARAB as type 84 PELTASTS ASIAN
    (400, 48),    // type 98 ARQUEBUSIERS ARAB as type 98 ARQUEBUSIERS DEFAULT
    (401, 753),   // type 99 MUSKETEERS ARAB as type 99 MUSKETEERS AMERICAN
    (402, 50),    // type 100 RIFLEMEN ARAB as type 100 RIFLEMEN DEFAULT
    (404, 57),    // type 102 INFANTRY ARAB as type 107 REDGUARD DEFAULT
    (406, 1462),  // type 104 ASSAULTINFANTRY ARAB as type 104 ASSAULTINFANTRY NA
    (425, 73),    // type 123 MACHINEGUN ARAB as type 123 MACHINEGUN DEFAULT
    (427, 73),    // type 125 HVYMACHINEGUN ARAB as type 123 MACHINEGUN DEFAULT
    (429, 1133),  // type 127 ADVMACHINEGUN ARAB as type 127 ADVMACHINEGUN ASIAN
    (438, 86),    // type 136 FUSILIER ARAB as type 136 FUSILIER DEFAULT
    (439, 87),    // type 137 ANTITANKRIFLE ARAB as type 137 ANTITANKRIFLE DEFAULT
    (474, 122),   // type 172 CROSSBOWMEN ARAB as type 172 CROSSBOWMEN DEFAULT
    (569, 217),   // type 267 BOMBARD ARAB as type 267 BOMBARD DEFAULT
    (570, 218),   // type 268 CANNON ARAB as type 268 CANNON DEFAULT
    (571, 219),   // type 269 ARTILLERY ARAB as type 269 ARTILLERY DEFAULT
    (584, 232),   // type 282 FLAKGUN ARAB as type 282 FLAKGUN DEFAULT
    (586, 234),   // type 284 AVENGER ARAB as type 284 AVENGER DEFAULT
    (682, 57),    // type 380 INFANTRY ARAB as type 107 REDGUARD DEFAULT
    (683, 57),    // type 381 INFANTRY ARAB as type 107 REDGUARD DEFAULT
    (684, 57),    // type 382 INFANTRY ARAB as type 107 REDGUARD DEFAULT
    (685, 57),    // type 383 INFANTRY ARAB as type 107 REDGUARD DEFAULT
    (688, 753),   // type 386 MUSKETEERS ARAB as type 99 MUSKETEERS AMERICAN
    (738, 737),   // type 84 PELTASTS AMERICAN as type 83 JAVELINEERS AMERICAN
    (781, 1133),  // type 127 ADVMACHINEGUN AMERICAN as type 127 ADVMACHINEGUN ASIAN
    (922, 218),   // type 268 CANNON AMERICAN as type 268 CANNON DEFAULT
    (1040, 753),  // type 386 MUSKETEERS AMERICAN as type 99 MUSKETEERS AMERICAN
    (1106, 67),   // type 100 RIFLEMEN ASIAN as type 117 MANCHURIFLEMEN DEFAULT
    (1129, 73),   // type 123 MACHINEGUN ASIAN as type 123 MACHINEGUN DEFAULT
    (1329, 273),  // type 323 BARK ASIAN as type 323 BARK DEFAULT
    (1346, 290),  // type 340 TRIREME ASIAN as type 340 TRIREME DEFAULT
    (1347, 291),  // type 341 GALLEY ASIAN as type 341 GALLEY DEFAULT
    (1349, 293),  // type 343 FRIGATE ASIAN as type 343 FRIGATE DEFAULT
    (1350, 294),  // type 344 MANOWAR ASIAN as type 344 MANOWAR DEFAULT
    (1383, 294),  // type 377 MANOWAR ASIAN as type 344 MANOWAR DEFAULT
    (1386, 52),   // type 380 INFANTRY ASIAN as type 102 INFANTRY DEFAULT
    (1387, 52),   // type 381 INFANTRY ASIAN as type 102 INFANTRY DEFAULT
    (1388, 52),   // type 382 INFANTRY ASIAN as type 102 INFANTRY DEFAULT
    (1389, 52),   // type 383 INFANTRY ASIAN as type 102 INFANTRY DEFAULT
    (1392, 49),   // type 386 MUSKETEERS ASIAN as type 99 MUSKETEERS DEFAULT
    (1402, 293),  // type 396 FRIGATE ASIAN as type 343 FRIGATE DEFAULT
    (1426, 18),   // type 68 PARTISANS NA as type 68 PARTISANS DEFAULT
    (1460, 52),   // type 102 INFANTRY NA as type 102 INFANTRY DEFAULT
    (1481, 73),   // type 123 MACHINEGUN NA as type 123 MACHINEGUN DEFAULT
    (1483, 73),   // type 125 HVYMACHINEGUN NA as type 123 MACHINEGUN DEFAULT
    (1497, 1849), // type 139 BAZOOKA NA as type 139 BAZOOKA INDIA
    (1530, 120),  // type 172 CROSSBOWMEN NA as type 170 BOWMEN DEFAULT
    (1624, 216),  // type 266 TREBUCHET NA as type 266 TREBUCHET DEFAULT
    (1627, 219),  // type 269 ARTILLERY NA as type 269 ARTILLERY DEFAULT
    (1738, 52),   // type 380 INFANTRY NA as type 102 INFANTRY DEFAULT
    (1739, 52),   // type 381 INFANTRY NA as type 102 INFANTRY DEFAULT
    (1740, 52),   // type 382 INFANTRY NA as type 102 INFANTRY DEFAULT
    (1741, 52),   // type 383 INFANTRY NA as type 102 INFANTRY DEFAULT
    (1744, 1457), // type 386 MUSKETEERS NA as type 99 MUSKETEERS NA
    (1778, 18),   // type 68 PARTISANS INDIA as type 68 PARTISANS DEFAULT
    (1810, 1458), // type 100 RIFLEMEN INDIA as type 100 RIFLEMEN NA
    (1833, 73),   // type 123 MACHINEGUN INDIA as type 123 MACHINEGUN DEFAULT
    (1835, 73),   // type 125 HVYMACHINEGUN INDIA as type 123 MACHINEGUN DEFAULT
    (1846, 1494), // type 136 FUSILIER INDIA as type 136 FUSILIER NA
    (1847, 1495), // type 137 ANTITANKRIFLE INDIA as type 137 ANTITANKRIFLE NA
    (1880, 1176), // type 170 BOWMEN INDIA as type 170 BOWMEN ASIAN
    (1978, 218),  // type 268 CANNON INDIA as type 268 CANNON DEFAULT
    (1979, 219),  // type 269 ARTILLERY INDIA as type 269 ARTILLERY DEFAULT
    (2090, 52),   // type 380 INFANTRY INDIA as type 102 INFANTRY DEFAULT
    (2091, 52),   // type 381 INFANTRY INDIA as type 102 INFANTRY DEFAULT
    (2092, 52),   // type 382 INFANTRY INDIA as type 102 INFANTRY DEFAULT
    (2093, 52),   // type 383 INFANTRY INDIA as type 102 INFANTRY DEFAULT
    (2096, 1809), // type 386 MUSKETEERS INDIA as type 99 MUSKETEERS INDIA
    (8114, 1074), // type 68 PARTISANS INDIA gender as type 68 PARTISANS ASIAN
];

/// The piece whose rows `gpiece` reads: [`SAME_AS`]'s, or its own.
fn rows_of(gpiece: i32) -> i32 {
    SAME_AS
        .iter()
        .find(|(p, _)| *p == gpiece)
        .map_or(gpiece, |&(_, q)| q)
}

/// The bay a piece releases from, or `None` when none is measured: a
/// [`BAYS`] row by the event's frame, or a [`GUNS`] row by its node,
/// read on [`rows_of`]'s piece.
pub fn bay(gpiece: i32, anim: i8, starttime: u32, node: i8) -> Option<Bay> {
    let gpiece = rows_of(gpiece);
    BAYS.iter()
        .find(|(p, a, t, _)| *p == gpiece && *a == anim && *t == starttime)
        .map(|(_, _, _, b)| *b)
        .or_else(|| {
            GUNS.iter()
                .find(|(p, a, n, _)| *p == gpiece && *a == anim && *n == node)
                .map(|(_, _, _, b)| *b)
        })
}

/// Every piece [`BAYS`] or [`GUNS`] holds a row for, ascending and once
/// each — the families the bays' lister walks (item 4651).
pub fn bay_pieces() -> Vec<i32> {
    let mut v: Vec<i32> = BAYS
        .iter()
        .map(|r| r.0)
        .chain(GUNS.iter().map(|r| r.0))
        .collect();
    v.sort_unstable();
    v.dedup();
    v
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
        None => {
            // SEAM: no row measures this key, so the round leaves the
            // figure's square (`docs/COMBAT.md` §55; items 4559, 4574).
            crate::seam!("fallback");
            pos
        }
    }
}

#[cfg(test)]
mod tests;
