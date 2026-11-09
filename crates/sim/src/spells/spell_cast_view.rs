//! **`SpellType::cast@00676ce0`** (item 3391), the dispatch behind every
//! cast order, as a pure walk over a [`SpellCastView`]: the cost gate
//! (`SpellTypeData::is_castable(p1, p2, 1)` must answer **3**) and the switch
//! on the spell type's index, one arm a craft.
//!
//! The crate has no single method of this name: the arms live in
//! `Sim::do_cast_effect` (untargeted) and `Sim::cast_on` (targeted), each
//! carrying only the crafts the diff reaches. This module is the whole
//! switch, kept **unwired** so the sweep can hold the table (which index
//! reaches which callee, with which `ecx` and which arguments) against the
//! original; the callees stay the crate's own.
//!
//! Several arms hand their callee an `ecx` the function never loads: it is
//! whatever `is_castable` left, which the view answers beside its result.

/// A callee of the switch, by its name in the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Callee {
    /// `cast_bribe@00672d00`
    Bribe,
    /// `cast_pilfer@00673e80`
    Pilfer,
    /// `cast_counterintel@00673740`
    CounterIntel,
    /// `cast_assimilate@00671820`
    Assimilate,
    /// `cast_create_decoy@00674370`
    CreateDecoy,
    /// `cast_ambush@00671610`
    Ambush,
    /// `cast_march@00671500`
    March,
    /// `cast_entrench@00672820`
    Entrench,
    /// `cast_supplies@006714c0`
    Supplies,
    /// `cast_double_agent@00673a80`
    DoubleAgent,
    /// `cast_sabotage@006720e0`
    Sabotage,
    /// `cast_blow_tread@00672cb0`
    BlowTread,
    /// `cast_paradrop@00671f80`
    Paradrop,
    /// `cast_research_project@00670990`
    ResearchProject,
    /// `cast_depopulate@00671e50`
    Depopulate,
    /// `cast_move_capital@00671760`
    MoveCapital,
    /// `cast_lock_gate@00671480`
    LockGate,
    /// `cast_unlock_gate@00671440`
    UnlockGate,
    /// `cast_transport@00670db0`
    Transport,
    /// `cast_pack@00670be0`
    Pack,
    /// `cast_unpack@006709c0`
    Unpack,
    /// `cast_civilian@006704a0`
    Civilian,
    /// `cast_to_arms@00670880`
    ToArms,
    /// `cast_immortal_arch@00670740`
    ImmortalArch,
    /// `cast_immortal_hinf@00670600`
    ImmortalHinf,
    /// `cast_disband@00671ed0`
    Disband,
}

/// The `ecx` a callee is entered with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ecx {
    /// `this`, the spell type's row.
    This,
    /// A value: an argument of the function (`counterintel`'s `p2`), or the
    /// `ecx` `is_castable` left, which several arms never reload and some
    /// push as an argument besides.
    Val(i32),
}

/// What the switch asks of the world.
pub trait SpellCastView {
    /// `[this + 4]`, the spell type's index.
    fn spell(&self) -> i32;
    /// `SpellTypeData::is_castable(this, p1, p2, 1)`: its answer, and the `ecx`
    /// it leaves behind.
    fn is_castable(&mut self, p1: i32, p2: i32) -> (i32, i32);
    /// `SpellTypeData::is_valid_target(this, p2, p3, p4, 0)`.
    fn is_valid_target(&mut self, p2: i32, p3: i32, p4: i32) -> bool;
    /// One direct callee, entered with `ecx` and the stack arguments in order.
    fn call(&mut self, callee: Callee, ecx: Ecx, args: &[i32]);
    /// The airdefense arm's first step: `objects[p2][p1]`'s virtual slot `+0xac`
    /// (no arguments), answering the object the upgrade is handed to.
    fn air_target(&mut self, p1: i32, p2: i32) -> i32;
    /// `LeaderData::current_upgrade(leaders[p2], ti)`.
    fn current_upgrade(&mut self, p2: i32, ti: i32) -> i32;
    /// The answered object's virtual slot `+0x80`, handed `(upgrade, 0)`.
    fn air_set(&mut self, target: i32, upgrade: i32, zero: i32);
}

/// `00676ce0`: the whole switch.
pub fn spell_cast_steps(v: &mut impl SpellCastView, p: [i32; 4]) {
    let [p1, p2, p3, p4] = p;
    let (r, left) = v.is_castable(p1, p2);
    if r != 3 {
        return;
    }
    use Callee::*;
    let (this, l) = (Ecx::This, Ecx::Val(left));
    match v.spell() {
        0x275 => v.call(Bribe, this, &[p1, p2, p3, p4]),
        0x276 => v.call(Pilfer, l, &[left, p2, p3, p4]),
        0x277 => {
            if v.is_valid_target(p2, p3, p4) {
                v.call(CounterIntel, Ecx::Val(p2), &[p3, p4]);
            }
        }
        0x278 => v.call(Assimilate, this, &[p1, p2, p3, p4]),
        0x27a => v.call(CreateDecoy, this, &[p1, p2, p3, left]),
        0x27b => v.call(Ambush, this, &[p1, p2]),
        0x27c => v.call(March, this, &[p1, p2]),
        0x27d => v.call(Entrench, l, &[p1, p2]),
        0x27e => v.call(Supplies, l, &[p1, p2]),
        0x27f => v.call(DoubleAgent, l, &[p1, p2, p3, p4]),
        0x280 | 0x281 => v.call(Sabotage, this, &[p1, p2, p3, p4]),
        0x282 => v.call(BlowTread, l, &[left, p2, p3, p4]),
        0x284 => v.call(Paradrop, this, &[p1, p2, p3, p4]),
        0x285 => v.call(ResearchProject, l, &[left, p2]),
        0x286 => v.call(Depopulate, this, &[p1, p2]),
        0x287 => v.call(MoveCapital, l, &[p1, p2, left]),
        0x288 => v.call(LockGate, l, &[p1, p2]),
        0x289 => v.call(UnlockGate, l, &[p1, p2]),
        0x28a => v.call(Transport, l, &[p1, p2]),
        0x28b | 0x28d | 0x28f | 0x291 => v.call(Pack, l, &[p1, p2]),
        0x28c | 0x28e | 0x290 | 0x292 => v.call(Unpack, l, &[p1, p2]),
        0x293 => v.call(Civilian, l, &[p1, p2]),
        0x294 => v.call(ToArms, l, &[p1, p2]),
        0x297 => v.call(ImmortalArch, l, &[p1, p2]),
        0x298 => v.call(ImmortalHinf, l, &[p1, p2]),
        AIR_DEFENSE => {
            let t = v.air_target(p1, p2);
            let up = v.current_upgrade(p2, AIRDEFENSE_TYPE);
            v.air_set(t, up, 0);
        }
        0x29a => v.call(Disband, l, &[p1, p2]),
        _ => {}
    }
}

/// The airdefense spell's index: the one arm that is no `cast_*` call but an
/// object's slot `+0x80` handed the leader's current upgrade of the
/// `AIRDEFENSE` type.
pub const AIR_DEFENSE: i32 = 0x299;
/// The `TypeIndex` `AIRDEFENSE` (`0x20b`, the argument the arm hands `current_upgrade`).
pub const AIRDEFENSE_TYPE: i32 = 0x20b;
