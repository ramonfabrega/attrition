use super::*;

/// **The American Information lobby's aircraft and TOWs leave from their
/// rows** (item 3760, `docs/AI.md` §372.4): every round of who=1's Jet
/// Fighters (piece 245), Strategic Bomber (255) and TOW (91) that
/// run1439 prints from its first block over 4220..4292, beside its
/// figure's `GUY` pose on that block — `(piece, x, y, angle, sx, sy,
/// sz − z)` — is reproduced by one of the piece's rows, and each row of
/// the two aircraft reproduces at least one. Made to fail with the
/// rows removed: every one leaves from the figure's own point.
/// **The Attack Helicopter's rockets leave from its two pods** (item
/// 3804, `docs/GOLDEN.md` §152.3): run1469's `2/15` at (13560, 16056),
/// angle 0, releases node 0's round from (13488, 16015) on 627 and
/// node 1's from (13638, 16029) on 633, both 41 over its figure. Made
/// to fail with the rows removed: both leave from the figure's point.
#[test]
fn run1469_s_attack_helicopter_rockets_leave_from_its_pods() {
    let at = Pos::new(13560, 16056);
    for (node, start, want) in [(0, 0, (13488, 16015)), (1, 6, (13638, 16029))] {
        let p = launch_point(at, Angle(0), 261, crate::anim::ATTACK2, start, node);
        assert_eq!((p.x, p.y), want, "node {node}");
        assert_eq!(
            release_dz(261, crate::anim::ATTACK2, start, node),
            Some(41),
            "node {node}"
        );
    }
}

/// **The Advanced Machine Gun's pair leaves from its muzzle** (item
/// 3813, `docs/GOLDEN.md` §152.7): run1473's `2/25` at (20520, 18360),
/// angle −449,576,960, looses frames 3 and 4 of `CHAR_ATTACK3` on 712
/// and 713 from (20432, 18244) and (20434, 18245), 98 and 99 over its
/// figure (`AMMO` `sz` 554 and 555 over the ground's 456). Made to fail
/// with the rows removed: both leave from the figure's point.
#[test]
fn run1473_s_advanced_machine_gun_pair_leaves_from_its_muzzle() {
    let (at, facing) = (Pos::new(20520, 18360), Angle(-449_576_960));
    for (start, want, dz) in [(3, (20432, 18244), 98), (4, (20434, 18245), 99)] {
        let p = launch_point(at, facing, 77, crate::anim::ATTACK3, start, 0);
        assert_eq!((p.x, p.y), want, "frame {start}");
        assert_eq!(
            release_dz(77, crate::anim::ATTACK3, start, 0),
            Some(dz),
            "frame {start}"
        );
    }
}

/// **Piece 77's rows agree with the original at every heading** (item
/// 3813): `get_position.py … sweep 77 0 <anim> <time>` on run1472's
/// packet, one file a row, `deg i x y z` at every whole degree 0..=360.
/// `$RON_BAY_SWEEPS_77` is the directory holding
/// `sweep-77-n0-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-11-03-item-3813/sweeps/`).
#[test]
fn the_original_s_release_agrees_at_every_heading_for_piece_77() {
    let Ok(dir) = std::env::var("RON_BAY_SWEEPS_77") else {
        eprintln!("skipping: set RON_BAY_SWEEPS_77 (item 3813's oracle, run1472)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for &(_, a, t, b) in BAYS.iter().filter(|r| r.0 == 77) {
        let path = format!("{dir}/sweep-77-n0-a{a}-t{t}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_BAY_SWEEPS_77");
        for line in table.lines() {
            let f: Vec<i64> = line
                .split_whitespace()
                .take(5)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            let (x, y) = b.offset((f[0] + 180) % 360);
            if (x, y, i64::from(b.dz)) != (f[2], f[3], f[4]) {
                off.push(format!(
                    "a{a} t{t} deg {}: {x},{y} against {},{}",
                    f[0], f[2], f[3]
                ));
            }
            rows += 1;
        }
    }
    assert_eq!(rows, 31 * 361, "{dir}");
    assert!(off.is_empty(), "{} headings disagree: {off:?}", off.len());
}

/// **The two Fighter Bomber lines loose from their wings** (item 3829,
/// `docs/GOLDEN.md` §152.12): run1477's Fighter Bomber `0/15` (piece
/// 258) at (15096, 15672), angle 1,431,655,765, looses its pair of
/// block 822 from (15179, 15597) and (15074, 15782), 26 below its
/// figure; run1468's Jet Fighter Bomber `2/18` (piece 259) at (15864,
/// 16632), the same angle, looses block 801's from (15657, 16725) and
/// (15843, 16406), 71 and 70 below (parked 3816a). Made to fail with
/// the rows removed: every round leaves from the figure's point.
#[test]
fn the_fighter_bombers_rounds_leave_from_their_wings() {
    let facing = Angle(1_431_655_765);
    for (piece, at, node, want, dz) in [
        (258, (15096, 15672), 0, (15179, 15597), -26),
        (258, (15096, 15672), 1, (15074, 15782), -26),
        (259, (15864, 16632), 0, (15657, 16725), -71),
        (259, (15864, 16632), 1, (15843, 16406), -70),
    ] {
        let p = launch_point(
            Pos::new(at.0, at.1),
            facing,
            piece,
            crate::anim::ATTACK2,
            1,
            node,
        );
        assert_eq!((p.x, p.y), want, "piece {piece} node {node}");
        assert_eq!(
            release_dz(piece, crate::anim::ATTACK2, 1, node),
            Some(dz),
            "piece {piece} node {node}"
        );
    }
}

/// **Pieces 258 and 259 agree with the original at every heading**
/// (item 3829): `get_position.py … sweep <piece> <node> 12 <time>` on
/// run1472's packet, one file a node. `$RON_BAY_SWEEPS_3829` is the
/// directory, outside git
/// (`~/ron-data/lab-experiments/2026-11-04-item-3829/sweeps/`).
#[test]
fn the_original_s_release_agrees_at_every_heading_for_pieces_258_and_259() {
    let Ok(dir) = std::env::var("RON_BAY_SWEEPS_3829") else {
        eprintln!("skipping: set RON_BAY_SWEEPS_3829 (item 3829's oracle, run1472)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for (piece, node, time) in [(258, 0, 1), (258, 1, 1), (259, 0, 9), (259, 1, 4)] {
        let b = bay(piece, crate::anim::ATTACK2, 1, node).expect("a row");
        let path = format!("{dir}/sweep-{piece}-n{node}-a12-t{time}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_BAY_SWEEPS_3829");
        for line in table.lines() {
            let f: Vec<i64> = line
                .split_whitespace()
                .take(5)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            let (x, y) = b.offset((f[0] + 180) % 360);
            if (x, y, i64::from(b.dz)) != (f[2], f[3], f[4]) {
                off.push(format!("{piece} n{node} deg {}: {x},{y}", f[0]));
            }
            rows += 1;
        }
    }
    assert_eq!(rows, 4 * 361, "{dir}");
    assert!(off.is_empty(), "{} headings disagree: {off:?}", off.len());
}

/// **The MG42's burst leaves from its muzzle** (item 3827,
/// `docs/GOLDEN.md` §152.11): run1477's `0/18` at (17400, 15672),
/// angle 593,756,160, looses frames 1, 3 and 5 of `CHAR_ATTACK2` on
/// the Anti-Tank Missile `1/22` from (17510, 15576), (17508, 15575)
/// and (17505, 15573), 88, 93 and 98 over its figure (`AMMO` `sz` 565,
/// 570 and 575 over the ground's 477; blocks 816, 818 and 820). Made to
/// fail with the rows removed: each leaves from the figure's point.
#[test]
fn run1477_s_mg42_burst_leaves_from_its_muzzle() {
    let (at, facing) = (Pos::new(17400, 15672), Angle(593_756_160));
    for (start, want, dz) in [
        (1, (17510, 15576), 88),
        (3, (17508, 15575), 93),
        (5, (17505, 15573), 98),
    ] {
        let p = launch_point(at, facing, 80, crate::anim::ATTACK2, start, 0);
        assert_eq!((p.x, p.y), want, "frame {start}");
        assert_eq!(
            release_dz(80, crate::anim::ATTACK2, start, 0),
            Some(dz),
            "frame {start}"
        );
    }
}

/// **Piece 80's rows agree with the original at every heading** (item
/// 3827, `docs/GOLDEN.md` §152.11): `get_position.py … sweep 80 0
/// <anim> <time>` on run1472's packet, as piece 77's.
/// `$RON_BAY_SWEEPS_80` is the directory holding
/// `sweep-80-n0-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-11-04-item-3827/sweeps/`).
#[test]
fn the_original_s_release_agrees_at_every_heading_for_piece_80() {
    let Ok(dir) = std::env::var("RON_BAY_SWEEPS_80") else {
        eprintln!("skipping: set RON_BAY_SWEEPS_80 (item 3827's oracle, run1472)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for &(_, a, t, b) in BAYS.iter().filter(|r| r.0 == 80) {
        let path = format!("{dir}/sweep-80-n0-a{a}-t{t}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_BAY_SWEEPS_80");
        for line in table.lines() {
            let f: Vec<i64> = line
                .split_whitespace()
                .take(5)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            let (x, y) = b.offset((f[0] + 180) % 360);
            if (x, y, i64::from(b.dz)) != (f[2], f[3], f[4]) {
                off.push(format!(
                    "a{a} t{t} deg {}: {x},{y} against {},{}",
                    f[0], f[2], f[3]
                ));
            }
            rows += 1;
        }
    }
    assert_eq!(rows, 27 * 361, "{dir}");
    assert!(off.is_empty(), "{} headings disagree: {off:?}", off.len());
}

/// **The Recoilless Guns loose from their muzzles** (item 3834,
/// `docs/GOLDEN.md` §152.13): run1479's `0/20` at (18312, 15672),
/// angle −181,600,256, looses frame 15 of `CHAR_ATTACK3` on 884 from
/// (18319, 15581, 586); `0/21` at (18168, 15528), −101,777,408, frame
/// 22 of `CHAR_ATTACK2` on 891 from (18179, 15450, 574); and `0/19` at
/// (18168, 15672), −87,359,488, the same frame on 892 from (18182,
/// 15594, 583) — 73, 77 and 77 over the ground's 513, 497 and 506
/// (`AMMO` blocks 885, 892 and 893, `GUY` angle and `cur_anim`). Made
/// to fail with the rows removed: each leaves from the figure's point.
#[test]
fn run1479_s_recoilless_rounds_leave_from_their_muzzles() {
    for (at, facing, anim, start, want, dz) in [
        (
            (18312, 15672),
            -181_600_256,
            crate::anim::ATTACK3,
            15,
            (18319, 15581),
            73,
        ),
        (
            (18168, 15528),
            -101_777_408,
            crate::anim::ATTACK2,
            22,
            (18179, 15450),
            77,
        ),
        (
            (18168, 15672),
            -87_359_488,
            crate::anim::ATTACK2,
            22,
            (18182, 15594),
            77,
        ),
    ] {
        let p = launch_point(Pos::new(at.0, at.1), Angle(facing), 94, anim, start, 0);
        assert_eq!((p.x, p.y), want, "{at:?}");
        assert_eq!(release_dz(94, anim, start, 0), Some(dz), "{at:?}");
    }
}

/// **Piece 94's rows agree with the original at every heading** (item
/// 3834, `docs/GOLDEN.md` §152.13): `get_position.py … sweep 94 0
/// <anim> <time>` on run1472's packet, as piece 80's.
/// `$RON_BAY_SWEEPS_94` is the directory holding
/// `sweep-94-n0-a<anim>-t<time>.txt`, outside git
/// (`~/ron-data/lab-experiments/2026-11-04-item-3834/sweeps/`).
#[test]
fn the_original_s_release_agrees_at_every_heading_for_piece_94() {
    let Ok(dir) = std::env::var("RON_BAY_SWEEPS_94") else {
        eprintln!("skipping: set RON_BAY_SWEEPS_94 (item 3834's oracle, run1472)");
        return;
    };
    let (mut rows, mut off) = (0, Vec::new());
    for &(_, a, t, b) in BAYS.iter().filter(|r| r.0 == 94) {
        let path = format!("{dir}/sweep-94-n0-a{a}-t{t}.txt");
        let table = std::fs::read_to_string(&path).expect("RON_BAY_SWEEPS_94");
        for line in table.lines() {
            let f: Vec<i64> = line
                .split_whitespace()
                .take(5)
                .map(|x| x.parse().unwrap_or_else(|_| panic!("{line}")))
                .collect();
            let (x, y) = b.offset((f[0] + 180) % 360);
            if (x, y, i64::from(b.dz)) != (f[2], f[3], f[4]) {
                off.push(format!(
                    "a{a} t{t} deg {}: {x},{y} against {},{}",
                    f[0], f[2], f[3]
                ));
            }
            rows += 1;
        }
    }
    assert_eq!(rows, 3 * 361, "{dir}");
    assert!(off.is_empty(), "{} headings disagree: {off:?}", off.len());
}

#[test]
fn run1439_s_aircraft_and_tow_rounds_leave_from_their_rows() {
    type Row = (i32, i32, i32, i32, i32, i32, i32);
    const M: &[Row] = &[
        (245, 4703, 30220, -1_327_890_432, 4889, 30343, -36),
        (245, 4315, 30372, -1_327_300_608, 4363, 30154, -36),
        (245, 4712, 30217, -1_327_759_360, 4898, 30340, -36),
        (245, 4287, 30382, -1_327_693_824, 4335, 30164, -36),
        (245, 1538, 30399, 891_563_984, 1368, 30254, -36),
        (245, 1971, 30487, 1_337_763_188, 1787, 30613, -36),
        (245, 1591, 30380, 891_563_984, 1421, 30235, -36),
        (255, 3717, 30596, -1_335_164_928, 3805, 30661, 0),
        (255, 3717, 30596, -1_335_164_928, 3732, 30489, 0),
        (255, 3642, 30626, -1_335_164_928, 3730, 30691, 0),
        (255, 3567, 30656, -1_335_230_464, 3582, 30549, 0),
        (255, 3492, 30686, -1_335_361_536, 3580, 30751, 0),
        (255, 3417, 30716, -1_335_492_608, 3432, 30609, 0),
        (255, 3267, 30777, -1_332_805_632, 3355, 30842, 0),
        (255, 3192, 30807, -1_331_232_768, 3207, 30700, 0),
        (255, 3117, 30836, -1_320_157_184, 3203, 30903, 0),
        (255, 3042, 30865, -1_320_157_184, 3059, 30758, 0),
        (255, 2967, 30894, -1_320_157_184, 3053, 30961, 0),
        (255, 2817, 30952, -1_320_157_184, 2834, 30845, 0),
        (91, 5832, 31320, -950_730_752, 5742, 31270, 147),
    ];
    let keys = |piece: i32| -> Vec<(i8, u32, i8)> {
        match piece {
            91 => vec![
                (crate::anim::ATTACK1, 17, 0),
                (crate::anim::ATTACK2, 7, 0),
                (crate::anim::ATTACK3, 13, 0),
            ],
            _ => vec![(crate::anim::ATTACK2, 1, 0), (crate::anim::ATTACK2, 1, 1)],
        }
    };
    let mut hit = std::collections::BTreeSet::new();
    for &(piece, x, y, angle, sx, sy, dz) in M {
        let at = Pos::new(x, y);
        let fits: Vec<_> = keys(piece)
            .into_iter()
            .filter(|&(a, t, n)| {
                launch_point(at, Angle(angle), piece, a, t, n) == Pos::new(sx, sy)
                    && release_dz(piece, a, t, n) == Some(dz)
            })
            .collect();
        assert!(
            !fits.is_empty(),
            "piece {piece} at ({x}, {y}): no row lands ({sx}, {sy})"
        );
        hit.extend(fits.into_iter().map(|k| (piece, k)));
    }
    for piece in [245, 255] {
        for n in [0, 1] {
            assert!(
                hit.contains(&(piece, (crate::anim::ATTACK2, 1, n))),
                "piece {piece} node {n} reproduces none"
            );
        }
    }
}

/// **The Royal Atl-Atls throw from piece 36's bays** (item 3613): all
/// ten rounds run1266 and run1256b print from their first frame, each
/// beside its figure's `GUY` pose on the block before, reproduced at the
/// dump's `sx, sy` and `sz − z` by exactly one of the piece's three
/// attack keys (the dump does not print which the figure played).
/// **The Apedemak Archers loose from piece 126's bays** (item 3759):
/// the three rounds run1438 prints from their first frame, 675, each
/// beside its figure's `GUY` pose on the block before, reproduced at
/// the dump's `sx, sy` and `sz − z` by exactly one of the piece's three
/// attack keys, and the six rounds of block 801 in run1413 by the pose
/// that block prints. Made to fail with the rows out of `BAYS`: each
/// round leaves the figure's own square.
#[test]
fn run1438_s_apedemak_archers_leave_from_piece_126_s_bays() {
    // (x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32); 6] = [
        (16632, 15672, -205_914_112, 16634, 15605, 101), // 674 0/14
        (16776, 15672, -304_218_112, 16768, 15606, 101), // 674 0/15
        (16680, 15816, -208_207_872, 16682, 15749, 101), // 674 0/16
        (16632, 15672, -451_149_824, 16626, 15601, 153), // run1413 801 0/14
        (16776, 15672, -507_838_464, 16819, 15570, 116), // run1413 801 0/15
        (16680, 15816, -427_360_256, 16735, 15720, 116), // run1413 801 0/16
    ];
    for (x, y, facing, sx, sy, dz) in rows {
        let keys: Vec<i8> = [11, 12, 13]
            .into_iter()
            .filter(|&anim| {
                let at = launch_point(Pos::new(x, y), Angle(facing), 126, anim, 15, 0);
                (at.x, at.y) == (sx, sy) && release_dz(126, anim, 15, 0) == Some(dz)
            })
            .collect();
        assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
    }
}

/// **The Kushite Archers loose from piece 124's bays** (item 3785,
/// `docs/GOLDEN.md` §150.1): who=0's six rounds run1457 prints from
/// their first frame over 660..720, each beside its figure's `GUY` pose
/// on the frame before, reproduced at the dump's `sx, sy` and `sz − z`
/// by exactly one of the piece's three attack keys — `0/14`'s of 674
/// among them, which flies 14 frames from there and strikes the Phalanx
/// `1/19` on 687. Made to fail with the rows out of `BAYS`: each round
/// leaves the figure's own square.
#[test]
fn run1457_s_kushite_archers_leave_from_piece_124_s_bays() {
    // (x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32); 6] = [
        (15096, 15672, -311_230_464, 15167, 15587, 116), // 675 0/14
        (15240, 15672, -375_980_032, 15240, 15600, 153), // 675 0/15
        (15144, 15816, -301_793_280, 15153, 15745, 153), // 675 0/16
        (15096, 15672, -54_198_272, 15193, 15618, 116),  // 705 0/14
        (15240, 15672, -107_347_968, 15333, 15611, 116), // 705 0/15
        (15144, 15816, -66_125_824, 15160, 15751, 101),  // 705 0/16
    ];
    for (x, y, facing, sx, sy, dz) in rows {
        let keys: Vec<i8> = [11, 12, 13]
            .into_iter()
            .filter(|&anim| {
                let at = launch_point(Pos::new(x, y), Angle(facing), 124, anim, 15, 0);
                (at.x, at.y) == (sx, sy) && release_dz(124, anim, 15, 0) == Some(dz)
            })
            .collect();
        assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
    }
}

/// **The Royal Balamob Slingers loose from piece 39's bays** (item
/// 3785, `docs/GOLDEN.md` §150.1): who=0's six stones run1457 prints
/// from their first frame over 660..720, beside each figure's `GUY`
/// pose on the frame before, reproduced at the dump's `sx, sy` and `sz
/// − z` by exactly one of the piece's three attack keys. Made to fail
/// with the rows out of `BAYS`: each stone leaves the figure's own
/// square.
#[test]
fn run1457_s_royal_balamob_slingers_leave_from_piece_39_s_bays() {
    // (x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32); 6] = [
        (16776, 15672, -197_001_216, 16772, 15529, 172), // 682 0/19
        (16680, 15816, -96_468_992, 16698, 15674, 172),  // 682 0/20
        (16632, 15672, -74_973_184, 16652, 15534, 161),  // 683 0/18
        (16776, 15672, -185_794_560, 16772, 15529, 172), // 715 0/19
        (16632, 15672, -26_476_544, 16662, 15535, 161),  // 717 0/18
        (16680, 15816, -49_086_464, 16702, 15670, 148),  // 718 0/20
    ];
    for (x, y, facing, sx, sy, dz) in rows {
        let keys: Vec<(i8, u32)> = [(11, 18), (12, 19), (13, 21)]
            .into_iter()
            .filter(|&(anim, t)| {
                let at = launch_point(Pos::new(x, y), Angle(facing), 39, anim, t, 0);
                (at.x, at.y) == (sx, sy) && release_dz(39, anim, t, 0) == Some(dz)
            })
            .collect();
        assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
    }
}

/// **The Arquebusiers loose from piece 752's bays** (item 3781,
/// `docs/GOLDEN.md` §149.8): who=2's four rounds run1451 prints from
/// their first frame over 905..960, each beside its figure's `GUY` pose
/// on the frame before, reproduced at the dump's `sx, sy` and `sz − z`
/// by exactly one of the piece's three attack keys — `2/16`'s of 953
/// among them, which flies 2 frames from there and kills `1/22` on 954.
/// Made to fail with the rows out of `BAYS`: each round leaves the
/// figure's own square.
#[test]
fn run1451_s_arquebusiers_leave_from_piece_752_s_bays() {
    // (x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32); 4] = [
        (15576, 16776, 208_994_304, 15628, 16654, 164), // 908 2/16
        (15096, 17208, 369_426_432, 15193, 17104, 151), // 911 2/15
        (15096, 17208, 541_917_184, 15226, 17127, 181), // 936 2/15
        (15576, 16776, 523_567_104, 15694, 16697, 151), // 954 2/16
    ];
    for (x, y, facing, sx, sy, dz) in rows {
        let keys: Vec<(i8, u32)> = [(11, 6), (12, 25), (13, 23)]
            .into_iter()
            .filter(|&(anim, t)| {
                let at = launch_point(Pos::new(x, y), Angle(facing), 752, anim, t, 0);
                (at.x, at.y) == (sx, sy) && release_dz(752, anim, t, 0) == Some(dz)
            })
            .collect();
        assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
    }
}

/// **The Spanish Riflemen loose from piece 50's bays** (item 3705):
/// two rounds of each attack key run1368b prints from a shooter's
/// first frame, beside its figure's `GUY` pose on the block before,
/// reproduced at the dump's `sx, sy` and `sz − z` by exactly one of the
/// piece's three attack keys. Made to fail with the rows out of
/// `BAYS`: each round leaves the figure's own square.
#[test]
fn run1368b_s_spanish_riflemen_leave_from_piece_50_s_bays() {
    // (x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32); 6] = [
        (4344, 33768, -249_954_304, 4315, 33650, 168), // 3198 1/54
        (4392, 33912, -361_758_720, 4343, 33801, 168), // 3264 1/53
        (4392, 33912, -361_758_720, 4328, 33795, 122), // 3229 1/53
        (4344, 33768, -377_946_112, 4278, 33652, 122), // 3243 1/54
        (4392, 33912, -361_758_720, 4342, 33802, 167), // 3310 1/53
        (4776, 34200, -428_736_512, 4714, 34096, 167), // 3283 1/55
    ];
    for (x, y, facing, sx, sy, dz) in rows {
        let keys: Vec<(i8, u32)> = [(11, 10), (12, 15), (13, 16)]
            .into_iter()
            .filter(|&(anim, t)| {
                let at = launch_point(Pos::new(x, y), Angle(facing), 50, anim, t, 0);
                (at.x, at.y) == (sx, sy) && release_dz(50, anim, t, 0) == Some(dz)
            })
            .collect();
        assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
    }
}

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

/// **The Mongol Musketeers loose from piece 1105's bays** (item 3692):
/// the three rounds run1346c prints from a Musketeer's first frame, each
/// beside its figure's `GUY` pose on the block before, reproduced at the
/// dump's `sx, sy` and `sz − z` by exactly one of the piece's three
/// attack keys. Made to fail with the rows out of `BAYS`: each round
/// leaves the figure's own square.
#[test]
fn run1346c_s_mongol_musketeers_leave_from_piece_1105_s_bays() {
    // (x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32); 3] = [
        (6456, 28584, -1_454_309_376, 6312, 28659, 178), // 4844 1/19
        (6552, 28728, -1_424_556_032, 6415, 28769, 218), // 4854 1/21
        (6312, 28344, -1_432_485_888, 6176, 28387, 218), // 4863 1/28
    ];
    for (x, y, facing, sx, sy, dz) in rows {
        let keys: Vec<(i8, u32)> = [(11, 1), (12, 7), (13, 4)]
            .into_iter()
            .filter(|&(anim, t)| {
                let at = launch_point(Pos::new(x, y), Angle(facing), 1105, anim, t, 0);
                (at.x, at.y) == (sx, sy) && release_dz(1105, anim, t, 0) == Some(dz)
            })
            .collect();
        assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
    }
}

/// **The Mongols' Crossbowmen loose from piece 1178's bays** (item
/// 3712): run1386's three bolts at the human's `0/2004` on 3981, 3983
/// and 3989, one of each attack key, each beside its figure's `GUY`
/// pose on the block before, reproduced at the dump's `sx, sy` and `sz −
/// z`. Made to fail with the rows out of `BAYS`: each bolt leaves the
/// figure's own square.
#[test]
fn run1386_s_mongol_crossbowmen_leave_from_their_bays() {
    type Round = (i8, u32, i32, i32, i32, i32, i32, i32, i32);
    const R: &[Round] = &[
        // `1/43`, 3981.
        (
            crate::anim::ATTACK2,
            3,
            6120,
            11208,
            19,
            -325_976_064,
            6103,
            11142,
            159,
        ),
        // `1/44`, 3983.
        (
            crate::anim::ATTACK1,
            5,
            5976,
            11208,
            15,
            -289_800_192,
            5950,
            11124,
            188,
        ),
        // `1/45`, 3989.
        (
            crate::anim::ATTACK3,
            6,
            6264,
            11208,
            23,
            -360_448_000,
            6238,
            11133,
            191,
        ),
    ];
    for &(anim, t, gx, gy, gz, ga, sx, sy, sz) in R {
        let got = launch_point(Pos::new(gx, gy), Angle(ga), 1178, anim, t, 0);
        let z = gz + release_dz(1178, anim, t, 0).unwrap_or(0);
        assert_eq!((got.x, got.y, z), (sx, sy, sz), "anim {anim} t {t}");
    }
}

/// **The Main Battle Tank looses from piece 203's bay** (item 3743):
/// run1421's `1/55` round of 3514, `CHAR_ATTACK2` frame 23, the figure
/// at (5352, 30792) and −371654656 with `z` 515, prints (5166, 30496,
/// 553) on 3515. Made to fail with the rows out of `BAYS`: the round
/// leaves the figure's own square.
#[test]
fn run1421_s_egyptian_main_battle_tank_leaves_from_its_bay() {
    let got = launch_point(
        Pos::new(5352, 30792),
        Angle(-371_654_656),
        203,
        crate::anim::ATTACK2,
        23,
        0,
    );
    let z = 515 + release_dz(203, crate::anim::ATTACK2, 23, 0).unwrap_or(0);
    assert_eq!((got.x, got.y, z), (5166, 30496, 553));
}

/// **The Japanese Bazooka looses from piece 1145's bay** (item 3745):
/// run1424b's `1/52` round of 3842, `CHAR_ATTACK2` frame 7, the figure
/// at (5400, 30744) and −1433075712 with `z` 514, prints (5299, 30765,
/// 664). Made to fail with the rows out of `BAYS`: the round leaves
/// the figure's own square.
#[test]
fn run1424b_s_japanese_bazooka_leaves_from_its_bay() {
    let got = launch_point(
        Pos::new(5400, 30744),
        Angle(-1_433_075_712),
        1145,
        crate::anim::ATTACK2,
        7,
        0,
    );
    let z = 514 + release_dz(1145, crate::anim::ATTACK2, 7, 0).unwrap_or(0);
    assert_eq!((got.x, got.y, z), (5299, 30765, 664));
}

/// **The Egyptian Infantry and Bazooka loose from pieces 52 and 89's
/// bays** (item 3747, `docs/AI.md` §371.1): run1426's `1/19` round of
/// 3723 and `1/54`'s of 3840, each from the figure's `GUY` pose on the
/// block before (3722, 3839) to the dump's `sx, sy, sz − z`, reproduced
/// by exactly one of the piece's attack keys. Made to fail with the
/// rows out of `BAYS`: each round leaves the figure's own square.
#[test]
fn run1426_s_egyptian_infantry_and_bazooka_leave_from_their_bays() {
    // (piece, x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32, i32); 2] = [
        (52, 6456, 31752, -544_014_336, 6417, 31701, 341 - 221), // 3723 1/19
        (89, 5880, 30648, -360_513_536, 5859, 30547, 612 - 462), // 3840 1/54
    ];
    for (piece, x, y, facing, sx, sy, dz) in rows {
        let attack: &[(i8, u32)] = if piece == 52 {
            &[(11, 12), (11, 13), (12, 4), (12, 14), (12, 24), (13, 12)]
        } else {
            &[(11, 17), (12, 7), (13, 13)]
        };
        let keys: Vec<(i8, u32)> = attack
            .iter()
            .copied()
            .filter(|&(anim, t)| {
                let at = launch_point(Pos::new(x, y), Angle(facing), piece, anim, t, 0);
                (at.x, at.y) == (sx, sy) && release_dz(piece, anim, t, 0) == Some(dz)
            })
            .collect();
        assert_eq!(
            keys.len(),
            1,
            "{piece} ({x}, {y}) at {facing}: keys {keys:?}"
        );
    }
}

/// **The Bantu Bazooka looses from piece 441's bay** (item 3748):
/// run1425's `1/86` round of about 4292, `CHAR_ATTACK1` frame 17, the
/// figure at (6648, 31224) and −1041694720 with `z` 267, prints
/// (6527, 31190, 394). Made to fail with the rows out of `BAYS`: the
/// round leaves the figure's own square.
#[test]
fn run1425_s_bantu_bazooka_leaves_from_its_bay() {
    let got = launch_point(
        Pos::new(6648, 31224),
        Angle(-1_041_694_720),
        441,
        crate::anim::ATTACK1,
        17,
        0,
    );
    let z = 267 + release_dz(441, crate::anim::ATTACK1, 17, 0).unwrap_or(0);
    assert_eq!((got.x, got.y, z), (6527, 31190, 394));
}

/// **The Japanese Infantry looses from piece 1108's bay** (item 3745):
/// run1403b's `1/18` rounds of 3157, `CHAR_ATTACK2` frame 14, the
/// figure at (4200, 34056) and −231997440 with `z` 489, print (4203,
/// 33991, 607) on 3158. Made to fail with the rows out of `BAYS`: the
/// round leaves the figure's own square.
#[test]
fn run1403b_s_japanese_infantry_leaves_from_its_bay() {
    let got = launch_point(
        Pos::new(4200, 34056),
        Angle(-231_997_440),
        1108,
        crate::anim::ATTACK2,
        14,
        0,
    );
    let z = 489 + release_dz(1108, crate::anim::ATTACK2, 14, 0).unwrap_or(0);
    assert_eq!((got.x, got.y, z), (4203, 33991, 607));
}

/// **The Egyptian Assault Infantry looses from piece 54's bay** (item
/// 3740): run1399b's `1/34` round of 3204, `CHAR_ATTACK1` frame 12, the
/// figure at (5640, 31800) and −1116995584 with `z` 364 on block 3204,
/// prints (5530, 31784, 533) on 3205. Made to fail with the rows out of
/// `BAYS`: the round leaves the figure's own square.
#[test]
fn run1399b_s_egyptian_assault_infantry_leaves_from_its_bay() {
    let got = launch_point(
        Pos::new(5640, 31800),
        Angle(-1_116_995_584),
        54,
        crate::anim::ATTACK1,
        12,
        0,
    );
    let z = 364 + release_dz(54, crate::anim::ATTACK1, 12, 0).unwrap_or(0);
    assert_eq!((got.x, got.y, z), (5530, 31784, 533));
}

/// **The Chinese Manchu Musketeers and Fusiliers loose from pieces 66
/// and 1142's bays** (item 3704): one round of each attack key run1350b
/// prints from a shooter's first frame, beside its figure's `GUY` pose
/// on the block before, reproduced at the dump's `sx, sy` and `sz − z`
/// by exactly one of the piece's three attack keys. Made to fail with
/// the rows out of `BAYS`: each round leaves the figure's own square.
#[test]
fn run1350b_s_chinese_musketeers_and_fusiliers_leave_from_their_bays() {
    // (piece, x, y, facing, sx, sy, sz − z)
    let rows: [(i32, i32, i32, i32, i32, i32, i32); 6] = [
        (66, 6216, 30360, -506_855_424, 6147, 30247, 214), // 3198 1/47
        (66, 5784, 31416, -655_032_320, 5661, 31322, 182), // 3244 1/35
        (66, 6216, 30840, -419_037_184, 6137, 30710, 178), // 3205 1/48
        (1142, 4872, 29208, -575_012_864, 4813, 29087, 112), // 3359 1/52
        (1142, 5112, 29112, -1_044_840_448, 4998, 29088, 117), // 3273 1/52
        (1142, 5976, 29784, -627_376_128, 5917, 29680, 111), // 3205 1/51
    ];
    for (piece, x, y, facing, sx, sy, dz) in rows {
        let attack: [(i8, u32); 3] = if piece == 66 {
            [(11, 1), (12, 7), (13, 4)]
        } else {
            [(11, 6), (12, 6), (13, 8)]
        };
        let keys: Vec<(i8, u32)> = attack
            .into_iter()
            .filter(|&(anim, t)| {
                let at = launch_point(Pos::new(x, y), Angle(facing), piece, anim, t, 0);
                (at.x, at.y) == (sx, sy) && release_dz(piece, anim, t, 0) == Some(dz)
            })
            .collect();
        assert_eq!(
            keys.len(),
            1,
            "{piece} ({x}, {y}) at {facing}: keys {keys:?}"
        );
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

/// **The Persian Javelineers throw from piece 1793's bay** (item 3663):
/// run1318b's `1/312` at the Citizen `0/12` on 10931, the figure at
/// (5064, 31368, 499) facing −2023817216 (its `GUY` record on 10930)
/// and the round at (5054, 31417, 669). Made to fail with the rows out of `BAYS`: the javelin
/// leaves the figure's own square, as ours did.
#[test]
fn run1318b_s_persian_javelin_leaves_from_piece_1793_s_bay() {
    let at = launch_point(
        Pos::new(5064, 31368),
        Angle(-2_023_817_216),
        1793,
        crate::anim::ATTACK2,
        6,
        0,
    );
    assert_eq!((at.x, at.y), (5054, 31417));
    assert_eq!(release_dz(1793, crate::anim::ATTACK2, 6, 0), Some(170));
}

/// **The Persian Archers loose from piece 1881's bays** (item 3663):
/// run1318b's three arrows at the Citizen `0/11` over 10980..10989, each
/// the figure's `GUY` pose on the block before the round's first print
/// and the dump's `sx, sy, sz − z`. Made to fail with the rows out of
/// `BAYS`: the arrows leave the figure's own square, as ours did.
#[test]
fn run1318b_s_persian_arrows_leave_from_piece_1881_s_bays() {
    // (anim, starttime, x, y, facing, sx, sy, sz − z)
    type Row = (i8, u32, i32, i32, i32, i32, i32, i32);
    let rows: [Row; 3] = [
        (11, 12, 5592, 30984, -534_249_472, 5528, 30923, 163),
        (12, 9, 5544, 31368, -455_737_344, 5490, 31300, 164),
        (12, 9, 6024, 30840, -733_413_376, 5947, 30799, 164),
    ];
    for (anim, start, x, y, facing, sx, sy, dz) in rows {
        let at = launch_point(Pos::new(x, y), Angle(facing), 1881, anim, start, 0);
        assert_eq!((at.x, at.y), (sx, sy), "({x}, {y}) ({anim}, {start})");
        assert_eq!(release_dz(1881, anim, start, 0), Some(dz));
    }
}

/// **The Anusiya and the Persian Catapult loose from their bays** (item
/// 3663): run1327b's rounds at `0/2000`, each the figure's `GUY` pose
/// on the block before the round's first print and the dump's `sx, sy,
/// sz − z`. Made to fail with the rows out of `BAYS`.
#[test]
fn run1327b_s_anusiya_and_catapult_rounds_leave_from_their_bays() {
    // (piece, anim, starttime, x, y, facing, sx, sy, sz − z)
    type Row = (i32, i8, u32, i32, i32, i32, i32, i32, i32);
    let rows: [Row; 3] = [
        (113, 12, 14, 3912, 31080, -839_254_016, 3821, 31024, 178),
        (113, 12, 14, 3384, 31560, -194_248_704, 3378, 31453, 178),
        (1975, 12, 12, 6264, 32904, -667_811_840, 6291, 32922, 245),
    ];
    for (piece, anim, start, x, y, facing, sx, sy, dz) in rows {
        let at = launch_point(Pos::new(x, y), Angle(facing), piece, anim, start, 0);
        assert_eq!((at.x, at.y), (sx, sy), "{piece} ({x}, {y})");
        assert_eq!(release_dz(piece, anim, start, 0), Some(dz), "{piece}");
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

/// **The Gun Mahout's round leaves from piece 213's howdah** (item
/// 3713, `docs/AI.md` §358.4): run1387's `AMMO` record of `1/99`'s
/// round on 4907, fired on the move (`CHAR_ATTACKWALK`, frame 14) from
/// the figure's pose on that block, (6921, 8712) at 197197824 and a
/// guy `z` of 273; theirs' round leaves (6929, 8608, 699).
#[test]
fn run1387_s_gun_mahout_round_leaves_from_piece_213_s_howdah() {
    let at = launch_point(
        Pos::new(6921, 8712),
        Angle(197_197_824),
        213,
        crate::anim::ATTACKWALK,
        14,
        0,
    );
    assert_eq!((at.x, at.y), (6929, 8608), "1/99's round on 4907");
    assert_eq!(
        release_dz(213, crate::anim::ATTACKWALK, 14, 0),
        Some(699 - 273)
    );
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

/// **The British Catapult throws from piece 215's bay** (item 3662,
/// `docs/AI.md` §342): run1326b's `1/119` (TI 265) on block 8855, the
/// figure at (6456, 10488, 39) and angle −38862848, its round from
/// (6458, 10521, 284). Made to fail with the row out of `BAYS`.
#[test]
fn run1326b_s_catapult_throws_from_piece_215_s_bay() {
    let at = launch_point(
        Pos::new(6456, 10488),
        Angle(-38_862_848),
        215,
        crate::anim::ATTACK2,
        12,
        0,
    );
    assert_eq!((at.x, at.y), (6458, 10521), "1/119's round on 8855");
    assert_eq!(
        39 + release_dz(215, crate::anim::ATTACK2, 12, 0).unwrap(),
        284
    );
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
    assert_eq!(
        fits,
        [&(11, 9), &(11, 12), &(12, 9)],
        "1/29's round on 4298"
    );
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

/// **The German Anti-tank Rifles' rounds leave from piece 88's bays**
/// (item 3677): run1334b's `AMMO=5` records of `1/50`'s `CHAR_ATTACK1`
/// and `1/51`'s `CHAR_ATTACK2` rounds of 4305, each beside its figure's
/// `GUY` pose on 4304, at the dump's `sx, sy` and `sz − z`. Made to
/// fail with the rows out of `BAYS`.
#[test]
fn run1334b_s_german_anti_tank_rifles_leave_from_piece_88_s_bays() {
    for (at, facing, anim, start, want, dz) in [
        (
            Pos::new(4920, 31848),
            -1_094_451_200,
            crate::anim::ATTACK1,
            18,
            Pos::new(4740, 31841),
            548 - 488,
        ),
        (
            Pos::new(4392, 31656),
            -1_232_142_336,
            crate::anim::ATTACK2,
            4,
            Pos::new(4217, 31694),
            652 - 589,
        ),
    ] {
        assert_eq!(launch_point(at, Angle(facing), 88, anim, start, 0), want);
        assert_eq!(release_dz(88, anim, start, 0), Some(dz));
    }
}

/// **The American Anti-tank Rifle's round leaves from piece 87's bay**
/// (item 3686): run1338b's `AMMO=5` record of `1/18`'s `CHAR_ATTACK3`
/// round of 3105 beside its `GUY` pose on 3104, at the dump's `sx, sy`
/// and `sz − z`. Made to fail with the rows out of `BAYS`.
#[test]
fn run1338b_s_american_anti_tank_rifle_leaves_from_piece_87_s_bay() {
    assert_eq!(
        launch_point(
            Pos::new(9528, 31080),
            Angle(-798_359_552),
            87,
            crate::anim::ATTACK3,
            16,
            0
        ),
        Pos::new(9379, 31019)
    );
    assert_eq!(release_dz(87, crate::anim::ATTACK3, 16, 0), Some(695 - 612));
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

/// **The French Musketeers leave from piece 49's bays** (item 3691,
/// `docs/AI.md` §351): run1344b's rounds at seven, six and six facings
/// of each of the three heights, each the figure's `GUY` pose on the
/// block before the round's first print, reproduced at the dump's `sx,
/// sy` and `sz − z` by exactly one of the piece's three attack keys.
/// The first row is the word's: `1/37` on 4499, which this crate
/// launched from the figure's own square.
#[test]
fn run1344b_s_french_musketeers_leave_from_piece_49_s_bays() {
    // (x, y, facing, sx, sy, sz − z)
    #[rustfmt::skip]
    const ROWS: &[(i32, i32, i32, i32, i32, i32)] = &[
        (6072, 9480, 315_359_232, 6152, 9334, 176), // 1/37 on 4499
        (7464, 8760, -417_398_784, 7378, 8617, 176), // 1/34 on 4733
        (7272, 9672, -253_689_856, 7221, 9514, 176), // 1/36 on 4556
        (6840, 9192, -183_042_048, 6808, 9029, 176), // 1/22 on 4642
        (6648, 9048, -130_482_176, 6628, 8883, 176), // 1/31 on 4628
        (6504, 9192, -77_070_336, 6496, 9026, 176), // 1/38 on 4636
        (6264, 8712, -8_650_752, 6272, 8546, 176), // 1/47 on 4681
        (5832, 9384, 111_149_056, 5866, 9221, 176), // 1/32 on 4570
        (7464, 8760, -417_398_784, 7382, 8621, 178), // 1/34 on 4642
        (7032, 9480, -213_778_432, 6994, 9323, 178), // 1/36 on 4641
        (6552, 8520, -134_873_088, 6528, 8360, 178), // 1/46 on 4679
        (6552, 9048, -98_435_072, 6542, 8886, 178), // 1/26 on 4674
        (6072, 9480, 42_663_936, 6092, 9319, 178), // 1/37 on 4585
        (6552, 9720, 127_467_520, 6592, 9563, 178), // 1/22 on 4495
        (7464, 8760, -417_398_784, 7411, 8627, 218), // 1/34 on 4683
        (6840, 8904, -211_353_600, 6829, 8761, 218), // 1/46 on 4588
        (6840, 9624, -150_863_872, 6840, 9481, 218), // 1/24 on 4539
        (6408, 9672, -39_518_208, 6431, 9531, 218), // 1/22 on 4548
        (6024, 8760, 78_643_200, 6071, 8625, 218), // 1/37 on 4675
        (5640, 9336, 170_917_888, 5705, 9209, 218), // 1/30 on 4654
    ];
    for &(x, y, facing, sx, sy, dz) in ROWS {
        let keys: Vec<(i8, u32)> = [
            (crate::anim::ATTACK1, 1),
            (crate::anim::ATTACK2, 7),
            (crate::anim::ATTACK3, 4),
        ]
        .into_iter()
        .filter(|&(anim, t)| {
            let at = launch_point(Pos::new(x, y), Angle(facing), 49, anim, t, 0);
            (at.x, at.y) == (sx, sy) && release_dz(49, anim, t, 0) == Some(dz)
        })
        .collect();
        assert_eq!(keys.len(), 1, "({x}, {y}) at {facing}: keys {keys:?}");
    }
}

/// run971's two Musketeer rounds (piece 49, item 3336), the rows
/// [`MEASURED`] was once fitted to and [`BAYS`] keeps (item 3691):
/// `1/136`'s `ATTACK3` at frame 4 on 20374 and `1/54`'s `ATTACK1` at
/// frame 1 on 20378, `(anim, t, gx, gy, ga, gz, sx, sy, sz)` as the
/// dump prints them.
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
/// keys the event walk never reaches. **49**, the Musketeer's two from
/// run971 (item 3336), are [`BAYS`] rows since run1344b's 92 rounds
/// turned them (item 3691). **122**, the
/// Crossbowman's three, are run982's (item 3347): keys 5, 3 and 6. The
/// Dragoon's, **139**, left for `pivot::RELEASES` (item 3359), the
/// Cavalry Archer's, **138**, too (item 3633), and the Horse Archer's,
/// **842** (item 3338), for `pivot::SAME_AS` (item 3658).
#[test]
fn the_table_is_every_release_event_a_capture_has_measured() {
    let mut keys: Vec<(i32, i8, u32)> = MEASURED.iter().map(|(p, a, t, _)| (*p, *a, *t)).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
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
