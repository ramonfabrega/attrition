//! The on-map heals of `Unit::process_healing@005e0670` — the civilian
//! heal (`docs/COMBAT.md` §72) and the CEO's (`docs/AI.md` §348.1).
//!
//! The function's `inside_up ≥ 0` branch is the garrison heal
//! ([`crate::Sim::garrison_heal`], `docs/CITIES.md` §6.7). Its on-map
//! branch runs up to seven heals one after another, each phased on
//! `(o + frame) % period` and each ending in `repair_damage`. Six of them
//! need a nation bonus, a wonder, a general, a Conquer-the-World hero or
//! a supply period that ships as zero (`docs/SUPPLY.md`, "Consumer 2");
//! the seventh, the **civilian heal**, needs none of these, and it is the
//! one a capture reaches. The supply heal (item 3978, `docs/AI.md` §410)
//! is built beside it: the French bonus and Versailles give it a period.

use crate::Sim;
use crate::ai_load::uflags2;
use crate::attrition::Domain;
use crate::world::Owner;

/// `ObjectData::is_worker@0046fa10`: the four citizen and scholar type
/// ids, by identity.
const WORKERS: std::ops::RangeInclusive<i32> = 0x32..=0x35;

/// The `FISHERMEN` type id, tested by identity here (`+4 != 0x13d`), not
/// by lineage.
const FISHERMEN: i32 = 0x13d;

/// `UnitData::unit_masks2` bit 0 — the unit was struck lately. It vetoes
/// the civilian heal (and forces a siege unit's slow reload,
/// `UnitData::recharge@0060fdf0`). `Unit::process@00610bc0`'s 32-frame
/// block decays it (`0x2` first, then `0x1`). **No setter is carried
/// here**: the decompile and a listing grep name none, and no dumped
/// unit on run356 or run373 holds `0x1` or `0x2` (`docs/COMBAT.md` §72.6).
pub const STRUCK: u32 = 0x1;

impl Sim {
    /// The civilian heal, the last arm of `process_healing`'s on-map
    /// branch (`005e0670`, `docs/COMBAT.md` §72.2):
    ///
    /// ```text
    /// captain, not air, has_damage(1), inside_up < 0     # the head
    /// domain == SEA                    → the ship heal     # not built
    /// is_supply                        → return
    /// …six heals that need a bonus, a wonder or a hero…    # not built
    /// CIVILIAN_HEAL_RATE != 0 and (o + frame) % rate == 0
    ///   and !(unit_masks2 & 1)
    ///   and (is_worker or is_caravan or is_merchant or type == 0x13d)
    ///   and the cell's owner is an ally:
    ///     repair_damage(1, 0, 1); healing = max(rate, healing)
    /// ```
    ///
    /// `repair_damage` with a whole point of damage takes one off and
    /// zeroes `damage_frac` (`+0x3b`); with none it leaves the fraction
    /// alone. The second argument 0 is the no-sparkle arm, so the heal
    /// draws nothing. `healing` is written by [`Sim::mark_healed`] (item
    /// 3961).
    /// **The CEO's heal** — the third of `process_healing`'s three
    /// economic-patriot arms (`005e0670:149`–`:232`, item 3686,
    /// `docs/AI.md` §348):
    ///
    /// ```text
    /// captain, not air, has_damage(1), inside_up < 0     # the head
    /// domain == SEA → the ship heal;  is_supply → return
    /// FRENCH_SUPPLY_HEAL_RATE != 0 and (o + frame) % rate == 0:
    ///   …the Senator's (0x161) and the President's (0x163) arms…
    ///   leader counts a 0x165:
    ///     unit_masks & 0x80 and !(unit_masks2 & 0x40000) and !is_supply
    ///                                         → skip to the next heal
    ///     has_general(0, 0x165) → repair_damage(1, 1, 1)
    /// ```
    ///
    /// Unlike the other two it asks nothing of the cell's owner, so a CEO
    /// heals its army abroad. `unit_masks & 0x80` is the period
    /// `Unit::process_attrition` assigns (it clears the bit on entry and
    /// sets it with the period), so it is read here as
    /// [`crate::Unit::attrition`]. The sparkle arm (the second argument)
    /// draws `internal_random` only for the console's allies, which is no
    /// stream this crate keeps; the third is the Conquer-the-World bonus.
    /// `healing` (`+0x38`) is [`Sim::mark_healed`] (item 3961); the
    /// `0x4000` clear is not carried. The Senator's arm is
    /// [`Self::senator_heal`].
    ///
    /// SEAM: the President's arm (`0x163`, `:179`–`:203`), which heals on
    /// an allied cell **or** one nobody owns; read, not built (parked
    /// 3686a, which named it the Comrade's).
    pub(crate) fn ceo_heal(&mut self, i: usize, frame: i64) {
        const THECEO: i32 = 0x165;
        let rate = i64::from(self.tuning.french_supply_heal_rate);
        if rate == 0 || (i64::from(self.units[i].index) + frame) % rate != 0 {
            return;
        }
        let u = &self.units[i];
        if !u.alive() || !u.on_map || u.inside.is_some() || u.inside_unit.is_some() {
            return;
        }
        if !u.captain || !matches!(u.kind.domain, Domain::Land) {
            return;
        }
        let Some(t) = u.ty else {
            return;
        };
        let squad = self.squad_members(i);
        let damaged = squad.iter().any(|&f| {
            let f = &self.units[f];
            f.alive() && (f.health < f.max_health || f.damage_frac != 0)
        });
        if !damaged || self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO) {
            return;
        }
        if u.attrition != 0 && u.unit_masks2 & 0x40000 == 0 {
            return;
        }
        if !self.has_general(i, THECEO) {
            return;
        }
        self.repair_unit_damage(i, 1);
        self.mark_healed(i, rate);
    }

    /// **The supply heal** — `process_healing`'s on-map arm after the
    /// three economic patriots' (`005e0670`, `LAB_005e0f1a`; item 3978,
    /// `docs/AI.md` §410):
    ///
    /// ```text
    /// captain, not air, has_damage(1), inside_up < 0     # the head
    /// domain == SEA → the ship heal;  is_supply → return
    /// rate = SUPPLY_HEAL_RATE
    ///   (+ FRENCH_SUPPLY_HEAL_RATE with has_tribe_bonus(10))
    /// has_wonder(VERSAILLES) and VERSAILLES_SUPPLY_HEAL_RATE != 0:
    ///   rate = rate == 0 ? VERSAILLES : (VERSAILLES + rate) / 4
    /// rate != 0 and (o + frame) % rate == 0
    ///   and Supplies::find_supply(x, y, who) >= 0:
    ///     repair_damage(1, 1, 1); healing = max(rate, healing)
    /// ```
    ///
    /// The period is [`crate::supply::heal_period`]; `find_supply` walks
    /// the player's supply list alone — a source alive and on the map
    /// within `SupplyData::get_radius × 0xc0` by `vector_dist` — so a
    /// supplying general's aura, which [`Sim::supplied_at`] reads for
    /// attrition, heals nobody here. It asks nothing of the cell's owner
    /// or the unit's motion. run1542's French Elite Special Forces `1/17`
    /// (TI 77), walking off under an `EXPLORETOORDER`, is 2648 from its
    /// Despot `1/164` (TI 352, `supply 0`) on block 1151 — inside 14
    /// tiles' 2688 — and prints `healing 13`: 20 set on 1143, `(17 +
    /// 1143) % 20 == 0`.
    ///
    /// The sparkle arm draws only for the console's allies, which is no
    /// stream this crate keeps; the `0x4000` clear is not carried.
    pub(crate) fn supply_heal(&mut self, i: usize, frame: i64) {
        let u = &self.units[i];
        if !u.alive() || !u.on_map || u.inside.is_some() || u.inside_unit.is_some() {
            return;
        }
        if !u.captain || !matches!(u.kind.domain, Domain::Land) {
            return;
        }
        let Some(t) = u.ty else {
            return;
        };
        let owner = u.owner;
        let french = self.tech.get(owner as usize).is_some_and(|p| {
            self.tech_tree
                .has_tribe_bonus(&self.setup, p, crate::ai::tribe::FRENCH)
        });
        let versailles = self.wonders_held(owner) & (1 << crate::tech::wonder::VERSAILLES) != 0;
        let rate = i64::from(crate::supply::heal_period(&self.tuning, french, versailles));
        if rate == 0 || (i64::from(self.units[i].index) + frame) % rate != 0 {
            return;
        }
        let damaged = self.squad_members(i).iter().any(|&f| {
            let f = &self.units[f];
            f.alive() && (f.health < f.max_health || f.damage_frac != 0)
        });
        if !damaged || self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO) {
            return;
        }
        let at = self.units[i].pos;
        let upgrades = self.supply_upgrade_level(owner);
        let terra_cotta = self.supply[owner as usize].player.terra_cotta;
        let radius = crate::supply::supply_radius(&self.tuning, upgrades, terra_cotta);
        let found = self.supply[owner as usize].list.find(at, radius, |w| {
            self.units.get(w).map(|w| crate::supply::Wagon {
                pos: w.pos,
                active: w.alive(),
                on_map: w.on_map,
            })
        });
        if found.is_some() {
            self.repair_unit_damage(i, 1);
            self.mark_healed(i, rate);
        }
    }

    /// **`Build::process@0061edf0`'s patriot repair** (`0061f2a0`..
    /// `LAB_0061f3b9`, item 3987, `docs/GOLDEN.md` §180): after the tower's
    /// attack and before `do_queue`, an active building of a leader who
    /// counts a Senator, a President or a CEO (`+0x59c0`, `+0x59c4`,
    /// `+0x59c8`: `+0x56fe + ti × 2` at `0x161`, `0x163`, `0x165`), damaged
    /// (`damage` or `damage_frac` non-zero), on `FRENCH_SUPPLY_HEAL_RATE`'s
    /// phase `(o + frame) % rate == 0`, takes one point:
    ///
    /// ```text
    /// t = owner of the cell under the building (−1 for none)
    /// if has_general(0, 0x163) >= 0:            # a President's reach
    ///     repair if t < 0 or is_ally(t)
    /// elif t >= 0 and is_ally(t):
    ///     repair if has_general(0, 0x161) >= 0 or has_general(0, 0x165) >= 0
    /// repair: vslot +0x168(1, 0, 1) = Build::repair_damage;
    ///         healing = max(rate, healing)
    /// ```
    ///
    /// `has_general` on a building is `find_hero` with the footprint's
    /// `(x_size + y_size) × 0x60` as slack, behind the leader's count of
    /// the type ([`Sim::hero_near`]). A President on an enemy's ground
    /// repairs nothing, and neither does a Senator or a CEO off allied
    /// ground: run1544's Barracks `0/2007`, struck beside its Senator, is
    /// never repaired and prints `healing 0` on every block.
    pub(crate) fn patriot_repair(&mut self, b: usize, frame: i64) {
        const THESENATOR: i32 = 0x161;
        const THEPRESIDENT: i32 = 0x163;
        const THECEO: i32 = 0x165;
        let bd = &self.buildings[b];
        let who = bd.owner;
        if ![THESENATOR, THEPRESIDENT, THECEO]
            .iter()
            .any(|&ti| self.hero_counted(who, ti))
        {
            return;
        }
        if bd.damage == 0 && bd.damage_frac == 0 {
            return;
        }
        let rate = i64::from(self.tuning.french_supply_heal_rate);
        if rate == 0 || bd.phase(frame) % rate != 0 {
            return;
        }
        let Some(ty) = bd.ty else {
            return;
        };
        let at = bd.pos;
        let reach = (self.build_types[ty].x_size + self.build_types[ty].y_size) * 0x60;
        let ally_ground = match self.world.owner_at(at) {
            crate::Owner::Player(t) => Some(self.is_ally(who, t)),
            _ => None,
        };
        let repair = if self.hero_near(who, THEPRESIDENT, at, reach) {
            ally_ground != Some(false)
        } else {
            ally_ground == Some(true)
                && (self.hero_near(who, THESENATOR, at, reach)
                    || self.hero_near(who, THECEO, at, reach))
        };
        if repair {
            self.repair_building(b, 1);
            let rate = i16::try_from(rate).unwrap_or(i16::MAX);
            let bd = &mut self.buildings[b];
            bd.healing = bd.healing.max(rate);
        }
    }

    /// `healing = max(rate, healing)`, the write every arm of
    /// `Unit::process_healing@005e0670` makes after its `repair_damage`,
    /// with the arm's own period (item 3961, `docs/AI.md` §405).
    pub(crate) fn mark_healed(&mut self, i: usize, rate: i64) {
        let rate = i16::try_from(rate).unwrap_or(i16::MAX);
        let u = &mut self.units[i];
        u.healing = u.healing.max(rate);
    }

    /// **The Senator's heal** — the first of `process_healing`'s three
    /// economic-patriot arms (`005e0670:149`–`:177`, item 3959,
    /// `docs/AI.md` §403), on the CEO's phase:
    ///
    /// ```text
    /// captain, not air, has_damage(1), inside_up < 0     # the head
    /// domain == SEA → the ship heal;  is_supply → return
    /// FRENCH_SUPPLY_HEAL_RATE != 0 and (o + frame) % rate == 0:
    ///   leader counts a 0x161 (THESENATOR)
    ///     and the cell's owner is an ally
    ///     and has_general(0, 0x161)  → repair_damage(1, 1, 1)
    /// ```
    ///
    /// It asks nothing of the unit's motion, so a column walking home
    /// under the Senator's radius heals on its phase: run1228b's Mohawk
    /// Spearmen captain `1/50`, under an `ATTACKTOORDER` with player 1's
    /// Senator `1/92` (TI 353) within reach, takes `healing 20` on blocks
    /// 8191, 8211 and 8231 and hands each point down to `1/52` (42 → 41 →
    /// 40 → 39); the Javelineers' `1/30` on 8411 (5 → 4). `0x161` is
    /// THESENATOR (the TypeIndex enum), the Republic's patriot, which
    /// parked 3686a called the Monarch's. The leader's count is answered
    /// by `has_general`'s walk, which finds no Senator without one.
    pub(crate) fn senator_heal(&mut self, i: usize, frame: i64) {
        const THESENATOR: i32 = 0x161;
        let rate = i64::from(self.tuning.french_supply_heal_rate);
        if rate == 0 || (i64::from(self.units[i].index) + frame) % rate != 0 {
            return;
        }
        let u = &self.units[i];
        if !u.alive() || !u.on_map || u.inside.is_some() || u.inside_unit.is_some() {
            return;
        }
        if !u.captain || !matches!(u.kind.domain, Domain::Land) {
            return;
        }
        let Some(t) = u.ty else {
            return;
        };
        let owner = u.owner;
        let damaged = self.squad_members(i).iter().any(|&f| {
            let f = &self.units[f];
            f.alive() && (f.health < f.max_health || f.damage_frac != 0)
        });
        if !damaged || self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO) {
            return;
        }
        let ally = match self.world.owner_at(self.units[i].pos) {
            Owner::Player(p) => self.is_ally(owner, p),
            _ => false,
        };
        if ally && self.has_general(i, THESENATOR) {
            self.repair_unit_damage(i, 1);
            self.mark_healed(i, rate);
        }
    }

    /// **`Unit::repair_damage@0060de10`** on a figure, the whole-point
    /// half: the figure's own damage first — `amount` off it, floored at
    /// none, and its fraction (`+0x3b`) zeroed when it had any — and what
    /// is left over, `amount − damage`, handed down `o_down` to the next
    /// live figure with the same call. A figure with no damage hands the
    /// whole of it on, so a captain's heal reaches the hurt figure below.
    ///
    /// **Past the chain's last live figure the excess regrows one**
    /// ([`Sim::regrow_figure`], item 3993, `docs/GOLDEN.md` §186): the
    /// listing's `o_down` arm (`:88`–`:100`) recurses only while the next
    /// figure is alive, and the dead tail `Unit::close` relinks there is
    /// the slot the new figure takes.
    pub(crate) fn repair_unit_damage(&mut self, i: usize, amount: i32) {
        if !self.units[i].alive() {
            return;
        }
        let mut cur = i;
        let mut left = amount;
        loop {
            let u = &mut self.units[cur];
            let damage = u.max_health - u.health;
            let rest = left - damage;
            if damage != 0 {
                u.health += left.min(damage);
                u.damage_frac = 0;
            }
            if rest <= 0 {
                return;
            }
            match self.units[cur].o_down {
                Some(d) if self.units[d].alive() => {
                    cur = d;
                    left = rest;
                }
                down => {
                    self.regrow_figure(cur, down, rest);
                    return;
                }
            }
        }
    }

    /// `Unit::repair_damage@0060de10:101`–`:217`, on the squad's last live
    /// figure `tail` with `left` points of heal still to give (item 3993,
    /// `docs/GOLDEN.md` §186):
    ///
    /// ```text
    /// uber_size (+0x308) > curr_uber_size(this, 0)
    ///   and (inside_up >= 0 or !(unit_masks & 4)):
    ///     n = Objects::find_free(who, 0, 2000, &mark, o_down)  # the dead tail's number
    ///     n.init(who, type, n, x, y)                            # Guy::init_real's draw
    ///     n.o_up, n.o_down = the dead tail's                    # restored over init's −1s
    ///     n.damage = hits / uber_size;  n.stance, n.unit_masks = this's
    ///     inside: insert_inside(n, host)  else: find_nearby_spot, add_move_order
    ///     console selection, this->group, the hotkey groups:
    ///       member(this.o) → kill(this.o, who, 0, 0); add(this.o, who, 0, 0)
    ///     n.repair_damage(left, sparkle, 0)
    /// ```
    ///
    /// `find_free` with a number asked for returns that number (`0065ad60`,
    /// the `param_5 >= 0` arm), so the figure is born into the dead
    /// tail's slot and the chain reads as it did before the death. The
    /// group pair is `Group::kill@00714110` then `Group::add@00714350`,
    /// and `add` writes no member's `+0x80`: every figure of the squad
    /// leaves the group it named. run1545's Hoplites `0/8..0/10`, short
    /// `0/9` since 810 and garrisoned in `0/2000`, take the heal at 15 on
    /// game frame 862 and print on block 863 a new `0/9` (uid 20) at
    /// `damage` 39 — 120 / 3, less the heal's one point — inside behind
    /// `0/10`, and `group` −1 on all three.
    ///
    /// SEAM: the outside arm's `find_nearby_spot` and move order — the
    /// figure stays on the tail's point; the console's selection and the
    /// hotkey groups, which this crate does not carry; and a chain with
    /// no dead tail (`o_down < 0`, the listing's error report), where the
    /// original numbers a fresh object — none is regrown here.
    pub(crate) fn regrow_figure(&mut self, tail: usize, down: Option<usize>, left: i32) {
        let Some(ty) = self.units[tail].ty else {
            return;
        };
        let uber = self.unit_types[ty].combat.uber_size;
        if self.curr_uber_size(tail) >= uber {
            return;
        }
        let inside = self.units[tail].inside;
        let boat = self.units[tail].inside_unit;
        if inside.is_none() && boat.is_none() && self.units[tail].in_danger {
            return;
        }
        let Some(dead) = down else {
            return;
        };
        let who = self.units[tail].owner;
        let index = self.units[dead].index;
        let at = self.units[tail].pos;
        let pos = crate::Pos::new(crate::gaia::init_snap(at.x), crate::gaia::init_snap(at.y));
        let n = self.init_figure(who, ty, index, pos);
        // The links are the dead tail's, restored over `init`'s −1s; this
        // crate keeps the dead occupant as its own entry, so the figures
        // either side are pointed at the new one and the old is unlinked.
        let (up, below) = (self.units[dead].o_up, self.units[dead].o_down);
        self.units[dead].o_up = None;
        self.units[dead].o_down = None;
        self.units[n].o_up = up;
        self.units[n].o_down = below;
        if let Some(a) = up {
            self.units[a].o_down = Some(n);
        }
        if let Some(b) = below {
            self.units[b].o_up = Some(n);
        }
        self.units[n].captain = false;
        self.units[n].combat.captain = self.units[tail].combat.captain;
        // `damage = hits / uber_size`, the hits the tail's vslot `+0x11c`
        // answers.
        let share = self.units[tail].max_health / uber.max(1);
        self.units[n].health = self.units[n].max_health - share;
        self.units[n].stance = self.units[tail].stance;
        self.units[n].combat.stance = self.units[tail].combat.stance;
        self.copy_unit_masks(tail, n);
        if let Some(b) = inside.or(boat) {
            // `Object::insert_inside@00647e90`: off the world, onto the
            // host's chain; a follower counts in no garrison tally.
            self.coll_remove(n);
            self.chain_remove(n);
            let u = &mut self.units[n];
            if inside.is_some() {
                u.inside = Some(b);
            } else {
                u.inside_unit = Some(b);
            }
            u.on_map = false;
        }
        if self.units[tail].group_ptr.is_some()
            && let Some(seat) = self.seat_of(tail)
            && self.seat_list(seat).contains(&tail)
        {
            self.seat_kill(seat, tail, false, false);
            self.seat_add(seat, tail, false, false);
        }
        self.repair_unit_damage(n, left);
    }

    /// `n.unit_masks = tail.unit_masks` — the word `repair_damage` copies
    /// whole onto a regrown figure (`0060de10:154`), over the bits this
    /// crate keeps as fields.
    fn copy_unit_masks(&mut self, from: usize, to: usize) {
        let f = self.units[from].clone();
        let t = &mut self.units[to];
        t.decoy = f.decoy;
        t.movement.mirror = f.movement.mirror;
        t.in_danger = f.in_danger;
        t.line_ok = f.line_ok;
        t.contact = f.contact;
        t.waiting_on = f.waiting_on;
        t.was_builder = f.was_builder;
        t.cloak = f.cloak;
        t.blown = f.blown;
        t.fought = f.fought;
        t.casting = f.casting;
        t.auto_transport = f.auto_transport;
        t.half_step = f.half_step;
        t.carry = f.carry;
        t.combat.packed = f.combat.packed;
    }

    /// `UnitData::has_damage(this, 1)@00609b50` from the captain: a figure
    /// of the chain that is hurt, holds a fraction, **or is dead** — the
    /// dead tail `Unit::close` leaves linked counts (item 3993).
    pub(crate) fn has_damage(&self, i: usize) -> bool {
        let mut cur = Some(i);
        let mut guard = 0;
        while let Some(f) = cur {
            let u = &self.units[f];
            if !u.alive() || u.health < u.max_health || u.damage_frac != 0 {
                return true;
            }
            cur = u.o_down;
            guard += 1;
            if guard > 64 {
                break;
            }
        }
        false
    }

    /// **The ship heal** — `process_healing`'s sea arm (`005e0670:39`–`:71`,
    /// item 3854, `docs/GOLDEN.md` §153.2), which returns before every
    /// other on-map heal:
    ///
    /// ```text
    /// captain, not air, has_damage(1), inside_up < 0     # the head
    /// domain == SEA:
    ///   !(unit_flags & 0x10)                    # not a transport
    ///     and (rare | rare_conquest) has CITRUS
    ///     and (o + frame) % UNIT_HEAL_RATE[0] == 0
    ///                         → repair_damage(1, 1, 1); healing = max(rate, healing)
    ///   is(AIRCRAFTCARRIER, 1)
    ///     and (o + frame) % UNIT_HEAL_RATE[0] == 0
    ///                         → repair_damage(1, 1, 1); healing = max(rate, healing)
    ///   return
    /// ```
    ///
    /// Both arms ride one phase, so a carrier of a Citrus holder takes two
    /// points on it. The rate is the table's first entry whatever the
    /// player's heal level: the garrison heal's `get_heal_level` index is
    /// not read here. The carrier test is `ObjectData::is(0x15f, 1)`, the
    /// strict lineage (the type itself, or a unit grafted onto it that is
    /// not unique). run1485's carriers `1/18` and `2/18` (`o` 18) heal on
    /// game frame 802 — block 803, both 1 down — and `2/18` on 782 before
    /// it (`healing 8` on block 795); ours healed neither, and on 1201
    /// each stood 23 points ahead.
    ///
    /// The sparkle arm (the second argument) draws only for the console's
    /// allies, which is no stream this crate keeps. `healing` (`+0x38`) is
    /// carried since item 3961 ([`Sim::mark_healed`]).
    pub(crate) fn ship_heal(&mut self, i: usize, frame: i64) {
        const AIRCRAFTCARRIER: i32 = 0x15f;
        let rate = i64::from(self.tuning.unit_heal_rate[0]);
        if rate == 0 || (i64::from(self.units[i].index) + frame) % rate != 0 {
            return;
        }
        let u = &self.units[i];
        if !u.alive() || !u.on_map || u.inside.is_some() || u.inside_unit.is_some() {
            return;
        }
        if self.captain_of(i) != i || !matches!(u.kind.domain, Domain::Sea) {
            return;
        }
        let Some(t) = u.ty else {
            return;
        };
        let damaged = self.squad_of(i).iter().any(|&f| {
            self.units[f].health < self.units[f].max_health || self.units[f].damage_frac != 0
        });
        if !damaged {
            return;
        }
        let owner = u.owner;
        let ut = &self.unit_types[t];
        let transport = ut.cols.flag(crate::ai_load::uflags::TRANSPORT);
        let carrier = ut.type_index == AIRCRAFTCARRIER
            || ut.tree.is_some_and(|tr| {
                self.tech_tree.types.get(AIRCRAFTCARRIER as usize).is_some()
                    && self.tech_tree.is(tr, AIRCRAFTCARRIER as usize, true)
            });
        if !transport && self.has_rare(owner, crate::economy::CITRUS) {
            self.repair_unit_damage(i, 1);
            self.mark_healed(i, rate);
        }
        if carrier {
            self.repair_unit_damage(i, 1);
            self.mark_healed(i, rate);
        }
    }

    /// **The Iroquois heal** — `process_healing`'s on-map arm after the
    /// Conquer-the-World hero's (`005e0670:118`–`:147`, item 3959,
    /// `docs/AI.md` §403):
    ///
    /// ```text
    /// captain, not air, has_damage(1), inside_up < 0     # the head
    /// domain == SEA → the ship heal;  is_supply → return
    /// has_tribe_bonus(0x12) and !(unit_masks & 0x1000)   # not blown
    ///   and !is_moving                                   # vslot +0xd8
    ///   and domain == LAND:
    ///     rate = IROQUOIS_HEAL_RATE[get_heal_level]
    ///     rate != 0 and (o + frame) % rate == 0
    ///       and the cell's owner is an ally:
    ///         repair_damage(1, 1, 1); healing = max(rate, healing)
    /// ```
    ///
    /// **The standing test** is vslot `+0xd8`, `UnitData::is_moving`
    /// ([`crate::Sim::is_moving`]: the head order is a move), read on the
    /// captain; both territory readers omitted it. A unit walking under
    /// an attack order is not moving by it. The point goes down the squad
    /// as the CEO's does ([`Self::repair_unit_damage`]): run1228b's
    /// Mohawk Spearmen captain `1/50` (unhurt, `healing 20` from block
    /// 8191) hands it to `1/52`, 42 to 41 on 8191, 40 on 8211, 39 on 8231,
    /// and at rate 15 (Herbal Lore, `get_heal_level` 1) 38 on 8246.
    ///
    /// The sparkle arm draws only for the console's allies, which is no
    /// stream this crate keeps; `healing` (`+0x38`) is
    /// [`Self::mark_healed`] (item 3961); the `0x4000` clear is not carried.
    pub(crate) fn iroquois_heal(&mut self, i: usize, frame: i64) {
        let u = &self.units[i];
        if !u.alive() || !u.on_map || u.inside.is_some() || u.inside_unit.is_some() {
            return;
        }
        if self.captain_of(i) != i || !matches!(u.kind.domain, Domain::Land) {
            return;
        }
        let Some(t) = u.ty else {
            return;
        };
        let owner = u.owner;
        if u.blown || !self.iroquois(owner) || self.is_moving(i) {
            return;
        }
        if self.unit_types[t].cols.flag2(uflags2::SUPPLY_OR_HERO) {
            return;
        }
        let rate = i64::from(self.tuning.iroquois_heal_rate[self.heal_level(owner)]);
        if rate == 0 || (i64::from(self.units[i].index) + frame) % rate != 0 {
            return;
        }
        let damaged = self.squad_of(i).iter().any(|&f| {
            self.units[f].health < self.units[f].max_health || self.units[f].damage_frac != 0
        });
        if !damaged {
            return;
        }
        let ally = match self.world.owner_at(self.units[i].pos) {
            Owner::Player(p) => self.is_ally(owner, p),
            _ => false,
        };
        if ally {
            self.repair_unit_damage(i, 1);
            self.mark_healed(i, rate);
        }
    }

    /// **`LeaderData::get_heal_level@006e12e0`**: how many of
    /// `HEAL_FASTER_1`..`HEAL_FASTER_3` (TypeIndex `0x2ef`..`0x2f1`,
    /// `TECHBONUSES` rows 67–69) the leader holds by `has_preq` — the
    /// bonus's prerequisite, Herbal Lore, Medicine and Pharmaceuticals —
    /// 0..3, as [`Self::speed_upgrade_level`] counts its ladders. The
    /// loop's `BUY_SELL` arm (`0x2ad`) lies outside its range and never
    /// runs.
    pub(crate) fn heal_level(&self, who: crate::world::Player) -> usize {
        let ladder = self.tech_tree.roles.heal_faster_preq;
        self.speed_upgrade_level(who, &ladder) as usize
    }

    pub(crate) fn civilian_heal(&mut self, i: usize, frame: i64) {
        let rate = i64::from(self.tuning.civilian_heal_rate);
        if rate == 0 || (i64::from(self.units[i].index) + frame) % rate != 0 {
            return;
        }
        let u = &self.units[i];
        if !u.alive() || !u.on_map || u.inside.is_some() || u.inside_unit.is_some() {
            return;
        }
        if self.captain_of(i) != i || !matches!(u.kind.domain, Domain::Land) {
            return;
        }
        let Some(t) = u.ty else {
            return;
        };
        // `UnitData::has_damage(1)`: this figure or one below it.
        let damaged = self.squad_of(i).iter().any(|&f| {
            self.units[f].health < self.units[f].max_health || self.units[f].damage_frac != 0
        });
        if !damaged {
            return;
        }
        let cols = self.unit_types[t].cols;
        if cols.flag2(uflags2::SUPPLY_OR_HERO) || u.unit_masks2 & STRUCK != 0 {
            return;
        }
        let id = self.unit_types[t].type_index;
        let civilian = WORKERS.contains(&id)
            || cols.flag2(uflags2::CARAVAN)
            || self.is_merchant(i)
            || id == FISHERMEN;
        if !civilian {
            return;
        }
        let owner = u.owner;
        let ally = match self.world.owner_at(u.pos) {
            Owner::Player(p) => self.is_ally(owner, p),
            _ => false,
        };
        if !ally {
            return;
        }
        let u = &mut self.units[i];
        if u.health < u.max_health {
            u.health += 1;
            u.damage_frac = 0;
        }
        self.mark_healed(i, rate);
    }
}

#[cfg(test)]
mod tests {
    use crate::world::Owner;

    /// **A citizen heals a point every 45 frames in its own territory**,
    /// on the frame `(o + frame) % 45 == 0`, and the heal clears the
    /// fraction (`docs/COMBAT.md` §72). Outside it, or on another frame,
    /// or as a soldier, nothing heals.
    #[test]
    fn a_citizen_heals_a_point_every_45_frames_in_its_own_territory() {
        use crate::world::{Pos, World};
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        let ty = |sim: &mut crate::Sim, type_index: i32| {
            sim.add_unit_type(crate::UnitType {
                hits: 40,
                type_index,
                combat: crate::combat::Profile {
                    uber_size: 1,
                    ..crate::combat::Profile::default()
                },
                ..crate::UnitType::default()
            })
        };
        let citizen = ty(&mut sim, 0x32);
        let hoplite = ty(&mut sim, 0x60);
        let put = |sim: &mut crate::Sim, t: usize, p: Pos| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = crate::Unit::new(0, index, p, 40);
            u.ty = Some(t);
            u.on_map = true;
            u.kind = sim.unit_types[t].kind;
            sim.add_unit(u)
        };
        let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
        let c = put(&mut sim, citizen, at);
        let soldier = put(&mut sim, hoplite, at);
        sim.world
            .set_owner(at.cell(), Owner::Player(0), Owner::Player(0));
        for u in [c, soldier] {
            sim.units[u].health = sim.units[u].max_health - 3;
            sim.units[u].damage_frac = 5;
        }
        let o = i64::from(sim.units[c].index);
        let due = 45 * 100 - o;
        sim.civilian_heal(c, due - 1);
        assert_eq!(
            (sim.units[c].health, sim.units[c].damage_frac),
            (sim.units[c].max_health - 3, 5),
            "off the phase, nothing"
        );
        assert_eq!(sim.units[c].healing, 0, "no heal, no clock");
        sim.civilian_heal(c, due);
        assert_eq!(
            (sim.units[c].health, sim.units[c].damage_frac),
            (sim.units[c].max_health - 2, 0),
            "on the phase, a point and the fraction"
        );
        // Item 3961: the heal sets `healing` to `max(45, healing)` —
        // run1151c's Citizen `0/8` reads 45 on 4493 and 4538.
        assert_eq!(sim.units[c].healing, 45, "the clock, at the rate");
        sim.units[c].healing = 50;
        sim.units[c].health -= 1;
        sim.civilian_heal(c, due + 45);
        assert_eq!(sim.units[c].healing, 50, "a longer clock stands");
        let os = i64::from(sim.units[soldier].index);
        sim.civilian_heal(soldier, 45 * 100 - os);
        assert_eq!(
            sim.units[soldier].health,
            sim.units[soldier].max_health - 3,
            "a soldier is not a civilian"
        );
        let cell = sim.units[c].pos.cell();
        sim.world.set_owner(cell, Owner::None, Owner::None);
        sim.civilian_heal(c, due + 45);
        assert_eq!(
            sim.units[c].health,
            sim.units[c].max_health - 2,
            "outside an ally's territory, nothing"
        );
    }

    /// **An Aircraft Carrier repairs a point every 20 frames, anywhere; a
    /// Citrus holder's ships do too** (item 3854, `docs/GOLDEN.md` §153.2):
    /// `process_healing`'s sea arm, phased on `(o + frame) %
    /// UNIT_HEAL_RATE[0]`. A Destroyer heals only under Citrus, a
    /// transport never, and a carrier under Citrus takes two points on its
    /// phase. run1485's carriers `1/18` and `2/18` (`o` 18) healed a point
    /// each on game frame 802, the cell nobody's.
    #[test]
    fn a_carrier_repairs_a_point_every_20_frames_and_citrus_heals_every_ship() {
        use crate::ai_load::{UnitCols, uflags};
        use crate::attrition::{Domain, UnitKind};
        use crate::world::{Pos, World};
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        let ty = |sim: &mut crate::Sim, type_index: i32, unit_flags: u32| {
            sim.add_unit_type(crate::UnitType {
                hits: 600,
                type_index,
                kind: UnitKind {
                    domain: Domain::Sea,
                    ..UnitKind::default()
                },
                cols: UnitCols {
                    unit_flags,
                    ..UnitCols::default()
                },
                combat: crate::combat::Profile {
                    uber_size: 1,
                    ..crate::combat::Profile::default()
                },
                ..crate::UnitType::default()
            })
        };
        let carrier = ty(&mut sim, 0x15f, 0);
        let destroyer = ty(&mut sim, 328, 0);
        let freighter = ty(&mut sim, 322, uflags::TRANSPORT);
        let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
        // `o` 18, 38 and 58: one phase of 20 for all three.
        let put = |sim: &mut crate::Sim, t: usize, o: i16| {
            let mut u = crate::Unit::new(1, o, at, 600);
            u.ty = Some(t);
            u.type_index = sim.unit_types[t].type_index;
            u.on_map = true;
            u.captain = true;
            u.kind = sim.unit_types[t].kind;
            let i = sim.add_unit(u);
            sim.units[i].health = 600 - 10;
            sim.units[i].damage_frac = 7;
            i
        };
        let ships = [
            put(&mut sim, carrier, 18),
            put(&mut sim, destroyer, 38),
            put(&mut sim, freighter, 58),
        ];
        let due = 820 - 18;
        let state =
            |sim: &crate::Sim| ships.map(|i| (sim.units[i].health, sim.units[i].damage_frac));
        for &i in &ships {
            sim.ship_heal(i, due - 1);
        }
        assert_eq!(state(&sim), [(590, 7); 3], "off the phase, nothing");
        for &i in &ships {
            sim.ship_heal(i, due);
        }
        assert_eq!(
            state(&sim),
            [(591, 0), (590, 7), (590, 7)],
            "on the phase, the carrier alone: a point and the fraction"
        );
        sim.ledgers[1].rare |= 1 << (crate::economy::CITRUS - crate::economy::BASE_RARE);
        for &i in &ships {
            sim.ship_heal(i, due + 20);
        }
        assert_eq!(
            state(&sim),
            [(593, 0), (591, 0), (590, 7)],
            "under Citrus: the carrier two, the destroyer one, the transport none"
        );
    }

    /// **A CEO heals its army abroad, a point every 20 frames, down the
    /// squad** (item 3686, `docs/AI.md` §348.1): `process_healing`'s third
    /// economic-patriot arm runs on the captain, phased on the captain's
    /// `o`, and its `repair_damage(1, …)` hands the point down `o_down`
    /// past an unhurt figure to the hurt one, zeroing its fraction. The
    /// cell is nobody's. Off the phase, without a CEO, or under an
    /// attrition period, nothing. run1338b's `1/36` (captain `1/35`) went
    /// 4 and 14/16 to 3 and 6/16 on block 3167, a point and then the
    /// bleed. Made to fail with the repair stopping at the captain
    /// (`tools/mutate.py`, `cur = None`).
    #[test]
    fn a_ceo_heals_the_hurt_figure_of_a_squad_abroad_every_20_frames() {
        use crate::ai_load::{UnitCols, uflags2};
        use crate::world::{Pos, World};
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        let rifle = sim.add_unit_type(crate::UnitType {
            hits: 645,
            type_index: 137,
            combat: crate::combat::Profile {
                uber_size: 3,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        let ceo = sim.add_unit_type(crate::UnitType {
            hits: 110,
            type_index: 0x165,
            cols: UnitCols {
                unit_flags2: uflags2::GENERAL | uflags2::SUPPLY_OR_HERO,
                ..UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
        let put = |sim: &mut crate::Sim, t: usize, hits: i32| {
            let index = i16::try_from(sim.units.len()).unwrap();
            let mut u = crate::Unit::new(1, index, at, hits);
            u.ty = Some(t);
            u.type_index = sim.unit_types[t].type_index;
            u.on_map = true;
            u.kind = sim.unit_types[t].kind;
            sim.add_unit(u)
        };
        let squad = [
            put(&mut sim, rifle, 215),
            put(&mut sim, rifle, 215),
            put(&mut sim, rifle, 215),
        ];
        sim.units[squad[0]].captain = true;
        sim.units[squad[0]].o_down = Some(squad[1]);
        sim.units[squad[1]].captain = false;
        sim.units[squad[1]].o_up = Some(squad[0]);
        sim.units[squad[1]].o_down = Some(squad[2]);
        sim.units[squad[2]].captain = false;
        sim.units[squad[2]].o_up = Some(squad[0]);
        let hurt = squad[2];
        sim.units[hurt].health = 215 - 4;
        sim.units[hurt].damage_frac = 14;
        let c = squad[0];
        let o = i64::from(sim.units[c].index);
        let due = 20 * 160 - o;
        let state = |sim: &crate::Sim| (sim.units[hurt].health, sim.units[hurt].damage_frac);
        sim.ceo_heal(c, due);
        assert_eq!(state(&sim), (211, 14), "no CEO, nothing");
        let boss = put(&mut sim, ceo, 110);
        sim.units[boss].captain = true;
        sim.ceo_heal(c, due - 1);
        assert_eq!(state(&sim), (211, 14), "off the phase, nothing");
        sim.ceo_heal(hurt, due);
        assert_eq!(
            state(&sim),
            (211, 14),
            "a figure below the captain is not asked"
        );
        sim.ceo_heal(c, due);
        assert_eq!(
            state(&sim),
            (212, 0),
            "a point down the chain, and the fraction"
        );
        assert_eq!(
            sim.units[squad[1]].health, 215,
            "the unhurt figure passes it on"
        );
        sim.units[c].attrition = 48;
        sim.ceo_heal(c, due + 20);
        assert_eq!(state(&sim), (212, 0), "under an attrition period, nothing");
    }

    /// A captain of player 1 on its own cell, 40 hits, three down with a
    /// fraction, in a tree whose one row is Herbal Lore, the heal ladder's
    /// first rung (`TECHBONUSES` row 67).
    fn iroquois_captain(type_index: i32) -> (crate::Sim, usize, usize) {
        use crate::world::{Pos, World};
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        let mut tree = crate::tech::TechTree::new();
        let herbal = tree.add(crate::tech::TypeDef::plain("Herbal Lore", 0));
        tree.roles.heal_faster_preq = [Some(herbal), None, None];
        sim.set_tech_tree(tree);
        let ty = sim.add_unit_type(crate::UnitType {
            hits: 40,
            type_index,
            combat: crate::combat::Profile {
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        let at = Pos::new(30 * 0x300 + 0x198, 30 * 0x300 + 0x198);
        let mut u = crate::Unit::new(1, 50, at, 40);
        u.ty = Some(ty);
        u.type_index = type_index;
        u.on_map = true;
        u.captain = true;
        u.kind = sim.unit_types[ty].kind;
        let c = sim.add_unit(u);
        sim.units[c].health = 37;
        sim.units[c].damage_frac = 5;
        sim.world
            .set_owner(at.cell(), Owner::Player(1), Owner::Player(1));
        (sim, c, herbal)
    }

    fn attack_to(dest: crate::world::Pos) -> crate::orders::Order {
        crate::orders::Order {
            flags: crate::orders::flag::ACTION,
            body: crate::orders::Body::Move(crate::orders::MoveOrder {
                kind: crate::orders::MoveKind::AttackTo,
                dest,
                angle: crate::movement::Angle(0),
                facing: None,
                has_waypoint: false,
                waypoint: dest,
                last: None,
                pause: 0,
                retry: 0,
                attempts: 0,
                timer: 0,
                coll: None,
                orig: None,
                group: None,
            }),
        }
    }

    /// **An Iroquois captain heals standing on its own ground, at the heal
    /// level's period** (item 3959, `docs/AI.md` §403.2): a point every 20
    /// frames, 15 under Herbal Lore, phased on `o`; nothing under an
    /// attack-move (`is_moving`), when blown, on another leader's ground,
    /// or for a leader without the bonus. run1228b's `1/50` was refused
    /// on 8186, 8201 and 8216 under its `ATTACKTOORDER` and healed on 8246
    /// under its `GUARDORDER`.
    /// **A French unit heals within its own supply source's radius, on
    /// its phase, walking** (item 3978, `docs/AI.md` §410): the supply
    /// heal's period is `SUPPLY_HEAL_RATE` (0) plus the French bonus's 20,
    /// and `find_supply` asks only the supply list's reach — 14 tiles,
    /// 2688 by `vector_dist`. run1542's Elite Special Forces `1/17` stood
    /// (1709, 1865) off the Despot `1/164`, 2648, and healed on 1143
    /// under an explore order; its `healing` read 13 on block 1151.
    #[test]
    fn a_french_unit_heals_within_its_supply_radius_every_20_frames() {
        use crate::world::{Pos, World};
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        sim.set_tech_tree(crate::tech::TechTree::new());
        let esf = sim.add_unit_type(crate::UnitType {
            hits: 130,
            type_index: 77,
            combat: crate::combat::Profile {
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        let despot = sim.add_unit_type(crate::UnitType {
            hits: 109,
            type_index: 352,
            kind: crate::attrition::UnitKind {
                supply_unit: true,
                ..crate::attrition::UnitKind::default()
            },
            ..crate::UnitType::default()
        });
        let at = Pos::new(25157, 9797);
        let put = |sim: &mut crate::Sim, t: usize, index: i16, pos: Pos| {
            let mut u = crate::Unit::new(1, index, pos, sim.unit_types[t].hits);
            u.ty = Some(t);
            u.type_index = sim.unit_types[t].type_index;
            u.on_map = true;
            u.captain = true;
            u.kind = sim.unit_types[t].kind;
            sim.add_unit(u)
        };
        let c = put(&mut sim, esf, 17, at);
        sim.units[c].health = 130 - 11;
        sim.units[c].damage_frac = 4;
        let state = |sim: &crate::Sim| {
            let u = &sim.units[c];
            (u.health, u.damage_frac, u.healing)
        };
        sim.supply_heal(c, 1143);
        assert_eq!(state(&sim), (119, 4, 0), "no bonus, no period");
        sim.tech[1].power = Some(crate::nations::power::FRENCH);
        sim.supply_heal(c, 1143);
        assert_eq!(state(&sim), (119, 4, 0), "no supply source, nothing");
        let d = put(&mut sim, despot, 164, Pos::new(at.x - 1709, at.y + 1865));
        assert!(
            sim.units[d].supply_slot.is_some(),
            "the Despot holds a slot"
        );
        sim.supply_heal(c, 1142);
        assert_eq!(state(&sim), (119, 4, 0), "off the phase, nothing");
        sim.units[c]
            .orders
            .push_front(attack_to(Pos::new(25848, 10488)));
        sim.supply_heal(c, 1143);
        assert_eq!(
            state(&sim),
            (120, 0, 20),
            "(17 + 1143) % 20, walking, 2648 off: a point and the fraction"
        );
        sim.units[d].pos = Pos::new(at.x - 1709 - 60, at.y + 1865);
        sim.supply_heal(c, 1163);
        assert_eq!(state(&sim), (120, 0, 20), "past 2688, nothing");
    }

    #[test]
    fn an_iroquois_captain_heals_standing_on_its_own_ground() {
        let (mut sim, c, herbal) = iroquois_captain(167);
        let state = |sim: &crate::Sim| (sim.units[c].health, sim.units[c].damage_frac);
        sim.iroquois_heal(c, 8190);
        assert_eq!(state(&sim), (37, 5), "no bonus, nothing");
        sim.tech[1].power = Some(crate::nations::power::IROQUOIS);
        assert!(sim.iroquois(1), "the bonus holds in this setup");
        sim.iroquois_heal(c, 8189);
        assert_eq!(state(&sim), (37, 5), "off the phase, nothing");
        let at = sim.units[c].pos;
        sim.units[c].orders.push_front(attack_to(at));
        sim.iroquois_heal(c, 8190);
        assert_eq!(state(&sim), (37, 5), "an attack-move is moving");
        sim.units[c].orders.clear();
        sim.units[c].blown = true;
        sim.iroquois_heal(c, 8190);
        assert_eq!(state(&sim), (37, 5), "a blown unit, nothing");
        sim.units[c].blown = false;
        sim.iroquois_heal(c, 8190);
        assert_eq!(state(&sim), (38, 0), "standing, level 0: (50 + 8190) % 20");
        sim.tech[1].tech[herbal] = true;
        assert_eq!(sim.heal_level(1), 1, "Herbal Lore is the first rung");
        sim.iroquois_heal(c, 8210);
        assert_eq!(state(&sim), (38, 0), "level 1 is not on the 20s");
        sim.iroquois_heal(c, 8245);
        assert_eq!(state(&sim), (39, 0), "level 1: (50 + 8245) % 15");
        let cell = sim.units[c].pos.cell();
        sim.world
            .set_owner(cell, Owner::Player(0), Owner::Player(0));
        sim.iroquois_heal(c, 8260);
        assert_eq!(state(&sim), (39, 0), "another leader's ground, nothing");
    }

    /// **The Senator heals a walking column on allied ground, every 20
    /// frames** (item 3959, `docs/AI.md` §403.3): `process_healing`'s
    /// `0x161` arm asks no standing test, only an allied cell and a
    /// Senator of the owner in reach. run1228b's `1/50`, walking under an
    /// `ATTACKTOORDER` with `1/92` near, healed on 8191, 8211 and 8231.
    #[test]
    fn the_senator_heals_a_walking_column_on_allied_ground() {
        use crate::ai_load::{UnitCols, uflags2};
        let (mut sim, c, _) = iroquois_captain(167);
        let state = |sim: &crate::Sim| (sim.units[c].health, sim.units[c].damage_frac);
        let at = sim.units[c].pos;
        sim.units[c].orders.push_front(attack_to(at));
        sim.senator_heal(c, 8190);
        assert_eq!(state(&sim), (37, 5), "no Senator, nothing");
        let senator = sim.add_unit_type(crate::UnitType {
            hits: 110,
            type_index: 0x161,
            cols: UnitCols {
                unit_flags2: uflags2::GENERAL | uflags2::SUPPLY_OR_HERO,
                ..UnitCols::default()
            },
            ..crate::UnitType::default()
        });
        let mut u = crate::Unit::new(1, 92, sim.units[c].pos, 110);
        u.ty = Some(senator);
        u.type_index = 0x161;
        u.on_map = true;
        u.captain = true;
        u.kind = sim.unit_types[senator].kind;
        sim.add_unit(u);
        sim.senator_heal(c, 8189);
        assert_eq!(state(&sim), (37, 5), "off the phase, nothing");
        sim.senator_heal(c, 8190);
        assert_eq!(state(&sim), (38, 0), "walking, a point and the fraction");
        let cell = sim.units[c].pos.cell();
        sim.world.set_owner(cell, Owner::None, Owner::None);
        sim.senator_heal(c, 8210);
        assert_eq!(state(&sim), (38, 0), "nobody's ground, nothing");
    }

    /// **A patriot repairs its leader's damaged building by the ground under
    /// it** (item 3987, `docs/GOLDEN.md` §180; `Build::process@0061edf0`'s
    /// arm to `0061f3b9`): a President's reach repairs on unowned or allied
    /// ground and nowhere else — and, found, it closes the Senator's branch
    /// even where that one would repair; a Senator's only on allied ground;
    /// a point on the phase `(o + frame) % 20`, the fraction cleared,
    /// `healing` 20; nothing without the leader's count of the type.
    #[test]
    fn a_patriot_repairs_its_leader_s_building_by_the_ground_under_it() {
        use crate::ai_load::{UnitCols, uflags2};
        use crate::world::{Pos, World};
        let mut sim = crate::Sim::new(crate::tuning::Tuning::RON, World::new(60, 60), 2);
        sim.build_types.push(crate::build::BuildType {
            x_size: 3,
            y_size: 3,
            ..crate::build::BuildType::default()
        });
        let at = Pos::new(9000, 9000);
        let b = sim.add_building(1, at, 0);
        sim.buildings[b].ty = Some(sim.build_types.len() - 1);
        sim.buildings[b].index = 2007;
        sim.buildings[b].active = true;
        sim.buildings[b].hits = 1200;
        let hero = |sim: &mut crate::Sim, type_index: i32| {
            let t = sim.add_unit_type(crate::UnitType {
                hits: 110,
                type_index,
                cols: UnitCols {
                    unit_flags2: uflags2::GENERAL | uflags2::SUPPLY_OR_HERO,
                    ..UnitCols::default()
                },
                ..crate::UnitType::default()
            });
            let mut u = crate::Unit::new(1, 6 + type_index as i16, at, 110);
            u.ty = Some(t);
            u.type_index = type_index;
            u.on_map = true;
            u.captain = true;
            u.kind = sim.unit_types[t].kind;
            (sim.add_unit(u), t)
        };
        let (senator, st) = hero(&mut sim, 0x161);
        let (president, pt) = hero(&mut sim, 0x163);
        sim.units[president].pos.x += 40 * 192;
        let n = sim.unit_types.len();
        sim.muster[1].by_type.resize(n, 0);
        let cell = at.cell();
        let run = |sim: &mut crate::Sim, frame: i64, ground: Owner| {
            sim.world.set_owner(cell, ground, Owner::None);
            let bd = &mut sim.buildings[b];
            (bd.damage, bd.damage_frac, bd.healing) = (10, 5, 0);
            sim.patriot_repair(b, frame);
            let bd = &sim.buildings[b];
            (bd.damage, bd.damage_frac, bd.healing)
        };
        let own = Owner::Player(1);
        assert_eq!(run(&mut sim, 13, own), (10, 5, 0), "no patriot counted");
        sim.muster[1].by_type[st] = 1;
        assert_eq!(
            run(&mut sim, 13, own),
            (9, 0, 20),
            "a Senator on its own ground"
        );
        assert_eq!(run(&mut sim, 14, own), (10, 5, 0), "off the phase");
        assert_eq!(
            run(&mut sim, 13, Owner::None),
            (10, 5, 0),
            "a Senator on nobody's ground"
        );
        assert_eq!(
            run(&mut sim, 13, Owner::Player(0)),
            (10, 5, 0),
            "a Senator on an enemy's"
        );
        sim.muster[1].by_type[pt] = 1;
        sim.units[president].pos = at;
        assert_eq!(
            run(&mut sim, 33, Owner::None),
            (9, 0, 20),
            "a President on nobody's ground"
        );
        assert_eq!(run(&mut sim, 33, own), (9, 0, 20), "a President on its own");
        sim.units[senator].pos.x += 40 * 192;
        assert_eq!(
            run(&mut sim, 33, Owner::Player(0)),
            (10, 5, 0),
            "a President on an enemy's"
        );
        sim.units[senator].pos = at;
        assert_eq!(
            run(&mut sim, 33, Owner::Player(0)),
            (10, 5, 0),
            "a President found closes the Senator's branch"
        );
    }
}
