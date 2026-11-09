//! A spell's row and its job times — `SpellType::get_job_time`'s arms
//! (`docs/STRUCTURE.md` §3, `spells/`).
//!
//! Moved out of `lib.rs` unchanged by item 4014 (`docs/STRUCTURE.md` §5, move 6).

use crate::*;

/// `TypeIndex` `HVYMACHINEGUN` — the lineage `get_job_time` halves the
/// machine gun's pack and unpack for (`is(0x7d, 0)`, [`Sim::cast_job_time`]).
const HVYMACHINEGUN: tech::TypeId = 0x7d;

/// `TypeIndex` `HOWITZER`, `KATYUSHA` and `MLRS` — the three types
/// `get_job_time` shortens the siege pack and unpack for, each asked
/// strictly (`is(x, 1)`, [`Sim::cast_job_time`]).
const HOWITZER: tech::TypeId = 0x10f;

const KATYUSHA: tech::TypeId = 0x116;

const MLRS: tech::TypeId = 0x111;

impl Sim {
    /// `GameAccess::spelltypes[spell]` — the craft's row, or `None` when
    /// the index is not a craft or the table was never loaded.
    pub fn spell(&self, spell: i32) -> Option<orders::SpellType> {
        let i = usize::try_from(spell - orders::spell::FIRST).ok()?;
        self.spells.get(i).copied()
    }

    /// `SpellTypeData::get_job_time(o, who)@00675800` — the row's own
    /// `JOB_TIME`, which is the whole answer for every craft this crate
    /// casts.
    ///
    /// SEAM, and it is a list rather than a shrug: the function adjusts
    /// nine of the fifty-five rows. `0x27d` Entrench takes the French
    /// tribe bonus and Antipater's rate; ~~`0x275` Bribe and `0x27f`
    /// Informer halve under `SPIES_CRAFT_FASTER`~~ ([`Sim::cast_job_time`],
    /// item 3985); `0x280`/`0x281`,
    /// Sabotage and Sniper, halve under a tribe bonus and flatten to 10 for
    /// one mask; and the siege pair's Turkish `turk_pack` and Napoleon's
    /// `napoleon_pack` — none of them reached, and neither constant loaded.
    /// ~~The siege pair's three lineages and the machine gun's one~~:
    /// [`Sim::cast_job_time`] (item 1608). The fishing boat's `0x292` and
    /// the transport `0x28a` are named by no arm, so the record's field is
    /// the number — run58's forty frames between the queue on 4948 and the
    /// unpack on 4989 (`docs/ORDERS.md` §6.9).
    pub fn spell_job_time(&self, spell: i32) -> i16 {
        self.spell(spell).map_or(0, |s| s.job_time)
    }

    /// `get_job_time`'s `0x27d` arm (`00675816`–`006758d1`, item 3919,
    /// `docs/GOLDEN.md` §166): a French caster's leader
    /// (`has_tribe_bonus(10)`) scales the time by `FRENCH_ENTRENCH` percent,
    /// and then **every** live caster takes `ANTIPATER_ENTRENCH_RATE` in 8.8
    /// (`cltd; and 0xff; sar 8`, toward zero). The listing calls `is(0x169,
    /// 0)` on the caster and never tests the answer, so the 1/2 is not
    /// Antipater's alone: run1513's General casts in 60 where the row says
    /// 120.
    pub(crate) fn entrench_job_time(&self, u: usize, t: i16) -> i16 {
        use crate::nations::power::FRENCH;
        let who = self.units[u].owner as usize;
        let mut t = i32::from(t);
        if self
            .tables
            .tech_tree
            .has_tribe_bonus(&self.tables.setup, &self.tech[who], FRENCH)
        {
            t = self.tables.tuning.french_entrench.wrapping_mul(t) / 100;
        }
        if self.units[u].alive() {
            t = self.tables.tuning.antipater_entrench_rate.wrapping_mul(t) / 256;
        }
        t as i16
    }

    /// `get_job_time(o, who)@00675800` for caster `u`: [`Sim::spell_job_time`]
    /// with the two type arms, read off the listing.
    ///
    /// - **The siege pair** `0x28b`/`0x28c` (`00675a10`–`00675ac1`): a
    ///   caster whose type `is(0x10f, 1)` (Howitzer) or `is(0x116, 1)`
    ///   (Katyusha) takes the row's `JOB_TIME` halved; otherwise one that
    ///   `is(0x111, 1)` (MLRS) takes it quartered (`cltd; and 3; sar 2`,
    ///   toward zero). The Catapult's 80 is 20 for an MLRS: the coverage
    ///   pair's `1/65` casts its unpack on 2879 and stands unpacked on
    ///   block 2900.
    /// - **The machine gun's** `0x28d`/`0x28e` (`00675ac3`–`00675b16`): a
    ///   caster whose type `is(0x7d, 0)` — the Heavy Machine Gun's lineage,
    ///   the Advanced Machine Gun in it — takes it halved. 50 is 25: the
    ///   pair's Advanced Machine Gun `1/74` casts on 2868 and stands
    ///   unpacked on block 2894 (`docs/AI.md` §170).
    pub(crate) fn cast_job_time(&self, u: usize, spell: i32) -> i16 {
        use orders::spell;
        let t = self.spell_job_time(spell);
        let exactly = |x: tech::TypeId| {
            self.unit_tree(u)
                .is_some_and(|ut| self.tables.tech_tree.is(ut, x, true))
        };
        match spell {
            spell::PACK | spell::UNPACK if exactly(HOWITZER) || exactly(KATYUSHA) => t / 2,
            spell::PACK | spell::UNPACK if exactly(MLRS) => t / 4,
            spell::PACK_MACHINEGUN | spell::UNPACK_MACHINEGUN
                if self.unit_line_is(u, HVYMACHINEGUN) =>
            {
                t / 2
            }
            spell::ENTRENCH => self.entrench_job_time(u, t),
            spell::BRIBE | spell::INFORMER if self.spies_craft_faster(u) => t / 2,
            _ => t,
        }
    }

    /// `get_job_time`'s spy arm (`0x275`, `0x27f`; item 3985,
    /// `docs/GOLDEN.md` §178): the caster's leader `has_preq
    /// (SPIES_CRAFT_FASTER)` — Strategy, `TECHBONUSES` row 88 — halves the
    /// hold. run1543's Informer, 40 frames in the row, lands 20 frames
    /// after its Spy stands.
    pub(crate) fn spies_craft_faster(&self, u: usize) -> bool {
        let who = self.units[u].owner as usize;
        self.tables
            .tech_tree
            .roles
            .spies_craft_faster_preq
            .is_some_and(|t| {
                self.tables
                    .tech_tree
                    .has_tech(&self.tables.setup, &self.tech[who], t)
            })
    }
}
