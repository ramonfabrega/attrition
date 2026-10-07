//! The `BUILDDATA` record: a building's identity, queue and gather list.

/// One field of a building's **production queue** the two sides disagree
/// on — `BuildQueue::log_data`'s own record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueueDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// The field, named as the dump writes it: `queued`, or
    /// `queue[k].<field>` for a slot's.
    pub field: String,
    pub ours: i64,
    pub theirs: i64,
}

/// One field of a building's identity the two sides disagree on — its
/// position, or the type it was created as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// The field, named as `BuildData::log_data` writes it.
    pub field: &'static str,
    pub ours: i64,
    pub theirs: i64,
}

/// One field of a building's **gather record** the two sides disagree on —
/// `BuildData::gather_from` and the `MiningList` header it is written
/// under, plus `gather_down`'s chain head (`docs/ECONOMY.md`, "The spine";
/// `docs/ORDERS.md` §6.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GatherDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// The field, named as the dump writes it.
    pub field: &'static str,
    /// The entry of the mining list this is about, or `−1` for a scalar.
    pub at: i64,
    pub ours: i64,
    pub theirs: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::testkit::*;
    use crate::diff::*;
    use crate::gamelog::Block;

    use crate::testenv::{dump, install};
    use std::collections::BTreeSet;

    /// `docs/AI.md` §5's table, first row: the opening script's first call
    /// at game frame 1 — `defensive` steps 6, 7, 9, 10 in one call (Written
    /// Word researched, the fourth farm placed, City State researched, three
    /// citizens queued) and 11 blocking. The dump's `FRAME 2` is the state
    /// after that frame. The AI's personality and script are run8's
    /// `PERSONALITY` block (`LEADERS=9`), since the harness's stream is not
    /// the original's at `Leader::init`.
    #[test]
    fn run7_s_first_script_call_fills_the_queues_the_dump_shows() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run7-ancient-nubian-orders.txt") else {
            eprintln!("skipping: no gamelog-run7-ancient-nubian-orders.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        assert_eq!(loaded.scripts.len(), 3, "the three shipped scripts");
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        // The script's frame-1 coin is thrown on whatever stream frame 0
        // leaves: the siblings supply the true one (run11's setup trace,
        // run12's end-of-frame words), which is the boom order the original
        // took (`docs/SYNC.md` §5).
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        assert!(built.sim.scripts.is_some(), "the scripts compiled");
        assert!(!built.sim.nation[1].human, "player 1 is the AI");
        let ai = &mut built.sim.ai[1];
        ai.pers = sim::ai::Personality {
            rush: 1,
            cities: 1,
            upgrades: 0,
            arms: -1,
            army: -1,
            army_size: 1,
            raid: 1,
            invade: 1,
            target: 0,
            strategy: 1,
            raze: 0,
            spells: 0,
            forts: -1,
            nukes: 1,
            air: 1,
            naval: 0,
            market: -1,
            scouts: 1,
            civilians: -1,
            early_army: 1,
            friendly_human: 1,
            alliance_human: -1,
            friendly_ai: 1,
            alliance_ai: -1,
        };
        ai.script = Some("defensive".to_string());
        ai.script_live = true;
        ai.script_step = 1;

        // Frame 0 arms the machine; frame 1 is the script's first call.
        built.tick();
        built.tick();
        assert_eq!(built.sim.frame, 2);
        assert!(built.sim.ai[1].script_live, "the script is not done");
        assert_eq!(built.sim.ai[1].script_step, 11, "6 → 7 → 9 → 10 → 11");

        // The city's queue: three citizens, on the ramp the dump shows.
        let city_b = built
            .sim
            .cities
            .iter()
            .find(|c| c.alive && c.owner == 1)
            .map(|c| c.building)
            .expect("the AI's city");
        let citizen = loaded.unit_named("Citizen").expect("a Citizen record");
        let q = &built.sim.buildings[city_b].queue;
        assert_eq!(
            q.items.iter().map(|i| (i.ty, i.tech)).collect::<Vec<_>>(),
            vec![(citizen, None); 3]
        );
        assert_eq!(
            q.items
                .iter()
                .map(|i| i32::from(i.cost[0]))
                .collect::<Vec<_>>(),
            [25, 26, 27],
            "the citizens' food ramp in the dump's FRAME 2"
        );

        // The library's queue: Written Word, then City State.
        let lib = built.sim.first_library(1).expect("the AI's library");
        let tech = |n: &str| loaded.tech_tree[loaded.tech_named(n).unwrap()];
        assert_eq!(
            built.sim.buildings[lib]
                .queue
                .items
                .iter()
                .map(|i| i.tech)
                .collect::<Vec<_>>(),
            [Some(tech("Written Word")), Some(tech("City State"))]
        );

        // The fourth farm: a site placed for the AI this frame, with a
        // citizen ordered onto it. Its tile is the map's to decide
        // (`docs/AI.md` §13); that it exists is the script's.
        let farm = loaded.build_named("Farm").unwrap();
        let sites: Vec<usize> = built
            .sim
            .buildings
            .iter()
            .enumerate()
            .filter(|(_, b)| b.owner == 1 && b.alive && !b.active && b.ty == Some(farm))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(sites.len(), 1, "one farm site");
        assert_eq!(
            i32::from(built.sim.buildings[sites[0]].index),
            2006,
            "the dump's site number"
        );
        let builder = built.sim.units.iter().any(|u| {
            u.owner == 1
                && u.orders
                    .iter()
                    .any(|o| matches!(o.body, sim::orders::Body::Build(b) if b == sites[0]))
        });
        assert!(builder, "a citizen holds the build order");
    }

    /// The coastal ring's guard (audit B4-k): run20 (`gamelog-run20-islands-
    /// dumpall.txt`, 2026-08-25) is the East Indies lobby — `MAP_STYLE 18`,
    /// **`sea_map 4`** — under `DUMP_ALL` for frames 0–3, so it carries its
    /// own setup trace, heights and per-frame words and needs no sibling.
    /// It is the first capture in which `compute_site_stats`' step 5
    /// (`docs/AI.md` §2.13: many landmasses, no dock, an ocean cell on the
    /// radius-5 ring → `base × 30`) is reachable at all; every dump before
    /// it reported `sea_map 1`. The ring is walked from the *original*
    /// sampled cell, not the one the 5×5 slide moved it to — the correction
    /// neither reading had — and this test is the assertion that was owed:
    /// all ten `SITE` records of the AI's frame-1 leader record, slot for
    /// slot, on the original's own stream and heights. Made to fail once by
    /// re-centring the ring on the slid cell before it was landed.
    #[test]
    fn run20_s_islands_sites_walk_the_coastal_ring_from_the_original_cell() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        // Self-sufficient: the setup trace, the heights and the frame words
        // are the run's own, or the assertion below would be on the sim's
        // own luck rather than the original's.
        assert!(
            !init.checksums.is_empty(),
            "run20's setup trace was not read"
        );
        assert!(
            !init.heights.is_empty(),
            "run20's height table was not read"
        );
        assert!(
            !init.frame_seeds.is_empty(),
            "run20's frame words were not read"
        );
        let sea_map = init
            .world
            .iter()
            .find(|(k, _)| *k == "sea_map")
            .and_then(|(_, v)| v.trim().parse::<i32>().ok())
            .expect("the WORLD block's sea_map");
        assert!(sea_map > 2, "an islands map: sea_map {sea_map}");
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        assert_eq!(built.sim.lobby.map_style, 18, "East Indies");
        assert_eq!(
            built.sim.world.sea_map(),
            sea_map,
            "the harness carries the original's `sea_map`"
        );
        // What the first run of this test found: `sea_map` is not a
        // landmass count. The dump's cells fall into twelve land regions
        // and one sea region here, and the original still says 4 — the
        // style's own class (`World::sea_map`).
        let land_regions = (0..built.sim.world.region_count())
            .filter(|&r| built.sim.world.terrain(r as u16) == sim::world::Terrain::Land)
            .count();
        assert!(
            land_regions > 10,
            "twelve islands in the cells, not {land_regions}"
        );
        assert!(
            built
                .notes
                .iter()
                .any(|n| n.contains("heights pinned per tile")),
            "{:?}",
            built.notes
        );

        let theirs = log.leader_block(1, 1).expect("FRAME 1 LEADERDATA who 1");
        assert_eq!(theirs.int("peasants"), Some(5), "the record is the AI's");
        // The fog grid is loaded and is what the original's `was_seen`
        // reads: the AI's city cell carries its bit, and the ten sites
        // below do not — they are seen through the territory arm.
        assert!(
            built.notes.iter().any(|n| n.contains("fog: seen2 loaded")),
            "{:?}",
            built.notes
        );
        assert_eq!(
            built.sim.world.seen2(103, 105),
            Some(2),
            "the AI's city, fog (103,105)"
        );
        assert_eq!(
            built.sim.world.seen2(89, 105),
            Some(0),
            "site (44,52), fog (89,105)"
        );
        built.sim.tick();
        // The per-region census, **every region** — run9's check compares
        // the home region alone, which on one landmass is every land
        // region there is. The site scorer's step 9 multiplies by what
        // `reg_cities`/`reg_land` say about the *site's* region, so an
        // island the AI does not live on is exactly where a census error
        // would hide.
        let c = &built.sim.ai[1].census;
        let arr = |key: &str| -> Vec<i64> {
            theirs
                .all(key)
                .iter()
                .map(|v| v.trim().parse().unwrap_or(0))
                .collect()
        };
        let mut wrong = Vec::new();
        for (key, ours) in [
            ("reg_active", &c.reg_active),
            ("reg_peasants", &c.reg_peasants),
            ("reg_free_peasants", &c.reg_free_peasants),
            ("reg_gatherers", &c.reg_gatherers),
            ("reg_gather_slots", &c.reg_gather_slots),
            ("reg_cities", &c.reg_cities),
            ("reg_pop", &c.reg_pop),
            ("reg_land", &c.reg_land),
            ("strategy", &c.strategy),
        ] {
            let t = arr(&format!("{key}[scan]"));
            // The land arrays are `[64]`, indexed by the land region
            // numbers `0..0x3e`; the sea regions have their own.
            for &(dump_r, sim_r) in built.region_map.iter().filter(|(d, _)| *d < 0x3f) {
                let o = i64::from(sim::ai::Census::reg(ours, sim_r));
                let tv = t.get(dump_r as usize).copied();
                if tv != Some(o) {
                    wrong.push(format!("{key}[{dump_r}]: ours {o} theirs {tv:?}"));
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "the per-region census disagrees:\n  {}",
            wrong.join("\n  ")
        );
        // The `CITY` record, whole: step 13's picture — `ocean`, `land`,
        // `filled`, `dock_tile` (`Sim::is_dock_tile`, `docs/TRANSPORT.md`
        // §5.6; 1 for the AI's city, 0 for the human's), `space[3]` — and
        // steps 2/10's `free`, `busy`, `gatherers`, `peasant_dist`,
        // `in_port`, and `ter[6]` out of `World::gather_at`.
        let mut wrong = Vec::new();
        let mut matched = 0;
        let frame1 = log
            .frames()
            .into_iter()
            .find(|(n, _)| *n == 1)
            .map(|(_, b)| b)
            .expect("FRAME 1");
        let cities = frame1.find("CITIES").expect("FRAME 1 CITIES");
        for rec in cities.kids("CITY").filter(|c| c.int("who") == Some(1)) {
            let (x, y) = (rec.int("x").unwrap_or(-1), rec.int("y").unwrap_or(-1));
            let Some(c) = built
                .sim
                .cities
                .iter()
                .position(|c| c.alive && i64::from(c.pos.x) == x && i64::from(c.pos.y) == y)
            else {
                wrong.push(format!("no city of ours at {x},{y}"));
                continue;
            };
            matched += 1;
            let ours = built.sim.ai[1].city_ai[c];
            let space: Vec<i64> = rec
                .all("space[scan]")
                .iter()
                .map(|v| v.trim().parse().unwrap_or(0))
                .collect();
            let fields = [
                ("ocean", ours.ocean),
                ("land", ours.land),
                ("filled", ours.filled),
                ("dock_tile", ours.dock_tile),
                ("free", i32::from(ours.free)),
                ("busy", ours.busy),
                ("gatherers", ours.gatherers),
                ("peasant_dist", ours.peasant_dist),
                ("in_port", ours.in_port),
            ];
            for (key, o) in fields {
                let t = rec.int(key);
                if t != Some(i64::from(o)) {
                    wrong.push(format!("{key}: ours {o} theirs {t:?}"));
                }
            }
            // `space[0..3]`, whole. `space[0]` and `space[1]` were pinned
            // at ours 48 / theirs 58 for a day: `space_at_corner` walks its
            // sixteen tiles centre-2×2-first (`grid_index_x/y`), so its
            // early-out is on the centre, not the top row, and ≥ 8 blocked
            // with the centre free is 2, not 0 (`docs/AI.md` §15.9). The
            // pin flipped the moment the walk order landed.
            for (i, o) in ours.space.iter().enumerate() {
                let t = space.get(i).copied();
                if t != Some(i64::from(*o)) {
                    wrong.push(format!("space[{i}]: ours {o} theirs {t:?}"));
                }
            }
            // `ter[0..6]`, the per-good best over the circle
            // (`docs/AI.md` §24).
            let ter: Vec<i64> = rec
                .all("ter[scan]")
                .iter()
                .map(|v| v.trim().parse().unwrap_or(0))
                .collect();
            for (i, o) in ours.ter.iter().enumerate() {
                let t = ter.get(i).copied();
                if t != Some(i64::from(*o)) {
                    wrong.push(format!("ter[{i}]: ours {o} theirs {t:?}"));
                }
            }
        }
        assert_eq!(matched, 1, "the AI's one city");
        assert!(
            wrong.is_empty(),
            "the CITY record disagrees:\n  {}",
            wrong.join("\n  ")
        );
        built.sim.tick();
        let theirs_full: Vec<(i64, i64, i64, i64, i64)> = theirs
            .kids("SITE")
            .map(|s| {
                (
                    s.int("wx").unwrap_or(0),
                    s.int("wy").unwrap_or(0),
                    s.int("val").unwrap_or(0),
                    s.int("rank").unwrap_or(0),
                    s.int("dist").unwrap_or(0),
                )
            })
            .collect();
        assert_eq!(theirs_full.len(), 10, "ten SITE records");
        let ours_full: Vec<(i64, i64, i64, i64, i64)> = built.sim.ai[1]
            .sites
            .iter()
            .map(|s| {
                (
                    i64::from(s.wx),
                    i64::from(s.wy),
                    i64::from(s.val),
                    i64::from(s.rank),
                    i64::from(s.dist),
                )
            })
            .collect();
        eprintln!("run20 sites — theirs: {theirs_full:?}\n              ours:   {ours_full:?}");
        let differing: Vec<String> = theirs_full
            .iter()
            .zip(ours_full.iter())
            .enumerate()
            .filter(|(_, (t, o))| t != o)
            .map(|(i, (t, o))| format!("slot {i}: theirs {t:?} ours {o:?}"))
            .collect();
        assert!(
            differing.is_empty(),
            "the ten site records differ (wx, wy, val, rank, dist):\n  {}",
            differing.join("\n  ")
        );
    }

    /// **Where the buildings stand** — run56's `BUILDDATA` position, on
    /// every linked building of every frame.
    ///
    /// `x_internal` and `y_internal` are written at every detail level and
    /// the parser has carried them since the record existed; nothing ever
    /// compared them. What they say is *where the AI put it*, and a
    /// building the AI sites 16 tiles away still links by `(who, o)` and
    /// still agrees on every gather field, so the whole of
    /// `docs/AI.md` §2.20 was uncheckable against a capture until this.
    ///
    /// **92,626 fields, and every one of them right.** The last residue
    /// was one building's `y` — player 1's Dock `o 2010`, laid on frame
    /// 2977 two cells north of the original's — and it closed with the
    /// dock's own sub-position slide (`docs/AI.md` §21): the slide is not
    /// what places this dock, it is what makes the spiral *accept* a cell
    /// three ring-6 candidates earlier, and the stride of three that
    /// engages there lands on the original's own index.
    #[test]
    fn run56_s_buildings_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 3_000,
            "run56's length is {} — a short file here is a wrong file",
            report.frames.len()
        );
        let seen: usize = report.frames.iter().map(|f| f.build_compared).sum();
        let bad: Vec<BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter().copied())
            .collect();
        // Which buildings, and from which frame — the successor's own
        // statement, printed whether or not the assertion below fails.
        let mut first: Vec<(i64, i64, i64, &'static str, i64, i64)> = Vec::new();
        for d in &bad {
            if !first.iter().any(|&(w, o, ..)| (w, o) == (d.who, d.o)) {
                first.push((d.who, d.o, d.frame, d.field, d.ours, d.theirs));
            }
        }
        eprintln!(
            "run56 buildings: {seen} fields compared, {} wrong on {} building(s)",
            bad.len(),
            first.len()
        );
        for &(who, o, frame, field, ours, theirs) in &first {
            eprintln!("  {who}/{o} from f{frame}: {field} ours {ours} theirs {theirs}");
        }
        // **140,491 → 186,804 on item 478**, and the rise is one field:
        // `build_masks & 0x100`, the replan flag, which the reader had
        // been taking off the wrong block and nothing had ever compared
        // (`docs/ROADS.md` §1.2). `bad` stays empty, so the flag's whole
        // life agrees here as well as on Great Lakes. **186,804 → 279,430
        // on item 661**: the `city` slot and the `city_down` link, and
        // `bad` stays empty on both (`docs/AI.md` §63). **279,430 →
        // 372,056 on item 763**: `damage` and `damage_frac`, and `bad`
        // stays empty (parked 728).
        assert_eq!(
            seen, 372_056,
            "the site and the clock on every linked building-frame"
        );
        // **Every building of both players stands on the original's own
        // point for all 3,000 frames** — the pre-placed ones, the farms
        // and the second city this crate sites itself, the camp of item
        // 85, and now the AI's Dock.
        assert!(
            bad.is_empty(),
            "a building stands somewhere the original's does not: {first:?}"
        );
    }

    /// Run13's window (`docs/SYNC.md` §4.1, §5): run10 stepped with run11,
    /// run3 and **run13** as the siblings, so run13's end-of-frame words for
    /// sim-frames 94–103 are installed and the per-frame counts compared.
    /// Pinned: the frames the sim matches outright (98, 102, 103 — six
    /// farm draws and no `do_move` draw for the seven walks that start on
    /// 102 and 103), the two the animation clock owes (100: twelve fish
    /// wraps; 101: the sheep's arrival and the scout's wrap on top of the
    /// farmers' twelve and the farms' seven), and — with the unit loop
    /// rotated so the AI's units go first at frame 101 — the AI's three
    /// farmers' re-target goals, which the dump shows at frame 103.
    #[test]
    fn run13_s_window_counts_and_the_ai_farmers_re_targets_are_matched() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run10-world6-long.txt") else {
            eprintln!("skipping: no gamelog-run10-world6-long.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let texts: Vec<crate::capture::Text> = [
            "gamelog-run11-checksum.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run13-window-95-105.txt",
        ]
        .iter()
        .filter_map(|n| dump(n))
        .map(crate::capture::read)
        .collect();
        if texts.len() != 3 {
            eprintln!("skipping: run11, run3 and run13 are all needed");
            return;
        }
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        assert_eq!(
            inits[2].frame_seeds.first(),
            Some(&(94, 0x5f8f_3d9d)),
            "run13's first word is the end of sim-frame 94"
        );
        assert_eq!(inits[2].frame_seeds.len(), 10);
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, Some(105), None, &refs, None).unwrap();
        let count = |f: i64| -> (Option<u32>, Option<u32>) {
            let &(_, ours, theirs) = report
                .rng_frames
                .iter()
                .find(|(n, _, _)| *n == f)
                .unwrap_or_else(|| panic!("frame {f} was not traced"));
            (ours, theirs)
        };
        // With the animation clock (`docs/ANIM.md` §6): the new citizen's
        // two creation draws at 99, the twelve fish wraps at 100 and the
        // scout's wrap at 101 are the sim's. ~~The one left at 101 is the
        // sheep's arrival, whose walk the sim does not have.~~ **Closed
        // 2026-08-27**: the sheep wandered on an untraced frame, so the
        // harness re-seats gaia's animals from every traced frame's dump
        // (`sim::Sim::reseat_animal`) and its arrival now falls where the
        // original's does — draw 6 of 21, between the AI's three farmers
        // and the human's.
        assert_eq!(count(98), (Some(6), Some(6)));
        assert_eq!(count(99), (Some(8), Some(8)), "the new citizen's two");
        assert_eq!(count(100), (Some(18), Some(18)), "the twelve fish wraps");
        assert_eq!(
            count(101),
            (Some(21), Some(21)),
            "the scout's wrap; the sheep"
        );
        // **102 was the six-frames-late re-target, and it closed with the
        // body** (item 34, 2026-08-27). It had been booked as the stale-fog
        // row when item 32 landed; item 33 revealed cells as units move and
        // it went 24 → 22, which is what sent someone to the trace instead
        // of the theory — and the trace said the movement layer.
        //
        // What it was. Both sides give the AI scout `1/0` the same
        // `EXPLORE_TO`, to `(45048, 19704)`, and both walk it there. On
        // **frame 62** the original stands still for exactly one frame,
        // turning three degrees, and then steps a constant `(−19, +29)`
        // from 63 to the end. The simulation stood for *seven* frames and
        // eased into the heading over eight more, arriving at frame 101
        // where the original arrived at 95 — so its eighteen `think_scout`
        // ring draws landed on 102 where the original's landed on 96.
        //
        // Two things behind one number, and both are the body's
        // (`docs/MOVEMENT.md`, "The body step"). Guy 0 does not chase the
        // unit at eleven eighths; it is **written onto** it, with
        // `last_speed` the Euclidean length of the unit's own step — so the
        // average settles at 33 rather than 46 and divides the turn rate by
        // nine rather than eleven. And a unit that spent its frame turning
        // in place has `last_speed` zero that same frame, which is what
        // arms `guy_flags & 0x10`, the instant turn from a standstill —
        // which no unit in this simulation had, because nothing populated
        // the flag from the type. With both, the scout is on the
        // original's position and both of its angles on every frame from
        // 57 to 91.
        assert_eq!(
            count(102),
            (Some(6), Some(6)),
            "the scout's re-target is on the original's frame"
        );
        assert_eq!(count(103), (Some(6), Some(6)));
        // The farmers re-target on sim-frame 101 and walk from 102; their
        // path goals are compared on the log's frame 103 (the end of
        // sim-frame 102, the walk's first step). ~~The human's three draw
        // one place late (the sheep's arrival is not modelled) and are not
        // pinned.~~ **Both players are pinned now** (2026-08-27): the
        // sheep's arrival is draw 6, and it is the *ordering* that mattered
        // — the AI's six draws come before it and the human's six after, so
        // without it the human's three farmers spent the AI's leftovers and
        // walked one tile wrong on both axes.
        let at_103 = report
            .frames
            .iter()
            .find(|f| f.frame == 103)
            .expect("the log's frame 103");
        for who in 0..=1 {
            for o in 3..=5 {
                let bad: Vec<_> = at_103
                    .order_diverged
                    .iter()
                    .filter(|d| d.who == who && d.o == o)
                    .collect();
                assert!(bad.is_empty(), "farmer {who}/{o} at frame 103: {bad:?}");
            }
        }
    }

    /// Income against the original's own ledger: run8's `FRAME 2`
    /// The first dock, under run22's window (`gamelog-run22-islands-dock-
    /// window.txt`, 2026-08-25: the run21 lobby, `DUMP_ALL` for `[3579,
    /// 3582)`). `docs/TRANSPORT.md` §5, §10, §12 — the docks registry and
    /// the transport level, checked against the original's own records:
    ///
    /// - block 3579 has no active `DOCK` and none of the AI's units carry
    ///   `unit_masks & 0x800000`; block 3580 has one dock and every AI unit
    ///   carries the bit, the human's none (`Leader::check_transport`);
    /// - the `DOCK` record's `reg` is the region of the building's cell in
    ///   the run's own `WORLD` block — the **sea**, so `Dock::init`'s
    ///   `reg < 0x40` guard leaves `reg_docks` untouched;
    /// - a dock placed and activated in the harness at that position takes
    ///   slot 0, records the same region, leaves `reg_docks` at 0, and — the
    ///   AI holding the bonus's prerequisite — grants the civilian level to
    ///   every land unit of its owner and to nobody else's.
    #[test]
    fn run22_s_first_dock_registers_in_the_sea_and_grants_the_level() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run22-islands-dock-window.txt") else {
            eprintln!("skipping: no gamelog-run22-islands-dock-window.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        assert_eq!(built.sim.lobby.map_style, 18, "East Indies");

        let frames = log.frames();
        let block = |n: i64| -> Block<'_> {
            let (_, b) = *frames
                .iter()
                .find(|(f, _)| *f == n)
                .expect("the frame block");
            b.kid("FULL DUMP").unwrap_or(b)
        };
        let sub = |b: Block<'_>, key: &str| -> Option<i64> { b.find("SUBOBJECT")?.int(key) };
        // The unit bits, by owner: (with the bit, without).
        let bits = |b: Block<'_>, who: i64| -> (usize, usize) {
            let mut on = 0;
            let mut off = 0;
            for u in b.kids("UNITDATA") {
                if sub(u, "who") != Some(who) {
                    continue;
                }
                if u.int("unit_masks").unwrap_or(0) & 0x80_0000 != 0 {
                    on += 1;
                } else {
                    off += 1;
                }
            }
            (on, off)
        };
        fn active_docks<'b>(b: Block<'b>) -> Vec<Block<'b>> {
            b.kid("DOCKS")
                .map(|d| {
                    d.kids("DOCK")
                        .filter(|r| r.int("dock_flags").unwrap_or(0) & 1 != 0)
                        .collect()
                })
                .unwrap_or_default()
        }

        let before = block(3579);
        assert!(active_docks(before).is_empty(), "no dock before 3579");
        let (on, off) = bits(before, 1);
        assert_eq!((on, off), (0, 14), "the AI's 14 units before the dock");

        let after = block(3580);
        let docks = active_docks(after);
        assert_eq!(docks.len(), 1, "one dock at 3580");
        let d = docks[0];
        assert_eq!(d.int("dock"), Some(0), "slot 0");
        assert_eq!(d.int("who"), Some(1), "the AI's");
        let o = d.int("o").expect("o");
        let dump_reg = d.int("reg").expect("reg");
        let (on, off) = bits(after, 1);
        assert_eq!((on, off), (14, 0), "every AI unit has the bit at 3580");
        assert_eq!(bits(after, 0).0, 0, "no human unit has it");

        // The building, and its cell in the harness's world.
        let bd = after
            .kids("BUILDDATA")
            .find(|&b| sub(b, "o") == Some(o) && sub(b, "who") == Some(1))
            .expect("the dock's BUILDDATA");
        assert_eq!(bd.int("orig_type"), Some(432), "DOCK");
        let pos = Pos::new(
            sub(bd, "x_internal").unwrap() as i32,
            sub(bd, "y_internal").unwrap() as i32,
        );
        let sim_reg = built
            .sim
            .world
            .region_of(pos.cell())
            .expect("the dock's cell has a region");
        let mapped = built
            .region_map
            .iter()
            .find(|(_, r)| *r == sim_reg)
            .map(|(d, _)| *d);
        assert_eq!(mapped, Some(dump_reg), "the DOCK's reg is the cell's");
        assert_eq!(
            built.sim.world.terrain(sim_reg),
            sim::world::Terrain::Sea,
            "a dock's centre cell is water — the guard skips reg_docks"
        );

        // The same dock in the harness: the AI holds the bonus's
        // prerequisite by frame 201 (run21's `check_transport`), so grant
        // it, then place and finish the dock.
        let preq = built
            .sim
            .tech_tree
            .roles
            .transport_preq
            .expect("Written Word, from rules.xml's third TECHBONUS");
        built.sim.gain_tech(1, preq);
        assert_eq!(
            built.sim.transport_level(1),
            sim::transport::TransportType::None,
            "the prerequisite alone grants nothing"
        );
        let ty = loaded.build_of_type_index(432).expect("the dock type");
        let b = built.sim.add_building(1, pos, 1);
        built.sim.buildings[b].ty = Some(ty);
        built.sim.buildings[b].orig_ty = Some(ty);
        built.sim.buildings[b].started = true;
        built.sim.activate(b, false, true);
        assert_eq!(built.sim.buildings[b].dock_slot, Some(0));
        assert_eq!(built.sim.docks[1].slots[0].reg, Some(sim_reg));
        assert!(
            built.sim.ai[1].census.reg_docks.iter().all(|n| *n == 0),
            "reg_docks stays 0 for a dock in the sea"
        );
        assert_eq!(
            built.sim.transport_level(1),
            sim::transport::TransportType::Civilian
        );
        let ai_units: Vec<usize> = (0..built.sim.units.len())
            .filter(|&u| built.sim.units[u].owner == 1 && built.sim.units[u].alive())
            .collect();
        assert!(!ai_units.is_empty());
        for &u in &ai_units {
            assert!(
                built.sim.units[u].auto_transport,
                "unit {} of the AI has the bit",
                built.sim.units[u].index
            );
        }
        assert!(
            built
                .sim
                .units
                .iter()
                .filter(|u| u.owner == 0)
                .all(|u| !u.auto_transport),
            "the human's units do not"
        );
        assert_eq!(
            built.sim.transport_level(0),
            sim::transport::TransportType::None
        );
    }

    /// **The British Barracks pays an archer, and the archer is three
    /// units** (2026-09-04, item 215).
    ///
    /// `Build::activate`'s high-water block, end to end on the shipped
    /// tables and run53's own lobby: the AI is tribe 11 and the human tribe
    /// 4, so a Barracks finished for the AI pays one Bowmen and the same
    /// Barracks finished for the human pays nothing. What lands is
    /// `Objects::init_unit`'s `uber_size` loop — **three** objects on an
    /// `o_up`/`o_down` list, of which only the head is counted. **Each**
    /// Barracks pays, because the gate is a high-water mark rather than a
    /// first-one flag; what the mark buys is that a *replacement* for one
    /// that died pays nothing.
    ///
    /// Made to fail on purpose first: with the block gone the frame the
    /// word parts is 6612 rather than 6650, and with the loop running once
    /// this reports one unit for the three.
    #[test]
    fn a_british_barracks_pays_one_bowmen_as_three_chained_units() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run53-greatlakes-24k-trace.txt") else {
            eprintln!("skipping: no run53 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let sim = &mut built.sim;
        assert!(sim.nation[1].british, "run53's AI is tribe 11");
        assert!(!sim.nation[0].british, "and the human is tribe 4");

        let barracks = loaded.build_named("Barracks").expect("a Barracks type");
        let bowmen = loaded.unit_named("Bowmen").expect("a Bowmen type");
        assert_eq!(
            sim.unit_types[bowmen].combat.uber_size, 3,
            "UBER_SIZE 3, from the install's own unitrules.xml"
        );
        let pop = sim.unit_types[bowmen].price.pop;
        let archers = |s: &sim::Sim, who: sim::Player| -> Vec<usize> {
            s.units
                .iter()
                .enumerate()
                .filter(|(_, u)| u.alive() && u.owner == who && u.ty == Some(bowmen))
                .map(|(i, _)| i)
                .collect()
        };

        // A spot clear of everything: `init_build` is the original's own
        // `Build::init`, which runs no placement test.
        let spot = |n: i32| Pos::new(4000 + n * 400, 4000);
        let control = sim.muster[1].control;
        let b = sim.init_build(1, barracks, spot(0), false);
        sim.activate(b, false, true);

        let squad = archers(sim, 1);
        assert_eq!(squad.len(), 3, "one Bowmen is three objects");
        assert!(sim.units[squad[0]].captain, "the head is the captain");
        assert_eq!(sim.units[squad[0]].o_up, None);
        assert_eq!(sim.units[squad[0]].o_down, Some(squad[1]));
        assert_eq!(sim.units[squad[1]].o_up, Some(squad[0]));
        assert_eq!(sim.units[squad[1]].o_down, Some(squad[2]));
        assert_eq!(sim.units[squad[2]].o_up, Some(squad[1]));
        assert_eq!(sim.units[squad[2]].o_down, None);
        for f in &squad[1..] {
            assert!(!sim.units[*f].captain);
            assert_eq!(
                i32::from(sim.units[squad[0]].index),
                sim.units[*f].combat.captain,
                "every member reports to the head"
            );
        }
        // Only the head is counted: `Objects::init_unit` hands every unit
        // with an `o_up` straight back to `track_unit_type(·, −1, ·)`.
        assert_eq!(sim.muster[1].by_type[bowmen], 1, "one unit, not three");
        assert_eq!(sim.muster[1].control - control, pop);

        // **Each** Barracks pays — the mark is a high-water mark, not a
        // first-one flag, so the second raises it to two and pays again.
        let b2 = sim.init_build(1, barracks, spot(4), false);
        sim.activate(b2, false, true);
        assert_eq!(archers(sim, 1).len(), 6, "the second Barracks pays too");

        // What the mark buys is that a **replacement** does not. Lose one
        // and build it again and the count is back at two, which is no
        // longer past the mark.
        sim.close_building(b2, false);
        let b3 = sim.init_build(1, barracks, spot(8), false);
        sim.activate(b3, false, true);
        assert_eq!(archers(sim, 1).len(), 6, "a rebuild is not a new one");

        // And the Nubian human is paid nothing at all —
        // `NUBIAN_FREE_CARAVAN` is zero and no other arm is theirs.
        let b4 = sim.init_build(0, barracks, spot(12), false);
        sim.activate(b4, false, true);
        assert!(archers(sim, 0).is_empty(), "the human is not British");
    }

    /// **run63 — the frame the AI's colony site appears, and the citizen
    /// that was waiting for it** (2026-09-02).
    ///
    /// East Indies' word stood at **5592** and the frame was one citizen's:
    /// `1/15` walks to the explore target `think_scout` gave it on 5455,
    /// arrives on 5592 — position for position with the original, all 137
    /// frames of it — and then, on its very first idle frame, re-targets
    /// and walks off. The original's stands there for **seventy-three
    /// frames** and re-targets on 5666.
    ///
    /// The trace said which arm: `Unit::think_scout+0xaba <
    /// Unit::think_peasant+0x2ac` is `think_peasant`'s **no-site** call and
    /// `+0x2ca` its `idle > 6` one (`docs/SCOUT.md` §11.1; the listing at
    /// `005f5a07`/`005f5a25` settles which is which). The original spends
    /// `+0x2ac` on 5455 and `+0x2ca` on 5666 — so between those two frames
    /// one of leader 1's ten sites came to claim the citizen's region, and
    /// a claimed region is worth waiting six idle frames in.
    ///
    /// run63 is the capture that could say so: run58's recipe with the
    /// `[End Frame]` narrowed to `[5430, 5700)` and `LEADERS=9` in it, so
    /// the `SITE` record is on the record for every frame either side of
    /// the parting. Sixteen minutes and 482 MB, and `rngcmp.py` against
    /// run54 is **5,701 frames, zero differing**.
    ///
    /// **What it says.** On 5550 leader 1's tenth site is `(44, 52) val
    /// 403` and on 5593 it is `(34, 33) val 9728 reg 7 dist 2` — the
    /// citizen's own cell, scored the moment it stands there. This crate
    /// scored every cell of that region **zero**, because
    /// `compute_site_stats`' step 2 zeroes the base when
    /// `blocked_site(TOWN, …)` refuses, and `blocked_location` refused
    /// every one of them with `COLONIZE 0x1c`: a first city in a region
    /// where the leader has none is gated on `has_preq(COLONIZE_BONUS
    /// 0x2af)`, which this crate carried as a **nation** flag that nothing
    /// ever set. It is a technology's — the fourth of `rules.xml`'s
    /// `TECHBONUSES`, `preq0 = Coinage` — and the AI's library takes its
    /// Coinage job on 5177 (`RUN58_QUEUE_TAIL`). The bonus lands inside
    /// this window and nowhere earlier, which is why no capture before
    /// this one could have found it. `docs/CITIES.md` §2.6.1's open
    /// question 4 is what it closes.
    ///
    /// With the gate on the technology the word is **5669**.
    ///
    /// What is asserted here is the whole window: every dumped unit stands
    /// where the original's does on every one of its frames, and the ten
    /// `SITE`s of both leaders are compared field for field — the widening
    /// the ledger wanted (`docs/QUEUE.md` item 87), and the record that
    /// carries the finding.
    #[test]
    fn run63_s_window_is_where_the_ai_s_colony_site_appears() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run63-islands-scoutwalk.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run63.log"),
        ) else {
            eprintln!("skipping: no run63 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let frames = report.frames.len();
        assert!(
            frames >= 260,
            "run63's window is [5430, 5700) — {frames} frames is a wrong file"
        );
        assert_eq!(
            report.frames.first().map(|f| f.frame),
            Some(5430),
            "the window opens where the stanza says"
        );

        // **Every unit, every frame of the window.** The citizen the item
        // is about is `1/15`, and its seventy-three idle frames are in
        // here: nothing may leave the original's point before the word.
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let compared: usize = report.frames.iter().map(|f| f.compared).sum();
        eprintln!(
            "run63: {compared} unit-frames over {frames} frames, {} ever off position",
            parted.len()
        );
        for (&(who, o), &frame) in &parted {
            eprintln!("  {who}/{o} parts at {frame}");
        }
        // **Nothing parts, on any frame of the window.** Both of the
        // residues this capture opened with are closed, and each was a
        // mechanic rather than a route:
        //
        // - ~~**`1/18` is five frames ahead.**~~ Closed 2026-09-02. It was
        //   never five lost frames: the AI's Transport Barge is *born* on
        //   5342, two tiles closer to its destination than the original's,
        //   and stayed that far ahead for the rest of its life. The water
        //   it is born on is `UnitType::find_nearby_spot`'s `(-1, -1)`
        //   form, whose collision half is `find_unit_with_radius` and not
        //   the pairwise pair — `docs/ORDERS.md` §10, `docs/TRANSPORT.md`
        //   §6.1, and `run59_s_census_is_where_the_ai_s_timber_goes` is
        //   where it was measured, 88 frames before this window opens.
        // - ~~**`1/17` turns three frames late.**~~ Closed 2026-09-02, and
        //   it was never the turn model: the original's boat is **faster**
        //   from 5552, because leader 1's second Fisherman settles on a
        //   **whale** that frame and `Unit::update_speed`'s one rare arm
        //   puts every naval type at `(WHALES_SHIPS_MOVE + 100)%` —
        //   `myspeed` 38 → 45 on all three boats and 25 → 30 on the barge,
        //   in the dump and now here (`crates/sim/src/rares.rs`). The
        //   apparent late turn was a slower boat reaching its waypoint
        //   later; the twenty-three frames it lost afterwards were East
        //   Indies' word.
        //
        // Pinned at empty, so the day anything parts here the assertion
        // says which unit and on what frame.
        let early: Vec<(i64, i64, i64)> = parted
            .iter()
            .filter(|(_, f)| **f < LONG_WORD_EAST_INDIES)
            .map(|(&(w, o), &f)| (w, o, f))
            .collect();
        assert_eq!(
            early,
            vec![],
            "the window stands where the original's does, on every frame"
        );

        // **The `SITE` record, whole.** Ten slots a leader a frame, five
        // fields apiece — `reg` is left out because the dump's region
        // numbers are the generator's and this crate's are its own
        // (`Built::region_map` is the translation, and this record is
        // keyed by slot rather than by region).
        let mut built = {
            let mut init = log.initial().unwrap();
            borrow_from_siblings(&mut init, &refs);
            borrow_pasture(&mut init, &tr);
            build_sim(&loaded, &init, Tuning::RON)
        };
        let last = report.frames.last().map_or(0, |f| f.frame);
        let mut site_compared = 0usize;
        // (who, slot, field) → (frames wrong, ours and theirs on the first).
        let mut site_bad: std::collections::BTreeMap<(i64, usize, &str), (usize, i64, i64)> =
            std::collections::BTreeMap::new();
        for n in 1..=last {
            built.tick();
            for who in 0..2i64 {
                let Some(block) = log.leader_block(n, who) else {
                    continue;
                };
                let theirs: Vec<Block> = block.kids("SITE").collect();
                if theirs.len() != sim::ai::SITES {
                    continue;
                }
                for (i, t) in theirs.iter().enumerate() {
                    let o = built.sim.ai[who as usize].sites[i];
                    for (field, ours, theirs) in [
                        ("wx", i64::from(o.wx), t.int("wx")),
                        ("wy", i64::from(o.wy), t.int("wy")),
                        ("val", i64::from(o.val), t.int("val")),
                        ("dist", i64::from(o.dist), t.int("dist")),
                        ("rank", i64::from(o.rank), t.int("rank")),
                    ] {
                        let Some(theirs) = theirs else { continue };
                        site_compared += 1;
                        if ours != theirs {
                            let e = site_bad.entry((who, i, field)).or_insert((0, ours, theirs));
                            e.0 += 1;
                        }
                    }
                }
            }
        }
        eprintln!(
            "run63 sites: {site_compared} fields compared, {} shapes wrong",
            site_bad.len()
        );
        for ((who, i, field), (n, ours, theirs)) in &site_bad {
            eprintln!("  who {who} slot {i} {field}: {n} frames, ours {ours} theirs {theirs}");
        }
        assert_eq!(
            site_compared, RUN63_SITE_FIELDS,
            "ten slots, two leaders, five fields, every frame of the window"
        );
        // **The residue, pinned as it stands** rather than filtered out —
        // 7,122 of 27,000, on leader 1 alone (the human's ten slots are
        // empty on both sides and agree on every frame).
        //
        // The slot the item is about is the **tenth**, and it agrees: from
        // 5577 the original holds `(34, 33) val 9728 dist 2` and so does
        // this crate, the same cell at the same score on the same frame.
        // What is left is the four sites the AI has been carrying since
        // long before this window, and their shape is old: run59's census
        // has it 250 frames earlier and nothing compared it until now —
        // `(45, 52)` and `(44, 52)` score twice and four times the
        // original's, and one extra site takes an empty slot the original
        // leaves alone, which drags every `rank` and every slot *order*
        // with it. That is `compute_site_stats`' arithmetic, and it is
        // booked (`docs/QUEUE.md`); this run moved the **region**, not the
        // score.
        //
        // **Item 688 took all 7,122**: the twice and four times were
        // `City::fix_world_vals` (`docs/AI.md` §67), the second city
        // halving and quartering the site values round it, which this
        // crate never did. Every field of both leaders' ten slots now
        // agrees on every frame of the window.
        let wrong: usize = site_bad.values().map(|(n, _, _)| n).sum();
        assert_eq!(
            wrong, RUN63_SITE_WRONG,
            "the site record's standing residue, and it only moves deliberately"
        );
    }

    /// run63's two site counts: what the window holds, and what is still
    /// wrong in it. The first is structural — ten slots, two leaders, five
    /// fields, every frame the `LEADERS=9` window wrote — and the second is
    /// `compute_site_stats`' own residue, which
    /// `run63_s_window_is_where_the_ai_s_colony_site_appears` names.
    const RUN63_SITE_FIELDS: usize = 27_000;
    const RUN63_SITE_WRONG: usize = 0;

    /// **run40 and run41 — the ten `SITE` records, and the territory the
    /// site scorer reads through them (item 113).**
    ///
    /// The census windows the price item captured
    /// ([`run40_s_census_prices_the_ai_s_second_city_at_sixty`]) carry more
    /// than goods: `LEADERS=9` prints `Leader::sites` whole — ten
    /// `{wx, wy, val, reg, dist, rank}` a leader a frame — and
    /// `LeaderData::territory` beside it. Nine tenths of that record had
    /// gone uncompared, and inside it was the whole of Great Lakes' `1/1`.
    ///
    /// **The territory, first, because it is the item.** `World::compute_
    /// reg_territory@006b0bb0` rebuilds its per-player bonus table at the
    /// top of every region pass (lines 125–260) and reads
    /// `data_encrypted->epoch[1] ^ 0x63187` — the **Civic** library level,
    /// the same field `LeaderData::get_city_limit` names — for both
    /// `CIVIC_UPGRADE_TERR` and the Russians' per-step flat bonus. This
    /// crate built that table once, in `add_player`, and never rewrote it,
    /// so the AI's City State never widened its border: **261 cells against
    /// the original's 290**, with the human's 266 exact on every frame of
    /// both windows. [`sim::Sim::player_borders`] reads the level live and
    /// `sync_territory` rebuilds all eight rows, as the original does.
    ///
    /// Twenty-nine cells is not a rounding error in this mechanic, because
    /// `compute_site_stats`' fog test is `WorldData::was_seen@006b53f0` and
    /// its **first** arm is territorial: a cell owned by an ally — `is_ally`
    /// is reflexive — is seen wherever that owner has a city in the region.
    /// Every one of those cells was dark to the site scorer, so its 5×5
    /// slide could not move onto them. On run10's frame 575 the AI samples
    /// cell `(48, 29)`, the original slides it to `(47, 28)` for 6,181 and
    /// this crate could not see `(47, 28)` at all: it slid to `(49, 27)` for
    /// 1,159, and from there the second city went up in the wrong place.
    ///
    /// **What is still wrong, asserted as it stands.** One slot. Where the
    /// original's 5×5 leaves cell `(48, 29)` for `(47, 28)`, this crate
    /// keeps `(48, 29)` — `q = 19` at offset 0 against 17 at offset 1, and
    /// the original's `blocked_town` must be refusing the centre where this
    /// crate's `site_clear` allows it. `BuildTypeData::blocked_site@00636a50`
    /// counts the footprint tiles whose fog half-cell the placer has never
    /// seen and returns `0x24` when more than half of them are dark, with
    /// only a Dock exempt; `blocked_tcoord` here grants visibility
    /// everywhere (`docs/CITIES.md` §11). Landing that rule alone moves
    /// none of these numbers — the AI's own territory is "seen" through the
    /// same territorial arm — so it is booked rather than guessed at. The
    /// one wrong slot drags the `rank` of the four slots it outscores with
    /// it, which is where the rest of the residue comes from.
    #[test]
    fn run40_and_run41_s_sites_and_territory_are_the_original_s() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut ran = 0;
        for (file, lo, hi, fields, floor) in [
            (
                "gamelog-run40-census.txt",
                560i64,
                600i64,
                4_000usize,
                580usize,
            ),
            ("gamelog-run41-census.txt", 770, 800, 3_000, 520),
        ] {
            let Some(path) = dump(file) else {
                eprintln!("skipping: no {file} (set RON_GAMELOG_DIR)");
                continue;
            };
            ran += 1;
            let text = crate::capture::read(&path);
            let log = Log::parse(&text);
            let mut init = log.initial().unwrap();
            borrow_from_siblings(&mut init, &refs);
            let mut built = build_sim(&loaded, &init, Tuning::RON);
            let mut compared = 0usize;
            let mut wrong: Vec<String> = Vec::new();
            let mut terr_compared = 0usize;
            let mut terr_wrong: Vec<String> = Vec::new();
            for n in 1..hi {
                built.tick();
                if n < lo {
                    continue;
                }
                // The live owned-cell count, which is what
                // `LeaderData::territory` is. The census's own
                // `my_team_terr` is a snapshot of it taken on the AI's
                // sweep and lags by up to a sweep, so it is not the field
                // to compare.
                let mut owned = vec![0i64; built.sim.players.len()];
                for y in 0..built.sim.world.height() {
                    for x in 0..built.sim.world.width() {
                        if let sim::world::Owner::Player(p) =
                            built.sim.world.owner(sim::world::Cell::new(x, y))
                        {
                            owned[p as usize] += 1;
                        }
                    }
                }
                for who in 0..2i64 {
                    let Some(block) = log.leader_block(n, who) else {
                        continue;
                    };
                    terr_compared += 1;
                    let theirs = block.int("territory");
                    if theirs != Some(owned[who as usize]) {
                        terr_wrong.push(format!(
                            "frame {n} who {who}: ours {} theirs {theirs:?}",
                            owned[who as usize]
                        ));
                    }
                    let sites: Vec<(i64, i64, i64, i64, i64)> = block
                        .kids("SITE")
                        .map(|s| {
                            (
                                s.int("wx").unwrap_or(0),
                                s.int("wy").unwrap_or(0),
                                s.int("val").unwrap_or(0),
                                s.int("dist").unwrap_or(0),
                                s.int("rank").unwrap_or(0),
                            )
                        })
                        .collect();
                    assert_eq!(sites.len(), 10, "ten SITE records, frame {n} who {who}");
                    for (i, t) in sites.iter().enumerate() {
                        let o = built.sim.ai[who as usize].sites[i];
                        let ours = (
                            i64::from(o.wx),
                            i64::from(o.wy),
                            i64::from(o.val),
                            i64::from(o.dist),
                            i64::from(o.rank),
                        );
                        compared += 5;
                        if ours != *t {
                            wrong.push(format!(
                                "frame {n} who {who} [{i}] ours {ours:?} theirs {t:?}"
                            ));
                        }
                    }
                }
            }
            assert_eq!(
                compared, fields,
                "{file}: ten slots, five fields, two leaders"
            );
            // **The item.** Both players' territory, every frame of the
            // window, exact — 261 against 290 before the Civic level
            // reached the border pass.
            assert!(
                terr_wrong.is_empty(),
                "{file}: {} of {terr_compared} leader-frames hold the original's territory:\n  {}",
                terr_compared - terr_wrong.len(),
                terr_wrong.join("\n  ")
            );
            let bad = wrong.len() * 5;
            eprintln!("{file}: {bad} of {compared} site fields disagree");
            assert!(
                bad <= floor,
                "{file}: the site record fell to {bad} of {compared} disagreeing fields, \
                 the ceiling is {floor}:\n  {}",
                wrong.join("\n  ")
            );
        }
        assert!(ran > 0, "neither census window is on this machine");
    }

    /// **run80 — the gem term, and Great Lakes' territory at 23,999
    /// (item 117).**
    ///
    /// The other territory check above ticks a game from frame 1 and
    /// compares as it goes. That cannot reach the far end of a 24,000-frame
    /// capture — Great Lakes' word is 6862 — so this one is **seeded**: the
    /// world and its regions come from run80's own start dump, the cities
    /// and the two leader rows from the frame under test, and the only
    /// thing computed is the border pass itself. It is a check of
    /// `World::compute_reg_territory`'s table and sweep against
    /// `LeaderData::territory`, not of the game that produced the state.
    ///
    /// **What it establishes.** Player 1 has a Merchant on the map's one
    /// Gems deposit — the frame's `rares_collected[44]` is non-zero at
    /// `{11, 13, 23, 27}` and `known_rares` is 4 — and bit 23 is
    /// `GEMS - BASE_RARE`. With the gem arm wired
    /// ([`sim::Sim::has_rare`], [`sim::Sim::player_borders`]) both players
    /// land on the original's own count, **266 and 568**, on every frame
    /// of the window. Without it player 1 comes out at **525**: the gem is
    /// worth forty-three cells here, which is the number
    /// `docs/ATTRITION.md` had down as unestablished. That second count is
    /// asserted too, so the check cannot pass by accident.
    ///
    /// **Why `rares_collected` and not the `rare` mask.** The frame writes
    /// both, and they agree — `BitMask<44>::log_data` prints
    /// `040128800` for player 1, which is bytes `[0, 40, 128, 8, 0, 0]`
    /// and the same four bits. But it prints each byte with `%u` and no
    /// separator, so a six-byte mask is not uniquely decodable from the
    /// text (`[0, 4, 0, 128, 80, 0]` reads it too). `rares_collected` is
    /// an `int[44]` written one value a line by the *same* walk —
    /// `Leader::calc_gather@006ceee0` clears it at the top and hands it to
    /// `Unit::do_gather` beside `rare_owned` — so it is the same fact in a
    /// form that parses.
    ///
    /// **What it does not establish.** The gem's flat addition and its one
    /// step onto the distance limit fire together, so this frame cannot
    /// separate them; 43 is their sum for two Small Cities at Civic 2.
    /// Nothing here checks *when* the bit arrives, which is
    /// `Leader::calc_gather`'s walk and needs a capture either side of the
    /// Merchant settling.
    #[test]
    fn run80_s_gem_widens_the_ai_s_border_by_forty_three_cells() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run80-greatlakes-latecensus.txt") else {
            eprintln!("skipping: no gamelog-run80-greatlakes-latecensus.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);

        // The window's own `CITY` records. They do not move inside it, and
        // that is asserted rather than assumed: a city founded mid-window
        // would make one seed serve forty frames it did not belong to.
        let window: Vec<(i64, Block)> = log
            .frames()
            .into_iter()
            .filter(|(n, _)| (23_960..24_000).contains(n))
            .collect();
        assert_eq!(window.len(), 40, "run80's window is [23960, 24000)");
        let sited = |f: &Block| -> Vec<(i64, i64, i64)> {
            f.find("CITIES")
                .into_iter()
                .flat_map(|c| c.kids("CITY").collect::<Vec<_>>())
                .map(|c| {
                    (
                        c.int("who").unwrap_or(-1),
                        c.int("x").unwrap_or(0),
                        c.int("y").unwrap_or(0),
                    )
                })
                .collect()
        };
        let cities = sited(&window[0].1);
        assert_eq!(cities.len(), 3, "three cities stand at 23,960");
        for (n, f) in &window {
            assert_eq!(
                sited(f),
                cities,
                "frame {n}: the cities moved inside the window"
            );
        }

        // `build_sim` stands the two **starting** cities up from the start
        // dump; the AI's second is founded during the game, so it is placed
        // here at the position the frame gives it.
        let ty = loaded
            .build_named("Small City")
            .expect("no Small City type");
        for (who, x, y) in &cities {
            let pos = sim::world::Pos::new(*x as i32, *y as i32);
            if built
                .sim
                .cities
                .iter()
                .any(|c| c.alive && c.pos == pos && i64::from(c.owner) == *who)
            {
                continue;
            }
            let b = built
                .sim
                .init_build(u8::try_from(*who).unwrap(), ty, pos, false);
            built.sim.activate(b, false, false);
        }
        assert_eq!(
            built.sim.cities.iter().filter(|c| c.alive).count(),
            3,
            "the window's three cities, and no more"
        );

        // Each frame's own leader row, and the border pass under it.
        let owned = |sim: &sim::Sim| -> Vec<i64> {
            let mut out = vec![0i64; sim.players.len()];
            for y in 0..sim.world.height() {
                for x in 0..sim.world.width() {
                    if let sim::world::Owner::Player(p) =
                        sim.world.owner(sim::world::Cell::new(x, y))
                    {
                        out[p as usize] += 1;
                    }
                }
            }
            out
        };
        let mut compared = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        let mut gemless: Vec<Vec<i64>> = Vec::new();
        for (n, _) in &window {
            for who in 0..2usize {
                let blk = log.leader_block(*n, who as i64).expect("a leader row");
                for (i, e) in blk.all("epoch_get(scan)").iter().enumerate() {
                    built.sim.tech[who].epoch[i] = e.trim().parse().unwrap();
                }
                let mut rare = 0u64;
                for (i, v) in blk.all("rares_collected[scan]").iter().enumerate() {
                    if v.trim() != "0" {
                        rare |= 1 << i;
                    }
                }
                built.sim.ledgers[who].rare = rare;
            }
            built.sim.sync_territory();
            built.sim.settle_borders();
            let ours = owned(&built.sim);
            for (who, mine) in ours.iter().enumerate().take(2) {
                let blk = log.leader_block(*n, who as i64).expect("a leader row");
                compared += 1;
                let theirs = blk.int("territory");
                if theirs != Some(*mine) {
                    wrong.push(format!(
                        "frame {n} who {who}: ours {mine} theirs {theirs:?}"
                    ));
                }
            }
            // The same seed with the gem bit taken back out, which is what
            // this crate computed before the term was wired.
            if gemless.is_empty() {
                for who in 0..2usize {
                    built.sim.ledgers[who].rare &=
                        !(1u64 << (sim::economy::GEMS - sim::economy::BASE_RARE));
                }
                built.sim.sync_territory();
                built.sim.settle_borders();
                gemless.push(owned(&built.sim));
            }
        }
        assert!(
            wrong.is_empty(),
            "{} of {compared} leader-frames hold the original's territory:\n  {}",
            compared - wrong.len(),
            wrong.join("\n  ")
        );
        assert_eq!(compared, 80, "forty frames, two leaders");
        assert_eq!(
            gemless[0],
            vec![266, 525],
            "without the gem the AI's border is forty-three cells short"
        );
    }

    /// **The production queues, whole**, against run39's own record —
    /// every building of both players, every live slot, every field
    /// `BuildQueue::log_data` writes.
    ///
    /// `docs/PRODUCTION.md` is read end to end and was, until this test,
    /// backed by a single hand-transcribed frame (run7's `[25, 26, 27]`
    /// ramp in
    /// [`run7_s_first_script_call_fills_the_queues_the_dump_shows`]). The
    /// dump has carried the record all along and nothing parsed it: the
    /// clock, the charge, the ramp and the handover are all in it, on
    /// every frame, for every building. This is the working agreement's
    /// "when the original dumps a record, diff the whole record", and it
    /// is what says the counter's arithmetic is the original's rather
    /// than merely plausible.
    ///
    /// **What it pins.** The AI's city hall queues its first citizen on
    /// frame 176 and the counter climbs 100 a frame to the `train_time`
    /// 9,750, capping there on 273 and handing the guy over on 274 — the
    /// two draws `docs/SYNC.md` §3.16 left as the word's residue. Three
    /// citizens sit in that queue at costs 25, 26 and 27, so the price
    /// ramp is checked per entry rather than at the head alone, and the
    /// library's research entry is checked beside them.
    ///
    /// **And the library's clock is the science discount's diff.** The AI
    /// queues Written Word and City State at frame 2; both are
    /// `JOB_TIME 200`. Written Word is researched at Science 0 and takes
    /// the full 20,000 hundredths, landing on 201; City State is then a
    /// level behind the player's Science and takes 18,000, landing on 382.
    /// Until `TECH_SCIENCE_SPEEDUP` was applied this simulation charged the
    /// second one 20,000 too and emptied the queue on 403 — twenty frames
    /// of `queued ours 1 theirs 0`, which were twenty of the twenty-one
    /// fields this test used to disagree on. `docs/PRODUCTION.md`
    /// §"The base, and the research step"; queue item 86.
    ///
    /// Only the first `queued` slots are compared: the tail of the array
    /// holds whatever it was last left with (`type −1` on a queue never
    /// used, `type 0` on one that has been), which is not state either
    /// side owns.
    ///
    /// **The loop this test used to own is now [`compare`]'s** (2026-09-01),
    /// so every capture the harness reads gets the queue rather than this
    /// one — which is how the AI Dock's twenty-eight late frames on run58
    /// were found. This test keeps the numbers: run39's own floor, its
    /// ceiling of one, and the field count, which changed only because the
    /// shared loop counts a slot's eight fields singly where this counted
    /// four groups.
    #[test]
    fn run39_s_build_queues_are_the_original_s_clock() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();

        let compared: usize = report.frames.iter().map(|f| f.queue_compared).sum();
        let wrong: Vec<&QueueDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.queue_diverged.iter())
            .collect();
        let frames = report.frames.len();
        let first = wrong
            .first()
            .map_or(report.frames.len() as i64, |d| d.frame);
        eprintln!(
            "run39 queues: {compared} fields over {frames} frames, \
             {} disagree, first at {first}",
            wrong.len()
        );
        for d in wrong.iter().take(24) {
            eprintln!(
                "  f{} {}/{} {}: ours {} theirs {}",
                d.frame, d.who, d.o, d.field, d.ours, d.theirs
            );
        }
        assert!(
            compared >= 40_199,
            "the record is being read: {compared} fields"
        );
        assert!(
            first >= 1851,
            "the queues part at {first}; the floor is 1851"
        );
        assert!(
            wrong.len() <= 1,
            "{} queue fields disagree; the ceiling is 1",
            wrong.len()
        );
    }

    /// **The gather record, whole**, against run39's own — every
    /// building of both players, every entry of every `MiningList`, on
    /// every frame of the capture.
    ///
    /// The list is the *input* `crates/sim/src/gather.rs` surveys the slot
    /// count out of, and nothing had ever compared it: the leader record's
    /// `gather_slots` is checked (the census windows), and the count is
    /// derived from the list, so a wrong count and a wrong list read the
    /// same on the only oracle there was. This is the difference —
    /// `docs/QUEUE.md` item 85, and the working agreement's "when the
    /// original dumps a record, diff the whole record".
    #[test]
    fn run39_s_mining_lists_are_the_gather_record() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run39.log"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let (compared, wrong, first) = gather_verdict("run39", &report);
        assert!(
            compared >= 594_618,
            "the record is being read: {compared} fields"
        );
        // **1851 is the run's own last frame, and its block is the quit's.**
        // The `1850 !quit` interrupts the frame the block belongs to and the
        // end-of-game dump is written into it — 17,201 lines against 15,705
        // on every frame before it — and in that block *the human player's*
        // four buildings carry `gather_down −1` while the AI's carry the
        // same heads they have held since 1845. Four buildings losing every
        // registered gatherer on one frame, on one side only, is a teardown
        // rather than a game, so the four rows are the ceiling and the
        // floor is the frame they sit on: no disagreement inside the game.
        assert!(first >= 1851, "the mining lists part at {first}");
        assert!(
            wrong <= 4,
            "{wrong} gather fields disagree; the ceiling is 4, the quit's own"
        );
    }

    /// **The mine lists its mountain range whole** — Great Lakes' first
    /// mine, and the value diff for item 344 (`docs/ECONOMY.md`, "The
    /// mine's range").
    ///
    /// `BuildTypeData::find_gather_tcoords@0063bdc0` has two arms and only
    /// the timber one was modelled: a mine built during a run got an
    /// **empty** list, which cost the 828 draws its shuffle spends and made
    /// Great Lakes 8382 a 46-against-865 frame. The metal arm takes the
    /// nearest mountain range whole and keeps every tile that is not
    /// `SURFACE_FOREST`, stands on nobody else's territory and is not
    /// already gathered from.
    ///
    /// **What the dump says.** run80's per-frame blocks carry the
    /// `MiningList` header the closing whole-map block does not: player 1's
    /// `2021` reads `mtn 6`, `cliff -1`, `length 207`, and run97's frame
    /// 8383 — the frame after the placement — carries the same 207 tiles,
    /// so the list never moves once it is built. This crate's range is the
    /// eight-connected component of mountain tiles around the nearest one,
    /// **244** tiles, of which exactly 37 carry `SURFACE_FOREST`: 207.
    ///
    /// The comparison is a **set**, and that is not a weakening: the dump's
    /// order is the shuffle's, and the shuffle reads the list the map
    /// generator's own range order produced. This crate walks rows, so the
    /// order it starts from is not the original's and the order it ends
    /// with cannot be either. The count is what the sync stream sees —
    /// `4 × 207` draws — and the count is asserted.
    ///
    /// The second mine is **printed, not asserted**: it is placed past this
    /// map's word, so nothing says the two sides should agree about it. On
    /// the tree that landed this item it agrees anyway — `1/2022`, `mtn 0`,
    /// 232 tiles for 232 — which is the reconstruction answering a range it
    /// was not fitted to.
    #[test]
    fn great_lakes_first_mine_lists_its_mountain_range() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr), Some(late)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            trace("rontrace-run53.log"),
            dump("gamelog-run80-greatlakes-latecensus.txt"),
        ) else {
            eprintln!("skipping: no run53/run80 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let late_text = crate::capture::read(&late);
        let late_log = Log::parse(&late_text);
        // 23,999 is the last frame run80 writes a `BUILDDATA` block for;
        // its closing whole-map block carries no `MiningList` at all.
        let theirs = late_log.frame_builds(23_999);
        let mines: Vec<&crate::gamelog::BuildDump> = theirs
            .iter()
            .filter(|b| b.mtn.is_some_and(|m| m >= 0))
            .collect();
        assert_eq!(mines.len(), 2, "run80's frame 23,999 holds two mines");

        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        borrow_pasture(&mut init, &tr);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        for _ in 0..23_999 {
            built.tick();
        }

        let mut checked = 0;
        for m in mines {
            let want: BTreeSet<(i64, i64)> = m.gather_from.iter().copied().collect();
            let found = (0..built.sim.buildings.len()).find(|&b| {
                let bd = &built.sim.buildings[b];
                bd.alive && i64::from(bd.owner) == m.who && i64::from(bd.index) == m.o
            });
            let got: BTreeSet<(i64, i64)> = found.map_or_else(BTreeSet::new, |b| {
                built.sim.buildings[b]
                    .gather_from
                    .iter()
                    .map(|p| (i64::from(p.x), i64::from(p.y)))
                    .collect()
            });
            eprintln!(
                "run80 23999: mine {}/{} mtn {:?} theirs {} tiles, ours {} \
                 ({} shared)",
                m.who,
                m.o,
                m.mtn,
                want.len(),
                got.len(),
                want.intersection(&got).count()
            );
            // `MiningList::mtn` is `find_nearest`'s placed index, and with
            // the generator's placements in (item 604) this crate names the
            // same range the dump does, wherever it built the mine.
            if let Some(b) = found {
                let bd = &built.sim.buildings[b];
                let ty = bd.ty.expect("a typed mine");
                let corner = built.sim.tile_corner(ty, bd.pos);
                let site = built.sim.footprint_centre(ty, corner);
                let ours = built.sim.nearest_placed(site).map(|(_, i, _)| i as i64);
                assert_eq!(ours, m.mtn, "mine {}/{}: the range index", m.who, m.o);
            }
            // `1/2021` is placed on 8382, below this map's word, so it is
            // the one the two sides must agree about.
            if (m.who, m.o) != (1, 2021) {
                continue;
            }
            checked += 1;
            assert_eq!(m.gather_from.len(), 207, "run80's own list length");
            assert_eq!(
                got.len(),
                want.len(),
                "mine {}/{}: {} tiles against the dump's {}",
                m.who,
                m.o,
                got.len(),
                want.len()
            );
            assert_eq!(got, want, "mine {}/{}: the tiles themselves", m.who, m.o);
        }
        assert_eq!(checked, 1, "run80's frame 23,999 has no mine 1/2021");
    }

    /// **A mine's reach is measured to the nearest solid mountain cell**,
    /// and run144's packet is the oracle (`docs/AI.md` §59, §60).
    ///
    /// East Indies' word 10582 is `produce_building`'s spiral placing Mine
    /// `1/2018` for city `1/2007`: twelve friendless candidates drew here
    /// against the original's seven. `BuildTypeData::blocked_site` was run
    /// on the packet (logger frame 10582, taken before the tick that
    /// places) at every one of them. The five extras were all refused
    /// `NoMountain` (12), because `MountainsData::find_nearest` measures
    /// **1536** to the nearest `solid_mount` cell's centre, past the
    /// `gather_radius · 0xc0 = 1152` reach, where this crate measured to
    /// the nearest mountain *tile*, inside it.
    ///
    /// The eight rows are the packet's `find_nearest` distance at eight
    /// mine sites, as world-unit centres, and this crate's
    /// [`sim::Sim::find_nearest_mountain`] must give each exactly. Item 597
    /// closed the first (by measuring to cells) and pinned the last four as
    /// a residue: the stand-in's membership rule, "the centre tile is a
    /// mountain", admitted `(42, 45)` and `(41, 46)`, which reached them at
    /// 1152 and 768. Item 604 laid the generator's placed templates down
    /// and all eight agree; with the stand-in's rule back, the four part
    /// again.
    #[test]
    fn east_indies_10582_mine_sites_measure_to_the_nearest_solid_cell() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
        ) else {
            eprintln!("skipping: no run54/run38 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let built = build_sim(&loaded, &init, Tuning::RON);
        assert_eq!(built.sim.mountains.len(), 18, "run38's placed mountains");
        let reach = sim::gather::MINE_RADIUS * sim::world::UNITS_PER_TILE;
        // (site, the packet's `find_nearest` distance)
        let rows: [((i32, i32), i32); 8] = [
            ((38784, 37248), 1536),
            ((34176, 34944), 1152),
            ((34176, 35136), 984),
            ((32256, 36864), 576),
            ((33408, 34176), 1536),
            ((32640, 34176), 1536),
            ((31104, 35712), 1536),
            ((31104, 36480), 1536),
        ];
        for ((x, y), theirs) in rows {
            let site = sim::Pos::new(x, y);
            let got = built.sim.find_nearest_mountain(site).map(|(d, _, _)| d);
            assert_eq!(
                got,
                Some(theirs),
                "({x}, {y}): the packet measures {theirs}"
            );
            let kept = built.sim.nearest_mountain_cell(site).map(|(d, _)| d);
            assert_eq!(kept, (theirs <= reach).then_some(theirs), "({x}, {y})");
        }
    }

    /// run144's packet's solid lists (`docs/AI.md` §59.3), where item 597
    /// left them: `~/ron-data/lab-experiments/2026-09-23-item-597/
    /// solid-mount-cells.json`, or `$RON_PACKET_SOLID`. Outside git, like
    /// the packet. Each entry is `(placed index, template, loc, cells)`.
    /// One packet range: `(placed index, template, loc, solid cells)`.
    type PacketRange = (i64, i64, (i64, i64), Vec<(i64, i64)>);

    fn packet_solid_cells() -> Option<Vec<PacketRange>> {
        let path = std::env::var("RON_PACKET_SOLID").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{home}/ron-data/lab-experiments/2026-09-23-item-597/solid-mount-cells.json")
        });
        let text = std::fs::read_to_string(path).ok()?;
        // `[{"i": 0, "type": 14, "loc": [51, 46], "cells": [[50, 43], …]}, …]`:
        // every object's integers in order are i, type, loc and the pairs.
        let ints = |t: &str| -> Vec<i64> {
            t.split(|c: char| !(c.is_ascii_digit() || c == '-'))
                .filter_map(|w| w.parse().ok())
                .collect()
        };
        text.split('{')
            .skip(1)
            .map(|o| {
                let v = ints(o);
                (v.len() >= 4 && v.len() % 2 == 0).then(|| {
                    let cells = v[4..].chunks(2).map(|p| (p[0], p[1])).collect();
                    (v[0], v[1], (v[2], v[3]), cells)
                })
            })
            .collect()
    }

    /// **The solid cells are the templates' own, and the templates give
    /// all of the packet's** (`docs/AI.md` §60).
    ///
    /// run38's `DUMP_ALL` head prints the generator's eighteen placed
    /// mountains on East Indies (`GameLog::dump_mountains`); each one's
    /// template, read from the install's `TEMPLATE_TEX` alpha as
    /// `MountainRange::init` reads it, is laid at its location. run144's
    /// packet copied the original's own `solid_mount_wx`/`_wy` for every
    /// range, 107 cells, and this compares them **in the original's list
    /// order**, range by range. It was made to fail on purpose by reading
    /// the image bottom row first, the file's own origin: 3 of 3 template
    /// lists part.
    #[test]
    fn the_templates_give_the_packet_s_solid_cells() {
        let Some(inst) = install() else { return };
        let Some(sib) = dump("gamelog-run38-islands-start.txt") else {
            eprintln!("skipping: no run38 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let Some(packet) = packet_solid_cells() else {
            eprintln!(
                "skipping: no run144 packet solid list \
                 (~/ron-data/lab-experiments/2026-09-23-item-597/solid-mount-cells.json \
                 or RON_PACKET_SOLID)"
            );
            return;
        };
        let templates = crate::mountains::templates(&inst);
        assert_eq!(
            templates.len(),
            16,
            "effects_graphics.xml's sixteen MOUNTAINs"
        );
        let text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let init = log.initial().expect("run38 is a start dump");
        assert_eq!(init.mountains.len(), 18, "East Indies' placed mountains");
        assert_eq!(packet.len(), 18, "the packet's placed mountains");
        let mut cells = 0;
        for (k, (m, (i, ty, loc, want))) in init.mountains.iter().zip(&packet).enumerate() {
            assert_eq!((k as i64, m.t, (m.x, m.y)), (*i, *ty, *loc), "range {k}");
            let got: Vec<(i64, i64)> = templates[m.t as usize]
                .solid
                .iter()
                .map(|&(dx, dy)| (m.x + i64::from(dx), m.y + i64::from(dy)))
                .collect();
            assert_eq!(&got, want, "range {k} (template {}, loc {loc:?})", m.t);
            cells += got.len();
        }
        assert_eq!(cells, 107, "the packet's 107 solid cells");
    }

    /// **The placed templates' tiles are the map's mountain tiles**, on
    /// both maps: the union over every placed range of its template's
    /// tiles at `4 · loc` is exactly the set of tiles the start dump marks
    /// `OBJECT_MOUNTAIN`. This needs no packet — the generator's own
    /// `add_mountain` stamps those tiles from the same lists — so it is the
    /// check that the tile rule and the placement frame are right wherever
    /// a range lies, Great Lakes' included, which no packet has read.
    #[test]
    fn the_placed_templates_tile_the_map_s_mountains() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        for (name, ranges) in [
            ("gamelog-run38-islands-start.txt", 18),
            ("gamelog-run12-dumpall-seeds.txt", 13),
        ] {
            let Some(path) = dump(name) else {
                eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
                continue;
            };
            let text = crate::capture::read(&path);
            let log = Log::parse(&text);
            let init = log.initial().expect("a start dump");
            let built = build_sim(&loaded, &init, Tuning::RON);
            let sim = &built.sim;
            assert_eq!(sim.mountains.len(), ranges, "{name}: placed ranges");
            let placed: BTreeSet<(i32, i32)> = sim
                .mountains
                .iter()
                .flat_map(|m| m.tiles.iter().map(|t| (t.x, t.y)))
                .collect();
            let mut marked = BTreeSet::new();
            for y in 0..sim.world.height() * sim::world::TILES_PER_CELL {
                for x in 0..sim.world.width() * sim::world::TILES_PER_CELL {
                    let t = sim::Pos::new(x, y);
                    if sim.world.tile_mask(t) & sim::world::tile::OBJECT
                        == sim::world::tile::OBJECT_MOUNTAIN
                    {
                        marked.insert((x, y));
                    }
                }
            }
            let only_placed: Vec<_> = placed.difference(&marked).take(8).collect();
            let only_marked: Vec<_> = marked.difference(&placed).take(8).collect();
            assert!(
                only_placed.is_empty() && only_marked.is_empty(),
                "{name}: {} placed tiles, {} marked; placed only {only_placed:?}, \
                 marked only {only_marked:?}",
                placed.len(),
                marked.len()
            );
        }
    }

    /// **`Build::find_gather_tiles` re-derives the original's own list.**
    ///
    /// run39's two camps are pre-placed, so their `gather_from` comes
    /// straight from the dump — 73 tiles apiece — and until now nothing in
    /// this crate could have produced one. This clears the `0x1000` marks a
    /// camp's own tiles carry and runs [`sim::Sim::gather_tcoords`] at its
    /// corner: the walk has to come back with **exactly** the same tiles,
    /// as a set, on a map neither the walk nor the survey has seen.
    ///
    /// The order is a separate question and this cannot answer it — the
    /// dump's order is the shuffle's, and the shuffle needs the stream the
    /// camp was built on, which for a pre-placed building is the map
    /// generator's rather than the game's.
    #[test]
    fn find_gather_tiles_rederives_run39_s_camp_lists() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib)) = (
            dump("gamelog-run39-islands-longtrace.txt"),
            dump("gamelog-run38-islands-start.txt"),
        ) else {
            eprintln!("skipping: no East Indies capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let camps: Vec<usize> = (0..built.sim.buildings.len())
            .filter(|&b| !built.sim.buildings[b].gather_from.is_empty())
            .collect();
        assert_eq!(camps.len(), 2, "run39's two woodcutter's camps");
        for b in camps {
            let (ty, who, pos) = {
                let bd = &built.sim.buildings[b];
                (bd.ty.expect("a typed camp"), bd.owner, bd.pos)
            };
            let theirs = built.sim.buildings[b].gather_from.clone();
            // The camp's own tiles are marked, so the walk would refuse
            // every cell of them; unmarking is what makes this the survey
            // the original ran before the camp existed.
            for &t in &theirs {
                built
                    .sim
                    .world
                    .clear_tile_bits(t, sim::gather::GATHERED_FROM);
            }
            let corner = built.sim.tile_corner(ty, pos);
            let ours = built.sim.gather_tcoords(ty, who, corner);
            let (mut a, mut c) = (ours.clone(), theirs.clone());
            a.sort_unstable_by_key(|p| (p.x, p.y));
            c.sort_unstable_by_key(|p| (p.x, p.y));
            assert_eq!(
                a.len(),
                c.len(),
                "player {who}'s camp: the walk found {} tiles, the dump has {}",
                a.len(),
                c.len()
            );
            assert_eq!(a, c, "player {who}'s camp: the tiles themselves");
            for &t in &theirs {
                built.sim.world.set_tile_bits(t, sim::gather::GATHERED_FROM);
            }
        }
    }

    /// The same widening on **run56**, East Indies' 3,000-frame capture —
    /// the first one long enough to hold a camp the *game* built.
    ///
    /// run39 and run33 both stop at 1,851, and both maps' camps are
    /// pre-placed, so their lists are the dump's own by construction and
    /// agreeing costs the simulation nothing. run54's word parts at **2176**,
    /// on a frame the original spends 192 draws filling a new camp's list,
    /// and this capture is run39's game carried past it at run39's detail —
    /// which makes the frame's `BUILDDATA` the first record that can say
    /// whether `Build::find_gather_tiles` produces the original's list rather
    /// than merely the original's *tiles* (`docs/ECONOMY.md`, "The gather
    /// list, and its shuffle").
    ///
    /// **Closed 2026-09-01**: this simulation now builds that camp too, on
    /// the original's own frame and tile, so every gather record of the
    /// capture agrees except the four the quit's own frame carries.
    #[test]
    fn run56_s_mining_lists_reach_past_the_word() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 3_000,
            "run56's length is {} — this is the long East Indies capture, \
             and a short file here is a wrong file",
            report.frames.len()
        );
        // What the original holds and this simulation does not, building by
        // building — the successor's own statement, printed whether or not
        // the assertion below is what fails.
        let mut missing: Vec<(i64, i64, i64)> = Vec::new();
        for f in &report.frames {
            for (who, o) in f
                .gather_diverged
                .iter()
                .filter(|d| d.field == "length" && d.ours == 0 && d.theirs > 0)
                .map(|d| (d.who, d.o))
            {
                if !missing.iter().any(|&(w, b, _)| (w, b) == (who, o)) {
                    missing.push((who, o, f.frame));
                }
            }
        }
        for (who, o, frame) in &missing {
            eprintln!("  run56: {who}/{o} has a list from frame {frame}; this one has none");
        }
        let (compared, wrong, first) = gather_verdict("run56", &report);
        assert!(
            compared >= 1_048_118,
            "the record is being read: {compared} fields"
        );
        // **Nothing is missing.** The camp `1/2009` used to stand here as
        // the one list the original had and this simulation did not; item
        // 126 closed it, and the count above rose by the 80,850 fields the
        // camp's own frames contribute.
        assert!(
            missing.is_empty(),
            "the original has a list this one does not: {missing:?}"
        );
        let strays: Vec<&GatherDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.gather_diverged.iter())
            .filter(|d| d.frame < report.frames.len() as i64)
            .collect();
        assert!(
            strays.is_empty(),
            "{} gather rows before the quit's own frame: {:?}",
            strays.len(),
            &strays[..strays.len().min(4)]
        );
        assert!(first >= 3001, "the mining lists part at {first}");
        assert!(
            wrong <= 4,
            "{wrong} gather fields disagree; the ceiling is 4 — the quit's \
             own frame, where four farms hold a `gather_down` the dump has \
             as −1"
        );
    }

    /// **The shuffle, seed-anchored** — the whole of
    /// `Build::find_gather_tiles` against the one camp the *game* built.
    ///
    /// run56's frame 2176 is where the original places player 1's second
    /// Woodcutter's Camp, `o 2009`, and spends 192 draws shuffling its
    /// 48-tile list. ~~This crate's AI does not place it (that is the
    /// successor, and the queue holds it)~~ — item 126 closed that on
    /// 2026-09-01 and this crate's AI now places it in the tick, which is
    /// what [`run56_s_mining_lists_reach_past_the_word`] checks. This test
    /// keeps the *seed-anchored* form, which is stronger: it does not
    /// depend on the AI reaching the frame at all, because the stream is
    /// knowable on its own — run56's trace gives
    /// `game_random`'s word at `do_frame` entry of 2176, and
    /// `find_gather_tiles` is the **first** thing that frame draws.
    ///
    /// So: step to 2176, install the original's word, place the camp where
    /// the original placed it, and compare the list it comes back with —
    /// **in order**, entry for entry — against the dump's. That is the walk,
    /// the marking, the shuffle's round count and the shuffle's arithmetic
    /// in one assertion, and the order is the half
    /// [`find_gather_tiles_rederives_run39_s_camp_lists`] cannot reach.
    #[test]
    fn run56_s_new_camp_is_this_crate_s_own_shuffle() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        const FRAME: i64 = 2176;
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let sib_text = crate::capture::read(&sib);
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        // The original's own record for the camp, from the first frame that
        // carries it — its position, its type and its list.
        let theirs = log
            .frames()
            .into_iter()
            .filter(|(n, _)| *n > FRAME)
            .find_map(|(_, b)| {
                crate::gamelog::records(b, false)
                    .1
                    .into_iter()
                    .find(|d| d.who == 1 && d.o == 2009)
            })
            .expect("run56 carries player 1's o 2009");
        assert_eq!(
            theirs.gather_from.len(),
            48,
            "the camp's list is 48 tiles, and 4 × 48 is the frame's 192 draws"
        );
        let word = tr
            .frames
            .iter()
            .find(|(n, _)| *n == FRAME)
            .map(|&(_, w)| w)
            .expect("run56's trace carries frame 2176");
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        while built.sim.frame < FRAME {
            built.tick();
        }
        built.sim.rng.seed = word;
        let ty = loaded
            .build_of_type_index(theirs.orig_type.expect("orig_type") as i32)
            .expect("the camp's build type");
        let b = built.sim.init_build(1, ty, pos_of(theirs.pos), false);
        assert_eq!(
            i64::from(built.sim.buildings[b].index),
            2009,
            "find_free hands out the original's own object number"
        );
        let ours: Vec<(i64, i64)> = built.sim.buildings[b]
            .gather_from
            .iter()
            .map(|p| (i64::from(p.x), i64::from(p.y)))
            .collect();
        eprintln!(
            "run56 camp 1/2009: ours {} tiles, theirs {}; draws {:?}",
            ours.len(),
            theirs.gather_from.len(),
            draws_between(word, built.sim.rng.seed)
        );
        assert_eq!(
            draws_between(word, built.sim.rng.seed),
            Some(4 * theirs.gather_from.len() as u32),
            "the shuffle is four rounds a tile, one draw each"
        );
        assert_eq!(
            ours.len(),
            theirs.gather_from.len(),
            "the walk found {} tiles, the original {}",
            ours.len(),
            theirs.gather_from.len()
        );
        assert_eq!(
            ours, theirs.gather_from,
            "the shuffled list, entry for entry"
        );
        assert_eq!(
            built.sim.buildings[b].gather_max,
            Some(built.sim.max_gatherers(b)),
            "gather_max was recomputed from the filled list"
        );
    }

    /// The same widening on **Great Lakes** — run33's own record, which has
    /// no pasture and a different AI opening, so it is the second map's
    /// independent word on the same claim.
    #[test]
    fn run33_s_mining_lists_are_the_gather_record() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(tr)) = (
            dump("gamelog-run33-longtrace.txt"),
            trace("rontrace-run33.log"),
        ) else {
            eprintln!("skipping: no run33 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let (compared, wrong, first) = gather_verdict("run33", &report);
        assert!(
            compared >= 584_712,
            "the record is being read: {compared} fields"
        );
        assert!(first >= 1772, "the mining lists part at {first}");
        assert!(wrong <= 4, "{wrong} gather fields disagree");
    }

    /// The gather record's verdict, printed the same way for either map:
    /// how many fields were compared, how many disagreed, the first frame
    /// one did, and the first few rows.
    fn gather_verdict(tag: &str, report: &Report) -> (usize, usize, i64) {
        let compared: usize = report.frames.iter().map(|f| f.gather_compared).sum();
        let unlinked: usize = report.frames.iter().map(|f| f.build_unlinked).sum();
        let rows: Vec<&GatherDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.gather_diverged.iter())
            .collect();
        let first = rows.first().map_or(report.frames.len() as i64, |d| d.frame);
        eprintln!(
            "{tag} gather: {compared} fields over {} frames, {} disagree, \
             first at {first}, {unlinked} unlinked building-frames",
            report.frames.len(),
            rows.len()
        );
        for d in rows.iter().take(16) {
            let at = if d.at < 0 {
                String::new()
            } else {
                format!("[{}]", d.at)
            };
            eprintln!(
                "  frame {} {}/{} {}{at}: ours {} theirs {}",
                d.frame, d.who, d.o, d.field, d.ours, d.theirs
            );
        }
        (compared, rows.len(), first)
    }

    /// 268 was on a stream that was not the original's.
    #[test]
    fn run10_s_opening_trains_the_original_s_citizens_on_its_frames() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run10-world6-long.txt") else {
            eprintln!("skipping: no gamelog-run10-world6-long.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        // The sync stream from run11's trace and the heights from run3 —
        // the same map, seed and lobby.
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, None).unwrap();
        assert_eq!(report.frames.len(), 1772);

        // **The headline.** Phase 3's score is ticks before divergence, and
        // this is the longest capture there is, so this is the number. It is
        // pinned as a **floor** so that it can only go up: a session that
        // lowers it fails here before it reaches any sub-score below, and a
        // session that raises it moves the floor and adds a line to the
        // history. `first_divergence` is the breakdown the single score
        // hides — one unit the simulation cannot yet drive pins the score
        // while every other unit may be tracking to the end.
        //
        // History:
        //   2026-08-27  ticks 3, orders 2; player 0 @ 103, player 1 @ 4
        //               (item 34 landed; the first pin)
        //   2026-08-27  ticks 99, orders 102; player 0 @ 103, player 1 @ 100
        //               (item 25: the tile choice's access filter). Player
        //               1's woodcutter was being sent to a tile ringed by
        //               its own forest, which the original never considers,
        //               and the walk it queued for it was the frame-3 second
        //               order the simulation did not have. With the filter
        //               the two sides pick the same tree, and the score is
        //               no longer any one unit's: what is left at 100 and
        //               103 is a whole cohort at once — the AI's citizen
        //               `1/6` on 100, and player 0's three farmers on 103.
        //   2026-08-27  ticks 102, orders 102; **both players @ 103**
        //               (item 43: `come_out`'s exit ring). A trained unit
        //               was being put on its trainer's centre tile; the
        //               original builds it there, walks it inside and lets
        //               it out onto a ring five tiles clear of the wall.
        //               Every citizen run10's AI trains now appears where
        //               the original puts it, and the only frame either
        //               player still parts on is 103 — the farm re-target,
        //               which is one mechanic and not five units.
        //   2026-08-27  ticks 122, orders 122; player 1 @ 123, player 0 @
        //               182 (item 44). Frame 103 was **not** the farm
        //               re-target: the arithmetic was right and the AI's
        //               three farmers already matched. It was two other
        //               things, one per player.
        //               *Player 1's* `1/6` walked to a spot 48 units off
        //               because `find_nearby_spot` rejected the camp's own
        //               tile as blocked. A building's blocked bits are a
        //               **per-tile template** — `masks.txt`, named by the
        //               graphic (`docs/DATALAYER.md`) — and a Woodcutter's
        //               Camp blocks nothing at all, where this crate had
        //               been blocking every non-flat footprint whole.
        //               *Player 0's* three farmers spent the wrong draws:
        //               frame 101's stream runs AI farmers, **a sheep's
        //               arrival**, human farmers, and the sheep had
        //               wandered on an untraced frame the sim's stream
        //               cannot reach. The harness re-seats gaia's animals
        //               from every traced frame's dump; the draw fell back
        //               into place and 101 went 20/21 → 21/21.
        //   2026-08-27  ticks 170, orders 166; player 1 @ 171, player 0 @
        //               182 (item 46: **unit collision**, `docs/COLLISION.md`).
        //               `1/6` walked into `1/3` at frame 122. The original
        //               detects it on the 48-cell occupancy bitmask, names
        //               the other unit off the world cell's object chain,
        //               finds the corner rule does not let them slip past,
        //               snaps the walker onto its own cell centre and
        //               re-plans on the 48-grid. Every field of that is in
        //               the dump and every one of them now matches: the
        //               five `collide*` fields, `coll_x`/`coll_y`, the
        //               position, and all seven path entries. `1/6` went
        //               from 123 to 208, and what pins player 1 now is
        //               `1/1`'s gather `dist_mod` at 167.
        //   2026-08-27  ticks 181, orders 168; player 1 @ 203, player 0 @
        //               182 (item 47: **the AI builder does not keep what
        //               it built**). `Unit::do_build`'s two gather
        //               predicates and `Unit::do_repair`'s each carry a
        //               `unit_masks & 0x40000` term this crate did not
        //               have: only a **human** builder adopts the site it
        //               has just finished. `1/1` finished its farm on frame
        //               167 and this simulation put it on that farm; the
        //               original sent it back through `build_done`, whose
        //               AI arm searches afresh and picked the Woodcutter's
        //               Camp `2001` — the same camp, the same tile
        //               `(212, 93)`, the same `dist_mod 4`. The first
        //               gather-tile disagreement went 169 → 430.
        //   2026-08-28  ticks 185, orders 168; player 1 @ 203, player 0 @
        //               186 (item 50: **the bird**, `docs/SYNC.md` §3.9).
        //               Frame 96's sampling hatches one, and from 104 a
        //               live bird spends three `Animal::think_bird` draws
        //               every eighth frame that this crate was not
        //               spending — twenty-four of them between the last
        //               traced word and the frame `1/1` picks its tile.
        //               The order score does not move with it: at the tile
        //               draw the stream is still ten draws short, and
        //               `1/1`'s `wait` went 581 → 476 against 460. Six of
        //               the ten are the bird's own animation, which needs
        //               a length no dump carries.
        //   2026-08-28  ticks 181, orders **180**; player 0 @ 182, player 1
        //               @ 203 (item 52: **the bird's wing beat**,
        //               `docs/SYNC.md` §3.9). The length no dump carries is
        //               in the install: `WILDBIRD` plays *Bird Soar* for
        //               `CHAR_WALK` and *Bird Flap* for `CHAR_JOG`, and the
        //               `.bha` files say 31 frames and 23
        //               (`rondata::artdata`). With them the bird spends the
        //               hatch frame's wrap, `do_air_physics`'s birth coin
        //               and every later one, and the ledger went 184 → 192.
        //
        //               **The two scores met, and that is why `ticks` reads
        //               lower.** The residue is one unit and it did not
        //               change: `0/3`'s order list goes wrong when its
        //               gather tile is picked off a stream that is still
        //               short, and that pick moved **169 → 181**. What
        //               moved with it is when the wrong order starts
        //               *moving* the unit — at 169 it did not for another
        //               seventeen frames, so `ticks` read 185 while the
        //               orders had already parted; now the position follows
        //               the order by one frame, as it should. A `ticks`
        //               above `orders` is the accident, not the gain.
        //   2026-08-28  ticks **202**, orders 168; player 0 @ **213**,
        //               player 1 @ 203 (item 55: **the road, and the
        //               heights of another game**). Two things landed
        //               together. `borrow_from_siblings` was taking
        //               `master_land_heights` from run3 — the same seed,
        //               style and size, a different `GAME_RULES`, and a
        //               grid that differs on 237 corners because its
        //               starting buildings terraformed elsewhere — so the
        //               road search, a third of whose cost is the climb
        //               term, had been reading another game's terrain.
        //               With the map's own heights the search is the
        //               original's node for node on every capture that
        //               shows it a frame's own world (run14's 10 and 11,
        //               run32's 104–107), and the road it lays on fresh
        //               ground is the original's tile for tile, so
        //               `Sim::plan_roads` came on.
        //
        //               **`orders` fell, and it is the same kind of luck
        //               `ticks` used to be.** Frames 10 and 11 now spend
        //               the 468 draws they always should have, so the
        //               stream is the original's through frame 17 rather
        //               than through 9 — and every value after the new
        //               divergence at 18 is a *different* wrong value.
        //               `0/3`, which pinned the old 180, now holds to 213;
        //               what pins 168 is `1/1`'s gather wait at 169, off
        //               by ten, which the old stream happened to land on.
        //               The number that is not luck is in the ledger test:
        //               the first frame whose draws differ at all, 10 → 18.
        //   2026-08-28  ticks **190**, orders **185**; player 0 @ 191,
        //               player 1 @ 203 (item 59: **the builder's own
        //               animation**). `Unit::do_build`'s step 4 and
        //               `Unit::do_repair`'s first line put the worker on
        //               `CHAR_BUILD` / `CHAR_SOW` / `CHAR_REPAIR`, which
        //               this crate never modelled — so its builders stayed
        //               on `CHAR_WALK` and spent an arrival stand
        //               (`Guy::move+0x19f`) the original does not spend.
        //               One draw, on run14's frame 18, and it was the whole
        //               of the residue: the traced stream now parts at
        //               **99** rather than 18 and matches **219** of 284
        //               frames rather than 173.
        //
        //               **`orders` rose and `ticks` fell, and the two are
        //               the same 81 frames.** `1/1`'s gather `wait` at
        //               frame 169 — what pinned `orders` at 168 — is now
        //               the original's, because the draws between 18 and 99
        //               are. What pins 185 is `0/4`, a human farmer whose
        //               re-target moved 220 → 186: its `wait` was already
        //               wrong at 220 and the shape of the disagreement is
        //               unchanged (a `MOVE_TO` in front of a gather the
        //               original never re-issues), so it is the same defect
        //               on a different frame. Every other unit held or
        //               improved — `1/1` went 577 → 647 — and run6's
        //               totals fell from 2,591/1,613 to 1,588/1,415.
        //   2026-08-28  ticks **192**, orders 185; player 0 @ **193**,
        //               player 1 @ 203 (item 49: **the blocked stand**).
        //               `Unit::move_step:281` asks for `CHAR_DEFAULT` the
        //               moment a step is refused, *before* the three
        //               give-up tests, and this crate skipped the call —
        //               so a blocked unit spent no idle roll where the
        //               original spends one. It could not be made until
        //               `Animal::do_idle`'s own `detect_unit_collision`
        //               was: gaia's sheep stand shoulder to shoulder, and
        //               without that gate `8/1` wandered off on a frame
        //               the original's has not moved on in 120, taking a
        //               blocked stand of its own at 112 and parting the
        //               word *earlier*. With both, run14's word runs
        //               122 → **185** and 219 → **235** of 284 frames
        //               match. What parts 185 is item 61's farmer.
        //   2026-08-28  ticks **200**, orders **200**; player 0 @ **213**,
        //               player 1 @ 201 (item 61: **the farmer's cell
        //               index**). `FarmStruct::status` is a `uchar[4][4]`
        //               indexed `status[dx][dy]` — `Farms::grow`'s own
        //               addressing — and `do_farm` read `status[dy][dx]`.
        //               The transpose is invisible for a hundred frames,
        //               because every starting farmer stands on `(2, 2)`;
        //               from frame 101, when six of them re-pick a cell,
        //               all six sow the wrong one and ripen on the wrong
        //               frame. run14's word runs 185 → **201** and 235 →
        //               **251** of 284 frames match, and the trace's
        //               re-targets now agree to the draw on 101, 199, 211
        //               and 217.
        //
        //               **Player 1's own number fell, 203 → 201**, and it
        //               is the newly-correct 199 that exposes it: the
        //               AI's `1/4` re-picks the cell it is *standing on*,
        //               and the original's move there is refused by a
        //               collision (`collide_o 2` in run10's frame-201
        //               record) and killed without a step, so it re-picks
        //               again on 201. Ours paths and walks. That is the
        //               successor item, and the first divergence now.
        //   2026-08-28  ticks **207**, orders **206**; player 0 @ **326**,
        //               player 1 @ **208** (item 63: **the waypoint's own
        //               collision test**). `Unit::do_move`'s waypoint take
        //               — the block that runs once per leg, on the frame
        //               the waypoint is first read off the path stack —
        //               ends with a `detect_unit_collision` at the
        //               waypoint that this crate did not make, and a
        //               **final** waypoint under a `GATHER`, `ATTACK` or
        //               `BUILD_AT` action that another unit is standing on
        //               kills the whole move outright (`docs/ORDERS.md`
        //               §4.4). `1/4` re-picks farm cell `(0, 3)` on 199,
        //               where its sibling `1/2` is already working; the
        //               original names `1/2` on `collide_o`/`collide_who`,
        //               kills the walk without a step, and `do_farm` picks
        //               `(2, 1)` on 201 instead. Ours pathed and walked
        //               off across the farm.
        //
        //               **Player 0 moved 213 → 326 with it**: the human
        //               farmers `0/3` and `0/5` had been parting at 213 on
        //               a walk to a cell that was not the original's, and
        //               they now hold to 325. What parts them at 326 is a
        //               re-target the original has made a frame before
        //               this simulation makes its own, onto a different
        //               cell again. What parts player 1 is `1/6` at 208:
        //               the original's woodcutter collides on 206 and
        //               repaths onto a seven-entry stack this simulation
        //               does not build.
        //   2026-08-28  ticks **209**, orders **208**; player 0 @ 326,
        //               player 1 @ **210** (item 64: **`is_flat`, the
        //               fence on `resolve_unit_collision`'s step 2**).
        //               The step that kills a walk because the unit is
        //               standing inside its own gather target's footprint
        //               asks that target's type `+0x94` —
        //               `BuildTypeData::is_flat`, `build_flags &
        //               0x10000000` — and this crate read the virtual as
        //               true (a stated seam, `docs/COLLISION.md` §9). So
        //               a woodcutter that collides while standing on its
        //               *camp's* footprint killed its own walk and re-made
        //               it on the next frame, for ever: run10's `1/6`
        //               pushed and killed the same `MOVE_TO` every other
        //               frame from 206 to the end of the capture. With the
        //               fence it falls through to the repath and builds
        //               the original's seven-entry stack, field for field
        //               — and the **whole collision block goes to zero
        //               disagreements** over 48,790 field-frames, the
        //               sticky `collide_guy` included. `1/6` parts at 253
        //               rather than 208; what parts player 1 now is `1/7`
        //               at 210, the citizen trained on frame 206.
        //   2026-08-28  ticks **252**, orders **252**; player 0 @ 326,
        //               player 1 @ **253** (item 66: **`find_nearby_spot`'s
        //               collision half**). Every walk an order makes ends
        //               at a point the ring-and-bearing sweep returns, and
        //               the sweep's last test — `Objects::find_collision`
        //               then `Objects::find_ordered_collision`, both
        //               against the unit as "me" — had never been
        //               implemented, so the first *passable* candidate won
        //               whether or not somebody was standing on it. The
        //               AI's new citizen `1/7`, sent to its camp on frame
        //               208, was given the exact quarter-tile `1/6` was
        //               standing on; the original refuses that and the six
        //               bearings behind it and lands seven cells further
        //               south. With the test the two agree, and `1/6` and
        //               `1/7` both hold to 253.
        //
        //               **run14's trace is now spent.** Its word — the
        //               per-frame draw count — used to part at 232; it now
        //               runs to the end of all 284 frames, 282 of which
        //               match draw for draw, and gaia's bird hatches on
        //               the original's 96, 192 and 256 and on no frame of
        //               its own. Nothing on disk can say where the
        //               simulation next parts by site, which is item 38.
        //   2026-08-29  ticks **322**, orders **320**; player 0 @ **356**,
        //               player 1 @ **323** (item 68: **the think tail
        //               conscripted the woodcutters**). `Unit::think`'s
        //               tail at `005f7615` is `if (!is_supply &&
        //               !is_hero) { … return } add_to_army(this)` —
        //               `docs/SCOUT.md` §2 had the listing right all
        //               along — and `Sim::think_join_army` also joined
        //               "an attacker that is not a scout or a caravan",
        //               which is `docs/ARMY.md` §4's prose for the
        //               *other* caller, `think_attack@005f5a80:155`,
        //               grafted onto the wrong site. A citizen has an
        //               attack, so run10's AI woodcutters `1/6` and
        //               `1/7` joined army 0 on frames 99 and 205, and
        //               leader 1's army 0 ticks at `frame ≡ 252 (mod
        //               256)`: on **252** `do_forming` sent both on a
        //               siege attack across the map. The original never
        //               calls `add_to_army` **at all** in this game —
        //               `Unit::add_to_army@005f7740` and
        //               `Army::add_unit@006f9f40` are absent from
        //               run33's 6,703 entered functions over 1,850
        //               frames — and every citizen's dumped `group` is
        //               −1 on every frame it exists, against the scout's
        //               65 and then 64.
        //
        //               The second half is the same listing's line
        //               `005f7195`: a worker whose `think_peasant`
        //               **found something** returns there, and the
        //               return value was being dropped.
        //
        //               `1/6` went 253 → 735 and `1/7` 253 → 937.
        //               **Player 0 moved with them, 326 → 356**, because
        //               the stream it shares is the original's for
        //               seventy frames more: item 65's farmer re-target
        //               at 326 is gone, `0/3` and `0/5` now hold to 450
        //               and 455, and what pins player 0 is `0/4`'s farm
        //               walk at 356 (its path goal parts at 351, one
        //               tile north-west of the original's). What pins
        //               player 1 is the AI's ninth citizen `1/8` at
        //               321: its gather order names the Woodcutter's
        //               Camp `2001` with `dist_mod 4` where the original
        //               names `2006` with `dist_mod 0` — a **farm** —
        //               so `find_gather_spot`'s choice is the successor.
        //   2026-08-29  ticks **355**, orders **350**; player 0 @ 356,
        //               player 1 @ **363** (item 70: **the gather score
        //               is cap headroom, not distance**).
        //               `find_gather_spot@005f5170`'s numerator is not
        //               the rate it was taken for: over the six goods the
        //               leader has and the building gathers, it is
        //               `resource_cap[g] − income[g]` — the unused part
        //               of the commerce cap, both sixteenths, skipped
        //               entirely while `over_cap[g]` is set. Both
        //               candidates for `1/8` sat in the same distance
        //               bucket (`1958 / 0xc0` and `2041 / 0xc0` are both
        //               10), so the tie was the whole question, and with
        //               four woodcutters against three farmers the food
        //               headroom is the larger. `1/8` takes the farm,
        //               parts at 506 rather than 323, and what pins
        //               player 1 now is the **scout** `1/0` at 363.
        //               Player 0 is unmoved at 356 and is the headline:
        //               item 71, `0/4`'s farm walk.
        //
        //               The rest of the function came with it — the
        //               `tregion` gate, the city-crossing rule at
        //               `:108`, `is_gathering_at` in place of the
        //               gatherer chain, and the strict `local_20 < score`
        //               that makes a zero-scoring building unpickable.
        //   2026-08-29  ticks **362**, orders **361**; player 0 @ **464**,
        //               player 1 @ 363 (item 71: **the idle variant's
        //               length, from the install**). `0/4`'s farm walk
        //               was never `do_gather`'s: the two `% 4` draws it
        //               takes on frame 349 came off a stream four words
        //               ahead of the original's, and what had put them
        //               there was an animation length nothing knew. The
        //               human scout rolled `CHAR_IDLE1` on frame 284 and
        //               this crate played it as `CHAR_DEFAULT`, because
        //               `Art::lengths` came out of dumps and no dump had
        //               ever shown that slot. `unit_graphics.xml` has the
        //               whole table, and
        //               `GraphicPieces::init_piece_ranges`' four strides
        //               say which piece each `<UNIT>` entry is
        //               (`crate::artdata::piece_lengths`, 1,359 of them).
        //               With the lengths in, `0/4` takes the original's
        //               cell and **player 0 goes 356 → 464**.
        //
        //               Two more came with it, both exposed by the first.
        //               The carrying walk is `unit_masks & 0x78000000`,
        //               not the gather order's `goto_build`: the walk a
        //               citizen makes to its camp before it has ever
        //               reached a tile is the plain `CHAR_WALK`, and it
        //               ends in the arrival stand `Guy::move+0x19f`.
        //               And `man_walk.bha` carries thirty-one keys where
        //               its own count says thirty, so the reader's
        //               "the keys fill the chunk" test had been throwing
        //               away every citizen's walk.
        //
        //               What pins the headline now is player 1's
        //               **scout** `1/0` at 363, unmoved by this item.
        //   2026-08-30  ticks **436**, orders **427**; player 0 @ 450,
        //               player 1 @ **437** (item 76: **the scout's
        //               surface probe read the wrong tile**). The cell
        //               filter's `TData` read (`docs/SCOUT.md` §7) had
        //               been transcribed as `(4x, 4y + 2)` from a
        //               decompiled `+4` that is **two elements** of a
        //               two-byte record, not a field offset: the listing
        //               at `005f6542` is `movb 0x4(%eax,%ecx,2)` over
        //               `ecx = (4y + 2)·tile_xs + 4x`, so the tile is the
        //               cell **centre**, the same one `invalid_loc` is
        //               handed two lines later. On run33's frame 361 the
        //               AI scout `1/0` re-targets; cell `(48, 23)` is
        //               ocean at tile `(192, 94)` and land at
        //               `(194, 94)`, so the candidate the original takes
        //               was refused — twenty-six draws against
        //               twenty-seven — and the scout went to `(48, 20)`.
        //               With the centre tile it takes the original's
        //               cell: `1/0` goes 363 → **484**, and eight of the
        //               twelve compared units improve
        //               (`1/5` 421 → 663, `1/3` 470 → 583, `0/4`
        //               464 → 577).
        //
        //               **Player 0 falls 464 → 450**, and it is
        //               downstream: run33's word now parts at **432**
        //               (from 361), and every unit that fell — `0/5` at
        //               450, `0/3` at 455, `1/4` at 437 — parts after
        //               that frame, on a stream that is nobody's. What
        //               parts the word at 432 is a gather stand, and it
        //               is the successor item.
        let ticks = report.ticks_before_divergence();
        let orders = report.order_ticks_before_divergence();
        let first: Vec<i64> = report
            .first_divergence
            .iter()
            .map(|&(_, f)| f.unwrap_or(i64::MAX))
            .collect();
        eprintln!(
            "run10: ticks {ticks}, orders {orders}, first divergence {:?}",
            report.first_divergence
        );
        // The breakdown behind those two numbers: which unit parts when.
        // It is what says whether an item moved the whole or only the
        // unit it was about, and every history line below quotes it.
        eprintln!("run10 by unit: {:?}", report.first_divergence_by_unit());
        eprintln!(
            "run10 orders by unit: {:?}",
            report.order_divergence_by_unit()
        );
        // 2026-08-30, the third steer (Fable): items 81 and 74 had moved
        // orders 576 → 776 and player 0's first divergence 687 → 802 and
        // left this line where it was; the queue carried the numbers and
        // the assertion did not. Raised to what the run prints.
        //
        // 2026-08-31, item 69: ticks **572 -> 781** and player 1 **573 ->
        // 782**, on `do_move`'s `goto STEP` (`docs/ORDERS.md` §4.4). The
        // jump the straight-line check takes when it succeeds lands past
        // the pause check, so a unit that re-verifies its line under a
        // collision pause **steps that frame and does not tick the
        // pause**. `1/2` is the case, field for field in the dump: the
        // collision at 572 writes `pause 3` and `coll_x/coll_y`, 573 steps
        // with the pause still 3, and 574–576 are the three still frames.
        // Ticking on 573 put this simulation one frame ahead for the rest
        // of that walk and parted the position at 573. The order score
        // holds at 776 under a **wider** comparison — the same item added
        // the whole `MOVEORDER` row — and East Indies goes to 1374/1373.
        //
        // 2026-08-31, item 84: ticks **781 -> 910**, player 0 **802 ->
        // 1154** and player 1 **782 -> 911**, on the farm stand's byte
        // (`docs/ORDERS.md` §6.5). The switch reads the guy's live
        // `cur_anim`; this crate kept a flag of its own, so a farmer
        // whose reap animation had been replaced by a blocked stand
        // re-picked a tile where the original sowed. Orders hold at 776
        // — `1/1`'s move at 777 is untouched — and every one of the
        // thirteen units parts later, the median by three hundred
        // frames.
        //
        // 2026-08-31, item 112: player 0 **1154 -> 1385**, and **ticks and
        // orders do not move at all** — the herd's wander centre took the
        // word 986 -> 1372 and this map's headline stayed at 910/776, so
        // for the first time since the two were tied the tick score is no
        // longer sitting on the word's own parting. What holds it is one
        // unit: `1/1` parts at **911**, four hundred and sixty frames
        // before the word does, and every other unit of the fourteen parts
        // at 1375 or later. That unit, not the word, is Great Lakes' next
        // item. Six of the fourteen moved and all six later (`0/3` 1154 ->
        // 1385, `0/4` 1227 -> 1462, `0/5` 1191 -> 1409, `1/5` 1359 ->
        // 1379, `1/8` 1376 unchanged, `1/10` 1552 unchanged).
        //
        // 2026-08-31, item 113: ticks **910 -> 1375**, orders **776 ->
        // 791**, player 1 **911 -> 1376**, and this map's headline is a
        // fifth of a game further in. `1/1` is **gone from the by-unit
        // list entirely** — it never parts on position across the whole
        // 1,772 — and every other one of the fourteen parts on the frame
        // it did before, to the frame. Two defects, and the second was
        // only reachable once the first was fixed.
        //
        // The first is the AI's borders. `World::compute_reg_territory`
        // rebuilds its per-player bonus table at the top of every pass and
        // reads the **Civic** level out of it; this crate built the table
        // once in `add_player`. The AI's City State never widened its
        // border, so it held 261 cells against the original's 290 — and
        // `WorldData::was_seen`'s first arm is territorial, so those
        // twenty-nine cells were dark to `compute_site_stats`. Its 5×5
        // slide could not reach `(47, 28)` and the second city went up in
        // the wrong place
        // ([`run40_and_run41_s_sites_and_territory_are_the_original_s`]).
        //
        // The second is `find_wpath`'s `scouting` mode, and it took the
        // word *down* to 786 on its own before it was found. `scouting`
        // prices seen ground at `0x400` against unseen `8` — the whole of
        // "exploration seeks the unexplored" — and it is set only when the
        // **type** is a scout (`role & 0x10`, `is(SCOUT)` on land,
        // `is(BARK)` at sea) *and* the order is `EXPLORE_TO`. This crate
        // tested the order alone, so the citizen the AI sends to its city
        // site under an `EXPLORETO` took a ten-cell detour west through
        // the fog where the original walks seven south-east. No capture
        // had ever exercised it: until the borders were right, no
        // non-scout in either game was given an explore order over any
        // distance.
        //
        // What now holds the order score at 791 is `1/1` again, and it is
        // one field: on frame 792 the original's `coll_x/coll_y` is
        // `(40539, 18258)` and this crate's `(40632, 18044)` — the same
        // walk, a different collision point.
        //
        // 2026-08-31, item 115: orders **791 -> 1374**, and it is one
        // line. `detect_unit_collision` writes `coll_x/coll_y` **into the
        // order**; the original then walks the rest of `move_step` on a
        // *pointer* to that order, so the write is simply there, while
        // this crate steps on a copy and every `store_move` below put the
        // stale pair back. The arm that shows it is the blocked stand a
        // unit takes while it still owes a turn (`docs/COLLISION.md` §5):
        // it stores and returns without stepping. `1/1` probed
        // `(40539, 18258)` on the original's own frame and this crate
        // threw the answer away.
        //
        // Nothing else moved: **ticks hold at 1375**, both first
        // divergences hold, every one of the fourteen units parts on the
        // frame it did, and run33's word and sequence hold at 1372 with
        // its two totals at 1467/1436. What holds the order score now is
        // `1/8`, whose move order's `x` is `40440` here against `40248`
        // at **1375** — the same unit that holds the tick score at 1376,
        // and both are past the word.
        //
        // 2026-08-31, item 114: ticks **1375 -> 1772**, orders **1374 ->
        // 1772**, and **neither player diverges at all** — the by-unit
        // list is empty on positions and carries nothing that scores on
        // orders. This capture is in lockstep for its whole length.
        //
        // One mechanic did it, and it is one call: `Build::activate`
        // reaches `Farms::add_animals` for every food gather building it
        // completes, so the AI's fourth farm — a pasture — stocks five
        // animals on frame 1372 for twenty draws, and five more follow
        // that frame as the newborns' clocks wrap. This crate spent none
        // of the twenty-five. run33's word goes 1372 -> **1802**, past
        // this capture's own 1,772, and everything the two residue items
        // were about goes with it: `1/8`'s move order `x` (item 118) and
        // `1/1`'s cleared `last_x/last_y` (item 119) were both downstream
        // of a stream that had been wrong since 1372.
        //
        // **What this is not.** run10 is 1,772 frames of one map. The
        // finish line `CLAUDE.md` names is two maps at their full length,
        // and East Indies still parts at 1374/1373 with its word at 1373,
        // so the number that is still moving is that one. What Great
        // Lakes now needs is a *longer* capture (queue item 91) — this
        // one has run out of frames to disagree on.
        assert!(
            ticks >= FLOORS[1].ticks
                && orders >= FLOORS[1].orders
                && first[0] >= 1772
                && first[1] >= 1772,
            "the headline fell: ticks {ticks}, orders {orders}, first divergence {:?} \
             — the floor is ticks {}, orders {}, and neither player parting",
            report.first_divergence,
            FLOORS[1].ticks,
            FLOORS[1].orders
        );
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.starts_with("rng: seeded 0x3bd39ae9")),
            "the stream is the original's at frame 0: {:?}",
            report.notes
        );
        // Every unit the original has before frame 1297 exists here too.
        let early_unlinked: usize = report
            .frames
            .iter()
            .filter(|f| f.frame < 1297)
            .map(|f| f.unlinked)
            .sum();
        assert_eq!(
            early_unlinked, 0,
            "a unit the original trained before 1/9 that the simulation did not"
        );
        let mut missing: Vec<(i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.unlinked_units.iter().copied())
            .collect();
        missing.sort_unstable();
        missing.dedup();
        // **The roster, both ways.** `unlinked` is what the original has and
        // this simulation does not; `extra` is the mirror, and it exists
        // because the one-sided count could be *paid off by
        // over-producing* — which is exactly what was happening.
        //
        // 2026-08-24 read "1/9 now trains on the original's frame, and only
        // the last citizen is missing", on 268 unlinked unit-frames. It did
        // not: the AI's ninth citizen was standing here from frame **897**
        // against the original's 1297, four hundred frames early, and the
        // one-sided measure could not see a unit that arrives too soon —
        // from 1297 the link exists and the earlier frames cost nothing.
        // The mirror was added in item 47 and reported those 400 at once.
        //
        // Item 47 (the AI builder's `unit_masks & 0x40000`) took `1/1` off
        // the farm it had built and put it on the original's woodcutter,
        // which is right and is what moved the headline — and with one
        // fewer farmer the AI now never reaches its ninth citizen inside
        // 1,772 frames. So the gap did not appear here; it changed sign,
        // and the two-sided total went 668 → 744. **The AI's long-run
        // economy is the item this measure now names**, and until it is
        // taken the honest statement is a floor on the *pair*.
        //
        // Item 70 (`find_gather_spot`'s cap-headroom score) changed the
        // sign back: preferring the good whose income is furthest below
        // its commerce cap puts the AI back on its farms, `1/9` is trained
        // inside the capture again, and the pair returns to **268 + 400**
        // — the ninth citizen four hundred frames early, exactly where it
        // stood before item 47. The over-production is unexplained and is
        // its own item; what this pin says is that the whole is not worse
        // for it.
        // Item 81 (the building ramp's missing ceiling) changed the sign a
        // third time, and this is the first reading in which the pair is
        // **one-sided again**: the AI's second city costs sixty rather than
        // twenty-two, so it is bought on the original's frame 776 rather
        // than 576, and the hundred and twenty food and timber it no longer
        // has two hundred frames early are what `1/9` was trained on. `1/9`
        // now arrives at **1497** against the original's 1297 — late, where
        // it used to be four hundred frames early — so it joins `1/10` in
        // `missing` and leaves `extras` empty. 268 + 400 → **468 + 0**.
        //
        // Item 74 (the AI's thirty-two food) closes it: the farm's
        // completion bonus and the science re-pricing put the AI back on
        // the original's food, `1/9` is trained on the original's own
        // frame, and it leaves `missing` altogether. **468 + 0 → 268 + 0**,
        // the lowest the pair has been, and the 268 are `1/10` alone — the
        // tenth citizen, which the original trains at 1772 and this does
        // not reach.
        //
        // Item 98 (the script's statics, `docs/AI.md` §17) closes that
        // one too: `needed_citizens` survives the call it was set in, so
        // the opening's every-call `train_unit_with_need` keeps training,
        // and `1/10` arrives. **268 + 0 → 0 + 0** — the roster is the
        // original's, both ways, over the whole capture.
        assert!(
            missing.is_empty(),
            "a citizen the original trains and the AI does not reach: {missing:?}"
        );
        let extras: Vec<(i64, i64)> = report
            .frames
            .iter()
            .flat_map(|f| f.extra_units.iter().copied())
            .collect();
        assert!(
            extras.is_empty(),
            "nothing is ahead of the original any more: {extras:?}"
        );
        let unlinked: usize = report.frames.iter().map(|f| f.unlinked).sum();
        eprintln!(
            "run10 roster: {unlinked} missing + {} extra, missing units {missing:?}",
            extras.len()
        );
        assert!(
            unlinked + extras.len() <= 268,
            "roster unit-frames: {unlinked} missing + {} extra — 2026-08-30 \
             was 268 + 0 with item 74 and 468 + 0 before it, 2026-08-29 was \
             268 + 400, 2026-08-27 was 744 + 0",
            extras.len()
        );

        // **`ObjectData::mylos` over the whole run** — the differential
        // check `docs/VISION.md` §2 hangs on, and the longest one available:
        // every object record carries `mylos` at every detail level, so this
        // is 26,433 unit-frames of the original's own line of sight against
        // `Sim::unit_los`.
        //
        // Exactly one disagreement, and it is a **cache**, not a formula.
        // `mylos` is stored on the object and refreshed by
        // `Leader::calc_unit_stats`, which `Leader::process` runs on the
        // frame *after* `gain_tech` sets `leader_flags & 0x4000000`. So when
        // player 1's first science level lands, the simulation's pure
        // function reports `4 + 1 × 2 = 6` on the frame the level is gained
        // and the original still reports 4 until the next one. `docs/VISION.md`
        // §7 books modelling the cache; the pair is what proves `epoch[3]`
        // is the Science line and `science_los` its multiplier.
        //
        // 26,433 → 25,957 with item 47: the 476 are `1/9`'s, the citizen
        // the AI no longer reaches (see the roster note above). Item 70
        // put it back and the count with it — 25,957 → 26,433, the same
        // 476 unit-frames. 26,433 → **26,233** with item 81, and it is the
        // same citizen a third time: `1/9` is trained two hundred frames
        // later than the original now rather than four hundred early, so
        // two hundred of its unit-frames are no longer comparable. The one
        // disagreement is unmoved through all four.
        //
        // 26,233 → **26,433** with item 74, and it is the same citizen a
        // fourth time: the farm's completion bonus and the science
        // re-pricing give the AI back the thirty-two food it was short, so
        // `1/9` is trained on the original's own frame again and its two
        // hundred unit-frames come back into view.
        //
        // 26,433 → **26,701** with item 98 (the script's statics): `1/10`
        // is trained at last, and its 268 unit-frames are the difference.
        // The one disagreement is unmoved through all five.
        //
        // **Item 1354 closed it** (`docs/VISION.md` §2): the scout `1/0`
        // was 6 against 4 on frame 202 — the Science epoch gained on 201
        // reaching this crate's live derivation a frame before the
        // original's `calc_unit_stats` refreshed its cache. `mylos` is the
        // cache here now, refreshed at the next leader pass.
        let los_seen: usize = report.frames.iter().map(|f| f.los_compared).sum();
        assert_eq!(los_seen, 26_701, "every compared unit-frame carries mylos");
        let bad: Vec<LosDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.los_diverged.iter().copied())
            .collect();
        assert_eq!(
            bad,
            Vec::<LosDivergence>::new(),
            "every unit's line of sight, the cache refreshed where the original's is"
        );

        // **The collision block over the whole run** (`docs/COLLISION.md`
        // §8). `UnitData::log_data` writes `collide`, `collide_o`,
        // `collide_who`, `collide_guy` and `safe` at every detail level, so
        // this is 40,600 field-frames of the original's own collision state
        // against `crates/sim/src/collide.rs` — the widest single record
        // this harness compares, and the one that says the mechanic is
        // right rather than merely plausible.
        //
        // 285 disagree, none before frame 201, and 277 of those are one
        // sticky byte: `collide_guy` is written to 0 by a hard collision
        // and **never cleared** (the clear path writes only `collide_o` and
        // `collide_who`), so a single collision this simulation has and the
        // original does not leaves `1/3` reading 0 against −1 for every one
        // of its remaining frames. The other eight are two units and two
        // collisions: `1/4` at 201–202 and `1/6` at 207, both well past the
        // score.
        //
        // 40,600 → 42,630 with item 47, and 285 → 400 with it. Both are
        // the *same* sticky byte and the same frame: `1/3` holds its
        // position against the original for 115 more frames than it did,
        // so 115 more of its unit-frames come into view — and every one of
        // them reads that one stale `collide_guy`. 392 of the 400 are
        // `1/3`'s; the field tally is the check that says so, and it is
        // asserted below rather than left to the total.
        //
        // 42,630 → **41,225** with item 50 (the bird): the *coverage*
        // moved, not the mechanic. This counter is five fields on every
        // unit-frame whose position already agrees, so it follows each
        // unit's own parting frame rather than the headline's — and the
        // headline went 181 → 185 while `0/1`, `1/1` and `1/2`, hundreds
        // of frames out on a stream the bird has changed, part 281
        // unit-frames sooner between them. The two assertions that say the
        // mechanic is still right are below and both held.
        //
        // 41,225 → **43,340** with item 52 (the bird's wing beat), the
        // same way and in the other direction: the stream is closer, so
        // more unit-frames hold their positions and come into view.
        //
        // 43,340 → **40,750** with item 55 (the road). The same coverage
        // effect once more, and this time the headline went *up* while
        // this went down: `0/3` holds 31 frames longer, but the AI's units
        // — which are hundreds of frames out either way — sit on a stream
        // whose values have all changed, and between them they part
        // sooner. The two assertions that say the mechanic is right are
        // below, and both held.
        //
        // 40,750 → **42,615** with item 59 (the builder's animation), and
        // the same coverage effect a third time: the AI's units hold their
        // positions longer on a stream that is the original's for 81 more
        // frames, so 1,865 more of their unit-frames come into view. `1/1`
        // alone parts 70 frames later.
        //
        // 42,615 → **43,575** with item 49 (the blocked stand), the same
        // coverage effect a fourth time and in the up direction: the
        // headline moved 190 → 192 and the AI's units, hundreds of frames
        // out either way, hold their positions on a stream that is the
        // original's for 63 more frames.
        //
        // 43,575 → **39,950** with item 61 (the farmer's cell index), the
        // coverage effect in the down direction while the headline went
        // 192 → 200. Six farmers now walk off their cells on the
        // original's frames rather than fourteen early, which moves every
        // value the stream carries after 185 — and the AI's units, which
        // are hundreds of frames out either way, part 3,625 unit-frames
        // sooner between them. The two assertions that say the mechanic
        // is right are below and both held.
        //
        // 39,950 → **42,840** with item 63 (the waypoint's own collision
        // test), the coverage effect a sixth time and in the up direction:
        // `1/4` no longer walks off its farm on 200, so it and the two
        // farmers behind it hold their positions for hundreds of frames
        // more, and 2,890 further unit-frames come into view.
        //
        // 42,840 → **48,790** with item 64 (`is_flat`, §6 step 2's fence),
        // and the disagreements went **245 → 0**. This is the run that
        // says the mechanic is right: every one of the 48,790 field-frames
        // agrees, the sticky `collide_guy` included — 243 of the 245 were
        // that one byte, written by a hard collision and never cleared, so
        // a collision this simulation had and the original did not left a
        // unit reading 0 against −1 for the rest of the run, and the fence
        // is what stops the collision happening. The assertion below is
        // now emptiness rather than a ceiling, so any single field on any
        // unit-frame of the capture fails it.
        //
        // 48,790 → **46,941** with item 66 (`find_nearby_spot`'s collision
        // half), the coverage effect a seventh time and in the down
        // direction while the headline went 209 → 252: the AI's `1/6` and
        // `1/7`, hundreds of frames out either way, now part at 253 rather
        // than being carried along by a walk that was already wrong.
        //
        // 46,941 → **62,307** with item 68 (the think tail's conscription),
        // the coverage effect an eighth time and in the up direction while
        // the headline went 252 → 322: `1/6` and `1/7` are no longer
        // marched across the map on frame 252, so the two of them alone
        // bring 15,366 further field-frames into view before they part.
        //
        // 62,307 → **60,247** with item 70 (`find_gather_spot`'s score),
        // the ninth time and in the down direction while the headline went
        // 322 → 355. Two units moved and they moved opposite ways: `1/8`
        // parts at 506 rather than 323, and `1/4` at 437 rather than 550.
        // This count is scoped to **agreement**, not to first divergence —
        // a farmer that walks off and comes back keeps contributing after
        // it has parted — so a re-tasked farmer costs more field-frames
        // than its own parting alone accounts for.
        //
        // 60,247 → **62,932** with item 71 (the idle variant's length),
        // and 62,932 → **62,957** with item 76 (the scout's surface
        // probe) — the eleventh time, and a small one: the scout holds a
        // hundred and twenty frames longer and three units behind it part
        // earlier, so the two nearly cancel.
        //
        // 62,957 → **74,429** with item 78 (the gather write-back), the
        // twelfth time and the largest single move it has made: every one
        // of the twelve compared units parts later, so 11,472 further
        // field-frames are comparable.
        //
        // 74,429 → **77,211** with item 79 (the vision projection), the
        // thirteenth: player 1's units all hold longer and player 0's
        // three farmers part earlier, and the AI's five are worth more
        // field-frames than the human's three cost.
        //
        // 77,211 → **80,161** with item 80 (the repath throttle's decay),
        // the fourteenth — and it is the collision block's own item, so
        // the count is the one to read: player 0's three farmers recover
        // everything item 79 cost them and pass it, 574/577/579 →
        // 687/700/703.
        //
        // 80,161 → **87,548** with item 81 (the building ramp), the
        // fifteenth and the largest single move it has made: the AI's
        // second city lands on the original's frame, so every unit of both
        // players holds two hundred frames longer.
        //
        // 87,548 → **93,341** with item 74 (the AI's thirty-two food), the
        // sixteenth: `1/9` is trained on the original's frame again, so its
        // own field-frames return, and eight of the other twelve units hold
        // longer with them.
        //
        // 93,341 -> **93,357** with items 36 and 93, and -> **93,398**
        // with item 95 (the animal's hurry): the AI's `1/4` and `1/5` each
        // hold a few frames longer, so their collision blocks are
        // comparable for longer.
        //
        // 93,398 -> **91,210** with the blocked animal's dropped walk, the
        // seventeenth and the second time it has fallen while a score
        // rose. **One** of the thirteen units moved: the AI's `1/9` parts
        // at 1320 rather than 1377, and the other twelve are unchanged to
        // the frame, as are both players' first divergences (802 and 573)
        // and the headline 572/776. run33's draws are identical to frame
        // 1128 under the same change, so a unit whose position parts at
        // 1320 has been on a stream of its own for two hundred frames
        // (`docs/SYNC.md` §3.14).
        //
        // 91,210 -> **92,766** with item 98 (the script's statics), the
        // eighteenth: `1/10` exists, so its own field-frames join the
        // count, and the other twelve are unchanged to the frame.
        //
        // 92,766 -> **97,333** with item 100 (the bird's landing search),
        // the nineteenth and the largest rise yet from a mechanic with no
        // unit in it: sixty draws a landing is enough of the stream that
        // several of the thirteen hold hundreds of frames longer. The
        // headline 572/776 and both players' first divergences (802 and
        // 573) are unchanged.
        //
        // 97,333 -> **97,118** with item 101 (the chopping guy's own
        // wait), the twentieth and the third fall while a score rose.
        // **One** of the fourteen moved: the AI's `1/10` parts at 1522
        // rather than 1579, and the other thirteen are unchanged to the
        // frame, as are the headline 572/776 and both players' first
        // divergences. A woodcutter that now stays at its tile three times
        // as long is a different unit on the map from frame 500 on, and
        // 1/10 is the unit that had been holding longest on the old one.
        //
        // 97,118 -> **97,108** with item 102 (the far wander's literal
        // bearing), the twenty-first and the smallest move it has ever
        // made: ten fields, two unit-frames, and **not one** of the
        // fourteen units parts on a different frame. Great Lakes' herd
        // wanders to different spots from this map's, so its animals'
        // arrivals fall on different frames deep past the parting, and two
        // unit-frames that used to re-agree by coincidence no longer do.
        // The headline 572/776, both players' first divergences and every
        // by-unit parting are unchanged; East Indies' word goes 742 → 867
        // and run33's own window totals rise 977/866 → 986/884.
        //
        // 97,108 -> **98,019** with item 106 (the scout's walk to a goody
        // box), the twenty-second. **Three** of the fourteen moved and all
        // three later: `1/0` parts at 959 rather than 872, `1/8` at 906
        // rather than 904, and `1/10` at 1552 rather than 1522. `1/0` is
        // the AI's scout and it is the unit the mechanic is about; the
        // other two move because a scout that walks somewhere else is a
        // different stream from frame 800 on. The headline 572/776 and
        // both players' first divergences (802 and 573) are unchanged;
        // East Indies' word goes 879 → 1256.
        //
        // 98,019 -> **99,343** with item 69's two halves, the twenty-third.
        // The total is scoped to agreeing unit-frames, so it moves with the
        // headline; ticks 572 -> 781 is what moved it.
        //
        // 99,343 -> **112,447** with item 84, the twenty-fourth: ticks
        // 781 -> 910 and every one of the thirteen parting later.
        //
        // 112,447 -> **123,500** with item 112, the twenty-fifth, and this
        // one moved without the headline. Six of the fourteen part later —
        // player 0's three by two hundred frames apiece — so eleven
        // thousand more unit-frames are comparable at all, and the block
        // stays empty over every one of them.
        //
        // 123,500 -> **128,672** with item 113, the twenty-sixth, and it
        // moves with the headline: `1/1` never parts at all now, so its
        // whole run is comparable where only nine hundred frames of it
        // were.
        //
        // 128,672 -> **139,514** with item 114, the twenty-seventh and the
        // last one this capture can make: no unit parts anywhere, so every
        // field-frame run10 holds is comparable and the number is now the
        // capture's own size rather than a score. ~~It moves again only
        // when a longer capture replaces this one.~~ Or when the record
        // grows a **field**: 139,514 -> **166,215** on 2026-09-07, item
        // 267, which added `unit_masks & 0x100000` — the soft one-shot,
        // printed since the first capture and compared by nothing
        // (`docs/COLLISION.md` §8.4). The block is still empty over every
        // one of them.
        let coll_seen: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        assert_eq!(
            coll_seen, 166_215,
            "six fields on every agreeing unit-frame"
        );
        // **The emptiness, scoped to what the capture can speak to.**
        // Item 64 took the block to zero over the whole run; item 66
        // moved `1/3` from parting at 103 to parting at 345, and 158
        // frames past that it takes a collision the original does not —
        // one sticky `collide_guy 0` from frame 503 to 1600, 461
        // field-frames of the same byte. A unit whose position has been
        // wrong for a hundred frames is not evidence about collision, so
        // what is asserted is emptiness **before each unit's own first
        // divergence**: every field-frame of the capture that is
        // comparable at all, and any single one of them fails it.
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let coll_bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        // 2026-08-31, item 69: **two field-frames, and they are the
        // item's own residue.** With ticks 572 -> 781 the comparable
        // window grew by two hundred frames on every unit, and `1/6`'s
        // frame 797 came inside it: this crate's `collide` counter reads
        // **3** where the original's reads 2 and its `collide_frame` is
        // **796** against 795 — one collision more, entered a frame later,
        // on the unit whose position then parts at 798. It is the
        // collision seam one layer under the pause, and it is the queue's
        // successor to this item. Everything else in the window is still
        // empty, and any third row fails this.
        assert_eq!(
            coll_bad
                .iter()
                .map(|d| (d.frame, d.who, d.o, d.field, d.ours, d.theirs))
                .collect::<Vec<_>>(),
            Vec::new(),
            "the collision block agrees on every comparable field-frame of {coll_seen}"
        );

        // **The tile choice, asserted where it was wrong** (item 25). The
        // earliest frame on which the two sides' `GATHERORDER` name
        // different resource tiles: **2** before the access filter landed,
        // when player 1's woodcutter picked `(214, 93)` — a tree ringed by
        // its own forest, with no orthogonal neighbour to stand on — and
        // the original picked `(213, 92)`. That one tile pinned the
        // headline at ticks 3. It went to 169 — `1/1`, the citizen the
        // original turned into a woodcutter and this simulation kept on the
        // farm it had built — and with item 47 it is 430, where the
        // woodcutter `1/6` chooses its second tree of the game.
        //
        // **413 with item 50 (the bird), and this one moved the wrong
        // way.** Which tree a woodcutter's second choice lands on is decided
        // by `find_gather_spot` on a stream that is still ten draws short
        // of the original's from frame 168 (`docs/SYNC.md` §3.9), so it is
        // luck rather than a rule until that closes — and this time the
        // luck went against. Said plainly rather than folded into a
        // ceiling: the two scores that decide the item both rose, and this
        // sub-score fell 17 frames.
        //
        // **407 with item 52 (the wing beat)**, six frames the same way and
        // for the same reason: `1/6`'s second tree is still drawn off a
        // stream that has drifted by then, and a closer stream at frame 180
        // is not a closer one at 407.
        //
        // **430 with item 55 (the road)**, back where item 47 left it, and
        // by the same luck in the other direction.
        //
        // **407 with item 59**, the same twenty-three frames back the other
        // way. `1/6`'s second tree is drawn on a frame the stream reaches
        // long after it has parted (122 on run14's trace), so which tree it
        // is remains luck; the numbers that are not are the two the item
        // was booked on.
        //
        // **415 with item 61**, eight frames the same luck's way again:
        // `1/6`'s second tree is still drawn on a frame long past the
        // word's divergence (201 now), and the six farmers' corrected
        // cells move every value the stream carries from 185 on.
        //
        // **407 with item 63**, those same eight frames back the other
        // way — the sixth time this sub-score has bounced between 407 and
        // 430 on a draw nobody has fixed. It stops being luck when the
        // stream reaches frame 407 in step, and not before.
        //
        // **1,298 with item 70**, and this one is not the same bounce: the
        // cap-headroom score sends the AI's citizens to the buildings the
        // original sends them to, so `1/6`'s trees stop being the question
        // and every gather tile of the capture agrees until the frame
        // after the original trains `1/9`. Nine hundred frames is three
        // times the span the bounce ever covered.
        //
        // **1,299 with item 76**, one frame further: the AI's `1/9` is
        // still trained late here, and the frame its gather tile first
        // disagrees is the frame after that.
        //
        // **1,298 with item 78**, the same frame back: `1/9` is still the
        // unit, and it is still trained late (item 74).
        //
        // **1,384 with item 106**, eleven frames on from the 1,373 the
        // intervening items had reached: the scout's walk to a goody box
        // moves `1/0` and, through the stream, the frames the AI's
        // citizens re-pick on.
        //
        // **1,598 with item 112**, two hundred frames on, and it is the
        // word carrying it: with the herd's wander centre right the stream
        // is the original's to 1372, so the citizens re-pick on the
        // original's frames for two hundred frames more.
        //
        // **Never, with item 114.** The word runs past this capture's own
        // length, so there is no frame left on which a gather tile can
        // disagree: every citizen of both players works the original's
        // tile for all 1,772 frames. `None` is the assertion now, and it
        // is a stronger one than any frame number — a regression anywhere
        // in the run fails it.
        let tile_row = |d: &&OrderDivergence| {
            matches!(
                d.what,
                OrderMismatch::Gather {
                    field: "tx" | "ty",
                    ..
                }
            )
        };
        let first_tile = report
            .frames
            .iter()
            .find(|f| f.order_diverged.iter().any(|d| tile_row(&d)))
            .map(|f| f.frame);
        assert_eq!(
            first_tile, None,
            "the first frame on which a gather tile disagrees"
        );
        assert!(
            !report
                .frames
                .iter()
                .flat_map(|f| f.order_diverged.iter())
                .any(|d| d.who == 0 && tile_row(&d)),
            "player 0's woodcutters chop the original's trees for all 1,772 frames"
        );

        // **Where a trained unit appears** (item 43). The AI's citizens are
        // created on frames 100, 206 and 320, and on each of those frames
        // the simulation stands its unit on the original's own tile —
        // `(42360, 17208)`, due south of London on the exit ring. Before
        // `come_out` was wired into the handover the unit was left on its
        // trainer's centre and this reported the city's position against
        // that one, a thousand units away.
        for (born, o) in [(100i64, 6i64), (206, 7), (320, 8)] {
            let f = report
                .frames
                .iter()
                .find(|f| f.frame == born)
                .expect("the frame the citizen is trained on");
            assert!(
                !f.diverged.iter().any(|d| d.who == 1 && d.o == o),
                "1/{o} does not appear where the original puts it on frame {born}: {:?}",
                f.diverged.iter().find(|d| d.who == 1 && d.o == o)
            );
        }

        // **Both angles, on every unit-frame where the positions agree**
        // (`docs/MOVEMENT.md` §"Two angles", item 34): `UnitData::angle`
        // against the heading and guy 0's `angle` against the facing.
        // 13,542 → 15,336 with item 25: this counts only the unit-frames
        // whose *positions* agree, so the tile-choice fix bought 897 of
        // them outright — player 1's woodcutter alone now stands where the
        // original stands it from frame 4 to frame 567.
        // 15,336 → 15,010 with item 43. Every unit's **first** divergence
        // held or improved, and the AI's trained citizens now appear on the
        // original's own tile; what fell is agreement deep in the untraced
        // stretch, where those citizens are alive and walking instead of
        // standing on their city, so their later frames are their own.
        // 15,010 → 15,318 with item 44, and the same caveat holds twice
        // over: the headline went 102 → 122 while this moved by 308, which
        // is the useful reminder that a total over 1,772 frames is not the
        // score. The score is where the *first* divergence falls.
        // 15,318 → 16,206 with item 46 (collision): `1/6` alone holds from
        // frame 123 to frame 208.
        // 16,206 → 17,018 with item 47, against 6,866 → 6,926 bad: of the
        // 812 rows the AI builder's fix brought into view, 752 agree.
        // 17,018 → **16,456** with item 50 (the bird), the same coverage
        // move the collision tally makes: this counts unit-frames whose
        // positions agree, so it follows each unit's own parting frame,
        // and three units hundreds of frames out part sooner on a stream
        // the bird has changed. The headline went 181 → 185.
        // 16,456 → **17,302** with item 52 (the wing beat), the other way
        // round: 846 more unit-frames hold their positions on a stream the
        // bird's two animation lengths have brought closer.
        // 17,302 → **16,266** with item 55 (the road): coverage again, and
        // again against the headline's direction, which went 181 → 202.
        // 16,266 → **17,012** with item 59 (the builder's animation): 746
        // more unit-frames hold their positions on a stream that is the
        // original's for 81 more frames.
        // 17,012 → **17,396** with item 49 (the blocked stand): 384 more,
        // and the headline went 190 → 192.
        // 17,396 → **15,946** with item 61 (the farmer's cell index):
        // coverage against the headline's direction a fourth time, and
        // for the same reason as the collision tally three paragraphs up
        // — the six corrected farmers move every value the stream carries
        // after 185, and the units hundreds of frames out part sooner on
        // it. The headline went 192 → 200.
        // 15,946 → **17,102** with item 63 (the waypoint's own collision
        // test): 1,156 more, the coverage effect back in the headline's
        // direction — `1/4` stays on its farm from 200 rather than walking
        // off it, and the farmers behind it hold with it.
        // 17,102 → **19,464** with item 64 (`is_flat`): 2,362 more, the
        // coverage effect in the headline's direction again — `1/6` no
        // longer stalls at its own gather target from 207, so it and the
        // AI's later citizens hold their positions for hundreds of frames
        // more. The headline went 207 → 209.
        // 19,464 → **18,724** with item 66 (`find_nearby_spot`'s collision
        // half): 740 fewer, coverage against the headline's direction a
        // fifth time while the headline went 209 → 252. Every unit that
        // now walks to a different spot walks a different route after it,
        // and three of the AI's citizens part sooner in the deep untraced
        // stretch than they did off a walk that was already wrong.
        // 18,724 → **24,120** with item 68 (the think tail's conscription):
        // 5,396 more, the coverage effect back in the headline's direction
        // and the largest single move it has made — `1/6` and `1/7` are no
        // longer marched off on frame 252 and hold to 735 and 937, and the
        // human's farmers hold with them on a stream that is the original's
        // for seventy frames more. The headline went 252 → 322.
        // 24,120 → **23,296** with item 70 (`find_gather_spot`'s score):
        // 824 fewer, coverage against the headline's direction a sixth
        // time while the headline went 322 → 355, and the same two units
        // the collision tally names — `1/4` re-tasked 113 frames sooner
        // costs more agreeing frames than `1/8`'s 183 extra ones pay for.
        // 23,296 → **24,370** with item 71 (the idle variant's length), and
        // 24,370 → **24,380** with item 76 (the scout's surface probe): ten
        // more, the two directions nearly cancelling — the scout holds a
        // hundred and twenty frames longer, three units behind it part
        // earlier.
        // 24,380 → **28,916** with item 78 (the gather write-back), and
        // this is the first widening whose *disagreements* fell with it:
        // 4,536 more rows compared and 2,012 fewer bad. The paragraph
        // below opened on `0/2`'s 2,680 rows — the woodcutter that
        // "walks to `(4440, 28680)` on frame 432 … and on 433 the original
        // turns it to face what it is about to gather while the simulation
        // leaves it pointing the way it walked". That was not one of
        // `set_angle`'s seventeen other callers after all: with the
        // write-back landing, `0/2` reaches the camp-arrival branch on 433
        // and faces the camp there, and its share of the residue is 140.
        // 28,916 → **29,878** with item 79 (the vision projection): 962
        // more, the AI's five holding longer against the human farmers'
        // earlier parting.
        // 29,878 → **31,022** with item 80 (the repath throttle's decay):
        // 1,144 more, the human farmers holding a hundred frames longer.
        // 31,022 → **33,992** with item 81 (the building ramp): 2,970
        // more, of which 1,684 agree.
        // 33,992 → **35,868** with item 74 (the AI's thirty-two food):
        // 1,876 more, `1/9`'s own rows and the eight units that hold
        // longer beside it.
        // Unmoved by item 36 — it changes no position — and 35,868 →
        // **35,984** with item 93.
        // 35,984 → **35,942** with item 95 (the animal's hurry): 42 fewer,
        // the coverage effect against the headline's direction for the
        // seventh time, and it is `1/4` and `1/5` parting a frame or two
        // earlier deep in the untraced stretch. run10's headline is
        // unmoved at 572/776 and its collision rows *rose*, 93,357 →
        // 93,398.
        // 35,942 → **35,188** with the blocked animal's dropped walk: 754
        // fewer, and all of them `1/9`'s, which parts at 1320 rather than
        // 1377 (see the collision rows above). The headline is unmoved at
        // 572/776 and the other map's word goes 91 → 201.
        // 35,188 → **35,742** with item 98 (the script's statics): 554
        // more, and all of them `1/10`'s, the citizen the AI reaches for
        // the first time.
        // 35,742 → **37,376** with item 100 (the bird's landing search):
        // 1,634 more, the same holding-longer the collision rows show.
        // 37,376 → **37,174** with item 101 (the chopping guy's own wait):
        // 202 fewer, and all of them `1/10`'s, which parts at 1522 rather
        // than 1579 (see the collision rows above). The headline is
        // unmoved at 572/776 and East Indies' word goes 645 → 742.
        // 37,174 → **37,170** with item 102 (the far wander's literal
        // bearing): four fewer, the same two unit-frames the collision rows
        // lost, and no unit parts on a different frame. East Indies' word
        // goes 742 → 867.
        // 37,170 → **37,450** with item 106 (the scout's walk to a goody
        // box): 280 more, the three units that hold longer for it —
        // `1/0` at 959, `1/8` at 906, `1/10` at 1552.
        // 38,104 → **37,838** with item 69's two halves. The four units
        // whose parting moved are `1/2` 573 → **1184**, `1/1` 794 → 797,
        // `1/6` 795 → 798 and `1/10` 1579 → **1528**; the total counts
        // every agreeing unit-frame of the whole capture, coincidences
        // past a parting included, so it falls by 266 while the score it
        // sits beside rises by 209 frames. The number to read is the
        // headline.
        // 37,838 → **43,202** with item 84's farm byte, and → **47,364**
        // with item 112's herd centre: both rises are the comparable
        // window growing on six of the fourteen units at once, and neither
        // is a coincidence past a parting — the word carries them.
        // 47,364 → **49,088** with item 113's two halves, and again the
        // word carries it: `1/1` never parts, so its whole 1,772 frames
        // are comparable where nine hundred were.
        // 49,088 → **53,402** with item 114's pasture, and this is the
        // ceiling: no unit parts anywhere, so the number is every guy-frame
        // the capture holds. Like the collision rows it moves again only
        // when a longer capture replaces this one.
        let angles: usize = report.frames.iter().map(|f| f.angle_compared).sum();
        assert_eq!(angles, 53_402, "two per agreeing unit-frame that has a guy");
        let bad: Vec<AngleDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.angle_diverged.iter().copied())
            .collect();
        // 2026-08-27, on landing item 34: 5,435. The residue is **not** the
        // step's — it is `Unit::set_angle`'s other seventeen callers, which
        // this simulation does not make (`Sim::unit_set_angle`). Unit `0/2`
        // alone is 2,680 of it: it walks to `(4440, 28680)` on frame 432
        // with both sides agreeing on the position, the path and both
        // angles, and on 433 the original turns it to face what it is about
        // to gather while the simulation leaves it pointing the way it
        // walked. The farmers (`o` 3 to 5) are most of the rest, and they
        // are doing a different job at the same spot (`docs/SYNC.md` §6).
        // 5,435 → 6,382 with item 25, against 13,542 → 15,336 compared: the
        // ceiling rose because 1,794 rows the harness could not see before
        // came into view, and 847 of them agree. Item 36 is still the item
        // that takes this down.
        // 6,382 → 6,866 with item 46, against 15,318 → 16,206 compared:
        // 404 of the 888 rows collision brought into view agree, and the
        // rest are the same seventeen callers.
        // 6,926 → 7,042 with item 52, against 16,456 → 17,302 compared:
        // 730 of the 846 the wing beat brought into view agree, and the
        // 116 that do not are those same callers again (item 36).
        // 7,042 → 8,737 with item 64, against 17,102 → 19,464 compared:
        // 667 of the 2,362 rows the fence brought into view agree, and the
        // 1,695 that do not are the same seventeen callers on units that
        // now stand where the original stands them for far longer. Item 36
        // is still the item that takes this down.
        // 8,737 → 9,378 with item 68, against 19,464 → 24,120 compared:
        // 4,755 of the 5,396 rows the think tail's correction brought into
        // view agree and 641 do not, which is the best ratio any widening
        // has had here — and the 641 are the same seventeen callers on
        // two woodcutters that now work their trees for another five
        // hundred frames. Item 36 is still the item that takes this down.
        // 9,378 → **7,366** with item 78, against 24,380 → 28,916
        // compared: `0/2`'s 2,680 rows were never item 36's, and what is
        // left is the farmers (`0/3`–`0/5` and `1/3`–`1/5`, 6,658 of the
        // 7,366) doing a different job at the same spot.
        // 7,366 → **7,242** with item 79, against 28,916 → 29,878
        // compared.
        // 7,242 → **7,870** with item 80, against 29,878 → 31,022
        // compared: 516 of the 1,144 rows the throttle's decay brought
        // into view agree, and the residue is item 36's and nothing else —
        // `0/3`–`0/5`, `1/3`–`1/5` and `1/8`, every one of them a farmer,
        // are 7,818 of the 7,870, and no other unit contributes more than
        // twenty-eight rows.
        // 7,870 → **9,156** with item 81, against 31,022 → 33,992
        // compared: 1,684 of the 2,970 new rows agree, and the residue is
        // still the farmers — `0/3`–`0/5`, `1/3`–`1/5` and `1/8` are 8,866
        // of the 9,156. What is new is `0/1` (140) and `0/2` (136), the two
        // woodcutters, which item 36 did not have to account for before.
        // 9,156 → **8,969** with item 37, against 33,992 → 35,868
        // compared: the standing body's instant turn
        // (`docs/MOVEMENT.md`, "The body step") both removed the scout's
        // three and kept 1,876 more rows in view.
        // 8,969 → **1,227** with item 36, on the same 35,868 — and it was
        // never the seventeen callers. It was two predicates in code this
        // crate already had (`docs/SYNC.md` §3.12): `add_move_order` took
        // the angle to the **snapped** destination where the listing takes
        // it to the point the caller handed over, and `move_step`'s gather
        // clause was applied to both arrival arms where the original has it
        // on the Manhattan snap alone. Every farmer left the residue and
        // the earliest surviving row went 110 → 820.
        // 1,227 → **1,910** against 35,868 → 35,984 with item 93 (the
        // pasture's snap and `Guy::move`'s turn arm, §3.11's pair, landed
        // the same session because item 36 unblocked it): 116 more rows in
        // view, and the coverage effect deep in the untraced stretch that
        // every widening has had here.
        assert!(
            bad.len() <= 1_910,
            "angle disagreements grew: {} of {angles}",
            bad.len()
        );
        // The scout was item 37: **three** rows in 1,772 frames, each the
        // frame after an arrival, where the original's body had already
        // snapped onto the order's angle and the simulation's turned a
        // frame later — 96, 362 and 721. **None, since 2026-08-30**:
        // `Guy::move` zeroes `last_speed` at the head of its at-des branch
        // and `GuyData::turn_speed` reads that zero, so a standing foot or
        // mounted body swallows whatever turn it is owed in one frame.
        // The whole of the residue above is item 36's farmers now.
        let scout: Vec<AngleDivergence> = bad
            .iter()
            .copied()
            .filter(|d| (d.who, d.o) == (1, 0))
            .collect();
        assert!(scout.is_empty(), "the AI scout turns late again: {scout:?}");
    }

    /// The slot count against the original's own survey: run9's frame-1
    /// `BUILDDATA` gives each camp its `gather_from` list — 82 tiles for
    /// player 0's, 61 for player 1's — and the `LEADERS=9` record gives the
    /// counts those lists produced, `gather_slots[1]` of 7 and 5
    /// (`crates/sim/src/gather.rs`).
    #[test]
    fn run9_s_camps_get_the_original_s_gatherer_counts() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run9-world6.txt") else {
            eprintln!("skipping: no gamelog-run9-world6.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let built = build_sim(&loaded, &log.initial().unwrap(), Tuning::RON);
        let sim = &built.sim;
        let mut got: Vec<(sim::Player, usize, i32)> = Vec::new();
        for b in 0..sim.buildings.len() {
            if sim.building_ident(b) == sim::build::Ident::Woodcutter {
                got.push((
                    sim.buildings[b].owner,
                    sim.buildings[b].gather_from.len(),
                    sim.max_gatherers(b),
                ));
            }
        }
        got.sort_unstable();
        assert_eq!(got, vec![(0, 82, 7), (1, 61, 5)], "the camps' slot counts");
    }

    /// **The Tobacco rare on the Tower's clock — Great Lakes 7176 →
    /// 7455** (2026-09-07, item 261).
    ///
    /// The value diff beside the word. Great Lakes' AI places a Tower
    /// `1/2017` on frame 6494 and the original's `constr_time` for it
    /// reads the type's own `job_time × 100` — **100000** — through frame
    /// 6751, and **90909** from **6752** to the end of the game: exactly
    /// `× 100 / (TOBACCO_BUILDING_SPEED + 100)` with the constant at 10.
    /// 6751 is the frame this crate's own `rare_owned` gains bit
    /// `19 − BASE_RARE = 13`, so the rare arrives here on the original's
    /// frame and the *bake* is what was missing: `Leader::gather@006ce280`
    /// raises **`0xc000000`** when the mask moves — the unit-stats flag
    /// and the **wall-stats** one — and this crate raised only the first,
    /// so `Wall::update_construct_time` never ran again and the Tower
    /// carried 100000 for four hundred frames.
    ///
    /// The consequence is the word: the site finishes when `job_counter`
    /// (150 a frame) passes `constr_time`, so the original's finishes on
    /// **7176** — `job_counter` 90800 → 0, `construct_hits` 749 → 750,
    /// `flags` 3 → 7 — its builder `1/20` drops its `BUILDORDER` and goes
    /// `idle 1` the frame after, and that idle is the
    /// `Guy::set_anim+0x97a < Unit::do_idle+0x7d` draw the crate did not
    /// spend. Great Lakes' long word had sat at 7176 for exactly this.
    ///
    /// Both halves are asserted: the original's own record, read straight
    /// off two dumps so that nothing this crate does can move it, and
    /// this crate's agreement with it frame for frame. `constr_time` and
    /// `job_counter` are compared on **every** capture now
    /// (`crate::diff::compare`), so this test is the dated statement and
    /// the harness is the guard.
    /// **Great Lakes 9451 is two arrows landing on one farm** (item 394,
    /// 2026-09-19, `docs/COMBAT.md` §7.2 step 3 and §20).
    ///
    /// The headline frame, and the mechanism is the fraction. `0/2004` is
    /// a Farm, 400 hit points, and the first hit point lost in the whole
    /// game is lost here. run100's own record says how:
    ///
    /// ```text
    /// block 9452   damage 0 -> 1   damage_frac 0 -> 10
    /// block 9465   damage 1 -> 2   damage_frac 10 -> 7
    /// block 9470   damage 2 -> 3   damage_frac 7 -> 4
    /// ```
    ///
    /// Every later step is **thirteen sixteenths** — one Longbowman
    /// figure's share of a damage of 5 (`5 × 0x100 / AMMO_PER_ATT 2 /
    /// UBER_SIZE 3 = 213`, `213 >> 4 = 13`). The first step is **twenty-six**,
    /// which is two of them, so two arrows land on sim-frame 9451. That
    /// is what makes `Object::take_damage+0xe1` draw **twice** on one
    /// frame: the gate is `damage == 0`, the whole-hit count, and the
    /// first arrow moves only `damage_frac` — `0 + 13` carries nothing —
    /// so the second arrow finds the gate still open.
    ///
    /// The negative for the gate itself lives beside the stream, in
    /// `run53_s_24000_frames_put_the_ceiling_where_run33_did`: adding
    /// `damage_frac == 0` to it halves the first wounds from two to one,
    /// and — the trap — makes 9452 agree eight-for-eight. This test holds
    /// the other half, the values.
    ///
    /// ~~**What this crate does not yet have** is the second arrow on the
    /// same frame~~ — **the seam closed on item 396** and this assertion
    /// is what failed and said so, which is what it was written for.
    /// `1/28`'s `CHAR_ATTACK2` shot flew 27 frames here and 26 there
    /// because the original launches from the release node's own world
    /// position; run109 measured that offset and `docs/COMBAT.md` §22 is
    /// the table. **The two step lists are now equal, all eight of them**,
    /// and that equality is what is asserted below — a stronger claim than
    /// the lag it replaces, and the one that fails if the table moves.
    #[test]
    fn run100_says_great_lakes_9451_is_two_arrows_on_one_farm() {
        const FIRST: i64 = 9_440;
        const LAST: i64 = 9_500;
        const FARM: (i64, i64) = (0, 2004);
        let Some(inst) = install() else { return };
        let (Some(path), Some(r100)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run100-greatlakes-valuewindow2.txt"),
        ) else {
            eprintln!("skipping: no run53/run100 capture (set RON_GAMELOG_DIR)");
            return;
        };
        // The original's own record of the farm, block by block.
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r100).unwrap();
        let mut theirs: Vec<(i64, i64, i64)> = Vec::new();
        for f in FIRST..=LAST {
            let Some(at) = ix.frames().iter().position(|x| x.number == f) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            let parsed = Log::parse(&body);
            let Some(block) = parsed
                .frames()
                .into_iter()
                .find(|(n, _)| *n == f)
                .map(|(_, b)| b)
            else {
                continue;
            };
            let (_, builds, _) = crate::gamelog::records(block, false);
            let Some(b) = builds.iter().find(|b| (b.who, b.o) == FARM) else {
                continue;
            };
            theirs.push((
                f,
                b.damage.expect("BUILDDATA carries damage"),
                b.damage_frac.expect("BUILDDATA carries damage_frac"),
            ));
        }
        assert!(
            theirs.len() >= 50,
            "run100 does not cover [{FIRST}, {LAST}]: {} blocks",
            theirs.len()
        );
        // In sixteenths, and the per-block steps.
        let sixteenths = |d: i64, f: i64| d * 16 + f;
        let steps: Vec<(i64, i64)> = theirs
            .windows(2)
            .filter_map(|w| {
                let d = sixteenths(w[1].1, w[1].2) - sixteenths(w[0].1, w[0].2);
                (d != 0).then_some((w[1].0, d))
            })
            .collect();
        eprintln!("  run100 {FARM:?}: {} steps, {steps:?}", steps.len());
        assert_eq!(
            steps.first().copied(),
            Some((9_452, 26)),
            "the first hit point in the game is lost in block 9452, and it \
             is twenty-six sixteenths — two arrows on sim-frame 9451"
        );
        assert!(
            steps[1..].iter().all(|&(_, d)| d == 13),
            "every later step is one figure's thirteen sixteenths: {steps:?}"
        );
        // This crate, over the same frames. `built.tick()` leaves the state
        // the dump numbers `f + 1`, the convention every window test here
        // uses.
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut ours: Vec<(i64, i64, i64)> = Vec::new();
        for f in 0..LAST {
            built.tick();
            if f + 1 < FIRST {
                continue;
            }
            let b = crate::diff::harness::link_building(&built.sim, FARM.0, FARM.1)
                .expect("this crate has the farm");
            let bd = &built.sim.buildings[b];
            ours.push((f + 1, i64::from(bd.damage), i64::from(bd.damage_frac)));
        }
        let our_steps: Vec<(i64, i64)> = ours
            .windows(2)
            .filter_map(|w| {
                let d = sixteenths(w[1].1, w[1].2) - sixteenths(w[0].1, w[0].2);
                (d != 0).then_some((w[1].0, d))
            })
            .collect();
        eprintln!(
            "  ours   {FARM:?}: {} steps, {our_steps:?}",
            our_steps.len()
        );
        // **Every hit this crate lands is the original's thirteen.** That
        // is the damage formula, the `AMMO_PER_ATT`/`UBER_SIZE` pair of
        // divisions and the sixteenths accumulator in one number, and it
        // is the half that does not depend on the launch seam.
        assert_eq!(
            our_steps, steps,
            "this crate's damage steps on the farm are not the original's, \
             step for step"
        );
        assert_eq!(
            our_steps.first().copied(),
            Some((9_452, 26)),
            "the first step is two arrows on one frame: {our_steps:?}"
        );
        assert!(
            our_steps[1..].iter().all(|&(_, d)| d == 13),
            "every later hit is one figure's thirteen sixteenths: \
             {our_steps:?}"
        );
        // **The first two arrows, and they land on the same frame.**
        // Both of them on 9451, so block 9452 carries `damage 1` and
        // `damage_frac 10` on both sides — the value the launch seam used
        // to cost a frame (§22.4).
        let at = |v: &[(i64, i64, i64)], f: i64| {
            v.iter().find(|x| x.0 == f).map(|x| (x.1, x.2)).unwrap()
        };
        assert_eq!(at(&theirs, 9_452), (1, 10), "the original's first wound");
        assert_eq!(
            at(&ours, 9_452),
            at(&theirs, 9_452),
            "this crate lands both first arrows on 9451, as the original \
             does — the launch seam is §22's measured table"
        );
        assert_eq!(
            at(&ours, 9_453),
            at(&theirs, 9_453),
            "and the block after it still agrees"
        );
        assert_eq!(at(&theirs, 9_453), (1, 10), "and the value is 1 and 10");
    }

    /// **run109 says where Great Lakes' arrows start, and where they come
    /// down** (item 396, `docs/COMBAT.md` §22).
    ///
    /// `AmmoData::log_data` prints the arrow's own `sx, sy` — the release
    /// node's world position, which is the guy's plus a per-(piece, anim,
    /// starttime) vector turned by the guy's facing — and `ex, ey` and
    /// `total_time` beside it. run109 raised `AMMO=5` over `[9420, 9480)`
    /// on run100's game, so **nine** arrows are on the disk with all five
    /// numbers, and this test is the value diff against them: not the
    /// launch *frames*, which
    /// [`super::super::harness::tests::run53_s_24000_frames_put_the_ceiling_where_run33_did`]
    /// takes from the trace, but where each one leaves from, where it is
    /// aimed and how long it flies.
    ///
    /// It is read out of the archive each run rather than pinned as
    /// constants, which is what `crates/sim/src/launch.rs`'s own unit test
    /// is not: that one carries the dump's columns as literals and so
    /// cannot notice the archive changing. This can.
    ///
    /// The window bounds what it can say. An arrow launched on sim-frame
    /// `L` first prints in block `L + 1`, so run109 reaches launches up to
    /// 9478 and no further; the seven launches this crate makes between
    /// 9479 and the word have no record on this disk at all.
    #[test]
    fn run109_says_great_lakes_s_launches_land_where_the_bow_hand_aims() {
        const LAST: i64 = 9_480;
        let Some(inst) = install() else { return };
        let (Some(path), Some(r109)) = (
            dump("gamelog-run53-greatlakes-24k-trace.txt"),
            dump("gamelog-run109-greatlakes-ammolaunch.txt"),
        ) else {
            eprintln!("skipping: no run53/run109 capture (set RON_GAMELOG_DIR)");
            return;
        };
        // ---- the original's own record. Every `BEGIN AMMO` block in the
        // window, reduced to its first appearance: the block an arrow is
        // first printed in is the one after the frame it launched on.
        let mut ix = crate::capture::indexed::IndexedCapture::open(&r109).unwrap();
        let mut theirs: Vec<(i64, i64, i64, i64, i64, i64)> = Vec::new();
        let mut seen: Vec<(i64, i64, i64, i64, i64)> = Vec::new();
        for f in 9_420..LAST {
            let Some(at) = ix.frames().iter().position(|x| x.number == f) else {
                continue;
            };
            let body = ix.read_frame(at).unwrap();
            for a in ammo_blocks(&body) {
                if !seen.contains(&a) {
                    seen.push(a);
                    theirs.push((f - 1, a.0, a.1, a.2, a.3, a.4));
                }
            }
        }
        eprintln!("  run109: {} arrows, {theirs:?}", theirs.len());
        assert_eq!(
            theirs.len(),
            9,
            "run109's window holds nine arrows; it holds {}",
            theirs.len()
        );

        // ---- this crate, over the same frames.
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        if let Some(t) = trace("rontrace-run53.log") {
            borrow_pasture(&mut init, &t);
        }
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // `built.tick()` at 0-based `f` leaves the state the dump numbers
        // `f + 1`, so a projectile that appears in it launched on `f`.
        let mut ours: Vec<(i64, i64, i64, i64, i64, i64)> = Vec::new();
        let mut known: Vec<(i32, i32, i32, i32, i32)> = Vec::new();
        for f in 0..LAST {
            built.tick();
            for p in &built.sim.projectiles {
                let k = (
                    p.launch.x,
                    p.launch.y,
                    p.landing.x,
                    p.landing.y,
                    p.total_time,
                );
                if !known.contains(&k) {
                    known.push(k);
                    ours.push((
                        f,
                        i64::from(k.0),
                        i64::from(k.1),
                        i64::from(k.2),
                        i64::from(k.3),
                        i64::from(k.4),
                    ));
                }
            }
        }
        // Only the ones run109 can speak to: its window ends at block 9479.
        let ours: Vec<_> = ours.into_iter().filter(|r| r.0 <= 9_478).collect();
        eprintln!("  ours:   {} arrows, {ours:?}", ours.len());

        // **Launch frame, launch point, landing point and flight time, all
        // nine, all five numbers.** The launch point is §22's table; the
        // landing point is the aim taken from *that* point rather than from
        // the unit's own square; the flight time is the two together
        // through §9.1's truncating divide.
        assert_eq!(
            ours, theirs,
            "Great Lakes' arrows do not leave, aim or fly as the original's \
             do — left is this crate, right is run109's own AMMO record, \
             each row (launch frame, sx, sy, ex, ey, total_time)"
        );
    }

    /// Every `BEGIN AMMO` block in one frame body, as
    /// `(sx, sy, ex, ey, total_time)`.
    ///
    /// The walk itself is [`crate::diff::ammo::blocks`], which item 402
    /// widened to the whole twenty-seven-field record; this is the five
    /// numbers §22 is about. One parser, because the walk has a trap in it
    /// — `Objects::dump_ammo` prints a frame's arrows as consecutive
    /// siblings and a version that reset rather than flushed on the open
    /// lost six of nine of them silently.
    fn ammo_blocks(body: &str) -> Vec<(i64, i64, i64, i64, i64)> {
        crate::diff::ammo::blocks(body)
            .into_iter()
            .map(|(a, _)| (a.sx, a.sy, a.ex, a.ey, a.total_time))
            .collect()
    }

    #[test]
    fn run76_and_run79_date_the_tobacco_rare_on_the_tower_s_clock() {
        let Some(inst) = install() else { return };
        let (Some(r76), Some(r79), Some(tr)) = (
            dump("gamelog-run76-greatlakes-archermarch.txt"),
            dump("gamelog-run79-greatlakes-secondsquad.txt"),
            trace("rontrace-run79.log"),
        ) else {
            eprintln!("skipping: no run76/run79 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let t76 = crate::capture::read(&r76);
        let t79 = crate::capture::read(&r79);
        let l76 = Log::parse(&t76);
        let l79 = Log::parse(&t79);

        // ---- the original's own record, off the disk ----
        let tower = |log: &Log| -> Vec<(i64, i64, i64, i64)> {
            log.frame_states()
                .into_iter()
                .filter_map(|f| {
                    let b = f.builds.iter().find(|b| b.who == 1 && b.o == 2017)?;
                    Some((f.n, b.constr_time?, b.job_counter?, b.flags))
                })
                .collect()
        };
        let early = tower(&l76);
        let late = tower(&l79);
        assert!(
            early.len() >= 220 && late.len() >= 330,
            "run76 gives {} Tower blocks and run79 {} — the wrong files",
            early.len(),
            late.len()
        );
        // The transition, to the frame: the last 100000 and the first
        // 90909 are consecutive blocks, and 6752 is the second.
        let baked = early
            .iter()
            .find(|&&(_, ct, _, _)| ct == 90_909)
            .map(|&(n, _, _, _)| n);
        assert_eq!(baked, Some(6752), "run76 dates the re-bake: {early:?}");
        assert!(
            early
                .iter()
                .all(|&(n, ct, _, _)| ct == if n < 6752 { 100_000 } else { 90_909 }),
            "run76's Tower clock is 100000 then 90909 and nothing else"
        );
        assert!(
            late.iter().all(|&(_, ct, _, _)| ct == 90_909),
            "run79's Tower clock holds the baked 90909 to the end"
        );
        // And the completion: `job_counter` climbs 150 a frame to 90800
        // and is zeroed on 7176, where `flags` takes the `4` bit.
        let done = late
            .iter()
            .find(|&&(_, _, _, flags)| flags & 4 != 0)
            .map(|&(n, _, jc, _)| (n, jc));
        assert_eq!(done, Some((7176, 0)), "run79 dates the Tower's finish");
        let before = late
            .iter()
            .find(|&&(n, _, _, _)| n == 7175)
            .map(|&(_, _, jc, _)| jc);
        assert_eq!(before, Some(90_800), "the last block before the finish");

        // ---- and this crate, frame for frame ----
        let loaded = crate::load::load(&inst).unwrap();
        let texts = sibling_texts();
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let report = run_traced(&loaded, &l79, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        let clock: Vec<&BuildDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.build_diverged.iter())
            .filter(|d| d.field == "constr_time" || d.field == "job_counter")
            .collect();
        assert!(
            clock.is_empty(),
            "the construction clock parted on {} of run79's building rows, \
             first eight: {:?}",
            clock.len(),
            &clock[..clock.len().min(8)]
        );
        let compared: usize = report.frames.iter().map(|f| f.build_compared).sum();
        assert!(
            compared >= 16_000,
            "run79's building rows: {compared} — a capture without BUILDDATA \
             would pass this test saying nothing"
        );
    }
}
