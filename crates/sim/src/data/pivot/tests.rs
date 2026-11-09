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

/// **The Chinese Dragoon's round leaves from its pivot** (item 3639,
/// `docs/AI.md` §333.2): piece 1195 reads piece 139's rows
/// ([`SAME_AS`]), and run1283b's `AMMO=5` record of `1/97`'s
/// `CHAR_ATTACKWALK` round of 5130 — the figure at (8022, 10152), z
/// 265, angle −332857344 on block 5130 — leaves from (8009, 10001) at
/// height 470, which one turret step fits.
#[test]
fn the_chinese_dragoon_s_round_leaves_from_its_pivot() {
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3, ATTACKWALK};
    for (a, t) in [
        (ATTACKWALK, 12),
        (ATTACK1, 12),
        (ATTACK2, 13),
        (ATTACK3, 18),
    ] {
        assert_eq!(release(1195, a, t), release(139, a, t), "anim {a}");
        assert!(release(1195, a, t).is_some(), "anim {a}");
    }
    assert_eq!(node(1195, 4), node(139, 4));
    let r = release(1195, ATTACKWALK, 12).unwrap();
    let f = Angle(-332_857_344);
    let fits: Vec<i32> = (-128..128)
        .filter(|i| {
            let (dx, dy, _) = release_offset(&r, f, i << 24);
            (8022 + dx, 10152 + dy) == (8009, 10001)
        })
        .collect();
    assert_eq!(fits.len(), 1, "one turret step: {fits:?}");
    assert_eq!(265 + release_offset(&r, f, 0).2, 470, "the dump's height");
}

/// **The A7V's round leaves from its turret's muzzle** (item 3677,
/// `docs/AI.md` §346): piece 192 reads piece 189's rows ([`SAME_AS`]),
/// and run1334b's `AMMO=5` record of `1/48`'s `CHAR_ATTACK3` round of
/// 4304 — the figure at (5544, 31512), z 409, angle −748683264 on block
/// 4303 — leaves from (5427, 31534) at height 587, which one turret step
/// fits. Made to fail with the row out of `SAME_AS`.
#[test]
fn the_a7v_s_round_leaves_from_its_turret() {
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3};
    for (a, t) in [(ATTACK1, 4), (ATTACK2, 21), (ATTACK3, 14)] {
        assert_eq!(release(192, a, t), release(189, a, t), "anim {a}");
        assert!(release(192, a, t).is_some(), "anim {a}");
    }
    assert_eq!(node(192, 4), node(189, 4));
    let r = release(192, ATTACK3, 14).unwrap();
    let f = Angle(-748_683_264);
    let fits: Vec<i32> = (-128..128)
        .filter(|i| {
            let (dx, dy, _) = release_offset(&r, f, i << 24);
            (5544 + dx, 31512 + dy) == (5427, 31534)
        })
        .collect();
    assert!(!fits.is_empty(), "a turret step: {fits:?}");
    assert_eq!(409 + release_offset(&r, f, 0).2, 587, "the dump's height");
}

/// **The Egyptian Light Tank's rounds leave from its turret's muzzle**
/// (item 3707, `docs/AI.md` §360.2): piece 193 (`A7VDES`, unit type 243)
/// is `lighttank.bh3` at scale 0.9 with `<RESTRICTION name="A7VDES"
/// node="4">` and piece 189's anims and `HOWITZER BLAST` nodes, so it reads
/// piece 189's rows ([`SAME_AS`]). All seven of run1374b's `AMMO=5`
/// rounds from who=1's `1/54` and `1/66` at their first frame, each beside
/// its figure's `GUY` pose on the block before, are reproduced at the
/// dump's `sx, sy` and `sz − z` by exactly one of the three keys at some
/// turret step. Made to fail with the row out of `SAME_AS`.
#[test]
fn run1374b_s_egyptian_light_tank_rounds_leave_from_its_turret() {
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3};
    // (x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32); 7] = [
        (6120, 10968, -505_479_168, 6098, 10851, 178), // 4940 1/54
        (6120, 10968, -505_479_168, 6085, 10848, 177), // 4970 1/54
        (6792, 10536, -317_063_168, 6724, 10430, 177), // 4972 1/66
        (6792, 10536, -317_063_168, 6736, 10431, 178), // 5025 1/66
        (5928, 10632, -350_420_992, 5866, 10530, 178), // 5032 1/54
        (6792, 10536, -317_063_168, 6778, 10418, 178), // 5066 1/66
        (5304, 10296, -107_675_648, 5279, 10165, 175), // 5102 1/54
    ];
    for (x, y, facing, sx, sy, dz) in rows {
        let f = Angle(facing);
        let keys: Vec<i8> = [(ATTACK1, 4), (ATTACK2, 21), (ATTACK3, 14)]
            .into_iter()
            .filter(|&(a, t)| {
                let Some(r) = release(193, a, t) else {
                    return false;
                };
                release_offset(&r, f, 0).2 == dz
                    && (-128..128).any(|i| {
                        let (dx, dy, _) = release_offset(&r, f, i << 24);
                        (x + dx, y + dy) == (sx, sy)
                    })
            })
            .map(|(a, _)| a)
            .collect();
        assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
    }
    assert_eq!(node(193, 4), node(189, 4));
}

/// **The Armored Cavalry's rounds leave from its turret's muzzle**
/// (item 3756, `docs/AI.md` §372.2): piece 170 is a pivot piece by
/// `<RESTRICTION name="ARMOREDCAVALRY" node="4">` at the foot of
/// `unit_graphics.xml`, and its rows are run854's packet's. All 29 of
/// run1434b's rounds from who=1's Armored Cavalry (unit type 220) at
/// their first frame, beside the figure's `GUY` pose on the same block,
/// are reproduced at the dump's `sx, sy` and `sz − z` by one of its eight
/// keys at some turret step; the seven fired on the move, by the
/// `CHAR_ATTACKWALK` key at the frame this crate's clock reads there.
/// Without the rows, `1/16`'s round of 3587 left the hull's square.
#[test]
fn run1434b_s_armored_cavalry_rounds_leave_from_its_turret() {
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3, ATTACKWALK};
    // (x, y, facing, sx, sy, sz − z, the walk key's frame or 0)
    let rows: [(i32, i32, i32, i32, i32, i32, u32); 29] = [
        (6418, 32805, -1_867_776_923, 6307, 32645, 138, 19), // 3587 1/16
        (6258, 33429, -1_902_209_484, 6182, 33245, 140, 4),  // 3602 1/16
        (6038, 33844, -1_413_723_124, 6020, 33655, 138, 19), // 3617 1/16
        (7016, 30893, -1_475_176_589, 6892, 30848, 140, 4),  // 3619 1/33
        (5273, 34075, -1_398_093_810, 5167, 33975, 140, 4),  // 3632 1/16
        (6641, 31304, -1_842_472_436, 6495, 31204, 138, 19), // 3634 1/33
        (4563, 34598, -1_541_668_864, 4490, 34431, 138, 19), // 3647 1/16
        (4136, 34877, -1_171_827_246, 4131, 34710, 140, 0),  // 3662 1/16
        (3884, 34620, -625_062_205, 3873, 34489, 138, 0),    // 3677 1/16
        (3446, 34488, -1_108_104_113, 3452, 34321, 140, 0),  // 3692 1/16
        (5698, 30683, -810_063_876, 5599, 30635, 140, 0),    // 3720 1/28
        (3336, 34488, -299_958_272, 3275, 34400, 140, 0),    // 3726 1/16
        (3336, 34488, -299_958_272, 3251, 34423, 140, 0),    // 3741 1/16
        (3336, 34488, -299_958_272, 3358, 34368, 140, 0),    // 3757 1/16
        (3336, 34488, -299_958_272, 3326, 34374, 140, 0),    // 3772 1/16
        (3770, 34204, -383_703_436, 3784, 34082, 140, 0),    // 3779 1/33
        (3336, 34488, -299_958_272, 3389, 34369, 140, 0),    // 3787 1/16
        (3336, 34488, -299_958_272, 3389, 34369, 140, 0),    // 3802 1/16
        (3336, 34488, -299_958_272, 3358, 34368, 140, 0),    // 3820 1/16
        (3336, 34488, -299_958_272, 3326, 34374, 140, 0),    // 3835 1/16
        (3336, 34488, -299_958_272, 3389, 34369, 140, 0),    // 3851 1/16
        (5606, 31406, -793_431_754, 5506, 31362, 140, 0),    // 3856 1/20
        (2589, 33785, 1_663_874_657, 2625, 33627, 138, 0),   // 3857 1/28
        (3336, 34488, -299_958_272, 3389, 34369, 140, 0),    // 3866 1/16
        (4782, 31254, -1_078_687_663, 4674, 31215, 138, 0),  // 3871 1/20
        (2568, 33768, 203_948_032, 2582, 33661, 140, 0),     // 3878 1/28
        (3336, 34488, -299_958_272, 3304, 34383, 140, 0),    // 3882 1/16
        (2568, 33768, 203_948_032, 2549, 33662, 140, 0),     // 3893 1/28
        (3336, 34488, -299_958_272, 3275, 34400, 140, 0),    // 3897 1/16
    ];
    let fits = |a: i8, t: u32, (x, y, facing, sx, sy, dz): (i32, i32, i32, i32, i32, i32)| {
        let Some(r) = release(170, a, t) else {
            return false;
        };
        let f = Angle(facing);
        release_offset(&r, f, 0).2 == dz
            && (-128..128).any(|i| {
                let (dx, dy, _) = release_offset(&r, f, i << 24);
                (x + dx, y + dy) == (sx, sy)
            })
    };
    for (x, y, facing, sx, sy, dz, walk) in rows {
        let shot = (x, y, facing, sx, sy, dz);
        if walk != 0 {
            assert!(
                fits(ATTACKWALK, walk, shot),
                "({x}, {y}) at {facing}: walk {walk}"
            );
            continue;
        }
        let any = [ATTACKWALK, ATTACK1, ATTACK2, ATTACK3]
            .into_iter()
            .any(|a| [4, 19].into_iter().any(|t| fits(a, t, shot)));
        assert!(any, "({x}, {y}) at {facing}: no key");
    }
    assert_eq!(
        node(170, 4),
        Some(Node {
            px: -4_480_000,
            py: 14_200_000
        })
    );
}

/// **The `MARK4`'s round leaves from its turret's muzzle** (item 3683,
/// `docs/AI.md` §347): run1352's `AMMO=5` record of who=1's `1/28`
/// `CHAR_ATTACK1` round of 2934 — the figure at (9432, 31128), z 619,
/// angle −1081802752 on block 2933 — leaves from (9218, 31078) at
/// height 757, which one turret step fits. Made to fail with piece 190's
/// rows out.
#[test]
fn the_mark4_s_round_leaves_from_its_turret() {
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3};
    for (a, t) in [(ATTACK1, 4), (ATTACK2, 21), (ATTACK3, 1)] {
        assert!(release(190, a, t).is_some(), "anim {a}");
    }
    assert_eq!(
        node(190, 4),
        Some(Node {
            px: -80_000,
            py: -11_530_000
        })
    );
    let r = release(190, ATTACK1, 4).unwrap();
    let f = Angle(-1_081_802_752);
    let fits: Vec<i32> = (-128..128)
        .filter(|i| {
            let (dx, dy, _) = release_offset(&r, f, i << 24);
            (9432 + dx, 31128 + dy) == (9218, 31078)
        })
        .collect();
    assert!(!fits.is_empty(), "a turret step: {fits:?}");
    assert_eq!(619 + release_offset(&r, f, 0).2, 757, "the dump's height");
}

/// **The original's pivot branch on every cell, for piece 190** (item
/// 3683): `tools/recomp/get_position.py … pivot 190 0 11 4 1` on
/// run1355's packet, one sweep for the three rows (their entries are the
/// same bits). `$RON_RELEASE_TABLE_190` is the directory holding
/// `pivot-190-n0-a11-t4.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-26-item-3683/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_190() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_190") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_190 (item 3683's oracle, run1355)");
        return;
    };
    let path = format!("{dir}/pivot-190-n0-a11-t4.txt");
    let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_190");
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACK1, 4),
        (crate::anim::ATTACK2, 21),
        (crate::anim::ATTACK3, 1),
    ] {
        let r = release(190, slot, time).expect("the row");
        assert_eq!(r.node, 0);
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

/// **The MLRS's rockets leave from its rack, past its one restriction**
/// (item 3975, `docs/GOLDEN.md` §177): node 1 is past the
/// `<RESTRICTION node="4">`'s count, so the row's entry alone turns with
/// the figure. Each cell is a sweep's (`deg 0 x y z`), and the last is
/// run1542's own round of 649 from `2/155`, facing 1021181952 at
/// (17736, 16104, z 501): the dump's (17714, 16065, 741) on 651. Made to
/// fail with the rows out of `RELEASES`: no row, and the rocket leaves
/// the launcher's square.
#[test]
fn the_mlrs_s_rockets_leave_from_its_rack() {
    let alone = |t| {
        let r = release_node(223, crate::anim::ATTACK2, t, 1).expect("the row");
        assert_eq!(r.node, 1);
        r
    };
    for (t, d, xyz) in [
        (26, 180, (-41, 4, 226)),
        (26, 0, (41, -4, 226)),
        (5, 90, (17, 41, 240)),
        (20, 270, (-9, -41, 232)),
    ] {
        let r = Release {
            pivot: [0; 3],
            ..alone(t)
        };
        assert_eq!(release_offset_at(&r, d, 0), xyz, "t {t} d {d}");
    }
    let facing = Angle(1_021_181_952);
    let t = (5..=26).step_by(3).find(|&t| {
        release_point(&alone(t), facing, &[0; 4], 1) == (17714 - 17736, 16065 - 16104, 240)
    });
    assert!(
        t.is_some(),
        "no frame of the rack leaves where run1542's round did"
    );
}

/// **Every cell of piece 223's eight rows against the original**
/// (item 3975): `get_position.py … sweep 223 1 12 <time>` on run883b's
/// packet, the non-pivot arm at every whole degree. `$RON_RELEASE_TABLE_223`
/// is the directory holding `sweep-223-n1-a12-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-11-07-item-3975/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_223() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_223") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_223 (item 3975's oracle, run883b)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for time in [5, 8, 11, 14, 17, 20, 23, 26] {
        let path = format!("{dir}/sweep-223-n1-a12-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_223");
        let r = release_node(223, crate::anim::ATTACK2, time, 1).expect("the row");
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(5)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            let want = release_offset_at(&Release { pivot: [0; 3], ..r }, f[0], 0);
            if want != (f[2], f[3], f[4]) {
                off.push(format!(
                    "t{time} ({}, 0, {}, {}) z {}",
                    f[0], f[2], f[3], f[4]
                ));
            }
            rows += 1;
        }
    }
    assert_eq!(rows, 8 * 361, "{dir}");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The Main Battle Tank's round leaves from its muzzle** (item 3810,
/// `docs/GOLDEN.md` §152.6): three cells of each of piece 201's rows
/// from run1472's sweeps (`deg i t x y z`), the muzzle some 350 out and
/// 142 up. Made to fail with the rows out of `RELEASES`: no row, and
/// the round leaves the hull's square.
#[test]
fn the_main_battle_tank_s_round_leaves_from_its_muzzle() {
    for (slot, time, cells) in [
        (
            crate::anim::ATTACK1,
            11,
            [
                (0, 0, (0, 347, 142)),
                (144, 77, (227, -262, 142)),
                (224, 238, (-340, -71, 142)),
            ],
        ),
        (
            crate::anim::ATTACK2,
            23,
            [
                (0, 0, (0, 350, 142)),
                (144, 77, (229, -264, 142)),
                (224, 238, (-342, -72, 142)),
            ],
        ),
        (
            crate::anim::ATTACK3,
            27,
            [
                (0, 0, (0, 308, 140)),
                (144, 77, (202, -232, 140)),
                (224, 238, (-302, -63, 140)),
            ],
        ),
    ] {
        let r = release(201, slot, time).expect("piece 201's row");
        for (d1, t, want) in cells {
            assert_eq!(release_offset_at(&r, d1, t), want, "a{slot} ({d1}, {t})");
        }
    }
}

/// **Piece 201's three rows agree with the original on every cell**
/// (item 3810): `get_position.py … pivot 201 0 <anim> <time> 1` on
/// run1472's packet, one sweep a row (the muzzle's entries differ by
/// anim). `$RON_RELEASE_TABLE_201` is the directory holding
/// `pivot-201-n0-a11-t11.txt`, `-a12-t23` and `-a13-t27`, outside git
/// (`~/ron-data/lab-experiments/2026-11-03-item-3810/sweeps-2/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_201() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_201") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_201 (item 3810's oracle, run1472)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACK1, 11),
        (crate::anim::ATTACK2, 23),
        (crate::anim::ATTACK3, 27),
    ] {
        let path = format!("{dir}/pivot-201-n0-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_201");
        let r = release(201, slot, time).expect("the row");
        assert_eq!(r.node, 0);
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

/// **The Mongol Nomad's rounds leave from its pivot** (item 3581):
/// every Nomad round run1222's and run1224's `AMMO=5` windows print to
/// 9826, at the figure's pose on the block before the round's first
/// print — `(anim, time, guy x, y, angle, z, sx, sy, sz, turret
/// steps)`, the key this crate's release event names. Each pose's
/// round fits the steps listed and no other, and the height is the
/// dump's; run1224b's `GUYS=4` prints two of the turrets, and they are
/// the fitted steps.
#[test]
fn the_mongol_nomad_s_rounds_leave_from_its_pivot() {
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3, ATTACKWALK};
    // (anim, time, guy x, y, angle, z, sx, sy, sz, turret steps)
    type Shot = (i8, u32, i32, i32, i32, i32, i32, i32, i32, &'static [i32]);
    #[rustfmt::skip]
    let shots: [Shot; 10] = [
        (ATTACK3, 19, 7560, 27240, -1_787_953_152, 376, 7484, 27210, 660, &[0]),
        (ATTACKWALK, 18, 7524, 27864, -2_108_948_480, 411, 7467, 27925, 654, &[34]),
        (ATTACK1, 18, 7752, 28104, -1_939_144_704, 463, 7733, 28190, 678, &[-1]),
        (ATTACK3, 19, 7512, 28344, -2_033_123_328, 525, 7431, 28344, 809, &[0, 1]),
        (ATTACK2, 16, 7512, 27864, -2_108_948_480, 411, 7511, 27953, 628, &[2]),
        (ATTACK3, 19, 7752, 28104, -1_939_144_704, 463, 7671, 28094, 747, &[-1]),
        (ATTACK2, 16, 7512, 28344, -2_033_123_328, 525, 7507, 28433, 742, &[0]),
        (ATTACK1, 18, 7512, 27864, -2_108_948_480, 411, 7508, 27952, 626, &[2]),
        (ATTACK2, 16, 7752, 28104, -1_939_144_704, 463, 7735, 28191, 680, &[-1]),
        (ATTACK1, 18, 7512, 28344, -2_033_123_328, 525, 7505, 28431, 740, &[0]),
    ];
    for (a, t, x, y, f, z, sx, sy, sz, steps) in shots {
        let r = release(149, a, t).expect("every Nomad release row");
        let fits: Vec<i32> = (-128..128)
            .filter(|i| {
                let (dx, dy, _) = release_offset(&r, Angle(f), i << 24);
                (x + dx, y + dy) == (sx, sy)
            })
            .collect();
        assert_eq!(fits, steps, "anim {a} at ({x}, {y})");
        assert_eq!(z + release_offset(&r, Angle(f), 0).2, sz, "anim {a} height");
    }
    // run1224b's own turrets: `1/31` on 9745, `1/27` on 9760.
    for (a, t, x, y, f, turret, sx, sy) in [
        (
            ATTACKWALK,
            18,
            7524,
            27864,
            -2_108_948_480,
            581_828_608,
            7467,
            27925,
        ),
        (
            ATTACK1,
            18,
            7752,
            28104,
            -1_939_144_704,
            -65_536,
            7733,
            28190,
        ),
    ] {
        let (dx, dy, _) = release_offset(&release(149, a, t).unwrap(), Angle(f), turret);
        assert_eq!(
            (x + dx, y + dy),
            (sx, sy),
            "the dump's own turret, anim {a}"
        );
    }
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

/// **The Camel Raider's rounds leave from its pivot** (item 3770,
/// `docs/GOLDEN.md` §149.6): the two `0/17` rounds run1440's window
/// prints from their first frame, 801 and 825, beside the figure's `GUY`
/// pose — (17400, 15672), `angle` −1618280448, `z` 477, `turret_angles[0]`
/// 73138176 — are `CHAR_ATTACK3`'s and `CHAR_ATTACK2`'s at the dump's own
/// turret step 4 and no other. Made to fail with piece 143's rows out:
/// both leave the figure's own square.
#[test]
fn the_camel_raider_s_rounds_leave_from_its_pivot() {
    use crate::anim::{ATTACK2, ATTACK3};
    let (x, y, z, f) = (17400, 15672, 477, -1_618_280_448);
    for (a, t, sx, sy, sz) in [
        (ATTACK3, 18, 17282, 15746, 742),
        (ATTACK2, 13, 17274, 15736, 758),
    ] {
        let r = release(143, a, t).expect("every Camel Raider release row");
        let fits: Vec<i32> = (-128..128)
            .filter(|i| {
                let (dx, dy, _) = release_offset(&r, Angle(f), i << 24);
                (x + dx, y + dy) == (sx, sy)
            })
            .collect();
        assert_eq!(fits, [73_138_176 >> 24], "anim {a}");
        assert_eq!(
            z + release_offset(&r, Angle(f), 73_138_176).2,
            sz,
            "anim {a} height"
        );
    }
}

/// **The later Camel Raider's rounds leave from its pivot** (item
/// 3855, `docs/GOLDEN.md` §153.3): piece 144 reads piece 143's rows
/// ([`SAME_AS`]), and the two `0/14` `CHAR_ATTACK2` rounds the fourth
/// melee's windows print from their first block — run1485's 798 and
/// run1486's 1430 — leave, beside the figure's `GUY` pose on the block
/// before ((30456, 27384), `angle` −2023030784, `z` 17), from the
/// dump's point at the dump's own turret step and no other. Made to
/// fail with the row out: both leave the figure's own square.
#[test]
fn the_later_camel_raider_s_rounds_leave_from_its_pivot() {
    use crate::anim::ATTACK2;
    let (x, y, z, f) = (30456, 27384, 17, -2_023_030_784);
    for (turret, sx, sy) in [(-262_144, 30400, 27516), (-347_799_552, 30461, 27533)] {
        let r = release(144, ATTACK2, 13).expect("piece 144 reads piece 143's rows");
        assert_eq!(r, release(143, ATTACK2, 13).unwrap());
        let fits: Vec<i32> = (-128..128)
            .filter(|i| {
                let (dx, dy, _) = release_offset(&r, Angle(f), i << 24);
                (x + dx, y + dy) == (sx, sy)
            })
            .collect();
        assert!(fits.contains(&(turret >> 24)), "turret {turret}: {fits:?}");
        assert!(fits.len() <= 2, "turret {turret}: {fits:?}");
        assert_eq!(
            z + release_offset(&r, Angle(f), turret).2,
            298,
            "the dump's height"
        );
    }
    assert_eq!(node(144, 4), node(143, 4));
    assert_eq!(scale(144), scale(143));
}

/// **The Camel Archer's rounds leave from its pivot** (item 3785,
/// `docs/GOLDEN.md` §150.1): the two `0/17` rounds run1457's window
/// prints from their first frame, 675 and 700, beside the figure's `GUY`
/// pose on the frame before — (15864, 15672), `angle` −1613430784, `z`
/// 566 — are `CHAR_ATTACK2`'s at the dump's own turret step 1 (step 0
/// lands on the same point) and `CHAR_ATTACK1`'s at step 7 and no
/// other. The second is the round run1450's block 701 prints at (15828,
/// 15703, 835). Made to fail with piece 141's rows out: both leave the
/// figure's own square.
#[test]
fn the_camel_archer_s_rounds_leave_from_its_pivot() {
    use crate::anim::{ATTACK1, ATTACK2};
    let (x, y, z, f) = (15864, 15672, 566, -1_613_430_784);
    for (a, t, turret, sx, sy, sz) in [
        (ATTACK2, 13, 22_085_632, 15830, 15714, 833),
        (ATTACK1, 13, 128_581_632, 15828, 15703, 835),
    ] {
        let r = release(141, a, t).expect("every Camel Archer release row");
        let fits: Vec<i32> = (-128..128)
            .filter(|i| {
                let (dx, dy, _) = release_offset(&r, Angle(f), i << 24);
                (x + dx, y + dy) == (sx, sy)
            })
            .collect();
        assert!(fits.contains(&(turret >> 24)), "anim {a}: {fits:?}");
        assert!(fits.len() <= 2, "anim {a}: {fits:?}");
        assert_eq!(
            z + release_offset(&r, Angle(f), turret).2,
            sz,
            "anim {a} height"
        );
    }
}

/// **Piece 143, the Camel Raider: the original's release on every
/// cell** (item 3770): `tools/recomp/get_position.py … pivot 143 0
/// <anim> <time> 1` on run1440's packet, 361 facings × 256 turret steps
/// for each of the four rows, agrees with [`release_offset_at`] on every
/// cell, its [`Release::float_lands`] included; and its pivot node's
/// `sweep 143 4 0 0` with [`offset_at`] on every degree. The tables stay
/// outside git (`RON_RELEASE_TABLE_143`, the directory holding them).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_143() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_143") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_143 (item 3770's oracle, run1440)");
        return;
    };
    let read = |name: &str, cols: usize| -> Vec<Vec<i32>> {
        let path = format!("{dir}/{name}");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{path}"))
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .take(cols)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect()
            })
            .collect()
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACKWALK, 12),
        (crate::anim::ATTACK1, 12),
        (crate::anim::ATTACK2, 13),
        (crate::anim::ATTACK3, 18),
    ] {
        let r = release(143, slot, time).expect("the row");
        for f in read(&format!("pivot-143-a{slot}-t{time}.txt"), 6) {
            let (x, y, z) = release_offset_at(&r, f[0], f[2]);
            if (x, y, z) != (f[3], f[4], f[5]) {
                off.push(format!(
                    "{slot} ({}, {}, {}, {}) z {z}",
                    f[0], f[2], f[3], f[4]
                ));
            }
            rows += 1;
        }
    }
    assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
    for f in read("sweep-143-4-0-0.txt", 5) {
        if offset_at(143, 4, f[0]) != (f[2], f[3]) {
            off.push(format!("node {} ({}, {})", f[0], f[2], f[3]));
        }
    }
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **Piece 141, the Camel Archer: the original's release on every
/// cell** (item 3785): `tools/recomp/get_position.py … pivot 141 0
/// <anim> <time> 1` on run1457's packet, 361 facings × 256 turret steps
/// for each of the four rows, agrees with [`release_offset_at`] on every
/// cell, its [`Release::float_lands`] included; and its pivot node's
/// `sweep 141 4 0 0` with [`offset_at`] on every degree. The tables stay
/// outside git (`RON_RELEASE_TABLE_141`,
/// `~/ron-data/lab-experiments/2026-10-29-item-3785/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_141() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_141") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_141 (item 3785's oracle, run1457)");
        return;
    };
    let read = |name: &str, cols: usize| -> Vec<Vec<i32>> {
        let path = format!("{dir}/{name}");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{path}"))
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .take(cols)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect()
            })
            .collect()
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACKWALK, 13),
        (crate::anim::ATTACK1, 13),
        (crate::anim::ATTACK2, 13),
        (crate::anim::ATTACK3, 10),
    ] {
        let r = release(141, slot, time).expect("the row");
        for f in read(&format!("pivot-141-a{slot}-t{time}.txt"), 6) {
            let (x, y, z) = release_offset_at(&r, f[0], f[2]);
            if (x, y, z) != (f[3], f[4], f[5]) {
                off.push(format!(
                    "{slot} ({}, {}, {}, {}) z {z}",
                    f[0], f[2], f[3], f[4]
                ));
            }
            rows += 1;
        }
    }
    assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
    for f in read("sweep-141-4-0-0.txt", 5) {
        if offset_at(141, 4, f[0]) != (f[2], f[3]) {
            off.push(format!("node {} ({}, {})", f[0], f[2], f[3]));
        }
    }
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The Battleship's rounds leave from its three turrets** (item 3736,
/// `docs/AI.md` §365.1): every round of the Battleships `1/45` and
/// `1/46` run1395b's `AMMO=5` prints at its first frame, beside its
/// figure's `GUY` pose on the block before, is reproduced at the dump's
/// `sx, sy` and `sz − z` by piece 299's rows at some turret step — and
/// its height names the turret: 106 node 0, 105 node 2, 144 node 1.
/// Made to fail with piece 299's rows out: each round leaves the hull's
/// own square at `z` 0.
#[test]
fn run1395b_s_battleship_rounds_leave_from_its_turrets() {
    use crate::anim::{ATTACK1, ATTACK2};
    // (x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32); 32] = [
        (2808, 11832, -531_824_640, 2629, 11443, 106), // 3693 1/45
        (3144, 11640, -550_830_080, 3591, 11875, 105), // 3698 1/46
        (2808, 11832, -531_824_640, 3251, 12075, 105), // 3699 1/45
        (3144, 11640, -550_830_080, 3084, 11395, 144), // 3703 1/46
        (2808, 11832, -531_824_640, 2752, 11586, 144), // 3704 1/45
        (2328, 8904, -613_482_496, 2797, 9091, 105),   // 3803 1/46
        (2232, 9144, -626_196_480, 2704, 9323, 105),   // 3804 1/45
        (2328, 8904, -613_482_496, 2246, 8670, 144),   // 3808 1/46
        (2232, 9144, -626_196_480, 2146, 8911, 144),   // 3809 1/45
        (2040, 6840, -290_783_232, 2005, 6413, 106),   // 3887 1/46
        (1512, 6888, -267_124_736, 1499, 6460, 106),   // 3888 1/45
        (2040, 6840, -290_783_232, 2374, 7222, 105),   // 3893 1/46
        (1512, 6888, -267_124_736, 1826, 7287, 105),   // 3894 1/45
        (2040, 6840, -290_783_232, 2071, 6589, 144),   // 3898 1/46
        (1512, 6888, -267_124_736, 1556, 6639, 144),   // 3899 1/45
        (2040, 6840, -252_837_888, 2029, 6417, 106),   // 3928 1/46
        (1512, 6888, -229_572_608, 1513, 6460, 106),   // 3929 1/45
        (2040, 6840, -252_837_888, 2354, 7239, 105),   // 3934 1/46
        (1512, 6888, -229_572_608, 1813, 7300, 105),   // 3935 1/45
        (2040, 6840, -252_837_888, 2086, 6596, 144),   // 3939 1/46
        (1512, 6888, -229_572_608, 1564, 6641, 144),   // 3940 1/45
        (2040, 6840, -360_513_536, 1963, 6424, 106),   // 3969 1/46
        (1512, 6888, -323_747_840, 1455, 6464, 106),   // 3970 1/45
        (2040, 6840, -360_513_536, 2412, 7185, 105),   // 3975 1/46
        (1512, 6888, -323_747_840, 1866, 7252, 105),   // 3976 1/45
        (2040, 6840, -360_513_536, 2047, 6592, 144),   // 3980 1/46
        (1512, 6888, -323_747_840, 1532, 6641, 144),   // 3981 1/45
        (2040, 6840, -231_014_400, 2043, 6417, 106),   // 4009 1/46
        (1512, 6888, -209_518_592, 1529, 6465, 106),   // 4010 1/45
        (2040, 6840, -231_014_400, 2340, 7250, 105),   // 4015 1/46
        (1512, 6888, -209_518_592, 1797, 7308, 105),   // 4016 1/45
        (2040, 6840, -231_014_400, 2094, 6598, 144),   // 4020 1/46
    ];
    let keys = [1, 7, 12].map(|t| [(ATTACK1, t), (ATTACK2, t)]).concat();
    for (x, y, facing, sx, sy, dz) in rows {
        let f = Angle(facing);
        let fits: Vec<(i8, u32)> = keys
            .iter()
            .copied()
            .filter(|&(a, t)| {
                let Some(r) = release(299, a, t) else {
                    return false;
                };
                release_offset(&r, f, 0).2 == dz
                    && (-128..128).any(|i| {
                        let (dx, dy, _) = release_offset(&r, f, i << 24);
                        (x + dx, y + dy) == (sx, sy)
                    })
            })
            .collect();
        assert!(
            !fits.is_empty(),
            "({x}, {y}) at {facing} to ({sx}, {sy}, +{dz})"
        );
        let t = fits[0].1;
        assert!(fits.iter().all(|k| k.1 == t), "one turret: {fits:?}");
    }
}

/// **The original's pivot branch on every cell, for piece 299** (item
/// 3736): `tools/recomp/get_position.py … pivot 299 <node> <anim>
/// <time> 3` on run1165's packet, one sweep a row (the two anims'
/// entries differ in their bits). `$RON_RELEASE_TABLE_299` is the
/// directory holding `pivot-299-n<node>-a<anim>-t<time>.txt`, outside
/// git (`~/ron-data/lab-experiments/2026-10-27-item-3736/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_299() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_299") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_299 (item 3736's oracle, run1165)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for slot in [crate::anim::ATTACK1, crate::anim::ATTACK2] {
        for (node, time) in [(0, 1), (2, 7), (1, 12)] {
            let path = format!("{dir}/pivot-299-n{node}-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_299");
            let r = release(299, slot, time).expect("the row");
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
    }
    assert!(rows >= 3 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 200** (item
/// 3737): `tools/recomp/get_position.py … pivot 200 0 <anim> <time> 1`
/// on run1416's packet, one sweep a row. `$RON_RELEASE_TABLE_200` is the
/// directory holding `pivot-200-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-27-item-3737/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_200() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_200") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_200 (item 3737's oracle, run1416)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACK1, 4),
        (crate::anim::ATTACK2, 21),
        (crate::anim::ATTACK3, 1),
    ] {
        let path = format!("{dir}/pivot-200-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_200");
        let r = release(200, slot, time).expect("the row");
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

/// **run982's three Cavalry Archer rounds leave from piece 138's pivot
/// rows** (item 3633): each is the figure's pose on the block before
/// its first `AMMO` print (the rows `launch::MEASURED` held from item
/// 3347 until this item), and each lands on exactly one turret step,
/// `dz` exact. The two `1/39` loosed from one pose share step 22.
#[test]
fn run982_s_cavalry_archer_rounds_fit_piece_138_s_pivot_rows() {
    // (anim, starttime, x, y, facing, sx, sy, sz − z, the step)
    type Row = (i8, u32, i32, i32, i32, i32, i32, i32, i32);
    let rows: [Row; 3] = [
        (
            crate::anim::ATTACK3,
            20,
            9480,
            30936,
            -1_535_115_264,
            9401,
            30970,
            228,
            22,
        ),
        (
            crate::anim::ATTACK2,
            16,
            9480,
            30936,
            -1_535_115_264,
            9388,
            30958,
            208,
            22,
        ),
        (
            crate::anim::ATTACK1,
            18,
            7944,
            30840,
            -990_642_176,
            7860,
            30861,
            225,
            243,
        ),
    ];
    for (anim, time, x, y, facing, sx, sy, dz, step) in rows {
        let r = release(138, anim, time).expect("the row");
        let fits: Vec<i32> = (0..256)
            .filter(|&i| {
                let (ox, oy, oz) = release_offset(&r, Angle(facing), i << 24);
                (x + ox, y + oy, oz) == (sx, sy, dz)
            })
            .collect();
        assert_eq!(fits, vec![step], "anim {anim}");
    }
}

/// **Piece 842's rounds fit piece 138's pivot rows** (item 3658,
/// [`SAME_AS`]): run973e/f's four Horse Archer rounds of the Maya Great
/// Sahara seed — the fourth, `CHAR_ATTACK1` on 12287, the miss
/// `launch::MEASURED`'s planar row pinned at (8587, 30273) — and run1308b's
/// standing Aztec rounds, each the figure's pose on the block before its
/// first `AMMO` print against the round's `sx, sy, sz − z`, and each on the
/// turret steps listed.
#[test]
fn piece_842_s_horse_archer_rounds_fit_piece_138_s_pivot_rows() {
    // (anim, starttime, x, y, facing, sx, sy, sz − z, the steps)
    type Row = (i8, u32, i32, i32, i32, i32, i32, i32, &'static [i32]);
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3};
    #[rustfmt::skip]
    let rows: [Row; 9] = [
        (ATTACK3, 20, 8808, 31944, -730_923_008, 8725, 31923, 228, &[0]),
        (ATTACK1, 18, 8808, 31944, -730_923_008, 8729, 31908, 225, &[0]),
        (ATTACK2, 16, 8664, 30312, -704_315_392, 8573, 30286, 208, &[248]),
        (ATTACK1, 18, 8664, 30312, -704_315_392, 8581, 30287, 225, &[248]),
        (ATTACK2, 16, 5784, 30600, -494_338_048, 5715, 30536, 208, &[255]),
        (ATTACK2, 16, 5784, 30600, -494_338_048, 5757, 30510, 208, &[22]),
        (ATTACK3, 20, 5592, 31464, -369_426_432, 5530, 31405, 228, &[255]),
        (ATTACK1, 18, 5640, 31176, 334_168_064, 5674, 31096, 225, &[0]),
        (ATTACK2, 16, 5640, 31176, 334_168_064, 5676, 31089, 208, &[0]),
    ];
    for (anim, time, x, y, facing, sx, sy, dz, steps) in rows {
        let r = release(842, anim, time).expect("piece 842 reads 138's rows");
        let fits: Vec<i32> = (0..256)
            .filter(|&i| {
                let (ox, oy, oz) = release_offset(&r, Angle(facing), i << 24);
                (x + ox, y + oy, oz) == (sx, sy, dz)
            })
            .collect();
        assert_eq!(fits, steps, "({x}, {y}) anim {anim}");
    }
}

/// **The original's pivot branch on every cell, for piece 138, the
/// Cavalry Archer** (item 3633): `tools/recomp/get_position.py … pivot
/// 138 0 <anim> <time> 1` on run1293's packet for its four rows.
/// `$RON_RELEASE_TABLE_138` is the directory holding
/// `pivot-138-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-25-item-3633/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_138() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_138") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_138 (item 3633's oracle, run1293)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACKWALK, 18),
        (crate::anim::ATTACK1, 18),
        (crate::anim::ATTACK2, 16),
        (crate::anim::ATTACK3, 20),
    ] {
        let path = format!("{dir}/pivot-138-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_138");
        let r = release(138, slot, time).expect("the row");
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

/// **The original's pivot branch on every cell, for piece 169, the
/// German Armored Scout Car** (item 3738): `tools/recomp/get_position.py
/// … pivot 169 0 <anim> <time> 1` on run1417's packet for its sixteen
/// rows. `$RON_RELEASE_TABLE_169` is the directory holding
/// `pivot-169-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-27-item-3738/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_169() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_169") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_169 (item 3738's oracle, run1417)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for slot in [crate::anim::ATTACKWALK, crate::anim::ATTACK2] {
        for time in [3, 9, 12, 17, 20, 22, 24, 29] {
            let path = format!("{dir}/pivot-169-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_169");
            let r = release(169, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!(
                        "{slot} {time} {} {} {} {} {}",
                        f[0], f[2], f[3], f[4], f[5]
                    ));
                }
                rows += 1;
            }
        }
    }
    assert!(rows >= 16 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 170, the
/// Armored Cavalry** (item 3756): `tools/recomp/get_position.py … pivot
/// 170 0 <anim> <time> 1` on run854's packet for its eight rows.
/// `$RON_RELEASE_TABLE_170` is the directory holding
/// `pivot-170-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-28-item-3756/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_170() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_170") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_170 (item 3756's oracle, run854)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for slot in [
        crate::anim::ATTACKWALK,
        crate::anim::ATTACK1,
        crate::anim::ATTACK2,
        crate::anim::ATTACK3,
    ] {
        for time in [4, 19] {
            let path = format!("{dir}/pivot-170-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_170");
            let r = release(170, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!(
                        "A{slot}_T{time} ({}, {}, {}, {})",
                        f[0], f[2], f[3], f[4]
                    ));
                }
                rows += 1;
            }
        }
    }
    assert!(rows >= 8 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 204, the
/// `ABRAM`** (item 3758): `tools/recomp/get_position.py … pivot 204 0
/// <anim> <time> 1` on run1437's packet for its three gun rows.
/// `$RON_RELEASE_TABLE_204` is the directory holding
/// `pivot-204-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-28-item-3758/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_204() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_204") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_204 (item 3758's oracle, run1437)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACK1, 11),
        (crate::anim::ATTACK2, 23),
        (crate::anim::ATTACK3, 27),
    ] {
        let path = format!("{dir}/pivot-204-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_204");
        let r = release(204, slot, time).expect("the row");
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(6)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                off.push(format!(
                    "A{slot}_T{time} ({}, {}, {}, {})",
                    f[0], f[2], f[3], f[4]
                ));
            }
            rows += 1;
        }
    }
    assert!(rows >= 3 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The Abrams' tracers leave from their own node, turned by the
/// facing alone** (item 3760, `docs/AI.md` §372.4): `1/38`'s two
/// tracers of run1439, first printed on 4256 and 4262 with the figure
/// at (5304, 31512), angle −1503002624, `z` 448, leave from (5177,
/// 31589, 578) and (5176, 31589, 574) — the node-1 rows of frame 1 and
/// 7, past piece 204's one restriction, whatever the turret reads. Made
/// to fail with the pivot node kept, and with the turret applied.
#[test]
fn a_node_past_the_restriction_count_is_its_own_entry_alone() {
    let facing = Angle(-1_503_002_624);
    let turrets = [0x1234_5678, 0, 0, 0];
    for (time, want) in [(1, (-127, 77, 130)), (7, (-128, 77, 126))] {
        let r = release(204, crate::anim::ATTACK2, time).expect("the tracer's row");
        assert_eq!(r.node, 1);
        assert_eq!(release_point(&r, facing, &turrets, 1), want, "frame {time}");
        assert_eq!(release_point(&r, facing, &[0; 4], 1), want, "no turret");
    }
    // The gun, node 0, is governed: its turret turns it.
    let gun = release(204, crate::anim::ATTACK2, 23).expect("the gun's row");
    assert_ne!(
        release_point(&gun, facing, &turrets, 1),
        release_point(&gun, facing, &[0; 4], 1)
    );
}

/// **The original's `get_position` on every cell, for piece 204's four
/// tracers** (item 3760): `tools/recomp/get_position.py … pivot 204 1
/// <anim> <time> 1` on run1437's packet — the call `execute_game_events`
/// makes, the count 1 — reads one point per facing at all 256 turret
/// steps, and [`release_point`] reproduces it. `$RON_RELEASE_TABLE_204_TRACERS`
/// is the directory holding `pivot-204-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-28-item-3760/sweeps/`).
#[test]
fn the_original_s_tracer_agrees_on_every_cell_for_piece_204() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_204_TRACERS") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_204_TRACERS (item 3760's oracle, run1437)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACK2, 1),
        (crate::anim::ATTACK2, 7),
        (crate::anim::ATTACK3, 1),
        (crate::anim::ATTACK3, 7),
    ] {
        let path = format!("{dir}/pivot-204-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_204_TRACERS");
        let r = release(204, slot, time).expect("the row");
        let alone = Release { pivot: [0; 3], ..r };
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(6)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            if release_offset_at(&alone, f[0], 0) != (f[3], f[4], f[5]) {
                off.push(format!(
                    "A{slot}_T{time} ({}, {}, {}, {})",
                    f[0], f[2], f[3], f[4]
                ));
            }
            rows += 1;
        }
    }
    assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 199, the
/// Chi-Ha** (item 3745): `tools/recomp/get_position.py … pivot 199 0
/// <anim> <time> 1` on run1424's packet for its three rows.
/// `$RON_RELEASE_TABLE_199` is the directory holding
/// `pivot-199-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-28-item-3745/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_199() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_199") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_199 (item 3745's oracle, run1424)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACK1, 4),
        (crate::anim::ATTACK2, 21),
        (crate::anim::ATTACK3, 1),
    ] {
        let path = format!("{dir}/pivot-199-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_199");
        let r = release(199, slot, time).expect("the row");
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(6)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                off.push(format!(
                    "199 {slot} ({}, {}, {}, {})",
                    f[0], f[2], f[3], f[4]
                ));
            }
            rows += 1;
        }
    }
    assert!(rows >= 3 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 207, the
/// T80DES** (item 3743): `tools/recomp/get_position.py … pivot 207 0
/// <anim> <time> 1` on run1419b's packet for its three rows. `$RON_RELEASE_TABLE_207` is
/// the directory holding `pivot-<piece>-a<anim>-t<time>.txt`, outside
/// git (`~/ron-data/lab-experiments/2026-10-28-item-3743/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_207() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_207") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_207 (item 3743's oracle, run1419b)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for piece in [207] {
        for (slot, time) in [
            (crate::anim::ATTACK1, 11),
            (crate::anim::ATTACK2, 23),
            (crate::anim::ATTACK3, 27),
        ] {
            let path = format!("{dir}/pivot-{piece}-a{slot}-t{time}.txt");
            let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_207");
            let r = release(piece, slot, time).expect("the row");
            for line in table.lines() {
                let f: Vec<i32> = line
                    .split_whitespace()
                    .take(6)
                    .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                    .collect();
                if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                    off.push(format!(
                        "{piece} {slot} ({}, {}, {}, {})",
                        f[0], f[2], f[3], f[4]
                    ));
                }
                rows += 1;
            }
        }
    }
    assert!(rows >= 3 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 196, the
/// Sherman** (item 3766): `tools/recomp/get_position.py … pivot 196 0
/// <anim> <time> 1` on run1164's packet for its three rows.
/// `$RON_RELEASE_TABLE_196` is the directory holding
/// `pivot-196-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-28-item-3766/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_196() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_196") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_196 (item 3766's oracle, run1164)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACK1, 4),
        (crate::anim::ATTACK2, 21),
        (crate::anim::ATTACK3, 1),
    ] {
        let path = format!("{dir}/pivot-196-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_196");
        let r = release(196, slot, time).expect("the row");
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
    assert!(rows >= 3 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 208, the
/// Tiger Tank** (item 3738): `tools/recomp/get_position.py … pivot 208
/// 0 <anim> 3 1` on run1417's packet for its three rows.
/// `$RON_RELEASE_TABLE_208` is the directory holding
/// `pivot-208-a<anim>-t3.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-27-item-3738/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_208() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_208") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_208 (item 3738's oracle, run1417)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for slot in [
        crate::anim::ATTACK1,
        crate::anim::ATTACK2,
        crate::anim::ATTACK3,
    ] {
        let path = format!("{dir}/pivot-208-a{slot}-t3.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_208");
        let r = release(208, slot, 3).expect("the row");
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
    assert!(rows >= 3 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 150, the
/// Steppe Nomad** (item 3714): `tools/recomp/get_position.py … pivot
/// 150 0 <anim> <time> 1` on run1391's packet for its four rows.
/// `$RON_RELEASE_TABLE_150` is the directory holding
/// `pivot-150-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-27-item-3714/sweeps/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_150() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_150") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_150 (item 3714's oracle, run1391)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACKWALK, 18),
        (crate::anim::ATTACK1, 18),
        (crate::anim::ATTACK2, 16),
        (crate::anim::ATTACK3, 19),
    ] {
        let path = format!("{dir}/pivot-150-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_150");
        let r = release(150, slot, time).expect("the row");
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
    assert_eq!(
        node(1546, 4),
        Some(Node {
            px: -60_000,
            py: -1_480_000
        })
    );
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
    assert_eq!(
        node(841, 4),
        Some(Node {
            px: -60_000,
            py: -1_480_000
        })
    );
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
    assert_eq!(
        node(1898, 4),
        Some(Node {
            px: -60_000,
            py: -1_480_000
        })
    );
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
    assert_eq!(
        node(1545, 4),
        Some(Node {
            px: -60_000,
            py: -1_480_000
        })
    );
}

/// **The original's pivot branch on every cell, for piece 156** (item
/// 3588): `tools/recomp/get_position.py … pivot 156 0 <anim> <time> 1`
/// on run1235b's packet for its four rows. `$RON_RELEASE_TABLE_156` is
/// the directory holding `pivot-156-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-24-item-3588/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_156() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_156") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_156 (item 3588's oracle, run1235b)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACKWALK, 18),
        (crate::anim::ATTACK1, 18),
        (crate::anim::ATTACK2, 16),
        (crate::anim::ATTACK3, 18),
    ] {
        let path = format!("{dir}/pivot-156-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_156");
        let r = release(156, slot, time).expect("the row");
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(6)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                off.push(format!("{slot} ({}, {}, {}, {})", f[0], f[2], f[3], f[4]));
            }
            rows += 1;
        }
    }
    assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 157** (item
/// 3645): `tools/recomp/get_position.py … pivot 157 0 <anim> <time> 1`
/// on run1305b's packet for its four rows. `$RON_RELEASE_TABLE_157` is
/// the directory holding `pivot-157-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-26-item-3645/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_157() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_157") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_157 (item 3645's oracle, run1305b)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACKWALK, 12),
        (crate::anim::ATTACK1, 12),
        (crate::anim::ATTACK2, 13),
        (crate::anim::ATTACK3, 18),
    ] {
        let path = format!("{dir}/pivot-157-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_157");
        let r = release(157, slot, time).expect("the row");
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(6)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                off.push(format!("{slot} ({}, {}, {}, {})", f[0], f[2], f[3], f[4]));
            }
            rows += 1;
        }
    }
    assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The original's pivot branch on every cell, for piece 297** (item
/// 3655): `tools/recomp/get_position.py … pivot 297 <node> <anim> 3 2`
/// on run1185's packet for its three rows. `$RON_RELEASE_TABLE_297` is
/// the directory holding `pivot-297-a<anim>-t3.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-26-item-3655/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_297() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_297") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_297 (item 3655's oracle, run1185)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for slot in [
        crate::anim::ATTACK1,
        crate::anim::ATTACK2,
        crate::anim::ATTACK3,
    ] {
        let path = format!("{dir}/pivot-297-a{slot}-t3.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_297");
        let r = release(297, slot, 3).expect("the row");
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(6)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                off.push(format!("{slot} ({}, {}, {}, {})", f[0], f[2], f[3], f[4]));
            }
            rows += 1;
        }
    }
    assert!(rows >= 3 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **run1291b's Bomb Ketch `1/82`** (item 3655, `docs/AI.md` §337):
/// its round at the Citizen `0/4`, fired in the undumped gap before
/// block 3956. This crate's left the hull's own square, (3912, 12792)
/// at `z` 0; the original's (4013, 12593, 133) is a cell of the stern
/// mortar's sweep from that square at the figure's facing on 3956,
/// 323813376, and its height is the row's. Made to fail with piece
/// 297's rows out of `RELEASES`: no row, the hull's square.
#[test]
fn run1291b_s_bomb_ketch_fires_through_piece_297_s_pivot() {
    let r = release(297, crate::anim::ATTACK1, 3).expect("the Bomb Ketch's row");
    let d = release_rotation(Angle(323_813_376));
    let fits: Vec<i32> = (0..360)
        .filter(|&t| {
            let (x, y, _) = release_offset_at(&r, d, t);
            (3912 + x, 12792 + y) == (4013, 12593)
        })
        .collect();
    assert!(!fits.is_empty(), "no turret step reproduces 1/82's round");
    assert_eq!(release_offset_at(&r, d, 0).2, 133);
    assert_eq!(
        release(297, crate::anim::ATTACK2, 3).map(|r| release_offset_at(&r, d, 0).2),
        Some(130)
    );
    assert_eq!(
        node(297, 4),
        Some(Node {
            px: 0,
            py: -50_370_000
        })
    );
    assert_eq!(
        node(297, 5),
        Some(Node {
            px: 0,
            py: 5_860_000
        })
    );
}

/// **run1305's Musket Horse `1/54`** (item 3645, `docs/AI.md` §336):
/// its round at the Citizen `0/5`, fired on the move on 4181
/// (`CHAR_ATTACKWALK` at frame 12), from the figure's (8712, 27240) at
/// facing −1073741824 to the dump's `AMMO` (8560, 27252) — a cell of
/// the turret's sweep. Made to fail with the rows out of `RELEASES`:
/// the round leaves from the figure and flies 4 frames against 3.
#[test]
fn run1305_s_musket_horse_fires_on_the_move_through_piece_157_s_pivot() {
    let r = release(157, crate::anim::ATTACKWALK, 12).expect("the Musket Horse's row");
    let d = release_rotation(Angle(-1_073_741_824));
    let fits: Vec<i32> = (0..360)
        .filter(|&t| {
            let (x, y, _) = release_offset_at(&r, d, t);
            (8712 + x, 27240 + y) == (8560, 27252)
        })
        .collect();
    assert!(!fits.is_empty(), "no turret step reproduces 1/54's round");
    assert_eq!(
        node(157, 4),
        Some(Node {
            px: 80_000,
            py: -1_480_000
        })
    );
}

/// **run1235's Eagle Feather Horse `1/32`** (item 3588, `docs/AI.md`
/// §314): its round at the Citizen `0/5`, `CHAR_ATTACK3` at frame 18 on
/// 4127, from the figure's (5544, 10152, 34) at facing 56098816 to the
/// dump's `AMMO` (5484, 10096, 263) — the sweep's cell at the facing's
/// degree 184 and the turret's 316 (step 225). Made to fail with the rows out of
/// `RELEASES`: the round leaves from the figure.
#[test]
fn run1235_s_eagle_feather_horse_looses_through_piece_156_s_pivot() {
    let r = release(156, crate::anim::ATTACK3, 18).expect("the Eagle Feather Horse's row");
    assert_eq!(release_rotation(Angle(56_098_816)), 184);
    let (x, y, z) = release_offset_at(&r, 184, 316);
    assert_eq!((5544 + x, 10152 + y, 34 + z), (5484, 10096, 263));
    assert_eq!(
        node(156, 4),
        Some(Node {
            px: -60_000,
            py: -1_480_000
        })
    );
}

/// **The original's pivot branch on every cell, for piece 153** (item
/// 3632): `tools/recomp/get_position.py … pivot 153 0 <anim> <time> 1`
/// on run1294's packet for its four rows. `$RON_RELEASE_TABLE_153` is
/// the directory holding `pivot-153-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-10-25-item-3632/`).
#[test]
fn the_original_s_release_agrees_on_every_cell_for_piece_153() {
    let Ok(dir) = std::env::var("RON_RELEASE_TABLE_153") else {
        eprintln!("skipping: set RON_RELEASE_TABLE_153 (item 3632's oracle, run1294)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (slot, time) in [
        (crate::anim::ATTACKWALK, 18),
        (crate::anim::ATTACK1, 18),
        (crate::anim::ATTACK2, 16),
        (crate::anim::ATTACK3, 19),
    ] {
        let path = format!("{dir}/pivot-153-a{slot}-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_RELEASE_TABLE_153");
        let r = release(153, slot, time).expect("the row");
        for line in table.lines() {
            let f: Vec<i32> = line
                .split_whitespace()
                .take(6)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            if release_offset_at(&r, f[0], f[2]) != (f[3], f[4], f[5]) {
                off.push(format!("{slot} ({}, {}, {}, {})", f[0], f[2], f[3], f[4]));
            }
            rows += 1;
        }
    }
    assert!(rows >= 4 * 92_000, "{dir}: {rows} rows");
    assert!(off.is_empty(), "{} cells disagree: {off:?}", off.len());
}

/// **The Horde's nine dumped rounds** (item 3632, `docs/AI.md` §331.2):
/// every round run1292c and run1292b print from a Horde, beside its
/// figure's `GUY` pose on the block before the round's first print, is
/// one of piece 153's cells at the facing's degree — some key, some
/// turret step — and `sz − z` is the key's height. The turret is the
/// simulation's to get right and the windows compare it; this pins the
/// geometry. Made to fail with the rows out of `RELEASES`: every round
/// would leave from the figure.
#[test]
fn run1292_s_horde_rounds_leave_through_piece_153_s_pivot() {
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3, ATTACKWALK};
    // (x, y, facing, sx, sy, sz − z)
    let rounds: [(i32, i32, i32, i32, i32, i32); 9] = [
        (5592, 10920, -174_784_512, 5570, 10826, 235), // run1292c 4459 1/21 at 0/5
        (5592, 10920, -174_784_512, 5678, 10901, 285), // run1292c 4487
        (5544, 10536, -121_896_960, 5543, 10440, 233), // run1292b 4526
        (5544, 10536, -121_896_960, 5549, 10441, 233), // run1292b 4551
        (5544, 10536, -121_896_960, 5549, 10440, 235), // run1292b 4575
        (5544, 10536, -121_896_960, 5522, 10443, 233), // run1292b 4602
        (6888, 9288, 541_917_184, 6951, 9215, 235),    // run1292b 4697
        (5976, 10536, -449_642_496, 5959, 10443, 235), // run1292b 4730 1/23
        (7320, 8808, 494_338_048, 7387, 8865, 285),    // run1292b 4746
    ];
    for (x, y, facing, sx, sy, dz) in rounds {
        let d = release_rotation(Angle(facing));
        let hit = [
            (ATTACKWALK, 18),
            (ATTACK1, 18),
            (ATTACK2, 16),
            (ATTACK3, 19),
        ]
        .into_iter()
        .any(|(slot, time)| {
            let r = release(153, slot, time).expect("the Horde's row");
            (0..256)
                .any(|i| release_offset_at(&r, d, fast_degrees(i << 24)) == (sx - x, sy - y, dz))
        });
        assert!(
            hit,
            "({x}, {y}) at {facing} (degree {d}): ({sx}, {sy}, {dz})"
        );
    }
    assert_eq!(
        node(153, 4),
        Some(Node {
            px: 0,
            py: -3_380_000
        })
    );
}

/// **The Golden Horde's dumped rounds read piece 153's rows** (item
/// 3692, `docs/AI.md` §352.1): every round run1346c prints from a Golden
/// Horde's first frame, beside its figure's `GUY` pose — the block
/// before for the three that stand, the round's own block for the eight
/// that loose on the move (`CHAR_ATTACKWALK`, whose figure has stepped
/// by the release) — is one of piece 154's cells at the facing's degree — some
/// key, some turret step — and `sz − z` is the key's height. Two prints
/// whose launch is an earlier round's, at a target of −1, are a reused
/// slot and are left out. Made to fail with piece 154 out of
/// `SAME_AS`: every round would leave from the figure.
#[test]
fn run1346c_s_golden_horde_rounds_leave_through_piece_153_s_pivot() {
    use crate::anim::{ATTACK1, ATTACK2, ATTACK3, ATTACKWALK};
    // (x, y, z, facing, sx, sy, sz)
    let rounds: [(i32, i32, i32, i32, i32, i32, i32); 11] = [
        (6264, 28728, 480, -1_476_001_792, 6180, 28775, 715), // 4841 1/97 at 0/6
        (5976, 28248, 445, -1_642_528_768, 5917, 28325, 680), // 4845 1/109
        (7032, 28344, 269, -1_137_115_136, 6949, 28391, 504), // 4855 1/99
        (6641, 28971, 445, 1_582_825_472, 6586, 28991, 705),  // 4901 1/109
        (6006, 29631, 511, -1_586_495_488, 5920, 29630, 771), // 4902 1/108
        (5895, 30114, 493, -1_648_295_936, 5816, 30100, 753), // 4912 1/98
        (6342, 29724, 444, 2_105_212_928, 6273, 29733, 704),  // 4913 1/97
        (6131, 29338, 516, -1_073_808_640, 6050, 29297, 776), // 4921 1/110
        (5437, 30156, 548, -1_586_954_240, 5362, 30212, 808), // 4927 1/108
        (5335, 30550, 536, -1_197_015_040, 5242, 30565, 796), // 4937 1/98
        (5544, 29646, 567, -1_531_969_536, 5456, 29669, 827), // 4946 1/110
    ];
    let mut off = Vec::new();
    for (x, y, z, facing, sx, sy, sz) in rounds {
        let d = release_rotation(Angle(facing));
        let hit = [
            (ATTACKWALK, 18),
            (ATTACK1, 18),
            (ATTACK2, 16),
            (ATTACK3, 19),
        ]
        .into_iter()
        .any(|(slot, time)| {
            let r = release(154, slot, time).expect("the Golden Horde's row");
            (0..256).any(|i| {
                release_offset_at(&r, d, fast_degrees(i << 24)) == (sx - x, sy - y, sz - z)
            })
        });
        if !hit {
            off.push(format!(
                "({x}, {y}) at {facing} (degree {d}): ({sx}, {sy}, {sz})"
            ));
        }
    }
    assert!(off.is_empty(), "{off:#?}");
    assert_eq!(
        node(154, 4),
        Some(Node {
            px: 0,
            py: -3_380_000
        })
    );
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
        sim.roster
            .art
            .pivots
            .insert(BATTERY, [(4, (-180, 180))].into_iter().collect());
        sim.roster
            .art
            .lengths
            .insert((PIECE, crate::anim::ATTACK2), 20);
        sim.roster.art.releases.insert(
            PIECE,
            [(
                crate::anim::ATTACK2,
                vec![(4, node, false, false, false, false)],
            )]
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
