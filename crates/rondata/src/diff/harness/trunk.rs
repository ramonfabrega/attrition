//! **The trunk**: checkpoints left along a silent run-up, keyed by the
//! state itself (item 4220, DECISIONS 132 §11; `docs/TEST_PERFORMANCE.md`,
//! "The group unit").
//!
//! The kept walk ([`super::prefix`]) hands one state from a widening to the
//! next, keyed by the inputs' names, and libtest runs a group's tests in
//! name order: a window that starts before the kept frame stands its game
//! up again, and the shapes that name no inputs — a closing state, a
//! packet's frame — never resumed at all. Each of them does the same thing
//! first: it ticks a state it has just stood up, comparing nothing, to the
//! frame its check starts on. [`advance`] is that run-up.
//!
//! **The key is the state.** A state at frame 0 is printed whole
//! ([`Print`]: its derived `Debug`, hashed, with the print's length beside
//! the hash), and two states with one print walk one trajectory:
//! [`Built::tick`] reads nothing but the state, the simulation holds no
//! hash map and no mutable static, and the diagnostics that read the
//! environment turn sharing off. So nothing is asserted about which inputs
//! stand up equal: a test that stages a script, turns the audit on, skips a
//! re-seat or builds from another start prints differently and rides no
//! other test's trunk.
//!
//! **What is shared, and what is not.** A rung is a clone of the state at a
//! frame of the cadence, left by whichever run-up passed it first. A later
//! run-up from an equal frame-0 state takes the highest rung at or before
//! its frame and ticks the rest. The test still stands its own game up, so
//! its fixture requests are its own and the audit's rows do not move; the
//! ground guard's count travels in the state, as it would have grown on
//! the test's own walk. No verdict, comparison or recorded field is shared:
//! a run-up compares nothing.
//!
//! **A game's two stand-ups ride one trunk** (item 4224). A window
//! stands its game up from the capture's replay setup; the closing state,
//! the walk to the word and the packets from the whole log. The two
//! states differ in what a tick of the walk never writes: the guys of the
//! frames no word is installed on (a tick reads `frame_guys` only beside
//! a word), the animation lengths the capture's later blocks show, and
//! the stand-up's own notes. The trunk's key is the frame-0 print less
//! those ([`Root`]), and a rung taken by the other stand-up carries the
//! taker's own of each from there on ([`Rung::carry`]). The lengths are
//! read by a tick, so the join rests on the standing check below, not on
//! construction: a length one stand-up holds and the other does not, read
//! before the rung's frame, turns the verifier red.
//!
//! **The standing check** is the whole gate's: under `RON_VERIFY_ORACLE`
//! (or `RON_VERIFY_PREFIX_REUSE`) a run-up that took a rung also ticks its
//! own state from frame 0 and the two prints at the frame must be equal.
use crate::diff::Built;
use std::fmt::Write as _;
use std::hash::Hasher as _;
use std::sync::Mutex;

/// A state's identity: its whole `Debug` print, hashed, the print's length
/// in bytes, and the frame it stands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Print {
    hash: u64,
    len: u64,
    frame: i64,
}

struct Hashing {
    hasher: std::collections::hash_map::DefaultHasher,
    len: u64,
}
impl std::fmt::Write for Hashing {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.hasher.write(s.as_bytes());
        self.len += s.len() as u64;
        Ok(())
    }
}

/// `built`, printed whole and hashed. Every field of the state and of the
/// simulation is in it (`Debug` is derived throughout; a unit's
/// `HeadSerial` prints as its name, being identity and not state).
pub(crate) fn print(built: &Built) -> Print {
    let mut out = Hashing {
        hasher: std::collections::hash_map::DefaultHasher::new(),
        len: 0,
    };
    write!(out, "{built:?}").expect("a hasher takes every write");
    Print {
        hash: out.hasher.finish(),
        len: out.len,
        frame: built.sim.frame,
    }
}

/// A trunk's key and what the stand-up behind it brought: the frame-0
/// state printed less the fields two stand-ups of one game differ in and
/// a tick never writes (the module's header), and the stand-up's own
/// notes, which a rung's notes begin with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Root {
    print: Print,
    notes: usize,
}

/// `built`, standing on frame 0, as a trunk's [`Root`].
fn root(built: &Built) -> Root {
    let mut keyed = built.clone();
    let seeded: std::collections::BTreeSet<i64> =
        keyed.frame_seeds.iter().map(|(frame, _)| *frame).collect();
    keyed.frame_guys.retain(|(frame, _)| seeded.contains(frame));
    keyed.sim.roster.art.lengths.clear();
    keyed.notes.clear();
    let print = print(&keyed);
    keyed.retire();
    Root {
        print,
        notes: built.notes.len(),
    }
}

/// A checkpoint, and how many of its notes its stand-up wrote.
#[derive(Clone)]
struct Rung {
    built: Built,
    notes: usize,
}

impl Rung {
    fn frame(&self) -> i64 {
        self.built.sim.frame
    }

    /// The rung as `own`'s walk would have reached it: `own`'s stand-up
    /// fields and notes, the rung's ticks' notes after them. `own` is a
    /// state of the same trunk whose stand-up wrote `own_notes` notes.
    /// Whether anything was carried that the rung did not already hold.
    fn carry(self, own: &Built, own_notes: usize) -> (Built, bool) {
        let mut built = self.built;
        let joined = built.frame_guys != own.frame_guys
            || built.sim.roster.art.lengths != own.sim.roster.art.lengths
            || built.notes[..self.notes] != own.notes[..own_notes];
        if joined {
            built.frame_guys.clone_from(&own.frame_guys);
            built
                .sim
                .roster
                .art
                .lengths
                .clone_from(&own.sim.roster.art.lengths);
            let ticked = built.notes.split_off(self.notes);
            built.notes = own.notes[..own_notes].to_vec();
            built.notes.extend(ticked);
        }
        (built, joined)
    }
}

/// The first cadence: a rung every 256 frames, doubled each time a trunk
/// would pass [`RUNGS`].
const CADENCE: i64 = 256;
/// The rungs one trunk keeps.
const RUNGS: usize = 12;
/// The trunks a process keeps: a group's tests name two games at most
/// between two tests of one (`suite_groups.py`'s `GAME_TRACES`).
const TRUNKS: usize = 2;
/// A run-up shorter than this prints nothing and takes no rung: the print
/// costs more than the ticks.
const SHORT: i64 = 64;

struct Trunk {
    root: Print,
    cadence: i64,
    /// Ascending by frame, each on a multiple of `cadence`.
    rungs: Vec<Rung>,
    used: u64,
}

#[derive(Default)]
struct Store {
    trunks: Vec<Trunk>,
    clock: u64,
    /// Frames a run-up did not tick, and rungs taken: the suite's log line.
    saved: i64,
    taken: usize,
}

static STORE: Mutex<Option<Store>> = Mutex::new(None);

fn store<T>(with: impl FnOnce(&mut Store) -> T) -> T {
    // A test that panicked inside left nothing half-written: a rung is
    // pushed whole or not at all.
    let mut guard = STORE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    with(guard.get_or_insert_with(Store::default))
}

impl Store {
    fn trunk(&mut self, root: Print) -> &mut Trunk {
        self.clock += 1;
        let at = match self.trunks.iter().position(|t| t.root == root) {
            Some(at) => at,
            None => {
                if self.trunks.len() == TRUNKS {
                    let oldest = (0..TRUNKS)
                        .min_by_key(|&i| self.trunks[i].used)
                        .expect("a full store");
                    for rung in self.trunks.swap_remove(oldest).rungs {
                        rung.built.retire();
                    }
                }
                self.trunks.push(Trunk {
                    root,
                    cadence: CADENCE,
                    rungs: Vec::new(),
                    used: 0,
                });
                self.trunks.len() - 1
            }
        };
        self.trunks[at].used = self.clock;
        &mut self.trunks[at]
    }
}

impl Trunk {
    /// Whether a state on `frame` is a rung this trunk wants.
    fn wants(&self, frame: i64) -> bool {
        frame % self.cadence == 0 && self.rungs.iter().all(|r| r.frame() != frame)
    }

    fn leave(&mut self, rung: Rung) {
        let at = self.rungs.partition_point(|r| r.frame() < rung.frame());
        self.rungs.insert(at, rung);
        while self.rungs.len() > RUNGS {
            self.cadence *= 2;
            let cadence = self.cadence;
            let (kept, thinned): (Vec<Rung>, Vec<Rung>) = std::mem::take(&mut self.rungs)
                .into_iter()
                .partition(|r| r.frame() % cadence == 0);
            self.rungs = kept;
            for rung in thinned {
                rung.built.retire();
            }
        }
    }
}

thread_local! {
    /// A unit test's own switch: the process's is read once from the
    /// environment, and a test of the trunk itself must not depend on it.
    static FORCED: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
    /// A unit test's own verifier switch.
    static VERIFYING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A group's process rides trunks wherever it shares its kept walk;
/// `RON_DISABLE_TRUNK` is a hand's, for a run with the kept walk alone.
fn sharing() -> bool {
    FORCED.with(std::cell::Cell::get).unwrap_or_else(|| {
        static OFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        super::prefix::kept_walks_shared()
            && !*OFF.get_or_init(|| std::env::var_os("RON_DISABLE_TRUNK").is_some())
    })
}

fn verifying() -> bool {
    VERIFYING.with(std::cell::Cell::get)
        || std::env::var_os("RON_VERIFY_ORACLE").is_some()
        || std::env::var_os("RON_VERIFY_PREFIX_REUSE").is_some()
}

/// Tick `built` to `to`, leaving the rungs `root`'s trunk wants.
fn walk(built: &mut Built, to: i64, root: Root) {
    let leaves = Leaves(Some(root));
    while built.sim.frame < to {
        built.tick();
        leaves.passed(built);
    }
}

/// An empty state to swap a moved rung out of: `Built` answers for its
/// ground when it drops, so it is never left half-moved.
fn placeholder() -> Built {
    Built {
        sim: sim::Sim::new(
            sim::core::tuning::Tuning::RON,
            sim::world::World::new(2, 2),
            2,
        ),
        gaia_reseat_skip: None,
        correction_audit: None,
        units: vec![],
        builds: vec![],
        region_map: vec![],
        notes: vec![],
        frame_seeds: vec![],
        rng_frames: vec![],
        frame_guys: vec![],
        frame_sites: vec![],
        type_index: vec![],
        unit_type_names: vec![],
        unit_tree: vec![],
        staged: None,
    }
}

/// Whether `built`, standing on frame 0, may be a trunk's root.
fn rides(built: &Built) -> bool {
    built.sim.frame == 0
        // A staged walk's state prints the whole install beside its script.
        && built.staged.is_none()
        && sharing()
}

/// `built`, a descendant of `root` by ticks alone, ticked to `to`: from the
/// highest rung past it and at or before `to`, when the trunk holds one.
fn ride(built: &mut Built, root: Root, to: i64) {
    let here = built.sim.frame;
    let rung = store(|s| {
        s.trunk(root.print)
            .rungs
            .iter()
            .rfind(|r| here < r.frame() && r.frame() <= to)
            .cloned()
    });
    let key = root.print;
    let Some(rung) = rung else {
        eprintln!(
            "trunk {:016x}/{}: a run-up from {here} to {to} found no rung",
            key.hash, key.len
        );
        walk(built, to, root);
        return;
    };
    let from = rung.frame();
    let (mut resumed, joined) = rung.carry(built, root.notes);
    walk(&mut resumed, to, root);
    store(|s| {
        s.saved += from - here;
        s.taken += 1;
    });
    let other = if joined {
        " (the game's other stand-up)"
    } else {
        ""
    };
    eprintln!(
        "trunk {:016x}/{}: a run-up from {here} to {to} resumed on {from}{other}",
        key.hash, key.len
    );
    let verified = verifying();
    if verified {
        while built.sim.frame < to {
            built.tick();
        }
        let (rode, fresh) = (print(&resumed), print(built));
        assert_eq!(
            rode, fresh,
            "the trunk's rung on {from} did not walk to the state a fresh walk reaches on {to}"
        );
        eprintln!("trunk: the rung on {from} matches a fresh walk on {to}");
    }
    tally_line(here, to, from, verified, joined);
    // The test's own state (verifying, its own walk) gives way to the
    // equal one; it answers for no ground of its own.
    std::mem::replace(built, resumed).retire();
}

/// **The silent run-up**: `built` ticked until it stands on `to`, exactly
/// as `while built.sim.frame < to { built.tick() }` leaves it. In a
/// group's process a state fresh from its stand-up takes the highest rung
/// an equal state left at or before `to`, and leaves its own beyond it.
pub(crate) fn advance(built: &mut Built, to: i64) {
    if to >= SHORT && rides(built) {
        let root = root(built);
        ride(built, root, to);
    }
    while built.sim.frame < to {
        built.tick();
    }
}

/// **A walk of several run-ups**: a state the rider owns, lent out between
/// them by shared reference only, so each run-up starts from the state the
/// last one left and the whole walk is one descent from its frame-0 print.
pub(crate) struct Rider {
    built: Built,
    root: Option<Root>,
}

impl Rider {
    pub(crate) fn new(built: Built) -> Self {
        Self { built, root: None }
    }

    /// [`advance`], from wherever the last run-up left the state.
    pub(crate) fn advance(&mut self, to: i64) {
        if self.root.is_none() && to >= SHORT && rides(&self.built) {
            self.root = Some(root(&self.built));
        }
        if let Some(root) = self.root {
            ride(&mut self.built, root, to);
        }
        while self.built.sim.frame < to {
            self.built.tick();
        }
    }

    pub(crate) fn state(&self) -> &Built {
        &self.built
    }
}

/// **A walk that ticks itself leaves rungs too** (the walk to the word,
/// `third::walk_word`). It compares every frame's draws and so resumes
/// nothing, but it stands up the state a closing state and a packet's walk
/// stand up — every stand-up through `build_sim` carries the phase trace,
/// so the word's walk is the plain walk with a reader beside it — and 112
/// closing states run next after it in their module. The walk promises
/// what [`advance`] does by construction: between the root and each call
/// of [`Leaves::passed`] it has done nothing to the state but tick it.
pub(crate) struct Leaves(Option<Root>);

impl Leaves {
    /// `built` as the walk is about to tick it for the first time.
    pub(crate) fn root(built: &Built) -> Self {
        Self(rides(built).then(|| root(built)))
    }

    /// After a tick: leave the rung the trunk wants on this frame.
    pub(crate) fn passed(&self, built: &Built) {
        let Some(root) = self.0 else {
            return;
        };
        let frame = built.sim.frame;
        if frame % CADENCE != 0 || !store(|s| s.trunk(root.print).wants(frame)) {
            return;
        }
        // Cloned outside the lock; a rung another thread left meanwhile
        // is found again under it.
        let mut rung = built.clone();
        store(|s| {
            let trunk = s.trunk(root.print);
            if trunk.wants(frame) {
                trunk.leave(Rung {
                    built: std::mem::replace(&mut rung, placeholder()),
                    notes: root.notes,
                });
            }
        });
        rung.retire();
    }
}

/// `RON_TRUNK_LOG=<directory>`: one line a rung taken — the test, the
/// frame the run-up stood on, the frame it reached, the rung's frame,
/// whether the verifier held it and whether the rung was the game's other
/// stand-up's — in a file a process. libtest keeps a
/// passing test's output, so this is how a run counts what its verifier
/// checked: a verifier that never fired reads as green too.
fn tally_line(here: i64, to: i64, from: i64, verified: bool, joined: bool) {
    use std::io::Write as _;
    let Some(dir) = std::env::var_os("RON_TRUNK_LOG") else {
        return;
    };
    let path = std::path::Path::new(&dir).join(format!("trunk-{}.tsv", std::process::id()));
    let test = std::thread::current()
        .name()
        .unwrap_or_default()
        .to_string();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .unwrap_or_else(|e| panic!("RON_TRUNK_LOG: {}: {e}", path.display()));
    writeln!(file, "{test}\t{here}\t{to}\t{from}\t{verified}\t{joined}").expect("a tally line");
}

/// `frames saved, rungs taken`, for the suite's log.
pub(crate) fn tally() -> (i64, usize) {
    store(|s| (s.saved, s.taken))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Forced;
    impl Forced {
        fn on(verify: bool) -> Self {
            FORCED.with(|f| f.set(Some(true)));
            VERIFYING.with(|v| v.set(verify));
            Self
        }
    }
    impl Drop for Forced {
        fn drop(&mut self) {
            FORCED.with(|f| f.set(None));
            VERIFYING.with(|v| v.set(false));
        }
    }

    /// An empty two-player world on `seed`: it ticks, and its print is its
    /// seed's.
    fn game(seed: u32) -> Built {
        let mut built = placeholder();
        built.sim.rng.seed = seed;
        built
    }

    fn plain(seed: u32, to: i64) -> Built {
        let mut built = game(seed);
        while built.sim.frame < to {
            built.tick();
        }
        built
    }

    fn rungs(root: Print) -> Vec<i64> {
        store(|s| {
            s.trunks
                .iter()
                .find(|t| t.root == root)
                .map_or(vec![], |t| t.rungs.iter().map(Rung::frame).collect())
        })
    }

    /// The store keeps [`TRUNKS`] trunks for the process: these tests hold
    /// it one at a time, so none evicts another's.
    static SERIAL: Mutex<()> = Mutex::new(());
    fn alone() -> std::sync::MutexGuard<'static, ()> {
        SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[test]
    fn a_run_up_resumes_the_rung_an_equal_state_left_and_reaches_the_plain_state() {
        let _alone = alone();
        let _on = Forced::on(false);
        let seed = 0x4220_0001;
        let root = root(&game(seed)).print;
        let mut first = game(seed);
        advance(&mut first, 700);
        assert_eq!(rungs(root), [256, 512]);
        assert_eq!(print(&first), print(&plain(seed, 700)));
        let before = tally();
        let mut second = game(seed);
        advance(&mut second, 600);
        assert_eq!(tally().0 - before.0, 512, "resumed on the rung at 512");
        assert_eq!(print(&second), print(&plain(seed, 600)));
        // A run-up to a rung's own frame takes it whole.
        let mut third = game(seed);
        advance(&mut third, 256);
        assert_eq!(print(&third), print(&plain(seed, 256)));
        // Another state rides no rung of this one.
        let before = tally();
        let mut other = game(seed ^ 0x1000);
        advance(&mut other, 600);
        assert_eq!(tally(), before, "another seed is another trunk");
        assert_eq!(print(&other), print(&plain(seed ^ 0x1000, 600)));
    }

    #[test]
    fn a_state_past_frame_zero_a_short_run_up_and_a_process_not_sharing_tick_plainly() {
        let _alone = alone();
        let seed = 0x4220_0002;
        let root = root(&game(seed)).print;
        // Not sharing: no rung is left.
        FORCED.with(|f| f.set(Some(false)));
        let mut off = game(seed);
        advance(&mut off, 600);
        FORCED.with(|f| f.set(None));
        assert_eq!(print(&off), print(&plain(seed, 600)));
        let _on = Forced::on(false);
        assert_eq!(rungs(root), [] as [i64; 0]);
        let mut short = game(seed);
        advance(&mut short, SHORT - 1);
        assert_eq!(short.sim.frame, SHORT - 1);
        assert_eq!(rungs(root), [] as [i64; 0]);
        // Past frame 0 the state is no root: it ticks, and leaves nothing.
        advance(&mut short, 600);
        assert_eq!(print(&short), print(&plain(seed, 600)));
        assert_eq!(rungs(root), [] as [i64; 0]);
    }

    #[test]
    fn a_trunk_past_its_rungs_doubles_its_cadence() {
        let _alone = alone();
        let _on = Forced::on(false);
        let seed = 0x4220_0003;
        let root = root(&game(seed)).print;
        let mut built = game(seed);
        advance(&mut built, CADENCE * (RUNGS as i64 + 1));
        let left = rungs(root);
        assert_eq!(
            left,
            (1..=6).map(|i| i * 2 * CADENCE).collect::<Vec<_>>(),
            "thirteen rungs thin to the six on the doubled cadence"
        );
        let mut again = game(seed);
        advance(&mut again, 2 * CADENCE + 10);
        assert_eq!(
            print(&again),
            print(&plain(seed, 2 * CADENCE + 10)),
            "a thinned trunk still resumes"
        );
    }

    #[test]
    fn a_rider_s_later_run_ups_take_the_rungs_past_where_it_stands() {
        let _alone = alone();
        let _on = Forced::on(true);
        let seed = 0x4220_0005;
        let mut first = game(seed);
        advance(&mut first, 1100);
        let before = tally();
        let mut rider = Rider::new(game(seed));
        for to in [300, 310, 900, 1100] {
            rider.advance(to);
            assert_eq!(print(rider.state()), print(&plain(seed, to)), "on {to}");
        }
        // 0 to 300 on the rung at 256; 300 to 310 ticked; 310 to 900 on
        // 768; 900 to 1100 on 1024.
        assert_eq!(
            (tally().0 - before.0, tally().1 - before.1),
            (256 + (768 - 310) + (1024 - 900), 3)
        );
        // A rider that starts with a short run-up is no root after it.
        let before = tally();
        let mut late = Rider::new(game(seed));
        late.advance(SHORT - 1);
        late.advance(900);
        assert_eq!(tally(), before);
        assert_eq!(print(late.state()), print(&plain(seed, 900)));
    }

    #[test]
    fn a_walk_that_ticks_itself_leaves_the_rungs_a_run_up_takes() {
        let _alone = alone();
        let _on = Forced::on(true);
        let seed = 0x4220_0006;
        let root = root(&game(seed)).print;
        let mut walked = game(seed);
        let leaves = Leaves::root(&walked);
        while walked.sim.frame < 600 {
            walked.tick();
            leaves.passed(&walked);
        }
        assert_eq!(rungs(root), [256, 512]);
        // A run-up from the same stand-up takes the rung, under the
        // verifier: the rung is the state its own ticks reach.
        let before = tally();
        let mut after = game(seed);
        advance(&mut after, 599);
        assert_eq!(tally().0 - before.0, 512);
        assert_eq!(print(&after), print(&plain(seed, 599)));
    }

    /// `game(seed)` as a stand-up that brought `notes`, `lengths` and the
    /// guys of `guys`' frames, with a word installed on frame 5.
    fn stood_up(seed: u32, notes: &[&str], lengths: &[((i32, i8), u32)], guys: &[i64]) -> Built {
        let mut built = game(seed);
        built.notes = notes.iter().map(|n| (*n).to_string()).collect();
        built.sim.roster.art.lengths = lengths.iter().copied().collect();
        // The word is any other than the seed's own; a hex literal here reads
        // as a document's constant built (`docs_guard`).
        built.frame_seeds = vec![(5, seed.rotate_left(7))];
        built.frame_guys = guys.iter().map(|&f| (f, vec![])).collect();
        built
    }

    fn stood_up_plain(
        seed: u32,
        notes: &[&str],
        lengths: &[((i32, i8), u32)],
        guys: &[i64],
        to: i64,
    ) -> Built {
        let mut built = stood_up(seed, notes, lengths, guys);
        while built.sim.frame < to {
            built.tick();
        }
        built
    }

    /// A stand-up's notes, animation lengths and the frames it holds guys on.
    type StandUp = (
        &'static [&'static str],
        &'static [((i32, i8), u32)],
        &'static [i64],
    );

    #[test]
    fn the_game_s_other_stand_up_rides_the_trunk_and_carries_its_own_fields() {
        let _alone = alone();
        let _on = Forced::on(true);
        let seed = 0x4224_0001;
        let whole: StandUp = (
            &["anim: 2 lengths"],
            &[((7, 1), 61), ((8, 2), 90)],
            &[5, 300],
        );
        let window: StandUp = (&["anim: 1 length", "another note"], &[((7, 1), 61)], &[5]);
        let mut first = stood_up(seed, whole.0, whole.1, whole.2);
        advance(&mut first, 700);
        assert_eq!(
            rungs(root(&first.clone()).print),
            [] as [i64; 0],
            "a key is a frame-0 state's"
        );
        // The window's stand-up: other notes, fewer lengths, no guys on an
        // unseeded frame; it takes the rung on 512 under the verifier and
        // stands where its own ticks would have, with its own fields.
        let before = tally();
        let mut second = stood_up(seed, window.0, window.1, window.2);
        advance(&mut second, 600);
        assert_eq!(
            tally().0 - before.0,
            512,
            "resumed on the other stand-up's rung"
        );
        let own = stood_up_plain(seed, window.0, window.1, window.2, 600);
        assert_eq!(print(&second), print(&own));
        assert_eq!(second.notes, own.notes);
        // Guys on a frame a word is installed on are read by the tick: a
        // stand-up that differs there is another trunk.
        let before = tally();
        let mut third = stood_up(seed, window.0, window.1, &[]);
        advance(&mut third, 600);
        assert_eq!(tally(), before, "the guys of a seeded frame are in the key");
        assert_eq!(
            print(&third),
            print(&stood_up_plain(seed, window.0, window.1, &[], 600))
        );
    }

    #[test]
    fn the_verifier_refuses_a_rung_that_is_not_the_plain_walk_s_state() {
        let _alone = alone();
        let seed = 0x4220_0004;
        let root = root(&game(seed)).print;
        {
            let _on = Forced::on(true);
            let mut first = game(seed);
            advance(&mut first, 300);
            // A sound rung passes under the verifier.
            let mut second = game(seed);
            advance(&mut second, 300);
            assert_eq!(print(&second), print(&plain(seed, 300)));
        }
        // Perturb the rung: one word of the stream.
        store(|s| s.trunk(root).rungs[0].built.sim.rng.seed ^= 1);
        let refused = std::panic::catch_unwind(|| {
            let _on = Forced::on(true);
            let mut third = game(seed);
            advance(&mut third, 300);
        });
        FORCED.with(|f| f.set(None));
        VERIFYING.with(|v| v.set(false));
        let message = refused.expect_err("a perturbed rung must be refused");
        let message = message
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default();
        assert!(
            message.contains("did not walk to the state a fresh walk reaches"),
            "{message}"
        );
    }
}
