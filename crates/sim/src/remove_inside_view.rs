//! **`Object::remove_from_inside@006480f0`** (item 3372): a unit leaving the
//! chain of whatever holds it, as a pure walk over a [`RemoveInsideView`].
//!
//! The listing in four parts, in the order it runs them:
//!
//! 1. **the host's word** — `ObjectData::get_inside@00651a80(&list)` names the
//!    container's slot in the objects table, and the container's vtable slot
//!    `0x20` answers a flag kept in `edi` for the tail. A negative slot is
//!    flag 0 and no call;
//! 2. **the gate** — `this`'s own slot `0x8` (a unit, not a building) and its
//!    unit record's `+0x82` (its `inside_up` index) being non-negative;
//! 3. **the splice** — the neighbour above (`objects[up_list][up]`) takes
//!    this unit's `inside_down` (`+0x28`, `+0x3e`), the neighbour below
//!    (`units[down_list][down]`, when `down >= 0`) takes its `inside_up`
//!    (`+0x82`, `+0xb4`); this unit's own `+0x82` and `+0x28` become `-1`;
//!    `add_to_world` last;
//! 4. **the tail** — the leader's garrison counter (peasant, Stable, scout,
//!    Barracks) down one, and the tribe's flag.
//!
//! The tail is [`garrison_tail`], shared with `Object::insert_inside@00647e90`'s
//! mirror (`delta` +1; that function is not swept). The crate keeps no
//! `inside_up`/`inside_down` words — a unit's holder is one pointer
//! (`Unit::inside`, `Unit::inside_unit`) — so [`SimRemoveInside`]'s splice is
//! empty and says it is in a chain.

/// The garrison counters the tail moves: `LeaderData +0xa28`, `+0xa2c`,
/// `+0xa30`, `+0xa34`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Garr {
    Peasants,
    Stable,
    Scout,
    Barracks,
}

/// The type word (`+0x40` of the type record) of a Stable.
pub const TYPE_STABLE: i32 = 0x1ac;
/// The type word of a Barracks.
pub const TYPE_BARRACKS: i32 = 0x1ab;
/// `LeaderData::has_tribe_bonus(0x13)`, the Lakota's.
pub const BONUS_LAKOTA: i32 = 0x13;
/// `has_tribe_bonus(0x14)`, the Americans'.
pub const BONUS_AMERICANS: i32 = 0x14;

/// What the walk reads and writes, one leaf a method.
pub trait RemoveInsideView {
    /// `get_inside(&list)`, then the container's vslot `0x20` when the slot
    /// is not negative: `edi != 0`.
    fn host_flag(&mut self) -> bool;
    /// `this`'s vslot `0x8`.
    fn is_unit(&mut self) -> bool;
    /// This unit's record `+0x82` (short) and `+0xb4` (byte).
    fn own_up(&mut self) -> (i32, i32);
    /// `this +0x28` (short) and `+0x3e` (byte).
    fn own_down(&mut self) -> (i32, i32);
    /// `objects[list][idx]`'s `+0x28`, `+0x3e` set.
    fn set_object_down(&mut self, list: i32, idx: i32, down: (i32, i32));
    /// `units[list][idx]`'s `+0x82`, `+0xb4` set.
    fn set_unit_up(&mut self, list: i32, idx: i32, up: (i32, i32));
    /// This unit's record `+0x82` and `+0x28` both `-1`.
    fn clear_own_links(&mut self);
    /// `add_to_world@0064d8c0(this)`.
    fn add_to_world(&mut self);
    /// The unit record's `+0x68 & 1`.
    fn masked(&mut self) -> bool;
    /// The unit record's vslot `0xe8`: `UnitData::is_captain`.
    fn is_captain(&mut self) -> bool;
    /// `ObjectData::is_peasant@0046d310(this)`.
    fn is_peasant(&mut self) -> bool;
    /// `+0x40` of `this`'s type record.
    fn type_word(&mut self) -> i32;
    /// `this`'s vslot `0xd4`: `is_special`, the scout bit.
    fn is_special(&mut self) -> bool;
    /// `LeaderData::has_tribe_bonus(id)` on this unit's leader.
    fn has_tribe_bonus(&mut self, id: i32) -> bool;
    /// `leaders[who].counter += delta`.
    fn bump(&mut self, which: Garr, delta: i32);
    /// `constants->lakota_food` (`+0x848`).
    fn lakota_food(&mut self) -> i32;
    /// `constants->americans_barracks_gather` (`+0x888`).
    fn barracks_gather(&mut self) -> i32;
    /// `leader_flags |= 0x2000000`.
    fn set_leader_flag(&mut self);
}

/// `remove_from_inside` from its entry to its return.
pub fn remove_from_inside_steps<V: RemoveInsideView>(v: &mut V) {
    let host = v.host_flag();
    if !v.is_unit() {
        return;
    }
    let (up, up_list) = v.own_up();
    if up < 0 {
        return;
    }
    let down = v.own_down();
    v.set_object_down(up_list, up, down);
    if down.0 >= 0 {
        v.set_unit_up(down.1, down.0, (up, up_list));
    }
    v.clear_own_links();
    v.add_to_world();
    if v.masked() {
        return;
    }
    if !v.is_captain() || !host {
        return;
    }
    garrison_tail(v, -1);
}

/// The listing's last arm (`648280`..`6483a9`): the counter whose kind this
/// unit is, moved by `delta`, then the tribe's flag. The first kind that
/// matches wins — peasant, a Stable type, `is_special`, a Barracks type — and
/// only the Barracks arm asks for the Americans' bonus and constant.
pub fn garrison_tail<V: RemoveInsideView>(v: &mut V, delta: i32) {
    let (which, bonus) = if v.is_peasant() {
        (Garr::Peasants, BONUS_LAKOTA)
    } else if v.type_word() == TYPE_STABLE {
        (Garr::Stable, BONUS_LAKOTA)
    } else if v.is_special() {
        (Garr::Scout, BONUS_LAKOTA)
    } else if v.type_word() == TYPE_BARRACKS {
        (Garr::Barracks, BONUS_AMERICANS)
    } else {
        return;
    };
    v.bump(which, delta);
    if !v.has_tribe_bonus(bonus) {
        return;
    }
    let on = if bonus == BONUS_LAKOTA {
        v.lakota_food()
    } else {
        v.barracks_gather()
    };
    if on != 0 {
        v.set_leader_flag();
    }
}

/// [`RemoveInsideView`] over a [`crate::Sim`] for the captain `unit` leaving
/// its building.
///
/// SEAM: the container's slot `0x20` is not modelled (the view answers true),
/// the unit record's `+0x68 & 1` is not read (the view answers false), and the
/// chain words do not exist, so the splice is empty and the unit says it is in
/// a chain. The Americans' Barracks gather constant is not in the tuning (the
/// view answers 0): the Barracks arm counts and raises nothing.
pub(crate) struct SimRemoveInside<'a> {
    pub(crate) sim: &'a mut crate::Sim,
    pub(crate) unit: usize,
}

impl RemoveInsideView for SimRemoveInside<'_> {
    fn host_flag(&mut self) -> bool {
        true
    }
    fn is_unit(&mut self) -> bool {
        true
    }
    fn own_up(&mut self) -> (i32, i32) {
        (0, 0)
    }
    fn own_down(&mut self) -> (i32, i32) {
        (-1, 0)
    }
    fn set_object_down(&mut self, _list: i32, _idx: i32, _down: (i32, i32)) {}
    fn set_unit_up(&mut self, _list: i32, _idx: i32, _up: (i32, i32)) {}
    fn clear_own_links(&mut self) {}
    fn add_to_world(&mut self) {}
    fn masked(&mut self) -> bool {
        false
    }
    fn is_captain(&mut self) -> bool {
        self.sim.units[self.unit].captain
    }
    fn is_peasant(&mut self) -> bool {
        self.sim.units[self.unit].ty.is_some_and(|t| {
            matches!(
                self.sim.unit_types[t].worker,
                crate::orders::Worker::Citizen
            )
        })
    }
    fn type_word(&mut self) -> i32 {
        let Some(t) = self.sim.units[self.unit].ty else {
            return 0;
        };
        match self.sim.trainer_ident(t) {
            Some(crate::build::Ident::Stable) => TYPE_STABLE,
            Some(crate::build::Ident::Barracks) => TYPE_BARRACKS,
            _ => 0,
        }
    }
    fn is_special(&mut self) -> bool {
        // `UnitData::is_special@0046cea0`: `unit_flags2 & 0x10`, the land
        // scout line alone — a Bark (the role word's other `0x10`) is not
        // one, and a Bark born at a dock raises nothing.
        self.sim.units[self.unit].ty.is_some_and(|t| {
            self.sim.unit_types[t]
                .cols
                .flag2(crate::ai_load::uflags2::SCOUT)
        })
    }
    fn has_tribe_bonus(&mut self, id: i32) -> bool {
        let who = self.sim.units[self.unit].owner as usize;
        id == BONUS_LAKOTA && self.sim.nation.get(who).is_some_and(|n| n.lakota)
    }
    fn bump(&mut self, which: Garr, delta: i32) {
        let who = self.sim.units[self.unit].owner as usize;
        let census = &mut self.sim.ai[who].census;
        match which {
            Garr::Peasants => census.peasants_garr += delta,
            Garr::Stable => census.stable_garr += delta,
            Garr::Scout => census.scout_garr += delta,
            Garr::Barracks => census.barracks_garr += delta,
        }
    }
    fn lakota_food(&mut self) -> i32 {
        self.sim.tuning.lakota_food
    }
    fn barracks_gather(&mut self) -> i32 {
        0
    }
    fn set_leader_flag(&mut self) {
        let who = self.sim.units[self.unit].owner;
        self.sim.economy_changed(who);
    }
}
