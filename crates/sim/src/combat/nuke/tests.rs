use super::*;

/// **The ring is the emulated original's on every frame**
/// (`tools/emu/nuke_blast.py`: the executable's own `Nuke::do_damage`,
/// its SSE float32 run under unicorn with the nuke's pair (100, 1920),
/// the radius read off the `Objects::find_units` it calls). Made to
/// fail with the grown term rounded up (`(… + 29) / 30`: 161 on t = 11).
#[test]
fn the_ring_is_the_emulated_original_s_on_every_frame() {
    #[rustfmt::skip]
    const EMULATED: [i32; 40] = [
        100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 160, 221, 282,
        342, 403, 464, 524, 585, 646, 706, 767, 828, 888, 949, 1010, 1070,
        1131, 1192, 1252, 1313, 1374, 1434, 1495, 1556, 1616, 1677, 1738,
        1798, 1859,
    ];
    for (t, &r) in EMULATED.iter().enumerate() {
        assert_eq!(
            ring_radius(t as i64, RING_MIN, 10 * RING_PER_SPLASH),
            Some(r),
            "t = {t}"
        );
    }
    assert_eq!(ring_radius(40, RING_MIN, 1920), None, "the ring is spent");
    assert_eq!(ring_radius(-1, RING_MIN, 1920), None);
}

/// **The ring strikes each unit once, on the first frame it reaches
/// it, and never past 1859** (`Nuke::do_damage@0092bc80`; chapter
/// thirty-seven's four probes, `docs/GOLDEN.md` §46): a nuke landing at
/// (23040, 34560) on 3200 strikes units at 518, 1039 and 1804 on 3217,
/// 3226 and 3239, a fifth at 1870 never, a Barracks at a building
/// distance of 1824 on 3239, and stamps its player. Made to fail with
/// the ring at `max` from its first grown frame (every probe on 3211,
/// and 1870's struck), with each-once dropped, and with `Sim::land`'s
/// nuke arm dropped (no stamp, and the splash walk).
#[test]
fn the_ring_strikes_each_on_the_frame_it_reaches_it() {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(60, 60),
        2,
    );
    s.at_war[0][1] = true;
    s.at_war[1][0] = true;
    let nuke = s.add_unit_type(crate::UnitType {
        hits: 2,
        combat: crate::combat::Profile {
            attack: 1200,
            splash_area: 10,
            uber_size: 1,
            domain: crate::attrition::Domain::Air,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    s.tables.unit_types[nuke].tree = Some(crate::airbase::NUCLEARMISSILE);
    let probe = s.add_unit_type(crate::UnitType {
        hits: 130,
        combat: crate::combat::Profile {
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |s: &mut Sim, ty: usize, p: Pos| {
        let index = i16::try_from(s.units.len()).unwrap();
        let hits = s.tables.unit_types[ty].hits;
        let mut u = crate::Unit::new(0, index, p, hits);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = s.tables.unit_types[ty].kind;
        s.add_unit(u)
    };
    let n = put(&mut s, nuke, Pos::new(19200, 34560));
    s.units[n].on_map = false;
    let probes = [
        put(&mut s, probe, Pos::new(23544, 34680)),
        put(&mut s, probe, Pos::new(23160, 33528)),
        put(&mut s, probe, Pos::new(21240, 34680)),
        put(&mut s, probe, Pos::new(21240, 35064)),
    ];
    let bt = s.add_build_type(crate::build::BuildType {
        x_size: 4,
        y_size: 4,
        ..crate::build::BuildType::default()
    });
    let far = s.add_building(1, Pos::new(25152, 35520), 0);
    s.buildings[far].ty = Some(bt);
    s.buildings[far].hits = 1200;
    s.buildings[far].health = 1200;
    s.buildings[far].active = true;
    s.buildings[far].combat = Some(crate::combat::Profile {
        x_size: 4,
        y_size: 4,
        ..crate::combat::Profile::default()
    });
    let round = crate::combat::Projectile {
        shooter: Obj::Unit(n),
        owner: 0,
        target: None,
        launch: Pos::new(19091, 34561),
        landing: Pos::new(23040, 34560),
        cur_time: 120,
        total_time: 120,
        accuracy: 0,
        angle: crate::movement::Angle(0),
        splash_area: 10,
        num_guys: 1,
        rolling: false,
        missed: false,
        harmless: false,
        air: false,
        sz: 0,
        ez: 0,
        v1z: crate::single::Single::ZERO,
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
    };
    // Through the round's own landing (`Sim::land`'s nuke arm): the
    // round's last frame lays the blast, and strikes nothing where it
    // lands.
    s.projectiles.push(crate::combat::Projectile {
        cur_time: 119,
        ..round
    });
    s.process_projectiles(3200);
    assert!(s.projectiles.is_empty(), "the round is closed");
    assert_eq!((s.nukes.stamp_of(0), s.nukes.used_of(0)), (3200, 1));
    assert_eq!(s.buildings[far].health, 1200, "no splash walk");
    let mut first = [None; 4];
    let mut far_first = None;
    for f in 3200..3260 {
        if f > 3200 {
            s.nuke_do_damage(f);
        }
        for (k, &u) in probes.iter().enumerate() {
            if first[k].is_none() && s.units[u].health < 130 {
                first[k] = Some(f);
            }
        }
        if far_first.is_none() && s.buildings[far].health < 1200 {
            far_first = Some(f);
        }
    }
    assert_eq!(first, [Some(3217), Some(3226), Some(3239), None]);
    assert_eq!(far_first, Some(3239));
    assert_eq!(s.nukes.blasts[0].struck.len(), 4, "each once");
}

/// **The ring never strikes an animal** (item 1591,
/// `Nuke::do_damage@0092bc80`'s `Objects::find_units(…, 1)`, whose
/// last argument keeps the walk to `who < 8` on both its arms): a
/// player's unit and an animal side by side at 518 from ground zero,
/// and only the unit is struck. run710's chicken `9/6`, beside Napata,
/// died in this crate on 1735 with a death draw the original never
/// spent. Made to fail with the owner gate dropped.
#[test]
fn the_ring_never_strikes_an_animal() {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(60, 60),
        2,
    );
    let probe = s.add_unit_type(crate::UnitType {
        hits: 130,
        combat: crate::combat::Profile {
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |s: &mut Sim, who: Player, p: Pos| {
        let index = i16::try_from(s.units.len()).unwrap();
        let mut u = crate::Unit::new(who, index, p, 130);
        u.ty = Some(probe);
        u.on_map = true;
        s.add_unit(u)
    };
    let unit = put(&mut s, 0, Pos::new(23544, 34680));
    let animal = put(&mut s, 9, Pos::new(23544, 34680));
    let shooter = put(&mut s, 1, Pos::new(19200, 34560));
    s.units[shooter].on_map = false;
    s.nukes.blasts.push(Blast {
        at: Pos::new(23040, 34560),
        min: RING_MIN,
        max: 10 * RING_PER_SPLASH,
        start: 3200,
        who: 1,
        shooter: Obj::Unit(shooter),
        struck: Vec::new(),
    });
    for f in 3201..3260 {
        s.nuke_do_damage(f);
    }
    assert!(s.units[unit].health < 130, "the player's unit is struck");
    assert_eq!(s.units[animal].health, 130, "the animal is not");
}

/// **A landing lights its circle for everyone, in both planes, and a
/// hit across players marks both leaders' treaties** (item 1620,
/// `Ammo::do_damage@00678060:268`–`:287`; `Object::do_damage:327`). A
/// nuke of `splash_area` 10 lands at tile (120, 180) of a fogged 60-cell
/// world: the fog cell under it, one five tiles out and one ten tiles
/// out carry `0xff` in `seen` and `seen2` the frame it lands; one thirty
/// tiles out does not; the struck player's `treaties` and the
/// shooter's carry bit 1 from the first ring. Made to fail with the
/// reveal dropped (the first three read 0) and with the treaty write
/// dropped (a hit leaves bit 1 clear).
#[test]
fn a_landing_lights_its_circle_and_a_hit_marks_the_treaties() {
    let mut s = Sim::new(
        crate::tuning::Tuning::RON,
        crate::world::World::new(60, 60),
        2,
    );
    assert!(s.world.set_fog(vec![0; 60 * 60 * 4]));
    s.at_war[0][1] = true;
    s.at_war[1][0] = true;
    let nuke = s.add_unit_type(crate::UnitType {
        hits: 2,
        combat: crate::combat::Profile {
            attack: 1200,
            splash_area: 10,
            uber_size: 1,
            domain: crate::attrition::Domain::Air,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    s.tables.unit_types[nuke].tree = Some(crate::airbase::NUCLEARMISSILE);
    let probe = s.add_unit_type(crate::UnitType {
        hits: 130,
        combat: crate::combat::Profile {
            uber_size: 1,
            ..crate::combat::Profile::default()
        },
        ..crate::UnitType::default()
    });
    let put = |s: &mut Sim, who: Player, ty: usize, p: Pos| {
        let index = i16::try_from(s.units.len()).unwrap();
        let hits = s.tables.unit_types[ty].hits;
        let mut u = crate::Unit::new(who, index, p, hits);
        u.ty = Some(ty);
        u.on_map = true;
        u.kind = s.tables.unit_types[ty].kind;
        s.add_unit(u)
    };
    let n = put(&mut s, 1, nuke, Pos::new(19200, 34560));
    s.units[n].on_map = false;
    put(&mut s, 0, probe, Pos::new(23040, 34560));
    assert_eq!(s.treaties[0][1], 0);
    s.projectiles.push(crate::combat::Projectile {
        shooter: Obj::Unit(n),
        owner: 1,
        target: None,
        launch: Pos::new(19091, 34561),
        landing: Pos::new(23040, 34560),
        cur_time: 119,
        total_time: 120,
        accuracy: 0,
        angle: crate::movement::Angle(0),
        splash_area: 10,
        num_guys: 1,
        rolling: false,
        missed: false,
        harmless: false,
        air: false,
        sz: 0,
        ez: 0,
        v1z: crate::single::Single::ZERO,
        slot: 0,
        flame: false,
        punctures: false,
        spent: false,
        crash: None,
        spline: None,
    });
    s.process_projectiles(3200);
    // Tile (120, 180) is fog cell (60, 90).
    for (fx, fy, want) in [(60, 90, 0xff), (62, 90, 0xff), (65, 90, 0xff), (75, 90, 0)] {
        assert_eq!(s.world.seen2(fx, fy), Some(want), "seen2 ({fx}, {fy})");
        assert_eq!(s.world.seen(fx, fy), Some(want), "seen ({fx}, {fy})");
    }
    assert_eq!(s.treaties[0][1] & 2, 2, "the struck leader's slot");
    assert_eq!(s.treaties[1][0] & 2, 2, "the shooter's slot");
}

/// **The struck fraction is the emulated original's**: 256 at d 0, 243
/// at 100, 187 at 518, 118 at 1039, 16 at 1804, 9 at 1859
/// (`tools/emu/nuke_blast.py`, `Object::do_damage`'s `count`).
#[test]
fn the_struck_fraction_is_the_emulated_original_s() {
    for (d, c) in [
        (0, 256),
        (100, 243),
        (518, 187),
        (1039, 118),
        (1804, 16),
        (1859, 9),
    ] {
        assert_eq!(struck_count(d, 1920), c, "d = {d}");
    }
}
