//! **East Meets West, the held-out style** (`docs/MAPGEN.md` §24, item
//! 3941; DECISIONS 54, 61): `sim::map_east_west`, written from the reading
//! and committed (`004dbbab7`) before run1524 was launched, walked through
//! the §15 chain on run1524's one file — the default lobby at `--map 19`
//! with `--dump-all-start`. Its first comparison is the number: the
//! phases held are pinned (`floors::MAPGEN`'s `EastMeetsWest` row), and a
//! phase that parts is parked with its checkpoint, never fixed against this
//! take. The later phases are the chain's shared code, wired as East
//! Indies' are (the nearest water style); the style file is `EASTWEST.xml`.

use super::*;

const STYLE: i64 = 19;
const XML: &str = "EASTWEST.xml";

/// One East Meets West generator run: the mask from 7805's seed, both
/// seats with no team (the default lobby), `num_sides` the nations.
fn at_7807(
    root: &Path,
    run: GlRun,
    range: i32,
) -> (MapGrid, Draws, sim::map_east_west::EastWestMap) {
    let (seed, nations, xs, ys, map_size) = run;
    let style = lake_style(
        root,
        XML,
        (xs, ys, nations, map_size),
        base_edge_before(seed),
        range,
    )
    .expect("East Meets West's style");
    let mut teams = [None; 8];
    for t in teams.iter_mut().take(nations as usize) {
        *t = Some(sim::map_east_west::NO_TEAM);
    }
    let mut grid = MapGrid::new(xs, ys);
    let mut draws = Draws::new(seed);
    let map = sim::map_east_west::east_west_make_continents(
        &mut grid, nations, nations, &teams, style, &mut draws,
    )
    .expect("a lobby inside its allowance");
    (grid, draws, map)
}

/// The mask's segment of a setup path: from the first draw inside
/// `MapEastMeetsWest::make_continents`, `fill_cont`,
/// `place_start_in_region` or the growers, to the first that is none.
fn mask_part(setup: &[SetupDraw]) -> &[SetupDraw] {
    let inside = |s: &u32| {
        (0x0069_6640..0x0069_7540).contains(s)
            || (0x0068_a960..0x0068_ac00).contains(s)
            || (0x0068_ac00..0x0068_ae50).contains(s)
            || (0x0069_c600..0x0069_d3f0).contains(s)
    };
    let Some(from) = setup.iter().position(|(s, _)| inside(s)) else {
        return &[];
    };
    let len = setup[from..]
        .iter()
        .position(|(s, _)| !inside(s))
        .unwrap_or(setup.len() - from);
    &setup[from..from + len]
}

/// The tileset list whose draw is the 7805 seed stepped back once: the
/// style file has no `TILECHANCE`, so its `init_map_data` draws nothing and
/// the draw before `base_edge`'s is `default.xml`'s (`docs/MAPGEN.md` §4).
fn chances(root: &Path) -> Vec<(String, i32)> {
    let own = tile_chances(root, XML).expect("EASTWEST.xml's tilesets");
    if own.is_empty() {
        tile_chances(root, "default.xml").expect("default.xml's tilesets")
    } else {
        own
    }
}

/// The 7854 grid → `place_all` (7859), as [`super::east_indies_at_7859`]
/// is built.
fn at_7859(root: &Path, g: &StartGrid, run: GlRun, range: i32) -> (MapGrid, Draws, String) {
    let tileset = tileset_before(&chances(root), run.0).expect("a drawn tileset");
    let fert = fertility_of(root, XML, &tileset).expect("its fertility");
    let (mut grid, draws, _) = at_7807(root, run, range);
    let mut regions = Regions::default();
    regions.clear_all(&mut grid);
    regions.find_all(&mut grid);
    grid.fix_diag_land();
    grid.make_coastlines();
    regions.clear_all(&mut grid);
    regions.find_all(&mut grid);
    let fractal = Fractal::init(g.xs, g.ys, fert.clump_factor, 0, g.seed as u32);
    grid.fill_fertile(&fractal, &fertile_partitions(&fert.baseland));
    let inputs = place_all_inputs_of(root, XML, &tileset, run.2, run.3, STYLE as i32)
        .expect("East Meets West's place_all inputs");
    let mut d = Draws::new(draws.rng.seed);
    place_all(&mut grid, &mut d, &inputs);
    (grid, d, tileset)
}

/// What each phase's checks found: `Ok` with what it reached, `Err` with
/// the first checkpoint that parted.
type Verdict = Result<String, String>;

/// `(seed, part 2, part 5, part 6)` ours against theirs at a `map.cpp`
/// line; `None` when the file does not carry it.
fn at_line(g: &StartGrid, line: i64, grid: &MapGrid, d: &Draws, six: bool) -> Option<Verdict> {
    let (_, _, seed, p) = g.at("map.cpp", line)?;
    let ours = (
        d.rng.seed,
        grid.start_part(),
        grid.cell_part(),
        six.then(|| grid.tile_part()),
    );
    let theirs = (*seed, p[2], p[5], six.then_some(p[6]));
    Some(if ours == theirs {
        Ok(format!("{line}"))
    } else {
        Err(format!(
            "{line}: (seed, part 2, part 5, part 6) ours {ours:?} against theirs {theirs:?}"
        ))
    })
}

/// The phases' verdicts, in §15's order, on one file.
fn walk(root: &Path, name: &str, g: &StartGrid, range: i32) -> Vec<(&'static str, Verdict)> {
    let before = g.at("map.cpp", 7805).expect("7805");
    let run = (before.2, g.nations, g.xs, g.ys, g.map_size);
    let mut out: Vec<(&'static str, Verdict)> = Vec::new();
    let all_of = |checks: Vec<Option<Verdict>>| -> Verdict {
        let mut met = Vec::new();
        for c in checks.into_iter().flatten() {
            met.push(c?);
        }
        if met.is_empty() {
            return Err("no checkpoint on file".into());
        }
        Ok(met.join(", "))
    };

    // 1. The mask: the stream against the trace's segment, then 7807.
    let (grid, draws, map) = at_7807(root, run, range);
    eprintln!(
        "{name}: mask {} draws, stalls {}, unplaced {}, avoid_continent {}, centroids {:?}, starts {:?}",
        draws.log.len(),
        map.stalls,
        map.unplaced,
        map.map.avoid_continent,
        map.centroids,
        map.starts
    );
    let run_name = name
        .strip_prefix("gamelog-")
        .and_then(|r| r.split('-').next())
        .unwrap_or("");
    let stream: Verdict = match crate::testenv::dump(&format!("rontrace-{run_name}.log")) {
        None => Err(format!("no rontrace-{run_name}.log")),
        Some(p) => {
            let setup = setup_draws(Path::new(&p))
                .expect("a readable trace")
                .expect("a trace");
            let seg = mask_part(&setup);
            match first_parting(&draws.log, seg) {
                None => Ok(format!("{} draws", seg.len())),
                Some(at) => {
                    let show =
                        |v: &[SetupDraw]| v.get(at).map(|&(s, w)| format!("{s:#x} word {w:#x}"));
                    Err(format!(
                        "stream draw {at} of ours {} / theirs {}: ours {:?}, theirs {:?}",
                        draws.log.len(),
                        seg.len(),
                        show(&draws.log),
                        show(seg)
                    ))
                }
            }
        }
    };
    let mask = stream.and_then(|s| {
        all_of(vec![at_line(g, 7807, &grid, &draws, false)]).map(|c| format!("{s}; {c}"))
    });
    out.push(("mask", mask));

    // 2. The zero-draw passes, 7835..7854.
    let mut grid = grid;
    let fert = match tileset_before(&chances(root), run.0) {
        Some(t) => fertility_of(root, XML, &t),
        None => fertility(root, XML),
    }
    .expect("its fertility");
    let mut regions = Regions::default();
    regions.clear_all(&mut grid);
    regions.find_all(&mut grid);
    grid.fix_diag_land();
    let c7835 = at_line(g, 7835, &grid, &draws, false);
    grid.make_coastlines();
    let c7843 = at_line(g, 7843, &grid, &draws, false);
    regions.clear_all(&mut grid);
    regions.find_all(&mut grid);
    let c7846 = at_line(g, 7846, &grid, &draws, false);
    let fractal = Fractal::init(g.xs, g.ys, fert.clump_factor, 0, g.seed as u32);
    grid.fill_fertile(&fractal, &fertile_partitions(&fert.baseland));
    let c7854 = at_line(g, 7854, &grid, &draws, false);
    out.push(("zero-draw passes", all_of(vec![c7835, c7843, c7846, c7854])));

    // 3. `place_all` and its tail, 7859..7870.
    let (mut grid, mut d, tileset) = at_7859(root, g, run, range);
    let ts = tileset_groups(root, &tileset).expect("the drawn tileset's groups");
    let c7859 = at_line(g, 7859, &grid, &d, false);
    check_player_forest(&mut grid, sim::tuning::Tuning::RON.city_center_radius);
    let c7861 = at_line(g, 7861, &grid, &d, false);
    nubify_forest(&mut grid, &mut d);
    let c7865 = at_line(g, 7865, &grid, &d, false);
    change_bases(&mut grid, &mut d, &ts);
    let c7870 = at_line(g, 7870, &grid, &d, false);
    out.push((
        "place_all and tail",
        all_of(vec![c7859, c7861, c7865, c7870]),
    ));

    // 4. The resources, 7877..7927.
    let c7877 = at_line(g, 7877, &grid, &d, true);
    let c7880 = at_line(g, 7880, &grid, &d, true);
    let inputs = resource_inputs(root, XML, run.2, run.3, run.1).expect("East Meets West's rows");
    place_resources(&mut grid, &mut d, &inputs, run.1);
    let c7927 = at_line(g, 7927, &grid, &d, true);
    out.push(("resources", all_of(vec![c7877, c7880, c7927])));

    // 5. The values, 7937 and 7945.
    let groups = place_all_inputs_of(root, XML, &tileset, run.2, run.3, STYLE as i32)
        .expect("East Meets West's place_all inputs");
    let makes = land_makes(root).expect("rules.xml's LANDS");
    let mut regions = regions_of(&grid);
    adjust_for_blocking(&mut grid, &mut regions, &groups.templates);
    let c7937 = at_line(g, 7937, &grid, &d, true);
    compute_all_vals(&mut grid, &mut d, &makes);
    let c7945 = at_line(g, 7945, &grid, &d, true);
    out.push(("values", all_of(vec![c7937, c7945])));

    // 6. `Terrain::init` and its half-land, 734..5012.
    let sea = sea_map(root, XML).expect("EASTWEST.xml's SEA_MAP");
    let empty = HalfLand::default().parts().expect("the lists start empty");
    let state = |grid: &MapGrid, d: &Draws, lists: [u32; 4]| {
        (
            d.rng.seed,
            grid.start_part(),
            grid.cell_part(),
            grid.tile_part(),
            lists,
        )
    };
    let theirs = |seed: u32, p: &[u32; 16]| (seed, p[2], p[5], p[6], [p[10], p[11], p[12], p[13]]);
    let mut six = Vec::new();
    if let (Some(b), Some(a)) = (g.at("map.cpp", 7945), g.at("setup.cpp", 734)) {
        let want = adler_rewrite(
            b.3[4],
            PART4_LEN,
            PART4_SEA_MAP_AT,
            &WORLD_SEA_MAP_BEFORE.to_le_bytes(),
            &sea_map_after_make(WORLD_SEA_MAP_BEFORE, sea).to_le_bytes(),
        );
        six.push(if want == a.3[4] {
            Ok("734 part 4".to_string())
        } else {
            Err(format!("734 part 4: ours {want} theirs {}", a.3[4]))
        });
    }
    let before_half = state(&grid, &d, empty);
    for (f, l, seed, p) in before_halfland_records(g) {
        let t = theirs(*seed, p);
        six.push(if before_half == t {
            Ok(format!("{f} {l}"))
        } else {
            Err(format!("{f} {l}: ours {before_half:?} theirs {t:?}"))
        });
    }
    let mut coord = coord_flags(&grid);
    let f = sim::map_heights::fractals(grid.xs, grid.ys, g.seed as u32);
    let mut z = sim::map_heights::generate_land(&grid, &coord, &f);
    sim::map_heights::build_coastal_data(&grid, &mut coord, &mut z, g.seed as u32);
    let combo = sim::halfland::Combo {
        coast_base: ts.coast_base,
        num_tiles: [
            Some(ts.base_tiles),
            Some(ts.coastal_tiles),
            Some(ts.ocean_tiles),
            None,
        ],
    };
    let mut depth = 0;
    let lists = match sim::terrain_init::check_and_set_halfland_heights(
        &mut grid, &mut coord, &z, combo, &mut depth,
    ) {
        Ok(l) => l,
        Err(e) => {
            six.push(Err(format!("check_and_set_halfland_heights: {e:?}")));
            out.push(("Terrain::init", all_of(six.into_iter().map(Some).collect())));
            return out;
        }
    };
    let p = lists.parts().expect("the lists' walk");
    let lp = [p[0], p[1], p[2], p[3]];
    let s720 = state(&grid, &d, lp);
    sim::halfland::create_doobers_flags(&mut grid);
    let s738 = state(&grid, &d, lp);
    sim::map_heights::create_doobers(&grid, &mut z);
    let beach = sim::halfland::Pass::new(&mut grid, &mut coord, &z, combo)
        .with_depth(depth)
        .convert_beach_to_sandy();
    let s5012 = state(&grid, &d, lp);
    eprintln!(
        "{name}: {} half-land cells listed, beach {beach:?}, depth {depth}",
        lists.locs.len()
    );
    if let (Some(from), Some(to)) = (
        g.checksums
            .iter()
            .position(|(f, l, _, _)| f == "Terrain.cpp" && *l == 720),
        g.checksums
            .iter()
            .position(|(f, l, _, _)| f == "map.cpp" && *l == 5012),
    ) {
        for (f, l, seed, p) in &g.checksums[from..=to] {
            let ours = match (f.as_str(), *l) {
                ("Terrain.cpp", 720) => s720,
                ("map.cpp", 5012) => s5012,
                _ => s738,
            };
            // Part 2 is not compared from 720 on, as East Indies' walk.
            let (o, t) = ((ours.0, ours.2, ours.3, ours.4), theirs(*seed, p));
            let t = (t.0, t.2, t.3, t.4);
            six.push(if o == t {
                Ok(format!("{f} {l}"))
            } else {
                Err(format!(
                    "{f} {l}: (seed, parts 5, 6, 10–13) ours {o:?} theirs {t:?}"
                ))
            });
        }
    }
    let six_ok = all_of(six.into_iter().map(Some).collect());
    out.push((
        "Terrain::init",
        six_ok.map(|s| format!("{} records", s.split(", ").count())),
    ));

    // 7. The rivers, 5012..5118.
    let thresh = river_thresh(root, XML).expect("RIVER_THRESHOLD");
    let rivers = river_list(root, XML).expect("a RIVERLIST");
    let standard = standard_map_size(root).expect("the Standard map size");
    let mut regions = regions_of(&grid);
    let mut ours = Vec::new();
    let phase = Phase {
        thresh,
        city_radius: sim::tuning::Tuning::RON.city_center_radius,
        rivers: &rivers,
        standard,
        seed: g.seed as u32,
        nations: run.1,
    };
    make_rivers(&mut grid, &mut regions, &phase, &mut d, |l, g, d| {
        ours.push((
            l,
            (d.rng.seed, g.start_part(), g.cell_part(), g.tile_part(), lp),
        ))
    });
    let reached = river_records(g);
    let lines: Vec<i64> = reached.iter().map(|r| r.1).collect();
    let printed: Vec<i64> = ours.iter().map(|o| o.0).collect();
    let seven: Verdict = if reached.is_empty() {
        Err("no 5012..5118 record".into())
    } else if lines != printed {
        Err(format!(
            "make_rivers prints {printed:?}, the file {lines:?}"
        ))
    } else {
        all_of(
            reached
                .iter()
                .zip(&ours)
                .map(|((f, l, seed, p), (_, o))| {
                    let t = theirs(*seed, p);
                    Some(if *o == t {
                        Ok(format!("{f} {l}"))
                    } else {
                        Err(format!("{f} {l}: ours {o:?} theirs {t:?}"))
                    })
                })
                .collect(),
        )
    };
    out.push((
        "rivers",
        seven.map(|s| {
            format!(
                "{s} ({} river{})",
                rivers.len(),
                if rivers.len() == 1 { "" } else { "s" }
            )
        }),
    ));

    // 8. The heights' tail at 894: the troughs, `adjust_for_mountains`,
    // and `Cliffs::init`'s slope arm (no cliff group in `EASTWEST.xml`,
    // so the slope tiles are reported, not run).
    let install = crate::Install::new(root);
    let meshes = crate::mountains::meshes(&install);
    let eight: Verdict = match sim::map_troughs::calculate_all(
        &mut grid,
        &mut coord,
        &mut z,
        ts.riverbase,
        combo,
        depth,
        None,
    ) {
        Err(e) => Err(format!("the troughs: {e:?}")),
        Ok(troughs) => {
            sim::map_heights::adjust_for_mountains(&grid, &mut z, &groups.templates, &meshes);
            let slope = sim::map_heights::slope_tiles(&grid);
            eprintln!(
                "{name}: trough sections {:?}, {} slope tiles",
                troughs.sections,
                slope.len()
            );
            match g.at("Terrain.cpp", 894) {
                None => Err("no 894 on file".into()),
                Some((_, _, seed, p)) => {
                    let o = state(&grid, &d, lp);
                    let t = theirs(*seed, p);
                    if o == t && slope.is_empty() {
                        Ok("894".into())
                    } else if o == t {
                        Err(format!("894 met with {} slope tiles unrun", slope.len()))
                    } else {
                        Err(format!("894: ours {o:?} theirs {t:?}"))
                    }
                }
            }
        }
    };
    out.push(("heights at 894", eight));
    out
}

/// The phases East Meets West's port held on run1524's first comparison
/// (item 3941): the count of §15 phases, in order, before the first that
/// parts. The held-out rule pins this as measured and nothing in `sim` is
/// changed against it.
const EAST_MEETS_WEST_HELD: i64 = 8;

/// **East Meets West on its first take** (`docs/MAPGEN.md` §24, item
/// 3941): the chain walked on every style-19 file carrying 7805 —
/// run1524's one — each phase's checkpoints in §15's order, the phases held
/// before the first parting counted and pinned.
#[test]
fn east_meets_west_on_its_first_take() {
    let (Some(all), Some(root)) = (archive(), crate::testenv::install_root()) else {
        eprintln!("skipped: no kept dumps or no install on this machine");
        return;
    };
    let root = Path::new(&root);
    let range = max_range_349(root);
    let files: Vec<&(String, StartGrid)> = all
        .iter()
        .filter(|(_, g)| g.style == STYLE && g.xs > 0 && g.at("map.cpp", 7805).is_some())
        .collect();
    assert!(
        !files.is_empty(),
        "no East Meets West setup trace on file (run1524)"
    );
    for (name, g) in files {
        let verdicts = walk(root, name, g, range);
        for (phase, v) in &verdicts {
            eprintln!("{name}: {phase}: {v:?}");
        }
        let held = verdicts.iter().take_while(|(_, v)| v.is_ok()).count() as i64;
        eprintln!("{name}: {held} of 8 phases held on the first take");
        assert_eq!(
            held, EAST_MEETS_WEST_HELD,
            "{name}: the held-out count moved"
        );
        let pinned = MAPGEN
            .styles
            .iter()
            .find(|s| s.map == "EastMeetsWest")
            .expect("MAPGEN's EastMeetsWest row");
        assert_eq!(
            (pinned.ported, pinned.phases),
            (EAST_MEETS_WEST_HELD, 8),
            "the pin and the walk"
        );
    }
}
