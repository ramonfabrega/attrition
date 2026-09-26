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

use crate::economy::{Ledger, RESOURCES, Resource};
use crate::tuning::Tuning;

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
    }

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
    /// No free slot — or the University's six-scholar rule, which is the
    /// one special case `could_queue` carries and is not modelled here.
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
    }

    /// Appends a technology entry for tree id `t` and returns its slot.
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
    let tick = (time * t.research_tick_premium) >> 8;
    (tick * times.research_premium_time) >> 8
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
/// [`crate::cost::science_discount`].
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
    let base = base_time(t, times, researched);
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
        if !crate::build::is_training_building(&self.build_types, rec) {
            return false;
        }
        let researched = &self.muster[b.owner as usize].researched;
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

    /// **`Group::action_queue_up@006fdbb0`** for a train job — a unit
    /// type whose availability bit is set — on a group of buildings, from
    /// `CommandPackage::process_queue_up@00948230`. The members are first
    /// sorted by `queued`, least first (a selection sort from each live
    /// member's slot); then `num` times over, each live, finished member
    /// gets `Build::queue_up(type, 1)`, whose answer is not read. A
    /// missile silo asks `can_carry(type)` first; no silo is carried here.
    /// Returns the entries laid.
    ///
    /// SEAM: the other arm, a research entry (a unit type whose bit is
    /// clear, or a technology): `LeaderData::researching`, then one
    /// building with an empty queue first. No capture reaches it.
    pub fn action_queue_up(&mut self, buildings: &[usize], ty: usize, num: i32) -> usize {
        let who = match buildings.first() {
            Some(&b) => self.buildings[b].owner,
            None => return 0,
        };
        if !self.muster[who as usize].researched[ty] {
            return 0;
        }
        let mut list = buildings.to_vec();
        let live = |sim: &crate::Sim, b: usize| sim.buildings[b].alive && sim.buildings[b].active;
        for i in 0..list.len().saturating_sub(1) {
            if !live(self, list[i]) {
                continue;
            }
            for j in i + 1..list.len() {
                let queued = |b: usize| self.buildings[b].queue.items.len();
                if queued(list[j]) < queued(list[i]) {
                    list.swap(i, j);
                }
            }
        }
        let mut laid = 0;
        for _ in 0..num {
            for &b in &list {
                if live(self, b) && self.queue_up(b, ty).is_ok() {
                    laid += 1;
                }
            }
        }
        laid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> Tuning {
        Tuning::RON
    }

    fn charges(pairs: &[(Resource, i32)]) -> [i32; RESOURCES] {
        let mut out = [0; RESOURCES];
        for &(r, n) in pairs {
            out[r.index()] = n;
        }
        out
    }

    #[test]
    fn the_accelerators_all_ship_as_one_frame_per_call() {
        let t = tuning();
        for job in [Job::Construct, Job::Train, Job::Research] {
            assert_eq!(accel(&t, job, 1), TIME_SCALE);
        }
    }

    #[test]
    fn ai_speed_multiplies_the_accelerator_only_above_one() {
        let t = tuning();
        assert_eq!(accel(&t, Job::Train, 0), TIME_SCALE);
        assert_eq!(accel(&t, Job::Train, 1), TIME_SCALE);
        assert_eq!(accel(&t, Job::Train, 4), TIME_SCALE * 4);
    }

    #[test]
    fn a_citizen_takes_four_seconds_before_any_are_owned() {
        let t = tuning();
        // JOB_TIME 50 frames, UNIT_RATE_BASE 6/5 loaded as 120.
        let times = Times {
            job_time: 50,
            research_premium_time: 512,
            job_extra_time: 10,
        };
        let time = train_time(&t, &times, true, 0, &[]);
        assert_eq!(time, 6000);
        assert_eq!(time / TIME_SCALE, 60);
        assert_eq!(time / TIME_SCALE / crate::FRAMES_PER_SECOND, 4);
    }

    #[test]
    fn each_citizen_already_owned_adds_half_a_second() {
        let t = tuning();
        let times = Times {
            job_time: 50,
            research_premium_time: 512,
            job_extra_time: 10,
        };
        // JOB_EXTRA_TIME 10 times UNIT_RATE_PROGRESSION 75 is 750 hundredths.
        let none = train_time(&t, &times, true, 0, &[]);
        let one = train_time(&t, &times, true, 1, &[]);
        assert_eq!(one - none, 750);
        // Seven and a half frames. The comparison is in hundredths because
        // that is where the half is; dividing to frames first would lose it.
        assert_eq!((one - none) * 2, crate::FRAMES_PER_SECOND * TIME_SCALE);
    }

    #[test]
    fn the_time_ramp_triples_and_stops() {
        let t = tuning();
        let times = Times {
            job_time: 50,
            research_premium_time: 512,
            job_extra_time: 10,
        };
        let base = train_time(&t, &times, true, 0, &[]);
        assert_eq!(train_time(&t, &times, true, 16, &[]), base * RAMP_CEILING);
        assert_eq!(train_time(&t, &times, true, 200, &[]), base * RAMP_CEILING);
    }

    #[test]
    fn the_first_of_a_type_is_a_research_job_and_takes_twice_as_long() {
        let t = tuning();
        let times = Times {
            job_time: 50,
            research_premium_time: 512, // the `2` that 356 records carry
            job_extra_time: 10,
        };
        // Research skips the ramp, so compare against the unscaled base.
        assert_eq!(train_time(&t, &times, false, 0, &[]), 50 * TIME_SCALE * 2);
        // And it advances at the research accelerator, by the same bit.
        assert_eq!(Job::for_unit(false), Job::Research);
        assert_eq!(Job::for_unit(true), Job::Train);
    }

    #[test]
    fn an_entry_records_only_three_of_six_resources() {
        let c = charges(&[
            (Resource::Food, 10),
            (Resource::Timber, 20),
            (Resource::Wealth, 30),
            (Resource::Knowledge, 40),
        ]);
        let item = Item::queued(7, &c);
        let paid: Vec<_> = item.paid().collect();
        assert_eq!(
            paid,
            vec![
                (Resource::Food, 10),
                (Resource::Timber, 20),
                (Resource::Wealth, 30)
            ]
        );
        assert_eq!(item.good[2], Resource::Wealth.index() as i16);
    }

    #[test]
    fn zero_amounts_do_not_consume_a_pair() {
        let c = charges(&[
            (Resource::Food, 0),
            (Resource::Metal, 5),
            (Resource::Oil, 7),
        ]);
        let item = Item::queued(7, &c);
        let paid: Vec<_> = item.paid().collect();
        assert_eq!(paid, vec![(Resource::Metal, 5), (Resource::Oil, 7)]);
        assert_eq!(item.good[2], NO_GOOD);
    }

    #[test]
    fn done_is_read_before_the_increment() {
        let mut item = Item::default();
        // A target of two frames, one frame per call.
        let target = 200;
        assert!(!advance(&mut item, target, 100));
        assert_eq!(item.job_counter, 100);
        assert!(!advance(&mut item, target, 100));
        assert_eq!(item.job_counter, 200);
        // The counter is at the target, and only now does a call see it.
        assert!(advance(&mut item, target, 100));
    }

    #[test]
    fn the_counter_never_passes_the_target() {
        let mut item = Item::default();
        advance(&mut item, 150, 100);
        advance(&mut item, 150, 100);
        assert_eq!(item.job_counter, 150);
    }

    #[test]
    fn a_target_of_one_completes_on_the_first_call() {
        let mut item = Item::default();
        assert!(advance(&mut item, MIN_TIME, 100));
    }

    #[test]
    fn the_population_cap_stops_the_handover_not_the_clock() {
        // Progress runs to completion at the train accelerator regardless.
        let t = tuning();
        let times = Times {
            job_time: 1,
            research_premium_time: 512,
            job_extra_time: 10,
        };
        let target = train_time(&t, &times, true, 0, &[]);
        let mut item = Item::default();
        let accel = accel(&t, Job::Train, 1);
        while !advance(&mut item, target, accel) {}
        assert_eq!(item.job_counter, target);
        // And then stalls, at full progress, forever.
        assert_eq!(hand_over(10, 10, 1, false), Handover::Population);
        assert_eq!(hand_over(10, 9, 1, false), Handover::Trained);
    }

    #[test]
    fn a_limit_refuses_differently_from_the_cap() {
        assert_eq!(hand_over(10, 0, 1, true), Handover::Limit);
        // The cap is checked first, so it wins when both apply.
        assert_eq!(hand_over(0, 0, 1, true), Handover::Population);
    }

    #[test]
    fn cancelling_a_run_removes_the_last_of_it() {
        let mut q = Queue::new(8);
        let c = charges(&[(Resource::Food, 10)]);
        for _ in 0..3 {
            q.push(4, &c);
        }
        q.push(9, &c);
        // The one in progress is slot 0 and keeps its progress.
        q.items[0].job_counter = 4200;
        let mut ledger = Ledger::default();
        q.unqueue(0, true, &mut ledger);
        assert_eq!(q.items.len(), 3);
        assert_eq!(q.items[0].job_counter, 4200);
        assert_eq!(q.items.iter().map(|i| i.ty).collect::<Vec<_>>(), [4, 4, 9]);
    }

    #[test]
    fn cancelling_refunds_the_recorded_price_and_completing_does_not() {
        let mut q = Queue::new(8);
        let c = charges(&[(Resource::Food, 30), (Resource::Timber, 12)]);
        q.push(4, &c);
        q.push(4, &c);

        let mut ledger = Ledger::default();
        q.unqueue(0, true, &mut ledger);
        assert_eq!(ledger.bucket[Resource::Food.index()], 30);
        assert_eq!(ledger.bucket[Resource::Timber.index()], 12);

        q.unqueue(0, false, &mut ledger);
        assert_eq!(ledger.bucket[Resource::Food.index()], 30);
        assert_eq!(ledger.bucket[Resource::Timber.index()], 12);
        assert!(q.items.is_empty());
    }

    #[test]
    fn science_reprices_a_queued_item_in_place() {
        let t = tuning();
        let mut ledger = Ledger::default();
        let c = charges(&[(Resource::Knowledge, 100)]);
        let mut item = Item::queued(4, &c);

        // First level: nothing was discounted yet, so the base is the price.
        let back = reprice(&mut item, t.tech_science_discount, 0, &mut ledger);
        assert_eq!(back, 10);
        assert_eq!(ledger.bucket[Resource::Knowledge.index()], 10);
        assert_eq!(item.cost[0], 90);

        // Second level re-prices from the new stored amount, not the old one.
        let back = reprice(&mut item, t.tech_science_discount, 1, &mut ledger);
        assert_eq!(item.cost[0], 80);
        assert_eq!(back, 10);
    }

    #[test]
    fn the_science_speedup_is_run39_s_second_library_entry() {
        // run39's own clock. The AI queues Written Word and City State at
        // frame 2; both are `JOB_TIME 200`, so both start at 20,000
        // hundredths. Written Word is researched at Science 0 — its own level
        // is 0, and the gate is *strict* — so it takes the full 20,000 and
        // lands on 201. City State is then researched with `epoch[3]` at one
        // against its own level of zero, takes 18,000, and lands on 382. The
        // twenty frames between 382 and 402 were the whole of the twenty-one
        // fields `run39_s_build_queues_are_the_original_s_clock` disagreed on.
        let t = tuning();
        assert_eq!(
            science_speedup(&t, 0, 0),
            None,
            "level with it is not ahead"
        );
        let up = science_speedup(&t, 1, 0).expect("a level ahead");
        assert_eq!(up, Adjust::Off(10));
        assert_eq!(up.apply(20_000), 18_000);
        // Two levels ahead is twice off the *whole*, not a compounding.
        assert_eq!(science_speedup(&t, 2, 0).unwrap().apply(20_000), 16_000);
        // And being behind costs nothing in time, where on the price side the
        // same distance is a surcharge.
        assert_eq!(science_speedup(&t, 0, 3), None);
    }

    #[test]
    fn taking_a_percentage_off_is_not_scaling_by_its_complement() {
        // `Adjust::Off(pct)` truncates the term and subtracts it;
        // `Adjust::Scale(100 - pct)` truncates the product. The original
        // spells the science speedup the first way — a magic multiply by
        // −0x51eb851f added to the time — so the difference is not cosmetic.
        // Seven hundredths of a frame at ten percent: 7 − 0 = 7, against
        // 630 / 100 = 6.
        assert_eq!(Adjust::Off(10).apply(7), 7);
        assert_eq!(Adjust::Scale(90).apply(7), 6);
        // They agree wherever the product is a round hundred.
        for time in [0, 100, 1_000, 20_000] {
            assert_eq!(Adjust::Off(10).apply(time), Adjust::Scale(90).apply(time));
        }
    }

    #[test]
    fn the_magic_multiply_is_a_division_by_a_hundred() {
        for x in [0, 1, 99, 100, 250, 1000, 3200, 12345, 1_000_000] {
            assert_eq!(neg_hundredth(x), -(x / 100));
        }
    }

    #[test]
    fn a_research_entry_completes_without_asking_the_cap() {
        // The bit is clear, the player is over the cap — and the answer is
        // still positive, because the cap check is inside the bit-set branch.
        assert_eq!(finished(false, 0, 10, 1, true), Handover::Researched);
        assert_eq!(finished(true, 0, 10, 1, true), Handover::Population);
        assert_eq!(finished(true, 10, 0, 1, false), Handover::Trained);
    }

    #[test]
    fn a_stuck_head_lets_the_first_research_entry_through() {
        let mut q = Queue::new(8);
        let c = charges(&[(Resource::Food, 1)]);
        for ty in [1, 2, 3, 2, 4] {
            q.push(ty, &c);
        }
        let researched = [false, true, true, false, false];
        // Slot 0 is never a candidate, even if it is itself a research job.
        assert_eq!(q.next_research(|ty| !researched[ty]), Some(2));
        // Only train jobs behind the head: nothing gets through.
        assert_eq!(q.next_research(|_| false), None);
    }

    #[test]
    fn libraries_decide_how_many_slots_advance() {
        assert_eq!(parallel_slots(1, 5), 1);
        assert_eq!(parallel_slots(4, 5), 4);
        assert_eq!(parallel_slots(4, 2), 2);
        assert_eq!(parallel_slots(0, 3), 0);
    }

    #[test]
    fn a_queue_refuses_when_it_is_full() {
        let mut q = Queue::new(2);
        let c = charges(&[(Resource::Food, 1)]);
        assert!(q.has_room());
        q.push(1, &c);
        assert!(q.has_room());
        q.push(1, &c);
        assert!(!q.has_room());
    }
}

/// The infinite queue on a Sim (item 877, `docs/GOLDEN.md` §33).
#[cfg(test)]
mod infinite_tests {
    use crate::build::{BuildType, Ident, flags};
    use crate::economy::RESOURCES;
    use crate::{Sim, tech};

    /// A Barracks (a training building) and one unit type it trains.
    fn barracks(researched: bool) -> (Sim, usize, usize) {
        let mut w = crate::world::World::new(16, 16);
        w.fill_region(
            crate::world::Terrain::Land,
            crate::world::Cell::new(0, 0),
            crate::world::Cell::new(15, 15),
        );
        let mut sim = Sim::new(crate::Tuning::RON, w, 1);
        let mut tree = tech::TechTree::new();
        for n in ["Food", "Timber", "Wealth", "Knowledge", "Metal", "Oil"] {
            tree.add(tech::TypeDef::good(n));
        }
        let unit = tree.add(tech::TypeDef::unit("Bowmen", tech::UnitTraits::default()));
        let build = tree.add(tech::TypeDef::building("Barracks"));
        tree.types[unit].where_ = Some(build);
        tree.finalize();
        sim.set_tech_tree(tree);
        sim.tech[0].tech[unit] = true;
        sim.tech[0].tech[build] = true;
        let mut ty = crate::UnitType {
            tree: Some(unit),
            ..crate::UnitType::default()
        };
        ty.price.pop = 1;
        ty.price.base[crate::economy::Resource::Wealth.index()] = 50;
        let rec = sim.add_unit_type(ty);
        sim.muster[0].researched[rec] = researched;
        let brec = sim.build_types.len();
        sim.build_types.push(BuildType {
            ident: Ident::Barracks,
            tree: Some(build),
            flags: flags::TRAINS,
            x_size: 2,
            y_size: 2,
            hits: 100,
            ..BuildType::default()
        });
        let b = sim.add_building(0, crate::Pos::new(4 * 256, 4 * 256), 20);
        sim.buildings[b].ty = Some(brec);
        sim.muster[0].cap = 100;
        sim.ledgers[0].bucket = [10_000; RESOURCES];
        sim.holdings[0].available = [true; RESOURCES];
        (sim, b, rec)
    }

    /// **The button needs a train job** (`BuildData::can_infinite@
    /// 0062d4d0`, under the emulator): nothing on an empty queue or a
    /// research entry, a toggle on a train job, and the queue's emptying
    /// clears it. Made to fail with the gate answering for any queue.
    #[test]
    fn the_infinite_button_needs_a_train_job() {
        let (mut s, b, rec) = barracks(false);
        assert_eq!(s.action_buildmask(&[b], 0x40), 0, "an empty queue");
        s.queue_up(b, rec).expect("a research entry");
        assert_eq!(s.action_buildmask(&[b], 0x40), 0, "a research entry");
        let (mut s, b, rec) = barracks(true);
        s.queue_up(b, rec).expect("a train job");
        assert_eq!(s.action_buildmask(&[b], 0x40), 1);
        assert!(s.buildings[b].queue.infinite, "4096 -> 4160");
        assert_eq!(s.action_buildmask(&[b], 0x40), 1);
        assert!(!s.buildings[b].queue.infinite, "4160 -> 4096");
        s.action_buildmask(&[b], 0x40);
        let mut ledger = crate::economy::Ledger::default();
        s.buildings[b].queue.unqueue(0, false, &mut ledger);
        assert!(!s.buildings[b].queue.infinite, "the empty queue clears it");
    }

    /// **A train job finished under the bit re-queues itself, paid again,
    /// until the stockpile refuses** (`do_queue`'s `61ec24`, run285's 1060
    /// and 1272). Made to fail with the bit read after the unqueue, and
    /// with the re-queue skipped.
    #[test]
    fn a_finished_train_job_requeues_under_the_bit() {
        let (mut s, b, rec) = barracks(true);
        s.queue_up(b, rec).expect("queued");
        s.action_buildmask(&[b], 0x40);
        let finish = |s: &mut Sim| {
            let n = s.buildings[b].queue.items.len();
            for _ in 0..100_000 {
                let trained = !s.process_queues().is_empty();
                if trained || s.buildings[b].queue.items.len() != n {
                    return trained;
                }
            }
            panic!("the entry never finished");
        };
        assert!(finish(&mut s), "the first trains");
        let q = &s.buildings[b].queue;
        assert_eq!(q.items.len(), 1, "re-queued");
        assert_eq!(q.items[0].job_counter, 0);
        assert!(q.infinite, "the bit survives the empty queue's clear");
        assert_ne!(q.items[0].cost, [0; super::PAIRS], "paid again");
        // A stockpile short of the next price in every resource it costs.
        let price = s.price_of(0, rec);
        for (r, p) in price.iter().enumerate() {
            if *p > 0 {
                s.ledgers[0].bucket[r] = p - 1;
            }
        }
        assert!(finish(&mut s), "the second trains");
        let q = &s.buildings[b].queue;
        assert!(q.items.is_empty(), "the re-queue is refused");
        assert!(!q.infinite, "and the bit goes with it");
    }
}
