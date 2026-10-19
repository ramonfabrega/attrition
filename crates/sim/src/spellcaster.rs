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
//! SEAMS, each drawing nothing where it stands:
//! - **the targets**: every target search goes through
//!   [`Sim::spell_valid_target`], which since item 2971 reads the Sniper's
//!   and Counterintelligence's arms (`docs/GOLDEN.md` §115); Bribe's
//!   (item 2812) too, and since item 2994 Sabotage's and Pilfer's
//!   (`docs/GOLDEN.md` §116);
//! - **the Spy's cloak**: not carried, so a Spy is read as uncloaked and
//!   returns 0 — the original's Spy arm draws nothing either way;
//! - ~~**the hero arm**~~: built as [`Sim::think_hero`] (item 1305,
//!   `docs/AI.md` §99.13), reached from `Army::use_generals`;
//! - **the human arm**, which `Unit::think`'s special turn reaches: it
//!   draws nothing and casts only through the same refused targets.

use crate::Sim;
use crate::orders::QueuePos;

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
    /// Bribe, the Spy's.
    pub const BRIBE: i32 = 0x275;
    /// Counterintelligence, a Spy's and a Scout's.
    pub const COUNTERINTEL: i32 = 0x277;
    /// Sniper, the special unit's.
    pub const SNIPER: i32 = 0x281;
}

/// `find`'s search mode (its third argument).
pub mod search {
    /// `SEARCH_INDEX_SEARCH_ALL`.
    pub const ALL: i32 = 0;
    /// `SEARCH_FRIENDLY`.
    pub const FRIENDLY: i32 = 1;
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
/// Forced March does beyond its flags (`crate::cast`).
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

/// [`CasterView`] over the crate's [`Sim`] for the unit `u`. The targets
/// and the casting predicates are the module's SEAMs (above): the Spy is
/// never cloaked, `is_castable` and the searches answer no.
struct SimCaster<'a> {
    s: &'a mut Sim,
    u: usize,
    /// The leader's units, in the order the original's list holds them.
    mine: Vec<usize>,
}

impl CasterView for SimCaster<'_> {
    fn who(&self) -> i32 {
        i32::from(self.s.units[self.u].owner)
    }
    fn human(&self) -> bool {
        self.s
            .nation
            .get(self.who() as usize)
            .is_some_and(|n| n.human)
    }
    fn forced_march(&self) -> bool {
        self.s.forced_march[self.who() as usize]
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
        // SEAM: the cloak (module doc); a Spy is read as uncloaked.
        false
    }
    fn avg_speed(&self) -> i32 {
        let unit = &self.s.units[self.u];
        unit.guys
            .first()
            .and_then(|g| g.follow)
            .map_or(unit.movement.body.avg_speed, |f| f.body.avg_speed)
    }
    fn castable(&mut self, _spell: i32) -> bool {
        // SEAM: the Sniper's and Counterintelligence's searches (module doc).
        false
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
    fn range(&mut self, _spell: i32) -> i32 {
        0
    }
    fn find(&mut self, _spell: i32, _mode: i32, _range: i32) -> i32 {
        -1
    }
    fn find_unit(&mut self, _spell: i32, _range: i32) -> i32 {
        -1
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
        self.s.rng.roll() as i32
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
        // A targeted cast is the SEAM of the searches; only the hero arm's
        // casts on the unit's own position are reached.
        debug_assert!(target < 0, "a targeted cast is unreachable");
        let at = self.s.units[self.u].pos;
        self.s
            .add_cast_order_on(self.u, spell, None, at, QueuePos::First, false);
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
        think_spellcaster_steps(&mut SimCaster { s: self, u, mine })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Tuning;
    use crate::ai_load::uflags2;
    use crate::world::{Cell, Terrain, World};

    /// A computer leader's scout on open land, in no army.
    fn ai_scout() -> (Sim, usize) {
        let mut world = World::new(10, 10);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(9, 9));
        let mut s = Sim::new(Tuning::RON, world, 2);
        s.nation[0].human = true;
        s.nation[1].human = false;
        let t = s.add_unit_type(crate::UnitType::default());
        s.unit_types[t].cols.unit_flags2 = uflags2::SCOUT | uflags2::CASTER;
        let mut u = crate::Unit::new(1, 0, crate::Pos::new(5 * 768 + 384, 5 * 768 + 384), 20);
        u.ty = Some(t);
        let u = s.add_unit(u);
        (s, u)
    }

    /// run346's frame 0 (`docs/AI.md` §80.5): at Easiest and Easy the arm
    /// returns before any draw, from Moderate up it throws the coin once,
    /// and with no army and a coin not divisible by three it returns 0.
    #[test]
    fn an_ai_scout_throws_the_special_coin_from_difficulty_two() {
        let (mut sim, scout) = ai_scout();
        for d in 0..=5 {
            sim.lobby.difficulty = d;
            let mut probe = sim.rng;
            let coin = probe.roll();
            let before = sim.rng;
            assert!(!sim.think_spellcaster(scout), "difficulty {d}");
            if d < 2 {
                assert_eq!(sim.rng, before, "difficulty {d}: no draw");
            } else {
                assert_eq!(sim.rng, probe, "difficulty {d}: one draw, {coin}");
            }
        }
    }

    /// A computer leader's General on the move at (5, 5), its type
    /// `MANA 1000`, and the three crafts of the hero arm priced 500, 500
    /// and 1,000. Returns the sim, the General and a seed per coin parity.
    fn ai_general() -> (Sim, usize, [u32; 2]) {
        use crate::orders::{MoveKind, QueuePos, SpellType, spell};
        let (mut s, _) = ai_scout();
        s.lobby.difficulty = 5;
        let t = s.add_unit_type(crate::UnitType::default());
        s.unit_types[t].cols.unit_flags2 = uflags2::GENERAL;
        s.unit_types[t].mana = 1000;
        // `TypeIndex` `0x36`, the General: the crafts' `FROM`.
        s.unit_types[t].tree = Some(0x36);
        let mut u = crate::Unit::new(1, 1, crate::Pos::new(5 * 768 + 384, 5 * 768 + 384), 20);
        u.ty = Some(t);
        u.captain = true;
        let g = s.add_unit(u);
        s.add_move_order(
            g,
            crate::Pos::new(8 * 768, 8 * 768),
            MoveKind::MoveTo,
            QueuePos::New,
            false,
        );
        s.units[g].movement.body.avg_speed = 5;
        let mut rows = vec![SpellType::default(); 55];
        let row = |x: i32| (x - spell::FIRST) as usize;
        for (craft, mana) in [
            (spell::FORCED_MARCH, 500),
            (spell::AMBUSH, 500),
            (spell::CREATE_DECOY, 1000),
        ] {
            rows[row(craft)] = SpellType {
                mana,
                job_time: 10,
                from: [Some(0x36), None],
                ..Default::default()
            };
        }
        s.spells = rows;
        let seed = |odd: i32| {
            (1..)
                .find(|&k| crate::combat::Rng::new(k).roll() & 1 == odd)
                .unwrap()
        };
        (s, g, [seed(0), seed(1)])
    }

    /// run470's frame 8856 (`docs/AI.md` §99.13): a moving hero throws
    /// the coin at `+0x589`; an odd one lays Forced March first, an even
    /// one reads the cell's owner — its own leader's, or no one's, and
    /// nothing is cast; an enemy's, and Ambush is.
    #[test]
    fn a_moving_general_throws_the_hero_coin() {
        use crate::orders::{Body, spell};
        use crate::world::Owner;
        let cast = |sim: &Sim, g: usize| match sim.units[g].orders.front().map(|o| o.body) {
            Some(Body::Cast(c)) => Some(c.spell),
            _ => None,
        };
        let (mut sim, g, [even, odd]) = ai_general();
        sim.rng = crate::combat::Rng::new(odd);
        let mut probe = sim.rng;
        probe.roll();
        assert!(sim.think_spellcaster(g));
        assert_eq!(sim.rng, probe, "one draw");
        assert_eq!(cast(&sim, g), Some(spell::FORCED_MARCH));

        let (mut sim, g, _) = ai_general();
        sim.rng = crate::combat::Rng::new(even);
        assert!(!sim.think_spellcaster(g), "no one's cell");
        assert_ne!(
            sim.rng,
            crate::combat::Rng::new(even),
            "the coin was thrown"
        );
        sim.world
            .set_owner(Cell::new(5, 5), Owner::Player(1), Owner::None);
        sim.rng = crate::combat::Rng::new(even);
        assert!(!sim.think_spellcaster(g), "its own leader's cell");
        sim.world
            .set_owner(Cell::new(5, 5), Owner::Player(0), Owner::None);
        sim.rng = crate::combat::Rng::new(even);
        assert!(sim.think_spellcaster(g), "an enemy's cell");
        assert_eq!(cast(&sim, g), Some(spell::AMBUSH));

        // Short of the craft's mana, nothing is cast after the coin.
        let (mut sim, g, _) = ai_general();
        sim.units[g].mana_burn = 501;
        sim.rng = crate::combat::Rng::new(odd);
        assert!(!sim.think_spellcaster(g));
        assert_ne!(sim.rng, crate::combat::Rng::new(odd));
    }

    /// run470's 9112 and 9240: Forced March, cast, raises the leader's
    /// `0x8000` and the hero's until `DURATION` runs out — no coin on the
    /// turn between — and a hero's craft of its own draws no animation
    /// roll (`set_anim(CHAR_ATTACK2, 0, 0)`).
    #[test]
    fn a_forced_march_holds_the_coin_until_it_runs_out() {
        use crate::orders::{Body, spell};
        let (mut sim, g, [_, odd]) = ai_general();
        let row = (spell::FORCED_MARCH - spell::FIRST) as usize;
        sim.spells[row].job_time = 0;
        sim.spells[row].duration = 150;
        sim.rng = crate::combat::Rng::new(odd);
        assert!(sim.think_spellcaster(g));
        let Some(Body::Cast(c)) = sim.units[g].orders.front().map(|o| o.body) else {
            panic!("a cast order");
        };
        let before = sim.rng;
        sim.do_cast(g, c);
        assert_eq!(sim.rng, before, "the attack slot rolls nothing");
        assert!(sim.forced_march[1]);
        assert_eq!(sim.units[g].marching, Some(sim.frame + 150));
        // The next turn: moving again, no coin while the march stands.
        let to = crate::Pos::new(8 * 768, 8 * 768);
        sim.add_move_order(
            g,
            to,
            crate::orders::MoveKind::MoveTo,
            crate::orders::QueuePos::New,
            false,
        );
        let before = sim.rng;
        assert!(!sim.think_spellcaster(g));
        assert_eq!(sim.rng, before, "no coin while marching");
        // Past the end frame the spell runs out and the flag with it.
        let end = sim.units[g].marching.unwrap();
        sim.process_spells(g, end);
        assert!(sim.forced_march[1], "held through its end frame");
        sim.process_spells(g, end + 1);
        assert!(!sim.forced_march[1] && sim.units[g].marching.is_none());
    }

    /// run511's block 9318 (`docs/AI.md` §99.15): under a Forced March
    /// every land unit of the hero's leader within the hero's radius —
    /// `GENERAL_RADIUS × 3 / 2` tiles at no upgrade, an economic patriot's
    /// one more — walks at `FORCED_MARCH_SPEED`, 42, unless it is faster
    /// already; one outside it, one of the sea, another leader's and every
    /// unit once the march runs out keep their own.
    #[test]
    fn a_forced_march_lifts_the_units_near_its_hero() {
        let (mut sim, g, _) = ai_general();
        let row = (crate::orders::spell::FORCED_MARCH - crate::orders::spell::FIRST) as usize;
        sim.spells[row].duration = 150;
        let at = sim.units[g].pos;
        let t = sim.add_unit_type(crate::UnitType::default());
        let mut o = 40;
        let mut near = |dx: i32, who: crate::Player, speed: i32| {
            o += 1;
            let mut u = crate::Unit::new(who, o, crate::Pos::new(at.x + dx, at.y), 20);
            u.ty = Some(t);
            u.movement.speed = speed;
            sim.add_unit(u)
        };
        // Nine tiles, 1,728 position units, is the General's reach.
        let (a, edge, out, fast, theirs) = (
            near(768, 1, 25),
            near(1728, 1, 25),
            near(1729, 1, 25),
            near(0, 1, 60),
            near(64, 0, 25),
        );
        assert_eq!(sim.unit_speed(a), 25, "no march yet");
        sim.cast_march(g);
        assert_eq!(sim.unit_speed(a), 42);
        assert_eq!(sim.unit_speed(edge), 42, "the radius is inclusive");
        assert_eq!(sim.unit_speed(out), 25, "one past it");
        assert_eq!(sim.unit_speed(fast), 60, "a march never slows a unit");
        assert_eq!(sim.unit_speed(theirs), 25, "another leader's");
        sim.unit_types[t].combat.domain = crate::attrition::Domain::Sea;
        sim.units[a].kind.domain = crate::attrition::Domain::Sea;
        assert_eq!(sim.unit_speed(a), 25, "a type not of the land");
        sim.unit_types[t].combat.domain = crate::attrition::Domain::Land;
        sim.units[a].kind.domain = crate::attrition::Domain::Land;
        // The Senator's reach: an economic patriot's tile more.
        sim.units[g].type_index = 0x161;
        assert_eq!(sim.unit_speed(out), 42, "ten tiles for a Senator");
        let end = sim.units[g].marching.unwrap();
        sim.process_spells(g, end + 1);
        assert_eq!(sim.unit_speed(a), 25, "the march has run out");
    }

    /// Before the coin: a guy standing (`avg_speed` 0) returns 0 with no
    /// draw, and a hero not moving takes the decoy arm — no draw, Create
    /// Decoys with the mana, and nothing while one of its leader's decoy
    /// captains stands.
    #[test]
    fn a_general_standing_draws_nothing_and_a_still_one_makes_decoys() {
        use crate::orders::{Body, spell};
        let (mut sim, g, _) = ai_general();
        sim.units[g].movement.body.avg_speed = 0;
        let before = sim.rng;
        assert!(!sim.think_spellcaster(g));
        assert_eq!(sim.rng, before, "avg_speed 0: no draw");

        let (mut sim, g, _) = ai_general();
        sim.kill_current_order(g);
        let before = sim.rng;
        assert!(sim.think_spellcaster(g));
        assert_eq!(sim.rng, before, "the decoy arm draws nothing");
        assert!(matches!(
            sim.units[g].orders.front().map(|o| o.body),
            Some(Body::Cast(c)) if c.spell == spell::CREATE_DECOY
        ));

        let (mut sim, g, _) = ai_general();
        sim.kill_current_order(g);
        let mut d = crate::Unit::new(1, 2, crate::Pos::new(2 * 768, 2 * 768), 20);
        d.captain = true;
        d.decoy = true;
        sim.add_unit(d);
        assert!(!sim.think_spellcaster(g), "a decoy captain stands");
    }

    /// A human's scout never reaches the computer's arm, and a decoy
    /// returns before the coin.
    #[test]
    fn a_human_s_scout_and_a_decoy_throw_nothing() {
        let (mut sim, scout) = ai_scout();
        sim.lobby.difficulty = 5;
        sim.units[scout].decoy = true;
        let before = sim.rng;
        assert!(!sim.think_spellcaster(scout));
        assert_eq!(sim.rng, before, "a decoy");
        sim.units[scout].decoy = false;
        sim.nation[1].human = true;
        assert!(!sim.think_spellcaster(scout));
        assert_eq!(sim.rng, before, "a human's");
    }
}
