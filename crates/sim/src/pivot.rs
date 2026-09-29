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
            float_lands: &[(155, 23, 35, 34), (335, 23, -35, -34)],
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
            float_lands: &[],
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
            float_lands: &[(141, 208, 97, 169), (321, 208, -97, -169)],
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
            float_lands: &[(141, 136, 148, 100), (321, 136, -148, -100)],
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
    release_offset_at(r, rotation(facing), fast_degrees(turret))
}

/// [`release_offset`] at the facing's degree `d1` and the turret's `t`.
pub fn release_offset_at(r: &Release, d1: i32, t: i32) -> (i32, i32, i32) {
    let z = SCALE.0 * (r.pivot[2] + r.at[2]) / (SCALE.1 * 1_000_000);
    if let Some(&(_, _, x, y)) = r.float_lands.iter().find(|c| (c.0, c.1) == (d1, t)) {
        return (x, y, z as i32);
    }
    let d2 = d1 + t;
    let (c1, s1) = (i128::from(cos_deg(d1)), i128::from(sin_deg(d1)));
    let (c2, s2) = (i128::from(cos_deg(d2)), i128::from(sin_deg(d2)));
    let [px, py, _] = r.pivot.map(i128::from);
    let [ex, ey, _] = r.at.map(i128::from);
    let (num, den) = (i128::from(SCALE.0), i128::from(SCALE.1) * 1_000_000);
    let one = i128::from(ONE);
    let x = num * (px * c1 + py * s1 + ex * c2 + ey * s2) / (den * one);
    let y = num * (px * s1 - py * c1 + ex * s2 - ey * c2) / (den * one);
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
                release_offset_at(&r, rotation(Angle(facing)), t),
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
                [(crate::anim::ATTACK2, vec![(4, node, false)])]
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
