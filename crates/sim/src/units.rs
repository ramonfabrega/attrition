//! `units/` — the files `docs/STRUCTURE.md` §3 maps here (item 4011).

pub mod add_to_world_view;
pub mod anim;
pub mod come_out_view;
pub mod count_inside_view;
pub mod disband;
pub mod eject;
pub mod garrison;
pub mod heal;
pub mod process;
pub mod remove_inside_view;
pub(crate) mod set_type_adapter;
pub mod slots;
pub mod stance;
pub mod starting;
pub mod store;
pub mod think;
pub mod unit;
pub mod unit_set_type_view;
pub mod unit_speed_view;
pub mod unit_update_hits_view;

/// The `units/` subsystem's fields of [`crate::Sim`] (`docs/STRUCTURE.md`
/// §3, item 4016): moved whole, each with its doc, in `Sim`'s order.
#[derive(Clone, Debug)]
pub struct UnitsState {
    /// One per player: whether `Leader::calc_unit_stats` is owed — the
    /// `LeaderData` flag `0x4000000`, which `Leader::process` acts on
    /// at its next pass, before any unit moves. Its writers modelled here:
    /// a change in the player's rare mask (`Sim::calc_unit_stats`),
    /// `Leader::gain_tech`'s tail, and a wonder's `Build::activate` and
    /// `Build::close` (`docs/VISION.md` §2).
    pub unit_stats_dirty: Vec<bool>,
    /// One per player: what they have built, and the population it occupies.
    pub muster: Vec<crate::Muster>,
    /// One per player: the object-number marks [`Sim::find_free`] allocates
    /// against.
    pub marks: Vec<crate::Marks>,
    /// One per player: `GameAccess::leader_options->list[who]`, the four
    /// per-player option words `Unit::init` reads to give a new unit its
    /// stance — `docs/ORDERS.md` §5.10.
    pub leader_options: Vec<crate::units::stance::LeaderOptions>,
    /// The animation art the clocks read — lengths and pieces, an input
    /// (`docs/ANIM.md` §3).
    pub art: crate::units::anim::Art,
    /// One per player: the specials list's in-use bits and mark
    /// (`Specials`, `LeaderData +0x43c`; `docs/SCOUT.md` §16).
    pub specials: Vec<crate::units::slots::SlotList>,
    /// One per player: the heroes list's (`Heroes`, `LeaderData +0x420`).
    pub heroes: Vec<crate::units::slots::SlotList>,
    /// The global hold-doober table's fades (`docs/ORDERS.md` §6.12).
    pub hold_doobers: crate::units::slots::HoldDoobers,
}
