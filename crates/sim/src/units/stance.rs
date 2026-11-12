//! The stance a unit is born with — `docs/ORDERS.md` §5.10.
//!
//! `UnitData::stance` (`+0xb1`) is one byte and it means four different
//! things depending on what kind of unit carries it. `Unit::init@00612100`
//! asks the type which of the four it is and then reads a *different* place
//! for each: two of them come from the player's own options, one from the
//! lobby, one from the leader's flags.
//!
//! This crate wrote a flat `1` on every unit it created until 2026-09-03,
//! which is right for exactly one of the five arms.

use crate::Player;

/// `StanceTypes` — the PDB's enum, values and all
/// (`rise.pdb`, `LF_ENUM StanceTypes`, field list `0x1DE9`).
///
/// The numbers matter: they are the `switch` labels in
/// `Unit::init@00612100:282–309`, and `GroupData::get_stance_option@0070bab0`
/// uses the same enum to size the option array — 6 options for a combat
/// unit, 4 for a worker, 2 each for a caster and a packer. So a stance is
/// always an index into a list whose length its *type* fixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StanceType {
    /// `STANCE_COMBAT = 0`.
    Combat,
    /// `STANCE_WORKER = 1`.
    Worker,
    /// `STANCE_CASTER = 2`.
    Caster,
    /// `STANCE_PACKER = 3`.
    Packer,
    /// `STANCE_NONE = -1`, and `Unit::init`'s `default:` arm.
    None,
}

/// One player's `LeaderOption` — `LeaderOptions::list[who]`, 0x20 bytes, and
/// the PDB names every field of it (`LeaderOptionData`; the four the log
/// prints are `who`, `peasants`, `peasants_wait`, `buildings`, read out of
/// the executable's own UTF-16 literals at `0xae1628`, `0xae1630`,
/// `0xae156c`, `0xae1668`).
///
/// The defaults are `LeaderOptions::init@006f1d40`'s, which runs once per
/// game for all ten slots: `peasants` 0, `peasants_wait` 2, `buildings` 0,
/// and a bitmask cleared and then given bits **1 and 3**. Nothing else
/// writes them without a player command — `CommandPackage::process_leader_options`
/// and `ScenarioFuncSet::set_auto_peasant_level` are the two writers, and no
/// capture on disk issues either — so for every run measured so far these
/// values *are* the options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaderOptions {
    /// `+0x4` — the auto-peasant level, 0..3, which
    /// `ScenarioFuncSet::set_auto_peasant_level@009ff620` bounds and then
    /// pushes onto every citizen the player already owns. It is a **human's**
    /// worker stance; an AI never reads it.
    pub peasants: i32,
    /// `+0x8` — the idle-citizen **option index**, not the threshold itself:
    /// [`LeaderOptions::idle_wait`] is the table it selects a row of.
    pub peasants_wait: i32,
    /// `+0xc` — the combat stance every new **combat** unit takes. Named for
    /// buildings and used for units; the name is the PDB's, not a reading of
    /// it.
    pub buildings: i32,
    /// `+0x1c` — `BitMask`'s inline storage, the low byte of which is all
    /// `Unit::init` ever reads: bit 3 for a packer, bit 4 for a caster, and
    /// **complemented** in both cases, so a set bit means stance 0.
    pub flags: u8,
}

impl Default for LeaderOptions {
    /// `LeaderOptions::init@006f1d40`.
    fn default() -> LeaderOptions {
        LeaderOptions {
            peasants: 0,
            peasants_wait: 2,
            buildings: 0,
            // `*pbVar1 |= 2` then `*pbVar1 |= 8` — bits 1 and 3. Bit 4 is
            // clear, which is why a caster is born at 1 and a packer at 0.
            flags: 0b0000_1010,
        }
    }
}

impl LeaderOptions {
    /// Bit `n` of the option bitmask's first byte.
    const fn bit(self, n: u32) -> bool {
        self.flags & (1 << n) != 0
    }

    /// `peasants_wait`'s **switch**, not its value: the number of idle
    /// frames a *human's* worker or caravan waits before it looks for work
    /// again (`docs/ORDERS.md` §21.1).
    ///
    /// `think_peasant@005f5760:16` and `think_caravan@005f5650:10` are the
    /// same six lines twice over — `switch (leader_options->list[who] +0x8)`
    /// with arms `1 → 7`, `2 → 0xc`, `3 → 0x11`, `4 → 0x20`, `5 → 0x3e` and
    /// **`default → 2`** — so the option is an index into five waits and 2
    /// is what an *out-of-range* index falls back to.
    ///
    /// The distinction is the whole of item 494. `LeaderOptions::init`'s own
    /// default for the field is **2**, which selects `0xc`: every capture on
    /// disk prints `peasants_wait 2`, so the wait every human in every run
    /// has actually had is **12** frames, and this crate read the index as
    /// the wait and used 2.
    #[must_use]
    pub const fn idle_wait(self) -> u8 {
        match self.peasants_wait {
            1 => 7,
            2 => 12,
            3 => 17,
            4 => 32,
            5 => 62,
            _ => 2,
        }
    }
}

/// `UnitTypeData::get_stance_type@0061d350` — which of the four kinds of
/// stance this type carries.
///
/// The order of the tests is the whole of it, and it is not the order the
/// names suggest: **military first**, so a packing military unit is a packer
/// and every other military unit is `Combat`; then the worker type indices,
/// which is the same `0x32..=0x35` set
/// `ScenarioFuncSet::set_auto_peasant_level` walks; and only what is left
/// can be a caster.
///
/// The military arm's own return is worth spelling out, because the
/// decompiler writes it as arithmetic: `-(uint)((unit_flags2 & 4) != 0) &
/// STANCE_PACKER` is `Packer` when the type packs and **`Combat`** — 0, not
/// `None` — when it does not.
#[must_use]
pub const fn stance_type(role: u32, unit_flags2: u32, type_index: i32) -> StanceType {
    if role & crate::ai::ai_load::role::ROLE_MILITARY != 0 {
        if unit_flags2 & crate::ai::ai_load::uflags2::PACKS != 0 {
            return StanceType::Packer;
        }
        return StanceType::Combat;
    }
    if type_index >= 0x32 && type_index <= 0x35 {
        return StanceType::Worker;
    }
    // `(unit_flags2 & 6) == 2`: a caster that does **not** also pack. The
    // packing casters are picked off by the military arm above, so this is
    // the residue.
    if unit_flags2 & (crate::ai::ai_load::uflags2::CASTER | crate::ai::ai_load::uflags2::PACKS)
        == crate::ai::ai_load::uflags2::CASTER
    {
        return StanceType::Caster;
    }
    StanceType::None
}

/// `Unit::init@00612100:282–309` — the byte itself.
///
/// `human` is `leader_flags & 4`, the same bit [`crate::cities::city::Nation::human`]
/// already carries, and `resources_unlimited` is `starting_resources == 8`
/// read **raw** off the lobby: `Unit::init` takes `game->info` directly and
/// not `get_starting_resources(who)`, so the asymmetric-teams row never
/// reaches this.
///
/// The worker arm is the one that matters and it is inverted from what the
/// name suggests: it is the **AI** that takes the lobby's number, and a human
/// that takes their own option.
#[must_use]
pub const fn init_stance(
    kind: StanceType,
    options: LeaderOptions,
    human: bool,
    resources_unlimited: bool,
) -> u8 {
    match kind {
        StanceType::Combat => options.buildings as u8,
        StanceType::Worker => {
            if human {
                options.peasants as u8
            } else {
                // `(starting_resources == 8) + 1`.
                if resources_unlimited { 2 } else { 1 }
            }
        }
        // `~(flags >> 4) & 1` and `~(flags >> 3) & 1`.
        StanceType::Caster => !options.bit(4) as u8,
        StanceType::Packer => !options.bit(3) as u8,
        StanceType::None => 0,
    }
}

/// `UnitData::get_combat_stance@00610a30`, `get_worker_stance@006109f0`,
/// `get_caster_stance@006109b0` and `get_packer_stance@00610970` — one
/// body four times (item 2072): the unit's `stance` byte (`+0xb1`),
/// **sign-extended** (`movsx`), when its type has stance type `kind`
/// (`holds`, the type's `has_stance_type(kind)`, which each getter asks
/// through the unit's vtable slot `+0x104` or, devirtualised, the type
/// record's `+0x134`), and otherwise the kind's default: `COMBAT_STAND_GROUND`
/// 2, `WORKER_STAND` 3, `CASTER_NONE` 1, `PACKER_NEVER` 1 (the listing's
/// constants). `None` has no getter and answers 0. `BuildData`'s four —
/// `get_combat_stance@0062e0f0`, `get_worker_stance@0062e0b0`,
/// `get_caster_stance@0062e070`, `get_packer_stance@0062e030` — are the same
/// body over the building's own `stance` byte (`+0x7e`), compared with the
/// same base `UnitData::has_stance_type` (item 2083).
#[must_use]
pub const fn get_stance(holds: bool, stance: u8, kind: StanceType) -> i32 {
    if holds {
        return stance as i8 as i32;
    }
    match kind {
        StanceType::Combat => 2,
        StanceType::Worker => 3,
        StanceType::Caster | StanceType::Packer => 1,
        StanceType::None => 0,
    }
} // absorbs BuildData::get_packer_stance; absorbs BuildData::get_caster_stance; absorbs BuildData::get_worker_stance; absorbs BuildData::get_combat_stance

/// `BuildTypeData::get_stance_type@006396c0` — the *building's* kind, which
/// is what decides whether a unit it trains inherits its stance at all.
///
/// A building that trains nothing has no stance kind. Among those that do,
/// the two lineage tests come first and the rest is a list of type indices
/// tested by **identity**, not through `FROM`.
///
/// The last line reads oddly and is worth keeping literal: `return (iVar2 !=
/// 0x208) - STANCE_WORKER` is `0` (`Combat`) for every training building that
/// is not the Missile Silo, and `-1` (`None`) for it.
#[must_use]
pub fn build_stance_type(types: &[crate::cities::build::BuildType], t: usize) -> StanceType {
    use crate::cities::build::{Ident, is_city, is_fort, is_training_building};
    if !is_training_building(types, t) {
        return StanceType::None;
    }
    if is_city(types, t) {
        return StanceType::Worker;
    }
    if is_fort(types, t) {
        return StanceType::Caster;
    }
    match types[t].ident {
        // `0x1ae` SIEGEFACTORY, `0x1af` FACTORY.
        Ident::SiegeFactory | Ident::Factory => StanceType::Packer,
        // `0x1b4` MARKET, `0x1a4` UNIVERSITY, `0x1bf` AIRBASE — and `0x208`
        // MISSILESILO through the arithmetic above.
        Ident::Market | Ident::University | Ident::Airbase | Ident::MissileSilo => StanceType::None,
        _ => StanceType::Combat,
    }
} // twin of BuildTypeData::get_stance_type

/// `Build::init@00629740:92–113` — a **building's** own stance byte
/// (`Build +0x7e`).
///
/// It is `Unit::init`'s switch with two arms rewritten, and the rewrites are
/// not cosmetic:
///
/// - **Worker.** `Unit::init` is `human ? peasants : (sr == 8) + 1`; this is
///   `(!human && sr == 8) ? 2 : peasants`. So a *building* takes the
///   player's own option in every lobby but the unlimited one, where a
///   unit's takes it only for a human. That single difference is why an AI's
///   **starting** citizens are born at 1 and every citizen it **trains** is
///   0: the trained one inherits its city's byte.
/// - **Caster.** A flat `1`, not `!bit4`.
#[must_use]
pub const fn build_init_stance(
    kind: StanceType,
    options: LeaderOptions,
    human: bool,
    resources_unlimited: bool,
) -> u8 {
    match kind {
        StanceType::Combat => options.buildings as u8,
        StanceType::Worker => {
            if !human && resources_unlimited {
                2
            } else {
                options.peasants as u8
            }
        }
        StanceType::Caster => 1,
        StanceType::Packer => !options.bit(3) as u8,
        StanceType::None => 0,
    }
}

impl crate::Sim {
    /// [`stance_type`] for a type in [`crate::Sim::unit_types`].
    #[must_use]
    pub fn type_stance_type(&self, ty: usize) -> StanceType {
        let t = &self.tables.unit_types[ty];
        stance_type(t.cols.role, t.cols.unit_flags2, t.type_index)
    }

    /// The stance `Unit::init` gives a unit of type `ty` created for `who`.
    #[must_use]
    pub fn init_stance(&self, who: Player, ty: usize) -> u8 {
        let human = self
            .cities
            .nation
            .get(who as usize)
            .is_some_and(|n| n.human);
        init_stance(
            self.type_stance_type(ty),
            self.leader_options(who),
            human,
            self.ai.lobby.resources_unlimited(),
        )
    }

    /// `Build +0x7e` for a building on the map, computed rather than stored.
    ///
    /// `Build::init` writes the byte once and only two functions ever change
    /// it — `CommandPackage::process_leader_options@009441d0` and
    /// `ScenarioFuncSet::set_auto_peasant_level@009ff620`, a player's click
    /// and a scenario call. Neither runs in any capture on disk and neither
    /// is modelled here, so the stored field and this function agree
    /// everywhere they can be compared. **SEAM**: give the sim either writer
    /// and the byte must become a field.
    #[must_use]
    pub fn build_stance(&self, b: usize) -> u8 {
        let bl = &self.buildings[b];
        let Some(t) = bl.ty else { return 0 };
        let human = self
            .cities
            .nation
            .get(bl.owner as usize)
            .is_some_and(|n| n.human);
        build_init_stance(
            build_stance_type(&self.tables.build_types, t),
            self.leader_options(bl.owner),
            human,
            self.ai.lobby.resources_unlimited(),
        )
    }

    /// `Build::train@0062f9b0:86–101` — the stance a unit trained out of
    /// building `b` actually carries.
    ///
    /// `Unit::init` gives it one, and then `train` **overwrites** it with the
    /// building's whenever the two stance *kinds* match. So a citizen out of
    /// a city takes the city's, a hoplite out of a barracks takes the
    /// barracks', and a unit whose kind its trainer does not share keeps the
    /// one it was born with — `None` here, because `train` then writes
    /// nothing at all: the byte stays whatever `Unit::init` left, the
    /// computer's Bark's RAID (`Unit::init@00612100:598-611`) included.
    #[must_use]
    pub fn trained_stance(&self, ty: usize, b: usize) -> Option<u8> {
        let t = self.buildings[b].ty?;
        (self.type_stance_type(ty) == build_stance_type(&self.tables.build_types, t))
            .then(|| self.build_stance(b))
    }

    /// `GameAccess::leader_options->list[who]`, or `LeaderOptions::init`'s
    /// defaults for a player outside the table.
    #[must_use]
    pub fn leader_options(&self, who: Player) -> LeaderOptions {
        self.roster
            .leader_options
            .get(who as usize)
            .copied()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests;
