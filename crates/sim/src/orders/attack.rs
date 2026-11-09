//! The attack order: `Unit::fight`'s order side, chase, ground attack, retarget and the target-path check (STRUCTURE §5, move 7).

use super::*;

/// `ACCEL_CONSTRUCT`-independent literals of the mechanic, position units.
/// `TypeIndex` `MERCHANTDUTCH` — `think_attack`'s packed arm skips its
/// lineage (`is(0x3e, 1)`).
pub(super) const MERCHANTDUTCH: crate::tech::TypeId = 0x3e;

/// `TypeIndex` `MACHINEGUN` — the lineage whose packed arm unpacks at
/// `idle 3` and whose unpack `add_cast_order` re-aims at `0x28e`.
pub(super) const MACHINEGUN: crate::tech::TypeId = 0x7b;

/// What `Unit::do_attack_ground@005f1410` asks of its unit and the world:
/// the one body [`do_attack_ground_steps`] runs over [`Sim`] (`SimAttackGround`)
/// and, in `crates/sim/tests/sweep/unit_do_attack_ground.rs`, over the emulated
/// original's stubs. The attack-unit-0 arm (a player's ground click: the
/// peace test, the range test, the nearby-spot search, the walk-in) is the
/// SEAM the steps keep as a kill.
pub trait AttackGroundView {
    fn can_attack_ground(&mut self) -> bool;
    fn kill_current_order(&mut self);
    /// The order's point (`+4`, `+8`).
    fn at(&self) -> (i32, i32);
    /// The unit's own square, decoded.
    fn pos(&self) -> (i32, i32);
    fn find_angle(&mut self, dx: i32, dy: i32) -> i32;
    /// `unit_flags & 0x40`.
    fn sideways(&mut self) -> bool;
    /// `unit_flags & 0x2000000`.
    fn rocks(&self) -> bool;
    fn heading(&self) -> i32;
    fn recharging(&self) -> u8;
    /// The order's flag byte, read signed: bit 7.
    fn fired(&self) -> bool;
    /// Guy 0's current animation slot, a signed byte.
    fn guy0_anim(&self) -> i8;
    fn set_anim(&mut self, anim: i8, force: bool, p3: bool);
    /// A bookkeeping hook of the port's alone.
    fn reload_idle_site(&mut self) {}
    fn attack_unit(&self) -> u8;
    fn set_attack_unit(&mut self, n: u8);
    fn work(&mut self);
    fn clear_attack(&mut self);
    /// The type's packer bit and the unit's packed mask.
    fn packs_and_packed(&self) -> bool;
    fn unpack(&mut self);
    /// `unit_masks |= 0x11000`.
    fn mark_masks(&mut self);
    fn set_angle(&mut self, angle: i32);
    fn set_fired(&mut self);
    /// The type's fire graphic (`+0x2cc`) is set, so `fire_ammo(-1, -1)` runs.
    fn fires(&self) -> bool;
    fn fire_ammo(&mut self);
    fn recharge(&mut self) -> u8;
    fn set_recharging(&mut self, n: u8);
}

/// **`Unit::do_attack_ground@005f1410`** over a view (`docs/COMBAT.md` §57.3,
/// `docs/ORDERS.md` §26), in the original's order; see [`Sim::do_attack_ground`].
pub fn do_attack_ground_steps<V: AttackGroundView>(v: &mut V) {
    if !v.can_attack_ground() || v.attack_unit() == 0 {
        v.kill_current_order();
        return;
    }
    let (ax, ay) = v.at();
    let (px, py) = v.pos();
    let (dx, dy) = (ax.wrapping_sub(px), ay.wrapping_sub(py));
    let direct = v.find_angle(dx, dy);
    let mut facing = direct;
    if v.sideways() {
        const QUARTER: i32 = crate::ids::scale::QUARTER_TURN;
        let heading = v.heading();
        let off = |a: i32| {
            let d = heading.wrapping_sub(a) as u32;
            if d > 0x8000_0000 { !d } else { d }
        };
        let minus = direct.wrapping_sub(QUARTER);
        let plus = direct.wrapping_add(QUARTER);
        facing = if off(minus) < off(plus) { minus } else { plus };
    }
    if v.recharging() != 0 {
        if v.fired() {
            return;
        }
        if (0..4).contains(&v.guy0_anim()) {
            return;
        }
        v.reload_idle_site();
        v.set_anim(anim::DEFAULT, true, true);
        return;
    }
    let shots = v.attack_unit();
    if shots == 1 {
        v.kill_current_order();
        v.work();
        return;
    }
    v.set_attack_unit(1);
    v.clear_attack();
    if v.packs_and_packed() {
        v.unpack();
        return;
    }
    v.mark_masks();
    if facing != v.heading() {
        v.set_angle(facing);
    }
    v.set_fired();
    if v.rocks() {
        let again = v.find_angle(dx, dy);
        let slot = if again.wrapping_sub(facing) >= 0 {
            anim::ATTACK2
        } else {
            anim::ATTACK1
        };
        v.set_anim(slot, false, false);
    } else {
        v.set_anim(anim::ATTACK1, false, true);
    }
    if v.fires() {
        v.fire_ammo();
    }
    let r = v.recharge();
    v.set_recharging(r.wrapping_add(1));
}

/// [`AttackGroundView`] over the live [`Sim`].
pub(super) struct SimAttackGround<'a> {
    pub(super) s: &'a mut Sim,
    pub(super) u: usize,
    pub(super) frame: i64,
    pub(super) g: AttackGroundOrder,
}

impl AttackGroundView for SimAttackGround<'_> {
    fn can_attack_ground(&mut self) -> bool {
        self.s.can_attack_ground(self.u)
    }
    fn kill_current_order(&mut self) {
        self.s.kill_current_order(self.u);
    }
    fn at(&self) -> (i32, i32) {
        (self.g.at.x, self.g.at.y)
    }
    fn pos(&self) -> (i32, i32) {
        let p = self.s.units[self.u].pos;
        (p.x, p.y)
    }
    fn find_angle(&mut self, dx: i32, dy: i32) -> i32 {
        crate::movement::find_angle(dx, dy).0
    }
    fn sideways(&mut self) -> bool {
        self.s.units[self.u].ty.is_some_and(|t| {
            self.s.tables.unit_types[t]
                .cols
                .flag(crate::ai_load::uflags::SIDEWAYS)
        })
    }
    fn rocks(&self) -> bool {
        self.s.units[self.u].ty.is_some_and(|t| {
            self.s.tables.unit_types[t]
                .cols
                .flag(crate::ai_load::uflags::ROCKS)
        })
    }
    fn heading(&self) -> i32 {
        self.s.units[self.u].movement.heading.0
    }
    fn recharging(&self) -> u8 {
        self.s.units[self.u].combat.recharging
    }
    fn fired(&self) -> bool {
        self.s
            .current_order(self.u)
            .is_some_and(|o| o.flags & flag::FIRED != 0)
    }
    fn guy0_anim(&self) -> i8 {
        self.s.units[self.u].guys.first().map_or(-1, |g| g.anim)
    }
    fn set_anim(&mut self, anim: i8, force: bool, p3: bool) {
        self.s.set_anim(self.u, anim, force, p3);
    }
    fn reload_idle_site(&mut self) {
        self.s.mark(anim::SITE_RELOAD_IDLE);
    }
    fn attack_unit(&self) -> u8 {
        self.g.attack_unit
    }
    fn set_attack_unit(&mut self, n: u8) {
        self.g.attack_unit = n;
        let g = self.g;
        if let Some(front) = self.s.units[self.u].orders.front_mut() {
            front.body = Body::AttackGround(g);
        }
    }
    fn work(&mut self) {
        self.s.work(self.u, self.frame);
    }
    fn clear_attack(&mut self) {
        self.s.clear_attack(self.u);
    }
    fn packs_and_packed(&self) -> bool {
        self.s.units[self.u]
            .ty
            .is_some_and(|t| self.s.tables.unit_types[t].combat.packs)
            && self.s.units[self.u].combat.packed
    }
    fn unpack(&mut self) {
        self.s
            .add_cast_order_at(self.u, spell::UNPACK, QueuePos::First);
    }
    fn mark_masks(&mut self) {
        // `unit_masks |= 0x11000` (item 2785).
        self.s.units[self.u].blown = true;
        self.s.units[self.u].fought = true;
    }
    fn set_angle(&mut self, angle: i32) {
        self.s.unit_set_angle(self.u, crate::movement::Angle(angle));
    }
    fn set_fired(&mut self) {
        if let Some(front) = self.s.units[self.u].orders.front_mut() {
            front.flags |= flag::FIRED;
        }
    }
    fn fires(&self) -> bool {
        !(self.s.max_range_of(Obj::Unit(self.u)) == 0 || self.s.launches_from_anim(self.u))
    }
    fn fire_ammo(&mut self) {
        let from = self.s.units[self.u].pos;
        let direct = crate::movement::find_angle(self.g.at.x - from.x, self.g.at.y - from.y);
        let sz = self.s.ground_z(from) + 100;
        self.s.fire_ammo_ground(
            Obj::Unit(self.u),
            self.g,
            direct,
            self.frame,
            from,
            sz,
            0,
            false,
            false,
            // SEAM: a shot no animation launches has no release event, so
            // no ammo name to read `flame` or `puncture` off
            // (`docs/COMBAT.md` §9.0).
            false,
            false,
        );
    }
    fn recharge(&mut self) -> u8 {
        self.s.reload_frames(self.u)
    }
    fn set_recharging(&mut self, n: u8) {
        self.s.units[self.u].combat.recharging = n;
    }
}

/// `UnitTypeData::can_attack_ground@0061db20` past its rush-rules gate: a
/// `max_range` (`+0x1fc`), neither of `unit_flags & 0x102000` (`+0x2b4`),
/// then `has_objmask` asked EXPLOSIVE, SIEGE, NAVAL and BOMBARD in that
/// order (`obj_masks`, `+0x1e4`, unless slot `+0xf0` is overridden), and
/// last `is(MACHINEGUN, 0)` (slot `+0x60`). Item 2247's sweep holds it.
pub(crate) fn ground_attacker(
    max_range: i32,
    barred: bool,
    mut has: impl FnMut(u32) -> bool,
    machinegun: impl FnOnce() -> bool,
) -> bool {
    use crate::combat::mask;
    if max_range == 0 || barred {
        return false;
    }
    has(mask::EXPLOSIVE)
        || has(mask::SIEGE)
        || has(mask::NAVAL)
        || has(mask::BOMBARD)
        || machinegun()
}

impl Sim {
    // ------------------------------------------------------------------
    // The attack order — §7.2
    // ------------------------------------------------------------------

    /// **`fight`'s capture approach** (`Unit::fight@005fd4d0`,
    /// `005fd8d1`–`005fda9b`; `docs/ORDERS.md` §7.2). `Object::valid_target`
    /// refused the target and said, through its out-word, that the target is
    /// a capture-eligible city building the call's own capture did not
    /// take. Under a mandatory order, or on a computer's unit (`unit_masks &
    /// 0x40000`), a **land** unit (type `+0x218 == 0`) that can capture
    /// (`UnitData::get_capture_value`, vslot `+0xec`, non-zero) — or a
    /// computer's siege unit (type vslot `+0x10c`), which counts nothing —
    /// further than `city_capture_radius × 0xc0` from the city, centre to
    /// centre (`vector_dist` at `005fd99c`), asks `find_nearby_spot(0, 0x180,
    /// 0, 0x55555555, FILTER_NOT_ME)` round the **midpoint** of the two
    /// centres (each half `(a + b) / 2`, truncated) and walks there:
    /// `add_move_order(spot, 1, 0, QUEUE_FIRST, 0)`, the attack kept under
    /// it. The siege case then drags its army onto the city
    /// (`Army::charge(ox, whom)`). Answers whether it walked, which is
    /// `fight`'s `return 0`; otherwise the refusal goes on to
    /// `find_new_target` as before.
    ///
    /// run1171's `1/49` (`LIGHTTANK`, 239, land), on an `ATTACKORDER` on
    /// Napata `0/2000` reduced on 4341, is 3107 from the city on 4342 and
    /// walks to (6360, 8568), the spot round (6372, 8556).
    pub(crate) fn capture_approach(&mut self, u: usize, target: Obj, mandatory: bool) -> bool {
        let ai = self.unit_ai_bit(self.units[u].owner);
        if !mandatory && !ai {
            return false;
        }
        let siege = ai && self.is_siege_unit(u);
        let unit = &self.units[u];
        if unit.kind.domain != crate::attrition::Domain::Land {
            return false;
        }
        let can_capture = !unit.decoy && !self.is_siege_unit(u);
        if !can_capture && !siege {
            return false;
        }
        let (m, t) = (unit.pos, self.pos_of(target));
        if vector_dist((m.x - t.x).abs(), (m.y - t.y).abs())
            <= self.tables.tuning.city_capture_radius * 0xc0
        {
            return false;
        }
        let mid = Pos::new((t.x + m.x) / 2, (t.y + m.y) / 2);
        let Some(spot) = self.find_nearby_spot(u, mid, 0, 0x180, 0, Angle(0x5555_5555), None)
        else {
            return false;
        };
        self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::First, false);
        if siege && let Some(slot) = self.army_of(u) {
            self.army_charge(self.units[u].owner, slot, target);
        }
        true
    }

    /// `Unit::do_attack` → `fight`'s order-management half, on the existing
    /// attack step: the target check, the reload gate with `new_ord`, the
    /// 1/5 re-search, range → strike, out of range → the chase inserted in
    /// front.
    pub(crate) fn do_attack_fight(&mut self, u: usize, frame: i64) -> bool {
        let Some(Order {
            body: Body::Attack(mut a),
            flags,
        }) = self.current_order(u).copied()
        else {
            return false;
        };
        let me = Obj::Unit(u);
        // **`do_attack`'s own gate is the type's `attack`, and it is the
        // only aliveness test the function has** (item 463).
        // `Unit::do_attack@005f1b80` tests the target object's `flags & 1`
        // in exactly two places — the strafe arm and the grouped arm —
        // and *both* sit under `*(int *)(ptype + 0x1e8) == 0`, which the
        // type record names `attack`. A unit whose type can attack falls
        // through both to `LAB_005f2224`'s unconditional `fight(...)`
        // without ever asking whether its target still exists. So the
        // validity kill below is `fight`'s, not this function's, and it
        // sits where `fight` puts it: **behind the reload gate**.
        if self.attack_of(me) == 0 {
            self.kill_current_order(u);
            return false;
        }
        let state = self.units[u].combat;
        // **A mandatory order ignores the stance.** `Unit::do_attack@
        // 005f1b80:225`-`244`: the order's `+0x1c` (mandatory) jumps
        // straight to `LAB_005f2224`, which is the unconditional
        // `fight(...)` — every stance test in the function sits under
        // `mandatory == 0`. The probe of `docs/COMBAT.md` §17 sets
        // `action_stance(5)` — HOLD_FIRE — on the six units it then
        // gives a **mandatory** attack order to, and run19's block 8187
        // has all six chasing, so this arm is what lets them.
        if state.stance == combat::Stance::HoldFire && !state.mandatory {
            return false;
        }
        // **The cell-centre snap at `fight`'s own entry** (§7.11).
        // `Unit::fight@005fd4d0`, `LAB_005fd648`:
        // `set_new_location(this, div_3_table[x >> 4] * 0x30 + 0x18, …, 1,
        // 0)` — the unit is put on its 48-unit cell centre before the
        // function does anything else. §7.11's table has four rows and
        // this is the first: the snap is skipped **only** by a recharging
        // unit that is neither §7.10's re-entry latch on an invalid target
        // nor a melee-and-ranged type (`unit_flags & 0x400`). Both rows
        // are built: the latch by item 3540, the type by item 3568.
        //
        // **The emplaced weapon's latch** (item 3540, `docs/ORDERS.md`
        // §7.12): `Unit::fight@005fd4d0:94`–`102`. `local_1c` is set when
        // the type's `unit_flags2 & 1` ([`uflags2::EMPLACED`], the machine
        // gun and flamethrower lineages) is set **and** the order's target
        // fails `Object::valid_target` (or its `uid` no longer matches);
        // the latch walks past the reload gate, so such a unit drops a dead
        // target's order on the next frame whatever its `recharging`.
        // run1184's Flamethrowers `1/84` and `1/129` (TypeIndex 131) drop
        // their attack on the human's dead building `0/2004` on block 4345
        // with `recharging` 23 and 12. The latch's other half — the
        // current order's [`flag::FIGHT_REENTRY`] on a valid target — is a
        // SEAM: it would enter the strike arms while recharging, and no
        // capture has an emplaced weapon there.
        let emplaced = self.units[u]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].cols.flag2(uflags2::EMPLACED));
        let latch = emplaced
            && match self.units[u].combat.target {
                Some(t) => !self.valid_target(me, t),
                None => true,
            };
        // **A melee-and-ranged type walks past the reload gate** (item
        // 3568, `docs/AI.md` §308): `fight@005fd4d0:106`'s `unit_flags &
        // 0x400` test sits ahead of the idle block, and a set bit falls
        // through to `LAB_005fd648`'s snap and on into the function, as the
        // latch does. The Mahout (TypeIndex 262, `mjvyk`) is the witness:
        // run1189b's `1/46` reads `recharging` 35 on block 4170 and theirs
        // rolls the guard's `fight+0x824` under its `ATTACKORDER`, where
        // ours stood and rolled the reload idle.
        let walks_on = self.units[u].ty.is_some_and(|t| {
            self.tables.unit_types[t]
                .cols
                .flag(uflags::MELEE_AND_RANGED)
        });
        if state.recharging == 0 || latch || walks_on {
            let snap = crate::collide::ucell_centre(crate::collide::ucell(self.units[u].pos));
            self.set_new_location(u, snap, true);
        }
        // **The reload gate**, `Unit::fight@005fd4d0:100-127`. A recharging
        // unit returns, and it does not turn. On a **fresh** order
        // (`new_ord`, the order's `+0x20`) whose target guy 0 is not
        // already aimed at (`GuyData +0x8e`/`+0x9f`), and whose guy 0 is
        // not on a slot below 4, it first asks for the idle:
        // `Unit::set_anim(CHAR_DEFAULT, 1, 1)`, which is the idle roll
        // (`Guy::set_anim+0x97a`).
        //
        // ~~Turns toward the target and returns~~ — the first reading's
        // (the orders landing), and never diffed until item 530. Golden
        // chapter one's `1/7` is the witness. It takes an attack on `0/8`
        // from its attack-move's look on 773 while recharging 13. On 774
        // it stands with its heading unchanged and rolls the idle, and
        // that roll is the chapter's word (`docs/ORDERS.md` §22).
        if state.recharging != 0 && !latch && !walks_on {
            if a.new_ord
                && let Some(target) = self.units[u].combat.target
                && let Some(g0) = self.units[u].guys.first()
                && g0.aim != Some(target)
                && !(0..4).contains(&g0.anim)
            {
                self.mark(anim::SITE_RELOAD_IDLE);
                self.set_anim(u, anim::DEFAULT, true, true);
            }
            return false;
        }
        // **`fight`'s own target test, and it is downstream of the gate
        // above** — `Unit::fight@005fd4d0:196`'s `Object::valid_target`,
        // which the recharging arm at `:102` returns before reaching.
        // `Unit::do_attack@005f1b80` reads **no** order flag — grep it —
        // so the `flags & 0x10` half this test used to carry was nobody's
        // reading and, since nothing wrote the bit, a clause that could
        // not fire. Item 329 gave the bit its real writer
        // ([`flag::FIGHT_REENTRY`]) and its real readers, both in `fight`.
        //
        // ~~SEAM: `fight`'s head does reach the validity question while
        // recharging, through `local_1c` … unmodelled~~ — item 3540 built
        // its invalid-target half (`latch` above); run1184 has two
        // Flamethrowers in it.
        // **The follower takes its captain's target here, before the
        // validity test — and that is how two of three bowmen strike on
        // the frame the third spends searching** (item 502,
        // `docs/COMBAT.md` §43.2).
        //
        // `Unit::fight@005fd4d0`, the block between the cell-centre snap
        // and `:196`'s `Object::valid_target`. It runs for a unit that is
        // **not** a captain (`vtable+0xe8`, `o_up >> 15`) whose
        // `collide_frame` is older than the frame before this one, reads
        // the captain's **action** through `UnitData::get_captain`
        // (`+0xe4`, which recurses to the head of the chain) and
        // `get_action`, and takes its `(ox, whom, uid)` when all of:
        //
        // - the captain's action is an ATTACK;
        // - its target is not the one this unit already names;
        // - `Object::valid_target` passes **on this unit**;
        // - the unit's combat stance is below `STAND_GROUND`, or the
        //   captain's target is already in range.
        //
        // It writes the pair into the **existing** order rather than
        // adding one, and the dump says so: run112's `0/7` and `0/8`
        // carry `ox 6 whom 1 uid 12` at block 696 with `new_ord 0` and
        // `ever_in_range 1` — the same order object, retargeted — where
        // their captain `0/6` carries the same target with `new_ord 1`
        // and `ever_in_range 0`, which is what a fresh `add_attack_order`
        // leaves. Those two fields are the value diff that tells the two
        // mechanisms apart, and they were on disk before this landing.
        //
        // This is a **second** captain mirror, and it is not
        // [`Self::captain_mirror`]: that one is `Unit::think`'s opening
        // arm (§21), runs only on an idle unit and adds a `QUEUE_NEW`
        // order. This one runs on the attack step of a unit that already
        // has an order, and it is why a squad does not lose a frame each
        // time its captain retargets.
        if !self.units[u].captain && self.units[u].collide_frame < frame - 1 {
            let cap = self.squad_captain(u);
            if cap != u
                && let Some(k) = self.action_of(cap)
                && matches!(self.units[cap].orders[k].body, Body::Attack(_))
                && let Some(t) = self.units[cap].combat.target
                && Some(t) != self.units[u].combat.target
                && self.valid_target(me, t)
                && (matches!(
                    state.stance,
                    combat::Stance::Aggressive | combat::Stance::Defensive
                ) || self.is_in_range(me, t))
            {
                self.units[u].combat.target = Some(t);
            }
        }
        let Some(target) = self.units[u].combat.target else {
            self.kill_current_order(u);
            return false;
        };
        let mut capturable = 0;
        if !self.valid_target_out(me, target, Some(&mut capturable)) {
            // **The capture approach** (item 3528, `docs/ORDERS.md` §7.2):
            // `Unit::fight@005fd4d0`, `005fd8d1`–`005fda9b`, the first arm
            // under the refused target, ahead of `find_new_target`.
            if capturable != 0 && self.capture_approach(u, target, state.mandatory) {
                return false;
            }
            if let Some(done) = self.mandatory_substitute(u, target, state.mandatory, frame) {
                return done;
            }
            // **The search budget** (item 3598, `docs/AI.md` §318.3):
            // `LAB_005fdb9e`'s first arm. While the unit's `waiting`
            // (`+0xad`) is below 5 and its leader has counted more than ten
            // invalid targets this frame, `fight` sets the frozen mark, adds
            // 2 to `waiting` and returns: the order stands, nothing is
            // searched and nothing is counted. run1241b's Phalanx `1/116`
            // keeps its `ATTACKORDER` on the dead Citizen `0/12` one frame
            // past ours (`waiting` 2, `unit_masks2` 16 on block 6718).
            let who = usize::from(self.units[u].owner);
            if self.units[u].waiting < 5 && self.retargets[who] > 10 {
                self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
                self.units[u].waiting += 2;
                return false;
            }
            // `find_new_target`: the order dropped, then
            // `find_melee_target(−1, NULL, 0, 1, 0)`, **whose head is the
            // squad's** (item 1089, `docs/COMBAT.md` §79). A follower takes
            // its captain's `ATTACK` target with no search, and finds
            // nothing when its captain is not attacking; a captain, or a
            // follower whose captain's target it may not take, searches.
            //
            // **And its defensive-post arm** (`bVar3`, item 3784,
            // `docs/GOLDEN.md` §149.10): `LAB_005fdb9e` calls
            // `find_new_target` whole, so a `DEFENSIVE` unit whose dead
            // target ends its attack keeps its post in what it finds or
            // walks home to it ([`Self::keep_def_post`]). run1413's Dragoon
            // `2/20` stands back on its post (14808, 16968) on block 1001.
            let post = if state.stance == combat::Stance::Defensive {
                self.def_post(u)
            } else {
                None
            };
            self.kill_current_order(u);
            let found = self.find_melee_target_added(u);
            if let Some(p) = post {
                self.keep_def_post(u, found.is_some(), p);
            }
            // **The frozen frame's mark** (item 502, `docs/COMBAT.md`
            // §43.2). `Unit::fight@005fd4d0`'s invalid-target branch ends
            // at `LAB_005fdb9e`, and its tail is
            //
            //     find_new_target(this, NULL, 0)
            //     leaders[who].searches += 1
            //     if (order_type() != ATTACK)                 return 0
            //     if (recharging != 0 && !reentry_latch)       return 0
            //     unit_masks2 |= 0x10
            //
            // — so a unit whose target went invalid does **not** strike
            // this frame, and it carries [`combat::umask2::NOT_FIRING`]
            // out of the order step, which stops its figures' animation
            // clocks in phase 7 (`docs/ANIM.md` §5). `recharging` is
            // zero here by the reload gate above, so the only condition
            // left is that the search put an attack order back in front:
            // a unit that finds nothing has no order to be "still
            // ordered" under and takes no mark.
            //
            // run112's chapter-two word is this arm: `0/6` retargets to
            // `1/6`, freezes, and spends **no** attack-end wrap on 695
            // where this crate spent one — 21 draws against 20.
            //
            // The count is [`Sim::retargets`], `LeaderData +0x9f4`
            // (item 668): `Leader::process@006b88b0` zeroes it at the
            // head of each leader's frame, and `resolve_unit_collision`'s
            // enemy ladder reads `< 10` (`docs/COLLISION.md` §14).
            //
            // ~~SEAM: the **search budget** it feeds here~~ — built above
            // (item 3598). run112 says it did not fire there: `0/6`'s
            // `waiting` is 0 on block 696, where the throttle would have
            // left 2.
            self.retargets[who] += 1;
            if matches!(
                self.current_order(u).map(|o| &o.body),
                Some(Body::Attack(_))
            ) {
                self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
            }
            return false;
        }
        // **A guard's attack is leashed to its post** (`docs/COMBAT.md`
        // §63). `Unit::fight@005fd4d0`, `005fdc91`–`005fdd4f`, straight
        // after `valid_target` passes and ahead of both captain arms below:
        // when the unit's **activity** is a `GUARD` (`get_activity`, type
        // `0xc`), it asks `check_target(o, who, 1, NULL, 1, use_poor, 0)`,
        // whose guarding arm refuses a target — or a guard — further than
        // `unit_guard_respond_range × 2 × 0x60` from the post. Refused, the
        // attack is killed and `find_melee_target(−1, NULL, 0, 1, 0)` runs
        // the guard's own search, which adds what it finds at QUEUE_FIRST;
        // a unit left under an `ATTACK` and not recharging carries the
        // frozen mark. A captain with `unit_masks & 0x40000` first rolls
        // (`fight+0x824`) and drops the attack on an odd draw, and asks
        // with `use_poor`.
        //
        // Chapter eleven is the witness (run190): the chariot `0/6` shot
        // `1/6` from its post on 1011, `1/6` walked off on its army's
        // order, and on tick 1036, its first unrecharged `fight`, `1/6`
        // stood ≈1,780 from the post against a leash of 1,536. The guard
        // drops the attack with no draw, and its search, leashed too,
        // writes `near` −1.
        if let Some(g) = self.guard_activity(u) {
            let mut use_poor = false;
            let mut drop = false;
            if self.units[u].captain && self.ai_driven(self.units[u].owner) {
                use_poor = true;
                self.mark(crate::fight::SITE_FIGHT_GUARD_ROLL);
                drop = self.rng.roll() & 1 != 0;
            }
            if drop || !self.guard_check_target(u, &g, target, use_poor) {
                self.kill_current_order(u);
                // The squad head first (item 2032, `docs/GOLDEN.md` §74).
                self.find_melee_target_headed(u);
                // `005fdd3c`: no mark on a recharging unit (item 3568).
                if self.order_type(u) == index::ATTACK && self.units[u].combat.recharging == 0 {
                    self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
                }
                return false;
            }
        }
        // **`fight`'s captain arm, `LAB_005fddf7`: one search for both
        // kinds of target** (`docs/COMBAT.md` §62, §80). A captain whose
        // attack is not mandatory asks for a re-search:
        //
        // - on a **building** every frame, with no roll and no
        //   `poor_target` (vtable `+0x18`, `005fdeb1`-`005fdeb6` →
        //   `005fdf50`; run177's packet, the Fighter `0/6` on tick 632 of
        //   chapter six-b);
        // - on a **unit** after the one-in-five draw (§8.2 step 0,
        //   `005fde80`-`005fde99`): the draw is spent first and **then**
        //   the suppressions are read — a combat-role target, `roll % 5 ==
        //   0`, or the current order carrying the chase latch. So a
        //   latched order costs the draw and skips the search, which is
        //   what keeps the frame's count right.
        //
        // **Either way the search is `find_new_target(this, &who, 0)` at
        // `005fdee2`** (`fight@005fd4d0:413`): `repath`, the attack
        // **killed**, and `find_melee_target(-1, &who, 0, 1, 0)`, which
        // adds what it finds. The kill is what item 1099 measured: with
        // the attack gone the head is the attack-move again, so the
        // search's `flags` word is the attack-move's (`0x20010`, §64.1)
        // and a city scores half. This crate searched under the attack
        // and re-pointed it in place (`retarget_attack`), so run347's
        // `1/24` took the city `0/2000` beside the scout `0/0` on 5161
        // where the original, re-finding the scout, struck it.
        //
        // Nothing found, and the attack is gone (`LAB_005fe001`); another
        // target, and the search's own order stands with the frozen mark
        // (`005fdf85`-`005fdfea`, item 1040, §70: under an `ATTACK` with
        // `recharging` zero, `orl $0x10, 0x6c(%ebx)`, so the figures'
        // clocks stand still in phase 7); the same one, and `fight` goes
        // on with the order the search added, **fresh** — which is why the
        // original's head attack reads `in_range 0, new_ord 1` for one
        // block after every retarget (parked 1073, §71.6).
        //
        // **A suppressed re-search still runs when the chase is futile**
        // (`005fded5`, `fight@005fd4d0:466`–`469`; item 2099, `docs/GOLDEN.md`
        // §76): on a unit target whose search the role, the roll or the
        // latch turned off, `Object::poor_target(this, target)` is asked,
        // and a yes jumps to the same `find_new_target`. Its own
        // `check_target` asks `poor_target` of every candidate, so the
        // target that sent it there is never re-found: run814's captain
        // `0/10`, latched on its chase of who=1's Citizen `1/6` on 778,
        // drops the attack and stands on its `GUARD` (`near` −1).
        //
        // SEAM: the retarget arm's early return when the search names
        // `fight`'s own entry arguments (`local_20`/`local_24`) after the
        // target was changed above; nothing here changes it in between.
        // SEAM: the cavalry archer's `find_melee_target` in place of
        // `find_new_target`.
        // The arm is `recharging == 0`'s alone (`fight@005fd4d0:368`): a
        // recharging melee-and-ranged unit past the gate skips it.
        if !state.mandatory
            && state.recharging == 0
            && state.captain == i32::from(self.units[u].index)
        {
            let research = match target {
                Obj::Building(_) => true,
                Obj::Unit(t) => {
                    self.mark(crate::fight::SITE_FIGHT_RESEARCH);
                    let roll = self.rng.roll();
                    // `fight@005fd4d0:394-400`: a unit with
                    // `unit_masks2 & 4` (a Bark, item 2165, parked 2100)
                    // re-finds a target whose role is combat and whose
                    // type is not `n` ("destroyed when it attacks",
                    // `unit_flags & 0x2000`); any other re-finds one whose
                    // role is not combat. run835's `0/7` re-finds the Bark
                    // `1/8` on 634 and keeps it here; the chase's retarget
                    // ([`Sim::retarget_admits`]) hands it the fishing boat.
                    let off = self.captain_research_admits(u, t);
                    (off && roll % 5 != 0 && flags & flag::FIGHT_REENTRY == 0)
                        || self.poor_target(me, target)
                }
            };
            if research {
                match self.find_new_target(u, false) {
                    None => return false,
                    Some(f) if f != target => {
                        if self.units[u].combat.recharging == 0
                            && matches!(
                                self.current_order(u).map(|o| &o.body),
                                Some(Body::Attack(_))
                            )
                        {
                            self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
                        }
                        return false;
                    }
                    Some(_) => {
                        let Some(Order {
                            body: Body::Attack(fresh),
                            ..
                        }) = self.current_order(u).copied()
                        else {
                            return false;
                        };
                        a = fresh;
                    }
                }
            }
        }
        // **The slot, never the health** (`fight@005fd4d0:482`, the
        // eight-argument `is_in_range@006486b0` with no margin; item 2864,
        // `docs/AI.md` §238): a city at its ceiling still stands in range.
        // run883b's packet answers 1 for the peace pair's MLRS `1/95` at
        // 4500 of 4806 against `0/2000` at `damage` 1560 of 1560, and it
        // unpacks; with the health term ours chased it instead.
        let in_range = self.is_in_range_struck(me, target);
        a.in_range = in_range;
        if in_range {
            a.ever_in_range = true;
            if self.packed_unpacks(u, target, a) {
                return false;
            }
            // **Siege fire at a unit is ground fire** (`docs/COMBAT.md`
            // §57.2): the arm returns before the strike's tail, so the
            // attack keeps `new_ord 1` under the order it pushes, as
            // run146's `0/6` does from 781 to 863.
            if self.siege_ground_arm(u, target, a, frame) {
                return false;
            }
            a.new_ord = false;
            self.store_attack(u, a);
            return self.fight_pub(u, target, frame);
        }
        self.store_attack(u, a);
        // **An unpacked packer that has been in range dies out of it**
        // (`Unit::fight@005fd4d0:496`, `LAB_005fe0e5`): `unit_flags2 & 4`,
        // not packed, and the order's `ever_in_range` → kill, before the
        // stance and the chase. It is how run146's catapult loses its
        // attack on 864, the frame its ground order ends: the hoplites
        // have walked inside the three-tile minimum (§57.3).
        let packs = self.units[u]
            .ty
            .is_some_and(|t| self.tables.unit_types[t].combat.packs);
        if packs && !self.units[u].combat.packed && a.ever_in_range {
            self.kill_current_order(u);
            return false;
        }
        // **A packer re-searches before it chases** (`fight:1051`,
        // `docs/COMBAT.md` §60.3): `find_new_target(this, &who, 0)`
        // kills the attack and runs the idle search, which adds the
        // order it finds, and the tail goes on only with a target in
        // hand. An unpacked packer's search refuses what it cannot
        // reach (`find_nearby_target`'s `local_24`), so a catapult whose
        // attacker stands inside its minimum drops the attack and stays
        // put: run146's `0/6` on 868, 870 and 871.
        //
        // **A mandatory attack does neither** (item 2864, `docs/AI.md`
        // §238): `do_attack@005f1b80:254` calls `fight(o, who,
        // attack->mandatory, 0, 0)`, and `fight`'s third argument (`esi`,
        // reloaded from `[ebp+0x10]` at `5fe0fc`) skips the re-search
        // (`5fe108`) and the STAND_GROUND and entrenched gates (`5fe135`)
        // below, straight to `find_attack_pos` (`5fe17f`). run883b's
        // packet runs the peace pair's MLRS `1/124` on 6821: no
        // `find_new_target`, no kill, the ring from `fight+0xcb4`.
        let target = if packs && !state.mandatory {
            self.kill_current_order(u);
            if let Some(t) = self.take_siege_army_target(u) {
                t
            } else {
                let Some(t) = self.find_melee_target(u, -1) else {
                    return false;
                };
                self.nearby_normalize(u);
                let mandatory = self.nearby_siege_city(u, t);
                let pos = if state.stance == combat::Stance::Defensive {
                    QueuePos::First
                } else {
                    QueuePos::New
                };
                self.add_attack_order(u, t, pos, mandatory, false);
                t
            }
        } else {
            target
        };
        // **A STAND_GROUND unit chases only on duty** (`fight@005fd4d0`,
        // `005fe139`–`005fe151`): `get_combat_stance() == 2 &&
        // !Unit::on_duty()` skips the chase, so an attack-move's or a
        // guard's still walks. Every packer and worker reads 2
        // (`get_combat_stance@00610a30`, item 2265): run589's cannon
        // `1/132`, packed on its attack-move, walks at `0/2000` on 17945.
        // And a unit in a trench the marines did not dig (`005fd8f0`'s
        // `unit_masks & 0x2000000 && !(unit_masks2 & 0x20000)`, item 3661).
        if !state.mandatory
            && ((state.stance == combat::Stance::StandGround && !self.on_duty(u))
                || self.hard_trench(u))
        {
            return false;
        }
        // **The chase** (`docs/COMBAT.md` §17): the unit asks where to
        // stand — `Unit::fight@005fd4d0+0xcb4`'s arm, through the
        // seven-argument thunk that fills the last two arguments with the
        // caller's own position — and the move goes in front, to run from
        // the next frame. A ring walk that finds nothing, and every
        // target the ring does not cover, falls back to the target's own
        // position, which is what this crate did for all of them until
        // item 328.
        let here = self.units[u].pos;
        let dest = match self.find_attack_pos(u, target, here, crate::fight::SITE_ATTACK_POS_FIGHT)
        {
            Some(_) if self.fight_post_leash(u, target, state.mandatory) => return false,
            Some(p) => p,
            None if self.attack_pos_refused(u, target) => {
                self.chase_refused(u, target);
                return false;
            }
            None => self.pos_of(target),
        };
        self.add_move_order(u, dest, MoveKind::MoveTo, QueuePos::First, false);
        self.chase_reentry(u, here, dest, frame);
        false
    }

    /// **`fight`'s own defensive leash** (`Unit::fight@005fd4d0`,
    /// `005fe18c`–`005fe264` and its tail to `005fe502`; `fight:1104`–
    /// `1172`; item 3779, `docs/GOLDEN.md` §149.7). When `find_attack_pos`
    /// answers, a `DEFENSIVE` unit whose attack is not mandatory (`esi`,
    /// `fight`'s third argument) and carries a post (`+0x1d`, both of
    /// `+0x14/+0x18` non-negative) does not chase from `reach × 0xc0` or
    /// more from the post — `reach` the larger of the type's vslot `+0x130`
    /// and `unit_defensive_respond_range`, as `check_target_path`'s leash
    /// reads them. It runs `find_new_target(NULL, 0)` instead, and then:
    ///
    /// - no `ATTACK` in front, or the unit recharging: nothing more;
    /// - the same target again: the attack is killed and, with nothing
    ///   left, the unit walks to the post — `add_move_order(post, 1, 0,
    ///   QUEUE_NEW)`, `flags |= 8`;
    /// - another target: the frozen mark (`LAB_005fe502`).
    ///
    /// `true` when the leash took the frame. run1451's packet runs the
    /// Inti Macemen `2/31` on 945: the review's `find_new_target` re-adds
    /// its captain's `1/22` with the post kept, then `fight+0xd94`'s
    /// second call re-adds it again and the walk home goes in (block 946,
    /// `MOVEORDER` to (21624, 18360), `flags 8`).
    pub(super) fn fight_post_leash(&mut self, u: usize, target: Obj, mandatory: bool) -> bool {
        if mandatory || self.units[u].combat.stance != combat::Stance::Defensive {
            return false;
        }
        let Some(Order {
            body:
                Body::Attack(AttackOrder {
                    defensive: true,
                    def: Some(post),
                    ..
                }),
            ..
        }) = self.current_order(u).copied()
        else {
            return false;
        };
        if post.x < 0 || post.y < 0 {
            return false;
        }
        let r = self.max_range_of(Obj::Unit(u));
        let mut reach = self
            .tables
            .tuning
            .unit_defensive_respond_range
            .wrapping_mul(0xc0);
        if reach < r.wrapping_mul(0xc0) {
            reach = r.wrapping_mul(0xc0);
        }
        let p = self.units[u].pos;
        if vector_dist((p.x - post.x).wrapping_abs(), (p.y - post.y).wrapping_abs()) < reach {
            return false;
        }
        self.find_new_target(u, false);
        if self.order_type(u) != index::ATTACK || self.units[u].combat.recharging != 0 {
            return true;
        }
        if self.units[u].combat.target == Some(target) {
            let walk = match self.current_order(u).map(|o| o.body) {
                Some(Body::Attack(AttackOrder {
                    defensive: true,
                    def: Some(p),
                    ..
                })) if p.x >= 0 && p.y >= 0 => Some(p),
                _ => None,
            };
            self.kill_current_order(u);
            if let Some(p) = walk
                && self.units[u].orders.is_empty()
            {
                self.add_move_order(u, p, MoveKind::MoveTo, QueuePos::New, false);
                if let Some(o) = self.units[u].orders.front_mut() {
                    o.flags |= flag::POST;
                }
            }
            return true;
        }
        self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
        true
    }

    /// **The chase that `find_attack_pos` refused** — `Unit::fight@
    /// 005fd4d0`, `5fe186`'s `je` to `5fe3f8`–`5fe502` (`fight:1056`–
    /// `1100`, `docs/AI.md` §138). A guard's attack on a building gets no
    /// spot ([`Sim::attack_pos_refused`]), and the original then:
    ///
    /// 1. runs `find_new_target(this, NULL, 0)` — the attack killed and
    ///    the guard's own search, which adds what it finds;
    /// 2. if the head is an `ATTACK` on the **same** target again, kills
    ///    it, and walks to the attack's defensive post (`+0x1d`, the
    ///    point at `+0x14/+0x18`) when it had one and nothing else is
    ///    left, the move flagged `POST` (`flags |= 8`, `5fe4b8`–);
    /// 3. if the head is an `ATTACK` still and the unit is not
    ///    recharging (`+0xae`), sets the frozen mark (`5fe502`).
    ///
    /// So a guard's search that only finds the building it was chasing
    /// leaves it on its `GUARD`: run662's `1/115` on 17318, a lone
    /// `GUARD` on block 17319 with no draw (run665's packet runs this
    /// arm: `find_attack_pos` 0, then `find_new_target` twice).
    ///
    /// SEAM: the post walk of step 2 — `add_move_order(x, y, 1, 0,
    /// QUEUE_NEW, …)` — takes this crate's `action` for the call's `1`. No
    /// dump on disk holds a guard with an attack that carries a post (scan:
    /// `grep -alE '^ *defensive 1' gamelog*.txt`, 5 of the 338 dumps, runs 16,
    /// 17, 320, 324 and 405; then per `UNITDATA` block a `defensive 1` beside
    /// a `BEGIN GUARDORDER`: none in any of the five).
    pub(super) fn chase_refused(&mut self, u: usize, target: Obj) {
        self.find_new_target(u, false);
        if let Some(Order {
            body: Body::Attack(a),
            ..
        }) = self.current_order(u).copied()
            && self.units[u].combat.target == Some(target)
        {
            let post = if a.defensive { a.def } else { None };
            self.kill_current_order(u);
            if let Some(p) = post
                && self.units[u].orders.is_empty()
            {
                self.add_move_order(u, p, MoveKind::MoveTo, QueuePos::New, true);
                if let Some(o) = self.units[u].orders.front_mut() {
                    o.flags |= flag::POST;
                }
            }
        }
        if self.units[u].combat.recharging == 0
            && matches!(
                self.current_order(u).map(|o| &o.body),
                Some(Body::Attack(_))
            )
        {
            self.units[u].unit_masks2 |= combat::umask2::NOT_FIRING;
        }
    }

    /// **A mandatory attack on a dead figure takes its squad's captain, in
    /// the same order** (item 4002, `docs/GOLDEN.md` §191). `Unit::fight@
    /// 005fd4d0:300`–`:335`, the invalid-target branch past the capture
    /// approach, with `param_5 == 0`:
    ///
    /// ```text
    /// action = get_action(); if !action || action.type != ATTACK: return 0
    /// if !mandatory || ox < 0:                       goto LAB_005fdb9e
    /// c = objects[whom][ox]->get_captain()            ; vslot +0xe4
    /// if c != ox && valid_target(c, whom):
    ///     order.target = (c, whom, uid(c))            ; mandatory, ever_in_range,
    ///     flags ^= 0x10; if now set: work()           ;   new_ord untouched
    ///     return 0
    /// if next_action_type() in (NONE, ATTACK_TO, GROUP_ATTACK_TO): goto LAB_005fdb9e
    /// LAB_005fe0e5: kill_current_order(0); return 0
    /// ```
    ///
    /// `UnitData::get_captain@00610ab0` walks `o_up` and asks nothing of
    /// life, and `Unit::close`'s relink hands a dead middle figure the tail
    /// as its `o_up` ([`Self::relink_squad`]), so the walk from the dead
    /// figure reaches the squad's live head. A building answers its own `o`
    /// (`ObjectData::get_captain@00472400`) and never substitutes.
    ///
    /// run1545's Hoplite `1/14` (chapter 129) is the witness: its mandatory
    /// attack on `0/9`, who died on 809 with `o_up` 8 → 10 after the relink,
    /// names `0/8` on block 810 with `mandatory 1`, `ever_in_range 1` and
    /// `new_ord 0`, a move to (5352, 31704) over it — the re-entered `work`'s
    /// chase. Ours searched and took `0/10` fresh.
    ///
    /// Returns `None` where the original goes on to `LAB_005fdb9e`, and
    /// `Some(false)`, `fight`'s 0, where it returns.
    pub(super) fn mandatory_substitute(
        &mut self,
        u: usize,
        target: Obj,
        mandatory: bool,
        frame: i64,
    ) -> Option<bool> {
        let k = self.action_of(u);
        if !k.is_some_and(|k| matches!(self.units[u].orders[k].body, Body::Attack(_))) {
            return Some(false);
        }
        if !mandatory {
            return None;
        }
        if let Obj::Unit(t) = target {
            let mut c = t;
            for _ in 0..self.units.len() {
                match self.units[c].o_up {
                    Some(up) if up != c => c = up,
                    _ => break,
                }
            }
            let me = Obj::Unit(u);
            if c != t && self.valid_target(me, Obj::Unit(c)) {
                self.units[u].combat.target = Some(Obj::Unit(c));
                let front = self.units[u].orders.front_mut()?;
                if front.flags & flag::FIGHT_REENTRY == 0 {
                    front.flags |= flag::FIGHT_REENTRY;
                    self.work(u, frame);
                } else {
                    front.flags &= !flag::FIGHT_REENTRY;
                }
                return Some(false);
            }
        }
        let next = self.units[u]
            .orders
            .iter()
            .skip(1)
            .find(|o| !o.is_transit() && o.index() != index::CHANGE_FORM)
            .map_or(index::NONE, Order::index);
        if matches!(
            next,
            index::NONE | index::ATTACK_TO | index::GROUP_ATTACK_TO
        ) {
            return None;
        }
        self.kill_current_order(u);
        Some(false)
    }

    /// **The chase runs in the frame it is ordered** — `Unit::fight@
    /// 005fd4d0`'s tail, `005fe395`-`005fe3e7`, and `docs/ORDERS.md`
    /// §7.10.
    ///
    /// `add_move_order` has just put the chase in front of the attack.
    /// The original does not then leave the unit standing: it re-enters
    /// **`Unit::work`** through vtable `+0x188`, so the move is dispatched,
    /// planned and — when the plan is short enough — walked on this same
    /// tick. `docs/ORDERS.md` §7.10 has the value diff: run19's block 8187
    /// carries all six of the probe's units with a **full path stack**
    /// (43, 43, 43, 94, 47, 46 entries) at the end of sim-frame 8186, the
    /// frame the chase was ordered, and this crate planned nothing until
    /// 8187.
    ///
    /// Two gates, both from the listing:
    ///
    /// - `005fe395`-`005fe3b1`: two `je`s out, one per coordinate. The
    ///   re-entry happens only when the chosen spot differs from the
    ///   unit's own position in **both** x and y — matching either one
    ///   returns.
    /// - `005fe3b7`-`005fe3e7`: the action order's [`flag::FIGHT_REENTRY`].
    ///   Clear, it is set and `work` is called; set, it is cleared and
    ///   nothing recurses. That is the whole recursion guard, and it
    ///   leaves the latch **standing** on the ordinary path, because the
    ///   nested call dispatches the new move rather than `fight`.
    pub(super) fn chase_reentry(&mut self, u: usize, here: Pos, dest: Pos, frame: i64) {
        if dest.x == here.x || dest.y == here.y {
            return;
        }
        let Some(a) = self.update_action(u) else {
            return;
        };
        if self.units[u].orders[a].flags & flag::FIGHT_REENTRY == 0 {
            self.units[u].orders[a].flags |= flag::FIGHT_REENTRY;
            self.work(u, frame);
        } else {
            self.units[u].orders[a].flags &= !flag::FIGHT_REENTRY;
        }
    }

    /// **A packed packer in range unpacks** (`Unit::fight@005fd4d0:512`–
    /// `543`, item 1182, `docs/GOLDEN.md` §50): in the attack's in-range
    /// branch, for a type that packs (`unit_flags2 & 4`) and is not the
    /// Dutch merchant, still packed (`unit_masks & 0x80000`),
    /// `add_cast_order(−1, −1, −1, −1, UNPACK, QUEUE_FIRST, 0)` and the
    /// frame ends — no strike, and the attack keeps `new_ord 1` with
    /// `in_range` and `ever_in_range` written. run436's human Catapult
    /// `0/10`, on its `@attack` at a Barracks in range: the `CASTORDER`
    /// of spell 652 beside its attack on block 624.
    ///
    /// **The computer's siege arm**, ahead of the cast (`fight:517`–`541`,
    /// item 1937): a computer's (`unit_masks & 0x40000`) siege engine
    /// against a **building** that is armed (`attack`, `+0x1e8`), whose
    /// range plus one is under its own and which reaches it, asks
    /// `find_attack_pos` for a better spot and, when the answer differs
    /// from where it stands, moves there at `QUEUE_NEW` instead of
    /// unpacking. The attack's `in_range` and `ever_in_range` are written
    /// either way. Run790's Rocket Artillery `1/96` against Napata's city
    /// centre on 3805.
    pub(super) fn packed_unpacks(&mut self, u: usize, target: Obj, a: AttackOrder) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if !self.tables.unit_types[t].combat.packs
            || !self.units[u].combat.packed
            || self
                .unit_tree(u)
                .is_some_and(|ti| self.tables.tech_tree.is(ti, MERCHANTDUTCH, true))
        {
            return false;
        }
        self.store_attack(u, a);
        if self.unit_ai_bit(self.units[u].owner)
            && self.profile(Obj::Unit(u)).siege
            && let Obj::Building(_) = target
            && self.profile(target).attack != 0
            && self.max_range_of(target) + 1 < self.max_range_of(Obj::Unit(u))
            && self.is_in_range(target, Obj::Unit(u))
        {
            let here = self.units[u].pos;
            let spot =
                match self.find_attack_pos(u, target, here, crate::fight::SITE_ATTACK_POS_SIEGE) {
                    Some(p) => Some(p),
                    None if self.attack_pos_refused(u, target) => None,
                    None => Some(self.pos_of(target)),
                };
            if let Some(spot) = spot
                && (spot.x != here.x || spot.y != here.y)
            {
                self.add_move_order(u, spot, MoveKind::MoveTo, QueuePos::New, false);
                return true;
            }
        }
        self.add_cast_order(u, spell::UNPACK);
        true
    }

    /// **`Unit::fight@005fd4d0`'s siege arm** (`fight:496`–`572`,
    /// `docs/COMBAT.md` §57.2), in the attack's in-range branch and
    /// ahead of the strike. For an attacker whose type packs
    /// (`unit_flags2 & 4`) and is not the Dutch merchant:
    ///
    /// - packed, the original unpacks here or moves to a better spot:
    ///   [`Sim::packed_unpacks`], ahead of this (item 1182).
    /// - unpacked, a **unit** target (the target's vslot `+0x18`,
    ///   `is_unit`) and a **siege** type (vslot `+0x10c`, `is_siege`):
    ///   `set_attacking`, then an `ATTACK_GROUND` order at the head
    ///   holding the target's own position, `accuracy = (domain == sea)`
    ///   and `attack_unit = 2`, with the fired and action bits clear;
    ///   `clear_partial_path`, the head rotated onto it, `update_action`,
    ///   and **`Unit::work`** (vslot `+0x188`), so the new order fires on
    ///   this same frame.
    ///
    /// The attack's `in_range` and `ever_in_range` are written before it
    /// (`fight:491`–`493`) and its `new_ord` is left standing: the strike
    /// tail that clears it (`fight:847`) is never reached.
    pub(super) fn siege_ground_arm(
        &mut self,
        u: usize,
        target: Obj,
        a: AttackOrder,
        frame: i64,
    ) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        if !self.tables.unit_types[t].combat.packs
            || self.units[u].combat.packed
            || self
                .unit_tree(u)
                .is_some_and(|ti| self.tables.tech_tree.is(ti, MERCHANTDUTCH, true))
        {
            return false;
        }
        let Obj::Unit(v) = target else {
            return false;
        };
        if !self.profile(Obj::Unit(u)).siege {
            return false;
        }
        self.set_attacking(u, self.owner_of(target));
        self.store_attack(u, a);
        let at = self.units[v].pos;
        let sea = matches!(self.profile(target).domain, crate::attrition::Domain::Sea);
        let order = Order {
            flags: 0,
            body: Body::AttackGround(AttackGroundOrder {
                at,
                sea,
                attack_unit: 2,
            }),
        };
        self.enqueue(u, order, QueuePos::First);
        self.work(u, frame);
        true
    }

    /// **`Unit::do_attack_ground@005f1410`** — one frame of an
    /// `ATTACK_GROUND` order (`docs/COMBAT.md` §57.3, `docs/ORDERS.md`
    /// §26), in the original's order:
    ///
    /// 1. `can_attack_ground`, else kill. A player's order (`attack_unit
    ///    0`) is also killed on a point at peace with the owner, and walks
    ///    into range; SEAM: no command here issues one, so both arms are
    ///    the kill.
    /// 2. The facing, `find_angle` to the point, a quarter turn off for a
    ///    broadside type (`unit_flags & 0x40`).
    /// 3. **Reloading**: a fired order holds the unit still (`flags &
    ///    0x80`, the byte read signed); otherwise a figure off slots 0–3
    ///    rolls the idle, as `fight`'s own reload gate does.
    /// 4. **`attack_unit`**: 1 is the ready frame after the shot, and it
    ///    kills the order and re-enters `Unit::work` (vslot `+0x188`), so
    ///    the attack beneath runs on this frame; 2 becomes 1 and fires.
    /// 5. `set_attack(−1, −1)`: the aimed figures forget their target. A
    ///    packed packer unpacks instead of firing, **before** the mask is
    ///    written (the sweep's reading: the mask is after the pack test).
    /// 6. `unit_masks |= 0x11000` (SEAM: this crate keeps neither bit
    ///    apart), the turn, `flags |= 0x80`, the swing — **its own two
    ///    arms, not `fight`'s four**: a rocking type (`unit_flags &
    ///    0x2000000`) asks `find_angle` again and takes `ATTACK2` when
    ///    that is not behind the facing, every other type `ATTACK1` with
    ///    the idle re-roll; no radar-jam arm — the shot when the type has a
    ///    fire graphic, and **`recharging = recharge() + 1`**: run146's
    ///    catapult reads 83 on the push where its direct strike at a
    ///    building reads 82 (run44 `0/15`, 408).
    ///
    /// The body is [`do_attack_ground_steps`] over [`AttackGroundView`];
    /// `crates/sim/tests/sweep/unit_do_attack_ground.rs` runs it against the
    /// original under the emulator (item 3289).
    pub(crate) fn do_attack_ground(&mut self, u: usize, frame: i64) {
        let Some(Order {
            body: Body::AttackGround(g),
            ..
        }) = self.current_order(u).copied()
        else {
            return;
        };
        do_attack_ground_steps(&mut SimAttackGround {
            s: self,
            u,
            frame,
            g,
        });
    }

    /// **`UnitTypeData::can_attack_ground@0061db20`**, from the listing
    /// (`61db20`–`61dc13`): the rush-rules age gate (SEAM: this crate
    /// plays without rush rules), a `max_range` (`+0x1fc`), neither of
    /// `unit_flags & 0x102000`, and then any of `obj_masks` EXPLOSIVE,
    /// SIEGE, NAVAL or BOMBARD, or the machine-gun lineage.
    pub(crate) fn can_attack_ground(&self, u: usize) -> bool {
        let Some(t) = self.units[u].ty else {
            return false;
        };
        let ty = &self.tables.unit_types[t];
        let p = ty.combat;
        ground_attacker(
            p.max_range,
            ty.cols.flag(0x10_2000),
            |m| p.has(m),
            || self.unit_line_is(u, MACHINEGUN),
        )
    }

    pub(super) fn store_attack(&mut self, u: usize, a: AttackOrder) {
        if let Some(front) = self.units[u].orders.front_mut()
            && let Body::Attack(x) = &mut front.body
        {
            *x = a;
        }
    }

    /// `Unit::find_new_target(this, NULL, stand)@005ff6a0`: from
    /// `resolve_unit_collision`'s enemy ladder, `stand` 1 (`docs/COLLISION.md`
    /// §14, the listing at `005f9ffb`–`005fa001` pushes `1, 0`); from
    /// `fight`'s captain re-search, `(&who, 0)`; and from the refused
    /// chase, `(NULL, 0)` ([`Sim::chase_refused`]). The out-parameter only
    /// reports the owner found, so all three are this one call.
    ///
    /// 1. `repath`: the leading transit legs go.
    /// 2. The current order goes too: `kill_current_order`, or for a
    ///    group order `Group::kill_group_order`.
    /// 3. With `stand`, the unit's stance (`+0xb1`) reads **2,
    ///    `STAND_GROUND`**, across `find_melee_target(-1, NULL, 0, 1, 0)`
    ///    and is put back after. That is the whole of what `stand` does,
    ///    and it does two things inside the search: `find_nearby_target`'s
    ///    `local_24` makes every candidate pass `is_in_range` from where
    ///    the unit stands, and the order it adds is `QUEUE_NEW`, because
    ///    the stance it reads is not `DEFENSIVE` (`00649b6d`).
    ///
    /// So a captain bumped by an enemy that is not its target drops the
    /// walk and the old attack where it stands, and takes whatever it can
    /// strike from there. run171's `1/6` on 658: `0/7`.
    ///
    /// **The defensive-post arm** (`bVar3`, [`Self::keep_def_post`], item
    /// 3779): a `DEFENSIVE` unit whose `find_def_pos` answers keeps its
    /// post in the attack it finds, or walks back to it.
    ///
    /// SEAM: `has_stance_type`'s refusal (vslot `+0x104`, a type with no
    /// stance type never reads `DEFENSIVE`) is this crate's stance alone;
    /// and the group order's `kill_group_order`, which this crate has no
    /// group attack order for. `1/6` reaches neither, `stance 0` and not
    /// grouped in its order.
    /// **`fight`'s re-search predicate, which the chase's retarget copies**
    /// (`do_move@005f7b30:262`–`271`, `uVar17`; `fight@005fd4d0:394`–`400`):
    /// a unit without `unit_masks2 & 4` takes a candidate whose role is
    /// combat; a Bark (item 2165's bit) takes one that is **not** a combat
    /// role without `n` (`unit_flags & 0x2000`, destroyed when it attacks).
    /// run839's packet (item 2205, `docs/GOLDEN.md` §80): on tick 634 the
    /// Bark `0/7` re-finds the Bark `1/8`, and `do_move+0x6e6` hands it
    /// `change_target(1/8 → 1/7)`, the fishing boat in range.
    pub(crate) fn retarget_admits(&self, u: usize, c: Obj) -> bool {
        let combat = self.profile(c).combat_role;
        if self.units[u].unit_masks2 & combat::umask2::MASK2_BARK == 0 {
            return combat;
        }
        let n = match c {
            Obj::Unit(t) => self.units[t]
                .ty
                .is_some_and(|ty| self.tables.unit_types[ty].cols.unit_flags & 0x2000 != 0),
            Obj::Building(_) => false,
        };
        !combat || n
    }

    pub(crate) fn find_new_target(&mut self, u: usize, stand: bool) -> Option<Obj> {
        // `bVar3` (`005ff6a0`–`005ff70d`): a `DEFENSIVE` unit whose
        // `find_def_pos` answers, read before the order goes.
        let post = if self.units[u].combat.stance == combat::Stance::Defensive {
            self.def_post(u)
        } else {
            None
        };
        self.repath(u);
        self.kill_current_order(u);
        let stance = self.units[u].combat.stance;
        if stand {
            self.units[u].combat.stance = combat::Stance::StandGround;
        }
        let found = self.find_melee_target_headed(u);
        self.units[u].combat.stance = stance;
        if let Some(p) = post {
            self.keep_def_post(u, found.is_some(), p);
        }
        found
    }

    /// **`find_new_target`'s defensive-post arm** (`005ff7b0`–`005ff95e`,
    /// item 3779, `docs/GOLDEN.md` §149.7): a find whose order is an
    /// `ATTACK` takes the post into the attack (`+0x1d = 1`, `+0x14/+0x18`),
    /// so the leash still measures from where the unit stood; a unit left
    /// with no order walks back to the post — `MOVE_TO` at the post's
    /// `0x30` cell centre, `QUEUE_NEW`, the action bit clear, the angle
    /// `find_angle(post − pos)` (`5ff840`–`5ff85e`), flagged `POST`
    /// (`flags |= 8`). run1413's Inti Macemen `2/29` and `2/30` stand on
    /// their posts (21624, 18168) and (21720, 18264) on block 1001.
    pub(super) fn keep_def_post(&mut self, u: usize, found: bool, post: Pos) {
        let ty = self.order_type(u);
        if found && ty == index::ATTACK {
            if let Some(Order {
                body: Body::Attack(a),
                ..
            }) = self.units[u].orders.front_mut()
            {
                a.defensive = true;
                a.def = Some(post);
            }
            return;
        }
        if ty == index::NONE {
            self.add_move_order(u, post, MoveKind::MoveTo, QueuePos::New, false);
            if let Some(o) = self.units[u].orders.front_mut() {
                o.flags |= flag::POST;
            }
        }
    }

    /// `UnitData::find_def_pos@00608560`'s two answering arms: a DEFENSIVE
    /// attack's post (`+0x1d`, `+0x14/+0x18`), else a move carrying the
    /// post flag (`flags & 8`); `None` where the original returns 0.
    pub(super) fn def_post(&self, u: usize) -> Option<Pos> {
        for o in &self.units[u].orders {
            match o.body {
                Body::Attack(a) if a.defensive => {
                    if let Some(p) = a.def {
                        return Some(p);
                    }
                }
                Body::Move(m) if o.has(flag::POST) => return Some(m.dest),
                _ => {}
            }
        }
        None
    }

    /// **`find_melee_target(−1, NULL, 0, 1, 0)` whole, the squad head
    /// first** (`Unit::find_melee_target@005ff9c0`, lines 32–91; item 2032,
    /// `docs/GOLDEN.md` §74), as `find_new_target@005ff6a0` and `Unit::fight
    /// @005fd4d0`'s guard refusal call it. The head sits above both the
    /// search and the add flag, so it answers every caller: a follower whose
    /// captain's action is an `ATTACK` on a target it may take adds that
    /// target ([`Self::squad_head_add`]) and searches nothing; a follower
    /// whose captain is not attacking gets −1. A captain, or a follower whose
    /// captain's target it may not take, searches, and `find_nearby_target`
    /// adds the find ([`Self::nearby_add`]).
    ///
    /// run814 is the witness: `0/8`, guarding beside its captain's Stockade
    /// and out of reach of it, has its chase refused every sixteen frames
    /// from 689, and the original's `near` stays −1 there (the head writes
    /// none); on 770 the search would take who=1's Citizen `1/7` where the
    /// head re-adds the Stockade, and `chase_refused` kills it.
    pub(crate) fn find_melee_target_headed(&mut self, u: usize) -> Option<Obj> {
        match self.melee_squad_head(u) {
            SquadHead::Nothing => None,
            SquadHead::Captain(t, mandatory) => {
                self.squad_head_add(u, t, mandatory);
                Some(t)
            }
            SquadHead::Search => {
                // `find_nearby_target`'s siege shortcut (item 3654).
                if let Some(t) = self.take_siege_army_target(u) {
                    return Some(t);
                }
                let t = self.find_melee_target(u, -1)?;
                // A group's attack-move hands the target to
                // `Group::action_attack` ([`Self::nearby_add`], item 3634).
                self.nearby_add(u, t);
                Some(t)
            }
        }
    }

    /// The head's add (`005ff9c0:60`–`85`): `QUEUE_FIRST` under an
    /// `ATTACK_TO`, a `GROUP_ATTACK_TO` or a `GUARD` in front, `QUEUE_NEW`
    /// under anything else, with the captain's `mandatory` byte (the
    /// action's `+0x1c`).
    pub(super) fn squad_head_add(&mut self, u: usize, t: Obj, mandatory: bool) {
        let pos = if matches!(
            self.order_type(u),
            index::ATTACK_TO | index::GROUP_ATTACK_TO | index::GUARD
        ) {
            QueuePos::First
        } else {
            QueuePos::New
        };
        self.add_attack_order(u, t, pos, mandatory, false);
    }

    /// **`Object::find_nearby_target`'s add** (`00649b3a`–`00649bc0`, the
    /// decompile's lines 545–606; `docs/AI.md` §134), for a searcher whose
    /// third argument asks for the order:
    ///
    /// - **`local_2c`, a unit whose activity is a `GUARD`**
    ///   ([`Sim::guard_activity`], `:176`–`184`): an `ATTACK_TO` in front is
    ///   **killed** (`Unit::kill_current_order(0)`) and the attack is added
    ///   `QUEUE_FIRST`. A guard walks to its post on an attack-move leg
    ///   laid over the `GUARD`, and the attack takes the leg's place
    ///   rather than stacking on it: French East Indies' `1/69` on 17166
    ///   holds `[Attack(0/2000), Guard]` in the original, path length 0
    ///   (run657's block 17167), where this crate kept the leg beneath.
    /// - otherwise an attack-move in front (`ATTACK_TO`, `GROUP_ATTACK_TO`)
    ///   keeps its place under a `QUEUE_FIRST` attack;
    /// - otherwise `QUEUE_FIRST` only for a `DEFENSIVE` stance (vslot
    ///   `+0xf4 == 1`), and `QUEUE_NEW` for every other.
    ///
    /// SEAM: the naval refusal (`+0x218 == 2`) under an attack-move.
    ///
    /// **The arm is one for every caller** (item 3634): `find_nearby_target`'s
    /// add (`00648da0:545`–`606`) is the attack-move look's
    /// [`Self::attack_move_add`] — a `GROUP_ATTACK_TO` head on an active
    /// member of a group still walking its attack-move hands the find to
    /// `Group::action_attack` — whoever asked. `fight`'s captain re-search is
    /// the witness: `find_new_target` kills the attack, the head is the
    /// group's attack-move again, and run1296's packet has East Indies'
    /// Hussar `1/94` hand the Citizen `0/11` to group 80 on tick 4632
    /// (`find_nearby_target+0xf73` → `Group::action_attack`, then
    /// `set_up_insert`), where this crate took it alone.
    pub(crate) fn nearby_add(&mut self, u: usize, t: Obj) {
        self.attack_move_add(u, t);
    }

    /// **`Object::find_nearby_target`'s first act on a find it adds**
    /// (`00648da0`, the decompile's lines 495–499; item 2002,
    /// `docs/GOLDEN.md` §73): a unit whose group (`+0x80`) is set
    /// **normalizes that group** — `Group::normalize@00711540`'s prune,
    /// `find_role`, and the speed tail, `speed = new_speed =
    /// UnitData::speed(find_leader)` — before the `local_2c` kill and the
    /// order. Every caller of `find_melee_target` that passes the add flag
    /// (`do_attack_to`, `do_group_attack_to`, `do_guard`, `think_attack`,
    /// `find_new_target`, `fight`'s guard refusal) reaches it.
    ///
    /// It is what primes a squad pushed with a zero cap: chapter fifty-nine's
    /// Hoplites (pool slot 1, pushed on 622 at `(0, 0)`) read 25 from 625,
    /// the frame the captain's guard look finds the Stockade (run801).
    /// `find_role` is not modelled ([`Sim::groups_process`]).
    pub(crate) fn nearby_normalize(&mut self, u: usize) {
        if let Some(seat) = self.seat_of(u) {
            self.seat_normalize(seat);
        }
    }

    /// **`find_nearby_target`'s siege-city arm** (`00649912`–`00649b40`,
    /// read off the listing; item 3744, `docs/AI.md` §368.1): between the
    /// group's `normalize` and the queue choice, the add's `mandatory`
    /// (the reused `param_1` slot, `0x8(%ebp)`) is written 0 at `64992a`
    /// and **1** at `649a2d` for a unit that is an active member of its
    /// group (`+0x80 >= 0`, `GroupData::member(o, who, 1)@0070f8f0`), of
    /// a siege type (vslot `+0x10c`, `is_siege`), computer-driven
    /// (`unit_masks & 0x40000`, [`Sim::ai_driven`]'s stand-in), whose find
    /// is a building (vslot `+0x1c`) with `flags & 0x20` — a city centre.
    /// The same arm then points the unit's army (`Object::get_army@00649d70`)
    /// at the find: `+0x30/+0x34` the target, `+0x48/+0x4c` the **unit's**
    /// cell (`div_3_table[x >> 8]`), and `+0x50`
    /// `find_angle(target − unit)` — the muster cell and angle the next
    /// march's formation origin is stepped from.
    ///
    /// Witness: East Indies' MLRS `1/99` (TI 273) on 5887, whose
    /// attack-move look ranks the city `0/2000` at `attack_dist` 4684, past
    /// the packed shortcut's 4608 ([`Self::take_siege_army_target`]):
    /// run1423b holds its `ATTACKORDER mandatory=1` where this crate added
    /// 0.
    pub(crate) fn nearby_siege_city(&mut self, u: usize, t: Obj) -> bool {
        let Obj::Building(b) = t else {
            return false;
        };
        let who = self.units[u].owner;
        let siege = self.units[u]
            .ty
            .is_some_and(|ty| self.tables.unit_types[ty].cols.flag(uflags::FLAG_SIEGE));
        let city = self.buildings[b]
            .city
            .is_some_and(|c| self.cities[c].building == b);
        if !siege
            || !self.ai_driven(who)
            || !city
            || !self.group_of(u).is_some_and(|g| g.list.contains(&u))
        {
            return false;
        }
        if let Some(slot) = self.army_of(u) {
            let (at, to) = (self.units[u].pos, self.buildings[b].pos);
            let a = &mut self.armies[who as usize].list[slot];
            a.target = Some(t);
            a.muster = at.cell();
            a.muster_angle = find_angle(to.x - at.x, to.y - at.y);
        }
        true
    }

    /// [`Self::nearby_add`]'s order, once the group is normalized, with
    /// [`Self::nearby_siege_city`]'s `mandatory`.
    pub(super) fn nearby_add_order(&mut self, u: usize, t: Obj, mandatory: bool) {
        let head = self.order_type(u);
        let pos = if self.guard_activity(u).is_some() {
            if head == index::ATTACK_TO {
                self.kill_current_order(u);
            }
            QueuePos::First
        } else if matches!(head, index::ATTACK_TO | index::GROUP_ATTACK_TO)
            || self.units[u].combat.stance == combat::Stance::Defensive
        {
            QueuePos::First
        } else {
            QueuePos::New
        };
        self.add_attack_order(u, t, pos, mandatory, false);
    }

    /// **`do_move`'s flank clause on a chase** (item 1061, `docs/COMBAT.md`
    /// §71), `Unit::do_move@005f7b30`, `5f7fbe`–`5f7ff6`: a unit target whose
    /// heading, less the chaser's own, less a half turn, lies at or past
    /// `0x2aaaaaaa` and inside [`combat::flanking`]'s window, and which is
    /// moving, is running away from the chaser, and the chase is not ended
    /// for being in range. A building target never flees here.
    pub(crate) fn chase_target_flees(&self, u: usize, t: Obj) -> bool {
        let Obj::Unit(tu) = t else { return false };
        let e = (self.units[tu].movement.heading.0 as u32)
            .wrapping_sub(self.units[u].movement.heading.0 as u32)
            .wrapping_sub(0x8000_0000);
        e >= 0x2aaa_aaaa && combat::flanking(e) != 0 && self.is_moving(tu)
    }
}

/// What `Unit::check_target_path@005e22d0` asks of its unit, its action
/// order, the target and the world, in the order it asks (item 3304).
/// [`SimCheckTarget`] answers from this crate's state; the sweep answers
/// from the emulated original's windows.
///
/// The target is the order's `(ox, whom)` pair; the view holds it, so no
/// method takes one but the group arm's (`map_unit`, `action_attack`), which
/// name a *second* object, the order's own site record.
pub trait CheckTargetView {
    /// `ox >= 0`: the order names an object.
    fn target_set(&mut self) -> bool;
    /// The target's vslot `+0x48` (`is_seen`), asked of the unit's player.
    fn target_seen(&mut self) -> bool;
    /// The order's vslot `+0x10`: its action type.
    fn action_type(&mut self) -> i32;
    /// The cast arm's pair: the target's `flags & 1` **and**
    /// `SpellTypeData::is_valid_target(player, ox, whom, null)` of the
    /// spell the order names (vslot `+0x7c`'s `+0x20`).
    fn cast_target_valid(&mut self) -> bool;
    /// The target's vslot `+0x8`: on a unit `SubObjectData::is_active`
    /// (`flags & 1`), on a building 0.
    fn target_is_unit(&mut self) -> bool;
    /// The target unit's vslot `+0xd8`, `UnitData::is_moving`.
    fn target_moving(&mut self) -> bool;
    /// `Unit::repath`.
    fn repath(&mut self);
    /// `Unit::kill_current_order(0)`.
    fn kill_current_order(&mut self);
    /// The target's vslot `+0x1c`: 1 on a building, 0 on a unit.
    fn target_is_building(&mut self) -> bool;
    /// The target's position less the unit's: `(dx, dy)`.
    fn target_offset(&mut self) -> (i32, i32);
    /// The target unit's `+0x50`, its heading.
    fn target_heading(&mut self) -> i32;
    /// `ObjectData::is_in_range@00648d70(ox, whom, ·, 0)`.
    fn in_range_plain(&mut self) -> bool;
    /// The target's vslot `+0xbc`, `UnitData::is_on_map`.
    fn target_on_map(&mut self) -> bool;
    /// The unit's vslot `+0x170`, `adjacent_to(ox, whom)`.
    fn adjacent(&mut self) -> bool;
    /// `ObjectData::is_in_range@006486b0(ox, whom, orders_x, orders_y, ·, 0,
    /// 0)`, asked from where the unit is walking.
    fn in_range_from_orders(&mut self) -> bool;
    /// The order's vslot `+0x18`: an attack-shaped order.
    fn order_is_attack(&mut self) -> bool;
    /// The target's vslot `+0x18`: 1 on a unit, 0 on a building.
    fn target_is_unit_class(&mut self) -> bool;
    /// The `BUILD_AT` arm's pair of position records (`UnitData::get_order`
    /// twice, each's vslot `+0xb8`), against the target's position:
    /// `(target.x − first.+8, target.y − second.+4)`.
    fn pair_offsets(&mut self) -> (i32, i32);
    /// `max_range == 0` (the type record's `+0x1fc`) **and** one node on the
    /// path (`+0xc0 == 1`), the pair of reads the listing makes in order.
    fn melee_lone_node(&mut self) -> bool;
    /// `Unit::update_order`: the head's order is there.
    fn update_order(&mut self) -> bool;
    /// The `update_order` result's vslot `+0x2c`: a group order.
    fn head_is_group_order(&mut self) -> bool;
    /// The `update_order` result's vslot `+0x14`.
    fn head_vslot_14(&mut self) -> bool;
    /// The unit's `+0x80`, a signed short: its group.
    fn group(&mut self) -> i32;
    /// `Group::is_moving_to` of the unit's group.
    fn group_moving(&mut self) -> bool;
    /// The order record (vslot `+0x50`) byte `+0x1c`: `mandatory`.
    fn site_mandatory(&mut self) -> bool;
    /// The order record's byte `+0x1d`: `defensive`.
    fn site_defensive(&mut self) -> bool;
    /// The order record's `+0x14`, `+0x18`: the post.
    fn site_post(&mut self) -> (i32, i32);
    /// The order record's `+0x8`, `+0xc`: the order's own site.
    fn site(&mut self) -> (i32, i32);
    /// The order's flag byte `& 4`, through its virtual base.
    fn order_flag4(&mut self) -> bool;
    /// `ObjectData::is_map_unit` of `objects[whom][ox]`.
    fn map_unit(&mut self, ox: i32, whom: i32) -> bool;
    /// `Group::kill_next_order` of the unit's group.
    fn kill_next_order(&mut self);
    /// `Group::action_attack(ox, whom, 1, QUEUE_FIRST, 0)`.
    fn action_attack(&mut self, ox: i32, whom: i32);
    /// `Unit::find_attack_pos@00602e60(ox, whom, 1, &x, &y, 0)` from the
    /// unit's own position.
    fn find_attack_pos(&mut self) -> Option<(i32, i32)>;
    /// The unit's vslot `+0xf4`, `get_combat_stance`.
    fn stance(&mut self) -> i32;
    /// The unit's vslot `+0x130`, `max_range`.
    fn max_range_slot(&mut self) -> i32;
    /// `constants->unit_defensive_respond_range`.
    fn respond_range(&mut self) -> i32;
    /// The unit's position.
    fn unit_pos(&mut self) -> (i32, i32);
    /// `Unit::find_new_target(NULL, 0)`.
    fn find_new_target(&mut self);
    /// `Unit::add_move_order(x, y, 1, 0, QUEUE_FIRST, 0, ·, −1, −1)`.
    fn add_move_order(&mut self, x: i32, y: i32);
}

/// **`Unit::check_target_path@005e22d0`'s steps** (item 3304): the review of
/// a chase, a cast or a garrison walk against where its target now is.
///
/// 1. No target, or one the unit's player cannot see: 0.
/// 2. A **cast** (`0xe`): a target that is no longer a valid one ends the
///    walk **and the cast** (`repath`, `kill_current_order`); a valid
///    **unit** that is moving ends the walk only; anything else 0
///    (`docs/GOLDEN.md` §110, §112).
/// 3. Type 8 is 0, and type `0xd` on a building target is 0.
/// 4. **A unit target** is flanked-and-running (`target.heading −
///    bearing + 0x8000_0000` past a sixth of the circle, `flanking` nonzero,
///    and moving) or it is not: if not, in reach on the plain radius ends
///    the walk (`repath`, 1) and out of reach is 0 (`docs/COMBAT.md` §36);
///    if so, the review falls through.
/// 5. Off the map: `repath`, 1. A **garrison** (`0x1a`) beside its
///    building: `repath`, 1 (`docs/ORDERS.md` §29).
/// 6. In reach from where the unit is walking: 0.
/// 7. An order that is not attack-shaped is 0 — but for type `0xb` on a
///    unit-class target farther than `0x900` from the pair of positions its
///    two orders hold: `repath`, 1.
/// 8. A melee type on one node, or a building target: 0. Otherwise the
///    chase is re-aimed: a group head moving on its plan re-issues the
///    attack through the group (`docs/COMBAT.md` §67's SEAM), a group head
///    standing pops its orders down to the one that moves; a unit with no
///    group `repath`s and asks `find_attack_pos` — nothing found is 1 with
///    the legs gone, a **defensive** unit on a post it has strayed
///    `max(respond range, max range) × 0xc0` from is told to
///    `find_new_target`, and anything else is `add_move_order` to the spot
///    (`docs/COMBAT.md` §67).
///
/// The `ecx` the listing hands `is_in_range` and `add_move_order` as a
/// stale register is not a value (the sweep masks it).
pub fn check_target_path_steps<V: CheckTargetView>(v: &mut V) -> bool {
    if !v.target_set() || !v.target_seen() {
        return false;
    }
    let ty = v.action_type();
    if ty == 0xe {
        if !v.cast_target_valid() {
            v.repath();
            v.kill_current_order();
            return true;
        }
        if v.target_is_unit() && v.target_moving() {
            v.repath();
            return true;
        }
        return false;
    }
    if ty == 8 || (ty == 0xd && v.target_is_building()) {
        return false;
    }
    if v.target_is_unit() {
        let (dx, dy) = v.target_offset();
        let bearing = find_angle(dx, dy);
        let e = (v.target_heading() as u32)
            .wrapping_sub(bearing.0 as u32)
            .wrapping_add(0x8000_0000);
        // `5e2493`: the negative half of the window is tested inline and
        // jumps past the call, so `flanking` alone is not the predicate.
        let fleeing = e >= 0x2aaa_aaaa && combat::flanking(e) != 0 && v.target_moving();
        if !fleeing {
            if !v.in_range_plain() {
                return false;
            }
            v.repath();
            return true;
        }
    }
    if !v.target_on_map() {
        v.repath();
        return true;
    }
    if ty == 0x1a && v.adjacent() {
        v.repath();
        return true;
    }
    if v.in_range_from_orders() {
        return false;
    }
    if !v.order_is_attack() {
        if ty == 0xb && v.target_is_unit_class() {
            let (dx, dy) = v.pair_offsets();
            if vector_dist(dx.wrapping_abs(), dy.wrapping_abs()) > 0x900 {
                v.repath();
                return true;
            }
        }
        return false;
    }
    if v.melee_lone_node() || v.target_is_building() {
        return false;
    }
    v.update_order();
    if v.head_is_group_order() && v.group() >= 0 {
        if v.group_moving() {
            v.repath();
            if v.order_is_attack() && v.site_mandatory() {
                let (ox, whom) = v.site();
                if v.map_unit(ox, whom) {
                    v.kill_next_order();
                    v.action_attack(ox, whom);
                }
            }
        } else if v.update_order() {
            loop {
                v.update_order();
                if !v.head_vslot_14() {
                    break;
                }
                v.kill_current_order();
                if !v.update_order() {
                    break;
                }
            }
        }
        return true;
    }
    v.repath();
    let Some((x, y)) = v.find_attack_pos() else {
        return true;
    };
    if v.order_is_attack()
        && v.stance() == 1
        && !v.site_mandatory()
        && !v.order_flag4()
        && v.site_defensive()
    {
        let (px, py) = v.site_post();
        if px >= 0 && py >= 0 {
            let a = v.max_range_slot();
            let mut reach = v.respond_range();
            if reach.wrapping_mul(0xc0) < a.wrapping_mul(0xc0) {
                reach = v.max_range_slot();
            }
            let (ux, uy) = v.unit_pos();
            let dist = vector_dist((ux - px).wrapping_abs(), (uy - py).wrapping_abs());
            if reach.wrapping_mul(0xc0) <= dist {
                v.find_new_target();
                return true;
            }
        }
    }
    v.add_move_order(x, y);
    true
}

/// [`CheckTargetView`] over the live [`Sim`]: the action order `ty` names
/// (an `ATTACK` on the unit's combat target, a `CAST` on its spell's target
/// object, a `GARRISON` on its building).
///
/// The group arm (`5e25d1`–`5e26e9`) is reached by the leader of a group
/// move ([`Sim::check_target_path_review`], item 3939, `docs/GOLDEN.md`
/// §171): `is_moving_to`, `kill_next_order` and `action_attack` are the
/// group's own. SEAM: the `BUILD_AT` arm (type `0xb`) is not routed here;
/// `head_vslot_14` answers 0, so the standing-group pop
/// (`is_moving_to == 0`) kills nothing; its staging is parked 3939a. A **building** target answers `is_on_map` yes, as this
/// crate always has.
pub(super) struct SimCheckTarget<'a> {
    pub(super) s: &'a mut Sim,
    pub(super) u: usize,
    pub(super) ty: i32,
    pub(super) target: Option<Obj>,
    pub(super) spell: i32,
}

impl SimCheckTarget<'_> {
    pub(super) fn me(&self) -> Obj {
        Obj::Unit(self.u)
    }
    pub(super) fn t(&self) -> Obj {
        self.target.expect("target_set was asked first")
    }
    /// The action order's body, read when asked: after `repath` the head
    /// is the action itself, which is where the original's order object
    /// has been all along.
    pub(super) fn action(&self) -> Option<Order> {
        self.s
            .action_of(self.u)
            .map(|a| self.s.units[self.u].orders[a])
    }
}

impl CheckTargetView for SimCheckTarget<'_> {
    fn target_set(&mut self) -> bool {
        self.target.is_some()
    }
    fn target_seen(&mut self) -> bool {
        self.s.target_is_seen(self.me(), self.t())
    }
    fn action_type(&mut self) -> i32 {
        self.ty
    }
    fn cast_target_valid(&mut self) -> bool {
        let who = self.s.units[self.u].owner;
        let t = self.t();
        self.s.obj_alive(t) && self.s.spell_valid_target(self.spell, who, t)
    }
    fn target_is_unit(&mut self) -> bool {
        // `SubObjectData::is_active` (`flags & 1`) on a unit: the corpse
        // `Object::close` leaves in `objects[whom][ox]` answers 0.
        matches!(self.t(), Obj::Unit(tu) if self.s.units[tu].alive())
    }
    fn target_moving(&mut self) -> bool {
        matches!(self.t(), Obj::Unit(tu) if self.s.is_moving(tu))
    }
    fn repath(&mut self) {
        self.s.repath(self.u);
    }
    fn kill_current_order(&mut self) {
        self.s.kill_current_order(self.u);
    }
    fn target_is_building(&mut self) -> bool {
        matches!(self.t(), Obj::Building(_))
    }
    fn target_offset(&mut self) -> (i32, i32) {
        let (tp, mp) = (self.s.pos_of(self.t()), self.s.units[self.u].pos);
        (tp.x - mp.x, tp.y - mp.y)
    }
    fn target_heading(&mut self) -> i32 {
        match self.t() {
            Obj::Unit(tu) => self.s.units[tu].movement.heading.0,
            Obj::Building(_) => 0,
        }
    }
    fn in_range_plain(&mut self) -> bool {
        self.s.is_in_range(self.me(), self.t())
    }
    fn target_on_map(&mut self) -> bool {
        match self.t() {
            Obj::Unit(tu) => self.s.units[tu].on_map,
            Obj::Building(_) => true,
        }
    }
    fn adjacent(&mut self) -> bool {
        match self.t() {
            Obj::Building(b) => self.s.adjacent_to(self.u, b),
            Obj::Unit(_) => false,
        }
    }
    fn in_range_from_orders(&mut self) -> bool {
        let at = self.s.units[self.u].orders_pos;
        self.s.is_in_range_at(self.me(), at, self.t())
    }
    fn order_is_attack(&mut self) -> bool {
        self.ty == 10
    }
    fn target_is_unit_class(&mut self) -> bool {
        matches!(self.t(), Obj::Unit(_))
    }
    fn pair_offsets(&mut self) -> (i32, i32) {
        (0, 0)
    }
    fn melee_lone_node(&mut self) -> bool {
        self.s.profile(self.me()).max_range == 0 && self.s.units[self.u].path.len() == 1
    }
    fn update_order(&mut self) -> bool {
        !self.s.units[self.u].orders.is_empty()
    }
    fn head_is_group_order(&mut self) -> bool {
        matches!(self.s.current_order(self.u).map(|o| o.body), Some(Body::Move(m)) if m.group.is_some())
    }
    fn head_vslot_14(&mut self) -> bool {
        false
    }
    fn group(&mut self) -> i32 {
        if self.s.units[self.u].group_ptr.is_some() {
            0
        } else {
            -1
        }
    }
    fn group_moving(&mut self) -> bool {
        // `Group::is_moving_to@00713bd0` (item 3939, `docs/GOLDEN.md` §171):
        // `normalize`, then every member alive and on the map with an order
        // must head a `MOVE_TO` (1) or a `GROUP_MOVE` (`0x13`); a member with
        // none, or any other head, answers 0.
        let Some(g) = self.s.group_of(self.u) else {
            return false;
        };
        self.s.group_set_speed(&g);
        g.list.iter().all(|&m| {
            !self.s.units[m].alive()
                || !self.s.units[m].on_map
                || self.s.units[m].orders.is_empty()
                || matches!(self.s.order_type(m), index::MOVE_TO | index::GROUP_MOVE)
        })
    }
    fn site_mandatory(&mut self) -> bool {
        self.s.units[self.u].combat.mandatory
    }
    fn site_defensive(&mut self) -> bool {
        matches!(self.action(), Some(Order { body: Body::Attack(a), .. }) if a.defensive)
    }
    fn site_post(&mut self) -> (i32, i32) {
        match self.action() {
            Some(Order {
                body: Body::Attack(a),
                ..
            }) => a.def.map_or((-1, -1), |p| (p.x, p.y)),
            _ => (-1, -1),
        }
    }
    fn site(&mut self) -> (i32, i32) {
        // The action's record `+0x8`/`+0xc`: the attack's own target.
        self.target
            .map_or((-1, -1), |t| crate::attack_view::obj_key(self.s, t))
    }
    fn order_flag4(&mut self) -> bool {
        self.action().is_some_and(|o| o.flags & 4 != 0)
    }
    fn map_unit(&mut self, _ox: i32, _whom: i32) -> bool {
        // `ObjectData::is_map_unit@0046ffa0` of the record's object, which is
        // the target `site` named: vslot `+0x8` (a live unit) and `+0xbc`.
        matches!(self.target, Some(Obj::Unit(t)) if self.s.units[t].alive() && self.s.units[t].on_map)
    }
    fn kill_next_order(&mut self) {
        // `Group::kill_next_order@00713d90`: `normalize`, then
        // `kill_current_order(0)` on every member alive and on the map.
        let Some(g) = self.s.group_of(self.u) else {
            return;
        };
        group_member_walk(
            self.s,
            g.list.len(),
            |s| s.group_set_speed(&g),
            |_, k| g.list[k],
            |s, u| s.units[u].alive(),
            |s, u| s.units[u].on_map,
            |s, u| s.kill_current_order(u),
        );
    }
    fn action_attack(&mut self, _ox: i32, _whom: i32) {
        // `Group::action_attack(ox, whom, 1, QUEUE_FIRST, 0)` on the target
        // `site` named.
        let (Some(g), Some(t)) = (self.s.group_of(self.u), self.target) else {
            return;
        };
        self.s.group_action_attack(&g, t, true, QueuePos::First, 0);
    }
    fn find_attack_pos(&mut self) -> Option<(i32, i32)> {
        let here = self.s.units[self.u].pos;
        self.s
            .find_attack_pos(self.u, self.t(), here, crate::fight::SITE_ATTACK_POS_REVIEW)
            .map(|p| (p.x, p.y))
    }
    fn stance(&mut self) -> i32 {
        i32::from(self.s.units[self.u].combat.stance == combat::Stance::Defensive)
    }
    fn max_range_slot(&mut self) -> i32 {
        self.s.max_range_of(self.me())
    }
    fn respond_range(&mut self) -> i32 {
        self.s.tables.tuning.unit_defensive_respond_range
    }
    fn unit_pos(&mut self) -> (i32, i32) {
        let p = self.s.units[self.u].pos;
        (p.x, p.y)
    }
    fn find_new_target(&mut self) {
        self.s.find_new_target(self.u, false);
    }
    fn add_move_order(&mut self, x: i32, y: i32) {
        self.s.add_move_order(
            self.u,
            Pos::new(x, y),
            MoveKind::MoveTo,
            QueuePos::First,
            false,
        );
    }
}
