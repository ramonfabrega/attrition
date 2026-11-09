//! Supply: the counter to attrition, and the two other things nobody mentions
//! in the same breath.
//!
//! One radius query answers three questions. A unit inside a friendly supply
//! radius takes no attrition at all; on a slower period it repairs damage; and
//! a siege unit outside one reloads more slowly. `docs/SUPPLY.md` is the
//! specification this implements.
//!
//! # The shape of it
//!
//! A supply source is **a unit whose type carries the supply flag** and nothing
//! else — not a city, not a fort, not a building. Each registers a slot in its
//! owner's list when it is created and gives the slot back when it dies, and
//! the query is a linear scan of that list.
//!
//! The flag is set at load by `UnitType::init_final_flags`, for the supply
//! wagon line **and the three military patriots** — the Despot, the Monarch and
//! the Comrade. A patriot therefore supplies at the wagon radius rather than at
//! his general radius, and inherits everything else the bit carries: no supply
//! heal, no war-time territorial attrition, and the wagon's HP and speed
//! upgrades. This module calls a source a `Wagon` because that is what it
//! almost always is; nothing here branches on which it is.
//!
//! Two consequences fall out of that and are easy to get wrong:
//!
//! - **The radius depends only on the player.** Every wagon a player owns has
//!   the same reach, so which one supplies a unit is not observable through
//!   the mechanic — only through the interface, which shows the slot index.
//! - **It is your own list only.** An ally's supply wagon does nothing for
//!   your units.
//!
//! Distances go through [`crate::world::vector_dist`], the same deliberately
//! wrong hypotenuse the territory pass measures borders with. Supply radii are
//! octagonal in exactly the way borders are.

use crate::attrition::Domain;
use crate::tuning::Tuning;
use crate::world::{Owner, Player, Pos, UNITS_PER_TILE, vector_dist};

// ---------------------------------------------------------------------------
// The radius
// ---------------------------------------------------------------------------

/// A supply source's reach, in tiles.
///
/// `upgrades` is how many steps of the supply chain the owner holds, 0 to 3 —
/// `LeaderData::get_supply_upgrade` counting `has_preq` over the three
/// `SUPPLY_WAGONS_*` steps, and nothing else. The original's loop carries an
/// arm that would credit a nation bonus instead of the prerequisite for one
/// particular step, but the step it names is outside the range the loop walks,
/// so it never fires; the same arm appears in every one of these counters and
/// is a shared loop macro rather than a rule.
///
/// The Terra Cotta Army is wired in and contributes nothing: `TERRA_COTTA_RANGE`
/// ships as 0. It is kept because leaving it out would silently change what a
/// mod that sets the constant does.
pub const fn supply_radius(t: &Tuning, upgrades: i32, terra_cotta: bool) -> i32 {
    let mut r = t.supply_radius + t.supply_radius_upgrade * upgrades;
    if terra_cotta {
        r += t.terra_cotta_range;
    }
    r
}

/// Which half of the patriot roster a general belongs to. The two carry
/// different radius bonuses and nothing else here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Patriot {
    Military,
    Economic,
    /// Both rosters at once: no shipped type is in both, but the original tests
    /// the two sets independently and adds each bonus it finds.
    Both,
}

/// Everything a general's aura radius depends on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct General {
    /// Steps of the general upgrade chain the owner holds.
    pub upgrades: i32,
    /// Parmenio scales the radius by 3/2.
    pub parmenio: bool,
    /// Wellington doubles it.
    pub wellington: bool,
    /// Kutosov triples it — and Kutosov is also one of the three generals whose
    /// aura supplies, so this is the largest supply radius in the game.
    pub kutosov: bool,
    /// The Terra Cotta Army, which the original asks the *owner* about and not
    /// the hero: it applies to any general, the three supplying ones included.
    /// It ships contributing nothing.
    pub terra_cotta: bool,
    /// The patriot bonuses, unlike Terra Cotta, are strict type tests, so they
    /// are the patriots' alone.
    pub patriot: Option<Patriot>,
}

/// A general's aura radius, in tiles.
///
/// This is the general's radius, not supply's: it drives their other bonuses
/// too. It lives here because supply is the first mechanic to need it, and it
/// moves the day a generals document exists.
///
/// `PARMENIO_RADIUS_ADJUST` is written `3/2` in `rules.xml` and applied by the
/// original as a multiply and a shift right by eight, which means the loader
/// stores it as 8.8 fixed point — 384. That is the one number in this module
/// inferred rather than read; two independent facts agree on it.
pub const fn general_radius(t: &Tuning, g: &General) -> i32 {
    // The original's arithmetic wraps at 32 bits (item 3396).
    let mut r = t.general_radius.wrapping_mul(g.upgrades.wrapping_add(3)) / 2;
    if g.parmenio {
        r = r.wrapping_mul(t.parmenio_radius_adjust) / 256;
    }
    if g.wellington {
        r = r.wrapping_mul(t.wellington_radius) / 100;
    }
    if g.kutosov {
        r = r.wrapping_mul(t.kutosov_radius) / 100;
    }
    if g.terra_cotta {
        r = r.wrapping_add(t.terra_cotta_range);
    }
    // The original tests the military and economic patriot sets separately.
    // A hero belongs to one of them, so an `Option` says the same thing.
    match g.patriot {
        Some(Patriot::Military) => r = r.wrapping_add(t.mil_patriot_radius_bonus),
        Some(Patriot::Economic) => r = r.wrapping_add(t.econ_patriot_radius_bonus),
        Some(Patriot::Both) => {
            r = r
                .wrapping_add(t.mil_patriot_radius_bonus)
                .wrapping_add(t.econ_patriot_radius_bonus);
        }
        None => {}
    }
    r
}

// ---------------------------------------------------------------------------
// The registry
// ---------------------------------------------------------------------------

/// What the query needs to know about a registered source, resolved from the
/// unit at query time exactly as the original resolves it.
///
/// The original's `Supply` record stores only the unit's index and reads the
/// rest live, so a wagon that moves, dies, or boards a transport takes effect
/// immediately with nothing to keep in sync. This mirrors that.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wagon {
    pub pos: Pos,
    /// `SubObjectData::is_active`.
    pub active: bool,
    /// `UnitData::is_on_map`. A wagon in a transport or a garrison supplies
    /// nothing.
    pub on_map: bool,
}

impl Wagon {
    pub const fn supplies(&self) -> bool {
        self.active && self.on_map
    }
}

/// One player's supply list.
///
/// The original keeps a `PtrArray<Supply>` per player and, separately, a
/// high-water count of used slots in `LeaderData`. Free slots inside the list
/// are reused; free slots at the end are trimmed. That is reproduced here
/// because slot indices are observable — the interface shows which source is
/// supplying a unit — and because it decides scan order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SupplyList {
    slots: Vec<Option<usize>>,
}

impl SupplyList {
    pub const fn new() -> SupplyList {
        SupplyList { slots: Vec::new() }
    }

    /// Registers a unit as a supply source and returns its slot.
    ///
    /// The first free slot wins, so a list that has churned does not grow.
    pub fn register(&mut self, unit: usize) -> usize {
        match self.slots.iter().position(Option::is_none) {
            Some(i) => {
                self.slots[i] = Some(unit);
                i
            }
            None => {
                self.slots.push(Some(unit));
                self.slots.len() - 1
            }
        }
    }

    /// Gives a slot back, and trims any free slots left at the end.
    pub fn close(&mut self, slot: usize) {
        if let Some(s) = self.slots.get_mut(slot) {
            *s = None;
        }
        while let Some(None) = self.slots.last() {
            self.slots.pop();
        }
    }

    /// Slots scanned by [`Self::find`], free ones included.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// The unit registered in a slot, if any.
    pub fn unit(&self, slot: usize) -> Option<usize> {
        self.slots.get(slot).copied().flatten()
    }

    /// The first source in slot order whose radius covers `at`.
    ///
    /// First match, not nearest — and since every source of a player shares one
    /// radius, the difference is only in which index comes back. `radius_tiles`
    /// is hoisted out of the loop for that reason: the original recomputes it
    /// per candidate from the slot's owner, which is the same number every time.
    ///
    /// The comparison is `<=`, so the boundary is inside.
    pub fn find<F>(&self, at: Pos, radius_tiles: i32, wagon: F) -> Option<usize>
    where
        F: Fn(usize) -> Option<Wagon>,
    {
        let reach = radius_tiles * UNITS_PER_TILE;
        for (slot, held) in self.slots.iter().enumerate() {
            let Some(unit) = *held else { continue };
            let Some(w) = wagon(unit) else { continue };
            if !w.supplies() {
                continue;
            }
            if vector_dist(at.x - w.pos.x, at.y - w.pos.y) <= reach {
                return Some(slot);
            }
        }
        None
    }
}

/// The three hero types whose aura supplies, in the order the original tries
/// them. The order is not observable — the caller only asks whether some
/// source was found — but it is free to keep and it names them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SupplyGeneral {
    Darius,
    Kutosov,
    Chandragupta,
}

/// A general standing on the map, with the radius [`general_radius`] gives it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeneralAura {
    pub kind: SupplyGeneral,
    pub pos: Pos,
    pub radius_tiles: i32,
    pub active: bool,
    pub on_map: bool,
}

/// One player's supply network: what their sources reach, who they are, and
/// which generals are standing where.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Network {
    pub player: PlayerSupply,
    pub list: SupplyList,
    pub generals: Vec<GeneralAura>,
}

impl Network {
    /// Whether anything of this player's supplies a unit standing at `at`.
    ///
    /// Wagons first, then generals — the original's order, and the only thing
    /// the caller does with the answer is test it.
    ///
    /// `upgrades` is `LeaderData::get_supply_upgrade` asked now
    /// ([`crate::Sim::supply_upgrade_level`]), never a stored copy: the
    /// radius reads the research the player holds on the frame it is
    /// asked (`SupplyData::get_radius`). Item 3641: a stored count in [`PlayerSupply`]
    /// had no writer, so every wagon reached 14 tiles. run1299's Bombard
    /// `1/98` fired on 5324 with who=1's Supply Wagon `1/102` 2737 units
    /// off — out at 14 tiles (2688), in at the 16 Forage gives (who=1
    /// researched it on 5192) — and
    /// theirs reloads 60 where ours took the out-of-supply 120.
    pub fn supplies<F>(&self, t: &Tuning, upgrades: i32, at: Pos, wagon: F) -> bool
    where
        F: Fn(usize) -> Option<Wagon>,
    {
        let radius = supply_radius(t, upgrades, self.player.terra_cotta);
        self.list.find(at, radius, wagon).is_some() || find_general(&self.generals, at).is_some()
    }
}

/// Everything about a player that the supply network reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
///
/// The steps of the supply upgrade chain are not here: they are research,
/// asked when the radius is ([`Network::supplies`]). A stored copy stood
/// here with no writer until item 3641.
pub struct PlayerSupply {
    pub terra_cotta: bool,
    /// The nation bonus that makes supply heal.
    pub heal_bonus: bool,
    pub versailles: bool,
}

impl PlayerSupply {
    pub const fn heal_period(&self, t: &Tuning) -> i32 {
        heal_period(t, self.heal_bonus, self.versailles)
    }
}

/// The first supplying general covering `at`, tried in the original's order.
///
/// The original's `ObjectData::has_general` matches the asking unit itself
/// before it looks at the hero list: a hero of the type asked for returns its
/// own index with no distance test. So the three supplying generals are always
/// supplied — which matters, because heroes are not exempt from attrition.
/// Scanning the list at distance 0 reaches the same answer for every general
/// who is active and on the map, and one who is not would not be processed.
pub fn find_general(generals: &[GeneralAura], at: Pos) -> Option<SupplyGeneral> {
    for kind in [
        SupplyGeneral::Darius,
        SupplyGeneral::Kutosov,
        SupplyGeneral::Chandragupta,
    ] {
        let covered = generals.iter().any(|g| {
            g.kind == kind
                && g.active
                && g.on_map
                && vector_dist(at.x - g.pos.x, at.y - g.pos.y) <= g.radius_tiles * UNITS_PER_TILE
        });
        if covered {
            return Some(kind);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Consumer 2: healing
// ---------------------------------------------------------------------------

/// The period, in frames, at which a supplied unit repairs damage. Zero means
/// never.
///
/// **Zero is the shipped answer.** `SUPPLY_HEAL_RATE` is 0 and its own
/// annotation says `0 means don't heal at all`, so supply heals nobody by
/// default. It heals only for a player with the nation bonus (20 frames), or
/// with Versailles (20), or with both — where the fold gives 10, the fastest
/// supply healing in the game.
///
/// The fold is written as though these were rates and they are periods, so
/// adding the nation bonus to a nonzero base would make healing *slower*. It
/// is only correct because the base ships at zero.
///
/// This is the period alone. The heal it drives is gated on the unit being a
/// captain, damaged, not garrisoned, land, and not itself a supply unit —
/// `docs/SUPPLY.md`, consumer 2. Nothing here applies the heal yet, so those
/// gates have nowhere to live.
pub const fn heal_period(t: &Tuning, nation_bonus: bool, versailles: bool) -> i32 {
    let mut base = t.supply_heal_rate;
    if nation_bonus {
        base += t.french_supply_heal_rate;
    }
    if versailles && t.versailles_supply_heal_rate != 0 {
        if base == 0 {
            t.versailles_supply_heal_rate
        } else {
            (t.versailles_supply_heal_rate + base) / 4
        }
    } else {
        base
    }
}

// ---------------------------------------------------------------------------
// Consumer 3: reload
// ---------------------------------------------------------------------------

/// Whether a unit's reload cares about supply, and how much.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SiegeClass {
    /// Reload never cares about supply.
    #[default]
    None,
    /// Out of supply, reloads at 3/2 the delay.
    Siege,
    /// The Bombard line. Out of supply, reloads at twice the delay.
    Artillery,
}

/// The reload query, which is **not** the attrition query.
///
/// Three clauses attrition has no equivalent of:
///
/// - **Anything that is not a land unit is always in supply.** The original
///   tests the type's `domain` field — the same `ObjectTypeData + 0x218` that
///   decides which attrition period a unit can reach, which is why this takes
///   [`Domain`] rather than a second name for the same three values.
/// - **Your own territory supplies you**, with no wagon anywhere.
/// - And only *your own*: the test is against the unit's own player, so
///   allied territory does not supply you either. A siege train fighting on an
///   ally's land reloads at the out-of-supply rate unless it brings a wagon.
pub fn in_supply(domain: Domain, ground: Owner, owner: Player, found: bool) -> bool {
    if !matches!(domain, Domain::Land) {
        return true;
    }
    if matches!(ground, Owner::Player(p) if p == owner) {
        return true;
    }
    found
}

/// A unit's reload delay in frames, given its base and its supply state.
///
/// `ARTILLERY_UNDER_ATTACK_FIRES_SLOWLY` ships enabled, so a siege unit that is
/// currently under attack is treated as out of supply wherever it stands.
///
/// The `3/2` and the `2` are literals in the original.
/// `SIEGE_OUT_OF_SUPPLY_RELOAD` and `ARTILLERY_OUT_OF_SUPPLY_RELOAD` exist in
/// `rules.xml`, are loaded, and are not read here — their annotations describe
/// the literals rather than feeding them, and editing them in a mod changes
/// nothing. They are deliberately absent from [`Tuning`]: an entry that cannot
/// change the result would be a lie about what the simulation reads.
pub const fn reload_frames(
    t: &Tuning,
    base: i32,
    class: SiegeClass,
    under_attack: bool,
    in_supply: bool,
) -> i32 {
    if matches!(class, SiegeClass::None) {
        return base;
    }
    let forced_out = under_attack && t.artillery_under_attack_fires_slowly != 0;
    if !forced_out && in_supply {
        return base;
    }
    match class {
        SiegeClass::Artillery => base * 2,
        _ => base * 3 / 2,
    }
}

impl crate::Sim {
    /// `LeaderData::get_spy_upgrade@006e1090`: how many of `SPIES_UPGRADE_1..3`
    /// the player holds.
    pub(crate) fn spy_upgrade_level(&self, who: Player) -> i32 {
        let p = &self.tech[who as usize];
        self.tables
            .tech_tree
            .roles
            .spy_upgrade_preq
            .iter()
            .flatten()
            .filter(|&&t| self.tables.tech_tree.has_tech(&self.tables.setup, p, t))
            .count() as i32
    }

    /// `LeaderData::get_general_upgrade@006e0830`: how many of
    /// `GENERALS_UPGRADE_1..3` the player holds — every one, not a leading
    /// run (`docs/AI.md` §172). A hero's speed, hits and sight climb with
    /// it, and so do its radius, its march, its ambush and its decoys.
    pub(crate) fn general_upgrade_level(&self, who: Player) -> i32 {
        let Some(p) = self.tech.get(who as usize) else {
            return 0;
        };
        self.tables
            .tech_tree
            .roles
            .general_upgrade_preq
            .iter()
            .flatten()
            .filter(|&&t| self.tables.tech_tree.has_tech(&self.tables.setup, p, t))
            .count() as i32
    }

    /// `ObjectData::is(0x3a, 0)` on a unit type: the Spy line.
    pub(crate) fn is_spy_type(&self, rec: usize) -> bool {
        self.tables.unit_types[rec].tree.is_some_and(|t| {
            self.tables.tech_tree.types.get(0x3a).is_some()
                && self.tables.tech_tree.is(t, 0x3a, false)
        })
    }

    /// `LeaderData::get_supply_upgrade@006e0880`: count every held
    /// supply-upgrade prerequisite, not a leading run.
    pub(crate) fn supply_upgrade_level(&self, who: Player) -> i32 {
        let p = &self.tech[who as usize];
        self.tables
            .tech_tree
            .roles
            .supply_upgrade_preq
            .iter()
            .flatten()
            .filter(|&&t| self.tables.tech_tree.has_tech(&self.tables.setup, p, t))
            .count() as i32
    }
}

#[cfg(test)]
mod tests;
