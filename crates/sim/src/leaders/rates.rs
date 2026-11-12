//! **The combat rates** — item 4177, step IV of the leaders line
//! (`docs/LEADERS.md` §3): `frame_battle`, the four `average_*_rate`, the
//! four `*_current_frame` and the four `*_fifteen_seconds` words of
//! `LeaderData` (`+0xa4c..+0xa66`).
//!
//! Four event writers feed the `_current_frame` words, each a live inlined
//! copy of a dead `Leader::inc_*_event`: the blow and the structure kill in
//! `Object::do_damage@0064a480` and a unit's countable death in
//! `Object::take_damage@00652020` (`crate::combat::fight::delivery`), and
//! the Sniper's unit arm of `SpellType::cast_sabotage@006720e0`. One fold,
//! `Leader::process_event_frame@006ec180`, runs every fiftieth frame at the
//! tail of `Game::do_frame` and writes everything else ([`Sim::fold_rates`]).
//!
//! Every word but `frame_battle` is a `ushort` and every add a 16-bit
//! wrapping add. All thirteen are printed by `LeaderData::log_data@006e5110`,
//! so `rondata::diff::leader` compares them on every dump that prints the
//! record.

use crate::{Player, Sim};

/// The fold's period: `process_event_frame` returns unless `frame % 50 == 0`
/// (§3.2, W5).
pub const FOLD_PERIOD: i64 = 50;

/// The battle event's lock-out: `frame − frame_battle >= 1800` (§3.2).
pub const BATTLE_LOCKOUT: i64 = 1800;

/// What the battle event writes to both `average_death_rate` and
/// `average_kill_rate` (`movl $0xfc18fc18`; §3.2).
pub const BATTLE_SENTINEL: u16 = 0xfc18;

/// The four kinds, in the record's order of the `_current_frame` and
/// `_fifteen_seconds` words.
pub const DEATHS: usize = 0;
/// `kills_*` (§3.2).
pub const KILLS: usize = 1;
/// `hits_*` (§3.2).
pub const HITS: usize = 2;
/// `damage_*` (§3.2).
pub const DAMAGE: usize = 3;

/// One leader's §3 words.
///
/// The arrays are indexed by kind ([`DEATHS`], [`KILLS`], [`HITS`],
/// [`DAMAGE`]). Note that the record's averages are laid out death, kill,
/// **damage**, hit (`+0xa50`, `+0xa52`, `+0xa54`, `+0xa56`), where the other
/// two rows are death, kill, hit, damage; [`Sim::leader_rate_leaf`] names
/// each by its own field.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rates {
    /// `frame_battle` (`+0xa4c`, `int`): the last battle event's frame, 0
    /// for never.
    pub frame_battle: i64,
    /// `average_*_rate`, by kind.
    pub average: [u16; 4],
    /// `*_current_frame`, by kind: the window's count, zeroed at each fold.
    pub current: [u16; 4],
    /// `*_fifteen_seconds`, by kind: despite the name, a game-long total
    /// mod 2^16 (only `Leader::init` clears it).
    pub fifteen: [u16; 4],
}

/// The leaves [`Sim::leader_rate_leaf`] answers, in the record's order.
pub const RATE_LEAVES: [&str; 13] = [
    "frame_battle",
    "average_death_rate",
    "average_kill_rate",
    "average_damage_rate",
    "average_hit_rate",
    "deaths_current_frame",
    "kills_current_frame",
    "hits_current_frame",
    "damage_current_frame",
    "deaths_fifteen_seconds",
    "kills_fifteen_seconds",
    "hits_fifteen_seconds",
    "damage_fifteen_seconds",
];

/// Of [`RATE_LEAVES`], the ones the leaders line emits unheld although the crate
/// holds them: the record diff parts each beside a parent that already
/// parts (4167b's rule; `docs/LEADERS.md` §3.5).
pub const RATES_OFF_THE_LINE: [&str; 0] = [];

/// One average's step 3 (§3.2): `c == 0 ? avg·7 >> 3 : (c + avg) >> 1`,
/// unsigned 32-bit on the zero-extended words, stored back as a `u16`.
pub fn fold_average(avg: u16, c: u16) -> u16 {
    let (a, c) = (u32::from(avg), u32::from(c));
    let v = if c == 0 { (a * 7) >> 3 } else { (c + a) >> 1 };
    // Both arms fit: `7·65535 >> 3` and `(65535 + 65535) >> 1` are under
    // 2^16.
    u16::try_from(v).unwrap_or(u16::MAX)
}

impl Rates {
    /// `X_current_frame += n`, 16-bit wrapping.
    pub fn add(&mut self, kind: usize, n: u16) {
        self.current[kind] = self.current[kind].wrapping_add(n);
    }

    /// **The fold, steps 1–3, 5 and 6** (`process_event_frame@006ec180`,
    /// §3.2), on a frame that is a multiple of 50; `age` is the leader's
    /// own 0-based `ages`. Step 4, the console's music, writes no leaf.
    pub fn fold(&mut self, frame: i64, age: i32) {
        for k in 0..4 {
            // 1. The raw count, then 2. the count ×100, stored back.
            self.fifteen[k] = self.fifteen[k].wrapping_add(self.current[k]);
            let c = self.current[k].wrapping_mul(100);
            self.current[k] = c;
            // 3. The average.
            self.average[k] = fold_average(self.average[k], c);
        }
        // 5. The battle test, signed `i32` on zero-extended words.
        let d = i32::from(self.average[DEATHS]);
        let k = i32::from(self.average[KILLS]);
        let open = self.frame_battle == 0 || frame - self.frame_battle >= BATTLE_LOCKOUT;
        if d + k >= (age + 1) * 125 && open {
            let lead = 10 * age + 20;
            // `Achieve::add_event(1 defeat | 0 victory, who, "")` is
            // unported and writes no leaf.
            if d >= k + lead || k >= d + lead {
                self.average[DEATHS] = BATTLE_SENTINEL;
                self.average[KILLS] = BATTLE_SENTINEL;
                self.frame_battle = frame;
            }
        }
        // 6. The window starts again.
        self.current = [0; 4];
    }
}

impl Sim {
    fn rates_mut(&mut self, who: Player) -> Option<&mut Rates> {
        // Gaia's slots 8 and 9 never fold (`docs/LEADERS.md` §3.6), and no
        // ordinary blow reaches them; the record keeps no row for them.
        if who >= 8 {
            return None;
        }
        self.leaders.rates.get_mut(usize::from(who))
    }

    /// One leader's [`Rates`], or the clear's for a slot out of range.
    pub fn leader_rates(&self, who: Player) -> Rates {
        self.leaders
            .rates
            .get(usize::from(who))
            .cloned()
            .unwrap_or_default()
    }

    /// **One of [`RATE_LEAVES`] as the record holds it.**
    pub fn leader_rate_leaf(&self, who: Player, name: &str) -> Option<i64> {
        let r = self.leader_rates(who);
        let w = |x: u16| Some(i64::from(x));
        match name {
            "frame_battle" => Some(r.frame_battle),
            "average_death_rate" => w(r.average[DEATHS]),
            "average_kill_rate" => w(r.average[KILLS]),
            "average_damage_rate" => w(r.average[DAMAGE]),
            "average_hit_rate" => w(r.average[HITS]),
            "deaths_current_frame" => w(r.current[DEATHS]),
            "kills_current_frame" => w(r.current[KILLS]),
            "hits_current_frame" => w(r.current[HITS]),
            "damage_current_frame" => w(r.current[DAMAGE]),
            "deaths_fifteen_seconds" => w(r.fifteen[DEATHS]),
            "kills_fifteen_seconds" => w(r.fifteen[KILLS]),
            "hits_fifteen_seconds" => w(r.fifteen[HITS]),
            "damage_fifteen_seconds" => w(r.fifteen[DAMAGE]),
            _ => None,
        }
    }

    /// **W1, the blow** (`do_damage@0064a480`, `64a8e3..64a91e`): past the
    /// `count < 1` and decoy returns, `hits_current_frame[attacker] += s`
    /// and `damage_current_frame[victim] += s`, `s = 5` for a siege
    /// attacker on a building, else 1. No owner, ally or self test; it
    /// counts blows, never hit points.
    pub(crate) fn note_blow(&mut self, by: Player, victim: Player, siege_on_building: bool) {
        let s = if siege_on_building { 5 } else { 1 };
        if let Some(r) = self.rates_mut(by) {
            r.add(HITS, s);
        }
        if let Some(r) = self.rates_mut(victim) {
            r.add(DAMAGE, s);
        }
    }

    /// **The kill and the death** (W2–W4): `kills_current_frame[by] += k`
    /// and `deaths_current_frame[victim] += k` — `k` 5 for an active
    /// building razed, 2 for an inactive one, 1 for a countable unit death.
    pub(crate) fn note_rate_kill(&mut self, by: Player, victim: Player, k: u16) {
        if let Some(r) = self.rates_mut(by) {
            r.add(KILLS, k);
        }
        if let Some(r) = self.rates_mut(victim) {
            r.add(DEATHS, k);
        }
    }

    /// **W5, the fold** (`Leader::process_event_frame@006ec180`), which
    /// `Game::do_frame@00591ef0` calls for slots 0..7 in use (`leader_flags
    /// & 1`, a defeated seat too) after `Achieve::capture_data` and before
    /// `frame += 1`. It returns unless `frame % 50 == 0`.
    pub(crate) fn fold_rates(&mut self, frame: i64) {
        if frame % FOLD_PERIOD != 0 {
            return;
        }
        let n = self.leaders.rates.len().min(8);
        for who in 0..n {
            // The leader's own age, `data_encrypted->+0xdc ^ 0x62766`.
            let age = self.tech.get(who).map_or(0, |t| t.ages);
            self.leaders.rates[who].fold(frame, age);
        }
    }
}

#[cfg(test)]
mod tests;
