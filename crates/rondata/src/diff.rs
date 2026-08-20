//! The gamelog diff: the original's per-frame dump against the simulation.
//!
//! Phase 3's score is ticks before divergence. This is the harness that
//! produces it: [`build_sim`] stands a `sim::Sim` up from a start-of-game
//! dump ([`crate::gamelog::Initial`]) with the loaded types
//! ([`crate::load::Loaded`]), and [`run`] steps it frame by frame against the
//! dump's `FRAME n` blocks, comparing what both sides hold.
//!
//! # What it can see today, and what it cannot
//!
//! At detail level 0 the original writes, per unit per frame, the object base
//! — owner, object number, `x_internal`/`y_internal`/`z_internal` — and per
//! leader its `score` and flags. So the comparison is **positions of units
//! keyed by `(who, o)`**, and the score is reported but not matched (the
//! simulation has no score yet). The per-frame `GUY` blocks are empty, the
//! leaders carry no goods, and buildings are not written per frame under the
//! categories the logged runs enabled; `DUMP_ALL=1` is the untried lever for
//! more (`docs/ORACLE.md`).
//!
//! And the simulation has no AI and sees no orders: in the logged runs the
//! human's units stand still and the AI's move as soon as it decides to, so
//! the honest expectation is that player 0 matches for as long as the human
//! did nothing and the AI player diverges the frame its first order lands.
//! That is still worth running — it proves the ids, the types, the world
//! scale and the step cadence agree end to end — and it is the scaffold the
//! order stream plugs into when the recorded-game container is read.

use crate::gamelog::{Frame, Initial, Log, Pos as LogPos};
use crate::load::Loaded;
use sim::{Pos, Sim, Tuning, Unit, World};

/// What a `(who, o)` unit in the log is in the simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitLink {
    pub who: i64,
    pub o: i64,
    /// The simulation's unit index.
    pub unit: usize,
    /// The unit record, if the `GUY` type named one.
    pub kind: Option<usize>,
}

/// The simulation plus the maps back into the log's ids.
#[derive(Debug)]
pub struct Built {
    pub sim: Sim,
    pub units: Vec<UnitLink>,
    /// Anything that could not be carried over, and why.
    pub notes: Vec<String>,
}

/// How many player slots the log's leaders occupy: the `who`s below eight.
/// The original keeps eight slots and uses 8 and 9 for the two non-player
/// leaders (`LEADERDATA who 8`, `who 9` in every dump).
pub fn player_count(init: &Initial) -> usize {
    init.leaders
        .iter()
        .filter(|l| (0..8).contains(&l.who))
        .map(|l| l.who as usize + 1)
        .max()
        .unwrap_or(0)
}

fn pos_of(p: LogPos) -> Pos {
    Pos::new(p.x as i32, p.y as i32)
}

/// Stands a simulation up from the initial dump.
///
/// The world is `xs × ys` cells from `WORLD`; every unit the dump lists for
/// a player slot becomes one simulation unit at its object position, typed
/// by its first `GUY`'s `type` (a `TypeIndex`; `0x32` is the Citizen), with
/// its squad size the number of `GUY`s and its health the type's `HITS`.
/// Each player's tribe comes from `LEADERDATA`. The city each player starts
/// with is placed by type at the `CITIES` position when the placement rules
/// allow it, and as an untyped building otherwise — with a note.
pub fn build_sim(loaded: &Loaded, init: &Initial, tuning: Tuning) -> Built {
    let mut notes = Vec::new();
    let get = |k: &str| -> Option<i64> {
        init.world
            .iter()
            .find(|(key, _)| *key == k)
            .and_then(|(_, v)| v.trim().parse().ok())
    };
    let xs = get("xs").unwrap_or(60).max(1) as i32;
    let ys = get("ys").unwrap_or(60).max(1) as i32;
    let players = player_count(init).max(1);
    let mut sim = loaded.sim(tuning, World::new(xs, ys), players);

    for l in &init.leaders {
        if (0..players as i64).contains(&l.who) {
            let who = l.who as usize;
            sim.tech[who].tribe = l.tribe.max(0) as usize;
            sim.tech[who].power = Some(l.tribe.max(0) as usize);
        }
    }

    // The starting city, by type at the logged position.
    let city_ty = loaded.build_named("Small City");
    for c in &init.cities {
        if !(0..players as i64).contains(&c.who) {
            continue;
        }
        let who = c.who as sim::Player;
        let pos = Pos::new(c.x as i32, c.y as i32);
        let placed = match city_ty {
            Some(ty) => sim
                .place_building(who, ty, pos)
                .map_err(|e| format!("{e:?}")),
            None => Err("no Small City type".to_string()),
        };
        if let Err(why) = placed {
            let b = sim.add_building(who, pos, 8);
            sim.buildings[b].ty = city_ty;
            notes.push(format!(
                "player {who}: the city at {} {} was added untyped — place_building refused it ({why})",
                c.x, c.y
            ));
        }
    }

    // The units.
    let mut units = Vec::new();
    for u in &init.units {
        if !(0..players as i64).contains(&u.who) {
            continue;
        }
        let kind = u
            .guys
            .first()
            .and_then(|g| g.kind)
            .and_then(|t| loaded.unit_of_type_index(t as i32));
        if u.guys.first().is_some_and(|g| g.kind.is_some()) && kind.is_none() {
            notes.push(format!(
                "unit who {} o {}: GUY type {:?} is not a unit TypeIndex",
                u.who, u.o, u.guys[0].kind
            ));
        }
        let ty = kind.map(|k| &loaded.unit_types[k]);
        let health = ty.map_or(1, |t| t.hits.max(1));
        let mut unit = Unit::new(u.who as sim::Player, u.o as i16, pos_of(u.pos), health);
        unit.squad_size = u.guys.len().max(1) as i32;
        unit.ty = kind;
        if let Some(t) = ty {
            unit.kind = t.kind;
            unit.movement.speed = t.moves;
            unit.movement.turning.type_turn_speed = t.turn_speed;
        }
        let idx = sim.add_unit(unit);
        units.push(UnitLink {
            who: u.who,
            o: u.o,
            unit: idx,
            kind,
        });
    }

    Built { sim, units, notes }
}

/// One unit whose position the two sides disagree on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Divergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub ours: Pos,
    pub theirs: Pos,
}

/// The outcome of one frame's comparison.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameResult {
    pub frame: i64,
    /// Units the log has for a player and the simulation does not.
    pub unlinked: usize,
    pub compared: usize,
    pub diverged: Vec<Divergence>,
    /// The leaders' scores as logged, by `who`.
    pub scores: Vec<(i64, i64)>,
}

/// The whole run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub frames: Vec<FrameResult>,
    /// Per player slot: the first frame a unit of theirs diverged, if any.
    pub first_divergence: Vec<(i64, Option<i64>)>,
    pub notes: Vec<String>,
}

impl Report {
    /// Ticks before divergence: the last frame on which every compared unit
    /// agreed, or the number of frames if none ever disagreed.
    pub fn ticks_before_divergence(&self) -> i64 {
        self.frames
            .iter()
            .find(|f| !f.diverged.is_empty())
            .map_or(self.frames.len() as i64, |f| f.frame - 1)
    }
}

/// Compares one logged frame against the simulation as it stands.
pub fn compare(built: &Built, frame: &Frame, players: usize) -> FrameResult {
    let mut r = FrameResult {
        frame: frame.n,
        ..FrameResult::default()
    };
    for u in &frame.units {
        if !(0..players as i64).contains(&u.who) {
            continue;
        }
        let Some(link) = built.units.iter().find(|l| l.who == u.who && l.o == u.o) else {
            r.unlinked += 1;
            continue;
        };
        let ours = built.sim.units[link.unit].pos;
        let theirs = pos_of(u.pos);
        r.compared += 1;
        if ours != theirs {
            r.diverged.push(Divergence {
                frame: frame.n,
                who: u.who,
                o: u.o,
                ours,
                theirs,
            });
        }
    }
    r.scores = frame.leaders.iter().map(|l| (l.who, l.score)).collect();
    r
}

/// Builds the simulation from the log's initial state and steps it through
/// every logged frame, comparing as it goes. `limit` caps the frames.
pub fn run(loaded: &Loaded, log: &Log, tuning: Tuning, limit: Option<usize>) -> Option<Report> {
    let init = log.initial()?;
    let players = player_count(&init);
    let mut built = build_sim(loaded, &init, tuning);
    let mut report = Report {
        notes: std::mem::take(&mut built.notes),
        ..Report::default()
    };
    let frames = log.frame_states();
    let mut last = 0i64;
    for f in frames.iter().take(limit.unwrap_or(usize::MAX)) {
        // `FRAME n` is the state at the end of frame n; step up to it.
        while last < f.n {
            built.sim.tick();
            last += 1;
        }
        report.frames.push(compare(&built, f, players));
    }
    report.first_divergence = (0..players as i64)
        .map(|who| {
            let first = report
                .frames
                .iter()
                .find(|f| f.diverged.iter().any(|d| d.who == who))
                .map(|f| f.frame);
            (who, first)
        })
        .collect();
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gamelog::{CityDump, Guy, LeaderDump, UnitDump};

    fn install() -> Option<crate::Install> {
        let root = std::env::var("RON_INSTALL").ok().or_else(|| {
            let here = env!("CARGO_MANIFEST_DIR");
            Some(format!("{here}/../../game"))
        })?;
        let i = crate::Install::new(root);
        i.looks_valid().then_some(i)
    }

    fn initial() -> Initial<'static> {
        Initial {
            world: vec![("xs", "60"), ("ys", "60"), ("seed", "7236")],
            cities: vec![CityDump {
                x: 3168,
                y: 30816,
                pop: 1,
                who: 0,
            }],
            units: vec![
                UnitDump {
                    flags: 65,
                    o: 0,
                    who: 0,
                    pos: LogPos {
                        x: 4248,
                        y: 32664,
                        z: 528,
                    },
                    guys: vec![
                        Guy {
                            kind: Some(0x32 + 19),
                            ..Guy::default()
                        },
                        Guy {
                            kind: Some(0x32 + 19),
                            ..Guy::default()
                        },
                    ],
                },
                UnitDump {
                    flags: 1,
                    o: 1,
                    who: 0,
                    pos: LogPos {
                        x: 4248,
                        y: 28680,
                        z: 496,
                    },
                    guys: vec![Guy {
                        kind: Some(0x32),
                        ..Guy::default()
                    }],
                },
                // An animal, which the harness ignores.
                UnitDump {
                    flags: 1,
                    o: 3,
                    who: 255,
                    pos: LogPos::default(),
                    guys: vec![],
                },
            ],
            leaders: vec![
                LeaderDump {
                    who: 0,
                    tribe: 11,
                    ..LeaderDump::default()
                },
                LeaderDump {
                    who: 8,
                    ..LeaderDump::default()
                },
            ],
            ..Initial::default()
        }
    }

    #[test]
    fn player_count_is_the_highest_player_slot_plus_one() {
        assert_eq!(player_count(&initial()), 1);
        let mut i = initial();
        i.leaders.push(LeaderDump {
            who: 1,
            ..LeaderDump::default()
        });
        assert_eq!(player_count(&i), 2);
    }

    #[test]
    fn the_simulation_is_built_from_the_dump_and_idle_units_hold() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let init = initial();
        let built = build_sim(&loaded, &init, Tuning::RON);
        assert_eq!(built.units.len(), 2);
        assert_eq!(built.units[0].kind, Some(19)); // the Scout
        assert_eq!(built.units[1].kind, Some(0)); // the Citizen
        assert_eq!(built.sim.units[built.units[1].unit].health, 40);
        assert_eq!(built.sim.units[built.units[0].unit].squad_size, 2);
        assert_eq!(built.sim.tech[0].tribe, 11);
        assert_eq!(built.sim.units[built.units[1].unit].movement.speed, 25);
        // Idle units stand still, so a frame that repeats the positions agrees.
        let frame = Frame {
            n: 1,
            units: init.units.clone(),
            ..Frame::default()
        };
        let mut built = built;
        built.sim.tick();
        let r = compare(&built, &frame, 1);
        assert_eq!(r.compared, 2);
        assert!(r.diverged.is_empty());
        // And a moved one is reported.
        let mut moved = frame.clone();
        moved.units[1].pos.x += 10;
        let r = compare(&built, &moved, 1);
        assert_eq!(r.diverged.len(), 1);
        assert_eq!(r.diverged[0].o, 1);
    }
}
