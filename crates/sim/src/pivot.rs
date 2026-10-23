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
//! integers: the node's model point in hundredths, the scale as the
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

/// A node's model-space point, in **hundredths** of a model unit, as the
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
/// SEAM: every other restricted piece. The original fills a row for each
/// when its events are built; a packet from a game that fields the type
/// reads it with `tools/recomp/get_position.py … entries <piece>`.
const NODES: &[(i32, i32, Node)] = &[
    (145, 4, Node { px: 0, py: 2464 }),
    (296, 4, Node { px: 0, py: -2945 }),
    (491, 4, Node { px: 0, py: -148 }),
    (137, 4, Node { px: -6, py: -148 }),
    (194, 4, Node { px: 0, py: -96 }),
    (1545, 4, Node { px: -6, py: -148 }),
    (1546, 4, Node { px: -6, py: -148 }),
    (1898, 4, Node { px: -6, py: -148 }),
    (841, 4, Node { px: -6, py: -148 }),
    (139, 4, Node { px: 0, py: -148 }),
    (168, 4, Node { px: 0, py: 179 }),
    (166, 4, Node { px: 0, py: 1114 }),
    (279, 4, Node { px: 0, py: -9165 }),
    (279, 5, Node { px: 0, py: -6548 }),
    (279, 6, Node { px: 0, py: 10528 }),
    (298, 4, Node { px: 0, py: -8488 }),
    (298, 5, Node { px: 0, py: 10036 }),
    (278, 4, Node { px: 0, py: -5697 }),
    (278, 5, Node { px: 0, py: -3695 }),
    (278, 6, Node { px: 0, py: 10774 }),
    (189, 4, Node { px: 0, py: -396 }),
];

/// `guy_scale` (`0xc06244`, `4.8f`), as the rational it was typed as.
const SCALE: (i64, i64) = (48, 10);

/// **The piece's own `RData +0x88`**, which `get_position` multiplies
/// `guy_scale` by (`90b7ad`; `local_28` in the decompile), as a rational:
/// 1.0 for the Chariot's piece 145 and **0.8** for the Bomb Vessel's 296
/// (`0x3f4ccccd`, run486's packet). A piece with no row is 1.0.
const RDATA_SCALE: &[(i32, (i64, i64))] = &[
    (296, (4, 5)),
    (194, (9, 10)),
    (279, (9, 10)),
    (168, (19, 20)),
    (166, (11, 10)),
    (298, (9, 10)),
    (278, (9, 10)),
    (189, (9, 10)),
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

/// `n / d` toward zero, as `cvttss2si` truncates.
const fn trunc_div(n: i64, d: i64) -> i32 {
    (n / d) as i32
}

/// The node's point.
pub fn node(piece: i32, node: i32) -> Option<Node> {
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
    let (px, py) = (n.px, n.py);
    let (c, s) = (cos_deg(d), sin_deg(d));
    let (num, den) = scale(piece);
    let den = den * 100 * ONE;
    (
        trunc_div(num * (px * c + py * s), den),
        trunc_div(num * (px * s - py * c), den),
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

/// Piece 189's [`Release::float_lands`], one list per anim (item 3529).
const FLOAT_LANDS_189_A11: &[(i32, i32, i32, i32)] = &[(35, 193, 82, -72), (305, 193, -72, -82)];
const FLOAT_LANDS_189_A12: &[(i32, i32, i32, i32)] = &[
    (14, 262, 114, 26),
    (104, 262, -26, 114),
    (194, 262, -114, -26),
    (284, 262, 26, -114),
];
const FLOAT_LANDS_189_A13: &[(i32, i32, i32, i32)] = &[];

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
/// SEAM: every other pivot piece that releases (the camel archers, the
/// Mameluke's line, the machine-gun ships). A packet
/// from a game that fields the type reads each row in a minute
/// (`tools/recomp/get_position.py … entries <piece>`).
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
        14,
        Release {
            piece: 189,
            node: 8,
            pivot: [32_120, -466_466, 37_334_103],
            at: [155_086, -27_273_699, 3_934_948],
            float_lands: FLOAT_LANDS_189_A13,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
        15,
        Release {
            piece: 278,
            node: 2,
            pivot: [-290_015, 107_739_998, 9_675_645],
            at: [2_406_699, -13_229_012, 4_556_933],
            float_lands: FLOAT_LANDS_278_N2,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
        18,
        Release {
            piece: 139,
            node: 0,
            pivot: [0, -1_480_000, 31_049_999],
            at: [-4_867_605, -28_791_185, 12_283_539],
            float_lands: &[],
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACK3,
        29,
        Release {
            piece: 166,
            node: 0,
            pivot: [0, 11_139_999, 28_069_998],
            at: [0, -19_058_563, -304_922],
            float_lands: &[],
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
    (
        145,
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
        3,
        Release {
            piece: 296,
            node: 0,
            pivot: [0, -29_576_756, 18_779_999],
            at: [0, -11_213_131, 16_269_558],
            float_lands: &[],
        },
    ),
    // **The Dragoon** (item 1776): run773's packet, piece 491's nine entries
    // — anims 10 (`CHAR_ATTACKWALK`), 11, 12 (the stationary round) and 13.
    (
        491,
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
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
        crate::anim::ATTACKWALK,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK3,
        18,
        Release {
            piece: 841,
            node: 0,
            pivot: [-60_000, -1_480_000, 32_630_001],
            at: [3_391_179, -16_833_715, 14_928_272],
            float_lands: &[(129, 155, 77, 0), (219, 155, 0, 77), (309, 155, -77, 0)],
        },
    ), // **The Dreadnought, piece 298** (unit type 348; item 3523, `docs/AI.md`
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK1,
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
        crate::anim::ATTACK2,
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
        crate::anim::ATTACK2,
        4,
        Release {
            piece: 298,
            node: 1,
            pivot: [636_882, 100_359_993, 12_243_423],
            at: [372_467, -27_349_998, 7_160_307],
            float_lands: FLOAT_LANDS_298_A12_N1,
        },
    ),
];

/// The release row for `(piece, anim, starttime frame)`, if the piece
/// pivots and its row is measured.
pub fn release(piece: i32, anim: i8, time: u32) -> Option<Release> {
    RELEASES
        .iter()
        .find(|(p, a, t, _)| (*p, *a, *t) == (piece, anim, time))
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
mod tests {
    use super::*;

    /// **The French Dragoon's rounds leave from its pivot** (item 3359):
    /// every `1/39` round the French lobby's `AMMO=5` windows print as a
    /// Dragoon, at the figure's pose on the block before the round's first
    /// print — `(anim, time, guy x, y, angle, z, sx, sy, sz, turret step)`.
    /// One turret step per pose fits all of that pose's rounds and no other
    /// step does: 16763's is the dump's own `turret_angles[0]` (−196608,
    /// `GUYS=4` on run985b), and 14621..14676's three, bearing on `0/2002`,
    /// share step −13, which no dump there prints.
    #[test]
    fn the_french_dragoon_s_rounds_leave_from_its_pivot() {
        use crate::anim::{ATTACK1, ATTACK2, ATTACK3};
        #[rustfmt::skip]
        let shots = [
            (ATTACK3, 18, 7944, 30840, -990_642_176, 658, 7797, 30845, 866, -13),
            (ATTACK1, 12, 7944, 30840, -990_642_176, 658, 7794, 30874, 881, -13),
            (ATTACK2, 13, 7944, 30840, -990_642_176, 658, 7883, 30822, 920, -13),
            (ATTACK2, 13, 6792, 31368, -271_515_648, 610, 6805, 31302, 872, -1),
        ];
        for (a, t, x, y, f, z, sx, sy, sz, step) in shots {
            let r = release(139, a, t).expect("every Dragoon release row");
            let fits: Vec<i32> = (-128..128)
                .filter(|i| {
                    let (dx, dy, _) = release_offset(&r, Angle(f), i << 24);
                    (x + dx, y + dy) == (sx, sy)
                })
                .collect();
            assert_eq!(fits, [step], "anim {a} at ({x}, {y})");
            assert_eq!(z + release_offset(&r, Angle(f), 0).2, sz, "anim {a} height");
        }
        let (dx, dy, _) = release_offset(
            &release(139, ATTACK2, 13).unwrap(),
            Angle(-271_515_648),
            -196_608,
        );
        assert_eq!(
            (6792 + dx, 31368 + dy),
            (6805, 31302),
            "the dump's own turret"
        );
    }

    /// **The dump's own line** (run147, `misc:MISC=10`): `GUY
    /// get_positiong -180 180 -102 -59 1512 7944`, `Guy::set_all_pivots`
    /// saying the Chariot's node vector on 684 against `1/7`, from `0/8`
    /// facing `1431655765` (120°). Every Chariot of chapter three prints
    /// the same pair on every frame it bears.
    #[test]
    fn the_chariot_s_node_is_the_one_its_log_line_prints() {
        let facing = Angle(1_431_655_765);
        assert_eq!(rotation(facing), 300);
        assert_eq!(offset(145, 4, facing), (-102, -59));
        // No row: the unit's own point, as `get_position` leaves it.
        assert_eq!(offset(145, 5, facing), (0, 0));
        assert_eq!(offset(12817, 4, facing), (0, 0));
    }

    /// **The bearing it gives, as the dump prints it.** run147 prints
    /// `GUYS=4`, which carries `GuyData +0x20`'s `des_turret_angles`:
    /// `set_all_pivots` writes the node's bearing less the figure's angle
    /// there. `0/8`'s reads `-496063829` on 685, after bearing on `1/7` at
    /// `1512, 7944` from `984, 8136` — the node's `(−102, −59)`, then
    /// `find_angle`, to the unit. On 684 it still reads `-382227797`, the
    /// last bearing it took before: `1/8` at `1828, 8040`, as the `MISC=10`
    /// line printed it on 658.
    #[test]
    fn the_bearing_from_the_node_is_the_dump_s_turret_angle() {
        use crate::movement::find_angle;
        let facing = Angle(1_431_655_765);
        let (vx, vy) = offset(145, 4, facing);
        let turret = |tx: i32, ty: i32| {
            find_angle(tx - 984 - vx, ty - 8136 - vy)
                .0
                .wrapping_sub(facing.0)
        };
        assert_eq!(turret(1512, 7944), -496_063_829);
        assert_eq!(turret(1828, 8040), -382_227_797);
    }

    /// **The original's pivot branch on every cell, for piece 1545** (item
    /// 3255): `tools/recomp/get_position.py … pivot 1545 0 <anim> <time> 1`
    /// on run943c's packet for its four rows. `$RON_RELEASE_TABLE_1545` is
    /// the directory holding `pivot-1545-a<anim>-t<time>.txt` (`d₁ i t x y
    /// z`), outside git (`~/ron-data/lab-experiments/2026-10-18-item-3255/`).
    /// The nine cells that part are the rows' `float_lands`; made to fail
    /// by emptying one.
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_1545() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_1545") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_1545 (item 3255's oracle, run943c)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, time) in [
            (crate::anim::ATTACKWALK, 18),
            (crate::anim::ATTACK1, 18),
            (crate::anim::ATTACK2, 16),
            (crate::anim::ATTACK3, 18),
        ] {
            let path = format!("{dir}/pivot-1545-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_1545");
            let r = release(1545, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!("{slot} {} {} {} {}", f[0], f[2], f[3], f[4]));
                }
                rows += 1;
            }
        }
        assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }

    /// **The original's pivot branch on every cell, for piece 298** (item
    /// 3523): `tools/recomp/get_position.py … pivot 298 <node> <anim> <time>
    /// 2` on run1169's packet, one sweep a row (the two anims' entries
    /// differ). `$RON_RELEASE_TABLE_298` is the directory holding
    /// `pivot-298-n<node>-a<anim>-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-23-item-3523/sweeps/`).
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_298() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_298") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_298 (item 3523's oracle, run1169)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, node, time) in [
            (crate::anim::ATTACK1, 0, 1),
            (crate::anim::ATTACK1, 1, 4),
            (crate::anim::ATTACK2, 0, 1),
            (crate::anim::ATTACK2, 1, 4),
        ] {
            let path = format!("{dir}/pivot-298-n{node}-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_298");
            let r = release(298, slot, time).expect("the row");
            assert_eq!(r.node, node);
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!(
                        "a{slot} n{node} ({}, {}, {}, {}) z {}",
                        f[0], f[2], f[3], f[4], f[5]
                    ));
                }
                rows += 1;
            }
        }
        assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }

    /// **The original's pivot branch on every cell, for piece 278** (item
    /// 3525): `tools/recomp/get_position.py … pivot 278 <node> 10 <time> 3`
    /// on run1169's packet, one sweep a turret (the three anims' entries
    /// are the same bits), checked against all nine rows.
    /// `$RON_RELEASE_TABLE_278` is the directory holding
    /// `pivot-278-n<node>-a10-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-23-item-3525/sweeps/`).
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_278() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_278") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_278 (item 3525's oracle, run1169)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (node, time) in [(0, 10), (1, 12), (2, 15)] {
            let path = format!("{dir}/pivot-278-n{node}-a10-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_278");
            for slot in [
                crate::anim::ATTACKWALK,
                crate::anim::ATTACK1,
                crate::anim::ATTACK2,
            ] {
                let r = release(278, slot, time).expect("the row");
                assert_eq!(r.node, node);
                for line in table.lines() {
                    let f: Vec<i32> = line
                        .split_whitespace()
                        .take(6)
                        .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                        .collect();
                    if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                        off.push(format!(
                            "n{node} a{slot} ({}, {}, {}, {}) z {}",
                            f[0], f[2], f[3], f[4], f[5]
                        ));
                    }
                    rows += 1;
                }
            }
        }
        assert!(rows >= 9 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }

    /// **The original's pivot branch on every cell, for piece 189** (item
    /// 3529): `tools/recomp/get_position.py … pivot 189 8 <anim> <time> 1`
    /// on run1169's packet, one sweep a row (the three anims' entries
    /// differ). `$RON_RELEASE_TABLE_189` is the directory holding
    /// `pivot-189-n8-a<anim>-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-23-item-3529/sweeps/`).
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_189() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_189") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_189 (item 3529's oracle, run1169)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, time) in [
            (crate::anim::ATTACK1, 4),
            (crate::anim::ATTACK2, 21),
            (crate::anim::ATTACK3, 14),
        ] {
            let path = format!("{dir}/pivot-189-n8-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_189");
            let r = release(189, slot, time).expect("the row");
            assert_eq!(r.node, 8);
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!(
                        "a{slot} ({}, {}, {}, {}) z {}",
                        f[0], f[2], f[3], f[4], f[5]
                    ));
                }
                rows += 1;
            }
        }
        assert!(rows >= 3 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }

    /// **The original's pivot branch on every cell, for piece 279** (item
    /// 3519): `tools/recomp/get_position.py … pivot 279 <node> 10 <time> 3`
    /// on run1165's packet, one sweep a turret (the three anims' entries
    /// are the same bits), checked against all nine rows.
    /// `$RON_RELEASE_TABLE_279` is the directory holding
    /// `pivot-279-n<node>-a10-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-23-item-3519/`).
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_279() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_279") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_279 (item 3519's oracle, run1165)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (node, time) in [(0, 10), (1, 12), (2, 15)] {
            let path = format!("{dir}/pivot-279-n{node}-a10-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_279");
            for slot in [
                crate::anim::ATTACKWALK,
                crate::anim::ATTACK1,
                crate::anim::ATTACK2,
            ] {
                let r = release(279, slot, time).expect("the row");
                assert_eq!(r.node, node);
                for line in table.lines() {
                    let f: Vec<i32> = line
                        .split_whitespace()
                        .take(6)
                        .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                        .collect();
                    if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                        off.push(format!(
                            "n{node} a{slot} ({}, {}, {}, {}) z {}",
                            f[0], f[2], f[3], f[4], f[5]
                        ));
                    }
                    rows += 1;
                }
            }
        }
        assert!(rows >= 9 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }

    /// **The original's pivot branch on every cell, for piece 168** (item
    /// 3518): `tools/recomp/get_position.py … pivot 168 0 <anim> <time> 1`
    /// on run1164's packet for its four rows. `$RON_RELEASE_TABLE_168` is
    /// the directory holding `pivot-168-a<anim>-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-23-item-3518/`).
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_168() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_168") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_168 (item 3518's oracle, run1164)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, time) in [
            (crate::anim::ATTACKWALK, 1),
            (crate::anim::ATTACK1, 1),
            (crate::anim::ATTACK2, 11),
            (crate::anim::ATTACK3, 12),
        ] {
            let path = format!("{dir}/pivot-168-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_168");
            let r = release(168, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!(
                        "{slot} {} {} {} {} {}",
                        f[0], f[2], f[3], f[4], f[5]
                    ));
                }
                rows += 1;
            }
        }
        assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }

    /// **The original's pivot branch on every cell, for piece 166** (item
    /// 3511): `tools/recomp/get_position.py … pivot 166 0 <anim> <time> 1`
    /// on run1155c's packet for its thirty-two rows — four anims, eight
    /// release frames each. `$RON_RELEASE_TABLE_166` is the directory
    /// holding `pivot-166-a<anim>-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-23-item-3511/sweeps/`).
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_166() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_166") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_166 (item 3511's oracle, run1155c)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for slot in [
            crate::anim::ATTACKWALK,
            crate::anim::ATTACK1,
            crate::anim::ATTACK2,
            crate::anim::ATTACK3,
        ] {
            for time in [3, 9, 12, 17, 20, 22, 24, 29] {
                let path = format!("{dir}/pivot-166-a{slot}-t{time}.txt");
                let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_166");
                let r = release(166, slot, time).expect("the row");
                for line in table.lines() {
                    let f: Vec<i32> = line
                        .split_whitespace()
                        .take(6)
                        .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                        .collect();
                    if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                        off.push(format!(
                            "a{slot} t{time}: ({}, {}, {}, {}) z {}",
                            f[0], f[2], f[3], f[4], f[5]
                        ));
                    }
                    rows += 1;
                }
            }
        }
        assert!(rows >= 32 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:#?}", off.len());
    }

    /// **The original's pivot branch on every cell, for piece 1546** (item
    /// 3375): `tools/recomp/get_position.py … pivot 1546 0 <anim> <time> 1`
    /// on run1001d's packet for its four rows. `$RON_RELEASE_TABLE_1546` is
    /// the directory holding `pivot-1546-a<anim>-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-20-item-3375/`). The three rows
    /// 1545 shares agree with 1545's `float_lands`; `CHAR_ATTACK3`'s two
    /// cells are its own. Made to fail by emptying them.
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_1546() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_1546") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_1546 (item 3375's oracle, run1001d)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, time) in [
            (crate::anim::ATTACKWALK, 18),
            (crate::anim::ATTACK1, 18),
            (crate::anim::ATTACK2, 16),
            (crate::anim::ATTACK3, 20),
        ] {
            let path = format!("{dir}/pivot-1546-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_1546");
            let r = release(1546, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!("{slot} {} {} {} {}", f[0], f[2], f[3], f[4]));
                }
                rows += 1;
            }
        }
        assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }

    /// **run1001c's Cavalry Archers `1/43` and `1/44`** (item 3375,
    /// `docs/AI.md` §288): every round of the window, the figure's `x, y`,
    /// `angle` and `turret_angles[0]` on the block the round is first
    /// printed — the release block, which for the three walking rounds is
    /// a step on from the block before — against the round's `sx, sy`.
    /// **The original's pivot branch on every cell, for piece 1898** (item
    /// 3383): `tools/recomp/get_position.py … pivot 1898 0 <anim> <time> 1`
    /// on run1007b's packet for its four rows. `$RON_RELEASE_TABLE_1898` is
    /// the directory holding `pivot-1898-a<anim>-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-20-item-3383/`). No row has a
    /// `float_lands`: every cell agrees with the integer form.
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_1898() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_1898") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_1898 (item 3375's oracle, run1007b)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, time) in [
            (crate::anim::ATTACKWALK, 18),
            (crate::anim::ATTACK1, 18),
            (crate::anim::ATTACK2, 16),
            (crate::anim::ATTACK3, 20),
        ] {
            let path = format!("{dir}/pivot-1898-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_1898");
            let r = release(1898, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!("{slot} {} {} {} {}", f[0], f[2], f[3], f[4]));
                }
                rows += 1;
            }
        }
        assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }
    /// **The original's pivot branch on every cell, for piece 841** (item
    /// 3385): `tools/recomp/get_position.py … pivot 841 0 <anim> <time> 1`
    /// on run1011b's packet for its four rows. `$RON_RELEASE_TABLE_841` is
    /// the directory holding `pivot-841-a<anim>-t<time>.txt`, outside git
    /// (`~/ron-data/lab-experiments/2026-10-20-item-3385/`). Nine cells of
    /// the four rows land on the float's side, and are the rows'
    /// `float_lands`.
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_piece_841() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_841") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_841 (item 3385's oracle, run1011b)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, time) in [
            (crate::anim::ATTACKWALK, 18),
            (crate::anim::ATTACK1, 18),
            (crate::anim::ATTACK2, 16),
            (crate::anim::ATTACK3, 18),
        ] {
            let path = format!("{dir}/pivot-841-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_841");
            let r = release(841, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!("{slot} {} {} {} {}", f[0], f[2], f[3], f[4]));
                }
                rows += 1;
            }
        }
        assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
    }

    /// **run1001c's Cavalry Archers `1/43` and `1/44`** (item 3375,
    /// `docs/AI.md` §288): every round of the window, the figure's `x, y`,
    /// `angle` and `turret_angles[0]` on the block the round is first
    /// printed — the release block, which for the three walking rounds is
    /// a step on from the block before — against the round's `sx, sy`.
    #[test]
    fn run1001c_s_cavalry_archers_loose_through_piece_1546_s_pivot() {
        use crate::anim::{ATTACK1, ATTACK2, ATTACK3, ATTACKWALK};
        for (slot, time, (fx, fy), facing, turret, (sx, sy)) in [
            (
                ATTACKWALK,
                18,
                (9135, 31207),
                -1_147_535_360,
                -584_384_512,
                (9081, 31274),
            ),
            (
                ATTACKWALK,
                18,
                (8804, 31456),
                -1_082_130_432,
                -785_186_816,
                (8764, 31529),
            ),
            (
                ATTACK2,
                16,
                (9144, 31224),
                -1_777_139_712,
                -1_441_792,
                (9102, 31312),
            ),
            (
                ATTACK3,
                20,
                (8568, 31464),
                -2_013_921_280,
                5_505_024,
                (8573, 31553),
            ),
            (
                ATTACKWALK,
                18,
                (7495, 31438),
                -1_064_763_392,
                551_092_224,
                (7425, 31385),
            ),
            (
                ATTACK3,
                20,
                (7416, 31416),
                -488_112_128,
                -65_536,
                (7342, 31367),
            ),
            (
                ATTACKWALK,
                18,
                (7611, 30979),
                -720_240_640,
                0,
                (7531, 30936),
            ),
            (
                ATTACK2,
                16,
                (7416, 31416),
                -488_112_128,
                -65_536,
                (7344, 31350),
            ),
            (
                ATTACK1,
                18,
                (6264, 30696),
                -827_326_464,
                23_986_176,
                (6178, 30670),
            ),
            (
                ATTACK2,
                16,
                (6264, 30696),
                -827_326_464,
                304_414_720,
                (6189, 30634),
            ),
            (
                ATTACK2,
                16,
                (6264, 30696),
                -827_326_464,
                -162_529_280,
                (6167, 30693),
            ),
        ] {
            let r = release(1546, slot, time).expect("the Cavalry Archer's row");
            let (x, y, _) = release_offset(&r, Angle(facing), turret);
            assert_eq!((fx + x, fy + y), (sx, sy), "slot {slot} from ({fx}, {fy})");
        }
        assert_eq!(node(1546, 4), Some(Node { px: -6, py: -148 }));
    }

    /// **run1011b's Inca Horse Archer `1/72`** (item 3385, `docs/AI.md`
    /// §293): the round loosed at the Nubians' building `0/2000` on block
    /// 13112, `CHAR_ATTACKWALK` at frame 18, from the figure at (7629,
    /// 31482) `z` 652, facing −1717370880, `turret_angles[0]` 1182990336
    /// — the dump's `AMMO` `sx, sy, sz` (7563, 31441, 893), `total_time`
    /// 16, where this crate loosed from the figure's own point, 17, and the
    /// building was struck a frame late (the word 13126). Made to fail with
    /// 841's rows out of `RELEASES`.
    #[test]
    fn run1011b_s_inca_horse_archer_looses_through_piece_841_s_pivot() {
        let r = release(841, crate::anim::ATTACKWALK, 18).expect("the Inca Horse Archer's row");
        let (x, y, z) = release_offset(&r, Angle(-1_717_370_880), 1_182_990_336);
        assert_eq!((7629 + x, 31482 + y, 652 + z), (7563, 31441, 893));
        assert_eq!(node(841, 4), Some(Node { px: -6, py: -148 }));
    }

    /// **run1007b's Heavy Horse Archer `1/64`** (item 3383, `docs/AI.md`
    /// §291): the Indians' one round of the window, `CHAR_ATTACK1` at frame
    /// 18, loosed at the Nubians' building `0/2005` from the figure at
    /// (8760, 30264) `z` 641, facing −1104936960, `turret_angles[0]`
    /// 262209536 — the dump's `AMMO` `sx, sy, sz` (8675, 30245, 866),
    /// `total_time` 8, where this crate loosed from the figure's own point,
    /// 9, and the building was struck a frame late (the word 15922). Made
    /// to fail with 1898's rows out of `RELEASES`.
    #[test]
    fn run1007b_s_heavy_horse_archer_looses_through_piece_1898_s_pivot() {
        let r = release(1898, crate::anim::ATTACK1, 18).expect("the Heavy Horse Archer's row");
        let (x, y, z) = release_offset(&r, Angle(-1_104_936_960), 262_209_536);
        assert_eq!((8760 + x, 30264 + y, 641 + z), (8675, 30245, 866));
        assert_eq!(node(1898, 4), Some(Node { px: -6, py: -148 }));
    }

    /// **run943b's Horse Archers `1/51`** (item 3255, `docs/AI.md` §257):
    /// two rounds at the figure's own `turret_angles[0]` 291307520 (24°
    /// by the top byte), facing −1280966656 — `CHAR_ATTACKWALK` at frame
    /// 18 on 10687 from (5034, 31074) to the dump's `AMMO` (4944, 31073),
    /// and `CHAR_ATTACK3` at 18 on 10739 from (5016, 31080) to (4928,
    /// 31089). Made to fail with the rows out of `RELEASES`: the rounds
    /// leave from the figure.
    #[test]
    fn run943b_s_horse_archer_looses_through_its_pivot() {
        use crate::anim::{ATTACK3, ATTACKWALK};
        let (facing, turret) = (Angle(-1_280_966_656), 291_307_520);
        for (slot, (fx, fy), (sx, sy)) in [
            (ATTACKWALK, (5034, 31074), (4944, 31073)),
            (ATTACK3, (5016, 31080), (4928, 31089)),
        ] {
            let r = release(1545, slot, 18).expect("the Horse Archers' row");
            let (x, y, _) = release_offset(&r, facing, turret);
            assert_eq!((fx + x, fy + y), (sx, sy), "slot {slot}");
        }
        assert_eq!(node(1545, 4), Some(Node { px: -6, py: -148 }));
    }

    /// The table is the sine: a quarter turn, the thirty and the axes.
    #[test]
    fn the_whole_degree_sine_folds_to_every_quadrant() {
        assert_eq!(sin_deg(30), ONE / 2);
        assert_eq!(sin_deg(90), ONE);
        assert_eq!(sin_deg(150), ONE / 2);
        assert_eq!(sin_deg(210), -ONE / 2);
        assert_eq!(sin_deg(330), -ONE / 2);
        assert_eq!(sin_deg(360), 0);
        assert_eq!(cos_deg(0), ONE);
        assert_eq!(cos_deg(180), -ONE);
        assert_eq!(cos_deg(60), ONE / 2);
    }

    /// **The dump's own rounds, from the unit's point** (run145 and
    /// run146, `AMMO=5` and `GUYS=2`): the shooter's facing, the slot's
    /// row (its `dz`, 208 or 196, tells `CHAR_ATTACK1`/`3` from `2`),
    /// and the turret step that reproduces `sx, sy, sz` less the figure.
    /// The turret itself is the simulation's to get right, and the
    /// chapter's widening compares every round; this pins the geometry.
    /// Made to fail first: with the pivot term dropped (the release node
    /// about the unit's point, as `get_position` would place it with no
    /// `(4, anim, time)` entry) 651's round comes out at `(74, −6)`.
    #[test]
    fn a_chariot_round_leaves_through_its_turret() {
        use crate::anim::{ATTACK1, ATTACK2, ATTACK3};
        // (facing, slot, frame, turret degree, offset from the unit)
        let rounds = [
            (1_431_655_765, ATTACK1, 18, 325, (-28, -65, 208)), // run145 651 0/8
            (1_431_655_765, ATTACK2, 17, 338, (-31, -56, 196)), // run145 676 0/7
            (1_431_655_765, ATTACK3, 18, 347, (-31, -37, 208)), // run145 678 0/6
            (1_124_925_440, ATTACK2, 17, 42, (-66, 42, 196)),   // run146 707 0/7
            (898_039_808, ATTACK1, 18, 22, (-40, 40, 208)),     // run146 709 0/9
        ];
        for (facing, slot, frame, t, want) in rounds {
            let r = release(145, slot, frame).expect("the Chariot's row");
            assert_eq!(
                release_offset_at(&r, release_rotation(Angle(facing)), t),
                want,
                "{facing} {slot} {t}"
            );
        }
        // `fast_angle_to_degrees`' steps: the top byte, then the round.
        assert_eq!(fast_degrees(0x00ff_ffff), 0);
        assert_eq!(fast_degrees(0x0100_0000), 1);
        assert_eq!(fast_degrees(0x0200_0000), 3);
        assert_eq!(fast_degrees(-410_670_421), 325);
    }

    /// **The Bomb Vessel's rounds leave through its turret at the table's
    /// facing** (item 1257, run484, `AMMO=5` and `GUYS=4`): `1/6` stands at
    /// `(21288, 14712)` facing `2077229056` with `turret_angles[0]`
    /// `96862208` (7°), and every round it fires — `CHAR_ATTACK1` on 632
    /// and 852, `CHAR_ATTACK2` on 687 — prints `sx, sy, sz` `(21301, 14867,
    /// 134)`. By `angle_to_degrees` the facing is 354 and the offset `(11,
    /// 156)`, which is what the original's own `get_position` answers on
    /// run486's packet at 354; by `fast_angle_to_degrees` it is 353, and
    /// the offset is the dump's. Made to fail with [`release_rotation`]
    /// answering [`rotation`]: `(11, 156, 134)`.
    #[test]
    fn a_bomb_vessel_round_leaves_through_its_turret_at_the_table_s_facing() {
        use crate::anim::{ATTACK1, ATTACK2};
        let facing = Angle(2_077_229_056);
        assert_eq!(rotation(facing), 354);
        assert_eq!(release_rotation(facing), 353);
        for slot in [ATTACK1, ATTACK2] {
            let r = release(296, slot, 3).expect("the Bomb Vessel's row");
            assert_eq!(
                release_offset(&r, facing, 96_862_208),
                (13, 155, 134),
                "{slot}"
            );
        }
        // The pivot node the turret bears from, at 0.8 of `guy_scale`:
        // `(0, −29.45)` turned by 354° is `(11, 111)`.
        assert_eq!(offset(296, 4, facing), (11, 112));
    }

    /// **The original's pivot branch on every reachable cell.**
    /// `get_position(145, node 0, anim, time, d₁, pivot_angles = [t],
    /// count 1)` under unicorn on run147's packet, for all four rows, `d₁
    /// = 0..=360` and `t` each of `fast_angle_to_degrees`' 256 steps (the
    /// packet's own table): 369,664 cells, 23 s. The exact form misses
    /// six, each pinned in its row's [`Release::float_lands`]; made to
    /// fail by emptying one. The table is `$RON_RELEASE_TABLE`, `slot d₁ t
    /// ret x y z …`, outside git
    /// (`~/ron-data/lab-experiments/2026-09-23-item-602-release-oracle.txt`).
    #[test]
    fn the_original_s_release_agrees_on_every_cell() {
        let Ok(path) = std::env::var("RON_RELEASE_TABLE") else {
            eprintln!("skipping: set RON_RELEASE_TABLE (item 602's oracle, run147)");
            return;
        };
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE");
        let (mut rows, mut off) = (0, Vec::new());
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(7)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            let slot = i8::try_from(f[0]).expect("a slot");
            let frame = if slot == crate::anim::ATTACK2 { 17 } else { 18 };
            let r = release(145, slot, frame).expect("the row");
            if release_offset_at(&r, f[1], f[2]) != (f[4], f[5], f[6]) {
                off.push(line.to_string());
            }
            rows += 1;
        }
        assert!(rows >= 369_664, "{path}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
        eprintln!("{rows} cells agree");
    }

    /// **The original's pivot branch on every reachable cell, for the Bomb
    /// Vessel** (item 1257): `tools/recomp/get_position.py … pivot 296 0
    /// <anim> 3 1` on run486's packet, for its three rows, `d₁ = 0..=360`
    /// and each of the 256 turret steps: 277,248 cells. The table is
    /// `$RON_RELEASE_TABLE_296`, `slot d₁ t 0 x y z`, outside git
    /// (`~/ron-data/lab-experiments/2026-09-30-item-1257/release-oracle-296.txt`).
    /// Every cell agrees, so the rows carry no `float_lands`; made to fail
    /// by scaling the piece at 1.0.
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_the_bomb_vessel() {
        let Ok(path) = std::env::var("RON_RELEASE_TABLE_296") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_296 (item 1257's oracle, run486)");
            return;
        };
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_296");
        let (mut rows, mut off) = (0, Vec::new());
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(7)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            let slot = i8::try_from(f[0]).expect("a slot");
            let r = release(296, slot, 3).expect("the row");
            if release_offset_at(&r, f[1], f[2]) != (f[4], f[5], f[6]) {
                off.push(line.to_string());
            }
            rows += 1;
        }
        assert!(rows >= 277_248, "{path}: {rows} rows");
        assert!(
            off.is_empty(),
            "{} cells disagree: {:?}",
            off.len(),
            &off[..off.len().min(20)]
        );
        eprintln!("{rows} cells agree");
    }

    /// **The original's pivot branch on every reachable cell, for the
    /// Dragoon** (item 1776): `tools/recomp/get_position.py … pivot 491 0
    /// <anim> <time> 1` on run773's packet for its four rows, `d₁ = 0..=360`
    /// and each of the 256 turret steps. `$RON_RELEASE_TABLE_491` is the
    /// directory holding `release-oracle-491-a<anim>.txt`, `d₁ i t x y z`
    /// a line, outside git (`~/ron-data/lab-experiments/2026-10-13-item-1776/`).
    /// Every cell agrees, `float_lands` aside.
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_the_dragoon() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_491") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_491 (item 1776's oracle, run773)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, time) in [
            (crate::anim::ATTACKWALK, 12),
            (crate::anim::ATTACK1, 12),
            (crate::anim::ATTACK2, 13),
            (crate::anim::ATTACK3, 18),
        ] {
            let path = format!("{dir}/release-oracle-491-a{slot}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_491");
            let r = release(491, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!("{slot} {} {} {} {}", f[0], f[2], f[3], f[4]));
                }
                rows += 1;
            }
        }
        assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
        eprintln!("{rows} cells agree");
    }

    /// **The original's pivot branch on every reachable cell, for the Horse
    /// Archer** (item 2997): `tools/recomp/get_position.py … pivot 137 0
    /// <anim> <time> 1` on run912e's packet for its four rows, `d₁ =
    /// 0..=360` and each of the 256 turret steps; and its non-pivot sweep
    /// of node 4 at `(anim 0, time 0)`, which [`offset_at`] reproduces.
    /// `$RON_RELEASE_TABLE_137` is the directory holding
    /// `pivot-137-a<anim>-t<time>.txt` (`d₁ i t x y z`) and
    /// `node-137-n4.txt` (`d 0 x y z …`), outside git
    /// (`~/ron-data/lab-experiments/2026-10-17-item-2997/`).
    #[test]
    fn the_original_s_release_agrees_on_every_cell_for_the_horse_archer() {
        let Ok(dir) = std::env::var("RON_RELEASE_TABLE_137") else {
            eprintln!("skipping: set RON_RELEASE_TABLE_137 (item 2997's oracle, run912e)");
            return;
        };
        let (mut rows, mut off) = (0, Vec::new());
        for (slot, time) in [
            (crate::anim::ATTACKWALK, 18),
            (crate::anim::ATTACK1, 18),
            (crate::anim::ATTACK2, 16),
            (crate::anim::ATTACK3, 18),
        ] {
            let path = format!("{dir}/pivot-137-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_137");
            let r = release(137, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!("{slot} {} {} {} {}", f[0], f[2], f[3], f[4]));
                }
                rows += 1;
            }
        }
        assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
        assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
        let table = std::fs::read_to_string(format!("{dir}/node-137-n4.txt")).expect("node table");
        let mut degrees = 0;
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(4)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            // The sweep's argument is the degree `get_position` is handed,
            // which is [`rotation`]'s answer.
            assert_eq!(offset_at(137, 4, f[0]), (f[2], f[3]), "node 4 at {}", f[0]);
            degrees += 1;
        }
        assert_eq!(degrees, 361);
        eprintln!("{rows} cells and {degrees} degrees agree");
    }

    /// **The original on every degree.** `get_position(145, node 4, anim
    /// 0, time 0, d)` for `d = 0..=360`, called under unicorn on run147's
    /// packet, `cvttss2si` of each component; the table is
    /// `$RON_PIVOT_TABLE`, `get_position <piece> <node> <deg> -> <v0> <v1>
    /// …`, outside git (`~/ron-data/lab-experiments/2026-09-23-item-603/`).
    /// A machine without it says so.
    #[test]
    fn the_original_s_node_agrees_on_every_degree() {
        let Ok(path) = std::env::var("RON_PIVOT_TABLE") else {
            eprintln!("skipping: set RON_PIVOT_TABLE (item 603's oracle, run147)");
            return;
        };
        let table = std::fs::read_to_string(&path).expect("RON_PIVOT_TABLE");
        let mut rows = 0;
        for line in table.lines() {
            let f: Vec<&str> = line.split_whitespace().collect();
            assert!(
                f.len() >= 7 && f[0] == "get_position" && f[4] == "->",
                "{line}"
            );
            let v = |i: usize| f[i].parse::<i32>().unwrap_or_else(|_| panic!("{line}"));
            let (piece, n, d) = (v(1), v(2), v(3));
            assert_eq!(offset_at(piece, n, d), (v(5), v(6)), "{line}");
            rows += 1;
        }
        assert!(rows >= 360, "{path}: {rows} rows");
        eprintln!("{rows} degrees agree");
    }

    /// **The gate is the event's, whatever the vectors** (item 1117,
    /// `docs/COMBAT.md` §85): `execute_game_events`' `008e4c28`–`008e4c4c`
    /// hold a release on node `n` of a piece with restrictions while
    /// `node_flags` lacks bit `n & 3`, and pass it once `n & 3` is past the
    /// restriction count. run404's Battery `1/9` on 779: one `<RESTRICTION>`
    /// (node 4), `CHAR_ATTACK2`'s node-0 event on frame 4 held with
    /// `node_flags` 14, its node-1 event on frame 7 fired on 782. Its piece
    /// has no [`RELEASES`] row, which is what the gate had been keyed on.
    ///
    /// Made to fail first: with the gate keyed on [`release`] again, the
    /// held case fires a round (ours on 779, the word).
    #[test]
    fn a_turret_short_of_its_aim_holds_its_release_on_any_piece() {
        use crate::combat::Obj;
        use crate::world::{Pos, World};
        const BATTERY: i32 = 282;
        const PIECE: i32 = 12_345;
        let fire = |node: i8, flags: u16| -> usize {
            let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
            sim.at_war[0][1] = true;
            sim.at_war[1][0] = true;
            let ty = sim.add_unit_type(crate::UnitType {
                hits: 100,
                type_index: BATTERY,
                combat: crate::combat::Profile {
                    attack: 15,
                    max_range: 8,
                    uber_size: 1,
                    ..crate::combat::Profile::default()
                },
                ..crate::UnitType::default()
            });
            sim.art
                .pivots
                .insert(BATTERY, [(4, (-180, 180))].into_iter().collect());
            sim.art.lengths.insert((PIECE, crate::anim::ATTACK2), 20);
            sim.art.releases.insert(
                PIECE,
                [(crate::anim::ATTACK2, vec![(4, node, false, false)])]
                    .into_iter()
                    .collect(),
            );
            let put = |sim: &mut crate::Sim, who: u8, p: Pos| {
                let index = i16::try_from(sim.units.len()).unwrap();
                let mut u = crate::Unit::new(who, index, p, 100);
                u.ty = Some(ty);
                u.on_map = true;
                sim.add_unit(u)
            };
            let me = put(&mut sim, 1, Pos::new(21384, 17256));
            let foe = put(&mut sim, 0, Pos::new(21384, 16872));
            sim.order_attack(me, Obj::Unit(foe));
            sim.units[me].combat.target = Some(Obj::Unit(foe));
            let mut g = crate::anim::Guy::fresh(PIECE);
            g.anim = crate::anim::ATTACK2;
            g.end_time = 20;
            g.cur_time = 3;
            g.last_time = 2;
            g.stopped = false;
            g.turret.node_flags = flags;
            sim.units[me].guys = vec![g];
            sim.guys_inc_time();
            assert_eq!(
                sim.units[me].guys[0].cur_time, 4,
                "the clock reached the event"
            );
            sim.projectiles.len()
        };
        assert_eq!(release(PIECE, crate::anim::ATTACK2, 4), None);
        assert_eq!(fire(0, 14), 0, "node 0, bit 0 clear: held (run404's 779)");
        assert_eq!(fire(0, 15), 1, "node 0, bit 0 set: fired");
        assert_eq!(fire(1, 14), 1, "node 1, past the one restriction: fired");
        assert_eq!(fire(1, 0), 1, "whatever the bits");
    }
}
