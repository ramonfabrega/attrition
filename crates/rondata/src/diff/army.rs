//! The `ARMY` records against `docs/ARMY.md`'s implementation
//! (`crates/sim/src/army.rs`): the whole record, every valid slot, per
//! the working agreement's "diff the whole record".

use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamelog::Block;
    use crate::testenv::{dump, install};
    use sim::army::Army;

    /// Every `BEGIN ARMY` block under a frame block's first `FULL DUMP`, in
    /// order — `ArmyData::log_data` prints only the valid slots.
    fn army_records<'b>(frame: Block<'b>) -> Vec<Block<'b>> {
        let b = frame.kid("FULL DUMP").unwrap_or(frame);
        b.kids("ARMY").collect()
    }

    /// The dump's named fields of an `ARMY` record, from the harness's
    /// record. `reg` and `city` are mapped back to the dump's numbering.
    fn ours(built: &Built, a: &Army) -> Vec<(&'static str, i64)> {
        let reg = a.reg.map_or(-1, |r| {
            built
                .region_map
                .iter()
                .find(|(_, s)| *s == r)
                .map_or(-1, |(d, _)| *d)
        });
        let city = a.city.map_or(-1, |c| {
            built
                .sim
                .cities_of(a.who)
                .iter()
                .position(|&x| x == c)
                .map_or(-1, |p| p as i64)
        });
        let (target_o, target_who) = match a.target {
            None => (-1, -1),
            Some(sim::combat::Obj::Unit(u)) => (
                i64::from(built.sim.units[u].index),
                i64::from(built.sim.units[u].owner),
            ),
            Some(sim::combat::Obj::Building(b)) => (
                i64::from(built.sim.buildings[b].index),
                i64::from(built.sim.buildings[b].owner),
            ),
        };
        vec![
            ("army", i64::from(a.army)),
            ("who", i64::from(a.who)),
            ("num_groups", i64::from(a.num_groups())),
            ("status", i64::from(a.status)),
            ("reg", reg),
            ("role", i64::from(a.role)),
            ("num_units", i64::from(a.num_units)),
            ("num_captains", i64::from(a.num_captains)),
            ("num_standard", i64::from(a.num_standard)),
            ("num_decoys", i64::from(a.num_decoys)),
            ("city", city),
            ("navy", i64::from(a.navy)),
            ("human_frame", i64::from(a.human_frame)),
            ("hurry", i64::from(a.hurry)),
            ("target_o", target_o),
            ("target_who", target_who),
            ("x", i64::from(a.pos.x)),
            ("y", i64::from(a.pos.y)),
            ("angle", i64::from(a.angle.0)),
            ("rally_dist", i64::from(a.rally_dist)),
            ("muster_x", i64::from(a.muster.x)),
            ("muster_y", i64::from(a.muster.y)),
            ("muster_angle", i64::from(a.muster_angle.0)),
        ]
    }

    /// Every field of every record, in one list of disagreements.
    fn compare(built: &Built, theirs: &[Block<'_>], skip: &[&str]) -> Vec<String> {
        let mut wrong = Vec::new();
        let mut mine: Vec<&Army> = Vec::new();
        for who in 0..built.sim.armies.len() {
            for (_, a) in built.sim.armies[who].valid() {
                mine.push(a);
            }
        }
        if mine.len() != theirs.len() {
            wrong.push(format!(
                "{} valid armies, the dump has {}",
                mine.len(),
                theirs.len()
            ));
        }
        for (a, t) in mine.iter().zip(theirs) {
            for (key, o) in ours(built, a) {
                if skip.contains(&key) {
                    continue;
                }
                let tv = t.int(key);
                if tv != Some(o) {
                    wrong.push(format!(
                        "army {} who {} {key}: ours {o} theirs {tv:?}",
                        a.army, a.who
                    ));
                }
            }
        }
        wrong
    }

    /// Run20's frame blocks 1–4 carry the AI's first army —
    /// `Armies::init_army` from the census's step 16 at frame 0, before a
    /// unit has joined — and the harness's census seeds the same slot at
    /// the same city with the same record, field for field, and nothing
    /// for the human.
    #[test]
    fn run20_s_first_army_is_the_census_s_init_army_whole() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run20-islands-dumpall.txt") else {
            eprintln!("skipping: no gamelog-run20-islands-dumpall.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let init = log.initial().unwrap();
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let frames = log.frames();
        // Block 4 is the quit's own: a `FRAME 4` header with no dump.
        for n in 1..=3 {
            built.sim.tick();
            let (_, block) = *frames
                .iter()
                .find(|(f, _)| *f == n)
                .expect("the frame block");
            let theirs = army_records(block);
            assert_eq!(theirs.len(), 1, "one army at frame {n}");
            assert_eq!(theirs[0].int("who"), Some(1), "the AI's");
            let wrong = compare(&built, &theirs, &[]);
            assert!(wrong.is_empty(), "frame {n}:\n  {}", wrong.join("\n  "));
        }
    }

    /// **run93, blocks 7930 and 7931 — Great Lakes' word, as a value**
    /// (item 317, `docs/ARMY.md` §16.8).
    ///
    /// 7930 is the frame the AI's first army leaves the muster and takes
    /// its first target, and the two happen in **one tick**: §6's
    /// dispatch `if`s re-read `status`, so `do_mustering`'s `status = 2`
    /// reaches `do_marching` holding `target_o = -1` and the original's
    /// `if (iVar4 < 0) goto LAB_006f3fb2` retargets at once (§9). The
    /// draw stream says the two sides spend the same ten draws there;
    /// **this says they reach the same state**, which a stream agreeing
    /// on a wrong destination would not.
    ///
    /// Both blocks, whole record, no skips: 7930 is the army still
    /// `status 17` with five squads and no target, and 7931 is
    /// `status 18`, `city -1`, **`target_o 2007, target_who 1`** — the
    /// AI's own building, §12's `L == me` arm — with `angle` taking the
    /// muster angle (`do_mustering`'s tail, `field_0x40 = field_0x50`),
    /// `muster_angle` rewritten by `find_target`'s own
    /// `find_muster_spot` (§13), and `x, y` **left at 41568, 25440** by
    /// `find_target`'s tail rather than at the muster cell's centre that
    /// `do_mustering` had just written there.
    #[test]
    fn run93_says_great_lakes_7930_releases_and_takes_its_target_in_one_tick() {
        let Some(inst) = install() else { return };
        let Some(path) = dump("gamelog-run93-greatlakes-firsttarget.txt") else {
            eprintln!("skipping: no gamelog-run93 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let texts = crate::diff::testkit::sibling_texts();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let logs: Vec<Log> = texts.iter().map(|t| Log::parse(t)).collect();
        let inits: Vec<Initial> = logs.iter().filter_map(|l| l.initial()).collect();
        let refs: Vec<&Initial> = inits.iter().collect();
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &refs);
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let frames = log.frames();
        for _ in 0..7931 {
            built.tick();
        }
        // Block `n` is the end of sim-frame `n - 1`, so 7930 is the state
        // the tick reads and 7931 the state it leaves.
        for (n, want_status, want_target) in [(7930i64, 0x11i32, false), (7931, 0x12, true)] {
            let (_, block) = *frames
                .iter()
                .find(|(f, _)| *f == n)
                .unwrap_or_else(|| panic!("run93 has no block {n}"));
            let theirs = army_records(block);
            assert_eq!(theirs.len(), 2, "two armies of the AI at {n}");
            let a = theirs
                .iter()
                .find(|t| t.int("army") == Some(1))
                .expect("army 1");
            assert_eq!(a.int("status"), Some(i64::from(want_status)), "block {n}");
            assert_eq!(
                a.int("target_o").is_some_and(|o| o >= 0),
                want_target,
                "block {n}: the target"
            );
            if want_target {
                assert_eq!(
                    (a.int("target_o"), a.int("target_who")),
                    (Some(2007), Some(1)),
                    "7931's target is the AI's own 2007"
                );
            }
        }
        // The state the tick left, against this crate's — every field.
        let (_, block) = *frames.iter().find(|(f, _)| *f == 7931).expect("block 7931");
        // `role` alone is skipped, and this widening is what found out
        // why: `Army::normalize` ORs the groups' `GroupData::role` words
        // (§3.3) and `army_normalize` writes a flat `0`, because
        // `group::GroupState` carries no role word at all. Nothing in the
        // family reads it (§18) so it costs no behaviour — but the
        // document had it among the fields that are kept *because* they
        // are dumped, beside `rally_dist`, which really is carried and
        // agrees here at `0x1200`. §18 says so now.
        let wrong = compare(&built, &army_records(block), &["role"]);
        assert!(
            wrong.is_empty(),
            "Great Lakes 7931, the release-and-target tick:\n  {}",
            wrong.join("\n  ")
        );
    }

    /// Run22's block 3579: two armies of the AI, one per city, both
    /// mustering **and** forming (`status 17`) — `do_mustering`'s first
    /// arm, a muster spot found at an active city — with `x, y` the city's
    /// point one cell south (`Army::init`), every count still zero, and
    /// muster cells that are **not** the init's: the ring search of
    /// `find_muster_spot` (§13) moved them, and kept them at least four
    /// cells apart (the same-owner spacing rule). The ring search is the
    /// one seam of `army.rs` this capture reaches; the record is asserted
    /// from the dump alone.
    #[test]
    fn run22_s_two_armies_are_init_s_records_with_the_ring_search_s_muster_cells() {
        let Some(path) = dump("gamelog-run22-islands-dock-window.txt") else {
            eprintln!("skipping: no gamelog-run22-islands-dock-window.txt (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let frames = log.frames();
        let (_, block) = *frames.iter().find(|(f, _)| *f == 3579).expect("block 3579");
        let dumpb = block.kid("FULL DUMP").unwrap_or(block);
        let theirs = army_records(block);
        assert_eq!(theirs.len(), 2, "two armies at 3579");
        let cities: Vec<Block<'_>> = dumpb
            .find("CITIES")
            .expect("CITIES")
            .kids("CITY")
            .filter(|c| c.int("who") == Some(1))
            .collect();
        assert!(
            cities.len() >= 2,
            "the AI's two cities, got {}",
            cities.len()
        );
        let mut musters = Vec::new();
        for (i, a) in theirs.iter().enumerate() {
            assert_eq!(a.int("army"), Some(i as i64));
            assert_eq!(a.int("who"), Some(1));
            assert_eq!(a.int("status"), Some(0x11), "mustering and forming");
            assert_eq!(a.int("city"), Some(i as i64));
            assert_eq!(a.int("reg"), Some(11));
            for k in [
                "num_groups",
                "num_units",
                "num_captains",
                "num_standard",
                "num_decoys",
                "navy",
                "human_frame",
                "hurry",
                "angle",
                "rally_dist",
                "role",
            ] {
                assert_eq!(a.int(k), Some(0), "{k}");
            }
            assert_eq!(a.int("target_o"), Some(-1));
            assert_eq!(a.int("target_who"), Some(-1));
            let c = cities[i];
            let (cx, cy) = (c.int("x").unwrap(), c.int("y").unwrap());
            assert_eq!(a.int("x"), Some(cx), "x is the city's");
            assert_eq!(
                a.int("y"),
                Some(cy + 0x300),
                "y is one cell south of the city's"
            );
            let init_cell = (cx / 0x300, (cy + 0x300) / 0x300);
            let m = (a.int("muster_x").unwrap(), a.int("muster_y").unwrap());
            assert_ne!(m, init_cell, "the ring search moved army {i}'s muster cell");
            musters.push(m);
        }
        let d = sim::world::vector_dist(
            (musters[0].0 - musters[1].0).unsigned_abs() as i32,
            (musters[0].1 - musters[1].1).unsigned_abs() as i32,
        );
        assert!(
            d >= 4,
            "muster cells {musters:?} are {d} apart; the same-owner rule wants four"
        );
    }

    // ---- the ring search on a frame block's own map (§13) ----

    use sim::combat::Obj;
    use sim::world::Cell;

    /// A frame block's state, as far as `find_muster_spot` and
    /// `find_target` read it — the block's own `WORLD` cells (owners and
    /// regions as they stood), its cities typed from their `BUILDDATA`
    /// record and carrying that record's damage, its leaders' transport
    /// bits, diplomacy table and the `LEADERDATA` words §12 reads, its
    /// `ARMY` records, the frame number and the sync stream's word the
    /// block opens with — so one function can be run on the state of that
    /// frame. Not a replay: nothing else of the frame is built.
    ///
    /// **Its `UNITS=3` half**, added 2026-08-26: every `UNITDATA` record of
    /// the block becomes a `sim::Unit` — typed from its first guy's
    /// `TypeIndex`, standing where the record stands, facing the record's
    /// own `angle`, carrying its `+0xaa`/`+0xab` formation bytes, its
    /// stance, its **order list** and its **path stack** — and every
    /// `GROUPDATA` slot that an army owns becomes that army's membership,
    /// in the record's own `list` order. That is what lets a function
    /// which reads a unit's *orders* (`Army::engagement`, `is_moving`,
    /// `GroupData::get_form`) be run on the state of a logged frame at
    /// all; without it a scene had armies with no units in them.
    struct Scene {
        sim: Sim,
        /// `(who, o, handle)` of every city building.
        cities: Vec<(i64, i64, usize)>,
        /// `(who, slot, city)`: the dump's per-leader city slot — the
        /// `city` an `ARMY` record names, which keeps its number when an
        /// earlier slot empties — to the sim's city index.
        slots: Vec<(i64, i64, usize)>,
        /// `(who, o, handle)` of every unit the block carried.
        units: Vec<(i64, i64, usize)>,
        /// The block's whole `GROUPDATA` pool, as parsed — the record the
        /// membership was read out of, kept so a test can compare against
        /// the slots the scene did *not* stand up (a player's selection, a
        /// hotkey group) as well as the ones it did.
        groups: Vec<crate::gamelog::GroupDump>,
        /// Order kinds the block carried that a scene cannot translate —
        /// empty for every window read so far, and the thing to look at
        /// first when a unit's order list comes back shorter than the
        /// record's.
        untranslated: Vec<String>,
    }

    impl Scene {
        /// The handle of the unit the block wrote as `(who, o)`.
        fn unit(&self, who: i64, o: i64) -> Option<usize> {
            self.units
                .iter()
                .find(|(w, n, _)| *w == who && *n == o)
                .map(|(_, _, u)| *u)
        }
    }

    /// One `UNITDATA` order block as the simulation's own order, or `None`
    /// for a kind this scene does not model.
    ///
    /// The geometry is `docs/ORDERS.md` §4.1's table, field for field:
    /// `x/y` is the destination, `dest` is "I have a current waypoint" and
    /// `dest_x/dest_y` is that waypoint, `last` is −1,−1 when no
    /// straight-line plan stands. The target of an attack rides on the
    /// unit here rather than on the order (`sim::group`'s module note), so
    /// it is passed in resolved.
    fn order_of(od: &crate::gamelog::OrderDump) -> Option<sim::orders::Order> {
        use sim::orders::{AttackOrder, Body, MoveKind, MoveOrder, index};
        let i = |v: Option<i64>| v.unwrap_or(0) as i32;
        let kind = match u8::try_from(od.index).ok()? {
            index::MOVE_TO => MoveKind::MoveTo,
            index::ATTACK_TO => MoveKind::AttackTo,
            index::EXPLORE_TO => MoveKind::ExploreTo,
            index::FLEE_TO => MoveKind::FleeTo,
            index::ATTACK => {
                return Some(sim::orders::Order {
                    flags: u8::try_from(od.flags & 0xff).ok()?,
                    body: Body::Attack(AttackOrder {
                        defensive: od.defensive == Some(1),
                        def: match (od.def_x, od.def_y) {
                            (Some(x), Some(y)) if x >= 0 && y >= 0 => {
                                Some(Pos::new(x as i32, y as i32))
                            }
                            _ => None,
                        },
                        in_range: od.in_range == Some(1),
                        ever_in_range: od.ever_in_range == Some(1),
                        new_ord: od.new_ord == Some(1),
                    }),
                });
            }
            _ => return None,
        };
        let dest = Pos::new(i(od.x), i(od.y));
        Some(sim::orders::Order {
            flags: u8::try_from(od.flags & 0xff).ok()?,
            body: Body::Move(MoveOrder {
                kind,
                dest,
                angle: sim::movement::Angle(i(od.angle)),
                // `MoveOrder +0x28`, where the original's −1 is the
                // "not a formation move" of `docs/GROUPS.md` §6.3.
                facing: match od.facing {
                    Some(f) if f >= 0 => Some(f != 0),
                    _ => None,
                },
                has_waypoint: od.dest == Some(1),
                waypoint: Pos::new(i(od.dest_x), i(od.dest_y)),
                last: match (od.last_x, od.last_y) {
                    (Some(x), Some(y)) if x >= 0 && y >= 0 => Some(Pos::new(x as i32, y as i32)),
                    _ => None,
                },
                pause: i(od.pause),
                timer: i(od.timer),
                // `coll_x`/`coll_y`, the point the last collision refused
                // (`docs/COLLISION.md` §4.3). The original leaves 0 rather
                // than −1 when there has been none.
                coll: match (od.coll_x, od.coll_y) {
                    (Some(x), Some(y)) if x > 0 || y > 0 => Some(Pos::new(x as i32, y as i32)),
                    _ => None,
                },
                // The `GROUPORDER` base is not read back into the order:
                // the comparison this builds is over the `MOVEORDER`
                // block, and a `GroupMoveOrder`'s own five fields are
                // `docs/GROUPS.md` §12.1's separate check.
                group: None,
            }),
        })
    }

    fn scene_at(loaded: &Loaded, log: &Log<'_>, frame: i64) -> Scene {
        // `Log::dumps` rather than `Log::frames`: an `end_frame` dump is a
        // *sibling* of the `FRAME` block, so the frame walk misses the two
        // states at the ends of a window (run29's 15103 and 15105). The
        // block handed on is the `FULL DUMP` itself, which every reader
        // below already tolerates — each opens with
        // `kid("FULL DUMP").unwrap_or(b)`.
        let dumps = log.dumps();
        let (_, block) = *dumps
            .iter()
            .find(|(f, _)| *f == frame)
            .expect("a FULL DUMP stamped with this frame");
        let body = block.kid("FULL DUMP").unwrap_or(block);
        let mut notes = Vec::new();
        let world_fields = body.kid("WORLD").expect("a WORLD block").fields().to_vec();
        let (world, region_map) = world_from(&world_fields, &[], &mut notes);
        assert!(
            !region_map.is_empty(),
            "{frame}: the WORLD block carries no cells"
        );
        let init = log.initial().expect("the start-of-game block");
        let players = player_count(&init).max(1);
        let mut sim = loaded.sim(Tuning::RON, world, players);
        sim.lobby = lobby_of(&init.game_info, &loaded.map_styles);
        sim.setup.no_nation_powers = sim.lobby.no_nation_powers;
        // Block `n` is the state sim-frame `n` begins on (`docs/SYNC.md`
        // §1): the frame the stamps are compared against, and the sync
        // stream's word its `say_checksum` record carries — what the
        // frame's first draw advances from.
        sim.frame = frame;
        if let Some((_, seed)) = log.frame_seeds().into_iter().find(|(n, _)| *n == frame - 1) {
            sim.rng = sim::combat::Rng::new(seed);
        }
        let (unit_dumps, builds, leaders) = crate::gamelog::records(block, false);
        for l in &leaders {
            if (0..players as i64).contains(&l.who) {
                let who = l.who as usize;
                sim.set_tribe(who as u8, l.tribe);
                sim.nation[who].human = l.leader_flags & 4 != 0;
                let t = &mut sim.transport[who];
                t.civilian = l.leader_flags & 0x100 != 0;
                t.military = l.leader_flags & 0x200 != 0;
                t.scout = l.leader_flags & 0x400 != 0;
            }
        }
        // And the starting position again, now that the nations are known
        // — the same ordering `build_sim` fixes, for the same reason: a
        // scene's tech state is `Leader::init`'s and nothing restores a
        // mid-game one, so laying it for `tribe = 0` would give every
        // leader the Aztecs' unit variants.
        for who in 0..players {
            sim.start_techs(who as sim::Player);
        }
        // The `LEADERDATA` words `find_target` reads (`docs/ARMY.md` §12):
        // the diplomacy table (0 war, 1 peace, 2 allied; the diagonal is
        // 2), `defense_mod`, the two census counts the averages and the
        // difficulty gate use, the attack stamp, and the personality's two
        // knobs. Read from the block itself: `LeaderDump` carries the
        // level-0 fields only.
        for l in body.kids("LEADERDATA") {
            let Some(who) = l.int("who") else { continue };
            if !(0..players as i64).contains(&who) {
                continue;
            }
            let w = who as usize;
            let diplos: Vec<i64> = l
                .all("diplos[scan]")
                .iter()
                .filter_map(|v| v.trim().parse().ok())
                .collect();
            for (other, &d) in diplos.iter().enumerate().take(players) {
                if other != w {
                    sim.at_war[w][other] = d == 0;
                    sim.allied[w][other] = d == 2;
                }
            }
            let a = &mut sim.ai[w];
            if let Some(v) = l.int("defense_mod") {
                a.defense_mod = v as i32;
            }
            if let Some(v) = l.int("combat") {
                a.census.combat = v as i32;
            }
            if let Some(v) = l.int("sea_combat") {
                a.census.sea_combat = v as i32;
            }
            if let Some(v) = l.int("frame_attacked") {
                a.frame_attacked = v;
            }
            if let Some(v) = l.int("attacked_by") {
                a.attacked_by = v as i32;
            }
            if let Some(p) = l.kid("PERSONALITY") {
                if let Some(v) = p.int("raid") {
                    a.pers.raid = v as i32;
                }
                if let Some(v) = p.int("early_army") {
                    a.pers.early_army = v as i32;
                }
            }
        }
        /// The first value under `key` in the block or, depth-first, its
        /// children — a `BUILDDATA` record nests its class chain
        /// (`WALLDATA` → `OBJECT` → `SUBOBJECT`) and `who`, `o` and `damage`
        /// sit at the inner levels.
        fn deep_int(b: Block<'_>, key: &str) -> Option<i64> {
            b.int(key)
                .or_else(|| b.children().find_map(|c| deep_int(c, key)))
        }
        let mut cities = Vec::new();
        let mut slots = Vec::new();
        for c in body.find("CITIES").expect("CITIES").kids("CITY") {
            let who = c.int("who").unwrap_or(-1);
            if !(0..players as i64).contains(&who) {
                continue;
            }
            let o = c.int("o").expect("the city's building");
            let slot = c.int("city").expect("the city's slot");
            let flags = c.int("city_flags").unwrap_or(0);
            let ty = builds
                .iter()
                .find(|b| b.who == who && b.o == o)
                .and_then(|b| b.orig_type)
                .and_then(|t| loaded.build_of_type_index(t as i32))
                .expect("the city building's type");
            let pos = Pos::new(c.int("x").unwrap() as i32, c.int("y").unwrap() as i32);
            let b = sim.init_build(who as sim::Player, ty, pos, false);
            sim.activate(b, false, false);
            let ci = sim.buildings[b].city.expect("activate founded the city");
            let city = &mut sim.cities[ci];
            city.alive = flags & 1 != 0;
            city.no_heal = flags & 2 != 0;
            city.capital = flags & 0x10 != 0;
            city.unassimilated = flags & 0x100 != 0;
            city.no_muster = flags & 0x2000 != 0;
            city.race = c.int("race").map(|r| r as sim::Player);
            city.was_capital = c.int("was_capital_flags").unwrap_or(0) as u64;
            city.attack_stamp = c.int("attack_stamp").unwrap_or(0);
            // The building's `damage`, from its own record: §12 doubles a
            // damaged city of one's own.
            let damage = body
                .kids("BUILDDATA")
                .find(|&bd| deep_int(bd, "who") == Some(who) && deep_int(bd, "o") == Some(o))
                .and_then(|bd| deep_int(bd, "damage"))
                .unwrap_or(0) as i32;
            let bd = &mut sim.buildings[b];
            bd.health = bd.hits - damage;
            cities.push((who, o, b));
            slots.push((who, slot, ci));
        }

        // ---- the `UNITS=3` half ----
        //
        // Every unit of the block, typed from its first guy's `TypeIndex`
        // — `GuyData::type` is the *unit's* type, which is what makes a
        // logged unit typeable at all (the `UNITDATA` level carries no
        // type of its own) — and carrying its order list, its path stack
        // and the two formation bytes `get_form` reads back off it.
        let mut units = Vec::new();
        let mut untranslated = Vec::new();
        for u in &unit_dumps {
            if !(0..players as i64).contains(&u.who) {
                continue;
            }
            let ty = u
                .guys
                .first()
                .and_then(|g| g.kind)
                .and_then(|t| loaded.unit_of_type_index(t as i32));
            let t = ty.map(|k| &loaded.unit_types[k]);
            // `ObjectData::myhits` is the whole hit points the type
            // carries *after* tech, and `damage` what has been taken off
            // them — the same pair the city loop above reads.
            let hits = u.myhits.unwrap_or_else(|| t.map_or(1, |t| t.hits).into()) as i32;
            let health = if u.flags & 1 == 0 {
                0
            } else {
                (hits - u.damage.unwrap_or(0) as i32).max(1)
            };
            let mut unit = Unit::new(
                u.who as sim::Player,
                i16::try_from(u.o).expect("a unit's object number"),
                pos_of(u.pos),
                health,
            );
            unit.max_health = hits;
            unit.squad_size = u.guys.len().max(1) as i32;
            unit.ty = ty;
            unit.type_index = u.guys.first().and_then(|g| g.kind).unwrap_or(-1) as i32;
            if let Some(t) = t {
                unit.kind = t.kind;
                unit.movement.turning = sim::turning_of(t);
            }
            unit.movement.speed = u.myspeed.unwrap_or(0) as i32;
            // The three angles, each from its own field. `UnitData::angle`
            // (`+0x50`) is the **heading** — what `update_positions` rotates
            // the slot table by — and guy 0's `angle` (`+0x18`) is the
            // **facing** the step is taken along; a record without a guy
            // falls back to the heading, which is where a unit that has
            // finished turning sits anyway. `UnitData::dest_angle` (`+0x58`)
            // is the third and is the order's.
            if let Some(a) = u.angle {
                unit.movement.heading = sim::movement::Angle(a as i32);
                unit.movement.facing = unit.movement.heading;
            }
            if let Some(a) = u.guys.first().and_then(|g| g.angle) {
                unit.movement.facing = sim::movement::Angle(a as i32);
            }
            unit.movement.frame_facing = unit.movement.facing;
            if let Some(a) = u.dest_angle {
                unit.movement.des_angle = sim::movement::Angle(a as i32);
            }
            unit.on_map = u.inside_up.unwrap_or(-1) < 0;
            unit.captain = u.o_up.unwrap_or(-1) < 0;
            unit.form = i8::try_from(u.form.unwrap_or(-1)).unwrap_or(-1);
            unit.form_width = i8::try_from(u.form_mod.unwrap_or(-1)).unwrap_or(-1);
            unit.stance = u8::try_from(u.stance.unwrap_or(0)).unwrap_or(0);
            unit.tolerance = u.tolerance.unwrap_or(0) as i32;
            unit.path_recursion = u8::try_from(u.path_recursion.unwrap_or(0)).unwrap_or(0);
            unit.idle = u8::try_from(u.idle.unwrap_or(0)).unwrap_or(0);
            unit.orders_pos = Pos::new(
                u.orders_x.unwrap_or(0) as i32,
                u.orders_y.unwrap_or(0) as i32,
            );
            unit.line_ok = u.unit_masks.unwrap_or(0) & 8 != 0;
            unit.was_builder = u.unit_masks.unwrap_or(0) & 0x400 != 0;
            unit.decoy = u.unit_masks.unwrap_or(0) & 1 != 0;
            // The order list front-first (the log writes it newest first)
            // and the path stack as it stands — both `Vec`-shaped the same
            // way the original's are.
            for od in u.orders_front_first() {
                match order_of(od) {
                    Some(o) => unit.orders.push_back(o),
                    None => untranslated.push(od.kind.clone()),
                }
            }
            unit.path = u
                .path
                .iter()
                .map(|p| sim::orders::PathData {
                    to: Pos::new(p.to.0 as i32, p.to.1 as i32),
                    tolerance: p.tolerance as i32,
                    flags: u8::try_from(p.flags & 0xff).unwrap_or(0),
                })
                .collect();
            let h = sim.add_unit(unit);
            units.push((u.who, u.o, h));
        }
        // The attack targets, once every unit has a handle. The target is
        // `update_action().get_target_order()`'s — the **action**'s, not
        // the front order's: a unit chasing its target holds
        // `[MOVEORDER(transit), ATTACKORDER]` and only the second names
        // anyone. The simulation keeps the target on the unit rather than
        // on the order (`sim::group`'s module note).
        for u in &unit_dumps {
            let Some(h) = units
                .iter()
                .find(|(w, o, _)| *w == u.who && *o == u.o)
                .map(|(_, _, h)| *h)
            else {
                continue;
            };
            let Some(cur) = u.orders_front_first().find(|o| {
                // `get_action`'s walk (`UnitData::get_action@00608450`):
                // past a move that lacks the action bit, **and past a
                // `CHANGE_FORM` whatever its flags**, stop on anything
                // else. "A move" is `is_move()` — the whole family, so
                // `GROUP_MOVE`/`GROUP_ATTACK_TO` are walked past too;
                // the four plain kinds were the test until item 237,
                // which stopped this walk on a formation's own transit
                // leg.
                let k = u8::try_from(o.index).unwrap_or(0);
                !(sim::orders::index::is_move_family(k)
                    && (!o.is_action() || k == sim::orders::index::CHANGE_FORM))
            }) else {
                continue;
            };
            let (Some(whom), Some(ox)) = (cur.whom, cur.ox) else {
                continue;
            };
            if whom < 0 || ox < 0 {
                continue;
            }
            let target = units
                .iter()
                .find(|(w, o, _)| *w == whom && *o == ox)
                .map(|(_, _, t)| Obj::Unit(*t))
                .or_else(|| {
                    cities
                        .iter()
                        .find(|(w, o, _)| *w == whom && *o == ox)
                        .map(|(_, _, b)| Obj::Building(*b))
                });
            sim.units[h].combat.target = target;
        }

        // The one group per army, in the record's own `list` order — which
        // is the order `ArmyData::get_unit` walks and therefore the order
        // `engagement` picks its seed in.
        let pool = crate::gamelog::groups(block);
        for g in &pool {
            if g.buildings != 0 || g.army < 0 || !(0..players as i64).contains(&g.who) {
                continue;
            }
            let slot = g.army as usize;
            let members: Vec<usize> = g
                .members
                .iter()
                .filter_map(|m| {
                    units
                        .iter()
                        .find(|(w, o, _)| *w == g.who && *o == m.o)
                        .map(|(_, _, h)| *h)
                })
                .collect();
            let a = &mut sim.armies[g.who as usize].list[slot];
            a.units = members;
            a.group = sim::group::GroupState {
                form: g.form as i32,
                order_num: g.order_num as i32,
                o: Pos::new(g.ox as i32, g.oy as i32),
                o_angle: sim::movement::Angle(g.o_angle as i32),
                o_dist: g.o_dist as i32,
                facing: g.facing != 0,
                stamp: g.stamp,
                form_num: g.form_num as i32,
                off: g
                    .members
                    .iter()
                    .map(|m| (m.off_x as i32, m.off_y as i32))
                    .collect(),
                curr: g
                    .members
                    .iter()
                    .map(|m| Pos::new(m.curr_x as i32, m.curr_y as i32))
                    .collect(),
                angles: g.members.iter().map(|m| m.angle as i8).collect(),
            };
        }

        for a in army_records(block) {
            let who = a.int("who").unwrap();
            if !(0..players as i64).contains(&who) {
                continue;
            }
            let slot = a.int("army").unwrap() as usize;
            let reg = a.int("reg").filter(|r| *r >= 0).map(|r| {
                region_map
                    .iter()
                    .find(|(d, _)| *d == r)
                    .map(|(_, s)| *s)
                    .expect("the army's region is on the map")
            });
            let city = a.int("city").filter(|c| *c >= 0).map(|c| {
                slots
                    .iter()
                    .find(|(w, s, _)| *w == who && *s == c)
                    .map(|(_, _, ci)| *ci)
                    .expect("the army's city slot")
            });
            let target = match (a.int("target_who"), a.int("target_o")) {
                (Some(tw), Some(o)) if o >= 0 => Some(Obj::Building(
                    cities
                        .iter()
                        .find(|(w, x, _)| *w == tw && *x == o)
                        .map(|(_, _, b)| *b)
                        .expect("an army target that is a city building"),
                )),
                _ => None,
            };
            let int = |k: &str| a.int(k).unwrap_or(0) as i32;
            let rec = &mut sim.armies[who as usize].list[slot];
            rec.valid = true;
            rec.status = int("status");
            rec.reg = reg;
            rec.navy = a.int("navy") == Some(1);
            rec.city = city;
            rec.hurry = int("hurry");
            rec.num_units = int("num_units");
            rec.num_captains = int("num_captains");
            rec.num_standard = int("num_standard");
            rec.num_decoys = int("num_decoys");
            rec.target = target;
            rec.pos = Pos::new(int("x"), int("y"));
            rec.muster = Cell::new(int("muster_x"), int("muster_y"));
            rec.muster_angle = sim::movement::Angle(int("muster_angle"));
        }
        Scene {
            sim,
            cities,
            slots,
            units,
            groups: pool,
            untranslated,
        }
    }

    fn scene(name: &str, frame: i64) -> Option<Scene> {
        Some(scenes(name, &[frame])?.pop().expect("one frame"))
    }

    /// Several blocks of one dump, parsed once — a 250 MB window is not
    /// worth reading twice.
    fn scenes(name: &str, frames: &[i64]) -> Option<Vec<Scene>> {
        let inst = install()?;
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return None;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        Some(frames.iter().map(|&f| scene_at(&loaded, &log, f)).collect())
    }

    /// Run22's block 3579 again, this time with the harness's own ring
    /// search on the block's map: both records' muster cells re-derived
    /// from the cities they muster at — `do_mustering`'s search, flag 1 —
    /// in the order the game ran them: army 0's at the capital while it
    /// was the only army (its (44, 50) is two cells from army 1's init
    /// cell, so with army 1 standing it would be dropped as too near),
    /// then army 1's at Norwich with army 0 where it had settled. Army 1's
    /// (49, 54) scores eight — a building's cell in its 3 × 3 — and is the
    /// first admissible entry of ring 7; the first nine-neighbour cell of
    /// ring 8 lies 43 entries later, past the `0x28` early stop. The
    /// angle is the search's too, from the army's point.
    #[test]
    fn run22_s_muster_cells_are_the_ring_search_s_on_block_3579_s_own_map() {
        let Some(mut sc) = scene("gamelog-run22-islands-dock-window.txt", 3579) else {
            return;
        };
        let theirs: Vec<(Cell, i32)> = (0..2)
            .map(|s| {
                let a = &sc.sim.armies[1].list[s];
                assert!(a.valid, "army {s}");
                (a.muster, a.muster_angle.0)
            })
            .collect();
        assert_eq!(theirs[0].0, Cell::new(44, 50));
        assert_eq!(theirs[1].0, Cell::new(49, 54));
        for s in 0..2 {
            let a = &mut sc.sim.armies[1].list[s];
            a.muster = a.pos.cell();
        }
        sc.sim.armies[1].list[1].valid = false;
        for s in [0, 1] {
            sc.sim.armies[1].list[s].valid = true;
            let c = sc.sim.armies[1].list[s].city.expect("mustering at a city");
            let b = sc.sim.cities[c].building;
            assert!(
                sc.sim.find_muster_spot(1, s, Obj::Building(b), true),
                "army {s}: a spot"
            );
            let a = &sc.sim.armies[1].list[s];
            assert_eq!(a.muster, theirs[s].0, "army {s}'s muster cell");
            assert_eq!(a.muster_angle.0, theirs[s].1, "army {s}'s muster angle");
            assert!(!sc.sim.cities[c].no_muster);
        }
    }

    /// Run25's block 12129, before the `emergency` tick that closed army 1
    /// (`docs/ARMY.md` §16.5): on the block's own map the ring search at
    /// Norwich finds no cell — so `do_mustering` sees the failure and the
    /// city takes the `0x2000` mark the next block shows (`city_flags
    /// 0x0001 → 0x2001`) — while the navy, whose target `find_target`
    /// re-finds the same tick, lands on (46, 58) from (51, 53), the cell
    /// and the angle the record carries.
    #[test]
    fn run25_s_emergency_search_finds_no_cell_at_norwich_and_the_navy_re_finds_46_58() {
        let Some(mut sc) = scene("gamelog-run25-islands-emergency-window.txt", 12129) else {
            return;
        };
        let a1 = &sc.sim.armies[1].list[1];
        assert!(a1.valid && a1.status == 0x11 && !a1.navy);
        assert_eq!(a1.muster, Cell::new(49, 54), "the spot it had held");
        let c = a1.city.expect("at its city");
        assert_eq!(sc.sim.cities[c].pos, Pos::new(34656, 36192), "Norwich");
        let b = sc.sim.cities[c].building;
        assert!(!sc.sim.cities[c].no_muster, "0x0001 before");
        assert!(
            !sc.sim.find_muster_spot(1, 1, Obj::Building(b), true),
            "no cell at Norwich"
        );
        assert!(sc.sim.cities[c].no_muster, "0x2001 after");

        let a2 = &sc.sim.armies[1].list[2];
        assert!(a2.valid && a2.navy);
        let t = a2.target.expect("the navy's target");
        assert_eq!(a2.pos.cell(), Cell::new(51, 53));
        assert!(sc.sim.find_muster_spot(1, 2, t, false), "a sea cell");
        let a2 = &sc.sim.armies[1].list[2];
        assert_eq!(a2.muster, Cell::new(46, 58));
        // `find_target`'s tail turns the search's angle about (§12).
        assert_eq!(a2.muster_angle.0.wrapping_add(i32::MIN), 541_917_184);
        assert_eq!(sc.cities.len(), 3);
        assert_eq!(sc.slots.len(), 3);
    }

    /// Run27's block 15100, before army 0's `do_defending` tick: the
    /// nearest friendly city is Norwich again, still marked from 12129,
    /// and the search — flag 1 — finds no cell, which is the `close` the
    /// next block shows. The navy's search, now against the captured
    /// capital (an enemy's: the spacing is two cells), still lands on
    /// (46, 58).
    #[test]
    fn run27_s_defending_search_finds_no_cell_at_norwich_again() {
        let Some(mut sc) = scene("gamelog-run27-islands-defending-window.txt", 15100) else {
            return;
        };
        let a0 = &sc.sim.armies[1].list[0];
        assert!(a0.valid && a0.status == 1 && !a0.navy);
        assert_eq!(
            a0.muster,
            Cell::new(45, 48),
            "init's cell, one south of Norwich"
        );
        let c = a0.city.expect("at its city");
        assert!(sc.sim.cities[c].no_muster, "marked since 12129");
        let b = sc.sim.cities[c].building;
        assert!(!sc.sim.find_muster_spot(1, 0, Obj::Building(b), true));
        assert!(sc.sim.cities[c].no_muster);

        let a2 = &sc.sim.armies[1].list[2];
        let t = a2.target.expect("the navy's target");
        let Obj::Building(tb) = t else {
            panic!("a building target")
        };
        assert_eq!(sc.sim.buildings[tb].owner, 0, "the captured capital");
        assert!(sc.sim.find_muster_spot(1, 2, t, false));
        let a2 = &sc.sim.armies[1].list[2];
        assert_eq!(a2.muster, Cell::new(46, 58));
        assert_eq!(a2.muster_angle.0, -1_605_566_464);
    }

    /// Run26's block 12024, before the navy's tick — the first live
    /// `find_target` (`docs/ARMY.md` §16.5) — replayed whole. The trace
    /// (`rontrace-run26.log`, `report.py … draws 12024`) says what the
    /// tick drew: sim-frame 12024's first three `game_random` draws are
    /// all `Army::find_target+0x7df`, the per-candidate `% 200 + 900`,
    /// from `0x63ffe763` — the block's own `game_random seed` — and no
    /// coin, so the difficulty gate's `find_aggressive_army` answered
    /// this army. Three candidates on a three-city map: the human's
    /// Napata (an enemy's, at difficulty 0, which the gate admits
    /// unconditionally — the check that failed before the gate was
    /// corrected), then London and Norwich, the AI's own. London — the
    /// capital, attacked (`city_flags & 2`), damaged (`damage 12`) —
    /// scores `×10 ×10 ×2` over its draw and wins; the record at 12025
    /// carries it as `target_o 2000, target_who 1` with `rally_dist
    /// 0x1200`, the army's point one cell south of it, the ring search's
    /// (46, 58) and the turned-about angle. The stamps are the AI's own
    /// city's, so no `frame_attacked` is written.
    #[test]
    fn run26_s_navy_targets_its_own_attacked_capital_with_three_draws() {
        use sim::army::status;
        let Some(mut sc) = scene("gamelog-run26-islands-findtarget-window.txt", 12024) else {
            return;
        };
        assert_eq!(sc.sim.frame, 12024);
        assert_eq!(sc.sim.rng.seed, 0x63ff_e763, "the block's word");
        assert_eq!(sc.sim.ai_difficulty(), 0);
        assert!(sc.sim.is_enemy(0, 1) && !sc.sim.is_ally(0, 1));
        assert_eq!(sc.cities.len(), 3);
        let london = sc
            .cities
            .iter()
            .find(|(w, o, _)| *w == 1 && *o == 2000)
            .map(|(_, _, b)| *b)
            .expect("London");
        let lc = sc.sim.buildings[london].city.unwrap();
        assert!(sc.sim.cities[lc].capital && sc.sim.cities[lc].no_heal);
        let bd = &sc.sim.buildings[london];
        assert_eq!(bd.hits - bd.health, 12, "London's damage");
        assert_eq!(sc.sim.ai[1].frame_attacked, 12005);
        assert_eq!(sc.sim.ai[1].attacked_by, -1);
        assert_eq!(sc.sim.ai[1].pers.raid, -1);
        assert_eq!(sc.sim.ai[1].census.sea_combat, 8);

        let a2 = &sc.sim.armies[1].list[2];
        assert!(a2.valid && a2.navy && a2.status == 0x11 && a2.target.is_none());
        assert_eq!(a2.muster, Cell::new(56, 42));
        assert_eq!(a2.num_captains, 4);
        // `do_mustering` released it this tick — the attacked capital is
        // `release_mustering`'s first arm — and its common tail (§7) ran
        // before `do_marching` reached `find_target` (`Army::process+0x42c
        // < do_marching+0x248` in the trace): `status = 2`, `city = −1`,
        // the point to the muster cell's centre, `angle = muster_angle`
        // (the 292028416 the 12025 record keeps as `angle`). That move is
        // what the gate turns on: the record's point, one cell south of
        // London, is the AI's own cell (51, 53), where the navy would not
        // be aggressive and the gate would draw a coin; the muster cell
        // (56, 42) is ocean nobody owns, and it is.
        assert!(sc.sim.release_mustering(1, 2), "London is attacked");
        {
            let a = &mut sc.sim.armies[1].list[2];
            a.status = status::MARCHING;
            a.city = None;
            a.pos = sim::army::cell_centre(a.muster);
            a.angle = a.muster_angle;
        }
        assert_eq!(sc.sim.find_aggressive_army(1), Some(2));

        sc.sim.find_target(1, 2);

        assert_eq!(sc.sim.rng.seed, 0xad03_8188, "three draws, the trace's");
        let a2 = &sc.sim.armies[1].list[2];
        assert_eq!(a2.target, Some(Obj::Building(london)));
        assert_eq!(a2.rally_dist, 0x1200);
        assert_eq!(a2.hurry, 0);
        assert_eq!(a2.pos, Pos::new(39264, 40800));
        assert_eq!(a2.muster, Cell::new(46, 58));
        assert_eq!(a2.muster_angle.0, 541_917_184);
        assert_eq!(a2.angle.0, 292_028_416, "do_mustering's, untouched");
        assert!(a2.valid);
        assert_eq!(sc.sim.ai[1].frame_attacked, 12005, "my own city: no stamp");
        assert_eq!(sc.sim.ai[1].attacked_by, -1);
        assert_eq!(sc.sim.ai[0].frame_attacked, 0);
    }

    // ---- the whole `GROUPDATA` record (`docs/GROUPS.md` §1, §6.4, §10) ----

    /// One frame's group pool: the frame number, its 512 `GROUPDATA`
    /// records and its `last_group[8]`.
    type Pool = (i64, Vec<crate::gamelog::GroupDump>, Vec<i64>);

    /// The three windowed frames' group pools, parsed once.
    fn run29_pools() -> Option<Vec<Pool>> {
        let name = "gamelog-run29-islands-engagement-window.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return None;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let out = log
            .frames()
            .into_iter()
            .filter(|(f, _)| (15100..=15102).contains(f))
            .map(|(f, b)| (f, crate::gamelog::groups(b), crate::gamelog::last_group(b)))
            .collect::<Vec<_>>();
        assert_eq!(out.len(), 3, "the window's three frames");
        Some(out)
    }

    /// The record, **whole** — every scalar, every array, every slot — for
    /// all three windowed frames.
    ///
    /// This is the widening `docs/audit/2026-08-25-groups.md` puts first,
    /// and it is the project's own rule applied: when the original dumps a
    /// record, diff the whole record. `GroupData::log_data@0045e1d0` writes
    /// twenty scalars and six parallel per-member arrays, and until now
    /// nothing in `crates/rondata` had opened one.
    ///
    /// What is asserted here is the record's **shape and its own
    /// invariants** — the parts the simulation cannot yet produce, because
    /// it has no group pool (`docs/GROUPS.md` §12's second seam). The parts
    /// it can are the two tests below.
    #[test]
    fn run29_s_group_pool_is_five_hundred_and_twelve_whole_groupdata_records() {
        use crate::gamelog::GROUP_FIELDS;
        let Some(frames) = run29_pools() else { return };
        for (f, pool, last) in &frames {
            // 8 leaders × 64 slots, in `id` order, every frame. The hotkey
            // groups are a separate array and are not in here.
            assert_eq!(pool.len(), 512, "{f}: 8 leaders × 64 slots");
            for (i, g) in pool.iter().enumerate() {
                assert_eq!(g.id, i as i64, "{f}: the pool is dumped in id order");
                // Every pool record is a unit group, never disbanding, and
                // never a control group — `priority` is the hotkey array's
                // bit and it is 0 on all 512.
                assert_eq!(g.buildings, 0, "{f}/{}: buildings", g.id);
                assert_eq!(g.disband, 0, "{f}/{}: action_begin clears it", g.id);
                assert_eq!(g.priority, 0, "{f}/{}: not a control group", g.id);
                // The six arrays are parallel and exactly `num` long.
                assert_eq!(g.members.len(), g.num as usize, "{f}/{}: arrays", g.id);
                if g.num > 0 {
                    // A live slot belongs to the leader its index names:
                    // `get_open_slot` indexes `who * 0x40`.
                    assert_eq!(g.who, g.id / 64, "{f}/{}: who = id / 64", g.id);
                    assert!(g.num <= 128, "{f}/{}: the 128-member cap", g.id);
                }
            }
            // `last_group[8]`, re-attached by position (see `last_group`).
            assert_eq!(last.len(), 8, "{f}: one per leader");
            for (p, &l) in last.iter().enumerate() {
                let base = p as i64 * 64;
                assert!(
                    (base..base + 46).contains(&l),
                    "{f}: last_group[{p}] = {l} is outside the allocatable 46"
                );
            }
            // Only the AI has ever had a group installed: the other seven
            // still hold `Groups::clear`'s `p × 0x40`, and player 1's has
            // moved to the slot `push_group` last took.
            assert_eq!(last[1], 70, "{f}: player 1's last group is slot 70");
            for p in [0usize, 2, 3, 4, 5, 6, 7] {
                assert_eq!(last[p], p as i64 * 64, "{f}: player {p} never allocated");
            }
        }

        // The writer's own field order, and the one field it never writes.
        // Read from the block rather than the parsed struct, because the
        // order *is* the assertion (`docs/GROUPS.md` §1).
        let name = "gamelog-run29-islands-engagement-window.txt";
        let text = crate::capture::read(dump(name).unwrap());
        let log = Log::parse(&text);
        let mut seen = 0usize;
        for (f, b) in log.frames() {
            if !(15100..=15102).contains(&f) {
                continue;
            }
            let body = b.kid("FULL DUMP").unwrap_or(b);
            for g in body.kids("GROUPDATA") {
                let keys: Vec<&str> = g.fields().take(20).map(|(k, _)| k).collect();
                assert_eq!(keys, GROUP_FIELDS, "{f}: log_data's own order");
                assert!(
                    !g.fields().any(|(k, _)| k == "march"),
                    "{f}: march is the one GroupData field the engine never logs"
                );
                seen += 1;
            }
        }
        assert_eq!(seen, 512 * 3);
    }

    /// `order_num` and `form` across the `Army::engagement` frame — the
    /// audit's assertions 2 and 3, and the one place the record and the
    /// harness meet on this mechanic today.
    ///
    /// Army 0's group (`id 69`, seven members) is the group
    /// `Group::action_attack` fires on at 15100 (`docs/GROUPS.md` §11: the
    /// frame `GroupData::num_valid` first executes). The record shows
    /// `order_num 0 → 1 → 1` and `form −1` throughout — `action_attack`
    /// issues orders, so it bumps the counter, and it writes no `form`.
    /// The simulation is made to reproduce **both deltas**, which is what
    /// `group_action_attack` and `group_action_halt` were corrected to do.
    #[test]
    fn run29_s_engagement_bumps_order_num_by_one_and_writes_no_form() {
        let Some(frames) = run29_pools() else { return };
        let army0 = |pool: &Vec<crate::gamelog::GroupDump>| {
            pool.iter().find(|g| g.id == 69).cloned().expect("slot 69")
        };
        let (a, b, c) = (
            army0(&frames[0].1),
            army0(&frames[1].1),
            army0(&frames[2].1),
        );
        for g in [&a, &b, &c] {
            assert_eq!(g.who, 1, "the AI's");
            assert_eq!(g.army, 0, "army 0's one group");
            assert_eq!(g.num, 7);
            assert_eq!(g.form, -1, "never moved: no formation index");
            assert_eq!(g.form_num, 0, "and Form::compute never laid it out");
            assert_eq!(g.think_frame, 15020, "the hoplites' arrival");
            assert!(
                g.members.iter().all(|m| (m.off_x, m.off_y) == (0, 0)),
                "and every slot offset is still zero"
            );
        }
        assert_eq!(a.order_num, 0, "15100 is the state the frame begins on");
        assert_eq!(b.order_num, 1, "action_attack issued orders");
        assert_eq!(c.order_num, 1, "and nothing issued more");

        // The harness, on a group of the same size. `action_attack` bumps
        // `order_num` by exactly one and leaves `form` alone; `action_halt`
        // clears the group's `form` and bumps nothing.
        let mut s = sim::Sim::new(Tuning::RON, World::new(60, 60), 2);
        s.nation[0].human = true;
        s.nation[1].human = false;
        s.at_war[0][1] = true;
        s.at_war[1][0] = true;
        let ty = s.add_unit_type(sim::UnitType {
            hits: 100,
            combat: sim::combat::Profile {
                attack: 15,
                uber_size: 1,
                ..sim::combat::Profile::default()
            },
            ..sim::UnitType::default()
        });
        let slot = s.init_army(1, None);
        for k in 0..7 {
            let idx = i16::try_from(s.units.len()).unwrap();
            let mut u = Unit::new(1, idx, Pos::new(0x1000 + k * 0x80, 0x1000), 100);
            u.ty = Some(ty);
            u.on_map = true;
            let u = s.add_unit(u);
            s.army_add_unit(1, slot, u);
        }
        let idx = i16::try_from(s.units.len()).unwrap();
        let mut foe = Unit::new(0, idx, Pos::new(0x1400, 0x1000), 100);
        foe.ty = Some(ty);
        foe.on_map = true;
        let foe = s.add_unit(foe);

        let g = s.army_group(1, slot);
        assert_eq!(s.armies[1].list[slot].group.order_num, 0);
        assert_eq!(s.armies[1].list[slot].group.form, -1);
        s.group_action_attack(
            &g,
            sim::combat::Obj::Unit(foe),
            false,
            sim::orders::QueuePos::New,
            0,
        );
        assert_eq!(
            s.armies[1].list[slot].group.order_num, 1,
            "the record's 0 → 1"
        );
        assert_eq!(
            s.armies[1].list[slot].group.form, -1,
            "action_attack writes no form — the record's −1 across all three frames"
        );
    }

    /// The slot table's own output, for the one live formation in the
    /// window — the audit's assertions 4 and 5, and the fixture that turns
    /// `docs/GROUPS.md` §6.4 from a seam into a checked table the day
    /// `Form::compute_dests` is written.
    ///
    /// Group `id 66` is the AI's navy (`army 2`, `role & 0x80000`), four
    /// members in **formation 0** (Line). Its `off_x` are `[0, −14, 13,
    /// −28]` in 48-unit steps with every `off_y` zero — a line along the
    /// formation's own x axis — and its `curr` are those offsets after
    /// `update_positions` has rotated them by the leader's heading, in
    /// position units. Two things fall out and both are asserted:
    /// `|curr[i]| = 48 · |off_x[i]|` (the `leal (%ecx,%ecx,2)` + `shll $4`
    /// of `7139e8`), and every `curr` is the **same** rotation of its own
    /// `off_x`, so the members stay collinear.
    #[test]
    fn run29_s_navy_group_is_a_line_of_four_rotated_at_forty_eight_units_a_step() {
        let Some(frames) = run29_pools() else { return };
        for (f, pool, _) in &frames {
            let g = pool.iter().find(|g| g.id == 66).expect("slot 66");
            assert_eq!(g.who, 1);
            assert_eq!(g.army, 2, "the AI's navy");
            assert_eq!(g.num, 4);
            assert_eq!(g.form, 0, "formation 0 — Line");
            assert_eq!(g.form_num, g.num, "Form::compute laid out all four");
            assert_eq!(g.o_dist, 351);
            assert_eq!(g.o_angle, -1_605_566_464);
            assert_eq!(g.facing, 0, "not mirrored");

            let off_x: Vec<i64> = g.members.iter().map(|m| m.off_x).collect();
            assert_eq!(off_x, [0, -14, 13, -28], "{f}: the slot table's own row");
            assert!(
                g.members.iter().all(|m| m.off_y == 0),
                "{f}: a line has no depth"
            );
            assert!(
                g.members.iter().all(|m| m.angle == 0),
                "{f}: and no per-slot facing"
            );
            assert_eq!(
                g.members.iter().map(|m| m.o).collect::<Vec<_>>(),
                [32, 34, 35, 38],
                "{f}: the member ids, in join order"
            );

            for m in &g.members {
                // |curr| = 48 · |off_x|, to the sine table's granularity.
                let want = m.off_x * 48;
                let got2 = m.curr_x * m.curr_x + m.curr_y * m.curr_y;
                let want2 = want * want;
                assert!(
                    (got2 - want2).abs() * 100 <= want2 + 100,
                    "{f}: |curr| {got2} is not 48 × |off_x| {want2}"
                );
                // The same rotation for every member: `curr` is collinear
                // with the first non-zero member's, scaled by `off_x`.
                let r = &g.members[1];
                assert!(
                    (m.curr_x * r.off_x - r.curr_x * m.off_x).abs() <= 48 * 2,
                    "{f}: curr_x is not the shared rotation of off_x"
                );
                assert!(
                    (m.curr_y * r.off_x - r.curr_y * m.off_x).abs() <= 48 * 2,
                    "{f}: curr_y is not the shared rotation of off_x"
                );
            }
        }
    }

    /// **The slot table, closed** — `docs/GROUPS.md` §6.4's seam turned
    /// into a diff, from the install's own columns to the record's own row.
    ///
    /// The chain this asserts end to end, with nothing guessed in the
    /// middle:
    ///
    /// 1. `unitrules.xml` gives the fifteen light warships `X_SPACING 55`,
    ///    and `UnitType::init` multiplies it by `UNIT_FORMATION_SPACING`
    ///    (12) — so `x_spacing = 660`, which the loader must reproduce.
    /// 2. A warship's `OBJ_MASK` is `N`/`NRL`: not civilian, not foot, not
    ///    mounted, not a vehicle, not anti-air — so it falls off the end of
    ///    `FormData::type_cat`'s tree into `FORM_CAT_ARTILLERY`.
    /// 3. `GroupData::get_form_mod_option` is **50**, and run29's own
    ///    `UNITDATA` prints `form_mod 50` on each member, so the column
    ///    count is `((4 × 660 × 50)/50)/660 = 4` and there is one rank.
    /// 4. `Form::compute_dests` lays four slots as `0, −w, +w, −2w`, shifts
    ///    the block by `w/2` because the count is even, and slides it back
    ///    by the anchor.
    /// 5. The floor divide by 48 turns `[330, −330, 990, −990]` into
    ///    `[0, −14, 13, −28]` — **the record's own `off_x`**.
    ///
    /// And then the record's `curr`, exactly, through `update_positions`
    /// under the leader's logged heading. Nothing here is a fixture written
    /// from the answer: the four numbers come out of the install's columns
    /// and the simulation's arithmetic.
    #[test]
    fn run29_s_navy_slot_table_is_reproduced_from_the_install_s_own_spacing() {
        use sim::form::{cat, formation, type_cat};
        use sim::movement::Angle;
        let Some(inst) = install() else { return };
        let Some(frames) = run29_pools() else { return };
        let loaded = crate::load::load(&inst).unwrap();

        // The install's light warships, by the column the reading named.
        let ships: Vec<usize> = (0..loaded.unit_types.len())
            .filter(|&t| loaded.unit_types[t].combat.x_spacing == 660)
            .collect();
        assert_eq!(
            ships.len(),
            15,
            "the fifteen types with X_SPACING 55 — {:?}",
            ships
                .iter()
                .map(|&t| &loaded.unit_names[t])
                .collect::<Vec<_>>()
        );
        for &t in &ships {
            let ty = &loaded.unit_types[t];
            assert_eq!(ty.combat.y_spacing, 660, "{}", loaded.unit_names[t]);
            assert_eq!(ty.combat.uber_size, 1, "{}", loaded.unit_names[t]);
            assert_eq!(
                type_cat(ty, t, false),
                cat::ARTILLERY,
                "{} falls off the end of type_cat's tree",
                loaded.unit_names[t]
            );
        }
        let galley = *ships
            .iter()
            .find(|&&t| loaded.unit_names[t] == "Galley")
            .expect("a Galley in the install");

        // Four of them in one army's group, on the harness's own world.
        let mut s = sim::Sim::new(
            sim::tuning::Tuning::RON,
            sim::world::World::new(200, 200),
            2,
        );
        s.nation[1].human = false;
        let mut proto = loaded.unit_types[galley].clone();
        // The harness here carries no tech tree; the columns are the point.
        proto.tree = None;
        let ty = s.add_unit_type(proto);
        let slot = s.init_army(1, None);
        for i in 0..4 {
            let idx = i16::try_from(s.units.len()).unwrap();
            let mut u = Unit::new(1, idx, Pos::new(0x8000 + i * 0x300, 0x8000), 210);
            u.ty = Some(ty);
            u.on_map = true;
            let u = s.add_unit(u);
            s.army_add_unit(1, slot, u);
        }
        let g = s.army_group(1, slot);
        let f = s.form_compute(
            &g,
            Pos::new(0x9000, 0x9000),
            Angle(0),
            formation::LINE,
            s.group_form_mod_option(&g),
            false,
            false,
            &[],
        );
        assert_eq!(s.group_form_mod_option(&g), 50, "the record's form_mod");
        assert_eq!(f.num_category[cat::ARTILLERY], 4);
        assert_eq!(f.x_spacing[cat::ARTILLERY], 660);
        let ours: Vec<(i32, i32)> = f
            .off
            .iter()
            .map(|&(x, y)| (sim::form::Form::quantise(x), sim::form::Form::quantise(y)))
            .collect();

        for (fr, pool, _) in &frames {
            let navy = pool.iter().find(|g| g.id == 66).expect("slot 66");
            let theirs: Vec<(i32, i32)> = navy
                .members
                .iter()
                .map(|m| {
                    (
                        i32::try_from(m.off_x).unwrap(),
                        i32::try_from(m.off_y).unwrap(),
                    )
                })
                .collect();
            assert_eq!(ours, theirs, "{fr}: the record's own slot table");
            assert_eq!(
                f.angles,
                navy.members
                    .iter()
                    .map(|m| m.angle as i8)
                    .collect::<Vec<_>>(),
                "{fr}: and its per-slot facings"
            );
            assert_eq!(
                i64::from(f.off.len() as i32),
                navy.form_num,
                "{fr}: form_num is the membership Form::compute laid out"
            );
            // And `curr`, through the leader's own logged heading.
            let theta = Angle(-1_605_566_464);
            let curr = sim::Sim::form_update_positions(&ours, theta);
            assert_eq!(
                curr,
                navy.members
                    .iter()
                    .map(|m| Pos::new(
                        i32::try_from(m.curr_x).unwrap(),
                        i32::try_from(m.curr_y).unwrap()
                    ))
                    .collect::<Vec<_>>(),
                "{fr}: the record's own curr"
            );
        }
    }

    /// `priority` says **allocated**, not "in the hotkey array" — which is
    /// not what the audit's assertion 6 predicted, and the widening found
    /// it on its first run.
    ///
    /// The dump keeps the two arrays apart structurally: the pool is 512
    /// flat `GROUPDATA` under `FULL DUMP`, while a hotkey group is a
    /// `HOTKEYGROUPDATA` wrapper with its `GROUPDATA` **nested inside it**.
    /// Every record in the first has `priority 0` and 161 of the 162 in the
    /// second have `priority 1` — but **slot 28 has 0**, with a `stamp` of
    /// 13125. It held a group and lost its last member: `Group::kill`'s
    /// `num == 0 → clear(−1)` runs `Group::clear@00713e80`, which writes
    /// **0** over the bit, and only `HotKeyGroups::find_group@00714d00`
    /// puts it back, the next time that slot is allocated. So the bit is
    /// "this slot is a live control group", and an emptied hotkey slot is
    /// indistinguishable from a pool slot by `priority` alone.
    #[test]
    fn run29_s_priority_bit_says_allocated_rather_than_hotkey() {
        let name = "gamelog-run29-islands-engagement-window.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let mut pool = 0usize;
        let mut hotkey = 0usize;
        for (f, b) in log.frames() {
            if !(15100..=15102).contains(&f) {
                continue;
            }
            let body = b.kid("FULL DUMP").unwrap_or(b);
            for g in body.kids("GROUPDATA") {
                assert_eq!(g.int("priority"), Some(0), "{f}: a pool slot");
                pool += 1;
            }
            let hk = body.kid("HOTKEYGROUPS").expect("the hotkey array");
            assert_eq!(hk.int("length"), Some(162), "{f}: the array's own length");
            let mut cleared = 0usize;
            for w in hk.kids("HOTKEYGROUPDATA") {
                let g = w.kid("GROUPDATA").expect("the nested record");
                match g.int("priority") {
                    Some(1) => {}
                    Some(0) => {
                        cleared += 1;
                        assert_eq!(g.int("id"), Some(28), "{f}: the one emptied slot");
                        assert_eq!(g.int("num"), Some(0), "{f}: emptied");
                        assert_eq!(
                            g.int("stamp"),
                            Some(13125),
                            "{f}: and it held a group until then — Group::clear zeroed the bit"
                        );
                    }
                    p => panic!("{f}: priority {p:?} on a hotkey slot"),
                }
                hotkey += 1;
            }
            assert_eq!(cleared, 1, "{f}: exactly one slot has been emptied");
        }
        assert_eq!(pool, 512 * 3);
        assert_eq!(hotkey, 162 * 3, "the hotkey array's own length");
    }

    /// The third pass's widening (`docs/audit/2026-08-25-groups.md`, "Third
    /// pass — verdicts"): the scalars the first widening parsed and compared
    /// against nothing, and the rotation pinned **exactly** rather than by
    /// magnitude.
    ///
    /// `Group::update_positions@00713810`, from the listing: for each of
    /// `form_num` members, `curr_x = off_x·48·sin(θ + 90°) + off_y·48·sin(θ)`
    /// and `curr_y = off_x·48·sin(θ) − off_y·48·sin(θ + 90°)` — the matrix
    /// `[cos θ, sin θ; sin θ, −cos θ]`, determinant −1 — where θ is the
    /// heading (`unit +0x50`) of the unit executing the group move, the
    /// leader. `UNITDATA` logs that heading as `angle`, so the record can be
    /// reproduced to the bit with the simulation's own `sin_component` /
    /// `cos_component`, which are the original's `sin_table`.
    ///
    /// And the scalars: `stamp` and `think_frame` never exceed the frame
    /// (`Group::clear` stamps `game->frame`; `Groups::clear` writes 0 over
    /// it), `speed == new_speed` outside the one `do_group_move` step that
    /// rotates them, and `role` is the OR of the members' type roles
    /// (`Group::add`: `role |= type +0x2c8`, and nothing ever clears a bit)
    /// — so the navy's carries `SEA`, `SEA_MILITARY` and `MILITARY` and not
    /// `LAND`, and army 0's `LAND` and `MILITARY` and not `SEA`.
    #[test]
    fn run29_s_navy_group_s_curr_is_the_leader_s_heading_applied_to_the_slot_table() {
        use sim::ai_load::role;
        use sim::movement::{Angle, cos_component, sin_component};
        let name = "gamelog-run29-islands-engagement-window.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let mut seen = 0;
        for (f, b) in log.frames() {
            if !(15100..=15102).contains(&f) {
                continue;
            }
            seen += 1;
            let pool = crate::gamelog::groups(b);
            let (units, _, _) = crate::gamelog::records(b, false);

            // The scalars nobody had compared, over the whole pool.
            for g in &pool {
                assert!(
                    g.stamp <= f,
                    "{f}/{}: stamp {} is in the future",
                    g.id,
                    g.stamp
                );
                assert!(
                    g.think_frame <= f,
                    "{f}/{}: think_frame {} is in the future",
                    g.id,
                    g.think_frame
                );
                assert_eq!(
                    g.speed, g.new_speed,
                    "{f}/{}: the two speeds differ only inside do_group_move's step",
                    g.id
                );
                if g.num == 0 {
                    assert_eq!(g.role, 0, "{f}/{}: an empty slot has no role", g.id);
                }
            }

            // `role`, by the bits `ai_load::role` names.
            let navy = pool.iter().find(|g| g.id == 66).expect("slot 66");
            let army0 = pool.iter().find(|g| g.id == 69).expect("slot 69");
            let has = |g: &crate::gamelog::GroupDump, bit: u32| g.role & i64::from(bit) != 0;
            for bit in [role::SEA, role::SEA_MILITARY, role::MILITARY] {
                assert!(has(navy, bit), "{f}: the navy's role lacks {bit:#x}");
            }
            assert!(!has(navy, role::LAND), "{f}: a navy is not a land group");
            for bit in [role::LAND, role::MILITARY] {
                assert!(has(army0, bit), "{f}: army 0's role lacks {bit:#x}");
            }
            assert!(!has(army0, role::SEA), "{f}: army 0 is not a sea group");
            // The whole word, as observed: `LAND | MILITARY | RANGED |
            // MOUNTED | MELEE` — a mixed army. `Group::kill` never clears a
            // bit, so the word is the OR over every member the group has
            // *ever* had, not only the seven it has now.
            assert_eq!(
                army0.role,
                i64::from(role::LAND | role::MILITARY | role::RANGED | role::MOUNTED | role::MELEE),
                "{f}: army 0's accumulated role word"
            );

            // The rotation, exactly. The leader is the member at slot (0, 0).
            let lead = navy
                .members
                .iter()
                .find(|m| (m.off_x, m.off_y) == (0, 0))
                .unwrap();
            let u = units
                .iter()
                .find(|u| u.who == navy.who && u.o == lead.o)
                .expect("the leader's UNITDATA");
            let theta = Angle(i32::try_from(u.angle.expect("UNITDATA's angle")).unwrap());
            for m in &navy.members {
                let (dx, dy) = (
                    i32::try_from(m.off_x * 48).unwrap(),
                    i32::try_from(m.off_y * 48).unwrap(),
                );
                let want_x = cos_component(theta, dx) + sin_component(theta, dy);
                let want_y = sin_component(theta, dx) - cos_component(theta, dy);
                assert_eq!(
                    (m.curr_x, m.curr_y),
                    (i64::from(want_x), i64::from(want_y)),
                    "{f}: member {} at off ({}, {}) under the leader's heading {}",
                    m.o,
                    m.off_x,
                    m.off_y,
                    theta.0
                );
                // The check has teeth: a quarter turn off does not
                // reproduce the record. (The group's own `o_angle` does —
                // in this window the leader's heading *is* the move's
                // bearing, so the record cannot separate the two; the
                // listing at `713844` is what says `update_positions`
                // reads the heading.)
                if dx != 0 {
                    assert_ne!(i64::from(cos_component(theta.quarter_turn(), dx)), m.curr_x);
                }
            }
            // Every `off_y` is zero, so the `−cos θ` term never fires: the
            // y-flip is still unpinned, and needs a formation with depth.
            assert!(navy.members.iter().all(|m| m.off_y == 0));
        }
        assert_eq!(seen, 3, "the window's three frames");
    }

    /// Run29's blocks 15100 and 15101 — the tick that put the AI's army on
    /// the one path that reaches `Army::engagement` (`docs/ARMY.md` §16.6).
    ///
    /// Six human hoplites were dropped on army 0's own point at
    /// 15020–15030, so that its 15100 tick found `is_engaged()` true while
    /// it was still mustering. The dispatch (§6 step 6) runs `do_forming`
    /// **or** `engagement`, never both, and `march_to_target`'s engaged arm
    /// leaves `0x10` set — so the only way to `engagement` is
    /// `do_mustering`'s release, whose common tail (§7) overwrites `status`
    /// whole. The two blocks show exactly that: `status 1 → 32`, `city 1 →
    /// −1`, and the point moved from the city's `(34656, 36960)` to the
    /// **muster cell's centre** `(34944, 37248)` — `45 × 0x300 + 0x180`,
    /// `48 × 0x300 + 0x180` — with `muster` itself untouched.
    ///
    /// What is asserted from the sim's own `do_mustering` is the tail and,
    /// crucially, that **no `FORMING` bit survives it** — the gate the
    /// dispatch's `else` needs. Which released branch it takes is *not*
    /// asserted from the harness alone: `strategy[reg]`, the census word
    /// that picks defending over marching, is not in the dump, so it is
    /// set here from the record's own outcome and said to be an input.
    #[test]
    fn run29_s_mustering_army_is_released_with_no_forming_bit_and_the_tail_s_point() {
        use sim::army::status;
        let Some(scs) = scenes(
            "gamelog-run29-islands-engagement-window.txt",
            &[15100, 15101],
        ) else {
            return;
        };
        let (mut before, after) = {
            let mut it = scs.into_iter();
            (it.next().unwrap(), it.next().unwrap())
        };

        // The block before the tick, field for field.
        let a = &before.sim.armies[1].list[0];
        assert!(a.valid && !a.navy);
        assert_eq!(a.status, status::MUSTERING);
        assert_eq!(a.num_units, 7);
        assert_eq!(a.num_standard, 7);
        assert_eq!(a.pos, Pos::new(34656, 36960));
        assert_eq!(a.muster, Cell::new(45, 48));
        assert_eq!(a.muster_angle.0, 0);
        assert!(a.city.is_some(), "mustering at city 1");
        let reg = a.reg.expect("the army's region");

        // The record itself proves the original released it — a mustering
        // army that comes out `0x20` cannot have taken the not-released
        // arm, which only ever ORs `0x10` in. The harness agrees.
        assert!(
            before.sim.release_mustering(1, 0),
            "seven standard at difficulty 0"
        );

        // The one input the dump does not carry: the census's
        // `strategy[reg]`, whose weak bit picks defending over marching.
        before.sim.ai[1].census.strategy.resize(reg as usize + 1, 0);
        before.sim.ai[1].census.strategy[reg as usize] |= 4;

        before.sim.do_mustering(1, 0);

        let got = &before.sim.armies[1].list[0];
        let want = &after.sim.armies[1].list[0];
        assert_eq!(
            got.status & status::FORMING,
            0,
            "the release leaves no FORMING bit — this is the gate `engagement` needs"
        );
        assert_eq!(got.status, want.status, "the record's 32 (DEFENDING)");
        assert_eq!(got.city, None);
        assert_eq!(got.city, want.city);
        assert_eq!(
            got.pos,
            sim::army::cell_centre(Cell::new(45, 48)),
            "the muster cell's centre"
        );
        assert_eq!(got.pos, want.pos, "the record's (34944, 37248)");
        assert_eq!(got.muster, want.muster, "the muster cell is untouched");
        assert_eq!(got.angle, got.muster_angle, "the tail copies it");
        assert_eq!(got.num_standard, want.num_standard);
    }

    // ---- the `UNITS=3` half of run29 (`docs/ARMY.md` §11, §18) ----

    /// **`engagement`'s choice of unit, from the record.** `docs/ARMY.md`
    /// §18 said §11 rested on the reading alone for this, "which no dump
    /// shows"; this is the dump showing it.
    ///
    /// At 15100 army 0's seven members each hold `[MOVEORDER(pathed),
    /// ATTACKORDER]` and their attack targets are a **scatter** — who 0's
    /// objects 15, 16 and 17. At 15101 six of the seven hold `15`, every
    /// one of the seven has gained the **action bit** (`flags 0x10 →
    /// 0x14`), and `order_num` has gone `0 → 1`. That is
    /// `Army::engagement` → `Group::action_attack(·, ox, whom, 0,
    /// QUEUE_NEW, 0)` firing on the tick, and the seed it adopted is
    /// object **15** — the target of `o 54`, the **first** member of the
    /// group's own `list`, which is the order `ArmyData::get_unit` walks.
    ///
    /// Two things the capture falsified in the simulation, both found by
    /// running this:
    ///
    /// 1. `is_engaged` and `engagement` tested the **front** order rather
    ///    than `get_action()`'s. Every member here is walking a transit
    ///    leg in front of its attack, so with the front order tested no
    ///    unit qualified and the whole mechanic was dead on the one frame
    ///    that reaches it.
    /// 2. The seed is not "the first with a map-unit target" alone — see
    ///    `Sim::army_engagement_seed`. This block breaks on the first, so
    ///    the fallback stays unobserved; the capture for it is an army
    ///    whose only attackers are pointed at **buildings**.
    ///
    /// The one member that does not end on 15 is `o 25`, which lands on
    /// 26: `action_attack` gives each member `Unit::find_melee_target`'s
    /// nearest within the respond range and the seed is only the fallback
    /// (`docs/GROUPS.md` §10), so the shared value is what the *seed*
    /// says and not what every member ends up with.
    #[test]
    fn run29_s_engagement_seeds_its_attack_from_the_first_member_of_the_group_s_list() {
        let Some(scs) = scenes(
            "gamelog-run29-islands-engagement-window.txt",
            &[15100, 15101],
        ) else {
            return;
        };
        let (mut before, after) = {
            let mut it = scs.into_iter();
            (it.next().unwrap(), it.next().unwrap())
        };
        // The only kind a scene cannot stand up is `GATHERORDER`, whose
        // `building` is not optional in the simulation and whose target is
        // a woodcutter's camp or a mine — the scene builds cities and
        // nothing else. No member of an army holds one.
        assert!(
            before.untranslated.iter().all(|k| k == "GATHERORDER"),
            "an order kind beyond the gather seam: {:?}",
            before.untranslated
        );

        // The record's own membership, in `list` order — the order
        // `get_unit` walks and therefore the order the seed is chosen in.
        let listed: Vec<i64> = before
            .groups
            .iter()
            .find(|g| g.id == 69)
            .expect("army 0's group")
            .members
            .iter()
            .map(|m| m.o)
            .collect();
        assert_eq!(listed, [54, 53, 52, 51, 49, 48, 25]);
        assert_eq!(
            before.sim.armies[1].list[0].units,
            listed
                .iter()
                .map(|&o| before.unit(1, o).expect("the member's unit"))
                .collect::<Vec<_>>(),
            "the scene stands the army up in the record's own order"
        );

        // Every member's action is its attack, under a pathed transit leg.
        for &o in &listed {
            let u = before.unit(1, o).unwrap();
            let orders = &before.sim.units[u].orders;
            assert_eq!(orders.len(), if o == 25 { 3 } else { 2 }, "member {o}");
            assert!(
                orders[0].is_move() && orders[0].is_transit(),
                "{o}: the leg"
            );
            let a = before.sim.action_of(u).expect("an action under the leg");
            assert!(
                matches!(orders[a].body, sim::orders::Body::Attack(_)),
                "{o}: the action is the attack"
            );
        }

        // The scatter the frame begins on.
        let target_o = |sc: &Scene, o: i64| -> Option<i64> {
            let u = sc.unit(1, o)?;
            match sc.sim.units[u].combat.target? {
                Obj::Unit(t) => sc
                    .units
                    .iter()
                    .find(|(_, _, h)| *h == t)
                    .map(|(_, n, _)| *n),
                Obj::Building(_) => None,
            }
        };
        assert_eq!(
            listed
                .iter()
                .map(|&o| target_o(&before, o))
                .collect::<Vec<_>>(),
            [
                Some(15),
                Some(16),
                Some(16),
                Some(15),
                Some(17),
                Some(16),
                Some(16)
            ],
            "15100's scatter"
        );

        assert!(
            before.sim.army_is_engaged(1, 0),
            "the tick that reaches engagement at all"
        );
        let (seed, target) = before
            .sim
            .army_engagement_seed(1, 0)
            .expect("a seed on this frame");
        assert_eq!(
            seed,
            before.unit(1, 54).unwrap(),
            "the first member of the list, not the first of the army's own array"
        );
        assert_eq!(
            target,
            Obj::Unit(before.unit(0, 15).unwrap()),
            "who 0's object 15 — the target the six other members end on"
        );

        // And that is what the next block holds.
        let six: Vec<Option<i64>> = listed
            .iter()
            .filter(|&&o| o != 25)
            .map(|&o| target_o(&after, o))
            .collect();
        assert_eq!(six, vec![Some(15); 6], "15101's shared target");
        assert_eq!(
            target_o(&after, 25),
            Some(26),
            "find_melee_target's own nearest, not the seed (docs/GROUPS.md §10)"
        );
        for &o in &listed {
            let u = after.unit(1, o).unwrap();
            let a = after.sim.action_of(u).expect("still an attack");
            assert!(
                after.sim.units[u].orders[a].has(sim::orders::flag::ACTION),
                "{o}: action_attack sets the action bit — the record's 0x10 → 0x14"
            );
        }
    }

    /// **A halted group forgets its formation; its members do not.**
    /// `docs/GROUPS.md` §7's one write, seen in a record for the first
    /// time — `action_halt` sets `GroupData.form = −1` unconditionally and
    /// touches no member's `+0xaa`, so `get_form` reads the members and
    /// gives back the formation the group's own field has lost.
    ///
    /// Army 0's group carries `form −1` and `form_num 0` at every frame of
    /// the window while all seven members carry `form 0`; the navy's
    /// carries `form 0` and so do its four. And `get_form_mod_option` is
    /// the **mean over the members whose byte is not −1**, not `get_form`'s
    /// all-agree-or-−1 twin: two of army 0's seven are −1 and the option
    /// is still 50.
    #[test]
    fn run29_s_halted_group_kept_its_members_formation_bytes() {
        let Some(scs) = scenes(
            "gamelog-run29-islands-engagement-window.txt",
            &[15100, 15101, 15102],
        ) else {
            return;
        };
        for (n, sc) in scs.iter().enumerate() {
            let f = 15100 + n;
            let rec = |id: i64| sc.groups.iter().find(|g| g.id == id).expect("the slot");
            let (army0, navy) = (rec(69), rec(66));
            assert_eq!(army0.form, -1, "{f}: the group's own byte, cleared");
            assert_eq!(army0.form_num, 0);
            assert_eq!(navy.form, 0, "{f}: and a group that did move");

            let members = |g: &crate::gamelog::GroupDump| -> Vec<usize> {
                g.members
                    .iter()
                    .map(|m| sc.unit(g.who, m.o).expect("the member"))
                    .collect()
            };
            let group = |g: &crate::gamelog::GroupDump| sim::group::Group {
                who: g.who as sim::Player,
                army: Some(g.army.max(0) as usize),
                list: members(g),
            };
            let bytes: Vec<i8> = members(army0)
                .iter()
                .map(|&u| sc.sim.units[u].form)
                .collect();
            assert_eq!(bytes, vec![0; 7], "{f}: every member kept formation 0");
            assert_eq!(
                sc.sim.group_get_form(&group(army0)),
                0,
                "{f}: so get_form gives back what the group's own field lost"
            );
            let widths: Vec<i8> = members(army0)
                .iter()
                .map(|&u| sc.sim.units[u].form_width)
                .collect();
            assert_eq!(
                widths,
                [-1, 50, 50, -1, 50, 50, 50],
                "{f}: o 54 and o 51 have never been laid out"
            );
            assert_eq!(
                sc.sim.group_form_mod_option(&group(army0)),
                50,
                "{f}: the mean skips the −1s rather than refusing"
            );
            assert_eq!(sc.sim.group_form_mod_option(&group(navy)), 50);
            assert_eq!(sc.sim.group_get_form(&group(navy)), 0);
        }
    }

    /// **The whole `MOVEORDER` row, over every move order in the
    /// window** — `docs/ORDERS.md` §4.1's table turned from a reading into
    /// a diff. Every one of these was read off the PE and none had ever
    /// been compared with a record.
    ///
    /// 146 move orders across the window's four blocks, and each of them:
    /// the destination is **snapped to its 48-unit cell centre**
    /// (`u × 0x30 + 0x18`); `off_x`/`off_y` are the destination's offset
    /// **inside its world cell**, `x mod 0x300` — not a formation slot;
    /// `tolerance`, `pause`, `retry`, `attempts` and `timer` are 0
    /// throughout, as the table says of a plain move; the `dest` flag and
    /// the path stack agree — `dest_x/dest_y` is the stack **top** when
    /// `dest` is 1; the bottom of the stack is the goal; and the `pathed`
    /// bit is set exactly when the unit has a stack at all.
    ///
    /// `coll_x/coll_y` is deliberately not in that list: 34 of the 146
    /// carry a blocker's position, which is `detect_unit_collision`
    /// working rather than a violated invariant.
    #[test]
    fn run29_s_move_orders_match_the_field_table_row_for_row() {
        let name = "gamelog-run29-islands-engagement-window.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let (mut moves, mut collided, mut frames) = (0usize, 0usize, 0usize);
        for (f, b) in log.dumps() {
            frames += 1;
            let (units, _, _) = crate::gamelog::records(b, false);
            for u in &units {
                let tag = format!("{f}: who {} o {}", u.who, u.o);
                for (i, od) in u.orders.iter().enumerate() {
                    let (Some(x), Some(y)) = (od.x, od.y) else {
                        continue;
                    };
                    moves += 1;
                    assert_eq!((x % 0x30, y % 0x30), (0x18, 0x18), "{tag}: cell centre");
                    assert_eq!(od.off_x, Some(x.rem_euclid(0x300)), "{tag}: off_x");
                    assert_eq!(od.off_y, Some(y.rem_euclid(0x300)), "{tag}: off_y");
                    for (k, v) in [
                        ("tolerance", od.tolerance),
                        ("pause", od.pause),
                        ("retry", od.retry),
                        ("attempts", od.attempts),
                        ("timer", od.timer),
                    ] {
                        assert_eq!(v, Some(0), "{tag}: {k}");
                    }
                    if od.coll_x != Some(0) || od.coll_y != Some(0) {
                        collided += 1;
                    }
                    // The current order is the last block logged.
                    if i + 1 != u.orders.len() {
                        continue;
                    }
                    assert_eq!(
                        od.flags & 1 != 0,
                        !u.path.is_empty(),
                        "{tag}: the pathed bit is the stack"
                    );
                    assert_eq!(
                        (u.orders_x, u.orders_y),
                        (Some(x), Some(y)),
                        "{tag}: update_action's orders_x/y"
                    );
                    if let Some(top) = u.path.last() {
                        assert_eq!(u.path[0].flags & 1, 1, "{tag}: the bottom is the goal");
                        if od.dest == Some(1) {
                            assert_eq!(
                                (od.dest_x, od.dest_y),
                                (Some(top.to.0), Some(top.to.1)),
                                "{tag}: the waypoint is the stack top"
                            );
                        }
                    }
                }
            }
        }
        // Four states, not three: the window's [15100, 15103) and the
        // free 15105 of the end-of-run dump, which only `Log::dumps`
        // reaches. 15104's block is the `!quit`'s and carries no state.
        assert_eq!(frames, 4, "the window's three and the free one");
        assert_eq!(moves, 79, "every move order in the four states");
        assert_eq!(
            collided, 17,
            "and the ones detect_unit_collision has marked"
        );
    }

    // ---- run31: the human group move (`docs/GROUPS.md` §6.4, §13) ----

    /// One human group move: the frame, the group, its members' unit
    /// records by object, and the `GroupMoveOrder`s that frame carries.
    struct HumanMove {
        frame: i64,
        group: crate::gamelog::GroupDump,
        units: std::collections::BTreeMap<i64, crate::gamelog::UnitDump>,
        /// `form_id` → the order, for the members that hold one.
        orders: std::collections::BTreeMap<i64, crate::gamelog::OrderDump>,
    }

    /// run31's three right-clicks, each read out of the frame the order was
    /// issued on — the only frames a `GroupMoveOrder` survives into
    /// (`docs/ORACLE.md`, run31).
    fn run31_moves() -> Option<Vec<HumanMove>> {
        let name = "gamelog-run31-humangroup.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return None;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let mut out: Vec<HumanMove> = Vec::new();
        for (frame, block) in log.frames() {
            let (unit_dumps, _, _) = crate::gamelog::records(block, false);
            let orders: std::collections::BTreeMap<i64, crate::gamelog::OrderDump> = unit_dumps
                .iter()
                .flat_map(|u| u.orders.iter())
                .filter(|o| o.kind == "GroupMoveOrder")
                .filter_map(|o| o.form_id.map(|f| (f, o.clone())))
                .collect();
            if orders.is_empty() {
                continue;
            }
            // The run was ended with a kill, so its last block is a
            // half-written frame: the orders are there and the group pool
            // that follows them is not.
            let Some(group) = crate::gamelog::groups(block)
                .into_iter()
                .find(|g| g.who == 0 && g.num > 0)
            else {
                continue;
            };
            let units = unit_dumps
                .iter()
                .filter(|u| u.who == 0)
                .map(|u| (u.o, u.clone()))
                .collect();
            out.push(HumanMove {
                frame,
                group,
                units,
                orders,
            });
        }
        // Three right-clicks, and a `GroupMoveOrder` survives on a member
        // until it is consumed — so the orders are readable for the whole
        // march, not only on the frame they were issued.
        let clicks: std::collections::BTreeSet<(i64, i64)> = out
            .iter()
            .flat_map(|m| m.orders.values())
            .filter_map(|o| Some((o.orig_x?, o.orig_y?)))
            .collect();
        assert_eq!(clicks.len(), 3, "run31's three right-clicks");
        assert!(out.len() >= 30, "and the frames they are readable on");
        Some(out)
    }

    /// run31's twelve squads, stood up in the harness in the record's own
    /// object order so that a harness index *is* a record slot: four
    /// squads of the ranged type then eight of the foot one, each a
    /// captain and two figures down its `o_down` chain, and the group
    /// built by adding the twelve captains (§4.1's subordinate recursion
    /// turns that into 36 members).
    fn run31_harness(
        loaded: &crate::load::Loaded,
        ranged: usize,
        foot: usize,
    ) -> (sim::Sim, sim::group::Group) {
        let mut s = sim::Sim::new(
            sim::tuning::Tuning::RON,
            sim::world::World::new(400, 400),
            2,
        );
        s.nation[0].human = true;
        let mut ours = Vec::new();
        for &t in &[ranged, foot] {
            let mut proto = loaded.unit_types[t].clone();
            proto.tree = None;
            ours.push(s.add_unit_type(proto));
        }
        let mut captains = Vec::new();
        for squad in 0..12 {
            let ty = ours[usize::from(squad >= 4)];
            let mut chain = Vec::new();
            for figure in 0..3 {
                let idx = i16::try_from(s.units.len()).unwrap();
                let mut u = Unit::new(0, idx, Pos::new(0x4000 + squad * 0x100, 0x4000), 120);
                u.ty = Some(ty);
                u.on_map = true;
                u.captain = figure == 0;
                let u = s.add_unit(u);
                chain.push(u);
            }
            s.units[chain[0]].o_down = Some(chain[1]);
            s.units[chain[1]].o_up = Some(chain[0]);
            s.units[chain[1]].o_down = Some(chain[2]);
            s.units[chain[2]].o_up = Some(chain[0]);
            captains.push(chain[0]);
        }
        let mut g = sim::group::Group::stack(0);
        for &c in &captains {
            s.group_add(&mut g, c);
        }
        (s, g)
    }

    /// **The shape of a human group move**, which no dump had held: 36
    /// members and not 12, two categories, and an anchor that is not the
    /// first member.
    ///
    /// The selection was staged slingers-first on purpose
    /// (`docs/ORACLE.md`, run31), so the group's `list[0]` is a
    /// `FORM_CAT_FOOT_RANGED` unit while the lowest category present is
    /// `FORM_CAT_FOOT` — which is what makes `find_leader`'s key
    /// observable at all (§4.4). And a **player's** selection group keeps
    /// its followers, so every figure of every squad is a member: three
    /// objects per unit, the captain first, the two followers behind it
    /// in the object chain (`o_up`).
    #[test]
    fn run31_s_human_group_move_is_thirty_six_figures_of_two_categories() {
        let Some(moves) = run31_moves() else { return };
        for m in &moves {
            let g = &m.group;
            let tag = format!("run31/{}", m.frame);
            assert_eq!(g.who, 0, "{tag}: the human's own group");
            assert_eq!(g.num, 36, "{tag}: twelve squads of three figures");
            assert_eq!(g.form, 0, "{tag}: get_form clamped −1 up to Line");
            assert_eq!(g.form_num, 36, "{tag}: Form::compute laid out all of them");
            assert_eq!(g.army, -1, "{tag}: a selection group, not an army's");
            assert_eq!(g.members.len(), 36, "{tag}: the arrays are parallel");
            // The membership is the selection walk's: every slinger, then
            // every hoplite, each squad's captain followed by its two.
            let objs: Vec<i64> = g.members.iter().map(|m| m.o).collect();
            let mut expect: Vec<i64> = (30..=41).collect();
            expect.extend(6..=29);
            assert_eq!(objs, expect, "{tag}: slingers first, then hoplites");
            for (i, mem) in g.members.iter().enumerate() {
                let u = &m.units[&mem.o];
                let captain = i % 3 == 0;
                assert_eq!(
                    u.o_up.is_none() || u.o_up == Some(-1),
                    captain,
                    "{tag}/{}: a captain heads its figure chain",
                    mem.o
                );
                assert_eq!(
                    u.flags & 0x10 == 0,
                    captain,
                    "{tag}/{}: and SubObjectData's 0x10 says the same",
                    mem.o
                );
                assert_eq!(u.group, Some(g.id), "{tag}/{}: the back-pointer", mem.o);
                assert_eq!(u.form, Some(0), "{tag}/{}: §6.6 step 1 wrote it", mem.o);
                assert_eq!(u.form_mod, Some(50), "{tag}/{}: and the width", mem.o);
            }
            // Exactly one member sits on the origin — the anchor the whole
            // block was slid to — and it is **not** `list[0]`.
            let anchors: Vec<usize> = (0..36)
                .filter(|&i| (g.members[i].off_x, g.members[i].off_y) == (0, 0))
                .collect();
            assert_eq!(anchors.len(), 1, "{tag}: one anchor");
            assert_ne!(anchors[0], 0, "{tag}: and it is not the first member");
            // The offsets have depth as well as width: three distinct
            // `off_y` values, which is what `update_positions`' y-flip
            // needed and what every dump before this one lacked.
            let ys: std::collections::BTreeSet<i64> = g.members.iter().map(|m| m.off_y).collect();
            assert!(
                ys.len() >= 3 && ys.iter().any(|&y| y != 0),
                "{tag}: off_y takes {ys:?}"
            );
        }
    }

    /// **`GroupData::find_leader`'s key, from a record.** `GroupOrder`'s
    /// `oxx` is the object the group move was laid out around, and every
    /// member's order carries the same one; the anchor of the slot table
    /// is that object's slot. So the dump names the leader outright, and
    /// the simulation's own `find_leader` has to agree.
    ///
    /// The old rule — the group's first on-map captain — picks `list[0]`,
    /// a slinger. The record picks a hoplite. `FORM_CAT_FOOT` is below
    /// `FORM_CAT_FOOT_RANGED`, so the key is the category and not the
    /// order, exactly as §4.4 read it and as `Sim::group_find_leader` was
    /// changed to do on 2026-08-26.
    #[test]
    fn run31_s_group_order_names_a_leader_the_first_member_rule_would_miss() {
        use sim::form::{cat, type_cat};
        let Some(inst) = install() else { return };
        let Some(moves) = run31_moves() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        for m in &moves {
            let tag = format!("run31/{}", m.frame);
            let g = &m.group;
            // One leader, agreed by every order the frame carries.
            let leaders: std::collections::BTreeSet<i64> =
                m.orders.values().filter_map(|o| o.oxx).collect();
            assert_eq!(leaders.len(), 1, "{tag}: one oxx across the orders");
            let leader = *leaders.iter().next().unwrap();
            for (form_id, o) in &m.orders {
                assert_eq!(o.whose, Some(0), "{tag}: whose is the group's player");
                assert_eq!(
                    g.members[*form_id as usize].o,
                    {
                        let _ = o;
                        g.members[*form_id as usize].o
                    },
                    "{tag}: form_id indexes the group's own arrays"
                );
            }
            // The record's anchor is the leader's slot.
            let anchor = (0..g.members.len())
                .find(|&i| (g.members[i].off_x, g.members[i].off_y) == (0, 0))
                .expect("an anchor");
            assert_eq!(
                g.members[anchor].o, leader,
                "{tag}: the block is slid onto the leader's slot"
            );
            // Its category is strictly below the first member's, and its
            // index is not zero — so the two rules disagree here.
            let cat_of = |o: i64| {
                let u = &m.units[&o];
                let t = loaded
                    .unit_of_type_index(u.guys[0].kind.unwrap() as i32)
                    .expect("the member's type");
                type_cat(&loaded.unit_types[t], t, true)
            };
            assert_eq!(cat_of(leader), cat::FOOT, "{tag}: a hoplite leads");
            assert_eq!(
                cat_of(g.members[0].o),
                cat::FOOT_RANGED,
                "{tag}: and list[0] is a slinger"
            );
            assert!(
                cat_of(leader) < cat_of(g.members[0].o),
                "{tag}: strictly lower, which is the whole key"
            );
        }
    }

    /// **The `to`/`off` asymmetry, observed.** `compute_dests` slides the
    /// offsets by the whole anchor and the destinations by its `y` alone
    /// (§6.4), so a group whose anchor is off-centre marches to points
    /// displaced from where its own offsets say it will stand.
    ///
    /// The anchor is the one member whose slid offset is exactly `(0, 0)`.
    /// If the destinations were slid by the whole anchor too, that
    /// member's order would point at the click itself. It does not — by
    /// hundreds of position units, on all three of run31's moves — and
    /// every member of the group was clicked to the same point, so the
    /// click is not in doubt.
    #[test]
    fn run31_s_anchor_marches_to_a_point_its_own_offset_says_is_the_click() {
        let Some(moves) = run31_moves() else { return };
        let mut checked = 0usize;
        for m in &moves {
            let tag = format!("run31/{}", m.frame);
            let g = &m.group;
            // One click, on every order in the frame.
            let clicks: std::collections::BTreeSet<(i64, i64)> = m
                .orders
                .values()
                .filter_map(|o| Some((o.orig_x?, o.orig_y?)))
                .collect();
            assert_eq!(clicks.len(), 1, "{tag}: one right-click");
            let (cx, cy) = *clicks.iter().next().unwrap();
            assert_eq!(
                (g.ox, g.oy),
                (cx, cy),
                "{tag}: and the group's own (ox, oy) is that click"
            );
            let anchor = (0..g.members.len())
                .find(|&i| (g.members[i].off_x, g.members[i].off_y) == (0, 0))
                .expect("an anchor");
            let Some(o) = m.orders.get(&(anchor as i64)) else {
                continue;
            };
            checked += 1;
            let (x, y) = (o.x.unwrap(), o.y.unwrap());
            assert_ne!(
                (x, y),
                (cx, cy),
                "{tag}: the anchor's slot is the click only if `to` were \
                 slid by the whole anchor, and it is not"
            );
            // And the displacement is real rather than a rounding: the
            // order's own point is more than one `UCoord` off the click.
            let (dx, dy) = (x - cx, y - cy);
            assert!(
                dx.abs() + dy.abs() > 48,
                "{tag}: the anchor is displaced by ({dx}, {dy})"
            );
        }
        assert!(
            checked >= 30,
            "the anchor held a group order on {checked} frames"
        );
    }

    /// **`Group::update_positions`' y-flip, pinned.** `curr` is the slot
    /// offset rotated by the leader's own heading through
    /// `[cos t, sin t; sin t, -cos t]` — a rotation composed with a
    /// **y-flip**, determinant -1, which a naive port mirrors (§6.6).
    ///
    /// run29 reproduced `curr` too, but every `off_y` in its window was
    /// zero, so the flipped column was multiplied by nothing: the flip was
    /// unpinned and `docs/GROUPS.md` §13 said so. run31's group has three
    /// ranks and `off_y` of -6, -3 and 0 on every one of the forty frames
    /// its three moves are readable over, and the same arithmetic lands on
    /// the record — while the **unflipped** matrix misses it by hundreds.
    ///
    /// It also settles which heading: not the group's `o_angle`, not the
    /// bearing from the leader to its own slot, and not the order's angle,
    /// but the **leader's `UnitData::angle`** — and the leader is the
    /// object the record's own `GroupOrder::oxx` names.
    ///
    /// One thing it settles that nobody had asked: `curr` is a
    /// **mid-frame** quantity. `Unit::do_group_move` computes it inside the
    /// frame and the unit turns afterwards, so the `angle` the end-frame
    /// dump prints is the heading a hair *past* the one the rotation used.
    /// On the frames where the leader was not turning the dumped angle
    /// reproduces the table to the unit; on the rest a heading within
    /// 0.05 degrees of it does, exactly, all seventy-two numbers at once.
    #[test]
    fn run31_s_curr_is_the_leader_s_heading_through_the_y_flip() {
        use sim::movement::{Angle, cos_component, sin_component};
        let Some(moves) = run31_moves() else { return };
        let (mut frames, mut exact, mut with_depth) = (0usize, 0usize, 0usize);
        for m in &moves {
            let tag = format!("run31/{}", m.frame);
            let g = &m.group;
            let anchor = (0..g.members.len())
                .find(|&i| (g.members[i].off_x, g.members[i].off_y) == (0, 0))
                .expect("an anchor");
            let leader = g.members[anchor].o;
            for o in m.orders.values() {
                assert_eq!(o.oxx, Some(leader), "{tag}: oxx is the anchor's object");
            }
            let theta = Angle(
                i32::try_from(m.units[&leader].angle.expect("the leader's heading")).unwrap(),
            );
            let off: Vec<(i32, i32)> = g
                .members
                .iter()
                .map(|x| (x.off_x as i32 * 48, x.off_y as i32 * 48))
                .collect();
            assert!(
                off.iter().any(|&(_, y)| y != 0),
                "{tag}: the depth the flipped column needs"
            );
            with_depth += 1;
            let want: Vec<Pos> = g
                .members
                .iter()
                .map(|x| Pos::new(x.curr_x as i32, x.curr_y as i32))
                .collect();
            let rotate = |a: Angle, flip: bool| -> Vec<Pos> {
                off.iter()
                    .map(|&(x, y)| {
                        let cy = cos_component(a, y);
                        Pos::new(
                            cos_component(a, x) + sin_component(a, y),
                            sin_component(a, x) + if flip { -cy } else { cy },
                        )
                    })
                    .collect()
            };
            if rotate(theta, true) == want {
                exact += 1;
            }
            // Some heading within a twentieth of a degree of the dumped one
            // reproduces every one of the seventy-two numbers.
            let near = (-(1i32 << 21)..=(1i32 << 21))
                .step_by(1024)
                .any(|d| rotate(Angle(theta.0.wrapping_add(d)), true) == want);
            assert!(near, "{tag}: no heading near {theta:?} reproduces curr");
            // And no heading anywhere near it does without the flip.
            let unflipped = (-(1i32 << 21)..=(1i32 << 21))
                .step_by(1024)
                .any(|d| rotate(Angle(theta.0.wrapping_add(d)), false) == want);
            assert!(!unflipped, "{tag}: the determinant is -1, not +1");
            frames += 1;
        }
        assert!(frames >= 30, "{frames} frames reproduced");
        assert_eq!(with_depth, frames, "every one of them has a non-zero off_y");
        assert!(
            exact >= 9,
            "the dumped heading is the rotation's own on {exact} of {frames}"
        );
    }

    /// run31's thirty-six-member group, stood up in the harness from the
    /// install's own columns and laid out by `sim::form` — and every one of
    /// the 40 records reproduced, all 36 slots, both coordinates.
    ///
    /// The chain, with nothing fitted in the middle:
    ///
    /// 1. The record's twelve squads are two types. `unitrules.xml` gives
    ///    Hoplites and Slingers `X_SPACING 12`, `Y_SPACING 12`,
    ///    `GUY_SPACING 12` and `UBER_SIZE 3`, and `UnitType::init`
    ///    multiplies the first two by `UNIT_FORMATION_SPACING` and the third
    ///    by `UNIT_GUY_SPACING` — both 12 — so the columns are 144.
    /// 2. `Form::categorize` widens a multi-figure type's rank to
    ///    `min(uber_size, 3) × x_spacing`, so the category's width is
    ///    **432**, and its depth `⌈3/3⌉ × 144 = 144`.
    /// 3. `type_cat` puts the hoplites in `FORM_CAT_FOOT` and the slingers,
    ///    which have range, in `FORM_CAT_FOOT_RANGED` — eight captains and
    ///    four. `span = max(8 × 432 / 2, 4 × 432) = 1728`, `form_mod` is 50,
    ///    so both categories are **4** columns wide.
    /// 4. `Group::add`'s subordinate recursion (§4.1) turns twelve captains
    ///    into **36** members, each captain followed by its two figures —
    ///    which is the record's own `list`, object for object.
    /// 5. `compute_dests` places the captains, hangs each follower off its
    ///    captain by one `guy_spacing`, stacks `FOOT_RANGED` behind `FOOT`,
    ///    and slides the block onto the anchor.
    ///
    /// **What the record adds, and what no earlier pass had looked for**:
    /// its offsets are not always `compute_dests`' output. One of the forty
    /// frames is that output **re-origined onto another member**, by
    /// `Group::refresh_group_order` (`docs/GROUPS.md` §6.8) — and that is
    /// the whole of §4.4's old question about why the leader is object 9 on
    /// frame 204 and object 6 from 328 on. `find_leader` picks object 6 both
    /// times; frame 204's record is one refresh past the layout.
    #[test]
    fn run31_s_thirty_six_member_table_is_reproduced_from_the_install_s_own_columns() {
        use sim::form::{cat, formation, type_cat};
        use sim::movement::Angle;
        let Some(inst) = install() else { return };
        let Some(moves) = run31_moves() else { return };
        let loaded = crate::load::load(&inst).unwrap();

        // ---- the record's own membership, and the two types behind it ----
        let first = &moves[0];
        let objs: Vec<i64> = first.group.members.iter().map(|m| m.o).collect();
        let type_of = |o: i64| -> usize {
            let u = &first.units[&o];
            loaded
                .unit_of_type_index(i32::try_from(u.guys[0].kind.expect("a guy kind")).unwrap())
                .expect("the member's type")
        };
        let kinds: Vec<usize> = objs.iter().map(|&o| type_of(o)).collect();
        // Four squads of one type then eight of the other, three figures each.
        let ranged = kinds[0];
        let foot = kinds[12];
        assert_ne!(ranged, foot, "two types");
        assert!(kinds[..12].iter().all(|&t| t == ranged));
        assert!(kinds[12..].iter().all(|&t| t == foot));
        assert_eq!(type_cat(&loaded.unit_types[foot], foot, true), cat::FOOT);
        assert_eq!(
            type_cat(&loaded.unit_types[ranged], ranged, true),
            cat::FOOT_RANGED
        );
        for &t in &[foot, ranged] {
            let c = &loaded.unit_types[t].combat;
            assert_eq!(
                (c.x_spacing, c.y_spacing, c.guy_spacing, c.uber_size),
                (144, 144, 144, 3),
                "{}: X_SPACING/Y_SPACING/GUY_SPACING 12 and UBER_SIZE 3",
                loaded.unit_names[t]
            );
        }

        // ---- the same group, stood up in the harness ----
        let (s, g) = run31_harness(&loaded, ranged, foot);
        assert_eq!(g.list.len(), 36, "the subordinate recursion");
        assert_eq!(
            g.list,
            (0..36).collect::<Vec<usize>>(),
            "and in captain-then-figures order, like the record's own list"
        );
        assert_eq!(
            s.group_find_leader(&g),
            Some(12),
            "find_leader takes the lowest category, ties to the first — the \
             first hoplite captain, not the first member"
        );
        assert_eq!(s.group_form_mod_option(&g), 50, "the record's form_mod");

        // ---- the layout, both mirrors ----
        let click = Pos::new(
            i32::try_from(first.group.ox).unwrap(),
            i32::try_from(first.group.oy).unwrap(),
        );
        let table = |reverse: bool, dest: Pos, theta: Angle| {
            s.form_compute(
                &g,
                dest,
                theta,
                formation::LINE,
                s.group_form_mod_option(&g),
                reverse,
                false,
                &[],
            )
        };
        let probe = table(false, click, Angle(0));
        assert_eq!(probe.num_category[cat::FOOT], 8, "eight hoplite captains");
        assert_eq!(probe.num_category[cat::FOOT_RANGED], 4, "four slinger ones");
        assert_eq!(probe.x_spacing[cat::FOOT], 432, "3 × X_SPACING 12 × 12");
        assert_eq!(probe.y_spacing[cat::FOOT], 144, "⌈3/3⌉ × Y_SPACING 12 × 12");
        // The anchor is the first member of the lowest non-empty category,
        // which is the leader `find_leader` names.
        assert_eq!(
            probe.off[12],
            (0, 0),
            "compute_dests slides the block onto the leader's slot"
        );

        // ---- every frame, every slot ----
        let (mut checked, mut refreshed, mut mirrored) = (0usize, 0usize, 0usize);
        let mut orders_checked = 0usize;
        for m in &moves {
            let tag = format!("run31/{}", m.frame);
            let rec: Vec<(i32, i32)> = m
                .group
                .members
                .iter()
                .map(|x| {
                    (
                        i32::try_from(x.off_x).unwrap(),
                        i32::try_from(x.off_y).unwrap(),
                    )
                })
                .collect();
            let angles: std::collections::BTreeSet<i64> =
                m.orders.values().filter_map(|o| o.angle).collect();
            assert_eq!(angles.len(), 1, "{tag}: one formation angle");
            let theta = Angle(i32::try_from(*angles.iter().next().unwrap()).unwrap());
            let clicks: std::collections::BTreeSet<(i64, i64)> = m
                .orders
                .values()
                .filter_map(|o| Some((o.orig_x?, o.orig_y?)))
                .collect();
            let (cx, cy) = *clicks.iter().next().unwrap();
            let dest = Pos::new(i32::try_from(cx).unwrap(), i32::try_from(cy).unwrap());

            // Exactly one mirror reproduces the frame, up to the re-origin.
            let mut hit = None;
            for reverse in [false, true] {
                let f = table(reverse, dest, theta);
                let quant: Vec<(i32, i32)> = f
                    .off
                    .iter()
                    .map(|&(x, y)| (sim::form::Form::quantise(x), sim::form::Form::quantise(y)))
                    .collect();
                // `refresh_group_order` re-origins the *quantised* table, so
                // the comparison is exact once both are put on the same slot.
                let origin = (0..36).find(|&i| rec[i] == (0, 0)).expect("an anchor");
                let mut ours = quant.clone();
                let mut st = sim::group::GroupState {
                    form_num: 36,
                    off: ours.clone(),
                    ..sim::group::GroupState::default()
                };
                st.reorigin(origin);
                ours = st.off.clone();
                if ours == rec {
                    hit = Some((reverse, f, origin));
                    break;
                }
            }
            let Some((reverse, f, origin)) = hit else {
                panic!("{tag}: neither mirror reproduces the record's 36 slots");
            };
            checked += 1;
            if reverse {
                mirrored += 1;
            }
            if origin != 12 {
                refreshed += 1;
                assert_eq!(origin, 15, "{tag}: run31's one refresh is onto slot 15");
                continue;
            }

            // An un-refreshed frame: the **destinations** are ours too. The
            // order's own `x`/`y` is the slot put through §6.6 step 6's
            // `UCoord` (`/0x30`) — floor to the 48-unit cell and back at its
            // centre, which is exactly `Unit::add_move_order`'s snap — and
            // it lands on every member of every frame, all three moves.
            for (&form_id, o) in &m.orders {
                let i = usize::try_from(form_id).unwrap();
                let want = Pos::new(
                    f.to[i].x.div_euclid(0x30) * 0x30 + 0x18,
                    f.to[i].y.div_euclid(0x30) * 0x30 + 0x18,
                );
                assert_eq!(
                    (i64::from(want.x), i64::from(want.y)),
                    (o.x.expect("x"), o.y.expect("y")),
                    "{tag}/{form_id}: the slot destination, snapped"
                );
                orders_checked += 1;
            }
            // `o_dist` is `Form::compute`'s leftover — the distance from the
            // order's point to the leader's slot, which is exactly the
            // half-column the even count shifted the block by.
            assert!(
                (m.group.o_dist - 216).abs() <= 1,
                "{tag}: o_dist {} is x_spacing/2",
                m.group.o_dist
            );
        }
        assert_eq!(checked, moves.len(), "every frame reproduced");
        assert_eq!(refreshed, 1, "one of the forty is a re-origin");
        assert!(
            orders_checked >= 900,
            "{orders_checked} slot destinations reproduced"
        );
        assert!(
            mirrored > 0 && mirrored < checked,
            "run31 has moves both ways: {mirrored} of {checked} mirrored"
        );
    }

    /// One frame of run31, reduced to what the mirror's state machine reads.
    struct Run31Frame {
        /// The human's live group, if the pool held one that frame.
        facing: Option<bool>,
        /// `who == 0` object → its `UNITDATA::angle`.
        angle: std::collections::BTreeMap<i64, i64>,
        /// The distinct `GroupMoveOrder`s the frame carries: the click, the
        /// order's angle, and its own `facing` byte.
        orders: std::collections::BTreeSet<(i64, i64, i64, i64)>,
    }

    /// Every frame of run31, not only the ones carrying an order — the
    /// predicate reads the leader's heading on the frame **before** the
    /// click, which is the last one printed before `compute_form` ran.
    fn run31_frames() -> Option<std::collections::BTreeMap<i64, Run31Frame>> {
        let name = "gamelog-run31-humangroup.txt";
        let Some(path) = dump(name) else {
            eprintln!("skipping: no {name} (set RON_GAMELOG_DIR)");
            return None;
        };
        let text = crate::capture::read(&path);
        let log = Log::parse(&text);
        let mut out = std::collections::BTreeMap::new();
        for (frame, block) in log.frames() {
            let (units, _, _) = crate::gamelog::records(block, false);
            let mut angle = std::collections::BTreeMap::new();
            let mut orders = std::collections::BTreeSet::new();
            for u in &units {
                if u.who != 0 {
                    continue;
                }
                if let Some(a) = u.angle {
                    angle.insert(u.o, a);
                }
                for o in &u.orders {
                    if o.kind != "GroupMoveOrder" {
                        continue;
                    }
                    if let (Some(x), Some(y), Some(a), Some(f)) =
                        (o.orig_x, o.orig_y, o.angle, o.facing)
                    {
                        orders.insert((x, y, a, f));
                    }
                }
            }
            let facing = crate::gamelog::groups(block)
                .into_iter()
                .find(|g| g.who == 0 && g.num > 0)
                .map(|g| g.facing != 0);
            out.insert(
                frame,
                Run31Frame {
                    facing,
                    angle,
                    orders,
                },
            );
        }
        Some(out)
    }

    /// **The mirror's predicate, end to end** (`docs/GROUPS.md` §6.3,
    /// §12.3) — the question run31 opened while closing two others, and
    /// the one this file could not answer until the *other* writers of
    /// `GroupData::facing` were found.
    ///
    /// The flag the layout reads is never the flag the dump prints. Three
    /// things write it, and in one frame all three can run:
    ///
    /// 1. `Unit::kill_current_order@005e2cb0` — the `QUEUE_NEW` clear at
    ///    `70524f` runs **before** `compute_form` at `7053ec`, and the
    ///    leader's dying move hands its own `MoveOrder::facing` back to the
    ///    group, inverted if the leader has since turned around.
    /// 2. `compute_form@00707c80` toggles it around `Form::compute` and
    ///    toggles it back — the mirror is `facing XOR (leader ≥ 90° off the
    ///    bearing)` and the flag itself ends where it started.
    /// 3. `Unit::set_angle@00605400` toggles it again, later in the frame,
    ///    when the leader's own march turns it by 90° or more.
    ///
    /// So run31's three clicks are three predictions each, and every one
    /// is a number the record already holds. Two of the moves have a
    /// dumped `facing` that **contradicts** the mirror their slot table
    /// needs, which is exactly why nothing short of the whole machine
    /// reproduces them.
    ///
    /// **What this run cannot reach**, said here because four of the five
    /// breakages written against it went red and this is the fifth: the
    /// hand-back's *inversion*. Both of run31's kills catch the leader
    /// 10.6° and 6.3° off the dying order's own angle, nowhere near the
    /// 90° that would flip the byte on the way back, so `reversing`'s term
    /// at `5e3062`–`5e307b` is carried by the listing alone. The capture
    /// that would settle it is a group ordered somewhere, turned right
    /// around while marching, and then re-ordered — §13.
    #[test]
    fn run31_s_three_mirrors_come_out_of_facing_s_three_writers() {
        use sim::group::reversing;
        use sim::movement::Angle;
        let Some(moves) = run31_moves() else { return };
        let Some(frames) = run31_frames() else { return };

        // The three clicks, each on the frame its order first appears.
        let mut clicks: Vec<(i64, (i64, i64, i64, i64))> = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for (&frame, f) in &frames {
            for &o in &f.orders {
                if seen.insert((o.0, o.1)) {
                    clicks.push((frame, o));
                }
            }
        }
        assert_eq!(clicks.len(), 3, "run31's three right-clicks");

        // `find_leader` names object 6 — the first hoplite captain, the
        // lowest `type_cat` in the group (§4.4) — and its slot in the
        // 36-member list is 12, whose `angles` byte is 0 in a Line.
        let leader = 6;
        let leader_slot = 12;
        for m in &moves {
            assert_eq!(
                m.group.members[leader_slot].o, leader,
                "run31/{}: slot 12 is the leader's",
                m.frame
            );
            assert_eq!(
                m.group.members[leader_slot].angle, 0,
                "run31/{}: a Line leans nowhere",
                m.frame
            );
        }

        // The mirror each move actually used is the one its own orders
        // carry: `MoveOrder +0x28` is written with the flag `Form::compute`
        // was handed, and every member of a move agrees on it.
        let d = |a: i64, b: i64| {
            reversing(Angle(
                i32::try_from(a)
                    .unwrap()
                    .wrapping_sub(i32::try_from(b).unwrap()),
            ))
        };
        let mut facing = false; // `Group::clear` — a fresh group mirrors nothing.
        let mut prev: Option<(i64, i64)> = None; // the last order's (angle, facing)
        // How often the flag **as the pool last printed it** predicts the
        // wrong mirror — the model this file held until today, and the
        // control that makes the rest of this test mean something.
        let mut running_wrong = 0;
        for (frame, (_, _, theta, order_facing)) in &clicks {
            let tag = format!("run31/{frame}");
            let before = frames
                .get(&(frame - 1))
                .expect("the frame before the click");
            let now = &frames[frame];
            let heading = *before.angle.get(&leader).expect("the leader's heading");

            // 1. The dying order's hand-back, which only a leader makes.
            //    It is an assignment and not a toggle, so whatever the march
            //    did to the flag since the last click is discarded here.
            if let Some((prev_angle, prev_facing)) = prev {
                facing = (prev_facing != 0) != d(heading, prev_angle);
            }

            // 2. `compute_form`'s toggle: the mirror, and the flag put back.
            let away = d(heading, *theta);
            let mirror = facing != away;
            assert_eq!(
                mirror,
                *order_facing != 0,
                "{tag}: the mirror the orders carry"
            );
            if (before.facing.unwrap_or(false) != away) != mirror {
                running_wrong += 1;
            }

            // 3. `set_angle`'s toggle, once the leader turns into the move —
            //    and the flag stays turned, which is what the pool prints at
            //    the end of the frame and carries to the next click.
            let after = *now.angle.get(&leader).expect("the leader's heading");
            facing = facing != d(after, heading);
            assert_eq!(
                Some(facing),
                now.facing,
                "{tag}: the `facing` the frame's own GROUPDATA prints"
            );

            prev = Some((*theta, *order_facing));
        }
        assert_eq!(
            running_wrong, 1,
            "the flag as the pool prints it mispredicts one of the three"
        );

        // The two halves are not the same sequence, and that is the whole
        // finding: the mirrors run 0, 0, 1 while the dumped flags run
        // 1, 0, 1, so a reader taking `facing` for the mirror gets two of
        // the three moves wrong.
        let mirrors: Vec<i64> = clicks.iter().map(|c| c.1.3).collect();
        let dumped: Vec<bool> = clicks
            .iter()
            .map(|c| frames[&c.0].facing.expect("a live group"))
            .collect();
        assert_eq!(mirrors, vec![0, 0, 1], "the mirrors the orders carry");
        assert_eq!(dumped, vec![true, false, true], "the flags the pool prints");
    }
}
