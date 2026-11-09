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
    t.cols.role |= role::ROLE_MILITARY;
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

/// **An Iroquois foot soldier is modern infantry in any age** (item
/// 3211): `is_modern_infantry@00607b40` asks `has_tribe_bonus(0x12)`
/// first and answers the type's `0x100` alone, the age unread.
/// run930's Slingers (TI 82, age 1) step 35 on a speed of 28 and take
/// the follower scatter on block 5564. Made to fail with the arm gone.
#[test]
fn an_iroquois_flagged_type_is_modern_infantry_in_any_age() {
    let mut s = sim();
    let t = ty(&mut s, 0, 0, 0);
    s.unit_types[t].cols.unit_flags |= 0x100;
    s.unit_types[t].combat.age = 1;
    let u = spawn(&mut s, t, Pos::new(1000, 1000));
    assert!(!s.is_modern_infantry(u), "no nation power, age 1");
    s.tech[1].power = Some(crate::nations::power::IROQUOIS);
    let bonus = s
        .tech_tree
        .has_tribe_bonus(&s.setup, &s.tech[1], crate::ai::tribe::IROQUOIS);
    assert_eq!(s.is_modern_infantry(u), bonus, "the Iroquois arm");
    assert!(bonus, "the bonus holds in this setup");
    s.unit_types[t].cols.unit_flags &= !0x100;
    assert!(!s.is_modern_infantry(u), "the flag is still asked");
    s.tech[1].power = None;
    s.unit_types[t].cols.unit_flags |= 0x100;
    s.unit_types[t].combat.age = 6;
    assert!(s.is_modern_infantry(u), "anyone's, past age 5");
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
        cat::FORM_CAT_ARTILLERY,
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

    assert_eq!(f.num_category[cat::FORM_CAT_ARTILLERY], 4);
    assert_eq!(f.x_spacing[cat::FORM_CAT_ARTILLERY], 660);
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
    assert_eq!(type_cat(&s.unit_types[foot], foot, false), cat::FORM_CAT_FOOT);
    assert_eq!(type_cat(&s.unit_types[ship], ship, false), cat::FORM_CAT_ARTILLERY);
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
    assert_eq!(f.category, [cat::FORM_CAT_FOOT, cat::FORM_CAT_FOOT, cat::FORM_CAT_ARTILLERY]);
    assert_eq!(f.num_category[cat::FORM_CAT_FOOT], 2);
    assert_eq!(f.num_category[cat::FORM_CAT_ARTILLERY], 1);
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

/// **The Mob (9) rides rings** (item 1330, `compute_dests`' arm at
/// `72ce81`): slot 0 on the anchor, the rest on a first ring of five
/// at radius `x_spacing`, from a third of a turn round and two fifths
/// of a turn apart. run514's three Citizens, `x_spacing` 144, sent to
/// (3840, 36864) on `-1532166144` unmirrored, stood their slots at
/// (3768, 36984) and (3960, 36792) — the tile centres below.
///
/// Made to fail with the arm placing every slot on the anchor, as it
/// did.
#[test]
fn the_mob_rides_rings_from_a_third_of_a_turn() {
    let mut s = sim();
    let t = ty(&mut s, mask::FOOT, 144, 144);
    let us: Vec<usize> = (0..3)
        .map(|i| spawn(&mut s, t, Pos::new(7032 + i * 200, 34344)))
        .collect();
    let dest = Pos::new(3840, 36864);
    let at = |reverse| {
        s.form_compute(
            &group(&us),
            dest,
            Angle(-1_532_166_144),
            formation::MOB,
            50,
            reverse,
            false,
            &[],
        )
    };
    let f = at(false);
    assert_eq!(f.to[0], dest, "slot 0 on the anchor");
    assert_eq!(crate::orders::snapped(f.to[1]), Pos::new(3768, 36984));
    assert_eq!(crate::orders::snapped(f.to[2]), Pos::new(3960, 36792));
    let m = at(true);
    assert_eq!(m.off[1].0, -f.off[1].0, "the mirror negates x");
    assert_eq!(m.off[1].1, f.off[1].1, "and keeps y");

    // The rings: five on the first, then ten from a half-step on.
    let mut ring = MobRing::new();
    for slot in 1..=5 {
        let _ = ring.place(slot, 144, false);
    }
    assert_eq!((ring.n, ring.count), (10, 0));
    assert_eq!(
        ring.angle,
        0x5555_5555u32
            .wrapping_add(0x6666_6666u32.wrapping_mul(5))
            .wrapping_add(u32::MAX / 20)
    );
    let (x, y) = ring.place(6, 144, false);
    let r2 = x * x + y * y;
    assert!(
        (287 * 287..=289 * 289).contains(&r2),
        "the second ring is twice as far out: ({x}, {y})"
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
    assert_eq!(f.num_category[cat::FORM_CAT_FOOT], 2, "two captains, six figures");
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
    assert_eq!(type_cat(&s.unit_types[civ], civ, false), cat::FORM_CAT_CIVILIAN);
    assert_eq!(type_cat(&s.unit_types[foot], foot, false), cat::FORM_CAT_FOOT);
    assert_eq!(type_cat(&s.unit_types[ship], ship, false), cat::FORM_CAT_ARTILLERY);
    assert_eq!(type_cat(&s.unit_types[mech], mech, false), cat::FORM_CAT_MECH);
    assert_eq!(type_cat(&s.unit_types[flak], flak, false), cat::FORM_CAT_ARTILLERY);
    assert_eq!(type_cat(&s.unit_types[horse], horse, false), cat::FORM_CAT_MOUNTED);
    // `max_range != 0` is the ranged discriminator for both pairs.
    s.unit_types[foot].combat.max_range = 6;
    s.unit_types[horse].combat.max_range = 6;
    assert_eq!(type_cat(&s.unit_types[foot], foot, false), cat::FORM_CAT_FOOT_RANGED);
    assert_eq!(
        type_cat(&s.unit_types[horse], horse, false),
        cat::FORM_CAT_MOUNTED,
        "the AI's horse archer forms up as plain cavalry"
    );
    assert_eq!(
        type_cat(&s.unit_types[horse], horse, true),
        cat::FORM_CAT_MOUNTED_RANGED,
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
        cat::FORM_CAT_COMMAND
    );
    s.unit_types[general].cols.unit_flags2 |= uflags2::FLAG2_GENERAL;
    assert_eq!(
        type_cat(&s.unit_types[general], general, false),
        cat::FORM_CAT_ARTILLERY
    );
}
