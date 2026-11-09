//! Where a pivot bears from: the node's offset (`docs/COMBAT.md` §54).
//!
//! `Guy::set_all_pivots@005d8bc0` takes each restricted node's bearing to
//! the target from the unit's point **plus the node's own vector**,
//! `GraphicPieces::get_position@0090b750` truncated by `cvttss2si`
//! (`005d8e19`, `005d8e2d`). The vector is the piece's `AttachPos` entry
//! for `(node, anim 0, time 0)`, which `GraphicEvents::init_unit_events`
//! fills from the model when the piece's events are built, rotated about
//! z by `angle_to_degrees(figure angle − 0x8000_0000)` **whole degrees**,
//! its y negated, and scaled by `guy_scale` (4.8) times the piece's own
//! `RData +0x88` (1.0 for every piece measured here).
//!
//! So for one `(piece, node)` the truncated pair is a function of one
//! integer, `0..=360`, and nothing else. The original gets there in
//! `float`: `Quat<float>::set` reads a 360-entry half-degree table built
//! from `cosf`/`sinf` at first use, and `Matrix<float>::fill` and the
//! product are single precision. This module is the same function in
//! integers: the node's model point in millionths, the scale as the
//! rational 48/10, and a whole-degree sine pinned at 2³⁰
//! (`docs/DECISIONS.md` entry 16's third shape).
//!
//! **Where the two can differ, measured.** Only where the exact product
//! is within the float's own error of an integer, since both truncate.
//! On the Chariot's node the float strays at most 4.0·10⁻⁵ from the exact
//! value over all 361 degrees, and no exact value comes closer than
//! 0.0161 to an integer (`25°`, `65°`, … give 49.9839): a margin of
//! 400×. The original's own rows, from run147's packet, agree on every
//! degree ([`tests::the_original_s_node_agrees_on_every_degree`], which
//! reads the table outside git). A node with another point is its own
//! sweep. The degree is never negative here: `angle_to_degrees` answers
//! `0..=360`, which matters because `Quat<float>::set` indexes its table
//! with `deg % 360`, and a negative degree reads before the table (−60°
//! gives `(−103, −57)` where 300° gives `(−102, −59)`).

use crate::movement::{Angle, angle_to_degrees};

/// A node's model-space point, in **millionths** of a model unit (the
/// float's value, rounded, as [`Release`] carries it; item 3770 widened it
/// from hundredths for piece 143's `−3.322791`), as the
/// piece's `AttachPos` holds it for `(node, anim 0, time 0)`. The height
/// is not carried: the bearing reads x and y only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    /// Across the body.
    pub px: i64,
    /// Along the body.
    pub py: i64,
}

/// `(piece, node) → point`. **Measured, not derived**: run147's packet
/// reads piece 145's `(4, 0, 0)` entry as `(0.0, 24.64, 11.55)` (`0x41c51eb9`
/// is 24.64 to the float's precision), and its `RData +0x88` as 1.0.
/// `set_all_pivots`' own `MISC=10` line prints the result on every
/// Chariot of chapter three, `(−102, −59)` at its 120° facing.
///
/// Piece 145 is the Chariot's figure 0 as `get_unit_gpiece` builds it for
/// chapter three's Nubians (`GUY get_restrictions … 145`). A piece with no
/// row here bears from the unit's own point.
///
/// **Piece 296, the Bomb Vessel** (item 1257): run486's packet, taken on
/// chapter forty-four's game, reads its `(4, 0, 0)` entry as `(0.0,
/// −29.45, 18.78)` (`0xc1eb9999`). The pivot sits behind the hull's
/// centre, where the Chariot's sits ahead of it.
///
/// **Piece 297, the Bomb Ketch** (item 3655, unit type 347, `MORTARSHIP`,
/// `BombKetch.bh3`): run1185's packet (logger frame 4582 of the British
/// Enlightenment lobby, which fields the Ketch) reads its two pivots,
/// `(4, 0, 0)` `(0.0, −50.37, 19.45)` and `(5, 0, 0)` `(0.0, 5.86, 19.45)`
/// (`0xc2497ae1`, `0x40bb851e`), `RData +0x88` 0.8 — one per
/// `<RESTRICTION name="MortarShip">`, the stern mortar on 4 and the bow's
/// on 5.
///
/// **Piece 491, the Dragoon** (item 1776): run773's packet (frame 961, the
/// game that fields it) reads its `(4, 0, 0)` entry as `(0.0, −1.48,
/// 31.05)` (`0xbfbd70a3`), `RData +0x88` 1.0. The bow-side pivot sits a
/// hair behind the figure's centre.
///
/// **Piece 1545, the Native American Horse Archers** (item 3255): run943c's
/// packet reads its `(4, 0, 0)` entry as `(−0.06, −1.48, 32.63)`
/// (`0xbd75c28f`, `0xbfbd70a3`), `RData +0x88` 1.0.
///
/// **Piece 1546, the Iroquois Cavalry Archer** (item 3375): run1001d's
/// packet reads the same `(4, 0, 0)` entry as 1545's.
///
/// **Piece 1898, the Indian Heavy Horse Archer** (item 3383): run1007b's
/// packet reads the same `(4, 0, 0)` entry as 1545's, `RData +0x88` 1.0.
///
/// **Piece 841, the Inca Horse Archer** (item 3385): run1011b's packet
/// reads the same `(4, 0, 0)` entry as 1545's, `RData +0x88` 1.0.
///
/// **Piece 156, the Lakota Eagle Feather Horse** (item 3588, unit type
/// 206): run1235b's packet reads the same `(4, 0, 0)` entry as 1545's,
/// `RData +0x88` 1.0.
///
/// **Piece 157, the Lakota Musket Horse** (item 3645, unit type 207):
/// run1305b's packet (logger frame 4181, the Lakota Gunpowder lobby) reads
/// its `(4, 0, 0)` entry as `(0.08, −1.48, 31.05)` (`0x3da3d70a`),
/// `RData +0x88` 1.0 — the saddle of 139's, set 0.08 to the right where
/// 156's sits 0.06 to the left.
///
/// **Piece 153, the Horde** (item 3632, unit type 203, `HORDE`):
/// run1294's packet (logger frame 4473, the Mongol Gunpowder lobby) reads
/// its `(4, 0, 0)` entry as `(0.0, −3.38, 32.63)` (`0xc05851ec`),
/// `RData +0x88` 1.0 — the saddle a little further back than the Horse
/// Archers'.
///
/// **Piece 168, the Armored Scout Car** (item 3518, `TankDestroyer`'s
/// `ARMOREDCAR-DEFAULT` piece, unit type 218): run1164's packet (logger
/// frame 4528, the American Modern lobby) reads its `(4, 0, 0)` entry as
/// `(0.0, 1.79, 30.92)` (`0x3fe51eba`), `RData +0x88` 0.95.
///
/// **Piece 166, the Armored Car** (item 3511, unit type 216, the Asian
/// set's `armcar.bh3`): run1155c's packet (logger frame 3003, the Chinese
/// Industrial lobby) reads its `(4, 0, 0)` entry as `(0.0, 11.14, 28.07)`
/// (`0x41323d70`), `RData +0x88` 1.1 — one `<RESTRICTION
/// name="ArmoredCar">`, a full circle.
///
/// **Piece 279, the Cruiser** (item 3519, unit type 329): run1165's packet
/// (logger frame 3719, the Russian Modern lobby) reads its three pivots,
/// `(4, 0, 0)` `(0.0, −91.65, 25.95)`, `(5, 0, 0)` `(0.0, −65.48, 34.09)`
/// and `(6, 0, 0)` `(0.0, 105.28, 25.47)` (`0xc2b74ccc`, `0xc282f5c2`,
/// `0x42d28f5c`), `RData +0x88` 0.9 — one per `<RESTRICTION name="Cruiser">`.
///
/// **Piece 298, the Dreadnought** (item 3523, unit type 348): run1169's
/// packet (logger frame 3769, the Japanese Industrial lobby) reads its two
/// pivots, `(4, 0, 0)` `(0.0, −84.88, 21.05)` and `(5, 0, 0)` `(0.0,
/// 100.36, 12.26)` (`0xc2a9c28f`, `0x42c8b851`), `RData +0x88` 0.9 — one
/// per `<RESTRICTION name="Dreadnought">`, the rear arc's `45..−45` on 5.
///
/// **Piece 278, the Destroyer** (item 3525, unit type 328): the same
/// packet reads its three pivots, `(4, 0, 0)` `(0.0, −56.97, 13.57)`,
/// `(5, 0, 0)` `(0.0, −36.95, 21.0)` and `(6, 0, 0)` `(0.0, 107.74, 9.68)`
/// (`0xc263e147`, `0xc213cccd`, `0x42d77ae1`), `RData +0x88` 0.9 — one per
/// `<RESTRICTION name="Destroyer">`, the stern's `50..−50` on 6.
///
/// **Piece 189, the Light Tank** (item 3529, unit type 239): run1169's
/// packet reads its `(4, 0, 0)` entry as `(0.0, −3.96, 37.31)`
/// (`0xc07d70a7`), `RData +0x88` 0.9 — one `<RESTRICTION>`, the turret.
///
/// **Piece 190, the British and Russian Light Tank `MARK4`** (item 3683,
/// unit type 240): run1355's packet (logger frame 2933, the British
/// Industrial Great Sahara lobby) reads its `(4, 0, 0)` entry as `(−0.08,
/// −11.53, 31.2)` (`0xbda3d70a 0xc1387ae2`), `RData +0x88` 0.9 — one
/// `<RESTRICTION name="MARK4">`, the turret. Its own model
/// (`lighttank_mkiv.bh3`), so not piece 189's rows.
///
/// **Piece 299, the Battleship** (item 3736, unit type 349,
/// `superbattleship.bh3`): run1165's packet (logger frame 3719, the Russian
/// Modern lobby) reads its three pivots, `(4, 0, 0)` `(0.0, −95.34, 13.85)`,
/// `(5, 0, 0)` `(0.0, −49.66, 22.79)` and `(6, 0, 0)` `(0.0, 117.49, 13.83)`
/// (`0xc2beae14`, `0xc246a3d6`, `0x42eafae1`), `RData +0x88` 0.9 — one per
/// `<RESTRICTION name="Battleship">` at the foot of `unit_graphics.xml`,
/// not in its own `<UNIT>` block.
///
/// **Piece 200, the Tank `PANZER`** (item 3737, unit type 250,
/// `panzer.bh3`): run1416's packet (logger frame 4016, the Spanish Modern
/// East Indies lobby) reads its `(4, 0, 0)` entry as `(0.0, 5.58, 35.68)`
/// (`0x40b28f5b`), `RData +0x88` 0.8 — one `<RESTRICTION name="PANZER">`,
/// the turret.
///
/// **Piece 204, the Main Battle Tank `ABRAM`** (item 3758, unit type 254,
/// `HvyTank.bh3`): run1437's packet (logger frame 3894, the American
/// Information Great Lakes lobby) reads its `(4, 0, 0)` entry as `(0.0,
/// 0.0, 27.11)` (`0x41d8e147`), `RData +0x88` 0.75 — one `<RESTRICTION
/// name="ABRAM">` at the foot of `unit_graphics.xml`, the turret. Its
/// desert twin `ABRAMDES` plays the same model and is not rowed.
///
/// SEAM: every other restricted piece. The original fills a row for each
/// when its events are built; a packet from a game that fields the type
/// reads it with `tools/recomp/get_position.py … entries <piece>`.
const NODES: &[(i32, i32, Node)] = &[
    // **The Missile Cruiser's four turrets, piece 280** (item 3849,
    // `docs/GOLDEN.md` §153.1): run1484's packet, each node's `(n, 0, 0)`
    // entry — 4 and 5 forward at (0.02, 100.24), 6 and 7 aft at (0.02,
    // −77.14). Bearing from the hull's square, ours turned all four alike;
    // run1484's `1/14` on 666 reads −739,115,008 forward and −1,335,033,856
    // aft.
    (
        280,
        4,
        Node {
            px: 20_000,
            py: 100_240_005,
        },
    ),
    (
        280,
        5,
        Node {
            px: 20_000,
            py: 100_240_005,
        },
    ),
    (
        280,
        6,
        Node {
            px: 20_000,
            py: -77_139_999,
        },
    ),
    (
        280,
        7,
        Node {
            px: 20_000,
            py: -77_139_999,
        },
    ),
    // **The Advanced Battleship's turrets, piece 300** (item 3849): the
    // packet's `(4, 0, 0)` (0.02, −151.34) and `(5, 0, 0)` (0.0557, −176.23).
    (
        300,
        4,
        Node {
            px: 20_000,
            py: -151_339_996,
        },
    ),
    (
        300,
        5,
        Node {
            px: 55_683,
            py: -176_234_314,
        },
    ),
    (
        145,
        4,
        Node {
            px: 0,
            py: 24_640_000,
        },
    ),
    (
        296,
        4,
        Node {
            px: 0,
            py: -29_450_000,
        },
    ),
    (
        297,
        4,
        Node {
            px: 0,
            py: -50_370_000,
        },
    ),
    (
        297,
        5,
        Node {
            px: 0,
            py: 5_860_000,
        },
    ),
    (
        491,
        4,
        Node {
            px: 0,
            py: -1_480_000,
        },
    ),
    (
        137,
        4,
        Node {
            px: -60_000,
            py: -1_480_000,
        },
    ),
    (
        149,
        4,
        Node {
            px: 0,
            py: -1_480_000,
        },
    ),
    (
        150,
        4,
        Node {
            px: 0,
            py: -3_380_000,
        },
    ),
    (
        194,
        4,
        Node {
            px: 0,
            py: -960_000,
        },
    ),
    (
        1545,
        4,
        Node {
            px: -60_000,
            py: -1_480_000,
        },
    ),
    (
        1546,
        4,
        Node {
            px: -60_000,
            py: -1_480_000,
        },
    ),
    (
        1898,
        4,
        Node {
            px: -60_000,
            py: -1_480_000,
        },
    ),
    (
        841,
        4,
        Node {
            px: -60_000,
            py: -1_480_000,
        },
    ),
    (
        156,
        4,
        Node {
            px: -60_000,
            py: -1_480_000,
        },
    ),
    (
        157,
        4,
        Node {
            px: 80_000,
            py: -1_480_000,
        },
    ),
    (
        153,
        4,
        Node {
            px: 0,
            py: -3_380_000,
        },
    ),
    (
        139,
        4,
        Node {
            px: 0,
            py: -1_480_000,
        },
    ),
    (
        168,
        4,
        Node {
            px: 0,
            py: 1_790_000,
        },
    ),
    (
        170,
        4,
        Node {
            px: -4_480_000,
            py: 14_200_000,
        },
    ),
    (
        166,
        4,
        Node {
            px: 0,
            py: 11_140_000,
        },
    ),
    (
        279,
        4,
        Node {
            px: 0,
            py: -91_650_000,
        },
    ),
    (
        279,
        5,
        Node {
            px: 0,
            py: -65_480_000,
        },
    ),
    (
        279,
        6,
        Node {
            px: 0,
            py: 105_280_000,
        },
    ),
    (
        298,
        4,
        Node {
            px: 0,
            py: -84_880_000,
        },
    ),
    (
        298,
        5,
        Node {
            px: 0,
            py: 100_360_000,
        },
    ),
    (
        278,
        4,
        Node {
            px: 0,
            py: -56_970_000,
        },
    ),
    (
        278,
        5,
        Node {
            px: 0,
            py: -36_950_000,
        },
    ),
    (
        278,
        6,
        Node {
            px: 0,
            py: 107_740_000,
        },
    ),
    (
        189,
        4,
        Node {
            px: 0,
            py: -3_960_000,
        },
    ),
    (
        190,
        4,
        Node {
            px: -80_000,
            py: -11_530_000,
        },
    ),
    (
        299,
        4,
        Node {
            px: 0,
            py: -95_340_000,
        },
    ),
    (
        299,
        5,
        Node {
            px: 0,
            py: -49_660_000,
        },
    ),
    (
        299,
        6,
        Node {
            px: 0,
            py: 117_490_000,
        },
    ),
    (
        208,
        4,
        Node {
            px: 0,
            py: 3_130_000,
        },
    ),
    (
        196,
        4,
        Node {
            px: 0,
            py: -970_000,
        },
    ),
    (169, 4, Node { px: 0, py: -70_000 }),
    (
        200,
        4,
        Node {
            px: 0,
            py: 5_580_000,
        },
    ),
    (207, 4, Node { px: 0, py: 0 }),
    (
        199,
        4,
        Node {
            px: -5_690_000,
            py: -9_190_000,
        },
    ),
    (204, 4, Node { px: 0, py: 0 }),
    (
        143,
        4,
        Node {
            px: 20,
            py: -3_322_791,
        },
    ),
    (
        141,
        4,
        Node {
            px: 908_200,
            py: -122_588,
        },
    ),
];

/// `guy_scale` (`0xc06244`, `4.8f`), as the rational it was typed as.
const SCALE: (i64, i64) = (48, 10);

/// **The piece's own `RData +0x88`**, which `get_position` multiplies
/// `guy_scale` by (`90b7ad`; `local_28` in the decompile), as a rational:
/// 1.0 for the Chariot's piece 145 and **0.8** for the Bomb Vessel's 296
/// (`0x3f4ccccd`, run486's packet). A piece with no row is 1.0.
const RDATA_SCALE: &[(i32, (i64, i64))] = &[
    (296, (4, 5)),
    (297, (4, 5)),
    (194, (9, 10)),
    (279, (9, 10)),
    (168, (19, 20)),
    (170, (7, 10)),
    (166, (11, 10)),
    (298, (9, 10)),
    (278, (9, 10)),
    (189, (9, 10)),
    (190, (9, 10)),
    (299, (9, 10)),
    (208, (17, 20)),
    (196, (17, 20)),
    (169, (11, 10)),
    (200, (4, 5)),
    (207, (3, 4)),
    (199, (4, 5)),
    (204, (3, 4)),
    (143, (9, 10)),
    (144, (9, 10)),
    (141, (9, 10)),
    (201, (4, 5)),
    (280, (9, 10)),
    (300, (4, 5)),
    (223, (69, 100)),
];

/// [`SCALE`] times the piece's [`RDATA_SCALE`], as `(num, den)`.
fn scale(piece: i32) -> (i64, i64) {
    let (n, d) = RDATA_SCALE
        .iter()
        .find(|(p, _)| *p == piece)
        .map_or((1, 1), |&(_, r)| r);
    (SCALE.0 * n, SCALE.1 * d)
}

/// `sin(k°)` for `k = 0..=90`, rounded at 2³⁰. The nearest any entry
/// comes to a rounding tie is 0.0076 of a unit, so the table is the
/// sine's, not a double's.
#[rustfmt::skip]
const SIN_DEG: [i64; 91] = [
    0, 18739379, 37473049, 56195305, 74900443, 93582766,
    112236583, 130856211, 149435979, 167970228, 186453311, 204879599,
    223243478, 241539355, 259761657, 277904834, 295963357, 313931728,
    331804471, 349576144, 367241333, 384794656, 402230767, 419544355,
    436730145, 453782903, 470697435, 487468587, 504091252, 520560366,
    536870912, 553017922, 568996477, 584801711, 600428808, 615873009,
    631129609, 646193961, 661061475, 675727625, 690187940, 704438018,
    718473518, 732290163, 745883746, 759250125, 772385229, 785285058,
    797945680, 810363241, 822533958, 834454122, 846120104, 857528349,
    868675383, 879557810, 890172315, 900515665, 910584710, 920376381,
    929887697, 939115760, 948057759, 956710970, 965072759, 973140576,
    980911966, 988384560, 995556083, 1002424350, 1008987269, 1015242840,
    1021189159, 1026824413, 1032146887, 1037154959, 1041847103, 1046221891,
    1050277989, 1054014162, 1057429273, 1060522280, 1063292242, 1065738315,
    1067859754, 1069655912, 1071126243, 1072270298, 1073087729, 1073578288,
    1073741824,
];

/// The table's scale.
const ONE: i64 = 1 << 30;

/// `sin(d°)` at 2³⁰, for any whole degree.
const fn sin_deg(d: i32) -> i64 {
    let d = d.rem_euclid(360) as usize;
    match d {
        0..=90 => SIN_DEG[d],
        91..=180 => SIN_DEG[180 - d],
        181..=270 => -SIN_DEG[d - 180],
        _ => -SIN_DEG[360 - d],
    }
}

/// `cos(d°)` at 2³⁰.
const fn cos_deg(d: i32) -> i64 {
    sin_deg(d + 90)
}

/// **A piece whose `AttachPos` rows are another's, bit for bit**: `(piece,
/// the piece it reads as)`. The rows are measured on the piece itself and
/// compared, never assumed from the unit's name.
///
/// **Piece 1195, the Chinese Dragoon** (item 3639, `docs/AI.md` §333.2):
/// run1237's packet (logger frame 2674, the Japanese Gunpowder lobby)
/// holds its nine entries, `pos_start` 102, `RData +0x88` 1.0, and every
/// float is piece 139's — the `(4, 0, 0)` pivot `(0.0, −1.48, 31.05)`
/// (`0xbfbd70a3`, `0x41f86666`) and the four `<RELEASEEVENT>`s' pivot and
/// release rows (`CHAR_ATTACKWALK` 12 at `0xc14f6142 0xc1dc8ff1
/// 0x413b1ad8`, …). The Chinese Gunpowder pair's Dragoon `1/97`
/// (TypeIndex 189) threw its `CHAR_ATTACKWALK` round of 5130 from its own
/// square here, flew four frames and wounded the Citizen `0/10` on 5133,
/// where the original's left from (8009, 10001, 470), flew three and
/// struck on 5132.
///
/// **Piece 842, the Horse Archer's style-2 art** (unit type 188,
/// `CAVARCHERS`, as the Aztecs, Maya and Inca draw it; item 3658, `docs/AI.md`
/// §338): run973g's packet (item 3338, the Maya Great Sahara seed) holds its
/// nine entries, `pos_start` 146, `RData +0x88` 1.0, and every float is
/// piece 138's off run1293's — the `(4, 0, 0)` pivot and the four
/// `<RELEASEEVENT>`s' pivot and release rows, `CHAR_ATTACKWALK` 18 among
/// them, which `launch::MEASURED`'s planar rows never had. The Aztec
/// Medieval pair's `1/22` threw its `CHAR_ATTACKWALK` round of 2556 from its
/// own square (7018, 31206), where the original's left from (7020, 31282,
/// 873), and the round's arc came down a frame later than theirs.
///
/// **Piece 192, the German Light Tank `A7V`** (unit type 242; item 3677,
/// `docs/AI.md` §346): run1351's packet (logger frame 4303, the German
/// Industrial Great Lakes lobby) holds its seven entries, `pos_start` 107,
/// `RData +0x88` 0.9, and every float is piece 189's off run1169 — the
/// `(4, 0, 0)` turret `(0.0, −3.96, 37.31)` (`0xc07d70a7`) and the three
/// `<RELEASEEVENT>`s' turret and muzzle rows (node 8; 4 of `CHAR_ATTACK1`,
/// 21 of `2`, 14 of `3`). `<RESTRICTION name="A7V" node="4">` makes it a
/// pivot piece. who=1's `1/48` fired its `CHAR_ATTACK3` round of 4304 from
/// the hull's square (5544, 31512, 409) here, where the original's left the
/// muzzle at (5427, 31534, 587).
///
/// **Piece 154, the Golden Horde** (unit type 204, `GOLDENHORDE`,
/// `nomad_goldenhorde.bh3`; item 3692, `docs/AI.md` §352.1): run1361's
/// packet (logger frame 4840, the Mongol Enlightenment Great Lakes lobby)
/// holds its nine entries, `pos_start` 126, `RData +0x88` 1.0, and every
/// float is piece 153's off run1294 — the `(4, 0, 0)` pivot `(0.0, −3.38,
/// 32.63)` (`0xc05851ec`) and the four `<RELEASEEVENT>`s' pivot and release
/// rows. `<RESTRICTION name="GOLDENHORDE" node="4">` makes it a pivot
/// piece. who=1's `1/97` loosed its `CHAR_ATTACK2` round of 4841 from its
/// own square (6264, 28728, 480) here, where the original's left from
/// (6180, 28775, 715), flew 11 frames against 10, and struck the Citizen
/// `0/6` a frame late.
///
/// **Piece 193, the Egyptian Light Tank `A7VDES`** (unit type 243; item
/// 3707, `docs/AI.md` §360.2): `unit_graphics.xml`'s `A7VDES-DEFAULT-AGE0` is
/// piece 189's `lighttank.bh3` at scale 0.9 under a desert texture, with its
/// anim files, its three `HOWITZER BLAST`s on node 8 and `<RESTRICTION
/// name="A7VDES" node="4">`. No packet on disk loads the piece (all 82
/// tried); run1374b's seven Light Tank rounds fit piece 189's rows, one key
/// each. who=1's `1/54` loosed its round of 4940 at the human's Citizen
/// `0/10` from the hull's square (6120, 10968, 22) here, where the
/// original's left the muzzle at (6098, 10851, 200), and `0/10` was struck
/// on 4943 in theirs alone.
///
/// **Piece 144, the Nubian Camel Raider of the later ages** (unit type 194,
/// `ECAMELRAIDER`; item 3855, `docs/GOLDEN.md` §153.3): run1484's packet
/// (logger frame 676 of the fourth melee's lobby) holds its nine entries,
/// `pos_start` 240, `RData +0x88` 0.9, and every float is piece 143's off
/// run1440, bit for bit — the `(4, 0, 0)` pivot `(0.0, −3.32, 49.24)` and
/// the four `<RELEASEEVENT>`s' pivot and release rows (node 0; 12 of
/// `CHAR_ATTACKWALK` and `CHAR_ATTACK1`, 13 of `2`, 18 of `3`).
/// `<RESTRICTION name="CAMELRAIDER" node="4">` makes it a pivot piece. The
/// Nubian `0/14` loosed its `CHAR_ATTACK2` round of 1430 from its own square
/// (30456, 27384, 17) here and flew 7 frames, where the original's left from
/// (30461, 27533, 298) — the pivot's 48.57 and the release node's 16.68
/// over the figure — flew 6, and struck the Aircraft Carrier `2/18` on 1434
/// in theirs alone.
const SAME_AS: &[(i32, i32)] = &[
    (1195, 139),
    (842, 138),
    (192, 189),
    (154, 153),
    (193, 189),
    (195, 194),
    (144, 143),
];

/// The piece whose rows `piece` reads: [`SAME_AS`]'s, or its own.
fn rows_of(piece: i32) -> i32 {
    SAME_AS
        .iter()
        .find(|(p, _)| *p == piece)
        .map_or(piece, |&(_, q)| q)
}

/// The node's point.
pub fn node(piece: i32, node: i32) -> Option<Node> {
    let piece = rows_of(piece);
    NODES
        .iter()
        .find(|(p, n, _)| (*p, *n) == (piece, node))
        .map(|&(_, _, pt)| pt)
}

/// The degree `set_all_pivots` rotates by: `angle_to_degrees` of the
/// figure's angle less a half turn (`005d8dac`–`005d8db7`), `0..=360`.
pub const fn rotation(facing: Angle) -> i32 {
    angle_to_degrees(Angle(facing.0.wrapping_sub(i32::MIN)))
}

/// **The node's vector from the unit's point**, `(v₀, v₁)`, both
/// truncated toward zero as `set_all_pivots` truncates them: `v₀ =
/// s·(px·cos d + py·sin d)` and `v₁ = s·(px·sin d − py·cos d)`, with `s`
/// the scale and `d` = [`rotation`]. The convention is the original's,
/// fitted to 361 degrees of two nodes on run147's packet (the pivot node,
/// and a release node with `px ≠ 0`) to within 2.6·10⁻⁵. `(0, 0)` for a
/// piece with no row, which is what `get_position` writes when it finds
/// no entry.
pub fn offset(piece: i32, node_index: i32, facing: Angle) -> (i32, i32) {
    offset_at(piece, node_index, rotation(facing))
}

/// [`offset`] at a rotation of `d` whole degrees. `Quat<float>::set`
/// takes `d % 360`, so 360 is 0.
pub fn offset_at(piece: i32, node_index: i32, d: i32) -> (i32, i32) {
    let Some(n) = node(piece, node_index) else {
        return (0, 0);
    };
    // Toward zero, as `cvttss2si` truncates; in `i128`: a millionth's product with the 2³⁰ sine and the scale
    // passes `i64` on the Battleship's 117.49.
    let (px, py) = (i128::from(n.px), i128::from(n.py));
    let (c, s) = (i128::from(cos_deg(d)), i128::from(sin_deg(d)));
    let (num, den) = scale(piece);
    let (num, den) = (
        i128::from(num),
        i128::from(den) * 1_000_000 * i128::from(ONE),
    );
    (
        (num * (px * c + py * s) / den) as i32,
        (num * (px * s - py * c) / den) as i32,
    )
}

// ------------------------------------------------------------------
// The release on a pivot piece (`docs/COMBAT.md` §55)
// ------------------------------------------------------------------

/// One release event on a pivot piece: the event's node, and the two
/// `AttachPos` entries `get_position`'s pivot branch reads for it, the
/// pivot node's `((node & 3) + 4, anim, time)` and the release node's own
/// `(node, anim, time)`, each `(x, y, z)` in **millionths** of a model
/// unit (the float's value, rounded; the nearest is 1.2·10⁻⁷ away).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Release {
    /// The piece the row is for: its [`RDATA_SCALE`] scales the result.
    pub piece: i32,
    /// The event's `node` (`GraphicEvent +0x23`).
    pub node: i32,
    /// The pivot node's entry at the event's anim and time.
    pub pivot: [i64; 3],
    /// The release node's entry.
    pub at: [i64; 3],
    /// **Where the float lands on the integer**: `(d₁, t, x, y)` for the
    /// cells the original's single-precision sum truncates differently
    /// from the exact value, because it rounds onto the integer from
    /// just below (`97.0` against 96.99999…). Measured, and complete: the
    /// original's `get_position` under unicorn on every reachable cell
    /// (361 facings × the 256 turret steps, for each row), and these
    /// are the only disagreements
    /// ([`tests::the_original_s_release_agrees_on_every_cell`]).
    pub float_lands: &'static [(i32, i32, i32, i32)],
}

/// Piece 223's [`Release::float_lands`], every row's (item 3975, run883b's
/// packet; the eight sweeps agree with the exact form on every heading).
const FLOAT_LANDS_223: &[(i32, i32, i32, i32)] = &[];
/// Piece 170's [`Release::float_lands`], one list per `(anim, time)` row
/// (item 3756, run854's packet).
const FLOAT_LANDS_170_A10_T4: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_170_A10_T19: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_170_A11_T4: &[(i32, i32, i32, i32)] = &[(281, 319, 87, -72)];
const FLOAT_LANDS_170_A11_T19: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_170_A12_T4: &[(i32, i32, i32, i32)] =
    &[(278, 295, 9, -138), (277, 356, 106, -12)];
const FLOAT_LANDS_170_A12_T19: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_170_A13_T4: &[(i32, i32, i32, i32)] = &[(281, 319, 87, -72)];
const FLOAT_LANDS_170_A13_T19: &[(i32, i32, i32, i32)] = &[];
/// Piece 143's [`Release::float_lands`], one list per anim (item 3770,
/// run1440's packet).
const FLOAT_LANDS_143_A10: &[(i32, i32, i32, i32)] = &[
    (212, 52, 140, 27),
    (302, 52, -27, 140),
    (343, 280, 136, 57),
    (130, 353, -88, -127),
    (310, 353, 88, 127),
];
const FLOAT_LANDS_143_A11: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_143_A12: &[(i32, i32, i32, i32)] = &[(204, 69, 114, 28)];
const FLOAT_LANDS_143_A13: &[(i32, i32, i32, i32)] = &[];
/// Piece 141's [`Release::float_lands`], one list per anim (item 3785,
/// run1457's packet).
const FLOAT_LANDS_141_A10: &[(i32, i32, i32, i32)] = &[(173, 125, 45, 21)];
const FLOAT_LANDS_141_A11: &[(i32, i32, i32, i32)] =
    &[(89, 277, -5, 52), (179, 277, -52, -5), (359, 277, 52, 5)];
const FLOAT_LANDS_141_A12: &[(i32, i32, i32, i32)] = &[(142, 121, 35, 3), (232, 121, -3, 35)];
const FLOAT_LANDS_141_A13: &[(i32, i32, i32, i32)] = &[(219, 30, 44, -9)];
/// Piece 204's [`Release::float_lands`], one list per `(anim, time)` row
/// (item 3758, run1437's packet).
const FLOAT_LANDS_204_A11_T11: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_204_A12_T23: &[(i32, i32, i32, i32)] = &[];
/// The four tracers' (item 3760, run1437's packet).
const FLOAT_LANDS_204_A12_T1: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_204_A12_T7: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_204_A13_T1: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_204_A13_T7: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_204_A13_T27: &[(i32, i32, i32, i32)] = &[
    (286, 59, 75, 279),
    (254, 91, 75, 279),
    (40, 135, -25, -289),
    (130, 135, 289, -25),
    (220, 135, 25, 289),
    (151, 204, 26, 289),
    (241, 204, -289, 26),
    (331, 204, -26, -289),
    (50, 215, 289, -26),
    (140, 215, 26, 289),
    (320, 215, -26, -289),
    (82, 243, 167, 236),
    (130, 246, -79, 278),
    (220, 246, -278, -79),
    (29, 326, 26, 288),
    (50, 326, -79, 278),
    (119, 326, -288, 26),
    (140, 326, -278, -79),
    (209, 326, -26, -288),
];
/// Piece 169's [`Release::float_lands`], one list per `(anim, time)` row
/// (item 3738, run1417's packet).
const FLOAT_LANDS_169_A10_T3: &[(i32, i32, i32, i32)] =
    &[(81, 6, -118, 3), (171, 6, -3, -118), (261, 6, 118, -3)];
const FLOAT_LANDS_169_A10_T9: &[(i32, i32, i32, i32)] = &[(199, 107, 101, 70)];
const FLOAT_LANDS_169_A10_T17: &[(i32, i32, i32, i32)] = &[(11, 135, -66, -97), (191, 135, 66, 97)];
const FLOAT_LANDS_169_A10_T22: &[(i32, i32, i32, i32)] = &[(161, 122, 109, 28)];
const FLOAT_LANDS_169_A10_T24: &[(i32, i32, i32, i32)] = &[(253, 34, 111, 28)];
const FLOAT_LANDS_169_A10_T29: &[(i32, i32, i32, i32)] = &[(123, 335, -117, -15)];
const FLOAT_LANDS_169_A12_T3: &[(i32, i32, i32, i32)] = &[
    (105, 90, 30, -113),
    (285, 90, -30, 113),
    (75, 120, 30, -113),
    (255, 120, -30, 113),
    (105, 240, 30, 113),
    (285, 240, -30, -113),
    (75, 270, 30, 113),
    (255, 270, -30, -113),
];
const FLOAT_LANDS_169_A12_T12: &[(i32, i32, i32, i32)] = &[
    (49, 4, -94, 70),
    (139, 4, -70, -94),
    (319, 4, 70, 94),
    (41, 356, -70, 94),
    (221, 356, 70, -94),
];
const FLOAT_LANDS_169_A12_T17: &[(i32, i32, i32, i32)] = &[
    (32, 28, -101, 59),
    (122, 28, -59, -101),
    (212, 28, 101, -59),
    (302, 28, 59, 101),
    (58, 332, -59, 101),
    (148, 332, -101, -59),
    (238, 332, 59, -101),
];
const FLOAT_LANDS_169_A12_T29: &[(i32, i32, i32, i32)] = &[
    (277, 23, 102, 58),
    (263, 37, 102, 58),
    (75, 193, 116, -4),
    (165, 193, 4, 116),
    (277, 323, 102, -58),
    (15, 343, 4, 117),
    (285, 343, 117, -4),
];
/// Piece 199's [`Release::float_lands`], one list per anim
/// (item 3745, run1424's packet).
const FLOAT_LANDS_199_A11: &[(i32, i32, i32, i32)] = &[(172, 55, 105, -111), (349, 291, 97, 65)];
const FLOAT_LANDS_199_A12: &[(i32, i32, i32, i32)] = &[(97, 51, -70, -101)];
const FLOAT_LANDS_199_A13: &[(i32, i32, i32, i32)] = &[
    (302, 28, 70, 140),
    (29, 79, -144, -22),
    (119, 79, 22, -144),
    (209, 79, 144, 22),
    (299, 79, -22, 144),
    (46, 159, 14, -93),
    (136, 159, 93, 14),
    (226, 159, -14, 93),
];
/// Piece 207's [`Release::float_lands`], one list per anim
/// (item 3743, run1419b's packet).
const FLOAT_LANDS_207_A11: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_207_A12: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_207_A13: &[(i32, i32, i32, i32)] = &[
    (286, 59, 75, 279),
    (254, 91, 75, 279),
    (40, 135, -25, -289),
    (130, 135, 289, -25),
    (220, 135, 25, 289),
    (151, 204, 26, 289),
    (241, 204, -289, 26),
    (331, 204, -26, -289),
    (50, 215, 289, -26),
    (140, 215, 26, 289),
    (320, 215, -26, -289),
    (82, 243, 167, 236),
    (130, 246, -79, 278),
    (220, 246, -278, -79),
    (29, 326, 26, 288),
    (50, 326, -79, 278),
    (119, 326, -288, 26),
    (140, 326, -278, -79),
    (209, 326, -26, -288),
];
/// Piece 196's [`Release::float_lands`], one list per anim (item 3766,
/// run1164's packet).
const FLOAT_LANDS_196_A11: &[(i32, i32, i32, i32)] = &[
    (25, 11, -149, 203),
    (115, 11, -203, -149),
    (86, 187, 243, 15),
    (68, 281, 42, 245),
    (158, 281, -245, 42),
    (274, 359, 251, 15),
];
const FLOAT_LANDS_196_A12: &[(i32, i32, i32, i32)] = &[
    (79, 7, -219, 14),
    (169, 7, -14, -219),
    (259, 7, 219, -14),
    (349, 7, 14, 219),
    (92, 68, -76, -203),
    (272, 68, 76, 203),
    (88, 72, -76, -203),
    (268, 72, 76, 203),
    (11, 345, 12, 219),
    (101, 345, -219, 12),
    (191, 345, -12, -219),
    (281, 345, 219, -12),
];
const FLOAT_LANDS_196_A13: &[(i32, i32, i32, i32)] = &[
    (48, 44, -252, -7),
    (138, 44, 7, -252),
    (228, 44, 252, 7),
    (318, 44, -7, 252),
    (150, 186, 97, 225),
    (330, 186, -97, -225),
    (320, 225, 26, -245),
    (220, 325, 26, -251),
];
/// Piece 208's [`Release::float_lands`], one list per anim (item 3738,
/// run1417's packet).
const FLOAT_LANDS_208_A11: &[(i32, i32, i32, i32)] = &[
    (125, 42, -76, -395),
    (215, 42, 395, -76),
    (16, 166, 24, -424),
    (106, 166, 424, 24),
    (196, 166, -24, 424),
    (286, 166, -424, -24),
    (74, 198, 424, 17),
    (164, 198, -17, 424),
    (254, 198, -424, -17),
    (344, 198, 17, -424),
];
const FLOAT_LANDS_208_A12: &[(i32, i32, i32, i32)] = &[
    (116, 98, 250, -336),
    (64, 150, 250, -347),
    (262, 333, 332, -231),
];
const FLOAT_LANDS_208_A13: &[(i32, i32, i32, i32)] = &[
    (57, 30, -405, 8),
    (147, 30, -8, -405),
    (327, 30, 8, 405),
    (197, 51, 384, -138),
    (75, 239, 307, 290),
    (165, 239, -290, 307),
    (345, 239, 290, -307),
];
/// Piece 279's [`Release::float_lands`], one list per turret (item 3519):
/// the three anims' entries are the same bits, so one sweep a node.
const FLOAT_LANDS_279_N0: &[(i32, i32, i32, i32)] = &[
    (31, 14, -245, 394),
    (59, 222, -272, 202),
    (69, 21, -438, 151),
    (108, 44, -418, -178),
    (121, 14, -394, -245),
    (198, 44, 178, -418),
    (203, 6, 179, -428),
    (211, 14, 245, -394),
    (233, 156, 296, -171),
    (239, 222, 272, -202),
    (249, 21, 438, -151),
    (282, 38, 439, 128),
    (301, 14, 394, 245),
    (329, 222, 202, 272),
];
const FLOAT_LANDS_279_N1: &[(i32, i32, i32, i32)] = &[
    (21, 24, -139, 310),
    (111, 24, -310, -139),
    (185, 72, 83, -304),
    (185, 134, 74, -245),
    (201, 24, 139, -310),
    (291, 24, 310, 139),
];
const FLOAT_LANDS_279_N2: &[(i32, i32, i32, i32)] = &[
    (25, 280, 242, -389),
    (38, 207, 324, -394),
    (115, 280, 389, 242),
    (128, 207, 394, 324),
    (167, 105, 163, 434),
    (205, 280, -242, 389),
    (218, 207, -324, 394),
    (242, 240, -453, 194),
    (257, 105, -434, 163),
    (267, 260, -476, -26),
    (308, 207, -394, -324),
    (354, 42, -76, -399),
];

/// Piece 299's [`Release::float_lands`], one list per row (item 3736):
/// `CHAR_ATTACK1`'s and `CHAR_ATTACK2`'s entries differ in their bits, so
/// each row is its own sweep.
const FLOAT_LANDS_299_A11_N0: &[(i32, i32, i32, i32)] = &[
    (108, 34, -483, -246),
    (110, 160, -237, -141),
    (116, 184, -240, -106),
    (198, 34, 246, -483),
    (200, 160, 141, -237),
    (320, 297, 409, 282),
];
const FLOAT_LANDS_299_A11_N2: &[(i32, i32, i32, i32)] = &[
    (51, 270, 483, -207),
    (104, 295, 400, 232),
    (113, 332, 323, 207),
    (141, 270, 207, 483),
    (203, 332, -207, 323),
    (282, 236, -549, -238),
    (321, 236, -277, -531),
    (321, 270, -207, -483),
];
const FLOAT_LANDS_299_A11_N1: &[(i32, i32, i32, i32)] = &[
    (47, 233, -24, 169),
    (137, 233, -169, -24),
    (227, 233, 24, -169),
    (317, 233, 169, 24),
];
const FLOAT_LANDS_299_A12_N0: &[(i32, i32, i32, i32)] = &[
    (70, 7, -532, 175),
    (72, 70, -484, 10),
    (86, 83, -440, -116),
    (113, 255, -399, -12),
    (162, 70, -10, -484),
    (176, 83, 116, -440),
    (203, 255, 12, -399),
    (252, 70, 484, -10),
    (266, 83, 440, 116),
    (293, 255, 399, 12),
    (302, 190, 238, 118),
    (342, 70, 10, 484),
    (356, 83, -116, 440),
];
const FLOAT_LANDS_299_A12_N2: &[(i32, i32, i32, i32)] = &[
    (26, 342, 205, -311),
    (45, 70, 228, -417),
    (116, 342, 311, 205),
    (187, 34, 30, 392),
    (204, 170, -241, 604),
    (267, 0, -361, 15),
    (277, 34, -392, 30),
    (296, 342, -311, -205),
];
const FLOAT_LANDS_299_A12_N1: &[(i32, i32, i32, i32)] = &[
    (82, 352, -344, 75),
    (105, 7, -338, -100),
    (172, 352, -75, -344),
    (285, 7, 338, 100),
    (352, 352, 75, 344),
];

/// Piece 200's [`Release::float_lands`], one list per anim (item 3737).
const FLOAT_LANDS_200_A11: &[(i32, i32, i32, i32)] = &[
    (91, 177, 282, -7),
    (142, 155, 245, 136),
    (218, 79, 218, 136),
    (269, 359, 239, -7),
];
const FLOAT_LANDS_200_A12: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_200_A13: &[(i32, i32, i32, i32)] = &[];

/// Piece 189's [`Release::float_lands`], one list per anim (item 3529).
const FLOAT_LANDS_189_A11: &[(i32, i32, i32, i32)] = &[(35, 193, 82, -72), (305, 193, -72, -82)];
const FLOAT_LANDS_189_A12: &[(i32, i32, i32, i32)] = &[
    (14, 262, 114, 26),
    (104, 262, -26, 114),
    (194, 262, -114, -26),
    (284, 262, 26, -114),
];
const FLOAT_LANDS_189_A13: &[(i32, i32, i32, i32)] = &[];

/// Piece 190's [`Release::float_lands`] (item 3683): the three anims'
/// entries are the same bits, so one sweep serves all three rows.
const FLOAT_LANDS_190: &[(i32, i32, i32, i32)] = &[];

/// Piece 201's [`Release::float_lands`], one list per anim (item 3810,
/// run1472's packet).
const FLOAT_LANDS_201_A11: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_201_A12: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_201_A13: &[(i32, i32, i32, i32)] = &[
    (227, 60, 295, 90),
    (255, 70, 177, 252),
    (252, 212, -300, -73),
    (342, 212, 73, -300),
    (4, 235, 264, -160),
    (94, 235, 160, 264),
    (86, 243, 160, 264),
    (356, 243, 264, -160),
    (108, 356, -299, -73),
    (198, 356, 73, -299),
];

/// Piece 298's [`Release::float_lands`], one list per anim and turret
/// (item 3523): `CHAR_ATTACK1`'s and `CHAR_ATTACK2`'s entries differ in
/// their bits, so each row is its own sweep.
const FLOAT_LANDS_298_A11_N0: &[(i32, i32, i32, i32)] = &[
    (43, 328, -275, 382),
    (82, 176, -247, 24),
    (133, 328, -382, -275),
    (172, 285, -166, -378),
    (223, 328, 275, -382),
    (250, 35, 459, -91),
    (258, 221, 256, -132),
    (262, 285, 378, -166),
    (269, 325, 462, -72),
    (313, 328, 382, 275),
    (335, 28, 146, 451),
    (352, 285, 166, 378),
];
const FLOAT_LANDS_298_A11_N1: &[(i32, i32, i32, i32)] = &[
    (37, 301, 300, -238),
    (49, 160, 384, -389),
    (106, 218, 485, 213),
    (114, 97, 460, 72),
    (120, 94, 444, 117),
    (139, 160, 389, 384),
    (204, 97, -72, 460),
    (217, 301, -300, 238),
    (229, 160, -384, 389),
    (254, 169, -522, 174),
    (294, 97, -460, -72),
    (340, 87, -261, -361),
];
const FLOAT_LANDS_298_A12_N0: &[(i32, i32, i32, i32)] = &[
    (59, 302, -314, 309),
    (72, 170, -244, 59),
    (149, 302, -309, -314),
    (162, 170, -59, -244),
    (165, 274, -213, -329),
    (226, 233, 145, -274),
    (268, 314, 444, -104),
    (329, 302, 309, 314),
];
const FLOAT_LANDS_298_A12_N1: &[(i32, i32, i32, i32)] =
    &[(41, 242, 402, -300), (221, 242, -402, 300)];

/// Piece 278's [`Release::float_lands`], one list per turret (item 3525):
/// the three anims' entries are the same bits, so one sweep a node.
const FLOAT_LANDS_278_N0: &[(i32, i32, i32, i32)] = &[(220, 24, 214, -227)];
const FLOAT_LANDS_278_N1: &[(i32, i32, i32, i32)] = &[(321, 3, 140, 166)];
const FLOAT_LANDS_278_N2: &[(i32, i32, i32, i32)] = &[(77, 124, 464, -163), (167, 124, 163, 464)];

/// Piece 297's [`Release::float_lands`], one list per turret (item 3655):
/// `CHAR_ATTACK1` and `3` are the same bits on node 0.
const FLOAT_LANDS_297_N0: &[(i32, i32, i32, i32)] = &[
    (27, 325, -83, 202),
    (117, 325, -202, -83),
    (207, 325, 83, -202),
    (297, 325, 202, 83),
    (325, 345, 134, 177),
];
const FLOAT_LANDS_297_N1: &[(i32, i32, i32, i32)] = &[];

/// `(piece, anim, starttime frame) → release`. **Measured, not derived**:
/// run147's packet holds piece 145's nine `AttachPos` entries, the
/// `(4, 0, 0)` [`NODES`] reads and, for each of the Chariot's four
/// `<RELEASEEVENT>`s (node 0; 1232 ms on `CHAR_ATTACK1`, `CHAR_ATTACK3`
/// and `CHAR_ATTACKWALK`, 1166 on `CHAR_ATTACK2`: frames 18 and 17), the
/// pivot node's and the release node's own. `GraphicEvents::init_unit_events
/// @008e2520:833–841` fills exactly these when the piece's events are
/// built. The bit patterns are in `docs/COMBAT.md` §55.2.
///
/// **Piece 296, the Bomb Vessel** (item 1257): run486's packet holds its
/// seven entries — the `(4, 0, 0)` [`NODES`] reads, and for each of its
/// three `<RELEASEEVENT>`s (node 0, frame 3 of `CHAR_ATTACK1`, `2` and `3`)
/// the pivot node's and the release node's. `CHAR_ATTACK1` and `3` share
/// the pivot's `−29.576756`; `2`'s is `−29.555523`.
///
/// **Piece 137, the Horse Archer** (item 2997): run912e's packet (frame
/// 12753 of the battery's German lobby) holds its nine entries — the
/// `(4, 0, 0)` [`NODES`] reads and, for each of its four `<RELEASEEVENT>`s
/// (node 0; 833, 899, 1266 and 866 ms on `CHAR_ATTACK1`..`3` and
/// `CHAR_ATTACKWALK`: frames 18, 16, 18 and 18), the pivot node's and the
/// release node's. who=1's `1/60` threw from its own point here and the
/// Scout `0/0` died on 12835 in the original, a frame before this crate's
/// round arrived (`docs/AI.md` §244).
///
/// **Piece 139, the French Dragoon** (item 3359): run985b's packet
/// (`RON_STATE_FRAME=16762` of the battery's French lobby) holds its nine
/// entries — the `(4, 0, 0)` [`NODES`] reads, `(0.0, −1.48, 31.05)` as
/// piece 491's, and for each of its four `<RELEASEEVENT>`s (node 0; 866,
/// 833, 899 and 1266 ms on `CHAR_ATTACKWALK` and `CHAR_ATTACK1`..`3`:
/// frames 12, 12, 13 and 18) the pivot node's and the release node's. The
/// `<RESTRICTION name="Dragoon" node="4">` makes it a pivot piece, so its
/// rounds take this branch, the release turned by the turret as well as the
/// figure: `1/39`'s two `CHAR_ATTACK2` rounds, (−61, −18) on run982b's
/// 14677 and (13, −66) on run982d's 16764, are one entry at two turrets,
/// and no facing-only node fits both (`docs/AI.md` §280).
///
/// SEAM: every other pivot piece that releases (the camel archers but
/// pieces 141 and 143, the Mameluke's line, the machine-gun ships). A packet
/// from a game that fields the type reads each row in a minute
/// (`tools/recomp/get_position.py … entries <piece>`).
/// Piece 280's [`Release::float_lands`], one list per `(anim, node)` row
/// (item 3849, run1484's packet).
const FLOAT_LANDS_280_A10_N0: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A10_N1: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A10_N2: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A10_N3: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A11_N0: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A11_N1: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A11_N2: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A11_N3: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A12_N0: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A12_N1: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A12_N2: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_280_A12_N3: &[(i32, i32, i32, i32)] = &[];
/// Piece 300's [`Release::float_lands`], one list per `(anim, node)` row
/// (item 3849, run1484's packet).
const FLOAT_LANDS_300_A11_N0: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A11_N1: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A11_N2: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A11_N3: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A11_N8: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A11_N9: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A12_N0: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A12_N1: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A12_N2: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A12_N3: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A12_N8: &[(i32, i32, i32, i32)] = &[];
const FLOAT_LANDS_300_A12_N9: &[(i32, i32, i32, i32)] = &[];
const RELEASES: &[(i32, i8, u32, Release)] = &[
    // **The Light Tank, piece 189** (unit type 239; item 3529, `docs/AI.md`
    // §303): run1169's packet holds the piece's seven entries — the turret
    // `(4, 0, 0)` and, for each of its three `<RELEASEEVENT>`s (node 8, the
    // turret's: frame 4 of `CHAR_ATTACK1`, 21 of `CHAR_ATTACK2`, 14 of
    // `CHAR_ATTACK3`), the turret's and the muzzle's. The muzzle sits 27
    // model units ahead of the turret, so a round leaves about 127 ahead of
    // the hull's square: who=1's `1/27` round of 4406 flies 5 frames and
    // bursts on the human's Citizen `0/3` on 4410, where this crate's left
    // the hull, flew 6, and came down behind the fleeing citizen on 4411.
    (
        189,
        crate::units::anim::ATTACK1,
        4,
        Release {
            piece: 189,
            node: 8,
            pivot: [18_803, -2_089_893, 37_354_836],
            at: [135_452, -27_332_077, 3_642_757],
            float_lands: FLOAT_LANDS_189_A11,
        },
    ),
    (
        189,
        crate::units::anim::ATTACK2,
        21,
        Release {
            piece: 189,
            node: 8,
            pivot: [3_905, -3_585_954, 37_319_122],
            at: [113_480, -27_390_808, 3_339_424],
            float_lands: FLOAT_LANDS_189_A12,
        },
    ),
    (
        189,
        crate::units::anim::ATTACK3,
        14,
        Release {
            piece: 189,
            node: 8,
            pivot: [32_120, -466_466, 37_334_103],
            at: [155_086, -27_273_699, 3_934_948],
            float_lands: FLOAT_LANDS_189_A13,
        },
    ),
    // **The Main Battle Tank, piece 201** (`HEAVYTANK`, unit type 251; item
    // 3810, `docs/GOLDEN.md` §152.6): run1472's `RON_STATE_FRAME=707` packet
    // on the third melee's lobby holds the piece's seven entries, `RData
    // +0x88` 0.8 — the turret `(4, 0, 0)` (0, 0, 27.11) and, for each of its
    // three `<RELEASEEVENT>`s (node 0; frame 11 of `CHAR_ATTACK1`, 23 of `2`,
    // 27 of `3`), the turret's and the muzzle's, which differ by anim. The
    // muzzle sits some 90 model units ahead of the turret: who=1's `1/21`
    // round of 707 on the Partisan `0/17` flew 5 frames from the hull's
    // square where the original's left the muzzle, and its blow came a tick
    // after `1/14`'s search of 712.
    (
        201,
        crate::units::anim::ATTACK1,
        11,
        Release {
            piece: 201,
            node: 0,
            pivot: [0, 0, 27_109_999],
            at: [140_000, -90_569_908, 10_065_222],
            float_lands: FLOAT_LANDS_201_A11,
        },
    ),
    (
        201,
        crate::units::anim::ATTACK2,
        23,
        Release {
            piece: 201,
            node: 0,
            pivot: [0, 0, 27_109_999],
            at: [140_000, -91_192_764, 10_107_065],
            float_lands: FLOAT_LANDS_201_A12,
        },
    ),
    (
        201,
        crate::units::anim::ATTACK3,
        27,
        Release {
            piece: 201,
            node: 0,
            pivot: [0, 139_500, 27_138_798],
            at: [140_000, -80_396_957, 9_363_552],
            float_lands: FLOAT_LANDS_201_A13,
        },
    ),
    // **The British and Russian Light Tank `MARK4`, piece 190** (unit type
    // 240; item 3683, `docs/AI.md` §347): run1355's packet holds the piece's
    // seven entries — the turret `(4, 0, 0)` and, for each of its three
    // `<RELEASEEVENT>`s (node 0, the muzzle: frame 4 of `CHAR_ATTACK1`, 21
    // of `CHAR_ATTACK2`, 1 of `CHAR_ATTACK3`), the turret's and the
    // muzzle's, the same bits on all three. The muzzle sits 28 model units
    // ahead of the turret and 30 below it. who=1's `1/28` round of 2934
    // leaves (9218, 31078) at 757 on the British Industrial pair, where
    // this crate's left the hull's square (9432, 31128) at 619.
    (
        190,
        crate::units::anim::ATTACK1,
        4,
        Release {
            piece: 190,
            node: 0,
            pivot: [-80_000, -11_530_001, 31_199_999],
            at: [-71_684, -39_843_819, 783_760],
            float_lands: FLOAT_LANDS_190,
        },
    ),
    (
        190,
        crate::units::anim::ATTACK2,
        21,
        Release {
            piece: 190,
            node: 0,
            pivot: [-80_000, -11_530_001, 31_199_999],
            at: [-71_684, -39_843_819, 783_760],
            float_lands: FLOAT_LANDS_190,
        },
    ),
    (
        190,
        crate::units::anim::ATTACK3,
        1,
        Release {
            piece: 190,
            node: 0,
            pivot: [-80_000, -11_530_001, 31_199_999],
            at: [-71_684, -39_843_819, 783_760],
            float_lands: FLOAT_LANDS_190,
        },
    ),
    // **The Destroyer, piece 278** (unit type 328; item 3525, `docs/AI.md`
    // §303): run1169's packet (logger frame 3769 of the Japanese Industrial
    // lobby) holds the piece's thirty-nine entries — its three pivots at
    // `(anim 0, time 0)` and, on `CHAR_ATTACKWALK`, `CHAR_ATTACK1` and
    // `CHAR_ATTACK2` alike (the bits are the same on all three), a release
    // from each turret: node 0 on frame 10 from pivot 4, node 1 on 12 from
    // pivot 5, node 2 on 15 from pivot 6 — the Cruiser's release nodes, bit
    // for bit, on other pivots. who=1's `1/66` round of 3800, the stern
    // turret's, flies 8 frames and comes down on 3808; this crate's left
    // from the hull's own square, flew 7 and struck the human's Citizen
    // `0/5` on 3807.
    (
        278,
        crate::units::anim::ATTACKWALK,
        10,
        Release {
            piece: 278,
            node: 0,
            pivot: [-379_492, -56_969_997, 13_564_682],
            at: [2_844_275, -15_566_135, 5_294_392],
            float_lands: FLOAT_LANDS_278_N0,
        },
    ),
    (
        278,
        crate::units::anim::ATTACKWALK,
        12,
        Release {
            piece: 278,
            node: 1,
            pivot: [-713_179, -36_950_001, 20_987_864],
            at: [2_388_635, -13_228_992, 4_566_566],
            float_lands: FLOAT_LANDS_278_N1,
        },
    ),
    (
        278,
        crate::units::anim::ATTACKWALK,
        15,
        Release {
            piece: 278,
            node: 2,
            pivot: [-290_015, 107_739_998, 9_675_645],
            at: [2_406_699, -13_229_012, 4_556_933],
            float_lands: FLOAT_LANDS_278_N2,
        },
    ),
    (
        278,
        crate::units::anim::ATTACK1,
        10,
        Release {
            piece: 278,
            node: 0,
            pivot: [-379_492, -56_969_997, 13_564_682],
            at: [2_844_275, -15_566_135, 5_294_392],
            float_lands: FLOAT_LANDS_278_N0,
        },
    ),
    (
        278,
        crate::units::anim::ATTACK1,
        12,
        Release {
            piece: 278,
            node: 1,
            pivot: [-713_179, -36_950_001, 20_987_864],
            at: [2_388_635, -13_228_992, 4_566_566],
            float_lands: FLOAT_LANDS_278_N1,
        },
    ),
    (
        278,
        crate::units::anim::ATTACK1,
        15,
        Release {
            piece: 278,
            node: 2,
            pivot: [-290_015, 107_739_998, 9_675_645],
            at: [2_406_699, -13_229_012, 4_556_933],
            float_lands: FLOAT_LANDS_278_N2,
        },
    ),
    (
        278,
        crate::units::anim::ATTACK2,
        10,
        Release {
            piece: 278,
            node: 0,
            pivot: [-379_492, -56_969_997, 13_564_682],
            at: [2_844_275, -15_566_135, 5_294_392],
            float_lands: FLOAT_LANDS_278_N0,
        },
    ),
    (
        278,
        crate::units::anim::ATTACK2,
        12,
        Release {
            piece: 278,
            node: 1,
            pivot: [-713_179, -36_950_001, 20_987_864],
            at: [2_388_635, -13_228_992, 4_566_566],
            float_lands: FLOAT_LANDS_278_N1,
        },
    ),
    (
        278,
        crate::units::anim::ATTACK2,
        15,
        Release {
            piece: 278,
            node: 2,
            pivot: [-290_015, 107_739_998, 9_675_645],
            at: [2_406_699, -13_229_012, 4_556_933],
            float_lands: FLOAT_LANDS_278_N2,
        },
    ),
    // **The Battleship, piece 299** (unit type 349; item 3736, `docs/AI.md`
    // §365.1): run1165's packet (logger frame 3719 of the Russian Modern
    // lobby) holds the piece's twenty-seven entries — its three pivots at
    // `(anim 0, time 0)` and, on `CHAR_ATTACK1` and `CHAR_ATTACK2` (whose
    // bits differ), a release from each turret: node 0 on frame 1 from
    // pivot 4, node 2 on 7 from pivot 6, node 1 on 12 from pivot 5 (the
    // `<RELEASEEVENT>`s' 133, 533 and 833 ms). The Spanish Modern pair's
    // Battleship `1/45` loosed at the Citizen `0/13` on 3693 from
    // `(2629, 11443, 106)` in the original, where this crate's left the
    // hull's own square `(2808, 11832, 0)` and flew 14 frames against 13.
    (
        299,
        crate::units::anim::ATTACK1,
        1,
        Release {
            piece: 299,
            node: 0,
            pivot: [-184_538, -95_339_996, 13_848_769],
            at: [-142_720, -34_599_060, 10_709_489],
            float_lands: FLOAT_LANDS_299_A11_N0,
        },
    ),
    (
        299,
        crate::units::anim::ATTACK1,
        7,
        Release {
            piece: 299,
            node: 2,
            pivot: [-442_118, 117_489_998, 13_822_918],
            at: [-337_668, -33_652_985, 10_556_601],
            float_lands: FLOAT_LANDS_299_A11_N2,
        },
    ),
    (
        299,
        crate::units::anim::ATTACK1,
        12,
        Release {
            piece: 299,
            node: 1,
            pivot: [-1_229_429, -49_659_996, 22_756_771],
            at: [-578_620, -32_191_475, 10_710_815],
            float_lands: FLOAT_LANDS_299_A11_N1,
        },
    ),
    (
        299,
        crate::units::anim::ATTACK2,
        1,
        Release {
            piece: 299,
            node: 0,
            pivot: [206_375, -95_339_996, 13_848_460],
            at: [159_609, -34_599_060, 10_709_254],
            float_lands: FLOAT_LANDS_299_A12_N0,
        },
    ),
    (
        299,
        crate::units::anim::ATTACK2,
        7,
        Release {
            piece: 299,
            node: 2,
            pivot: [519_764, 117_489_998, 13_820_213],
            at: [396_971, -33_652_985, 10_554_571],
            float_lands: FLOAT_LANDS_299_A12_N2,
        },
    ),
    (
        299,
        crate::units::anim::ATTACK2,
        12,
        Release {
            piece: 299,
            node: 1,
            pivot: [1_092_826, -49_659_996, 22_763_744],
            at: [514_329, -32_191_490, 10_714_073],
            float_lands: FLOAT_LANDS_299_A12_N1,
        },
    ),
    // **The Tank `PANZER`, piece 200** (unit type 250; item 3737,
    // `docs/AI.md` §365.2): run1416's packet (logger frame 4016 of the
    // Spanish Modern East Indies lobby) holds the piece's seven entries — the
    // turret `(4, 0, 0)` and, for each of its three `<RELEASEEVENT>`s (node 0:
    // frame 4 of `CHAR_ATTACK1`, 21 of `CHAR_ATTACK2`, 1 of `CHAR_ATTACK3`),
    // the turret's and the muzzle's. The muzzle sits 68 model units ahead
    // of a turret 36 up, so a round leaves about 240 ahead of the hull at
    // height 144: who=1's `1/35` round of 4017 at the human's Citizen
    // `0/13` left (7154, 9226, 312) in the original and the hull's square
    // (7032, 9432, 168) here, flew 7 frames against 6, and the citizen
    // outlived the original's on 4021.
    (
        200,
        crate::units::anim::ATTACK1,
        4,
        Release {
            piece: 200,
            node: 0,
            pivot: [0, 5_579_999, 35_680_000],
            at: [-453_512, -68_010_040, 1_908_176],
            float_lands: FLOAT_LANDS_200_A11,
        },
    ),
    (
        200,
        crate::units::anim::ATTACK2,
        21,
        Release {
            piece: 200,
            node: 0,
            pivot: [0, 5_640_605, 35_680_000],
            at: [-421_595, -60_046_501, 1_908_176],
            float_lands: FLOAT_LANDS_200_A12,
        },
    ),
    (
        200,
        crate::units::anim::ATTACK3,
        1,
        Release {
            piece: 200,
            node: 0,
            pivot: [0, 5_579_999, 35_680_000],
            at: [-454_502, -68_257_225, 1_908_176],
            float_lands: FLOAT_LANDS_200_A13,
        },
    ),
    // **The Cruiser, piece 279** (unit type 329; item 3519, `docs/AI.md`
    // §304): run1165's packet (logger frame 3719 of the Russian Modern
    // lobby) holds the piece's thirty-nine entries — its three pivots at
    // `(anim 0, time 0)` and, on `CHAR_ATTACKWALK`, `CHAR_ATTACK1` and
    // `CHAR_ATTACK2` alike (the bits are the same on all three), a release
    // from each turret: node 0 on frame 10 from pivot 4, node 1 on 12 from
    // pivot 5, node 2 on 15 from pivot 6. The Cruiser `1/25`'s rounds on
    // 3716 and 3718 leave from `(2999, 7827, 134)` and `(2956, 7931, 166)`,
    // where this crate's left from the hull's own square and flew a frame
    // longer.
    (
        279,
        crate::units::anim::ATTACKWALK,
        10,
        Release {
            piece: 279,
            node: 0,
            pivot: [-725_785, -91_649_994, 25_939_827],
            at: [2_844_275, -15_566_139, 5_294_392],
            float_lands: FLOAT_LANDS_279_N0,
        },
    ),
    (
        279,
        crate::units::anim::ATTACKWALK,
        12,
        Release {
            piece: 279,
            node: 1,
            pivot: [-1_157_794, -65_479_996, 34_070_297],
            at: [2_388_635, -13_228_996, 4_566_563],
            float_lands: FLOAT_LANDS_279_N1,
        },
    ),
    (
        279,
        crate::units::anim::ATTACKWALK,
        15,
        Release {
            piece: 279,
            node: 2,
            pivot: [-763_241, 105_279_999, 25_458_538],
            at: [2_406_699, -13_229_012, 4_556_934],
            float_lands: FLOAT_LANDS_279_N2,
        },
    ),
    (
        279,
        crate::units::anim::ATTACK1,
        10,
        Release {
            piece: 279,
            node: 0,
            pivot: [-725_785, -91_649_994, 25_939_827],
            at: [2_844_275, -15_566_139, 5_294_392],
            float_lands: FLOAT_LANDS_279_N0,
        },
    ),
    (
        279,
        crate::units::anim::ATTACK1,
        12,
        Release {
            piece: 279,
            node: 1,
            pivot: [-1_157_794, -65_479_996, 34_070_297],
            at: [2_388_635, -13_228_996, 4_566_563],
            float_lands: FLOAT_LANDS_279_N1,
        },
    ),
    (
        279,
        crate::units::anim::ATTACK1,
        15,
        Release {
            piece: 279,
            node: 2,
            pivot: [-763_241, 105_279_999, 25_458_538],
            at: [2_406_699, -13_229_012, 4_556_934],
            float_lands: FLOAT_LANDS_279_N2,
        },
    ),
    (
        279,
        crate::units::anim::ATTACK2,
        10,
        Release {
            piece: 279,
            node: 0,
            pivot: [-725_785, -91_649_994, 25_939_827],
            at: [2_844_275, -15_566_139, 5_294_392],
            float_lands: FLOAT_LANDS_279_N0,
        },
    ),
    (
        279,
        crate::units::anim::ATTACK2,
        12,
        Release {
            piece: 279,
            node: 1,
            pivot: [-1_157_794, -65_479_996, 34_070_297],
            at: [2_388_635, -13_228_996, 4_566_563],
            float_lands: FLOAT_LANDS_279_N1,
        },
    ),
    (
        279,
        crate::units::anim::ATTACK2,
        15,
        Release {
            piece: 279,
            node: 2,
            pivot: [-763_241, 105_279_999, 25_458_538],
            at: [2_406_699, -13_229_012, 4_556_934],
            float_lands: FLOAT_LANDS_279_N2,
        },
    ),
    (
        139,
        crate::units::anim::ATTACKWALK,
        12,
        Release {
            piece: 139,
            node: 0,
            pivot: [58_111, -1_380_116, 31_034_378],
            at: [-12_961_245, -27_570_284, 11_694_054],
            float_lands: &[],
        },
    ),
    (
        139,
        crate::units::anim::ATTACK1,
        12,
        Release {
            piece: 139,
            node: 0,
            pivot: [4_308_261, -1_638_021, 31_049_999],
            at: [-3_027_300, -29_267_675, 15_569_523],
            float_lands: &[],
        },
    ),
    (
        139,
        crate::units::anim::ATTACK2,
        13,
        Release {
            piece: 139,
            node: 0,
            pivot: [0, 6_483_696, 34_750_904],
            at: [-8_452_114, -17_947_832, 20_006_462],
            float_lands: &[],
        },
    ),
    (
        139,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 139,
            node: 0,
            pivot: [0, -1_480_000, 31_049_999],
            at: [-4_867_605, -28_791_185, 12_283_539],
            float_lands: &[],
        },
    ),
    // **The Camel Raider, piece 143** (unit type 193, `CAMELRAIDER`, the
    // Nubian camel archer; item 3770, `docs/GOLDEN.md` §149.6): run1440's
    // packet (logger frame 809 of the first melee's lobby) holds the piece's
    // nine entries — the `(4, 0, 0)` [`NODES`] reads, `RData +0x88` 0.9, and
    // for each of its four `<RELEASEEVENT>`s (node 0; frame 12 of
    // `CHAR_ATTACKWALK` and `CHAR_ATTACK1`, 13 of `CHAR_ATTACK2`, 18 of
    // `CHAR_ATTACK3`) the pivot node's and the release node's. Without them
    // the human's `0/17` loosed every round from its own square (17400,
    // 15672) at 477, where run1440's window prints (17282, 15746, 742) and
    // (17274, 15736, 758) — `CHAR_ATTACK3` and `CHAR_ATTACK2` at the dump's
    // own turret step 4 — and its round of 859 struck the Inca Bombard `2/23`
    // on 864, a frame after the original's.
    (
        143,
        crate::units::anim::ATTACKWALK,
        12,
        Release {
            piece: 143,
            node: 0,
            pivot: [1_680, -4_242_180, 47_731_030],
            at: [-12_995_936, -28_961_021, 7_432_457],
            float_lands: FLOAT_LANDS_143_A10,
        },
    ),
    (
        143,
        crate::units::anim::ATTACK1,
        12,
        Release {
            piece: 143,
            node: 0,
            pivot: [6_233, -3_512_383, 49_157_646],
            at: [-7_473_044, -25_987_869, 3_148_621],
            float_lands: FLOAT_LANDS_143_A11,
        },
    ),
    (
        143,
        crate::units::anim::ATTACK2,
        13,
        Release {
            piece: 143,
            node: 0,
            pivot: [2_467_945, -4_790_035, 48_574_203],
            at: [-10_437_510, -27_476_112, 16_682_030],
            float_lands: FLOAT_LANDS_143_A12,
        },
    ),
    (
        143,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 143,
            node: 0,
            pivot: [20, -3_322_791, 49_239_670],
            at: [-4_862_028, -28_742_996, 12_248_196],
            float_lands: FLOAT_LANDS_143_A13,
        },
    ),
    // **The Camel Archer, piece 141** (unit type 191, `CAMELRANGE2`, the
    // Nubians' Classical camel archer; item 3785, `docs/GOLDEN.md` §150.1):
    // run1457's packet (logger frame 700 of the second melee's lobby) holds
    // the piece's nine entries — the `(4, 0, 0)` [`NODES`] reads, `RData
    // +0x88` 0.9, and for each of its four `<RELEASEEVENT>`s (node 0; frame
    // 13 of `CHAR_ATTACKWALK`, `CHAR_ATTACK1` and `CHAR_ATTACK2`, 10 of
    // `CHAR_ATTACK3`) the pivot node's and the release node's. Without them
    // the human's `0/17` loosed its round of 699 at the Egyptian `2/22` from
    // its own square (15864, 15672) at 566, where run1450's block 701 prints
    // (15828, 15703, 835).
    (
        141,
        crate::units::anim::ATTACKWALK,
        13,
        Release {
            piece: 141,
            node: 0,
            pivot: [903_646, -1_816_183, 48_539_608],
            at: [-268_843, -13_453_966, 11_554_844],
            float_lands: FLOAT_LANDS_141_A10,
        },
    ),
    (
        141,
        crate::units::anim::ATTACK1,
        13,
        Release {
            piece: 141,
            node: 0,
            pivot: [908_200, -122_588, 48_459_412],
            at: [-48_422, -11_193_214, 14_021_603],
            float_lands: FLOAT_LANDS_141_A11,
        },
    ),
    (
        141,
        crate::units::anim::ATTACK2,
        13,
        Release {
            piece: 141,
            node: 0,
            pivot: [2_867_742, -1_472_081, 47_870_110],
            at: [-1_691_635, -11_148_353, 13_973_820],
            float_lands: FLOAT_LANDS_141_A12,
        },
    ),
    (
        141,
        crate::units::anim::ATTACK3,
        10,
        Release {
            piece: 141,
            node: 0,
            pivot: [908_200, -122_588, 48_459_412],
            at: [-2_347_111, -10_713_058, 14_215_946],
            float_lands: FLOAT_LANDS_141_A13,
        },
    ),
    // **The Tank** (item 3244): run938b's packet (logger frame 630, ch100's
    // lobby) holds piece 194's seven entries — the `(4, 0, 0)` pivot, and for
    // each of its three `<RELEASEEVENT>`s (node 0; `CHAR_ATTACK1` frame 4,
    // `CHAR_ATTACK2` 21, `CHAR_ATTACK3` 1) the pivot node's and the release
    // node's. `float_lands` is empty: the unicorn sweep of the cells has not
    // been run for this piece (`tools/recomp/get_position.py … pivot`).
    (
        194,
        crate::units::anim::ATTACK1,
        4,
        Release {
            piece: 194,
            node: 0,
            pivot: [0, -960_000, 31_920_000],
            at: [-350_481, -42_303_066, 1_908_175],
            float_lands: &[],
        },
    ),
    (
        194,
        crate::units::anim::ATTACK2,
        21,
        Release {
            piece: 194,
            node: 0,
            pivot: [0, -899_394, 31_920_000],
            at: [-318_564, -34_339_527, 1_908_175],
            float_lands: &[],
        },
    ),
    (
        194,
        crate::units::anim::ATTACK3,
        1,
        Release {
            piece: 194,
            node: 0,
            pivot: [0, -960_000, 31_920_000],
            at: [-351_471, -42_550_259, 1_908_175],
            float_lands: &[],
        },
    ),
    // **The Armored Car, piece 166** (unit type 216, the Chinese and the
    // Asian set's `armcar.bh3`; item 3511, `docs/AI.md` §305): run1155c's
    // packet (logger frame 3003 of the Chinese Industrial lobby) holds the
    // piece's seventy-three entries — the `(4, 0, 0)` pivot and, for each
    // of its nine `<RELEASEEVENT>`s on node 0 (frames 3, 9, 12, 17, 20, 22,
    // 24 and 29 twice, `starttime / 67`) on each of four anims, the pivot
    // node's and the release node's; the two events on 29 carry the same
    // bits. The car `1/23`'s round on 3001 leaves from `(9569, 31152,
    // 750)`, not its square `(9617, 31210)`, and flies 6 frames, not 7.
    (
        166,
        crate::units::anim::ATTACKWALK,
        3,
        Release {
            piece: 166,
            node: 0,
            pivot: [-613_426, 11_139_999, 28_061_401],
            at: [-175, -19_048_059, 5_388],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACKWALK,
        9,
        Release {
            piece: 166,
            node: 0,
            pivot: [-1_095_403, 11_139_999, 28_042_585],
            at: [14_944, -19_058_464, -299_030],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACKWALK,
        12,
        Release {
            piece: 166,
            node: 0,
            pivot: [-788_691, 11_139_999, 28_055_786],
            at: [8_712, -19_057_268, -242_453],
            float_lands: &[
                (18, 21, -49, 20),
                (60, 91, 0, -121),
                (150, 91, 121, 0),
                (240, 91, 0, 121),
                (330, 91, -121, 0),
            ],
        },
    ),
    (
        166,
        crate::units::anim::ATTACKWALK,
        17,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_059_999, -469_999],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACKWALK,
        20,
        Release {
            piece: 166,
            node: 0,
            pivot: [876_323, 11_139_999, 28_052_454],
            at: [733, -19_047_443, 17_534],
            float_lands: &[
                (89, 198, 155, 32),
                (132, 197, 92, 129),
                (149, 90, 112, 0),
                (215, 18, 42, -15),
                (222, 197, -129, 92),
                (312, 197, -92, -129),
            ],
        },
    ),
    (
        166,
        crate::units::anim::ATTACKWALK,
        22,
        Release {
            piece: 166,
            node: 0,
            pivot: [1_095_403, 11_139_999, 28_042_585],
            at: [-10_835, -19_056_625, -216_995],
            float_lands: &[(38, 248, 137, -15), (308, 248, -15, -137)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACKWALK,
        24,
        Release {
            piece: 166,
            node: 0,
            pivot: [1_051_587, 11_139_999, 28_044_735],
            at: [-445, -19_048_820, -9_926],
            float_lands: &[(281, 222, -117, -97)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACKWALK,
        29,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_058_563, -304_922],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK1,
        3,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_048_038, 6_250],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK1,
        9,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_058_460, -299_170],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK1,
        12,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_057_262, -242_228],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK1,
        17,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_059_999, -469_999],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK1,
        20,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_047_424, 18_330],
            float_lands: &[(89, 260, 78, 97)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK1,
        22,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_056_622, -216_919],
            float_lands: &[
                (24, 243, 124, -59),
                (66, 117, 59, -124),
                (66, 291, 59, 76),
                (114, 69, 59, -76),
                (114, 243, 59, 124),
                (156, 117, 124, 59),
                (156, 291, -76, 59),
                (204, 69, 76, 59),
                (204, 243, -124, 59),
                (230, 105, -2, 128),
                (246, 117, -59, 124),
                (246, 291, -59, -76),
                (294, 69, -59, 76),
                (294, 243, -59, -124),
                (336, 117, -124, -59),
                (336, 291, 76, -59),
            ],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK1,
        24,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_048_805, -9_279],
            float_lands: &[(134, 191, 100, 123)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK1,
        29,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_058_563, -304_922],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK2,
        3,
        Release {
            piece: 166,
            node: 0,
            pivot: [-437_479, 11_183_747, 28_008_846],
            at: [1_895_208, -18_952_995, 80_376],
            float_lands: &[(93, 38, -23, -57), (273, 38, 23, 57)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK2,
        9,
        Release {
            piece: 166,
            node: 0,
            pivot: [-700_925, 11_139_999, 28_058_773],
            at: [3_150_493, -18_796_585, -198_864],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK2,
        12,
        Release {
            piece: 166,
            node: 0,
            pivot: [-481_925, 11_139_999, 28_064_692],
            at: [2_204_829, -18_929_413, -194_298],
            float_lands: &[(52, 31, -53, -14)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK2,
        17,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-685_474, -19_047_649, -469_999],
            float_lands: &[
                (7, 198, 52, -148),
                (83, 212, 148, 38),
                (97, 198, 148, 52),
                (168, 87, 110, 35),
                (173, 212, -38, 148),
                (187, 198, -52, 148),
                (192, 63, 85, 35),
                (263, 212, -148, -38),
                (277, 198, -148, -52),
                (353, 212, 38, -148),
            ],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK2,
        20,
        Release {
            piece: 166,
            node: 0,
            pivot: [569_435, 11_139_999, 28_062_588],
            at: [-2_461_267, -18_886_921, 80_650],
            float_lands: &[(131, 31, 23, -58), (150, 51, 74, -36), (311, 31, -23, 58)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK2,
        22,
        Release {
            piece: 166,
            node: 0,
            pivot: [744_776, 11_139_999, 28_057_323],
            at: [-3_223_725, -18_781_982, -107_924],
            float_lands: &[(188, 149, 10, 155), (278, 149, -155, 10)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK2,
        24,
        Release {
            piece: 166,
            node: 0,
            pivot: [700_016, 11_183_750, 28_002_007],
            at: [-3_139_843, -18_786_903, 127_499],
            float_lands: &[(268, 83, -60, 98)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK2,
        29,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-464_580, -19_052_887, -305_227],
            float_lands: &[(71, 60, -18, -87), (161, 60, 87, -18), (341, 60, -87, 18)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK3,
        3,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-988_832, -19_022_318, 5_621],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK3,
        9,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-3_447_998, -18_743_118, -299_459],
            float_lands: &[(188, 48, 84, 18)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK3,
        12,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-3_447_889, -18_741_932, -242_613],
            float_lands: &[(137, 107, 137, 16), (223, 21, 56, 16)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK3,
        17,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-3_451_576, -18_744_328, -469_999],
            float_lands: &[(101, 56, 35, -87), (198, 121, 33, 142)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK3,
        20,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-3_446_998, -18_732_191, 17_504],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK3,
        22,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-3_447_831, -18_741_299, -217_348],
            float_lands: &[(128, 32, 29, -62), (232, 288, -63, -62)],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK3,
        24,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [-3_147_239, -18_786_549, -9_783],
            float_lands: &[],
        },
    ),
    (
        166,
        crate::units::anim::ATTACK3,
        29,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_058_563, -304_922],
            float_lands: &[],
        },
    ),
    // **The Armored Cavalry, piece 170** (unit type 220, `ARMOREDCAVALRY`,
    // `armoredcavalry.bh3`; item 3756, `docs/AI.md` §372.2): its
    // `<RESTRICTION name="ARMOREDCAVALRY" node="4">` sits at the foot of
    // `unit_graphics.xml`, not in its `<UNIT>` block. run854's packet (and
    // run865's, run882b's, run883b's, row for row) holds the piece's
    // seventeen entries, `RData +0x88` 0.7: the `(4, 0, 0)` pivot and, for
    // each of its eight `<RELEASEEVENT>`s (node 0, frames 4 and 19 of
    // `CHAR_ATTACKWALK` and `CHAR_ATTACK1..3`), the pivot node's and the
    // release node's. Without them who=1's `1/16` loosed its round of 3587
    // on the human's `0/13` from its own square (6418, 32805, 171) where the
    // original's left the muzzle at (6307, 32645, 309).
    (
        170,
        crate::units::anim::ATTACKWALK,
        4,
        Release {
            piece: 170,
            node: 0,
            pivot: [-3_658_126, 14_200_000, 41_171_295],
            at: [-77_994, -46_929_089, 601_677],
            float_lands: FLOAT_LANDS_170_A10_T4,
        },
    ),
    (
        170,
        crate::units::anim::ATTACKWALK,
        19,
        Release {
            piece: 170,
            node: 0,
            pivot: [-5_545_757, 14_200_000, 40_699_749],
            at: [-105_554, -46_874_546, 597_458],
            float_lands: FLOAT_LANDS_170_A10_T19,
        },
    ),
    (
        170,
        crate::units::anim::ATTACK1,
        4,
        Release {
            piece: 170,
            node: 0,
            pivot: [-4_480_000, 14_200_000, 41_220_001],
            at: [-90_000, -46_929_089, 599_998],
            float_lands: FLOAT_LANDS_170_A11_T4,
        },
    ),
    (
        170,
        crate::units::anim::ATTACK1,
        19,
        Release {
            piece: 170,
            node: 0,
            pivot: [-4_480_000, 14_200_000, 41_200_001],
            at: [-90_000, -46_874_546, 599_998],
            float_lands: FLOAT_LANDS_170_A11_T19,
        },
    ),
    (
        170,
        crate::units::anim::ATTACK2,
        4,
        Release {
            piece: 170,
            node: 0,
            pivot: [-4_480_000, 14_200_000, 41_220_001],
            at: [9_007_834, -46_055_683, 599_998],
            float_lands: FLOAT_LANDS_170_A12_T4,
        },
    ),
    (
        170,
        crate::units::anim::ATTACK2,
        19,
        Release {
            piece: 170,
            node: 0,
            pivot: [-4_480_000, 14_200_000, 41_200_001],
            at: [18_548_830, -43_047_009, 599_998],
            float_lands: FLOAT_LANDS_170_A12_T19,
        },
    ),
    (
        170,
        crate::units::anim::ATTACK3,
        4,
        Release {
            piece: 170,
            node: 0,
            pivot: [-4_480_000, 14_200_000, 41_220_001],
            at: [-90_000, -46_929_089, 599_998],
            float_lands: FLOAT_LANDS_170_A13_T4,
        },
    ),
    (
        170,
        crate::units::anim::ATTACK3,
        19,
        Release {
            piece: 170,
            node: 0,
            pivot: [-4_480_000, 14_200_000, 41_200_001],
            at: [-90_000, -46_874_546, 599_998],
            float_lands: FLOAT_LANDS_170_A13_T19,
        },
    ),
    // **The Main Battle Tank `ABRAM`, piece 204** (unit type 254,
    // `HvyTank.bh3`; item 3758, `docs/AI.md` §372.3): its `<RESTRICTION
    // name="ABRAM" node="4">` sits at the foot of `unit_graphics.xml`.
    // run1437's packet holds the piece's fifteen entries, `RData +0x88`
    // 0.75: the `(4, 0, 0)` pivot and, for each of its seven
    // `<RELEASEEVENT>`s, the pivot node's and the release node's. The three
    // gun rounds (`BazookaRocket`, node 0: frame 11 of `CHAR_ATTACK1`, 23
    // of `CHAR_ATTACK2`, 27 of `CHAR_ATTACK3`) and the four node-1
    // `NoDamage Tracer`s (frames 1 and 7 of `CHAR_ATTACK2` and
    // `CHAR_ATTACK3`; item 3760, `docs/AI.md` §372.4) are rowed here: a
    // tracer is a round the dump prints, harmless (`flags` 22), and it
    // holds a slot. Without the gun's rows who=1's `1/29` loosed
    // its round of 3895 on the human's `0/12` from its own square (4296,
    // 34056, 474) where the original's left the muzzle at (4153, 33763,
    // 605).
    (
        204,
        crate::units::anim::ATTACK1,
        11,
        Release {
            piece: 204,
            node: 0,
            pivot: [0, 0, 27_109_999],
            at: [140_000, -90_680_344, 9_331_987],
            float_lands: FLOAT_LANDS_204_A11_T11,
        },
    ),
    (
        204,
        crate::units::anim::ATTACK2,
        23,
        Release {
            piece: 204,
            node: 0,
            pivot: [0, 0, 27_109_999],
            at: [140_000, -91_304_268, 9_352_728],
            float_lands: FLOAT_LANDS_204_A12_T23,
        },
    ),
    (
        204,
        crate::units::anim::ATTACK3,
        27,
        Release {
            piece: 204,
            node: 0,
            pivot: [0, 139_500, 27_138_798],
            at: [140_000, -80_489_395, 8_974_987],
            float_lands: FLOAT_LANDS_204_A13_T27,
        },
    ),
    (
        204,
        crate::units::anim::ATTACK2,
        1,
        Release {
            piece: 204,
            node: 1,
            pivot: [0, 0, 27_109_999],
            at: [-4_200_000, -41_229_996, 36_380_001],
            float_lands: FLOAT_LANDS_204_A12_T1,
        },
    ),
    (
        204,
        crate::units::anim::ATTACK2,
        7,
        Release {
            piece: 204,
            node: 1,
            pivot: [0, 0, 27_109_999],
            at: [-4_200_000, -41_404_877, 35_217_476],
            float_lands: FLOAT_LANDS_204_A12_T7,
        },
    ),
    (
        204,
        crate::units::anim::ATTACK3,
        1,
        Release {
            piece: 204,
            node: 1,
            pivot: [0, 0, 27_109_999],
            at: [-4_200_000, -41_229_996, 36_380_001],
            float_lands: FLOAT_LANDS_204_A13_T1,
        },
    ),
    (
        204,
        crate::units::anim::ATTACK3,
        7,
        Release {
            piece: 204,
            node: 1,
            pivot: [0, 0, 27_109_999],
            at: [-6_611_136, -41_091_362, 35_218_819],
            float_lands: FLOAT_LANDS_204_A13_T7,
        },
    ),
    // **The Armored Scout Car, piece 168** (unit type 218; item 3518,
    // `docs/AI.md` §302): run1164's packet (logger frame 4528 of the
    // American Modern lobby) holds the piece's nine entries, the `(4, 0, 0)`
    // pivot and, for each of its four `<RELEASEEVENT>`s (node 0;
    // `CHAR_ATTACKWALK` and `CHAR_ATTACK1` frame 1, `CHAR_ATTACK2` 11,
    // `CHAR_ATTACK3` 12), the pivot node's and the release node's. The
    // car `1/80`'s round on 4526 leaves from `(9412, 31822, 727)`, 174 ahead
    // of the car's own square along the turret.
    (
        168,
        crate::units::anim::ATTACKWALK,
        1,
        Release {
            piece: 168,
            node: 0,
            pivot: [0, 1_790_000, 30_919_998],
            at: [0, -38_147_877, 3_000_000],
            float_lands: &[
                (91, 225, 129, 125),
                (269, 86, 7, 173),
                (271, 84, 7, 173),
                (271, 225, -129, -125),
            ],
        },
    ),
    (
        168,
        crate::units::anim::ATTACK1,
        1,
        Release {
            piece: 168,
            node: 0,
            pivot: [0, 3_297_407, 30_542_767],
            at: [0, -37_979_084, 4_673_923],
            float_lands: &[],
        },
    ),
    (
        168,
        crate::units::anim::ATTACK2,
        11,
        Release {
            piece: 168,
            node: 0,
            pivot: [0, 3_427_710, 30_542_767],
            at: [0, -35_851_234, 11_680_939],
            float_lands: &[
                (86, 339, -132, 67),
                (266, 339, 132, -67),
                (274, 21, 132, 67),
            ],
        },
    ),
    (
        168,
        crate::units::anim::ATTACK3,
        12,
        Release {
            piece: 168,
            node: 0,
            pivot: [2_242, 4_693_419, 30_337_618],
            at: [-7_660_163, -33_022_854, 5_185_888],
            float_lands: &[
                (80, 136, 137, -105),
                (170, 136, 105, 137),
                (260, 136, -137, 105),
                (350, 136, -105, -137),
                (354, 111, -138, -93),
            ],
        },
    ),
    (
        137,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 137,
            node: 0,
            pivot: [-60_000, -2_676_834, 34_506_775],
            at: [479_090, -15_615_540, 16_108_955],
            float_lands: &[],
        },
    ),
    (
        137,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 137,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [1_223_768, -16_729_307, 14_252_346],
            float_lands: &[],
        },
    ),
    (
        137,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 137,
            node: 0,
            pivot: [-60_000, -919_060, 31_409_014],
            at: [1_516_343, -18_837_709, 12_125_257],
            float_lands: &[],
        },
    ),
    (
        137,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 137,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [3_239_868, -16_143_440, 15_273_258],
            float_lands: &[
                (9, 3, -2, 86),
                (99, 3, -86, -2),
                (189, 3, 2, -86),
                (279, 3, 86, 2),
            ],
        },
    ),
    // **Piece 138, the Cavalry Archer** (unit type 188, `CAVARCHERS`; item
    // 3633, `docs/AI.md` §330): run1293's packet (`RON_STATE_FRAME=3606` of
    // the Roman Medieval pair) holds its nine entries — the `(4, 0, 0)`
    // [`NODES`] read, `(−0.06, −1.48, 32.63)`, and for each of its four
    // `<RELEASEEVENT>`s (node 0; `CHAR_ATTACKWALK` and `CHAR_ATTACK1`..`3`
    // at frames 18, 18, 16 and 20) the pivot node's and the release node's.
    // The first three are piece 137's to the bit; `CHAR_ATTACK3`'s release
    // node is its own. The `<RESTRICTION name="HEAVYCAVARCHER" node="4">`
    // makes it a pivot piece, so its rounds left from `launch::MEASURED`'s
    // facing-only nodes until now: who=1's `1/79` loosed at the human's
    // Village `0/2000` on 5517 from (6131, 7631), 305 units short of its
    // landing at a speed of 100, and flew 3 frames where theirs flew 2.
    (
        138,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 138,
            node: 0,
            pivot: [-60_000, -2_676_834, 34_506_775],
            at: [479_090, -15_615_540, 16_108_955],
            float_lands: &[],
        },
    ),
    (
        138,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 138,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [1_223_768, -16_729_307, 14_252_346],
            float_lands: &[],
        },
    ),
    (
        138,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 138,
            node: 0,
            pivot: [-60_000, -919_060, 31_409_014],
            at: [1_516_343, -18_837_709, 12_125_257],
            float_lands: &[],
        },
    ),
    (
        138,
        crate::units::anim::ATTACK3,
        20,
        Release {
            piece: 138,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [4_182_650, -16_023_108, 15_027_695],
            float_lands: &[],
        },
    ),
    // **Piece 149, the Mongol Nomad** (unit type 199; item 3581, `docs/AI.md`
    // §311): run1224b's packet (`RON_STATE_FRAME=9745` on the Mongol Great
    // Sahara seed) holds its nine entries — the `(4, 0, 0)` [`NODES`] read,
    // `(0.0, −1.48, 32.63)`, and for each of its four `<RELEASEEVENT>`s
    // (node 0; `CHAR_ATTACKWALK` and `CHAR_ATTACK1`..`3` at frames 18, 18,
    // 16 and 19) the pivot node's and the release node's. The
    // `<RESTRICTION name="NOMADS" node="4">` makes it a pivot piece: who=1's
    // `1/31` loosed at the Citizen `0/11` on 9746 from (7467, 27925), its
    // rider turned 48° on the horse (`turret_angles[0]` 581828608), where
    // this crate loosed from the figure's own (7524, 27864), 17 frames
    // against 16.
    (
        149,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 149,
            node: 0,
            pivot: [0, -2_686_834, 34_506_775],
            at: [475_811, -15_610_859, 16_119_343],
            float_lands: &[],
        },
    ),
    (
        149,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 149,
            node: 0,
            pivot: [0, -1_480_000, 32_630_001],
            at: [998_139, -16_888_825, 12_294_712],
            float_lands: &[],
        },
    ),
    (
        149,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 149,
            node: 0,
            pivot: [0, -1_480_000, 32_630_001],
            at: [1_526_957, -17_046_072, 12_739_372],
            float_lands: &[],
        },
    ),
    (
        149,
        crate::units::anim::ATTACK3,
        19,
        Release {
            piece: 149,
            node: 0,
            pivot: [0, -1_480_000, 32_583_332],
            at: [-16_889_647, -1_001_732, 26_665_501],
            float_lands: &[],
        },
    ),
    // **Piece 150, the Steppe Nomad** (unit type 200, `ELITENOMADS`; item
    // 3714, `docs/AI.md` §362.3): run1391's packet (`RON_STATE_FRAME=4174`
    // on the Mongol Medieval East Indies seed) holds its nine entries — the
    // `(4, 0, 0)` [`NODES`] read, `(0.0, −3.38, 32.63)`, and for each of the
    // four `<RELEASEEVENT>`s piece 149 has (node 0; `CHAR_ATTACKWALK` and
    // `CHAR_ATTACK1`..`3` at frames 18, 18, 16 and 19) the pivot node's and
    // the release node's. Its own rider, not piece 149's rows: the pivot
    // sits 1.9 further back and every release node differs. who=1's `1/22`
    // loosed at the human's `0/13` on 4175 from (6319, 10344) at 271, where
    // this crate loosed from the figure's own (6312, 10440) at 38.
    (
        150,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 150,
            node: 0,
            pivot: [0, -4_586_834, 34_506_775],
            at: [224_176, -15_053_246, 19_689_476],
            float_lands: &[],
        },
    ),
    (
        150,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 150,
            node: 0,
            pivot: [0, -3_380_000, 32_630_001],
            at: [888_244, -16_728_748, 16_062_641],
            float_lands: &[],
        },
    ),
    (
        150,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 150,
            node: 0,
            pivot: [0, -3_380_000, 32_630_001],
            at: [1_242_316, -16_864_357, 16_494_263],
            float_lands: &[],
        },
    ),
    (
        150,
        crate::units::anim::ATTACK3,
        19,
        Release {
            piece: 150,
            node: 0,
            pivot: [0, -3_380_000, 32_583_332],
            at: [-18_684_111, 2_242_623, 26_876_545],
            float_lands: &[(307, 187, 82, -47)],
        },
    ),
    // **Piece 169, the German Armored Scout Car** (unit type 219,
    // `ARMOREDCAR2`; item 3738, `docs/AI.md` §366.1): run1417's packet
    // holds its thirty-seven entries — the `(4, 0, 0)` [`NODES`] read,
    // `(0.0, −0.07, 32.23)`, and for each of its sixteen `<RELEASEEVENT>`s
    // (node 0; `CHAR_ATTACKWALK` and `CHAR_ATTACK2` at frames 3, 9, 12,
    // 17, 20, 22, 24 and 29) the pivot node's and the release node's,
    // `RData +0x88` 1.1 (the table carries `(10, 29)` twice, the same
    // bits). who=1's `1/53` loosed on 4043 from its muzzle, about 118 out
    // and 170 up, where this crate loosed from the figure's own square.
    (
        169,
        crate::units::anim::ATTACKWALK,
        3,
        Release {
            piece: 169,
            node: 0,
            pivot: [-613_426, -70_001, 32_111_404],
            at: [-4_728, -22_314_007, 167_805],
            float_lands: FLOAT_LANDS_169_A10_T3,
        },
    ),
    (
        169,
        crate::units::anim::ATTACKWALK,
        9,
        Release {
            piece: 169,
            node: 0,
            pivot: [-1_095_403, -70_001, 32_092_583],
            at: [11_485, -22_327_740, -229_975],
            float_lands: FLOAT_LANDS_169_A10_T9,
        },
    ),
    (
        169,
        crate::units::anim::ATTACKWALK,
        12,
        Release {
            piece: 169,
            node: 0,
            pivot: [-876_323, -70_001, 32_102_455],
            at: [5_432, -22_325_630, -136_356],
            float_lands: &[],
        },
    ),
    (
        169,
        crate::units::anim::ATTACKWALK,
        17,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_329_754, -391_079],
            float_lands: FLOAT_LANDS_169_A10_T17,
        },
    ),
    (
        169,
        crate::units::anim::ATTACKWALK,
        20,
        Release {
            piece: 169,
            node: 0,
            pivot: [876_323, -70_001, 32_102_455],
            at: [6_831, -22_313_908, 169_796],
            float_lands: &[],
        },
    ),
    (
        169,
        crate::units::anim::ATTACKWALK,
        22,
        Release {
            piece: 169,
            node: 0,
            pivot: [1_051_587, -70_001, 32_094_734],
            at: [-5_001, -22_324_760, -104_692],
            float_lands: FLOAT_LANDS_169_A10_T22,
        },
    ),
    (
        169,
        crate::units::anim::ATTACKWALK,
        24,
        Release {
            piece: 169,
            node: 0,
            pivot: [1_051_587, -70_001, 32_094_734],
            at: [6_270, -22_315_861, 129_742],
            float_lands: FLOAT_LANDS_169_A10_T24,
        },
    ),
    (
        169,
        crate::units::anim::ATTACKWALK,
        29,
        Release {
            piece: 169,
            node: 0,
            pivot: [138_462, -70_001, 32_159_561],
            at: [-1_070, -22_326_462, -169_674],
            float_lands: FLOAT_LANDS_169_A10_T29,
        },
    ),
    (
        169,
        crate::units::anim::ATTACK2,
        3,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_214_104, 164_017],
            float_lands: FLOAT_LANDS_169_A12_T3,
        },
    ),
    (
        169,
        crate::units::anim::ATTACK2,
        9,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_227_757, -231_813],
            float_lands: &[],
        },
    ),
    (
        169,
        crate::units::anim::ATTACK2,
        12,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_225_658, -138_548],
            float_lands: FLOAT_LANDS_169_A12_T12,
        },
    ),
    (
        169,
        crate::units::anim::ATTACK2,
        17,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_229_755, -391_697],
            float_lands: FLOAT_LANDS_169_A12_T17,
        },
    ),
    (
        169,
        crate::units::anim::ATTACK2,
        20,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_214_008, 165_936],
            float_lands: &[],
        },
    ),
    (
        169,
        crate::units::anim::ATTACK2,
        22,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_224_792, -107_159],
            float_lands: &[],
        },
    ),
    (
        169,
        crate::units::anim::ATTACK2,
        24,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_215_950, 126_045],
            float_lands: &[],
        },
    ),
    (
        169,
        crate::units::anim::ATTACK2,
        29,
        Release {
            piece: 169,
            node: 0,
            pivot: [0, -70_001, 32_119_999],
            at: [0, -22_226_486, -171_846],
            float_lands: FLOAT_LANDS_169_A12_T29,
        },
    ),
    // **Piece 199, the Chi-Ha** (unit type 249, `CHIHA-DEFAULT-AGE0`,
    // `mediumtank_asian.bh3`; item 3745, `docs/AI.md` §369.1): run1424's
    // packet (`RON_STATE_FRAME=3156` on the Japanese Modern Great Lakes
    // seed) holds seven entries — the `(4, 0, 0)` [`NODES`] read, `(−5.69,
    // −9.19, 38.36)`, and for each node-0 `<RELEASEEVENT>` (`CHAR_ATTACK1`
    // frame 4, `CHAR_ATTACK2` 21, `CHAR_ATTACK3` 1) the pivot node's and
    // the release node's, `RData +0x88` 0.8. The art's `<RESTRICTION
    // name="CHIHA" node="4">` makes it a pivot piece. who=1's `1/35` loosed
    // at the human's `0/2003` on 3183 from the figure's own square (4152,
    // 34728, 480) against the original's (4142, 34576, 629).
    (
        199,
        crate::units::anim::ATTACK1,
        4,
        Release {
            piece: 199,
            node: 0,
            pivot: [-5_690_000, -9_190_001, 38_359_997],
            at: [-1_854_394, -29_868_113, 607_136],
            float_lands: FLOAT_LANDS_199_A11,
        },
    ),
    (
        199,
        crate::units::anim::ATTACK2,
        21,
        Release {
            piece: 199,
            node: 0,
            pivot: [-5_690_000, -9_129_395, 38_359_997],
            at: [-1_822_477, -21_904_575, 607_136],
            float_lands: FLOAT_LANDS_199_A12,
        },
    ),
    (
        199,
        crate::units::anim::ATTACK3,
        1,
        Release {
            piece: 199,
            node: 0,
            pivot: [-5_690_000, -9_190_001, 38_359_997],
            at: [-1_855_385, -30_115_305, 607_136],
            float_lands: FLOAT_LANDS_199_A13,
        },
    ),
    // **Piece 207, the T80DES** (unit type 257, `T80DES-DEFAULT-AGE0`,
    // `T80.bh3`; item 3743, `docs/AI.md` §367.2): run1419b's packet
    // (`RON_STATE_FRAME=3212` on the Egyptian Information Great Lakes seed)
    // holds fifteen entries — the `(4, 0, 0)` [`NODES`] read, `(0.0, 0.0,
    // 27.11)`, and for each node-0 `<RELEASEEVENT>` (`CHAR_ATTACK1` frame
    // 11, `CHAR_ATTACK2` 23, `CHAR_ATTACK3` 27) the pivot node's and the
    // release node's, `RData +0x88` 0.75. The art's `<RESTRICTION
    // name="T80DES" node="4">` makes it a pivot piece; its four node-1
    // events take the non-pivot arm (`1 & 3` is not below the count of 1)
    // and are not rowed here. who=1's `1/42` loosed at the human's `0/13`
    // on 3214 from the muzzle, about 320 out on the turret and 131 up,
    // where this crate loosed from the figure's own square.
    (
        207,
        crate::units::anim::ATTACK1,
        11,
        Release {
            piece: 207,
            node: 0,
            pivot: [0, 0, 27_109_999],
            at: [140_000, -90_680_344, 9_331_987],
            float_lands: FLOAT_LANDS_207_A11,
        },
    ),
    (
        207,
        crate::units::anim::ATTACK2,
        23,
        Release {
            piece: 207,
            node: 0,
            pivot: [0, 0, 27_109_999],
            at: [140_000, -91_304_268, 9_352_728],
            float_lands: FLOAT_LANDS_207_A12,
        },
    ),
    (
        207,
        crate::units::anim::ATTACK3,
        27,
        Release {
            piece: 207,
            node: 0,
            pivot: [0, 139_500, 27_138_798],
            at: [140_000, -80_489_395, 8_974_987],
            float_lands: FLOAT_LANDS_207_A13,
        },
    ),
    // **Piece 208, the Tiger Tank** (unit type 258, `TIGERTANK`; item 3738,
    // `docs/AI.md` §366.1): run1417's packet (`RON_STATE_FRAME=4016` on the
    // German Modern Great Lakes seed) holds its seven entries — the
    // `(4, 0, 0)` [`NODES`] read, `(0.0, 3.13, 28.48)`, and for each of its
    // three `<RELEASEEVENT>`s (node 0; `CHAR_ATTACK1`..`3` at frame 3) the
    // pivot node's and the release node's, `RData +0x88` 0.85. The
    // `<RESTRICTION name="TigerTank" node="4">` sits at the foot of
    // `unit_graphics.xml`. who=1's `1/38` loosed at the human's `0/12` on
    // 4017 from its muzzle, about 100 model units out on the turret, where
    // this crate loosed from the figure's own square.
    (
        208,
        crate::units::anim::ATTACK1,
        3,
        Release {
            piece: 208,
            node: 0,
            pivot: [0, 3_130_000, 28_357_819],
            at: [-1_575_654, -101_029_350, 15_676_765],
            float_lands: FLOAT_LANDS_208_A11,
        },
    ),
    (
        208,
        crate::units::anim::ATTACK2,
        3,
        Release {
            piece: 208,
            node: 0,
            pivot: [0, 3_130_000, 28_357_819],
            at: [-1_597_266, -102_177_773, 2_460_506],
            float_lands: FLOAT_LANDS_208_A12,
        },
    ),
    (
        208,
        crate::units::anim::ATTACK3,
        3,
        Release {
            piece: 208,
            node: 0,
            pivot: [0, 3_130_000, 28_357_819],
            at: [-1_483_080, -101_951_836, 7_676_441],
            float_lands: FLOAT_LANDS_208_A13,
        },
    ),
    // **Piece 196, the Sherman** (unit type 246, `SHERMAN-DEFAULT-AGE0`,
    // `mediumTank.bh3`; item 3766, `docs/AI.md` §374.1): run1164's packet
    // (`RON_STATE_FRAME=4528` on the American Modern Great Sahara seed)
    // holds its seven entries — the `(4, 0, 0)` [`NODES`] read, `(0.0,
    // −0.97, 38.36)`, and for each node-0 `<RELEASEEVENT>` (`CHAR_ATTACK1`
    // frame 4, `CHAR_ATTACK2` 21, `CHAR_ATTACK3` 1) the pivot node's and
    // the release node's, `RData +0x88` 0.85. The art's `<RESTRICTION
    // name="SHERMAN" node="4">` at the foot of `unit_graphics.xml` makes it
    // a pivot piece. The Aztecs' `1/40` loosed at the human's `0/2000` on
    // 3309 from the figure's own square (4344, 31848, 580) against the
    // original's (4155, 31681, 744).
    (
        196,
        crate::units::anim::ATTACK1,
        4,
        Release {
            piece: 196,
            node: 0,
            pivot: [0, -970_000, 38_359_997],
            at: [-424_810, -60_848_732, 1_908_176],
            float_lands: FLOAT_LANDS_196_A11,
        },
    ),
    (
        196,
        crate::units::anim::ATTACK2,
        21,
        Release {
            piece: 196,
            node: 0,
            pivot: [0, -909_394, 38_359_997],
            at: [-392_893, -52_885_201, 1_908_176],
            float_lands: FLOAT_LANDS_196_A12,
        },
    ),
    (
        196,
        crate::units::anim::ATTACK3,
        1,
        Release {
            piece: 196,
            node: 0,
            pivot: [0, -970_000, 38_359_997],
            at: [-425_801, -61_095_924, 1_908_176],
            float_lands: FLOAT_LANDS_196_A13,
        },
    ),
    (
        145,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 145,
            node: 0,
            pivot: [369_230, 24_640_001, 12_299_541],
            at: [2_249_614, -15_437_737, 30_873_745],
            float_lands: &[(155, 23, 35, 34), (335, 23, -35, -34)],
        },
    ),
    (
        145,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 145,
            node: 0,
            pivot: [0, 24_640_001, 11_550_000],
            at: [-101_367, -15_533_024, 31_973_480],
            float_lands: &[],
        },
    ),
    (
        145,
        crate::units::anim::ATTACK2,
        17,
        Release {
            piece: 145,
            node: 0,
            pivot: [0, 25_720_001, 11_550_000],
            at: [1_031_301, -15_774_166, 29_316_223],
            float_lands: &[(141, 208, 97, 169), (321, 208, -97, -169)],
        },
    ),
    (
        145,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 145,
            node: 0,
            pivot: [0, 24_640_001, 11_550_000],
            at: [49_376, -15_435_911, 31_925_163],
            float_lands: &[(141, 136, 148, 100), (321, 136, -148, -100)],
        },
    ),
    (
        296,
        crate::units::anim::ATTACK1,
        3,
        Release {
            piece: 296,
            node: 0,
            pivot: [0, -29_576_756, 18_779_999],
            at: [0, -11_213_131, 16_269_558],
            float_lands: &[],
        },
    ),
    (
        296,
        crate::units::anim::ATTACK2,
        3,
        Release {
            piece: 296,
            node: 0,
            pivot: [0, -29_555_523, 18_779_999],
            at: [0, -11_213_131, 16_269_558],
            float_lands: &[],
        },
    ),
    (
        296,
        crate::units::anim::ATTACK3,
        3,
        Release {
            piece: 296,
            node: 0,
            pivot: [0, -29_576_756, 18_779_999],
            at: [0, -11_213_131, 16_269_558],
            float_lands: &[],
        },
    ),
    // **The Bomb Ketch, piece 297** (unit type 347, `MORTARSHIP`; item
    // 3655, `docs/AI.md` §337): run1185's packet (logger frame 4582 of the
    // British Enlightenment lobby) holds the piece's eleven entries — its
    // two pivots at `(anim 0, time 0)` and, for each `<RELEASEEVENT>` (266
    // ms, frame 3), the pivot's and the release node's: node 0 from pivot 4
    // on `CHAR_ATTACK1` and `3` (the same bits), node 1 from pivot 5 on
    // `CHAR_ATTACK2`. Without them its rounds left the hull's own square at
    // the water's `z` 0: the Iroquois Enlightenment lobby's `1/82` loosed
    // at the Citizen `0/4` from (4013, 12593, 133) and flew 41 frames,
    // where this crate's left (3912, 12792, 0) and flew 43, so theirs
    // struck on 3962 and ours was still in the air (the word 3962's draw).
    (
        297,
        crate::units::anim::ATTACK1,
        3,
        Release {
            piece: 297,
            node: 0,
            pivot: [0, -50_437_000, 19_346_149],
            at: [49_741, -7_746_784, 15_496_639],
            float_lands: FLOAT_LANDS_297_N0,
        },
    ),
    (
        297,
        crate::units::anim::ATTACK2,
        3,
        Release {
            piece: 297,
            node: 1,
            pivot: [0, 5_809_750, 19_372_950],
            at: [23_246, -7_899_697, 14_525_358],
            float_lands: FLOAT_LANDS_297_N1,
        },
    ),
    (
        297,
        crate::units::anim::ATTACK3,
        3,
        Release {
            piece: 297,
            node: 0,
            pivot: [0, -50_437_000, 19_346_149],
            at: [49_741, -7_746_784, 15_496_639],
            float_lands: FLOAT_LANDS_297_N0,
        },
    ),
    // **The Dragoon** (item 1776): run773's packet, piece 491's nine entries
    // — anims 10 (`CHAR_ATTACKWALK`), 11, 12 (the stationary round) and 13.
    (
        491,
        crate::units::anim::ATTACKWALK,
        12,
        Release {
            piece: 491,
            node: 0,
            pivot: [58111, -1380116, 31034378],
            at: [-12961245, -27570284, 11694054],
            float_lands: &[
                (92, 28, -90, -120),
                (45, 342, -120, 94),
                (135, 342, -94, -120),
                (225, 342, 120, -94),
            ],
        },
    ),
    (
        491,
        crate::units::anim::ATTACK1,
        12,
        Release {
            piece: 491,
            node: 0,
            pivot: [4308261, -1638021, 31049999],
            at: [-3027300, -29267675, 15569523],
            float_lands: &[(87, 138, 102, -68), (52, 221, 146, 43), (337, 295, 162, 18)],
        },
    ),
    (
        491,
        crate::units::anim::ATTACK2,
        13,
        Release {
            piece: 491,
            node: 0,
            pivot: [0, 6483696, 34750904],
            at: [-8452114, -17947832, 20006462],
            float_lands: &[],
        },
    ),
    (
        491,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 491,
            node: 0,
            pivot: [0, -1480000, 31049999],
            at: [-4867605, -28791185, 12283539],
            float_lands: &[],
        },
    ),
    // **The Native American Horse Archers, piece 1545** (item 3255,
    // `docs/AI.md` §257): run943c's packet (frame 10674), the piece's nine
    // entries, anims 10 (`CHAR_ATTACKWALK`) to 13 and the pivot's
    // `(4, 0, 0)`. `float_lands` are the cells where `get_position.py
    // pivot 1545 0 <anim> <time> 1`'s 92,416 rows part from the integer
    // form; every other cell agrees.
    (
        1545,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 1545,
            node: 0,
            pivot: [-60_000, -2_676_834, 34_506_775],
            at: [577_225, -16_320_717, 15_752_338],
            float_lands: &[(82, 186, 65, -4), (308, 321, 88, 4)],
        },
    ),
    (
        1545,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 1545,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [1_322_394, -17_432_364, 13_891_037],
            float_lands: &[(176, 155, 45, 63)],
        },
    ),
    (
        1545,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 1545,
            node: 0,
            pivot: [-60_000, -919_060, 31_409_014],
            at: [1_600_198, -19_470_568, 11_654_627],
            float_lands: &[(11, 295, 79, 52), (191, 295, -79, -52), (281, 295, 52, -79)],
        },
    ),
    (
        1545,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 1545,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [3_391_179, -16_833_715, 14_928_272],
            float_lands: &[(129, 155, 77, 0), (219, 155, 0, 77), (309, 155, -77, 0)],
        },
    ),
    // **The Iroquois Cavalry Archer, piece 1546** (unit type 188,
    // `CAVARCHERS`; item 3375, `docs/AI.md` §288): run1001d's packet
    // (`RON_STATE_FRAME=13760`), the piece's nine entries. Three of its four
    // releases are 1545's bit for bit — the pivot's and the release node's
    // entries both, at the same scale — so their rows and `float_lands` are
    // 1545's; `CHAR_ATTACK3` releases at frame 20, from a node of its own,
    // and its `float_lands` are `get_position.py pivot 1546 0 13 20 1`'s.
    (
        1546,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 1546,
            node: 0,
            pivot: [-60_000, -2_676_834, 34_506_775],
            at: [577_225, -16_320_717, 15_752_338],
            float_lands: &[(82, 186, 65, -4), (308, 321, 88, 4)],
        },
    ),
    (
        1546,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 1546,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [1_322_394, -17_432_364, 13_891_037],
            float_lands: &[(176, 155, 45, 63)],
        },
    ),
    (
        1546,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 1546,
            node: 0,
            pivot: [-60_000, -919_060, 31_409_014],
            at: [1_600_198, -19_470_568, 11_654_627],
            float_lands: &[(11, 295, 79, 52), (191, 295, -79, -52), (281, 295, 52, -79)],
        },
    ),
    (
        1546,
        crate::units::anim::ATTACK3,
        20,
        Release {
            piece: 1546,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [4_378_072, -16_710_272, 14_675_652],
            float_lands: &[(147, 1, -63, -62), (237, 1, 62, -63)],
        },
    ),
    // **The Indian Heavy Horse Archer, piece 1898** (unit type 188,
    // `CAVARCHERS`; item 3383, `docs/AI.md` §291): run1007b's packet
    // (`RON_STATE_FRAME=15915` on the Indians' Great Sahara seed), the
    // piece's nine entries. Its pivot node is 1545's; its four release
    // nodes are its own, none of them 1545's or 1546's, and
    // `get_position.py pivot 1898 0 <anim> <time> 1` agrees with the
    // integer form on every cell of all four, so none has a `float_lands`.
    (
        1898,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 1898,
            node: 0,
            pivot: [-60_000, -2_676_834, 34_506_775],
            at: [479_090, -15_615_540, 16_108_955],
            float_lands: &[],
        },
    ),
    (
        1898,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 1898,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [1_223_768, -16_729_307, 14_252_346],
            float_lands: &[],
        },
    ),
    (
        1898,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 1898,
            node: 0,
            pivot: [-60_000, -919_060, 31_409_014],
            at: [1_516_343, -18_837_709, 12_125_257],
            float_lands: &[],
        },
    ),
    (
        1898,
        crate::units::anim::ATTACK3,
        20,
        Release {
            piece: 1898,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [4_182_650, -16_023_108, 15_027_695],
            float_lands: &[],
        },
    ),
    // **The Inca Horse Archer, piece 841** (unit type 187,
    // `HORSEARCHERS` as the Inca field it; item 3385, `docs/AI.md` §293):
    // run1011b's packet (`RON_STATE_FRAME=13105` on the Inca's Great
    // Sahara seed), the piece's nine entries. Its pivot node is 1545's;
    // its four release nodes are its own, and `CHAR_ATTACK3` releases at
    // 18 where 1898's does at 20. `get_position.py pivot 841 0 <anim>
    // <time> 1` agrees with the integer form on all but nine cells of the
    // four, which are the rows' `float_lands`.
    (
        841,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 841,
            node: 0,
            pivot: [-60_000, -2_676_834, 34_506_775],
            at: [577_225, -16_320_717, 15_752_338],
            float_lands: &[(82, 186, 65, -4), (308, 321, 88, 4)],
        },
    ),
    (
        841,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 841,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [1_322_394, -17_432_364, 13_891_037],
            float_lands: &[(176, 155, 45, 63)],
        },
    ),
    (
        841,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 841,
            node: 0,
            pivot: [-60_000, -919_060, 31_409_014],
            at: [1_600_198, -19_470_568, 11_654_627],
            float_lands: &[(11, 295, 79, 52), (191, 295, -79, -52), (281, 295, 52, -79)],
        },
    ),
    (
        841,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 841,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [3_391_179, -16_833_715, 14_928_272],
            float_lands: &[(129, 155, 77, 0), (219, 155, 0, 77), (309, 155, -77, 0)],
        },
    ),
    // **The Lakota Musket Horse, piece 157** (unit type 207,
    // `MUSKETHORSE`; item 3645, `docs/AI.md` §336): run1305b's packet
    // (`RON_STATE_FRAME=4181` on the Lakota's held Gunpowder Great Sahara
    // seed), the piece's nine entries. `<RESTRICTION name="MUSKETHORSE"
    // node="4">` makes it a pivot piece; without these rows its rounds left
    // the figure's own point. who=1's `1/54` fires on the move at the
    // Citizen `0/5` on 4181 (`CHAR_ATTACKWALK`, frame 12) from (8560,
    // 27252, 666) and the round flies 3; this crate's left (8712, 27240),
    // flew 4 and struck a frame late, which is the word 4187's draw.
    (
        157,
        crate::units::anim::ATTACKWALK,
        12,
        Release {
            piece: 157,
            node: 0,
            pivot: [138_111, -1_380_116, 31_034_378],
            at: [-12_961_245, -27_570_284, 11_694_054],
            float_lands: &[(37, 219, 140, 34), (217, 219, -140, -34)],
        },
    ),
    (
        157,
        crate::units::anim::ATTACK1,
        12,
        Release {
            piece: 157,
            node: 0,
            pivot: [4_388_261, -1_638_021, 31_049_999],
            at: [-3_027_300, -29_267_675, 15_569_523],
            float_lands: &[],
        },
    ),
    (
        157,
        crate::units::anim::ATTACK2,
        13,
        Release {
            piece: 157,
            node: 0,
            pivot: [80_000, 6_483_696, 34_750_904],
            at: [-8_452_114, -17_947_832, 20_006_462],
            float_lands: &[],
        },
    ),
    (
        157,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 157,
            node: 0,
            pivot: [80_000, -1_480_000, 31_049_999],
            at: [-4_867_605, -28_791_185, 12_283_539],
            float_lands: &[
                (63, 197, 134, 2),
                (153, 197, -2, 134),
                (333, 197, 2, -134),
                (263, 253, -27, -137),
                (75, 271, 3, 141),
            ],
        },
    ),
    // **The Lakota Eagle Feather Horse, piece 156** (unit type 206,
    // `EAGLEFEATHER`; item 3588, `docs/AI.md` §314): run1235b's packet
    // (`RON_STATE_FRAME=4127` on the Lakota's held Medieval East Indies
    // seed), the piece's nine entries. Its pivot node is 1545's; its four
    // release nodes are its own. who=1's `1/32` looses at the Citizen
    // `0/5` on 4127 from (5484, 10096, 263) and the round flies 17; this
    // crate's left the figure's own (5544, 10152), flew 18 and struck a
    // frame after the original's. `get_position.py pivot 156 0 <anim>
    // <time> 1` agrees with the integer form on all but four cells of
    // `CHAR_ATTACK3`, which are its `float_lands`.
    (
        156,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 156,
            node: 0,
            pivot: [-60_000, -2_676_834, 34_506_775],
            at: [479_090, -15_615_540, 16_108_955],
            float_lands: &[],
        },
    ),
    (
        156,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 156,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [1_223_768, -16_729_307, 14_252_346],
            float_lands: &[],
        },
    ),
    (
        156,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 156,
            node: 0,
            pivot: [-60_000, -919_060, 31_409_014],
            at: [1_516_343, -18_837_709, 12_125_257],
            float_lands: &[],
        },
    ),
    (
        156,
        crate::units::anim::ATTACK3,
        18,
        Release {
            piece: 156,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [3_239_868, -16_143_440, 15_273_258],
            float_lands: &[
                (9, 3, -2, 86),
                (99, 3, -86, -2),
                (189, 3, 2, -86),
                (279, 3, 86, 2),
            ],
        },
    ),
    // **The Horde, piece 153** (unit type 203, `HORDE`; item 3632,
    // `docs/AI.md` §331.2): run1294's packet (`RON_STATE_FRAME=4473` on the
    // Mongols' held Gunpowder East Indies seed), the piece's nine entries —
    // the pivot at `(4, anim, time)` and the release node 0 at the four
    // attack keys. who=1's `1/21` looses at the Citizen `0/5` on 4458 from
    // (5570, 10826, 245) and the round flies 17; this crate's left the
    // figure's own (5592, 10920), flew 18 and struck a frame after the
    // original's (the leaders' `treaties` bit 2, run1292c's block 4475).
    // `get_position.py pivot 153 0 <anim> <time> 1` agrees with the integer
    // form on every cell but one of `CHAR_ATTACK3`, its `float_lands`.
    (
        153,
        crate::units::anim::ATTACKWALK,
        18,
        Release {
            piece: 153,
            node: 0,
            pivot: [0, -4_586_834, 34_506_775],
            at: [224_176, -15_053_246, 19_689_476],
            float_lands: &[],
        },
    ),
    (
        153,
        crate::units::anim::ATTACK1,
        18,
        Release {
            piece: 153,
            node: 0,
            pivot: [0, -3_380_000, 32_630_001],
            at: [888_244, -16_728_748, 16_062_641],
            float_lands: &[],
        },
    ),
    (
        153,
        crate::units::anim::ATTACK2,
        16,
        Release {
            piece: 153,
            node: 0,
            pivot: [0, -3_380_000, 32_630_001],
            at: [1_242_316, -16_864_357, 16_494_263],
            float_lands: &[],
        },
    ),
    (
        153,
        crate::units::anim::ATTACK3,
        19,
        Release {
            piece: 153,
            node: 0,
            pivot: [0, -3_380_000, 32_583_332],
            at: [-18_684_110, 2_242_623, 26_876_545],
            float_lands: &[(307, 187, 82, -47)],
        },
    ),
    // **The Dreadnought, piece 298** (unit type 348; item 3523, `docs/AI.md`
    // §303): run1169's packet (logger frame 3769 of the Japanese Industrial
    // lobby) holds the piece's fourteen entries — its two pivots at
    // `(anim 0, time 0)` and, on `CHAR_ATTACK1` and `CHAR_ATTACK2` (whose
    // bits differ), a release from each turret: node 0 on frame 1 from
    // pivot 4, node 1 on frame 4 from pivot 5. who=1's `1/74` round on
    // 3758 leaves from `(2885, 11948, 121)` and flies 12; this crate's left
    // from the hull's own square, flew 13 and came down on 3770, a frame
    // after the original's struck the human's Citizen `0/5`.
    (
        298,
        crate::units::anim::ATTACK1,
        1,
        Release {
            piece: 298,
            node: 0,
            pivot: [-516_802, -84_879_997, 21_043_644],
            at: [-176_032, -27_349_998, 7_167_835],
            float_lands: FLOAT_LANDS_298_A11_N0,
        },
    ),
    (
        298,
        crate::units::anim::ATTACK1,
        4,
        Release {
            piece: 298,
            node: 1,
            pivot: [-783_855, 100_359_993, 12_234_891],
            at: [-458_421, -27_349_998, 7_155_316],
            float_lands: FLOAT_LANDS_298_A11_N1,
        },
    ),
    (
        298,
        crate::units::anim::ATTACK2,
        1,
        Release {
            piece: 298,
            node: 0,
            pivot: [641_064, -84_879_997, 21_040_218],
            at: [218_358, -27_349_998, 7_166_668],
            float_lands: FLOAT_LANDS_298_A12_N0,
        },
    ),
    (
        298,
        crate::units::anim::ATTACK2,
        4,
        Release {
            piece: 298,
            node: 1,
            pivot: [636_882, 100_359_993, 12_243_423],
            at: [372_467, -27_349_998, 7_160_307],
            float_lands: FLOAT_LANDS_298_A12_N1,
        },
    ),
    // **The Missile Cruiser, piece 280** (`AEGISCRUISER`, unit type 330;
    // item 3849, `docs/GOLDEN.md` §153.1): run1484's `RON_STATE_FRAME=676`
    // packet on the fourth melee's lobby holds the piece's 64 entries,
    // `RData +0x88` 0.9 — four turrets, nodes 4 and 5 forward (y 100.1)
    // and 6 and 7 aft (−76.9), some 20.4 up, and two launch cells a
    // turret, the event's nodes 0 and 1 at frame 10 and 2 and 3 at frame
    // 14 of anims 10, 11 and 12, each 7.7 to a side of its turret. Two
    // nodes release on one frame, so a row is found by its node too
    // ([`release_node`]). From the hull's own square at the water's 0, ours'
    // `1/14` missiles re-planned to 6 and 6 and closed on `2/15` on 677;
    // the original's leave (28521, 30127, 102) and (28486, 30184, 104),
    // re-plan to 7 and 4, and close on 678.
    (
        280,
        10,
        10,
        Release {
            piece: 280,
            node: 0,
            pivot: [-296_077, 100_137_749, 20_446_144],
            at: [7_727_216, -1_302_498, 3_747_517],
            float_lands: FLOAT_LANDS_280_A10_N0,
        },
    ),
    (
        280,
        10,
        10,
        Release {
            piece: 280,
            node: 1,
            pivot: [-302_371, 100_138_992, 21_074_867],
            at: [-7_752_890, -1_303_741, 2_870_945],
            float_lands: FLOAT_LANDS_280_A10_N1,
        },
    ),
    (
        280,
        10,
        14,
        Release {
            piece: 280,
            node: 2,
            pivot: [110_616, -76_929_840, 19_816_257],
            at: [-7_779_023, -2_321_671, 3_494_957],
            float_lands: FLOAT_LANDS_280_A10_N2,
        },
    ),
    (
        280,
        10,
        14,
        Release {
            piece: 280,
            node: 3,
            pivot: [106_620, -76_929_047, 20_215_446],
            at: [7_742_033, -2_322_464, 3_343_988],
            float_lands: FLOAT_LANDS_280_A10_N3,
        },
    ),
    (
        280,
        crate::units::anim::ATTACK1,
        10,
        Release {
            piece: 280,
            node: 0,
            pivot: [-311_503, 100_148_033, 20_618_393],
            at: [7_727_217, -1_302_498, 3_747_517],
            float_lands: FLOAT_LANDS_280_A11_N0,
        },
    ),
    (
        280,
        crate::units::anim::ATTACK1,
        10,
        Release {
            piece: 280,
            node: 1,
            pivot: [-317_796, 100_149_277, 21_247_116],
            at: [-7_752_890, -1_303_741, 2_870_945],
            float_lands: FLOAT_LANDS_280_A11_N1,
        },
    ),
    (
        280,
        crate::units::anim::ATTACK1,
        14,
        Release {
            piece: 280,
            node: 2,
            pivot: [72_743, -76_772_446, 20_328_693],
            at: [-7_786_291, -2_293_282, 3_497_929],
            float_lands: FLOAT_LANDS_280_A11_N2,
        },
    ),
    (
        280,
        crate::units::anim::ATTACK1,
        14,
        Release {
            piece: 280,
            node: 3,
            pivot: [67_948, -76_768_471, 20_727_873],
            at: [7_735_579, -2_296_265, 3_377_615],
            float_lands: FLOAT_LANDS_280_A11_N3,
        },
    ),
    (
        280,
        crate::units::anim::ATTACK2,
        10,
        Release {
            piece: 280,
            node: 0,
            pivot: [-296_077, 100_137_749, 20_446_144],
            at: [7_727_216, -1_302_498, 3_747_517],
            float_lands: FLOAT_LANDS_280_A12_N0,
        },
    ),
    (
        280,
        crate::units::anim::ATTACK2,
        10,
        Release {
            piece: 280,
            node: 1,
            pivot: [-302_371, 100_138_992, 21_074_867],
            at: [-7_752_890, -1_303_741, 2_870_945],
            float_lands: FLOAT_LANDS_280_A12_N1,
        },
    ),
    (
        280,
        crate::units::anim::ATTACK2,
        14,
        Release {
            piece: 280,
            node: 2,
            pivot: [110_616, -76_929_840, 19_816_257],
            at: [-7_779_023, -2_321_671, 3_494_957],
            float_lands: FLOAT_LANDS_280_A12_N2,
        },
    ),
    (
        280,
        crate::units::anim::ATTACK2,
        14,
        Release {
            piece: 280,
            node: 3,
            pivot: [106_620, -76_929_047, 20_215_446],
            at: [7_742_033, -2_322_464, 3_343_988],
            float_lands: FLOAT_LANDS_280_A12_N3,
        },
    ),
    // **The Advanced Battleship, piece 300** (`ADVANCEDBATTLESHIP`, unit
    // type 350; item 3849, `docs/GOLDEN.md` §153.1): run1484's packet holds
    // the piece's 38 entries, `RData +0x88` 0.8 — the turrets 4 (y −149.8,
    // 25 up) and 5 (−174.5, 40 up), and six release nodes on each of
    // `CHAR_ATTACK1` and `CHAR_ATTACK2`, one a frame (19..23 and 26..31),
    // nodes 2 and 9 together on 23. The type carries one restriction, so
    // nodes 0 and 8 (`& 3 == 0`) turn on turret 4 and nodes 1, 2, 3 and 9
    // are their own entries alone ([`release_point`]); the pivot of a node
    // with no `(node & 3) + 4` entry is zero. The rounds leave 172 up off a
    // turret and 20 off the hull's low guns, run1484's `2/16` `sz` 171–177
    // and 18–20; ours left the hull's square at 0, and `1/16` was struck
    // on 654 where the original's was on 652.
    (
        300,
        crate::units::anim::ATTACK1,
        19,
        Release {
            piece: 300,
            node: 0,
            pivot: [571_737, -149_793_243, 24_929_735],
            at: [-18_225_920, 99_009_468, 19_860_529],
            float_lands: FLOAT_LANDS_300_A11_N0,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK1,
        20,
        Release {
            piece: 300,
            node: 3,
            pivot: [0, 0, 0],
            at: [18_775_419, -50_747_276, 44_384_666],
            float_lands: FLOAT_LANDS_300_A11_N3,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK1,
        21,
        Release {
            piece: 300,
            node: 1,
            pivot: [1_068_248, -174_586_807, 39_508_366],
            at: [-18_311_508, 97_502_579, 5_313_362],
            float_lands: FLOAT_LANDS_300_A11_N1,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK1,
        22,
        Release {
            piece: 300,
            node: 8,
            pivot: [761_789, -149_731_522, 24_445_127],
            at: [18_419_985, 72_731_796, 19_904_051],
            float_lands: FLOAT_LANDS_300_A11_N8,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK1,
        23,
        Release {
            piece: 300,
            node: 2,
            pivot: [0, 0, 0],
            at: [-17_190_834, -103_420_090, 45_343_250],
            float_lands: FLOAT_LANDS_300_A11_N2,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK1,
        23,
        Release {
            piece: 300,
            node: 9,
            pivot: [1_068_248, -174_414_276, 39_870_686],
            at: [18_165_951, 70_992_447, 4_818_855],
            float_lands: FLOAT_LANDS_300_A11_N9,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK2,
        26,
        Release {
            piece: 300,
            node: 0,
            pivot: [-73_290, -149_060_257, 25_461_151],
            at: [-18_755_333, 98_966_965, 19_601_112],
            float_lands: FLOAT_LANDS_300_A12_N0,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK2,
        27,
        Release {
            piece: 300,
            node: 3,
            pivot: [0, 0, 0],
            at: [17_962_345, -50_244_362, 45_542_000],
            float_lands: FLOAT_LANDS_300_A12_N3,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK2,
        28,
        Release {
            piece: 300,
            node: 1,
            pivot: [364_993, -174_101_746, 40_694_000],
            at: [-18_415_348, 97_501_938, 4_889_656],
            float_lands: FLOAT_LANDS_300_A12_N1,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK2,
        29,
        Release {
            piece: 300,
            node: 8,
            pivot: [531_288, -149_501_785, 25_405_689],
            at: [18_205_505, 72_739_967, 20_055_214],
            float_lands: FLOAT_LANDS_300_A12_N8,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK2,
        30,
        Release {
            piece: 300,
            node: 2,
            pivot: [0, 0, 0],
            at: [-17_281_055, -103_435_577, 46_056_293],
            float_lands: FLOAT_LANDS_300_A12_N2,
        },
    ),
    (
        300,
        crate::units::anim::ATTACK2,
        31,
        Release {
            piece: 300,
            node: 9,
            pivot: [1_078_248, -174_673_279, 40_683_582],
            at: [18_165_951, 70_992_447, 4_818_851],
            float_lands: FLOAT_LANDS_300_A12_N9,
        },
    ),
    // **The MLRS, piece 223** (unit type 273; item 3975, `docs/GOLDEN.md`
    // §177): run883b's packet (the peace pair, which fields the MLRS `1/95`
    // and `1/124`) holds the piece's seventeen entries, `RData +0x88` 0.69:
    // the `(4, 0, 0)` pivot of its `<RESTRICTION name="MLRS" node="4">` and,
    // for each of its eight `<RESTRICTION>`-free rockets (node 1, frames 5,
    // 8, … 26 of `CHAR_ATTACK2`), the pivot node's entry and the rack's.
    // Node 1 is past the piece's one restriction, so `get_position` takes
    // its non-pivot arm (`0090b750:53`, `node & 3 < param_7`): the rack's
    // entry alone, turned by the figure's facing ([`release_point`]). The
    // pivot entries are kept as the packet holds them and read by nothing.
    // Without them run1542's rockets left from the launcher's own square,
    // 240 low (`2/155`'s of 649: ours (17736, 16104, 501) against the
    // original's (17714, 16065, 741)), and every blast landed elsewhere.
    (
        223,
        crate::units::anim::ATTACK2,
        5,
        Release {
            piece: 223,
            node: 1,
            pivot: [459_189, 33_981_510, 21_489_197],
            at: [12_569_457, 5_370_341, 72_714_676],
            float_lands: FLOAT_LANDS_223,
        },
    ),
    (
        223,
        crate::units::anim::ATTACK2,
        8,
        Release {
            piece: 223,
            node: 1,
            pivot: [460_000, 33_990_000, 21_770_000],
            at: [12_530_000, 3_478_557, 71_661_247],
            float_lands: FLOAT_LANDS_223,
        },
    ),
    (
        223,
        crate::units::anim::ATTACK2,
        11,
        Release {
            piece: 223,
            node: 1,
            pivot: [460_000, 33_990_000, 21_770_000],
            at: [12_530_000, 4_849_190, 72_723_297],
            float_lands: FLOAT_LANDS_223,
        },
    ),
    (
        223,
        crate::units::anim::ATTACK2,
        14,
        Release {
            piece: 223,
            node: 1,
            pivot: [460_000, 33_990_000, 21_770_000],
            at: [12_530_000, 4_849_190, 72_723_297],
            float_lands: FLOAT_LANDS_223,
        },
    ),
    (
        223,
        crate::units::anim::ATTACK2,
        17,
        Release {
            piece: 223,
            node: 1,
            pivot: [460_000, 33_990_000, 21_770_000],
            at: [12_530_000, 4_001_522, 71_734_413],
            float_lands: FLOAT_LANDS_223,
        },
    ),
    (
        223,
        crate::units::anim::ATTACK2,
        20,
        Release {
            piece: 223,
            node: 1,
            pivot: [460_000, 33_990_000, 21_770_000],
            at: [12_530_000, 2_771_771, 70_213_913],
            float_lands: FLOAT_LANDS_223,
        },
    ),
    (
        223,
        crate::units::anim::ATTACK2,
        23,
        Release {
            piece: 223,
            node: 1,
            pivot: [460_000, 33_990_000, 21_770_000],
            at: [12_530_000, 1_440_200, 68_446_014],
            float_lands: FLOAT_LANDS_223,
        },
    ),
    (
        223,
        crate::units::anim::ATTACK2,
        26,
        Release {
            piece: 223,
            node: 1,
            pivot: [460_000, 33_990_000, 21_770_000],
            at: [12_530_000, 1_440_200, 68_446_014],
            float_lands: FLOAT_LANDS_223,
        },
    ),
];

/// The release row for `(piece, anim, starttime frame)`, if the piece
/// pivots and its row is measured.
pub fn release(piece: i32, anim: i8, time: u32) -> Option<Release> {
    let piece = rows_of(piece);
    RELEASES
        .iter()
        .find(|(p, a, t, _)| (*p, *a, *t) == (piece, anim, time))
        .map(|&(_, _, _, r)| r)
}

/// [`release`] for the event's own `node`: a piece that releases two
/// nodes on one frame (the Missile Cruiser's launch cells, item 3849)
/// keeps a row a node, and the row whose node is the event's is the one
/// `get_position` reads; a piece with one row a frame answers it whatever
/// the node.
pub fn release_node(piece: i32, anim: i8, time: u32, node: i8) -> Option<Release> {
    let piece = rows_of(piece);
    let rows = || {
        RELEASES
            .iter()
            .filter(move |(p, a, t, _)| (*p, *a, *t) == (piece, anim, time))
    };
    rows()
        .find(|(_, _, _, r)| r.node == i32::from(node))
        .or_else(|| rows().next())
        .map(|&(_, _, _, r)| r)
}

/// **`fast_angle_to_degrees@00a28f70`**: a 256-entry table of
/// `(float)angle_to_degrees(i << 24)`, built at first use and indexed by
/// the angle's top byte (`a28ffe`: `sar 0x18`, `movzbl`). So a turret
/// angle is read to its 1.40625° step and then rounded to a whole degree
/// by [`angle_to_degrees`]'s own steps.
pub const fn fast_degrees(a: i32) -> i32 {
    angle_to_degrees(Angle(((a as u32) & 0xff00_0000) as i32))
}

/// **Where a round leaves a pivot piece**: `GraphicPieces::get_position
/// @0090b750`'s pivot branch, as `GraphicEvents::execute_game_events
/// @008e48e0` calls it for a release (listing `008e4a8a`–`008e4ac8`), the
/// vector the event adds to the figure's `x, y, z` (`cvttss2si`,
/// `008e4acd`–`008e4ae7`).
///
/// - `param_5` is `fast_angle_to_degrees(angle − 0x8000_0000)`
///   (`8e4c7d`–`8e4ca1`), **the top-byte table, not the [`rotation`]
///   `set_all_pivots` takes** (item 1257): `008e4a99`'s `angle_to_degrees`
///   is the kind-5 event's, a unit put out of a carrier, and the release's
///   `add_ammo` returns to `8e4ced` (`execute_game_events+0x40d`, the
///   trace's own caller). The two differ by up to two degrees: the Bomb
///   Vessel `1/6` of run484 faces `2077229056`, 354 by `angle_to_degrees`
///   and 353 by the table, and only 353 puts its rounds at the dump's
///   `(13, 155)`. `param_6` is the package's
///   `pivot_angles`, `fast_angle_to_degrees` of each `turret_angles[k]`
///   (`Guy::execute_events@005d99c0`, `005d9a2e`–`005d9a6d`).
/// - With `node & 3` below the piece's restriction count and `param_6`
///   set, the branch first takes the **pivot node's** own vector at the
///   event's anim and time (`90b882`–`90b8a6`: node `(node & 3) + 4`,
///   `param_4` from `0x14(%ebp)`) at `param_5` degrees, then rotates the
///   release node's entry by `cvttss2si(pivot_angles[node & 3] +
///   param_5)` (`90b916`–`90b926`) and adds the two.
///
/// Both degrees are whole numbers, so, as for [`offset`], the result is a
/// function of integers: `v = s·(R(d₁)·P + R(d₂)·E)` with y negated, `d₁` the
/// facing's degree and `d₂ = d₁ + fast_degrees(turret)`, each component
/// truncated once. `z` is `s·(P_z + E_z)`: the rotation is about z.
pub fn release_offset(r: &Release, facing: Angle, turret: i32) -> (i32, i32, i32) {
    release_offset_at(r, release_rotation(facing), fast_degrees(turret))
}

/// The offset a [`Release`] row puts a round at, for a piece with
/// `restrictions` pivots whose turrets read `turrets`.
///
/// **A node past the restriction count is the release node alone** (item
/// 3760, `docs/AI.md` §372.4): `execute_game_events` (`8e4c9d`) hands
/// `get_position` the `pivot_angles` and the count on every release, and for
/// a node no restriction governs the original adds neither the pivot node
/// nor a turret — run1437's sweep of the Abrams' tracer (node 1, one
/// restriction) reads one point at all 256 turret steps, 130 over the
/// figure: the tracer's own entry, without the pivot's 27.11.
pub fn release_point(
    r: &Release,
    facing: Angle,
    turrets: &[i32; 4],
    restrictions: usize,
) -> (i32, i32, i32) {
    let k = (r.node & 3) as usize;
    if k < restrictions {
        release_offset(r, facing, turrets[k])
    } else {
        release_offset(
            &Release {
                pivot: [0; 3],
                ..*r
            },
            facing,
            0,
        )
    }
}

/// The degree the release rotates by: `fast_angle_to_degrees` of the
/// figure's angle less a half turn (`8e4c7d`–`8e4c98`), `0..=360`.
pub const fn release_rotation(facing: Angle) -> i32 {
    fast_degrees(facing.0.wrapping_sub(i32::MIN))
}

/// [`release_offset`] at the facing's degree `d1` and the turret's `t`.
pub fn release_offset_at(r: &Release, d1: i32, t: i32) -> (i32, i32, i32) {
    let (sn, sd) = scale(r.piece);
    let z = sn * (r.pivot[2] + r.at[2]) / (sd * 1_000_000);
    if let Some(&(_, _, x, y)) = r.float_lands.iter().find(|c| (c.0, c.1) == (d1, t)) {
        return (x, y, z as i32);
    }
    let d2 = d1 + t;
    let (c1, s1) = (i128::from(cos_deg(d1)), i128::from(sin_deg(d1)));
    let (c2, s2) = (i128::from(cos_deg(d2)), i128::from(sin_deg(d2)));
    let [px, py, _] = r.pivot.map(i128::from);
    let [ex, ey, _] = r.at.map(i128::from);
    let (num, den) = (i128::from(sn), i128::from(sd) * 1_000_000);
    let one = i128::from(ONE);
    let x = num * (px * c1 + py * s1 + ex * c2 + ey * s2) / (den * one);
    let y = num * (px * s1 - py * c1 + ex * s2 - ey * c2) / (den * one);
    (x as i32, y as i32, z as i32)
}

#[cfg(test)]
mod tests;
