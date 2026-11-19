//! `leaders/` — the files `docs/STRUCTURE.md` §3 maps here (item 4012).

pub mod capital;
pub mod declare;
pub mod diplomacy;
pub mod endgame;
pub mod finished;
pub mod marks;
pub mod rates;
pub mod relations;
pub mod resign;
pub mod score;
pub mod tallies;
pub mod taunt;
pub mod tribute;
pub mod words;

#[cfg(test)]
mod flag_bits_tests;

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
    /// `leader_flags & 0x4000` per player: one of its heroes cast Ambush.
    /// `SpellType::cast_ambush@00671610:52` sets it and
    /// `Leader::verify_spell_flags@006ce190` clears it, on an Ambush's or a
    /// Forced March's expiry, once no hero's `unit_masks & 0x2000` stands
    /// (item 4400).
    pub ambush: Vec<bool>,
    /// Whether each player has been defeated — `leader_flags & 2` clear.
    pub defeated: Vec<bool>,
    /// Each slot's lobby team (`GameInfo::player[k].team` of the seat
    /// `LeaderData::get_player` finds; 0..3 a team, 8 none), as
    /// [`Sim::init_diplos`] reads it. `LeaderData::is_team@006ebd30` and
    /// so `num_team_members` read it in play (item 4159's step VII-d).
    pub team: Vec<i32>,
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
    /// `leader_flags & 0x80000`: a citizen of this slot (`TypeIndex` 0x32
    /// or 0x33) reached `Unit::think@005f6e40:85` past its captain test
    /// this frame. That store is the bit's one writer; its one clear is
    /// the last statement of a live slot's step in
    /// `Leaders::process_all@006ed2a0:75` (`Leader::process`'s twin at
    /// `:68` has no caller), so a checksum sees the bit on a frame some
    /// citizen of a live slot idled through `do_idle`, and a defeated
    /// slot keeps it (`docs/CHECKSUM.md` §25.4, item 4332).
    pub citizen_thought: Vec<bool>,
    /// `leader_flags & 0x20000`: the slot holds a unit of a `SPECIALFORCES`
    /// (`TypeIndex` 0x4b) type. `Leader::track_unit_type@006e0dd0:62–69` is
    /// its one writer: on every count move of a type that `is(0x4b)` it sets
    /// the bit while that type's own `num_units` is nonzero and clears it
    /// at zero, so the last move of any such type decides it
    /// (`docs/CHECKSUM.md` §23.8, item 4400).
    pub special_forces: Vec<bool>,
    /// `leader_flags & 0x40000`: some seated other slot, not allied both
    /// ways, holds [`Self::special_forces`]. `Leaders::process_all@006ed2a0:19–36`
    /// clears and re-derives it at the head of each live slot's step;
    /// `ObjectData::is_jammed@00653660` reads it (`docs/CHECKSUM.md` §23.8).
    pub jam_threat: Vec<bool>,
    /// The slot marks, the build counters and the queued bins
    /// (`docs/LEADERS.md` §13, §14; [`marks`], item 4167), one per slot.
    pub marks: Vec<marks::Marks>,
    /// The damage path's leaves: the kill, loss and raze counters, the
    /// victim and attrition stamps, the repair stamp, `missiles_used` and
    /// `nukes_launched` (`docs/LEADERS.md` §2, §4, §5, §15; [`tallies`],
    /// item 4176), one per slot.
    pub tallies: Vec<tallies::Tallies>,
    /// The combat rates: `frame_battle`, the four averages, the four
    /// window counts and the four totals (`docs/LEADERS.md` §3; [`rates`],
    /// item 4177), one per slot.
    pub rates: Vec<rates::Rates>,
    /// The end of the game and the lost capital: `defeated_by`, the defeat
    /// and victory words, the wonder clock and the capital clock
    /// (`docs/LEADERS.md` §10, §11; [`endgame`], [`capital`], item 4185),
    /// one per slot.
    pub endgame: Vec<endgame::EndGame>,
    /// The last building and unit finished, by type (`docs/LEADERS.md`
    /// §19; [`finished`], item 4187), one per slot.
    pub finished: Vec<finished::Finished>,
    /// The score, its eleven parts and the units part's dirty bit
    /// (`docs/LEADERS.md` §18; [`score`], item 4187), one per slot.
    pub score: Vec<score::Score>,
    /// The declaration's leaves: `dow`, `blacken` and `broke_alliance`
    /// (`docs/LEADERS.md` §7; [`declare`], item 4159's step VII-b1), one
    /// per slot.
    pub declare: Vec<declare::Declared>,
    /// The tribute leaves and the deal records: `tribute_stamp`,
    /// `gift_stamp`, `counteroffer`, `tribute_demanded`, `tributes`, the
    /// two totals and `dip[8]` (`docs/LEADERS.md` §8; [`tribute`], item
    /// 4159's step VII-b2), one per slot.
    pub tribute: Vec<tribute::Tribute>,
    /// The taunts: `last_taunt`, `taunt_frame` and the incoming ring
    /// (`docs/LEADERS.md` §9; [`taunt`], item 4159's step VII-b2), one per
    /// slot.
    pub taunts: Vec<taunt::Taunts>,
    /// `Game::semaphore[0] & 0x40`: `Game::check_victory@005926b0` has
    /// decided the game and is not asked again (`docs/LEADERS.md` §10.2;
    /// [`Sim::check_victory`], item 4189).
    pub victory_checked: bool,
    /// `Game::semaphore[2] & 0x40`: the game is over. `Game::do_frame`'s
    /// tail runs `Game::process_end_game@00591ce0` on it, whose modal end
    /// window is where a human quits once the game has ended
    /// (`docs/LEADERS.md` §10.2; item 4189).
    pub game_over: bool,
}

impl crate::Sim {
    /// `leader_flags & 0x40000` of live slot `who`, re-derived at its step's
    /// head (`Leaders::process_all@006ed2a0:19–36`; `Leader::process@006b88b0:16–29`
    /// is its uncalled twin): cleared, then raised when any seated slot
    /// other than `who` whose `diplos` with `who` is not 2 (allied) both
    /// ways holds [`LeadersState::special_forces`] (item 4400).
    pub(crate) fn refresh_jam_threat(&mut self, who: usize) {
        let seats = self
            .economy
            .ledgers
            .len()
            .min(crate::checksum::leaders::LEADER_SLOTS);
        // `diplos[a][b] == 2` as the checksum's leaders line emits it: allied
        // and not at war.
        let ally = |a: usize, b: usize| self.leaders.allied[a][b] && !self.leaders.at_war[a][b];
        let threat = (0..seats).any(|j| {
            j != who
                && !(ally(j, who) && ally(who, j))
                && self.leaders.special_forces.get(j).copied().unwrap_or(false)
        });
        if let Some(b) = self.leaders.jam_threat.get_mut(who) {
            *b = threat;
        }
    }
}
