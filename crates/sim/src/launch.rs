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
    // **49 is `MUSKETEERS`** (unit type 99; item 3336, run971 on the
    // Egyptian Great Sahara seed, `AMMO=5` over `[20368, 20380]`): the
    // Musketeer's two release events a round of run971 leaves from —
    // `1/136`'s `CHAR_ATTACK3` at 333 ms (frame 4) on tick 20374, guy
    // (5448, 26856) at angle −1656487936, `sx, sy, sz` (5333, 26970, 682)
    // over a guy `z` of 504; and `1/54`'s `CHAR_ATTACK1` at 0 ms (frame 1)
    // on tick 20378, guy (5544, 26808) at −1641873408, (5426, 26889, 713)
    // over 495. One shot each, so the bearing is any of the exact fits
    // (the middle one is kept) and the radius the only one there is;
    // `CHAR_ATTACK2`'s frame 7 is not measured.
    (
        49,
        crate::anim::ATTACK1,
        1,
        Node {
            bearing: 157_810_688,
            radius: 142,
            dz: 218,
        },
    ),
    (
        49,
        crate::anim::ATTACK3,
        4,
        Node {
            bearing: 49_020_928,
            radius: 161,
            dz: 178,
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
mod tests {
    use super::*;

    /// **The Royal Atl-Atls throw from piece 36's bays** (item 3613): all
    /// ten rounds run1266 and run1256b print from their first frame, each
    /// beside its figure's `GUY` pose on the block before, reproduced at the
    /// dump's `sx, sy` and `sz − z` by exactly one of the piece's three
    /// attack keys (the dump does not print which the figure played).
    #[test]
    fn run1266_s_aztec_javelins_leave_from_the_royal_atl_atls_bays() {
        // (x, y, facing, sx, sy, sz − z)
        let rows: [(i32, i32, i32, i32, i32, i32); 10] = [
            (6312, 10008, -125173760, 6327, 9920, 182),
            (6504, 9864, 29687808, 6540, 9742, 168),
            (6648, 9912, -57475072, 6685, 9802, 186),
            (7512, 9960, -112590848, 7541, 9848, 186),
            (7464, 9768, -101777408, 7476, 9641, 168),
            (7800, 9960, -280952832, 7801, 9844, 186),
            (6312, 10008, 145489920, 6359, 9932, 182),
            (7032, 9288, 70647808, 7072, 9208, 182),
            (7080, 8664, 0, 7129, 8559, 186),
            (6744, 8568, 179109888, 6795, 8495, 182),
        ];
        for (x, y, facing, sx, sy, dz) in rows {
            let keys: Vec<i8> = [11, 12, 13]
                .into_iter()
                .filter(|&anim| {
                    let at = launch_point(Pos::new(x, y), Angle(facing), 36, anim, 11, 0);
                    (at.x, at.y) == (sx, sy) && release_dz(36, anim, 11, 0) == Some(dz)
                })
                .collect();
            assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
        }
    }

    /// **The Mongol Archers and Javelineers loose from their bays** (item
    /// 3579): run1222's three rounds at `0/8` over 9694..9706, each the
    /// figure's `GUY` pose on the block before the round's first print, the
    /// key this crate's release event names, and the dump's `sx, sy, sz − z`.
    #[test]
    fn run1222_s_mongol_rounds_leave_from_their_bays() {
        // (piece, anim, starttime, x, y, facing, sx, sy, sz − z)
        type Row = (i32, i8, u32, i32, i32, i32, i32, i32, i32);
        let rows: [Row; 3] = [
            (1177, 13, 15, 7128, 27240, -1_790_246_912, 7087, 27292, 164),
            (1177, 11, 12, 6840, 27240, -1_755_316_224, 6795, 27316, 163),
            (1089, 12, 6, 7272, 27912, -1_382_547_456, 7227, 27933, 170),
        ];
        for (piece, anim, start, x, y, facing, sx, sy, dz) in rows {
            let at = launch_point(Pos::new(x, y), Angle(facing), piece, anim, start, 0);
            assert_eq!((at.x, at.y), (sx, sy), "{piece} ({anim}, {start})");
            assert_eq!(
                release_dz(piece, anim, start, 0),
                Some(dz),
                "{piece} ({anim}, {start})"
            );
        }
    }

    /// **The Greek Archers' arrows leave from piece 121's bays** (item
    /// 3574): run1217b's fifteen distinct `AMMO` poses over 7692..7848, each the
    /// figure on its unit's own square at the `GUY` record's angle, the
    /// key this crate's release event names, and `sx, sy` the dump's own.
    #[test]
    fn run1217b_s_greek_arrows_leave_from_the_archer_s_bays() {
        let rows: [(i8, u32, i32, i32, i32, i32, i32); 15] = [
            (12, 9, 8232, 8856, -229_310_464, 8203, 8774),
            (12, 9, 8040, 9144, -100_663_296, 8029, 9058),
            (12, 9, 8232, 8856, -243_597_312, 8201, 8775),
            (12, 9, 8040, 9144, 7_995_392, 8040, 9057),
            (12, 9, 7752, 9048, 115_474_432, 7764, 8962),
            (12, 9, 7752, 9048, 107_675_648, 7764, 8962),
            (12, 9, 8232, 8856, -60_686_336, 8224, 8770),
            (11, 12, 8328, 8712, -305_201_152, 8287, 8634),
            (11, 12, 8232, 8856, -68_419_584, 8220, 8769),
            (11, 12, 8040, 9144, 7_995_392, 8039, 9056),
            (13, 15, 8088, 9384, -104_660_992, 8087, 9317),
            (13, 15, 8424, 9240, -250_740_736, 8410, 9175),
            (13, 15, 7752, 9048, 20_709_376, 7762, 8982),
            (13, 15, 7752, 9048, 115_474_432, 7770, 8984),
            (13, 15, 8232, 8856, -68_419_584, 8233, 8789),
        ];
        for (anim, start, x, y, facing, sx, sy) in rows {
            let at = launch_point(Pos::new(x, y), Angle(facing), 121, anim, start, 0);
            assert_eq!((at.x, at.y), (sx, sy), "({anim}, {start}) at {facing}");
        }
        assert_eq!(release_dz(121, 12, 9, 0), Some(164));
        assert_eq!(release_dz(121, 11, 12, 0), Some(163));
    }

    /// **The Japanese Bowmen loose from piece 1176's bays** (item 3610):
    /// every Bowmen round run1264 and run1252b print, the first print beside
    /// the figure's `GUY` record on the block before — `x, y`, its angle,
    /// and the dump's `sx, sy` and `sz − z` — each reproduced exactly by
    /// one of the piece's three keys at the figure's own square.
    #[test]
    fn run1264_s_japanese_arrows_leave_from_the_bowmen_s_bays() {
        // (x, y, facing, sx, sy, sz − z)
        let rows: [(i32, i32, i32, i32, i32, i32); 17] = [
            (7608, 5784, -768_344_064, 7529, 5749, 160),
            (7080, 5544, -671_481_856, 7008, 5499, 160),
            (6936, 5688, -552_271_872, 6896, 5636, 161),
            (7080, 5544, -802_357_248, 7002, 5511, 160),
            (6936, 5688, -689_635_328, 6886, 5647, 161),
            (7368, 5256, -1_045_495_808, 7303, 5246, 161),
            (6936, 5688, -837_615_616, 6854, 5660, 160),
            (7080, 5544, -913_113_088, 6995, 5527, 160),
            (7080, 5544, -769_720_320, 7025, 5508, 161),
            (6456, 5400, -579_403_776, 6414, 5351, 161),
            (7080, 5544, -801_374_208, 7002, 5511, 160),
            (6456, 5400, -837_484_544, 6376, 5371, 160),
            (7560, 6216, -1_455_685_632, 7488, 6261, 160),
            (7128, 5784, 2_102_591_488, 7133, 5869, 160),
            (6888, 5688, 1_901_068_288, 6920, 5768, 160),
            (7560, 6216, 1_861_091_328, 7595, 6293, 160),
            (6888, 5688, 1_624_899_584, 6948, 5748, 160),
        ];
        let keys = [
            (crate::anim::ATTACK1, 12),
            (crate::anim::ATTACK2, 9),
            (crate::anim::ATTACK3, 15),
        ];
        for (x, y, facing, sx, sy, dz) in rows {
            let hit = keys.iter().find(|&&(a, t)| {
                let at = launch_point(Pos::new(x, y), Angle(facing), 1176, a, t, 0);
                (at.x, at.y) == (sx, sy) && release_dz(1176, a, t, 0) == Some(dz)
            });
            assert!(
                hit.is_some(),
                "({x}, {y}) at {facing}: no key reaches ({sx}, {sy}, {dz})"
            );
        }
    }

    /// **The Korean stone and the Hwarang's arrow leave from pieces 1088's
    /// and 130's bays** (item 3578): run1221's two `AMMO` records on block
    /// 4287 — the Slingers `1/29`'s stone at the Citizen `0/13`, thrown on
    /// 4286 on `CHAR_ATTACK3` at 16, and the Hwarang `1/44`'s arrow at the
    /// same Citizen, loosed on 4279 on `CHAR_ATTACK1` at 22 — each figure on
    /// its unit's own square at its facing, `sx, sy` the dump's own; the
    /// heights are the packet sweep's `z` (the dump's `sz` 264 and 193).
    #[test]
    fn run1221_s_korean_stone_and_arrow_leave_from_their_bays() {
        let stone = launch_point(
            Pos::new(6600, 9768),
            Angle(-124_059_648),
            1088,
            crate::anim::ATTACK3,
            16,
            0,
        );
        assert_eq!((stone.x, stone.y), (6541, 9663));
        assert_eq!(release_dz(1088, crate::anim::ATTACK3, 16, 0), Some(176));
        let arrow = launch_point(
            Pos::new(6072, 10728),
            Angle(163_053_568),
            130,
            crate::anim::ATTACK1,
            22,
            0,
        );
        assert_eq!((arrow.x, arrow.y), (6085, 10656));
        assert_eq!(release_dz(130, crate::anim::ATTACK1, 22, 0), Some(167));
    }

    /// **The Elite Hwarang's arrows leave from piece 131's bays** (item
    /// 3609): run1263's three `AMMO` records on block 8362, each the
    /// figure's pose on its release frame (`1/327` on 8354, `1/325` on
    /// 8355, `1/135` on 8361), the key this crate's release event names,
    /// the dump's `sx, sy`, and the sweep's `z`.
    #[test]
    fn run1263_s_elite_hwarang_arrows_leave_from_piece_131_s_bays() {
        // (anim, starttime, x, y, facing, sx, sy, dz)
        type Row = (i8, u32, i32, i32, i32, i32, i32, i32);
        let rows: [Row; 3] = [
            (13, 10, 9720, 30600, -665_780_224, 9675, 30558, 182),
            (13, 24, 9384, 31224, -397_213_696, 9356, 31175, 181),
            (12, 4, 9432, 28584, -1_585_250_304, 9389, 28645, 138),
        ];
        for (anim, start, x, y, facing, sx, sy, dz) in rows {
            let at = launch_point(Pos::new(x, y), Angle(facing), 131, anim, start, 0);
            assert_eq!((at.x, at.y), (sx, sy), "131 ({anim}, {start})");
            assert_eq!(
                release_dz(131, anim, start, 0),
                Some(dz),
                "131 ({anim}, {start})"
            );
        }
    }

    /// **The Lakota's Arquebusiers and Bombard leave from pieces 1456's and
    /// 1625's bays** (item 3645, `docs/AI.md` §336): run1289b's `AMMO`
    /// records beside each shooter's pose on the block before — the
    /// Arquebusiers `1/39`'s round on 4217, which one of 1456's three keys
    /// reproduces (the dump does not print the figure's anim), and the
    /// Bombard `1/58`'s on 4303, `CHAR_ATTACK2` at frame 9. 1456's rows are
    /// piece 48's to the digit: the same model.
    #[test]
    fn run1289b_s_lakota_arquebusiers_and_bombard_leave_from_their_bays() {
        let keys = [(11, 6), (12, 25), (13, 23)];
        let fits: Vec<_> = keys
            .iter()
            .filter(|&&(anim, start)| {
                let at = launch_point(
                    Pos::new(8520, 27864),
                    Angle(-1_529_217_024),
                    1456,
                    anim,
                    start,
                    0,
                );
                (at.x, at.y) == (8395, 27931)
            })
            .collect();
        assert_eq!(fits, [&(11, 6)], "1/39's round on 4217");
        for (anim, start) in keys {
            assert_eq!(bay(1456, anim, start, 0), bay(48, anim, start, 0));
        }
        let ball = launch_point(
            Pos::new(9000, 27144),
            Angle(-1_623_785_472),
            1625,
            crate::anim::ATTACK2,
            9,
            0,
        );
        assert_eq!((ball.x, ball.y), (8898, 27250), "1/58's round on 4303");
        assert_eq!(release_dz(1625, crate::anim::ATTACK2, 9, 0), Some(104));
    }

    /// **The Iroquois Musketeers leave from piece 1457's bays** (item 3655,
    /// `docs/AI.md` §337): run1320's `1/75` on block 5067, the figure at
    /// (7512, 8664, 298) and angle 426639360 on 5066, its round at
    /// (7614, 8539, 476) — `CHAR_ATTACK3`'s row, and no other key's.
    /// Made to fail with the rows out of `BAYS`: the round leaves the
    /// figure's own point at `z` 298.
    #[test]
    fn run1320_s_iroquois_musketeer_leaves_from_piece_1457_s_bay() {
        let keys = [(11, 1), (12, 7), (13, 4)];
        let fits: Vec<_> = keys
            .iter()
            .filter(|&&(anim, start)| {
                let at = launch_point(
                    Pos::new(7512, 8664),
                    Angle(426_639_360),
                    1457,
                    anim,
                    start,
                    0,
                );
                (at.x, at.y) == (7614, 8539)
            })
            .collect();
        assert_eq!(fits, [&(13, 4)], "1/75's round on 5067");
        assert_eq!(298 + release_dz(1457, 13, 4, 0).unwrap(), 476);
    }

    /// **The Inca Musketeers leave from piece 753's bays** (item 3659,
    /// `docs/AI.md` §339): run1323's `1/25` on block 3053, the figure at
    /// (7752, 30840, 640) and angle 67698688 on 3052, its round at
    /// (7795, 30717, 838) — `CHAR_ATTACK1`'s row, and no other key's.
    /// Made to fail with the rows out of `BAYS`: the round leaves the
    /// figure's own point at `z` 640.
    #[test]
    fn run1323_s_inca_musketeer_leaves_from_piece_753_s_bay() {
        let keys = [(11, 1), (12, 7), (13, 4)];
        let fits: Vec<_> = keys
            .iter()
            .filter(|&&(anim, start)| {
                let at = launch_point(
                    Pos::new(7752, 30840),
                    Angle(67_698_688),
                    753,
                    anim,
                    start,
                    0,
                );
                (at.x, at.y) == (7795, 30717)
            })
            .collect();
        assert_eq!(fits, [&(11, 1)], "1/25's round on 3053");
        assert_eq!(640 + release_dz(753, 11, 1, 0).unwrap(), 838);
    }

    /// **The Inca Fusiliers leave from piece 790's bays** (item 3659,
    /// `docs/AI.md` §339): run1323c's `1/44` on block 3815, the figure at
    /// (8136, 31080, 662) and angle −733020160 on 3814, its round at the
    /// capital from (8048, 31009, 772) — `CHAR_ATTACK1`'s row, and no
    /// other key's.
    #[test]
    fn run1323c_s_inca_fusilier_leaves_from_piece_790_s_bay() {
        let keys = [(11, 6), (12, 6), (13, 8)];
        let fits: Vec<_> = keys
            .iter()
            .filter(|&&(anim, start)| {
                let at = launch_point(
                    Pos::new(8136, 31080),
                    Angle(-733_020_160),
                    790,
                    anim,
                    start,
                    0,
                );
                (at.x, at.y) == (8048, 31009)
            })
            .collect();
        assert_eq!(fits, [&(11, 6)], "1/44's round on 3815");
        assert_eq!(662 + release_dz(790, 11, 6, 0).unwrap(), 772);
    }

    /// **The Man o' War broadsides from piece 294's bays** (item 3660,
    /// `docs/AI.md` §340): run1324's `1/97` on block 4082, the hull at
    /// (3000, 8472) and angle −354811904 on 4081, its round at the
    /// Citizen `0/13` from (2866, 8510, −33) — `CHAR_ATTACK2`'s row, and
    /// no other key's. Made to fail with the rows out of `BAYS`: the
    /// round leaves the hull's own square at `z` 0.
    #[test]
    fn run1324_s_man_o_war_broadsides_from_piece_294_s_bay() {
        let keys = [(11, 7), (12, 7)];
        let fits: Vec<_> = keys
            .iter()
            .filter(|&&(anim, start)| {
                let at = launch_point(
                    Pos::new(3000, 8472),
                    Angle(-354_811_904),
                    294,
                    anim,
                    start,
                    0,
                );
                (at.x, at.y) == (2866, 8510)
            })
            .collect();
        assert_eq!(fits, [&(12, 7)], "1/97's round on 4082");
        assert_eq!(release_dz(294, 12, 7, 0).unwrap(), -33);
    }

    /// **The Indian Musketeers leave from piece 1809's bays** (item 3660,
    /// `docs/AI.md` §340): run1324b's `1/80` on block 4905, the figure at
    /// (11256, 7608, 315) and angle −1409024000 on 4904, its round at the
    /// Citizen `0/10` from (11095, 7683, 489) — `CHAR_ATTACK2`'s row, and
    /// no other key's.
    #[test]
    fn run1324b_s_indian_musketeer_leaves_from_piece_1809_s_bay() {
        let keys = [(11, 1), (12, 7), (13, 4)];
        let fits: Vec<_> = keys
            .iter()
            .filter(|&&(anim, start)| {
                let at = launch_point(
                    Pos::new(11256, 7608),
                    Angle(-1_409_024_000),
                    1809,
                    anim,
                    start,
                    0,
                );
                (at.x, at.y) == (11095, 7683)
            })
            .collect();
        assert_eq!(fits, [&(12, 7)], "1/80's round on 4905");
        assert_eq!(315 + release_dz(1809, 12, 7, 0).unwrap(), 489);
    }

    /// **The Galley looses from piece 291's bays** (item 3662,
    /// `docs/AI.md` §342): run1316c's `1/29` on block 4298, the hull at
    /// (2472, 4200) and angle −1683357696, its round at the human's
    /// `0/2004` from (2690, 3917, 52). Made to fail with the rows out of
    /// `BAYS`: the round leaves the hull's own square at `z` 0.
    #[test]
    fn run1316c_s_galley_looses_from_piece_291_s_bay() {
        let keys = [(11, 9), (11, 12), (11, 17), (12, 9), (12, 12), (12, 17)];
        let fits: Vec<_> = keys
            .iter()
            .filter(|&&(anim, start)| {
                let at = launch_point(
                    Pos::new(2472, 4200),
                    Angle(-1_683_357_696),
                    291,
                    anim,
                    start,
                    0,
                );
                (at.x, at.y) == (2690, 3917) && release_dz(291, anim, start, 0) == Some(52)
            })
            .collect();
        assert_eq!(fits, [&(11, 9), &(11, 12), &(12, 9)], "1/29's round on 4298");
    }

    /// **The Egyptian Arquebusiers' rounds leave from piece 48's bays**
    /// (item 3603, `docs/AI.md` §324): run1254c's three `AMMO` records at
    /// the Citizen `0/9` on block 3115, each figure on its unit's own
    /// square at its facing, the key the art's release event names, and
    /// `sx, sy` the dump's own.
    #[test]
    fn run1254c_s_egyptian_arquebusiers_fire_from_piece_48_s_bays() {
        for (anim, start, x, y, facing, sx, sy, dz) in [
            (12, 25, 7560, 30840, -1_633_288_192, 7459, 30915, 159),
            (13, 23, 5784, 30552, 1_828_126_720, 5827, 30661, 166),
            (12, 25, 8424, 31320, -1_279_787_008, 8299, 31334, 159),
        ] {
            let at = launch_point(Pos::new(x, y), Angle(facing), 48, anim, start, 0);
            assert_eq!((at.x, at.y), (sx, sy), "({anim}, {start}) at {facing}");
            assert_eq!(release_dz(48, anim, start, 0), Some(dz));
        }
    }

    /// **The Balamob Slingers' stones leave from piece 38's bays** (item
    /// 3599, `docs/AI.md` §320.2): run1247's two `AMMO` records at the
    /// Citizen `0/10` on block 4306, each figure on its unit's own square
    /// at the `GUY` record's angle, `sx, sy` the dump's own and `sz` less
    /// the figure's `z` 194 naming the key.
    #[test]
    fn run1247_s_balamob_stones_leave_from_piece_38_s_bays() {
        for (anim, start, x, y, facing, sx, sy, dz) in [
            (12, 19, 6072, 8616, -756_678_656, 5950, 8535, 362 - 194),
            (11, 18, 6072, 8856, -643_039_232, 5963, 8757, 371 - 194),
        ] {
            let at = launch_point(Pos::new(x, y), Angle(facing), 38, anim, start, 0);
            assert_eq!((at.x, at.y), (sx, sy), "({anim}, {start}) at {facing}");
            assert_eq!(release_dz(38, anim, start, 0), Some(dz));
        }
    }

    /// **The Fire Lances' rounds leave from piece 46's bays** (item 3577):
    /// run1220's three `AMMO` records at the Citizen `0/10`, each the
    /// figure on its unit's own square at the `GUY` record's angle, the key
    /// the art's release event names, and `sx, sy` the dump's own, `sz` over
    /// the figure's `z`.
    #[test]
    fn run1220_s_fire_lances_leave_from_piece_46_s_bays() {
        for (anim, start, x, y, facing, sx, sy, dz) in [
            (12, 27, 9768, 30984, -1_127_809_024, 9690, 30990, 662 - 584),
            (13, 19, 9096, 31224, -1_013_448_704, 9038, 31212, 766 - 636),
            (11, 24, 9528, 30840, -1_204_224_000, 9462, 30840, 709 - 614),
        ] {
            let at = launch_point(Pos::new(x, y), Angle(facing), 46, anim, start, 0);
            assert_eq!((at.x, at.y), (sx, sy), "({anim}, {start}) at {facing}");
            assert_eq!(release_dz(46, anim, start, 0), Some(dz));
        }
    }

    /// **The Lakota Slingers' stones leave from piece 1440's bays** (item
    /// 3345): run980b's `AMMO` records, the stone `1/18` threw on 6046 at
    /// the citizen `0/2` and `1/20`'s on 6050 at the city centre — each
    /// figure on its unit's own square at its facing, `sx, sy` the dump's
    /// own.
    #[test]
    fn run980b_s_lakota_stones_leave_from_the_na_slinger_s_bays() {
        let at = |x, y, facing, key: (i8, u32)| {
            launch_point(Pos::new(x, y), Angle(facing), 1440, key.0, key.1, 0)
        };
        for (x, y, facing, want) in [
            (5016, 30264, -48_431_104, Pos::new(4962, 30153)),
            (5064, 30744, -1_097_334_784, Pos::new(4943, 30718)),
        ] {
            let keys = [
                (crate::anim::ATTACK1, 21),
                (crate::anim::ATTACK2, 22),
                (crate::anim::ATTACK3, 16),
            ];
            let got: Vec<_> = keys.iter().map(|&k| at(x, y, facing, k)).collect();
            assert!(got.contains(&want), "{want:?} from {got:?}");
        }
    }

    /// **The Persian Slingers' stones leave from piece 1792's bays** (item
    /// 3355): run984d's `AMMO` records, the stones `1/22` and `1/24` threw
    /// at the citizen `0/3` — each from its unit's own square at block
    /// 6317's facing, `sx, sy` the dump's own.
    #[test]
    fn run984d_s_persian_stones_leave_from_piece_1792_s_bays() {
        for (x, y, facing, want) in [
            (2616, 33000, -1_092_616_192, Pos::new(2504, 33053)),
            (2472, 32424, -1_435_041_792, Pos::new(2398, 32523)),
        ] {
            let got: Vec<_> = [
                (crate::anim::ATTACK1, 21),
                (crate::anim::ATTACK2, 22),
                (crate::anim::ATTACK3, 16),
            ]
            .iter()
            .map(|&(anim, start)| launch_point(Pos::new(x, y), Angle(facing), 1792, anim, start, 0))
            .collect();
            assert!(got.contains(&want), "{want:?} from {got:?}");
        }
    }

    /// **The Persian Immortals' arrows leave from piece 112's bays** (item
    /// 3355): run984's `AMMO` record of the arrow `1/16` threw on 6012 at
    /// the Scout `0/0` — `sx, sy` (3766, 32893), `total_time` 10 — from
    /// the unit's own square (3768, 33000) at block 6020's facing.
    #[test]
    fn run984_s_immortal_arrow_leaves_from_piece_112_s_bays() {
        let got: Vec<_> = [(crate::anim::ATTACK2, 14), (crate::anim::ATTACK3, 6)]
            .iter()
            .map(|&(anim, start)| {
                launch_point(
                    Pos::new(3768, 33000),
                    Angle(-173_146_112),
                    112,
                    anim,
                    start,
                    0,
                )
            })
            .collect();
        assert!(got.contains(&Pos::new(3766, 32893)), "{got:?}");
    }

    /// **The Chinese Anti-tank Rifles' rounds leave from piece 1143's
    /// bays** (item 3538): run1155c's packet, `get_position.py … sweep 1143
    /// 0 <anim> <time>`, at the four facings whose top byte is a whole
    /// degree — 0, `0x40`, `0xa0` and `0xe0`, the sweep's 180°, 270°, 45°
    /// and 135° — `(x, y)` off the figure. The muzzle is some 180 units
    /// ahead; from the figure's own square `1/41`'s round at `0/4` flew a
    /// frame long.
    #[test]
    fn run1155c_s_rifle_rounds_leave_from_piece_1143_s_bays() {
        for (anim, start, want) in [
            (
                crate::anim::ATTACK1,
                18,
                [(17, -180), (180, 17), (-139, 115), (-115, -139)],
            ),
            (
                crate::anim::ATTACK2,
                4,
                [(4, -179), (179, 4), (-130, 123), (-123, -130)],
            ),
            (
                crate::anim::ATTACK3,
                16,
                [(-2, -161), (161, -2), (-112, 116), (-116, -112)],
            ),
        ] {
            for (byte, (x, y)) in [0u32, 0x40, 0xa0, 0xe0].into_iter().zip(want) {
                let facing = Angle((byte << 24) as i32);
                let got = launch_point(Pos::new(10_000, 20_000), facing, 1143, anim, start, 0);
                assert_eq!(
                    got,
                    Pos::new(10_000 + x, 20_000 + y),
                    "anim {anim} at {start}, byte {byte:#x}"
                );
            }
        }
    }

    /// **The Highlanders' rounds leave from piece 64's bays** (item 3542):
    /// run1185's packet, `get_position.py … sweep 64 0 <anim> <time>`, at
    /// the four facings whose top byte is a whole degree, as 1143's.
    #[test]
    fn run1185_s_highlander_rounds_leave_from_piece_64_s_bays() {
        for (anim, start, want) in [
            (
                crate::anim::ATTACK1,
                6,
                [(13, -144), (144, 13), (-111, 92), (-92, -111)],
            ),
            (
                crate::anim::ATTACK2,
                10,
                [(13, -145), (145, 13), (-112, 93), (-93, -112)],
            ),
            (
                crate::anim::ATTACK3,
                12,
                [(13, -145), (145, 13), (-112, 93), (-93, -112)],
            ),
        ] {
            for (byte, (x, y)) in [0u32, 0x40, 0xa0, 0xe0].into_iter().zip(want) {
                let facing = Angle((byte << 24) as i32);
                let got = launch_point(Pos::new(10_000, 20_000), facing, 64, anim, start, 0);
                assert_eq!(
                    got,
                    Pos::new(10_000 + x, 20_000 + y),
                    "anim {anim} at {start}, byte {byte:#x}"
                );
            }
        }
    }

    /// **The Sloop's rounds leave from piece 277's bays** (item 3542):
    /// run1185's packet, `get_position.py … sweep 277 <node> <anim> 4`, at
    /// the four facings whose top byte is a whole degree, as 1143's; and
    /// the round `1/61` threw on 4580 from (3336, 6840) at its own facing
    /// leaves from (3315, 6861), the live `Ammo`'s `sx, sy` on the packet.
    #[test]
    fn run1185_s_sloop_rounds_leave_from_piece_277_s_bays() {
        assert_eq!(
            launch_point(
                Pos::new(3336, 6840),
                Angle(1_621_622_784),
                277,
                crate::anim::ATTACK1,
                4,
                0
            ),
            Pos::new(3315, 6861)
        );
        for (anim, node, want) in [
            (
                crate::anim::ATTACK1,
                0,
                [(29, 0), (0, 29), (-21, -21), (21, -21)],
            ),
            (
                crate::anim::ATTACK2,
                1,
                [(-98, 4), (-4, -98), (72, 66), (-66, 72)],
            ),
            (
                crate::anim::ATTACKWALK,
                1,
                [(-98, 4), (-4, -98), (72, 66), (-66, 72)],
            ),
        ] {
            for (byte, (x, y)) in [0u32, 0x40, 0xa0, 0xe0].into_iter().zip(want) {
                let facing = Angle((byte << 24) as i32);
                let got = launch_point(Pos::new(10_000, 20_000), facing, 277, anim, 4, node);
                assert_eq!(
                    got,
                    Pos::new(10_000 + x, 20_000 + y),
                    "anim {anim} byte {byte:#x}"
                );
            }
        }
    }

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
            let p = launch_point(Pos::new(x, y), Angle(a), 254, crate::anim::ATTACK2, t, 0);
            assert_eq!((p.x, p.y), (sx, sy), "frame {t} at {a}");
        }
        assert_eq!(M.len(), 49);
        assert_eq!(whole_degrees(Angle(1_003_487_232)), 83);
        assert_eq!(release_dz(254, crate::anim::ATTACK2, 21, 1), Some(-19));
    }

    /// **run265's 36 Fighter rounds leave from its two guns** (item 853,
    /// `docs/ORDERS.md` §39.3): `(block, node, figure x, y, angle, figure
    /// z, sx, sy, sz)`, the `GUY` record and the `AMMO` record on the
    /// round's first block. Eleven headings, so a stored world vector
    /// fails; and two rows on every block, so a table keyed on the frame
    /// alone fails. Made to fail with the node dropped from [`bay`].
    #[test]
    fn run265_s_fighter_rounds_leave_from_its_two_guns() {
        type Row = (i32, i8, i32, i32, i32, i32, i32, i32, i32);
        const M: &[Row] = &[
            (924, 0, 19979, 16045, 1340473344, 872, 20042, 15990, 873),
            (924, 1, 19979, 16045, 1340473344, 872, 19989, 16127, 873),
            (926, 0, 20117, 16103, 1339752448, 850, 20180, 16048, 851),
            (926, 1, 20117, 16103, 1339752448, 850, 20127, 16185, 851),
            (930, 0, 20393, 16219, 1337327616, 788, 20456, 16164, 789),
            (930, 1, 20393, 16219, 1337327616, 788, 20403, 16301, 789),
            (934, 0, 20669, 16333, 1333657600, 754, 20732, 16278, 755),
            (934, 1, 20669, 16333, 1333657600, 754, 20679, 16415, 755),
            (937, 0, 20876, 16417, 1330708480, 752, 20939, 16362, 753),
            (937, 1, 20876, 16417, 1330708480, 752, 20886, 16499, 753),
            (990, 0, 22125, 15956, -1255828824, 1921, 22107, 16037, 1922),
            (990, 1, 22125, 15956, -1255828824, 1921, 22068, 15896, 1922),
            (992, 0, 21988, 16020, -1401597384, 1879, 21988, 16104, 1880),
            (992, 1, 21988, 16020, -1401597384, 1879, 21919, 15974, 1880),
            (996, 0, 21755, 16171, -1466021880, 1765, 21763, 16254, 1766),
            (996, 1, 21755, 16171, -1466021880, 1765, 21682, 16133, 1766),
            (1000, 0, 21568, 16278, -1410491000, 1611, 21570, 16362, 1612),
            (1000, 1, 21568, 16278, -1410491000, 1611, 21498, 16235, 1612),
            (1003, 0, 21459, 16336, -1402404864, 1469, 21459, 16420, 1470),
            (1003, 1, 21459, 16336, -1402404864, 1469, 21390, 16290, 1470),
            (1007, 0, 21327, 16406, -1397555200, 1269, 21327, 16490, 1270),
            (1007, 1, 21327, 16406, -1397555200, 1269, 21258, 16360, 1270),
            (1011, 0, 21162, 16491, -1390804992, 1095, 21161, 16575, 1096),
            (1011, 1, 21162, 16491, -1390804992, 1095, 21094, 16444, 1096),
            (1059, 0, 19852, 16317, 1193875480, 1511, 19903, 16251, 1512),
            (1059, 1, 19852, 16317, 1193875480, 1511, 19878, 16396, 1512),
            (1061, 0, 19997, 16355, 1252877040, 1513, 20053, 16293, 1514),
            (1061, 1, 19997, 16355, 1252877040, 1513, 20017, 16435, 1514),
            (1065, 0, 20289, 16419, 1179233548, 1489, 20338, 16351, 1490),
            (1065, 1, 20289, 16419, 1179233548, 1489, 20317, 16497, 1490),
            (1069, 0, 20585, 16454, 1142554624, 1425, 20632, 16385, 1426),
            (1069, 1, 20585, 16454, 1142554624, 1425, 20616, 16531, 1426),
            (1072, 0, 20802, 16478, 1142685696, 1351, 20849, 16409, 1352),
            (1072, 1, 20802, 16478, 1142685696, 1351, 20833, 16555, 1352),
            (1076, 0, 21040, 16504, 1139998720, 1217, 21084, 16433, 1218),
            (1076, 1, 21040, 16504, 1139998720, 1217, 21074, 16580, 1218),
        ];
        assert_eq!(M.len(), 36);
        for &(n, node, x, y, a, z, sx, sy, sz) in M {
            let p = launch_point(Pos::new(x, y), Angle(a), 239, crate::anim::ATTACK2, 1, node);
            assert_eq!((p.x, p.y), (sx, sy), "block {n} node {node}");
            assert_eq!(
                z + release_dz(239, crate::anim::ATTACK2, 1, node).unwrap(),
                sz,
                "block {n} node {node}"
            );
        }
    }

    /// **Every distinct launch point of run256's twelve Bowman arrows**,
    /// piece 120, reproduced exactly (item 813): `(anim, starttime, figure
    /// x, y, angle, figure z, sx, sy, sz)`, the dump's own columns — the
    /// `GUY` record on the block before the arrow's first and the `AMMO`
    /// record's launch. Three facings, so a row that were a stored world
    /// vector would fail on at least one.
    #[test]
    fn run256_s_bowman_arrows_leave_from_the_measured_bays() {
        type Row = (i8, u32, i32, i32, i32, i32, i32, i32, i32);
        const M: &[Row] = &[
            (
                crate::anim::ATTACK2,
                9,
                6792,
                32760,
                -999_686_144,
                92,
                6706,
                32751,
                256,
            ),
            (
                crate::anim::ATTACK2,
                9,
                6648,
                32760,
                -987_168_768,
                119,
                6562,
                32750,
                283,
            ),
            (
                crate::anim::ATTACK2,
                9,
                6696,
                32904,
                -885_391_360,
                107,
                6612,
                32882,
                271,
            ),
            (
                crate::anim::ATTACK1,
                12,
                6696,
                32904,
                -885_391_360,
                107,
                6610,
                32883,
                270,
            ),
            (
                crate::anim::ATTACK1,
                12,
                6792,
                32760,
                -999_686_144,
                92,
                6704,
                32753,
                255,
            ),
            // Chapter 103's bowman `1/9` on block 933 (run945b, item
            // 3271, `docs/GOLDEN.md` §139): facing 319°, offset (−59,
            // −65), where the two rows above alone gave (−59, −66) and
            // flew the arrow one frame short.
            (
                crate::anim::ATTACK1,
                12,
                15912,
                16200,
                -483_196_928,
                82,
                15853,
                16135,
                245,
            ),
            (
                crate::anim::ATTACK3,
                15,
                6648,
                32760,
                -987_168_768,
                119,
                6584,
                32743,
                283,
            ),
            (
                crate::anim::ATTACK3,
                15,
                6792,
                32760,
                -999_686_144,
                92,
                6727,
                32744,
                256,
            ),
        ];
        for &(anim, t, gx, gy, ga, gz, sx, sy, sz) in M {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), 120, anim, t, 0);
            assert_eq!(
                (got.x, got.y),
                (sx, sy),
                "piece 120 anim {anim} t {t} facing {ga}"
            );
            assert_eq!(
                gz + release_dz(120, anim, t, 0).unwrap(),
                sz,
                "piece 120 anim {anim} dz"
            );
        }
    }

    /// **Every distinct launch point of run17's 27 Slinger stones**, piece
    /// 32, reproduced exactly (item 1040): `(anim, starttime, figure x, y,
    /// angle, figure z, sx, sy, sz)`, the dump's own columns — the `GUY`
    /// record and the `AMMO` record on the stone's first block (the
    /// Slingers stand while they throw). The key is the release event the
    /// stone's launch frame names against the unit's `recharging` jump.
    /// Twenty headings, so a stored world vector would fail on most.
    #[test]
    fn run17_s_slinger_stones_leave_from_the_measured_bays() {
        type Row = (i8, u32, i32, i32, i32, i32, i32, i32, i32);
        #[rustfmt::skip]
        const M: &[Row] = &[
            (crate::anim::ATTACK2, 22, 37176, 20760, -2116288512, -1, 37133, 20877, 150),
            (crate::anim::ATTACK2, 22, 37512, 20136, -1443561472, -12, 37391, 20163, 139),
            (crate::anim::ATTACK2, 22, 37656, 19992, -1478754304, -7, 37536, 20025, 144),
            (crate::anim::ATTACK2, 22, 37512, 19752, -1628569600, -23, 37403, 19812, 128),
            (crate::anim::ATTACK2, 22, 37656, 19992, -508821504, -7, 37604, 19879, 144),
            (crate::anim::ATTACK2, 22, 37656, 19992, 69140480, -7, 37709, 19879, 144),
            (crate::anim::ATTACK2, 22, 37512, 19752, 318177280, -23, 37599, 19663, 128),
            (crate::anim::ATTACK2, 22, 37512, 20136, 934674432, -12, 37636, 20149, 139),
            (crate::anim::ATTACK2, 22, 37656, 19992, 965083136, -7, 37779, 20012, 144),
            (crate::anim::ATTACK2, 22, 37512, 19752, 1141964800, -23, 37625, 19805, 128),
            (crate::anim::ATTACK2, 22, 38040, 17256, 654901248, 28, 38160, 17223, 179),
            (crate::anim::ATTACK2, 22, 38184, 17448, 530382848, 31, 38295, 17392, 182),
            (crate::anim::ATTACK2, 22, 38040, 17400, 593100800, 27, 38156, 17354, 178),
            (crate::anim::ATTACK2, 22, 38040, 17256, 936706048, 28, 38164, 17269, 179),
            (crate::anim::ATTACK2, 22, 38184, 17448, 723517440, 31, 38306, 17425, 182),
            (crate::anim::ATTACK2, 22, 38184, 17448, 438894592, 31, 38287, 17379, 182),
            (crate::anim::ATTACK2, 22, 38040, 17400, 507314176, 27, 38149, 17340, 178),
            (crate::anim::ATTACK2, 22, 39192, 17928, 1329725440, 55, 39287, 18008, 206),
            (crate::anim::ATTACK3, 16, 37512, 20136, -256573440, -12, 37431, 20047, 165),
            (crate::anim::ATTACK3, 16, 37512, 19752, 587202560, -23, 37569, 19646, 154),
            (crate::anim::ATTACK3, 16, 38040, 17400, 800587776, 27, 38126, 17316, 204),
            (crate::anim::ATTACK3, 16, 38040, 17256, 621740032, 28, 38103, 17154, 205),
            (crate::anim::ATTACK1, 21, 37512, 20136, 131006464, -12, 37592, 20082, 166),
            (crate::anim::ATTACK1, 21, 38040, 17400, 593100800, 27, 38136, 17408, 205),
        ];
        for &(anim, t, gx, gy, ga, gz, sx, sy, sz) in M {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), 32, anim, t, 0);
            assert_eq!(
                (got.x, got.y),
                (sx, sy),
                "piece 32 anim {anim} t {t} facing {ga}"
            );
            assert_eq!(
                gz + release_dz(32, anim, t, 0).unwrap(),
                sz,
                "piece 32 anim {anim} dz"
            );
        }
    }

    /// run448's two javelins, from the `GUY` and `AMMO` records on each
    /// round's first block (`docs/COMBAT.md` §70.2): the figure's point,
    /// facing and `z`, then the round's `sx, sy, sz`. The rows themselves
    /// are the packet's sweep of `get_position`; these are the dump's own
    /// check on them.
    #[test]
    fn run448_s_javelins_leave_from_the_measured_bays() {
        type Row = (i8, u32, i32, i32, i32, i32, i32, i32, i32);
        #[rustfmt::skip]
        const M: &[Row] = &[
            (crate::anim::ATTACK1, 12, 6120, 28248, -1661468672, 578, 6028, 28300, 741),
            (crate::anim::ATTACK3, 27, 6360, 28488, -1532166144, 626, 6289, 28517, 782),
        ];
        for &(anim, t, gx, gy, ga, gz, sx, sy, sz) in M {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), 33, anim, t, 0);
            assert_eq!((got.x, got.y), (sx, sy), "piece 33 anim {anim} t {t}");
            assert_eq!(gz + release_dz(33, anim, t, 0).unwrap(), sz, "piece 33 dz");
        }
        assert!(
            bay(33, crate::anim::ATTACK2, 6, 0).is_some(),
            "all three of the piece's events have a row"
        );
    }

    /// **Every distinct launch point of chapter sixty-four's Barks and
    /// chapter sixty-five's Catapult** (item 2304, `docs/GOLDEN.md` §83):
    /// run835's three Barks (`0/6`, `0/7` and the computer's `1/8`, piece
    /// 273) at five headings, and run843's Catapult `0/9` (piece 567),
    /// `(anim, starttime, figure x, y, angle, figure z, sx, sy, sz)` — the
    /// `GUY` record and the `AMMO` record on the round's first block. The
    /// rows were read off two packets (run839, run842), and so were the
    /// Trireme's that fire beside them; these are the dump,
    /// so the two are independent. Made to fail first: with the rows taken
    /// out every round leaves the figure's square at its own `z`.
    #[test]
    fn run835_s_barks_and_run843_s_catapult_leave_from_their_release_nodes() {
        type Row = (i8, u32, i32, i32, i32, i32, i32, i32, i32);
        #[rustfmt::skip]
        const BARK: &[Row] = &[
            (crate::anim::ATTACK2, 2, 11640, 34680, 982056960, 0, 11707, 34671, 52), // 638 0/6
            (crate::anim::ATTACK2, 2, 12312, 34872, 1344339968, 0, 12374, 34898, 52), // 638 0/7
            (crate::anim::ATTACK2, 10, 11640, 34680, 982056960, 0, 11531, 34696, 52), // 646 0/6
            (crate::anim::ATTACK2, 10, 12312, 34872, 1344339968, 0, 12211, 34830, 52), // 646 0/7
            (crate::anim::ATTACK2, 17, 11640, 34680, 982056960, 0, 11614, 34681, 52), // 653 0/6
            (crate::anim::ATTACK2, 17, 12312, 34872, 1344339968, 0, 12289, 34861, 52), // 653 0/7
            (crate::anim::ATTACK2, 2, 13464, 35880, 494338048, 0, 13508, 35829, 52), // 658 1/8
            (crate::anim::ATTACK2, 2, 11640, 34680, 531824640, 0, 11687, 34631, 52), // 663 0/6
            (crate::anim::ATTACK2, 2, 12312, 34872, 787349504, 0, 12373, 34844, 52), // 663 0/7
            (crate::anim::ATTACK2, 10, 13464, 35880, 494338048, 0, 13393, 35963, 52), // 666 1/8
            (crate::anim::ATTACK2, 10, 11640, 34680, 531824640, 0, 11565, 34760, 52), // 671 0/6
            (crate::anim::ATTACK2, 10, 12312, 34872, 787349504, 0, 12213, 34919, 52), // 671 0/7
            (crate::anim::ATTACK2, 17, 13464, 35880, 494338048, 0, 13446, 35898, 52), // 673 1/8
            (crate::anim::ATTACK2, 17, 11640, 34680, 531824640, 0, 11621, 34697, 52), // 678 0/6
            (crate::anim::ATTACK2, 17, 12312, 34872, 787349504, 0, 12288, 34881, 52), // 678 0/7
            (crate::anim::ATTACK1, 2, 11640, 34680, 531824640, 0, 11687, 34631, 52), // 688 0/6
            (crate::anim::ATTACK2, 2, 12312, 34872, 494338048, 0, 12356, 34821, 52), // 688 0/7
            (crate::anim::ATTACK1, 10, 11640, 34680, 531824640, 0, 11565, 34760, 52), // 696 0/6
            (crate::anim::ATTACK2, 10, 12312, 34872, 494338048, 0, 12241, 34955, 52), // 696 0/7
            (crate::anim::ATTACK1, 17, 11640, 34680, 531824640, 0, 11621, 34697, 52), // 703 0/6
            (crate::anim::ATTACK2, 17, 12312, 34872, 494338048, 0, 12294, 34890, 52), // 703 0/7
            (crate::anim::ATTACK1, 2, 13464, 35880, 494338048, 0, 13508, 35829, 52), // 708 1/8
            (crate::anim::ATTACK1, 10, 13464, 35880, 494338048, 0, 13393, 35963, 52), // 716 1/8
            (crate::anim::ATTACK1, 17, 13464, 35880, 494338048, 0, 13446, 35898, 52), // 723 1/8
        ];
        for &(anim, t, gx, gy, ga, gz, sx, sy, sz) in BARK {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), 273, anim, t, 0);
            assert_eq!(
                (got.x, got.y),
                (sx, sy),
                "piece 273 anim {anim} t {t} facing {ga}"
            );
            assert_eq!(
                gz + release_dz(273, anim, t, 0).unwrap_or(0),
                sz,
                "piece 273 dz"
            );
        }
        // run835's computer Trireme `1/6` (piece 290), whose rows are the
        // packet's since this item: 634's third key was a unit out.
        #[rustfmt::skip]
        const TRIREME: &[Row] = &[
            (crate::anim::ATTACK2, 5, 12408, 36216, 756678656, 0, 12448, 36196, 88), // 622
            (crate::anim::ATTACK2, 9, 12408, 36216, 756678656, 0, 12395, 36221, 88), // 626
            (crate::anim::ATTACK2, 17, 12408, 36216, 756678656, 0, 12349, 36244, 87), // 634
            (crate::anim::ATTACK1, 17, 12408, 36216, 756678656, 0, 12349, 36244, 87), // 713
        ];
        for &(anim, t, gx, gy, ga, gz, sx, sy, sz) in TRIREME {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), 290, anim, t, 0);
            assert_eq!((got.x, got.y), (sx, sy), "piece 290 anim {anim} t {t}");
            assert_eq!(
                gz + release_dz(290, anim, t, 0).unwrap_or(0),
                sz,
                "piece 290 dz"
            );
        }
        // run843 block 818: `0/9`'s first round on the Barracks `1/2008`.
        let got = launch_point(
            Pos::new(34680, 24696),
            Angle(605_028_352),
            567,
            crate::anim::ATTACK2,
            12,
            0,
        );
        assert_eq!((got.x, got.y), (34655, 24716), "piece 567");
        assert_eq!(
            50 + release_dz(567, crate::anim::ATTACK2, 12, 0).unwrap_or(0),
            295,
            "piece 567 dz"
        );
    }

    /// run587's two arrows from 1/135, independent of the packet sweep
    /// used to fit the bays (COMBAT §87). Both launch at the same facing.
    #[test]
    fn run587_s_yeoman_arrows_leave_the_bow_hand() {
        let pos = Pos::new(6408, 9960);
        let facing = Angle(-240_975_872);
        for (time, x, y, z) in [(4, 6364, 9890, 198), (22, 6376, 9885, 230)] {
            let at = launch_point(pos, facing, 129, crate::anim::ATTACK2, time, 0);
            assert_eq!((at.x, at.y), (x, y));
            assert_eq!(
                61 + release_dz(129, crate::anim::ATTACK2, time, 0).unwrap(),
                z
            );
        }
        for (anim, time) in [(11, 6), (11, 22), (12, 4), (12, 22), (13, 10), (13, 24)] {
            assert!(
                bay(129, anim, time, 0).is_some(),
                "every measured release event"
            );
        }
    }

    /// run912d's javelin from who=1's Elite Javelineers `1/42` on 12754
    /// (item 2997), independent of run912e's packet sweep the bays were
    /// fitted to: the figure at (6504, 8904), angle −600637440, `ATTACK2`
    /// at frame 6; the dump's `AMMO` leaves from (6466, 8872).
    #[test]
    fn run912d_s_javelin_leaves_the_throwing_hand() {
        let at = launch_point(
            Pos::new(6504, 8904),
            Angle(-600_637_440),
            34,
            crate::anim::ATTACK2,
            6,
            0,
        );
        assert_eq!((at.x, at.y), (6466, 8872));
        for (anim, time) in [(11, 12), (12, 6), (13, 27)] {
            assert!(bay(34, anim, time, 0).is_some(), "every AttachPos row");
        }
    }

    /// run943c's arrow from who=1's Archers `1/32` on 10674 (item 3255),
    /// independent of the packet sweep the bays were fitted to: the
    /// figure at (5400, 31416), angle −1239220224, `ATTACK2` at frame 9;
    /// the dump's `AMMO` leaves from (5316, 31436), `total_time` 18.
    #[test]
    fn run943c_s_arrow_leaves_the_bow_hand() {
        let at = launch_point(
            Pos::new(5400, 31416),
            Angle(-1_239_220_224),
            1529,
            crate::anim::ATTACK2,
            9,
            0,
        );
        assert_eq!((at.x, at.y), (5316, 31436));
        for (anim, time) in [(11, 12), (12, 9), (13, 15)] {
            assert!(
                bay(1529, anim, time, 0).is_some(),
                "every attack AttachPos row"
            );
        }
    }

    /// run991c's three Hwarang rounds (item 3365), independent of the
    /// packet sweep the bays were fitted to: `1/74`'s `ATTACK1` at frame 6
    /// from (9336, 30840) at −916717568, `1/76`'s from (9288, 30648) at
    /// −979763200, and `1/75`'s `ATTACK3` at frame 10 from (9240, 30936)
    /// at −872480768; the dump's `AMMO` leave from (9264, 30827), (9215,
    /// 30643) and (9183, 30911). No planar [`Node`] fits the two `ATTACK1`
    /// rounds at once.
    #[test]
    fn run991c_s_hwarang_arrows_leave_the_bow_hand() {
        for (x, y, facing, anim, time, want) in [
            (
                9336,
                30840,
                -916_717_568,
                crate::anim::ATTACK1,
                6,
                (9264, 30827),
            ),
            (
                9288,
                30648,
                -979_763_200,
                crate::anim::ATTACK1,
                6,
                (9215, 30643),
            ),
            (
                9240,
                30936,
                -872_480_768,
                crate::anim::ATTACK3,
                10,
                (9183, 30911),
            ),
        ] {
            let at = launch_point(Pos::new(x, y), Angle(facing), 132, anim, time, 0);
            assert_eq!((at.x, at.y), want);
        }
        for (anim, time) in [(11, 6), (11, 22), (12, 4), (12, 22), (13, 10), (13, 24)] {
            assert!(
                bay(132, anim, time, 0).is_some(),
                "every attack AttachPos row"
            );
        }
    }

    /// run943b's javelin from who=1's Javelineers `1/27` (item 3255),
    /// independent of the packet sweep: the figure at (4872, 31272),
    /// angle −895811584, `ATTACK1` at frame 12; the dump's `AMMO` leaves
    /// from (4820, 31215) on block 10727, `total_time` 18.
    #[test]
    fn run943b_s_javelin_leaves_the_throwing_hand() {
        let at = launch_point(
            Pos::new(4872, 31272),
            Angle(-895_811_584),
            1441,
            crate::anim::ATTACK1,
            12,
            0,
        );
        assert_eq!((at.x, at.y), (4820, 31215));
        for (anim, time) in [(11, 12), (12, 6), (13, 27)] {
            assert!(
                bay(1441, anim, time, 0).is_some(),
                "every attack AttachPos row"
            );
        }
    }

    /// run971's two Musketeer rounds (piece 49, item 3336), the rows
    /// [`MEASURED`] was fitted to: `1/136`'s `ATTACK3` at frame 4 on 20374
    /// and `1/54`'s `ATTACK1` at frame 1 on 20378, `(anim, t, gx, gy, ga,
    /// gz, sx, sy, sz)` as the dump prints them.
    #[test]
    fn run971_s_musketeer_rounds_leave_from_their_release_nodes() {
        type Row = (i8, u32, i32, i32, i32, i32, i32, i32, i32);
        #[rustfmt::skip]
        const ROWS: &[Row] = &[
            (crate::anim::ATTACK3, 4, 5448, 26856, -1_656_487_936, 504, 5333, 26970, 682),
            (crate::anim::ATTACK1, 1, 5544, 26808, -1_641_873_408, 495, 5426, 26889, 713),
        ];
        for &(anim, t, gx, gy, ga, gz, sx, sy, sz) in ROWS {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), 49, anim, t, 0);
            assert_eq!((got.x, got.y), (sx, sy), "piece 49 anim {anim} t {t}");
            assert_eq!(
                gz + release_dz(49, anim, t, 0).unwrap_or(0),
                sz,
                "piece 49 dz"
            );
        }
        assert_eq!(
            launch_point(
                Pos::new(5448, 26856),
                Angle(0),
                49,
                crate::anim::ATTACK2,
                7,
                0
            ),
            Pos::new(5448, 26856),
            "ATTACK2's frame 7 is not measured: the unit's own position"
        );
    }

    /// **Piece 1531, `HEAVYARCHERS-NA`** (item 3375): run1001b's round of
    /// `1/35`, `CHAR_ATTACK2` at frame 9 — the dump's own columns,
    /// independent of run1001d's packet sweep the bays were fitted to.
    #[test]
    fn run1001b_s_heavy_archer_round_leaves_its_release_node() {
        let got = launch_point(
            Pos::new(9048, 31128),
            Angle(-1_464_008_704),
            1531,
            crate::anim::ATTACK2,
            9,
            0,
        );
        assert_eq!((got.x, got.y), (8976, 31176));
        assert_eq!(
            635 + release_dz(1531, crate::anim::ATTACK2, 9, 0).unwrap_or(0),
            799
        );
        for (anim, time) in [(11, 12), (12, 9), (13, 15)] {
            assert!(
                bay(1531, anim, time, 0).is_some(),
                "every attack AttachPos row"
            );
        }
    }

    /// **Piece 1531's bays against run1001c's rounds** (item 3375): `1/36`
    /// and `1/37`, the Heavy Archers, at three facings — all three attack
    /// keys, the figure's `x, y` and `angle` on the release block against
    /// the round's `sx, sy`. Independent of run1001d's packet sweep the bays
    /// were fitted to; the walk reaches only `CHAR_ATTACK2`'s.
    #[test]
    fn run1001c_s_heavy_archer_rounds_leave_piece_1531_s_bays() {
        for (anim, time, (gx, gy), facing, (sx, sy)) in [
            (11, 12, (9096, 30984), -1_696_006_144, (9045, 31055)),
            (11, 12, (9096, 30984), -673_775_616, (9020, 30939)),
            (11, 12, (9000, 31320), -565_313_536, (8933, 31262)),
            (12, 9, (9096, 30984), -673_775_616, (9023, 30938)),
            (12, 9, (9000, 31320), -565_313_536, (8936, 31262)),
            (13, 15, (9000, 31320), -1_610_022_912, (8947, 31360)),
            (13, 15, (9096, 30984), -673_775_616, (9045, 30941)),
            (13, 15, (9000, 31320), -565_313_536, (8958, 31269)),
        ] {
            let got = launch_point(Pos::new(gx, gy), Angle(facing), 1531, anim, time, 0);
            assert_eq!(
                (got.x, got.y),
                (sx, sy),
                "piece 1531 anim {anim} at {facing}"
            );
        }
    }

    /// **Piece 123's bays against run1011's rounds** (item 3385, `docs/AI.md`
    /// §293): the Inca Heavy Archers `1/49`, `1/50` and `1/51` loosing at
    /// the Nubians' `0/2002`, the figure's `x, y, z` and `angle` on the
    /// round's first-printed block against its `sx, sy, sz`. Independent of
    /// run1011c's packet sweep the rows were read from. Made to fail with
    /// the rows keyed off: the rounds leave the figure's own point.
    #[test]
    fn run1011_s_inca_heavy_archer_rounds_leave_piece_123_s_bays() {
        for (anim, time, (gx, gy, gz), facing, (sx, sy, sz)) in [
            (13, 15, (6600, 31800, 626), -614_596_608, (6554, 31752, 790)),
            (12, 9, (6792, 31704, 609), -677_904_384, (6719, 31658, 773)),
            (12, 9, (5256, 31992, 603), -116_981_760, (5242, 31906, 767)),
        ] {
            let got = launch_point(Pos::new(gx, gy), Angle(facing), 123, anim, time, 0);
            assert_eq!(
                (got.x, got.y),
                (sx, sy),
                "piece 123 anim {anim} at {facing}"
            );
            assert_eq!(gz + release_dz(123, anim, time, 0).unwrap_or(0), sz);
        }
    }

    /// **Piece 275, the Caravel** (item 3633): four headings of each of its
    /// nine rows' `get_position.py` sweeps on run1293's packet, `(anim,
    /// starttime, degree, x, y)`, read the way
    /// [`the_peltasts_bays_are_the_packet_s_sweep`] reads 1442's.
    #[test]
    fn the_caravel_s_bays_are_the_packet_s_sweep() {
        const S: &[(i8, u32, i64, i32, i32)] = &[
            (10, 5, 0, -66, -135),
            (10, 5, 97, 142, -49),
            (10, 5, 233, -68, 133),
            (10, 5, 300, -149, -10),
            (10, 8, 0, -66, -135),
            (10, 8, 97, 142, -49),
            (10, 8, 233, -68, 133),
            (10, 8, 300, -149, -10),
            (10, 13, 0, -65, -135),
            (10, 13, 97, 141, -48),
            (10, 13, 233, -68, 133),
            (10, 13, 300, -149, -11),
            (11, 5, 0, -66, -135),
            (11, 5, 97, 142, -49),
            (11, 5, 233, -68, 133),
            (11, 5, 300, -149, -10),
            (11, 8, 0, -66, -135),
            (11, 8, 97, 142, -49),
            (11, 8, 233, -68, 133),
            (11, 8, 300, -149, -10),
            (11, 13, 0, -65, -135),
            (11, 13, 97, 141, -48),
            (11, 13, 233, -68, 133),
            (11, 13, 300, -149, -11),
            (12, 5, 0, -66, -135),
            (12, 5, 97, 142, -49),
            (12, 5, 233, -68, 133),
            (12, 5, 300, -149, -10),
            (12, 8, 0, -66, -135),
            (12, 8, 97, 142, -49),
            (12, 8, 233, -68, 133),
            (12, 8, 300, -149, -10),
            (12, 13, 0, -65, -135),
            (12, 13, 97, 141, -48),
            (12, 13, 233, -68, 133),
            (12, 13, 300, -149, -11),
        ];
        for &(anim, time, deg, x, y) in S {
            let want = (deg + 180) % 360;
            let facing = (0..1u32 << 12)
                .map(|i| Angle((i << 20) as i32))
                .find(|&a| whole_degrees(a) == want)
                .expect("a facing at the degree");
            let b = bay(275, anim, time, 0).expect("every attack AttachPos row");
            let at = b.point(Pos::new(10_000, 10_000), facing);
            assert_eq!(
                (at.x - 10_000, at.y - 10_000),
                (x, y),
                "{anim} {time} {deg}"
            );
        }
    }

    /// **The Frigate's piece 293** (item 3643): four headings of each row's
    /// `get_position.py` sweep on run1304's packet, `(anim, starttime,
    /// degree, x, y)`, the degree the sweep's own column (180 off
    /// [`whole_degrees`]); and the round run1301b prints, `1/29` at
    /// (1896, 5352) facing 104464384 on `CHAR_ATTACK2`, leaving (1809, 5233).
    #[test]
    fn the_frigate_s_bays_are_the_packet_s_sweep() {
        const S: &[(i8, u32, i64, i32, i32)] = &[
            (11, 7, 0, 117, -11),
            (11, 7, 97, -3, 117),
            (11, 7, 233, -79, -87),
            (11, 7, 300, 49, -107),
            (12, 7, 0, 103, 106),
            (12, 7, 97, -118, 89),
            (12, 7, 188, -87, -119),
            (12, 7, 233, 22, -146),
            (12, 7, 300, 143, -36),
        ];
        for &(anim, time, deg, x, y) in S {
            let want = (deg + 180) % 360;
            let facing = (0..1u32 << 12)
                .map(|i| Angle((i << 20) as i32))
                .find(|&a| whole_degrees(a) == want)
                .expect("a facing at the degree");
            let b = bay(293, anim, time, 0).expect("every attack AttachPos row");
            let at = b.point(Pos::new(10_000, 10_000), facing);
            assert_eq!(
                (at.x - 10_000, at.y - 10_000),
                (x, y),
                "{anim} {time} {deg}"
            );
        }
        let at = launch_point(
            Pos::new(1896, 5352),
            Angle(104_464_384),
            293,
            crate::anim::ATTACK2,
            7,
            0,
        );
        assert_eq!((at.x, at.y), (1809, 5233));
        assert_eq!(release_dz(293, crate::anim::ATTACK2, 7, 0), Some(20));
    }

    /// **Piece 1442, the Native American Peltasts** (item 3287): four
    /// headings of each row's `get_position.py` sweep on run943c's packet,
    /// `(anim, starttime, degree, x, y)`, the degree `fast_angle_to_degrees`'
    /// (the sweep's own column, 180 off [`whole_degrees`]).
    #[test]
    fn the_peltasts_bays_are_the_packet_s_sweep() {
        const S: &[(i8, u32, i64, i32, i32)] = &[
            (11, 12, 0, -43, 64),
            (11, 12, 97, -58, -50),
            (11, 12, 233, 77, -4),
            (11, 12, 300, 34, 69),
            (12, 6, 0, 0, 57),
            (12, 6, 97, -56, -6),
            (12, 6, 233, 45, -34),
            (12, 6, 300, 49, 28),
            (13, 27, 0, -20, 36),
            (13, 27, 97, -33, -24),
            (13, 27, 233, 41, -5),
            (13, 27, 300, 21, 35),
        ];
        for &(anim, time, deg, x, y) in S {
            let want = (deg + 180) % 360;
            let facing = (0..1u32 << 12)
                .map(|i| Angle((i << 20) as i32))
                .find(|&a| whole_degrees(a) == want)
                .expect("a facing at the degree");
            let b = bay(1442, anim, time, 0).expect("every attack AttachPos row");
            let at = b.point(Pos::new(10_000, 10_000), facing);
            assert_eq!(
                (at.x - 10_000, at.y - 10_000),
                (x, y),
                "{anim} {time} {deg}"
            );
        }
    }

    /// **The Maya Bowmen's piece 824** (item 3600): four headings of each
    /// attack row's sweep off run1247's packet, the way
    /// [`the_peltasts_bays_are_the_packet_s_sweep`] reads 1442's — and the
    /// rows are piece 473's, the Archers', key for key and row for row.
    #[test]
    fn the_maya_bowmen_s_bays_are_the_packet_s_sweep_and_piece_473_s() {
        const S: &[(i8, u32, i64, i32, i32)] = &[
            (11, 12, 0, 1, 88),
            (11, 12, 97, -88, -9),
            (11, 12, 233, 69, -54),
            (11, 12, 300, 77, 42),
            (12, 9, 0, 0, 87),
            (12, 9, 97, -86, -10),
            (12, 9, 233, 70, -52),
            (12, 9, 300, 75, 43),
            (13, 15, 0, -9, 66),
            (13, 15, 97, -64, -17),
            (13, 15, 233, 58, -32),
            (13, 15, 300, 52, 41),
        ];
        for &(anim, time, deg, x, y) in S {
            let want = (deg + 180) % 360;
            let facing = (0..1u32 << 12)
                .map(|i| Angle((i << 20) as i32))
                .find(|&a| whole_degrees(a) == want)
                .expect("a facing at the degree");
            let b = bay(824, anim, time, 0).expect("every attack AttachPos row");
            let at = b.point(Pos::new(10_000, 10_000), facing);
            assert_eq!(
                (at.x - 10_000, at.y - 10_000),
                (x, y),
                "{anim} {time} {deg}"
            );
            assert_eq!(Some(b), bay(473, anim, time, 0), "{anim} {time}");
        }
    }

    /// **The Umpakati's pieces 43 and 44** (item 3591): four headings of each
    /// attack row's sweep off run1238c's packet, the way
    /// [`the_peltasts_bays_are_the_packet_s_sweep`] reads 1442's, and the
    /// four rounds run1238b prints (`AMMO=5`, `sx, sy, sz` against the
    /// figure's pose on the block before the round's first print).
    #[test]
    fn the_umpakati_s_bays_are_the_packet_s_sweep_and_run1238b_s_rounds() {
        const S: &[(i8, u32, i64, i32, i32)] = &[
            (11, 12, 0, -33, 65),
            (11, 12, 97, -61, -41),
            (11, 12, 233, 72, -12),
            (11, 12, 300, 39, 61),
            (12, 6, 0, 13, 50),
            (12, 6, 97, -51, 6),
            (12, 6, 233, 32, -40),
            (12, 6, 300, 49, 13),
            (13, 27, 0, -14, 40),
            (13, 27, 97, -38, -19),
            (13, 27, 233, 41, -12),
            (13, 27, 300, 27, 32),
        ];
        for &(anim, time, deg, x, y) in S {
            let want = (deg + 180) % 360;
            let facing = (0..1u32 << 12)
                .map(|i| Angle((i << 20) as i32))
                .find(|&a| whole_degrees(a) == want)
                .expect("a facing at the degree");
            // Piece 44, the Yangombi Umpakati, sweeps identically.
            for piece in [43, 44] {
                let b = bay(piece, anim, time, 0).expect("every attack AttachPos row");
                let at = b.point(Pos::new(10_000, 10_000), facing);
                assert_eq!(
                    (at.x - 10_000, at.y - 10_000),
                    (x, y),
                    "{piece} {anim} {time} {deg}"
                );
            }
        }
        // `(shooter, anim, time, figure, z, facing, the round's sx sy sz)`.
        type Round = (&'static str, i8, u32, (i32, i32), i32, i32, (i32, i32, i32));
        const R: &[Round] = &[
            (
                "1/86",
                12,
                6,
                (6792, 8520),
                288,
                283_836_416,
                (6799, 8469, 475),
            ),
            (
                "1/87",
                11,
                12,
                (6936, 8232),
                311,
                273_154_048,
                (6992, 8185, 496),
            ),
            (
                "1/90",
                11,
                12,
                (6024, 8808),
                185,
                -649_068_544,
                (5990, 8743, 370),
            ),
            (
                "1/91",
                11,
                12,
                (5496, 9144),
                85,
                -292_028_416,
                (5498, 9071, 270),
            ),
        ];
        for &(who, anim, time, (x, y), z, facing, want) in R {
            let at = launch_point(Pos::new(x, y), Angle(facing), 43, anim, time, 0);
            let dz = release_dz(43, anim, time, 0).expect("a bay");
            assert_eq!((at.x, at.y, z + dz), want, "{who}");
        }
    }

    /// **The Archers' piece 473** (item 3592): four headings of each attack
    /// row's sweep off run1238c's packet (`get_position.py … sweep 473 0
    /// <anim> <time>`), and the sweep's `z` as the release height. Piece
    /// 120's `Bay`s, at the same keys, miss 28, 124 and 60 of the 361
    /// headings (`tools/recomp/fit_bay.py --check`), so the rows are the
    /// piece's own.
    #[test]
    fn the_archers_bays_are_the_packet_s_sweep() {
        const S: &[(i8, u32, i64, i32, i32)] = &[
            (11, 12, 0, 1, 88),
            (11, 12, 97, -88, -9),
            (11, 12, 233, 69, -54),
            (11, 12, 300, 77, 42),
            (12, 9, 0, 0, 87),
            (12, 9, 97, -86, -10),
            (12, 9, 233, 70, -52),
            (12, 9, 300, 75, 43),
            (13, 15, 0, -9, 66),
            (13, 15, 97, -64, -17),
            (13, 15, 233, 58, -32),
            (13, 15, 300, 52, 41),
        ];
        for &(anim, time, deg, x, y) in S {
            let want = (deg + 180) % 360;
            let facing = (0..1u32 << 12)
                .map(|i| Angle((i << 20) as i32))
                .find(|&a| whole_degrees(a) == want)
                .expect("a facing at the degree");
            let b = bay(473, anim, time, 0).expect("every attack AttachPos row");
            let at = b.point(Pos::new(10_000, 10_000), facing);
            assert_eq!(
                (at.x - 10_000, at.y - 10_000),
                (x, y),
                "{anim} {time} {deg}"
            );
        }
        for (anim, time, dz) in [(11, 12, 163), (12, 9, 164), (13, 15, 164)] {
            assert_eq!(release_dz(473, anim, time, 0), Some(dz), "{anim} {time}");
        }
    }

    /// **The Persian Peltasts' piece 1794 and the Mahout's piece 212**
    /// (item 3555): four headings of each attack row's sweep off run1007b's
    /// packet (`get_position.py … sweep <piece> <node> <anim> <time>`), the
    /// way [`the_peltasts_bays_are_the_packet_s_sweep`] reads 1442's.
    #[test]
    fn the_persian_peltasts_and_the_mahouts_bays_are_the_packet_s_sweep() {
        const S: &[(i32, i8, u32, i64, i32, i32)] = &[
            (1794, 11, 12, 0, -38, 60),
            (1794, 11, 12, 97, -55, -45),
            (1794, 11, 12, 233, 71, -5),
            (1794, 11, 12, 300, 32, 63),
            (1794, 12, 6, 0, -1, 50),
            (1794, 12, 6, 97, -50, -7),
            (1794, 12, 6, 233, 41, -29),
            (1794, 12, 6, 300, 43, 26),
            (1794, 13, 27, 0, -21, 35),
            (1794, 13, 27, 97, -32, -25),
            (1794, 13, 27, 233, 41, -3),
            (1794, 13, 27, 300, 19, 36),
            (212, 10, 16, 0, 29, 87),
            (212, 10, 16, 97, -90, 19),
            (212, 10, 16, 233, 51, -76),
            (212, 10, 16, 300, 90, 17),
            (212, 11, 16, 0, 0, 71),
            (212, 11, 16, 97, -71, -8),
            (212, 11, 16, 233, 56, -43),
            (212, 11, 16, 300, 62, 35),
            (212, 12, 16, 0, 17, -7),
            (212, 12, 16, 97, 5, 18),
            (212, 12, 16, 233, -16, -9),
            (212, 12, 16, 300, 2, -19),
            (212, 13, 16, 0, 18, 60),
            (212, 13, 16, 97, -62, 10),
            (212, 13, 16, 233, 37, -51),
            (212, 13, 16, 300, 61, 14),
            (212, 14, 16, 0, 18, 79),
            (212, 14, 16, 97, -81, 8),
            (212, 14, 16, 233, 52, -62),
            (212, 14, 16, 300, 77, 23),
        ];
        for &(piece, anim, time, deg, x, y) in S {
            let want = (deg + 180) % 360;
            let facing = (0..1u32 << 12)
                .map(|i| Angle((i << 20) as i32))
                .find(|&a| whole_degrees(a) == want)
                .expect("a facing at the degree");
            let node = if piece == 212 { 1 } else { 0 };
            let b = bay(piece, anim, time, node).expect("every attack AttachPos row");
            let at = b.point(Pos::new(10_000, 10_000), facing);
            assert_eq!(
                (at.x - 10_000, at.y - 10_000),
                (x, y),
                "{piece} {anim} {time} {deg}"
            );
        }
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
            let got = launch_point(Pos::new(gx, gy), Angle(ga), 127, anim, t, 0);
            assert_eq!(
                (got.x, got.y),
                (sx, sy),
                "anim {anim} t {t} from ({gx},{gy}) facing {ga}"
            );
        }
    }

    /// run982's rounds (item 3347): `(piece, anim, starttime, guy x, y, z,
    /// angle, sx, sy, sz)`, the dump's own columns, reproduced to the unit —
    /// the Crossbowman's three. The Cavalry Archer's three are
    /// `pivot::tests::run982_s_cavalry_archer_rounds_fit_piece_138_s_pivot_rows`
    /// since item 3633.
    #[test]
    fn run982_s_crossbow_and_cavalry_archer_launches_are_reproduced_to_the_unit() {
        type Round = (i32, i8, u32, i32, i32, i32, i32, i32, i32, i32);
        const M: &[Round] = &[
            (
                122,
                crate::anim::ATTACK2,
                3,
                10008,
                30984,
                573,
                -1_120_927_744,
                9943,
                30974,
                707,
            ),
            (
                122,
                crate::anim::ATTACK1,
                5,
                9624,
                31176,
                603,
                -1_057_882_112,
                9540,
                31165,
                769,
            ),
            (
                122,
                crate::anim::ATTACK3,
                6,
                9672,
                30984,
                597,
                -1_130_299_392,
                9596,
                30977,
                758,
            ),
        ];
        for &(piece, anim, t, gx, gy, gz, ga, sx, sy, sz) in M {
            let got = launch_point(Pos::new(gx, gy), Angle(ga), piece, anim, t, 0);
            let z = gz + release_dz(piece, anim, t, 0).unwrap_or(0);
            assert_eq!(
                (got.x, got.y, z),
                (sx, sy, sz),
                "piece {piece} anim {anim} t {t}"
            );
        }
    }

    /// A piece nothing has measured keeps the old behaviour exactly.
    #[test]
    fn an_unmeasured_piece_launches_from_the_unit() {
        let p = Pos::new(1234, 5678);
        assert_eq!(
            launch_point(p, Angle(0x1234_5678), 32, crate::anim::ATTACK1, 6, 0),
            p
        );
        assert_eq!(
            launch_point(p, Angle(0), 127, crate::anim::ATTACK1, 7, 0),
            p
        );
    }

    /// The table is the install's own release events and no others — the
    /// key is `rondata::artdata::release_frame`'s frame, so a row this
    /// crate cannot reach is a row that will never fire.
    ///
    /// Piece **127** is the Longbowman's six (run109, §22.1); **472** and
    /// **384** are chapter two's bowmen and slingers, three keys and two,
    /// from run112's own `AMMO` records (item 485, §41.3). The Trireme's
    /// three, **290**, were rows here from run127's (item 542, §50.2) and
    /// are [`BAYS`] rows since run437 turned them (item 1200); their keys
    /// are 5, 9 and 17 because the frame is `starttime / 67` (§50.1):
    /// under the old `× 3 / 200` the first and third would be 6 and 18,
    /// keys the event walk never reaches. **49**, the Musketeer's two, are
    /// run971's (item 3336): 0 ms and 333 ms, keys 1 and 4. **122**, the
    /// Crossbowman's three, are run982's (item 3347): keys 5, 3 and 6. The
    /// Dragoon's, **139**, left for `pivot::RELEASES` (item 3359), the
    /// Cavalry Archer's, **138**, too (item 3633), and the Horse Archer's,
    /// **842** (item 3338), for `pivot::SAME_AS` (item 3658).
    #[test]
    fn the_table_is_every_release_event_a_capture_has_measured() {
        let mut keys: Vec<(i32, i8, u32)> =
            MEASURED.iter().map(|(p, a, t, _)| (*p, *a, *t)).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                (49, crate::anim::ATTACK1, 1),
                (49, crate::anim::ATTACK3, 4),
                (122, crate::anim::ATTACK1, 5),
                (122, crate::anim::ATTACK2, 3),
                (122, crate::anim::ATTACK3, 6),
                (127, crate::anim::ATTACK1, 6),
                (127, crate::anim::ATTACK1, 22),
                (127, crate::anim::ATTACK2, 4),
                (127, crate::anim::ATTACK2, 22),
                (127, crate::anim::ATTACK3, 10),
                (127, crate::anim::ATTACK3, 24),
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
            let got = launch_point(Pos::new(gx, gy), Angle(ga), gp, anim, t, 0);
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

    /// **The Dromon's rounds leave from piece 274's bays** (item 3616,
    /// `docs/AI.md` §327): every round run1260b prints from its first
    /// block, the Dromon `1/24` sailing north at facing 0 on an
    /// `EXPLORETOORDER` and firing at the Farm `0/2004` on
    /// `CHAR_ATTACKWALK` — the hull's `GUY` point on that block, the key
    /// the swing's frame names, and the dump's own `sx, sy, sz`. The same
    /// three offsets come round every 25-frame swing.
    #[test]
    fn run1260b_s_dromon_rounds_leave_from_piece_274_s_bays() {
        // (block, swing frame, hull x, hull y, sx, sy, sz)
        const M: &[(i32, u32, i32, i32, i32, i32, i32)] = &[
            (4499, 10, 2808, 6088, 2809, 6079, 52),
            (4506, 17, 2808, 5668, 2807, 5575, 52),
            (4516, 2, 2808, 5068, 2808, 4881, 52),
            (4524, 10, 2808, 4588, 2809, 4579, 52),
            (4531, 17, 2808, 4168, 2807, 4075, 52),
        ];
        for &(block, t, x, y, sx, sy, sz) in M {
            let got = launch_point(Pos::new(x, y), Angle(0), 274, crate::anim::ATTACKWALK, t, 0);
            assert_eq!((got.x, got.y), (sx, sy), "block {block} t {t}");
            assert_eq!(
                release_dz(274, crate::anim::ATTACKWALK, t, 0),
                Some(sz),
                "block {block}"
            );
        }
    }

    /// **run1270's Inca Archers loose from piece 825's bays** (item 3617):
    /// who=1's three rounds at the Scout `0/0` over 5327..5342, `(shooter,
    /// first block, anim, guy x, guy y, guy z, angle, sx, sy, sz)` off the
    /// `GUY` and `AMMO` records on the round's first block.
    #[test]
    fn run1270_s_inca_archers_loose_from_piece_825_s_bays() {
        type Round = (&'static str, i32, i8, i32, i32, i32, i32, i32, i32, i32);
        const M: &[Round] = &[
            (
                "1/58",
                5328,
                crate::anim::ATTACK1,
                7224,
                8952,
                251,
                477_233_152,
                7278,
                8882,
                414,
            ),
            (
                "1/73",
                5332,
                crate::anim::ATTACK1,
                7080,
                9000,
                240,
                505_610_240,
                7138,
                8933,
                403,
            ),
            (
                "1/74",
                5343,
                crate::anim::ATTACK3,
                7080,
                8376,
                308,
                683_016_192,
                7140,
                8347,
                472,
            ),
        ];
        for &(who, block, anim, x, y, z, angle, sx, sy, sz) in M {
            let time = [
                (crate::anim::ATTACK1, 12),
                (crate::anim::ATTACK2, 9),
                (crate::anim::ATTACK3, 15),
            ]
            .iter()
            .find(|k| k.0 == anim)
            .expect("an attack key")
            .1;
            let got = launch_point(Pos::new(x, y), Angle(angle), 825, anim, time, 0);
            let dz = release_dz(825, anim, time, 0).expect("a bay");
            assert_eq!((got.x, got.y, z + dz), (sx, sy, sz), "{who} block {block}");
        }
    }

    /// **Every trireme launch point run127 recorded on its first two
    /// volleys**, both ships (`docs/COMBAT.md` §50.2). The columns are the
    /// dump's own: the hull's position and angle from its `UNITDATA`, and
    /// `sx, sy, sz` from the `AMMO` record on the block each round first
    /// prints. Both ships hold the same broadside facing, so this pins the
    /// three nodes at one facing, exactly; run437's rows below turn them
    /// (item 1200).
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
                let got = launch_point(Pos::new(x, y), Angle(671_481_856), 290, anim, t, 0);
                assert_eq!((got.x, got.y), (sx, sy), "block {block} t {t}");
                assert_eq!(release_dz(290, anim, t, 0), Some(sz), "block {block}");
            }
        }
    }

    /// **run437's trireme `0/6` at 176°** (item 1200, `docs/GOLDEN.md`
    /// §50): the same three nodes on another whole degree, `(block, guy x,
    /// guy y, starttime, sx, sy)` off the `GUY` record and the `AMMO`
    /// record on the round's first block — every one of its 87 rounds is
    /// one of these three offsets. The planar rows put the third a unit or
    /// two off here (`(−1, +2)` on block 1449), and one of its flights on
    /// 1448 a frame long. Made to fail with the rows back in `MEASURED`.
    #[test]
    fn run437_trireme_rounds_turn_with_the_hull() {
        const M: &[(i32, i32, i32, u32, i32, i32)] = &[
            (1437, 34872, 15864, 5, 34875, 15909),
            (1441, 34872, 15864, 9, 34872, 15850),
            (1449, 34872, 15864, 17, 34870, 15798),
        ];
        for &(block, x, y, t, sx, sy) in M {
            let got = launch_point(
                Pos::new(x, y),
                Angle(2_100_559_872),
                290,
                crate::anim::ATTACK2,
                t,
                0,
            );
            assert_eq!((got.x, got.y), (sx, sy), "block {block} t {t}");
        }
    }

    /// **The Biplane's two guns, piece 237** (item 1200, `docs/GOLDEN.md`
    /// §50): run437's rounds at four of the eleven whole degrees,
    /// `(block, guy x, guy y, guy angle, node, sx, sy)` — the `GUY` record
    /// and the `AMMO` record on the round's first block. Made to fail with
    /// the rows out of `GUNS`: the rounds leave from the figure.
    #[test]
    fn run875_biplane_rounds_leave_from_the_589_guns() {
        for &(x, y, a, node, sx, sy) in &[
            (37463, 16578, -590_544_896, 0, 37377, 16525),
            (37463, 16578, -590_544_896, 1, 37392, 16506),
            (37379, 16508, -590_020_608, 0, 37293, 16455),
            (37379, 16508, -590_020_608, 1, 37308, 16436),
        ] {
            let got = launch_point(Pos::new(x, y), Angle(a), 589, crate::anim::ATTACK2, 1, node);
            assert_eq!((got.x, got.y), (sx, sy), "node {node} at {x}");
            assert_eq!(release_dz(589, crate::anim::ATTACK2, 1, node), Some(39));
        }
    }

    #[test]
    fn run437_biplane_rounds_leave_from_its_nose_guns() {
        const M: &[(i32, i32, i32, i32, i8, i32, i32)] = &[
            (965, 42715, 17577, -816_583_618, 0, 42616, 17553),
            (965, 42715, 17577, -816_583_618, 1, 42625, 17531),
            (1055, 40013, 17031, 1_393_562_276, 0, 40109, 17065),
            (1055, 40013, 17031, 1_393_562_276, 1, 40098, 17087),
            (1608, 42763, 17091, -1_234_463_660, 0, 42668, 17128),
            (1608, 42763, 17091, -1_234_463_660, 1, 42663, 17104),
            (1614, 42500, 17145, -1_224_702_464, 0, 42405, 17180),
            (1614, 42500, 17145, -1_224_702_464, 1, 42399, 17156),
        ];
        for &(block, x, y, a, node, sx, sy) in M {
            let got = launch_point(Pos::new(x, y), Angle(a), 237, crate::anim::ATTACK2, 1, node);
            assert_eq!((got.x, got.y), (sx, sy), "block {block} node {node}");
            assert_eq!(release_dz(237, crate::anim::ATTACK2, 1, node), Some(39));
        }
    }
}
