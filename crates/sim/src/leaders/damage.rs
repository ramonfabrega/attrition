//! **The damage path's leaves** — item 4176, step III of the leaders line:
//! the kill, loss and raze counters (`docs/LEADERS.md` §2), the victim
//! stamps (§4), the attrition stamps and `give_att_disabled` (§5), and the
//! missiles, repairs, bribes and nukes (§15).
//!
//! Each is a leaf of `LeaderData` the leaders line of the sync checksum
//! walks (`docs/CHECKSUM.md` §20). All but `units_bribed` and
//! `give_att_disabled` are printed by `LeaderData::log_data@006e5110`, so
//! `rondata::diff::leader` compares them on every kept dump that prints the
//! record; those two have the checksum line alone.
//!
//! The events live where the original's do — `Object::do_damage@0064a480`
//! and `Object::take_damage@00652020` are `crate::combat::fight::delivery`,
//! the attrition stamps `crate::territory::upkeep`, the third
//! `Sim::process_building`, the repair refusal `crate::orders::jobs`, the
//! missile's landing `crate::combat::fight::projectiles` and the nuke's
//! launch `Sim::come_out`. This module holds the record and the rules each
//! site calls.

use crate::{Player, Sim};

/// The window every victim stamp waits (`0x12c`, strict: `frame − stamp >
/// 300`; `docs/LEADERS.md` §4.2).
pub const STAMP_WINDOW: i64 = 300;

/// `attrition_stamp`'s re-arm window (`0x384`, strict; §5.2).
pub const ATTRITION_WINDOW: i64 = 900;

/// One leader's §2, §4, §5 and §15 state that nothing else holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Damage {
    /// `units_killed` (`+0x80c`).
    pub units_killed: i32,
    /// `units_lost` (`+0x810`).
    pub units_lost: i32,
    /// `buildings_razed` (`+0x818`).
    pub buildings_razed: i32,
    /// `buildings_lost` (`+0x81c`).
    pub buildings_lost: i32,
    /// `attack_stamp` (`+0xf4`), the victim's row, by attacker.
    pub attack_stamp: [i64; 8],
    /// `raid_stamp` (`+0x114`), by attacker.
    pub raid_stamp: [i64; 8],
    /// `ally_stamp` (`+0x154`), by attacker.
    pub ally_stamp: [i64; 8],
    /// `attrition_stamp` (`+0x1f4`): the peace or assassin arm's frame.
    pub attrition_stamp: i64,
    /// `attrition_stamp2` (`+0x1f8`): the slot's first attrition tick.
    pub attrition_stamp2: i64,
    /// `attrition_stamp3` (`+0x1fc`): a building's bleed in a rival's land.
    pub attrition_stamp3: i64,
    /// `repair_stamp` (`+0x208`): the frame a repair was last refused.
    pub repair_stamp: i64,
    /// `missiles_used` (`+0x7c0`): V2 and Cruise Missile rounds landed.
    pub missiles_used: i32,
    /// `nukes_launched` (`+0xa38`): nukes out holding a strafe.
    pub nukes_launched: i32,
}

/// The leaves [`Sim::leader_damage_leaf`] answers, in the record's order.
/// `units_bribed` and `give_att_disabled` are the two `log_data` does not
/// print.
pub const LEAVES: [&str; 16] = [
    "attack_stamp",
    "raid_stamp",
    "ally_stamp",
    "attrition_stamp",
    "attrition_stamp2",
    "attrition_stamp3",
    "ctw_hero_stamp",
    "repair_stamp",
    "missiles_used",
    "give_att_disabled",
    "units_killed",
    "units_lost",
    "buildings_razed",
    "buildings_lost",
    "units_bribed",
    "nukes_launched",
];

/// `frame − stamp > window`, the strict test every stamp here shares.
pub fn past(frame: i64, stamp: i64, window: i64) -> bool {
    frame - stamp > window
}

impl Sim {
    /// One leader's [`Damage`], mutable, or `None` for a slot the record
    /// has no row for (gaia's).
    fn damage_mut(&mut self, who: Player) -> Option<&mut Damage> {
        self.leaders.damage.get_mut(usize::from(who))
    }

    /// One leader's [`Damage`], or the clear's for a slot out of range.
    pub fn leader_damage(&self, who: Player) -> Damage {
        self.leaders
            .damage
            .get(usize::from(who))
            .cloned()
            .unwrap_or_default()
    }

    /// **One of [`LEAVES`] as the record holds it**: a stamp array its
    /// eight cells, any other leaf one value. `ctw_hero_stamp` is
    /// Conquer the World's and 0 for a plain game (§4.2); `units_bribed`
    /// is `PlayerTech::bribes` and `give_att_disabled` the trigger flag
    /// `PlayerState::give_disabled` (§15.3, §5.3).
    pub fn leader_damage_leaf(&self, who: Player, name: &str) -> Option<Vec<i64>> {
        let d = self.leader_damage(who);
        let w = usize::from(who);
        let one = |v: i64| Some(vec![v]);
        match name {
            "attack_stamp" => Some(d.attack_stamp.to_vec()),
            "raid_stamp" => Some(d.raid_stamp.to_vec()),
            "ally_stamp" => Some(d.ally_stamp.to_vec()),
            "attrition_stamp" => one(d.attrition_stamp),
            "attrition_stamp2" => one(d.attrition_stamp2),
            "attrition_stamp3" => one(d.attrition_stamp3),
            "ctw_hero_stamp" => one(0),
            "repair_stamp" => one(d.repair_stamp),
            "missiles_used" => one(i64::from(d.missiles_used)),
            "give_att_disabled" => one(i64::from(
                self.territory
                    .players
                    .get(w)
                    .is_some_and(|p| p.give_disabled),
            )),
            "units_killed" => one(i64::from(d.units_killed)),
            "units_lost" => one(i64::from(d.units_lost)),
            "buildings_razed" => one(i64::from(d.buildings_razed)),
            "buildings_lost" => one(i64::from(d.buildings_lost)),
            "units_bribed" => one(i64::from(self.tech.get(w).map_or(0, |t| t.bribes))),
            "nukes_launched" => one(i64::from(d.nukes_launched)),
            _ => None,
        }
    }

    /// **"A countable unit death"** (`docs/LEADERS.md` §2.2): the struck
    /// figure is its squad's captain, alone (`curr_uber_size == 1`, no live
    /// follower) and not a decoy. A building is never one; the caller asks
    /// only of a unit. Read before the death's relink.
    pub(crate) fn countable_death(&self, i: usize) -> bool {
        self.is_captain(i) && !self.units[i].decoy && self.curr_uber_size(i) == 1
    }

    /// `take_damage`'s death branch on a countable unit death:
    /// `units_lost[victim] += 1`, and `units_killed[by] += 1` when the call
    /// carries an attacker object (`param_7 >= 0`), with no ally, self or
    /// ownership test (§2.2).
    pub(crate) fn note_unit_death(&mut self, victim: Player, by: Option<Player>) {
        if let Some(d) = self.damage_mut(victim) {
            d.units_lost += 1;
        }
        if let Some(k) = by {
            self.note_unit_kill(k);
        }
    }

    /// `units_killed[who] += 1`: `take_damage`'s attacker arm, and the
    /// Sniper's, Commando's and counterintel's (§2.2).
    pub(crate) fn note_unit_kill(&mut self, who: Player) {
        if let Some(d) = self.damage_mut(who) {
            d.units_killed += 1;
        }
    }

    /// `buildings_razed[who] += 1`: `do_damage`'s non-zero take on a
    /// building, and sabotage's (§2.2).
    pub(crate) fn note_raze(&mut self, who: Player) {
        if let Some(d) = self.damage_mut(who) {
            d.buildings_razed += 1;
        }
    }

    /// `buildings_lost[owner] += 1`: `take_damage`'s building death, the
    /// city clamp excluded (§2.2).
    pub(crate) fn note_building_lost(&mut self, owner: Player) {
        if let Some(d) = self.damage_mut(owner) {
            d.buildings_lost += 1;
        }
    }

    /// The victim's two leader stamps a city arm writes together
    /// (`raid_stamp[att] = attack_stamp[att] = frame`, §4.2 arms 1–3).
    pub(crate) fn stamp_raid_and_attack(&mut self, vic: Player, att: Player, frame: i64) {
        let a = usize::from(att);
        if a < 8
            && let Some(d) = self.damage_mut(vic)
        {
            d.raid_stamp[a] = frame;
            d.attack_stamp[a] = frame;
        }
    }

    /// `ally_stamp`'s step after arms 1 and 2 (§4.2): `frame −
    /// ally_stamp[att] > 300` takes the frame, for every pair of players.
    pub(crate) fn step_ally_stamp(&mut self, vic: Player, att: Player, frame: i64) {
        let a = usize::from(att);
        if a < 8
            && let Some(d) = self.damage_mut(vic)
            && past(frame, d.ally_stamp[a], STAMP_WINDOW)
        {
            d.ally_stamp[a] = frame;
        }
    }

    /// **The territory step** (§4.2 step 4), after the city arms, on every
    /// blow past `do_damage`'s head: `raid` picks the raid branch
    /// (`raid_stamp[att]`), else the attack branch (`attack_stamp[att]`),
    /// each on its own 300-frame window. The two are exclusive.
    pub(crate) fn step_territory_stamp(
        &mut self,
        vic: Player,
        att: Player,
        raid: bool,
        frame: i64,
    ) {
        let a = usize::from(att);
        if a >= 8 {
            return;
        }
        let Some(d) = self.damage_mut(vic) else {
            return;
        };
        let cell = if raid {
            &mut d.raid_stamp[a]
        } else {
            &mut d.attack_stamp[a]
        };
        if past(frame, *cell, STAMP_WINDOW) {
            *cell = frame;
        }
    }

    /// **`attrition_stamp`'s peace and assassin arms**
    /// (`Unit::process_attrition@005e11a0`, §5.2), on the unit owner's
    /// slot: `stamp == 0 || frame − stamp > 900` takes the frame.
    pub(crate) fn note_attrition_arm(&mut self, who: Player, frame: i64) {
        if let Some(d) = self.damage_mut(who)
            && (d.attrition_stamp == 0 || past(frame, d.attrition_stamp, ATTRITION_WINDOW))
        {
            d.attrition_stamp = frame;
        }
    }

    /// `Game::do_frame`'s timed zeroing of `attrition_stamp` on every slot
    /// in play, on the frame the peace ends (§5.2).
    pub(crate) fn zero_attrition_stamps(&mut self) {
        for d in &mut self.leaders.damage {
            d.attrition_stamp = 0;
        }
    }

    /// `attrition_stamp2` (`+0x1f8`), the word `suffer_attrition`'s
    /// warning tests.
    pub(crate) fn attrition_stamp2(&self, who: usize) -> i64 {
        self.leaders
            .damage
            .get(who)
            .map_or(0, |d| d.attrition_stamp2)
    }

    /// `attrition_stamp2 = frame`, for every slot (the console gates only
    /// the message).
    pub(crate) fn set_attrition_stamp2(&mut self, who: usize, frame: i64) {
        if let Some(d) = self.leaders.damage.get_mut(who) {
            d.attrition_stamp2 = frame;
        }
    }

    /// **`attrition_stamp3`** (`Wall::process@00640450`, §5.2), on the
    /// building owner's slot: an unstarted site's disband re-stamps every
    /// time (`always`); a started building that survives its hit stamps
    /// only a zero word.
    pub(crate) fn note_building_bleed(&mut self, who: Player, frame: i64, always: bool) {
        if let Some(d) = self.damage_mut(who)
            && (always || d.attrition_stamp3 == 0)
        {
            d.attrition_stamp3 = frame;
        }
    }

    /// `repair_stamp` (`+0x208`), `do_repair`'s refusal throttle.
    pub(crate) fn repair_stamp(&self, who: Player) -> i64 {
        self.leader_damage(who).repair_stamp
    }

    /// `repair_stamp = frame`, for every player, before the console test.
    pub(crate) fn set_repair_stamp(&mut self, who: Player, frame: i64) {
        if let Some(d) = self.damage_mut(who) {
            d.repair_stamp = frame;
        }
    }

    /// `missiles_used[who]`, which `get_support_count` adds for a V2 or a
    /// Cruise Missile (§15.2).
    pub(crate) fn missiles_used(&self, who: Player) -> i32 {
        self.leaders
            .damage
            .get(usize::from(who))
            .map_or(0, |d| d.missiles_used)
    }

    /// `missiles_used[who] += 1`, a missile's round landed past its shield
    /// and the nuke arm.
    pub(crate) fn note_missile_landed(&mut self, who: Player) {
        if let Some(d) = self.damage_mut(who) {
            d.missiles_used += 1;
        }
    }

    /// `nukes_launched[who] += 1`, `come_out`'s strafing nuke (§15.2).
    pub(crate) fn note_nuke_launched(&mut self, who: Player) {
        if let Some(d) = self.damage_mut(who) {
            d.nukes_launched += 1;
        }
    }
}

#[cfg(test)]
mod tests;
