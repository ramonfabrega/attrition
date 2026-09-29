//! **The City's alarm and a player's gather** — two issuers of the
//! Militia's Civilian that no chapter before forty staged (item 1167,
//! `docs/GOLDEN.md` §49).
//!
//! - `CommandPackage::process_alarm@00947ef0` → `Group::action_alarm@
//!   0070ec30` on a group of the player's own buildings: the bell, and
//!   the all-clear that makes every Militia inside the city's buildings a
//!   Citizen again (`SpellType::cast_civilian`) before it empties them.
//! - `CommandPackage::process_gather@009488b0` → `Group::action_gather@
//!   00700b90(ox, queued)` on a group of units: a Militia among them takes
//!   the Civilian in front of its gather.

use crate::Sim;
use crate::group::Group;
use crate::orders::{QueuePos, index, spell};
use crate::world::{Cell, Player, Pos, vector_dist};

impl Sim {
    /// `Group::action_alarm@0070ec30` on the buildings `list` of `who`,
    /// in the group's order.
    ///
    /// ```text
    /// for each member that is a live city building (+8 & 0x20, its
    ///   CityData at +0x72) whose city has city_flags & 1:
    ///     0x40 clear, and no all-clear yet this call: the BELL
    ///     0x40 set,   and no bell yet this call:      the ALL-CLEAR
    /// ```
    ///
    /// Every press sets the leader's `0x2000000` first (`0070ed27`), the
    /// economy's dirty bit ([`Sim::economy_changed`], `crate::holdings`):
    /// run430's income on 889, eight frames after the bell, counts the
    /// Citizens under their GARRISONs out.
    ///
    /// SEAM: the scenario's `ignore_orders` sweep; the two sounds;
    /// `action_alarm_peasant@006fd980`, a group of units' alarm.
    pub fn action_alarm(&mut self, who: Player, list: &[usize]) {
        self.economy_changed(who);
        let (mut rang, mut cleared) = (false, false);
        for &b in list {
            let bd = &self.buildings[b];
            if !bd.alive || bd.owner != who || !self.building_is_city(b) {
                continue;
            }
            let Some(c) = bd.city else {
                continue;
            };
            if !self.cities[c].alive {
                continue;
            }
            if !self.cities[c].alarm {
                if cleared {
                    continue;
                }
                rang = true;
                self.alarm_bell(who, c);
            } else {
                if rang {
                    continue;
                }
                cleared = true;
                self.alarm_all_clear(who, c);
            }
        }
    }

    /// `ObjectData::is(ti, 0)` on a unit, `ti` a `TypeIndex`: its lineage
    /// holds the type's tree node.
    fn unit_is_index(&self, u: usize, ti: i32) -> bool {
        self.unit_rec_of_index(ti)
            .and_then(|r| self.unit_types[r].tree)
            .is_some_and(|t| self.unit_line_is(u, t))
    }

    /// `UnitData::control_cost(0)@00609d40`: a decoy 0, else the type's
    /// `POP` (`+0x2f0`).
    fn control_cost(&self, u: usize) -> i32 {
        if self.units[u].decoy {
            return 0;
        }
        self.units[u].ty.map_or(0, |t| self.unit_types[t].price.pop)
    }

    /// The bell (`0070ec30`'s `0x40`-clear arm):
    ///
    /// ```text
    /// room   = num_inside(city building), limit = its garrison limit
    ///          + both of every other member a Citizen can garrison
    /// r      = CityData::get_radius (tiles);  rc = (r + 3) / 4 (cells)
    /// for each cell of circle_x/y[..circle_radius[rc]] round the city's
    ///   cell that find_city_at names this city:
    ///     its object chain, while room < limit: a unit of who's on the
    ///     map, a Citizen (0x32/0x33), within vector_dist r in tiles, not
    ///     yet taken: taken, room += control_cost
    /// any taken: action_garrison(taken, city building, QUEUE_NEW,
    ///            search 1); city_flags |= 0x40
    /// ```
    ///
    /// SEAM: a building on the chain, whose Citizens inside are taken too
    /// (this crate threads units only, `Sim::cell_chain`); the Citizen's
    /// tribe substitution.
    fn alarm_bell(&mut self, who: Player, c: usize) {
        let cb = self.cities[c].building;
        let mut room = self.num_inside(cb);
        let mut limit = self.garrison_limit(cb);
        let citizen = self.unit_rec_of_index(crate::cast::PEASANTS);
        let mut chain = self.city_chain(c).into_iter();
        chain.next();
        for b in chain {
            let can = citizen
                .zip(self.buildings[b].ty)
                .is_some_and(|(ut, bt)| self.can_garrison(ut, bt));
            if can {
                room += self.num_inside(b);
                limit += self.garrison_limit(b);
            }
        }
        let city_tile = self.cities[c].pos.tile();
        let r = self.radius_of(c);
        let rc = ((r + 3) / 4).clamp(0, 0x40) as usize;
        let home = self.cities[c].pos.cell();
        let circ = crate::ai_place::circle();
        let mut g = Group::stack(who);
        for k in 0..circ.radius[rc] {
            let cell = Cell::new(home.x + circ.x[k], home.y + circ.y[k]);
            if cell.x < 0
                || cell.y < 0
                || cell.x >= self.world.width()
                || cell.y >= self.world.height()
            {
                continue;
            }
            let centre = Pos::new(cell.x * 0x300 + 0x180, cell.y * 0x300 + 0x180);
            if self.find_city_at(who, centre, None) != Some(c) {
                continue;
            }
            for u in self.cell_chain(cell) {
                if room >= limit {
                    break;
                }
                let unit = &self.units[u];
                if unit.owner != who || !unit.alive() || !unit.on_map {
                    continue;
                }
                if !matches!(unit.type_index, 0x32 | 0x33) {
                    continue;
                }
                let t = unit.pos.tile();
                if vector_dist(t.x - city_tile.x, t.y - city_tile.y) > r || g.list.contains(&u) {
                    continue;
                }
                self.group_add(&mut g, u);
                room += self.control_cost(u);
            }
        }
        if !g.list.is_empty() {
            self.group_action_garrison_search(&g, cb, QueuePos::New, true);
            self.cities[c].alarm = true;
        }
    }

    /// The all-clear (`0070ec30`'s `0x40`-set arm):
    ///
    /// ```text
    /// for each cell of circle_x/y[..circle_radius[rc]] round the city's
    ///   cell, its object chain: a unit of who's on the map, a Citizen,
    ///   within r tiles, whose action is a GARRISON (0x1a): repath,
    ///   kill_current_order
    /// for each building of the city's chain that is finished (+8 & 4):
    ///     every unit inside that is(MILITIA 0x42): cast_civilian
    ///     eject_contents(0, PEASANTS, 0, 0)
    /// city_flags &= ~0x40
    /// ```
    fn alarm_all_clear(&mut self, who: Player, c: usize) {
        let city_tile = self.cities[c].pos.tile();
        let r = self.radius_of(c);
        let rc = ((r + 3) / 4).clamp(0, 0x40) as usize;
        let home = self.cities[c].pos.cell();
        let circ = crate::ai_place::circle();
        for k in 0..circ.radius[rc] {
            let cell = Cell::new(home.x + circ.x[k], home.y + circ.y[k]);
            if cell.x < 0
                || cell.y < 0
                || cell.x >= self.world.width()
                || cell.y >= self.world.height()
            {
                continue;
            }
            for u in self.cell_chain(cell) {
                let unit = &self.units[u];
                if unit.owner != who || !unit.alive() || !unit.on_map {
                    continue;
                }
                if !matches!(unit.type_index, 0x32 | 0x33) {
                    continue;
                }
                let t = unit.pos.tile();
                if vector_dist(t.x - city_tile.x, t.y - city_tile.y) > r {
                    continue;
                }
                let garrisoning = self
                    .action_of(u)
                    .is_some_and(|a| self.units[u].orders[a].index() == index::GARRISON);
                if garrisoning {
                    self.repath(u);
                    self.kill_current_order(u);
                }
            }
        }
        for b in self.city_chain(c) {
            if !self.buildings[b].alive || !self.buildings[b].active {
                continue;
            }
            for u in self.buildings[b].garrison.clone() {
                if self.unit_is_index(u, crate::cast::MILITIA) {
                    self.cast_civilian(u);
                }
            }
            self.eject_citizens_now(b);
        }
        self.cities[c].alarm = false;
    }

    /// `Object::eject_contents(0, PEASANTS, 0, 0)@0064cd20` on a building:
    /// the typed arm, which is **immediate** (only a negative type defers
    /// through the building's `0x4000`). Walking the inside chain from the
    /// head: a unit that `is(PEASANTS)` comes out (`repath` first when it
    /// has an action); one that is not, but `is(MILITIA)`, comes out too
    /// with a Civilian at `QUEUE_FIRST` on the building's point; anything
    /// else stays. A `come_out` that fails ends the walk (`param_1` 0).
    fn eject_citizens_now(&mut self, b: usize) {
        let mut i = 0;
        while i < self.buildings[b].garrison.len() {
            let u = self.buildings[b].garrison[i];
            let citizen = self.unit_is_index(u, crate::cast::PEASANTS);
            let militia = !citizen && self.unit_is_index(u, crate::cast::MILITIA);
            if !citizen && !militia {
                i += 1;
                continue;
            }
            if self.action_of(u).is_some() {
                self.repath(u);
            }
            if militia && self.spell_castable(spell::CIVILIAN, u) {
                let at = self.buildings[b].pos;
                self.add_cast_order_on(u, spell::CIVILIAN, None, at, QueuePos::First, true);
            }
            if !self.come_out(u) {
                return;
            }
        }
    }

    /// `Group::action_gather@00700b90(b, queued)` on the group `g`: each
    /// member that gathers is given a gather building of its own player's
    /// found on a spiral round `b` (`move_x/move_y`, `world::MOVE_289`),
    /// and a Militia among them the Civilian in front of it.
    ///
    /// ```text
    /// queued QUEUE_FIRST is taken as QUEUE_NEW; the group's form -1
    /// region = b's cell's tregion (the coastal refinement)
    /// idle   = count(CAN_GATHER) - count(GATHER_WORKERS)
    /// for each member, the LAST first, alive and on the map:
    ///     a Scholar (0x34/0x35, or rare): the movers
    ///     a Citizen, or castable Civilian, or (a computer's) a transport
    ///       holding Citizens: a gatherer; anything else: the movers
    ///   a gatherer: skipped when idle != 0 and its action is a GATHER;
    ///     a human's gatherer not in b's region: skipped
    ///     the spiral's next offset k (k carries across members, 0 after
    ///       each assignment; past 0x78 every member left joins the
    ///       movers): a human's probe out of b's region is passed over;
    ///       on the probe cell's objects, the first of the player's that
    ///       is an active gather building: the member's +0x80 = -1, and
    ///       QUEUE_LAST: Civilian (castable) then gather, both QUEUE_LAST
    ///       else:       gather QUEUE_NEW, then Civilian QUEUE_FIRST
    ///       the member leaves the group (Group::kill); next member
    /// the movers: action_move_near(b's point, 0xc0, QUEUE_NEW, MOVE_TO)
    /// ```
    ///
    /// SEAM, no capture on file reaching them: a University target and
    /// its Scholars' arm; the good's `type_avail` test; the probe cell's
    /// objects are its buildings in this crate (`Sim::cell_chain` threads
    /// units, and a unit is never a gather building); the non-castable
    /// arm's oil-rig boarding and `Build::check_gatherers`; the movers'
    /// `action_move_near`.
    pub fn group_gather(&mut self, g: &Group, b: usize, queued: QueuePos) {
        let pos = if queued == QueuePos::First {
            QueuePos::New
        } else {
            queued
        };
        if let Some(st) = self.gstate_mut(g) {
            st.form = -1;
        }
        let who = g.who;
        let target = self.buildings[b].pos;
        let region = self.world.tregion_alt(target.tile());
        let human = self.nation[who as usize].human;
        let idle = g
            .list
            .iter()
            .filter(|&&u| self.worker_of(u) == crate::orders::Worker::Citizen)
            .filter(|&&u| {
                !self
                    .action_of(u)
                    .is_some_and(|a| self.units[u].orders[a].index() == index::GATHER)
            })
            .count();
        let mut members = g.list.clone();
        let mut k = 0usize;
        let mut i = members.len();
        'members: while i > 0 {
            let u = members[i - 1];
            if !self.units[u].alive() || !self.units[u].on_map {
                i -= 1;
                continue;
            }
            let ti = self.units[u].type_index;
            if matches!(ti, 0x34 | 0x35) {
                i -= 1;
                continue;
            }
            let gatherer = matches!(ti, 0x32 | 0x33) || self.spell_castable(spell::CIVILIAN, u);
            if !gatherer {
                i -= 1;
                continue;
            }
            let gathering = self
                .action_of(u)
                .is_some_and(|a| self.units[u].orders[a].index() == index::GATHER);
            if idle != 0 && gathering {
                i -= 1;
                continue;
            }
            if human && self.world.tregion_alt(self.units[u].pos.tile()) != region {
                i -= 1;
                continue;
            }
            loop {
                let Some(&(dx, dy)) = crate::world::MOVE_289.get(k) else {
                    break 'members;
                };
                k += 1;
                if k > 0x78 {
                    break 'members;
                }
                let probe = Pos::new(target.x + dx * 0x300, target.y + dy * 0x300);
                let (w, h) = (self.world.width() * 0x300, self.world.height() * 0x300);
                if probe.x < 0 || probe.y < 0 || probe.x >= w || probe.y >= h {
                    continue;
                }
                if human && self.world.tregion_alt(probe.tile()) != region {
                    continue;
                }
                let cell = probe.cell();
                let found = (0..self.buildings.len()).rev().find(|&x| {
                    let bd = &self.buildings[x];
                    bd.owner == who
                        && bd.alive
                        && bd.active
                        && bd.pos.cell() == cell
                        && self.is_gather_type(x)
                });
                let Some(site) = found else {
                    continue;
                };
                let at = self.units[u].pos;
                let castable = self.spell_castable(spell::CIVILIAN, u);
                if pos == QueuePos::Last {
                    if castable {
                        self.add_cast_order_on(u, spell::CIVILIAN, None, at, QueuePos::Last, true);
                    }
                    self.add_gather_order(u, site, QueuePos::Last, true);
                } else {
                    self.add_gather_order(u, site, QueuePos::New, true);
                    if castable {
                        self.add_cast_order_on(u, spell::CIVILIAN, None, at, QueuePos::First, true);
                    }
                }
                if let Some(seat) = Self::seat_of_group(g) {
                    self.seat_kill(seat, u, false, false);
                }
                members.retain(|&m| m != u);
                k = 0;
                i -= 1;
                continue 'members;
            }
        }
    }
}
