//! `leaders/` — the files `docs/STRUCTURE.md` §3 maps here (item 4012).

pub mod capital;
pub mod diplomacy;
pub mod endgame;
pub mod finished;
pub mod marks;
pub mod rates;
pub mod relations;
pub mod resign;
pub mod score;
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
