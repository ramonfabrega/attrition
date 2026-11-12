//! **The free runner** (`docs/RENDERER.md` §2.4, item 4020): a game stepped
//! from its start with nothing installed and nothing compared, handing a
//! client one [`Scene`] a tick.
//!
//! The diff harness (`diff::run_traced_observed`) steps the same simulation,
//! but it reads every frame of the dump to compare against and installs the
//! original's sync word and figure clocks wherever a capture traced them. A
//! client has no dump. So this runner:
//!
//! - builds the `Sim` with [`crate::diff::build_sim`] from the **setup
//!   blocks** of the captures alone — the start dump, its siblings, and the
//!   pasture's animals from the trace — and never reads a frame;
//! - each tick applies the order stream, when there is one
//!   ([`crate::input::Stream::apply`]), and calls `Sim::tick` through the
//!   order door (`sim::command`);
//! - reads the state through the read door (`sim::observe`) and nothing
//!   else.
//!
//! The setup door is reached only inside `build_sim`. A runner that read
//! `Sim`'s fields would have rebuilt the coupling item 4017's doors removed,
//! so the frame count is the runner's own.
//!
//! On a pair the game is uncorrected from frame 1 (`docs/DECISIONS.md` entry
//! 100 §4), so every scene this runner hands out must equal the harness's on
//! the same frame. `tests::the_free_run_s_scenes_are_the_harness_s` holds
//! that on every tick of a whole game.

use crate::capture::indexed::IndexedCapture;
use crate::diff::{Built, borrow_from_siblings, borrow_pasture, build_sim};
use crate::gamelog::{Initial, Log};
use crate::input::Stream;
use crate::load::Loaded;
use sim::Tuning;
use sim::observe::Scene;
use std::path::Path;

/// A game stepped from its start, with no dump past the setup block.
pub struct FreeRunner {
    built: Built,
    stream: Option<Stream>,
    /// Ticks run. After `n` ticks the state is the dump's `FRAME n`.
    frame: i64,
}

impl FreeRunner {
    /// A runner over the game `init` starts. `init` is a start dump's
    /// `Initial`, with whatever its siblings lend it already borrowed.
    pub fn new(loaded: &Loaded, init: &Initial<'_>, tuning: Tuning) -> Self {
        Self {
            built: build_sim(loaded, init, tuning),
            stream: None,
            frame: 0,
        }
    }

    /// A runner over a capture's game: `long`'s setup block, each
    /// `sibling`'s (the `DUMP_ALL` start a pair is stood up from) with what
    /// its frames print of the start — the figures' clocks and the
    /// animation lengths, the inputs `diff::borrow_from_siblings` lends —
    /// and the pasture's animals from `trace` (`diff::borrow_pasture`; no
    /// dump prints owner 9). No frame of `long` is read.
    pub fn from_captures(
        loaded: &Loaded,
        long: &Path,
        siblings: &[&Path],
        trace: Option<&crate::trace::Trace>,
        tuning: Tuning,
    ) -> std::io::Result<Self> {
        let text = IndexedCapture::open(long)?.read_replay_setup()?;
        let log = Log::parse(&text);
        with_sibling_initials(siblings, &[], &mut |refs| {
            let mut init = log
                .replay_initial()
                .ok_or_else(|| std::io::Error::other("no initial state"))?;
            borrow_from_siblings(&mut init, refs);
            if let Some(tr) = trace {
                borrow_pasture(&mut init, tr);
            }
            Ok(Self::new(loaded, &init, tuning))
        })
    }

    /// A runner over a lobby's **built start** (`docs/START.md` §7, item
    /// 4145): the capture's `GAMEINFO` and player rows — the lobby's
    /// choices — through [`crate::diff::from_lobby`] and
    /// [`crate::diff::built_from_lobby`], as the minted script's walk
    /// stands it up (`docs/RECGAME.md` §9). Nothing of the capture past its
    /// setup block is read, and no correction is installed.
    pub fn from_lobby(loaded: &Loaded, lobby: &Path, tuning: Tuning) -> std::io::Result<Self> {
        let mut source = IndexedCapture::open(lobby)?;
        let built = source.with_replay_initial(|init| {
            let ours = crate::diff::from_lobby(loaded, &init.game_info, &init.players, tuning);
            crate::diff::built_from_lobby(loaded, ours).0
        })?;
        Ok(Self {
            built,
            stream: None,
            frame: 0,
        })
    }

    /// A runner over **the page's choices** (item 4184, `docs/LOBBY.md`):
    /// the block they print ([`crate::lobby::Choices::block`]), parsed as a
    /// dump's is, through the same [`crate::diff::from_lobby`] and
    /// [`crate::diff::built_from_lobby`] as [`from_lobby`](Self::from_lobby)
    /// — so a page that prints a capture's block builds that capture's
    /// start. Also the walk's notes: what the built start could not build.
    pub fn from_choices(
        loaded: &Loaded,
        choices: &crate::lobby::Choices,
        tuning: Tuning,
    ) -> (Self, Vec<String>) {
        let lobby = choices.lobby();
        let (info, players) = lobby.borrowed();
        let ours = crate::diff::from_lobby(loaded, &info, &players, tuning);
        let (built, notes) = crate::diff::built_from_lobby(loaded, ours);
        (
            Self {
                built,
                stream: None,
                frame: 0,
            },
            notes,
        )
    }

    /// The simulation, for a comparison of two runners' whole state.
    #[cfg(test)]
    pub(crate) fn sim(&self) -> &sim::Sim {
        &self.built.sim
    }

    /// A person's order, issued now: one package of the script's
    /// vocabulary (`recgame::Order`) for the frame about to be stepped,
    /// handed to the turn pump ([`Stream::pump`]) as a recording's package
    /// would be — encoded and decoded through the wire, so what the walk
    /// takes is what a file written from the same order carries.
    pub fn issue(&mut self, order: &crate::recgame::Order) {
        let stream = self.stream.get_or_insert_with(Stream::pump_empty);
        stream.push(order);
    }

    /// The order stream a recorded game carries, applied before each tick.
    pub fn with_stream(mut self, stream: Stream) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Ticks run so far: the state is the dump's `FRAME n`.
    pub fn frame(&self) -> i64 {
        self.frame
    }

    /// One tick: the stream's orders for this frame, then `Sim::tick`.
    pub fn step(&mut self) -> Vec<sim::Tick> {
        if let Some(s) = &mut self.stream {
            // The harness's frame convention (`diff::Replay::step`): the
            // package for recording frame `n` is processed by the tick that
            // produces `FRAME n + 1`.
            s.apply(self.frame as i32, &mut self.built);
        }
        let ticks = self.built.sim.command().tick();
        self.frame += 1;
        ticks
    }

    /// This tick's drawable state as `viewer` sees it (`Sim::observe`).
    pub fn scene(&self, viewer: Option<sim::Player>) -> Scene {
        self.built.sim.observe(viewer)
    }

    /// The terrain grids as they stand (`Sim::observe_terrain`).
    pub fn terrain(&self) -> sim::observe::TerrainScene {
        self.built.sim.observe_terrain()
    }

    /// The run's height reads that touched a corner this crate cannot pin
    /// to the original's single (`docs/COMBAT.md` §46.3), taken: the count
    /// is zeroed, so `Built`'s guard on drop reads nothing and the caller
    /// holds the number to the pin it owns (item 4166's census, which
    /// walks many games under one test).
    #[cfg(test)]
    pub(crate) fn take_ground_inexact(&mut self) -> u32 {
        std::mem::take(&mut self.built.sim.trace.ground_inexact)
    }

    /// Step to `FRAME last`, handing each tick's scene to `see`.
    pub fn run_to(&mut self, last: i64, viewer: Option<sim::Player>, mut see: impl FnMut(&Scene)) {
        while self.frame < last {
            self.step();
            see(&self.scene(viewer));
        }
    }
}

/// **The scene as the draw list takes it** (`docs/RENDERER.md` §3.5, item
/// 4026): one [`render::draw::Placed`] per figure of every unit on the map,
/// at the figure's own point and height. `render` depends on nothing in
/// `sim`, so the copy is made here, in the crate that holds both; the
/// dependency stays one-way, `rondata` → `render`, as item 4019 set it.
pub fn placed(scene: &Scene) -> Vec<render::draw::Placed> {
    scene
        .units
        .iter()
        .flat_map(|u| {
            u.figures
                .iter()
                .enumerate()
                .map(move |(k, f)| render::draw::Placed {
                    who: u.id.who,
                    o: u.id.o,
                    figure: k as u16,
                    x: f.pos.x,
                    y: f.pos.y,
                    z: f.z,
                    piece: f.gpiece,
                    slot: f.anim,
                    frame: f.cur_time,
                    facing: f.facing.0 as u32,
                })
        })
        .collect()
}

/// **The scene's buildings as the draw list takes them** (item 4035): one
/// [`render::draw::PlacedBuilding`] per building in use, at its own point
/// and height, with its piece resolved through the loader
/// ([`crate::artdata::BuildLook::piece`]: the type it is now, its owner's
/// nation's style and backup style, its owner's age) and its type's
/// `halfoffset`. `sim` models no piece; the door carries what the walk
/// reads, and `tribes` is the install's nations in the tree's order
/// ([`crate::Install::tribe_defs`]). A bare object, with no type, is left
/// out: it has no `<BUILD>`. Its foundation and scaffold (item 4061) are
/// the look's for the type it is now and the piece it shows, which needs
/// a look [`crate::artdata::BuildLook::with_types`] has seen the types of.
pub fn placed_buildings(
    scene: &Scene,
    look: &crate::artdata::BuildLook,
    tribes: &[crate::TribeDef],
) -> Vec<render::draw::PlacedBuilding> {
    let base = crate::load::BASE_BUILDTYPES;
    scene
        .buildings
        .iter()
        .filter_map(|b| {
            let type_index = base + b.ty? as i32;
            let (style, backup) = tribes
                .get(b.tribe)
                .map_or((0, 0), |t| (t.build_continent, t.backup_build_continent));
            let piece = look.piece(type_index, style, backup, b.age);
            Some(render::draw::PlacedBuilding {
                who: b.id.who,
                o: b.id.o,
                x: b.pos.x,
                y: b.pos.y,
                z: b.z,
                type_index,
                orig_type: b.orig_ty.map_or(type_index, |t| base + t as i32),
                piece,
                half_offset: look.half_offset(type_index),
                active: b.active,
                job_counter: b.job_counter,
                construct_time: b.construct_time,
                field: b.field.map(|f| render::draw::Field {
                    x_size: f.x_size,
                    y_size: f.y_size,
                    farm_type: f.farm_type,
                    heights: f.heights,
                }),
                started: b.started,
                foundation: look.foundation(type_index, piece),
                scaffold: look.scaffold_table(type_index),
                age: b.age,
            })
        })
        .collect()
}

/// `viewer`'s sight on a scene observed for it, or `None` for a scene with
/// no fog plane — the observer's. `width` is the map's in cells
/// ([`sim::observe::TerrainScene::width`]).
pub fn sight(
    scene: &Scene,
    viewer: sim::Player,
    width: i32,
    height: i32,
) -> Option<render::draw::Sight<'_>> {
    scene.fog.as_deref().map(|plane| render::draw::Sight {
        viewer,
        plane,
        xs: width * 2,
        ys: height * 2,
        cell: sim::world::UNITS_PER_CELL / 2,
    })
}

/// Each sibling's `Initial` with its frames' observations, scoped to `f`.
fn with_sibling_initials<R>(
    paths: &[&Path],
    inits: &[&Initial<'_>],
    f: &mut dyn FnMut(&[&Initial<'_>]) -> std::io::Result<R>,
) -> std::io::Result<R> {
    let Some((path, rest)) = paths.split_first() else {
        return f(inits);
    };
    IndexedCapture::open(path)?.with_replay_initial(|init| {
        let mut refs = inits.to_vec();
        refs.push(&init);
        with_sibling_initials(rest, &refs, f)
    })?
}

#[cfg(test)]
pub(crate) mod ages;
#[cfg(test)]
pub(crate) mod tests;
