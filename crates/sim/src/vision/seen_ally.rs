//! **`Object::update_seen_ally@006537a0`** for a building — the reveal
//! `Setup::build_cities@005ab910` makes for each player's starting city,
//! after `Build::activate` and the `+0x174` whole disc (`docs/START.md`
//! §7.2). It is the start's own fog: a city founded by the setup walk sees
//! its whole territory radius, not only its line of sight, and that is
//! what the scouts' first `think_scout` reads on frame 0.

use crate::Sim;
use crate::ai::ai_place::circle;
use crate::vision::fog_of;

impl Sim {
    /// The leaders an object of `who`'s reveal reaches, as a mask: `who`
    /// itself; every leader allied with it both ways (`diplos` 2 each
    /// way); and when `who` is human, every other human (the original
    /// gates that arm on `semaphore[0] & 4` clear, which no lobby field
    /// sets). SEAM: the `game_rules == 8` arm (a team-0 leader under
    /// that rule set) is read and not carried (`docs/START.md` §7.5).
    pub(crate) fn seen_ally_reach(&self, who: crate::Player) -> u8 {
        let n = self.territory.players.len().min(8);
        let human = |p: usize| self.cities.nation.get(p).is_some_and(|x| x.human);
        let mut m = 0u8;
        for u in 0..n {
            let w = who as usize;
            let allied = w < n
                && self.leaders.allied.get(w).and_then(|r| r.get(u)) == Some(&true)
                && self.leaders.allied.get(u).and_then(|r| r.get(w)) == Some(&true);
            if u == w || allied || (human(w) && human(u)) {
                m |= 1 << u;
            }
        }
        m
    }

    /// **`Object::update_seen_ally`** on building `b`: the radius is the
    /// building's line of sight (vslot `+0x128`, `WallData::los`, the
    /// cached [`crate::Building::mylos`]), or for a city's own building
    /// (`flags & 0x20`, [`Sim::building_is_city`]) `LeaderData::get_radius`
    /// of its type capped at 64 and halved into fog cells (`r × 2 / 4`).
    /// The building's `ever_seen` takes every reached leader's bit; then
    /// each of the first `circle_radius[r]` points round its fog cell, on
    /// the grid, takes them in `seen2` alone (`World +0x160`; the current
    /// plane is not written). The per-cell `WData.was_seen` byte it also
    /// sets is not kept. Answers the fog cells it changed.
    pub fn update_seen_ally_build(&mut self, b: usize) -> usize {
        if !self.world.has_fog() || !self.buildings[b].alive {
            return 0;
        }
        let who = self.buildings[b].owner;
        let mut r = self.buildings[b].mylos;
        if self.building_is_city(b) {
            let rad = self.city_radius(who, self.buildings[b].ty).min(0x40);
            r = (rad * 2) / 4;
        }
        if r <= 0 {
            return 0;
        }
        let r = r.min(0x40) as usize;
        let mask = self.seen_ally_reach(who);
        self.buildings[b].ever_seen |= mask;
        let pos = self.buildings[b].pos;
        let (cx, cy) = (fog_of(pos.x), fog_of(pos.y));
        let c = circle();
        let mut changed = 0;
        for i in 0..c.radius[r] {
            if self.world.set_seen2_only(cx + c.x[i], cy + c.y[i], mask) {
                changed += 1;
            }
        }
        changed
    }
}
