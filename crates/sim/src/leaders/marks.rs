//! **The slot marks, the build counters and the queued bins** — item 4167,
//! step II of the leaders line: `fort_mark`, `oil_well_mark`,
//! `supply_mark`, `wonders_built`, `wonders_held`, `senates_built` and
//! `forts_built` (`docs/LEADERS.md` §13), and the six `*_queued` counters
//! and `scholar_militia` (§14).
//!
//! Each is a leaf of `LeaderData` the leaders line of the sync checksum
//! walks (`docs/CHECKSUM.md` §20). All but `wonders_built`, `wonders_held`
//! and `forts_built` are printed by `LeaderData::log_data@006e5110`, so
//! `rondata::diff::leader` compares them on every kept dump that prints
//! the record; those three have the checksum line alone.
//!
//! - **The two registries** ([`SlotList`], the shape `Specials` and
//!   `Heroes` have): `Forts::init_fort@0073ed70` and
//!   `init_oil_well@0073f6f0` take the first free record below the mark,
//!   else the mark's own, and raise the mark past it; `close_fort@0073ec90`
//!   and `close_oil_well@0073f670` clear the record and walk the mark down
//!   past every free one at the top. The building keeps its record's index
//!   (`Build +0x78`), so the give-back finds it and a building that never
//!   took one gives nothing back.
//! - **The counters** are stepped at `Build::activate@00623e20`'s own
//!   tests ([`Sim::note_activation_marks`]) and never decremented.
//! - **The queued bins** are `Leader::track_queued@006e0f30`'s rule, stepped
//!   beside every `num_queued` move ([`Sim::step_armed_queued`]).

use crate::units::slots::SlotList;
use crate::{Player, Sim};

/// `wonders_built`'s range: `TypeIndex` `0x20e..=0x21e`, Pyramids to Space
/// Program — the order of [`crate::tech::Roles::wonder_line`].
const WONDERS: usize = 17;

/// The six bins of `Leader::track_queued`, in the record's order: `+0xa10`
/// barracks, `+0xa14` stable, `+0xa18` factory, `+0xa1c` combat, `+0xa20`
/// dock, `+0xa24` air.
pub const ARMED_QUEUED: [&str; 6] = [
    "barracks_queued",
    "stable_queued",
    "factory_queued",
    "combat_queued",
    "dock_queued",
    "air_queued",
];

/// One leader's §13 and §14 state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    /// The fort records (`fort_mark`, `+0x428`).
    pub forts: SlotList,
    /// The oil-well and platform records (`oil_well_mark`, `+0x430`).
    pub oil_wells: SlotList,
    /// `wonders_built` (`+0x854`): wonder activations since the reset.
    pub wonders_built: i32,
    /// `wonders_held` (`+0x858`): the most wonder records live at once.
    pub wonders_held: i32,
    /// `senates_built` (`+0x85c`): Senates activated in a city, uncaptured.
    pub senates_built: i32,
    /// `forts_built` (`+0x8f0`): fort activations.
    pub forts_built: i32,
    /// `+0xa10..=+0xa24`, in [`ARMED_QUEUED`]'s order.
    pub queued: [i32; 6],
}

/// `track_queued`'s step 5 on one counter: `if !(c == 0 && d < 0) { c += d }`
/// — never a clamp, so a counter already below zero keeps falling
/// (`docs/LEADERS.md` §14.2).
pub fn step_bin(c: &mut i32, d: i32) {
    if !(*c == 0 && d < 0) {
        *c += d;
    }
}

/// `track_queued`'s step 4: which of the six bins an armed unit type steps,
/// from its trainer's `TypeIndex` (`where`, `+0x40`, compared by equality)
/// and whether its domain is air (`+0x218 == 2`). `[barracks, stable,
/// factory, combat, dock, air]`.
pub fn bins_of(trainer: Option<i64>, air: bool) -> [bool; 6] {
    match trainer {
        Some(0x1ab) => [true, false, false, true, false, false],
        Some(0x1ac) => [false, true, false, true, false, false],
        Some(0x1ae) => [false, false, true, false, false, false],
        Some(0x1b0) => [false, false, false, false, true, false],
        _ => [false, false, false, false, false, air],
    }
}

impl Sim {
    /// One leader's [`Marks`], or the clear's for a slot out of range.
    fn marks(&self, who: Player) -> Marks {
        self.leaders
            .marks
            .get(usize::from(who))
            .cloned()
            .unwrap_or_default()
    }

    /// `fort_mark` (`+0x428`).
    pub fn leader_fort_mark(&self, who: Player) -> i32 {
        self.marks(who).forts.mark as i32
    }

    /// `oil_well_mark` (`+0x430`).
    pub fn leader_oil_well_mark(&self, who: Player) -> i32 {
        self.marks(who).oil_wells.mark as i32
    }

    /// `supply_mark` (`+0x434`): the supply list's length, since
    /// [`crate::territory::supply::SupplyList`] trims its free tail exactly
    /// as `close_supply@0073acc0` walks the mark down, and takes the first
    /// free slot as `init_supply@0073ad40` does (`docs/LEADERS.md` §13.2).
    pub fn leader_supply_mark(&self, who: Player) -> i32 {
        self.territory
            .supply
            .get(usize::from(who))
            .map_or(0, |n| n.list.len() as i32)
    }

    /// `wonders_built` (`+0x854`).
    pub fn leader_wonders_built(&self, who: Player) -> i32 {
        self.marks(who).wonders_built
    }

    /// `wonders_held` (`+0x858`).
    pub fn leader_wonders_held(&self, who: Player) -> i32 {
        self.marks(who).wonders_held
    }

    /// `senates_built` (`+0x85c`).
    pub fn leader_senates_built(&self, who: Player) -> i32 {
        self.marks(who).senates_built
    }

    /// `forts_built` (`+0x8f0`).
    pub fn leader_forts_built(&self, who: Player) -> i32 {
        self.marks(who).forts_built
    }

    /// One of the six `*_queued` counters by its record name.
    pub fn leader_armed_queued(&self, who: Player, name: &str) -> Option<i32> {
        let i = ARMED_QUEUED.iter().position(|n| *n == name)?;
        Some(self.marks(who).queued[i])
    }

    /// Whether building type `ty` is one of the 17 wonders
    /// `Build::activate` counts (`TypeIndex` `0x20e..=0x21e`).
    pub fn is_counted_wonder(&self, ty: usize) -> bool {
        let line = &self.tables.tech_tree.roles.wonder_line;
        self.tables.build_types[ty]
            .tree
            .is_some_and(|t| line.iter().take(WONDERS).any(|&w| w == t))
    }

    /// **`Build::activate@00623e20`'s marks and counters**, on every
    /// activation, the captured and the city branch included (they sit
    /// ahead of the split; `docs/LEADERS.md` §13.2):
    ///
    /// - a wonder: `wonders_built += 1`, then `Wonders::init_wonder`'s
    ///   `wonders_held = max(wonders_held, live wonder records)`, the new
    ///   one counted;
    /// - `is(FORTX)`: `Forts::init_fort` takes a fort record and
    ///   `Fort::init` steps `forts_built`;
    /// - `is(OILWELL)` or `is(OILPLATFORM)`: `init_oil_well` takes a record.
    pub(crate) fn note_activation_marks(&mut self, b: usize, ty: usize) {
        let who = self.buildings[b].owner;
        let w = usize::from(who);
        if w >= self.leaders.marks.len() {
            return;
        }
        if self.is_counted_wonder(ty) {
            let live = (0..self.buildings.len())
                .filter(|&x| {
                    let bd = &self.buildings[x];
                    bd.alive
                        && bd.active
                        && bd.owner == who
                        && bd.ty.is_some_and(|t| self.is_counted_wonder(t))
                })
                .count() as i32;
            let m = &mut self.leaders.marks[w];
            m.wonders_built += 1;
            m.wonders_held = m.wonders_held.max(live);
        }
        let types = &self.tables.build_types;
        if crate::cities::build::is_fort(types, ty) {
            let m = &mut self.leaders.marks[w];
            self.buildings[b].fort_slot = Some(m.forts.take());
            m.forts_built += 1;
        }
        if crate::cities::build::is(types, ty, crate::cities::build::Ident::OilWell)
            || crate::cities::build::is(types, ty, crate::cities::build::Ident::OilPlatform)
        {
            self.buildings[b].oil_slot = Some(self.leaders.marks[w].oil_wells.take());
        }
    }

    /// **`Build::close@00628980`'s give-backs**: `Forts::close_fort` when
    /// the type `is_fort` and the building holds a record; `close_oil_well`
    /// only when the type is a gatherer (`build_flags & 0x40`) **and** a
    /// well or platform **and** the record is held (`docs/LEADERS.md`
    /// §13.2).
    pub(crate) fn give_back_marks(&mut self, b: usize) {
        let w = usize::from(self.buildings[b].owner);
        let Some(ty) = self.buildings[b].ty else {
            return;
        };
        if w >= self.leaders.marks.len() {
            return;
        }
        let types = &self.tables.build_types;
        let fort = crate::cities::build::is_fort(types, ty);
        let well = types[ty].flags & crate::cities::build::flags::GATHERER != 0
            && (crate::cities::build::is(types, ty, crate::cities::build::Ident::OilWell)
                || crate::cities::build::is(types, ty, crate::cities::build::Ident::OilPlatform));
        if fort && let Some(s) = self.buildings[b].fort_slot.take() {
            self.leaders.marks[w].forts.give_back(s);
        }
        if well && let Some(s) = self.buildings[b].oil_slot.take() {
            self.leaders.marks[w].oil_wells.give_back(s);
        }
    }

    /// `Build::activate`'s Senate arm, past its `city >= 0 && !captured`
    /// test: `senates_built += 1`.
    pub(crate) fn note_senate_built(&mut self, who: Player) {
        if let Some(m) = self.leaders.marks.get_mut(usize::from(who)) {
            m.senates_built += 1;
        }
    }

    /// **`Leader::track_queued@006e0f30`'s steps 3–5**, beside every move
    /// of `num_queued` for unit record `ty` by `d` (`docs/LEADERS.md`
    /// §14.2): an armed type (`attack != 0`) steps the bins its trainer and
    /// domain choose, each on its own zero test.
    pub(crate) fn step_armed_queued(&mut self, who: Player, ty: usize, d: i32) {
        let unit = &self.tables.unit_types[ty];
        if unit.combat.attack == 0 {
            return;
        }
        let trainer = unit
            .tree
            .and_then(|x| self.tables.tech_tree.types.get(x))
            .and_then(|t| t.where_)
            .map(|wh| self.tree_type_index(wh));
        let air = unit.combat.domain == crate::territory::attrition::Domain::Air;
        let bins = bins_of(trainer, air);
        let Some(m) = self.leaders.marks.get_mut(usize::from(who)) else {
            return;
        };
        for (c, on) in m.queued.iter_mut().zip(bins) {
            if on {
                step_bin(c, d);
            }
        }
    }

    /// `scholar_militia` (`+0x9f0`), derived from the live militia whose
    /// former type is a Scholar, as [`Sim::worker_support`] counts them.
    ///
    /// SEAM: the kept counter's stale-slot `+1` (`Unit::init`'s birth step
    /// against a recycled slot's `former_type`, `docs/LEADERS.md` §14.2) is
    /// not carried; the record prints 0 on every scored dump.
    pub fn leader_scholar_militia(&self, who: Player) -> i32 {
        self.scholar_militia(who)
    }
}

#[cfg(test)]
mod tests;
