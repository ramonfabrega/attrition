//! Production: spending a price over time.
//!
//! `docs/PRODUCTION.md` is the specification. A queue is a fixed array of
//! twenty-byte records hanging off a building. Queueing charges the price
//! immediately and writes down what was charged; every frame a counter climbs
//! by a constant; when it reaches the item's time the building tries to hand
//! the item over, and if it cannot, the entry sits at full progress and tries
//! again next frame.
//!
//! Three things here are worth knowing before reading any of it.
//!
//! **The counter is in hundredths of a frame.** `TypeData::time` returns
//! `JOB_TIME * 100` and `ACCEL_TRAIN` ships as 100, so one call advances
//! exactly one frame's worth. Neither number says so; both are facts about
//! their loaders. See `docs/DECISIONS.md` entry 14.
//!
//! **An entry remembers three resources, not six.** The price has six slots
//! and the record has three pairs, so a cost in four resources is partly
//! forgotten — and the refund reads the record.
//!
//! **The population cap does not stop the clock.** It stops the handover, at
//! full progress, indefinitely.
//!
//! Nothing here is fractional. Every step is an integer multiply followed by a
//! divide by a hundred or a shift by eight, in the original's order.

use crate::core::tuning::Tuning;
use crate::economy::{Ledger, RESOURCES, Resource};

/// How many `(resource, amount)` pairs a queue entry can remember.
///
/// Three, against a price with six slots. `BuildQueue::set_queue` walks the
/// six, skips any whose amount is zero, and returns once it has written three.
pub const PAIRS: usize = 3;

/// The `good` value meaning "this pair is empty" — the original's `0xffff`
/// read back as a signed short.
pub const NO_GOOD: i16 = -1;

/// One queue entry. The original's `QueueItem`, whose every field the PDB
/// names:
///
/// ```text
/// +0x00  int       job_counter
/// +0x04  short     type
/// +0x06  short[3]  good
/// +0x0c  short[3]  cost
/// ```
///
/// `cost` is a `short` in the original and a `short` here. A charge above
/// 32767 in one resource would truncate on the way in and refund the truncated
/// amount; nothing in the shipped data comes near it, and narrowing it here is
/// how that stays true rather than becoming an assumption.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Item {
    /// Progress, in hundredths of a frame.
    pub job_counter: i32,
    /// Which unit type is being made — unused when `tech` is set.
    pub ty: usize,
    /// A technology entry: the tree id being researched. The original's
    /// `type` is one `TypeIndex` space over units and techs alike; the
    /// simulation keys unit types and the tree separately, so an entry says
    /// which it holds. A tech entry is a research job in every sense
    /// `docs/PRODUCTION.md` gives the word.
    pub tech: Option<usize>,
    /// **A `DISBAND` entry** (`TypeIndex` 0x29a, [`crate::cities::queue_disband::DISBAND`];
    /// item 3918, `docs/GOLDEN.md` §165): a spell the building casts on itself when
    /// the job completes (`Build::finished@00628490`'s spell arm). `ty` and `tech`
    /// are unused when it is set.
    pub disband: bool,
    /// Resource indices of what was paid, `NO_GOOD` for an empty pair.
    pub good: [i16; PAIRS],
    /// Amounts paid, aligned with `good`.
    pub cost: [i16; PAIRS],
}

impl Item {
    /// A fresh entry for `ty`, recording what `charges` actually took.
    ///
    /// This is `BuildQueue::set_queue` with its fourth argument zero: the
    /// counter is reset, the type is written, and the price is folded into at
    /// most three pairs.
    pub fn queued(ty: usize, charges: &[i32; RESOURCES]) -> Item {
        let mut item = Item {
            job_counter: 0,
            ty,
            tech: None,
            disband: false,
            good: [NO_GOOD; PAIRS],
            cost: [0; PAIRS],
        };
        let mut slot = 0;
        for r in Resource::ALL {
            if charges[r.index()] == 0 {
                continue;
            }
            item.good[slot] = r.index() as i16;
            item.cost[slot] = charges[r.index()] as i16;
            slot += 1;
            if slot >= PAIRS {
                break;
            }
        }
        item
    } // twin of BuildQueue::set_queue

    /// A fresh entry researching tree id `t`, recording what `charges` took.
    pub fn tech_queued(t: usize, charges: &[i32; RESOURCES]) -> Item {
        let mut item = Item::queued(0, charges);
        item.tech = Some(t);
        item
    }

    /// The pairs that name a resource, in the order the original stored them.
    pub fn paid(&self) -> impl Iterator<Item = (Resource, i32)> + '_ {
        (0..PAIRS).filter_map(|i| {
            let g = self.good[i];
            if g < 0 {
                return None;
            }
            Resource::ALL
                .get(g as usize)
                .map(|&r| (r, i32::from(self.cost[i])))
        })
    }
}

/// Why `Build::queue_up` refused — the original's `BuildData::queue_fail`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueFail {
    /// The price could not be paid.
    Cost,
    /// This building cannot make this type at all.
    CantTrain,
    /// No free slot, or the University's scholar admission limit.
    Full,
}

/// A building's production queue.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Queue {
    /// `BuildQueueData::queue_size`. Where the original's comes from is
    /// unread — see `docs/PRODUCTION.md` — so it is an input.
    pub capacity: usize,
    /// The live entries, in order. Length is the original's `queued`.
    pub items: Vec<Item>,
    /// **`WallData::build_masks & 0x40`, the infinite queue**
    /// (`docs/PRODUCTION.md`, "The infinite queue"; `docs/GOLDEN.md`
    /// §33). The player's button sets and clears it
    /// ([`Sim::action_buildmask`](crate::Sim::action_buildmask)), a train
    /// job that finishes under it re-queues itself
    /// ([`Sim::requeue_infinite`](crate::Sim::requeue_infinite)), and
    /// [`Queue::unqueue`] clears it when the queue empties. It lives on the
    /// queue because every writer but the button is the queue's own.
    pub infinite: bool,
}

impl Queue {
    pub fn new(capacity: usize) -> Queue {
        Queue {
            capacity,
            items: Vec::new(),
            infinite: false,
        }
    }

    /// Whether there is room — the capacity half of `BuildData::could_queue`.
    pub fn has_room(&self) -> bool {
        self.items.len() < self.capacity
    }

    /// Appends an entry for `ty` recording `charges`, and returns its slot.
    ///
    /// The caller has already charged the stockpile: `Type::pay_cost` both
    /// debits and reports, and this takes the report. The price is charged on
    /// queue, which is what makes the refund exact.
    pub fn push(&mut self, ty: usize, charges: &[i32; RESOURCES]) -> usize {
        self.items.push(Item::queued(ty, charges));
        self.items.len() - 1
    }

    /// The first slot behind the head holding a research job, if any —
    /// `BuildQueueData::get_next_non_unit`.
    ///
    /// This is what a stuck head lets through. When slot 0 is done and
    /// `Build::finished` refuses it, `do_queue` walks from slot 1 looking for
    /// the lowest entry that is not a unit type, or is a unit type whose
    /// availability bit is clear, and advances that one instead. `research`
    /// answers "is this type's entry a research job" — in a simulation with no
    /// technology types, that is `!researched[ty]`.
    pub fn next_research<F: Fn(usize) -> bool>(&self, research: F) -> Option<usize> {
        (1..self.items.len()).find(|&i| self.items[i].tech.is_some() || research(self.items[i].ty))
    } // twin of BuildQueueData::get_next_non_unit

    /// Appends a technology entry for tree id `t` and returns its slot.
    /// A `DISBAND` entry, which costs nothing (`craftrules.xml`'s `COST` and
    /// `COST2` are empty) — `BuildQueue::set_queue` for 0x29a.
    pub fn push_disband(&mut self) -> usize {
        let mut item = Item::queued(0, &[0; RESOURCES]);
        item.disband = true;
        self.items.push(item);
        self.items.len() - 1
    }

    /// How many entries are `DISBAND` — `BuildData::count_queue(1, 0x29a)`
    /// (vslot `400`), as `CityData::is_disbanding` asks it.
    pub fn count_disband(&self) -> i32 {
        self.items.iter().filter(|i| i.disband).count() as i32
    }

    pub fn push_tech(&mut self, t: usize, charges: &[i32; RESOURCES]) -> usize {
        self.items.push(Item::tech_queued(t, charges));
        self.items.len() - 1
    }

    /// How many entries research tree id `t` — `num_type_queued`'s line
    /// match, for a tech, is exact.
    pub fn count_tech(&self, t: usize) -> i32 {
        self.items.iter().filter(|i| i.tech == Some(t)).count() as i32
    }

    /// Which slot a cancel of `i` actually removes.
    ///
    /// With a refund — that is, a player cancelling rather than an item
    /// completing — `Build::unqueue` walks forward while the next entry has
    /// the same type. Cancelling one of a run of five identical items removes
    /// the fifth, so the one in progress keeps its progress.
    pub fn cancel_target(&self, i: usize) -> usize {
        let mut i = i;
        while i + 1 < self.items.len()
            && self.items[i].ty == self.items[i + 1].ty
            && self.items[i].tech == self.items[i + 1].tech
        {
            i += 1;
        }
        i
    }

    /// Removes slot `i`, refunding what it recorded if `refund`.
    ///
    /// `refund` is the original's second argument and does two things at once:
    /// it selects the skip-forward above, and it decides whether the price
    /// comes back. Completion passes false, so a finished item neither skips
    /// nor refunds.
    pub fn unqueue(&mut self, i: usize, refund: bool, ledger: &mut Ledger) -> Option<Item> {
        if i >= self.items.len() {
            return None;
        }
        let i = if refund { self.cancel_target(i) } else { i };
        if refund {
            unpay(&self.items[i], ledger);
        }
        let item = self.items.remove(i);
        // `Build::unqueue@006207c0`'s tail: `queued` reaching 0 clears
        // `build_masks & 0x40` (under the emulator, 4168 → 4104).
        if self.items.is_empty() {
            self.infinite = false;
        }
        Some(item)
    }
}

/// Puts a cancelled entry's recorded price back — `Build::unpay_cost`.
///
/// The recorded amount, not a recomputed one. A discount that arrived while
/// the item sat in the queue cannot be harvested by cancelling.
pub fn unpay(item: &Item, ledger: &mut Ledger) {
    for (r, amount) in item.paid() {
        ledger.bucket[r.index()] += amount;
    }
}

/// Which accelerator a queue entry advances at.
///
/// The choice is `Build::do_queue`'s and it turns on one bit: a unit type
/// whose availability bit is clear is a *research* job, and that same bit
/// picks the research time in [`base_time`]. The first one of a unit type you
/// build is researched; every one after is trained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Job {
    /// A building type, or a disband.
    Construct,
    /// A unit type already available.
    Train,
    /// A technology, or the first of a unit type.
    Research,
}

impl Job {
    /// The job a unit type is, given whether it has been researched.
    pub const fn for_unit(researched: bool) -> Job {
        if researched {
            Job::Train
        } else {
            Job::Research
        }
    }
}

/// Hundredths of a frame added per call — `ACCEL_*`, times `ai_speed`.
///
/// All three constants ship as `1/1` and load as 100, so one call is one
/// frame. They are debug knobs and the designers say so in the file.
/// `ai_speed` above one multiplies; it is a game-speed control and a cheat,
/// and `docs/MOVEMENT.md` flags the same global as a possible desync.
pub const fn accel(t: &Tuning, job: Job, ai_speed: i32) -> i32 {
    let base = match job {
        Job::Construct => t.accel_construct,
        Job::Train => t.accel_train,
        Job::Research => t.accel_research,
    };
    if ai_speed > 1 { base * ai_speed } else { base }
}

/// The time inputs a single type carries.
///
/// `job_extra_time` arrives already scaled by a hundred, because the unit-type
/// loader parses `1/10tsx` by hand into `(1 * 100) / 10`. It does not go
/// through `String::fraction` and it is not in `Constants::init`.
/// `research_premium_time` two integers away in the same struct *does* go
/// through `String::fraction(s, 0x100)`, so it is 8.8 and a written `2` is
/// 512. See `docs/PRODUCTION.md`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Times {
    /// `JOB_TIME`, in frames.
    pub job_time: i32,
    /// `RESEARCH_PREMIUM_TIME`, as 8.8.
    pub research_premium_time: i32,
    /// `JOB_EXTRA_TIME`, already multiplied by a hundred.
    pub job_extra_time: i32,
}

/// Hundredths of a frame per frame of `JOB_TIME`.
pub const TIME_SCALE: i32 = 100;

/// The base time before the ramp — `TypeData::time` or `::research_time`.
///
/// ```text
/// time          = job_time * 100
/// research_time = time * RESEARCH_TICK_PREMIUM >> 8 * research_premium_time >> 8
/// ```
///
/// Two 8.8 multiplies inside a quantity already scaled by a hundred. Both
/// premiums ship as `1/1` and `2` respectively, so researching a unit usually
/// costs twice its build time.
pub fn base_time(t: &Tuning, times: &Times, researched: bool) -> i32 {
    let time = times.job_time * TIME_SCALE;
    if researched {
        return time;
    }
    let tick = research_tick(time, t.research_tick_premium);
    (tick * times.research_premium_time) >> 8
}

/// `TypeData::research_time@006639a0`'s arithmetic on what `time` answered: `(time · premium) >> 8`, the
/// product wrapping (`imul`) and the shift **logical** (`shr`), so a negative product is a large positive
/// (item 2090's sweep holds it).
pub fn research_tick(time: i32, premium: i32) -> i32 {
    (time.wrapping_mul(premium) as u32 >> 8) as i32
}

/// `TypeData::time@00663f20`: a spell (`spell`, `TypeData::is_spell_type`) the leader does not hold (`held`,
/// `LeaderData::has_spell`, asked only of a spell) costs `res_time · 100`; anything else `job_time · 100`, the
/// multiplies wrapping (`imul`). Item 2090's sweep holds the arms and the ask.
pub fn time_by(spell: bool, held: impl FnOnce() -> bool, job_time: i32, res_time: i32) -> i32 {
    if spell && !held() {
        res_time.wrapping_mul(TIME_SCALE)
    } else {
        job_time.wrapping_mul(TIME_SCALE)
    }
} // twin of TypeData::time

/// `TypeData::research_time@006639a0`: a spell costs `res_time · 100`; anything else asks `time` (the record's
/// slot `+0x6c`, [`time_by`] for a type this crate prices) and takes [`research_tick`] of it with
/// `RESEARCH_TICK_PREMIUM`. Item 2090's sweep holds the arms and the ask.
pub fn research_time_by(
    spell: bool,
    time: impl FnOnce() -> i32,
    premium: i32,
    res_time: i32,
) -> i32 {
    if spell {
        res_time.wrapping_mul(TIME_SCALE)
    } else {
        research_tick(time(), premium)
    }
}

/// How much of the base a build time may reach. Three, for everything.
///
/// The price side has four ceilings picked by unit class; the time side has
/// this one. Build time at most triples.
pub const RAMP_CEILING: i32 = 3;

/// The ramp: base scaled, then one term per unit of that type already owned.
///
/// ```text
/// t   = UNIT_RATE_BASE * t / 100
/// cap = t * 3
/// t   = owned * job_extra_time * UNIT_RATE_PROGRESSION + t
/// t   = min(t, cap)
/// ```
///
/// `owned` is `LeaderData::num_units[type]`, the live count. The original
/// reaches it as `leader + 0x56fe + type * 2`, which is not a separate array:
/// `num_units` is at `+0x5762`, unit type ids start at `0x32`, and the
/// compiler folded the subtraction into the base pointer. That identifies one
/// of the two count arrays `docs/COSTS.md` listed as unknown.
///
/// The original guards both terms against being negative before comparing,
/// which is an overflow guard rather than a clamp; the guard is kept.
pub fn ramped(t: &Tuning, base: i32, owned: i32, job_extra_time: i32) -> i32 {
    let scaled = t.unit_rate_base * base / 100;
    let cap = scaled * RAMP_CEILING;
    let ramped = owned * job_extra_time * t.unit_rate_progression + scaled;
    if ramped < 0 || cap < 0 {
        return 0;
    }
    ramped.min(cap)
}

/// One step of `train_time`'s modifier tail.
///
/// About thirty of these follow the ramp — nations, wonders, governments,
/// technologies, rares, generals, the lobby's handicap. They are all one of
/// three shapes. Which ones apply depends on the tech tree, the wonder list
/// and the nation roster, none of which the simulation models yet, so the tail
/// is an input here for the same reason `Movement::speed` is one:
/// `docs/PRODUCTION.md` enumerates the predicates, and the arithmetic is
/// complete without them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Adjust {
    /// `t = t * 100 / (pct + 100)` — faster by `pct` percent.
    Faster(i32),
    /// `t = pct * t / 100` — a direct scale.
    Scale(i32),
    /// `t = t * num / den`, truncating toward zero. The original writes the
    /// three-quarters cases as a shift and they are non-negative throughout.
    Ratio(i32, i32),
    /// `t = t - pct * t / 100` — a subtraction, and **not** the same thing as
    /// `Scale(100 - pct)`: the original truncates the term and takes it away,
    /// where a scale truncates the product. The two differ by one wherever
    /// `pct * t` is not a multiple of a hundred. The science speedup is the
    /// tail's only step of this shape, and it is written as a divide by −100
    /// in the listing, which is where the shape shows.
    Off(i32),
}

impl Adjust {
    pub const fn apply(self, time: i32) -> i32 {
        match self {
            Adjust::Faster(pct) => time * 100 / (pct + 100),
            Adjust::Scale(pct) => pct * time / 100,
            Adjust::Ratio(num, den) => time * num / den,
            // Written the original's way: the same `-0x51eb851f` magic
            // multiply [`neg_hundredth`] names, added rather than subtracted.
            Adjust::Off(pct) => time + neg_hundredth(pct * time),
        }
    }
}

/// Applies the tail in order. Order matters and the caller owns it.
pub fn adjusted(time: i32, tail: &[Adjust]) -> i32 {
    tail.iter().fold(time, |t, a| a.apply(t))
}

/// The research block's science speedup — the `TECH_SCIENCE_SPEEDUP` step of
/// `ObjectData::train_time@006508c0`, and the time-side twin of
/// [`crate::economy::cost::science_discount`].
///
/// ```text
/// if level < science { t -= (science - level) * TECH_SCIENCE_SPEEDUP * t / 100 }
/// ```
///
/// Three things separate it from the price side, and each is read off the
/// listing rather than carried over from it:
///
/// - the constant is `TECH_SCIENCE_SPEEDUP`, a second `Tuning` entry that
///   merely happens to ship at the same ten;
/// - `level` is the technology's `AGE` column **raw**, with none of the price
///   side's plus-one for a plain tech — so a plain Ancient tech is already a
///   level behind the player's first Science epoch here, and level with it
///   there;
/// - it is gated on `level < science`, so falling behind costs *nothing* in
///   time where the price side turns the same expression into a surcharge.
///
/// For a **unit or building** research job the level is not the type's own —
/// types other than techs have no `AGE` — but its first prerequisite's:
/// `TypeData +0x30` is `preq[0]`, and a negative one reads as level zero. The
/// caller resolves that; this takes the answer.
///
/// It returns nothing when it does not fire, so a caller can splice it into a
/// tail, and it belongs to the **research** half of the tail's partition: a
/// train job jumps over this block entirely and must never be handed one.
pub fn science_speedup(t: &Tuning, science: i32, level: i32) -> Option<Adjust> {
    (level < science).then(|| Adjust::Off((science - level) * t.tech_science_speedup))
}

/// The floor `ObjectData::train_time` returns through. One hundredth of a
/// frame — never zero, so an item always takes at least one call.
pub const MIN_TIME: i32 = 1;

/// A whole `ObjectData::train_time` for a unit type.
///
/// Base, then — only when the type is already researched — the ramp, then the
/// tail, then the floor. The ramp sits inside the availability branch in the
/// original, so the *first* of a type is priced in time with no ramp at all.
pub fn train_time(t: &Tuning, times: &Times, researched: bool, owned: i32, tail: &[Adjust]) -> i32 {
    train_time_from(t, times, researched, owned, 0, tail)
}

/// [`train_time`] with the base shortened `base_faster` percent before the
/// ramp (`t = t * 100 / (pct + 100)`; zero is no arm) — the Uranium arm of
/// `ObjectData::train_time@006508c0:79`–`85`, which runs on the type's own
/// time ahead of everything else.
pub fn train_time_from(
    t: &Tuning,
    times: &Times,
    researched: bool,
    owned: i32,
    base_faster: i32,
    tail: &[Adjust],
) -> i32 {
    let mut base = base_time(t, times, researched);
    if base_faster != 0 {
        base = base * 100 / (base_faster + 100);
    }
    let time = if researched {
        ramped(t, base, owned, times.job_extra_time)
    } else {
        base
    };
    adjusted(time, tail).max(MIN_TIME)
}

/// Advances one entry by one call, and says whether it was already done.
///
/// ```text
/// done        = target <= job_counter      # the OLD counter
/// job_counter = min(job_counter + accel, target)
/// ```
///
/// The comparison reads the counter *before* the increment, so an item takes
/// one extra call past the one on which it first reaches its target: the call
/// that lands on the target sets it, and the next call observes it. A target
/// of exactly one is special-cased — the original substitutes 1 for the
/// counter — so such an item is done on its first call.
pub fn advance(item: &mut Item, target: i32, accel: i32) -> bool {
    let counter = if target == MIN_TIME {
        MIN_TIME
    } else {
        item.job_counter
    };
    let done = target <= counter;
    item.job_counter = (counter + accel).min(target);
    done
}

/// What happened when a finished entry was handed over — `Build::finished`,
/// whose caller reads the sign of the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handover {
    /// Positive. The unit exists; the entry is removed with no refund.
    Trained,
    /// Positive, by the other route. The entry was a research job — a unit
    /// type whose availability bit was clear — and it completed through
    /// `Leader::gain_tech`, which sets the bit and places **nothing**. The
    /// entry is removed with no refund, and the population cap, caravan and
    /// aircraft checks below were never consulted: they sit inside the
    /// bit-set branch. The player queues again for the first trained one.
    Researched,
    /// Zero. The population cap. Nothing happens, the entry keeps its full
    /// progress, and the whole attempt repeats next frame — so a queue at the
    /// cap stalls at a hundred percent, paid for, until room appears.
    Population,
    /// Negative. A caravan or aircraft limit. The entry also stays, but the
    /// building goes looking for a *different* slot to advance instead, so
    /// items behind it are not held up.
    Limit,
}

/// The handover test, in the original's order: population first, then limits.
///
/// The population comparison is `pop_cap < control + pop`, the same strict
/// form as `check_population` in `docs/COSTS.md` — a unit that lands you
/// exactly on the cap is allowed. This is the *train* half of
/// `Build::finished`; [`finished`] puts the research half in front of it.
pub const fn hand_over(cap: i32, control: i32, pop: i32, at_limit: bool) -> Handover {
    if cap < control + pop {
        return Handover::Population;
    }
    if at_limit {
        return Handover::Limit;
    }
    Handover::Trained
}

/// `Build::finished` for a unit-type entry, whole.
///
/// The availability bit is tested first. Clear, and the entry falls past the
/// training branch to `Leader::gain_tech`: the bit is set, nothing is placed,
/// the answer is positive. Set, and it is [`hand_over`]. (An earlier draft of
/// the simulation spawned a unit from the research entry too; the original
/// does not — see `docs/PRODUCTION.md`, "Completion".)
pub const fn finished(
    researched: bool,
    cap: i32,
    control: i32,
    pop: i32,
    at_limit: bool,
) -> Handover {
    if !researched {
        return Handover::Researched;
    }
    hand_over(cap, control, pop, at_limit)
}

/// How many of the **first library's** queue slots advance at once.
///
/// Inside its library branch — and only there — `Build::do_queue` recurses
/// into slot `i + 1` while `i + 1` is below both the live count and
/// `LeaderData::get_building_cities`, which counts cities holding a library.
/// Since every research job in the game is forwarded to the player's first
/// library, that count is the number of technologies that can progress
/// simultaneously. One library, one tech. Four libraries, four.
///
/// Every other building advances slot 0 only. (An earlier draft applied this
/// to every building, so a barracks with four library cities behind it trained
/// four at once; the recursion is under `is(LIBRARY)` in the decompile, and
/// `docs/audit/2026-08-20-production.md` settled the dropped argument from the
/// bytes.)
pub const fn parallel_slots(library_cities: usize, queued: usize) -> usize {
    if library_cities < queued {
        library_cities
    } else {
        queued
    }
}

/// The original's `x / 100` written as a magic multiply, reproduced as the
/// division it computes.
///
/// The compiler emits `mulhi(x, -0x51eb851f)`, an arithmetic shift by five and
/// a sign correction, which works out to exactly `-(x / 100)` truncating
/// toward zero. It is worth naming because the same sequence appears in
/// `train_time`'s science speedup, and because reading it as anything else
/// changes a refund.
const fn neg_hundredth(x: i32) -> i32 {
    -(x / 100)
}

/// Re-prices a queued entry after a science level, refunding the difference.
///
/// `Build::refund_cost`, whose sole caller is `Leader::gain_tech`. For each
/// recorded pair, with `levels` the number of science levels the player was
/// above the tech's own level *before* this one:
///
/// ```text
/// base = cost * 100 / (100 - TECH_SCIENCE_DISCOUNT * levels)
/// new  = base - (levels + 1) * TECH_SCIENCE_DISCOUNT * base / 100
/// stockpile += cost - new
/// cost = new
/// ```
///
/// The first line undoes the discount already baked into the recorded price
/// and the second applies one level more of it. The entry's stored amount is
/// updated in place, so a later cancellation refunds the new price and a
/// second science level re-prices from there.
///
/// This is the other half of the answer to `docs/COSTS.md`'s question about
/// discounts arriving mid-queue: cancelling cannot profit from one, but
/// sitting still does, automatically. Returns what was handed back.
pub fn reprice(item: &mut Item, discount: i32, levels: i32, ledger: &mut Ledger) -> i32 {
    let denominator = 100 - discount * levels;
    if denominator == 0 {
        return 0;
    }
    let mut refunded = 0;
    for slot in 0..PAIRS {
        let g = item.good[slot];
        if g < 0 {
            continue;
        }
        let Some(&r) = Resource::ALL.get(g as usize) else {
            continue;
        };
        let paid = i32::from(item.cost[slot]);
        let base = paid * 100 / denominator;
        let new = base + neg_hundredth((levels + 1) * discount * base);
        ledger.bucket[r.index()] += paid - new;
        refunded += paid - new;
        item.cost[slot] = new as i16;
    }
    refunded
}

/// The infinite queue and the player's queue-up (`docs/PRODUCTION.md`,
/// "The infinite queue"; `docs/GOLDEN.md` §33, run285).
impl crate::Sim {
    /// `BuildData::could_queue`'s capacity predicates (PRODUCTION,
    /// "Queueing charges the price", item 1445). The existing total matters:
    /// six admits one more, seven refuses. Include scholars walking here.
    pub(crate) fn unit_queue_has_room(&self, at: usize, ty: usize) -> bool {
        let b = &self.buildings[at];
        if !b.queue.has_room() {
            return false;
        }
        if self.building_ident(at) != crate::cities::build::Ident::University
            || self.tables.unit_types[ty].worker != crate::orders::Worker::Scholar
        {
            return true;
        }
        let queued = b
            .queue
            .items
            .iter()
            .filter(|q| {
                q.tech.is_none()
                    && self.tables.unit_types[q.ty].worker == crate::orders::Worker::Scholar
            })
            .count() as i32;
        queued + self.num_gatherers(at, false, false) <= 6
    }

    /// **`BuildData::can_infinite@0062d4d0`** — the gate
    /// `WallData::valid_buildmask@0063e2a0` asks for 0x40: a training
    /// building (`BuildTypeData::is_training_building`, `build_flags &
    /// 0x80000000`) with **a train job** in its queue, a unit type whose
    /// availability bit is set. A research entry, a tech entry and an
    /// empty queue answer 0 (under the emulator, and run285's 1302).
    pub fn can_infinite(&self, at: usize) -> bool {
        let b = &self.buildings[at];
        let Some(rec) = b.ty else {
            return false;
        };
        if !crate::cities::build::is_training_building(&self.tables.build_types, rec) {
            return false;
        }
        let researched = &self.roster.muster[b.owner as usize].researched;
        b.queue
            .items
            .iter()
            .any(|i| i.tech.is_none() && researched[i.ty])
    }

    /// **`Build::do_queue@0061e410`'s re-queue**, at `61ec24`: after a
    /// train job's `finished` answered > 0, the word is read before
    /// `unqueue(i, 0)` — which clears 0x40 when the queue empties — and
    /// with the bit read, `&= ~0x40`, `Build::queue_up@00620f40(type, 0)`,
    /// and `|= 0x40` on success. So the re-queue goes to the end, is paid
    /// again, and survives the empty queue's clear; a refusal leaves the
    /// bit off. `was` is the bit as read before the unqueue. run285: the
    /// Bowmen out on 1060, re-queued with 46 timber and 56 wealth, 4160;
    /// out again on 1272, refused on 19 wealth, 4096.
    pub fn requeue_infinite(&mut self, at: usize, ty: usize, was: bool) {
        if !was {
            return;
        }
        self.buildings[at].queue.infinite = false;
        if self.queue_up(at, ty).is_ok() {
            self.buildings[at].queue.infinite = true;
        }
    }

    /// **`Group::action_queue_up@006fdbb0`** on a group of buildings, from
    /// `CommandPackage::process_queue_up@00948230`, for a unit type: the
    /// crate's [`crate::groups::group_queue_up::action_queue_up_steps`] over this
    /// [`Sim`]. The members are first sorted by `queued`, least first. Then
    /// the type picks the arm:
    ///
    /// - **a train job**, a unit type whose availability bit is set: `num`
    ///   times over, each live, finished member gets `Build::queue_up(type,
    ///   1)`, whose answer is not read. A Missile Silo (`is(0x208)`) asks
    ///   `can_carry(type)` first and is skipped on a refusal
    ///   ([`Sim::silo_takes`], item 1019).
    /// - **a research job**, a unit type whose bit is clear: the research
    ///   arm, once whatever `num` says, behind
    ///   `LeaderData::researching@006db510` over every unit line
    ///   ([`Sim::researching_unit`]): two passes over the sorted members,
    ///   the first only those holding nothing, and **the first `queue_up`
    ///   that answers 0 ends the command**; a refusal goes on to the next.
    ///
    /// Returns the entries laid.
    pub fn action_queue_up(&mut self, buildings: &[usize], ty: usize, num: i32) -> usize {
        self.queue_group(
            buildings,
            crate::groups::group_queue_up::Queued::Unit(ty),
            ty,
            num,
        )
    }

    /// **`Group::action_queue_up@006fdbb0`** for a technology: always the
    /// research arm. `LeaderData::researching(t, −1, 0, 0)` first — a tech
    /// queued at any of the player's buildings, in the group or not,
    /// refuses the command whole (the console's player hears
    /// `add_feedback`, and nothing else is written) — then the arm with
    /// [`Sim::queue_tech`]. `num` is not read: the arm reuses its slot as
    /// the pass counter (`6fe08a`). Returns the entries laid, 0 or 1.
    pub fn action_queue_research(&mut self, buildings: &[usize], t: crate::tech::TypeId) -> usize {
        self.queue_group(
            buildings,
            crate::groups::group_queue_up::Queued::Tech(t),
            0,
            0,
        )
    }

    pub(crate) fn queue_group(
        &mut self,
        buildings: &[usize],
        what: crate::groups::group_queue_up::Queued,
        ty: usize,
        num: i32,
    ) -> usize {
        let who = match buildings.first() {
            Some(&b) => self.buildings[b].owner,
            None => return 0,
        };
        let mut view = crate::groups::group_queue_up::SimQueueUp {
            sim: self,
            who,
            list: buildings,
            what,
        };
        crate::groups::group_queue_up::action_queue_up_steps(
            &mut view,
            i32::try_from(ty).unwrap_or(0),
            num,
        )
    }

    /// **`LeaderData::researching@006db510`** for a unit type: an entry of
    /// the type in the queue of any of the player's active buildings, or —
    /// the unit arm, with `param_4` 0 — an entry of a unit type `u` with
    /// neither `u` nor the type's own bit set and the type `is(u, 0)`, its
    /// lineage (`6db5d9`..`6db62c`). A technology is
    /// [`Sim::researching`]'s equality alone.
    pub fn researching_unit(&self, who: crate::world::Player, ty: usize) -> bool {
        let researched = &self.roster.muster[who as usize].researched;
        let tree = self.tables.unit_types[ty].tree;
        self.buildings
            .iter()
            .filter(|b| b.owner == who && b.active)
            .flat_map(|b| b.queue.items.iter())
            .filter(|i| i.tech.is_none())
            .any(|i| {
                i.ty == ty
                    || (!researched[i.ty]
                        && !researched[ty]
                        && tree
                            .zip(self.tables.unit_types[i.ty].tree)
                            .is_some_and(|(t, u)| self.tables.tech_tree.is(t, u, false)))
            })
    }

    /// **`Build::action_unqueue@00620280(p)`**, the player's cancel, from
    /// `CommandPackage::process_unqueue@009466f0` (`docs/PRODUCTION.md`,
    /// "The player's cancel"; `docs/GOLDEN.md` §34). `p` is the command's
    /// selector, read off the listing (`6203c1`..`620470`) and run under
    /// the emulator:
    ///
    /// - an empty queue: nothing, **whatever the bit** (`620353`);
    /// - with `build_masks & 0x40` set: the bit is cleared first, and for
    ///   `p >= -1` that is all — **a single cancel on an infinite queue
    ///   removes nothing** (`6203c1`);
    /// - `p <= -10`: `unqueue(queued - 1, 1)` until the queue is empty;
    /// - `-10 < p <= -5`: the same, `min(queued, 5)` times;
    /// - otherwise `unqueue(p, 1)`, with a negative `p` read as the last
    ///   slot, `queued - 1` (`62045d`). A slot past the end is nothing.
    ///
    /// Each `unqueue` with its refund walks forward over a run of the
    /// entry's type and refunds the recorded price ([`Sim::cancel`]).
    /// Returns the entries removed.
    ///
    /// The forward to the first library skips a slot holding `DISBAND`
    /// (0x29a): the loop's `bVar3` reads slot `max(p, 0)` at the building
    /// pressed ([`Sim::unqueue_home`]; item 3989, `docs/GOLDEN.md` §182).
    pub fn action_unqueue(&mut self, at: usize, p: i32) -> usize {
        let at = self.unqueue_home(at, p.max(0) as usize);
        let len = |sim: &crate::Sim| sim.buildings[at].queue.items.len();
        if len(self) == 0 {
            return 0;
        }
        if self.buildings[at].queue.infinite {
            self.buildings[at].queue.infinite = false;
            if p >= -1 {
                return 0;
            }
        }
        let times = if p <= -10 {
            len(self)
        } else if p <= -5 {
            len(self).min(5)
        } else {
            let slot = if p < 0 { len(self) - 1 } else { p as usize };
            return usize::from(self.cancel(at, slot).is_some());
        };
        let mut removed = 0;
        for _ in 0..times {
            if len(self) == 0 {
                break;
            }
            let last = len(self) - 1;
            removed += usize::from(self.cancel(at, last).is_some());
        }
        removed
    }
}

#[cfg(test)]
mod tests;

/// The infinite queue on a Sim (item 877, `docs/GOLDEN.md` §33).
#[cfg(test)]
mod infinite_tests;
