//! The draw-site trace — reading `rontrace.log` from Rust.
//!
//! `tools/trace/` is the instrument (`docs/ORACLE.md`, "The draw-site trace
//! and function coverage"); `tools/trace/report.py` has been the only
//! reader of it since it existed. This is the second, and it exists for
//! one reason: **a count is a weak assertion and the trace carries a
//! sequence**. Every draw record names the *site* that took it — the
//! return address into the calling function — so a mechanic can be checked
//! draw for draw rather than by a total, and checked that way even while
//! the frame's stream has not yet been made to line up
//! (`docs/SCOUT.md` §10 is the first such check).
//!
//! Nothing here reads the user's install: a trace is a file the user's own
//! run wrote, taken by path exactly as a `gamelog.txt` is, and nothing
//! from it is committed.
//!
//! ## The format
//!
//! A 32-byte header then 32-byte records, each eight little-endian `u32`:
//! `[kind, a, b, c, d, e, f, frame]` (`tools/trace/tracer.c`'s `emit`). The
//! header is a record whose `kind` is the ASCII `RONT`, carrying the
//! version, the image base, `.text`'s RVA and size, the function count and
//! the window's low frame.
//!
//! | kind | a | b | c | d | e | f |
//! | --- | --- | --- | --- | --- | --- | --- |
//! | 0 `HIT` | function VA | thread | | | | |
//! | 1 `get()`, 3 `get(a,b)`, 4 `rand_real`, 6 `reseed` | the **caller's return address** | the `Random *` | the seed **before** the step | the caller's caller | and its caller | `arg0` |
//! | 2 `FRAME` | frame | `game_random`'s word | functions re-armed | `do_frame`'s caller | | |
//! | 5 `INFO` | code | … | | | | |
//!
//! Two things about that table decide everything below. The `Random *` in
//! `b` is what separates the **sync stream** from the renderer's four
//! other generators — only a step of `game_random` is simulation state.
//! And the seed in `c` is the word *before* the step, so a run of draws
//! reads as the LCG's own sequence and the first of them is the word to
//! seed a replay with.

use std::path::Path;

/// The image base every address here is normalised to. A run that
/// relocated reports its own base in the header and is folded back to this
/// one, so a site is comparable between runs and against the Ghidra
/// export.
pub const IMAGE_BASE: u32 = 0x0040_0000;

/// `game_random`'s RVA — the one generator whose steps are simulation
/// state. `tools/trace/report.py` carries the same constant.
pub const RVA_GAME_RANDOM: u32 = 0x00a3_7a8c;

/// The sites the simulation models, and the names it marks them with.
///
/// This is **not** a symbol table: naming the trace in general needs the
/// Ghidra export's `INDEX.tsv`, which never enters this repo, and
/// `tools/trace/report.py` stays where a name comes from. This is the far
/// smaller thing a differential check needs — the handful of addresses a
/// mechanic in `crate::sim` has claimed, each paired with the string that
/// mechanic's own `Sim::mark` writes. The label lives in `sim`, beside the
/// code that spends the draw; only the address lives here.
///
/// `via` is the disambiguator, and it is why the table is not a flat map.
/// One address can be several sites: `Guy::set_anim+0x97a` is the idle
/// roll for an animal, for an idle unit, for a gathering one's stand and
/// for the phase-7 wrap, and the trace tells them apart only by the `ebp`
/// chain ([`Draw::up`]). An entry with `via` matches when that address is
/// somewhere in the chain; the first matching entry wins, so a
/// chain-qualified entry must precede a bare one for the same site.
///
/// Every offset here is a return address into the named function, taken
/// from a run's own trace and checked against the Ghidra export
/// (`docs/SYNC.md` §3, §5).
pub const SITES: &[(u32, Option<u32>, &str)] = &[
    // `Leader::compute_sites@006cc950` — the AI's region sweep.
    (0x006c_cdfc, None, sim::ai_sites::SITE_STRIDE),
    (0x006c_ce5a, None, sim::ai_sites::SITE_MARK),
    // `Leader::make_stuff@006c8af0` — the two expiry walks, step 4's over
    // the head's type and step 6's over a bought slot's (`docs/AI.md` §2.6).
    (0x006c_8d11, None, sim::ai_make::SITE_EXPIRE_HEAD),
    (0x006c_912d, None, sim::ai_make::SITE_EXPIRE_SLOT),
    // `Leader::produce_building@006e1400` — the spiral's score and the
    // 2×2 jitter's.
    (0x006e_2099, None, sim::ai_place::SITE_SPIRAL),
    (0x006e_2c05, None, sim::ai_place::SITE_JITTER),
    // `GameDaemon::calc_market@00732270` — three a good.
    (0x0073_22c4, None, sim::market::SITE_A),
    (0x0073_22ee, None, sim::market::SITE_B),
    (0x0073_232e, None, sim::market::SITE_LENGTH),
    // `Guy::set_anim@005da300+0x97a` — one address, four callers.
    (
        0x005d_ac7a,
        Some(0x005d_7479), // `Animal::do_idle+0x19`
        sim::anim::SITE_IDLE_ANIMAL,
    ),
    (
        0x005d_ac7a,
        Some(0x0060_dd4d), // `Unit::do_idle+0x7d`
        sim::anim::SITE_IDLE_UNIT,
    ),
    (
        0x005d_ac7a,
        Some(0x005d_a081), // `Guy::inc_time+0x271`
        sim::anim::SITE_WRAP,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_027f), // `Unit::do_non_flat_gather+0x10f`
        sim::anim::SITE_STAND_GATHER,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_1144), // `Unit::do_non_flat_gather+0xfd4`
        sim::anim::SITE_STAND_TILE,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_0d09), // `Unit::do_non_flat_gather+0xb99`
        sim::anim::SITE_STAND_RETURN,
    ),
    (
        0x005d_ac7a,
        Some(0x005d_93df), // `Guy::move+0x19f`, the arrival stand
        sim::anim::SITE_ARRIVE,
    ),
    (
        0x005d_ac7a,
        Some(0x005f_b753), // `Unit::move_step+0x823`, the blocked stand
        sim::anim::SITE_BLOCKED,
    ),
    // `Unit::resolve_unit_collision@005f9d30+0xb52` — the head-on pair's
    // stagger, the collision mechanic's only draw.
    (0x005f_a882, None, sim::collide::SITE_PAUSE),
    // `Guy::set_anim@005da300+0x104b` — the gaia bird's wing-beat coin.
    // Its own address, so no chain is needed to tell it from the other
    // four (`docs/SYNC.md` §3.9).
    (0x005d_b34b, None, sim::anim::SITE_BIRD_COIN),
    // `Guy::init_real@005db6b0` — the creation roll.
    (0x005d_b702, None, sim::anim::SITE_INIT_REAL),
    // `Unit::do_non_flat_gather@005f0170` — the wood machine's own three.
    // The last two are one apparent branch and two real ones: `+0xcc3` is
    // the chopping guy's `% 100 + 300` and `+0xdad` the arrival frame's
    // `% 50 + 100` (`docs/ORDERS.md` §6.4).
    (0x005f_06bb, None, sim::orders::SITE_TILE_WAIT),
    (0x005f_0e33, None, sim::orders::SITE_WORK_WAIT),
    (0x005f_0f1d, None, sim::orders::SITE_ARRIVE_WAIT),
    // `GameAccess::rnd@0043cca0+0x20` — the frameless helper. Its address
    // says nothing on its own; the chain does, and `Unit::do_job+0x67` is
    // `do_gather`'s own return address (both it and `GameAccess::rnd` are
    // skipped by the `ebp` walk).
    (
        0x0043_ccc0,
        Some(0x0061_7a77), // `Unit::do_job+0x67` — `Unit::do_gather`
        sim::orders::SITE_FARM_CELL,
    ),
    // `Unit::do_move@005f7b30` — the grid draw.
    (0x005f_89b4, None, sim::orders::SITE_MOVE_GRID),
    // `Unit::think_scout@005f6010` — the ring walk (`docs/SCOUT.md` §10).
    (0x005f_6446, None, sim::scout::SITE_ROTATION),
    (0x005f_6468, None, sim::scout::SITE_PHASE),
    (0x005f_665c, None, sim::scout::SITE_CELL),
    // `Animal::do_idle@005d7460` — a herd animal's wander: the coin, then
    // the direction and the two step counts. Four addresses of its own.
    (0x005d_74e3, None, sim::gaia::SITE_WANDER_ROLL),
    (0x005d_7604, None, sim::gaia::SITE_WANDER_DIR),
    (0x005d_7634, None, sim::gaia::SITE_WANDER_X),
    (0x005d_7672, None, sim::gaia::SITE_WANDER_Y),
    // `Animal::think_farm_animal@005d7700` — a pasture animal's step.
    (0x005d_7842, None, sim::farms::SITE_ANIMAL_DIR),
    // `Objects::process_all@0065dce0` — the birds' sampling.
    (0x0065_dfbf, None, sim::gaia::SITE_BIRD_X),
    (0x0065_dfeb, None, sim::gaia::SITE_BIRD_Y),
    // `Animal::think_bird@005d79e0` — a live bird's three, every eighth
    // frame, under `Unit::do_air_patrol+0x28` < `Unit::do_job+0xd7`.
    (0x005d_7a62, None, sim::gaia::SITE_BIRD_WANDER_X),
    (0x005d_7a86, None, sim::gaia::SITE_BIRD_WANDER_Y),
    (0x005d_7bd8, None, sim::gaia::SITE_BIRD_LAND),
    // …and the landing search it opens, thirty rounds of two.
    (0x005d_7c8a, None, sim::gaia::SITE_BIRD_SEARCH_CELL),
    (0x005d_7cb3, None, sim::gaia::SITE_BIRD_SEARCH_SCORE),
    // `Herd::process@00741760` — one herd's walk.
    (0x0074_1777, None, sim::gaia::SITE_HERD_X),
    (0x0074_1796, None, sim::gaia::SITE_HERD_Y),
    // `MathUtilFuncSet::rand_int@009e1890` — the script VM's one draw,
    // under the interpreter's call-out.
    (
        0x009e_18a8,
        Some(0x009d_5901), // `ScriptFuncSet::call_func+0x401`
        sim::ai_host::SITE_RAND_INT,
    ),
    // `PathFinder::calc_road_cost@00686300` — the road jitter, one draw a
    // node costed, under `astar_caravan_road+0x52b < find_road+0x3a8`
    // (`docs/ROADS.md` §5). Its own address, so no chain is needed.
    (0x0068_6346, None, sim::roads::SITE_COST),
    // `Farms::inc_time@008d8600` — the crop clock.
    (0x008d_87ae, None, sim::farms::SITE_CHANCE),
    (0x008d_87de, None, sim::farms::SITE_SPROUT),
    // `Farms::add@008d8a40` — the pasture coin and the ambience emitter.
    (0x008d_8b68, None, sim::farms::SITE_TYPE_COIN),
    (0x008d_8c7f, None, sim::farms::SITE_AMBIENCE_X),
    (0x008d_8c9b, None, sim::farms::SITE_AMBIENCE_Y),
];

/// The header's `kind`: `RONT`, little-endian.
const MAGIC: u32 = 0x544e_4f52;

/// The record kinds that step a generator.
const DRAW_KINDS: [u32; 4] = [1, 3, 4, 6];

/// One draw, as the trace records it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    /// The **return address** into the function that drew, normalised to
    /// [`IMAGE_BASE`] — what `report.py` prints as `Class::method+0xNN`.
    pub site: u32,
    /// The `Random *` this stepped, normalised. [`Draw::sync`] is the test
    /// that matters.
    pub rng: u32,
    /// The generator's word **before** the step.
    pub seed: u32,
    /// The caller's caller, and its caller — two more frames of the `ebp`
    /// chain, normalised. Zero where the walk ran out.
    pub up: [u32; 2],
    /// The sim-frame, as `Game::do_frame` counts it; −1 for the setup path.
    pub frame: i64,
    /// The record kind — 1 `get()`, 3 `get(a, b)`, 4 `rand_real`,
    /// 6 `reseed`.
    pub kind: u32,
}

impl Draw {
    /// Whether this stepped `game_random`, which is the whole of the
    /// simulation's stream. A draw that did not is the renderer's and
    /// belongs to no frame's count.
    pub fn sync(&self) -> bool {
        self.rng == IMAGE_BASE + RVA_GAME_RANDOM
    }

    /// **What the draw returned.**
    ///
    /// The record carries the word *before* the step, so the outcome is
    /// recoverable without the game: step the LCG once, then apply
    /// `Random::get(lo, hi)`'s own scaling. Every site the documents cite
    /// is `(0, 0xffff)`, which is [`sim::combat::Rng::roll`].
    ///
    /// This is the only reader of a draw whose outcome **no dump holds** —
    /// a coin inside `Setup::build_empire`, a direction, an idle roll —
    /// and it is what makes a setup draw borrowable (`docs/SYNC.md`
    /// §3.11). `rand_real` and `reseed` return `None` rather than a number
    /// that would be a guess.
    pub fn value(&self) -> Option<i32> {
        if self.kind != 1 && self.kind != 3 {
            return None;
        }
        let mut rng = sim::combat::Rng::new(self.seed);
        Some(rng.roll())
    }
}

/// `Farms::add_animals@008d8f30`'s three draws per animal, at the offsets
/// run39's own trace names: the species coin, the `y` offset and the `x`
/// (`docs/SYNC.md` §3.11). The fourth of the four is `Guy::init_real`'s,
/// inside `Objects::init_unit`, and leaves nothing to borrow.
pub const ADD_ANIMALS_COIN: u32 = 0x008d_8fc2;
pub const ADD_ANIMALS_Y: u32 = 0x008d_9064;
pub const ADD_ANIMALS_X: u32 = 0x008d_90b2;

/// A parsed `rontrace.log`.
#[derive(Clone, Debug)]
pub struct Trace {
    /// The image base the run reported.
    pub base: u32,
    /// Every draw, in file order.
    pub draws: Vec<Draw>,
    /// Each `FRAME` record: the sim-frame and `game_random`'s word at its
    /// `do_frame` entry — the same pairing `gamelog::Log::frame_seeds`
    /// gives from a `DUMP_ALL` dump, from the other side.
    pub frames: Vec<(i64, u32)>,
}

impl Trace {
    /// Parses the bytes. `None` if the header is not a trace.
    pub fn parse(bytes: &[u8]) -> Option<Trace> {
        if bytes.len() < 32 {
            return None;
        }
        let word = |off: usize| -> u32 {
            u32::from_le_bytes([bytes[off], bytes[off + 1], bytes[off + 2], bytes[off + 3]])
        };
        if word(0) != MAGIC {
            return None;
        }
        let base = word(8);
        // A run that loaded elsewhere is folded back, so a site is the
        // same number as the Ghidra export's.
        let norm = |va: u32| -> u32 {
            if va == 0 {
                0
            } else {
                va.wrapping_sub(base).wrapping_add(IMAGE_BASE)
            }
        };
        let mut t = Trace {
            base,
            draws: Vec::new(),
            frames: Vec::new(),
        };
        let mut off = 32;
        // `emit` writes the frame counter into slot 7, and it is `-1`
        // before the first `do_frame`.
        while off + 32 <= bytes.len() {
            let r: Vec<u32> = (0..8).map(|i| word(off + i * 4)).collect();
            let frame = i64::from(r[7] as i32);
            if r[0] == 2 {
                t.frames.push((i64::from(r[1] as i32), r[2]));
            } else if DRAW_KINDS.contains(&r[0]) {
                t.draws.push(Draw {
                    site: norm(r[1]),
                    rng: norm(r[2]),
                    seed: r[3],
                    up: [norm(r[4]), norm(r[5])],
                    frame,
                    kind: r[0],
                });
            }
            off += 32;
        }
        Some(t)
    }

    /// Reads a trace off disk. `Ok(None)` when the file is not one.
    pub fn read(path: &Path) -> std::io::Result<Option<Trace>> {
        Ok(Trace::parse(&std::fs::read(path)?))
    }

    /// **The pasture's five, read back out of the setup path.**
    ///
    /// `Farms::add_animals` runs inside `Setup::build_empire`, whose stream
    /// the harness does not replay, so its three marks — the species coin
    /// and the two offsets — exist nowhere else: no dump prints an owner-9
    /// object at all (`docs/SYNC.md` §3.6). [`Draw::value`] recovers them
    /// from the seeds the records carry, which is what makes the five
    /// **borrowable** the way a sibling dump's heights and herds are.
    ///
    /// Returns one `Vec` a pasture, in the order the setup created them,
    /// each of [`sim::farms::FARM_ANIMALS`] seeds. A trace that opened
    /// after the setup — every windowed capture — returns nothing, and the
    /// simulation then keeps its stand-in.
    pub fn add_animals(&self) -> Vec<Vec<sim::farms::AnimalSeed>> {
        let sites = [ADD_ANIMALS_COIN, ADD_ANIMALS_Y, ADD_ANIMALS_X];
        let marks: Vec<&Draw> = self
            .draws
            .iter()
            .filter(|d| d.sync() && d.frame < 0 && sites.contains(&d.site))
            .collect();
        let mut seeds: Vec<sim::farms::AnimalSeed> = Vec::new();
        // The three are consecutive and in this order; anything else is a
        // trace whose window clipped the run, and a partial animal is
        // dropped rather than half-borrowed.
        for t in marks.chunks(3) {
            let [coin, y, x] = t else { break };
            if (coin.site, y.site, x.site) != (sites[0], sites[1], sites[2]) {
                break;
            }
            let (Some(coin), Some(y), Some(x)) = (coin.value(), y.value(), x.value()) else {
                break;
            };
            let fold =
                |v: i32| v.rem_euclid(sim::farms::ANIMAL_SPREAD) - sim::farms::ANIMAL_SPREAD / 2;
            seeds.push(sim::farms::AnimalSeed {
                chicken: coin & 1 == 0,
                dy: fold(y),
                dx: fold(x),
            });
        }
        seeds
            .chunks(sim::farms::FARM_ANIMALS as usize)
            .filter(|c| c.len() == sim::farms::FARM_ANIMALS as usize)
            .map(<[sim::farms::AnimalSeed]>::to_vec)
            .collect()
    }

    /// The sync-stream draws of one sim-frame, in the order they were
    /// taken. `-1` is the setup path.
    pub fn frame_draws(&self, frame: i64) -> Vec<Draw> {
        self.draws
            .iter()
            .filter(|d| d.frame == frame && d.sync())
            .copied()
            .collect()
    }

    /// The sync-stream draws of one sim-frame whose **site** lies inside
    /// `[lo, hi)` — one function's own draws, with everything its callees
    /// took left out. `hi` is the next function's address in the export.
    pub fn run_in(&self, frame: i64, lo: u32, hi: u32) -> Vec<Draw> {
        self.frame_draws(frame)
            .into_iter()
            .filter(|d| (lo..hi).contains(&d.site))
            .collect()
    }

    /// One draw's name, from [`SITES`]: the string the simulation's own
    /// `Sim::mark` writes at the same site, or the bare address when
    /// nothing models it. An unmodelled draw therefore reads as a hex
    /// number in the comparison, which is what makes a hole in the
    /// simulation legible rather than silent.
    pub fn label(&self, d: &Draw) -> String {
        SITES
            .iter()
            .find(|(site, via, _)| *site == d.site && via.is_none_or(|v| d.up.contains(&v)))
            .map_or_else(
                || format!("{:x}", d.site),
                |(_, _, name)| (*name).to_string(),
            )
    }

    /// A frame's sync draws as a **sequence of names** — the original's
    /// side of the comparison [`crate::diff::mark_sites`] builds for ours.
    ///
    /// This is the whole point of the naming table. `--diff` prints a
    /// per-phase count and `--trace` a per-site one, and lining the two up
    /// has been an eye exercise; two `Vec<String>`s of the same vocabulary
    /// are an `assert_eq!`, and where they part is where the simulation's
    /// frame parts from the original's.
    pub fn labels(&self, frame: i64) -> Vec<String> {
        self.frame_draws(frame)
            .iter()
            .map(|d| self.label(d))
            .collect()
    }

    /// A frame's sync draws folded into consecutive runs of one site —
    /// `5f6446 ×3, 5f6468, 5f665c ×4, …` — which is the harness's answer
    /// to `report.py … sites` for a *sequence* rather than a total, and
    /// the shape `Built::phase_fold` prints for our own side.
    ///
    /// Sites are bare addresses: naming them needs the Ghidra export's
    /// `INDEX.tsv`, which never enters this repo. `report.py` is where a
    /// name comes from.
    pub fn site_fold(&self, frame: i64) -> String {
        let draws = self.frame_draws(frame);
        let mut out: Vec<String> = Vec::new();
        let mut i = 0;
        while i < draws.len() {
            let mut j = i;
            while j + 1 < draws.len() && draws[j + 1].site == draws[i].site {
                j += 1;
            }
            let n = j - i + 1;
            out.push(if n > 1 {
                format!("{:x} ×{n}", draws[i].site)
            } else {
                format!("{:x}", draws[i].site)
            });
            i = j + 1;
        }
        out.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One header and three records, hand-built: a `FRAME`, a
    /// `game_random` draw and one of the renderer's.
    fn bytes(recs: &[[u32; 8]]) -> Vec<u8> {
        let mut v = Vec::new();
        for r in recs {
            for w in r {
                v.extend_from_slice(&w.to_le_bytes());
            }
        }
        v
    }

    const GAME_RANDOM: u32 = IMAGE_BASE + RVA_GAME_RANDOM;

    #[test]
    fn a_trace_parses_and_keeps_only_the_sync_stream() {
        let log = bytes(&[
            [MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 48_233, 0, 0xffff_ffff],
            [2, 0, 0x1111_1111, 3, 0, 0, 0, 0],
            [3, 0x005f_6446, GAME_RANDOM, 0x9c59_1b2b, 0, 0, 0, 0],
            // The renderer's — a different `Random *`, so not the stream.
            [3, 0x0022_3908, 0x0022_3908, 0xdead_beef, 0, 0, 0, 0],
            [3, 0x005f_6468, GAME_RANDOM, 0x70fc_74f0, 0, 0, 0, 0],
            [0, 0x005f_6010, 7, 0, 0, 0, 0, 0],
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(t.base, IMAGE_BASE);
        assert_eq!(t.frames, vec![(0, 0x1111_1111)]);
        assert_eq!(t.draws.len(), 3, "the HIT and the FRAME are not draws");
        let f0 = t.frame_draws(0);
        assert_eq!(f0.len(), 2, "the renderer's is not the sync stream");
        assert_eq!(f0[0].seed, 0x9c59_1b2b);
        assert_eq!(f0[1].site, 0x005f_6468);
    }

    /// A run that loaded somewhere other than `0x400000` folds back, so a
    /// site is the same number as the export's.
    #[test]
    fn a_relocated_run_normalises_to_the_export_s_addresses() {
        let base = 0x0100_0000;
        let log = bytes(&[
            [MAGIC, 1, base, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            [3, base + 0x001f_6446, base + RVA_GAME_RANDOM, 7, 0, 0, 0, 0],
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(t.draws[0].site, 0x005f_6446);
        assert!(t.draws[0].sync(), "and it is still the sync stream");
    }

    /// `run_in` isolates one function's own draws, and `site_fold` reads
    /// them as runs.
    #[test]
    fn run_in_isolates_a_function_and_site_fold_reads_it_as_runs() {
        let d = |site: u32| [3, site, GAME_RANDOM, 0, 0, 0, 0, 0];
        let log = bytes(&[
            [MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            d(0x005f_6446),
            d(0x005f_6446),
            d(0x005f_6468),
            // A callee's, outside `think_scout`'s range.
            d(0x005f_2800),
            d(0x005f_665c),
        ]);
        let t = Trace::parse(&log).expect("a trace");
        let mine = t.run_in(0, 0x005f_6010, 0x005f_6e40);
        assert_eq!(
            mine.iter().map(|d| d.site).collect::<Vec<_>>(),
            vec![0x005f_6446, 0x005f_6446, 0x005f_6468, 0x005f_665c],
            "the callee's draw is not this function's"
        );
        assert_eq!(t.site_fold(0), "5f6446 ×2, 5f6468, 5f2800, 5f665c");
    }

    /// **A setup draw's outcome, recovered from the seed it carries.**
    ///
    /// The three records are run39's own first animal, verbatim
    /// (`report.py <log> draws setup`): seeds `a236f580`, `588a6adf`,
    /// `51d23ab2`, which return 27358, 15025 and 55912 — an even coin, so
    /// a chicken, and `% 0x180 − 0xc0` on each of the other two, so
    /// `(dy −143, dx 40)`. Nothing else in the capture holds any of it:
    /// no dump prints an owner-9 object at all (`docs/SYNC.md` §3.11).
    #[test]
    fn a_setup_draw_s_outcome_is_recovered_from_the_seed_it_carries() {
        let setup = |site: u32, seed: u32| [3u32, site, GAME_RANDOM, seed, 0, 0, 0, 0xffff_ffff];
        let log = bytes(&[
            [MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 1, 0, 0xffff_ffff],
            setup(ADD_ANIMALS_COIN, 0xa236_f580),
            setup(ADD_ANIMALS_Y, 0x588a_6adf),
            setup(ADD_ANIMALS_X, 0x51d2_3ab2),
        ]);
        let t = Trace::parse(&log).expect("a trace");
        assert_eq!(
            t.draws.iter().map(Draw::value).collect::<Vec<_>>(),
            vec![Some(27358), Some(15025), Some(55912)],
            "the values report.py prints beside these three records"
        );
        // Four animals short of a pasture, so nothing is borrowable yet.
        assert_eq!(t.add_animals(), Vec::<Vec<sim::farms::AnimalSeed>>::new());

        let mut recs = vec![[MAGIC, 1, IMAGE_BASE, 0x1000, 0x2000, 1, 0, 0xffff_ffff]];
        for _ in 0..sim::farms::FARM_ANIMALS {
            recs.push(setup(ADD_ANIMALS_COIN, 0xa236_f580));
            recs.push(setup(ADD_ANIMALS_Y, 0x588a_6adf));
            recs.push(setup(ADD_ANIMALS_X, 0x51d2_3ab2));
        }
        let five = Trace::parse(&bytes(&recs)).expect("a trace");
        let got = five.add_animals();
        assert_eq!(got.len(), 1, "one pasture");
        assert_eq!(
            got[0],
            vec![
                sim::farms::AnimalSeed {
                    chicken: true,
                    dy: -143,
                    dx: 40,
                };
                sim::farms::FARM_ANIMALS as usize
            ]
        );
    }

    #[test]
    fn something_that_is_not_a_trace_is_refused() {
        assert!(Trace::parse(b"not a trace at all, no header here").is_none());
        assert!(Trace::parse(b"short").is_none());
    }
}
