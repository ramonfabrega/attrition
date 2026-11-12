//! `leaders/` — the files `docs/STRUCTURE.md` §3 maps here (item 4012).

pub mod diplomacy;
pub mod marks;
pub mod relations;
pub mod resign;
pub mod tallies;
pub mod words;

/// The `leaders/` subsystem's fields of [`crate::Sim`] (`docs/STRUCTURE.md`
/// §3, item 4016): moved whole, each with its doc, in `Sim`'s order.
#[derive(Clone, Debug)]
pub struct LeadersState {
    /// Diplomacy, as a full matrix. `at_war[a][b]` is symmetric in practice
    /// but stored both ways, because the original reads it both ways.
    pub at_war: Vec<Vec<bool>>,
    /// Alliance, as a matrix; `is_ally` needs it both ways.
    pub allied: Vec<Vec<bool>>,
    /// `LeaderData::treaties` (`+0x94`), `int[8]` — the standing agreements
    /// between two leaders, of which exactly one bit is modelled here:
    /// **bit 0, the met bit**, "these two have made contact".
    /// [`Sim::treaty_on`] is its only writer and [`Sim::has_met`] its only
    /// reader; `docs/VISION.md` §6.2 and `docs/AI.md` §46.
    pub treaties: Vec<Vec<i32>>,
    /// `LeaderData::agendas` (`+0xb4`), `int[8]`. One bit is modelled:
    /// **`2`, "your building stood in my armed unit's walk"**, which
    /// `Unit::resolve_block@005fccc0` ors into its owner's slot for a
    /// building of a player it is not at war with (`5fcdda`, item 1209);
    /// one direction only, where `treaty_on` writes both.
    pub agendas: Vec<Vec<i32>>,
    /// `leader_flags & 0x8000` per player: one of its heroes is on a
    /// Forced March. `cast_march` sets it and `Leader::verify_spell_flags`
    /// clears it once no hero's `unit_masks & 0x8000` stands; the hero
    /// arm of `think_spellcaster` throws no coin while it is up
    /// (`docs/AI.md` §99.13).
    pub forced_march: Vec<bool>,
    /// Whether each player has been defeated — `leader_flags & 2` clear.
    pub defeated: Vec<bool>,
    /// `LeaderData::city_name` (`+0x40c`): the cities this slot has named
    /// for its tribe, `City::generate_name`'s count ([`words`],
    /// `docs/LEADERS.md` §1).
    pub city_name: Vec<i32>,
    /// `LeaderData::anti_att` (`+0x7f4`), the record's IEEE single held as
    /// its bits: `Leader::calc_anti_attrition`'s recompute from 256.0
    /// ([`words`], `docs/LEADERS.md` §6).
    pub anti_att: Vec<u32>,
    /// `LeaderData::pop_issues` (`+0x7e8`): this frame's visits of a
    /// researched unit entry at the population cap — `Build::do_queue`'s
    /// and the carrier's — reset at a live leader's step
    /// (`docs/LEADERS.md` §12.2).
    pub pop_issues: Vec<i32>,
    /// The slot marks, the build counters and the queued bins
    /// (`docs/LEADERS.md` §13, §14; [`marks`], item 4167), one per slot.
    pub marks: Vec<marks::Marks>,
    /// The damage path's leaves: the kill, loss and raze counters, the
    /// victim and attrition stamps, the repair stamp, `missiles_used` and
    /// `nukes_launched` (`docs/LEADERS.md` §2, §4, §5, §15; [`tallies`],
    /// item 4176), one per slot.
    pub tallies: Vec<tallies::Tallies>,
}
