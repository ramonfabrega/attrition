//! `Form::compute`'s slot table — where in a formation each member stands
//! (`docs/GROUPS.md` §6.4).
//!
//! `Group::compute_form@00707c80` picks the formation angle, then hands the
//! group to `Form::compute@0072e8e0`, which is
//! `compute_rows_and_columns@0072d910` followed by
//! `compute_dests@0072cba0`. Between them they fill, for each member `i`, a
//! destination `to[i]`, an offset `off[i]` and a facing byte `angles[i]`;
//! `Group::update_positions@00713810` turns the offsets into the per-frame
//! `curr[i]` the leader's move adds to its own position.
//!
//! The `Form` objects are **both** the ten formation definitions
//! `Forms::init` loads from `rules.xml` **and** the scratch buffer the
//! computation writes into — `compute_form` takes `&forms.list[form]` and
//! `memset`s it from `+0x30` on. That is why [`Form::form`] and
//! [`Form::density`] are constants of the formation while everything else is
//! per-call.
//!
//! Every field name here is the PDB's own (`struct FormData`), and every
//! category name is the PDB's `FormCatIndex` record.
//!
//! **What is not reproduced**, each because of the original rather than the
//! port:
//!
//! - **Formation 6, Square, is dead code in the shipped executable.**
//!   `compute_rows_and_columns` has a whole `== 6` arm that fills
//!   `FormData::space[18][4]`, `across` and `per[7]` — and **nothing in the
//!   48k-function export reads any of the three**; `compute_dests` has no
//!   Square arm at all (`if (form != 6) { … }` with no `else`), and the arm
//!   that would have fired is disabled because the same branch sets
//!   `wedge = −1`. So a Square group's `to` and `off` are left at the
//!   `memset` zero. [`Sim::form_compute`] reproduces that, and the
//!   falsifying capture is named in `docs/GROUPS.md` §6.4.
//! - **A wedge's own row count is seeded from uninitialised stack.**
//!   `Form::compute` declares `int rows[18]` and never initialises it;
//!   `compute_rows_and_columns`' wedge arm reads `rows[wedge]` *before*
//!   writing it (`72dc90`, `movl (%ebx,%esi), %eax`). The placement inside
//!   the wedge does not depend on it — `compute_dests` re-accumulates from
//!   `FormData::total`, which *is* zeroed — but the rank stacking of every
//!   **other** category subtracts `x_spacing[wedge] · rows[wedge]`, so a
//!   wedge with a second category is not reproducible by anyone. This
//!   module seeds it 0.
//! - **The two type substitutions in `categorize`** — a loaded sea transport
//!   is sized by its cargo's type, and a land unit ordered onto **water** is
//!   sized as the leader's current Transport Barge (or Merchant Fleet for a
//!   caravan). The simulation keeps no cargo list and no water-move
//!   substitution, so each member is sized by its own type.
//! - **The follower arm.** `compute_dests` places a non-captain beside the
//!   last captain in the walk, alternating sides. `Group::add`'s own rule
//!   (§4.1) means a group holds captains unless it was built with
//!   `keep_captain != 0`, and nothing in this simulation does that.

use crate::group::Group;
use crate::movement::{Angle, cos_component, find_angle, sin_component};
use crate::orders::QueuePos;
use crate::world::{Pos, tile, vector_dist};
use crate::{Player, Sim, UnitType};

/// `NUM_FORM_CAT` — the count `FormData`'s three per-category arrays carry.
pub const NUM_FORM_CAT: usize = 18;

/// `FormCatIndex`, from the PDB's own `LF_ENUM` record.
///
/// `FormData::type_cat` returns only eight of the eighteen — 0, 2, 3, 4, 5,
/// 6, 8 and 10 — so the naval, sail and air categories are declared and
/// never produced, and four of the `_RANGED` variants are unreachable. The
/// ninth value that can occur is [`cat::COMMAND_RANGED`], and only as the
/// commander fold's fallback in a group of nothing but commanders.
pub mod cat {
    pub const MECH: usize = 0;
    pub const MECH_RANGED: usize = 1;
    pub const MOUNTED: usize = 2;
    pub const FOOT: usize = 3;
    pub const FOOT_RANGED: usize = 4;
    pub const MOUNTED_RANGED: usize = 5;
    pub const ARTILLERY: usize = 6;
    pub const ARTILLERY_RANGED: usize = 7;
    pub const COMMAND: usize = 8;
    pub const COMMAND_RANGED: usize = 9;
    pub const CIVILIAN: usize = 10;
    pub const CIVILIAN_RANGED: usize = 11;
    pub const SAIL: usize = 12;
    pub const SAIL_RANGED: usize = 13;
    pub const NAVAL: usize = 14;
    pub const NAVAL_RANGED: usize = 15;
    pub const AIR: usize = 16;
    pub const AIR_RANGED: usize = 17;
    /// `NUM_FORM_CAT` — the PDB's own end marker, and `find_leader`'s
    /// starting key (`docs/GROUPS.md` §4.4).
    pub const NUM: usize = 18;
}

/// The ten formations `rules.xml` lists, in document order — the index is
/// `FormData::form` and `Forms::init@0072e9a0` errors unless there are ten.
pub mod formation {
    pub const LINE: i32 = 0;
    pub const REFUSED: i32 = 1;
    pub const ENVELOP: i32 = 2;
    pub const ECHELON_RIGHT: i32 = 3;
    pub const ECHELON_LEFT: i32 = 4;
    pub const SPARSE: i32 = 5;
    /// Dead in the shipped executable — see the module docs.
    pub const SQUARE: i32 = 6;
    pub const WEDGE: i32 = 7;
    pub const COLUMN: i32 = 8;
    pub const MOB: i32 = 9;
}

/// `MERCHANT`, `MERCHANTDUTCH` and `FURTRAPPER` as record indices —
/// `TypeIndex − BASE_UNITTYPES (0x32)`. `FormData::type_cat` compares these
/// three ids outright, the same three `UnitType::init_final_flags` does
/// (`ai_load::Flags2Facts::trader_id`).
const TRADER_IDS: [usize; 3] = [0x3d - 0x32, 0x3e - 0x32, 0x190 - 0x32];

/// `Form::compute_dests@0072cba0`'s Mob arm (`72ce81`–`72cf3d`, read in
/// the listing: the decompiler drops `cosx`'s and `sinx`'s arguments).
///
/// Slot 0 stands on the anchor. Every later captain takes the next point
/// of a ring of `n` points, `n` 5 on the first ring and 5 more on each
/// ring out, at a radius of `x_spacing × n / 5`:
///
/// ```text
/// r = (w · n) / 5                      (the 0x66666667 divide, toward 0)
/// x = cosx(a, r), negated when reversed;   y = sinx(a, r)
/// count += 1
/// a += 2 · (0xffffffff / n)            n odd
///      0x4ccccccb                      n == 10
///      (n / 10 + 1) · (0xffffffff / n) otherwise   (unsigned divides)
/// count == n:  n += 5, count = 0, a += 0xffffffff / (2n)
/// ```
///
/// `cosx@0092d0c0` is `sinx@0092d100` a quarter turn on
/// ([`cos_component`], [`sin_component`]). The angle starts at
/// `0x55555555` and runs on across the whole walk; only this arm reads or
/// moves it.
struct MobRing {
    n: u32,
    count: u32,
    angle: u32,
}

impl MobRing {
    const fn new() -> Self {
        MobRing {
            n: 5,
            count: 0,
            angle: 0x5555_5555,
        }
    }

    fn place(&mut self, slot: i32, w: i32, reverse: bool) -> (i32, i32) {
        if slot == 0 {
            return (0, 0);
        }
        let r = w.wrapping_mul(self.n as i32) / 5;
        let a = Angle(self.angle as i32);
        let x = cos_component(a, r);
        let y = sin_component(a, r);
        self.count += 1;
        let step = if self.n & 1 != 0 {
            (u32::MAX / self.n).wrapping_mul(2)
        } else if self.n == 10 {
            0x4ccc_cccb
        } else {
            (self.n / 10 + 1).wrapping_mul(u32::MAX / self.n)
        };
        self.angle = self.angle.wrapping_add(step);
        if self.count == self.n {
            self.n += 5;
            self.count = 0;
            self.angle = self.angle.wrapping_add(u32::MAX / (2 * self.n));
        }
        (if reverse { -x } else { x }, y)
    }
}

/// `Form::init@0072dda0`'s last statement — `FormData::density`, a constant
/// of the formation: **2** for 0–4, **0** for Sparse, **1** for 6–9.
///
/// It reaches the arithmetic in only one place that can fire —
/// `compute_rows_and_columns`' wedge arm, whose formation has density 1, so
/// even there the `== 2` branch is dead. `compute_dests` reads it into the
/// `k` of the rank stack and then **overwrites it with 2** for any member
/// whose category is non-negative, which is every member; that is why `k`
/// is always 1 (`0072cba0:166`–`169`).
pub const fn density(form: i32) -> i32 {
    match form {
        0..=4 => 2,
        5 => 0,
        _ => 1,
    }
}

/// `FormData::type_cat@0072dfc0` — which of the eighteen categories a type
/// falls in.
///
/// `human` is `leaders[who].flags & 4`, and it gates exactly one arm: a
/// **mounted ranged** type is `MOUNTED_RANGED` for a human player and plain
/// `MOUNTED` for the AI, so the same army forms up differently under the two.
/// `index` is the type's record index, needed only for the three trader ids.
pub fn type_cat(t: &UnitType, index: usize, human: bool) -> usize {
    use crate::ai_load::uflags2;
    use crate::combat::mask;
    let p = &t.combat;
    // The gate into the main tree: an armed type that is neither supply nor
    // hero, is not a caravan, and is not one of the three traders.
    let armed = p.attack != 0
        && !t.cols.flag2(uflags2::SUPPLY_OR_HERO)
        && !t.cols.flag2(uflags2::CARAVAN)
        && !TRADER_IDS.contains(&index);
    if armed {
        if p.obj_masks & mask::CIVILIAN != 0 {
            return cat::CIVILIAN;
        }
        if p.obj_masks & mask::FOOT != 0 {
            // `+0x1fc max_range != 0` is the ranged discriminator, and it is
            // the *stored* one — `FLAGS k` has already zeroed a cavalry
            // archer's (`ai_load::RoleFacts::max_range`).
            return cat::FOOT + usize::from(p.max_range != 0);
        }
        if t.cols.flag2(uflags2::PACKS) || p.obj_masks & mask::ANTI_AIR != 0 {
            return cat::ARTILLERY;
        }
        if p.obj_masks & mask::MOUNTED != 0 {
            return if p.max_range != 0 && human {
                cat::MOUNTED_RANGED
            } else {
                cat::MOUNTED
            };
        }
        return if p.obj_masks & mask::VEHICLE != 0 {
            cat::MECH
        } else {
            cat::ARTILLERY
        };
    }
    // The tail: a general or a supply/hero lineage is artillery, and
    // everything else is a commander unless the `CIVILIAN` mask says
    // otherwise. `uflags2 & 0x60` is `ai_load::uflags2::SPECIAL_FORCES`.
    if t.cols.flag2(uflags2::SPECIAL_FORCES) {
        return cat::ARTILLERY;
    }
    if p.obj_masks & mask::CIVILIAN != 0 {
        cat::CIVILIAN
    } else {
        cat::COMMAND
    }
}

/// `FormData` — the ten formation definitions and the scratch buffer at
/// once (`docs/GROUPS.md` §6.4). Field names are the PDB's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Form {
    /// `+0x28`: which formation this is.
    pub form: i32,
    /// `+0x2c`: [`density`].
    pub density: i32,
    /// `+0x30`, `+0x34`: the leader — the member with the lowest category,
    /// and its index in the group's list. `Form::compute` measures the
    /// group's `o_angle`/`o_dist` from *its* slot.
    pub o: Option<usize>,
    pub idx: usize,
    /// `+0x38`.
    pub who: Player,
    /// `+0x3c`: how many members fell in each category.
    pub num_category: [i32; NUM_FORM_CAT],
    /// `+0x84`, `+0xcc`: the widest and deepest member of each category,
    /// in position units — the per-category maxima of the types' own
    /// `x_spacing`/`y_spacing`, widened for a multi-figure type.
    pub x_spacing: [i32; NUM_FORM_CAT],
    pub y_spacing: [i32; NUM_FORM_CAT],
    /// `+0x114`, `+0x314`: per member, its index within its category and
    /// the category itself. A member the walk skips keeps the `memset`
    /// zero in both.
    pub cat_id: Vec<i32>,
    pub category: Vec<usize>,
    /// `+0x514`, `+0x714`: per member, where it is being sent.
    pub to: Vec<Pos>,
    /// `+0x914`, `+0xb14`: per member, its offset from the anchor, in
    /// position units and **before** the divide by 48 that
    /// `GroupData::off_x` carries.
    pub off: Vec<(i32, i32)>,
    /// The group's `angles[i]` — a signed byte the move order adds to the
    /// formation angle as `angle + (i8 as i32) << 24`.
    pub angles: Vec<i8>,
    /// `+0xd14`: the wedge's priority category, −1 for every other
    /// formation.
    pub wedge: i32,
    /// `+0xd18`: the wedge's running triangular total.
    pub total: i32,
    /// `+0xd1c`: `compute_form`'s last argument — set by `action_guard`
    /// and 0 for every move.
    pub guarding: bool,
    /// `+0xe88`: the group's `facing`, which mirrors x inside the layout.
    /// Not the same flag as `compute_form`'s own reverse, which negates
    /// both offsets *after* the call.
    pub reverse: bool,
    /// **`compute_form`'s own reverse** — `bVar8` at `707e09`, the second
    /// of §6.3's two tests and the one that is not a mirror: when the
    /// caller supplied the angle and it is `>= 90°` from `find_angle(dest
    /// − group_loc)`, the tail at `707eb8` negates every member's
    /// `off_x/off_y` on **both** this record and the group's, and leaves
    /// [`Self::to`] alone.
    ///
    /// So a flipped layout marches to the destinations it was laid out
    /// with and *holds the opposite offsets* — which is what
    /// `Group::update_positions` rotates into `curr` every frame, and
    /// therefore where every follower of that formation stands relative to
    /// its leader.
    pub flipped: bool,
}

impl Form {
    /// `memset(&form->o, 0, 0xe60)` — `compute_form`'s own reset, before
    /// `categorize`. The formation's two constants survive it because they
    /// sit below `+0x30`.
    ///
    /// `angles` is seeded from the group's own, because `angles[]` lives on
    /// the **`GroupData`**, not on the `Form`, and the `memset` therefore
    /// does not reach it: Column and Mob never write it, so a group that
    /// takes either keeps the bytes its last Line move left.
    pub fn new(form: i32, who: Player, num: usize, angles: &[i8]) -> Form {
        let mut angles = angles.to_vec();
        angles.resize(num, 0);
        Form {
            form,
            density: density(form),
            o: None,
            idx: 0,
            who,
            num_category: [0; NUM_FORM_CAT],
            x_spacing: [0; NUM_FORM_CAT],
            y_spacing: [0; NUM_FORM_CAT],
            cat_id: vec![0; num],
            category: vec![0; num],
            to: vec![Pos::new(0, 0); num],
            off: vec![(0, 0); num],
            angles,
            wedge: -1,
            total: 0,
            guarding: false,
            reverse: false,
            flipped: false,
        }
    }

    /// `GroupData::off_x[i] = div_3_table[form.off_x[i] >> 4]` — the last
    /// thing `compute_dests` does per member, and the whole reason a
    /// formation offset in the record is a small number.
    ///
    /// `init_coord_lookup_array@00681db0` builds `div_3_table` as `j / 3`
    /// for `j >= 0` and `(j − 2) / 3` for `j < 0`, which is `floor(j / 3)`
    /// on both sides, and `>> 4` is arithmetic — so this is a **floor**
    /// divide by 48, not a truncation. run29's `[0, −14, 13, −28]` from one
    /// spacing of 660 could not have come out of a truncation.
    pub const fn quantise(v: i32) -> i32 {
        (v >> 4).div_euclid(3)
    }
}

/// What a follower hangs off in `Form::compute_dests`' walk: the last
/// captain passed (`local_1c`), its `cat_id` (`local_24`), and which side the
/// next follower takes (`bVar4`).
#[derive(Clone, Copy, Debug)]
struct Leading {
    cap: usize,
    slot: i32,
    right_side: bool,
}

/// `Form::compute_dests`' **modern-infantry scatter** (`72d454`–`72d50f`),
/// the `(x, y)` a follower for which `UnitData::is_modern_infantry` holds
/// adds to its step off its captain.
///
/// Both terms are three-valued, keyed on the formation's destination
/// (`dest`, the call's `Coord`s), the member's **object number** `o` (the
/// group's list entry), its **index** `i` in that list and the last
/// captain's `cat_id` (`slot`):
///
/// ```text
/// y += ((dest.x/113 · o · i + 7·slot) / 5 % 3) · 48 − 48
/// x += ((dest.y/97  · o · i + 13·slot) / 7 % 3) · gs / 4 − gs / 4
/// ```
///
/// Every division truncates (each is a magic multiply with the sign fixed,
/// `0x487ede05 >> 5`, `0x66666667 >> 1`, `0x151d07eb >> 3`, `0x92492493`
/// `+ n >> 2`, and the two `/ 4` the `>> 31 & 3` idiom), `% 3` is `idivl`'s
/// signed remainder, and the products are 32-bit `imul`s, so they wrap.
/// `gs` is the member's own type's `guy_spacing` (`+0x224`, reloaded at
/// `72d4b0`). `docs/GROUPS.md` §34.
pub(crate) fn form_scatter(dest: Pos, o: i16, i: usize, slot: i32, gs: i32) -> (i32, i32) {
    let o = i32::from(o);
    let i = i as i32;
    let along = (dest.x / 113)
        .wrapping_mul(o)
        .wrapping_mul(i)
        .wrapping_add(slot.wrapping_mul(7));
    let y = (along / 5 % 3) * 0x30 - 0x30;
    let across = (dest.y / 97)
        .wrapping_mul(o)
        .wrapping_mul(i)
        .wrapping_add(slot.wrapping_mul(13));
    let x = (across / 7 % 3).wrapping_mul(gs) / 4 - gs / 4;
    (x, y)
}

impl Sim {
    /// `Form::categorize@0072e250` — sort the members into categories and
    /// measure each category's spacing.
    ///
    /// Two passes: everything whose `type_cat` is not `COMMAND`, then the
    /// commanders, each folded into the first non-empty category **above**
    /// the biggest one. A group of nothing but commanders leaves the
    /// biggest at `COMMAND` and folds them all into `COMMAND_RANGED`, a
    /// category `type_cat` itself can never return.
    fn form_categorize(&self, f: &mut Form, g: &Group) {
        // `+0xd1c != 0` injects a phantom artillery member — this is what
        // `action_guard` reserves the escorted unit's rank with.
        if f.guarding {
            f.num_category[cat::ARTILLERY] += 1;
            f.x_spacing[cat::ARTILLERY] = 0xc0;
            f.y_spacing[cat::ARTILLERY] = 0x180;
        }
        let human = self.nation.get(f.who as usize).is_some_and(|n| n.human);
        let mut biggest = cat::COMMAND;
        let mut biggest_count = 0;
        let mut leader: Option<usize> = None;
        let mut leader_cat = NUM_FORM_CAT;
        for commanders in [false, true] {
            for (i, &u) in g.list.iter().enumerate() {
                if !self.form_member_counts(u) {
                    continue;
                }
                let Some(t) = self.units[u].ty else { continue };
                let c = type_cat(&self.unit_types[t], t, human);
                if (c == cat::COMMAND) != commanders {
                    continue;
                }
                let c = if commanders {
                    // The fold. Square keeps its commanders where they are;
                    // every other formation walks up from `biggest + 1` to
                    // the first non-empty category, and settles on
                    // `biggest + 1` itself when there is none.
                    if f.form == formation::SQUARE {
                        cat::COMMAND
                    } else {
                        let start = biggest + 1;
                        (start..NUM_FORM_CAT)
                            .find(|&c| f.num_category[c] != 0)
                            .unwrap_or(start)
                    }
                } else {
                    c
                };
                // `counts[18]` is `x_spacing[0]`: the original increments
                // out of bounds when the fold lands on 18, which needs a
                // member in `AIR_RANGED` and `type_cat` cannot put one
                // there. Bounded here rather than reproduced.
                if c >= NUM_FORM_CAT {
                    continue;
                }
                f.category[i] = c;
                f.cat_id[i] = f.num_category[c];
                f.num_category[c] += 1;

                let p = &self.unit_types[t].combat;
                let (w, d) = (p.x_spacing, p.y_spacing);
                if p.uber_size == 1 {
                    f.x_spacing[c] = f.x_spacing[c].max(w);
                    f.y_spacing[c] = f.y_spacing[c].max(d);
                } else {
                    // A multi-figure unit is as wide as its own rank of
                    // figures — three abreast, or two in a Column — and as
                    // deep as the ranks that takes.
                    let figures = if f.form == formation::COLUMN {
                        2
                    } else {
                        p.uber_size.min(3)
                    };
                    // The original divides by this. The shipped data never
                    // leaves `UBER_SIZE` under 1; a hand-built type can.
                    let figures = figures.max(1);
                    f.x_spacing[c] = f.x_spacing[c].max(figures * w);
                    let deep = ((p.uber_size - 1) / figures + 1) * d;
                    // Modern infantry stands a further `0x30` back — and
                    // only in the first pass; the commander fold does not
                    // ask.
                    let deep = if !commanders && self.is_modern_infantry(u) {
                        deep + 0x30
                    } else {
                        deep
                    };
                    f.y_spacing[c] = f.y_spacing[c].max(deep);
                }
                if biggest_count < f.num_category[c] {
                    biggest_count = f.num_category[c];
                    if !commanders {
                        // Only the first pass moves the argmax; the second
                        // raises the count without moving it.
                        biggest = c;
                    }
                }
                if leader.is_none() || c < leader_cat {
                    leader = Some(u);
                    leader_cat = c;
                    f.idx = i;
                }
            }
        }
        // No member survived the walk: the leader is the first in the list.
        if leader.is_none() {
            f.idx = 0;
            f.o = g.list.first().copied();
        } else {
            f.o = leader;
        }
    }

    /// The three tests `categorize` and `compute_dests` open every member
    /// with — active (`+0x8`), on the map (`+0xbc`), and a captain.
    fn form_member_counts(&self, u: usize) -> bool {
        self.form_member_active(u) && self.units[u].captain
    }

    /// The same two without the captain test — `compute_dests`' two tail
    /// loops, which slide **every** active on-map member including a
    /// follower.
    fn form_member_active(&self, u: usize) -> bool {
        self.units[u].alive() && self.units[u].on_map
    }

    /// `UnitData::is_modern_infantry@00607b40`: `unit_flags & 0x100` and a
    /// type age past 5 — or, for a player with tribe bonus `0x12`, the flag
    /// alone. The simulation has no tribe bonuses, so it is the flag and the
    /// age.
    pub(crate) fn is_modern_infantry(&self, u: usize) -> bool {
        self.units[u].ty.is_some_and(|t| {
            let ty = &self.unit_types[t];
            ty.cols.unit_flags & 0x100 != 0 && ty.combat.age > 5
        })
    }

    /// `Form::compute_rows_and_columns@0072d910` — how wide each category's
    /// block is, and how many ranks deep.
    ///
    /// Returns `(rows, cols)`. Column leaves `cols` at zero and every
    /// caller of it takes a branch that does not read `cols`; Square fills
    /// three fields nothing reads (the module docs).
    fn form_rows_and_columns(
        &self,
        f: &mut Form,
        width: i32,
    ) -> ([i32; NUM_FORM_CAT], [i32; NUM_FORM_CAT]) {
        let mut rows = [0i32; NUM_FORM_CAT];
        let mut cols = [0i32; NUM_FORM_CAT];
        if f.form == formation::SQUARE {
            f.wedge = -1;
            return (rows, cols);
        }
        if f.form == formation::WEDGE {
            // The priority category, by a fixed preference: foot, foot
            // ranged, mounted, mounted ranged, mech, mech ranged,
            // artillery, artillery ranged, then the first non-empty.
            const PREFERENCE: [usize; 8] = [
                cat::FOOT,
                cat::FOOT_RANGED,
                cat::MOUNTED,
                cat::MOUNTED_RANGED,
                cat::MECH,
                cat::MECH_RANGED,
                cat::ARTILLERY,
                cat::ARTILLERY_RANGED,
            ];
            let mut wedge = *PREFERENCE.last().expect("PREFERENCE is not empty");
            for &c in &PREFERENCE {
                if f.num_category[c] != 0 {
                    wedge = c;
                    break;
                }
            }
            if f.num_category[wedge] == 0 {
                wedge = (0..NUM_FORM_CAT)
                    .find(|&c| f.num_category[c] != 0)
                    .unwrap_or(wedge);
            }
            f.wedge = wedge as i32;
            for c in 0..NUM_FORM_CAT {
                let n = f.num_category[c];
                if c == wedge {
                    // Rows of 1, 2, 3, … until the members run out. The
                    // seed is the original's uninitialised stack slot; 0
                    // here (the module docs).
                    if n > 0 {
                        let (mut row, mut acc) = (0, 0);
                        while acc < n {
                            row += 1;
                            acc += row;
                        }
                        rows[c] = row;
                    }
                    continue;
                }
                // The density-2 halving cannot fire: Wedge's density is 1.
                cols[c] = (n / 2).max(10);
                if f.density == 2 {
                    let capped = cols[c].min(n);
                    cols[c] = ((capped + 1) / 2).max(1);
                }
                rows[c] = (n - 1 + cols[c]) / cols[c];
            }
            return (rows, cols);
        }
        // The general arm: how far the widest category reaches, halved once
        // it is six or more abreast, decides everyone's column count.
        let mut span = 0;
        for c in 0..NUM_FORM_CAT {
            let reach = f.x_spacing[c] * f.num_category[c];
            let reach = if f.num_category[c] < 6 {
                reach
            } else {
                reach / 2
            };
            span = span.max(reach);
        }
        for c in 0..NUM_FORM_CAT {
            let n = f.num_category[c];
            if f.form == formation::COLUMN {
                rows[c] = (n + 2) / 3;
                continue;
            }
            if n == 0 {
                cols[c] = 0;
                rows[c] = 0;
                continue;
            }
            // The original divides by `x_spacing[c]`, which the shipped
            // data never leaves at zero (the smallest `X_SPACING` is 8). A
            // hand-built type can, and then the whole category is one rank.
            let want = if f.x_spacing[c] == 0 {
                n
            } else {
                ((span * width) / 50) / f.x_spacing[c]
            };
            cols[c] = want.min(n).max(1);
            rows[c] = (n - 1 + cols[c]) / cols[c];
        }
        f.wedge = -1;
        (rows, cols)
    }

    /// `Form::compute_dests@0072cba0` — the slot each member stands in.
    fn form_dests(
        &self,
        f: &mut Form,
        g: &Group,
        dest: Pos,
        angle: Angle,
        rows: &[i32; NUM_FORM_CAT],
        cols: &[i32; NUM_FORM_CAT],
    ) {
        // `k = 2 − (density != 0)`, with `density` overwritten by 2 for any
        // member with a category — which is every member, so `k` is 1. It
        // is carried rather than folded away because the guarding tail
        // reads the same local after the walk.
        let k = 1;
        // The anchor: the first member of the **lowest-indexed** non-empty
        // category. Both loops that hunt for it write their `prev` only
        // when it is still negative (`72cfe2`/`72d81a`, and the `prev >= 0`
        // arm reloads the unwritten slot), so it sticks at the first
        // non-empty category and never advances to the last.
        let (mut anchor_x, mut anchor_y) = (0, 0);
        // `local_1c` and `bVar4`: the last captain the walk passed, and which
        // side the next follower takes. Both are set at the *top* of the
        // captain arm (`72cf4b`, `72cf58`), before any placement branch, and
        // both start at "member 0, right" so a follower ahead of every
        // captain hangs off slot 0.
        let mut last_cap = 0usize;
        let mut right_side = true;
        // `local_24`: the last captain's `cat_id`, which the modern-infantry
        // scatter reads (`72d469`). Set by the same captain arm, and 0 before
        // the first.
        let mut last_slot = 0;
        // The Mob's rings (`local_18`, `local_2c` and the angle at
        // `-0x2c(%ebp)`): five slots on the first, and five more on each
        // ring out; the angle starts a third of a turn round
        // (`72cbd4`–`72cbdb`) and is shared by the whole walk.
        let mut ring = MobRing::new();
        for (i, &u) in g.list.iter().enumerate() {
            if !self.form_member_active(u) {
                continue;
            }
            if !self.units[u].captain {
                let lead = Leading {
                    cap: last_cap,
                    slot: last_slot,
                    right_side,
                };
                self.form_follower_slot(f, u, i, lead, dest, angle);
                right_side = false;
                continue;
            }
            last_cap = i;
            right_side = true;
            let c = f.category[i];
            let slot = f.cat_id[i];
            last_slot = slot;
            let w = f.x_spacing[c];
            if f.wedge == c as i32 {
                let (x, y) = self.form_wedge_slot(f, c, slot, w, k);
                f.angles[i] = 0;
                f.off[i] = (if f.reverse { -x } else { x }, y);
                f.to[i] = Self::form_rotate(dest, angle, f.off[i]);
                continue;
            }
            if f.form == formation::SQUARE {
                // No placement arm at all — the module docs.
                continue;
            }
            let (x, mut y) = match f.form {
                formation::COLUMN => {
                    // Centre, right, left, repeating; one rank of three per
                    // three members.
                    let x = ((slot + 1) % 3 - 1) * w;
                    let y = -(slot / 3) * f.y_spacing[c];
                    (if f.reverse { -x } else { x }, y)
                }
                formation::MOB => ring.place(slot, w, f.reverse),
                _ => {
                    let ncols = cols[c].max(1);
                    let col = slot % ncols;
                    let row = slot / ncols;
                    // Alternate right and left of the centre line.
                    let step = (col + 1) >> 1;
                    let mut x = if col % 2 == 0 { step * w } else { -(step * w) };
                    // An even rank has no centre, so the whole block shifts
                    // half a step — unless this is `action_guard`.
                    if ncols % 2 == 0 && !f.guarding {
                        x += w / 2;
                    }
                    // And these categories stagger their odd rows.
                    if (c < 3 || c == cat::ARTILLERY || c > 11) && row % 2 != 0 {
                        x += w / 2;
                    }
                    let base = -k * row * f.y_spacing[c];
                    let y = match f.form {
                        formation::REFUSED => base - x.abs(),
                        formation::ENVELOP => base + x.abs(),
                        formation::ECHELON_LEFT if f.reverse => base - x,
                        formation::ECHELON_LEFT => base + x,
                        formation::ECHELON_RIGHT if f.reverse => base + x,
                        formation::ECHELON_RIGHT => base - x,
                        _ => base,
                    };
                    let mirrored = if f.reverse { -x } else { x };
                    f.angles[i] = Self::form_angle_byte(f.form, mirrored, y - base);
                    (mirrored, y)
                }
            };
            // The wedge's priority category is laid out in front of
            // everything else, so everyone else steps back behind it.
            if f.wedge >= 0 {
                let p = f.wedge as usize;
                y -= f.x_spacing[p] * rows[p] * k;
            }
            // The rank stack: each earlier non-empty category pushes this
            // one back by its own depth, and adjacent categories are parted
            // by half a rank.
            let mut prev: Option<usize> = None;
            for (c2, (&rank, &depth)) in rows.iter().zip(f.y_spacing.iter()).enumerate().take(c + 1)
            {
                if c2 as i32 == f.wedge || f.num_category[c2] == 0 {
                    continue;
                }
                if prev.is_some() {
                    y -= (k * depth) / 2;
                } else {
                    prev = Some(c2);
                }
                if c2 < c {
                    // `(rows − 0.5f) · depth · k`, truncated toward zero.
                    // The two seven-instruction float copies at `72d00a`
                    // and `72d846` are the only floats in the family, and
                    // this is integer-exact for anything a 128-member group
                    // can reach.
                    y -= ((2 * rank - 1) * depth * k) / 2;
                }
            }
            f.off[i] = (x, y);
            f.to[i] = Self::form_rotate(dest, angle, (x, y));
            if prev == Some(c) && slot == 0 {
                anchor_x = x;
                anchor_y = y;
                if f.guarding && c == cat::ARTILLERY {
                    anchor_x = 0;
                    anchor_y = 0;
                }
            }
        }
        // Slide the block so the anchor sits at the origin — and note that
        // `to` is only slid by the anchor's **y**: the x term is
        // `cosx(angle, 0)`, a literal zero pushed at `72d737`. So a group
        // whose anchor is off-centre marches to destinations that do not
        // match its own offsets, by exactly the rotated `anchor_x`.
        let slide = Self::form_rotate(Pos::new(0, 0), angle, (0, -anchor_y));
        for (i, &u) in g.list.iter().enumerate() {
            if !self.form_member_active(u) {
                continue;
            }
            f.to[i] = Pos::new(f.to[i].x + slide.x, f.to[i].y + slide.y);
            f.off[i] = (f.off[i].0 - anchor_x, f.off[i].1 - anchor_y);
        }
        // `action_guard`'s phantom rank, subtracted from every member's
        // depth after the fact.
        if f.guarding {
            let mut shift = 0;
            let mut prev: Option<usize> = None;
            for (c2, (&rank, &depth)) in rows.iter().zip(f.y_spacing.iter()).enumerate().take(7) {
                if c2 as i32 == f.wedge || f.num_category[c2] == 0 {
                    continue;
                }
                if prev.is_some() {
                    shift -= (k * depth) / 2;
                } else {
                    prev = Some(c2);
                }
                if c2 < cat::ARTILLERY {
                    shift -= ((2 * rank - 1) * depth * k) / 2;
                }
            }
            for (i, &u) in g.list.iter().enumerate() {
                if self.form_member_active(u) {
                    f.off[i].1 -= shift;
                }
            }
        }
    }

    /// `Form::compute_dests`' **follower arm** (`0072d3a0`–`0072d4f0`) — the
    /// branch a member takes when `is_captain` says no.
    ///
    /// A follower is never categorised (`Form::categorize` opens on a captain
    /// too), so it has no slot of its own: it hangs off the **last captain
    /// the walk passed**, one `guy_spacing` to the side, alternating, and its
    /// destination is that captain's destination plus the same step rotated.
    /// Both are *additions to the previous member's values*, so a run of
    /// followers walks outwards rather than all landing on one spot.
    ///
    /// The lean, `dy`, is the same formation-by-formation rule the captain
    /// arm applies to its own `x`, read off the **last captain's raw
    /// `off_x`** — which is why it is computed here rather than shared: the
    /// follower has no `x` of its own to lean on. For Line, and for every
    /// formation but the four that tilt, it is zero.
    ///
    /// A **modern infantry** follower is then scattered
    /// ([`form_scatter`], `72d454`–`72d50f`), after its facing byte
    /// and before its destination: `docs/GROUPS.md` §34.
    fn form_follower_slot(
        &self,
        f: &mut Form,
        u: usize,
        i: usize,
        lead: Leading,
        dest: Pos,
        angle: Angle,
    ) {
        let (last_cap, last_slot, right_side) = (lead.cap, lead.slot, lead.right_side);
        let gs = self.units[u]
            .ty
            .map_or(0, |t| self.unit_types[t].combat.guy_spacing);
        let base = if right_side { gs } else { -gs };
        let dx = if f.reverse { -base } else { base };
        // The last captain's own `x`, before the anchor slide.
        let lean = f.off[last_cap].0;
        let dy = match f.form {
            formation::REFUSED => match lean.cmp(&0) {
                std::cmp::Ordering::Greater => -dx,
                std::cmp::Ordering::Equal => -dx.abs(),
                std::cmp::Ordering::Less => dx,
            },
            formation::ENVELOP => match lean.cmp(&0) {
                std::cmp::Ordering::Less => -dx,
                std::cmp::Ordering::Equal => dx.abs(),
                std::cmp::Ordering::Greater => dx,
            },
            formation::ECHELON_RIGHT => -dx,
            formation::ECHELON_LEFT => dx,
            _ => 0,
        };
        // The same sign table as a captain's, minus the echelons' `x == 0`
        // arm: a follower's step is never zero unless the type's own
        // `guy_spacing` is.
        f.angles[i] = match (dx.cmp(&0), dy.cmp(&0)) {
            (std::cmp::Ordering::Less, std::cmp::Ordering::Less)
            | (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => -0x20,
            (std::cmp::Ordering::Less, std::cmp::Ordering::Greater)
            | (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => 0x20,
            _ => 0,
        };
        let (dx, dy) = if self.is_modern_infantry(u) {
            let (sx, sy) = form_scatter(dest, self.units[u].index, i, last_slot, gs);
            (dx + sx, dy + sy)
        } else {
            (dx, dy)
        };
        let step = Self::form_rotate(Pos::new(0, 0), angle, (dx, dy));
        f.to[i] = Pos::new(f.to[last_cap].x + step.x, f.to[last_cap].y + step.y);
        f.off[i] = (f.off[last_cap].0 + dx, f.off[last_cap].1 + dy);
    }

    /// The wedge's priority category: rows of 1, 2, 3, … around the centre.
    fn form_wedge_slot(&self, f: &mut Form, c: usize, slot: i32, w: i32, k: i32) -> (i32, i32) {
        f.total = 0;
        let mut row = 0;
        if slot > 0 {
            let mut acc = f.total;
            loop {
                row += 1;
                acc += row;
                if row + 1 + acc > slot {
                    break;
                }
            }
            f.total = acc;
        }
        let rem = slot - f.total;
        let mut x = if row % 2 == 0 {
            ((rem + 1) >> 1) * w
        } else {
            (rem >> 1) * w + w / 2
        };
        if rem % 2 != 0 {
            x = -x;
        }
        // The wedge steps back by its **width**, not its depth.
        let y = -k * w * row;
        let _ = c;
        (if f.reverse { -x } else { x }, y)
    }

    /// `group.angles[i]` — the eighth of a turn a member faces off the
    /// formation's own bearing, from the signs of its offset. Zero for
    /// every formation whose `y` does not lean on `x`, which is every one
    /// but Refused, Envelop and the two Echelons.
    fn form_angle_byte(form: i32, x: i32, lean: i32) -> i8 {
        const RIGHT: i8 = 0x20;
        const LEFT: i8 = -0x20; // 0xe0 as a signed byte
        match x.cmp(&0) {
            std::cmp::Ordering::Less => match lean.cmp(&0) {
                std::cmp::Ordering::Less => LEFT,
                std::cmp::Ordering::Greater => RIGHT,
                std::cmp::Ordering::Equal => 0,
            },
            std::cmp::Ordering::Equal => match form {
                formation::ECHELON_LEFT => LEFT,
                formation::ECHELON_RIGHT => RIGHT,
                _ => 0,
            },
            std::cmp::Ordering::Greater => match lean.cmp(&0) {
                std::cmp::Ordering::Less => RIGHT,
                std::cmp::Ordering::Greater => LEFT,
                std::cmp::Ordering::Equal => 0,
            },
        }
    }

    /// The one rotation the whole family uses: `[cos θ, sin θ; sin θ,
    /// −cos θ]`, a rotation composed with a y-flip. `compute_dests` applies
    /// it to a raw offset to reach a destination and
    /// `Group::update_positions` applies it to the quantised one to reach
    /// `curr`; a naive port mirrors, because the determinant is −1.
    fn form_rotate(origin: Pos, angle: Angle, (x, y): (i32, i32)) -> Pos {
        Pos::new(
            origin.x + cos_component(angle, x) + sin_component(angle, y),
            origin.y + sin_component(angle, x) - cos_component(angle, y),
        )
    }

    /// `Group::compute_form@00707c80` then `Form::compute@0072e8e0` — the
    /// whole table, for a group being sent to `dest` on `angle`.
    ///
    /// `reverse` is the group's `facing`, already toggled by `compute_form`
    /// if the leader is pointing more than 90° away from the formation's
    /// bearing; `guarding` is `action_guard`'s flag and false for a move.
    #[allow(clippy::too_many_arguments)]
    pub fn form_compute(
        &self,
        g: &Group,
        dest: Pos,
        angle: Angle,
        form: i32,
        width: i32,
        reverse: bool,
        guarding: bool,
        angles: &[i8],
    ) -> Form {
        let mut f = Form::new(form, g.who, g.list.len(), angles);
        f.guarding = guarding;
        f.reverse = reverse;
        self.form_categorize(&mut f, g);
        // `compute_form` seeds slot 0 with the order's own point, which is
        // what a leading follower would be placed against.
        if let Some(slot) = f.to.first_mut() {
            *slot = dest;
        }
        let (rows, cols) = self.form_rows_and_columns(&mut f, width);
        self.form_dests(&mut f, g, dest, angle, &rows, &cols);
        f
    }

    /// `GroupData::get_form_mod_option@0070bd00` — `Form::compute`'s
    /// `width`, and the only thing the general arm's column count scales
    /// with.
    ///
    /// **50 for an army**, and it stays 50: the mean is taken over the
    /// members' `unit +0xab` bytes, every one of which is −1 until a move
    /// writes it, and a move writes exactly this. A building group, an
    /// empty group and a group whose bytes are all −1 all take the same
    /// 0x32 fallback.
    pub fn group_form_mod_option(&self, g: &Group) -> i32 {
        let (mut sum, mut n) = (0, 0);
        for &u in &g.list {
            if !(self.units[u].alive() && self.units[u].on_map) || self.is_plane(u) {
                continue;
            }
            let w = i32::from(self.units[u].form_width);
            if w != -1 {
                sum += w;
                n += 1;
            }
        }
        if n > 0 { sum / n } else { 0x32 }
    }

    /// `Group::action_form(form, rotate, queue, ·, inserting)@00707220`
    /// (`docs/ORDERS.md` §30), reached from
    /// `CommandPackage::process_form@00949d90` with the command's three
    /// fields — a formation button's `form`, `rotate` 0 and `QUEUE_NEW`
    /// (`docs/GOLDEN.md` §22).
    ///
    /// **It makes no order of its own.** No `FormOrder` is built: the
    /// formation is each member's `form` byte and a group move laid out in
    /// it. For a unit group on the map with a leader that is no plane, and
    /// `QUEUE_NEW` or `QUEUE_FIRST`, the group's `form` goes to −1, the
    /// leader's action-bit orders are copied aside
    /// ([`Sim::group_set_up_insert`]), every member is halted, the call
    /// recurses — at `QUEUE_NEW` when nothing was copied, `QUEUE_FIRST`
    /// when something was — and the copies are replayed
    /// ([`Sim::group_finish_insert`]). The recursion writes `unit +0xaa =
    /// form` on every member that is no plane, figures and citizens alike,
    /// and then, at `QUEUE_NEW` or `QUEUE_LAST` only, moves the group to
    /// `get_loc_to` — where the leader will end up, which for a group that
    /// stood is where it stands — at `QUEUE_LAST`, `MOVE_TO`, **without
    /// the action bit**, `form −1` so `get_form` reads the byte just
    /// written. With `rotate` non-zero the move's angle is set: the group's
    /// `o_angle` when the point is the group's own `(ox, oy)`, else the
    /// leader's heading, plus `rotate`.
    ///
    /// So a group that stands re-forms on the spot, and a group that walks
    /// is halted and its move replayed to the same point in the new
    /// formation.
    ///
    /// SEAMS, none reached by a capture on file: the three negative
    /// formations — −1 and −3 step `get_form_option@0070beb0` forward and
    /// back through the five buttons, −2 is `Options::do_rotate`'s, which
    /// keeps the leader's own formation, sets the group's `form` to −2 and
    /// skips the insert dance — are not acted on here; the scenario's
    /// `ignore_orders` sweep (`action_begin`); a buildings group, which the
    /// command's `process_group` cannot build from units.
    pub fn group_action_form(&mut self, g: &Group, form: i32, rotate: i32, queue: QueuePos) {
        self.group_action_form_at(g, form, rotate, queue, false);
    }

    fn group_action_form_at(
        &mut self,
        g: &Group,
        form: i32,
        rotate: i32,
        queue: QueuePos,
        inserting: bool,
    ) {
        if !self.group_is_on_map(g) || g.list.is_empty() || form < 0 {
            return;
        }
        let Some(leader) = self.group_find_leader(g) else {
            return;
        };
        if self.is_plane(leader) {
            return;
        }
        if !inserting && matches!(queue, QueuePos::New | QueuePos::First) {
            if let Some(st) = self.gstate_mut(g) {
                st.form = -1;
            }
            let insert = self.group_set_up_insert(g);
            self.group_action_halt(g, 0);
            let queue = if insert.is_empty() {
                QueuePos::New
            } else {
                QueuePos::First
            };
            self.group_action_form_at(g, form, rotate, queue, true);
            self.group_finish_insert(g, insert);
            return;
        }
        for &u in &g.list {
            if !self.is_plane(u) {
                self.units[u].form = form as i8;
            }
        }
        if queue == QueuePos::First {
            return;
        }
        let Some(to) = self.group_loc_to(g) else {
            return;
        };
        let (set_angle, angle) = if rotate == 0 {
            (false, Angle(0))
        } else {
            let base = if to == self.group_o(g) {
                self.group_o_angle(g)
            } else {
                self.units[leader].movement.heading
            };
            (true, Angle(base.0.wrapping_add(rotate)))
        };
        self.group_action_move_to(
            g,
            to,
            QueuePos::Last,
            set_angle,
            angle,
            crate::orders::MoveKind::MoveTo,
            false,
        );
    }

    /// Whether the destination cell is water — `compute_form`'s own test,
    /// `(terrain & 0x30) == 0x20`, which is what turns `categorize`'s
    /// transport substitution on. Carried because the substitution is a
    /// declared seam and the flag is what a later reader has to reach for.
    pub fn form_dest_is_water(&self, dest: Pos) -> bool {
        self.world.tile_mask(dest.tile()) & tile::SURFACE == tile::SURFACE_OCEAN
    }

    /// `Form::compute`'s tail: the group's `o_angle` and `o_dist` are
    /// measured from the **leader's slot** to the order's own point — so
    /// `o_dist` is the leader's offset within the formation, not the
    /// distance it has to travel.
    pub fn form_leader_offset(f: &Form, dest: Pos) -> (Angle, i32) {
        let slot = f.to.get(f.idx).copied().unwrap_or(dest);
        let (dx, dy) = (dest.x - slot.x, dest.y - slot.y);
        (find_angle(dx, dy), vector_dist(dx, dy))
    }

    /// `Group::update_positions@00713810` — the per-frame formation
    /// position each member holds relative to the leader.
    ///
    /// The sine-table row is reached by `leal (%ecx,%ecx,2)` + `shll $4`,
    /// so the quantised offset is multiplied by **48** again, and the
    /// bound at `713a38` is `form_num`, not `num`. `theta` is the calling
    /// unit's own heading, replaced by the bearing to its current order's
    /// point when that order carries one.
    pub fn form_update_positions(off: &[(i32, i32)], theta: Angle) -> Vec<Pos> {
        off.iter()
            .map(|&(x, y)| Self::form_rotate(Pos::new(0, 0), theta, (x * 48, y * 48)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_load::role;
    use crate::combat::{self, mask};
    use crate::orders::{MoveKind, QueuePos};
    use crate::{Unit, UnitType};

    fn sim() -> Sim {
        let mut s = Sim::new(
            crate::tuning::Tuning::RON,
            crate::world::World::new(200, 200),
            2,
        );
        s.nation[0].human = true;
        s.nation[1].human = false;
        s
    }

    /// A type with a formation footprint. `masks` decides the category and
    /// `x`/`y` are already in position units — the `X_SPACING` column times
    /// `UNIT_FORMATION_SPACING`, as `UnitType::init` stores it.
    fn ty(s: &mut Sim, masks: u32, x: i32, y: i32) -> usize {
        let mut t = UnitType {
            hits: 100,
            combat: combat::Profile {
                attack: 220,
                uber_size: 1,
                obj_masks: masks,
                x_spacing: x,
                y_spacing: y,
                ..combat::Profile::default()
            },
            ..UnitType::default()
        };
        t.cols.role |= role::MILITARY;
        s.add_unit_type(t)
    }

    fn spawn(s: &mut Sim, ty: usize, p: Pos) -> usize {
        let index = i16::try_from(s.units.len()).unwrap();
        let hits = s.unit_types[ty].hits;
        let mut u = Unit::new(1, index, p, hits);
        u.ty = Some(ty);
        u.on_map = true;
        u.movement.speed = 25;
        s.add_unit(u)
    }

    fn group(list: &[usize]) -> Group {
        Group {
            who: 1,
            army: None,
            pushed: None,
            list: list.to_vec(),
        }
    }

    /// The quantised offsets of a whole group, which is what `GROUPDATA`
    /// carries.
    fn quantised(f: &Form) -> Vec<(i32, i32)> {
        f.off
            .iter()
            .map(|&(x, y)| (Form::quantise(x), Form::quantise(y)))
            .collect()
    }

    /// **The fixture.** run29's group `id 66` is the AI's navy: four light
    /// warships in formation 0, `form_num 4`, `form 0`, and the record's
    /// own `off_x = [0, −14, 13, −28]` with every `off_y` and every
    /// `angles` zero. `form_mod 50` is printed on each member's `UNITDATA`
    /// in the same window, so every input to the table is observed and only
    /// the arithmetic is ours.
    ///
    /// The chain, in one place: `X_SPACING 55 × 12 = 660`; `span = 4 × 660`
    /// because four is under six; `cols = ((2640 × 50)/50)/660 = 4`, so
    /// `rows = 1`; the slots alternate `0, −w, +w, −2w` about the centre;
    /// the even column count shifts the block by `w/2`; and the anchor
    /// slide takes that back out again. The floor divide by 48 is what
    /// turns `[330, −330, 990, −990]` into `[0, −14, 13, −28]` — a
    /// **truncation** gives `[0, −13, 13, −27]`, which is why the rounding
    /// had to be settled before this could be written.
    #[test]
    fn run29_s_navy_is_a_line_of_four_at_one_spacing_of_six_hundred_and_sixty() {
        let mut s = sim();
        // A Galley: `OBJ_MASK` `NRL`, so neither civilian, foot, mounted,
        // vehicle nor anti-air — which lands it in the artillery bucket.
        let t = ty(&mut s, mask::NAVAL | mask::ARCHERY | mask::LARGE, 660, 660);
        assert_eq!(
            type_cat(&s.unit_types[t], t, false),
            cat::ARTILLERY,
            "a warship falls off the end of type_cat's tree"
        );
        let us: Vec<usize> = (0..4)
            .map(|i| spawn(&mut s, t, Pos::new(0x8000 + i * 0x100, 0x8000)))
            .collect();
        let g = group(&us);
        let f = s.form_compute(
            &g,
            Pos::new(0x9000, 0x9000),
            Angle(0),
            formation::LINE,
            50,
            false,
            false,
            &[],
        );

        assert_eq!(f.num_category[cat::ARTILLERY], 4);
        assert_eq!(f.x_spacing[cat::ARTILLERY], 660);
        assert_eq!(f.cat_id, [0, 1, 2, 3], "one category, in list order");
        assert_eq!(
            f.off,
            [(0, 0), (-660, 0), (660, 0), (-1320, 0)],
            "the raw slots, after the anchor slide took the half-step back"
        );
        assert_eq!(
            quantised(&f),
            [(0, 0), (-14, 0), (13, 0), (-28, 0)],
            "run29's own row"
        );
        assert_eq!(f.angles, [0, 0, 0, 0], "a line has no per-slot facing");
    }

    /// The `curr` half of the same record: run29's four members carry
    /// `curr = [(0, 0), (473, 480), (−440, −446), (946, 960)]` under a
    /// leader heading of `−1_605_566_464`, and `update_positions` is what
    /// puts them there — `off × 48` through the rotation with the y-flip.
    #[test]
    fn run29_s_navy_curr_is_the_slot_table_under_the_leader_s_logged_heading() {
        let theta = Angle(-1_605_566_464);
        let off = [(0, 0), (-14, 0), (13, 0), (-28, 0)];
        assert_eq!(
            Sim::form_update_positions(&off, theta),
            [
                Pos::new(0, 0),
                Pos::new(473, 480),
                Pos::new(-440, -446),
                Pos::new(946, 960),
            ],
            "the record's own curr"
        );
        // A quarter turn off does not reproduce the record.
        assert_ne!(
            Sim::form_update_positions(&off, theta.quarter_turn()),
            Sim::form_update_positions(&off, theta)
        );

        // **The y-flip is not record-backed and this says so.** Every
        // `off_y` in run29's window is zero, so the `−cos θ · y` term never
        // fires there and no capture on disk can tell a rotation from a
        // rotation-with-a-flip. What pins it is the listing at
        // `713a38`: `curr_y = sin·x − cos·y`, a **subtraction**, giving the
        // matrix determinant −1. A depth offset is what would falsify it,
        // and the capture that would carry one is a formation with more
        // than one rank.
        let deep = [(0, -4)];
        assert_eq!(
            Sim::form_update_positions(&deep, Angle(0)),
            [Pos::new(0, 192)],
            "north is zero, so a member four quarter-tiles back sits +192 in y"
        );
    }

    /// The rank stack: a second category stands **behind** the first, by
    /// half its own depth plus `(rows − 0.5) × depth` for every category
    /// ahead of it — the two seven-instruction float copies, done in
    /// integers.
    ///
    /// Two foot (`x = y = 100`, one rank of two) and one warship
    /// (`x = y = 200`, alone). `span = 200`, so `cols[3] = 2` and
    /// `cols[6] = 1`; the foot rank is even and shifts by 50, which the
    /// anchor takes back; the warship is pushed back `((2·1 − 1)·100)/2 =
    /// 50` for the foot rank ahead of it and `200/2 = 100` for standing
    /// next to it.
    #[test]
    fn the_rank_stack_puts_a_later_category_behind_an_earlier_one() {
        let mut s = sim();
        let foot = ty(&mut s, mask::FOOT, 100, 100);
        let ship = ty(&mut s, mask::NAVAL, 200, 200);
        assert_eq!(type_cat(&s.unit_types[foot], foot, false), cat::FOOT);
        assert_eq!(type_cat(&s.unit_types[ship], ship, false), cat::ARTILLERY);
        let a = spawn(&mut s, foot, Pos::new(0x8000, 0x8000));
        let b = spawn(&mut s, foot, Pos::new(0x8100, 0x8000));
        let c = spawn(&mut s, ship, Pos::new(0x8200, 0x8000));
        let g = group(&[a, b, c]);
        let f = s.form_compute(
            &g,
            Pos::new(0x9000, 0x9000),
            Angle(0),
            formation::LINE,
            50,
            false,
            false,
            &[],
        );
        assert_eq!(f.category, [cat::FOOT, cat::FOOT, cat::ARTILLERY]);
        assert_eq!(f.num_category[cat::FOOT], 2);
        assert_eq!(f.num_category[cat::ARTILLERY], 1);
        assert_eq!(
            f.off,
            [(0, 0), (-100, 0), (-50, -150)],
            "the warship is a rank and a half back"
        );
        assert_eq!(quantised(&f), [(0, 0), (-3, 0), (-2, -4)]);
    }

    /// **The destinations drop the anchor's x and the offsets do not.**
    ///
    /// `compute_dests`' last loop slides `off` by the whole anchor but
    /// slides `to` by `cosx(angle, 0) + sinx(angle, −anchor_y)` — the x
    /// term is a literal zero, `xorl %edx, %edx` at `72d737`, and `cosx`
    /// returns 0 for a zero distance. So a group whose anchor is off-centre
    /// marches to destinations that are the rotated `anchor_x` away from
    /// where its own offsets say it will stand. An even column count is
    /// exactly what puts the anchor off-centre, so this fires on the
    /// commonest case there is.
    ///
    /// On the identity angle the discrepancy is visible directly: member 0
    /// has `off = (0, 0)` and yet `to != dest`.
    #[test]
    fn the_destinations_keep_the_anchor_s_x_that_the_offsets_lose() {
        let mut s = sim();
        let t = ty(&mut s, mask::NAVAL, 660, 660);
        let us: Vec<usize> = (0..4)
            .map(|i| spawn(&mut s, t, Pos::new(0x8000 + i * 0x100, 0x8000)))
            .collect();
        let dest = Pos::new(0x9000, 0x9000);
        let f = s.form_compute(
            &group(&us),
            dest,
            Angle(0),
            formation::LINE,
            50,
            false,
            false,
            &[],
        );
        assert_eq!(f.off[0], (0, 0), "member 0 is the anchor");
        assert_eq!(
            f.to[0],
            Pos::new(dest.x + 330, dest.y),
            "and yet it is sent half a spacing off the destination"
        );
        // Every member carries the same unslid x, so the block is coherent
        // — it is the block as a whole that is displaced.
        for (i, to) in f.to.iter().enumerate() {
            assert_eq!(to.x - dest.x - 330, f.off[i].0, "member {i}");
        }
    }

    /// Formation 6 is **dead code in the shipped executable**: the whole
    /// Square arm of `compute_rows_and_columns` writes `space[18][4]`,
    /// `across` and `per[7]`, and nothing in the 48k-function export reads
    /// any of the three; `compute_dests` has no Square arm and the same
    /// branch sets `wedge = −1` so the wedge arm cannot stand in. Every
    /// member's slot stays at the `memset` zero, and only member 0's `to`
    /// is anything at all, because `compute_form` seeds it with the order's
    /// own point before `Form::compute` runs.
    #[test]
    fn square_places_nobody_and_that_is_the_original_s_doing() {
        let mut s = sim();
        let t = ty(&mut s, mask::FOOT, 100, 100);
        let us: Vec<usize> = (0..4)
            .map(|i| spawn(&mut s, t, Pos::new(0x8000 + i * 0x100, 0x8000)))
            .collect();
        let dest = Pos::new(0x9000, 0x9000);
        let f = s.form_compute(
            &group(&us),
            dest,
            Angle(0),
            formation::SQUARE,
            50,
            false,
            false,
            &[],
        );
        assert_eq!(f.wedge, -1, "and so the wedge arm cannot fire either");
        assert_eq!(f.off, [(0, 0); 4], "no slot is ever written");
        assert_eq!(
            f.to,
            [dest, Pos::new(0, 0), Pos::new(0, 0), Pos::new(0, 0)],
            "only the seed survives"
        );
    }

    /// Column (8) is centre, right, left, repeating, three to a rank —
    /// and it leaves `cols[]` uninitialised, which is safe only because
    /// its own arm never reads it.
    #[test]
    fn a_column_is_centre_right_left_and_three_to_a_rank() {
        let mut s = sim();
        let t = ty(&mut s, mask::FOOT, 100, 100);
        let us: Vec<usize> = (0..5)
            .map(|i| spawn(&mut s, t, Pos::new(0x8000 + i * 0x100, 0x8000)))
            .collect();
        let f = s.form_compute(
            &group(&us),
            Pos::new(0x9000, 0x9000),
            Angle(0),
            formation::COLUMN,
            50,
            false,
            false,
            &[],
        );
        // Slots 0..4 give `(slot+1)%3 − 1` = 0, +1, −1, 0, +1 across and
        // `slot/3` = 0, 0, 0, 1, 1 back; the anchor is slot 0 at (0, 0).
        assert_eq!(f.off, [(0, 0), (100, 0), (-100, 0), (0, -100), (100, -100)]);
    }

    /// `get_form_mod_option` is 50 for an army and stays 50: the mean is
    /// over the members' `+0xab` bytes, which are −1 until a move writes
    /// them, and a move writes exactly the 50 the fallback produced.
    /// run29's `UNITDATA` prints `form_mod 50` on every member of the navy,
    /// which is the record saying the same thing.
    #[test]
    fn the_form_mod_option_is_fifty_and_a_move_keeps_it_there() {
        let mut s = sim();
        let t = ty(&mut s, mask::NAVAL, 660, 660);
        let us: Vec<usize> = (0..2)
            .map(|i| spawn(&mut s, t, Pos::new(0x8000 + i * 0x100, 0x8000)))
            .collect();
        let g = group(&us);
        assert_eq!(s.group_form_mod_option(&g), 50, "nobody has been formed up");
        s.group_action_move_to(
            &g,
            Pos::new(0x9000, 0x9000),
            QueuePos::New,
            true,
            Angle(0),
            MoveKind::MoveTo,
            false,
        );
        for &u in &us {
            assert_eq!(s.units[u].form, 0, "the move settled formation 0");
            assert_eq!(s.units[u].form_width, 50, "and wrote the width twin");
        }
        assert_eq!(s.group_form_mod_option(&g), 50, "the mean of four fifties");
    }

    /// The eight categories `type_cat` can actually return, one case each,
    /// and the one arm the player flag gates: a mounted ranged type is
    /// `MOUNTED_RANGED` for a human and plain `MOUNTED` for the AI, so the
    /// same army forms up differently under the two.
    /// **`compute_dests`' follower arm**, which only a human's selection
    /// group reaches: `Group::add` keeps a non-captain only when the group
    /// is built with `keep_captain`, and nothing an army does is.
    ///
    /// A follower has no slot of its own — `categorize` opens on a captain
    /// too — so it hangs off the last captain the walk passed, one
    /// `guy_spacing` to the side, **alternating**, each step added to the
    /// previous member rather than to the captain. run31's record is the
    /// fixture: twelve squads of three, `GUY_SPACING 12` through
    /// `UNIT_GUY_SPACING 12` giving 144, and every squad reading
    /// `(c, c + 3, c - 3)` once the floor divide by 48 has been applied.
    #[test]
    fn a_follower_hangs_off_the_last_captain_alternating_sides() {
        let mut s = sim();
        let t = ty(&mut s, mask::FOOT, 12 * 12, 12 * 12);
        s.unit_types[t].combat.uber_size = 3;
        s.unit_types[t].combat.guy_spacing = 12 * 12;
        // Two squads of three, laid out as the selection walks them.
        let mut list = Vec::new();
        for squad in 0..2 {
            for figure in 0..3 {
                let u = spawn(
                    &mut s,
                    t,
                    Pos::new(0x8000 + squad * 0x300, 0x8000 + figure * 0x60),
                );
                s.units[u].captain = figure == 0;
                list.push(u);
            }
        }
        let g = group(&list);
        let f = s.form_compute(
            &g,
            Pos::new(0x9000, 0x9000),
            Angle(0),
            formation::LINE,
            50,
            false,
            false,
            &[],
        );
        // Only the captains are categorised; the followers keep the memset
        // zero there and are placed by the arm under test.
        assert_eq!(f.num_category[cat::FOOT], 2, "two captains, six figures");
        let off = quantised(&f);
        for squad in 0..2usize {
            let (c, a, b) = (off[squad * 3], off[squad * 3 + 1], off[squad * 3 + 2]);
            assert_eq!(
                a,
                (c.0 + 3, c.1),
                "squad {squad}: the first follower is right"
            );
            assert_eq!(b, (c.0 - 3, c.1), "squad {squad}: and the second is left");
        }
        // And the destination takes the same step, rotated: on a zero angle
        // `[cos, sin; sin, -cos]` is `[1, 0; 0, -1]`, so a step of `+w` in x
        // is `+w` in the destination's x and nothing in its y.
        for squad in 0..2usize {
            let (c, a) = (f.to[squad * 3], f.to[squad * 3 + 1]);
            assert_eq!(a.x - c.x, 144, "squad {squad}: to follows off");
            assert_eq!(a.y - c.y, 0);
        }
    }

    /// **run404's squad** (`docs/GROUPS.md` §34, item 1113): the Infantry
    /// `1/6` (captain), `1/7` and `1/8` (its followers) laid out by the
    /// army's `ATTACKTO` of 764 — the point (38646, 13305), the angle
    /// −541917184, `facing 1`. A modern-infantry follower is scattered off
    /// its captain by the destination, its object number and its index, and
    /// run404's block 765 prints each member's `path[0].to` as theirs laid
    /// it: `1/7` a quarter of a spacing in, `1/8` a cell forward. Without the
    /// scatter both stand a whole `guy_spacing` either side, and ours
    /// walked `1/7` into the Battery on 794.
    #[test]
    fn a_modern_infantry_follower_is_scattered_off_its_captain() {
        let layout = |age: i32| {
            let mut s = sim();
            let t = ty(&mut s, mask::FOOT, 144, 144);
            s.unit_types[t].combat.uber_size = 3;
            s.unit_types[t].combat.guy_spacing = 144;
            s.unit_types[t].combat.age = age;
            s.unit_types[t].cols.unit_flags |= 0x100;
            let list: Vec<usize> = (0..3)
                .map(|k| {
                    let u = spawn(&mut s, t, Pos::new(20124 + k * 64, 16654));
                    s.units[u].index = 6 + k as i16;
                    s.units[u].captain = k == 0;
                    u
                })
                .collect();
            s.form_compute(
                &group(&list),
                Pos::new(38646, 13305),
                Angle(-541_917_184),
                formation::LINE,
                50,
                true,
                false,
                &[],
            )
        };
        let f = layout(6);
        assert!(f.to.len() == 3 && f.off.len() == 3);
        // Block 765's `path[0].to`, theirs, for `1/6`, `1/7` and `1/8`.
        assert_eq!(
            f.to,
            [
                Pos::new(38646, 13305),
                Pos::new(38569, 13382),
                Pos::new(38712, 13169)
            ],
            "the squad's destinations, as run404 prints them"
        );
        // Off the captain: `1/7` −144 + 36 across, `1/8` +144 across and a
        // cell (48) forward.
        assert_eq!(f.off[1], (f.off[0].0 - 108, f.off[0].1));
        assert_eq!(f.off[2], (f.off[0].0 + 144, f.off[0].1 + 48));
        // The Industrial Riflemen's age: not modern infantry, no scatter.
        let g = layout(5);
        assert_eq!(g.off[1], (g.off[0].0 - 144, g.off[0].1));
        assert_eq!(g.off[2], (g.off[0].0 + 144, g.off[0].1));
    }

    /// The scatter's arithmetic on its own, including a dividend the
    /// original's `idivl` leaves negative: the remainder keeps the sign, so
    /// `y` can reach −144.
    #[test]
    fn the_scatter_truncates_and_keeps_a_negative_remainder() {
        // run404's `1/7` and `1/8`.
        assert_eq!(form_scatter(Pos::new(38646, 13305), 7, 1, 0, 144), (36, 0));
        assert_eq!(form_scatter(Pos::new(38646, 13305), 8, 2, 0, 144), (0, 48));
        // `(342·o·i + 7·slot)/5 % 3` at its other two values: 0 and 2.
        assert_eq!(form_scatter(Pos::new(38646, 13305), 6, 0, 0, 144).1, -48);
        assert_eq!(form_scatter(Pos::new(38646, 13305), 6, 1, 0, 144).1, 48);
        // A product past `i32::MAX` wraps as `imul` does, and the remainder
        // of a negative quotient is negative.
        let (_, y) = form_scatter(Pos::new(100_000, 100_000), 30_000, 127, 0, 144);
        let along = (100_000i32 / 113).wrapping_mul(30_000).wrapping_mul(127);
        assert!(along < 0, "the fixture must wrap");
        assert_eq!(y, (along / 5 % 3) * 48 - 48);
        assert!(y <= -48, "a negative remainder steps back, as far as −144");
    }

    #[test]
    fn type_cat_returns_eight_of_its_eighteen_and_the_player_flag_moves_one() {
        use crate::ai_load::uflags2;
        let mut s = sim();
        let civ = ty(&mut s, mask::CIVILIAN, 100, 100);
        let foot = ty(&mut s, mask::FOOT, 100, 100);
        let ship = ty(&mut s, mask::NAVAL, 100, 100);
        let mech = ty(&mut s, mask::VEHICLE, 100, 100);
        let flak = ty(&mut s, mask::ANTI_AIR, 100, 100);
        let horse = ty(&mut s, mask::MOUNTED, 100, 100);
        assert_eq!(type_cat(&s.unit_types[civ], civ, false), cat::CIVILIAN);
        assert_eq!(type_cat(&s.unit_types[foot], foot, false), cat::FOOT);
        assert_eq!(type_cat(&s.unit_types[ship], ship, false), cat::ARTILLERY);
        assert_eq!(type_cat(&s.unit_types[mech], mech, false), cat::MECH);
        assert_eq!(type_cat(&s.unit_types[flak], flak, false), cat::ARTILLERY);
        assert_eq!(type_cat(&s.unit_types[horse], horse, false), cat::MOUNTED);
        // `max_range != 0` is the ranged discriminator for both pairs.
        s.unit_types[foot].combat.max_range = 6;
        s.unit_types[horse].combat.max_range = 6;
        assert_eq!(type_cat(&s.unit_types[foot], foot, false), cat::FOOT_RANGED);
        assert_eq!(
            type_cat(&s.unit_types[horse], horse, false),
            cat::MOUNTED,
            "the AI's horse archer forms up as plain cavalry"
        );
        assert_eq!(
            type_cat(&s.unit_types[horse], horse, true),
            cat::MOUNTED_RANGED,
            "and a human's does not"
        );
        // The tail: an unarmed type is a commander unless it is civilian,
        // and a general or a supply lineage is artillery whatever it is.
        let mut general = s.unit_types[foot].clone();
        general.combat.attack = 0;
        general.combat.obj_masks = 0;
        let general = s.add_unit_type(general);
        assert_eq!(
            type_cat(&s.unit_types[general], general, false),
            cat::COMMAND
        );
        s.unit_types[general].cols.unit_flags2 |= uflags2::GENERAL;
        assert_eq!(
            type_cat(&s.unit_types[general], general, false),
            cat::ARTILLERY
        );
    }
}
