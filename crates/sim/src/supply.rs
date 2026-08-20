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

use crate::tuning::Tuning;
use crate::world::{Owner, Player, Pos, UNITS_PER_TILE, vector_dist};

/// A unit type's supply category — the original's three-way switch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Category {
    /// Subject to the ordinary rules.
    #[default]
    Ordinary,
    /// Counts as in supply wherever it stands, for [`in_supply`].
    AlwaysSupplied,
    /// Never repairs damage.
    NeverHealed,
}

// ---------------------------------------------------------------------------
// The radius
// ---------------------------------------------------------------------------

/// A supply source's reach, in tiles.
///
/// `upgrades` is how many steps of the supply chain the owner holds, 0 to 3.
/// One of those steps is also granted outright by a nation bonus, so a player
/// with it counts the step whether or not they researched it; that resolution
/// happens before this is called.
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
    pub terra_cotta: bool,
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
    let mut r = t.general_radius * (g.upgrades + 3) / 2;
    if g.parmenio {
        r = r * t.parmenio_radius_adjust / 256;
    }
    if g.wellington {
        r = r * t.wellington_radius / 100;
    }
    if g.kutosov {
        r = r * t.kutosov_radius / 100;
    }
    if g.terra_cotta {
        r += t.terra_cotta_range;
    }
    // The original tests the military and economic patriot sets separately.
    // A hero belongs to one of them, so an `Option` says the same thing.
    match g.patriot {
        Some(Patriot::Military) => r += t.mil_patriot_radius_bonus,
        Some(Patriot::Economic) => r += t.econ_patriot_radius_bonus,
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
    pub fn supplies<F>(&self, t: &Tuning, at: Pos, wagon: F) -> bool
    where
        F: Fn(usize) -> Option<Wagon>,
    {
        self.list.find(at, self.player.radius(t), wagon).is_some()
            || find_general(&self.generals, at).is_some()
    }
}

/// Everything about a player that the supply network reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerSupply {
    /// Steps of the supply upgrade chain held, 0 to 3, with the nation bonus
    /// that grants one outright already folded in.
    pub upgrades: i32,
    pub terra_cotta: bool,
    /// The nation bonus that makes supply heal.
    pub heal_bonus: bool,
    pub versailles: bool,
}

impl PlayerSupply {
    pub const fn radius(&self, t: &Tuning) -> i32 {
        supply_radius(t, self.upgrades, self.terra_cotta)
    }

    pub const fn heal_period(&self, t: &Tuning) -> i32 {
        heal_period(t, self.heal_bonus, self.versailles)
    }
}

/// The first supplying general covering `at`, tried in the original's order.
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
/// Two clauses attrition has no equivalent of:
///
/// - **Your own territory supplies you**, with no wagon anywhere.
/// - And only *your own*: the test is against the unit's own player, so
///   allied territory does not supply you either. A siege train fighting on an
///   ally's land reloads at the out-of-supply rate unless it brings a wagon.
pub fn in_supply(category: Category, ground: Owner, owner: Player, found: bool) -> bool {
    if !matches!(category, Category::Ordinary) {
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

#[cfg(test)]
mod tests {
    use super::*;

    const T: Tuning = Tuning::RON;

    fn wagon(x: i32, y: i32) -> Wagon {
        Wagon {
            pos: Pos::new(x, y),
            active: true,
            on_map: true,
        }
    }

    #[test]
    fn a_plain_wagon_reaches_fourteen_tiles_and_a_researched_one_twenty() {
        assert_eq!(supply_radius(&T, 0, false), 14);
        assert_eq!(supply_radius(&T, 3, false), 20);
        // The wonder is wired in and ships contributing nothing.
        assert_eq!(supply_radius(&T, 0, true), 14);
    }

    #[test]
    fn the_radius_is_in_tiles_and_the_query_is_in_position_units() {
        let mut list = SupplyList::new();
        list.register(0);
        let at_tiles = |n: i32| Pos::new(n * UNITS_PER_TILE, 0);
        let found = |n: i32| list.find(at_tiles(n), 14, |_| Some(wagon(0, 0)));
        // Fourteen tiles out is inside; the comparison is `<=`.
        assert_eq!(found(14), Some(0));
        assert_eq!(found(15), None);
        // And a fraction of a tile past it is outside, which is the whole
        // reason the conversion has to be exact.
        assert_eq!(
            list.find(Pos::new(14 * UNITS_PER_TILE + 1, 0), 14, |_| Some(wagon(
                0, 0
            ))),
            None
        );
    }

    #[test]
    fn the_supply_radius_is_octagonal_like_a_border() {
        let mut list = SupplyList::new();
        list.register(0);
        let diag = |n: i32| Pos::new(n * UNITS_PER_TILE, n * UNITS_PER_TILE);
        // The metric charges 1.5x the leg on the diagonal against a true
        // 1.414x, so the reachable set is an octagon pulled in at the corners.
        // 9,9 is 12.73 tiles away and is charged 13.5, which still fits inside
        // 14. 10,10 is 14.14 away — barely outside a true circle — and is
        // charged 15, so it misses by a whole tile rather than by a tenth.
        assert_eq!(list.find(diag(9), 14, |_| Some(wagon(0, 0))), Some(0));
        assert_eq!(list.find(diag(10), 14, |_| Some(wagon(0, 0))), None);
    }

    #[test]
    fn a_wagon_off_the_map_supplies_nothing() {
        let mut list = SupplyList::new();
        list.register(7);
        let at = Pos::new(0, 0);
        assert_eq!(list.find(at, 14, |_| Some(wagon(0, 0))), Some(0));
        assert_eq!(
            list.find(at, 14, |_| Some(Wagon {
                on_map: false,
                ..wagon(0, 0)
            })),
            None
        );
        assert_eq!(
            list.find(at, 14, |_| Some(Wagon {
                active: false,
                ..wagon(0, 0)
            })),
            None
        );
    }

    #[test]
    fn slots_are_reused_from_the_front_and_trimmed_from_the_back() {
        let mut list = SupplyList::new();
        assert_eq!(list.register(10), 0);
        assert_eq!(list.register(11), 1);
        assert_eq!(list.register(12), 2);
        // A hole in the middle is refilled rather than appended to.
        list.close(1);
        assert_eq!(list.len(), 3);
        assert_eq!(list.register(13), 1);
        // Holes at the end are trimmed, and the trim walks down past every
        // free slot it meets rather than stopping at the first.
        list.close(2);
        list.close(1);
        assert_eq!(list.len(), 1);
        assert_eq!(list.unit(0), Some(10));
    }

    #[test]
    fn a_general_reaches_nine_tiles_and_kutosov_twenty_seven() {
        let plain = General::default();
        assert_eq!(general_radius(&T, &plain), 9);
        assert_eq!(
            general_radius(
                &T,
                &General {
                    kutosov: true,
                    ..plain
                }
            ),
            27
        );
        // Parmenio's 3/2 is stored as 8.8 fixed point: 9 * 384 / 256 = 13.
        assert_eq!(
            general_radius(
                &T,
                &General {
                    parmenio: true,
                    ..plain
                }
            ),
            13
        );
        assert_eq!(
            general_radius(
                &T,
                &General {
                    wellington: true,
                    ..plain
                }
            ),
            18
        );
        // The upgrade chain: 6 * (n + 3) / 2.
        assert_eq!(
            general_radius(
                &T,
                &General {
                    upgrades: 3,
                    ..plain
                }
            ),
            18
        );
        assert_eq!(
            general_radius(
                &T,
                &General {
                    patriot: Some(Patriot::Military),
                    ..plain
                }
            ),
            12
        );
        assert_eq!(
            general_radius(
                &T,
                &General {
                    patriot: Some(Patriot::Economic),
                    ..plain
                }
            ),
            10
        );
    }

    #[test]
    fn generals_are_tried_in_the_originals_order() {
        let aura = |kind, x| GeneralAura {
            kind,
            pos: Pos::new(x, 0),
            radius_tiles: 9,
            active: true,
            on_map: true,
        };
        let both = [
            aura(SupplyGeneral::Chandragupta, 0),
            aura(SupplyGeneral::Darius, 0),
        ];
        assert_eq!(
            find_general(&both, Pos::new(0, 0)),
            Some(SupplyGeneral::Darius)
        );
        // Out of range of everything.
        assert_eq!(find_general(&both, Pos::new(20 * UNITS_PER_TILE, 0)), None);
    }

    #[test]
    fn supply_heals_nobody_in_the_shipped_game() {
        assert_eq!(heal_period(&T, false, false), 0);
        assert_eq!(heal_period(&T, true, false), 20);
        assert_eq!(heal_period(&T, false, true), 20);
        // The fold, and the fastest supply healing there is.
        assert_eq!(heal_period(&T, true, true), 10);
    }

    #[test]
    fn out_of_supply_costs_siege_a_half_and_artillery_a_whole_reload() {
        let f =
            |class, under_attack, supplied| reload_frames(&T, 60, class, under_attack, supplied);
        assert_eq!(f(SiegeClass::None, false, false), 60);
        assert_eq!(f(SiegeClass::Siege, false, true), 60);
        assert_eq!(f(SiegeClass::Siege, false, false), 90);
        assert_eq!(f(SiegeClass::Artillery, false, false), 120);
        // Under attack, a siege unit fires as if out of supply wherever it is.
        assert_eq!(f(SiegeClass::Siege, true, true), 90);
        assert_eq!(f(SiegeClass::None, true, true), 60);
        // The 3/2 truncates.
        assert_eq!(reload_frames(&T, 61, SiegeClass::Siege, false, false), 91);
    }

    #[test]
    fn your_own_ground_supplies_you_and_your_allys_does_not() {
        let c = Category::Ordinary;
        assert!(in_supply(c, Owner::Player(2), 2, false));
        assert!(!in_supply(c, Owner::Player(3), 2, false));
        assert!(!in_supply(c, Owner::None, 2, false));
        assert!(in_supply(c, Owner::None, 2, true));
        // And a type that is always supplied never asks.
        assert!(in_supply(
            Category::AlwaysSupplied,
            Owner::Player(3),
            2,
            false
        ));
    }
}
