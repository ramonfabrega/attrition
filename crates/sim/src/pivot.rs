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
/// SEAM: every other restricted piece. The original fills a row for each
/// when its events are built; no capture on this disk has built one, and
/// the two packets taken before run147 hold the Chariot's pieces loaded
/// with no `AttachPos` at all.
const NODES: &[(i32, i32, Node)] = &[(145, 4, Node { px: 0, py: 2464 })];

/// `guy_scale` (`0xc06244`, `4.8f`), as the rational it was typed as,
/// times the piece's `+0x88` of 1.0.
const SCALE: (i64, i64) = (48, 10);

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
    let den = SCALE.1 * 100 * ONE;
    (
        trunc_div(SCALE.0 * (px * c + py * s), den),
        trunc_div(SCALE.0 * (px * s - py * c), den),
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
    /// The event's `node` (`GraphicEvent +0x23`).
    pub node: i32,
    /// The pivot node's entry at the event's anim and time.
    pub pivot: [i64; 3],
    /// The release node's entry.
    pub at: [i64; 3],
}

/// `(piece, anim, starttime frame) → release`. **Measured, not derived**:
/// run147's packet holds piece 145's nine `AttachPos` entries, the
/// `(4, 0, 0)` [`NODES`] reads and, for each of the Chariot's four
/// `<RELEASEEVENT>`s (node 0; 1232 ms on `CHAR_ATTACK1`, `CHAR_ATTACK3`
/// and `CHAR_ATTACKWALK`, 1166 on `CHAR_ATTACK2`: frames 18 and 17), the
/// pivot node's and the release node's own. `GraphicEvents::init_unit_events
/// @008e2520:833–841` fills exactly these when the piece's events are
/// built. The bit patterns are in `docs/COMBAT.md` §55.2.
///
/// SEAM: every other pivot piece that releases (the horse archers, the
/// camel archers, the Mameluke's line, the machine-gun ships). A packet
/// from a game that fields the type reads each row in a minute.
const RELEASES: &[(i32, i8, u32, Release)] = &[
    (
        145,
        crate::anim::ATTACKWALK,
        18,
        Release {
            node: 0,
            pivot: [369_230, 24_640_001, 12_299_541],
            at: [2_249_614, -15_437_737, 30_873_745],
        },
    ),
    (
        145,
        crate::anim::ATTACK1,
        18,
        Release {
            node: 0,
            pivot: [0, 24_640_001, 11_550_000],
            at: [-101_367, -15_533_024, 31_973_480],
        },
    ),
    (
        145,
        crate::anim::ATTACK2,
        17,
        Release {
            node: 0,
            pivot: [0, 25_720_001, 11_550_000],
            at: [1_031_301, -15_774_166, 29_316_223],
        },
    ),
    (
        145,
        crate::anim::ATTACK3,
        18,
        Release {
            node: 0,
            pivot: [0, 24_640_001, 11_550_000],
            at: [49_376, -15_435_911, 31_925_163],
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
/// - `param_5` is `(float)angle_to_degrees(angle − 0x8000_0000)`, the
///   [`rotation`] `set_all_pivots` takes; `param_6` is the package's
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
    let d1 = rotation(facing);
    let d2 = d1 + fast_degrees(turret);
    let (c1, s1) = (i128::from(cos_deg(d1)), i128::from(sin_deg(d1)));
    let (c2, s2) = (i128::from(cos_deg(d2)), i128::from(sin_deg(d2)));
    let [px, py, pz] = r.pivot.map(i128::from);
    let [ex, ey, ez] = r.at.map(i128::from);
    let (num, den) = (i128::from(SCALE.0), i128::from(SCALE.1) * 1_000_000);
    let one = i128::from(ONE);
    let x = num * (px * c1 + py * s1 + ex * c2 + ey * s2) / (den * one);
    let y = num * (px * s1 - py * c1 + ex * s2 - ey * c2) / (den * one);
    let z = num * (pz + ez) / den;
    (x as i32, y as i32, z as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
