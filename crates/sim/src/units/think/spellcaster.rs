//! `Unit::think_spellcaster@005f27a0` — an AI caster deciding whether to
//! cast by itself (`docs/AI.md` §80.5, item 971).
//!
//! Every capture of the first pair was on Easiest, where the computer
//! leader's arm returns before its first test (`get_diff() < 2`), so the
//! function drew nothing on any frame of either long capture and this crate
//! skipped it (`docs/SCOUT.md` §13 item 10). At Toughest the AI's scout
//! reaches it on frame 0 from `Unit::think_scout+0x7c`, and the special
//! unit's arm throws a coin there — the second pair's first word.
//!
//! What is built is the computer leader's arm, in the listing's order:
//!
//! ```text
//! get_diff() < 2                                  -> 0
//! unit_masks & 1 (a decoy)                        -> 0
//! is(SPY):  cloaked? Counterintelligence on a friend in range × 5,
//!           then Bribe on an enemy unit in range × 5            (no draw)
//! not is_special:  the hero arm   (Create Decoys; moving: seam)
//! is_special:
//!     Sniper (0x281) castable, with the mana, on an enemy in range -> cast
//!     coin = Random::get(game_random, 0, 0xffff)        // +0x413
//!     coin % 3 != 0 and no army                   -> 0
//!     Counterintelligence (0x277) castable, with the mana, on a target
//!     in range                                    -> cast
//!     -> 0
//! ```
//!
//! **The targets are searched** (item 4675, `docs/RECGAME.md` §13.5):
//! row 133's who=2 Spy `2/84` bribes who=1's Assault Infantry `1/87` on
//! run1797's frame 3478, where this crate's Spy arm stood at the cloak.
//! The four searches are [`Sim::caster_find`] (`ObjectsData::find
//! @0065c6b0`, Counterintelligence's) and [`Sim::caster_find_unit`]
//! (`ObjectsData::find_unit@0065ca80`, Bribe's and the Sniper's), each
//! filtered by `Search::valid_filter@0067dbb0`'s `FILTER_SPELL_TARGET`
//! arm (`0067e385`: [`Sim::spell_valid_target`], then for Bribe and the
//! Sniper an undamaged target, `+0x24 == 0`) and, for `find_unit`,
//! `FILTER_COMBAT` (`0067debf`: an attack and the combat role, `+0x2c8 &
//! 0x10000`). The cloak is [`Sim::unit_is_cloaked`], `is_castable` is
//! [`Sim::spell_castable`], and `get_range(who, -1, -1)` is
//! [`Sim::spell_range_at`] with no target.
//!
//! - ~~**the hero arm**~~: built as [`Sim::think_hero`] (item 1305,
//!   `docs/AI.md` §99.13), reached from `Army::use_generals`;
//! - **the human arm**, which `Unit::think`'s special turn reaches, casts
//!   through the same searches.
//!
//! SEAM: [`Sim::spell_castable`] is a partial reading of `is_castable`
//! (its own SEAM names the rest).

use crate::Sim;
use crate::combat::Obj;
use crate::orders::QueuePos;
use crate::world::{Player, Pos};

/// The special arm's coin, `Random::get` returning to `005f2bb3`.
pub const SITE_SPECIAL_COIN: &str = "Unit::think_spellcaster+0x413";

/// The hero arm's coin, `Random::get` returning to `005f2d29`: Forced
/// March on an odd one, the territory test on an even one.
pub const SITE_HERO_COIN: &str = "Unit::think_spellcaster+0x589";

/// `TypeIndex` `0x3a`, the Spy.
const SPY: crate::tech::TypeId = 0x3a;

/// The crafts `think_spellcaster` asks about (`SpellType` rows
/// `0x275..=0x281`, the table's slots `+0x9d4..=+0xa04`).
pub mod craft {
    pub use crate::ids::spell::{BRIBE, COUNTERINTEL, SNIPER};
}

/// `find`'s search mode (its third argument).
pub mod search {
    pub use crate::ids::search::{SEARCH_ALL as ALL, SEARCH_FRIENDLY as FRIENDLY};
}

/// What `Unit::think_spellcaster` reads and does, for the unit `this` it is
/// run on; [`think_spellcaster_steps`] is its control flow and
/// [`SimCaster`] answers it from the crate's [`Sim`], where
/// `sweep/unit_think_spellcaster.rs` answers it from the emulated
/// original's windows (item 3330).
pub trait CasterView {
    /// The unit's owner byte (`+0x9`).
    fn who(&self) -> i32;
    /// `leader_flags & 4`: a human leader.
    fn human(&self) -> bool;
    /// `leader_flags & 0x8000`: a hero of the leader's is on the march.
    fn forced_march(&self) -> bool;
    /// `gamec->semaphore.ptr[0..3]`.
    fn semaphore(&self) -> [u8; 3];
    /// The leader's own difficulty (`leaders.list[who] +0x50`).
    fn leader_diff(&self) -> i32;
    /// `gamec->info.difficulty`, a byte.
    fn game_diff(&self) -> i32;
    /// `unit_masks & 1`.
    fn decoy(&self) -> bool;
    /// The unit's `is(SPY)` (vslot `+0xb8`, or the type's `+0x60`).
    fn is_spy(&mut self) -> bool;
    /// `UnitData::is_special` (vslot `+0xd4`, or the type's `+0x2b8 & 0x10`).
    fn is_special(&mut self) -> bool;
    /// `UnitData::is_hero` (vslot `+0xc4`, or the type's `+0x2b8 & 0x20`).
    fn is_hero(&mut self) -> bool;
    /// The unit's vslot `+0xd8`.
    fn is_moving(&mut self) -> bool;
    /// `UnitData::is_cloaked@0060a6a0`.
    fn cloaked(&mut self) -> bool;
    /// Guy 0's `avg_speed` (`**(this+0xf4) +0x84`).
    fn avg_speed(&self) -> i32;
    /// `SpellTypeData::is_castable(spell, type, who, 0)`.
    fn castable(&mut self, spell: i32) -> bool;
    /// `UnitData::mana@00609a50`.
    fn mana(&mut self) -> i32;
    /// The unit's `mana_burn` (the short at `+0x96`).
    fn mana_burn(&self) -> i32;
    /// The craft's `MANA` (`+0x1d0` of its row).
    fn spell_mana(&self, spell: i32) -> i32;
    /// `SpellTypeData::get_range(who, -1, -1)`.
    fn range(&mut self, spell: i32) -> i32;
    /// `ObjectsData::find` at the unit's position, filtered to the craft.
    fn find(&mut self, spell: i32, mode: i32, range: i32) -> i32;
    /// `ObjectsData::find_unit` of the enemy, filtered to the craft.
    fn find_unit(&mut self, spell: i32, range: i32) -> i32;
    /// `Object::get_army`.
    fn army(&mut self) -> i32;
    /// `Random::get(game_random, 0, 0xffff)`, the hero arm's or the special's.
    fn coin(&mut self, hero: bool) -> i32;
    /// The owner byte of the cell the unit stands on (negative: no one).
    fn cell_owner(&self) -> i32;
    /// The leader's object count, and the object at a list position.
    fn count(&self) -> i32;
    /// The object at a position of the leader's list.
    fn nth(&self, k: i32) -> i32;
    /// `Object::flags & 1`.
    fn alive(&self, o: i32) -> bool;
    /// The object's vslot `+0xe8`.
    fn is_captain(&mut self, o: i32) -> bool;
    /// `unit_masks & 1` of the unit of the same list position.
    fn is_decoy(&self, o: i32) -> bool;
    /// `add_cast_order(target, .., x, y, spell, QUEUE_FIRST, 0)`.
    fn add_cast(&mut self, target: i32, spell: i32);
}

/// `mana() >= mana_burn + the craft's MANA`.
fn afford<V: CasterView>(v: &mut V, spell: i32) -> bool {
    let have = v.mana();
    let need = v.mana_burn().wrapping_add(v.spell_mana(spell));
    have >= need
}

/// `Unit::think_spellcaster@005f27a0`, call for call: whether a cast was
/// ordered (the original's 1).
///
/// ```text
/// human leader:  special, Counterintelligence castable + afforded,
///                find (SEARCH_ALL, range)                     -> cast
/// difficulty (semaphore, leader's, game's) < 2                -> 0
/// a decoy                                                     -> 0
/// is(SPY):   cloaked; Counterintelligence on a friend in range × 5,
///            then Bribe on an enemy in range × 5
/// not special:  the hero arm (below)
/// special:   Sniper on an enemy in range;  coin;  coin % 3 != 0 and no
///            army -> 0;  Counterintelligence in range
/// ```
pub fn think_spellcaster_steps<V: CasterView>(v: &mut V) -> bool {
    if v.human() {
        if !v.is_special() || !v.castable(craft::COUNTERINTEL) || !afford(v, craft::COUNTERINTEL) {
            return false;
        }
        let r = v.range(craft::COUNTERINTEL);
        let t = v.find(craft::COUNTERINTEL, search::ALL, r);
        if t < 0 {
            return false;
        }
        v.add_cast(t, craft::COUNTERINTEL);
        return true;
    }
    let [s0, s1, s2] = v.semaphore();
    let diff = if s0 & 4 != 0 {
        v.leader_diff()
    } else if (s1 & 0x10 == 0 && s2 & 2 == 0) || s2 & 2 != 0 || v.leader_diff() < 0 {
        v.game_diff()
    } else {
        v.leader_diff()
    };
    if diff < 2 || v.decoy() {
        return false;
    }
    if v.is_spy() {
        if !v.cloaked() {
            return false;
        }
        let mut t = -1;
        if v.castable(craft::COUNTERINTEL) && afford(v, craft::COUNTERINTEL) {
            let r = v.range(craft::COUNTERINTEL);
            t = v.find(craft::COUNTERINTEL, search::FRIENDLY, r.wrapping_mul(5));
        }
        if t >= 0 {
            v.add_cast(t, craft::COUNTERINTEL);
            return true;
        }
        if !v.castable(craft::BRIBE) || !afford(v, craft::BRIBE) {
            return false;
        }
        let r = v.range(craft::BRIBE);
        let t = v.find_unit(craft::BRIBE, r.wrapping_mul(5));
        if t < 0 {
            return false;
        }
        v.add_cast(t, craft::BRIBE);
        return true;
    }
    if !v.is_special() {
        return hero_arm_steps(v);
    }
    if v.castable(craft::SNIPER) && afford(v, craft::SNIPER) {
        let r = v.range(craft::SNIPER);
        let t = v.find_unit(craft::SNIPER, r);
        if t >= 0 {
            v.add_cast(t, craft::SNIPER);
            return true;
        }
    }
    let coin = v.coin(false);
    if coin % 3 != 0 && v.army() < 0 {
        return false;
    }
    if !v.castable(craft::COUNTERINTEL) || !afford(v, craft::COUNTERINTEL) {
        return false;
    }
    let r = v.range(craft::COUNTERINTEL);
    let t = v.find(craft::COUNTERINTEL, search::ALL, r);
    if t < 0 {
        return false;
    }
    v.add_cast(t, craft::COUNTERINTEL);
    true
}

/// **The hero arm** (`005f2cb3`–`005f2eac`), a computer leader's
/// non-special unit, as `Army::use_generals` reaches it on the army's
/// 128-frame turn (`docs/AI.md` §99.13, item 1305):
///
/// ```text
/// not is_hero (unit_flags2 & 0x20)                -> 0
/// is_moving (the head order's is_move):
///     guy 0's avg_speed (GuyData +0x84) == 0      -> 0
///     leader_flags & 0x8000 clear:
///         coin = Random::get(game_random, 0, 0xffff)   // +0x589
///         coin odd: FORCED_MARCH (0x27c) with the mana -> cast, else 0
///     the cell's owner (WData +0xf) negative or the caster's -> 0
///     AMBUSH (0x27b) with the mana                -> cast, else 0
/// not moving:
///     any of the leader's units a live decoy captain -> 0
///     CREATE_DECOY (0x27a) with the mana          -> cast, else 0
/// ```
///
/// "With the mana" is `mana() >= mana_burn + the craft's MANA`; no
/// arm asks `is_castable`. A cast is `add_cast_order(-1, -1, x, y,
/// craft, QUEUE_FIRST, 0)` on the unit's own position.
///
/// `leader_flags & 0x8000` is [`Sim::forced_march`]: a leader one of
/// whose heroes is on the march throws no coin and goes straight to
/// the territory test.
///
/// SEAM: Ambush's cloak (`0x800`, [`Sim::cast_ambush`]) and what
/// Forced March does beyond its flags (`crate::spells::cast`).
fn hero_arm_steps<V: CasterView>(v: &mut V) -> bool {
    use crate::orders::spell;
    if !v.is_hero() {
        return false;
    }
    let craft = if v.is_moving() {
        if v.avg_speed() == 0 {
            return false;
        }
        if !v.forced_march() && v.coin(true) & (i32::MIN | 1) != 0 {
            spell::FORCED_MARCH
        } else {
            let owner = v.cell_owner();
            if owner < 0 || owner == v.who() {
                return false;
            }
            spell::AMBUSH
        }
    } else {
        for k in 0..v.count() {
            let o = v.nth(k);
            if v.alive(o) && v.is_captain(o) && v.is_decoy(o) {
                return false;
            }
        }
        spell::CREATE_DECOY
    };
    if !afford(v, craft) {
        return false;
    }
    v.add_cast(-1, craft);
    true
}

/// [`CasterView`] over the crate's [`Sim`] for the unit `u`.
struct SimCaster<'a> {
    s: &'a mut Sim,
    u: usize,
    /// The leader's units, in the order the original's list holds them.
    mine: Vec<usize>,
    /// The last search's find, whose owner is `objects +0x200`, the
    /// `whom` the cast tail hands `add_cast_order` (`005f2c9b`).
    found: Option<Obj>,
}

impl CasterView for SimCaster<'_> {
    fn who(&self) -> i32 {
        i32::from(self.s.units[self.u].owner)
    }
    fn human(&self) -> bool {
        self.s
            .cities
            .nation
            .get(self.who() as usize)
            .is_some_and(|n| n.human)
    }
    fn forced_march(&self) -> bool {
        self.s.leaders.forced_march[self.who() as usize]
    }
    fn semaphore(&self) -> [u8; 3] {
        // SEAM: the multiplayer semaphore is not carried; its bytes read 0.
        [0; 3]
    }
    fn leader_diff(&self) -> i32 {
        // SEAM: a leader's own difficulty is not carried; negative reads the game's.
        -1
    }
    fn game_diff(&self) -> i32 {
        self.s.ai_difficulty()
    }
    fn decoy(&self) -> bool {
        self.s.units[self.u].decoy
    }
    fn is_spy(&mut self) -> bool {
        self.s.unit_line_is(self.u, SPY)
    }
    fn is_special(&mut self) -> bool {
        self.s.unit_is_special(self.u)
    }
    fn is_hero(&mut self) -> bool {
        self.s.is_hero_unit(self.u)
    }
    fn is_moving(&mut self) -> bool {
        self.s.is_moving(self.u)
    }
    fn cloaked(&mut self) -> bool {
        self.s.unit_is_cloaked(self.u)
    }
    fn avg_speed(&self) -> i32 {
        let unit = &self.s.units[self.u];
        unit.guys
            .first()
            .and_then(|g| g.follow)
            .map_or(unit.movement.body.avg_speed, |f| f.body.avg_speed)
    }
    fn castable(&mut self, spell: i32) -> bool {
        self.s.spell_castable(spell, self.u)
    }
    fn mana(&mut self) -> i32 {
        self.s.unit_mana(self.u)
    }
    fn mana_burn(&self) -> i32 {
        i32::from(self.s.units[self.u].mana_burn)
    }
    fn spell_mana(&self, spell: i32) -> i32 {
        self.s.spell(spell).map_or(0, |d| d.mana)
    }
    fn range(&mut self, spell: i32) -> i32 {
        let who = self.s.units[self.u].owner;
        self.s.spell_range_at(spell, who, None)
    }
    fn find(&mut self, spell: i32, mode: i32, range: i32) -> i32 {
        let who = self.s.units[self.u].owner;
        let at = self.s.units[self.u].pos;
        self.found = self.s.caster_find(spell, who, mode, at, range);
        self.found.map_or(-1, |o| i32::from(self.s.obj_number(o)))
    }
    fn find_unit(&mut self, spell: i32, range: i32) -> i32 {
        let who = self.s.units[self.u].owner;
        let at = self.s.units[self.u].pos;
        self.found = self
            .s
            .caster_find_unit(spell, who, at, range)
            .map(Obj::Unit);
        self.found.map_or(-1, |o| i32::from(self.s.obj_number(o)))
    }
    fn army(&mut self) -> i32 {
        self.s.army_of(self.u).map_or(-1, |a| a as i32)
    }
    fn coin(&mut self, hero: bool) -> i32 {
        self.s.mark(if hero {
            SITE_HERO_COIN
        } else {
            SITE_SPECIAL_COIN
        });
        self.s.rng.roll()
    }
    fn cell_owner(&self) -> i32 {
        self.s
            .world
            .owner_at(self.s.units[self.u].pos)
            .player()
            .map_or(-1, i32::from)
    }
    fn count(&self) -> i32 {
        self.mine.len() as i32
    }
    fn nth(&self, k: i32) -> i32 {
        self.mine[k as usize] as i32
    }
    fn alive(&self, o: i32) -> bool {
        self.s.units[o as usize].alive()
    }
    fn is_captain(&mut self, o: i32) -> bool {
        self.s.is_captain(o as usize)
    }
    fn is_decoy(&self, o: i32) -> bool {
        self.s.units[o as usize].decoy
    }
    fn add_cast(&mut self, target: i32, spell: i32) {
        // `add_cast_order(o, objects +0x200, x, y, spell, QUEUE_FIRST, 0)`
        // at the caster's own point (`005f2c80`): the target is the last
        // search's find, or none for the hero arm's `-1`.
        let at = self.s.units[self.u].pos;
        let t = if target >= 0 { self.found } else { None };
        self.s
            .add_cast_order_on(self.u, spell, t, at, QueuePos::First, false);
    }
}

impl Sim {
    /// `Unit::think_spellcaster@005f27a0`: [`think_spellcaster_steps`] over
    /// the unit. Returns whether a cast was ordered (the original's 1).
    pub fn think_spellcaster(&mut self, u: usize) -> bool {
        let who = self.units[u].owner;
        let mine = (0..self.units.len())
            .filter(|&i| self.units[i].owner == who)
            .collect();
        think_spellcaster_steps(&mut SimCaster {
            s: self,
            u,
            mine,
            found: None,
        })
    }

    /// `Search::valid_search@0067daa0(leader, mode, who)`: whether a
    /// candidate of `whom` is in the search `mode` of `who` (0 all, 1 own,
    /// 2 at peace, 3 enemy, 4 allied, 5 not allied, 6 not own, 7 not
    /// enemy), each asked of the candidate's leader.
    pub(crate) fn valid_search(&self, mode: i32, who: Player, whom: Player) -> bool {
        match mode {
            1 => whom == who,
            2 => !self.is_ally(whom, who) && !self.is_enemy(whom, who),
            3 => self.is_enemy(whom, who),
            4 => self.is_ally(whom, who),
            5 => !self.is_ally(whom, who),
            6 => whom != who,
            7 => !self.is_enemy(whom, who),
            _ => true,
        }
    }

    /// `valid_filter`'s `FILTER_SPELL_TARGET` arm (`0067e385`) for the
    /// craft `s` of `who`: `is_valid_target(s, who, o, whom, 0)`, then by
    /// the craft (the byte table at `0067e5e8`): Bribe and the Sniper
    /// (`0x275`, `0x281`) want the object undamaged (`+0x24 == 0`,
    /// `0067e425`), Counterintelligence (`0x277`) nothing more, the
    /// Informer (`0x27f`) a building the `0067e3c4` arm passes, and every
    /// other craft is refused (`0067e442`).
    fn spell_target_filter(&self, s: i32, who: Player, o: Obj) -> bool {
        use crate::ids::spell;
        if !self.spell_valid_target(s, who, o) {
            return false;
        }
        match s {
            spell::BRIBE | spell::SNIPER => match o {
                Obj::Unit(u) => self.units[u].max_health == self.units[u].health,
                Obj::Building(b) => self.buildings[b].hits == self.buildings[b].health,
            },
            spell::COUNTERINTEL => true,
            // SEAM: the Informer's building arm (`0067e3c4`: a building,
            // active, its vslot `+0x178` positive); no caller here asks it.
            _ => false,
        }
    }

    /// **`ObjectsData::find@0065c6b0(at, mode, who, range, ·,
    /// FILTER_SPELL_TARGET, s, who, ·, ·, ·, 0)`**, as
    /// `think_spellcaster` asks it for Counterintelligence: the nearest
    /// object, unit or building, of a leader below eight in the search
    /// `mode`, in use (`+8 & 1`), that the craft's filter keeps, within
    /// `range` by `vector_dist` of its own point — the **last** of a tie
    /// (`<=` against the running best, `first` 0). Under ten cells of
    /// `0x300` the walk is the circle's cells' chains round `at`'s cell;
    /// else each live leader's objects in number order (units below
    /// 2000, then buildings). `find`'s second filter is never read.
    pub(crate) fn caster_find(
        &self,
        s: i32,
        who: Player,
        mode: i32,
        at: Pos,
        range: i32,
    ) -> Option<Obj> {
        let mut best: Option<(i32, Obj)> = None;
        let mut take = |o: Obj| {
            let (whom, alive) = match o {
                Obj::Unit(u) => (self.units[u].owner, self.units[u].alive()),
                Obj::Building(b) => (self.buildings[b].owner, self.buildings[b].alive),
            };
            if whom >= crate::world::PLAYER_SLOTS
                || !alive
                || !self.valid_search(mode, who, whom)
                || !self.spell_target_filter(s, who, o)
            {
                return;
            }
            let p = self.pos_of(o);
            let d = crate::world::vector_dist((p.x - at.x).abs(), (p.y - at.y).abs());
            if (range < 0 || d <= range) && best.is_none_or(|(e, _)| d <= e) {
                best = Some((d, o));
            }
        };
        let ring = (range + 0x2ff) / 0x300;
        if range >= 0 && ring < 10 {
            let circ = crate::ai::ai_place::circle();
            let cell = at.cell();
            for k in 0..circ.radius[ring as usize] {
                let c = crate::Cell::new(cell.x + circ.x[k], cell.y + circ.y[k]);
                if c.x < 0 || c.y < 0 || c.x >= self.world.width() || c.y >= self.world.height() {
                    continue;
                }
                for o in self.object_chain(c) {
                    take(o);
                }
            }
        } else {
            let mut all: Vec<(Player, i32, Obj)> = (0..self.units.len())
                .map(|u| {
                    (
                        self.units[u].owner,
                        self.units[u].index as i32,
                        Obj::Unit(u),
                    )
                })
                .chain((0..self.buildings.len()).map(|b| {
                    (
                        self.buildings[b].owner,
                        self.buildings[b].index as i32,
                        Obj::Building(b),
                    )
                }))
                .filter(|&(w, _, _)| w < crate::world::PLAYER_SLOTS && self.player_alive(w))
                .collect();
            all.sort_by_key(|&(w, i, _)| (w, i));
            for (_, _, o) in all {
                take(o);
            }
        }
        best.map(|(_, o)| o)
    }

    /// **`ObjectsData::find_unit@0065ca80(at, SEARCH_ENEMY, who, range, 0,
    /// FILTER_SPELL_TARGET, s, who, FILTER_COMBAT, 0, 0)`**, as
    /// `think_spellcaster` asks it for Bribe and the Sniper (`005f2a74`,
    /// `005f2b5a`): the nearest enemy unit, in use and on the map, that the
    /// craft's filter keeps and that `FILTER_COMBAT` keeps (`0067debf`: its
    /// type's attack, `+0x1e8`, and the combat role, `+0x2c8 & 0x10000`),
    /// by [`Sim::find_unit_walk`].
    pub(crate) fn caster_find_unit(
        &self,
        s: i32,
        who: Player,
        at: Pos,
        range: i32,
    ) -> Option<usize> {
        self.find_unit_walk(at, range, |i| {
            let u = &self.units[i];
            u.alive()
                && u.on_map
                && self.valid_search(3, who, u.owner)
                && self.spell_target_filter(s, who, Obj::Unit(i))
                && self.attack_of(Obj::Unit(i)) != 0
                && self.profile(Obj::Unit(i)).combat_role
        })
        .map(|(_, i)| i)
    }
}

#[cfg(test)]
mod tests;
