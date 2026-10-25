//! **`Unit::suffer_attrition@005e1a10`** (item 3372): one tick of attrition on
//! one unit, as a pure walk over a [`SufferView`]. The swept steps read and
//! write the world only through the view, so the sweep's port is a view over
//! the emulated original's windows and the crate's is [`SimSuffer`]
//! (`docs/ATTRITION.md`, `docs/SUPPLY.md`).
//!
//! The control flow is the original's, call for call, in the order the
//! listing asks:
//!
//! 1. **the warning** (`5e1a25`..`5e1b8a`): once per leader until its stamp
//!    (`leader+0x1f8`) is set, and only for the local console's player on a
//!    tile that somebody else owns, two lines of text, a sound, and the
//!    stamp is written with the game's frame whether or not the lines showed.
//! 2. **the hurt** (`5e1b90`..`5e1bd1`): a type whose `uber_size` is 1 is
//!    dealt one whole point; any other is dealt [`attrition::damage`] of
//!    its squad's *current* size (the `curr_uber_size` walk), in sixteenths.
//!    Both go through the one virtual call at slot `0x16c`.
//! 3. **the graphic** (`5e1bfd`): a survivor gets `suffer_graphic_attrition`
//!    when the caller asked.
//!
//! Not modelled, each a leaf the port has no state for: the two lines of text,
//! the sound and the graphic (a headless sim has no console, so the warning's
//! arm is never entered: [`SimSuffer`] answers `console_who` -1); the stamp
//! itself, which nothing but this function reads.
//!
//! One finding the sweep holds and the port keeps: the warning asks
//! `is_ally(leaders[who], who)` — the unit's own player against itself, not
//! against the tile's owner — so with a true `is_ally(self, self)` the warning
//! could never show. Whether it does is `LeaderData::is_ally`'s, not this
//! function's.

use crate::Sim;
use crate::attrition;
use crate::combat;

/// `LeaderData::type_avail`'s FACTORY id.
pub const FACTORY: i32 = 0x1af;
/// The string-table offsets of the four lines.
pub const TEXT_OF_ATTRITION_A: i32 = 0x120c0;
pub const TEXT_OF_ATTRITION_B: i32 = 0x120d4;
pub const TEXT_NEED_FACTORY: i32 = 0x120e8;
pub const TEXT_NO_FACTORY: i32 = 0x120fc;
/// The leader kind (`leader+0xc`) that picks the first line's text.
pub const LEADER_KIND_FIRST_LINE: i32 = 0xd;
/// The sound `SoundGlobal::play` is handed.
pub const S_ATTRITION_WARNING: i32 = 0xa;

/// The arguments of the `0x16c` virtual call, in the order the original
/// pushes them: the whole points, the sixteenths, and the constants the
/// listing hands both arms (`1, -1, 1, 0x55555555, -1, -1, 0`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hurt {
    pub whole: i32,
    pub sixteenths: i32,
}

/// What `suffer_attrition` reads and does. Every method is one call or one
/// inline read of the listing.
pub trait SufferView {
    /// `this+9`, the unit's player byte.
    fn player(&self) -> usize;
    /// `leaders[who]+0x1f8`.
    fn warned_at(&self, who: usize) -> i32;
    /// `leaders[who]+0x1f8 = game.frame`.
    fn stamp_warned(&mut self, who: usize);
    /// `console->who`.
    fn console_who(&self) -> i32;
    /// The signed owner byte of the cell the unit stands in.
    fn tile_owner(&self) -> i32;
    /// `LeaderData::is_ally(leaders[who], other)`.
    fn is_ally(&mut self, who: usize, other: usize) -> bool;
    /// `leaders[who]+0xc`.
    fn leader_kind(&self, who: i32) -> i32;
    /// `LeaderData::type_avail(leaders[who], id, 1)`.
    fn type_avail(&mut self, who: usize, id: i32) -> bool;
    /// `String::operator=` of the table entry at `text`, then
    /// `MessageWin::add_message` at the unit's position; `shown` is the
    /// call's flag (1 for the first line) and `red` its colour.
    fn message(&mut self, text: i32, shown: i32, red: bool);
    /// `SoundGlobal::play`.
    fn play(&mut self, sound: i32);
    /// `*(type+0x308) == 1`.
    fn type_uber_one(&self) -> bool;
    /// `UnitData::curr_uber_size(this, 0)`.
    fn curr_uber_size(&mut self) -> i32;
    /// The virtual call at slot `0x16c`: nonzero when the unit fell.
    fn hurt(&mut self, h: Hurt) -> bool;
    /// `UnitOut::suffer_graphic_attrition`.
    fn graphic(&mut self);
}

/// The original's walk. `graphic` is the argument (`param_1 != 0`); the
/// answer is `true` when the hurt call answered nonzero.
pub fn suffer_attrition_steps<V: SufferView>(v: &mut V, graphic: bool) -> bool {
    let who = v.player();
    if v.warned_at(who) == 0 {
        if who as i32 == v.console_who() {
            let owner = v.tile_owner();
            if owner >= 0 && !v.is_ally(who, who) {
                let first = if v.leader_kind(owner) == LEADER_KIND_FIRST_LINE {
                    TEXT_OF_ATTRITION_A
                } else {
                    TEXT_OF_ATTRITION_B
                };
                v.message(first, 1, false);
                v.play(S_ATTRITION_WARNING);
                let second = if v.type_avail(who, FACTORY) {
                    TEXT_NEED_FACTORY
                } else {
                    TEXT_NO_FACTORY
                };
                v.message(second, 0, true);
            }
        }
        v.stamp_warned(who);
    }
    let h = if v.type_uber_one() {
        Hurt {
            whole: 1,
            sixteenths: 0,
        }
    } else {
        Hurt {
            whole: 0,
            sixteenths: attrition::damage(v.curr_uber_size()),
        }
    };
    if v.hurt(h) {
        return true;
    }
    if graphic {
        v.graphic();
    }
    false
}

/// [`SufferView`] over a [`Sim`]: the hurt is `Object::take_damage`'s
/// accumulator ([`combat::take`]), the squad is counted
/// ([`Sim::curr_uber_size`]), and everything the warning touches answers "no
/// console".
pub(crate) struct SimSuffer<'a> {
    pub(crate) sim: &'a mut Sim,
    pub(crate) unit: usize,
    /// What the hurt call did, for the caller's [`crate::Tick`].
    pub(crate) outcome: Option<(i32, i32, bool)>,
}

impl SufferView for SimSuffer<'_> {
    fn player(&self) -> usize {
        self.sim.units[self.unit].owner as usize
    }
    fn warned_at(&self, _who: usize) -> i32 {
        0
    }
    fn stamp_warned(&mut self, _who: usize) {}
    fn console_who(&self) -> i32 {
        -1
    }
    fn tile_owner(&self) -> i32 {
        -1
    }
    fn is_ally(&mut self, _who: usize, _other: usize) -> bool {
        true
    }
    fn leader_kind(&self, _who: i32) -> i32 {
        0
    }
    fn type_avail(&mut self, _who: usize, _id: i32) -> bool {
        false
    }
    fn message(&mut self, _text: i32, _shown: i32, _red: bool) {}
    fn play(&mut self, _sound: i32) {}
    fn type_uber_one(&self) -> bool {
        self.sim.profile(combat::Obj::Unit(self.unit)).uber_size == 1
    }
    fn curr_uber_size(&mut self) -> i32 {
        self.sim.curr_uber_size(self.unit)
    }
    fn hurt(&mut self, h: Hurt) -> bool {
        let sixteenths = h.whole * 16 + h.sixteenths;
        let (uber, alone) = {
            let u = &self.sim.units[self.unit];
            (
                self.sim.profile(combat::Obj::Unit(self.unit)).uber_size,
                u.combat.captain == i32::from(u.index) && u.squad_size == 1,
            )
        };
        let unit = &mut self.sim.units[self.unit];
        let share = combat::share(unit.max_health, uber, alone);
        let (taken, _, frac) = combat::take(
            unit.max_health - unit.health,
            unit.damage_frac,
            share,
            combat::Sixteenths {
                whole: sixteenths / 16,
                frac: sixteenths % 16,
            },
        );
        let (combat::Taken::Alive { lost } | combat::Taken::Died { lost, .. }) = taken;
        unit.damage_frac = frac;
        unit.health -= lost;
        let killed = matches!(taken, combat::Taken::Died { .. });
        if killed {
            unit.health = unit.health.min(0);
        }
        self.outcome = Some((sixteenths, lost, killed));
        killed
    }
    fn graphic(&mut self) {}
}
