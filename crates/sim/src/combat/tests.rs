use super::*;

/// **A Cruise Missile's spline on run1470** (item 3806, GOLDEN §152.4):
/// `2/19`'s round on the Anti-Aircraft Missile `1/20`, block 646. The
/// control points [`cruise_control_points`] lays — the launch, 250 up,
/// one ground point (`vector_dist` 2,113 over 768 is two steps) at its
/// ground 521 plus 250, the landing 250 up, the landing — and the
/// twenty-nine points [`Spline::cruise`] lays over them, every one to
/// the dump's six decimals; `Ammo::init`'s `total_time` is 28.
#[test]
fn a_cruise_missile_s_spline_is_run1470s() {
    let s = |m: i64| Single::from_millionths(m).0;
    let v = |x: i64, y: i64, z: i64| [s(x), s(y), s(z)];
    let (ctl, n) = cruise_control_points((16523, 16633, 958), (17398, 14709, 541), |x, y| {
        assert_eq!((x, y), (16960, 15671), "the one ground point");
        (521, 0)
    });
    let printed_ctl = [
        v(16_523_000_000, 16_633_000_000, 958_000_000),
        v(16_523_000_000, 16_633_000_000, 1_208_000_000),
        v(16_960_000_000, 15_671_000_000, 771_000_000),
        v(17_398_000_000, 14_709_000_000, 791_000_000),
        v(17_398_000_000, 14_709_000_000, 541_000_000),
    ];
    assert_eq!(&ctl[..n], &printed_ctl[..]);
    let path = Spline::cruise(&ctl[..n]);
    let printed = [
        v(16_523_000_000, 16_633_000_000, 958_000_000),
        v(16_526_263_672, 16_625_812_500, 1_004_612_793),
        v(16_535_742_188, 16_604_953_125, 1_038_158_081),
        v(16_550_951_172, 16_571_472_656, 1_059_910_034),
        v(16_571_417_969, 16_526_421_875, 1_071_142_822),
        v(16_596_667_969, 16_470_855_469, 1_073_130_737),
        v(16_626_216_797, 16_405_820_313, 1_067_147_827),
        v(16_659_593_750, 16_332_375_000, 1_054_468_750),
        v(16_696_320_313, 16_251_567_383, 1_036_367_554),
        v(16_735_914_063, 16_164_446_289, 1_014_117_981),
        v(16_777_900_391, 16_072_066_406, 988_994_934),
        v(16_821_804_688, 15_975_480_469, 962_272_278),
        v(16_867_152_344, 15_875_741_211, 935_224_548),
        v(16_913_457_031, 15_773_895_508, 909_125_610),
        v(16_960_250_000, 15_671_000_000, 885_250_000),
        v(17_007_046_875, 15_568_102_539, 864_522_949),
        v(17_053_376_953, 15_466_260_742, 846_475_220),
        v(17_098_750_000, 15_366_517_578, 830_288_086),
        v(17_142_697_266, 15_269_932_617, 815_143_555),
        v(17_184_732_422, 15_177_553_711, 800_223_083),
        v(17_224_376_953, 15_090_433_594, 784_708_374),
        v(17_261_156_250, 15_009_624_023, 767_781_128),
        v(17_294_585_938, 14_936_177_734, 748_623_047),
        v(17_324_185_547, 14_871_144_531, 726_415_771),
        v(17_349_480_469, 14_815_576_172, 700_341_064),
        v(17_369_988_281, 14_770_527_344, 669_580_322),
        v(17_385_232_422, 14_737_046_875, 633_315_491),
        v(17_394_728_516, 14_716_186_523, 590_728_027),
        v(17_398_000_000, 14_709_000_000, 541_000_000),
    ];
    assert_eq!(path.len(), printed.len());
    for (i, want) in printed.iter().enumerate() {
        for (c, (&ours, &theirs)) in path.point(i).iter().zip(want).enumerate() {
            assert!(
                ours.bits().abs_diff(theirs.bits()) <= u32::from(theirs.to_i32().abs() < 8),
                "point {i}[{c}]: {ours:?} against {theirs:?}"
            );
        }
    }
}

/// The clearance's three heights (`913c6e`–`913ccf`): bit 14 with both
/// low bits, a full surface field, and anything else.
#[test]
fn a_cruise_missile_clears_the_ground_by_its_tile() {
    assert_eq!(cruise_clearance(0), 250);
    assert_eq!(cruise_clearance(0x4003), 1250);
    assert_eq!(cruise_clearance(0x0003), 250, "bit 14 is half of it");
    assert_eq!(cruise_clearance(0x4002), 250);
    assert_eq!(cruise_clearance(0x0030), 750);
    assert_eq!(cruise_clearance(0x4033), 1250, "the first test wins");
}

/// **A V2's round is laid by `calc_nuke_spline`'s other arm** (item
/// 3814, GOLDEN §152.8): run1471's Nubian V2 `0/16` fires on `2/21`
/// from (16043, 15865, 925) to (18223, 16238, 631). Block 672 prints
/// the twelve control points, the knots `0 0 0 0 145 200 … 415` and
/// the first thirty of 121 points — `depth` 120, `total_time` 120 —
/// every one to the dump's six decimals, the basis sums' wobble in `x`
/// and `y` included.
#[test]
fn a_v2_s_spline_is_run1471s() {
    let s = |m: i64| Single::from_millionths(m).0;
    let v = |x: i64, y: i64, z: i64| [s(x), s(y), s(z)];
    let path = Spline::nuke((16043, 15865, 925), (18223, 16238, 631));
    let printed_ctl = [
        v(16_043_000_000, 15_865_000_000, 925_000_000),
        v(16_043_000_000, 15_865_000_000, 1_175_000_000),
        v(16_043_000_000, 15_865_000_000, 1_675_000_000),
        v(16_043_000_000, 15_865_000_000, 3_175_000_000),
        v(16_043_000_000, 15_865_000_000, 7_675_000_000),
        v(16_588_000_000, 15_958_250_000, 20_851_500_000),
        v(17_133_000_000, 16_051_500_000, 20_778_000_000),
        v(17_678_000_000, 16_144_750_000, 10_704_500_000),
        v(18_223_000_000, 16_238_000_000, 6_631_000_000),
        v(18_223_000_000, 16_238_000_000, 4_631_000_000),
        v(18_223_000_000, 16_238_000_000, 2_631_000_000),
        v(18_223_000_000, 16_238_000_000, 631_000_000),
    ];
    assert_eq!(printed_ctl.len(), 12);
    let printed = [
        v(16_043_000_000, 15_865_000_000, 925_000_000),
        v(16_043_001_953, 15_865_001_953, 943_084_045),
        v(16_043_000_000, 15_864_999_023, 961_575_867),
        v(16_042_998_047, 15_864_999_023, 980_499_878),
        v(16_043_002_930, 15_865_001_953, 999_880_493),
        v(16_043_000_000, 15_865_000_000, 1_019_740_845),
        v(16_043_000_000, 15_865_001_953, 1_040_105_713),
        v(16_042_999_023, 15_865_000_000, 1_060_999_023),
        v(16_042_997_070, 15_864_996_094, 1_082_444_580),
        v(16_043_001_953, 15_865_001_953, 1_104_467_041),
        v(16_043_000_000, 15_865_000_000, 1_127_089_966),
        v(16_042_999_023, 15_865_000_000, 1_150_337_402),
        v(16_043_000_000, 15_864_999_023, 1_174_233_887),
        v(16_043_000_000, 15_865_000_000, 1_198_802_856),
        v(16_043_000_977, 15_865_000_977, 1_224_068_970),
        v(16_042_999_023, 15_864_998_047, 1_250_055_908),
        v(16_043_000_977, 15_865_000_000, 1_276_788_086),
        v(16_043_000_977, 15_865_000_977, 1_304_289_307),
        v(16_043_000_000, 15_864_999_023, 1_332_583_496),
        v(16_043_001_953, 15_865_000_977, 1_361_695_313),
        v(16_042_998_047, 15_865_000_000, 1_391_648_071),
        v(16_042_999_023, 15_864_999_023, 1_422_466_431),
        v(16_043_000_977, 15_865_000_977, 1_454_174_438),
        v(16_043_000_000, 15_865_000_000, 1_486_795_654),
        v(16_043_002_930, 15_865_001_953, 1_520_354_736),
        v(16_042_998_047, 15_864_998_047, 1_554_875_244),
        v(16_042_999_023, 15_865_000_000, 1_590_381_714),
        v(16_042_999_023, 15_864_999_023, 1_626_897_827),
        v(16_043_000_000, 15_864_999_023, 1_664_447_998),
        v(16_042_999_023, 15_864_998_047, 1_703_056_152),
    ];
    assert_eq!(path.len(), 121, "depth 120 and the last point");
    for (i, want) in printed.iter().enumerate() {
        assert_eq!(path.point(i), *want, "block 672 point {i}");
    }
    assert_eq!(path.point(120), printed_ctl[11], "the landing");
}

/// **The guided round's two splines on run1469** (item 3802, GOLDEN
/// §152.2): the Russian Jet Fighter `2/14`'s round on `1/14`, launched
/// from (12775, 16410). Block 620 prints the spline its re-plan laid
/// (control points and nine points); block 624 the next, laid from that
/// one's `point[4]` truncated, the direction `P + 2·(point[5] − P)`, and
/// the target's point, at `200 × unit_move_speed` — six points, whose
/// `point[4]` (12848.98, 14983.03) is inside the Jet Fighter's
/// `target_size` and closes it. Every point to the dump's six decimals.
#[test]
fn guided_round_splines_are_run1469s() {
    let s = |m: i64| Single::from_millionths(m).0;
    let v = |x: i64, y: i64, z: i64| [s(x), s(y), s(z)];
    let speed = Single::from_i32(200 * UNIT_MOVE_SPEED);
    let first = Spline::from_dir(
        v(13_060_000_000, 16_384_000_000, -31_000_000),
        v(13_114_324_219, 16_195_578_125, -28_500_000),
        v(12_792_000_000, 14_712_000_000, 0),
        speed,
    );
    let printed_620 = [
        v(13_060_000_000, 16_384_000_000, -31_000_000),
        v(13_067_695_313, 16_316_658_203, -29_968_750),
        v(13_063_621_094, 16_208_841_797, -28_125_000),
        v(13_047_777_344, 16_060_552_734, -25_468_750),
        v(13_020_162_109, 15_871_789_063, -22_000_000),
        v(12_980_777_344, 15_642_552_734, -17_718_750),
        v(12_929_621_094, 15_372_841_797, -12_625_000),
        v(12_866_695_313, 15_062_658_203, -6_718_750),
        v(12_792_000_000, 14_712_000_000, 0),
    ];
    assert_eq!(first.len(), printed_620.len());
    for (i, want) in printed_620.iter().enumerate() {
        assert_eq!(first.point(i), *want, "block 620 point {i}");
    }
    let at = first.point(4).map(|c| Single::from_i32(c.to_i32()));
    let ahead = first.point(5);
    let two = Single::from_bits(0x4000_0000);
    let dir = [0, 1, 2].map(|c| ahead[c].subss(at[c]).mulss(two).addss(at[c]));
    assert_eq!(dir, v(12_941_554_688, 15_414_105_469, -13_437_500));
    let second = Spline::from_dir(at, dir, v(12_792_000_000, 14_712_000_000, 0), speed);
    let printed_624 = [
        v(13_020_000_000, 15_871_000_000, -22_000_000),
        v(12_985_778_320, 15_678_435_547, -18_380_001),
        v(12_945_866_211, 15_466_250_977, -14_370_001),
        v(12_900_266_602, 15_234_450_195, -9_969_999),
        v(12_848_977_539, 14_983_034_180, -5_179_999),
        v(12_792_000_000, 14_712_000_000, 0),
    ];
    assert_eq!(second.len(), printed_624.len());
    // A print of six decimals names a single uniquely only above
    // magnitude ~8: `z` -5.179999 is two singles, and either is it.
    for (i, want) in printed_624.iter().enumerate() {
        for (c, (&ours, &theirs)) in second.point(i).iter().zip(want).enumerate() {
            assert!(
                ours.bits().abs_diff(theirs.bits()) <= u32::from(theirs.to_i32().abs() < 8),
                "block 624 point {i}[{c}]: {ours:?} against {theirs:?}"
            );
        }
    }
}

/// **A sea or air shooter's lead is a float rotation, at one degree
/// past its figure** (item 3808, GOLDEN §152.5): run1471's Russian Jet
/// Fighter `2/14`, parked at 120°, fires its second round on `1/14`
/// from (12591, 16728, −36) to (12835, 14802, 0). Block 662 prints the
/// middle control point (13233.875, 17114.279297, −36) — 750 out at
/// 121° — and thirteen points; block 665 the re-plan from `point[4]`,
/// eleven points (this crate laid twelve and ten before, and landed
/// the round a frame early). Every point to the dump's six decimals.
#[test]
fn a_figure_s_lead_is_run1471s() {
    let s = |m: i64| Single::from_millionths(m).0;
    let v = |x: i64, y: i64, z: i64| [s(x), s(y), s(z)];
    // Six decimals name a single to the bit only above magnitude ~8;
    // below it the print is the value to half a millionth.
    let near = |got: [Single; 3], want: [Single; 3], what: &str| {
        for (c, (&ours, &theirs)) in got.iter().zip(&want).enumerate() {
            let (a, b) = (f32::from_bits(ours.bits()), f32::from_bits(theirs.bits()));
            assert!(
                ours == theirs || (theirs.to_i32().abs() < 8 && (a - b).abs() <= 5e-7),
                "{what}[{c}]: {a} against {b}"
            );
        }
    };
    let (launch, landing) = (Pos::new(12591, 16728), Pos::new(12835, 14802));
    let lead = Lead {
        angle: Angle(1_431_655_765),
        pitch: Single::ZERO,
        r: 750,
    };
    assert_eq!(
        missile_lead(launch, -36, lead),
        v(13_233_875_000, 17_114_279_297, -36_000_000),
        "the middle control point"
    );
    let first = missile_spline(launch, -36, landing, 0, Some(lead), 150);
    let printed_662 = [
        v(12_591_000_000, 16_728_000_000, -36_000_000),
        v(12_690_911_133, 16_773_640_625, -35_750_000),
        v(12_776_353_516, 16_781_798_828, -35_000_000),
        v(12_847_328_125, 16_752_480_469, -33_750_000),
        v(12_903_832_031, 16_685_677_734, -31_999_996),
        v(12_945_870_117, 16_581_398_438, -29_750_000),
        v(12_973_437_500, 16_439_640_625, -27_000_000),
        v(12_986_536_133, 16_260_400_391, -23_750_000),
        v(12_985_166_016, 16_043_679_688, -20_000_004),
        v(12_969_328_125, 15_789_480_469, -15_750_003),
        v(12_939_020_508, 15_497_798_828, -11_000_005),
        v(12_894_244_141, 15_168_640_625, -5_750_006),
        v(12_835_000_000, 14_802_000_000, -9),
    ];
    assert_eq!(first.len(), printed_662.len());
    for (i, want) in printed_662.iter().enumerate() {
        near(first.point(i), *want, &format!("block 662 point {i}"));
    }
    let at = first.point(4).map(|c| Single::from_i32(c.to_i32()));
    let ahead = first.point(5);
    let two = Single::from_bits(0x4000_0000);
    let dir = [0, 1, 2].map(|c| ahead[c].subss(at[c]).mulss(two).addss(at[c]));
    let speed = Single::from_i32(200 * UNIT_MOVE_SPEED);
    let second = Spline::from_dir(at, dir, v(12_792_000_000, 14_712_000_000, 0), speed);
    let printed_665 = [
        v(12_903_000_000, 16_685_000_000, -31_000_000),
        v(12_917_322_266, 16_627_970_703, -30_239_998),
        v(12_925_998_047, 16_539_775_391, -28_960_003),
        v(12_929_020_508, 16_420_404_297, -27_160_000),
        v(12_926_395_508, 16_269_863_281, -24_840_000),
        v(12_918_120_117, 16_088_148_438, -22_000_000),
        v(12_904_195_313, 15_875_261_719, -18_639_999),
        v(12_884_621_094, 15_631_204_102, -14_759_997),
        v(12_859_396_484, 15_355_974_609, -10_359_996),
        v(12_828_523_438, 15_049_573_242, -5_439_995),
        v(12_792_000_000, 14_712_000_000, 0),
    ];
    assert_eq!(second.len(), printed_665.len(), "the re-plan's count");
    for (i, want) in printed_665.iter().enumerate() {
        near(second.point(i), *want, &format!("block 665 point {i}"));
    }
}

/// **A land shooter's lead is half the distance out, at 45° up** (item
/// 3816, GOLDEN §152.9; `Ammo::init@0067bbf0`, `67c437`–`67c469`):
/// run1469's French Anti-Aircraft Missile `1/20` (`PROJ_SPEED` 400)
/// fires on the Russian `2/18` from (17346, 14755, 639) to (15778,
/// 16581, 0), the ammo's angle `−1,688,993,792` (218°). Block 639
/// prints the middle control point (16908.123047, 15295.732422,
/// 1334.793091) — `r` 984, `attack_dist` over two, rotated at 219° —
/// and the eight points; this crate laid the half-way point on the
/// straight line, seven points, and freed the round's slot four frames
/// early.
#[test]
fn a_land_shooter_s_lead_is_run1469s() {
    let s = |m: i64| Single::from_millionths(m).0;
    let v = |x: i64, y: i64, z: i64| [s(x), s(y), s(z)];
    let (launch, landing) = (Pos::new(17346, 14755), Pos::new(15778, 16581));
    let lead = Lead {
        angle: Angle(-1_688_993_792),
        pitch: Single::from_bits(0x4234_0000),
        r: 984,
    };
    assert_eq!(
        missile_lead(launch, 639, lead),
        v(16_908_123_047, 15_295_732_422, 1_334_793_091),
        "the middle control point"
    );
    let first = missile_spline(launch, 639, landing, 0, Some(lead), 400);
    let printed_639 = [
        v(17_346_000_000, 14_755_000_000, 639_000_000),
        v(17_206_765_625, 14_924_689_453, 796_357_544),
        v(17_039_275_391, 15_124_767_578, 870_833_984),
        v(16_843_529_297, 15_355_236_328, 862_429_199),
        v(16_619_529_297, 15_616_093_750, 771_143_494),
        v(16_367_274_414, 15_907_338_867, 596_976_746),
        v(16_086_764_648, 16_228_974_609, 339_928_894),
        v(15_778_000_000, 16_581_000_000, 0),
    ];
    assert_eq!(first.len(), printed_639.len(), "the point count");
    for (i, want) in printed_639.iter().enumerate() {
        assert_eq!(first.point(i), *want, "block 639 point {i}");
    }
}

/// **The lead's two index folds** (`67cbb3`–`67cc90`): a pitch reads
/// its truncated degree, a negative one `359 − (−p mod 360)`; the
/// angle's degree is negated first, so 0 stays 0 and 120 reads 239.
/// At pitch 0 and angle 0 the vector is `(0, −r, 0)` turned by nothing.
#[test]
fn the_lead_folds_its_degrees() {
    let o = Pos::new(1000, 1000);
    let at = |angle: i32, pitch: i32| {
        missile_lead(
            o,
            0,
            Lead {
                angle: Angle(angle),
                pitch: Single::from_i32(pitch),
                r: 1000,
            },
        )
        .map(Single::to_i32)
    };
    assert_eq!(at(0, 0), [1000, 0, 0], "due north, untouched");
    // 0x40000000 is 90°; the fold reads 269, a quarter turn and a degree.
    let [x, y, _] = at(0x4000_0000, 0);
    assert!(x > 999 && (17..=18).contains(&(y - 1000)), "({x}, {y})");
    // A pitch tilts the vector out of the plane: z moves, x does not.
    let [x, _, z] = at(0, 30);
    assert_eq!(x, 1000);
    assert_ne!(z, 0, "a pitched figure leads up or down");
}

/// **A splash walks one spiral cell past its square** (item 3636,
/// parked 3356c): `Ammo::do_damage`'s walk is `0..=radius[k]`, and the
/// entry at `radius[k] = (2k+1)²` is the first of ring `k + 1` —
/// `(−1, −2)` for a splash of up to three tiles, then the ring's
/// north-west corner. run1296b's Bombard round lands in cell (8, 9)
/// and wounds the Library in (7, 7) through this cell. Every shipped
/// splash (1, 2, 10, 20) is `k <= 6`.
#[test]
fn a_splash_walks_one_spiral_cell_past_its_square() {
    assert_eq!(splash_cells(1), 1);
    assert_eq!(splash_extra_cell(1), Some((-1, -2)));
    for k in 2..=7 {
        assert_eq!(splash_extra_cell(k), Some((-(k + 1), -(k + 1))), "k {k}");
    }
    assert_eq!(splash_extra_cell(splash_cells(20)), Some((-7, -7)));
    assert_eq!(splash_extra_cell(8), None, "past the crate's table");
}

/// **A missile flies thirteen frames once its polygon reaches twelve
/// speeds** (item 1625, `docs/AI.md` §174): run717's first round of
/// the Advanced Battleship `1/49`, launched on 3124 and landing on
/// 3136 — the straight line's 19 put it on 3142. Shorter shots count
/// their polygon in speeds; the floor is three points.
#[test]
fn a_missile_flies_its_spline_s_point_count() {
    let a = Angle(213_450_752);
    let (launch, land) = (Pos::new(4472, 12771), Pos::new(6221, 7338));
    assert_eq!(flight_time(5707 * 5707, 300), 19, "the straight line");
    assert_eq!(
        missile_flight_time(
            launch,
            171,
            land,
            193,
            Some(Lead {
                angle: a,
                pitch: Single::ZERO,
                r: 1500
            }),
            300
        ),
        13
    );
    assert_eq!(missile_flight_time(launch, 171, land, 193, None, 300), 13);
    // 1,500 north, then 500 back: a polygon of 2,000, six speeds.
    let o = Pos::new(10_000, 10_000);
    let near = Pos::new(10_000, 9_000);
    assert_eq!(
        missile_flight_time(
            o,
            0,
            near,
            0,
            Some(Lead {
                angle: Angle(0),
                pitch: Single::ZERO,
                r: 1500
            }),
            300
        ),
        7
    );
    assert_eq!(missile_flight_time(o, 0, near, 0, None, 300), 4);
    assert_eq!(
        missile_flight_time(o, 0, Pos::new(10_000, 9_900), 0, None, 300),
        3
    );
    assert_eq!(missile_flight_time(o, 0, near, 0, None, 0), 3);
}

const T: Tuning = Tuning::RON;

fn unit_profile(attack_tenths: i32, armor: i32, masks: u32) -> Profile {
    Profile {
        attack: attack_tenths,
        armor,
        obj_masks: masks,
        uber_size: 1,
        ammo_per_att: 1,
        splash_percent: 100,
        ..Profile::default()
    }
}

/// A unit facing south — toward an attack travelling north, so that the
/// default angle of the calls below is head-on and no flank applies.
fn unit_side() -> Side {
    Side {
        unit: true,
        facing: Angle::SOUTH,
        ..Side::default()
    }
}

#[test]
fn the_rng_is_the_lcg_and_a_degenerate_range_does_not_advance() {
    let mut r = Rng::new(1);
    let s0 = r.seed;
    assert_eq!(r.get(7, 7), 7);
    assert_eq!(r.seed, s0);
    let v = r.roll();
    assert_eq!(r.seed, 1u32.wrapping_mul(0x19660d).wrapping_add(0x3c6ef35f));
    assert_eq!(v, (((r.seed & 0xffff) * 0xffff) >> 16) as i32);
    assert!(v < 0xffff);
    // lo > hi swaps.
    let mut a = Rng::new(99);
    let mut b = Rng::new(99);
    assert_eq!(a.get(10, 3), b.get(3, 10));
}

#[test]
fn obj_mask_letters_parse_as_the_loader_does() {
    assert_eq!(mask::parse("A"), mask::ARMORED);
    assert_eq!(mask::parse("fh"), mask::FOOT | mask::HEAVY_INF);
    assert_eq!(mask::parse("3"), mask::AIR);
    assert_eq!(mask::parse("6"), mask::ANTI_AIR);
    assert_eq!(mask::parse("FW "), mask::FOOT | mask::MELEE);
}

#[test]
fn base_damage_is_attack_times_table_then_rounded_to_hits_minus_armor() {
    let a = unit_profile(120, 0, mask::FOOT | mask::HEAVY_INF);
    let b = unit_profile(100, 3, mask::FOOT | mask::LIGHT_INF);
    // 12 attack, 100 %: (120 + 5) / 10 − 3 = 9.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            unit_side(),
            a.attack,
            b.armor,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        9
    );
    // 150 %: 180 → (180 + 5) / 10 − 3 = 15.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            unit_side(),
            a.attack,
            b.armor,
            150,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        15
    );
    // Armour that exceeds the attack gives at least one.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            unit_side(),
            a.attack,
            30,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        1
    );
    // …unless it is splash.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            unit_side(),
            a.attack,
            30,
            100,
            Angle::NORTH,
            true,
            1,
            &Modifiers::default()
        ),
        -18
    );
}

#[test]
fn musket_infantry_face_a_third_more_armour_and_anti_air_never_scratches_the_ground() {
    let a = unit_profile(100, 0, mask::MUSKET_INF);
    let b = unit_profile(100, 6, mask::FOOT);
    // armour 6 × 133 / 100 = 7: (100 + 5) / 10 − 7 = 3.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            unit_side(),
            100,
            6,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        3
    );
    let aa = unit_profile(10, 0, mask::ANTI_AIR);
    assert_eq!(
        get_damage(
            &T,
            &aa,
            unit_side(),
            &b,
            unit_side(),
            10,
            5,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        -4
    );
}

#[test]
fn flanking_is_rear_one_side_two_and_front_nothing() {
    // Target faces north. An attack travelling north comes from behind.
    assert_eq!(flank_level(Angle::NORTH, Angle::NORTH), Some(1));
    // Travelling south it meets the target's face.
    assert_eq!(flank_level(Angle::NORTH, Angle::SOUTH), None);
    // From the sides.
    assert_eq!(flank_level(Angle::NORTH, Angle::EAST), Some(2));
    assert_eq!(flank_level(Angle::NORTH, Angle::WEST), Some(2));
    // 59° off the face is still the face; 61° is a flank.
    let deg = |d: i32| {
        Angle(
            Angle::SOUTH
                .0
                .wrapping_add(crate::movement::ONE_DEGREE.wrapping_mul(d)),
        )
    };
    assert_eq!(flank_level(Angle::NORTH, deg(59)), None);
    assert_eq!(flank_level(Angle::NORTH, deg(61)), Some(2));
    // 44° off the back is the back; 46° is a side.
    let back = |d: i32| {
        Angle(
            Angle::NORTH
                .0
                .wrapping_add(crate::movement::ONE_DEGREE.wrapping_mul(d)),
        )
    };
    assert_eq!(flank_level(Angle::NORTH, back(44)), Some(1));
    assert_eq!(flank_level(Angle::NORTH, back(46)), Some(2));

    let a = unit_profile(100, 0, mask::FOOT);
    let b = unit_profile(100, 0, mask::FOOT);
    let t = Side {
        facing: Angle::NORTH,
        ..unit_side()
    };
    // Rear: (50 × 1 + 100) × 100 / 100 = 150 → 15.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            t,
            100,
            0,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        15
    );
    // Side: 200 → 20.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            t,
            100,
            0,
            100,
            Angle::EAST,
            false,
            1,
            &Modifiers::default()
        ),
        20
    );
    // A mounted **target**'s bonus is 40/256 of that: 50 × 40 >> 8 = 7,
    // rear 107 → 11. The test reads the target's mask (`00644b3a` tests
    // `edx`, loaded from the target type at `00644ace`; item 601), so a
    // mounted attacker on a foot target flanks at the whole 50.
    let cav = unit_profile(100, 0, mask::MOUNTED);
    let flank = |a: &Profile, b: &Profile, attack: i32| {
        get_damage(
            &T,
            a,
            unit_side(),
            b,
            t,
            attack,
            0,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default(),
        )
    };
    assert_eq!(
        flank(&a, &cav, 100),
        11,
        "a flanked rider takes the reduced bonus"
    );
    assert_eq!(
        flank(&cav, &b, 100),
        15,
        "a rider flanking foot deals the whole bonus"
    );
    // VEHICLE is tested first: 50 × 33 >> 8 = 6, so at attack 1000 the
    // rear is 1060 → 106 where the cavalry reduction would give 107.
    let tank = unit_profile(100, 0, mask::VEHICLE | mask::MOUNTED);
    assert_eq!(
        flank(&a, &tank, 1000),
        106,
        "a flanked vehicle takes its own reduction"
    );
    assert_eq!(flank(&a, &cav, 1000), 107);
    // Civilians neither flank nor are flanked.
    let civ = unit_profile(100, 0, mask::CIVILIAN);
    assert_eq!(
        get_damage(
            &T,
            &civ,
            unit_side(),
            &b,
            t,
            100,
            0,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        10
    );
}

/// **The ranking's damage skips the overkill step** (item 601,
/// `docs/COMBAT.md` §53). `Object::compare_target` calls `get_damage`
/// with `check_overkill` 0 (`0064ebce`–`0064ebdf`), so a target another
/// squad has just hit ranks at its whole damage. Golden chapter three's
/// `0/7` takes the wounded `1/7` on 685 on it; with the overkill third
/// in the ranking it took `1/6`.
#[test]
fn the_ranking_s_damage_skips_the_overkill_step() {
    let mut a = unit_profile(300, 0, mask::FOOT_ARCHER);
    a.max_range = 5;
    let b = unit_profile(100, 0, mask::FOOT);
    let second = Side {
        captain: 2,
        ..unit_side()
    };
    let target = Side {
        damage_frame: 100,
        damage_o: 1,
        ..unit_side()
    };
    let hit = |check: bool| {
        get_damage_checked(
            &T,
            &a,
            second,
            &b,
            target,
            300,
            0,
            100,
            Angle::NORTH,
            false,
            check,
            110,
            &Modifiers::default(),
        )
    };
    assert_eq!(hit(true), 9, "do_damage's call takes the third");
    assert_eq!(hit(false), 30, "compare_target's call does not");
}

#[test]
fn overkill_cuts_a_second_ranged_squads_hit_to_a_third_inside_the_window() {
    let mut a = unit_profile(300, 0, mask::FOOT_ARCHER);
    a.max_range = 5;
    let b = unit_profile(100, 0, mask::FOOT);
    let first = Side {
        captain: 1,
        ..unit_side()
    };
    let target = Side {
        damage_frame: 100,
        damage_o: 1,
        ..unit_side()
    };
    // The owner of the window: full 30.
    assert_eq!(
        get_damage(
            &T,
            &a,
            first,
            &b,
            target,
            300,
            0,
            100,
            Angle::NORTH,
            false,
            110,
            &Modifiers::default()
        ),
        30
    );
    // Another squad inside it: 30 × 85 >> 8 = 9.
    let second = Side {
        captain: 2,
        ..unit_side()
    };
    assert_eq!(
        get_damage(
            &T,
            &a,
            second,
            &b,
            target,
            300,
            0,
            100,
            Angle::NORTH,
            false,
            110,
            &Modifiers::default()
        ),
        9
    );
    // After 30 frames the window is gone.
    assert_eq!(
        get_damage(
            &T,
            &a,
            second,
            &b,
            target,
            300,
            0,
            100,
            Angle::NORTH,
            false,
            130,
            &Modifiers::default()
        ),
        30
    );
    // A melee attacker neither suffers it…
    let mut m = a;
    m.max_range = 0;
    assert_eq!(
        get_damage(
            &T,
            &m,
            second,
            &b,
            target,
            300,
            0,
            100,
            Angle::NORTH,
            false,
            110,
            &Modifiers::default()
        ),
        30
    );
    // …and a catapult target halves it again for a non-siege attacker.
    let mut cat = b;
    cat.roles = role::IS_CATAPULT;
    assert_eq!(
        get_damage(
            &T,
            &a,
            second,
            &cat,
            target,
            300,
            0,
            100,
            Angle::NORTH,
            false,
            110,
            &Modifiers::default()
        ),
        4
    );
}

#[test]
fn terrain_and_entrenchment_apply_to_the_net() {
    let a = unit_profile(300, 0, mask::FOOT);
    let b = unit_profile(100, 0, mask::LIGHT_INF);
    let rocky = Side {
        rocky: true,
        ..unit_side()
    };
    // 30 × 170 >> 8 = 19.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            rocky,
            300,
            0,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        19
    );
    // Height: attacker 400 above → + 400 × 10 × 30 / (200 × 100) = +6.
    let high = Side {
        z: 400,
        ..unit_side()
    };
    assert_eq!(
        get_damage(
            &T,
            &a,
            high,
            &b,
            unit_side(),
            300,
            0,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        36
    );
    // Entrenched facing north, attacked from the north (the attack
    // travels south, and the unit faces north into it): protected,
    // 30 × 170 >> 8 = 19.
    let trench = Side {
        entrenched: true,
        trench_facing: Angle::NORTH,
        facing: Angle::NORTH,
        ..unit_side()
    };
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            trench,
            300,
            0,
            100,
            Angle::SOUTH,
            false,
            1,
            &Modifiers::default()
        ),
        19
    );
    // Attacked from behind (the attack travels north): the trench does
    // not help, and it is a rear flank besides: 45.
    assert_eq!(
        get_damage(
            &T,
            &a,
            unit_side(),
            &b,
            trench,
            300,
            0,
            100,
            Angle::NORTH,
            false,
            1,
            &Modifiers::default()
        ),
        45
    );
}

#[test]
fn scaling_splits_into_whole_and_sixteenths() {
    // A unit of three firing for 30: each figure's shot is 10.
    assert_eq!(
        scale(30, 0x100, true, true, 1, 3),
        Sixteenths { whole: 10, frac: 0 }
    );
    // For 31: 31 × 256 / 3 = 2645 → 165 sixteenths → 10 + 5/16.
    assert_eq!(
        scale(31, 0x100, true, true, 1, 3),
        Sixteenths { whole: 10, frac: 5 }
    );
    // Nothing below one whole hit before the division.
    assert_eq!(
        scale(0, 0x100, true, false, 1, 4),
        Sixteenths { whole: 0, frac: 4 }
    );
    assert_eq!(
        scale(-5, 0x100, true, false, 1, 1),
        Sixteenths { whole: 1, frac: 0 }
    );
    // A building's volley divides by ammo per attack and has no floor.
    assert_eq!(
        scale(8, 0x100, false, true, 4, 1),
        Sixteenths { whole: 2, frac: 0 }
    );
    // A splash fringe at half count.
    assert_eq!(
        scale(10, 0x80, true, true, 1, 1),
        Sixteenths { whole: 5, frac: 0 }
    );
}

#[test]
fn a_figure_takes_its_share_and_the_last_captain_the_remainder() {
    assert_eq!(share(100, 3, false), 33);
    assert_eq!(share(100, 3, true), 34);
    assert_eq!(share(90, 3, true), 30);
    assert_eq!(share(50, 1, false), 50);
    // Sixteenths carry.
    let (t, d, f) = take(0, 12, 33, Sixteenths { whole: 0, frac: 8 });
    assert_eq!((t, d, f), (Taken::Alive { lost: 1 }, 1, 4));
    // A hit of nothing is a sixteenth.
    let (t, d, f) = take(0, 0, 33, Sixteenths { whole: 0, frac: 0 });
    assert_eq!((t, d, f), (Taken::Alive { lost: 0 }, 0, 1));
    // Death at the share.
    let (t, ..) = take(30, 0, 33, Sixteenths { whole: 3, frac: 0 });
    assert_eq!(
        t,
        Taken::Died {
            lost: 3,
            overflow: 0
        }
    );
    let (t, ..) = take(30, 0, 33, Sixteenths { whole: 10, frac: 0 });
    assert_eq!(
        t,
        Taken::Died {
            lost: 10,
            overflow: 7
        }
    );
}

#[test]
fn recharge_and_accuracy_and_scatter() {
    assert_eq!(recharge(40, false, false), 40);
    assert_eq!(recharge(40, true, false), 60);
    assert_eq!(recharge(40, true, true), 80);
    // The byte.
    assert_eq!(recharge(200, true, true), 400 & 0xff);
    // Accuracy: to_hit 80, attenuate 1, ten tiles = 1920 units → 80 − 10;
    // a tile and a half is still one whole tile.
    assert_eq!(accuracy(80, 1, 1920), 70);
    assert_eq!(accuracy(80, 1, 287), 79);
    assert_eq!(accuracy(3, 0, 0), 5);
    // Scatter at 60 %: 96 × 100 / (8 + 60) = 141; at 120 %: 96 × 100 /
    // (−4 + 120) = 82 → quartered 20.
    assert_eq!(scatter(&T, 60, true, false, false), 141);
    assert_eq!(scatter(&T, 120, true, false, false), 20);
    assert_eq!(scatter(&T, 60, false, false, false), 0xc0);
    assert_eq!(scatter(&T, 60, true, true, false), 282);
    assert_eq!(scatter(&T, 60, true, false, true), 0);
    // Two draws, or none.
    let mut r = Rng::new(5);
    let s0 = r.seed;
    assert_eq!(
        scatter_point(&mut r, Pos::new(100, 100), 1),
        Pos::new(100, 100)
    );
    assert_eq!(r.seed, s0);
    let p = scatter_point(&mut r, Pos::new(100, 100), 10);
    assert!((p.x - 100).abs() <= 5 && (p.y - 100).abs() <= 5);
    assert_ne!(r.seed, s0);
}

/// [`fall_time`] against the host's own single precision — the test
/// oracle `no_float.rs` allows — over every drop from 1 to 40,000
/// units, the whole height a plane can fly. run223's `0/8` drops from
/// 1632 onto ground at 10: `sqrt(3244 / 10.4875)` is 17.59, so 17.
#[test]
fn fall_time_matches_the_float_expression() {
    let g = f32::from_bits(GRAV_Z.bits());
    for drop in 1..40_000i32 {
        let q = ((-drop) * 2) as f32 / g;
        let want = f64::from(q).sqrt() as f32;
        assert_eq!(fall_time(drop, 0), want as i32, "drop {drop}");
    }
    assert_eq!(fall_time(1632, 10), 17);
    assert_eq!(fall_time(10, 10), 0);
}

/// **A bomb released under its landing falls for 2^31 frames** (item
/// 3836, `docs/GOLDEN.md` §152.14): run1468's Stealth Bomber `1/17`
/// drops from `sz` 12 onto ground at `ez` 578, and every block from
/// 801 prints `total_time 2147483648` and `v1z 11260867584.000000`
/// for it. The root of a negative quotient is the NaN indefinite, its
/// truncation `0x80000000`; `v1z` divides by the time read unsigned;
/// and `Ammo::inc_time`'s end test, unsigned too, holds it up.
#[test]
fn a_bomb_released_under_its_landing_never_comes_down() {
    assert_eq!(fall_time(12, 578), i32::MIN);
    assert_eq!(fall_time(0, 511), i32::MIN);
    assert_eq!(fall_time(578, 578), 0, "a zero quotient roots to 0");
    let v = arc_v1z(12, 578, i32::MIN);
    assert_eq!(f32::from_bits(v.bits()), 11_260_867_584.0);
    for cur in [1, 488, 1888, i32::MAX] {
        assert!(in_flight_at(cur, i32::MIN), "still up at {cur}");
    }
    assert!(in_flight_at(4, 5));
    assert!(!in_flight_at(5, 5), "an ordinary round ends at its time");
}

#[test]
fn flight_time_matches_the_float_expression() {
    // Spot-checked against (int)(sqrtf(n) / (float)d) evaluated in
    // single precision.
    let check = |n: i64, d: i64, want: i32| {
        assert_eq!(flight_time(n, d), want, "n={n} d={d}");
        // The reference, in f32, for the values where the host's sqrt is
        // correctly rounded (it is, for IEEE hosts).
        let r = ((n as f32).sqrt() / d as f32) as i32;
        assert_eq!(flight_time(n, d), r, "host n={n} d={d}");
    };
    check(0, 100, 0);
    check(1, 1, 1);
    check(1_000_000, 100, 10);
    check(999_999, 100, 9);
    check(2_000_000, 137, 10);
    check(36_864 * 36_864 + 1920 * 1920, 200, 184);
    for n in [3, 7, 123_456, 9_999_991, 1 << 30, (1i64 << 40) + 17] {
        for d in [1, 3, 96, 200, 777, 4_000] {
            let r = ((n as f32).sqrt() / d as f32) as i32;
            assert_eq!(flight_time(n, d), r, "n={n} d={d}");
        }
    }
    assert_eq!(siege_flight_time(10, 200, 1), 1920 / 200);
}

#[test]
fn the_hit_tests_and_splash_count() {
    assert!(hits_unit(Pos::new(0, 0), Pos::new(30, 0), 48, 50));
    assert!(!hits_unit(Pos::new(0, 0), Pos::new(60, 0), 48, 50));
    // Over 100 % accuracy the distance is halved.
    assert!(hits_unit(Pos::new(0, 0), Pos::new(60, 0), 48, 101));
    assert!(hits_building(Pos::new(95, 0), Pos::new(0, 0), 1, 1));
    assert!(!hits_building(Pos::new(97, 0), Pos::new(0, 0), 1, 1));
    assert_eq!(splash_count(0, 2), 0x100);
    assert_eq!(splash_count(192, 2), 0x80);
    assert_eq!(splash_count(384, 2), 0);
    assert_eq!(splash_count(-10, 2), 0x100);
}

#[test]
fn attack_dist_is_edge_to_edge_on_the_quarter_tile_grid() {
    let u = (24 + 0x18, 24 + 0x18);
    // Two figures 4 tiles apart on the x axis, each with block radius
    // 24: 768 − 48 − 48 = 672.
    assert_eq!(
        attack_dist(Pos::new(0, 0), Pos::new(768, 0), u, u, false),
        672
    );
    // Inside the grid step nothing changes; one step out, one step more.
    assert_eq!(
        attack_dist(Pos::new(47, 0), Pos::new(768, 0), u, u, false),
        672
    );
    assert_eq!(
        attack_dist(Pos::new(48, 0), Pos::new(768, 0), u, u, false),
        624
    );
    // Overlapping extents clamp to zero.
    assert_eq!(attack_dist(Pos::new(0, 0), Pos::new(60, 0), u, u, false), 0);
    // A 2×2 building's extent is a tile each way.
    let b = (2 * 0x60, 2 * 0x60);
    assert_eq!(
        attack_dist(Pos::new(0, 0), Pos::new(768, 0), u, b, false),
        768 - 192 - 48
    );
    // A plane ignores extents.
    assert_eq!(
        attack_dist(Pos::new(0, 0), Pos::new(768, 0), u, u, true),
        768
    );
    // Range: melee 102; ranged 3 tiles = 582 inclusive.
    assert!(in_range(102, 0, 0, false, 0, false));
    assert!(!in_range(103, 0, 0, false, 0, false));
    assert!(in_range(246, 0, 0, true, 0, false));
    assert!(in_range(3 * 192 + 6, 3, 0, false, 0, false));
    assert!(!in_range(3 * 192 + 7, 3, 0, false, 0, false));
    // Inside the minimum, rescued by the big radii.
    assert!(!in_range(100, 5, 1, false, 0, false));
    assert!(in_range(100, 5, 1, false, 100, false));
    // Rings.
    assert_eq!(ring_of(0, 0), 0);
    assert_eq!(ring_of(1, 0), 1);
    assert_eq!(ring_of(1, 1), 1);
    assert_eq!(ring_of(2, 1), 2);
    assert_eq!(ring_of(2, 2), 3);
}

#[test]
fn garrison_arrows() {
    // A tower with attack 8, base 1, most 4: empty → 1; two archers of
    // attack 6 each → 1 + 12/8 = 2; capped.
    assert_eq!(super::garrison_arrows(80, 1, 4, 0), 1);
    assert_eq!(super::garrison_arrows(80, 1, 4, 12), 2);
    assert_eq!(super::garrison_arrows(80, 1, 4, 400), 5);
    // No base: half the attack is added first.
    assert_eq!(super::garrison_arrows(80, 0, 4, 0), 0);
    assert_eq!(super::garrison_arrows(80, 0, 4, 4), 1);
    // An attack under a whole hit fires nothing at all.
    assert_eq!(super::garrison_arrows(3, 1, 4, 100), 0);
}
